# ADR-0023 Stage 2 — cold-history representation comparison

Date: 2026-10-23. **Windows/Linux representation feasibility accepted; Stage 3 research authorized. Production integration and durability remain deferred.**
[Stage 1 acceptance](2026-10-23-cold-history-stage1-review.md), [ADR-0023](../adr/0023-document-transaction-history.md), [experiment plan](../architecture/disk-history-feasibility-plan.md).

[Parent acceptance, qualifications and Stage 3 gate](2026-10-23-history-representation-stage2-review.md).
The measurements below describe the submitted source; original evidence/manifests are preserved.

## Recommendation

Continue research with independently loadable checkpoints plus canonical-record deltas.
Use interval 16 as a conservative *next-experiment setting*, with 64 retained as a
space/retrieval comparator. Neither interval is a production default. The additional bounded-policy experiment checkpoints early at 64 KiB of cumulative
delta payload or 256 changed/deleted records. These thresholds are also research
settings, not production defaults. Keep the full-checkpoint baseline as the correctness
oracle and fallback design. Do not choose a production container yet.

Deltas substantially reduce logical disk bytes at a bounded resident cost. Shared
immutable objects avoid a replay chain, but this particular record/page design stores
more bytes than interval-64 deltas and creates 60,067–84,160 files for 10,000 small
replacements. This does not rule out a better packed object design. It does argue
against adopting this directory-of-objects prototype as a production store.

This is a representation comparison, not an Undo/Redo implementation. Crash/uncertain
commit reconciliation, durable cursor/branch decisions, compaction/GC, storage-full
handling, workbook association and recovery-decline remain unimplemented later gates.
The fixed-interval baseline is preserved separately from the subsequent adaptive-policy
runs, so the original measurements are not presented as validation of modified source.

## Implemented boundary and semantic evidence

All changes are behind `history-experiment`. The ordinary graph, protocol, desktop,
portable transfer and WB5 format retain their existing behavior. No limits or lifetime
identity ceilings were increased. Stage 1's accepted bytes remain the checkpoint
baseline; semantic record boundaries are additional encoder metadata.

The three research stores are complete checkpoints, checkpoints with canonical-record
deltas at intervals 1/16/64, and hash-addressed immutable records with 64-entry index
pages and per-state root descriptors. Keys distinguish ledger reservations, canonical
records, selection descriptors, placements, diagnostics/invalidation, workbook cells,
headers and checksum. Deltas include deletions and coupled metadata; applying them
reconstructs internal data, without authoring replay, parsing or rebinding.

A fixed 96-byte on-disk root entry permits direct transaction lookup. There is no
resident vector of historical transactions. Delta/object writers retain one preceding
record map; objects additionally retain the preceding root's page hashes. Read-only
reopen discards those caches and loads through the disk index. SHA-256 verifies
payloads, objects and the fully assembled checkpoint before the accepted cold adapter
reconstructs/activates it. These checks detect damage; they are not authentication.

The new integration test exercises all five baseline configurations and three
bounded-policy configurations over 71 snapshots and
loads transactions 0/1/15/16/31/63/64/70 after reopening the store. Exact accepted
checkpoint bytes and independently captured CellView/diagnostic ordering/invalidation/
ContentStateId oracles agree for same-session activation and a fresh runtime mapping.
Fixtures include equal-valued distinct bound targets, target retirement/movement,
blank anchors, text, exact negative-zero decimal parts, reversed mixed-dollar MAX,
SUM, invalid and cyclic drafts, Fill, TSV Paste and Clear. A retained invocation still
evaluates after historical activation. Corrupt data rejects without changing the live
state or authority. Byte-only and record-only limits of one force an independently loadable checkpoint
when even one delta cannot fit. A combined 64 KiB/256-record setting exercises
earlier checkpoints. Reopened readers enforce the same cumulative limits before
applying a delta. All owners finish at zero charge. Stage 1's separate real-child
restart, independent-copy, branch-ledger, cancellation/admission and stale-guard proofs
also remain passing; this comparison does not replace those identity contracts.

