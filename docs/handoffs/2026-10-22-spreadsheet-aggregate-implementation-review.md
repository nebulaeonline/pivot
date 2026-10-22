# ADR-0021 implementation review — final acceptance deferred

Parent review received 2026-10-22. Computational implementation and existing
three-platform evidence are acceptable; **final milestone acceptance is deferred**
until WB5-only correction and validation of the resulting candidate.
[Implementation and evidence](2026-10-22-spreadsheet-aggregate-implementation.md).

## Required correction

Remove WB4 decoding, reconstruction and migration pathways entirely. Support WB5
read/write only; explicitly reject WB1–WB4 before changing the live workbook.
Preserve workbook roots, path, revision, dirty state and saved checkpoint on failed
Open. Keep historical documentation and a small unsupported-WB4 rejection fixture.
Replace migration-specific tests with WB5 coverage for blank identity, explicit zero,
strict SUM, aggregate metadata, bound references, text, rejected drafts, stable
CellIds, checkpoints and malformed documents. Unknown tags/kinds must reject.

Retain the aggregate evaluator, private exact AVERAGE implementation, independent
544-case rational corpus, canonical extrema, range identity, immutable captures,
clipboard translation and resource accounting. No aggregate redesign is requested.

## Compatibility policy and validation

Experimental workbook, native clipboard and private application formats support
only their current versions unless a concrete consumer and explicit compatibility
commitment justify otherwise. Version cuts follow incompatible representation or
semantic changes, not feature milestones. Keep CB2, private application v4 and the
explicit frozen hosted v1 commitment. No additional adapter is authorized.

After correction, run final managed/desktop CI, strict all-feature lint/tests and
actual Native AOT publish/execute on Windows x64, Linux/WSLg x64 and native macOS
ARM64/APFS. Match source/build-input manifests, all eleven desktop fields, actual
Rust child execution, zero-charge shutdown and WB5 Save/Open. Earlier Mac evidence
remains historical evidence for the WB4-reading source. No new manual checklist,
ADR or architectural review is required.

The intermittent Windows ClipboardChecklistAsync failure remains unresolved.
Do not weaken tests or redesign clipboard behavior speculatively.

## Stop point

Submit the corrected, three-platform-validated handoff for final parent acceptance.
Do not start MEDIAN/MODE, formatting, Undo/Redo, structured-data expansion or further
compatibility work. This review records direction; no product code was changed
when receiving it. ADR-0021 remains open.
