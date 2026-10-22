# ADR-0020 contract addendum — approval and bounded implementation scope

Date received: 2026-10-22. Parent review supplied by the user.
**Disposition: Approved with two implementation clarifications.**
Authorized milestone: **Bounded Worksheet-to-Structured-Output Binding Proof**.
[ADR-0020](../adr/0020-worksheet-structured-output-bindings.md),
[contract addendum](2026-10-22-structured-output-binding-contract-addendum.md),
[acceptance matrix](2026-10-22-structured-output-binding-design.md).
This is design/contract acceptance, not implementation or validation acceptance.
ADR-0019 remains accepted and closed. No further architectural approval cycle is required
for the bounded contracts below.

## Contracts approved

Accept the distinct NodeId/CapturedScalarId adapter mapping and immutable capture-local
Decimal96-or-fault context, with exact graph/snapshot identity, complete coverage and
missing/duplicate/unknown/type validation. Keep producer identities, records, source
meaning and mutable last-value state out of the numeric core. Generic fault identity
propagates through canonical dependencies; bounded source diagnostics live in the
document. Graphs without source leaves retain existing behavior.

Approve stable document/producer/output/type binding, one shared resident slot including
retired/pinned ownership, and refusal of replacement allocation while that slot remains
retained. Do not reuse identities or raise limits to make lifecycle tests succeed.

Approve private `@{Producer.Output}` authoring through ResolveFormula/ApplyFormula.
Rust validates source, UTF-8 spans, occurrence order/count, portable IDs/type and exact
computational-state authority. Mixed A1 and structured operands use the canonical
evaluator. Unchanged editor exit preserves binding; explicit resubmission resolves
again. Invalid drafts remain isolated. This is not permanent `#` grammar.

Approve server-owned same-document DuplicateBlock preserving structured identities
and translating only A1 operands, with recomputed occurrence spans and atomic staging.
Refuse OS Copy/Cut containing structured formulas before changing either clipboard
representation, with a clear explanation. Keep ordinary PIVOTCB2 unchanged; unsupported
foreign/native structured payloads reject atomically without TSV fallback.

Approve composite authority rotation/validation, coherent computational captures and
immutable contexts built only from pinned roots. Failed publication attempts leave
authoritative heads unchanged with visible diagnostics; published unavailable states
must not use last-good data. Recalculation never acquires or refreshes a source.

## Required implementation safeguards

1. **Actual evaluator precedence is authoritative.** Verify source-fault versus
   arithmetic precedence and row-major SUM failures through the real canonical
   evaluator, including `badSource + (1/0)` and `(1/0) + badSource`. If actual accepted
   semantics differ from the addendum, stop and report the discrepancy; do not silently
   change existing numeric ordering or use a test-only traversal to claim agreement.
2. **Count distinct retained allocations.** Charge one immutable formula/version
   allocation once even when several captures share it. The 64-version limit counts
   distinct retained versions, not references. Separately owned indexes, contexts and
   diagnostics have separate charges. Failed staging releases reservations; retirement
   remains charged until its final owner is reclaimed.

Retain 64 registry/tombstone entries at 1,024 bytes each; one 2,048-byte resident slot;
8,192 bytes per distinct retained annotated formula version; 32 current/64 retained
versions; 16 occurrences per formula; 4,096 bytes per captured context; two captures.
These share the existing 1 MiB structured-state ceiling and do not expand ordinary
workbook/core capacities. They are conservative admission charges, not heap measurements.

## Authorized work and stop

Implement only the opt-in, non-savable experiment: one fixed producer, one Decimal96
output, one shared captured-source slot, ordinary worksheet formula consumers and
dependents including SUM, and two existing read-only projections. Demonstrate that
source publication changes ordinary formula results without changing formula definitions,
creating hidden cells or resolving names at calculation time.

Retain all adversarial identity/span/fault/lifecycle/quota cases in the design packet
and addendum. Require genuine C#–Rust worksheet-formula and desktop behavior, full
managed CI, strict all-feature checks and actual Native AOT publish/execute on Windows
x64, Linux/WSLg x64 and native macOS ARM64/APFS. Preserve zero-charge cleanup, existing
workbook/cross-process behavior and shared clipboard locking. Routine automation stays
the normal gate; this review creates no recurring manual replay requirement.

No SQL/connectors, automatic refresh, permanent `#` language, WB5, persisted projections,
keyed records, collections, per-record transformations, structural row editing,
formatting, printing or capacity expansion. Do not weaken hosted v1 or reinterpret
PIVOTCB2/WB4. Record exact implementation and cross-platform evidence, then stop for
parent implementation review. Permanent structured-data integration is not authorized.
