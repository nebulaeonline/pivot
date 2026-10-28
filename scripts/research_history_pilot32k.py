"""One-shot, fixed-profile 32K terminal-control research pilot; not production CI."""
import argparse
import ctypes
import gzip
import hashlib
import json
import os
from pathlib import Path
import shutil
import sys
import time
import traceback
import tracemalloc
import uuid

from research_history_compact import EMPTY, frame, pack_map, sha
from research_history_index import Backpressure, Unknown, encoded, ledger, distribution
from research_history_engine import EngineStore
from research_history_hierarchy_measure import allocation, detail
from research_history_verified_measure import own_memory
from research_history_accounting import collect, fingerprint
from research_sqlite_native import NativeEngine
from research_history_admission_model import Refusal
from validate import native_root

MIB = 1024 * 1024
COUNT = 32768
PROFILE = {'journal_mode': 'wal', 'synchronous': 2, 'cache_size': -2048,
           'mmap_size': 0, 'wal_autocheckpoint': 0, 'page_size': 4096,
           'max_page_count': 16384}


class Oracle:
    """Construct expected canonical decisions without reading stored receipt bodies."""
    def __init__(self, seed):
        self.instance = sha(encoded(seed['records']))
        self.life = sha(b'\x03' + pack_map(ledger(dict(seed['records']))))
        body = [EMPTY, 0, 0, None, self.life, None, 0, uuid.UUID(seed['content']).bytes,
                None, sha(b'\x02' + pack_map(dict(seed['records']))), bytes.fromhex(seed['oracle'])]
        self.manifest = hashlib.sha256()
        self.rows = 0
        self.previous = self.add(0, self.txid('seed'), sha(b'seed'), 0, 0, body)

    def txid(self, label):
        return sha(frame([self.instance, label.encode()]))[:16]

    def add(self, seq, txid, intent, kind, edit, body):
        fields = [seq, txid, intent, kind, edit, frame(body)]
        receipt = sha(b'R' + frame(fields))
        raw = frame(fields + [receipt])
        self.manifest.update(len(raw).to_bytes(8, 'little'))
        self.manifest.update(raw)
        self.rows += 1
        return receipt

    def control(self, ordinal):
        txid = self.txid('index-only-' + str(ordinal - 1))
        intent = sha(b'index-only-intent')
        body = [self.previous, 0, 0, None, self.life, None, None, None, None, None, None]
        self.previous = self.add(ordinal, txid, intent, 2, None, body)
        return txid, intent, self.previous


