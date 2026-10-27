"""Controlled compact SQLite comparison; no production compression selection."""
import argparse
import json
from pathlib import Path
import platform
import shutil
import sqlite3
import time
import tracemalloc
import zlib

from research_history_compact import CompactStore,sha,pack_map
from research_history_index import Unknown,encoded,distribution,checkpoint_bytes
from research_history_verified_measure import own_memory,storage


def measure(fixture,output,mode,*,fullfsync=False,store_class=CompactStore):
    output.mkdir(parents=True,exist_ok=False)
    if shutil.disk_usage(output).free<max(fixture.stat().st_size*4,512*1024*1024):raise RuntimeError('insufficient native research reserve')
    with fixture.open(encoding='utf-8') as stream:seed=json.loads(next(stream))
    instance=sha(encoded(seed['records']));path=output/'history.sqlite'
    store=store_class(path,instance,mode,fullfsync=fullfsync);store.create(seed);records=dict(seed['records'])
    report={'mode':mode,'sqlite':sqlite3.sqlite_version,'sqlite_source_id':store.db.execute('SELECT sqlite_source_id()').fetchone()[0],
            'compile_options':[r[0] for r in store.db.execute('PRAGMA compile_options')],
            'zlib_runtime':zlib.ZLIB_RUNTIME_VERSION,'zlib_level':6,'compression_policy':'research baseline; no production codec selected',
            'platform':platform.platform(),'memory_start':own_memory(),'instance':instance.hex()}
    samples={k:[] for k in ['oracle_ns','prepare_ns','backend_ns','proof_ns','sql_noncommit_ns','commit_ns','other_backend_ns']}
    cells=0;raw_deltas=0;compressed_deltas=0
    with fixture.open(encoding='utf-8') as stream,(output/'transactions.jsonl').open('w',encoding='utf-8') as raw:
        next(stream);n=0
        for line in stream:
            item=json.loads(line);prepared=store.prepare(item,records)
            before=dict(store.metrics);started=time.perf_counter_ns();receipt=store.publish(prepared,fixture_fresh=True);backend=time.perf_counter_ns()-started
            proof=store.metrics['proof_ns']-before['proof_ns'];sql=store.metrics['sql_ns']-before['sql_ns']-receipt['commit_ns']
            event={'seq':item['seq'],'case':item['case'],'oracle_ns':prepared['oracle_ns'],
                   'prepare_ns':prepared['prepare_ns'],'backend_ns':backend,'proof_ns':proof,
                   'sql_noncommit_ns':sql,'commit_ns':receipt['commit_ns'],'other_backend_ns':backend-proof-sql-receipt['commit_ns']}
            event['splits']=store.metrics.get('split_count',0)-before.get('split_count',0)
            event['directory_bytes']=len(store.meta[8])
            for metric in ['index_pages_read','index_pages_written','proof_read_bytes','proof_written_bytes']:
                event[metric]=store.metrics.get(metric,0)-before.get(metric,0)
            for key in samples:samples[key].append(event[key])
            raw.write(json.dumps(event)+'\n');records=prepared['records'];n+=1
            cells+=sum(bytes.fromhex(key)[0]==16 for key,_,_ in item['ops'])
            raw_deltas+=prepared['blobs'][0][2];compressed_deltas+=len(prepared['blobs'][0][3])
    report.update(transactions=n,changed_cell_records=cells,raw_delta_bytes=raw_deltas,compressed_delta_bytes=compressed_deltas,
                  timings_ns={key:distribution(values) for key,values in samples.items()},memory_after_ingest=own_memory(),
                  proof_metrics=dict(store.metrics),configuration={p:store.db.execute('PRAGMA '+p).fetchone()[0] for p in
                     ['journal_mode','synchronous','fullfsync','checkpoint_fullfsync','cache_size','mmap_size','wal_autocheckpoint','page_size']})
    report['pre_checkpoint']=storage(store)
    start=time.perf_counter_ns();report['checkpoint']=list(store.db.execute('PRAGMA wal_checkpoint(TRUNCATE)').fetchone())
    report['checkpoint_ns']=time.perf_counter_ns()-start
    report['closed_workload_storage']=storage(store)
    # Keep a separately loadable current-content cold image for real ordinary WB5 export.
    cold=store.cold(n);(output/'current.cold').write_bytes(checkpoint_bytes(cold))
    assert cold==records
    categories={}
    for kind,count,raw_bytes,compressed in store.db.execute('SELECT kind,count(*),sum(raw_length),sum(length(bytes)) FROM payload GROUP BY kind'):
        categories[str(kind)]={'count':count,'binary_bytes':raw_bytes,'compressed_bytes':compressed}
    report['payload_categories']=categories
    logical=sum(v['compressed_bytes'] for v in categories.values())
    binary=sum(v['binary_bytes'] for v in categories.values())
    directory=len(store.meta[8]);bucket_raw=store.db.execute('SELECT coalesce(sum(length(bytes)),0) FROM bucket').fetchone()[0]
    if hasattr(store,'authentication_extra_bytes'):bucket_raw+=store.authentication_extra_bytes()
    report['authentication_bytes']=bucket_raw+directory
    report['receipt_body_bytes']=store.db.execute('SELECT sum(length(id)+length(intent)+length(body)+length(hash)) FROM receipt').fetchone()[0]
    report['retained_compressed_payload_bytes']=logical;report['retained_binary_payload_bytes']=binary
    report['current_cold_bytes']=len(checkpoint_bytes(cold));report['current_binary_record_bytes']=len(pack_map(cold))
    report['db_bytes_per_transaction']=report['closed_workload_storage']['db_pages']*report['closed_workload_storage']['page_size']/n
    report['db_bytes_per_changed_cell_record']=report['closed_workload_storage']['db_pages']*report['closed_workload_storage']['page_size']/cells
    report['authentication_over_compressed_payload']=report['authentication_bytes']/logical
    report['authentication_over_binary_payload']=report['authentication_bytes']/binary
    objects=report['closed_workload_storage'].get('page_bytes_by_object')
    if objects:
        metadata=sum(objects.values())-objects.get('payload',0)
        report['physical_metadata_bytes']=metadata
        report['all_metadata_over_compressed_payload']=metadata/logical
        report['all_metadata_over_binary_payload']=metadata/binary
    report['database_minus_compressed_payload']=report['closed_workload_storage']['db_bytes']-logical
    report['bytes_note']='logical binary and compressed payloads include deltas/checkpoints/complete-ledger blobs; DB page overhead and metadata are separate'
    store.close()
    opens=[]
    for _ in range(5):
        started=time.perf_counter_ns();store=store_class(path,instance,mode,fullfsync=fullfsync);ctor=time.perf_counter_ns()-started
        result=store.open_verified();result['constructor_ns']=ctor;opens.append(result);store.close()
    report['open_samples']=opens
    tracemalloc.start();store=store_class(path,instance,mode,fullfsync=fullfsync);store.open_verified()
    report['activation_python_current_bytes'],report['activation_python_peak_bytes']=tracemalloc.get_traced_memory();tracemalloc.stop()
    positive=[];negative=[];unknowns=0
    for index in range(30):
        txid,intent=store.db.execute('SELECT id,intent FROM receipt WHERE edit=?',(1+index*n//30,)).fetchone()
        started=time.perf_counter_ns();answer=store.reconcile(txid,intent);positive.append(time.perf_counter_ns()-started)
        assert answer['outcome']==1
        started=time.perf_counter_ns()
        try:
            answer=store.reconcile(store.txid('negative-'+str(index)),sha(b'intent'))
            assert mode=='buckets' and answer['outcome']==2
        except Unknown:
            assert mode=='plain';unknowns+=1
        negative.append(time.perf_counter_ns()-started)
        if mode=='plain':store.open_verified()
    report['positive_ns']=distribution(positive);report['negative_ns']=distribution(negative);report['negative_unknown_count']=unknowns
    # Plain mode can terminalize only with an explicit complete audit under the writer lock.
    if mode=='plain':
        started=time.perf_counter_ns();answer=store.reconcile(store.txid('audited-negative'),sha(b'intent'),full_audit=True)
        assert answer['outcome']==2;report['explicit_audited_negative_ns']=time.perf_counter_ns()-started
    cold_samples=[]
    for edit in [0,n//2,n]:
        started=time.perf_counter_ns();store.cold(edit);cold_samples.append({'edit':edit,'ns':time.perf_counter_ns()-started,'deltas':edit%64})
    report['cold_samples']=cold_samples
    life=store.meta[9];store.move(n-1,'undo');store.move(n,'redo');assert store.meta[9]==life
    report['audit']=store.audit()
    report['final_storage']=storage(store);report['memory_final']=own_memory()
    report['status']='passed';report['research_targets']={'under_25MB_history_db':report['closed_workload_storage']['db_bytes']<25_000_000,
        'auth_only_under_20pct_compressed':report['authentication_over_compressed_payload']<.2,
        'all_metadata_under_20pct_compressed':None if 'all_metadata_over_compressed_payload' not in report else report['all_metadata_over_compressed_payload']<.2,
        'note':'report both compressed and binary denominators; do not substitute the favorable one'}
    report['limitations']=['fixed 256 buckets with a 64 KiB bucket research resource bound; overflow blocks rather than prunes',
        'plain ingestion times use the externally known-fresh fixture stream; general unknown-ID admission requires full audit',
        'plain SQL cannot authenticate negative lookup without full audit; no equivalent integrity claim',
        'same SQLite atomic-writer/collision/current-publication trust boundary as accepted experiment; no coherent rollback protection',
        'SQL API wall timer excludes cursor fetch overhead; residual includes Python/hash/driver work; sync syscalls not isolated',
        'configured cache is not measured native cache ownership; RSS includes measurement arrays/OS-cache uncertainty',
        'single-run/warm-cache samples; no production compression, backend, retention or index decision',
        'old Windows/Linux SQLite WAL-reset fix qualification remains open']
    (output/'summary.json').write_text(json.dumps(report,indent=2)+'\n',encoding='utf-8');store.close()
    print(json.dumps({'status':'passed','mode':mode,'transactions':n,'db_bytes':report['closed_workload_storage']['db_bytes'],
                      'compressed_payload_bytes':logical,'auth_bytes':report['authentication_bytes']}),flush=True)


if __name__=='__main__':
    parser=argparse.ArgumentParser(description=__doc__)
    parser.add_argument('fixture',type=Path);parser.add_argument('output',type=Path)
    parser.add_argument('--mode',choices=['plain','buckets'],required=True);parser.add_argument('--fullfsync',action='store_true')
    args=parser.parse_args();measure(args.fixture,args.output,args.mode,fullfsync=args.fullfsync)
