"""G3 SQLite-first research only. No production API, dependency or format selection.

Consume history_index_fixture JSONL on native storage. A conservative full audit
establishes a lookup horizon; this initial experiment does NOT meet bounded startup.
"""
import argparse
import hashlib
import json
from pathlib import Path
import platform
import shutil
import sqlite3
import struct
import time


class Unknown(RuntimeError):
    """Integrity/horizon is insufficient to report definitely uncommitted."""


class Backpressure(RuntimeError):
    pass


def encoded(value):
    return json.dumps(value, sort_keys=True, separators=(',', ':')).encode()


def digest(value):
    return hashlib.sha256(value).hexdigest()


def checkpoint_bytes(records):
    """Reconstruct the existing feature-only Rust cold encoding, not authoring."""
    data = bytearray()
    core = sum(len(bytes.fromhex(v)) for k, v in records.items() if 6 <= bytes.fromhex(k)[0] <= 14)
    for key, value in sorted(records.items()):
        if bytes.fromhex(key)[0] == 6:
            data.extend(struct.pack('<Q', core))
        data.extend(bytes.fromhex(value))
    data.extend(hashlib.sha256(data).digest())
    return bytes(data)


def apply_ops(records, ops):
    result = records.copy()
    keys = [op[0] for op in ops]
    if keys != sorted(set(keys)):
        raise Unknown('duplicate or unordered stable record keys')
    for key, before, after in ops:
        if result.get(key) != before:
            raise Unknown('original-state mismatch')
        if after is None:
            result.pop(key, None)
        else:
            result[key] = after
    return result


def ledger(records):
    # Complete G2 lifetime authority, including reservations, not only counters.
    return {k: v for k, v in records.items() if 1 <= bytes.fromhex(k)[0] <= 4}


SCHEMA = '''
CREATE TABLE payload(hash TEXT PRIMARY KEY, bytes BLOB NOT NULL) WITHOUT ROWID;
CREATE TABLE state(id INTEGER PRIMARY KEY, content TEXT UNIQUE NOT NULL,
 parent INTEGER REFERENCES state(id), base INTEGER NOT NULL,
 payload TEXT NOT NULL REFERENCES payload(hash), oracle TEXT NOT NULL,
 ledger TEXT NOT NULL REFERENCES payload(hash));
CREATE INDEX state_parent ON state(parent);
CREATE TABLE decision(seq INTEGER PRIMARY KEY, tx TEXT UNIQUE NOT NULL,
 intent TEXT NOT NULL, kind TEXT NOT NULL, parent_hash TEXT NOT NULL,
 body BLOB NOT NULL, hash TEXT UNIQUE NOT NULL);
CREATE TABLE authority(singleton INTEGER PRIMARY KEY CHECK(singleton=1),
 seq INTEGER NOT NULL, hash TEXT NOT NULL, head INTEGER NOT NULL,
 cursor INTEGER NOT NULL, redo INTEGER, ledger TEXT NOT NULL);
'''


