# ADR-0025: Typed spreadsheet values

Status: **Accepted with parent clarifications; implementation authorized as one bounded
feature milestone (2026-10-28).**
Date: 2026-10-28. Design accepted; implementation and scoped validation complete.
Implementation acceptance remains pending.
[Implementation handoff and evidence](../handoffs/2026-10-28-typed-spreadsheet-values.md).

## Decision and scope

Separate a cell's definition, calculated value and presentation. A literal and a
formula may both produce a String; they are not different string types. Display
rounding never modifies a calculation input. Formatting follows this milestone.

Implement Decimal96, Boolean and String results, retaining distinct Blank and typed
errors. Add comparisons, IF/AND/OR/NOT, LEN/MID and string concatenation. Reuse one
canonical Rust calculation graph, stable bindings, immutable captures, aggregates
and multiple worksheets. No frontend evaluator or second expression engine.

Do not implement Integer storage/arithmetic, dates, times, durations, floating point,
formatting, conversion functions, new rounding functions, connectors or history.
ADR-0023 remains parked. This is one product milestone after contract approval,
not a series of research gates.

Parent clarifications: short circuit must be genuinely demand-driven in the canonical
evaluator and tested there, not only in function helpers. Unicode comparison remains
ordinal by scalar sequence: canonically equivalent encodings may compare unequal;
there is no implicit normalization or linguistic/locale collation. Make only the
necessary typed representation cuts, verify clipboard ownership and malformed-data
handling, and do not introduce a compatibility or transport workstream.

## Value, definition and error contract

| Kind | Meaning in this slice |
| --- | --- |
| Decimal96 | Existing checked decimal value and numeric-operation policy |
| Boolean | Exactly TRUE or FALSE; not a numeric 1 or 0 |
| String | Bounded UTF-8 text; literal and calculated strings have identical value semantics |
| Blank | Missing/cleared cell, distinct from explicit zero, FALSE and empty String |
| Error | Typed failed evaluation with origin; separate from successful values |

Conceptually use `Value = Decimal | Boolean | String | Blank`, with evaluation
returning `Result<Value, EvaluationError>`. Error is a visible result category, not
an ordinary operand or authored literal. Preserve existing error distinctions and
captured-source fault identities; add distinguishable TypeMismatch,
InvalidArgument and TextTooLong failures. Rendering an error does not turn it into
a String. Resource/cancellation failures remain operation failures, not string data.

Keep definition categories (literal, expression, blank and invalid draft) distinct
from result kinds. Literal strings become real graph inputs rather than retaining
the numeric NonNumeric marker as a substitute for string evaluation. The document
remains their authority; graph definitions/captures share or retain admitted
immutable payloads, with no uncharged parallel text store. Submitted decimal scale
and authored formula provenance remain metadata, separate from result equality.

### Integer policy

For this milestone, every entered number, numeric literal, COUNT and LEN result is
Decimal96, including integer-valued results. `42+0.5` is Decimal 42.5; `5/2` is
Decimal 2.5. No inference from spelling, scale, display format or function name
creates an Integer type. No new fixed-width overflow boundary is introduced.

A future first-class Integer requires explicit promotion, division, comparison,
overflow, conversions and function-result contracts. Do not reserve Int64 semantics
while physically returning Decimal. Date/time are likewise future semantic types,
not implicitly interchangeable numeric serials. ISO-looking dates remain text.

## Authoring and operations

Bare TRUE/FALSE entries and formula literals are case-insensitive Booleans.
An apostrophe forces literal text as today: `'TRUE` remains a String. In formulas,
double quotes delimit strings and doubled double quotes escape a quote;
`="He said ""yes"""` produces `He said "yes"`. Backslashes have no escape role.
Retain single-line formula authoring and existing source/depth limits. Quoted sheet
names, including apostrophe escapes, retain ADR-0024's independent grammar.
Qualified references such as `TRUE!A1` still resolve their sheet qualifier before
literal recognition. Reserve Boolean literals in spreadsheet syntax only; preserve
existing hosted decimal-model name rules rather than silently rebinding named inputs.

