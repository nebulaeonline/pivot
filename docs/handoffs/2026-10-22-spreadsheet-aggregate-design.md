# Spreadsheet aggregate functions — design review packet

Date: 2026-10-22. **Design approved with canonical MIN/MAX equality clarification; implementation authorized, not started.**
[Approval and implementation boundary](2026-10-22-spreadsheet-aggregate-design-review.md).
The decision requests below record design provenance; no implementation or new
platform acceptance is claimed.
The user authorized this design after discussing the parent's recommendation to
return to conventional spreadsheet functionality. Propose MIN, MAX, AVERAGE and
COUNT; defer MEDIAN/MODE and aliases. ADR-0020 remains accepted and closed.
[ADR-0021](../adr/0021-spreadsheet-aggregate-functions.md),
[preceding acceptance](2026-10-22-worksheet-structured-binding-implementation-review.md).

## Decisions requested

1. One rectangular range per function, composable in arithmetic. Ignore literal text
   and blanks, include numeric zeros/formula values, propagate all formula errors.
   COUNT of no numbers is zero; other functions return typed NoNumericValues.
   Preserve strict existing SUM behavior and canonical error order.
2. A narrow generic Blank leaf marker, with ordinary scalar zero behavior, lets
   the canonical evaluator distinguish blanks from numeric zeros. Blank/zero edits
   invalidate consumers even when scalar values compare equal. No frontend evaluator.
3. AVERAGE uses an exact fixed-size 256-bit scaled accumulator and checked count,
   with one final nearest rounding, exact midpoint away from zero. No intermediate
   Decimal96 overflow, float fallback or arithmetic-division semantic changes.
4. WB5 writing plus bounded legacy WB4 reading, because existing persisted tokens
   identify SUM and cannot transparently represent the new operation/kind semantics.
   Reconstruct legacy blank markers only after full old-format validation. Preserve
   CellIds/bound references, checkpoint and failed-Open atomicity. Hosted v1 and
   model-database formats remain frozen; no structured producer persistence.
5. Preserve CB2 layout and private v4 when existing source/reference and response
   representations suffice. Validate compatibility rather than renumbering them
   speculatively. A genuine additional format conflict returns to review.

## Implementation inspection and arithmetic exploration

- `pivot-workbook/src/persistent.rs`: anchor/Clear create numeric-zero backing;
  literal text uses the existing generic NonNumeric leaf.
- `pivot-core/src/model/structural.rs`: SUM uses canonical selection members and
  checked row-major Decimal96 addition. This traversal/error contract must remain.
- `pivot-core/src/model/selection.rs`: 4,096 points, 65,536 member edges per formula;
  workbook defaults are 1,024 live cells. The 256 bound is clipboard/request scope,
  not range-member capacity.
- `pivot-workbook-document/src/lib.rs`: PIVOTWB4 and explicit SUM graph/workbook
  token encoding require an intentional compatibility decision.
- `pivot-workbook/src/clipboard.rs`: CB2 carries source/effective coordinates, so
  new function recognition need not itself change payload layout.
- `pivot-types/src/lib.rs`: explicit rounding defaults midpoint-away; ordinary
  checked division retains its backend precision reduction. These are distinct.

`artifacts/aggregate-design/decimal-feasibility.json` records a design-only Python
exact-rational exploration: common-scale maximum sum needs 202 magnitude bits for
4,096 inputs and 254 for a u64-count bound. It records MAX/MAX, cancellation,
repeating decimals and positive/negative 1e-28 midpoint examples. This supports
feasibility of fixed-width accumulation; it is not production arithmetic validation,
a Rust limb implementation, benchmark or claim of platform parity.

## Required acceptance matrix for the later slice

| Case | Expected result |
| --- | --- |
| Numeric values, signs, explicit zeros, different scales | Correct extrema/count/rounded exact mean |
| Equal extrema with differing scales/signed-zero metadata, reordered | Canonical normalized MIN/MAX output; traversal independent |
| Empty coordinates, retained blank anchors, only text | COUNT zero; other new functions NoNumericValues |
| Blank anchor created indirectly by another scalar formula | No change to range count/mean |
| Blank -> numeric zero -> blank | Count/mean change; stable CellId; old captures unchanged |
| Formula `=blankCell` or `=0` as a range member | Numeric zero included |
| Numeric-looking literal text and escaped text | Ignored by new functions; SUM remains strict |
| Formula arithmetic, invalid draft, cycle or text-dependency error | Existing validation/first-error behavior, never silently skipped |
| Failing structured member in opt-in binding proof | Generic fault identity/document reason preserved; proof remains non-savable |
| Multiple failures across rows/columns and outer expressions | Actual canonical row-major/postfix precedence |
| MAX/MAX, MIN/MIN, large cancellation, reordered numeric inputs | No intermediate overflow; deterministic mean |
| Positive/negative exact midpoint at scale 28 and coefficient-fit boundary | One final midpoint-away rounding; canonical zero |
| Multiple function kinds sharing rectangle; repeated calls | Separate reduction results with correct dependencies |
| Value edit, move in/out, clear, retire and coordinate reuse | Existing fixed-geometry occupancy semantics |
| Relative/mixed/absolute/reversed endpoints on copy | Correct source/annotation translation; source IDs unaffected |
| Native/TSV rectangle, overlap/blanks, cross-process clipboard | Atomic existing behavior; malformed native never falls back |
| WB5 save/reopen with text/blanks/formulas/selection metadata | Stable IDs, exact values, kinds, semantics and rejected drafts |
| Legacy WB4 open then WB5 save/reopen | Same user-visible content, stable bindings and checkpoint behavior |
| Unknown aggregate/kind, inconsistent graph/workbook metadata or malformed WB4/WB5 | Reject before live-root/path/dirty/checkpoint mutation |
| Frozen hosted/model export cannot represent new semantics | Explicit refusal; no zero/SUM semantic substitution |
| More than 256 members under existing workbook/core caps | Correct reduction; no accidental clipboard-sized limit |
| Quota/cancellation/retained captures/final-owner release | Atomic edits, unchanged old results and zero charge |
| Real inline/formula-bar editing, error display, copy and save | Canonical C#–Rust results; no C# computation |

Require full managed CI, strict all-feature lint/tests and actual Native AOT desktop
execution on Windows x64, Linux/WSLg x64 and native Mac ARM64/APFS. Reuse shared
clipboard locking and current automated suites; no recurring manual replay requirement.
Fresh Mac source/.NET/publish/fixtures must be on APFS; reports return to SMB artifacts.
Do not claim new test counts until implemented. Preserve the unresolved Windows
ClipboardChecklistAsync observation, with focused diagnostics if it recurs.

## Stop

The parent approved the design with the extrema clarification and authorized bounded
implementation without another architecture cycle. Implementation must stop after
three-platform evidence for parent implementation review. No parser, evaluator,
protocol, clipboard or persistence implementation was changed for this packet.
No new dependency or capacity increase was introduced. The feasibility arithmetic
exploration is the only new executable observation. Formatting and Undo/Redo remain
candidate subsequent priorities, not bundled implementation work.
