"""G3 native hierarchy/normalization comparison; no production migration."""
import argparse
import hashlib
import json
from pathlib import Path
import sqlite3
import sys
import time

from research_history_compact import sha,frame
from research_history_growth import GrowthStore
from research_history_hierarchy import HierarchyStore,NormalizedStore,PackedStore
from research_history_index import encoded,distribution
from research_history_layout_measure import pages
from research_history_storage_measure import measure
from research_history_verified_measure import storage,own_memory

CLASSES={'flat':GrowthStore,'hierarchy':HierarchyStore,'normalized':NormalizedStore,'packed':PackedStore}


def factory(candidate):
    def create(*args,**kwargs):return CLASSES[candidate](*args,layout='rowid',**kwargs)
    return create


def logical_manifest(store):
    result={}
    for name,query in [('payload','SELECT hash,kind,raw_length,bytes FROM payload ORDER BY hash'),
                       ('receipt','SELECT * FROM receipt ORDER BY seq')]:
        digest=hashlib.sha256();count=0;logical=0
        for row in store.execute(query):
            raw=frame(row);digest.update(len(raw).to_bytes(8,'little'));digest.update(raw);count+=1;logical+=len(raw)
        result[name]={'rows':count,'sha256':digest.hexdigest(),'canonical_bytes':logical}
    return result


def allocation(path):
    size=path.stat()
    result={'file_bytes':size.st_size}
    if hasattr(size,'st_blocks'):result['allocated_bytes']=size.st_blocks*512
    elif sys.platform=='win32':
        import ctypes
        from ctypes import wintypes
        class Standard(ctypes.Structure):
            _fields_=[('allocation',ctypes.c_longlong),('eof',ctypes.c_longlong),('links',wintypes.DWORD),('deleting',wintypes.BOOLEAN),('directory',wintypes.BOOLEAN)]
        kernel=ctypes.WinDLL('kernel32',use_last_error=True)
        kernel.CreateFileW.argtypes=[wintypes.LPCWSTR,wintypes.DWORD,wintypes.DWORD,ctypes.c_void_p,wintypes.DWORD,wintypes.DWORD,wintypes.HANDLE]
        kernel.CreateFileW.restype=wintypes.HANDLE
        kernel.GetFileInformationByHandleEx.argtypes=[wintypes.HANDLE,ctypes.c_int,ctypes.c_void_p,wintypes.DWORD]
        kernel.CloseHandle.argtypes=[wintypes.HANDLE]
        handle=kernel.CreateFileW(str(path),0,7,None,3,0,None)
        if handle==wintypes.HANDLE(-1).value:raise ctypes.WinError(ctypes.get_last_error())
        try:
            info=Standard()
            if not kernel.GetFileInformationByHandleEx(handle,1,ctypes.byref(info),ctypes.sizeof(info)):raise ctypes.WinError(ctypes.get_last_error())
            result['allocated_bytes']=info.allocation
        finally:kernel.CloseHandle(handle)
    return result


def detail(s,path):
    return {'canonical_manifest':logical_manifest(s),'pages':pages(s.db),'allocation':allocation(path),
            'receipt_fields':dict(zip(['id','intent','body','hash'],s.db.execute(
                'SELECT sum(length(id)),sum(length(intent)),sum(length(body)),sum(length(hash)) FROM receipt').fetchone())),
            'payload_reference_hash_bytes':s.db.execute('SELECT sum(length(hash)) FROM payload').fetchone()[0],
            'index_rows':sum(1 for _ in s.execute('SELECT id FROM bucket')),
            'metrics':dict(s.metrics),'cache_note':'SQLite cache_size=-2048 KiB; Python does not expose SQLite cache-miss/high-water counters. Logical authenticated-node reads are measured, not physical cache misses.'}


def replay(fixture,output,candidate):
    make=factory(candidate);measure(fixture,output,'buckets',fullfsync=sys.platform=='darwin',store_class=make)
    report=json.loads((output/'summary.json').read_text());s=make(output/'history.sqlite',bytes.fromhex(report['instance']),'buckets',fullfsync=sys.platform=='darwin')
    s.open_verified();s.execute('PRAGMA wal_checkpoint(TRUNCATE)')
    report.update(candidate=candidate,final_detail=detail(s,output/'history.sqlite'));s.close()
    report['limitations']=[v for v in report['limitations'] if not v.startswith(('fixed 256','plain ingestion','plain SQL'))]
    report['limitations'].append('1024-leaf, depth-24, eight-split research admission bounds; growth is not unbounded production qualification')
    report['limitations'].append('logical authenticated-node access is measured; SQLite physical cache misses and device writes are not instrumented')
    events=[json.loads(line) for line in (output/'transactions.jsonl').read_text().splitlines()]
    report['logical_publication']={field:distribution([e[field] for e in events]) for field in
        ['index_pages_read','index_pages_written','proof_read_bytes','proof_written_bytes']}
    report['split_publication_ns']=distribution([e['backend_ns'] for e in events if e['splits']])
    report['non_split_publication_ns']=distribution([e['backend_ns'] for e in events if not e['splits']])
    (output/'summary.json').write_text(json.dumps(report,indent=2)+'\n')


def stress(fixture,output,candidate,count=16384):
    if candidate=='flat' or count>16384:raise ValueError('hierarchy-only bounded control probe')
    output.mkdir(parents=True,exist_ok=False)
    with fixture.open() as stream:seed=json.loads(next(stream))
    instance=sha(encoded(seed['records']));make=factory(candidate);path=output/'index.sqlite'
    s=make(path,instance,'buckets',fullfsync=sys.platform=='darwin');s.create(seed);times=[];samples=[]
    start_memory=own_memory()
    with (output/'publications.jsonl').open('w') as stream:
        for i in range(count):
            before=dict(s.metrics);start=time.perf_counter_ns();s.reconcile(s.txid('index-only-'+str(i)),sha(b'index-only-intent'))
            event={'ordinal':i+1,'ns':time.perf_counter_ns()-start,'leaves':s.leaves}
            for field in ['split_count','index_pages_read','index_pages_written','proof_read_bytes','proof_written_bytes']:
                event[field]=s.metrics[field]-before[field]
            stream.write(json.dumps(event)+'\n');times.append(event['ns']);samples.append(event)
    assert s.leaves>256
    audit=s.audit();s.execute('PRAGMA wal_checkpoint(TRUNCATE)');report={
        'candidate':candidate,'controls':count,'not_g2_edits':True,'leaves':s.leaves,'generation':s.generation,
        'storage':storage(s),'detail':detail(s,path),'publication_ns':distribution(times),'audit':audit,
        'memory_start':start_memory,'memory_final':own_memory(),
        'logical_publication':{field:distribution([e[field] for e in samples]) for field in
            ['index_pages_read','index_pages_written','proof_read_bytes','proof_written_bytes']}}
    s.close();s=make(path,instance,'buckets',fullfsync=sys.platform=='darwin');report['reopen']=s.open_verified()
    assert s.reconcile(s.txid('index-only-'+str(count-1)),sha(b'index-only-intent'))['outcome']==2
    s.close();(output/'summary.json').write_text(json.dumps(report,indent=2)+'\n')


if __name__=='__main__':
    parser=argparse.ArgumentParser(description=__doc__);parser.add_argument('action',choices=['replay','stress'])
    parser.add_argument('fixture',type=Path);parser.add_argument('output',type=Path)
    parser.add_argument('--candidate',choices=list(CLASSES),required=True);args=parser.parse_args()
    (replay if args.action=='replay' else stress)(args.fixture,args.output,args.candidate)
