# G3 fixed engine/VFS and pre-write admission experiment

2026-10-27. Research-only implementation of the
[accepted assignment](../handoffs/2026-10-27-g3-accounting-acceptance-engine-qualification.md).
The 1,024-leaf profile, zlib 6, canonical receipts and original identity semantics
remain unchanged. No production package, runtime upgrade or backend selection.

## One reviewable source, actual connections

`scripts/research/sqlite-source.json` pins official SQLite **3.51.3**, source ID
`2026-03-13 10:38:09 737ae4a34738ffa0c3ff7f9bb18df914dd1cad163f28fd6b6e114a344fe6d618`.
The archive SHA-256 and amalgamation SHA3-256 are checked before native compilation.
The latter matches the [official release record](https://www.sqlite.org/releaselog/3_51_3.html),
which explicitly identifies the WAL-reset correction. This qualifies that known fix
by exact source, not an assumption about vendor version/backport status or proof
that all SQLite bugs have been eliminated.

`research_sqlite_build.py` compiles `scripts/research/sqlite_bridge.c` and the same
amalgamation on native Windows x64, Ubuntu x64 and Mac ARM64. Compiler/platform details
and full SQLite compile options are retained. Common explicit options enable dbstat
and memory status, disable extension loading and mmap, and enable serialized SQLite.
Public SQLite symbols are static inside this translation unit; only the small `pv_*`
shim is exported. Calls cannot silently interpose against Python's installed SQLite.

`research_sqlite_native.py` provides the controlled harness's execute/cursor interface.
`EngineStore` injects this factory into the existing research store and its schema
model connections. It does not globally replace Python's SQLite module. Exception
classes come from that module, not database connections. The adapter checks the
actual loaded source/version against the pin. It also checks column-materialization
OOM explicitly; failed BLOB/text allocation must not become an empty value.

Actual selected VFS is `pivot-observe`, delegating to **win32** on NTFS and **unix**
on ext4/APFS. The delegate name returned by the connection's file-control query is
recorded alongside the selected wrapper, rather than inferred from the operating
system. File locking, shared-memory, read/write, sync and remaining VFS operations
are forwarded to the real delegate. Installed engines and earlier evidence remain
untouched; this is a private compiled research library.

## What is measured

The actual connection exposes SQLite memory/current high-water, native heap limit,
cache/schema/statement bytes, lookaside, cache hit/miss, pager-write and cache-spill
counters. The private allocator wrapper supports explicit allocation failure. The
64 MiB hard limit applies to SQLite-managed allocations in this private library,
not Python objects, CRT/OS metadata, filesystem cache or all process memory.

VFS counters distinguish main, WAL and other files: attempted reads/writes/bytes,
truncate calls, xSync calls/flags, delegate wall time and injected I/O errors. WAL
frame-header write events count complete 24-byte headers in writes for the pinned
4 KiB page profile, including rewrites/uncommitted attempts. The connection's commit
hook separately records committed WAL-prefix frame counts. Checkpoint start/end
file controls and returned PRAGMA progress are observable.

SQLite's [connection-status API](https://www.sqlite.org/c3ref/db_status.html) provides
pager/cache counters; the [WAL hook](https://www.sqlite.org/c3ref/wal_hook.html) reports
the current committed prefix, not a cumulative write counter. Zero high-water values
for status operations that do not support them are not interpreted as zero peaks.
Current cache bytes are sampled; the allocator's memory high-water is library-wide.

Pinned-source introspection walks SQLite's dirty list **after SQL step boundaries**,
under SQLite/B-tree mutexes. This is deliberately version-specific research code,
not a public ABI. The maximum observed dirty list can miss within-statement peaks.
VFS/allocator/dirty counters are private-library process totals; cache/pager status
is queried on the actual connection. The harness uses one calling thread; wrapper
counters/fault controls are not qualified for concurrent caller threads.

xSync success/time measures the real VFS delegate boundary, not every OS syscall,
directory sync, backing-storage operation or physical durability. FULL synchronous
mode and `SQLITE_SYNC_FULL` flags are different settings. Windows/Linux WAL calls
use normal VFS sync flags in this configuration; Mac benchmark fullfsync enables
full VFS sync flags. Process-death tests are not power-cut tests.

Raw native events are streamed around each whole publication/reconciliation operation.
Postprocessing separates the first genuine G2 publications from later probes. Generic
backend timings include ctypes, dirty-list sampling, telemetry and JSON sink overhead;
the native operation timer excludes snapshots/sink output. Neither predicts a Rust
binding's cost or ranks platforms with different storage and sync behavior.

## Admission before payload/receipt mutation

`research_history_engine.py` implements `EngineStore` on the accepted packed layout.
After writer-excluded horizon verification and authenticated absence, it calls the
reviewed pure insertion planner with the **existing** configuration. It allocates
the admitted byte buffer, plans all splits, stages leaf bytes and branch dependencies,
and checks the staged node/byte counts against the independent model before payload
or receipt DML. Existing exact retries return their receipt without new staging.

Publication fills the one receipt-digest placeholder, computes dependent commitments
bottom-up and writes the staged nodes. No split planning or new permanent path
generation is deferred until after payload mutation. Bootstrap is separately admitted
before schema/seed creation; the normal transaction plan targets an existing leaf.
Cursor controls get the same admission before their receipt insert.
Opening/configuring a fresh SQLite file can initialize its file/header before
`create`; the bootstrap refusal guarantee concerns history schema/payload DML, not
the absence of any filesystem object or configuration write.

The default scratch buffer is an actual **1 MiB Python allocation**, with a configurable
smaller test limit. It stages encoded index bytes; it is not a cap on every Python
object or temporary hash buffer. Native SQLite admission requires **2 MiB headroom**
under the enforced **64 MiB library heap cap**. The headroom check is not a complete
future-allocation reservation: later SQLite OOM remains possible and must roll back.

Fresh experimental databases enforce `max_page_count=16384` with 4 KiB pages (64 MiB),
disable mmap and automatic WAL checkpoints, and use WAL/FULL. Mac measurement runs
explicitly set both fullfsync and checkpoint_fullfsync. The generic failure tests use
the portable FULL profile; they do not prove hardware durability for either setting.

Pre-write scratch/headroom/planning failures release the writer transaction without
data DML. The old base class's pre-transaction Backpressure behavior is extended to
roll back backpressure arising inside this new writer transaction. Partial SQL/native
allocation failures also roll back and invalidate mutation authority until verification.
COMMIT/sync failures require reconciliation after reopen; an error cannot be labelled
definitively absent merely because no acknowledgement was returned.

The WAL checkpoint trigger remains 64 MiB and recovery preflight remains 128 MiB.
Neither is promoted to a hard physical WAL quota. A pinned reader causes pre-write
backpressure when checkpointing cannot reduce the file. DB/heap caps and logical
index staging do not reserve physical transaction frames, filesystem blocks, Python
heap or host/hypervisor storage. These limitations remain explicit.

## Qualification boundary

New tests exercise actual source/VFS telemetry, canonical equivalence, replay/intent,
scratch and native-headroom refusal before DML, injected allocation failure after
payload staging, a real lowered heap cap during column materialization, SQLITE_FULL,
partial planned-node errors, pinned readers, stale writers/snapshots, invalid profiles,
corrupt pages, VFS write/sync errors and child process death before/after COMMIT.
Original hierarchy assertions also rerun as regressions of the default store path.

Genuine 1K/10K G2 fixtures retain independent reconstruction, complete audit, exact
canonical payload/receipt manifests and cold images from accepted native baselines.
No 32K/65K control or full 100K G2 run occurs. The
[results and scaling decision](../handoffs/2026-10-27-g3-engine-admission-results.md)
separate measured qualifications from remaining resource/durability gaps.
