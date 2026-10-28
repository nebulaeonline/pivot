"""Small-bound actual-connection SQLite qualification; no 32K/65K execution."""
import json
from pathlib import Path
import sys

from research_history_engine import EngineStore
from research_history_storage_measure import measure
from research_history_hierarchy_measure import detail
from research_sqlite_native import NativeEngine


def run(fixture,output):
    engine=NativeEngine();events=[];identities=[];scratch=[]
    # Stream native events outside the SQLite writer; no retained sample arrays.
    output.parent.mkdir(parents=True,exist_ok=True)
    eventfile=output.parent/(output.name+'-native.jsonl')
    with eventfile.open('w') as stream:
        def factory(*args,**kwargs):
            store=EngineStore(*args,**kwargs);identities.append(store.db.identity())
            store.telemetry_sink=lambda event:stream.write(json.dumps(event)+'\n')
            return store
        measure(fixture,output,'buckets',fullfsync=sys.platform=='darwin',store_class=factory)
    report=json.loads((output/'summary.json').read_text())
    store=EngineStore(output/'history.sqlite',bytes.fromhex(report['instance']),'buckets',fullfsync=sys.platform=='darwin')
    report['engine_identity']=store.db.identity();store.open_verified();store.db.execute('PRAGMA wal_checkpoint(TRUNCATE)').fetchall()
    report['final_detail']=detail(store,output/'history.sqlite');report['native_telemetry_final']=engine.telemetry(store.db)
    store.close();report['engine_identities_equal']=all(i==identities[0] for i in identities)
    report['limitations']=[s for s in report['limitations'] if not s.startswith(('fixed 256','plain ingestion','plain SQL','old Windows/Linux'))]
    report['limitations'] += [
        'fixed official SQLite 3.51.3 plus source-pinned delegating VFS shim; installed platform runtimes unchanged',
        'native heap has a 64 MiB library cap and prewrite headroom check; not a complete Python or physical transaction reservation',
        'SQL-step dirty-list samples exclude within-statement peaks; VFS write calls/frame-header events include attempted/uncommitted writes',
        'VFS xSync success/time is not proof of host/hypervisor/device durability; allocator/VFS counters are process-global to the private library',
        '1024-leaf runtime retained; 64 MiB WAL trigger may overshoot by one transaction; full 32K/65K scaling not authorized']
    (output/'summary.json').write_text(json.dumps(report,indent=2)+'\n')


if __name__=='__main__':run(Path(sys.argv[1]),Path(sys.argv[2]))
