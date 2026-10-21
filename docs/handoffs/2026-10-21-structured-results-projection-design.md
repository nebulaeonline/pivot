# Structured-result/projection design — parent review packet

Date: 2026-10-21. **Design approved with clarifications; bounded proof authorized,
implementation not started.** [Review decision](2026-10-21-structured-results-projection-design-review.md).
The proposal below records the submitted scope; the review supplies governing clarifications.
[Proposed ADR-0019](../adr/0019-structured-results-and-projection-identities.md),
[identity/data-flow overview](../architecture/structured-results-and-projections.md).
The preceding worksheet-text implementation remains accepted and closed.

## Proposed next experiment

One fixed InvoiceLines producer, two read-only projections and immutable publications
with zero, three and five records. Insert a presentation band after worksheet row 8,
leaving ordinary row 9, A9's CellId and existing formula/SUM bindings unchanged.
The two projections may show the same result in different visual order. Their local
`#1` labels are presentation ordinals; keys and fields carry computational identity.

The proposal uses stable typed UUID identities for producer/result/schema/field/output/
projection roles. A graph-backed producer would have an explicit portable node binding;
the fixed source does not require a collection runtime in the current numeric core.
The document capture coordinates numeric roots and the immutable result-generation map.
No UI callback becomes a source of calculation data.

Use unique LineKey values, including two lines for the same product, to prevent the
accidental assumption that product ID or visible row number identifies a line item.
A failed/duplicate-key publication keeps the old result. A successful empty result
is visibly different and gives exact zero for the proposed TotalAmount adapter.

Prove decimal graph consumption with existing public-input invocation on a captured
TotalAmount, not a new formula parser. Description remains structured presentation
text; it does not become a numeric-core string or worksheet NonNumeric placeholder.
The first experiment supplies fixed Amount fields; per-record transformation graphs
remain a later design.

**Approved scope choice:** the first proof is in-memory and
opt-in. Ordinary worksheet references to producer fields/outputs, projection Save/Open,
new native document versions and permanent `#` grammar are deferred to later binding/
persistence slices. Do not claim an end-to-end savable dynamic invoice yet. Experimental
Save must refuse clearly rather than silently omit objects from WB4. Ordinary workbook
Save remains unchanged. This narrows the earlier illustrative experiment to validate
identity, generation and layout foundations before committing new persisted syntax.

## Worked identity example

| Transition | Record/capture behavior | Ordinary worksheet behavior |
| --- | --- | --- |
| Publish generation G1 with keys L1/L2/L3 | Both projections read G1; visible ordinals are local | A9 retains its CellId and coordinate |
| Capture C1, then publish G2 with L1–L5 | C1 retains G1; new capture pins G2 | Row 9 moves down on screen, never becomes row 11 |
| Reorder records in a projection | L2 remains L2; `#2` may display another key | No graph revision or formula rebinding |
| Move/hide projection P1 | P2 and producer remain intact | No ordinary-cell move/clear/retire |
| Remove L2 in G3 | Live keyed read gives MissingRecord; C1 still sees old L2 | No replacement by the record now at ordinal 2 |
| Return logical L2 in G4 | Live keyed read resolves again; old captures remain unchanged | No ordinary anchor resurrection |
| Publish malformed/duplicate-key candidate | Reject atomically; retain last good generation and error status | No cell/result partial mutation |

## Proposed acceptance matrix after design approval

| Case | Required proof |
| --- | --- |
| Zero/three/five rows | Stable worksheet identities, distinct empty state and bounded reflow |
| Same producer, two projections | One result owner; independent ProjectionIds and placements |
| Record reorder, duplicate product | Keyed identity survives order; product ID is not assumed unique |
| Missing/duplicate keys | Explicit errors, no fallback to ordinal or partial publication |
| Disappear/return key | Reviewed live-reference policy; immutable old captures unaffected |
| Capture G1 while G2 publishes | Text fields, decimal total and numeric invocation all use the chosen capture |
| Recalculate without refresh | No publication, acquisition or result-generation change |
| Total and overflow | Checked Decimal96 canonical-order reduction; no zero/last-good fallback on failure |
| Sort/move/hide projection | No change to producer/result/aggregate or A1 bindings |
| A1 references/SUM across the band | Only ordinary logical coordinates participate |
| Worksheet selection across the band | Logical rectangle and copy exclude projected fields |
| Projected typing/clear/paste | Explicit read-only rejection; no hidden ordinary cell materialization |
| Removed selected record | Selection goes to the projection header, not a replacement ordinal |
| Late hit-test/read response | Map/capture revision guards prevent stale target installation |
| Schema/type mismatch | Reject atomically under fixed-schema contract |
| Exhausted result/pinned-capture budget | Refuse before publication; keep current result and live captures |
| Experimental Save | Explicit refusal; no silently incomplete WB4 |
| Cleanup and AOT | Existing shared clipboard locking, three-platform managed/native gates and zero charge |

Proposed caps: one producer, two projections, eight fields, 64 records, 256 KiB per
complete result and 1 MiB total structured ownership including staged/pinned roots.
Keep ordinary workbook, viewport, frame and runtime-domain ceilings. These caps are
admission bounds for the experiment, not scaling claims.

## Review decisions and stop point

Approve or revise the typed identity representation/graph-binding boundary, exact key
and missing/return semantics, captured publication consistency, canonical reduction
order, flow-band placement and tagged interaction rules. Explicitly decide whether
the in-memory first proof is sufficient before binding/persistence work. The parent
review approves that scope and the typed identity/coordination/presentation direction.

The subsequent review authorizes only the bounded experimental proof. Deliver it
with three-platform evidence, then stop for implementation review. Visibly identify
non-savable documents, provide deterministic experimental discard/close without
ordinary Save prompts, and force the disappearing-L2 stale-interaction regression. Do not start
SQL/connectors, a new record/string calculation engine, live refresh scheduling,
write-through edits, printing or a permanent dynamic-reference language automatically.

This turn changed documentation only. Review validation checks local links and
scope consistency; no executable or test result is claimed for the proposed design.
