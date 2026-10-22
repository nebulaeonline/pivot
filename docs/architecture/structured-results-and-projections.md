# Structured results and presentation projections

Status: **Bounded experimental proof accepted; milestone closed (2026-10-22)** under
[ADR-0019](../adr/0019-structured-results-and-projection-identities.md).
[Review decision](../handoffs/2026-10-21-structured-results-projection-design-review.md).
[Implementation acceptance and scope closure](../handoffs/2026-10-22-structured-results-projection-implementation-review.md).
The proof is opt-in and in memory. No permanent `#` syntax exists.
The numeric adapter does not establish ordinary worksheet-formula bindings.
Fixed row ordinals are accepted only for the present structurally fixed worksheet;
future row editing requires reviewed anchor identity and affinity semantics.
[Implementation and validation](../handoffs/2026-10-21-structured-results-projection-implementation.md).
[Review packet](../handoffs/2026-10-21-structured-results-projection-design.md).

```mermaid
flowchart LR
    Fixed[Fixed producer descriptor] --> Result[Immutable keyed result generation]
    Result --> P1[Read-only projection P1]
    Result --> P2[Read-only projection P2]
    Result --> Total[Captured Decimal96 total adapter]
    Total --> Graph[Existing numeric graph invocation]
    P1 --> Map[Presentation address map]
    P2 --> Map
    Sheet[Ordinary worksheet axes and cells] --> Map
    Map --> View[Tagged hit-testing and rendered flow]
```

Producer identity answers which result-producing object is bound. Result generation
answers which immutable publication a capture consumes. RecordKey/FieldId answer
which data item is addressed. ProjectionId answers which view is selected. Presentation
ordinals and pixel positions answer only where that view is drawn.

An ordinary A9 remains A9 when a projection between rows 8 and 9 grows. Ordinary
CellIds, scalar bindings and SUM rectangles do not incorporate projected rows.
The presentation map yields either an ordinary worksheet location, a generation-
qualified projection record/field, or projection chrome. It never converts a projected
location into an ordinary cell just because they occupy similar pixels.

Refresh/publication is explicit; recalculation consumes pinned roots without I/O.
A coordinator captures worksheet/graph state and the result map coherently. The
first proof invokes the numeric graph using an exact total from that pinned result;
it does not add a general record-valued graph node or install string values.

The proposed first experiment is read-only, fixed-schema and in-memory. One producer
can have two independently placed views of one result. Key uniqueness, missing-key
errors, canonical numeric reduction and budget refusal are explicit in the ADR.
Worksheet bindings to structured fields, persistent projection documents, concrete
`#` grammar and connectors require later reviewed slices. WB4 must not silently
serialize an experimental document without its projected objects.

## Implemented proof boundary

`pivot-workbook::projection` contains the fixed InvoiceLines descriptor, typed UUID
identities, immutable keyed generations, canonical checked total adapter and bounded
presentation frames. It is an experimental hidden module, not a collection-valued
core API. LineKey is producer-scoped and the deterministic fixture promises that a
returning key names the same invoice line. Amount is supplied record data; no
per-record computation graph or transformation language has been introduced.

The private `pivot-session` owner coordinates workbook mutation, publication and
composite capture in its single request loop under exclusive `&mut Session` access.
A capture pins the workbook root and current result plus the numeric invocation
model before another request can execute. Evaluation consumes that capture alone.
ADR-0019 introduced private envelope version 3; the current ADR-0020 proof uses
version **4** and rejects versions 1/2/3.
Hosted protocol v1, WB4 and PIVOTCB2/cell-block-v2 remain unchanged.

Two placements use ordinary stable row ordinals in the present fixed 1,024-row
worksheet (there is no axis insertion/deletion API). Projected rows have tagged
header/record/footer locations; they never acquire ordinary Position/CellId values.
The desktop paints the same bounded viewport rather than allocating record controls.
Selection is a key plus producer/generation/projection/field and epoch/map revision;
stale maps and delayed targets are rejected. Removed selected keys become an explicit
missing-record header selection. Scroll reflow preserves a visible ordinary/key anchor.

