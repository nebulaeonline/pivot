"""Actual pinned SQLite/VFS admission, atomic failures and bounded native evidence."""
import json
import os
from pathlib import Path
import sqlite3
import subprocess
import sys
import tempfile
import unittest

ROOT=Path(__file__).resolve().parents[2];sys.path.insert(0,str(ROOT/'scripts'))
from research_history_engine import EngineStore
from research_history_layout import LayoutStore
from research_history_compact import sha,frame
from research_history_index import Unknown,Backpressure
from research_sqlite_native import NativeEngine
from test_compact_store import data


@unittest.skipUnless(os.environ.get('PIVOT_RESEARCH_SQLITE'),'explicit private pinned engine required')
class EngineTests(unittest.TestCase):
    def setUp(self):
        self.temp=tempfile.TemporaryDirectory();self.path=Path(self.temp.name)/'engine.sqlite'
        self.seed,self.item=data();self.instance=sha(b'engine-test');self.config={'max_entries':1}
        self.s=self.open();self.s.create(self.seed);self.engine=self.s.engine
    def open(self):return EngineStore(self.path,self.instance,'buckets',**self.config)
    def reopen(self):self.s.close();self.s=self.open();self.s.open_verified()
    def tearDown(self):
        self.engine.fault(-1,-1,-1);self.engine.limit(64*1024*1024);self.s.close();self.temp.cleanup()
    def prepared(self):return self.s.prepare(self.item,dict(self.seed['records']))
    def manifest(self):
        return {table:[list(row) for row in self.s.db.execute('SELECT * FROM '+table+' ORDER BY 1')]
                for table in ['payload','receipt','bucket','branch','authority']}
    def negative(self,label='negative'):return self.s.reconcile(self.s.txid(label),sha(b'i'))

    def test_actual_engine_vfs_and_memory_are_measured(self):
        info=self.s.db.identity();self.assertEqual(info['version'],'3.51.3')
        self.assertIn(info['delegate_vfs'],['win32','unix']);self.assertIn(info['delegate_vfs'],info['connection_file_vfs'])
        self.assertIn('ENABLE_DBSTAT_VTAB',info['compile_options']);self.assertGreater(self.engine.telemetry(self.s.db)['memory_used'],0)
        self.assertGreater(self.engine.telemetry(self.s.db)['connection']['cache_bytes']['current'],0)

    def test_original_canonical_receipts_and_oracle_match(self):
        baseline=LayoutStore(Path(self.temp.name)/'baseline.sqlite',self.instance,'buckets',layout='rowid')
        try:
            baseline.create(self.seed)
            for s in [baseline,self.s]:
                prepared=s.prepare(self.item,dict(self.seed['records']));s.publish(prepared)
                s.move(0,'undo');s.move(1,'redo');s.reconcile(s.txid('terminal'),sha(b'i'));s.audit()
            self.assertEqual(list(baseline.execute('SELECT * FROM receipt ORDER BY seq')),list(self.s.execute('SELECT * FROM receipt ORDER BY seq')))
            self.assertEqual(baseline.cold(1),self.s.cold(1));self.reopen()
        finally:baseline.close()

    def test_exact_replay_and_conflicting_intent(self):
        prepared=self.prepared();one=self.s.publish(prepared)
        self.assertEqual(self.s.publish(prepared)['receipt'],one['receipt'])
        with self.assertRaises(Unknown):self.s.reconcile(prepared['id'],sha(b'wrong'))
        self.reopen();self.s.audit()

    def test_scratch_refusal_precedes_every_sql_mutation(self):
        before=self.manifest();self.s.scratch_limit=1;mutations=self.s.db.mutations
        with self.assertRaises(Backpressure):self.s.publish(self.prepared())
        self.assertEqual(self.s.db.mutations,mutations);self.assertEqual(self.manifest(),before)
        self.s.scratch_limit=1024*1024;self.s.open_verified();self.s.audit()

    def test_bootstrap_scratch_refusal_precedes_schema_and_payload_dml(self):
        fresh=EngineStore(Path(self.temp.name)/'bootstrap.sqlite',self.instance,'buckets',scratch_limit=1)
        try:
            before=fresh.db.mutations
            with self.assertRaises(Backpressure):fresh.create(self.seed)
            self.assertEqual(fresh.db.mutations,before)
            self.assertEqual(fresh.db.execute('SELECT count(*) FROM sqlite_schema').fetchone(),(0,))
        finally:fresh.close()

    def test_native_headroom_refusal_precedes_sql_mutation(self):
        before=self.manifest();self.s.native_headroom=64*1024*1024;mutations=self.s.db.mutations
        with self.assertRaises(Backpressure):self.negative()
        self.assertEqual(self.s.db.mutations,mutations);self.assertEqual(self.manifest(),before)

    def test_native_allocation_failure_rolls_back_payload_and_index(self):
        before=self.manifest()
        def fault(where):
            if where=='after_payload':self.engine.fault(-1,-1,0)
        try:
            with self.assertRaises(sqlite3.Error) as failure:self.s.publish(self.prepared(),fault=fault)
            self.assertEqual(failure.exception.sqlite_errorcode&255,7)
        finally:self.engine.fault(-1,-1,-1)
        if self.s.db.in_transaction:self.s.execute('ROLLBACK')
        self.reopen();self.assertEqual(self.manifest(),before);self.s.audit()
        self.assertGreater(self.engine.telemetry()['injected_allocation_failures'],0)

    def test_actual_native_heap_cap_refuses_column_materialization(self):
        before=self.manifest();self.engine.limit(512*1024)
        try:
            with self.assertRaises(sqlite3.Error) as failure:self.s.db.execute('SELECT zeroblob(2097152)').fetchone()
            self.assertEqual(failure.exception.sqlite_errorcode&255,7)
        finally:self.engine.limit(64*1024*1024)
        self.assertEqual(self.manifest(),before);self.s.audit()

    def test_sqlite_full_is_atomic(self):
        before=self.manifest();pages=self.s.db.execute('PRAGMA page_count').fetchone()[0]
        self.s.db.execute('PRAGMA max_page_count='+str(pages)).fetchone()
        prepared=self.prepared();prepared['blobs'].append(self.s.blob(1,os.urandom(128*1024)))
        with self.assertRaises(sqlite3.Error) as failure:self.s.publish(prepared)
        self.assertEqual(failure.exception.sqlite_errorcode&255,13)
        self.reopen();self.assertEqual(self.manifest(),before);self.s.audit()

    def test_partial_planned_node_failure_is_atomic(self):
        before=self.manifest()
        def fault(where):
            if where=='after_planned_node':raise OSError('partial planned index')
        self.s.split_fault=fault
        with self.assertRaises(OSError):self.s.publish(self.prepared())
        self.reopen();self.assertEqual(self.manifest(),before);self.s.audit()

    def test_pinned_reader_causes_prewrite_backpressure(self):
        self.s.execute('PRAGMA wal_checkpoint(TRUNCATE)').fetchall();reader=self.open()
        try:
            reader.open_verified();reader.db.execute('BEGIN');reader.read_meta()
            self.negative('first');self.s.wal_trigger=1;before=self.manifest();mutations=self.s.db.mutations
            with self.assertRaises(Backpressure):self.negative('blocked')
            self.assertEqual(self.s.db.mutations,mutations);self.assertEqual(self.manifest(),before)
            reader.db.execute('ROLLBACK');self.s.open_verified();self.negative('after-reader')
        finally:reader.close()

    def test_old_snapshot_and_stale_writer(self):
        reader=self.open()
        try:
            reader.open_verified();reader.db.execute('BEGIN');reader.read_meta();old=reader.find(reader.txid('seed'))
            self.negative();self.assertEqual(reader.find(reader.txid('seed')),old);self.assertIsNone(reader.find(reader.txid('negative')))
            reader.db.execute('ROLLBACK')
            with self.assertRaises(Unknown):reader.reconcile(reader.txid('other'),sha(b'i'))
        finally:reader.close()

    def test_invalid_resource_profile_and_larger_leaf_cap_refuse(self):
        for config in [{'max_leaves':4096},{'max_splits':9},{'scratch_limit':2*1024*1024},{'wal_trigger':0}]:
            with self.assertRaises(ValueError):EngineStore(Path(self.temp.name)/'invalid.sqlite',self.instance,'buckets',**config)

    def test_missing_and_corrupt_pages_fail_closed(self):
        self.negative();self.s.db.execute('UPDATE branch SET bytes=? WHERE id=1',(b'I'+bytes(64),))
        with self.assertRaises(Unknown):self.reopen()
        with self.assertRaises(Unknown):self.negative('later')

    def test_sync_and_write_errors_reopen_to_whole_old_or_new_state(self):
        prepared=self.prepared()
        for write,sync in [(0,-1),(-1,0)]:
            def fault(where):
                if where=='before_commit':self.engine.fault(write,sync,-1)
            try:
                with self.assertRaises(sqlite3.Error) as failure:self.s.publish(prepared,fault=fault)
                self.assertEqual(failure.exception.sqlite_errorcode&255,10)
            finally:self.engine.fault(-1,-1,-1)
            self.reopen();self.s.audit();self.assertIn(self.s.meta[5],[0,1])
            if self.s.meta[5]==1:
                self.assertEqual(self.s.cold(1),prepared['records'])
                # A failed sync can leave a committed frame: never claim definitely absent.
                self.assertEqual(self.s.reconcile(prepared['id'],prepared['intent'])['outcome'],1)
                break

    def test_process_death_before_and_after_commit(self):
        self.s.close()
        code="""import os,sys
sys.path.insert(0,sys.argv[1]);sys.path.insert(0,sys.argv[2])
from research_history_engine import EngineStore
from research_history_compact import sha
from test_compact_store import data
s=EngineStore(sys.argv[3],sha(b'engine-test'),'buckets',max_entries=1);s.open_verified()
seed,item=data()
def fault(point):
 if point==sys.argv[4]:os._exit(79)
s.split_fault=fault;s.publish(s.prepare(item,dict(seed['records'])),fault=fault)
"""
        for point in ['before_split','after_planned_node','before_commit','after_commit']:
            result=subprocess.run([sys.executable,'-c',code,str(ROOT/'scripts'),str(ROOT/'tests/history-index'),str(self.path),point],capture_output=True)
            self.assertEqual(result.returncode,79,result.stderr)
            self.s=self.open();self.s.open_verified();self.s.audit();self.assertEqual(self.s.meta[5],1 if point=='after_commit' else 0);self.s.close()
        self.s=self.open();self.s.open_verified()


if __name__=='__main__':unittest.main()
