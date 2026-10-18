# Pivot Handoff — Decimal compatibility and rounding

Date: 2026-10-18
Branch / commit: uncommitted
Status: Complete

## Objective

Begin aligning Rust/C# decimal semantics, expose all rounding modes, and make
ordinary nearest rounding with midpoint ties away from zero the default.

## Implemented

- Rust `PivotDecimal`: seven rounding modes, default rounding, scale validation,
  normalization, and canonical arithmetic outputs.
- .NET `Pivot.Client`: `DecimalRounding`, `RoundingMode`, and `DecimalArithmetic`.
- `tests/decimal-parity/`: runnable cross-language harness, independent rounding
  expectations, seeded arithmetic vectors, native representation report, and results.
- Updated status and numeric architecture notes. Graph work remains deferred.

## Interface / semantics

Nearest ties-away-from-zero gives `2.5 -> 3` and `-2.5 -> -3`. Other modes are
nearest ties-to-even, nearest ties-toward-zero, truncation, directed away-from-zero,
floor, and ceiling. Decimal places are restricted to 0–28.

Arithmetic results remove trailing fractional zeros and canonicalize zero.
Literal scale is retained. Explicit rounding does not pad scale. Native arithmetic
precision reduction is independent of the explicit rounding default. Backend
underflow to zero and rounded division remain possible without errors.

## Tests actually run

- `python tests/decimal-parity/verify.py` — PASS: 10,099 vectors, zero value/error
  mismatches and zero Pivot scale differences.
- `cargo test --workspace` — PASS: seven unit tests.
- `cargo clippy --workspace --all-targets -- -D warnings` — PASS.
- `cargo fmt --all -- --check` — PASS.
- C# harness includes validation for negative places and invalid enum values.

The initial 2,099-case characterization had zero value/error differences and
72 native scale differences. The expanded native representation comparison is
saved separately, so normalization does not conceal native backend differences.
The .NET harness is built and executed by the comparison script.

## Decisions made or proposed

- **Accepted:** owner-requested ordinary rounding default, interpreted symmetrically
  as nearest/ties-away-from-zero for positive and negative values.
- **Implementation choice:** canonical arithmetic output scale on both sides rather
  than reproducing every native backend scale-retention detail.
- **Still provisional:** internal precision reduction and underflow policy. Passing
  vectors do not establish universal arithmetic or representation compatibility.

## Open issues / risks

C# callers must use the Pivot helpers for canonical output; native decimal operators
retain .NET behavior. Normalization uses invariant decimal text in C# and is not
performance tuned. Sign of zero is not preserved as a contract. No float conversion
is used. No wire encoding, strict C# parsing API, AOT check, or graph work was added.

## Questions for ChatGPT / owner

No blocking question. Decide separately whether future business rules require
errors on underflow or precision loss rather than the current backend behavior.

## Next recommended slice

Review the compatibility results before graph execution. Expand characterization
when concrete financial formulas expose additional rounding requirements.
