# ADR-0023 Stage 3 — decision durability and restart research

Date: 2026-10-23. **Windows/Linux process-termination and persisted-recovery experiment
implemented; parent review pending. No power-loss, macOS or production history claim.**
[Stage 2 acceptance and authorization](2026-10-23-history-representation-stage2-review.md),
[ADR-0023](../adr/0023-document-transaction-history.md),
[experiment plan](../architecture/disk-history-feasibility-plan.md).

## Result and recommendation

Continue with checkpoint-plus-canonical-record-deltas as the preferred research
candidate. The persisted decision experiment recovers coherent state, cursor, Redo,
head and lifetime authority after the tested interruptions. Definitely uncommitted
branch replacements preserve Redo; complete decisions recover without replay even
when no acknowledgment was emitted. Mutation remains blocked while the exact
TransactionId is uncertain, including when reconciliation itself fails.

Request parent review of this evidence before advancing to Stage 4. Do not select a
production container or integrate Undo/Redo, RPC, desktop or Save. The single-file
layout below is controlled research I/O, not a production packaging decision.
Incremental serialization, efficient transaction-ID indexing, startup acceleration,
production admission policy and filesystem power-loss guarantees remain unresolved.

## Research protocol

All additions are behind `history-experiment`. `durability::Candidate::capture`
separates authoritative record construction/full export from decision persistence.
The journal accepts that candidate without parsing or rebinding formulas. A later
mutation-local encoder can replace record construction under separate review; the
full-export path is not declared a production transaction requirement.

The research journal has a checksummed 128-byte version/configuration header. Each
operation appends a checksummed 256-byte prepare header, an encoded payload and a
64-byte decision footer binding the prepare hash and its exact file offset. Prepare
metadata binds sequence/TransactionId, previous decision offset/hash, ContentStateId,
content parent, head/cursor, Redo pointers, latest lifetime-authority payload,
checkpoint/delta links and cumulative replay work. No native document/clipboard/
protocol format is changed. Unknown or corrupted research versions reject.

An edit payload is a complete sorted canonical-record checkpoint or a canonical
record delta against the preceding committed *authority payload*, including a
previously discarded branch when needed. Such dependency records stay on disk;
there is no collection/compaction in this experiment. Delta chains are limited to
at most interval minus one and cumulative delta bytes/record operations. Checkpoints
remain independently loadable and bound by the existing image/record limits.

Before appending an edit, the harness separately reconstructs/validates its candidate
and checks that its ledger can reconstruct the previous committed authority. This
rejects a mismatched document or incomplete ledger before writing. It is deliberately
expensive research validation, not a proposed production edit path. Its timing is
reported separately from full export and record construction.

The append sequence is:

1. Hold the actual file's exclusive OS lock; reject a duplicate TransactionId or
   unchanged ContentStateId before appending.
2. Write prepare metadata and payload, then call `sync_all` on the file.
3. Write the decision footer, then call `sync_all` again.
4. Only after successful synchronization return the receipt that permits publication
   and acknowledgment. The worker retains its old live workbook throughout a failed
   attempt; its independent oracle must remain unchanged.

Any error after writing may have started marks that exact operation pending. Commit,
Undo, Redo, restore, raw state/authority inspection and public outcome lookup
reject while pending; receipt inspection returns no authoritative receipt. Reconciliation
validates actual persisted records, reconstructs authority/current/head, synchronizes
the accepted prefix and only then classifies the exact TransactionId. A different
ID cannot unblock the writer. Failed reconciliation retains the pending barrier.
A complete decision observed before its original sync completed is accepted only
after recovery establishes synchronization; the experiment does not assert that
unsynced bytes disappear when a process dies.

A fresh process scans the persisted chain and verifies payload hashes, decision
binding and transition invariants. It reconstructs the latest ledger separately
from the selected historical content using the accepted cold adapter, issuing fresh
runtime authority rather than restoring process-local request tokens. No parent
process owns a workbook or a surviving transaction catalog for the recovery probe.

Only a partial final prepare/payload/decision with no complete decision is trimmed,
after validation of the committed prefix, under the exclusive lock and followed by
synchronization. A complete but corrupt decision, missing/damaged payload, invalid
metadata/transition or malformed version fails closed without truncation or a silent
rollback to an earlier committed state. This is not history compaction or deletion
of recoverable committed work.

## Cursor, branches and identity authority

Edits create a new content node parented by the committed cursor and reset Redo
only through the new durable decision. Undo/Redo decisions have no state payload;
they atomically change the cursor while preserving the latest committed lifetime
ledger and branch head. Disk-linked Undo records provide the Redo stack; operations
do not need a resident timeline or a scan from the branch tip to find the next Redo.
A new branch retains older operation receipts for reconciliation; they are not
replayed or resurrected as the active Redo branch.