class Store:
    def __init__(self, path, *, wal_limit=64 * 1024 * 1024):
        self.path = Path(path)
        self.wal_limit = wal_limit  # Temporary resource bound; not logical retention.
        self.db = sqlite3.connect(path, timeout=0.2, isolation_level=None)
        self.db.execute('PRAGMA foreign_keys=ON')
        if self.db.execute('PRAGMA journal_mode=WAL').fetchone()[0] != 'wal':
            raise Unknown('WAL unavailable')
        self.db.execute('PRAGMA synchronous=FULL')
        self.db.execute('PRAGMA cache_size=-2048')
        self.db.execute('PRAGMA mmap_size=0')
        self.db.execute('PRAGMA wal_autocheckpoint=0')
        self.verified = None

    def close(self):
        self.db.close()

    def put(self, value):
        raw = encoded(value)
        key = digest(raw)
        self.db.execute('INSERT OR IGNORE INTO payload VALUES (?,?)', (key, raw))
        if self.db.execute('SELECT bytes FROM payload WHERE hash=?', (key,)).fetchone()[0] != raw:
            raise Unknown('payload hash collision or corruption')
        return key

    def get(self, key):
        row = self.db.execute('SELECT bytes FROM payload WHERE hash=?', (key,)).fetchone()
        if row is None or digest(row[0]) != key:
            raise Unknown('missing or corrupt referenced payload')
        return json.loads(row[0])

    def authority(self):
        row = self.db.execute('SELECT seq,hash,head,cursor,redo,ledger FROM authority').fetchone()
        if row is None:
            raise Unknown('missing authority')
        return row

    def create(self, seed):
        self.db.executescript(SCHEMA)
        self.db.execute('BEGIN IMMEDIATE')
        try:
            records = dict(seed['records'])
            if digest(checkpoint_bytes(records)) != seed['oracle']:
                raise Unknown('seed oracle mismatch')
            payload, life = self.put(records), self.put(ledger(records))
            self.db.execute('INSERT INTO state VALUES (0,?,NULL,0,?,?,?)',
                            (seed['content'], payload, seed['oracle'], life))
            body = encoded({'head': 0, 'cursor': 0, 'redo': None, 'ledger': life, 'state': 0,
                            'state_row': [0, seed['content'], None, 0, payload, seed['oracle'], life]})
            root = digest(encoded([0, 'seed', 'seed', 'seed', '', digest(body)]))
            self.db.execute('INSERT INTO decision VALUES (0,?,?,?,?,?,?)',
                            ('seed', 'seed', 'seed', '', body, root))
            self.db.execute('INSERT INTO authority VALUES (1,0,?,0,0,NULL,?)', (root, life))
            self.db.execute('COMMIT')
            self.verified = (0, root)
        except BaseException:
            self.db.execute('ROLLBACK')
            raise

    def cold(self, state):
        row = self.db.execute('SELECT base,oracle FROM state WHERE id=?', (state,)).fetchone()
        if row is None:
            raise Unknown('missing state')
        base, expected = row
        if not 0 <= state - base < 64:
            raise Unknown('cold decode bound exceeded')
        checkpoint = self.db.execute('SELECT payload FROM state WHERE id=?', (base,)).fetchone()
        if checkpoint is None:
            raise Unknown('missing checkpoint')
        records = self.get(checkpoint[0])
        for index in range(base + 1, state + 1):
            delta = self.db.execute('SELECT parent,payload FROM state WHERE id=?', (index,)).fetchone()
            if delta is None or delta[0] != index - 1:
                raise Unknown('broken content chain')
            records = apply_ops(records, self.get(delta[1]))
        if digest(checkpoint_bytes(records)) != expected:
            raise Unknown('cold state oracle mismatch')
        return records

    def audit(self):
        """Deliberately O(history): a trusted horizon, NOT bounded normal startup."""
        self.verified = None
        self.db.execute('BEGIN')
        start = time.perf_counter_ns()
        count = 0
        try:
            if self.db.execute('PRAGMA integrity_check').fetchall() != [('ok',)]:
                raise Unknown('SQLite integrity check failed')
            if self.db.execute('PRAGMA foreign_key_check').fetchall():
                raise Unknown('foreign key check failed')
            authority = self.authority()
            previous, last_body, last_state = '', None, -1
            for seq, tx, intent, kind, parent_hash, body, root in self.db.execute('SELECT * FROM decision ORDER BY seq'):
                if seq != count or parent_hash != previous or digest(encoded([seq,tx,intent,kind,parent_hash,digest(body)])) != root:
                    raise Unknown('receipt chain/horizon mismatch')
                entry = json.loads(body)
                if entry['state'] is not None:
                    state = self.db.execute('SELECT * FROM state WHERE id=?', (entry['state'],)).fetchone()
                    if state is None or list(state) != entry['state_row'] or state[0] != last_state + 1:
                        raise Unknown('required state index disagreement')
                    records = self.cold(state[0])
                    if self.get(state[6]) != ledger(records):
                        raise Unknown('complete lifetime authority mismatch')
                    last_state = state[0]
                self.get(entry['ledger'])
                previous, last_body = root, entry
                count += 1
            if count == 0 or tuple(authority) != (count-1,previous,last_body['head'],last_body['cursor'],last_body['redo'],last_body['ledger']):
                raise Unknown('authority/receipt horizon disagreement')
            if self.db.execute('SELECT count(*) FROM state').fetchone()[0] != last_state + 1:
                raise Unknown('unbound state index')
            self.db.execute('COMMIT')
            self.verified = (authority[0], authority[1])
            return {'decisions_scanned': count, 'verified_startup_ns': time.perf_counter_ns()-start}
        except BaseException:
            self.db.execute('ROLLBACK')
            raise

    def begin(self):
        if self.wal_bytes() >= self.wal_limit:
            self.db.execute('PRAGMA wal_checkpoint(TRUNCATE)')
            if self.wal_bytes() >= self.wal_limit:
                raise Backpressure('reader pins WAL; release reader before accepting more writes')
        self.db.execute('BEGIN IMMEDIATE')
        authority = self.authority()
        if self.verified != authority[:2]:
            self.db.execute('ROLLBACK')
            raise Unknown('unverified or stale reader horizon; audit required')
        return authority

    def decide(self, tx, intent, kind, entry, authority, *, fault=None):
        seq, parent = authority[0]+1, authority[1]
        body = encoded(entry)
        root = digest(encoded([seq,tx,intent,kind,parent,digest(body)]))
        self.db.execute('INSERT INTO decision VALUES (?,?,?,?,?,?,?)', (seq,tx,intent,kind,parent,body,root))
        self.db.execute('UPDATE authority SET seq=?,hash=?,head=?,cursor=?,redo=?,ledger=?',
                        (seq,root,entry['head'],entry['cursor'],entry['redo'],entry['ledger']))
        if fault:
            fault('before_commit')
        start = time.perf_counter_ns()
        self.db.execute('COMMIT')
        committed = time.perf_counter_ns()
        self.verified = (seq,root)
        if fault:
            fault('after_commit')
        return {'outcome': kind, 'receipt': root, 'seq': seq,
                'commit_ns': committed-start, 'ack_ns': time.perf_counter_ns()-start}

    def receipt(self, tx, intent):
        row = self.db.execute('SELECT kind,hash,seq,intent FROM decision WHERE tx=?', (tx,)).fetchone()
        if row is not None:
            if row[3] != intent:
                raise Unknown('transaction identity reused with different intent')
            return {'outcome': row[0], 'receipt': row[1], 'seq': row[2]}
        return None

    def publish(self, item, *, fault=None):
        tx, intent = 'edit-'+str(item['seq']), digest(encoded(item))
        authority = self.begin()
        try:
            prior = self.receipt(tx,intent)
            if prior:
                self.db.execute('ROLLBACK')
                return prior
            n = item['seq']
            if authority[2] != n-1 or authority[3] != n-1:
                raise Unknown('fixture parent does not match current head/cursor')
            records = apply_ops(self.cold(n-1), item['ops'])
            if digest(checkpoint_bytes(records)) != item['oracle']:
                raise Unknown('independent Rust oracle mismatch')
            full = item.get('checkpoint')
            if full is not None and dict(full) != records:
                raise Unknown('checkpoint differs from delta')
            base = n if full is not None else self.db.execute('SELECT base FROM state WHERE id=?', (n-1,)).fetchone()[0]
            payload = self.put(records if full is not None else item['ops'])
            life = self.put(ledger(records))
            row = [n,item['content'],n-1,base,payload,item['oracle'],life]
            self.db.execute('INSERT INTO state VALUES (?,?,?,?,?,?,?)', row)
            if fault:
                fault('after_payload')
            return self.decide(tx,intent,'committed', {'head':n,'cursor':n,'redo':None,'ledger':life,'state':n,'state_row':row}, authority, fault=fault)
        except BaseException:
            if self.db.in_transaction:
                self.db.execute('ROLLBACK')
            raise

    def reconcile(self, tx, intent):
        # Missing receipt is terminalized under the same exclusive writer lock.
        # A stale snapshot/unknown tail cannot produce a negative answer.
        authority = self.begin()
        try:
            result = self.receipt(tx,intent)
            if result:
                self.db.execute('ROLLBACK')
                return result
            return self.decide(tx,intent,'uncommitted', {'head':authority[2], 'cursor':authority[3],
                'redo':authority[4], 'ledger':authority[5], 'state':None}, authority)
        except BaseException:
            if self.db.in_transaction:
                self.db.execute('ROLLBACK')
            raise

    def move(self, target, tx):
        authority = self.begin()
        try:
            self.cold(target)  # Validate before changing any cursor authority.
            if target not in (authority[3]-1, authority[4]):
                raise Unknown('only adjacent cursor/Redo movement in this linear research chain')
            redo = authority[3] if target == authority[3]-1 else (target+1 if target<authority[2] else None)
            return self.decide(tx,digest(encoded([target])),'cursor', {'head':authority[2],
                'cursor':target,'redo':redo,'ledger':authority[5],'state':None}, authority)
        except BaseException:
            if self.db.in_transaction:
                self.db.execute('ROLLBACK')
            raise

    def wal_bytes(self):
        path = Path(str(self.path)+'-wal')
        return path.stat().st_size if path.exists() else 0


