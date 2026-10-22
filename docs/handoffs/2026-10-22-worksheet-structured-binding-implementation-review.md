# ADR-0020 worksheet binding proof — implementation acceptance and scope closure

Date received: 2026-10-22. Parent review supplied by the user.
**Disposition: Accepted. Bounded Worksheet-to-Structured-Output Binding Proof is closed.**
[ADR-0020](../adr/0020-worksheet-structured-output-bindings.md),
[approved contracts](2026-10-22-structured-output-binding-contract-review.md),
[implementation and three-platform evidence](2026-10-22-worksheet-structured-binding-implementation.md).
This acceptance is separate from architectural/contract approval. It accepts the
submitted implementation and evidence, not an independent source-code audit.
Permanent structured-output integration is not authorized.

## Accepted computational boundary

Ordinary worksheet formulas, downstream arithmetic and SUM consume stable
DocumentId/ProducerId/OutputId/ExpectedType bindings through one shared captured
scalar source leaf in the canonical numerical graph. Immutable Decimal96-or-fault
contexts are prepared by the document coordinator. Producer identities, records,
acquisition and projections remain outside the numerical core. There are no hidden
ordinary worksheet inputs, literal substitution or competing evaluator.

Source publication updates results without changing formula definitions or CellIds.
Composite computational authority and coherent captures preserve old worksheet/source
roots and contexts. Rejected publication attempts retain the previous head; explicitly
unavailable heads return errors without last-good substitution. Rename preserves IDs;
delete/recreate yields explicit missing-identity errors rather than name-based repair.

The actual postfix and row-major SUM precedence tests satisfy the review clarification;
no numeric evaluation-order change is authorized. Generic CapturedSourceFailed IDs
propagate with bounded document diagnostics. Distinct immutable annotated-formula
allocations are charged once across shared owners; separately owned contexts, indexes
and buffers retain independent charges. Preserve retirement, final-owner reclamation,
atomic quota refusal, zero-charge tests and all existing limits: one resident slot,
32 current/64 retained formula versions, two contexts and the shared 1 MiB budget.

Accept private `@{Producer.Output}` authoring and strict Rust-side Resolve/Apply
span/order/identity/type/composite-state validation. Accept server-owned DuplicateBlock
preserving structured IDs and translating only ordinary A1 operands. Structured OS
Copy/Cut refusal occurs before clipboard mutation; ordinary PIVOTCB2 is preserved.
A permanent integration would need a separately reviewed ordinary clipboard workflow.

## Validation accepted

| Platform | Default Rust | All-feature Rust | Managed desktop / actual Native AOT |
| --- | ---: | ---: | --- |
| Windows x64 | 328 | 344 | Passed |
| Linux/WSLg x64 | 329 | 345 | Passed |
| Native macOS ARM64 / APFS | 329 | 345 | Passed |

All ten desktop fields and helper-process workflows pass with zero remaining Rust
allocation-domain charge. Preserve strict all-feature lint, warning-free native
publication, Mac APFS/ARM64 provenance, 390 source and 199 build-input matches,
475 independently hash-verified returned files, and failed development/environment
logs. No recurring manual replay requirement is added.

## Open reliability issue — Windows clipboard checklist

**Unresolved:** Windows ClipboardChecklistAsync encountered a null destination cell
after paste at line 52 in
`artifacts/worksheet-bindings/windows-ci-final/20261022T144326138920Z/desktop-ui/results.json`.
The unchanged full rerun passed, with independent Linux/Mac coverage. This does not
block the accepted binding semantics, but is not root-caused or closed.

If it recurs, capture clipboard payload availability, asynchronous UI sequencing,
edit commit acknowledgement and viewport/readback timing to distinguish the cause.
Do not weaken assertions/limits or initiate a speculative clipboard redesign.
Keep the WSLg startup abort, mounted-Windows-filesystem Save uncertainty and historic
Linux viewport exception as separate pre-existing issues.

## Scope closure and product-direction boundary

The experiment remains opt-in and non-savable. WB4, PIVOTCB2 and frozen hosted v1
are unchanged; private application-session version 4 is accepted for this proof.
Projection hide/delete lifecycle operations were not newly exercised by ADR-0020;
this does not weaken the accepted separation of computational and projection identity.

No SQL/connectors, refresh scheduling, generalized collections, per-record subgraphs,
WB5, persistent projections, permanent `#` grammar, formatting, printing, structural
row editing or capacity expansion is authorized. Stop before another implementation
milestone. No next feature or design is started by this acceptance record.

The parent recommends a short product-direction review rather than automatically
extending experimental infrastructure. Choose deliberately between permanent
structured-document/authoring integration and conventional spreadsheet capabilities
such as formatting, Undo/Redo and broader functions. These are candidate directions,
not approved implementation scope.