Strings are not normalized, trimmed, case-folded or parsed as numbers implicitly.
Existing literal text entry/escape rules otherwise remain. No automatic date or
currency parsing is added. Formula editing retains the expression rather than
replacing it with its displayed Boolean or String result.

| Operation | Admitted values and result |
| --- | --- |
| Unary +/- and + - * / | Decimal; Blank acts as Decimal zero. Boolean/String cause TypeMismatch. Existing decimal errors/rounding apply. |
| `&` | String concatenation; Blank acts as empty String. Decimal/Boolean cause TypeMismatch; explicit conversions are deferred. |
| `= <> < <= > >=` | Same-kind Decimal, Boolean or String operands; result Boolean. Decimal compares numerically, FALSE precedes TRUE, String compares ordinal Unicode scalar sequences. |
| Blank comparisons | Blank equals Blank; with Decimal, Blank acts as zero. Blank versus Boolean/String and other mixed-kind comparisons cause TypeMismatch. |
| IF(condition, then, else) | Exactly three scalar arguments. Condition must be Boolean or Blank (FALSE). Only the chosen value branch evaluates; result may be any successful kind. |
| AND / OR | One or more scalar Boolean/Blank arguments. Blank acts as FALSE. Left-to-right short circuit on FALSE / TRUE respectively. |
| NOT | Exactly one Boolean/Blank argument; Boolean result. |
| LEN(text) | String or Blank (empty String); exact Decimal count of Unicode scalar values. |
| MID(text,start,count) | String/Blank plus integer-valued Decimals: start >= 1, count >= 0. One-based Unicode scalar indexing. Beyond end returns empty String; count clamps to remaining text. |

MID rejects fractional/negative/out-of-domain arguments without rounding. Compare
indices against the bounded text length before machine-integer conversion; a valid
huge start yields empty String and a huge count clamps, without integer overflow.
LEN/MID count Unicode scalars, not UTF-8 bytes, UTF-16 units or grapheme clusters;
combining sequences can be split. Identical input has identical results across OSes.

Operator precedence, highest first: parentheses/function calls, unary +/-, * /,
+ -, &, comparisons. Binary arithmetic/concatenation are left-associative; chained
comparisons reject rather than acquiring implicit Boolean/numeric conversions.
Function names are case-insensitive. Logical functions do not accept bare ranges
in this slice; existing reducers continue to accept their scalar/range arguments.

Preserve the existing formula-reference blank convention: a formula whose final
successful result is Blank exposes Decimal zero. Thus an empty A1 is Blank, but
`B1 = A1` produces Decimal zero and COUNT(B1) is one. Intermediate Blank remains
available for the explicit rules above. An empty String result stays String.

### Aggregates and errors

SUM retains checked addition and strict nonnumeric handling: Decimal contributes,
Blank contributes zero, and Boolean/String produces TypeMismatch (or the existing
compatible nonnumeric diagnostic mapped to that category).
MIN/MAX/AVERAGE/COUNT include Decimal and ignore successful Blank/Boolean/String
arguments or members, regardless of literal versus formula definition. Numeric-looking
strings are not parsed. This explicitly extends the existing text-ignoring policy
to newly available calculated strings and Booleans. Errors still propagate; COUNT
does not suppress division-by-zero or type errors inside an argument expression.

Argument evaluation remains left-to-right; rectangles traverse row-major, overlaps
count per occurrence, and AVERAGE rounds only once per call. Preserve existing empty
reduction behavior. Ordinary operators/functions propagate the first encountered
error; IF/AND/OR do not propagate faults from branches/arguments they do not demand.
`IF(FALSE,1/0,"ok")` succeeds; `AND(FALSE,1/0)` is FALSE, while
`AND(1/0,FALSE)` fails. IF permits different result kinds in its branches.

