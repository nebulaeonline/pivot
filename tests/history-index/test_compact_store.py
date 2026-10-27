"""Compact encoding and controlled SQL/bucket semantic comparisons."""
import json
import os
from pathlib import Path
import queue
import sqlite3
import subprocess
import sys
import tempfile
import threading
import unittest
import uuid
import zlib

ROOT=Path(__file__).resolve().parents[2]
sys.path.insert(0,str(ROOT/'scripts'))
from research_history_compact import CompactStore,pack_ops,unpack_ops,pack_map,unpack_map,frame,unframe,decompress,sha
from research_history_index import Unknown,checkpoint_bytes,apply_ops
from test_index_store import fixture,key


def data():
    seed,item=fixture();seed['content']=str(uuid.UUID(int=0));item['content']=str(uuid.UUID(int=1))
    return seed,item


class CodecTests(unittest.TestCase):
    def test_preserves_before_after_null_empty_and_stable_keys(self):
        ops=[[key(1),None,''],[key(2),'00ff',None],[key(3),'ff','0102']]
        self.assertEqual(unpack_ops(pack_ops(ops)),ops)
        records={key(1):'',key(2):'00ff'}
        self.assertEqual(unpack_map(pack_map(records)),records)

    def test_rejects_truncation_trailing_and_wrong_versions(self):
        raw=pack_ops([[key(1),'00','01']])
        for value in [raw[:-1],raw+b'x',b'D0'+raw[2:]]:
            with self.assertRaises(Unknown):unpack_ops(value)
        with self.assertRaises(Unknown):unpack_ops(pack_ops([[key(2),None,'00'],[key(1),None,'01']]))

    def test_compression_length_and_trailing_data_fail_closed(self):
        raw=b'payload'*100;compressed=zlib.compress(raw)
        self.assertEqual(decompress(compressed,len(raw)),raw)
        for value,length in [(compressed,len(raw)-1),(compressed+b'x',len(raw)),(compressed[:-1],len(raw))]:
            with self.assertRaises(Unknown):decompress(value,length)

    def test_binary_frame_types_and_boundaries(self):
        fields=[None,0,127,128,2**63-1,b'',b'\0\xff']
        self.assertEqual(unframe(frame(fields)),fields)
        with self.assertRaises(Unknown):unframe(frame([b'abc'])[:-1])


