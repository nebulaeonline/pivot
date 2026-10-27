"""G3 research integrity tests; no application or GUI activation."""
import json
from pathlib import Path
import subprocess
import sys
import tempfile
import unittest

ROOT=Path(__file__).resolve().parents[2]
sys.path.insert(0,str(ROOT/'scripts'))
from research_history_index import Store, Unknown, Backpressure, checkpoint_bytes, digest


def key(tag):
    return (bytes([tag])+bytes(32)).hex()


def fixture():
    records={key(0):b'authority'.hex(),key(1):b'lifetime'.hex(),key(6):b'core'.hex(),key(7):b'one'.hex()}
    seed={'seq':0,'content':'content-0','records':list(records.items()),'oracle':digest(checkpoint_bytes(records))}
    updated=records.copy();updated[key(7)]=b'two'.hex()
    item={'seq':1,'case':'test','content':'content-1','ops':[[key(7),records[key(7)],updated[key(7)]]],
          'oracle':digest(checkpoint_bytes(updated)),'checkpoint':None}
    return seed,item


class IndexStoreTests(unittest.TestCase):
    def setUp(self):
        self.temp=tempfile.TemporaryDirectory(prefix='pivot-g3-test-')
        self.path=Path(self.temp.name)/'store.sqlite'
        self.store=Store(self.path)
        self.seed,self.item=fixture()
        self.store.create(self.seed)

    def tearDown(self):
        self.store.close()
        self.temp.cleanup()

    def reopen(self):
        self.store.close();self.store=Store(self.path)

    def test_exact_receipt_retry_and_intent_mismatch(self):
        first=self.store.publish(self.item)
        second=self.store.publish(self.item)
        self.assertEqual(first['receipt'],second['receipt'])
        self.assertEqual(self.store.authority()[0],1)
        changed=dict(self.item,case='different-intent')
        with self.assertRaises(Unknown):self.store.publish(changed)
        self.assertEqual(self.store.cold(1)[key(7)],b'two'.hex())

    def test_missing_receipt_requires_verified_horizon(self):
        self.reopen()
        with self.assertRaises(Unknown):self.store.reconcile('missing','intent')
        self.store.audit()
        result=self.store.reconcile('missing','intent')
        self.assertEqual(result['outcome'],'uncommitted')
        self.assertEqual(result['receipt'],self.store.reconcile('missing','intent')['receipt'])

    def test_terminal_negative_prevents_late_authoring(self):
        from research_history_index import encoded
        intent=digest(encoded(self.item))
        terminal=self.store.reconcile('edit-1',intent)
        self.assertEqual(self.store.publish(self.item)['receipt'],terminal['receipt'])
        self.assertEqual(self.store.authority()[2],0)
        self.assertEqual(self.store.db.execute('SELECT count(*) FROM state').fetchone()[0],1)

    def test_stale_reader_cannot_return_negative(self):
        stale=Store(self.path)
        try:
            stale.audit();self.store.publish(self.item)
            with self.assertRaises(Unknown):stale.reconcile('missing','intent')
            stale.audit()
            self.assertEqual(stale.reconcile('missing','intent')['outcome'],'uncommitted')
        finally:stale.close()

    def test_cursor_redo_keeps_lifetime_authority(self):
        self.store.publish(self.item)
        life=self.store.authority()[5]
        self.store.move(0,'undo')
        self.assertEqual(self.store.authority()[3:],(0,1,life))
        self.store.move(1,'redo')
        self.assertEqual(self.store.authority()[3:],(1,None,life))
        self.reopen();self.assertEqual(self.store.audit()['decisions_scanned'],4)

    def test_original_state_failure_is_atomic(self):
        bad=dict(self.item,ops=[[key(7),b'wrong'.hex(),b'two'.hex()]])
        with self.assertRaises(Unknown):self.store.publish(bad)
        self.assertEqual(self.store.authority()[0],0)
        self.assertEqual(self.store.db.execute('SELECT count(*) FROM state').fetchone()[0],1)

    def test_rejected_oracle_does_not_change_head(self):
        with self.assertRaises(Unknown):self.store.publish(dict(self.item,oracle='00'*32))
        self.assertEqual(self.store.authority()[0],0)

    def test_removed_receipt_cannot_prove_noncommit(self):
        self.store.publish(self.item)
        self.store.db.execute('DELETE FROM decision WHERE seq=1')
        self.reopen()
        with self.assertRaises(Unknown):self.store.audit()
        with self.assertRaises(Unknown):self.store.reconcile('edit-1','intent')

    def test_corrupt_payload_does_not_activate(self):
        self.store.publish(self.item)
        self.store.db.execute("UPDATE payload SET bytes=x'7b7d'")
        before=self.store.authority()
        with self.assertRaises(Unknown):self.store.cold(1)
        self.assertEqual(before,self.store.authority())
        self.reopen()
        with self.assertRaises(Unknown):self.store.audit()

    def test_required_index_disagreement_is_unknown(self):
        self.store.publish(self.item)
        self.store.db.execute('UPDATE state SET content=? WHERE id=1',('wrong',))
        self.reopen()
        with self.assertRaises(Unknown):self.store.audit()

    def test_payload_publication_fault_rolls_back_all_authority(self):
        def fail(point):
            if point=='after_payload':raise OSError('injected storage error')
        with self.assertRaises(OSError):self.store.publish(self.item,fault=fail)
        self.reopen();self.store.audit()
        self.assertEqual(self.store.authority()[0],0)

    def test_lost_ack_reconciles_same_receipt_without_replay(self):
        def fail(point):
            if point=='after_commit':raise OSError('injected lost ACK')
        with self.assertRaises(OSError):self.store.publish(self.item,fault=fail)
        self.reopen();self.store.audit()
        self.assertEqual(self.store.publish(self.item)['outcome'],'committed')
        self.assertEqual(self.store.authority()[0],1)

    def test_real_sqlite_exhaustion_does_not_partial_publish(self):
        import sqlite3
        pages=self.store.db.execute('PRAGMA page_count').fetchone()[0]
        self.store.db.execute('PRAGMA max_page_count='+str(pages))
        # Exhaust actual SQLite pages in the transaction, not a fabricated success.
        def exhaust(point):
            if point=='after_payload':self.store.put({'large':'x'*(1024*1024)})
        with self.assertRaises(sqlite3.OperationalError):self.store.publish(self.item,fault=exhaust)
        self.reopen();self.store.audit()
        self.assertEqual(self.store.authority()[0],0)

    def test_delayed_reader_applies_backpressure_without_retention_loss(self):
        import sqlite3
        self.store.db.execute('PRAGMA wal_checkpoint(TRUNCATE)')
        reader=sqlite3.connect(self.path,isolation_level=None)
        try:
            reader.execute('BEGIN');reader.execute('SELECT * FROM authority').fetchall()
            self.store.publish(self.item)
            self.store.wal_limit=1
            with self.assertRaises(Backpressure):self.store.reconcile('later','intent')
            self.assertEqual(self.store.authority()[2],1)
            reader.execute('ROLLBACK')
            self.assertEqual(self.store.reconcile('later','intent')['outcome'],'uncommitted')
        finally:reader.close()

    def test_real_process_death_at_publication_boundaries(self):
        self.store.close()
        script='''
import sys,time
from pathlib import Path
sys.path.insert(0,sys.argv[1])
from research_history_index import Store
import json
s=Store(Path(sys.argv[2]));s.audit()
def pause(point):
 if point==sys.argv[4]:
  print('READY',flush=True);time.sleep(30)
s.publish(json.loads(sys.argv[3]),fault=pause)
'''
        import queue
        import threading
        for point in ('after_payload','before_commit','after_commit'):
            child=subprocess.Popen([sys.executable,'-c',script,str(ROOT/'scripts'),str(self.path),json.dumps(self.item),point],
                                   stdout=subprocess.PIPE,stderr=subprocess.PIPE,text=True)
            messages=queue.Queue()
            threading.Thread(target=lambda:messages.put(child.stdout.readline()),daemon=True).start()
            try:
                self.assertEqual(messages.get(timeout=15).strip(),'READY')
            finally:
                child.kill();out,err=child.communicate(timeout=10)
            self.assertNotEqual(child.returncode,0,(out,err))
            probe=Store(self.path)
            try:
                probe.audit()
                self.assertEqual(probe.authority()[2],1 if point=='after_commit' else 0)
            finally:probe.close()
        self.store=Store(self.path)


if __name__=='__main__':unittest.main()
