# ADR-0023 Stage 1 — exact cold-history restoration

Date: 2026-10-23. **Feature-only Stage 1 proofs pass on Windows/Linux; submitted
for cold-restoration review. Not acceptance of a production history implementation.**
[Inventory acceptance and required gate](2026-10-23-cold-history-inventory-review.md),
[identity contract](2026-10-23-cold-history-identity-review.md),
[ADR-0023](../adr/0023-document-transaction-history.md),
[remaining experiment plan](../architecture/disk-history-feasibility-plan.md).

## Result

A separate bounded reconstruction path restores canonical and workbook state from
encoded bytes. It preserves exact runtime NodeIds within the live session, remaps
into one fresh runtime namespace after restart and isolates independent copies.
Existing portable transfer and NodeId representation are unchanged. No authoring
commands, formula reparsing/name resolution, WB5 Open or calculation captures are
used as the restoration mechanism.

All new APIs/code require `history-experiment`; sha2 is an optional workbook
research dependency. Default runtime semantics, WB5, CB2, private v4 and hosted v1
remain unchanged. This is a complete-checkpoint research baseline, not selection
of a production container, storage provider or retention policy.

## Reconstruction and identity authority

The core research image retains sparse live/retired canonical records, typed
inputs, already-bound scalar/range tokens, source/binding errors, selection
identities/geometry/placements, invalid diagnostics and invalidation provenance.
It stores node ordinals, never a persistent model-number owner. Decoder references
are mapped into the destination WorkingModel's runtime model namespace. Reverse,
name and selection-consumer indexes are reconstructed; serialized proof certificates
are not trusted. No historical cycle rejection is applied. Private construction
preserves error/draft semantics and later edits use existing validation paths.

The workbook image retains durable worksheet/graph/axis/cell/node UUIDs, positions,
exact DecimalInput parts, text, blank markers, typed formulas, original references,
diagnostics/provenance and ContentStateId. It validates correspondence with the
canonical definitions and exact bound tokens without parsing. Core/workbook halves
install together only after complete decoding/admission and final cancellation check.
Expected/state and canonical mutation authority advance freshly. Historical formula
Expected/Revision values are retained solely as provenance, never revived authority.

`ColdLedger` is an explicit research owner outside undoable content. The harness
observes each committed state before detaching it, retaining the full ordinal-to-
logical-UUID reservations and range identities from discarded futures. A new branch
allocates beyond the committed floors; Clear/eviction/restart cannot reclaim them.
The adapter refuses missing or stale ledgers instead of inventing lost reservations
from an older root. That also preserves the existing finite identity-budget issue.

Reopen takes two inputs: a capsule carrying the latest identity authority and a
capsule for the explicitly chosen historical content. It creates a fresh model/session
namespace and uses the same ordinal/logical map for subsequent A/B/C loads. This
explicit research call is not proof of durable transaction/head selection or verified
file association. Those belong to later journal/lifecycle stages.

## Executed semantic gates

`crates/pivot-workbook/tests/history_cold_restore.rs` has five assertion-bearing tests
plus one child-process test entry point. The child entry returns immediately in an
ordinary parent test run; the restart test actually invokes it in a fresh process.

| Gate | Executed evidence |
| --- | --- |
| Same-session cold eviction | Only encoded A bytes and a logical oracle remain, with B's calculation capture alive; cold A restores exact NodeIds, UUIDs, bindings, drafts and content identity under fresh guards |
| Branch and capture isolation | C allocates beyond B; cold B/C return their original runtime IDs; B's capture continues evaluating 30 after A/C activations |
| Actual process restart | Parent releases all models/runtime/captures; child reads stored bytes, allocates unrelated models first, then restores A/B/C into a fresh namespace with one consistent runtime mapping |
| Durable equivalence | Independent pre-encoding CellView/diagnostic/invalidation/ContentStateId oracle matches the child; source/provenance and bound target UUIDs remain exact; old Expected rejects |
| Simultaneous independent copies | Same logical UUIDs/content identity, different runtime model/session namespaces; cross-copy authoring/ledger authority rejects and captures remain isolated |
| Reservation preservation | Latest C ledger includes B-only objects even when A is chosen; after reopen, a new branch allocates beyond C; stale/incomplete ledgers reject without mutation |
| Failure atomicity and ownership | Truncation/checksum damage, checksummed oversized metadata/records, cancellation and actual domain exhaustion preserve live content/authority; late malformed-record failure releases partial reconstructed roots |

