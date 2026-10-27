"""G3 bounded-open research: authenticated indexes within the SAME SQLite transaction.

No compatibility reader, production format or alternative storage backend.
"""
import json
from pathlib import Path
import sqlite3
import time

from research_history_index import Store, SCHEMA, Unknown, Backpressure, encoded, digest, checkpoint_bytes, ledger

EXTRA_SCHEMA = '''
CREATE TABLE proof_node(hash TEXT PRIMARY KEY, bytes BLOB NOT NULL) WITHOUT ROWID;
CREATE TABLE publication(singleton INTEGER PRIMARY KEY CHECK(singleton=1), body BLOB NOT NULL, seal TEXT NOT NULL);
'''
MAX_WAL = 128 * 1024 * 1024


def row_value(row):
    return [{"blob":value.hex()} if isinstance(value,bytes) else value for value in row]


def expected_schema():
    db=sqlite3.connect(':memory:')
    try:
        db.executescript(SCHEMA+EXTRA_SCHEMA)
        return set(db.execute('SELECT type,name,tbl_name,sql FROM sqlite_schema'))
    finally:
        db.close()


def bits(key):
    return ''.join(format(b,'08b') for b in bytes.fromhex(key))


class VerifiedStore(Store):
    def __init__(self,path,expected_instance,*,fullfsync=False,**kwargs):
        self.expected_instance=expected_instance
        self.root=None
        self.integrity_failed=False
        self.proof_reads=0
        self.payload_bytes=0
        self.statement_count=0
        self.vm_steps_1000=0
        self.recovery_bytes=self.preflight(Path(path))
        started=time.perf_counter_ns()
        super().__init__(path,**kwargs)
        self.connect_ns=time.perf_counter_ns()-started
        self.db.execute('PRAGMA fullfsync='+str(int(fullfsync)))
        self.schema=expected_schema()
        self.db.set_trace_callback(self._trace)
        self.db.set_progress_handler(self._progress,1000)

    def corrupt(self,message):
        self.verified=None
        self.integrity_failed=True
        raise Unknown(message)

    def operation(self,call,*args,**kwargs):
        try:
            return call(*args,**kwargs)
        except Backpressure:
            raise  # Known refusal before publication; no ambiguous decision.
        except Exception:
            self.verified=None
            raise  # Keep the original error; never invent a negative/success.

    def publish(self,*args,**kwargs):
        return self.operation(super().publish,*args,**kwargs)

    def reconcile(self,*args,**kwargs):
        return self.operation(super().reconcile,*args,**kwargs)

    def move(self,*args,**kwargs):
        return self.operation(super().move,*args,**kwargs)

    def _trace(self,_sql):
        self.statement_count+=1

    def _progress(self):
        self.vm_steps_1000+=1
        return 0

    @staticmethod
    def preflight(path):
        # Stat BEFORE SQLite can rebuild its shared-memory WAL index.
        wal=Path(str(path)+'-wal')
        size=wal.stat().st_size if wal.exists() else 0
        if size>MAX_WAL:
            raise Unknown('WAL recovery budget exceeded; explicit maintenance required')
        if size:
            with wal.open('rb') as stream:
                header=stream.read(32)
            if len(header)!=32 or int.from_bytes(header[:4],'big') not in (0x377f0682,0x377f0683):
                raise Unknown('incomplete or invalid WAL header')
            page=int.from_bytes(header[8:12],'big')
            if page==1:page=65536
            if page<512 or page>65536 or page & (page-1) or (size-32)%(page+24):
                raise Unknown('incomplete WAL frame/tail')
        journal=Path(str(path)+'-journal')
        if journal.exists() and journal.stat().st_size:
            raise Unknown('unexpected rollback recovery journal')
        return size

    def node(self,key):
        self.proof_reads+=1
        row=self.db.execute('SELECT bytes FROM proof_node WHERE hash=?',(key,)).fetchone()
        if row is None or digest(row[0])!=key:
            self.corrupt('missing or corrupt authenticated index node')
        value=json.loads(row[0])
        if not isinstance(value,list) or len(value) not in (3,4):
            self.corrupt('invalid authenticated index shape')
        if value[0]=='leaf' and len(value)==3:
            if len(bytes.fromhex(value[1]))!=32 or len(bytes.fromhex(value[2]))!=32:
                self.corrupt('invalid authenticated leaf')
        elif value[0]=='branch' and len(value)==4:
            if len(value[1])>=256 or set(value[1])-{'0','1'} or value[2]==value[3]:
                self.corrupt('invalid authenticated branch')
        else:
            self.corrupt('invalid authenticated node kind')
        return value

    def write_node(self,value):
        raw=encoded(value);key=digest(raw)
        self.db.execute('INSERT OR IGNORE INTO proof_node VALUES (?,?)',(key,raw))
        if self.node(key)!=value:
            self.corrupt('authenticated node collision')
        return key

    def map_key(self,namespace,key):
        return digest(encoded([self.expected_instance,0,namespace,str(key)]))

    def lookup(self,namespace,key):
        wanted=self.map_key(namespace,key);path=bits(wanted)
        root=self.root;required=''
        for _ in range(257):
            if root is None:return None
            node=self.node(root)
            prefix=bits(node[1]) if node[0]=='leaf' else node[1]
            if not prefix.startswith(required):
                self.corrupt('authenticated path prefix disagreement')
            if node[0]=='leaf':
                return node[2] if node[1]==wanted else None
            if not path.startswith(prefix):return None
            bit=path[len(prefix)]
            root=node[2+int(bit)];required=prefix+bit
        self.corrupt('authenticated path bound exceeded')

    def insert(self,namespace,key,value):
        hashed=self.map_key(namespace,key);path=bits(hashed)
        leaf=['leaf',hashed,digest(encoded(value))]
        def build(root,required=''):
            if root is None:return self.write_node(leaf)
            node=self.node(root)
            prefix=bits(node[1]) if node[0]=='leaf' else node[1]
            if not prefix.startswith(required):self.corrupt('index insertion path disagreement')
            common=0
            while common<min(len(prefix),256) and prefix[common]==path[common]:common+=1
            if common<len(prefix):
                children=[root,self.write_node(leaf)] if path[common]=='1' else [self.write_node(leaf),root]
                return self.write_node(['branch',path[:common],*children])
            if node[0]=='leaf':
                if node!=leaf:self.corrupt('immutable authenticated key already bound')
                return root
            branch=int(path[len(prefix)])
            node[2+branch]=build(node[2+branch],prefix+str(branch))
            return self.write_node(node)
        self.root=build(self.root)

    def bind_row(self,namespace,key,row):
        expected=self.lookup(namespace,key)
        if expected is None:
            if row is not None:self.corrupt('SQL row missing from authenticated index')
        elif row is None or expected!=digest(encoded(row_value(row))):
            self.corrupt('authenticated index/SQL row disagreement')
        return row

    def state_row(self,key):
        row=self.db.execute('SELECT * FROM state WHERE id=?',(key,)).fetchone()
        return self.bind_row('state',key,row)

    def receipt(self,tx,intent):
        row=self.db.execute('SELECT * FROM decision WHERE tx=?',(tx,)).fetchone()
        self.bind_row('receipt',tx,row)
        if row is None:return None
        seq,_,old_intent,kind,parent,body,root=row
        if root!=digest(encoded([seq,tx,old_intent,kind,parent,digest(body)])):
            self.corrupt('receipt checksum mismatch')
        if old_intent!=intent:raise Unknown('transaction identity reused with different intent')
        return {'outcome':kind,'receipt':root,'seq':seq}

    def get(self,key):
        try:
            value=super().get(key)
        except Unknown as error:
            self.corrupt(str(error))
        self.payload_bytes+=len(encoded(value))
        return value

    def cold(self,state):
        return self.operation(self._cold,state)

    def _cold(self,state):
        row=self.state_row(state)
        if row is None:self.corrupt('missing state')
        base=row[3]
        if not 0<=state-base<64:self.corrupt('cold decode bound exceeded')
        checkpoint=self.state_row(base)
        if checkpoint is None or checkpoint[3]!=base:self.corrupt('missing logical checkpoint')
        records=self.get(checkpoint[4])
        for index in range(base+1,state+1):
            delta=self.state_row(index)
            if delta is None or delta[2]!=index-1:self.corrupt('broken content chain')
            from research_history_index import apply_ops
            records=apply_ops(records,self.get(delta[4]))
        if digest(checkpoint_bytes(records))!=row[5] or self.get(row[6])!=ledger(records):
            self.corrupt('cold oracle/lifetime mismatch')
        return records

    def seal(self):
        authority=self.authority()
        decision=self.db.execute('SELECT * FROM decision WHERE seq=?',(authority[0],)).fetchone()
        if decision is None:raise Unknown('missing current decision')
        body=json.loads(decision[5])
        if body['state'] is not None:
            row=self.db.execute('SELECT * FROM state WHERE id=?',(body['state'],)).fetchone()
            self.insert('state',body['state'],list(row))
        self.insert('receipt',decision[1],row_value(decision))
        publication={'format':'G3V1','instance':self.expected_instance,'epoch':0,
                     'authority':list(authority),'root':self.root,'pending':None}
        raw=encoded(publication)
        self.db.execute('INSERT OR REPLACE INTO publication VALUES (1,?,?)',(raw,digest(raw)))

    def create(self,seed):
        if digest(encoded(seed['records']))!=self.expected_instance:
            raise Unknown('wrong fixture instance')
        self.db.executescript(SCHEMA+EXTRA_SCHEMA)
        self.db.execute('BEGIN IMMEDIATE')
        try:
            records=dict(seed['records'])
            if digest(checkpoint_bytes(records))!=seed['oracle']:raise Unknown('seed oracle mismatch')
            payload,life=self.put(records),self.put(ledger(records))
            row=[0,seed['content'],None,0,payload,seed['oracle'],life]
            self.db.execute('INSERT INTO state VALUES (?,?,?,?,?,?,?)',row)
            body=encoded({'head':0,'cursor':0,'redo':None,'ledger':life,'state':0,'state_row':row})
            root=digest(encoded([0,'seed','seed','seed','',digest(body)]))
            self.db.execute('INSERT INTO decision VALUES (0,?,?,?,?,?,?)',('seed','seed','seed','',body,root))
            self.db.execute('INSERT INTO authority VALUES (1,0,?,0,0,NULL,?)',(root,life))
            self.seal()
            self.db.execute('COMMIT');self.verified=(0,root)
        except BaseException:
            if self.db.in_transaction:self.db.execute('ROLLBACK')
            self.root=None
            raise

    def publication(self):
        row=self.db.execute('SELECT body,seal FROM publication WHERE singleton=1').fetchone()
        if row is None or digest(row[0])!=row[1]:raise Unknown('missing/corrupt publication root')
        body=json.loads(row[0])
        if body['format']!='G3V1' or body['instance']!=self.expected_instance or body['epoch']!=0:
            raise Unknown('wrong instance/epoch/format')
        if body['pending'] is not None:raise Unknown('unresolved application recovery tail')
        if body['authority']!=list(self.authority()):raise Unknown('publication/authority disagreement')
        self.root=body['root']
        self.node(self.root)
        return body

    def verified_open(self,*,maintenance=False):
        """Bounded logical reads plus a separately capped SQLite recovery envelope."""
        self.verified=None
        if self.integrity_failed and not maintenance:
            raise Unknown('known integrity fault; full audit required before admission')
        started=time.perf_counter_ns();reads=self.proof_reads;statements=self.statement_count;decoded=self.payload_bytes
        self.db.execute('BEGIN IMMEDIATE')
        try:
            # Fixed schema structure, not a scan/count of indexed historical rows.
            present=set(self.db.execute('SELECT type,name,tbl_name,sql FROM sqlite_schema LIMIT 32'))
            if present!=self.schema:raise Unknown('missing or modified required indexed structure')
            publication=self.publication()
            authority=publication['authority']
            current=self.db.execute('SELECT * FROM decision WHERE seq=?',(authority[0],)).fetchone()
            if current is None:raise Unknown('missing current decision')
            self.receipt(current[1],current[2])
            if current[6]!=authority[1]:raise Unknown('current receipt/root mismatch')
            body=json.loads(current[5])
            if [body['head'],body['cursor'],body['redo'],body['ledger']]!=authority[2:]:
                raise Unknown('decision/current authority disagreement')
            for state in set(authority[2:5])-{None}:
                if self.state_row(state) is None:raise Unknown('missing current state')
            # Activate only verified current content and complete latest lifetime authority.
            head=self.state_row(authority[2])
            if head[6]!=authority[5] or not 0<=authority[3]<=authority[2]:
                raise Unknown('head/lifetime/cursor disagreement')
            if authority[4]!=(authority[3]+1 if authority[3]<authority[2] else None):
                raise Unknown('Redo link disagreement')
            self.cold(authority[3]);self.get(authority[5])
            self.db.execute('COMMIT');self.verified=tuple(authority[:2])
        except Exception as error:
            if self.db.in_transaction:self.db.execute('ROLLBACK')
            self.verified=None
            if isinstance(error,sqlite3.DatabaseError):
                raise Unknown('SQLite could not verify recovered state: '+str(error)) from error
            raise
        return {'verified_activation_ns':time.perf_counter_ns()-started,
                'proof_nodes_read':self.proof_reads-reads,'sql_statements':self.statement_count-statements,
                'payload_bytes_decoded':self.payload_bytes-decoded,'wal_preflight_bytes':self.recovery_bytes,
                'historical_decision_scan':False,'sqlite_physical_page_access':'not exposed by Python binding'}

    def begin(self):
        if self.integrity_failed:
            self.verified=None
            raise Unknown('known integrity fault; mutation requires successful full audit')
        authority=super().begin()
        try:
            self.publication()
            return authority
        except BaseException:
            self.verified=None
            self.db.execute('ROLLBACK')
            raise

    def decide(self,tx,intent,kind,entry,authority,*,fault=None):
        # Payloads, SQL receipt/state rows, authenticated map and publication seal
        # share this SINGLE SQLite commit, including terminal negative outcomes.
        def boundary(point):
            if point=='before_commit':self.seal()
            if fault:fault(point)
        try:
            return super().decide(tx,intent,kind,entry,authority,fault=boundary)
        except BaseException:
            self.verified=None
            # The caller rolls back if still in transaction. A lost ACK must reopen.
            raise

    def audit(self):
        # Maintenance audit validates original semantic chain plus authenticated rows.
        try:
            self.verified_open(maintenance=True)
            result=super().audit()
        except Exception:
            self.verified=None
            self.integrity_failed=True
            raise
        self.db.execute('BEGIN')
        try:
            self.publication()
            for row in self.db.execute('SELECT * FROM decision'):
                self.bind_row('receipt',row[1],row)
            for row in self.db.execute('SELECT * FROM state'):
                self.bind_row('state',row[0],row)
            # Reachable proof nodes must also have matching SQL rows, including ones
            # whose rows were deleted without changing the authenticated root.
            stack=[self.root];seen=set()
            while stack:
                key=stack.pop()
                if key in seen:raise Unknown('reused/cyclic authenticated subtree')
                seen.add(key);node=self.node(key)
                if node[0]=='branch':stack.extend(node[2:])
            # super.audit already requires gap-free decisions/state sequences and
            # binds the latest horizon. No normal-open path performs this scan.
            self.db.execute('COMMIT')
            self.integrity_failed=False
            return result
        except BaseException:
            if self.db.in_transaction:self.db.execute('ROLLBACK')
            self.verified=None
            self.integrity_failed=True
            raise
