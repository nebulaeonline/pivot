# Canonical spreadsheet aggregates and blank definitions

ADR-0021 design is accepted; the bounded implementation is in place. Platform
validation and parent implementation acceptance are recorded separately in the
[implementation handoff](../handoffs/2026-10-22-spreadsheet-aggregate-implementation.md).
[Semantics and scope](../adr/0021-spreadsheet-aggregate-functions.md).

MIN, MAX, AVERAGE and COUNT accept one same-sheet rectangle, case-insensitively,
and compose with ordinary arithmetic. The existing range AST/workbook tokens now
carry a closed AggregateKind. The canonical bound graph retains its existing SUM
operand and adds an operation-bearing aggregate selection operand. Dependency,
cycle, invalidation, capture and row-major membership infrastructure are shared.
Reduction caches key by operation and selection identity. No C# evaluator or general
function registry is introduced.

Direct Blank and NonNumeric leaves are excluded by the four new reducers. Numeric
inputs and successfully evaluated formulas count, including zero. Computed errors
propagate in existing row-major order; outer expression precedence remains postfix.
COUNT of no numbers is zero; MIN/MAX/AVERAGE return NoNumericValues. SUM remains
strict about literal text, treats blanks as zero and retains checked addition order.
MIN/MAX normalize results, including equal-scale variants and signed zero.

Blank is now a canonical definition kind in persistent definitions, inspection and
scalar snapshots. It evaluates as scalar zero but is not an ordinary numeric input.
Clear/anchor creation installs Blank; entering zero installs Input(0), with a real
definition revision and aggregate invalidation. Snapshot/frozen export cannot erase
this distinction silently: unsupported hosted/scalar exports reject. Workbook
selection-aware transfer carries a validated blank-slot overlay (internal version 3).
Its scalar skeleton has zero placeholders only within this explicit envelope.

The private AVERAGE helper uses four u64 magnitude limbs, a sign and checked count.
Inputs are aligned exactly to scale 28. The bounded sum fits within 202 magnitude
bits for 4,096 points. Final division uses the original exact sum at each candidate
scale 28..0; round nearest, exact midpoint away from zero, select a fitting 96-bit
coefficient and normalize. No Decimal96 intermediate sum, floating point, unbounded
bigint dependency or change to ordinary division is involved. Reducer/cache scratch
is covered by runtime-domain reservations. Operation-bearing formulas separately
reserve 192 bytes per token for cache/set metadata alongside existing stack/member
reservations; pure SUM admission is unchanged. Extra work is charged through
existing invocation budgets, and member traversal polls cancellation. The fixed
finalization loop is bounded independently of input values.

Native workbook read/write supports PIVOTWB5 only. WB1–WB4 explicitly reject;
no obsolete decoder or migration path remains. WB5 stores explicit blank slots
and aggregate metadata; redundant workbook/graph representations must match before
installation. Graph/workbook aggregate tag 5 carries MIN=1, MAX=2, AVERAGE=3,
COUNT=4; original SUM tags remain distinct. Failed Open preserves live roots, path,
revision, checkpoint and dirty state. Model-database and frozen hosted formats are
unchanged; structured proof documents remain non-savable.

Experimental workbook, native clipboard and private application formats support
only their current versions unless an identified consumer and explicit support
commitment justify compatibility. A feature alone does not require a version cut.

PIVOTCB2 still carries source/effective reference coordinates; new names reuse its
endpoint annotations. Translation preserves dollar flags, reversed ranges, scalar
CellId behavior and atomic paste. A preserved old reader rejects new-function native
payloads and WB5 files without installing partial content. Private application v4
continues carrying source/results/error messages without a schema change.

No resource ceilings increased. Clipboard/request bounds of 256 are distinct from
core MAX_POINTS=4,096 and default workbook live cells=1,024. MEDIAN/MODE, aliases,
formatting, Undo/Redo and structured-data expansion remain outside this slice.
