# Pivot Handoff — Snapshot evaluator

Date: 2026-10-18
Branch / commit: uncommitted
Status: Complete

## Objective

Implement the owner-authorized next slice from the architecture review: deterministic
single-threaded evaluation of immutable snapshots with calculation-local results.

## Implemented

- `crates/pivot-core/src/evaluation.rs`: `EvaluationContext`, `CalculationResult`,
  and typed `EvaluationError`.
- Decimal literal, bound identifier, unary, and binary expression execution.
- Selected-output dependency closure and deterministic dependency-order evaluation.
- Immutable results that retain the originating snapshot and revision.
- Four integration tests in `crates/pivot-core/tests/evaluation.rs`.
- Execution/error documentation and updated project status and README.

## Interface / semantics

`EvaluationContext::new(snapshot)` creates a context. `evaluate(&outputs)` returns
a request result; `evaluate_all()` computes the whole validated snapshot. Each call
has fresh local state. Arithmetic failures stay in per-node results; dependents
identify the immediate failed dependency and root failing node. Other outputs run.
Unknown selected IDs reject the request. Existing decimal behavior is unchanged.

## Tests actually run

- `cargo test --workspace` — PASS: fifteen tests total.
- `cargo clippy --workspace --all-targets -- -D warnings` — PASS.
- `cargo fmt --all -- --check` — PASS.

New coverage includes Revenue/Costs/Net/Margin, changed inputs and formulas,
retained old results, repeated evaluation, independent contexts on separate threads,
all arithmetic operators, decimal precision and underflow, propagated division by
zero, overflow, independent valid outputs, selected/duplicate/empty outputs, foreign
IDs, shared dependencies, and a 1,000-node chain.

The Rust/.NET parity suite was not rerun: numeric code and policies did not change.
No .NET/AOT build was required for this Rust-only implementation.

## Decisions made or proposed

- **Accepted:** ADR-0002 snapshot isolation and calculation-local state applied.
- **Implementation choices:** fresh result maps per call, iterative expression
  execution, selected-output dependency closure, and first-error expression behavior.
- **Reserved:** stable identity binding through renames, partial model validation,
  distinct published versions, and future calculation policy capture.

## Open issues / risks

Snapshot capture still requires the entire model to be structurally valid. Selected
evaluation isolates arithmetic failures but does not relax capture validation. No
incremental cache, dirty propagation, input overrides, model output declarations,
cancellation, IPC, persistence, or GUI integration was added. Results retain complete
snapshots; storage optimization remains private and deferred.

## Questions for ChatGPT / owner

No blocking questions. Partial validation and stable reference editing need separate
design before the current provisional working-model binding rules are extended.

## Next recommended slice

Review execution and error semantics, then choose between isolated invocation input
bindings and incremental recalculation. Neither is included in this slice.
