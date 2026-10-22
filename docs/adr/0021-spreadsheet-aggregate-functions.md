# ADR-0021: Bounded spreadsheet aggregate functions

Status: **Accepted with MIN/MAX canonical-equality clarification; bounded implementation in place; three-platform validation passed for current source; WB5-only correction and final acceptance pending (2026-10-22)**.
[Design acceptance and implementation boundary](../handoffs/2026-10-22-spreadsheet-aggregate-design-review.md).
Date: 2026-10-22.
[Review packet](../handoffs/2026-10-22-spreadsheet-aggregate-design.md).
Predecessors: [range semantics and SUM](0016-first-class-ranges-and-sum.md),
[worksheet text](0018-document-owned-worksheet-text.md),
[closed structured-binding proof](../handoffs/2026-10-22-worksheet-structured-binding-implementation-review.md).
ADR-0020 remains accepted and closed. This proposal returns attention to conventional
spreadsheet functionality; no new structured-data milestone is proposed.

## Interim compatibility correction — final acceptance condition

The interim parent response supersedes the WB4 compatibility target below:
final ADR-0021 acceptance requires WB5-only read/write and explicit WB1–WB4
rejection, retaining current-format identity, blank/SUM, atomic Open and dirty-state
invariants. The currently validated source still includes WB4 migration. The
completed Mac run is valid only for that source. Reader removal and fresh final
three-platform validation remain pending; the interim response explicitly is not
an implementation assignment or acceptance decision.
[Evidence and policy record](../handoffs/2026-10-22-spreadsheet-aggregate-implementation.md#completed-macos-evidence-and-interim-policy-correction).
Historical design approval below documents the earlier decision.

## Scope and authoring

Add MIN, MAX, AVERAGE and COUNT as case-insensitive, single-rectangle expression
primaries, for example `=AVERAGE(A1:A10)` and `=MIN($A$1:B4)+MAX(C1:C4)*2`.
Use the existing optional-leading-equals, whitespace, endpoint orientation and
absolute/relative reference rules. An aggregate result may participate in arithmetic;
this slice does not accept argument lists, scalar arguments, whole columns/rows,
cross-sheet ranges, nested range expressions or user-defined functions.

Retain existing SUM grammar and strict semantics. Defer MEDIAN, MODE, AVG and MEAN
aliases, formatting, Undo/Redo and general function registration. Propose a closed
AggregateKind plus a range-call AST node, not an extensible runtime function registry.
The Rust parser/binder/evaluator owns all semantics; C# only submits and displays.

## Observable reduction contract

All four new functions visit current range occupants in the captured generation.
Keep ADR-0016's inclusive fixed-axis rectangle, sparse membership, stable selection
identity and existing move/retire/coordinate-reuse rules. A scalar A1 operand remains
CellId-bound; a range follows occupants of its bound coordinates. Copy translates
range endpoints using their independent absolute flags, retaining source orientation.

| Member | MIN / MAX / AVERAGE / COUNT |
| --- | --- |
| Numeric input, including explicit zero | Include once |
| Formula successfully producing Decimal96 | Include once, even if its value is zero |
| Blank or absent coordinate | Ignore |
| Literal text, including numeric-looking text | Ignore |
| Formula error, including a text-dependent NonNumericInput | Propagate |
| Captured-source error in an ordinary member formula | Propagate existing fault ID and document reason |

Classification uses the member's committed definition, not its displayed value and
not an attempt to ignore arbitrary evaluation failures. A formula `=B1` returning
zero because B1 is blank is numeric and counts; B1 itself remains blank and does not.
Selecting/materializing an empty anchor must not change COUNT or AVERAGE.

COUNT returns an exact Decimal96 integer, including zero for no numeric members.
MIN/MAX return the numeric extremum; AVERAGE returns their arithmetic mean.
MIN/MAX compare numerically and return normalized canonical numeric results, including
canonical zero. Equal values with differing scales or signed-zero metadata cannot
make the result depend on which member appeared first. Preserve no source-format
provenance in extrema; regression-test ties under reordering. MIN,
MAX and AVERAGE return an explicit `NoNumericValues { node, operation, selection }`
error when the set has no numeric values. Empty geometry is not authored: a valid
rectangle may have zero occupied or zero numeric members. Reversed endpoints retain
existing canonicalization rather than creating an empty range.

All four propagate the first member error in established row-major traversal.
COUNT does not suppress formula errors. Outer expression error precedence remains
the existing canonical postfix behavior; independent outputs remain evaluable.
SUM still treats blanks as zero and literal text as NonNumericInput. No coercion of
text to decimals or general text-valued formula runtime is added.

## Narrow graph representation

Current workbook Blank/Clear uses an ordinary numeric-zero input. That representation
cannot distinguish an explicit zero for COUNT. Propose a generic **Blank leaf marker**
in the canonical graph, analogous in scope to the existing NonNumeric marker.
Its ordinary scalar evaluation is exact zero, preserving current arithmetic and SUM.
Its direct range-member classification is blank. No string, cell address, producer
or workbook callback enters the evaluator.

New sparse anchors and Clear use the marker; explicit SetDecimal(0) uses numeric input.
Blank-to-zero and zero-to-blank are definition changes even when scalar numeric results
are equal. They invalidate selection consumers, preserve CellId and stage atomically.
Move, retire, import and copy preserve this distinction. An invalid draft remains an
error, not blank. Numeric input overrides must not silently redefine marker kind.

Add a closed typed aggregate-selection operand alongside the existing SUM operand.
Reuse canonical selection indexes, dependency/cycle validation, invalidation, immutable
captures and cancellation. Reduction cache keys include operation and SelectionId;
COUNT and AVERAGE must not share a SUM result accidentally. Direct member definition
classifies Blank/NonNumeric; computed errors still propagate. No duplicate worksheet
calculation path, frontend reduction, hidden input or evaluated-value string parsing.

## AVERAGE: exact accumulation and one final rounding

Propose a fixed-size signed-magnitude 256-bit accumulator with a checked u64 count.
For each included Decimal96 coefficient c with scale s, accumulate signed
`c * 10^(28-s)` exactly. Do not sum through Decimal96 or round each addition.
The current MAX_POINTS is 4,096: even all maximum-magnitude coefficients need only
202 magnitude bits at this common scale. A u64 count's maximum would need 254 bits;
this is a width proof, not authorization to increase any capacity.

For a nonzero count n, select the greatest scale t in 0..28 for which rounding
`abs(sum) / (n * 10^(28-t))` yields a coefficient fitting 96 bits. Obtain the quotient
and remainder from the original exact sum for each candidate scale; never reround
an already rounded candidate. Round to nearest with exact midpoints **away from zero**,
apply sign and normalize trailing zeros/canonical zero. This matches the existing
explicit rounding default requested for Pivot. Existing checked arithmetic's backend
precision reduction remains unchanged; AVERAGE is a newly specified operation, not
an alteration to ordinary division.

Consequences:

- AVERAGE of 1,2,2 is `1.6666666666666666666666666667`.
- AVERAGE of MAX,MAX is MAX, without intermediate overflow.
- MAX,-MAX,1 averages to `0.3333333333333333333333333333`, independent of input order.
- Zero and 1e-28 average to 1e-28; zero and -1e-28 average to -1e-28.

With finite Decimal96 inputs the exact mean lies between the extrema. It has a
rounded representation within the Decimal96 interval; a valid bounded numeric range
must not fail merely because an intermediate Decimal96 sum would overflow.
Any unexpected internal width/count failure is explicit, not truncation or float
fallback. Prove and test multiply/power-of-ten alignment, signed cancellation,
division/remainder and final rounding before wiring this helper into the evaluator.

Use a small internal fixed-limb helper rather than an unbounded bigint dependency.
The 256-bit accumulator is transient arithmetic scratch, not a new public numeric
type. Reserve/account fixed reducer scratch before traversal; include per-operation
cache metadata in existing domain accounting. Reuse the bounded member enumeration;
do not collect/sort another array of values. Poll existing cancellation/work budgets.
MIN/MAX/COUNT need only scalar running state. All share existing graph/selection caps.

## Persistence, transfer and clipboard — explicit review decision

Inspection found WB4 explicitly serializes SUM-only graph/workbook tokens. It also
stores blank backing as zero. Reusing that meaning for another operation or silently
reading new aggregate tags as SUM would be incompatible. This proposal therefore
records approval for **WB5 writing with bounded legacy WB4 reading**, not a silent
reinterpretation of WB4. This is ordinary workbook evolution, not persisted structured
producers/projections; the ADR-0020 experiment remains non-savable.

WB5 stores explicit aggregate operation/selection metadata and blank-kind metadata,
with redundant workbook/graph source, identity and type consistency validated before
installation. The existing scalar document skeleton may retain zero placeholders
only inside this validated workbook envelope with explicit blank overlay; a standalone
frozen scalar export must not silently erase observable blank or aggregate semantics.
Keep hosted v1 and model-database formats unchanged; unsupported exports reject.

A legacy WB4 reader first validates the old document under its original rules, then
reconstructs blank-kind backing from authoritative Content::Blank metadata, preserving
ordinary CellIds, bound targets, ranges, text, rejected drafts and numeric values.
Open does not re-resolve source names or execute rejected drafts. Successful legacy
Open creates the normal saved checkpoint; subsequent Save writes WB5. WB1/2/3 remain
unsupported. Malformed/unsupported Open preserves live workbook, dirty state, path
and checkpoint. New WB5 files reject in old readers by magic/version. No automatic
batch file conversion or database migration is included.

PIVOTCB2 carries formula source and effective reference coordinates rather than an
encoded SUM opcode. Retain its layout if new functions can use the same validated
endpoint annotations; extend parsing/translation, not the native payload shape.
Require old-reader rejection of unsupported valid functions, no semantic downgrade,
and existing malformed-native no-TSV-fallback behavior. Stop for review if a CB format
change is actually needed. Ordinary TSV authoring follows the new Rust grammar.
Private application v4 can continue carrying source/results/error strings if no wire
shape changes are needed. Do not change frozen hosted protocol v1 for these functions.

## Limits, validation and approval boundary

The default workbook remains 1,024 live/8,192 allocated cells on 1,024-by-1,024 axes.
Core selection MAX_POINTS remains 4,096, MAX_MEMBER_EDGES_PER_FORMULA 65,536;
retain existing descriptor/consumer/identity/admission limits. The 256-cell clipboard
and session request/read bounds do not limit computational range cardinality.
No capacity increase, scheduler or new graph architecture is authorized by this design.

The linked acceptance matrix requires parser/binder, real evaluator, decimal reference,
copy/native/TSV, WB4 legacy/WB5 round trips, immutable captures, fault precedence,
atomic rejection, cancellation/accounting and desktop/IPC tests. Later implementation
must produce full managed CI, strict all-feature checks and actual Native AOT desktop
coverage on Windows, native Linux/WSLg and native macOS/APFS, with zero charge.
Do not invent expected final test counts before implementation.

**Design accepted with the canonical MIN/MAX equality clarification above.**
The parent authorizes the bounded implementation without another architecture review.
Build focused Rust coverage before desktop integration, then supply complete
three-platform managed/Native AOT evidence and stop for parent implementation review.
If compatibility requires a CB/private protocol change, return for review first.
The implementation is in place; [evidence and remaining gates](../handoffs/2026-10-22-spreadsheet-aggregate-implementation.md).
Acceptance here is design/scope approval, not implementation acceptance.

Parent implementation review: [computational implementation acceptable; final
acceptance deferred pending WB5-only correction and fresh three-platform gates](../handoffs/2026-10-22-spreadsheet-aggregate-implementation-review.md).
