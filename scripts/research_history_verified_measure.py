"""Measured G3 bounded-open experiment; actual SQLite databases on native storage."""
import argparse
import ctypes
import json
import os
from pathlib import Path
import platform
import queue
import shutil
import sqlite3
import subprocess
import sys
import threading
import time
import tracemalloc

from research_history_index import encoded,digest,distribution,Backpressure
from research_history_verified import VerifiedStore


def own_memory():
    if os.name=='nt':
        class Counters(ctypes.Structure):
            _fields_=[('cb',ctypes.c_ulong),('faults',ctypes.c_ulong)]+[(name,ctypes.c_size_t) for name in
                ['peak_ws','ws','peak_paged','paged','peak_nonpaged','nonpaged','pagefile','peak_pagefile','private']]
        counters=Counters();counters.cb=ctypes.sizeof(counters)
        kernel=ctypes.WinDLL('kernel32',use_last_error=True)
        kernel.GetCurrentProcess.restype=ctypes.c_void_p
        psapi=ctypes.WinDLL('psapi',use_last_error=True)
        psapi.GetProcessMemoryInfo.argtypes=[ctypes.c_void_p,ctypes.POINTER(Counters),ctypes.c_ulong]
        if not psapi.GetProcessMemoryInfo(kernel.GetCurrentProcess(),ctypes.byref(counters),counters.cb):
            return {'unavailable':ctypes.get_last_error()}
        return {'rss_bytes':counters.ws,'peak_rss_bytes':counters.peak_ws,'private_bytes':counters.private,
                'page_faults':counters.faults}
    import resource
    current=subprocess.check_output(['/bin/ps','-o','rss=','-p',str(os.getpid())],text=True).strip()
    usage=resource.getrusage(resource.RUSAGE_SELF)
    result={'rss_bytes':int(current)*1024,
            'peak_rss_bytes':int(usage.ru_maxrss)*(1 if sys.platform=='darwin' else 1024),
            'minor_faults':usage.ru_minflt,'major_faults':usage.ru_majflt}
    if sys.platform.startswith('linux'):
        # Own process only; no unrelated process observation.
        result['own_io']={k:int(v) for k,v in (line.split(':') for line in Path('/proc/self/io').read_text().splitlines())}
    return result


def storage(store):
    path=store.path
    result={'db_bytes':path.stat().st_size,'wal_bytes':store.wal_bytes(),
            'db_pages':store.db.execute('PRAGMA page_count').fetchone()[0],
            'page_size':store.db.execute('PRAGMA page_size').fetchone()[0]}
    result['allocated_db_bytes']=getattr(path.stat(),'st_blocks',0)*512 if hasattr(path.stat(),'st_blocks') else None
    result['allocated_wal_bytes']=None
    wal=Path(str(path)+'-wal')
    if wal.exists() and hasattr(wal.stat(),'st_blocks'):result['allocated_wal_bytes']=wal.stat().st_blocks*512
    try:result['page_bytes_by_object']={r[0]:r[1] for r in store.db.execute('SELECT name,sum(pgsize) FROM dbstat GROUP BY name')}
    except sqlite3.DatabaseError as error:result['dbstat_unavailable']=str(error)
    return result


