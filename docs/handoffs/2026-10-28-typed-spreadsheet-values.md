# Typed spreadsheet values — ADR-0025 implementation

Date: 2026-10-28. Design accepted; implementation and scoped validation complete,
ready for parent implementation review. ADR-0023 stays parked.

## Delivered behavior

Cells and formulas now produce genuine Decimal96, Boolean, String and Blank values,
with typed errors. Literal text is a graph value, not a numeric placeholder. Workbook
definitions retain literal/formula provenance independently of the result kind.
Integer-valued literals, COUNT and LEN remain Decimal96; no implicit numeric/string
or Boolean/numeric conversion was introduced.

Comparisons, IF/AND/OR/NOT, LEN, MID and `&` work through the canonical Rust graph:
`=IF(Stock!A1>0,"In stock","Sold out")`, `=MID("😀abc",2,99)` and
`=COUNT("42",TRUE,7)`. String comparisons use ordinal Unicode scalar sequences,
without normalization or locale collation; LEN/MID index scalars. MID validates
integer arguments and bounds huge indices before machine-integer conversion.

IF/AND/OR demand only necessary value branches. Explicit work stacks avoid recursion
through dependency chains. Static binding, invalidation, cycle validation, captured
source admission and resource checks still include inactive references. Tests verify
an unused erroring dependency is absent from actual evaluated results, through both
the typed API and ordinary canonical Decimal API, including scenario overrides.
The Decimal adapter dispatches typed closures to demand execution; existing pure
Decimal evaluation and the numerical contract remain available.

Empty String, FALSE, zero and Blank remain distinct. An apostrophe alone enters an
empty literal String; clear creates Blank. As agreed, a formula's final Blank result
exposes Decimal zero. MIN/MAX/AVERAGE/COUNT ignore successful String/Boolean/Blank
members, including calculated strings; SUM stays strict. Errors retain encounter
order and origin. Computed text is limited to 4,096 UTF-8 bytes, with admitted
payload/cache/temporary allocation and cancellation checks under existing budgets.

Typed results pass through immutable captures, cross-sheet formulas, rename, native
copy/paste, Fill and complete Save/reopen. The desktop displays Booleans and calculated
strings while F2 edits the original expression. Private v6 has an authoritative tagged
result independent of the definition, typed fault codes and a validated Decimal-only
convenience mirror. Native documents use WB8; Boolean clipboard slots and empty text
semantics require CB3 and distinct platform clipboard identifiers. Older versions
reject explicitly, and malformed native data never silently falls back to text.

## Validation and evidence

| Gate | Result |
| --- | --- |
| Changed Rust crates/private session, all features | 253 passed, including existing optional history regressions |
| Formatting and strict all-target/all-feature Clippy | Passed |
| .NET typed session, protocol, capture, clipboard and Save/reopen | Passed |
| Native headless CI before final admission correction | Windows x64/Linux x64/macOS ARM64 passed 13/13 each |
| Final admission correction headless CI | Windows x64/Linux x64/macOS ARM64 passed 13/13 each; all 356 implementation inputs match the current source |
| Managed actual-window suite | Windows x64/Linux x64/macOS ARM64 passed |
| Native AOT publish and actual-window suite | macOS ARM64 passed; no fresh Windows/Linux AOT claim |

Managed reports: [20261028T192408Z-5c798a3a](../../artifacts/validation-matrix/20261028T192408Z-5c798a3a/).
Native AOT: [20261028T192414Z-479adc13](../../artifacts/validation-matrix/20261028T192414Z-479adc13/).
Each GUI target/mode produced four successful real application reports, zero remaining
runtime charges and no owned process survivors. The explicit `typed_values` assertion
covers typed grid results, Unicode LEN, lazy IF, editor provenance, OS native clipboard
translation and WB8 Save/reopen. AOT additionally verifies the published native image,
generated JSON metadata, process architecture and peer processes.

Initial native headless evidence:
[20261028T191237Z-1cc8ae18](../../artifacts/validation-matrix/20261028T191237Z-1cc8ae18/).
All 356 non-Markdown implementation inputs matched those snapshots. Final native
headless reports and reconciled input hashes (356 matching inputs on each target,
with no subsequent implementation edits) are under
[`artifacts/typed-local/final-headless`](../../artifacts/typed-local/final-headless/).
Final local logs: `artifacts/typed-rust-admission-final.log` and
`artifacts/typed-clippy-final.log`; session evidence: `artifacts/typed-local/session`.

**Source scope:** managed/AOT ran their prepared snapshots before the final Rust-only
allocation-admission ordering correction. That correction reserves opcode/string
construction and selection-walk scratch before allocating; it changes no C#, protocol,
operator or value semantics. It passed the 253-test local suite and is separately
covered by the final native headless reruns. Earlier GUI/AOT results are evidence for
their recorded sources, not a claim that the final Rust child was rerun graphically.
Original reports and the prior Rust module are preserved. No unrelated six-RID or
VM isolation qualification was reopened.

Existing regressions caught and retained coverage for nested aggregate scalar results
and left-to-right type-error precedence. Updated expectations reflect approved changes
only: comparisons are supported, quoted empty text is String, and successful calculated
strings are ignored by the four non-SUM reducers. Parked adapters preserve their prior
literal-text behavior with the typed constant representation, explicitly refusing new
unsupported typed expressions/Booleans. No history research was resumed.

## Limits and stopping point

No formatting, Int64 arithmetic, automatic text/number conversion, dates/times,
additional text functions or ROUND function was added. Display rounding remains
separate from calculation. The frozen hosted v1 boundary remains Decimal-only;
unsupported typed transfers/invocations reject rather than inventing numeric values.
Existing worksheet limits and deferred operations from ADR-0024 remain.

WB8 rejects earlier experimental documents; no migration framework is included.
Establish a deliberate compatibility commitment before users depend on valuable
saved files. Implementation is complete; stop for parent implementation review with
this single handoff. Acceptance is not yet claimed. ADR-0023 remains parked at G3.