All syntactic references bind and participate in static dependencies, invalidation
and cycle checks, including inactive branches. No branch-sensitive graph pruning.
An unknown sheet or invalid syntax in an inactive branch remains an invalid draft;
an inactive cycle still fails graph validation/evaluation. Short circuit applies to
runtime value evaluation, not binding or structural validity. Captures pin one coherent
generation, including any captured sources; short circuit does not bypass source
snapshot admission, cancellation, dependency-scan work or other resource authority.

## Implementation plan and boundaries

1. Extend the shared value/error contract and the existing parser/bound representation.
   Use explicit conditional/logical expression structure with validated arity and
   branches, not eager arithmetic postfix disguised as IF. Add demand-driven branch
   evaluation to the canonical evaluator while retaining static structural checks.
   Existing decimal-facing APIs remain checked adapters; nondecimal results cannot
   be silently coerced or serialized as decimal zero.
2. Integrate typed inputs/results through workbook edits, reads, captures, aggregates,
   cross-sheet binding and copy/fill. Preserve immutable ownership and global budgets.
   Admit each string at <=4,096 UTF-8 bytes, including computed outputs. Check joined
   lengths before allocation, charge retained/cached strings and transient work to
   the existing runtime domain, and poll cancellation. Existing source, frame,
   clipboard, member/token/depth and workbook limits remain; no per-cell unlimited
   cache or multiplication of budgets by sheet count.
3. Update the owned session and .NET client with tagged successful results and typed
   errors, independently of the cell's definition arm. Desktop displays Booleans as
   TRUE/FALSE and String results as text using existing clipping; formula editing and
   stale-response guards remain. Literal versus formula definitions survive Save and
   clipboard, rather than copying only a displayed result.
4. Persist typed definitions and new expression controls, then run focused regressions
   and actual-window automation. Deliver one implementation handoff and stop for review.

Use **WB8**, **private session v6** and **native clipboard CB3** where required by incompatible
typed payloads/expression semantics. Explicitly reject earlier experimental versions;
malformed/unsupported native clipboard never falls back silently to text. Document
the format cut and the outstanding pre-user compatibility obligation. No migration
framework. Current clipboard transport identifiers must distinguish CB3 from CB2.

Frozen hosted v1 remains Decimal-only with its established protocol unchanged.
Retain existing decimal behavior; exports/invocations requiring unsupported typed
definitions or results reject explicitly at the boundary. Do not advertise new
spreadsheet types through the daemon by smuggling them into decimal fields.
Parked history adapters and separate numeric structured-result proofs may reject
unsupported typed inputs explicitly; keep existing numeric proof regressions working
and optional builds compiling without expanding either subsystem.

## Acceptance examples and stopping point

Cover literal/formula Booleans and strings, empty String versus Blank/zero/FALSE,
escaped quotes, supplementary Unicode and combining sequences, all comparison and
coercion rules, argument validation, short circuit versus static cycles, mixed-type
aggregates and error ordering. Include `COUNT("42",TRUE,7)=1`, strict SUM text errors,
`LEN("😀a")=2`, cross-sheet IF/text dependencies, old captures after type-changing
edits, renamed qualifiers, translated Copy/Fill, computed text limits and atomic
quota/cancellation failures. Save/reopen must preserve types, authored formulas,
bindings and all sheets; failed Open/paste preserves prior roots and dirty state.

Use focused Rust, typed session/protocol and decimal-regression tests, plus native
Windows x64/Linux x64/macOS ARM64 headless and desktop automation as warranted by
the changed code. Verify the tagged .NET result path is compatible with existing
Native AOT serialization; no new platform qualification program. Preserve existing
numeric and hosted-v1 commitments. Do not accept merely displaying Booleans/strings
while calculations still use a decimal-only placeholder path.

Parent approved this contract and authorized direct implementation as one milestone.
Deliver one concise feature handoff with relevant evidence and material limitations,
then stop for parent implementation review. Formatting follows without waiting for
Integer/date/time implementation; its metadata must not change these values.
