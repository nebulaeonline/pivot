# ADR-0019: Structured results and projection identities

Status: **Design approved with clarifications (2026-10-21); bounded experimental implementation accepted and milestone closed (2026-10-22)**.
[Implementation acceptance and scope closure](../handoffs/2026-10-22-structured-results-projection-implementation-review.md).
[Design acceptance and clarifications](../handoffs/2026-10-21-structured-results-projection-design-review.md).
[Implementation and three-platform evidence](../handoffs/2026-10-21-structured-results-projection-implementation.md).
Permanent product integration is not approved.
Date: 2026-10-21.
[Preceding implementation acceptance and direction](../handoffs/2026-10-21-worksheet-text-implementation-review.md),
[earlier fusion direction](../handoffs/2026-10-20-native-aot-next-scope.md).

## Context

Pivot is a computational document with a spreadsheet presentation. An invoice can
combine ordinary labels/numeric formulas with a variable-length set of line items.
The projected records must not become inserted worksheet rows or thousands of
ordinary CellIds merely to appear on screen. Existing scalar bindings follow stable
CellIds, SUM follows fixed worksheet rectangles, and captures are immutable.
Those accepted semantics must survive changes in dynamic presentation.

This approved direction covers a bounded fixed-record experiment, not database integration,
a general structured-value evaluator, or a permanent `#` language. All policies below
are accepted subject to the linked review clarifications. No implementation changes
accompany this design record.

## 1. Identities and ownership

| Identity | Meaning | Must not mean |
| --- | --- | --- |
| ProducerId | Stable document-owned result-producing object | Display label, query text, projection or current result |
| ResultGenerationId | One immutable publication by that producer | An edit revision, timestamp or visible row ordinal |
| SchemaId / revision | Explicit field/type contract for a result family | Current column headings or positions |
| FieldId | Stable field identity within the producer's schema | Display heading or column index |
| RecordKey | Producer-scoped logical record identity | Result order, visible `#1`, ItemId unless explicitly unique |
| ProjectionId | One presentation of a producer | The producer, a fetched result or a worksheet CellId |
| PresentationRevision | One immutable layout/map version | Calculation generation or worksheet graph revision |

Use opaque UUID-backed ProducerId, ResultGenerationId, SchemaId, FieldId,
OutputId and ProjectionId types. Portable references carry document/producer scope;
process-local handles remain private. A future graph-backed producer maps explicitly
to its portable GraphId / GraphNodeId, with one declared owner for publication. The
first fixed-source descriptor has no collection-valued numeric-core node and no
implicit alias to an ordinary cell node. Keep this representation economical: no generalized distributed identity service,
schema registry or independent graph namespace. Do not implement speculative graph
source integration. Never substitute a display name or process-local NodeId.
The producer registry owns fixed descriptors/publications, not a second evaluator
or independent dependency graph.

A producer may have many projections. Moving/hiding/removing one projection never
renames or deletes its producer or record identities. Removing a producer invalidates
its references; creating another with the same name allocates a new identity and
does not repair old bindings automatically. Names resolve at the client-command
boundary and are then retained only as source/provenance, consistent with ADR-0003.

A live record reference is logically `(ProducerId, RecordKey, FieldId)` plus the
expected field type. A captured read adds ResultGenerationId. Projection and screen
coordinates never participate in computational binding. A field identity cannot
be silently reused for a different meaning/type.

## 2. Fixed typed records and key semantics

Use one fixed InvoiceLines schema for the first proof: LineKey, Description,
Quantity, UnitPrice and Amount. Description is literal single-line UTF-8 data;
the other exposed data fields are exact Decimal96. LineKey is record identity,
not an editable numeric field. Item/product IDs are not suitable default keys:
an invoice can contain the same product twice.

Schema descriptors and each result generation are immutable. The fixed fixture
supplies Amount explicitly; arbitrary per-record transformation subgraphs are not
part of the first proof. Strings in result fields are structured presentation data,
not computational strings in Pivot's numeric graph. Neither worksheet LiteralText
nor its NonNumeric marker becomes a general record-value representation.

