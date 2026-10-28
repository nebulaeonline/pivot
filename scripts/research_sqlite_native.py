"""Minimal controlled-research DB API backed only by the pinned static SQLite shim."""
import ctypes as C
import json
import os
from pathlib import Path
import sqlite3  # Exception classes only; never supplies this adapter's connections.
import weakref

P=C.c_void_p;I=C.c_int;L=C.c_longlong;S=C.c_char_p


class NativeEngine:
    _instances={}
    def __new__(cls,path=None):
        path=str(Path(path or os.environ['PIVOT_RESEARCH_SQLITE']).resolve())
        if path not in cls._instances:
            instance=super().__new__(cls);instance._init(path);cls._instances[path]=instance
        return cls._instances[path]

    def _init(self,path):
        self.path=Path(path);self.lib=C.CDLL(path)
        specs={'init':(I,[]),'version':(S,[]),'source':(S,[]),'parent':(S,[]),
            'open':(I,[S,C.POINTER(P)]),'close':(I,[P]),'error':(S,[P]),'errcode':(I,[P]),'auto':(I,[P]),
            'prepare':(I,[P,S,C.POINTER(P)]),'exec':(I,[P,S]),'step':(I,[P]),'dirty':(I,[P]),'finalize':(I,[P]),'readonly':(I,[P]),
            'bind_null':(I,[P,I]),'bind_int':(I,[P,I,L]),'bind_double':(I,[P,I,C.c_double]),
            'bind_blob':(I,[P,I,P,I]),'bind_text':(I,[P,I,S,I]),
            'columns':(I,[P]),'type':(I,[P,I]),'int':(L,[P,I]),'double':(C.c_double,[P,I]),
            'blob':(P,[P,I]),'text':(P,[P,I]),'bytes':(I,[P,I]),'observe':(None,[P]),
            'counters':(None,[C.POINTER(L)]),'fault':(None,[I,I,I]),'limit':(L,[L]),
            'memory':(None,[C.POINTER(L)]),'status':(I,[P,I,C.POINTER(L),C.POINTER(L)]),'vfs':(I,[P,P,I])}
        for name,(restype,args) in specs.items():
            fn=getattr(self.lib,'pv_'+name);fn.restype=restype;fn.argtypes=args;setattr(self,name,fn)
        rc=self.init()
        if rc:raise RuntimeError('private SQLite initialization '+str(rc))
        pin=json.loads((Path(__file__).parent/'research/sqlite-source.json').read_text())
        if self.version().decode()!=pin['version'] or self.source().decode()!=pin['source_id']:
            raise RuntimeError('unqualified SQLite source identity')
        self.limit(64*1024*1024)

    def connect(self,path,**unused):return Connection(self,str(path))

    def telemetry(self,connection=None):
        counters=(L*48)();self.counters(counters);memory=(L*4)();self.memory(memory)
        result={'memory_used':memory[0],'memory_highwater':memory[1],'malloc_count':memory[2],'malloc_count_highwater':memory[3],
                'hard_heap_limit':self.limit(-1),'vfs':{},'wal_frame_header_write_events':counters[40],
                'commits':counters[41],'last_committed_wal_frames':counters[42],'max_committed_wal_frames':counters[43],
                'checkpoint_starts':counters[44],'checkpoint_completions':counters[45],
                'step_boundary_dirty_pages':counters[30],'step_boundary_dirty_pages_peak':counters[31],
                'injected_io_failures':counters[46],'injected_allocation_failures':counters[47]}
        for kind,offset in [('main',0),('wal',10),('other',20)]:
            result['vfs'][kind]=dict(zip(['read_calls','read_bytes','write_calls','write_bytes','sync_calls','sync_ns',
                'sync_full_calls','sync_normal_calls','truncate_calls'],counters[offset:offset+9]))
        if connection:
            result['connection']={}
            for name,op in [('lookaside',0),('cache_bytes',1),('schema_bytes',2),('statement_bytes',3),
                            ('cache_hits',7),('cache_misses',8),('cache_writes',9),('cache_spills',12)]:
                current=L();high=L();rc=self.status(connection.handle,op,C.byref(current),C.byref(high))
                result['connection'][name]={'current':current.value,'highwater':high.value,'rc':rc}
        return result


