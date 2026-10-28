"""Mutable authenticated radix pages owned by SQLite MVCC; G3 research only."""
import bisect
import time

from research_history_compact import EMPTY,frame,unframe,sha
from research_history_layout import LayoutStore


class HierarchyStore(LayoutStore):
    format_id=b'G3H1'
    payload_layout='rowid'
    def __init__(self,*args,max_entries=60,max_leaves=1024,max_depth=24,max_splits=8,**kwargs):
        if not 1<=max_entries<=60 or not 1<=max_leaves<=1024 or not 1<=max_depth<=24 or not 1<=max_splits<=24:
            raise ValueError('research resource contract')
        self.bounds=[max_entries,max_leaves,max_depth,max_splits]
        self.root=EMPTY;self.leaves=1;self.generation=0;self.split_fault=None
        kwargs['layout']=self.payload_layout;super().__init__(*args,**kwargs)
        if self.mode!='buckets':raise ValueError('authentication required')
        self.metrics.update(split_count=0,index_pages_read=0,index_pages_written=0,
                            path_depth_peak=0,proof_bytes_peak=0,publication_pages_peak=0)

    def digest(self,key,raw):return sha(b'H'+frame([key,raw]))

    def write_page(self,key,raw):
        self.execute('INSERT OR REPLACE INTO bucket VALUES (?,?)',(key,raw))
        self.metrics['proof_written_bytes']+=len(raw)
        self.metrics['index_pages_written']+=1
        return self.digest(key,raw)

    def page(self,key,expected):
        rows=self.execute('SELECT bytes FROM bucket WHERE id=?',(key,)).fetchall()
        if len(rows)!=1:self.bad('missing/ambiguous authenticated page')
        raw=rows[0][0]
        if len(raw)>3841 or self.digest(key,raw)!=expected:self.bad('page size/commitment mismatch')
        depth=key.bit_length()-1
        if raw[:1]==b'I':
            if len(raw)!=65 or depth>=self.bounds[2]:self.bad('invalid branch/coverage')
            result=('I',raw[1:33],raw[33:])
        elif raw[:1]==b'L':
            if (len(raw)-1)%64 or (len(raw)-1)//64>self.bounds[0]:self.bad('leaf admission bound')
            entries=[raw[i:i+64] for i in range(1,len(raw),64)]
            keys=[e[:32] for e in entries]
            if keys!=sorted(set(keys)):self.bad('duplicate/unordered leaf')
            if any((int.from_bytes(k,'big')>>(256-depth) if depth else 0)!=key-(1<<depth) for k in keys):
                self.bad('leaf prefix disagreement')
            result=('L',entries)
        else:self.bad('unknown page; coverage unproven')
        self.metrics['proof_read_bytes']+=len(raw);self.metrics['index_pages_read']+=1
        return result,len(raw)

    def proof_path(self,key):
        node=1;expected=self.root;path=[];total=0
        while True:
            page,size=self.page(node,expected);total+=size
            if page[0]=='L':break
            depth=node.bit_length()-1;bit=(int.from_bytes(key,'big')>>(255-depth))&1
            path.append((node,page,bit));node=node*2+bit;expected=page[1+bit]
        self.metrics['path_depth_peak']=max(self.metrics['path_depth_peak'],len(path))
        self.metrics['proof_bytes_peak']=max(self.metrics['proof_bytes_peak'],total)
        return path,node,page[1]

    def proof(self,txid):
        start=time.perf_counter_ns();sql=self.metrics['sql_ns'];key=self.key(txid)
        _,_,entries=self.proof_path(key);keys=[e[:32] for e in entries];at=bisect.bisect_left(keys,key)
        self.metrics['proof_ns']+=time.perf_counter_ns()-start-(self.metrics['sql_ns']-sql)
        return entries[at][32:] if at<len(entries) and keys[at]==key else None

    def read_meta(self):
        row=self.execute('SELECT body,hash FROM authority WHERE singleton=1').fetchone()
        if row is None or sha(b'M'+row[0])!=row[1]:self.bad('publication checksum')
        values=unframe(row[0])
        if len(values)!=11 or values[:3]!=[self.format_id,self.instance,0] or values[10] is not None:
            self.bad('hierarchy format/instance/epoch/pending')
        fields=unframe(values[8])
        if len(fields)!=8 or fields[:1]!=[b'HD1'] or fields[1:5]!=self.bounds:
            self.bad('hierarchy resource contract')
        if not isinstance(fields[5],int) or not isinstance(fields[6],int) or not 1<=fields[6]<=self.bounds[1] or not isinstance(fields[7],bytes) or len(fields[7])!=32:
            self.bad('hierarchy root envelope')
        self.generation,self.leaves,self.root=fields[5:];self.meta=values
        return row[1]

    def add_proof(self,txid,receipt):
        start=time.perf_counter_ns();sql=self.metrics['sql_ns'];before=self.metrics['index_pages_written']
        if self.meta[3]==-1:
            self.meta[0]=self.format_id;self.root=self.write_page(1,b'L')
        key=self.key(txid);path,node,entries=self.proof_path(key)
        if any(e[:32]==key for e in entries):self.bad('immutable receipt already bound')
        entries=sorted(entries+[key+receipt]);splits=0

        def build(current,items):
            nonlocal splits
            if len(items)<=self.bounds[0]:return self.write_page(current,b'L'+b''.join(items))
            depth=current.bit_length()-1
            if depth>=self.bounds[2] or splits>=self.bounds[3] or self.leaves>=self.bounds[1]:
                raise RuntimeError('hierarchy admission budget; no pruning')
            if self.split_fault:self.split_fault('before_split')
            splits+=1;self.leaves+=1;children=[[],[]]
            for item in items:children[(int.from_bytes(item[:32],'big')>>(255-depth))&1].append(item)
            if self.split_fault:self.split_fault('after_split_plan')
            left=build(current*2,children[0]);right=build(current*2+1,children[1])
            if self.split_fault:self.split_fault('after_children')
            return self.write_page(current,b'I'+left+right)

        digest=build(node,entries)
        for ancestor,page,bit in reversed(path):
            children=list(page[1:]);children[bit]=digest
            digest=self.write_page(ancestor,b'I'+b''.join(children))
        self.root=digest;self.generation+=1
        self.meta[8]=frame([b'HD1',*self.bounds,self.generation,self.leaves,self.root])
        self.metrics['proof_written_bytes']+=len(self.meta[8]);self.metrics['split_count']+=splits
        self.metrics['publication_pages_peak']=max(self.metrics['publication_pages_peak'],self.metrics['index_pages_written']-before)
        self.metrics['proof_ns']+=time.perf_counter_ns()-start-(self.metrics['sql_ns']-sql)

    def audit_rows(self):
        count=super().audit_rows();seen=set();entries=0;leaves=0
        def visit(key,expected):
            nonlocal entries,leaves
            seen.add(key);page,_=self.page(key,expected)
            if page[0]=='L':entries+=len(page[1]);leaves+=1
            else:visit(key*2,page[1]);visit(key*2+1,page[2])
        visit(1,self.root)
        if entries!=count or leaves!=self.leaves or set(r[0] for r in self.execute('SELECT id FROM bucket'))!=seen:
            self.bad('full directory coverage/count/orphan disagreement')
        return count