Require explicit nonempty unique keys, bounded to 128 UTF-8 bytes, using exact
case-sensitive bytes without implicit Unicode normalization or numeric conversion.
The source contract promises a key denotes the same logical entity across updates.
Missing/duplicate keys reject the entire candidate publication. No ordinal fallback,
silent deduplication, fabricated identities or partially accepted rows.

Within a generation, retain one explicit canonical record order and a unique-key
index. Display order may differ without changing this canonical order or identities.
A reordered publication with the same keys preserves keyed meaning. A disappeared
key produces MissingRecord for a live keyed reference; if the same logical key
returns later, the live keyed reference resolves again. A capture of the missing
or old generation never changes. Tombstone/retirement semantics for external systems
that recycle business keys require a different source contract, outside this proof.

The fixed-schema experiment rejects schema changes atomically. Future rename rules
may preserve FieldId when only a heading changes; removal/type changes require an
explicit contract, not automatic rebinding. No null/missing-value, date, boolean,
IEEE754, nested record or schema-evolution policy is implied by this fixed fixture.

## 3. Publication, refresh and captures

A result generation contains producer identity, schema contract, canonical rows,
key index and provenance identifying the fixed fixture/definition revision. Its
payload is immutable and owned in Rust. A generation ID is opaque; identical payloads
may still form distinct publications. Timestamps and content hashes are diagnostics,
not freshness authority or identity substitutes.

Explicit ReplaceFixture/Publish stages and validates the whole candidate before
publication. Recalculation never replaces a result or performs acquisition. There
are no network calls or database credentials in this experiment. A failed replacement
retains the last successful generation and reports the failure separately; it does
not replace records with an empty table. Zero records is a valid successful result,
visibly distinct from unavailable data, failed acquisition and stale parameters.

A coherent document/calculation capture pins worksheet/graph state, producer
bindings and the exact result-generation map. Reads, derived numeric inputs and
presentation of that capture use this same map. A result may advance while an old
capture is calculated or displayed; that capture continues to use its old values.
Updates do not mutate arrays behind a capture or attach a new result to an old
revision token.

Publication advances a document result epoch without changing ordinary worksheet
axes, CellIds or graph definitions merely to refresh the display. The coordinator
captures the worksheet/graph roots and result epoch/map atomically; reading those
roots independently and labelling them one capture is insufficient. Presentation
changes advance PresentationRevision, without recalculation or result replacement.

If a later slice installs persistent numeric adapter values, publication and those
values must share one staged document commit. Validate expected document state and
producer definition/parameter provenance,
reserve resources, build result/adapter roots, validate, then install together.
Failure or cancellation before installation publishes neither root. In the first
proof, TotalAmount is derived from the pinned result and supplied as an invocation-local
decimal override; no mutable adapter input root is installed. Calculation
or UI work may complete after a new publication, but late work cannot replace a
newer display frame. Future parameterized refreshes must reject late responses
whose captured producer definition/parameters no longer match; matching an HTTP
completion timestamp would be insufficient. The fixed fixture has no parameters.

A capture can be internally coherent across several independently published sources
without claiming those sources were acquired in one external transaction. The
first experiment uses only one producer; it introduces no cross-source transaction
or refresh scheduler.

## 4. Numeric consumers without layout dependencies

The target semantic binding is producer/field/key or producer/output identity,
never a ProjectionId or screen ordinal. Numeric access requires an explicitly
Decimal96 field/output; text consumption is a type error, not parsing/coercion.
Missing producer, missing record, missing field and schema/type mismatch are
separate errors. Zero-record aggregate success is distinct from unavailable result.

For the first proof, expose one typed numeric output, TotalAmount, with a stable
OutputId. It performs checked Decimal96 addition over Amount in canonical record
order, starting at zero. Empty result gives exact zero; overflow produces an output
error with no partial total. Display sorting or projection placement cannot change
reduction order or hide errors. This is a bounded adapter contract using existing
numeric operations, not a generalized collection expression engine.

