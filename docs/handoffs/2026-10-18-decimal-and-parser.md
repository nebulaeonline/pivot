# Pivot Handoff — Decimal foundation and expression parser

Date: 2026-10-18
Branch / commit: uncommitted
Status: Complete

## Objective

Record the agreed architecture in a numbered ADR, then implement the base
numeric type and basic parser/AST. Defer the rest of M1.

## Implemented

- `docs/adr/`: ADR naming/lifecycle convention and accepted ADR-0001.
- `crates/pivot-types/src/lib.rs`: private decimal backend, exact literal parsing,
  integer construction, display, comparison, checked arithmetic and typed errors.
- `crates/pivot-core/src/expression.rs`: arithmetic AST and bounded parser.
- `docs/STATUS.md` and `docs/architecture/expressions-and-decimals.md`: scope,
  grammar, provisional arithmetic behavior, and deferred work.

## Interface / semantics

Expressions support decimal literals, names, unary signs, binary `+ - * /`, and
parentheses. Operators use normal arithmetic precedence and left association.
Errors report byte offsets; complete consumption is required. Inputs are bounded
to 4096 bytes and 64 recursive parser levels.

Decimals never pass through floats. Backend errors do not leak through the public
API. Dependency `rust_decimal` 1.43.0 uses only its `std` feature. Only arrayvec,
num-traits, and the autocfg build helper are active transitive dependencies;
Cargo.lock also records optional dependency resolutions. No .NET dependency changed.

## Tests actually run

- `cargo test --workspace` — PASS: six tests covering decimals, financial arithmetic,
  overflow, division by zero, scale, literal rejection, provisional division and
  underflow behavior, precedence, association, malformed input, and resource limits.
- `cargo clippy --workspace --all-targets -- -D warnings` — PASS.
- `cargo fmt --all -- --check` — PASS.
- .NET reference comparisons and Native AOT publish — NOT RUN in this slice.

Sandboxed dependency access initially failed; tests and lint passed after an
approved dependency download.

## Decisions made or proposed

- **Accepted:** previously settled graph-first architecture recorded in ADR-0001.
- **Implemented initial scope:** small handwritten precedence parser; ASCII names;
  no parser dependency or formula execution.
- **Proposed:** retain backend rounding, precision reduction, scale, and underflow
  behavior during exploration. This does not ratify final numeric policy.

## Open issues / risks

Checked decimal arithmetic can round or underflow to zero. .NET semantic parity
is unverified. The initial syntax is deliberately limited; the AST has no source
spans beyond parser error offsets. Names are not resolved and expressions are not
evaluated. No graph, model invocation, or additional value types are implemented.

## Questions for ChatGPT / owner

No decision blocks the requested slice. Before numeric semantics are frozen,
review rounding, scale, and underflow with .NET reference vectors; recommended
next numeric step is to compare those cases before introducing a custom policy.

## Next recommended slice

Review numeric policy and reference vectors. Resume graph/model work only when
requested; it was explicitly deferred for this slice.