class CanonicalCursor:
    def __init__(self,cursor,decode):self.cursor=cursor;self.decode=decode
    def fetchone(self):
        row=self.cursor.fetchone();return None if row is None else self.decode(row)
    def fetchall(self):return list(self)
    def __iter__(self):
        for row in self.cursor:yield self.decode(row)


class NormalizedStore(HierarchyStore):
    """Lossless physical receipt encoding; canonical receipt hashes stay unchanged."""
    format_id=b'G3N1'
    payload_layout='local'  # INTEGER PRIMARY KEY keeps local references stable through VACUUM.
    def ref(self,key):
        if key is None:return None
        row=self.execute('SELECT rowid FROM payload WHERE hash=?',(key,)).fetchone()
        if row is None:self.bad('missing normalized payload reference')
        return row[0]

    def unref(self,local):
        if local is None:return None
        row=self.execute('SELECT hash FROM payload WHERE rowid=?',(local,)).fetchone()
        if row is None:self.bad('dangling normalized payload reference')
        return row[0]

    def canonical(self,row):
        seq,txid,intent,kind,edit,raw,digest=row
        values=unframe(raw)
        prior=EMPTY if seq==0 else self.execute('SELECT hash FROM receipt WHERE seq=?',(seq-1,)).fetchone()
        if seq and prior is None:self.bad('missing predecessor')
        if seq:prior=prior[0]
        if values[:1]!=[b'N1']:self.bad('normalized receipt version')
        if kind in (0,1):
            if edit is None or len(values)!=6:self.bad('normalized edit fields')
            life,content,delta,checkpoint,oracle=values[1:]
            body=[prior,edit,edit,None,self.unref(life),None if edit==0 else edit-1,
                  edit-edit%64,content,self.unref(delta),self.unref(checkpoint),oracle]
        elif kind in (2,3):
            if edit is not None or len(values)!=5:self.bad('normalized control fields')
            head,cursor,redo,life=values[1:]
            body=[prior,head,cursor,redo,self.unref(life),None,None,None,None,None,None]
        else:self.bad('normalized decision kind')
        return (seq,txid,intent,kind,edit,frame(body),digest)

    def execute(self,sql,params=()):
        if sql=='INSERT INTO receipt VALUES (?,?,?,?,?,?,?)':
            seq,txid,intent,kind,edit,raw,digest=params;body=unframe(raw)
            if kind in (0,1):values=[b'N1',self.ref(body[4]),body[7],self.ref(body[8]),self.ref(body[9]),body[10]]
            else:values=[b'N1',*body[1:4],self.ref(body[4])]
            params=(seq,txid,intent,kind,edit,frame(values),digest)
            if self.canonical(params)[5]!=raw:self.bad('normalization would lose canonical information')
        cursor=super().execute(sql,params)
        return CanonicalCursor(cursor,self.canonical) if sql.startswith('SELECT * FROM receipt') else cursor