After restart, tests compare the entire saved canonical capsule, independent typed
CellView/diagnostic ordering/invalidation/ContentStateId oracles and exact authority
capsule. They also check the expected lifetime floor and non-reuse of a discarded
B-only cell identity. Ledger authority is not rolled back when the cursor moves.
Stage 1's same-session exact runtime-ID and fresh-process mapping contracts remain
unchanged; no NodeId representation or identity limit changed.

## Executed fault and acknowledgment matrix

Each platform completed **171 primary scenarios plus 12 acknowledgment scenarios**:

| Per-platform scenarios | Count | Result |
| --- | ---: | --- |
| Fault/error/space-limit cases recovering old timeline | 96 | Exact previous receipt, cursor, Redo and authority |
| Fault/error cases recovering an unacknowledged complete decision | 42 | Exact operation and chosen state/ledger, no replay |
| Corrupt/missing data or invalid metadata/version | 28 | Reject/quarantine; file hash unchanged |
| 1,000-edit synchronized timing/reopen runs | 5 | Exact last state/receipt and zero-charge finish |
| Successful client acknowledgment followed by independent reopen | 12 | Exact acknowledged TransactionId/content recovered |

That is **366 completed scenarios across Windows and Linux**. Per platform, 63
primary fault workers were terminated by the parent (TerminateProcess/SIGKILL)
at a signaled hook, without Rust cleanup; 28 additional corruption workers were
terminated after decision sync before their files were damaged. Error cases use
real persisted writes and injected I/O outcomes, not test-only commit booleans.
Each completed non-killed worker/probe asserts zero remaining allocation charge.
Killed workers deliberately do not emit a finish receipt.

The primary delta16 candidate exercises all twelve points for edit-after-Undo,
Undo and Redo, with both parent termination and returned-error paths: before prepare;
half prepare; complete prepare; half payload; complete payload; before/after payload
sync; half decision; complete decision; before/after decision sync; before acknowledgment.
Full checkpoints, delta64 and adaptive64 repeat selected payload-sync, partial-decision
and durable-unacknowledged cases. Existing Stage 2 broader adaptive checkpoint tests
remain separate evidence; these cursor/branch fixtures are not a new adaptive-policy
performance study.

The space-limit seam permits only 0/128/256/257 further bytes, writes any admitted
prefix to the actual file and returns StorageFull when the next segment cannot fit.
This exercises partial-write/exhaustion recovery without filling the host volume.
It is injected storage exhaustion, not a test on a physically full filesystem.

Corruption covers payload flips, absent payload bytes with the intact decision kept
at its referenced offset, damaged decisions, prepare metadata damage, wrong ledger
and parent pointers with checksums/decision binding recomputed, and unsupported
version/magic. The missing-payload test models a hole/zeroed region inside the one-file
layout; there are no separately named payload files in this prototype. Checksums
provide integrity detection, not authentication against arbitrary malicious rewriting.

Successful acknowledgment controls use a non-binding 1 MiB write allowance: the
worker emits `ACK TransactionId ContentStateId elapsed_ns` only after ordinary synced
success. A separate process then loads and checks that exact operation. Returned
errors and killed workers emit no ACK; reconciliation receipts are labeled separately.
The independent oracles are fixture files, not surviving in-memory roots.

Rust tests additionally verify OS locking, no-op/duplicate rejection with Redo intact,
acknowledged branch/cursor reopen and failure of reconciliation after payload damage.
The latter must remain blocked, and subsequent fresh open must quarantine the file.
Existing Stage 1/2 semantic tests and ordinary regressions remain passing.

## Timing, memory and scalability limits

The five timing cases per platform retain a baseline plus 1,000 changed transactions:
full/delta16/delta64/adaptive64 at 16 total cells and delta16 at 1,024 total cells.
All calls use real file synchronization. The record map is one latest-authority map;
resident metadata is constant, and retrieval chains stay within the configured bounds.
The experiment does **not** establish a production cache/index policy.

Measured means for delta16 (milliseconds, 1,000 changed edits; metadata totals also
include the baseline commit, so their divided values are approximate):

| Platform / cells | Full export | Research validation | Record construction | File synchronization | Staging-to-receipt boundary |
| --- | ---: | ---: | ---: | ---: | ---: |
| Windows / 16 | 0.025 | 0.344 | 0.036 | 2.731 | 4.236 |
| Linux / 16 | 0.025 | 0.375 | 0.036 | 4.675 | 5.881 |
| Windows / 1,024 | 1.152 | 25.261 | 2.781 | 3.112 | 34.598 |
| Linux / 1,024 | 1.138 | 20.738 | 3.535 | 5.437 | 32.656 |