class Connection:
    def __init__(self,engine,path):
        self.engine=engine;self.handle=P();self.cursors=weakref.WeakSet();self.mutations=0
        rc=engine.open(path.encode(),C.byref(self.handle))
        if rc:self.raise_error(rc)
        self.sqlite_version=engine.version().decode()

    def raise_error(self,rc):
        message=self.engine.error(self.handle).decode(errors='replace')
        error=(sqlite3.IntegrityError if rc&255==19 else sqlite3.OperationalError)(message)
        error.sqlite_errorcode=rc;raise error

    @property
    def in_transaction(self):return bool(self.handle) and not self.engine.auto(self.handle)
    def execute(self,sql,params=()):return Cursor(self,sql,params)
    def executescript(self,script):
        self.mutations+=1;rc=self.engine.exec(self.handle,script.encode())
        if rc:self.raise_error(rc)
    def observe(self):self.engine.observe(self.handle)
    def identity(self):
        buf=C.create_string_buffer(512);rc=self.engine.vfs(self.handle,buf,len(buf))
        return {'version':self.sqlite_version,'source_id':self.engine.source().decode(),
                'selected_vfs':'pivot-observe','delegate_vfs':self.engine.parent().decode(),
                'connection_file_vfs':buf.value.decode() if not rc else None,
                'compile_options':[row[0] for row in self.execute('PRAGMA compile_options')]}
    def close(self):
        if self.handle:
            for cursor in list(self.cursors):cursor.close()
            rc=self.engine.close(self.handle)
            if rc:self.raise_error(rc)
            self.handle=P()


class Cursor:
    def __init__(self,connection,sql,params):
        self.connection=connection;self.statement=P();self.row=None;connection.cursors.add(self);e=connection.engine
        rc=e.prepare(connection.handle,sql.encode(),C.byref(self.statement))
        if rc:self.close();connection.raise_error(rc)
        try:
            for index,value in enumerate(params,1):
                if value is None:rc=e.bind_null(self.statement,index)
                elif isinstance(value,int):rc=e.bind_int(self.statement,index,value)
                elif isinstance(value,float):rc=e.bind_double(self.statement,index,value)
                elif isinstance(value,bytes):rc=e.bind_blob(self.statement,index,value,len(value))
                elif isinstance(value,str):
                    raw=value.encode();rc=e.bind_text(self.statement,index,raw,len(raw))
                else:raise TypeError('unsupported research SQL parameter')
                if rc:connection.raise_error(rc)
            keyword=sql.lstrip().split(None,1)[0].upper()
            if not e.readonly(self.statement) and keyword not in ('BEGIN','COMMIT','ROLLBACK','SAVEPOINT','RELEASE','PRAGMA'):
                connection.mutations+=1
            self.advance()
        except Exception:self.close();raise

    def advance(self):
        if not self.statement:self.row=None;return
        e=self.connection.engine;rc=e.step(self.statement)
        if rc==101:self.close();self.row=None;return
        if rc!=100:
            try:self.connection.raise_error(rc)
            finally:self.close()
        values=[]
        for index in range(e.columns(self.statement)):
            kind=e.type(self.statement,index)
            if kind==5:value=None
            elif kind==1:value=e.int(self.statement,index)
            elif kind==2:value=e.double(self.statement,index)
            else:
                ptr=(e.text if kind==3 else e.blob)(self.statement,index);size=e.bytes(self.statement,index)
                if e.errcode(self.connection.handle)&255==7:self.connection.raise_error(7)
                if size and not ptr:raise sqlite3.OperationalError('native column materialization failed')
                raw=C.string_at(ptr,size) if size else b'';value=raw.decode() if kind==3 else raw
            values.append(value)
        self.row=tuple(values)
    def fetchone(self):
        row=self.row
        try:
            if row is not None:self.advance()
        except Exception:self.close();raise
        return row
    def fetchall(self):return list(self)
    def __iter__(self):return self
    def __next__(self):
        row=self.fetchone()
        if row is None:raise StopIteration
        return row
    def close(self):
        if self.statement:self.connection.engine.finalize(self.statement);self.statement=P()
    def __del__(self):self.close()