class PackedStore(NormalizedStore):
    """Same proof nodes; separate small branches from variable-size leaves."""
    format_id=b'G3P1'
    def __init__(self,*args,**kwargs):
        super().__init__(*args,**kwargs)
        import sqlite3
        self.schema_sql+='CREATE TABLE branch(id INTEGER PRIMARY KEY,bytes BLOB NOT NULL);'
        model=self.connect(':memory:')
        try:
            model.executescript(self.schema_sql)
            self.schema=set(model.execute('SELECT type,name,tbl_name,sql FROM sqlite_schema'))
        finally:model.close()

    def execute(self,sql,params=()):
        if sql=='SELECT bytes FROM bucket WHERE id=?':
            sql='SELECT bytes FROM branch WHERE id=? UNION ALL SELECT bytes FROM bucket WHERE id=?'
            params=(*params,*params)
        elif sql=='SELECT id FROM bucket':sql='SELECT id FROM bucket UNION ALL SELECT id FROM branch'
        return super().execute(sql,params)

    def write_page(self,key,raw):
        if raw[:1]==b'I':
            self.execute('INSERT OR REPLACE INTO branch VALUES (?,?)',(key,raw))
            self.execute('DELETE FROM bucket WHERE id=?',(key,))
        else:self.execute('INSERT OR REPLACE INTO bucket VALUES (?,?)',(key,raw))
        self.metrics['proof_written_bytes']+=len(raw);self.metrics['index_pages_written']+=1
        return self.digest(key,raw)

    def authentication_extra_bytes(self):
        return self.db.execute('SELECT coalesce(sum(length(bytes)),0) FROM branch').fetchone()[0]

    def audit_rows(self):
        if self.db.execute('SELECT count(*) FROM branch JOIN bucket USING(id)').fetchone()[0]:
            self.bad('duplicate physical node address')
        return super().audit_rows()