Prove consumption with the existing numeric graph/public-input interface: invoke
an existing decimal calculation against the total from one pinned result, retaining
both capture identities in the proof. Do not turn failed totals into zero or a last-good
numeric input. This explicit invocation requires no new runtime string/record type.
Its output can appear in a read-only projection footer for the experiment.

Ordinary worksheet formulas referencing a producer are a **subsequent binding slice**.
Their future typed binding must carry stable producer/output/key/field identities
and have explicit numeric/error propagation and snapshot rules. The first proof
must not fabricate ordinary input cells, rewrite formula source or silently introduce
unreviewed tokens just to simulate such a formula. No parser changes are authorized.
The `#` namespace remains reserved conceptually: no concrete `#InvoiceLines[3]`
grammar, keyed syntax, aggregate syntax or ordinal binding is approved here.

## 5. Presentation flow and address translation

A read-only projection descriptor owns ProjectionId, producer binding, field layout,
and placement. Results and projection descriptors are distinct immutable objects.
The first placement is a full-width flow band **after an ordinary worksheet row**,
anchored to `(WorksheetId, stable row-axis identity)`, with deterministic descriptor
order when two bands share a boundary. Repositioning changes this boundary/order,
not worksheet axes, content, CellIds, scalar bindings or SUM rectangles.

Do not anchor the experiment to the incidental current screen index. Future structural
row deletion must have an explicit orphan-placement policy; structural editing of
anchors is excluded from the first proof. Ordinary cell Move does not move a band
anchored to the row boundary. Worksheet row identities remain authoritative.

A presentation map translates pixel/visible-row positions to tagged locations:

- WorksheetLocation: worksheet position and optional materialized CellId.
- ProjectedFieldLocation: ProjectionId, ProducerId, ResultGenerationId, RecordKey,
  FieldId and PresentationRevision.
- ProjectionChromeLocation: header, empty-state/footer or explicit layout diagnostic.

The map is presentation-owned, computed from a pinned generation map and descriptor
snapshot. It is used consistently for rendering, hit-testing, selection, scrolling
and later print planning. It does not live in formula binding or evaluation. Stale
pointer/async actions must be validated against the map revision and stable target;
never reinterpret an old pixel position using new record positions.

```text
Ordinary row 8       remains worksheet row 8
Projection header   read-only InvoiceLines
#1, #2, #3          visible ordinals of this projection's generation
Ordinary row 9       remains worksheet row 9, including A9 and existing SUM bounds
```

Growing from three to five records adds two visible record bands before row 9.
It does not insert ordinary rows. `#1` is a local presentation ordinal, explicitly
qualified in selection details by producer/projection; it is not a persistent record
address. Use a distinct background and a non-color read-only/source cue. Accessibility
labels must identify projected versus ordinary content. No physical accessibility
certification is claimed by a rendering experiment.

Zero records retains the projection header and a bounded explicit empty row. The
first layout is a linear vertical flow: bands never overlay ordinary cells or each
other, and no neighboring content is overwritten. Both vertical and horizontal
scrolling preserve tagged targets. Narrow viewports clip/scroll rather than wrap
rows or insert worksheet columns. If a layout cannot be admitted, show a bounded
projection diagnostic without mutating cells or partially drawing a different result.
Arbitrary free-position widgets, merging, pagination/page-break algorithms and print
implementation are deferred; later print plans must reuse the same identity mapping
rather than renumber worksheet rows to solve pagination.

Maintain scroll/selection using a stable tagged location plus within-row offset.
An ordinary selected cell stays selected through reflow and is revealed on demand.
A selected projected record follows its key within the same projection; if removed,
select that projection's header with an explicit missing-record indication. Never
silently select whichever new record occupies the old ordinal.

## 6. Editing, clipboard and save boundaries