The long-run harness checks oldest/middle/newest samples at 1,000 and 10,000 changes,
using exact original checkpoint bytes and an independent logical oracle. This is
sampled restoration, not exhaustive restoration of every stored transaction. It
reopens the store without the writer cache before its final load. Original samples
are bounded in memory and retained on disk for inspection.

## Executed matrix and storage

Each platform ran 38 separate processes: four workbook cases × five representations
× 10,000 fixed-identity replacements; four cases × full/delta16/objects × 1,000 mixed
transactions; small/large × full/delta16/objects × 1,000 synchronized replacements.
There are 76 successful runs total. The mixed pattern includes up-to-256-position
Fill, native row clipboard Paste, TSV Paste, single edits and row batches. Each batch
is one transaction. The 256-cell formula case preserves its formula columns. Cases
are 16 cells, 256 cells, 256 mostly-formula cells, and the accepted large 1,024-cell
case; the latter does not imply every theoretical quota maximum is simultaneously
admissible. Fixed lifetime floors are 16/256/256/1,024 throughout the long runs.

Logical stored MB at 10,000 single-cell replacements (decimal MB, including the
960,000-byte root index; matching Windows/Linux counts):

| Case | Full | Delta 1 | Delta 16 | Delta 64 | Objects |
| --- | ---: | ---: | ---: | ---: | ---: |
| 16 cells | 46.54 | 77.37 | 10.67 | 7.34 | 53.89 |
| 256 cells | 688.90 | 1,113.33 | 75.41 | 23.60 | 142.17 |
| 256 formula-heavy | 1,481.22 | 1,905.65 | 124.93 | 36.04 | 142.25 |
| 1,024 cells | 2,747.38 | 4,431.33 | 282.79 | 75.69 | 157.94 |

Delta-1 stores stable keys and framing on every checkpoint, explaining why it is
larger than raw complete checkpoints. Delta-64 reduces logical bytes by about
84–98% versus full checkpoints for these replacement workloads. At 1,000 mixed
transactions, full/delta16 bytes are 4.65/1.89 MB (small), 68.89/19.13 MB (medium),
148.12/14.64 MB (formulas) and 274.74/41.14 MB (large). Broad edits reduce the delta
advantage; they do not remove it in these cases. Object mixed-workload sizes vary
slightly with randomly allocated UUID sort/page placement across independent runs.

These are file lengths, not physical allocated sectors or filesystem metadata.
The object directory is its on-disk lookup index; filesystem index/cache costs are
outside the allocation domain and were not measured. No physical-space, garbage
collection or compaction result is claimed. Generated datasets are removed only
after successful receipts; original sampled checkpoints/oracles remain as evidence.

## Resource accounting

Stage 1's fixed 16 MiB writer buffers and 8 MiB ledger admission are replaced by
work-scaled reservations. Writers admit geometric byte/record-vector growth before
allocation, preserving both old/new charges during replacement. The existing 16 MiB
image bound remains a research safety limit. Ledger ownership reserves
`512 * node_floor + 128 * selection_floor + 4096` bytes; observation/decode also
admit temporary maps before replacement. Core/workbook decoder scratch reserves
`12 * input_bytes + 4096` and `8 * input_bytes + 4096`, respectively. These are
conservative bounds, not measured scratch allocation sizes or production constants.

Records reserve payload copies plus per-record map allowances. Actual byte/vector
capacity is reported separately; BTree allocator overhead is covered by reservations,
not measured by a heap profiler. Domain allocation/reservation/charge and simultaneous
phase peaks are distinct metrics; none is process RSS or OS page-cache memory.

Delta-16 replacement cases at 10,000 transactions, bytes (matching both platforms):

