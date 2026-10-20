# ADR-0017 rectangular clipboard — review decision

Date: 2026-10-20. **Approved with one compatibility-policy revision.**
Parent review supplied by the user; this records design acceptance and implementation
authorization, not implementation or validation acceptance.
[Accepted ADR](../adr/0017-rectangular-clipboard-and-formula-copy.md),
[bounded implementation plan](2026-10-20-rectangular-clipboard-design.md).

## Accepted decisions

Native formula copy translates effective source bindings from coherent generation S,
not historical source spellings. Dollar markers lock copy axes without changing
stable scalar identity. SUM translates bound endpoint geometry and creates fresh
selection bindings. Preserve exact decimal coefficient/sign/scale and source bytes
outside rewritten reference spans.

Copy/paste admits at most 256 slots, requires exact destination shape or expansion
from one cell, captures before overlap writes, includes blanks, and installs one
atomic canonical Rust batch. Existing destination identities remain; new positions
receive fresh identities. Bind against the complete staged destination, including
cells created later in row-major order. No partial installation or per-cell IPC.

Use bounded versioned native bytes plus UTF-8 TSV. Native formulas translate;
external formulas bind literal supplied addresses. Rust decodes, validates,
translates, binds and mutates; C# transports OS data and manages selection/focus.
Treat native bytes as untrusted. Validate dimensions, types, spans, annotations and
limits; claimed provenance grants no authority and source IDs are never imported.
Malformed native data must produce an explicit error without silent TSV fallback.

The admission ceilings are provisional and remain subordinate to all existing
workbook/selection/runtime budgets. Enforce native and TSV sizes and the combined
256 KiB copy-output ceiling. A 256-slot block is not guaranteed admission.

## Compatibility revision adopted

Use **PIVOTWB3** for this slice. WB2 invalid-draft restoration reparses source, so
expanded dollar syntax would change the interpretation of formerly rejected text.
Avoid permanent historical grammar/diagnostic modes. WB3 uses current grammar;
explicitly reject WB1/WB2, retain rejection fixtures, and add no compatibility reader
or migration. Frozen hosted storage/protocol v1 remains unchanged.

Current-session and native-copied invalid drafts retain their non-executable state
until explicit user resubmission. This is independent of historical file support.
Do not permit arbitrary invalid markers or weaken graph/selection-image validation.

## Required implementation evidence

Test all scalar axis modes, moved targets/origins, mixed/reversed SUM endpoints,
overlap, blank clearing, destination identity retention, cross-document native copy,
external TSV, exact signed zero/scale, UTF-8 spans and rejected drafts. Mandatory
cases include references to targets created later in the batch and deliberately
tampered native payloads. Verify complete rollback for late failures, translated
bounds errors, cancellation, quotas and stale revisions. Verify WB3 reopen,
non-executable drafts and historical-version rejection.

Retain exact-revision submission, response draining and late interaction guards.
No clipboard-owned graph or retained source session. Run full Windows, isolated
Linux and fresh native APFS/macOS validation, actual application clipboard workflows
and user-confirmed manual checks, with source hashes and zero-charge cleanup.
Preserve the known WSLg mounted-path uncertainty, historical native abort and
historical Linux viewport timing exception.

Cut, drag-fill, autofill, structural operations, formatting/text-value expansion,
a generalized clipboard framework, alternate graph, SQLite and capacity increases
remain excluded. No further general design round is requested. Deliver the bounded
implementation, evidence and handoff, then stop for implementation review.

No implementation or new feature-test evidence is claimed by this review record.
