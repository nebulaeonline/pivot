# Structured-result projection proof — implementation acceptance and scope closure

Date received: 2026-10-22. Parent review supplied by the user.
**Disposition: Accepted. Bounded Structured-Result and Presentation-Projection Proof is closed.**
[ADR-0019](../adr/0019-structured-results-and-projection-identities.md),
[implementation and three-platform evidence](2026-10-21-structured-results-projection-implementation.md).
This implementation acceptance is separate from the earlier ADR design approval.
Permanent product integration is not authorized.

## Accepted architecture and behavior

Retain separate producer, immutable result-generation, schema/field, producer-scoped
record-key, projection and presentation-revision identities. Two independent read-only
views share one result without duplicating the producer or computational result.
Projected records do not implicitly create ordinary Positions, CellIds or graph nodes.
The serialized private-session request owner is an accepted coherent publication and
capture boundary for this single-session proof, not a generalized concurrent service.
Detached evaluation uses captured workbook/graph/result roots. Recalculation must not
acquire data or refresh a producer.

Accept canonical-order checked Decimal96 TotalAmount aggregation supplied to an
existing named numeric graph invocation. Keep unavailable data, missing records,
schema failures and overflow explicit; errors must not become zero or last-good values.
This adapter is computational integration, not ordinary worksheet-formula binding,
collection-valued core evaluation or an implementation of `#InvoiceLines.TotalAmount`.

Retain tagged presentation locations, separate projected numbering, stable-key
selection and read-only flow bands. Reflow preserves ordinary A1, CellIds, scalar
bindings and SUM geometry. Preserve stale-L2 rejection, missing-record header fallback,
ordinary navigation/selection/clipboard isolation and bounded viewport rendering.

Fixed numeric row ordinals are acceptable anchors **only while structural row editing
is unavailable**. Future row insertion, deletion or movement requires separately
reviewed structural anchor identity and affinity semantics. Do not adopt ordinals as
the permanent policy. The at-most-20-by-10 painted viewport does not establish large
arbitrarily placed widget rendering or generalized scrolling performance.

## Lifecycle and validation accepted

The `--projections` proof remains opt-in, in memory and explicitly non-savable.
Save/Save As refusal and modified-document Discard/Cancel remain tested requirements.
Do not write partial WB4 documents that omit structured content; ordinary WB4 Save/Open
behavior stays unchanged.

| Platform | Default Rust | All-feature Rust | Managed desktop / Native AOT |
| --- | ---: | ---: | --- |
| Windows x64 | 318 | 334 | Passed |
| Linux/WSLg x64 | 319 | 335 | Passed |
| Native macOS ARM64 | 319 | 335 | Passed |

Preserve strict all-feature checks, real C#–Rust boundary, all nine desktop fields,
helper processes, warning-free native publication and zero-charge cleanup evidence.
Native Mac/APFS validation matched 373 source hashes and 190 build/test inputs and
returned 441 files, with no product changes or gate reruns. This records the parent's
acceptance of submitted evidence, not an independent source audit by that reviewer.
Historical WSLg allocator abort, Windows-mounted-path save uncertainty, Linux viewport
performance exception, physical-input exclusions and capacity limits remain separate.

## Next review boundary

The recommended next step is a **focused design proposal**, before any permanent
worksheet-reference implementation. Establish how an ordinary worksheet formula binds
to a stable ProducerId and OutputId independently of projection location. Cover rename
and projection movement, binding errors, captured-generation semantics and consistency
when new generations publish or a producer reports an error. Keep eventual `#` syntax
provisional. The choice between a later worksheet-binding implementation slice and
persistent producer/projection documents follows that design review.

Do not begin that next feature automatically. This acceptance authorizes no SQL,
live connectors, refresh scheduler, permanent `#` grammar, projection persistence,
WB5, generalized collection evaluator, per-record transformation graphs, write-through,
formatting or capacity expansion. No next feature or design is started by this
acceptance record.
