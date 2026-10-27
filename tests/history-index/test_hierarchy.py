"""Hierarchical authentication and lossless canonical receipt restoration."""
import os
from pathlib import Path
import subprocess
import sys
import tempfile
import unittest

ROOT=Path(__file__).resolve().parents[2];sys.path.insert(0,str(ROOT/'scripts'))
from research_history_compact import sha,frame,unframe
from research_history_hierarchy import HierarchyStore,NormalizedStore,PackedStore
from research_history_layout import LayoutStore
from research_history_index import Unknown
from test_compact_store import data


class HierarchyTests(unittest.TestCase):
    cls=HierarchyStore
    def setUp(self):
        self.tmp=tempfile.TemporaryDirectory();self.path=Path(self.tmp.name)/'history.sqlite'
        self.seed,self.item=data();self.instance=sha(b'hierarchy-test');self.config={'max_entries':1}
        self.s=self.open();self.s.create(self.seed)
    def open(self):return self.cls(self.path,self.instance,'buckets',**self.config)
    def reopen(self):self.s.close();self.s=self.open();return self.s.open_verified()
    def tearDown(self):self.s.close();self.tmp.cleanup()
    def add(self,n=0):return self.s.reconcile(self.s.txid('negative-'+str(n)),sha(b'intent'))

    def test_split_reopen_absence_and_complete_audit(self):
        for n in range(40):self.add(n)
        self.assertGreater(self.s.leaves,1);self.assertEqual(self.s.audit()['rows'],41)
        self.assertFalse(self.reopen()['historical_scan']);self.assertEqual(self.add(20)['seq'],21)
        self.assertLess(self.s.metrics['proof_bytes_peak'],6000)

    def test_stale_and_old_snapshot(self):
        r=self.open()
        try:
            r.open_verified();r.db.execute('BEGIN');r.read_meta();before=r.find(r.txid('seed'))
            self.add();self.assertEqual(r.find(r.txid('seed')),before);self.assertIsNone(r.find(r.txid('negative-0')))
            r.db.execute('ROLLBACK')
            with self.assertRaises(Unknown):r.reconcile(r.txid('another'),sha(b'i'))
        finally:r.close()

    def test_split_faults_are_atomic(self):
        old=self.s.db.execute('SELECT * FROM authority').fetchall()
        for point in ['before_split','after_split_plan','after_children','before_commit']:
            def fail(where):
                if where==point:raise OSError(point)
            self.s.split_fault=fail
            prepared=self.s.prepare(self.item,dict(self.seed['records']))
            with self.assertRaises(OSError):self.s.publish(prepared,fault=fail)
            self.reopen();self.assertEqual(self.s.db.execute('SELECT * FROM authority').fetchall(),old)
            self.assertEqual(self.s.audit()['rows'],1)

    def test_admission_refusal_retains_everything(self):
        self.s.close();self.path=Path(self.tmp.name)/'bounded.sqlite';self.config={'max_entries':1,'max_leaves':1}
        self.s=self.open();self.s.create(self.seed)
        with self.assertRaises(RuntimeError):self.add()
        self.reopen();self.assertEqual(self.s.audit()['rows'],1)

    def test_missing_or_corrupt_root_fails_closed(self):
        self.add();table='branch' if self.cls is PackedStore else 'bucket'
        self.s.db.execute('UPDATE '+table+' SET bytes=? WHERE id=1',(b'I'+bytes(64),))
        with self.assertRaises(Unknown):self.reopen()
        with self.assertRaises(Unknown):self.add(3)

    def test_valid_checksum_cannot_hide_missing_child(self):
        self.add();self.s.db.execute('DELETE FROM bucket WHERE id=2')
        if self.cls is PackedStore:self.s.db.execute('DELETE FROM branch WHERE id=2')
        with self.assertRaises(Unknown):self.s.audit()

    def test_invalid_branch_coverage_even_with_updated_root(self):
        raw=b'I'+bytes(63);self.s.db.execute('UPDATE bucket SET bytes=? WHERE id=1',(raw,))
        meta=unframe(self.s.db.execute('SELECT body FROM authority').fetchone()[0]);fields=unframe(meta[8])
        fields[-1]=self.s.digest(1,raw);meta[8]=frame(fields);encoded=frame(meta)
        self.s.db.execute('UPDATE authority SET body=?,hash=?',(encoded,sha(b'M'+encoded)))
        with self.assertRaises(Unknown):self.reopen()

    def test_no_retained_superseded_paths_and_bounded_updates(self):
        for n in range(100):self.add(n)
        self.s.audit();count=sum(1 for _ in self.s.execute('SELECT id FROM bucket'))
        self.assertEqual(count,2*self.s.leaves-1)
        before=dict(self.s.metrics);self.add(100)
        writes=self.s.metrics['index_pages_written']-before['index_pages_written']
        self.assertLessEqual(writes,1+self.s.bounds[2]+3*self.s.bounds[3])

    def test_canonical_authority_matches_unnormalized_baseline(self):
        baseline=LayoutStore(Path(self.tmp.name)/'baseline.sqlite',self.instance,'buckets',layout='rowid')
        try:
            baseline.create(self.seed)
            for s in [baseline,self.s]:
                prepared=s.prepare(self.item,dict(self.seed['records']));s.publish(prepared)
                s.move(0,'undo');s.move(1,'redo');s.reconcile(s.txid('terminal'),sha(b'i'));s.audit()
            self.assertEqual(list(baseline.execute('SELECT * FROM receipt ORDER BY seq')),
                             list(self.s.execute('SELECT * FROM receipt ORDER BY seq')))
            self.assertEqual(baseline.cold(1),self.s.cold(1))
        finally:baseline.close()

    def test_process_death_split_and_commit(self):
        self.s.close()
        code="""import os,sys
sys.path.insert(0,sys.argv[1])
from research_history_hierarchy import HierarchyStore,NormalizedStore,PackedStore
from research_history_compact import sha
s=globals()[sys.argv[4]](sys.argv[2],sha(b'hierarchy-test'),'buckets',max_entries=1);s.open_verified()
def fault(point):
 if point==sys.argv[3]:os._exit(79)
sys.path.insert(0,str(__import__('pathlib').Path(sys.argv[1]).parent/'tests/history-index'))
from test_compact_store import data
seed,item=data();s.split_fault=fault;s.publish(s.prepare(item,dict(seed['records'])),fault=fault)
"""
        for point in ['before_split','after_split_plan','after_children','before_commit','after_commit']:
            result=subprocess.run([sys.executable,'-c',code,str(ROOT/'scripts'),str(self.path),point,self.cls.__name__],capture_output=True)
            self.assertEqual(result.returncode,79,result.stderr)
            self.s=self.open();self.s.open_verified();self.assertEqual(self.s.audit()['rows'],2 if point=='after_commit' else 1);self.s.close()
        self.s=self.open();self.s.open_verified()


class NormalizationTests(HierarchyTests):
    cls=NormalizedStore
    def test_dangling_or_changed_reference_fails_canonical_binding(self):
        self.s.db.execute('UPDATE payload SET hash=? WHERE rowid=1',(bytes(32),))
        with self.assertRaises(Unknown):self.reopen()


class PackedTests(NormalizationTests):
    cls=PackedStore

    def test_duplicate_physical_address_fails_full_audit(self):
        self.add();self.s.db.execute('INSERT INTO bucket VALUES (1,?)',(b'L',))
        with self.assertRaises(Unknown):self.s.audit()

    def test_local_payload_references_survive_vacuum(self):
        prepared=self.s.prepare(self.item,dict(self.seed['records']));self.s.publish(prepared)
        before=list(self.s.execute('SELECT * FROM receipt ORDER BY seq'))
        self.s.db.execute('VACUUM');self.reopen();self.s.audit()
        self.assertEqual(before,list(self.s.execute('SELECT * FROM receipt ORDER BY seq')))


if __name__=='__main__':unittest.main()
