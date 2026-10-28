# Canonical spreadsheet aggregates and blank definitions

ADR-0025 current-source extension: reducers ignore successful String/Boolean/Blank
values for MIN/MAX/AVERAGE/COUNT, including calculated strings. SUM remains strict
and errors still propagate in encounter order. COUNT remains Decimal96. Native
documents now use WB8; the older format milestones below are historical context.

ADR-0021 design is accepted; the bounded implementation is in place. Platform
validation and parent implementation acceptance are recorded separately in the
[implementation handoff](../handoffs/2026-10-22-spreadsheet-aggregate-implementation.md).
[Semantics and scope](../adr/0021-spreadsheet-aggregate-functions.md).

## Multi-argument calls (2026-10-28)

**Accepted and closed by parent review; no further qualification round required.**
[Feature evidence and review disposition](../handoffs/2026-10-28-multi-argument-aggregates.md).

SUM, MIN, MAX, AVERAGE and COUNT accept one or more comma-separated arguments:
same-sheet rectangles, scalar decimal expressions, cell references and nested
aggregate calls. Examples: `=SUM(A1:A10,C1:C10,5)`, `=MIN(A1,2*3,MAX(B1:B4,7))`
and `=AVERAGE(A1:A2,10)`. Existing single-rectangle calls retain their semantics.
ADR-0024 now adds sheet-qualified scalar and rectangular arguments, including
`=SUM(Revenue!A1:A10,Costs!A1:A10,5)`, on the same canonical graph. Qualification
is written on the first rectangle endpoint only; each rectangle belongs to one sheet.
Empty lists, missing arguments, whole axes, strings and
bare ranges in arithmetic remain unsupported. Existing 4,096-byte/64-level parser
and workbook/core resource limits are unchanged.

Arguments reduce left to right, each rectangle in row-major order. Repeated cells
or overlapping rectangles contribute once per occurrence. A direct scalar cell
argument follows its stable CellId when moved. Range membership follows current
occupants. Copy/fill translates every scalar/range reference with its own dollar
flags; Save/reopen preserves the already-bound identities, without rebinding source.

MIN/MAX/AVERAGE/COUNT ignore directly referenced Blank/NonNumeric leaves, as they
do inside a range. Explicit zero counts. A computed expression is numeric if it
succeeds: `COUNT(A1)` excludes a blank A1, while `COUNT(A1+0)` counts numeric zero.
A computed error propagates, including text-dependent arithmetic. SUM stays strict
about literal text and visits additions in argument/member order, with checked
Decimal96 arithmetic. COUNT does not suppress errors. The first error in traversal
order wins. MIN/MAX/AVERAGE over no numeric arguments return the existing workbook
NoNumericValues diagnostic; SUM and COUNT return zero.

AVERAGE uses one exact fixed-width accumulation across all arguments and rounds
once at final division. It does not average per-range averages or introduce
Decimal96 intermediate sums. Extrema still normalize their numeric results.

The shared Rust parser lowers calls into explicit postfix Start/Range/Value/End
controls around the existing bound scalar/selection operands. The trusted seam
validates nested stack frames and argument boundaries before mutation. No synthetic
nodes, frontend evaluator, function registry or worksheet callback enters the core.
Existing dependency/cycle/invalidation/capture infrastructure is reused. Members
are reduced inline to preserve ordering, with cancellation checks and runtime-domain
scratch reservations; argument frames and final mean work are charged separately.


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

Current native workbook read/write is WB7 under ADR-0024; WB1–WB6 reject explicitly
and no migration reader is provided. WB5 introduced explicit blank slots, and WB6
added multi-argument call controls. WB7 retains explicit blank slots
and aggregate metadata; redundant workbook/graph representations must match before
installation. Graph/workbook aggregate tag 5 carries MIN=1, MAX=2, AVERAGE=3,
COUNT=4; original SUM tags remain distinct. WB6 adds token tag 6 with validated
call-control codes (Start SUM/MIN/MAX/AVERAGE/COUNT=0–4, Range=5, Value=6, End=7).
The representation change requires the format cut; no migration reader is added. Failed Open preserves live roots, path,
revision, checkpoint and dirty state. Model-database and frozen hosted formats are
unchanged; structured proof documents remain non-savable.

Experimental workbook, native clipboard and private application formats support
only their current versions unless an identified consumer and explicit support
commitment justify compatibility. A feature alone does not require a version cut.

PIVOTCB2 still carries source/effective reference coordinates; new names reuse its
endpoint annotations. Translation preserves dollar flags, reversed ranges, scalar
CellId behavior and atomic paste. Older native clipboard readers can reject the new formula syntax; the clipboard
representation is unchanged. Unsupported native data must not fall back silently. Private application v4
continues carrying source/results/error messages without a schema change.

No resource ceilings increased. Clipboard/request bounds of 256 are distinct from
core MAX_POINTS=4,096 and default workbook live cells=1,024. MEDIAN/MODE, aliases,
formatting, Undo/Redo and structured-data expansion remain outside this slice.