| Case | Final capsule | Max encoder vector capacity¹ | Resident records | Record buffer capacity² | Idle allocated | Idle reserved | Restore phase peak charge³ |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| small | 4,558 | 15,360 | 75 | 9,358 | 30,108 | 1,829,532 | 2,015,239 |
| medium | 68,794 | 245,760 | 1,035 | 135,034 | 365,156 | 2,326,644 | 4,585,031 |
| formulas | 148,026 | 376,832 | 1,035 | 214,266 | 549,540 | 2,485,108 | 6,074,737 |
| near | 274,642 | 983,040 | 4,107 | 537,490 | 1,443,092 | 3,917,988 | 12,824,375 |

¹ Output bytes plus record-span vector capacities, not all concurrent encoder scratch.
² Payload vector capacities plus key/value slot sizes; excludes BTree node overhead.
³ Whole-domain phase peak: live workbook, writer cache/latest ledger, input and privately
reconstructed workbook coexist. It is not the incremental decoder allocation alone.

For delta16, encode/append/load phase peak charges span 1.91/1.93/1.97 MB (small)
to 7.70/8.74/10.39 MB (large). Restore peaks span 2.02–12.82 MB. Idle charge spans
1.86–5.36 MB, including the existing runtime. Relative to full-store idle charge,
the preceding-record map adds roughly 32 KB (small) to 1.60 MB (large). Object
page-cache charge is a further 256–8,320 bytes in these cases. Full stores retain
no record cache. The same record/page counts, capacities and idle charges hold at
1,000 and 10,000 edits for all 20 long-run configurations on each platform.

Existing clipboard scratch influences mixed-workload mutation peaks (up to about
14.69 MB), separately from serialization/reconstruction. Stage 1's ~94 MB peak
included loose research reservations and is preserved as historical evidence; these
new numbers are different phase/workload measurements, not a revision of that run.
All baseline processes and the semantic test finish with zero remaining domain charge.

## Work, latency and retrieval qualifications

Every transaction still exports a complete accepted checkpoint and visits its current
records before constructing a delta/object root. **Mutation-local incremental
serialization is not implemented.** Lower disk bytes do not prove lower full-export
CPU or work proportional only to the changed cells. This common cost is measured
separately from record planning, object I/O, data/root writes and synchronization.

For 10,000 replacements, Windows mean full-export time is about 18–92 µs for the
small case and 1.15–1.90 ms for the large case, depending on mode/run conditions.
Delta16 mean append time is 39 µs/3.26 ms (small/large); Linux 37 µs/2.31 ms.
These append times exclude full export and mutation. The synchronized companion
runs average 2.93/6.95 ms for Windows delta16 append and 6.28/12.49 ms for Linux.
Object sync and file creation are appreciably more expensive in this implementation.
The raw receipts separate `sync_ns_total`; append p95 is a logarithmic histogram
*upper bound*, not an exact quantile. Maxima and sampled load/reconstruct times are
also retained. This harness records p95 and totals/maxima, not p50 distributions.

Direct root lookup is constant entry size. Full/object loads have no delta chain;
delta16 samples use at most 15 deltas; delta64 samples reach 51 (configured bound 63).
The reader currently rereads the target payload while fetching its base/chain; measured
read counts include that overhead. In the large replacement case, sampled maximum
read bytes are 274,738 (full), 886,266 (delta16/64) and 544,309 (objects).
These samples do not exhaust worst-case chain/byte combinations.

The preserved baseline comparator uses fixed intervals. The final source additionally
supports cumulative delta-payload and changed/deleted-record limits. Before appending
a delta that would exceed either limit, it instead writes an independently loadable
checkpoint, resets both counters and records that checkpoint as the next chain base.
Readers check cumulative limits before applying each delta. The checkpoint itself
remains separately bounded by the existing image/map limits; read-call overhead and
the reader's duplicate target read are not part of the delta-payload budget.