The last column includes staging, ledger observation, duplicate lookup, export,
validation, construction, writes and sync; it is not RPC/UI end-to-end latency.
Raw write totals, maxima and reconciliation timings are in receipts. No p50/p95
or acceptable-interactive-latency claim is made. Windows/Linux ran concurrently
on shared hardware, with regression overlap and warm/uncontrolled OS caches.
These are exploratory measurements, not isolated platform comparisons.

The 1,024-cell journal is 28,788,186 logical bytes after the baseline and 1,000
edits on both platforms. Small-case full/delta16/delta64/adaptive64 file lengths
are 8,937,096 / 1,356,318 / 976,464 / 976,464 bytes. These are file lengths, not
physical allocation or filesystem metadata measurements.

Startup and reconciliation scan the journal linearly with bounded resident memory.
Duplicate-ID preflight and explicit lookup also scan disk history. **Per-edit duplicate
lookup is therefore linear in history length in this research implementation.**
A durable bounded-memory lookup index and faster verified startup need design and
measurement before production. The 1,000-edit runs do not supersede Stage 2's
10,000-edit representation evidence or prove timeline-independent commit latency.
No fixed logical history retention cap was added.

Whole-domain timing-run peaks are approximately 18.69 MB (small) and 22.18 MB (large),
including fixture authoring/range scratch and simultaneous oracle/live/reconstructed
owners. The reported `idle_charge` is sampled during final verification with both
original and restored workbooks and expected bytes alive; it is **not steady-state
cache memory**. Reserved scratch, encoded buffers and reconstructed allocations retain
the accepted Stage 2 accounting. No RSS, decoder-only or production-memory assertion
can be inferred from these whole-run peaks.

## Filesystem guarantees and limits

Windows data/source were on local fixed NTFS J:. Linux source and journals were on
native WSL `/var/tmp`; Rust output/cache and copied evidence were on J:. New journal
creation synchronizes its header and, on Unix, the parent directory. Appends use the
existing locked file with no rename or rotation, and two real `sync_all` calls.
Windows parent-directory namespace synchronization is not implemented/certified.
Creation-name crash/flush interruption is not part of this matrix.

The checksummed chain verifies the present prefix; it cannot detect arbitrary
external removal of a whole valid suffix without an independent trusted receipt.
Loss of previously acknowledged bytes through storage failure is not simulated.
The evidence establishes the tested persisted process-termination/reopen behavior,
not physical power-loss durability. SIGKILL/TerminateProcess leave the OS/cache alive;
hardware write ordering and device-cache guarantees are not tested. Injected sync
errors model failure before a call and a successful call whose outcome is lost; they
are not actual hardware flush failures. Native macOS/APFS and broader filesystem
validation remain required before cross-platform disk-history durability claims.
No network-share guarantees are inferred from local storage.

## Validation, artifacts and review gate

Windows workspace default/all-feature suites: **346 / 380** passed. Linux:
**347 / 381** passed. Strict all-target/all-feature Clippy passes on both platforms;
formatting and diff checks accompany the source freeze. The final all-feature
suites include the uncertain-read barrier; default gates precede that feature-only
addition and their compiled default path is unchanged. No managed, desktop or
Native AOT changes were made, so those suites were not rerun.

Evidence is under `artifacts/history-stage3/`: final results are
`windows-results-current`, `linux-results-current`, and each platform's
`*-ack-results-current`. Earlier verified runs precede the final uncertain-read
barrier and are preserved separately; the current matrices and feature suites rerun
on the final barrier source. Primary fault summaries contain operation IDs, actual child termination/return
codes and probe commands; logs retain ACK/reconciliation/zero-charge receipts.
Native journals and independent oracle files are retained and hashed at their
execution locations before copying. Each platform has 2,197 original hashes
(2,053 primary and 144 acknowledgment); **4,394 copied original hashes** verify in
total. All **163 Rust build inputs** match the native WSL source. Source/build/native/evidence manifests and
verification receipts accompany the final packet. Earlier Stage 1/2 evidence and
manifests remain unchanged.

Initial failures are preserved: the large timing fixture accidentally exceeded the
existing cell quota; the Windows corruption test attempted to use a second handle
against a correctly locked file; and a wrong-parent fixture accidentally wrote the
original valid pointer. Corrections retain assertions and limits. Final matrices
use the corrected fixtures and reject a genuinely invalid, checksum-valid pointer.

Stop here for parent Stage 3 review. Stage 4 file/history association, Save/Save As,
saved-content authority, retained-history relocation and non-destructive recovery
decline remain unimplemented. Production container selection, general history,
RPC/UI integration, compaction, identity recycling, Optimize Worksheet, JIT/SIMD
and changes to WB5/CB2/private v4/hosted v1 remain outside this work.
