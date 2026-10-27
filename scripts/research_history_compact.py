"""G3 compact SQLite representations; research only, no production format/API.

plain: transactionally indexed receipts, no bounded authenticated negative result.
buckets: fixed commitment directory authenticates bounded receipt hash buckets.
"""
import hashlib
import sqlite3
import struct
import time
import uuid
import zlib

from research_history_index import Unknown,Backpressure,apply_ops,checkpoint_bytes,ledger
from research_history_verified import VerifiedStore

MAX_BYTES=32*1024*1024
BUCKETS=256
MAX_BUCKET_BYTES=64*1024
EMPTY=bytes(32)


def sha(raw):
    return hashlib.sha256(raw).digest()


def varint(number):
    if number<0:raise ValueError('negative integer')
    out=bytearray()
    while number>127:out.append((number&127)|128);number>>=7
    out.append(number);return bytes(out)


def frame(values):
    out=bytearray()
    for value in values:
        if value is None:out.append(0)
        elif isinstance(value,int):out.append(1);out.extend(varint(value))
        elif isinstance(value,bytes):out.append(2);out.extend(varint(len(value)));out.extend(value)
        else:raise TypeError('binary fields only')
    return bytes(out)


def unframe(raw):
    at=0;out=[]
    def number():
        nonlocal at
        value=0
        for shift in range(0,70,7):
            if at>=len(raw):raise Unknown('truncated integer')
            byte=raw[at];at+=1;value|=(byte&127)<<shift
            if byte<128:return value
        raise Unknown('oversized integer')
    while at<len(raw):
        tag=raw[at];at+=1
        if tag==0:out.append(None)
        elif tag==1:out.append(number())
        elif tag==2:
            length=number()
            if length>MAX_BYTES or at+length>len(raw):raise Unknown('truncated binary field')
            out.append(raw[at:at+length]);at+=length
        else:raise Unknown('unknown binary field')
    return out


def pack_ops(ops):
    out=bytearray(b'D1'+struct.pack('<I',len(ops)))
    for key,before,after in ops:
        rawkey=bytes.fromhex(key)
        if len(rawkey)!=33:raise Unknown('stable record key length')
        out.extend(rawkey)
        for value in [before,after]:
            data=None if value is None else bytes.fromhex(value)
            out.extend(struct.pack('<I',0xffffffff if data is None else len(data)))
            if data is not None:out.extend(data)
    if len(out)>MAX_BYTES:raise Unknown('research codec resource budget')
    return bytes(out)


def unpack_ops(raw):
    if raw[:2]!=b'D1' or len(raw)<6:raise Unknown('bad delta encoding')
    count=struct.unpack_from('<I',raw,2)[0];at=6;out=[]
    if count>(len(raw)-6)//41:raise Unknown('impossible record count')
    for _ in range(count):
        if at+33>len(raw):raise Unknown('truncated key')
        key=raw[at:at+33].hex();at+=33;values=[]
        for _ in range(2):
            if at+4>len(raw):raise Unknown('truncated length')
            length=struct.unpack_from('<I',raw,at)[0];at+=4
            if length==0xffffffff:values.append(None)
            else:
                if length>MAX_BYTES or at+length>len(raw):raise Unknown('truncated record')
                values.append(raw[at:at+length].hex());at+=length
        out.append([key,*values])
    if at!=len(raw):raise Unknown('trailing delta bytes')
    keys=[r[0] for r in out]
    if keys!=sorted(set(keys)):raise Unknown('unordered/duplicate record keys')
    return out


def pack_map(records):
    return pack_ops([[key,None,value] for key,value in sorted(records.items())])


def unpack_map(raw):
    ops=unpack_ops(raw)
    if any(before is not None or after is None for _,before,after in ops):raise Unknown('invalid checkpoint records')
    return {key:after for key,_,after in ops}