Projected records are read-only in the first proof. No ordinary edit/clear/move/
paste command can use a projected location as a Position. Reject edits there
without changing the worksheet, result or producer. An ordinary selection whose
endpoints are ordinary cells remains a logical worksheet rectangle even when a band
lies between them. Highlight only its ordinary pieces; do not highlight the band as
selected cells. A worksheet drag over a projected row does not extend until another
ordinary endpoint is reached. Starting in a projected record and extending into the
worksheet is refused explicitly. The first proof selects only one projected record
for read-only inspection; projected range/clipboard operations are excluded. Ordinary
keyboard navigation follows logical worksheet neighbors and skips presentation bands;
Escape from projected inspection restores the last ordinary selection (or A1).
An ordinary SUM spanning rows 8–9 includes only its ordinary coordinates,
not the records visually between them.

Projected clipboard, cross-region selection, materialize-to-cells, local overrides,
write-through and export are deferred. Even an eventual materialize command must
be an explicit admitted ordinary-cell mutation, not an automatic side effect of
refresh. A second projection shares one immutable result without rerunning the
producer or duplicating its graph/model.

The first experiment is an isolated, opt-in in-memory proof, not a normal savable
workbook feature. WB4 remains unchanged and must not silently omit experimental
objects. If a desktop proof opens an experimental document, its Save/Save As path
must explicitly refuse that document rather than write a misleading partial WB4.
Advertise the experimental non-savable state visibly before a close/save attempt.
Close/discard must be deterministic and use explicit experimental discard wording
where confirmation is needed, without a misleading ordinary dirty-save prompt or
Save choice. Ordinary documents keep existing Save/Open and close semantics. No persistence/reopen success
is claimed for projections in this first slice.

A subsequent persistence proposal must encode producer/schema/field/output identities,
projection descriptors, captured result payload/provenance and their references
consistently. It must distinguish saved result cache from permission to refresh,
validate referential integrity and choose an explicit format/version policy before
implementation. No WB5 cut, compatibility reader, SQL configuration or provider layer
is authorized by this design-only document.

## 7. Bounded proof and admission

Accepted experimental proof ceilings: one producer, two projections, eight fields, 64 records,
4,096 UTF-8 bytes per text field, 128 bytes per key, 256 KiB per complete result
including owned descriptors/index metadata, and 1 MiB total structured-state ownership
including staged and capture-pinned generations. The fixed invoice uses five fields.
Keep at most two retained calculation captures in the proof; release is explicit.
Charge shared payload once while charging separately owned indexes/maps and view
buffers; retain the charge until the final owner is reclaimed.

Stage/reserve before constructing unbounded buffers. Bound viewport reads by the
existing 256-slot and private-frame byte ceilings, counting projected fields too;
there is no automatic 64-by-eight whole-result IPC response. Account presentation
maps without allocating one ordinary row/CellId/core node per record/field. A new
result may be refused while old captures pin the budget; do not evict a live capture
or silently truncate to publish. These are experimental admission bounds,
not a capacity increase for ordinary workbooks or a performance promise.

## Review and implementation stop

The parent approves these semantics and the bounded in-memory proof, with the
linked clarifications. Implement only that proof, retain one coherent publication/
capture coordinator and independent detached evaluation, then stop for parent
implementation review. No permanent product integration is approved.

Regression tests must force an interaction started on L2 to complete after a newer
generation removes/reorders L2, proving the old ordinal cannot select its replacement.
Test visible non-savable state, explicit Save/Save As refusal and deterministic
experimental discard/close alongside unchanged ordinary WB4 lifecycle behavior.

No general collection/string runtime, database/network connector, automatic refresh,
per-record graph transformations, workbook formula grammar, permanent `#` syntax,
projection persistence, structural editing, formatting, printing or capacity expansion
is included. The proof must use the existing canonical graph, never a competing engine.
The [review packet and adversarial cases](../handoffs/2026-10-21-structured-results-projection-design.md)
define the accepted first experiment and deliberate follow-up slices.
Design acceptance does not claim implementation or validation of this proof.
