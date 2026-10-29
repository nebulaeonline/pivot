# ADR-0027: Calculation rounding functions

Date: 2026-10-29. Owner authorized the bounded feature following ADR-0026 acceptance.
Status: **Accepted and closed by parent implementation review (2026-10-29).**
[Single implementation handoff and validation evidence](../handoffs/2026-10-29-calculation-rounding.md).
Native Windows/Linux/Mac headless CI passed 13/13 each with 368 matching inputs;
268 all-feature Rust tests and real .NET session checks passed. Ubuntu recovery from
native disk exhaustion is recorded separately without relabeling the original failure.
Parent accepts the numeric contract, independent references, all-feature tests,
Clippy, .NET session regressions and native three-platform evidence. The frozen-source
Ubuntu recovery is accepted without reclassifying the original attempt. WB9/private
v7/CB4 remain unchanged. No additional GUI/AOT, migration, platform qualification or
reconciliation round is required. Artifact retention remains the housekeeping standard;
ADR-0023 remains parked at G3.

## Decision

Add `ROUND(number, digits)`, `ROUNDUP(number, digits)` and `ROUNDDOWN(number, digits)`
to the existing spreadsheet parser and canonical Rust evaluator. Both arguments
are required scalar expressions. ROUND is nearest, with ties away from zero;
ROUNDUP is away from zero, not ceiling; ROUNDDOWN is toward zero, not floor.
Results are genuine normalized Decimal96 values. Formatting remains separate.

Positive digits select fractional decimal places; zero selects units; negative
digits select tens, hundreds and larger places. Examples: `ROUND(-2.5,0)=-3`,
`ROUNDUP(-2.1,0)=-3`, `ROUNDDOWN(-2.9,0)=-2`, `ROUND(125,-1)=130`.
Digits must be an exact integer-valued Decimal96. Fractional digits produce
InvalidArgument rather than truncation. Existing numeric Blank-as-zero semantics
apply to both operands; Boolean/String values produce TypeMismatch. Validate
arguments left to right and preserve dependency error provenance. Calls in inactive
IF/AND/OR branches remain undemanded under the existing evaluator.

Use a single exact coefficient division and remainder decision, with bounded u128
arithmetic. Never divide/multiply through rounded Decimal96 intermediates or binary
floats. Quantization left of the decimal point checks final representability before
multiplying the integer quantum. Overflow is the existing arithmetic Overflow fault,
not saturation or a displayed approximation. Normalize successful computational zero.

Classify arbitrarily large integer-valued digits before machine conversion. Positive
digits >=28 leave a Decimal96 value numerically unchanged. At digits <=-30, ROUND
and ROUNDDOWN return zero; ROUNDUP of nonzero values overflows. At -29 a ROUND of
large magnitudes can overflow (e.g. Decimal96 maximum rounds to 10^29). Zero stays
zero for every valid integer digits argument. There is no arbitrary precision cap,
iteration proportional to digits, unbounded integer dependency or resource-budget change.

Extend the existing typed-operation vocabulary with codes 15/16/17. The token
encoding, value kinds, native envelope, private messages and source-based clipboard
representation remain unchanged: retain WB9/private v7/CB4. Existing WB9 files remain
readable without migration. New functions require an engine that implements them;
unsupported opcodes/names continue to be rejected, without silent fallback. A feature
addition alone does not require another file/protocol/clipboard version cut.

## Delivery and validation

Reuse existing cross-sheet binding, dependency tracking, immutable captures,
Copy/Paste/Fill, native persistence and typed error paths. No frontend evaluator,
new editor control, formatting function, Integer/date type or history work.
The public Rust decimal helper accepts signed places and all seven existing modes;
these three spreadsheet functions select the corresponding standard modes.

Test independent integer rounding references across signs and modes, exact ties,
large/small coefficients, extreme digits, overflow, invalid operands/arity,
left-to-right failures and lazy branches. Cover cross-sheet invalidation, old
captures, copied/filled references, display independence, WB9 reopen and real .NET
sessions. Use native Windows x64/Linux x64/macOS ARM64 headless CI. Interaction,
transport and AOT metadata are unchanged; this slice does not assign another
attended desktop/AOT qualification round. Deliver one concise implementation handoff
and stop for parent review. ADR-0023 remains parked at G3.
