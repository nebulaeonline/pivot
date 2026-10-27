"""Physical copies, actual G2 replays and a narrowly bounded index-only stress probe."""
import argparse
import json
from pathlib import Path
import sqlite3
import sys
import time

from research_history_compact import sha,frame
from research_history_layout import LayoutStore,manifest
from research_history_growth import GrowthStore
from research_history_index import distribution,encoded
from research_history_storage_measure import measure
from research_history_verified_measure import storage,own_memory


def pages(db):
    report={'page_size':db.execute('PRAGMA page_size').fetchone()[0],
            'page_count':db.execute('PRAGMA page_count').fetchone()[0],
            'freelist_count':db.execute('PRAGMA freelist_count').fetchone()[0]}
    try:
        report['objects']={row[0]:{'page_bytes':row[1],'used_payload_bytes':row[2],'unused_bytes':row[3],
            'overflow_pages':row[4],'pages':row[5]} for row in db.execute(
            "SELECT name,sum(pgsize),sum(payload),sum(unused),sum(pagetype='overflow'),count(*) FROM dbstat GROUP BY name")}
        report['residual']=report['page_count']*report['page_size']-sum(v['page_bytes'] for v in report['objects'].values())-report['freelist_count']*report['page_size']
    except sqlite3.DatabaseError as error:report['dbstat_unavailable']=str(error)
    return report


def physical(source,output,instance):
    output.mkdir(parents=True,exist_ok=False)
    origin=sqlite3.connect(source.resolve().as_uri()+'?mode=ro',uri=True);origin.execute('BEGIN')
    reference=manifest(origin);result={'source_manifest':reference,'candidates':{}}
    try:
        for layout in ['without','rowid','local','partial']:
            path=output/(layout+'.sqlite');s=LayoutStore(path,instance,'buckets',layout=layout,fullfsync=sys.platform=='darwin')
            try:
                s.db.executescript(s.schema_sql);s.db.execute('BEGIN IMMEDIATE')
                for table,columns,order in [
                    ('payload','hash,kind,raw_length,bytes','hash'),('receipt','seq,id,intent,kind,edit,body,hash','seq'),
                    ('bucket','id,bytes','id'),('authority','singleton,body,hash','singleton')]:
                    count=len(columns.split(','));cursor=origin.execute('SELECT '+columns+' FROM '+table+' ORDER BY '+order)
                    while True:
                        batch=cursor.fetchmany(256)
                        if not batch:break
                        s.db.executemany('INSERT INTO '+table+'('+columns+') VALUES ('+','.join(['?']*count)+')',batch)
                s.db.execute('COMMIT');actual=manifest(s.db)
                if actual!=reference:raise AssertionError('layout changed authoritative bytes')
                opened=s.open_verified();audit=s.audit();s.db.execute('PRAGMA wal_checkpoint(TRUNCATE)')
                result['candidates'][layout]={'manifest':actual,'pages':pages(s.db),'storage':storage(s),'open':opened,'audit':audit,
                    'query_plans':{name:s.db.execute('EXPLAIN QUERY PLAN '+sql,params).fetchall() for name,sql,params in [
                        ('payload','SELECT bytes FROM payload WHERE hash=?',(b'0'*32,)),
                        ('transaction','SELECT * FROM receipt WHERE id=?',(b'0'*16,)),
                        ('state','SELECT * FROM receipt WHERE edit=?',(10000,))]}}
            finally:s.close()
    finally:origin.close()
    (output/'summary.json').write_text(json.dumps(result,indent=2)+'\n',encoding='utf-8')


def replay(fixture,output,layout,growth):
    def factory(*args,**kwargs):return (GrowthStore if growth else LayoutStore)(*args,layout=layout,**kwargs)
    measure(fixture,output,'buckets',fullfsync=sys.platform=='darwin',store_class=factory)
    report=json.loads((output/'summary.json').read_text())
    db=sqlite3.connect((output/'history.sqlite').resolve().as_uri()+'?mode=ro',uri=True)
    report.update(layout=layout,growth=growth,manifest=manifest(db),page_accounting=pages(db));db.close()
    split=[];ordinary=[]
    with (output/'transactions.jsonl').open() as stream:
        for line in stream:
            row=json.loads(line);(split if row['splits'] else ordinary).append(row['backend_ns'])
    report['split_publication_ns']=distribution(split);report['non_split_publication_ns']=distribution(ordinary)
    report['split_publication_samples']=len(split)
    (output/'summary.json').write_text(json.dumps(report,indent=2)+'\n',encoding='utf-8')


def stress(fixture,output,count):
    if count>16384:raise ValueError('bounded index-only probe; not a 100K G2 run')
    output.mkdir(parents=True,exist_ok=False)
    with fixture.open() as stream:seed=json.loads(next(stream))
    instance=sha(encoded(seed['records']));s=GrowthStore(output/'index.sqlite',instance,'buckets',layout='rowid',fullfsync=sys.platform=='darwin')
    s.create(seed);timings=[];splits=[];start_memory=own_memory()
    with (output/'publications.jsonl').open('w') as raw:
        for i in range(count):
            before=s.metrics['split_count'];started=time.perf_counter_ns()
            answer=s.reconcile(s.txid('index-only-'+str(i)),sha(b'index-only-intent'))
            elapsed=time.perf_counter_ns()-started;delta=s.metrics['split_count']-before
            assert answer['outcome']==2
            raw.write(json.dumps({'ordinal':i+1,'ns':elapsed,'splits':delta,'leaves':len(s.directory),'directory_bytes':len(s.meta[8])})+'\n')
            timings.append(elapsed)
            if delta:splits.append(elapsed)
    assert len(s.directory)>256,'stress did not exercise growth beyond fixed prototype'
    s.audit();s.db.execute('PRAGMA wal_checkpoint(TRUNCATE)')
    report={'kind':'index-only terminal-control probe, NOT G2 workload or production scalability',
            'controls':count,'receipts':s.db.execute('SELECT count(*) FROM receipt').fetchone()[0],
            'leaves':len(s.directory),'generation':s.generation,'metrics':dict(s.metrics),
            'publication_ns':distribution(timings),'split_publication_ns':distribution(splits),
            'storage':storage(s),'pages':pages(s.db),'memory_start':start_memory,'memory_final':own_memory()}
    s.close();s=GrowthStore(output/'index.sqlite',instance,'buckets',layout='rowid',fullfsync=sys.platform=='darwin')
    report['reopen']=s.open_verified()
    assert s.reconcile(s.txid('index-only-'+str(count-1)),sha(b'index-only-intent'))['outcome']==2
    s.close();(output/'summary.json').write_text(json.dumps(report,indent=2)+'\n',encoding='utf-8')


if __name__=='__main__':
    p=argparse.ArgumentParser(description=__doc__);p.add_argument('action',choices=['physical','replay','stress'])
    p.add_argument('fixture',type=Path);p.add_argument('output',type=Path);p.add_argument('--source',type=Path)
    p.add_argument('--layout',choices=['without','rowid','local','partial'],default='rowid');p.add_argument('--growth',action='store_true')
    args=p.parse_args()
    if args.action=='physical':
        with args.fixture.open() as f:seed=json.loads(next(f))
        physical(args.source,args.output,sha(encoded(seed['records'])))
    elif args.action=='replay':replay(args.fixture,args.output,args.layout,args.growth)
    else:stress(args.fixture,args.output,16384)