def rss():
    if os.name == 'nt':
        value = own_memory()
        if 'rss_bytes' not in value:
            raise RuntimeError('RSS observation unavailable')
        return value
    import resource
    peak = resource.getrusage(resource.RUSAGE_SELF).ru_maxrss
    if sys.platform == 'darwin':
        class TaskInfo(ctypes.Structure):
            _fields_ = [(name, ctypes.c_uint64) for name in ['virtual', 'resident', 'maximum']] + [
                (name, ctypes.c_int32) for name in ['us', 'uu', 'ss', 'su', 'policy', 'suspend']]
        lib = ctypes.CDLL('/usr/lib/libSystem.B.dylib')
        task = ctypes.c_uint32.in_dll(lib, 'mach_task_self_').value
        value = TaskInfo()
        count = ctypes.c_uint32(ctypes.sizeof(value) // 4)
        lib.task_info.argtypes = [ctypes.c_uint32, ctypes.c_int, ctypes.c_void_p,
                                  ctypes.POINTER(ctypes.c_uint32)]
        if lib.task_info(task, 20, ctypes.byref(value), ctypes.byref(count)):
            raise RuntimeError('Mach RSS observation unavailable')
        current = value.resident
    else:
        current = int(Path('/proc/self/statm').read_text().split()[1]) * os.sysconf('SC_PAGE_SIZE')
        peak *= 1024
    return {'rss_bytes': current, 'peak_rss_bytes': peak}


def files(path):
    return {p.name: allocation(p) for p in [path, Path(str(path) + '-wal'), Path(str(path) + '-shm')]
            if p.exists()}


def write_json(path, value):
    path.write_text(json.dumps(value, indent=2) + '\n', encoding='utf-8')


def configuration(store):
    expected = dict(PROFILE, fullfsync=int(sys.platform == 'darwin'),
                    checkpoint_fullfsync=int(sys.platform == 'darwin'))
    actual = {p: store.db.execute('PRAGMA ' + p).fetchone()[0] for p in expected}
    assert actual == expected, ('profile disagreement', actual, expected)
    assert store.bounds == [60, 1024, 24, 8]
    assert (store.scratch_limit, store.native_headroom, store.wal_trigger) == (MIB, 2*MIB, 64*MIB)
    return actual


def preflight(args):
    """No pilot database is opened here. Accepted source/binaries are read only."""
    if args.output.exists():
        raise FileExistsError('one-shot output already exists: ' + str(args.output))
    fs = native_root(args.output.parent)
    assert fs.get('native_work_root') or fs.get('type') == 'apfs', fs
    free = shutil.disk_usage(args.output.parent).free
    assert free >= 768*MIB, ('disk provision plus free floor unavailable', free)
    accepted = json.loads((args.accepted / 'result.json').read_text())
    assert accepted['status'] == 'passed'
    source = Path(__file__).resolve().parent.parent
    checked = {}
    for relative, expected in accepted['tested_inputs'].items():
        relative = relative.replace('\\', '/')
        actual = fingerprint(source / relative)
        assert actual == expected, ('accepted source changed', relative)
        checked[relative] = actual
    build = json.loads((args.accepted / 'build/build.json').read_text())
    engine = NativeEngine()
    assert fingerprint(engine.path) == build['library_sha256'], 'accepted binary changed'
    assert fingerprint(source / 'scripts/research/sqlite_bridge.c') == build['bridge_sha256']
    pin = json.loads((source / 'scripts/research/sqlite-source.json').read_text())
    assert build['pin'] == pin
    connection = engine.connect(':memory:')
    identity = connection.identity()
    connection.close()
    assert identity['version'] == pin['version'] and identity['source_id'] == pin['source_id']
    assert identity['delegate_vfs'] == ('win32' if os.name == 'nt' else 'unix')
    memory = rss()
    reference = own_memory()
    assert abs(memory['rss_bytes'] - reference['rss_bytes']) < 32*MIB, ('RSS sampler disagreement', memory, reference)
    assert memory['rss_bytes'] < 512*MIB
    return {'filesystem': fs, 'free_bytes': free, 'identity': identity, 'build': build,
            'accepted_source_hashes': checked, 'rss': memory, 'rss_crosscheck': reference,
            'profile': dict(PROFILE), 'pilot_script_sha256': fingerprint(Path(__file__)),
            'limits': {'point_bytes': 256*MIB, 'free_floor_bytes': 512*MIB,
                       'rss_bytes': 512*MIB, 'wal_recovery_bytes': 128*MIB}}


def stale_reader(reader, store, txid):
    assert reader.find(txid) is None
    assert reader.find(reader.txid('seed')) is not None
    reader.execute('ROLLBACK')
    before = reader.db.mutations
    try:
        reader.reconcile(txid, sha(b'index-only-intent'))
    except Unknown as error:
        assert str(error) == 'stale verified horizon', str(error)
    else:
        raise AssertionError('stale writer was accepted')
    assert reader.db.mutations == before, 'stale writer performed DML'
    reader.close()
    assert store.meta[3] == 1
    return {'snapshot_absence': True, 'stale_writer_refused_before_dml': True}


def run(args):
    report = {'status': 'preflight', 'authorized_controls': COUNT, 'controls': 0,
              'not_genuine_g2_edits': True, 'milestones': {}}
    checked = preflight(args)
    args.output.mkdir(exist_ok=False)
    write_json(args.output / 'preflight.json', checked)
    # Existence of this marker permanently consumes this target's attempt.
    (args.output / 'ATTEMPT').write_text('single authorized attempt\n')
    store = reader = None
    path = args.output / 'index.sqlite'
    try:
        with args.fixture.open(encoding='utf-8') as stream:
            seed = json.loads(next(stream))
        oracle = Oracle(seed)
        report['instance'] = oracle.instance.hex()
        store = EngineStore(path, oracle.instance, 'buckets', fullfsync=sys.platform == 'darwin')
        report['configuration'] = configuration(store)
        report['identity'] = store.db.identity()
        assert report['identity']['delegate_vfs'] in report['identity']['connection_file_vfs']
        assert report['identity']['delegate_vfs'] == checked['identity']['delegate_vfs']
        store.create(seed)
        assert store.meta[4] == oracle.previous and store.meta[9] == oracle.life
        reader = EngineStore(path, oracle.instance, 'buckets', fullfsync=sys.platform == 'darwin')
        reader.open_verified()
        reader.execute('BEGIN')
        reader.read_meta()
        event = []
        store.telemetry_sink = lambda value: event.append(value)
        report['native_before_controls'] = store.engine.telemetry(store.db)
        committed_manifest = oracle.manifest.hexdigest()
        with gzip.open(args.output / 'publications.jsonl.gz', 'wt', encoding='utf-8', compresslevel=1) as stream:
            for ordinal in range(1, COUNT + 1):
                if args.stop.exists():
                    raise RuntimeError('peer stop requested; attempt preserved')
                if ordinal == 1 or ordinal % 256 == 0:
                    assert shutil.disk_usage(args.output).free >= 512*MIB, 'free disk floor'
                    retained = sum(p.stat().st_size for p in args.output.rglob('*') if p.is_file())
                    assert retained <= 256*MIB, ('point disk budget', retained)
                txid, intent, expected = oracle.control(ordinal)
                before = dict(store.metrics)
                prior_authority = list(store.meta)
                prior_mutations = store.db.mutations
                event.clear()
                trace = ordinal in (1, 10000, 16384, COUNT)
                if trace:
                    tracemalloc.start()
                started = time.perf_counter_ns()
                try:
                    answer = store.reconcile(txid, intent)
                except (Backpressure, Refusal) as error:
                    # Expected admission refusal is final; never retry under a new ID.
                    report.update(status='admission-refused', refusal=str(error), refused_ordinal=ordinal)
                    report['refusal_event'] = event[-1] if event else None
                    assert not store.db.in_transaction and store.db.mutations == prior_mutations
                    store.open_verified()
                    assert store.meta == prior_authority, 'refusal changed authority'
                    report['refusal_atomic_before_dml'] = True
                    break
                outer_ns = time.perf_counter_ns() - started
                assert answer['outcome'] == 2 and answer['seq'] == ordinal and answer['receipt'] == expected.hex()
                assert store.meta[5:8] == [0, 0, None] and store.meta[9] == oracle.life
                assert len(event) == 1 and event[0]['status'] == 'success'
                observation = {'ordinal': ordinal, 'id': txid.hex(), 'intent': intent.hex(),
                               'receipt': answer['receipt'], 'commit_ns': answer['commit_ns'],
                               'outer_ns': outer_ns, 'native': event[0], 'leaves': store.leaves,
                               'generation': store.generation, 'depth_peak': store.metrics['path_depth_peak'],
                               'logical': {k: store.metrics[k] - before[k] for k in
                                   ['split_count', 'index_pages_read', 'index_pages_written',
                                    'proof_read_bytes', 'proof_written_bytes', 'admission_ns']},
                               'files': files(path), 'memory': rss(), 'traced': trace}
                if trace:
                    observation['python_traced_current_peak'] = tracemalloc.get_traced_memory()
                    tracemalloc.stop()
                observation['staged_node_bytes'] = observation['logical']['proof_written_bytes'] - len(store.meta[8])
                assert observation['memory']['rss_bytes'] <= 512*MIB, 'RSS ceiling'
                assert store.wal_bytes() <= 128*MIB, 'unsafe WAL retention'
                assert observation['logical']['split_count'] <= 8 and store.leaves <= 1024
                assert store.metrics['path_depth_peak'] <= 24 and store.scratch is None
                assert event[0]['after']['memory_used'] <= 64*MIB
                stream.write(json.dumps(observation, separators=(',', ':')) + '\n')
                report['controls'] = ordinal
                committed_manifest = oracle.manifest.hexdigest()
                if ordinal == 1:
                    report['stale_reader'] = stale_reader(reader, store, txid)
                    reader = None
                if ordinal in (10000, 16384, COUNT):
                    milestone = {'controls': ordinal, 'leaves': store.leaves, 'metrics': dict(store.metrics),
                                 'files': files(path), 'native': store.engine.telemetry(store.db),
                                 'wal_checkpoint_noop': list(store.db.execute('PRAGMA wal_checkpoint(NOOP)').fetchone()),
                                 'canonical_receipts': {'rows': oracle.rows, 'sha256': oracle.manifest.hexdigest()}}
                    if ordinal == 16384:
                        old = json.loads(args.reference.read_text())['detail']['canonical_manifest']['receipt']
                        assert old['sha256'] == oracle.manifest.hexdigest() and old['rows'] == oracle.rows
                        milestone['accepted_16k_manifest_equal'] = True
                    report['milestones'][str(ordinal)] = milestone
                    write_json(args.output / 'progress.json', report)
                    print('controls', ordinal, 'leaves', store.leaves, flush=True)
        if tracemalloc.is_tracing():
            tracemalloc.stop()
        store.telemetry_sink = None
        report['native_after_controls'] = store.engine.telemetry(store.db)
        report['metrics_after_controls'] = dict(store.metrics)
        report['audit'] = store.audit()
        assert report['audit']['rows'] == report['controls'] + 1
        assert store.cold(0) == dict(seed['records'])
        report['detail'] = detail(store, path)
        # An admission refusal advances only the independent tentative oracle, not the database.
        canonical = report['detail']['canonical_manifest']['receipt']
        assert canonical['sha256'] == committed_manifest
        assert store.generation == report['controls'] + 1
        report['pre_final_checkpoint'] = files(path)
        report['final_checkpoint'] = list(store.db.execute('PRAGMA wal_checkpoint(TRUNCATE)').fetchone())
        report['post_final_checkpoint'] = files(path)
        store.close()
        store = None
        report['accounting'] = collect(path, Path(__file__).parent)
        assert report['accounting']['canonical_receipts'] == {'rows': canonical['rows'], 'sha256': canonical['sha256']}
        report['reopen'] = []
        for _ in range(5):
            start = time.perf_counter_ns()
            store = EngineStore(path, oracle.instance, 'buckets', fullfsync=sys.platform == 'darwin')
            constructor_ns = time.perf_counter_ns() - start
            queries = []
            execute = store.execute
            def observe(sql, params=()):
                queries.append(sql)
                return execute(sql, params)
            store.execute = observe
            activation = store.open_verified()
            assert not activation['historical_scan']
            assert all('WHERE' in q.upper() for q in queries if 'FROM RECEIPT' in q.upper())
            activation.update(constructor_ns=constructor_ns, queries=queries)
            report['reopen'].append(activation)
            store.close()
        store = EngineStore(path, oracle.instance, 'buckets', fullfsync=sys.platform == 'darwin')
        store.open_verified()
        lookups = {'present_ns': [], 'absent_ns': []}
        for i in range(30):
            for present in (True, False):
                label = 'index-only-' + str(i * report['controls'] // 30) if present else 'readonly-absent-' + str(i)
                store.execute('BEGIN')
                store.read_meta()
                started = time.perf_counter_ns()
                row = store.find(store.txid(label))
                lookups['present_ns' if present else 'absent_ns'].append(time.perf_counter_ns() - started)
                assert (row is not None) == present
                store.execute('ROLLBACK')
        report['lookups'] = {k: distribution(v) for k, v in lookups.items()}
        store.close()
        store = None
        report['memory_final'] = own_memory()
        assert report['memory_final']['rss_bytes'] <= 512*MIB
        assert sum(p.stat().st_size for p in args.output.rglob('*') if p.is_file()) <= 256*MIB
        if report['controls'] == COUNT:
            report['status'] = 'completed'
        samples = {key: [] for key in ['outer_ns', 'commit_ns', 'operation_ns',
                                       'index_pages_written', 'index_pages_read', 'staged_node_bytes']}
        peaks = {'rss_bytes': 0, 'wal_eof_bytes': 0, 'cache_bytes': 0, 'dirty_step_pages': 0}
        with gzip.open(args.output / 'publications.jsonl.gz', 'rt', encoding='utf-8') as stream:
            for line in stream:
                value = json.loads(line)
                peaks['rss_bytes'] = max(peaks['rss_bytes'], value['memory']['rss_bytes'])
                peaks['wal_eof_bytes'] = max(peaks['wal_eof_bytes'], value['files'].get('index.sqlite-wal', {}).get('file_bytes', 0))
                native = value['native']['after']
                peaks['cache_bytes'] = max(peaks['cache_bytes'], native['connection']['cache_bytes']['current'])
                peaks['dirty_step_pages'] = max(peaks['dirty_step_pages'], native['step_boundary_dirty_pages_peak'])
                # Four tracemalloc-instrumented observations remain in raw evidence.
                if value['traced']:
                    continue
                for key in samples:
                    number = value['native']['ns'] if key == 'operation_ns' else value['logical'][key] if key.startswith('index_pages') else value[key]
                    samples[key].append(number)
        report['distributions_untraced'] = {key: distribution(values) for key, values in samples.items()}
        report['publication_sampled_peaks'] = peaks
    except Exception as error:
        report.update(status='failed', error=str(error), traceback=traceback.format_exc())
        print(report['traceback'], flush=True)
    finally:
        for connection in (reader, store):
            if connection is not None:
                try:
                    connection.close()
                except Exception as error:
                    report.setdefault('close_errors', []).append(str(error))
        report['final_files'] = files(path) if path.exists() else {}
        write_json(args.output / 'summary.json', report)
    return report['status']


if __name__ == '__main__':
    parser = argparse.ArgumentParser(description=__doc__)
    for name in ['fixture', 'output', 'accepted', 'reference', 'stop']:
        parser.add_argument('--' + name, type=Path, required=True)
    args = parser.parse_args()
    status = run(args)
    print(status, flush=True)
    sys.exit(0 if status in ('completed', 'admission-refused') else 1)