The proof reserves generation payload/index metadata before cloning, retains one
charge per shared generation, and separately charges capture pins and bounded frame
buffers through the existing runtime allocation domain. Conservative metadata charges
cover descriptor/map overhead; this is domain accounting, not a process-RSS claim.
The response frame charge survives encoding until the next request. Rejecting a
candidate keeps the previous generation, captures and epoch. Numeric core allocations
retain their existing runtime-domain charges; the structured ownership cap is 1 MiB.

Launch `python scripts/run_desktop.py --projections` for the non-savable experiment.
Publication/layout buttons are test scaffolding. There is no automatic refresh or
persistence. The UI advertises the experimental document, refuses Save/Save As before
opening a picker, and uses only Discard/Cancel for a modified proof. Ordinary New/Open
replace the proof only through the existing successful lifecycle/discard boundary.

## Experimental ordinary worksheet output bindings

[ADR-0020](../adr/0020-worksheet-structured-output-bindings.md) and the
[focused addendum](../handoffs/2026-10-22-structured-output-binding-contract-addendum.md)
are accepted for the bounded experimental implementation.
[Contract acceptance](../handoffs/2026-10-22-structured-output-binding-contract-review.md).
The bounded implementation is in place. Three-platform managed CI, strict all-feature
checks and Native AOT validation are complete; parent implementation review is pending.
[Implementation evidence](../handoffs/2026-10-22-worksheet-structured-binding-implementation.md).

The accepted design binds document/producer/output/type identities to one shared
captured scalar source leaf in the canonical graph. The document prepares immutable
Decimal96-or-fault contexts from coherent roots. Core faults carry a generic stable
identity; source-specific explanations stay in the document. Rejected candidates
retain the prior head with diagnostics; a published unavailable head cannot fall back
to a cached value. Calculation identity includes source roots, not just worksheet
revision. Names resolve only on authoring; projections never participate in binding.

The proof uses private `@{Producer.Output}` authoring and server-owned DuplicateBlock.
OS Copy/Cut containing structured formulas refuses before clipboard mutation; ordinary
PIVOTCB2 remains unchanged. It stays opt-in/non-savable with one producer/output/slot,
ordinary formula dependents and SUM, two views and all agreed resource ceilings.

Verify claimed error precedence through the actual canonical evaluator; stop and report
any discrepancy rather than changing accepted semantics. Count/charge distinct immutable
formula versions once across shared owners, with separate buffers charged separately.
Three-platform managed and Native AOT evidence is complete; the implementation is
ready for parent review. Permanent `#` syntax, persistence and product integration stay separate.


The document maps portable output IDs to a private graph leaf and opaque slot ID.
Each core context owns its exact immutable graph snapshot and contains Decimal96
or a generic fault ID. The document retains the matching fault reason. Canonical
postfix and row-major SUM tests establish existing precedence; no second evaluator
or live source-value override is used. Ordinary context-free graph behavior is retained.

Annotated formulas own one shared immutable source/token payload per distinct version.
Captures and persistent indexes share that allocation; their own metadata/context
buffers remain separately charged. Adapter ownership pins its original registry
entries. Retirement can therefore continue to consume the sole slot until the last
retained owner is reclaimed. The 1 MiB structured budget is shared with generations,
including retained old generations, formula versions, registry entries and contexts.
The existing runtime additionally accounts for canonical graph/index allocations.

`python scripts/run_desktop.py --bindings` starts the non-savable worksheet binding
proof. Resolve/Apply validate ordered UTF-8 spans, portable IDs, Decimal96 type and
an exact computation token. Source publication rotates that token without changing
worksheet formula definitions. Pure layout changes leave it unchanged. Captures and
read frames carry computation identity and source provenance; the UI rejects stale
frames. DuplicateBlock preserves source IDs and translates ordinary A1 references.
OS Copy/Cut refuses structured formulas before touching clipboard data. No structured
native clipboard format or file encoding has been introduced.
