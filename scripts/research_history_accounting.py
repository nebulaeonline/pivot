"""Read-only G3 record/page accounting; no schema, codec or admission changes."""
import argparse
import hashlib
import json
from pathlib import Path
import sqlite3
import sys


def uint_bytes(value):
    if value<0:raise ValueError('unsigned value')
    return max(1,(value.bit_length()+6)//7) if value<(1<<56) else 9


def serial(value):
    if value is None:return 0,0
    if isinstance(value,int):
        if value in (0,1):return 8+value,0
        for code,width in [(1,1),(2,2),(3,3),(4,4),(5,6),(6,8)]:
            if -(1<<(width*8-1))<=value<(1<<(width*8-1)):return code,width
        raise ValueError('outside SQLite signed integer range')
    if isinstance(value,bytes):return 12+2*len(value),len(value)
    if isinstance(value,str):
        size=len(value.encode('utf-8'));return 13+2*size,size
    raise TypeError('unmodeled SQLite value')


def record(values):
    fields=[serial(value) for value in values];types=sum(uint_bytes(code) for code,_ in fields)
    header=types+1
    while uint_bytes(header)+types!=header:header=uint_bytes(header)+types
    content=sum(size for _,size in fields)
    return content,header


def fingerprint(path):
    digest=hashlib.sha256()
    with path.open('rb') as stream:
        for block in iter(lambda:stream.read(1024*1024),b''):digest.update(block)
    return digest.hexdigest()


def collect(path,scripts):
    sys.path.insert(0,str(scripts))
    from research_history_compact import frame,unframe,sha
    before={str(p.name):{'bytes':p.stat().st_size,'sha256':fingerprint(p)} for p in [path,Path(str(path)+'-wal')] if p.exists()}
    db=sqlite3.connect(path.resolve().as_uri()+'?mode=ro',uri=True);db.execute('PRAGMA query_only=ON');db.execute('BEGIN')
    result={'source':str(path),'read_only':True,'sqlite':sqlite3.sqlite_version,
            'source_id':db.execute('SELECT sqlite_source_id()').fetchone()[0],
            'page_size':db.execute('PRAGMA page_size').fetchone()[0],
            'page_count':db.execute('PRAGMA page_count').fetchone()[0],
            'freelist_pages':db.execute('PRAGMA freelist_count').fetchone()[0],
            'objects':{},'body_components':{}}
    queries=[(name,'SELECT * FROM '+name,0) for name in ['payload','receipt','bucket','branch','authority']]
    queries += [('sqlite_autoindex_payload_1','SELECT hash,local FROM payload',None),
                ('sqlite_autoindex_receipt_1','SELECT id,seq FROM receipt',None),
                ('sqlite_autoindex_receipt_2','SELECT edit,seq FROM receipt',None),
                ('sqlite_schema','SELECT type,name,tbl_name,rootpage,sql FROM sqlite_schema',None)]
    for name,sql,alias in queries:
        obj={'rows':0,'column_content_bytes':0,'record_header_bytes':0}
        for row in db.execute(sql):
            values=list(row)
            if alias is not None:values[alias]=None
            content,header=record(values);obj['rows']+=1;obj['column_content_bytes']+=content;obj['record_header_bytes']+=header
        obj['record_payload_bytes']=obj['column_content_bytes']+obj['record_header_bytes'];result['objects'][name]=obj
    try:
        for name,page_bytes,payload,unused,cells,overflow in db.execute(
            "SELECT name,sum(pgsize),sum(payload),sum(unused),sum(ncell),sum(pagetype='overflow') FROM dbstat GROUP BY name"):
            obj=result['objects'][name]
            if obj['record_payload_bytes']!=payload:raise AssertionError((name,'record model differs from dbstat',obj,payload))
            obj.update(page_bytes=page_bytes,dbstat_payload_bytes=payload,unused_bytes=unused,
                       page_and_cell_overhead_bytes=page_bytes-payload-unused,cells=cells,overflow_pages=overflow)
        result['dbstat_model_exact']=True
    except sqlite3.OperationalError as error:result['dbstat_unavailable']=str(error)
    result['compressed_payload_bytes']=db.execute('SELECT sum(length(bytes)) FROM payload').fetchone()[0]
    result['payload_identities']=db.execute('SELECT count(*) FROM payload').fetchone()[0]
    result['payload_reference_map']={'entries':result['payload_identities'],'hash_bytes':32*result['payload_identities']}
    refs=dict(db.execute('SELECT local,hash FROM payload'));contents=set();oracles=set();intents=set();ids=set();hashes=set()
    previous=bytes(32);manifest=hashlib.sha256();count=0;edits=0
    for seq,txid,intent,kind,edit,body,digest in db.execute('SELECT * FROM receipt ORDER BY seq'):
        values=unframe(body)
        if values[0]!=b'N1' or seq!=count:raise AssertionError('unexpected canonical source')
        if kind in (0,1):
            life,content,delta,checkpoint,oracle=values[1:];contents.add(content);oracles.add(oracle);edits+=1
            restored=[previous,edit,edit,None,refs[life],None if edit==0 else edit-1,edit-edit%64,
                      content,None if delta is None else refs[delta],None if checkpoint is None else refs[checkpoint],oracle]
            labels=['marker','lifetime_reference','content_identity','delta_reference','checkpoint_reference','oracle']
        else:
            head,cursor,redo,life=values[1:];restored=[previous,head,cursor,redo,refs[life],None,None,None,None,None,None]
            labels=['marker','control_head','control_cursor','control_redo','lifetime_reference']
        for label,value in zip(labels,values):result['body_components'][label]=result['body_components'].get(label,0)+len(frame([value]))
        canonical=frame([seq,txid,intent,kind,edit,frame(restored),digest])
        if sha(b'R'+frame([seq,txid,intent,kind,edit,frame(restored)]))!=digest:raise AssertionError('canonical receipt binding')
        manifest.update(len(canonical).to_bytes(8,'little'));manifest.update(canonical)
        ids.add(txid);intents.add(intent);hashes.add(digest);previous=digest;count+=1
    result['canonical_receipts']={'rows':count,'sha256':manifest.hexdigest()}
    result['fixed_field_budget']={'transaction_ids':16*len(ids),'distinct_intent_digests':32*len(intents),
        'receipt_commitments':32*len(hashes),'distinct_content_identities':16*len(contents),
        'distinct_oracles':32*len(oracles),'payload_identities':32*len(refs)}
    result['counts']={'receipts':count,'edits_including_seed':edits,'distinct_intents':len(intents),
        'contents':len(contents),'oracles':len(oracles),'branches':result['objects']['branch']['rows'],
        'leaves':result['objects']['bucket']['rows']}
    result['receipt_columns']=dict(zip(['id','intent','body','hash'],db.execute(
        'SELECT sum(length(id)),sum(length(intent)),sum(length(body)),sum(length(hash)) FROM receipt').fetchone()))
    result['authority_columns']=dict(zip(['body','hash'],db.execute('SELECT length(body),length(hash) FROM authority').fetchone()))
    result['authentication_raw']={'scoped_keys':32*count,'receipt_digest_copies':32*count,
        'branch_child_digests':64*result['counts']['branches'],'node_tags':result['counts']['branches']+result['counts']['leaves']}
    result['record_payload_floor']=sum(obj['record_payload_bytes'] for obj in result['objects'].values())
    result['column_content_floor']=sum(obj['column_content_bytes'] for obj in result['objects'].values())
    result['fixed_field_budget_total']=sum(result['fixed_field_budget'].values())
    payload=result['compressed_payload_bytes'];result['target_20pct_bytes']=payload*.2
    result['record_floor_over_compressed']=(result['record_payload_floor']-payload)/payload
    result['column_floor_over_compressed']=(result['column_content_floor']-payload)/payload
    result['fixed_field_budget_over_compressed']=result['fixed_field_budget_total']/payload
    if result.get('dbstat_model_exact'):
        result['unused_bytes']=sum(obj['unused_bytes'] for obj in result['objects'].values())
        result['page_and_cell_overhead_bytes']=sum(obj['page_and_cell_overhead_bytes'] for obj in result['objects'].values())
        if sum(obj['page_bytes'] for obj in result['objects'].values())+result['freelist_pages']*result['page_size']!=result['page_count']*result['page_size']:
            raise AssertionError('unexplained pages; extend accounting before claiming exactness')
    if sum(result['body_components'].values())!=result['receipt_columns']['body']:raise AssertionError('body accounting')
    db.execute('ROLLBACK');db.close()
    after={str(p.name):{'bytes':p.stat().st_size,'sha256':fingerprint(p)} for p in [path,Path(str(path)+'-wal')] if p.exists()}
    if before!=after:raise AssertionError('source changed during read-only accounting')
    result['source_fingerprints']=after
    result['limitations']=['record floor holds existing schema, field encodings, index keys and logical authentication representation fixed; not a universal information-theoretic lower bound',
        'fixed-field budget assumes independently retained fixed-width values; some may be re-derived with different work/format contracts',
        'unavailable dbstat is not zero; Windows record sizes are modeled from visible rows using documented SQLite record encoding']
    return result


if __name__=='__main__':
    parser=argparse.ArgumentParser(description=__doc__);parser.add_argument('database',type=Path);parser.add_argument('output',type=Path)
    parser.add_argument('--scripts',type=Path,required=True);args=parser.parse_args()
    result=collect(args.database,args.scripts);args.output.parent.mkdir(parents=True,exist_ok=True)
    if args.output.exists():raise FileExistsError(args.output)
    args.output.write_text(json.dumps(result,indent=2)+'\n')
