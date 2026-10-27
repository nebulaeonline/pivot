"""G3 same-content layouts and bounded atomic directory growth."""
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
from research_history_compact import sha,frame,unframe
from research_history_index import Unknown
from research_history_layout import LayoutStore,manifest
from research_history_growth import GrowthStore
from test_compact_store import data


def collision(store,bits=4):
    seed=store.key(store.txid('seed'));top=int.from_bytes(seed,'big')>>(256-bits)
    for n in range(100000):
        txid=store.txid('collision-'+str(n));key=int.from_bytes(store.key(txid),'big')
        if key>>(256-bits)==top and (key>>(255-bits))&1 != (int.from_bytes(seed,'big')>>(255-bits))&1:
            return txid
    raise AssertionError('unable to select deterministic collision')


class LayoutTests(unittest.TestCase):
    def test_all_layouts_preserve_identical_authority_bytes_and_queries(self):
        seed,item=data();instance=sha(b'layout test');expected=None
        with tempfile.TemporaryDirectory(prefix='pivot-layout-') as temp:
            for layout in ['without','rowid','local','partial']:
                s=LayoutStore(Path(temp)/(layout+'.sqlite'),instance,'buckets',layout=layout)
                try:
                    s.create(seed);prepared=s.prepare(item,dict(seed['records']));s.publish(prepared)
                    s.move(0,'undo');s.move(1,'redo');s.audit()
                    measured=manifest(s.db)
                    if expected is None:expected=measured
                    else:self.assertEqual(expected,measured)
                    self.assertEqual(s.cold(1),prepared['records'])
                    plan=s.db.execute('EXPLAIN QUERY PLAN SELECT * FROM receipt WHERE edit=?',(1,)).fetchall()
                    self.assertTrue(any('SEARCH' in row[-1] for row in plan),plan)
                    with self.assertRaises(sqlite3.IntegrityError):s.db.execute('INSERT INTO payload(hash,kind,raw_length,bytes) VALUES (NULL,1,0,?)',(b'',))
                finally:s.close()


