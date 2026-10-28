# Fixed-profile 32K control pilot

This is the bounded research execution authorized by the
[parent acceptance](../handoffs/2026-10-27-g3-engine-acceptance-32k-authorization.md).
It does not select a production database or implement product Undo/Redo.

## Workload and execution

One fresh attempt on Windows x64 NTFS, Ubuntu x64 ext4 and native Mac ARM64 APFS
publishes 32,768 terminal-negative decisions after a genuine G2 seed. These are
not 32K G2 edits. IDs, intents and canonical receipts retain the accepted 16K
control sequence as an exact prefix. Each target reuses its accepted, SHA-verified
private SQLite 3.51.3 binary and unchanged engine/admission source. No system
SQLite installation is replaced.

The harness is [research_history_pilot32k.py](../../scripts/research_history_pilot32k.py).
Before an attempt it checks accepted source/build hashes, engine identity/default
VFS, native filesystem, free space and the RSS sampler. A separate three-control
observer test checks the oracle, actual connection profile and stale-reader test.
The file-backed connection's VFS is checked before seed publication; an in-memory
preflight connection cannot report a file-control VFS.

An exclusive output directory and ATTEMPT marker consume the attempt. A refusal
or failure is retained, never retried under another identity. Peers check a
controller stop marker before each publication if a target fails unexpectedly.
No unattended GUI is involved. Only one Mac-hosted job runs.

## Unchanged bounds and stop conditions

The existing profile remains 60 entries/leaf, 1,024 leaves, depth 24, eight splits,
zlib level 6, 1 MiB staging allocation, 2 MiB native headroom estimate, 64 MiB
native hard heap, 16,384 database pages of 4 KiB, cache -2,048 KiB, mmap off,
automatic checkpoint off and synchronous FULL. Both fullfsync options are on for
Mac. The engine's 64 MiB WAL trigger and 128 MiB recovery limit remain unchanged.

The point has a 256 MiB retained-file budget and 512 MiB free-space floor;
preflight requires 768 MiB free. Retained size/free space are checked before the
first and each 256th publication. RSS is sampled each publication with a 512 MiB
ceiling; observed WAL EOF must not exceed 128 MiB. These are sampled safeguards,
not guaranteed reservations or within-operation peak bounds. Source/tool storage
is outside the point budget but consumes the same free-space floor.

Unexpected resource exhaustion, SQL/VFS failure, authentication/oracle mismatch,
unsafe WAL retention or bound violations stop the attempt. Explicit pre-write
admission refusal is a measured outcome: verify unchanged authority, no attempted
DML and no open transaction, then audit the committed prefix without resuming it.

## Correctness and measurements

An independent oracle constructs the seed and every expected canonical control
receipt without reading stored receipt bodies. Every returned ID/intent/ordinal
and commitment is checked. At 16,384 controls the canonical manifest must equal
the accepted control manifest. Full audit verifies complete current-directory
coverage and receipt/payload integrity; cold reconstruction must equal the seed.
Read-only accounting independently restores normalized canonical rows after the
writer closes and checks that its source files did not change.

A test-owned reader holds the seed snapshot across the first actual publication.
It sees authenticated absence, then its stale mutation is refused before DML.
Only that owned reader is closed. Five verified reopens record constructor and
activation time and SQL; ordinary receipt reads must remain keyed, with no full
receipt scan. Thirty authenticated present and absent read-only lookups do not
mint any extra decisions.

Per-publication JSONL is streamed through gzip as diagnostic artifact compression;
the database codec remains zlib 6. Samples retain commitments, leaf/depth/split and
logical read/write counts, staging bytes, native before/after telemetry, COMMIT
time, outer operation time, current/peak RSS, and DB/WAL/SHM EOF/native allocation.
`operation_ns` excludes the native telemetry snapshots; outer time includes them.
Neither includes subsequent file observations or artifact serialization. Four
tracemalloc samples include Python staging/observer work and are excluded from
untraced latency distributions. Native SQLite allocations are measured separately.

Read-only milestones at 10K, 16K and 32K capture the same-build curve, including
NOOP checkpoint progress; no midpoint checkpoint is forced. Final explicit audit,
TRUNCATE checkpoint, accounting and lookups are outside publication timings.
Per-operation events cover publication; cumulative native snapshots additionally
include the owned reader and milestone observations. Those activities must not
be silently attributed to publication-only deltas.

SQLite heap high-water, sampled connection cache, SQL-step dirty-list samples,
pager writes, WAL frame-header events, VFS bytes/sync/checkpoint counters and OS
file allocation describe different layers. They are not device writes or proof
of physical durability. Cache high-water zero can mean unsupported. RSS observes
the whole process, not just the store. Python tracing is a partial observation,
not a physical transaction reservation.

Compare the same-build milestones with accepted 10K genuine G2 results and the
older 16K control run, retaining workload, SQLite and timing-harness differences.
Return the packet for parent review; do not run 65K controls, enable 4,096 leaves,
or start full 100K G2, production integration or G4–G7.
