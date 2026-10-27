"""Bounded open / authenticated absence adversarial research tests."""
import json
from pathlib import Path
import queue
import sqlite3
import subprocess
import sys
import tempfile
import threading
import unittest

ROOT=Path(__file__).resolve().parents[2]
sys.path.insert(0,str(ROOT/'scripts'))
from research_history_index import Unknown, encoded, digest, checkpoint_bytes
from research_history_verified import VerifiedStore, MAX_WAL
from test_index_store import fixture, key


class VerifiedStoreTests(unittest.TestCase):
    def setUp(self):
        self.temp=tempfile.TemporaryDirectory(prefix='pivot-g3-open-')
        self.path=Path(self.temp.name)/'history.sqlite'
        self.seed,self.item=fixture()
        self.instance=digest(encoded(self.seed['records']))
        self.store=VerifiedStore(self.path,self.instance)
        self.store.create(self.seed)

    def tearDown(self):
        self.store.close();self.temp.cleanup()

    def reopen(self):
        self.store.close();self.store=VerifiedStore(self.path,self.instance)
        return self.store.verified_open()

    def extend(self,count):
        records=self.store.cold(self.store.authority()[2])
        for i in range(self.store.authority()[2]+1,count+1):
            old=records[key(7)];records[key(7)]=str(i).encode().hex()
            item={'seq':i,'content':'content-'+str(i),'case':'test',
                  'ops':[[key(7),old,records[key(7)]]],
                  'oracle':digest(checkpoint_bytes(records)),
                  'checkpoint':list(records.items()) if i%64==0 else None}
            self.store.publish(item)

    def test_normal_open_does_not_audit_history(self):
        self.store.publish(self.item)
        report=self.reopen()
        self.assertFalse(report['historical_decision_scan'])
        self.assertLess(report['proof_nodes_read'],257*8)
        intent=digest(encoded(self.item))
        self.assertEqual(self.store.reconcile('edit-1',intent)['outcome'],'committed')

    def test_duplicate_receipt_and_conflicting_intent(self):
        first=self.store.publish(self.item)
        self.reopen()
        self.assertEqual(first['receipt'],self.store.publish(self.item)['receipt'])
        with self.assertRaises(Unknown):self.store.publish(dict(self.item,case='conflict'))

    def test_terminal_negative_then_delayed_identical_edit(self):
        self.reopen();intent=digest(encoded(self.item))
        negative=self.store.reconcile('edit-1',intent)
        self.reopen()
        self.assertEqual(self.store.publish(self.item)['receipt'],negative['receipt'])
        self.assertEqual(self.store.authority()[2],0)

    def test_current_receipt_missing_blocks_open(self):
        self.store.publish(self.item)
        self.store.db.execute('DELETE FROM decision WHERE seq=1')
        with self.assertRaises(Unknown):self.reopen()
        with self.assertRaises(Unknown):self.store.reconcile('edit-1',digest(encoded(self.item)))

    def test_old_receipt_missing_is_not_fabricated_absence(self):
        self.extend(3)
        tx,intent=self.store.db.execute('SELECT tx,intent FROM decision WHERE seq=1').fetchone()
        self.store.db.execute('DELETE FROM decision WHERE seq=1')
        # Latent historical row loss is not necessarily read at bounded open.
        self.reopen()
        with self.assertRaises(Unknown):self.store.reconcile(tx,intent)
        with self.assertRaises(Unknown):self.store.audit()

    def test_discovered_historical_integrity_fault_blocks_further_mutation(self):
        self.extend(3)
        tx,intent=self.store.db.execute('SELECT tx,intent FROM decision WHERE seq=1').fetchone()
        self.store.db.execute('DELETE FROM decision WHERE seq=1')
        self.reopen()
        before=self.store.authority()
        with self.assertRaises(Unknown):self.store.reconcile(tx,intent)
        with self.assertRaises(Unknown):self.store.reconcile('unrelated','intent')
        with self.assertRaises(Unknown):self.store.verified_open()
        self.store.verified_open(maintenance=True)
        with self.assertRaises(Unknown):self.store.reconcile('still-blocked','intent')
        with self.assertRaises(Unknown):self.store.audit()
        self.assertEqual(self.store.authority(),before)

    def test_current_state_missing_blocks_open(self):
        self.store.publish(self.item)
        self.store.db.execute('DELETE FROM state WHERE id=1')
        with self.assertRaises(Unknown):self.reopen()

    def test_current_state_index_disagreement_blocks_open(self):
        self.store.publish(self.item)
        self.store.db.execute("UPDATE state SET content='tampered' WHERE id=1")
        with self.assertRaises(Unknown):self.reopen()

    def test_current_payload_corruption_blocks_activation(self):
        self.store.publish(self.item)
        ref=self.store.db.execute('SELECT payload FROM state WHERE id=1').fetchone()[0]
        self.store.db.execute('UPDATE payload SET bytes=? WHERE hash=?',(b'{}',ref))
        with self.assertRaises(Unknown):self.reopen()

    def test_old_payload_corruption_is_lazy_and_audited(self):
        self.extend(64)
        ref=self.store.db.execute('SELECT payload FROM state WHERE id=0').fetchone()[0]
        self.store.db.execute('UPDATE payload SET bytes=? WHERE hash=?',(b'{}',ref))
        self.reopen();before=self.store.authority()
        with self.assertRaises(Unknown):self.store.move(0,'invalid-jump')
        self.assertEqual(before,self.store.authority())
        with self.assertRaises(Unknown):self.store.cold(0)
        with self.assertRaises(Unknown):self.store.audit()

    def test_old_state_missing_is_lazy_and_audited(self):
        self.extend(64)
        self.store.db.execute('PRAGMA foreign_keys=OFF')
        self.store.db.execute('DELETE FROM state WHERE id=0')
        self.reopen()
        with self.assertRaises(Unknown):self.store.cold(0)
        with self.assertRaises(Unknown):self.store.audit()

    def test_corrupt_current_root_refuses(self):
        self.store.db.execute("UPDATE publication SET seal='broken'")
        with self.assertRaises(Unknown):self.reopen()

    def test_missing_root_node_refuses(self):
        root=self.store.root
        self.store.db.execute('DELETE FROM proof_node WHERE hash=?',(root,))
        with self.assertRaises(Unknown):self.reopen()

    def test_corrupt_old_lookup_proof_is_unknown_not_absent(self):
        self.extend(64)
        hashed=self.store.map_key('receipt','edit-1')
        target=None
        for node_hash,raw in self.store.db.execute('SELECT * FROM proof_node'):
            value=json.loads(raw)
            if value[0]=='leaf' and value[1]==hashed:target=node_hash;break
        self.assertIsNotNone(target)
        self.store.db.execute('UPDATE proof_node SET bytes=? WHERE hash=?',(b'[]',target))
        self.reopen()
        with self.assertRaises(Unknown):self.store.reconcile('edit-1','irrelevant-intent')
        with self.assertRaises(Unknown):self.store.audit()

    def test_actual_sqlite_full_rolls_back_map_and_authority(self):
        pages=self.store.db.execute('PRAGMA page_count').fetchone()[0]
        self.store.db.execute('PRAGMA max_page_count='+str(pages))
        def exhaust(point):
            if point=='before_commit':self.store.put({'large':'x'*(1024*1024)})
        with self.assertRaises(sqlite3.OperationalError) as caught:
            self.store.publish(self.item,fault=exhaust)
        self.assertIn('full',str(caught.exception).lower())
        self.reopen();self.store.audit()
        self.assertEqual(self.store.authority()[2],0)

    def test_missing_required_sql_index_refuses(self):
        self.store.db.execute('DROP INDEX state_parent')
        with self.assertRaises(Unknown):self.reopen()

    def test_authority_and_publication_disagreement_refuses(self):
        self.store.db.execute('UPDATE authority SET seq=500')
        with self.assertRaises(Unknown):self.reopen()

    def test_pending_recovery_marker_is_unknown_even_with_valid_checksum(self):
        raw=self.store.db.execute('SELECT body FROM publication').fetchone()[0]
        body=json.loads(raw);body['pending']={'operation':'ambiguous'};raw=encoded(body)
        self.store.db.execute('UPDATE publication SET body=?,seal=?',(raw,digest(raw)))
        with self.assertRaises(Unknown):self.reopen()

    def test_wrong_instance_and_epoch_refuse(self):
        other=VerifiedStore(self.path,'wrong-instance')
        try:
            with self.assertRaises(Unknown):other.verified_open()
        finally:other.close()
        raw=self.store.db.execute('SELECT body FROM publication').fetchone()[0]
        body=json.loads(raw);body['epoch']=1;raw=encoded(body)
        self.store.db.execute('UPDATE publication SET body=?,seal=?',(raw,digest(raw)))
        with self.assertRaises(Unknown):self.reopen()

    def test_stale_connection_cannot_terminalize_negative(self):
        other=VerifiedStore(self.path,self.instance)
        try:
            other.verified_open();self.store.publish(self.item)
            with self.assertRaises(Unknown):other.reconcile('missing','intent')
            other.verified_open()
            self.assertEqual(other.reconcile('missing','intent')['outcome'],'uncommitted')
        finally:other.close()

    def test_lost_ack_blocks_until_verified_reconciliation(self):
        def fail(point):
            if point=='after_commit':raise OSError('lost ACK')
        with self.assertRaises(OSError):self.store.publish(self.item,fault=fail)
        with self.assertRaises(Unknown):self.store.reconcile('edit-1',digest(encoded(self.item)))
        self.reopen()
        self.assertEqual(self.store.publish(self.item)['outcome'],'committed')

    def test_before_commit_fault_keeps_authenticated_root_and_content(self):
        def fail(point):
            if point=='before_commit':raise OSError('before commit')
        with self.assertRaises(OSError):self.store.publish(self.item,fault=fail)
        self.reopen()
        self.assertEqual(self.store.authority()[2],0)
        self.assertIsNone(self.store.lookup('receipt','edit-1'))

    def test_incomplete_physical_wal_tail_refuses_before_connect(self):
        wal=Path(str(self.path)+'-wal')
        with wal.open('ab') as stream:stream.write(b'x')
        with self.assertRaises(Unknown):VerifiedStore(self.path,self.instance)
        with wal.open('r+b') as stream:stream.truncate(wal.stat().st_size-1)

    def test_oversized_wal_refuses_before_sqlite_recovery(self):
        other=Path(self.temp.name)/'oversize.sqlite'
        with Path(str(other)+'-wal').open('wb') as stream:stream.truncate(MAX_WAL+1)
        with self.assertRaises(Unknown):VerifiedStore(other,self.instance)

    def test_index_membership_and_absence_match_independent_dictionary(self):
        self.store.db.execute('BEGIN IMMEDIATE')
        expected={str(i):{'n':i} for i in range(300)}
        for k,v in expected.items():self.store.insert('test',k,v)
        for i in range(600):
            before=self.store.proof_reads
            value=self.store.lookup('test',str(i))
            self.assertEqual(value,digest(encoded(expected[str(i)])) if str(i) in expected else None)
            self.assertLessEqual(self.store.proof_reads-before,257)
        self.store.db.execute('ROLLBACK')
        self.store.publication()

    def test_process_death_before_and_after_atomic_index_publication(self):
        self.store.close()
        code='''
import sys,time,json
from pathlib import Path
sys.path.insert(0,sys.argv[1])
from research_history_verified import VerifiedStore
s=VerifiedStore(Path(sys.argv[2]),sys.argv[3]);s.verified_open()
def pause(point):
 if point==sys.argv[5]:print('READY',flush=True);time.sleep(30)
s.publish(json.loads(sys.argv[4]),fault=pause)
'''
        for point in ('after_payload','before_commit','after_commit'):
            child=subprocess.Popen([sys.executable,'-c',code,str(ROOT/'scripts'),str(self.path),self.instance,json.dumps(self.item),point],
                                   stdout=subprocess.PIPE,stderr=subprocess.PIPE,text=True)
            messages=queue.Queue()
            threading.Thread(target=lambda:messages.put(child.stdout.readline()),daemon=True).start()
            try:self.assertEqual(messages.get(timeout=20).strip(),'READY')
            finally:child.kill();out,err=child.communicate(timeout=10)
            self.assertNotEqual(child.returncode,0,(out,err))
            probe=VerifiedStore(self.path,self.instance)
            try:
                probe.verified_open();probe.audit()
                self.assertEqual(probe.authority()[2],1 if point=='after_commit' else 0)
            finally:probe.close()
        self.store=VerifiedStore(self.path,self.instance)


if __name__=='__main__':unittest.main()
