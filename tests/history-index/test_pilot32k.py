"""Pilot observer checks; never runs the authorized scaling workload."""
import json
import os
from pathlib import Path
import sys
import tempfile
import unittest

ROOT = Path(__file__).resolve().parents[2]
sys.path.insert(0, str(ROOT / 'scripts'))
from research_history_pilot32k import Oracle, rss, configuration, stale_reader
from research_history_compact import sha
from research_history_hierarchy import PackedStore
from research_history_hierarchy_measure import logical_manifest
from test_compact_store import data


class PilotTests(unittest.TestCase):
    def test_independent_oracle_matches_seed_and_control_chain(self):
        seed, _ = data()
        oracle = Oracle(seed)
        with tempfile.TemporaryDirectory() as folder:
            s = PackedStore(Path(folder) / 'tiny.sqlite', oracle.instance, 'buckets')
            try:
                s.create(seed)
                self.assertEqual(s.meta[4], oracle.previous)
                for i in range(1, 4):
                    txid, intent, expected = oracle.control(i)
                    self.assertEqual(txid, s.txid('index-only-' + str(i - 1)))
                    self.assertEqual(intent, sha(b'index-only-intent'))
                    actual = s.reconcile(txid, intent)
                    self.assertEqual(actual['receipt'], expected.hex())
                    self.assertEqual(actual['seq'], i)
                actual = logical_manifest(s)['receipt']
                self.assertEqual(actual['rows'], oracle.rows)
                self.assertEqual(actual['sha256'], oracle.manifest.hexdigest())
            finally:
                s.close()

    def test_rss_is_available(self):
        self.assertGreater(rss()['rss_bytes'], 0)

    @unittest.skipUnless(os.environ.get('PIVOT_RESEARCH_SQLITE'), 'native observer check requires accepted private engine')
    def test_native_observer_and_stale_reader_on_three_controls(self):
        from research_history_engine import EngineStore
        seed, _ = data()
        oracle = Oracle(seed)
        with tempfile.TemporaryDirectory() as folder:
            path = Path(folder) / 'observer-only.sqlite'
            store = EngineStore(path, oracle.instance, 'buckets', fullfsync=sys.platform == 'darwin')
            reader = None
            try:
                configuration(store)
                store.create(seed)
                reader = EngineStore(path, oracle.instance, 'buckets', fullfsync=sys.platform == 'darwin')
                reader.open_verified()
                reader.execute('BEGIN')
                reader.read_meta()
                events = []
                store.telemetry_sink = events.append
                for i in range(1, 4):
                    txid, intent, expected = oracle.control(i)
                    actual = store.reconcile(txid, intent)
                    self.assertEqual(actual['receipt'], expected.hex())
                    self.assertEqual(events[-1]['status'], 'success')
                    if i == 1:
                        stale_reader(reader, store, txid)
                        reader = None
                self.assertEqual(store.audit()['rows'], 4)
                self.assertEqual(logical_manifest(store)['receipt']['sha256'], oracle.manifest.hexdigest())
            finally:
                if reader is not None:
                    reader.close()
                store.close()


if __name__ == '__main__':
    unittest.main()
