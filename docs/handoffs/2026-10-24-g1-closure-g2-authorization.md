# ADR-0023 — G1 accepted/closed; G2 authorized

Date: 2026-10-24. **Parent accepts ordinary native Save promotion and closes G1.
G2 is authorized as a bounded feature-only Rust mutation-local record producer.
G3–G7 remain deferred. ADR-0023 as a whole is not closed.**
[Accepted promotion](2026-10-24-historical-save-g1-promotion.md),
[final Mac reconciliation](2026-10-24-macos-g1-promotion-validation.md),
[architecture](../architecture/history-production-readiness.md),
[ADR-0023](../adr/0023-document-transaction-history.md).

## G1 acceptance

Accept the single ordinary current-content WB5 export contract in default builds
for current and historical roots, without fallback from failed strict transfer.
The real Rust session and research association use the same ordinary exporter;
generic full-ledger graph transfer remains strict. Preserve unsupported-capability
rejection, limits, durable UUIDs, runtime NodeIds, lifetime reservations, Expected,
ContentStateId, captures, tickets, cursor/Redo and verified-install uncertainty fences.
Save is not a document-history transaction. No WB5, CB2, private v4 or hosted v1
format change is approved or required.

The parent accepts the default-feature equivalence test and full Windows/native
Linux/native Mac Rust, managed/session/IPC, desktop Save/Open/Save As and actual
Native AOT evidence: Rust 347/390, 348/391, 348/391; 69 native cases per platform,
207 total and 2,652 original-file hashes. The promoted input manifest has 236 files.
This is not acceptance of a historical session/UI workflow that does not exist.

The parent noted that the detailed final Mac reconciliation was not among the
uploaded review files; its detailed assessment relied on the submitted promotion
and status reports. The local final Mac reconciliation and verification receipt
remain linked above. This qualification does not reopen the parent's G1 acceptance.
Preserve all G1 and Stage 1–4 evidence, failures and original manifests.

## Independent unresolved reliability observations

- **Linux managed Fill selection/scroll:** reversed selection/active cell/scroll
  assertion failed, followed by an unchanged-source passing run. Root cause remains
  unresolved. [Original report](../../artifacts/history-save-g1-promotion/linux-ci/20261024T131716640472Z/desktop-ui/results.json).
- **Windows Native AOT unsupported OS Copy:** clipboard-unchanged assertion failed,
  followed by an unchanged-source passing publish/execute run. Root cause remains
  unresolved and is not established to match the older null-destination defect.
  [Original report](../../artifacts/history-save-g1-promotion/windows-aot/20261024T132708097827Z/desktop-ui/results.json).

Neither observation is shown to be caused by the Save correction, so neither blocks
G1 closure. Do not mark them resolved, weaken assertions or speculate with unrelated
UI/clipboard fixes. Retain earlier clipboard, WSLg startup, mounted-filesystem Save
and Linux timing qualifications independently.

## Authorized G2 implementation

Implement feature-only Rust `PreparedRecordChanges` from privately staged canonical
and workbook mutations. Record resolved, typed, stable-key semantic changes, not
replayable authoring commands, display names or merely touched coordinates.

Cover all coupled authoritative changes: canonical definitions and already-bound
tokens; workbook typed content, identities, positions, Blank/text/decimal semantics;
live/retired records, tombstones and moves; selections, aggregate/range descriptors,
placements and dependent metadata; historical source, submitted-reference provenance
and diagnostics; invalidation needed by later editing/evaluation; ContentStateId
and committed lifetime identity reservations.

Coalesce repeated changes to the same stable record key within one Fill/Paste/batch
against the original committed state, producing deterministic final operations.
Private intermediate states must not become separate history actions. Failed and
no-op candidates publish no committed historical transition or ledger advance.

Keep production mutation semantics and resource/identity limits unchanged. Do not
wire unverified records into the accepted journal's persisted reconstruction path.
No production history coordinator, new journal, indexed store, SQLite dependency,
provider framework, witness, protocol fields, desktop Undo/Redo, identity recycling,
format migration, JIT/SIMD, optimization or physical history compaction.

## Differential verification and measurements

Retain the complete-state checkpoint encoder as an independent semantic oracle.
Apply mutation-local records to an independently constructed prior authoritative
record image and compare the reconstructed complete state with the actual private
candidate's full checkpoint. The producer's own changed-record list is not its
sole correctness oracle. Full export remains a correctness oracle, not a required
step in the mutation-local edit path.

Compare exact identities, bound targets, typed values/Decimal96 signed scale,
source/provenance, diagnostics and contractually stable ordering, invalid/cyclic/
rejected drafts, text/blank, aggregates, retirement/missing references, invalidation
and lifetime reservations. Cover individual edits, Clear, Move, copy, four-direction
Fill, native/external Paste, mixed/repeated batches, Undo-then-branch, failure/no-op,
cancellation and cold-restored roots. Simultaneous captures remain independent;
rejection preserves old content, Redo and mutation authority.

Measure records visited/emitted, full-oracle work, mutation-local work, scratch
admission and whole-domain peak/cleanup separately. Target work proportional to
actual authoritative changes plus necessary dependency/range processing; do not
claim every edit is constant time.

Run focused feature-only Rust semantic/admission tests and applicable regressions
on actual changed source. Report exact Windows/Linux/Mac coverage. Default-path
mutation promotion requires the normal source-matched three-platform evidence;
it is not authorized by feature-only results.

## Stop and review

Return the G2 implementation, coverage matrix, independent differential oracles,
allocation ownership and measurements for parent review. Stop on incomplete
coverage, semantic mismatch, stale-authority acceptance, lost reservations, broad
canonical redesign or a required capacity increase. Do not proceed to G3 or select
a backend. This receipt records G2 authorization; implementation has not begun in
this documentation reconciliation.

Implementation follow-through: [G2 feature-only producer and measurements](2026-10-24-mutation-local-records-g2.md).
This receipt records the original authorization, not G2 implementation acceptance.