class CompactTests(unittest.TestCase):
    mode='buckets'

    def setUp(self):
        self.temp=tempfile.TemporaryDirectory(prefix='pivot-compact-')
        self.path=Path(self.temp.name)/'history.sqlite'
        self.seed,self.item=data();self.records=dict(self.seed['records']);self.instance=sha(b'test instance')
        self.store=CompactStore(self.path,self.instance,self.mode);self.store.create(self.seed)

    def tearDown(self):self.store.close();self.temp.cleanup()

    def reopen(self):
        self.store.close();self.store=CompactStore(self.path,self.instance,self.mode)
        return self.store.open_verified()

    def extend(self,n):
        for i in range(self.store.meta[5]+1,n+1):
            old=self.records[key(7)];changed=self.records.copy();changed[key(7)]=str(i).encode().hex()
            item={'seq':i,'content':str(uuid.UUID(int=i)),'ops':[[key(7),old,changed[key(7)]]],
                  'oracle':sha(checkpoint_bytes(changed)).hex(),
                  'checkpoint':list(changed.items()) if i%64==0 else None}
            prepared=self.store.prepare(item,self.records);self.store.publish(prepared);self.records=prepared['records']

    def test_roundtrip_and_exact_receipt_replay(self):
        prepared=self.store.prepare(self.item,self.records);first=self.store.publish(prepared)
        self.reopen();second=self.store.publish(prepared)
        self.assertEqual(first['receipt'],second['receipt']);self.assertEqual(self.store.meta[5],1)
        self.assertEqual(self.store.cold(1),prepared['records'])
        with self.assertRaises(Unknown):self.store.publish(dict(prepared,intent=sha(b'other intent')))

    def test_sql_absence_is_not_silently_authenticated(self):
        txid=self.store.txid('missing');intent=sha(b'intent')
        if self.mode=='plain':
            with self.assertRaises(Unknown):self.store.reconcile(txid,intent)
            self.reopen();result=self.store.reconcile(txid,intent,full_audit=True)
        else:result=self.store.reconcile(txid,intent)
        self.assertEqual(result['outcome'],2)
        self.reopen();self.assertEqual(self.store.reconcile(txid,intent)['receipt'],result['receipt'])

    def test_terminal_negative_prevents_delayed_edit(self):
        prepared=self.store.prepare(self.item,self.records)
        result=self.store.reconcile(prepared['id'],prepared['intent'],full_audit=self.mode=='plain')
        self.reopen();self.assertEqual(self.store.publish(prepared)['receipt'],result['receipt'])
        self.assertEqual(self.store.meta[5],0)

    def test_deleted_historical_terminal_negative_cannot_allow_late_authoring(self):
        prepared=self.store.prepare(self.item,self.records)
        self.store.reconcile(prepared['id'],prepared['intent'],full_audit=self.mode=='plain')
        self.store.reconcile(self.store.txid('later-control'),sha(b'control'),full_audit=self.mode=='plain')
        self.store.db.execute('DELETE FROM receipt WHERE id=?',(prepared['id'],))
        self.reopen()
        with self.assertRaises(Unknown):self.store.publish(prepared)
        self.assertEqual(self.store.meta[5],0)

    def test_before_image_and_oracle_rejections(self):
        wrong=dict(self.item,ops=[[key(7),'00','01']])
        with self.assertRaises(Unknown):self.store.prepare(wrong,self.records)
        with self.assertRaises(Unknown):self.store.prepare(dict(self.item,oracle='00'*32),self.records)
        self.assertEqual(self.store.meta[5],0)

    def test_missing_old_receipt_is_unknown_not_uncommitted(self):
        self.extend(64)
        txid,intent=self.store.db.execute('SELECT id,intent FROM receipt WHERE edit=1').fetchone()
        self.store.db.execute('DELETE FROM receipt WHERE edit=1');self.reopen()
        with self.assertRaises(Unknown):self.store.reconcile(txid,intent)
        if self.mode=='plain':
            self.reopen()
            with self.assertRaises(Unknown):self.store.reconcile(txid,intent,full_audit=True)
        with self.assertRaises(Unknown):self.store.reconcile(self.store.txid('unrelated'),sha(b'intent'))
        with self.assertRaises(Unknown):self.store.audit()

    def test_old_payload_damage_is_lazy_and_blocks_after_discovery(self):
        self.extend(64)
        row=self.store.db.execute('SELECT body FROM receipt WHERE edit=0').fetchone()
        ref=unframe(row[0])[9]
        self.store.db.execute('UPDATE payload SET bytes=? WHERE hash=?',(b'broken',ref));self.reopen()
        with self.assertRaises(Unknown):self.store.cold(0)
        with self.assertRaises(Unknown):self.store.open_verified()
        with self.assertRaises(Unknown):self.store.audit()

    def test_current_state_or_root_damage_blocks_open(self):
        prepared=self.store.prepare(self.item,self.records);self.store.publish(prepared)
        self.store.db.execute('DELETE FROM receipt WHERE edit=1')
        with self.assertRaises(Unknown):self.reopen()

    def test_required_index_loss_refuses(self):
        self.store.db.execute('CREATE INDEX unwanted ON receipt(kind)')
        with self.assertRaises(Unknown):self.reopen()

    def test_wrong_instance_refuses(self):
        other=CompactStore(self.path,sha(b'wrong instance'),self.mode)
        try:
            with self.assertRaises(Unknown):other.open_verified()
        finally:other.close()

    def test_stale_connection_refuses_terminal_decision(self):
        other=CompactStore(self.path,self.instance,self.mode)
        try:
            other.open_verified();self.store.publish(self.store.prepare(self.item,self.records))
            with self.assertRaises(Unknown):other.reconcile(other.txid('missing'),sha(b'intent'))
        finally:other.close()

    def test_reader_snapshot_retains_old_proof_and_cannot_terminalize_stale_absence(self):
        reader=CompactStore(self.path,self.instance,self.mode)
        try:
            reader.open_verified();reader.db.execute('BEGIN');reader.read_meta()
            seedid=self.store.txid('seed');old=reader.find(seedid)
            prepared=self.store.prepare(self.item,self.records)
            if self.mode=='buckets':
                # Force the update to the exact bucket pinned by this reader.
                prefix=self.store.key(seedid)[0]
                for n in range(10000):
                    candidate=self.store.txid('reader-collision-'+str(n))
                    if self.store.key(candidate)[0]==prefix:prepared['id']=candidate;break
                else:self.fail('test could not select shared bucket')
            self.store.publish(prepared)
            self.assertEqual(reader.find(seedid),old)
            self.assertIsNone(reader.find(prepared['id']))
            reader.db.execute('ROLLBACK')
            with self.assertRaises(Unknown):reader.reconcile(prepared['id'],prepared['intent'])
        finally:reader.close()

    def test_lost_ack_requires_verified_reconciliation(self):
        prepared=self.store.prepare(self.item,self.records)
        def fault(point):
            if point=='after_commit':raise OSError('lost ACK')
        with self.assertRaises(OSError):self.store.publish(prepared,fault=fault)
        with self.assertRaises(Unknown):self.store.reconcile(prepared['id'],prepared['intent'])
        self.reopen();self.assertEqual(self.store.publish(prepared)['outcome'],1)

    def test_failure_before_commit_rolls_back_every_publication_component(self):
        def fault(point):
            if point=='before_commit':raise OSError('injected failure')
        with self.assertRaises(OSError):self.store.publish(self.store.prepare(self.item,self.records),fault=fault)
        self.reopen();self.store.audit();self.assertEqual(self.store.meta[5],0)

    def test_cursor_redo_retains_latest_lifetime(self):
        self.extend(2);life=self.store.meta[9]
        self.store.move(1,'undo');self.reopen()
        self.assertEqual(self.store.meta[5:8],[2,1,2]);self.assertEqual(self.store.meta[9],life)
        self.store.move(2,'redo');self.reopen();self.store.audit()
        self.assertEqual(self.store.meta[5:8],[2,2,None]);self.assertEqual(self.store.meta[9],life)

    def test_actual_sqlite_exhaustion_is_atomic(self):
        pages=self.store.db.execute('PRAGMA page_count').fetchone()[0]
        self.store.db.execute('PRAGMA max_page_count='+str(pages))
        def fault(point):
            if point=='before_commit':self.store.db.execute('INSERT INTO payload VALUES (?,?,?,?)',(sha(b'exhaust'),1,1024*1024,os.urandom(1024*1024)))
        with self.assertRaises(sqlite3.OperationalError) as caught:
            self.store.publish(self.store.prepare(self.item,self.records),fault=fault)
        self.assertIn('full',str(caught.exception).lower())
        self.reopen();self.store.audit();self.assertEqual(self.store.meta[5],0)

    def test_process_death_atomicity(self):
        prepared=self.store.prepare(self.item,self.records)
        self.store.close()
        code='''
import json,sys,time,uuid
from pathlib import Path
sys.path.insert(0,sys.argv[1]);sys.path.insert(0,sys.argv[2])
from research_history_compact import CompactStore,sha
from test_compact_store import data
seed,item=data();s=CompactStore(Path(sys.argv[3]),sha(b'test instance'),sys.argv[4]);s.open_verified()
p=s.prepare(item,dict(seed['records']))
def pause(point):
 if point==sys.argv[5]:print('READY',flush=True);time.sleep(30)
s.publish(p,fault=pause)
'''
        for point in ['after_payload','before_commit','after_commit']:
            child=subprocess.Popen([sys.executable,'-c',code,str(ROOT/'scripts'),str(Path(__file__).parent),str(self.path),self.mode,point],stdout=subprocess.PIPE,stderr=subprocess.PIPE,text=True)
            messages=queue.Queue();threading.Thread(target=lambda:messages.put(child.stdout.readline()),daemon=True).start()
            try:self.assertEqual(messages.get(timeout=20).strip(),'READY')
            finally:child.kill();out,err=child.communicate(timeout=10)
            self.assertNotEqual(child.returncode,0,(out,err))
            probe=CompactStore(self.path,self.instance,self.mode)
            try:probe.open_verified();probe.audit();self.assertEqual(probe.meta[5],int(point=='after_commit'))
            finally:probe.close()
        self.store=CompactStore(self.path,self.instance,self.mode)

    def test_bucket_corruption_is_not_absence(self):
        self.extend(64)
        if self.mode=='plain':self.skipTest('Plain baseline has no authenticated bucket guarantee')
        txid,intent=self.store.db.execute('SELECT id,intent FROM receipt WHERE edit=1').fetchone()
        index=self.store.key(txid)[0]
        self.store.db.execute('UPDATE bucket SET bytes=? WHERE id=?',(b'x',index))
        with self.assertRaises(Unknown):self.store.reconcile(txid,intent)
        with self.assertRaises(Unknown):self.store.reconcile(self.store.txid('other'),sha(b'intent'))


class PlainTests(CompactTests):mode='plain'


if __name__=='__main__':unittest.main()
