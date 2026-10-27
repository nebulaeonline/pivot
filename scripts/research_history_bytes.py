"""Read-only byte accounting of the preserved G3 JSON/hex/trie representation."""
import argparse
from collections import Counter
import json
from pathlib import Path
import sqlite3
import struct
import zlib


def binary_map(records):
    return b''.join(bytes.fromhex(k)+struct.pack('<I',len(bytes.fromhex(v)))+bytes.fromhex(v)
                    for k,v in sorted(records.items()))


def binary_ops(ops):
    out=bytearray()
    for key,before,after in ops:
        out.extend(bytes.fromhex(key))
        for value in [before,after]:
            raw=None if value is None else bytes.fromhex(value)
            out.extend(struct.pack('<I',0xffffffff if raw is None else len(raw)))
            if raw is not None:out.extend(raw)
    return bytes(out)


def account(database,fixture):
    db=sqlite3.connect(database.resolve().as_uri()+'?mode=ro',uri=True)
    totals=Counter();changes=Counter();payload_refs={};oracle=''
    with fixture.open(encoding='utf-8') as stream:
        seed=json.loads(next(stream));records=dict(seed['records'])
        initial=binary_map(records);totals['seed_binary_record_bytes']=len(initial)
        for line in stream:
            item=json.loads(line);totals['transactions']+=1
            raw=binary_ops(item['ops']);totals['delta_binary_before_after_bytes']+=len(raw)
            totals['delta_individually_compressed_bytes']+=len(zlib.compress(raw,6))
            for key,before,after in item['ops']:
                if bytes.fromhex(key)[0]==16:changes['changed_cell_records']+=1
                changes['all_changed_records']+=1
                if after is None:records.pop(key,None)
                else:records[key]=after
            if item['checkpoint'] is not None:
                payload=binary_map(dict(item['checkpoint']))
                totals['logical_checkpoints']+=1
                totals['checkpoint_binary_bytes']+=len(payload)
                totals['checkpoint_individually_compressed_bytes']+=len(zlib.compress(payload,6))
            oracle=item['oracle']
    for n,payload,life in db.execute('SELECT id,payload,ledger FROM state'):
        payload_refs[payload]='checkpoint' if n%64==0 else 'delta'
        payload_refs.setdefault(life,'ledger')
    columns={
        'payload':'length(hash)+length(bytes)',
        'proof_node':'length(hash)+length(bytes)',
        'decision':'length(tx)+length(intent)+length(kind)+length(parent_hash)+length(body)+length(hash)',
        'state':'length(content)+length(payload)+length(oracle)+length(ledger)',
        'authority':'length(hash)+length(ledger)',
        'publication':'length(body)+length(seal)',
    }
    sql_value_bytes={table:db.execute('SELECT coalesce(sum('+expression+'),0) FROM '+table).fetchone()[0]
                     for table,expression in columns.items()}
    stored=Counter();decoded=Counter();unique_ledger=0
    for key,raw in db.execute('SELECT * FROM payload'):
        kind=payload_refs.get(key,'unreferenced')
        stored[kind+'_json_hex_bytes']+=len(raw)
        value=json.loads(raw)
        if kind=='delta':decoded['delta_binary_bytes']+=len(binary_ops(value))
        elif kind in ('ledger','checkpoint'):
            decoded[kind+'_binary_bytes']+=len(binary_map(value))
        if kind=='ledger':unique_ledger+=1
    # Traverse only the latest authoritative map; older path versions are separately counted.
    publication=json.loads(db.execute('SELECT body FROM publication').fetchone()[0])
    stack=[publication['root']];reachable=set();live_bytes=0
    while stack:
        key=stack.pop()
        if key in reachable:raise ValueError('proof cycle/shared subtree')
        row=db.execute('SELECT bytes FROM proof_node WHERE hash=?',(key,)).fetchone()
        if row is None:raise ValueError('missing proof node')
        reachable.add(key);live_bytes+=len(key)+len(row[0])
        node=json.loads(row[0])
        if node[0]=='branch':stack.extend(node[2:])
    count,all_node_bytes=db.execute('SELECT count(*),coalesce(sum(length(hash)+length(bytes)),0) FROM proof_node').fetchone()
    page=db.execute('PRAGMA page_size').fetchone()[0];pages=db.execute('PRAGMA page_count').fetchone()[0]
    free=db.execute('PRAGMA freelist_count').fetchone()[0]
    result={'transactions':totals['transactions'],'fixture_totals':dict(totals),'change_denominators':dict(changes),
            'stored_payload_categories':dict(stored),'decoded_payload_categories':dict(decoded),
            'unique_full_ledger_payloads':unique_ledger,'ledger_references':db.execute('SELECT count(*) FROM state').fetchone()[0],
            'sql_variable_column_bytes':sql_value_bytes,'sql_note':'numeric fields, record headers and ordinary indexes appear in physical residual',
            'proof_nodes':{'all':count,'reachable':len(reachable),'superseded':count-len(reachable),
                           'all_key_value_bytes':all_node_bytes,'reachable_key_value_bytes':live_bytes,
                           'superseded_key_value_bytes':all_node_bytes-live_bytes},
            'page_size':page,'page_count':pages,'freelist_pages':free,'db_page_bytes':page*pages,
            'db_file_bytes':database.stat().st_size,'freelist_bytes':page*free,
            'physical_residual_after_variable_columns':page*pages-sum(sql_value_bytes.values()),
            'final_record_binary_bytes':len(binary_map(records)),'final_oracle':oracle}
    try:
        rows=db.execute('SELECT name,sum(pgsize),sum(payload),sum(unused),count(*) FROM dbstat GROUP BY name').fetchall()
        result['objects']={r[0]:{'page_bytes':r[1],'used_payload_bytes':r[2],'unused_bytes':r[3],
                                'page_header_and_pointer_bytes':r[1]-r[2]-r[3],'pages':r[4]} for r in rows}
        result['page_accounting_residual']=page*pages-sum(r[1] for r in rows)-page*free
    except sqlite3.DatabaseError as error:result['dbstat_unavailable']=str(error)
    result['db_bytes_per_transaction']=page*pages/totals['transactions']
    result['db_bytes_per_changed_cell_record']=page*pages/changes['changed_cell_records']
    result['compressed_delta_bytes_per_changed_cell_record']=totals['delta_individually_compressed_bytes']/changes['changed_cell_records']
    db.close()
    return result


if __name__=='__main__':
    parser=argparse.ArgumentParser(description=__doc__)
    parser.add_argument('database',type=Path);parser.add_argument('fixture',type=Path);parser.add_argument('output',type=Path)
    args=parser.parse_args();args.output.write_text(json.dumps(account(args.database,args.fixture),indent=2)+'\n',encoding='utf-8')