def decompress(raw,length):
    if not 0<=length<=MAX_BYTES:raise Unknown('decoded resource budget')
    decoder=zlib.decompressobj();out=decoder.decompress(raw,MAX_BYTES+1)
    if len(out)!=length or not decoder.eof or decoder.unused_data or decoder.unconsumed_tail:
        raise Unknown('compressed length/trailing data mismatch')
    return out


SCHEMA='''
CREATE TABLE payload(hash BLOB PRIMARY KEY,kind INTEGER NOT NULL,raw_length INTEGER NOT NULL,bytes BLOB NOT NULL) WITHOUT ROWID;
CREATE TABLE receipt(seq INTEGER PRIMARY KEY,id BLOB UNIQUE NOT NULL,intent BLOB NOT NULL,
 kind INTEGER NOT NULL,edit INTEGER UNIQUE,body BLOB NOT NULL,hash BLOB NOT NULL);
CREATE TABLE bucket(id INTEGER PRIMARY KEY,bytes BLOB NOT NULL);
CREATE TABLE authority(singleton INTEGER PRIMARY KEY CHECK(singleton=1),body BLOB NOT NULL,hash BLOB NOT NULL);
'''


class CompactStore:
    def __init__(self,path,instance,mode,*,fullfsync=False):
        if mode not in ('plain','buckets'):raise ValueError('research mode')
        self.path=path;self.instance=instance;self.mode=mode;self.verified=None;self.corrupt_handle=False
        self.meta=None;self.metrics={'proof_read_bytes':0,'proof_written_bytes':0,'proof_ns':0,'sql_ns':0,'commit_ns':0}
        self.recovery_bytes=VerifiedStore.preflight(path)
        self.db=sqlite3.connect(path,isolation_level=None,timeout=.2)
        model=sqlite3.connect(':memory:')
        try:
            model.executescript(SCHEMA)
            self.schema=set(model.execute('SELECT type,name,tbl_name,sql FROM sqlite_schema'))
        finally:model.close()
        self.execute('PRAGMA foreign_keys=ON')
        if self.execute('PRAGMA journal_mode=WAL').fetchone()[0]!='wal':raise Unknown('WAL required')
        for pragma,value in [('synchronous','FULL'),('cache_size','-2048'),('mmap_size','0'),('wal_autocheckpoint','0'),('fullfsync',str(int(fullfsync)))]:
            self.execute('PRAGMA '+pragma+'='+value)

    def execute(self,*args):
        started=time.perf_counter_ns()
        try:return self.db.execute(*args)
        finally:self.metrics['sql_ns']+=time.perf_counter_ns()-started

    def close(self):self.db.close()
    def wal_bytes(self):
        from pathlib import Path
        p=Path(str(self.path)+'-wal');return p.stat().st_size if p.exists() else 0

    def bad(self,message):
        self.verified=None;self.corrupt_handle=True;raise Unknown(message)

    def blob(self,kind,raw):
        key=sha(bytes([kind])+raw);compressed=zlib.compress(raw,6)
        return key,kind,len(raw),compressed

    def put(self,blob):
        self.execute('INSERT OR IGNORE INTO payload VALUES (?,?,?,?)',blob)
        row=self.execute('SELECT kind,raw_length,bytes FROM payload WHERE hash=?',(blob[0],)).fetchone()
        if row!=blob[1:]:self.bad('payload identity disagreement')
        return blob[0]

    def get(self,key,kind):
        row=self.execute('SELECT kind,raw_length,bytes FROM payload WHERE hash=?',(key,)).fetchone()
        if row is None or row[0]!=kind:self.bad('missing/wrong payload')
        try:raw=decompress(row[2],row[1])
        except (Unknown,zlib.error) as error:self.bad('invalid compressed payload: '+str(error))
        if sha(bytes([kind])+raw)!=key:self.bad('payload digest mismatch')
        return raw

    def txid(self,label):return sha(frame([self.instance,label.encode()]))[:16]

    def key(self,txid):return sha(frame([self.instance,0,txid]))

    def proof(self,txid):
        if self.mode=='plain':return None
        started=time.perf_counter_ns();sql_before=self.metrics['sql_ns'];proof_before=self.metrics['proof_ns'];key=self.key(txid);index=key[0]
        expected=self.meta[8][index*32:(index+1)*32]
        row=self.execute('SELECT bytes FROM bucket WHERE id=?',(index,)).fetchone()
        raw=b'' if row is None else row[0]
        if len(raw)>MAX_BUCKET_BYTES or len(raw)%63:self.bad('bucket shape/resource bound')
        if (sha(raw) if raw else EMPTY)!=expected:self.bad('bucket commitment disagreement')
        keys=[raw[i:i+31] for i in range(0,len(raw),63)]
        if keys!=sorted(set(keys)):self.bad('unordered bucket')
        self.metrics['proof_read_bytes']+=len(raw)
        self.metrics['proof_ns']=proof_before+time.perf_counter_ns()-started-(self.metrics['sql_ns']-sql_before)
        import bisect
        at=bisect.bisect_left(keys,key[1:])
        return raw[at*63+31:at*63+63] if at<len(keys) and keys[at]==key[1:] else None

    def bind(self,row):
        if row is None:return None
        if sha(b'R'+frame(row[:6]))!=row[6]:self.bad('receipt checksum mismatch')
        if self.mode=='buckets' and self.proof(row[1])!=row[6]:self.bad('receipt absent from committed bucket')
        return row

    def find(self,txid):
        expected=self.proof(txid) if self.mode=='buckets' else None
        row=self.execute('SELECT * FROM receipt WHERE id=?',(txid,)).fetchone()
        if self.mode=='buckets' and ((row is None)!=(expected is None) or (row is not None and row[6]!=expected)):
            self.bad('required receipt/index disagreement')
        return self.bind(row)

    def state(self,edit):
        row=self.execute('SELECT * FROM receipt WHERE edit=?',(edit,)).fetchone()
        if row is None:self.bad('missing state locator/receipt')
        return self.bind(row)

    def cold(self,edit):
        row=self.state(edit);body=unframe(row[5]);base=body[6]
        if not 0<=edit-base<64:self.bad('decode bound')
        checkpoint=unframe(self.state(base)[5])
        if checkpoint[6]!=base or checkpoint[9] is None:self.bad('missing checkpoint')
        records=unpack_map(self.get(checkpoint[9],2))
        for n in range(base+1,edit+1):
            data=unframe(self.state(n)[5])
            if data[5]!=n-1:self.bad('state parent mismatch')
            try:records=apply_ops(records,unpack_ops(self.get(data[8],1)))
            except Unknown as error:self.bad(str(error))
        if sha(checkpoint_bytes(records))!=body[10] or unpack_map(self.get(body[4],3))!=ledger(records):
            self.bad('oracle/lifetime mismatch')
        return records

    def read_meta(self):
        row=self.execute('SELECT body,hash FROM authority WHERE singleton=1').fetchone()
        if row is None or sha(b'M'+row[0])!=row[1]:self.bad('publication checksum')
        data=unframe(row[0])
        if len(data)!=11 or data[0]!=b'G3C1'+self.mode.encode() or data[1]!=self.instance or data[2]!=0:
            self.bad('wrong format/instance/epoch')
        # data: format, instance, epoch, seq, receipt, head, cursor, redo, directory, ledger, pending
        if len(data[8])!=(8192 if self.mode=='buckets' else 0) or data[10] is not None:self.bad('directory/pending recovery state')
        self.meta=data;return row[1]

    def open_verified(self,*,maintenance=False):
        if self.corrupt_handle and not maintenance:raise Unknown('known integrity fault; full audit required')
        self.verified=None;start=time.perf_counter_ns()
        self.execute('BEGIN IMMEDIATE')
        try:
            if set(self.execute('SELECT type,name,tbl_name,sql FROM sqlite_schema LIMIT 16'))!=self.schema:self.bad('required indexed schema changed')
            token=self.read_meta()
            row=self.execute('SELECT * FROM receipt WHERE seq=?',(self.meta[3],)).fetchone()
            if row is None or self.bind(row)[6]!=self.meta[4]:self.bad('missing current receipt')
            body=unframe(row[5])
            if body[1:5]!=[self.meta[5],self.meta[6],self.meta[7],self.meta[9]]:self.bad('authority/receipt mismatch')
            if not 0<=self.meta[6]<=self.meta[5] or self.meta[7]!=(self.meta[6]+1 if self.meta[6]<self.meta[5] else None):self.bad('cursor/Redo mismatch')
            head=unframe(self.state(self.meta[5])[5])
            if head[4]!=self.meta[9]:self.bad('latest lifetime binding')
            self.cold(self.meta[6]);self.get(self.meta[9],3)
            self.execute('COMMIT');self.verified=token
        except Exception:
            if self.db.in_transaction:self.execute('ROLLBACK')
            self.verified=None;raise
        return {'activation_ns':time.perf_counter_ns()-start,'authenticated_negative':self.mode=='buckets',
                'historical_scan':False,'recovery_input_bytes':self.recovery_bytes}

    def begin(self):
        if self.corrupt_handle or self.verified is None:raise Unknown('mutation authority blocked')
        if self.wal_bytes()>=64*1024*1024:
            self.execute('PRAGMA wal_checkpoint(TRUNCATE)')
            if self.wal_bytes()>=64*1024*1024:raise Backpressure('reader pins WAL')
        self.execute('BEGIN IMMEDIATE')
        try:
            if self.read_meta()!=self.verified:raise Unknown('stale verified horizon')
        except Exception:
            self.execute('ROLLBACK');self.verified=None;raise

    def add_proof(self,txid,receipt):
        if self.mode=='plain':return
        started=time.perf_counter_ns();sql_before=self.metrics['sql_ns'];proof_before=self.metrics['proof_ns'];key=self.key(txid);index=key[0]
        if self.proof(txid) is not None:self.bad('receipt already committed')
        row=self.execute('SELECT bytes FROM bucket WHERE id=?',(index,)).fetchone()
        entries=[] if row is None else [row[0][i:i+63] for i in range(0,len(row[0]),63)]
        entries.append(key[1:]+receipt);entries.sort();raw=b''.join(entries)
        if len(raw)>MAX_BUCKET_BYTES:raise Unknown('research bucket budget exceeded; no retention performed')
        self.execute('INSERT OR REPLACE INTO bucket VALUES (?,?)',(index,raw))
        directory=bytearray(self.meta[8]);directory[index*32:(index+1)*32]=sha(raw);self.meta[8]=bytes(directory)
        self.metrics['proof_written_bytes']+=len(raw)+len(directory)
        self.metrics['proof_ns']=proof_before+time.perf_counter_ns()-started-(self.metrics['sql_ns']-sql_before)

    def commit_row(self,txid,intent,kind,edit,body,*,fault=None):
        seq=self.meta[3]+1;raw=frame(body);receipt=sha(b'R'+frame([seq,txid,intent,kind,edit,raw]))
        self.execute('INSERT INTO receipt VALUES (?,?,?,?,?,?,?)',(seq,txid,intent,kind,edit,raw,receipt))
        self.add_proof(txid,receipt)
        self.meta[3:8]=[seq,receipt,body[1],body[2],body[3]];self.meta[9]=body[4]
        encoded=frame(self.meta);token=sha(b'M'+encoded)
        self.execute('INSERT OR REPLACE INTO authority VALUES (1,?,?)',(encoded,token))
        if fault:fault('before_commit')
        start=time.perf_counter_ns();self.execute('COMMIT');commit_ns=time.perf_counter_ns()-start
        self.verified=token
        if fault:fault('after_commit')
        return {'outcome':kind,'receipt':receipt.hex(),'seq':seq,'commit_ns':commit_ns,'ack_ns':time.perf_counter_ns()-start}

    def create(self,seed):
        self.db.executescript(SCHEMA);self.execute('BEGIN IMMEDIATE')
        try:
            records=dict(seed['records'])
            if sha(checkpoint_bytes(records)).hex()!=seed['oracle']:self.bad('seed oracle')
            checkpoint=self.put(self.blob(2,pack_map(records)));life=self.put(self.blob(3,pack_map(ledger(records))))
            self.meta=[b'G3C1'+self.mode.encode(),self.instance,0,-1,EMPTY,0,0,None,bytes(8192) if self.mode=='buckets' else b'',life,None]
            self.commit_row(self.txid('seed'),sha(b'seed'),0,0,
                [EMPTY,0,0,None,life,None,0,uuid.UUID(seed['content']).bytes,None,checkpoint,bytes.fromhex(seed['oracle'])])
        except Exception:
            if self.db.in_transaction:self.execute('ROLLBACK')
            self.verified=None;raise

    def prepare(self,item,prior):
        oracle_start=time.perf_counter_ns()
        records=apply_ops(prior,item['ops'])
        if sha(checkpoint_bytes(records)).hex()!=item['oracle']:raise Unknown('independent G2 oracle mismatch')
        if item['checkpoint'] is not None and dict(item['checkpoint'])!=records:raise Unknown('checkpoint/oracle disagreement')
        if (item['checkpoint'] is not None)!=(item['seq']%64==0):raise Unknown('fixture checkpoint cadence')
        oracle_ns=time.perf_counter_ns()-oracle_start
        started=time.perf_counter_ns();n=item['seq'];ops=pack_ops(item['ops'])
        delta=self.blob(1,ops);blobs=[delta]
        if any(1<=bytes.fromhex(k)[0]<=4 for k,_,_ in item['ops']):
            life=self.blob(3,pack_map(ledger(records)));blobs.append(life);life_hash=life[0]
        else:life_hash=self.meta[9]
        checkpoint=None
        if item['checkpoint'] is not None:
            checkpoint=self.blob(2,pack_map(dict(item['checkpoint'])));blobs.append(checkpoint)
        content=uuid.UUID(item['content']).bytes;oracle=bytes.fromhex(item['oracle'])
        intent=sha(frame([n,content,oracle,ops]))
        return {'n':n,'id':self.txid('edit-'+str(n)),'intent':intent,'content':content,'oracle':oracle,
                'delta':delta[0],'checkpoint':None if checkpoint is None else checkpoint[0],
                'ledger':life_hash,'blobs':blobs,'prepare_ns':time.perf_counter_ns()-started,'oracle_ns':oracle_ns,'records':records}

    def result(self,row,intent):
        if row[2]!=intent:raise Unknown('conflicting intent')
        return {'outcome':row[3],'receipt':row[6].hex(),'seq':row[0]}

    def operation(self,call):
        try:return call()
        except Backpressure:raise
        except Exception:
            if self.db.in_transaction:self.execute('ROLLBACK')
            self.verified=None;raise

    def publish(self,prepared,*,fault=None,fixture_fresh=False):
        def perform():
            self.begin();old=self.find(prepared['id'])
            if old:
                result=self.result(old,prepared['intent']);self.execute('ROLLBACK');return result
            if self.mode=='plain' and not fixture_fresh:
                # General unknown-ID admission requires the same complete proof as
                # negative reconciliation. Only the controlled, freshly generated
                # benchmark stream can bypass this O(history) measurement cost.
                self.audit_rows()
            n=prepared['n']
            if self.meta[5]!=n-1 or self.meta[6]!=n-1:raise Unknown('stale source/ambiguous receipt')
            for blob in prepared['blobs']:self.put(blob)
            if fault:fault('after_payload')
            return self.commit_row(prepared['id'],prepared['intent'],1,n,
                [self.meta[4],n,n,None,prepared['ledger'],n-1,n-n%64,prepared['content'],
                 prepared['delta'],prepared['checkpoint'],prepared['oracle']],fault=fault)
        return self.operation(perform)

    def reconcile(self,txid,intent,*,full_audit=False):
        def perform():
            self.begin();row=self.find(txid)
            if row:
                result=self.result(row,intent);self.execute('ROLLBACK');return result
            if self.mode=='plain':
                if not full_audit:raise Unknown('SQL absence is not authenticated; explicit full audit required')
                self.audit_rows()  # Same writer-excluded transaction as terminal decision.
            return self.commit_row(txid,intent,2,None,
                [self.meta[4],self.meta[5],self.meta[6],self.meta[7],self.meta[9],None,None,None,None,None,None])
        return self.operation(perform)

    def move(self,target,label):
        def perform():
            self.begin();self.cold(target)
            if target not in (self.meta[6]-1,self.meta[7]):raise Unknown('adjacent linear control only')
            redo=self.meta[6] if target==self.meta[6]-1 else (target+1 if target<self.meta[5] else None)
            return self.commit_row(self.txid(label),sha(frame([target])),3,None,
                [self.meta[4],self.meta[5],target,redo,self.meta[9],None,None,None,None,None,None])
        return self.operation(perform)

    def audit_rows(self):
        previous=EMPTY;count=0;head=-1;cursor=-1;redo=None;life=None;records={}
        for row in self.execute('SELECT * FROM receipt ORDER BY seq'):
            self.bind(row);body=unframe(row[5])
            if row[0]!=count or body[0]!=previous:self.bad('history receipt chain gap')
            if row[4] is not None:
                n=row[4]
                if n!=head+1:self.bad('state sequence gap')
                if row[3]!=(0 if n==0 else 1) or body[1:4]!=[n,n,None] or (n and cursor!=head):self.bad('edit/control disagreement')
                if n==0:records=unpack_map(self.get(body[9],2))
                else:
                    if body[5]!=head:self.bad('parent mismatch')
                    try:records=apply_ops(records,unpack_ops(self.get(body[8],1)))
                    except Unknown as error:self.bad(str(error))
                if body[9] is not None and unpack_map(self.get(body[9],2))!=records:self.bad('checkpoint mismatch')
                if sha(checkpoint_bytes(records))!=body[10] or unpack_map(self.get(body[4],3))!=ledger(records):self.bad('audit content/ledger mismatch')
                head=n;cursor=n;redo=None;life=body[4]
            elif row[3]==2:
                if body[1:5]!=[head,cursor,redo,life] or any(v is not None for v in body[5:]):self.bad('negative changes state authority')
            elif row[3]==3:
                target=body[2]
                if target not in (cursor-1,redo):self.bad('cursor relation')
                expected_redo=cursor if target==cursor-1 else (target+1 if target<head else None)
                if body[1:5]!=[head,target,expected_redo,life] or any(v is not None for v in body[5:]):self.bad('cursor authority mismatch')
                cursor=target;redo=expected_redo
            else:self.bad('unknown decision kind')
            previous=row[6];count+=1
        if count!=self.meta[3]+1 or previous!=self.meta[4] or [head,cursor,redo,life]!=[self.meta[5],self.meta[6],self.meta[7],self.meta[9]]:self.bad('audit publication disagreement')
        return count

    def audit(self):
        self.verified=None;start=time.perf_counter_ns();self.execute('BEGIN IMMEDIATE')
        try:
            token=self.read_meta()
            if self.execute('PRAGMA integrity_check').fetchall()!=[('ok',)]:self.bad('SQLite integrity')
            count=self.audit_rows()
            self.execute('COMMIT');self.corrupt_handle=False;self.verified=token
            return {'rows':count,'ns':time.perf_counter_ns()-start}
        except Exception:
            if self.db.in_transaction:self.execute('ROLLBACK')
            self.corrupt_handle=True;self.verified=None;raise
