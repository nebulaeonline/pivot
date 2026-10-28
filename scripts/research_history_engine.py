"""Fixed-engine research store with staged index writes and explicit admission."""
import time

from research_history_compact import EMPTY,frame,sha
from research_history_hierarchy import PackedStore
from research_history_admission_model import Proposal,plan
from research_history_index import Unknown,Backpressure
from research_sqlite_native import NativeEngine


class EngineStore(PackedStore):
    def __init__(self,*args,scratch_limit=1024*1024,native_headroom=2*1024*1024,wal_trigger=64*1024*1024,**kwargs):
        if not 0<=scratch_limit<=1024*1024 or not 0<=native_headroom<=64*1024*1024 or not 0<wal_trigger<=64*1024*1024:
            raise ValueError('small-bound resource profile')
        if kwargs.get('max_splits',8)>8:raise ValueError('unqualified split profile')
        self.engine=NativeEngine();kwargs['connection_factory']=self.engine.connect
        self.scratch_limit=scratch_limit;self.native_headroom=native_headroom;self.wal_trigger=wal_trigger
        self.scratch=None;self.staged=None;self.writer_active=False;self.bootstrap=False;self.telemetry_sink=None
        super().__init__(*args,**kwargs)
        if self.db.execute('PRAGMA page_size').fetchone()[0]!=4096:raise ValueError('4 KiB research pages required')
        if self.db.execute('PRAGMA max_page_count=16384').fetchone()[0]!=16384:raise ValueError('research DB quota')
        self.db.execute('PRAGMA checkpoint_fullfsync='+('ON' if kwargs.get('fullfsync',False) else 'OFF'))
        self.db.observe();self.metrics.update(admission_ns=0,staged_index_bytes_peak=0,scratch_reserved_peak=0,planned_node_writes_peak=0)

    def release(self):self.scratch=None;self.staged=None;self.writer_active=False;self.bootstrap=False

    def execute(self,sql,params=()):
        result=super().execute(sql,params)
        if sql in ('COMMIT','ROLLBACK'):self.release()
        return result

    def operation(self,call):
        before=self.engine.telemetry(self.db);mutations=self.db.mutations;start=time.perf_counter_ns();outcome='error'
        try:
            result=super().operation(call);outcome='success';return result
        except Backpressure:
            if self.db.in_transaction:self.execute('ROLLBACK')
            self.verified=None;raise
        finally:
            if not self.db.in_transaction:self.release()
            if self.telemetry_sink:self.telemetry_sink({'status':outcome,'ns':time.perf_counter_ns()-start,
                'sql_mutation_statements':self.db.mutations-mutations,'before':before,'after':self.engine.telemetry(self.db)})

    def reserve(self):
        memory=self.engine.telemetry()
        if memory['hard_heap_limit']<=0 or memory['memory_used']+self.native_headroom>memory['hard_heap_limit']:
            raise Backpressure('native SQLite headroom unavailable before mutation')
        self.scratch=bytearray(self.scratch_limit)
        self.metrics['scratch_reserved_peak']=max(self.metrics['scratch_reserved_peak'],len(self.scratch))

    def begin(self):
        if self.corrupt_handle or self.verified is None:raise Unknown('mutation authority blocked')
        if self.wal_bytes()>=self.wal_trigger:
            self.execute('PRAGMA wal_checkpoint(TRUNCATE)').fetchall()
            if self.wal_bytes()>=self.wal_trigger:raise Backpressure('test reader pins WAL; no new mutation admitted')
        super().begin();self.writer_active=True

    def create(self,seed):
        self.reserve();self.bootstrap=True
        try:
            self.admit(self.txid('seed'))
            return super().create(seed)
        finally:
            if not self.db.in_transaction:self.release()

    def find(self,txid):
        row=super().find(txid)
        if row is None and self.writer_active:self.admit(txid)
        return row

    def admit(self,txid):
        if self.staged is not None:
            if self.staged['txid']!=txid:raise Unknown('admission/transaction disagreement')
            return
        start=time.perf_counter_ns()
        if self.scratch is None:self.reserve()
        key=self.key(txid)
        if self.bootstrap:
            path=[];node=1;entries=[];decision={'splits':0,'new_leaves':1,'node_writes':1,'logical_node_bytes':65}
        else:
            path,node,entries=self.proof_path(key)
            decision=plan(node,[e[:32] for e in entries],key,self.leaves,Proposal(*self.bounds))
        nodes=[];offset=0;patch=None

        def stage(address,raw,dependencies):
            nonlocal offset
            if offset+len(raw)>len(self.scratch):raise Backpressure('index scratch exhausted before SQL mutation')
            self.scratch[offset:offset+len(raw)]=raw;nodes.append((address,offset,len(raw),dependencies));at=offset;offset+=len(raw);return at

        def build(address,items):
            nonlocal patch
            if len(items)<=self.bounds[0]:
                raw=b'L'+b''.join(items);at=stage(address,raw,[])
                for index,item in enumerate(items):
                    if item[:32]==key:patch=at+1+64*index+32
                return
            depth=address.bit_length()-1;children=[[],[]]
            for item in items:children[(int.from_bytes(item[:32],'big')>>(255-depth))&1].append(item)
            build(address*2,children[0]);build(address*2+1,children[1])
            stage(address,b'I'+bytes(64),[(1,address*2),(33,address*2+1)])

        build(node,sorted(entries+[key+EMPTY]))
        child=node
        for address,page,bit in reversed(path):
            stage(address,b'I'+page[1]+page[2],[(1+32*bit,child)]);child=address
        if patch is None or len(nodes)!=decision['node_writes'] or offset!=decision['logical_node_bytes']:
            raise AssertionError('staged index/independent admission model disagreement')
        self.staged={'txid':txid,'nodes':nodes,'patch':patch,'decision':decision}
        self.metrics['staged_index_bytes_peak']=max(self.metrics['staged_index_bytes_peak'],offset)
        self.metrics['planned_node_writes_peak']=max(self.metrics['planned_node_writes_peak'],len(nodes))
        self.metrics['admission_ns']+=time.perf_counter_ns()-start

    def put(self,blob):
        if self.scratch is None or (not self.bootstrap and self.staged is None):raise Unknown('payload mutation before admission')
        return super().put(blob)

    def commit_row(self,txid,*args,**kwargs):
        self.admit(txid)
        return super().commit_row(txid,*args,**kwargs)

    def add_proof(self,txid,receipt):
        if self.staged is None or self.staged['txid']!=txid:raise Unknown('missing pre-write index plan')
        start=time.perf_counter_ns();sql=self.metrics['sql_ns'];staged=self.staged;decision=staged['decision']
        at=staged['patch'];self.scratch[at:at+32]=receipt;digests={}
        if self.split_fault and decision['splits']:self.split_fault('before_split')
        for address,offset,length,dependencies in staged['nodes']:
            for relative,child in dependencies:self.scratch[offset+relative:offset+relative+32]=digests[child]
            raw=bytes(memoryview(self.scratch)[offset:offset+length]);digests[address]=self.write_page(address,raw)
            if self.split_fault:self.split_fault('after_planned_node')
        if self.split_fault and decision['splits']:self.split_fault('after_children')
        self.root=digests[1];self.leaves=decision['new_leaves'];self.generation+=1
        self.meta[0]=self.format_id;self.meta[8]=frame([b'HD1',*self.bounds,self.generation,self.leaves,self.root])
        self.metrics['proof_written_bytes']+=len(self.meta[8]);self.metrics['split_count']+=decision['splits']
        self.metrics['publication_pages_peak']=max(self.metrics['publication_pages_peak'],len(staged['nodes']))
        self.metrics['proof_ns']+=time.perf_counter_ns()-start-(self.metrics['sql_ns']-sql)
