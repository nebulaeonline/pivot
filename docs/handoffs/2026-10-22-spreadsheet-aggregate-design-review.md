# ADR-0021 aggregate functions — design acceptance and implementation boundary

Date received: 2026-10-22. Parent review supplied by the user.
**Disposition: Approved with one semantic clarification.**
Authorized milestone: **Bounded MIN/MAX/AVERAGE/COUNT Implementation**.
[ADR-0021](../adr/0021-spreadsheet-aggregate-functions.md),
[design packet and acceptance matrix](2026-10-22-spreadsheet-aggregate-design.md).
This approves design and bounded implementation, not an implemented capability or
new validation result. No further architecture review is required after recording
the extrema clarification below. ADR-0020 remains accepted and closed.

## Approved semantics and required clarification

Support precisely MIN, MAX, AVERAGE and COUNT, case-insensitive single-rectangle
primaries composable with arithmetic. Preserve scalar CellId bindings, fixed-axis
range occupancy, endpoint orientation, dollar flags and copy translation.
Ignore direct blanks and literal text; include numeric inputs and numeric formula
results, including zero. Propagate formula errors and captured-source faults in
existing canonical row-major order. COUNT of no numeric values is zero; MIN/MAX/
AVERAGE return NoNumericValues. Preserve existing strict SUM behavior unchanged.

**MIN/MAX equality clarification:** compare numerically and return canonical numeric
results. Differing source scales and signed-zero metadata must not make equal extrema
produce traversal-dependent output. Normalize insignificant zeros and canonicalize
numeric zero; do not preserve the winning input's representation. Retain regression
tests for equal extrema under reordering.

Approve a narrow canonical Blank definition marker: scalar evaluation remains zero,
but direct aggregate membership distinguishes blank from explicit numeric zero.
Blank/zero transitions must invalidate aggregate consumers despite equal scalar values.
Preserve stable identities, atomic edits, immutable captures and existing resource caps.

AVERAGE uses private fixed-size 256-bit exact accumulation, checked count and one
final nearest rounding with exact midpoints away from zero. Choose the greatest
fitting final scale from the original exact sum, never a rounded intermediate.
Avoid Decimal96 intermediate sum overflow. The helper stays private; no new public
decimal type, arbitrary-precision dependency or ordinary-division semantic change.
Require independent exact-rational reference tests, including positive/negative
midpoints, cancellation, coefficient-fit boundaries, reordering and extreme inputs.
The design-only feasibility exploration is not production arithmetic validation.

## Compatibility approved explicitly

Approve PIVOTWB5 writes and a bounded, version-specific WB4 compatibility reader.
Validate legacy WB4 under its original rules before reconstructing Blank backing
from authoritative workbook content. Preserve CellIds, bindings, strict SUM semantics,
text, rejected drafts, checkpoints and failure atomicity. Successful legacy Open may
save as WB5; WB1/2/3 remain unsupported. Malformed/unsupported Open must leave live
workbook, dirty state, path and checkpoint untouched. Do not introduce heuristic
fallback decoding or alter frozen hosted/model-database formats; unsupported exports
reject explicitly.

Retain PIVOTCB2 and private application v4 if their existing representations suffice.
Demonstrate native/TSV copy with mixed dollar flags, reversed endpoints and overlaps.
No unknown aggregate may become SUM or silently downgrade. If a genuine clipboard
or protocol representation conflict appears, stop for review before changing formats.

## Implementation order, validation and stop

Establish focused Rust parser/binder, Blank, aggregate and private arithmetic tests
before desktop integration. Cover real canonical error order, definition invalidation,
identity/move/retire, immutable captures, independent rational parity, admission,
cancellation, copy and WB4/WB5 persistence. C# remains presentation/transport only.
The 4,096 core selection-point bound and 256 clipboard-cell bound remain distinct;
no workbook/core capacity increase is authorized.

Require complete managed CI, strict all-feature lint/tests and actual Native AOT
publish/execute on Windows x64, Linux/WSLg x64 and native macOS ARM64/APFS, with
existing clipboard serialization and zero-charge cleanup. Return a three-platform
handoff and stop for parent implementation review. Fresh Mac testing must use APFS
for source/build/publish/fixtures, with reports returned to the SMB artifacts tree.

The Windows ClipboardChecklistAsync flakiness remains unresolved. Preserve failure
history; if it recurs, add focused clipboard availability, UI sequencing, commit and
readback diagnostics. Do not conceal it through this feature or weaken tests. Retain
pre-existing WSLg startup, mounted-filesystem Save and historical timing limits separately.

No MEDIAN/MODE/aliases, function registry, general argument lists, structured-data
expansion, permanent # syntax, connectors, formatting, Undo/Redo, structural row editing
or capacity expansion. Formatting/Undo/Redo remain candidate later priorities.

Implementation follow-up: [current bounded implementation and validation](2026-10-22-spreadsheet-aggregate-implementation.md).
Design acceptance above does not constitute implementation acceptance.