def distribution(samples):
    values = sorted(samples)
    return {name: values[min(len(values)-1,int((len(values)-1)*q))] for name,q in
            [('min',0),('p50',.5),('p95',.95),('p99',.99),('max',1)]} if values else {}


def run(fixture, output):
    output.mkdir(parents=True,exist_ok=False)
    if shutil.disk_usage(output).free < max(fixture.stat().st_size*4,512*1024*1024):
        raise Backpressure('insufficient experiment disk reserve')
    path = output/'history.sqlite'
    store = Store(path)
    timings = []
    with fixture.open(encoding='utf-8') as stream, (output/'transactions.jsonl').open('w',encoding='utf-8') as raw:
        store.create(json.loads(next(stream)))
        for line in stream:
            item = json.loads(line)
            start = time.perf_counter_ns()
            receipt = store.publish(item)
            receipt.update(total_ns=time.perf_counter_ns()-start,case=item['case'])
            raw.write(json.dumps(receipt)+'\n')
            timings.append(receipt)
    n = len(timings)
    checkpoint = list(store.db.execute('PRAGMA wal_checkpoint(PASSIVE)').fetchone())
    logical = store.db.execute('SELECT coalesce(sum(length(bytes)),0) FROM payload').fetchone()[0]
    before_close_wal = store.wal_bytes()
    plans = {name: store.db.execute('EXPLAIN QUERY PLAN '+sql,params).fetchall() for name,sql,params in [
        ('receipt','SELECT * FROM decision WHERE tx=?',('edit-'+str(n),)),
        ('state','SELECT * FROM state WHERE id=?',(n,)),
        ('child','SELECT * FROM state WHERE parent=?',(n-1,))]}
    store.close()
    store = Store(path)
    startup = store.audit()
    lookup = []
    positive = []
    cold = []
    for index in sorted({0,n//2,n}):
        start=time.perf_counter_ns(); store.cold(index);cold.append(time.perf_counter_ns()-start)
    for index in range(100):
        tx,intent=store.db.execute('SELECT tx,intent FROM decision WHERE seq=?', (1+index*n//100,)).fetchone()
        start=time.perf_counter_ns()
        assert store.reconcile(tx,intent)['outcome']=='committed'
        positive.append(time.perf_counter_ns()-start)
        start=time.perf_counter_ns()
        result=store.reconcile('missing-'+str(index),'fixed-intent')
        assert result['outcome']=='uncommitted'
        lookup.append(time.perf_counter_ns()-start)
    authority=store.authority()
    store.move(n-1,'cursor-undo');store.move(n,'cursor-redo')
    assert store.authority()[2:]==authority[2:]
    page_size=store.db.execute('PRAGMA page_size').fetchone()[0]
    pages=store.db.execute('PRAGMA page_count').fetchone()[0]
    result={'status':'passed','research_only':True,'transactions':n,'platform':platform.platform(),
        'python':platform.python_version(),'sqlite':sqlite3.sqlite_version,
        'sqlite_source_id':store.db.execute('SELECT sqlite_source_id()').fetchone()[0],
        'compile_options':[r[0] for r in store.db.execute('PRAGMA compile_options')],
        'configuration':{key:store.db.execute('PRAGMA '+key).fetchone()[0] for key in
            ['journal_mode','synchronous','cache_size','mmap_size','wal_autocheckpoint','page_size']},
        'filesystem_path':str(path.resolve()),'disk_free_bytes':shutil.disk_usage(output).free,
        'timings_ns':{key:distribution([r[key] for r in timings]) for key in ['total_ns','commit_ns','ack_ns']},
        'cold_ns':cold,'positive_receipt_ns':distribution(positive),
        'terminal_negative_ns':distribution(lookup),'verified_startup':startup,
        'payload_logical_bytes':logical,'db_page_bytes':page_size*pages,'wal_bytes_before_close':before_close_wal,
        'db_file_bytes':path.stat().st_size,'passive_checkpoint':checkpoint,'query_plans':plans,
        'limitations':['full-history integrity/semantic audit; bounded startup NOT established',
            'Python SQLite binding/default platform VFS; no production Rust dependency selected',
            'COMMIT wall time includes sync but sync syscalls are not separately instrumented',
            'cache_size is advisory SQLite cache, not Pivot domain/RSS/OS-cache accounting',
            'linear history only; no production Undo/Redo, retention, witness or power-loss claim']}
    (output/'summary.json').write_text(json.dumps(result,indent=2)+'\n',encoding='utf-8')
    store.close()
    return result


if __name__=='__main__':
    parser=argparse.ArgumentParser(description=__doc__)
    parser.add_argument('fixture',type=Path)
    parser.add_argument('output',type=Path)
    args=parser.parse_args()
    result=run(args.fixture,args.output)
    print(json.dumps({'status':result['status'],'transactions':result['transactions'],
                      'verified_startup':result['verified_startup']}))