class GrowthTests(unittest.TestCase):
    def setUp(self):
        self.temp=tempfile.TemporaryDirectory(prefix='pivot-growth-');self.path=Path(self.temp.name)/'growth.sqlite'
        self.seed,self.item=data();self.instance=sha(b'growth test');self.config={'max_entries':1}
        self.s=self.open();self.s.create(self.seed)

    def open(self):return GrowthStore(self.path,self.instance,'buckets',layout='rowid',**self.config)
    def reopen(self):self.s.close();self.s=self.open();return self.s.open_verified()
    def tearDown(self):self.s.close();self.temp.cleanup()

    def test_split_preserves_membership_absence_and_reopen_without_scan(self):
        txid=collision(self.s);intent=sha(b'intent');before=self.s.generation
        outcome=self.s.reconcile(txid,intent)
        self.assertEqual(outcome['outcome'],2);self.assertEqual(len(self.s.directory),17)
        self.assertGreater(self.s.generation,before);self.assertEqual(self.s.metrics['split_count'],1)
        self.reopen();self.assertEqual(self.s.reconcile(txid,intent)['receipt'],outcome['receipt'])
        self.s.audit();self.assertEqual(self.s.meta[5],0)

    def test_old_reader_survives_parent_retirement_then_is_stale(self):
        reader=self.open()
        try:
            reader.open_verified();reader.db.execute('BEGIN');reader.read_meta()
            seedid=self.s.txid('seed');old=reader.find(seedid);old_generation=reader.generation
            txid=collision(self.s);self.s.reconcile(txid,sha(b'intent'))
            self.assertEqual(reader.find(seedid),old);self.assertIsNone(reader.find(txid))
            self.assertEqual(reader.generation,old_generation)
            reader.db.execute('ROLLBACK')
            with self.assertRaises(Unknown):reader.reconcile(txid,sha(b'intent'))
        finally:reader.close()

    def test_partial_split_faults_rollback_every_component(self):
        txid=collision(self.s);before=manifest(self.s.db)
        for point in ['before_split','after_split_plan','after_children']:
            def fault(where):
                if where==point:raise OSError('injected '+point)
            self.s.split_fault=fault
            with self.assertRaises(OSError):self.s.reconcile(txid,sha(b'intent'))
            self.reopen();self.assertEqual(manifest(self.s.db),before)
            self.s.audit()

    def test_leaf_budget_refuses_without_receipt_loss(self):
        self.s.close();self.path=Path(self.temp.name)/'budget.sqlite';self.config={'max_entries':1,'max_leaves':16}
        self.s=self.open();self.s.create(self.seed);txid=collision(self.s);before=manifest(self.s.db)
        with self.assertRaises(RuntimeError):self.s.reconcile(txid,sha(b'intent'))
        self.reopen();self.assertEqual(manifest(self.s.db),before);self.s.audit()

    def test_incomplete_directory_is_not_negative_evidence(self):
        self.s.reconcile(collision(self.s),sha(b'intent'))
        body=unframe(self.s.db.execute('SELECT body FROM authority').fetchone()[0])
        directory=unframe(body[8]);directory=directory[:-2];body[8]=frame(directory)
        raw=frame(body);self.s.db.execute('UPDATE authority SET body=?,hash=?',(raw,sha(b'M'+raw)))
        with self.assertRaises(Unknown):self.reopen()

    def test_missing_child_and_known_integrity_fault_block_mutation(self):
        txid=collision(self.s);self.s.reconcile(txid,sha(b'intent'))
        child=self.s.route(self.s.key(txid));self.s.db.execute('DELETE FROM bucket WHERE id=?',(child,))
        with self.assertRaises(Unknown):self.s.reconcile(txid,sha(b'intent'))
        with self.assertRaises(Unknown):self.s.reconcile(self.s.txid('later'),sha(b'later'))
        with self.assertRaises(Unknown):self.reopen()

    def test_no_application_generation_retention_or_orphan_rows(self):
        for n in range(40):self.s.reconcile(self.s.txid('negative-'+str(n)),sha(b'intent'))
        self.s.audit()
        ids={row[0] for row in self.s.db.execute('SELECT id FROM bucket')}
        self.assertTrue(ids<=set(self.s.directory))
        self.assertEqual(self.s.db.execute('SELECT count(*) FROM authority').fetchone()[0],1)
        self.assertEqual(self.s.db.execute('SELECT count(*) FROM receipt').fetchone()[0],41)

    def test_growth_resource_contract_cannot_silently_change_on_open(self):
        other=GrowthStore(self.path,self.instance,'buckets',layout='rowid',max_entries=2)
        try:
            with self.assertRaises(Unknown):other.open_verified()
        finally:other.close()

    def test_process_death_split_and_publication_boundaries(self):
        self.s.close()
        code='''
import sys,time
from pathlib import Path
sys.path.insert(0,sys.argv[1]);sys.path.insert(0,sys.argv[2])
from research_history_growth import GrowthStore
from research_history_compact import sha
from test_layout_growth import collision
s=GrowthStore(Path(sys.argv[3]),sha(b'growth test'),'buckets',layout='rowid',max_entries=1);s.open_verified()
def pause(point):
 if point==sys.argv[4]:print('READY',flush=True);time.sleep(30)
s.split_fault=pause
txid=collision(s);intent=sha(b'intent')
if sys.argv[4] in ['before_commit','after_commit']:
 s.begin();s.commit_row(txid,intent,2,None,[s.meta[4],s.meta[5],s.meta[6],s.meta[7],s.meta[9],None,None,None,None,None,None],fault=pause)
else:s.reconcile(txid,intent)
'''
        for point in ['before_split','after_split_plan','after_children','before_commit','after_commit']:
            child=subprocess.Popen([sys.executable,'-c',code,str(ROOT/'scripts'),str(Path(__file__).parent),str(self.path),point],stdout=subprocess.PIPE,stderr=subprocess.PIPE,text=True)
            messages=queue.Queue();threading.Thread(target=lambda:messages.put(child.stdout.readline()),daemon=True).start()
            try:self.assertEqual(messages.get(timeout=20).strip(),'READY')
            finally:child.kill();out,err=child.communicate(timeout=10)
            self.assertNotEqual(child.returncode,0,(out,err));probe=self.open()
            try:
                probe.open_verified();probe.audit()
                self.assertEqual(probe.db.execute('SELECT count(*) FROM receipt').fetchone()[0],2 if point=='after_commit' else 1)
            finally:probe.close()
        self.s=self.open()


if __name__=='__main__':unittest.main()