Fixtures include a moved scalar target, SUM and reversed mixed-dollar MAX range,
blank reference anchor, Unicode text, retired/missing reference, invalid and cyclic
drafts, copied rejected formula, and exact negative-zero DecimalInput parts. The
oracle compares original diagnostic/reference provenance in addition to evaluating
valid outputs. The tests do not establish an exhaustive hostile-input/fuzzing result.

## Resource bounds and receipts

The research frame is bounded to 16 MiB per encoded image; this is a decoder/buffer
safety bound, not a cap on logical history or an edit-retention policy. UUID-ledger
storage/scratch reserves 8 MiB per owner; encoders reserve their bounded output and
scratch; decoders reserve checked input-dependent scratch. These deliberately loose
reservations keep prototype allocations admitted but are not a selected cache policy.
Persistent structures use the existing domain and cleanup tickets; partial core and
workbook owners retire through the existing bounded cleanup mechanism.

The A/B/C fixtures encode to **5,483 / 6,303 / 6,411 bytes**, with a final lifetime
node floor of 17. That is small-fixture data, not a storage-efficiency benchmark.
The same-session sample peaks at about **94.1 MB domain charge**, mostly conservative
buffer/ledger reservations and simultaneously held encoded capsules. It is not an
RSS measurement or 94 MB of canonical graph allocations. The pressure test deliberately
reserves the entire default 2 GiB domain to force admission failure; its peak must not
be presented as ordinary working-set usage. Every fixture/runtime, including the
fresh child, finishes with allocated and charged bytes both zero.

Measure/tune actual cache/staging policy and near-capacity behavior in the later
representation comparison. The prototype does not promise constant-time cold loads:
complete-state reconstruction traverses the bounded encoded state and indexes. Hot
root activation and ordinary mutation paths retain their existing mechanisms.

## Validation and source evidence

Windows workspace default/all-feature suites: **346 / 377** tests passed. Linux:
**347 / 378** passed. Focused cold suite: **6 parent entries** on each platform,
including the inactive helper; the real restart child separately executes its one
entry. Do not sum the child summary again into full-suite counts.

Formatting and strict workspace/all-target/all-feature Clippy pass. Final targeted
receipts include explicit encoder-scratch admission; final all-feature suites use
that source. Default suites precede that last feature-only reservation addition;
it is excluded from default builds. Raw logs preserve earlier passing iterations.

Linux source and temporary restart files run from native
`/var/tmp/pivot-history-feasibility`; the reused Rust target cache is on `/mnt/j`.
No native macOS/APFS run, .NET/desktop/Native AOT or cross-platform filesystem durability
claim is made. Child tests use ordinary temporary file writes and real process
reconstruction, not power-loss/flush or transaction-journal recovery.

New source/build/evidence hashes and command/toolchain receipts are under
`artifacts/history-cold-stage1/`. Accepted hot-root and inventory manifests are
unchanged. The only build-lock change adds the optional workbook sha2 dependency
edge; no new package was selected or downloaded. Development checks caught an unused
exact-decimal reader before it was wired in; final lint is clean. No failing runtime
proof was suppressed or replaced with replay/portable import.

## Next boundary

These results satisfy the implemented three-case Stage 1 test gate, pending parent
review. They do not prove recoverable extensible disk history. Remaining authorized
research compares full checkpoints, canonical record deltas and shared immutable
objects; then real durable commits/cursor/branch decisions, failure/restart
reconciliation, corruption/disk exhaustion and native workbook/history association.
Those stages must measure disk/write/sync cost, bounded live memory/index growth,
retrieval work, non-destructive recovery decline and retention/relocation policies.
Native macOS/APFS storage validation precedes cross-platform durability claims.

No general history stack, durable commit journal, production container, session/save
integration, protocol/desktop operations, identity recycling/limit expansion or
Optimize/JIT/SIMD was implemented. Existing reliability issues and the 8,192 lifetime
identity scaling constraint remain open. Submit this cold gate before treating it
as accepted evidence for representation work; production integration stays deferred.