def run(fixture,output,*,fullfsync=False):
    output.mkdir(parents=True,exist_ok=False)
    if shutil.disk_usage(output).free<max(fixture.stat().st_size*12,1024*1024*1024):
        raise Backpressure('insufficient native research disk reserve')
    with fixture.open(encoding='utf-8') as stream:seed=json.loads(next(stream))
    instance=digest(encoded(seed['records']))
    path=output/'history.sqlite'
    store=VerifiedStore(path,instance,fullfsync=fullfsync)
    report={'status':'running','research_only':True,'platform':platform.platform(),'python':platform.python_version(),
            'sqlite':sqlite3.sqlite_version,'sqlite_source_id':store.db.execute('SELECT sqlite_source_id()').fetchone()[0],
            'compile_options':[r[0] for r in store.db.execute('PRAGMA compile_options')],
            'vfs':'platform default; actual VFS identity not exposed by Python sqlite3',
            'sync_syscalls':'not instrumented; COMMIT includes backend sync wall time only',
            'physical_sqlite_page_access':'not exposed; SQL/proof work and own-process faults/IO are recorded',
            'memory_before':own_memory(),'sizes':{},'fullfsync_requested':fullfsync}
    commit=[];publication=[];ack=[];cases={};scratch={}
    with fixture.open(encoding='utf-8') as stream, (output/'transactions.jsonl').open('w',encoding='utf-8') as raw:
        store.create(json.loads(next(stream)))
        n=0
        for line in stream:
            item=json.loads(line);started=time.perf_counter_ns()
            receipt=store.publish(item);elapsed=time.perf_counter_ns()-started
            record={**receipt,'publication_ns':elapsed,'case':item['case']}
            raw.write(json.dumps(record)+'\n');n+=1
            commit.append(receipt['commit_ns']);publication.append(elapsed);ack.append(receipt['ack_ns'])
            cases.setdefault(item['case'],[]).append(elapsed)
    report.update(transactions=n,memory_after_publication=own_memory(),
                  configuration={key:store.db.execute('PRAGMA '+key).fetchone()[0] for key in
                     ['journal_mode','synchronous','fullfsync','checkpoint_fullfsync','cache_size','mmap_size','wal_autocheckpoint','page_size']})
    report['sizes']['pre_checkpoint']=storage(store)
    started=time.perf_counter_ns()
    report['passive_checkpoint']=list(store.db.execute('PRAGMA wal_checkpoint(PASSIVE)').fetchone())
    report['passive_checkpoint_ns']=time.perf_counter_ns()-started
    report['sizes']['post_passive']=storage(store)
    store.close()
    opens=[]
    for repeat in range(5):
        started=time.perf_counter_ns()
        store=VerifiedStore(path,instance,fullfsync=fullfsync)
        connected=time.perf_counter_ns()-started
        opened=store.verified_open()
        opened.update(constructor_ns=connected,sqlite_connect_ns=store.connect_ns)
        if opened['historical_decision_scan'] or opened['proof_nodes_read']>257*70:
            raise AssertionError('bounded logical open gate failed')
        opens.append(opened);store.close()
    report['normal_open_samples']=opens
    # A separate traced activation measures Python scratch, excluded from latency samples.
    tracemalloc.start()
    store=VerifiedStore(path,instance,fullfsync=fullfsync);store.verified_open()
    scratch['python_current_bytes'],scratch['python_peak_bytes']=tracemalloc.get_traced_memory()
    tracemalloc.stop()
    report['activation_scratch']=scratch
    report['memory_after_open']=own_memory()
    positives=[];negatives=[];lookups=[]
    for index in range(100):
        tx,intent=store.db.execute('SELECT tx,intent FROM decision WHERE seq=?',(1+index*n//100,)).fetchone()
        proof=store.proof_reads;sql=store.statement_count;started=time.perf_counter_ns()
        answer=store.reconcile(tx,intent)
        assert answer['outcome']=='committed'
        positives.append(time.perf_counter_ns()-started)
        lookups.append({'kind':'positive','proof_nodes':store.proof_reads-proof,'sql':store.statement_count-sql})
        proof=store.proof_reads;sql=store.statement_count;started=time.perf_counter_ns()
        assert store.reconcile('missing-'+str(index),'fixed-intent')['outcome']=='uncommitted'
        negatives.append(time.perf_counter_ns()-started)
        lookups.append({'kind':'terminal-negative','proof_nodes':store.proof_reads-proof,'sql':store.statement_count-sql})
    (output/'lookup-work.json').write_text(json.dumps(lookups,indent=2),encoding='utf-8')
    cold=[]
    for index in sorted({0,n//2,n}):
        proof=store.proof_reads;decoded=store.payload_bytes;started=time.perf_counter_ns()
        store.cold(index)
        cold.append({'state':index,'ns':time.perf_counter_ns()-started,'proof_nodes':store.proof_reads-proof,
                     'payload_bytes_decoded':store.payload_bytes-decoded,'delta_steps':index%64})
    report['cold']=cold
    # A deliberately held read snapshot tests checkpoint refusal, not a read lease policy.
    store.db.execute('PRAGMA wal_checkpoint(TRUNCATE)')
    reader=sqlite3.connect(path,isolation_level=None)
    reader.execute('PRAGMA wal_autocheckpoint=0');reader.execute('BEGIN')
    reader.execute('SELECT * FROM authority').fetchall()
    held=time.perf_counter_ns()
    store.reconcile('reader-probe','intent')
    old_limit=store.wal_limit;store.wal_limit=1
    before=store.authority();started=time.perf_counter_ns()
    try:store.reconcile('blocked-reader-probe','intent')
    except Backpressure:pass
    else:raise AssertionError('pinned WAL did not cause backpressure')
    report['reader_checkpoint_block_ns']=time.perf_counter_ns()-started
    assert store.authority()==before
    reader.execute('ROLLBACK');reader.close()
    report['reader_held_ns']=time.perf_counter_ns()-held
    started=time.perf_counter_ns()
    store.reconcile('blocked-reader-probe','intent')
    report['reader_release_admission_ns']=time.perf_counter_ns()-started
    store.wal_limit=old_limit
    # Full audit is explicitly measured maintenance, not hidden inside normal open.
    started=time.perf_counter_ns();report['full_audit']=store.audit()
    report['full_audit_wall_ns']=time.perf_counter_ns()-started
    report['sizes']['post_queries_audit']=storage(store)
    report['query_plans']={name:store.db.execute('EXPLAIN QUERY PLAN '+sql,params).fetchall() for name,sql,params in [
        ('receipt','SELECT * FROM decision WHERE tx=?',('edit-1',)),
        ('proof','SELECT bytes FROM proof_node WHERE hash=?',(store.root,)),
        ('state','SELECT * FROM state WHERE id=?',(n,))]}
    report['vm_progress_callbacks_1000']=store.vm_steps_1000
    store.db.execute('PRAGMA wal_checkpoint(TRUNCATE)');store.close()
    # Real termination after COMMIT leaves a small, committed WAL recovery envelope.
    code='''
import sys,time
from pathlib import Path
sys.path.insert(0,sys.argv[1])
from research_history_verified import VerifiedStore
s=VerifiedStore(Path(sys.argv[2]),sys.argv[3],fullfsync=sys.argv[4]=='1');s.verified_open()
def pause(point):
 if point=='after_commit':print('READY',flush=True);time.sleep(30)
a=s.begin()
s.decide('recovery-probe','intent','uncommitted',{'head':a[2],'cursor':a[3],'redo':a[4],'ledger':a[5],'state':None},a,fault=pause)
'''
    child=subprocess.Popen([sys.executable,'-c',code,str(Path(__file__).parent),str(path),instance,str(int(fullfsync))],
                           stdout=subprocess.PIPE,stderr=subprocess.PIPE,text=True)
    messages=queue.Queue()
    threading.Thread(target=lambda:messages.put(child.stdout.readline()),daemon=True).start()
    try:assert messages.get(timeout=30).strip()=='READY'
    finally:child.kill();out,err=child.communicate(timeout=10)
    assert child.returncode!=0,(out,err)
    started=time.perf_counter_ns();store=VerifiedStore(path,instance,fullfsync=fullfsync)
    recovery_constructor=time.perf_counter_ns()-started
    recovery=store.verified_open();recovery['constructor_ns']=recovery_constructor
    assert store.reconcile('recovery-probe','intent')['outcome']=='uncommitted'
    report.update(recovery_open=recovery,memory_final=own_memory(),status='passed',
                  publication_ns=distribution(publication),commit_ns=distribution(commit),ack_ns=distribution(ack),
                  positive_ns=distribution(positives),terminal_negative_ns=distribution(negatives),
                  publication_by_case_ns={case:distribution(values) for case,values in cases.items()})
    report['limitations']=['single-run timing; different engines and hosts',
        'bounded authenticated key probes; SQLite B-tree physical page/IO costs not directly measured',
        'current root/atomic-writer correctness and collision resistance assumed; coherent root rewrite/rollback not detected',
        'WAL bytes capped before SQLite recovery; oversize/incomplete tail requires explicit maintenance',
        'no production cache/domain/RSS admission or general reader leases',
        'unreachable historical proof nodes retained; no GC/retention implementation',
        'fullfsync PRAGMA requested/read back; VFS syscall behavior and power-loss durability not established',
        'older Windows/Linux SQLite builds have unqualified WAL-reset fix status; no competing checkpoint writers tested']
    (output/'summary.json').write_text(json.dumps(report,indent=2)+'\n',encoding='utf-8')
    store.close()
    print(json.dumps({'status':'passed','transactions':n,'opens':opens,'audit_ns':report['full_audit_wall_ns']}),flush=True)
    return report


if __name__=='__main__':
    parser=argparse.ArgumentParser(description=__doc__)
    parser.add_argument('fixture',type=Path);parser.add_argument('output',type=Path)
    parser.add_argument('--fullfsync',action='store_true')
    args=parser.parse_args()
    run(args.fixture,args.output,fullfsync=args.fullfsync)