The additional `bounded64` experiment combines a maximum interval of 64 with 64 KiB
and 256 records. Each platform runs 12 additional processes: four 10,000-replacement
cases, four 1,000-mixed cases, two 1,000-sync cases and two fixed-delta64 regression
cases on final source. The replacement workload remains within both work thresholds
and matches fixed-delta64 storage/cache results; mixed edits force earlier checkpoints.
For 1,000 mixed transactions (Windows; logical counts match Linux), the results are:

| Case | Stored MB | All checkpoints | Work-triggered checkpoints | Sampled max delta chain |
| --- | ---: | ---: | ---: | ---: |
| Small | 1.76 | 47 | 31 | 25 |
| Medium | 33.63 | 266 | 250 | 2 |
| Formula-heavy | 11.20 | 47 | 31 | 16 |
| Large | 121.89 | 266 | 250 | 2 |

The strict 256-record cap trades substantial extra medium/large storage for less
replay work compared with fixed delta16. This is useful policy evidence, not a reason
to adopt that cap universally. The baseline and final-source experiments total
**100 successful zero-charge processes** and **792 verified original sample hashes**.

Tiny byte-only and record-only limits test single-delta fallback;
the realistic setting also exercises earlier checkpoints on broad edits. These are
explicit retrieval bounds, not an approved production cache/scheduling policy.
Mutation-local serialization and production checkpoint admission/failure behavior
still need further work. No fixed interval or numeric threshold is approved by this
packet.

Windows data was on local fixed NTFS (J:); Linux data/source were on native WSL
`/var/tmp` (reported ext2/ext3 family), with build cache and copied receipts on J:.
The OS filesystem cache was not flushed: dropping a decoded root/cache is not a
cold filesystem read. Windows/Linux matrices ran concurrently on shared hardware,
with some regression-test overlap, so timings are exploratory, not isolated platform
comparisons or latency guarantees. `sync_data` was called on new objects and data,
then the root file in synchronized runs. There is no directory synchronization,
commit-decision protocol or power-loss/crash guarantee. Native macOS/APFS remains
required before cross-platform durability claims; it is unnecessary to repeat manual
desktop checks for this feature-only Rust representation research.

## Validation and review boundary

Windows workspace default/all-feature suites: **346 / 378** tests pass. The final
capture assertion also passes in the focused representation test and final full
all-feature rerun. Linux workspace default/all-feature suites: **347 / 379** pass. Strict all-target,
all-feature Clippy passes on both platforms; final adaptive-policy reruns are recorded
separately from the baseline gates. Formatting and diff checks are part of the final source freeze.
No managed/Desktop/Native AOT change is present, so those gates were not rerun.

Early local fixture failures (occupied Move destination; a retained test Blob charge)
and an object-root count decoder error were corrected with assertions preserved.
Initial build/lint failures and passing reruns are retained alongside final logs.
A final sandbox-only CLI regression attempt failed with localhost access denied;
the authorized rerun and subsequent final checks are preserved. A Get-Volume probe was denied; the read-only .NET DriveInfo query reports fixed NTFS.

Evidence: `artifacts/history-stage2/`, especially per-platform `matrix-summary.json`,
all `.jsonl`/stderr receipts, `original-sample-hashes.json`, copied original samples,
`matrix-verification.json`, final test/lint logs and source/build/evidence manifests.
Each platform has 38 baseline plus 12 final-source runs, with 308 plus 88 verified
original sample hashes. Fixed-baseline build-input hashes are preserved separately
from the final source/build-input manifests. The final all-feature suites exercise
the adaptive additions; fixed-baseline measurements retain their original source
qualification. Baseline default-feature suites remain applicable because all later
Rust changes are gated by `history-experiment`.
The Linux originals were hashed on native storage before copying to the share.
Earlier Stage 1/inventory evidence and manifests are unchanged.

Review this logical representation recommendation and its measured research policy
before expanding the experiment. Stage 3 must prove durable decision/fault/restart
behavior; Stage 4 must prove independent file/save association and non-destructive
recovery decline. Neither this packet nor a favorable byte comparison authorizes
production Undo/Redo, a format migration or a container choice.
