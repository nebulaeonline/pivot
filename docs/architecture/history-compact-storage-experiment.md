# G3 compact-storage comparison

Parent accepted this compact-storage research and the size objective on 2026-10-27.
G3 remains incomplete; page-layout, metadata and authenticated-growth research
are authorized next. [Acceptance and next scope](../handoffs/2026-10-27-g3-compact-storage-acceptance.md).

2026-10-27. Research only; no production schema, backend, index or compression
codec is selected. G1/G2, WB5, session/UI/protocol and product limits are unchanged.

## Separate obligations

The current workbook is exported independently through the unchanged ordinary WB5
path. History consists of authoritative G2 before/after record deltas and periodic
checkpoints. Transaction/state locations and receipt commitments are measured as
metadata, not folded into a misleading workbook-size baseline.

`research_history_bytes.py` reads the preserved trie database and original fixture
without modifying them. It separates payload classes, logical binary/JSON/hex and
compressed bytes, distinct ledger payloads versus references, live/superseded trie
nodes, SQL value bytes, dbstat payload/unused/header/index pages and freelist.
Platforms without dbstat report the physical residual rather than invent table
page allocations. Changed-cell denominators count actual G2 tag-16 record changes,
not every coordinate in a Paste/Fill rectangle or number of spreadsheet rows.

## Compact SQLite candidates

Both modes use the same G2 streams and full independent checkpoint oracle. A
strict binary research encoding retains all stable 33-byte record keys and both
optional before/after values, including null versus empty. Each resolved delta
is compressed independently; full logical checkpoints occur every 64 edits.
Payloads are hash-addressed, and identical lifetime payloads are reused. Complete
lifetime authority is serialized only when its G2 records change. All edit deltas
remain retained even when a checkpoint is also stored.

The backend prepares/compresses mutation-local bytes and writes compact immutable
receipts with indexed IDs/state locations. It does not write a full workbook on
ordinary edits. Full-state oracle validation is explicitly separate research work;
its time is reported separately from encoding and backend publication. Checkpoint
creation is the deliberate periodic exception.

Plain mode uses SQLite receipts and checksums without claiming authenticated
negative lookup. Ordinary uncertain absence returns Unknown. Explicit complete
audit under the same writer lock can establish and terminalize a negative. General
new unknown-ID admission also requires that audit. The ingestion measurement uses
an explicitly known-fresh generated fixture stream; its cheap append timings are
not advertised as a safe general request-admission cost. A deleted historical
terminal negative must not allow a late original edit.

Bucket mode additionally stores a fixed directory of 256 SHA-256 commitments in
current publication authority. Each bucket contains sorted unique residual receipt
key hashes and full row digests (63 bytes per entry). State locations resolve to
immutable receipt rows whose state identity and content references are authenticated
by the same directory; there is no second per-state proof map. SQL misses must agree
with a verified complete bucket before becoming terminal negatives.

All payloads, receipts, updated bucket/directory and head/cursor/Redo/latest lifetime
authority share one SQLite transaction. Receipt row digests bind immutable intent,
state references and previous receipt digest. Publication binds the expected
instance, fixed research epoch, latest receipt, complete lifetime reference and
commitment directory. This uses the same atomic-writer/collision/current-publication
trust boundary as the accepted trie; coherent root replacement/rollback remains
outside the guarantee.

## No retention or custom reclamation policy

Every historical receipt and content delta is retained. A historical state's proof
is resolved through its immutable receipt under the current directory. Superseded
directory/bucket representations need not be retained as application objects:
active SQLite reader transactions see the older root **and** older bucket rows in
their consistent snapshot. A stale snapshot cannot terminalize a current negative;
writer admission checks the verified horizon after acquiring exclusion.

Tests force a writer update into the exact bucket held by an older reader, verify
the old proof still works, and refuse reconciliation after that reader becomes
stale. SQLite owns physical MVCC/WAL version retention and later page reuse. This
does not implement history pruning, GC, file compaction, secure erasure or G5 reader
leases. No historical identities are deleted to obtain smaller files.

Each bucket has a 64 KiB **research admission budget**. Overflow blocks; it does not
discard receipts. Fixed 256-way partitioning is not an unbounded long-term product
index design. Proof work grows within a bucket until that bound; 100K/general scale
and a reviewed growth strategy are not established. The directory itself is 8 KiB
per current publication, not 8 KiB retained per historical edit. WAL still records
its updates, so write amplification is measured separately from retained size.

## Failure and resource boundaries

Preserve exact-intent replay, terminal negatives, stale-horizon refusal, current
content/lifetime validation and per-handle integrity quarantine. Only complete audit
can release a known integrity fault; ordinary opening cannot silently do so. Old
unvisited damage remains lazy. Compressed payload hashes, versions, lengths and
before-images validate before activation. Real process-death and SQLite exhaustion
tests retain failures rather than retry to green.

WAL/FULL, native filesystems, old engine qualification/WAL-reset caveats and explicit
Mac fullfsync policy remain. SQL API timing, proof computation, COMMIT and residual
Python work are distinguished; exact VFS sync/page-cache ownership remains unknown.
Process write_bytes is not a device-write claim. Separate compressed and raw binary
logical denominators; report authentication-only and all-metadata overhead, including
receipt rows. Do not select the favorable denominator to declare the 20% target met.

## Compression options supplied by the owner

The owner identified an existing .NET zstd wrapper at **`J:\coding\dotZstd`**;
Rust zstd is also an available implementation direction. Retain these options for
a controlled codec comparison if needed. This slice deliberately uses standard-
library **zlib level 6** to isolate binary encoding and index costs without adding
a permanent dependency. It does not select zlib over zstd, validate the wrapper's
platform/AOT behavior or claim zstd alone will solve receipt/index overhead.

[Accounting, comparison results and remaining targets](../handoffs/2026-10-27-g3-storage-efficiency-results.md).
