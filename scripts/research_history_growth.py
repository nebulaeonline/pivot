"""G3 bounded prefix splitting within SQLite, not a production index/growth policy."""
import time

from research_history_compact import EMPTY,frame,unframe,sha
from research_history_layout import LayoutStore


def prefix(bucket):return bin(bucket)[3:]


class GrowthStore(LayoutStore):
    def __init__(self,*args,max_entries=64,max_leaves=1024,max_depth=24,max_splits=8,**kwargs):
        if not 1<=max_entries<=1024 or not 16<=max_leaves<=1024 or not 4<=max_depth<=24 or not 1<=max_splits<=24:
            raise ValueError('research resource configuration')
        self.max_entries=max_entries;self.max_leaves=max_leaves;self.max_depth=max_depth;self.max_splits=max_splits
        self.directory={};self.generation=0;self.split_fault=None
        super().__init__(*args,**kwargs)
        if self.mode!='buckets':raise ValueError('growth research requires authenticated mode')
        self.metrics.update(split_count=0,split_publications=0,split_ns=0,directory_bytes_peak=0,
                            bucket_bytes_peak=0,leaves_peak=0,generations=0)

    def encode_directory(self):
        items=sorted(self.directory,key=prefix)
        raw=frame([b'GD1',self.generation,self.max_entries,self.max_leaves,self.max_depth,self.max_splits,
                   *[value for key in items for value in (key,self.directory[key])]])
        if len(raw)>64*1024:self.bad('directory research decode budget')
        return raw

    def decode_directory(self,raw):
        if len(raw)>64*1024:self.bad('directory research decode budget')
        fields=unframe(raw)
        if len(fields)<8 or fields[:1]!=[b'GD1'] or fields[2:6]!=[self.max_entries,self.max_leaves,self.max_depth,self.max_splits] or len(fields)%2:
            self.bad('directory format/resource contract mismatch')
        if not isinstance(fields[1],int):self.bad('directory generation')
        self.generation=fields[1];pairs=list(zip(fields[6::2],fields[7::2]))
        if len(pairs)>self.max_leaves:self.bad('directory admission budget')
        keys=[key for key,_ in pairs]
        if any(not isinstance(k,int) or k<1 or k.bit_length()-1>self.max_depth for k in keys):self.bad('invalid prefix')
        if len(set(keys))!=len(keys) or keys!=sorted(keys,key=prefix):self.bad('noncanonical directory')
        leafset=set(keys);coverage=0
        for key,value in pairs:
            if not isinstance(value,bytes) or len(value)!=32:self.bad('invalid prefix commitment')
            ancestor=key>>1
            while ancestor:
                if ancestor in leafset:self.bad('overlapping directory prefixes')
                ancestor>>=1
            coverage+=1<<(self.max_depth-(key.bit_length()-1))
        if coverage!=1<<self.max_depth:self.bad('incomplete authenticated keyspace')
        self.directory=dict(pairs)

    def read_meta(self):
        row=self.execute('SELECT body,hash FROM authority WHERE singleton=1').fetchone()
        if row is None or sha(b'M'+row[0])!=row[1]:self.bad('publication checksum')
        values=unframe(row[0])
        if len(values)!=11 or values[0]!=b'G3G1' or values[1]!=self.instance or values[2]!=0 or values[10] is not None:
            self.bad('growth instance/epoch/pending/format')
        self.meta=values;self.decode_directory(values[8]);return row[1]

    def route(self,key):
        number=int.from_bytes(key,'big')
        for depth in range(self.max_depth+1):
            bucket=(1<<depth)|(number>>(256-depth) if depth else 0)
            if bucket in self.directory:return bucket
        self.bad('directory has no authenticated route')

    def entries(self,bucket):
        row=self.execute('SELECT bytes FROM bucket WHERE id=?',(bucket,)).fetchone()
        raw=b'' if row is None else row[0]
        if len(raw)%64 or len(raw)>self.max_entries*64:self.bad('bucket decode/admission budget')
        if (sha(raw) if raw else EMPTY)!=self.directory[bucket]:self.bad('bucket commitment disagreement')
        entries=[raw[i:i+64] for i in range(0,len(raw),64)]
        keys=[entry[:32] for entry in entries]
        if keys!=sorted(set(keys)):self.bad('unordered/duplicate bucket')
        if any(self.route(key)!=bucket for key in keys):self.bad('bucket prefix disagreement')
        self.metrics['proof_read_bytes']+=len(raw)
        return entries

    def proof(self,txid):
        started=time.perf_counter_ns();sql=self.metrics['sql_ns'];key=self.key(txid)
        bucket=self.route(key);entries=self.entries(bucket)
        import bisect
        keys=[entry[:32] for entry in entries];at=bisect.bisect_left(keys,key)
        self.metrics['proof_ns']+=time.perf_counter_ns()-started-(self.metrics['sql_ns']-sql)
        return entries[at][32:] if at<len(keys) and keys[at]==key else None

    def add_proof(self,txid,receipt):
        started=time.perf_counter_ns();sql=self.metrics['sql_ns'];old_proof=self.metrics['proof_ns']
        if self.meta[3]==-1:
            self.meta[0]=b'G3G1';self.directory={i:EMPTY for i in range(16,32)};self.generation=0
        key=self.key(txid);bucket=self.route(key);entries=self.entries(bucket)
        if any(e[:32]==key for e in entries):self.bad('immutable receipt already bound')
        entries.append(key+receipt);entries.sort();pending=[(bucket,entries)];splits=0
        while pending:
            current,items=pending.pop()
            if len(items)<=self.max_entries:
                raw=b''.join(items)
                self.execute('INSERT OR REPLACE INTO bucket VALUES (?,?)',(current,raw))
                self.directory[current]=sha(raw) if raw else EMPTY
                self.metrics['proof_written_bytes']+=len(raw)
                self.metrics['bucket_bytes_peak']=max(self.metrics['bucket_bytes_peak'],len(raw))
                continue
            if current.bit_length()-1>=self.max_depth or splits>=self.max_splits or len(self.directory)+1>self.max_leaves:
                raise RuntimeError('research split admission budget exceeded; no receipt pruning')
            if self.split_fault:self.split_fault('before_split')
            depth=current.bit_length()-1
            children=[[],[]]
            for entry in items:
                bit=(int.from_bytes(entry[:32],'big')>>(255-depth))&1
                children[bit].append(entry)
            # The parent is absent only within this uncommitted generation.
            self.execute('DELETE FROM bucket WHERE id=?',(current,));del self.directory[current]
            self.directory[current*2]=EMPTY;self.directory[current*2+1]=EMPTY
            pending.extend([(current*2,children[0]),(current*2+1,children[1])]);splits+=1
            if self.split_fault:self.split_fault('after_split_plan')
        self.generation+=1;self.meta[8]=self.encode_directory()
        if self.split_fault:self.split_fault('after_children')
        self.metrics['split_count']+=splits;self.metrics['split_publications']+=int(bool(splits))
        if splits:self.metrics['split_ns']+=time.perf_counter_ns()-started
        self.metrics['generations']=self.generation
        self.metrics['directory_bytes_peak']=max(self.metrics['directory_bytes_peak'],len(self.meta[8]))
        self.metrics['leaves_peak']=max(self.metrics['leaves_peak'],len(self.directory))
        self.metrics['proof_written_bytes']+=len(self.meta[8])
        self.metrics['proof_ns']=old_proof+time.perf_counter_ns()-started-(self.metrics['sql_ns']-sql)

    def audit_rows(self):
        count=super().audit_rows()
        entries=sum(len(self.entries(bucket)) for bucket in self.directory)
        if entries!=count:self.bad('fully enumerated receipt/index disagreement')
        if any(row[0] not in self.directory for row in self.execute('SELECT id FROM bucket')):
            self.bad('unretired current bucket generation')
        return count
