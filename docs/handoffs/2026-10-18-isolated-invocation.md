# Pivot Handoff — Isolated model invocation

Date: 2026-10-18
Branch / commit: uncommitted
Status: Complete

## Objective

Implement the owner-authorized isolated invocation slice recommended in the
snapshot evaluator review, without advancing other milestones.

## Implemented

- `pivot-core::evaluation::InvocationRequest`: temporary decimal bindings and
  selected output IDs.
- `EvaluationContext::invoke`: binding validation and reuse of the existing
  execution path, without parsing or rebinding expressions.
- `InvocationError`: unknown input IDs, non-input nodes, duplicate bindings,
  and wrapped evaluation-request errors.
- `CalculationResult::input_overrides`: retained accepted invocation bindings.
- Four integration tests in `crates/pivot-core/tests/invocation.rs`.
- Updated execution documentation, status, and README.

## Interface / semantics

Bindings use stable NodeIds and apply only to input nodes. Unspecified inputs use
captured defaults. All supplied bindings are validated even if unused by selected
outputs. Duplicate IDs are rejected even if their values are equal. First-error
validation follows binding order: identity, input kind, then duplication.

Invocation state is local to the call. Results retain the unchanged snapshot and
revision plus accepted overrides. Snapshot defaults and evaluated input values
are distinguishable. A working model need not remain alive. No global mutable
inputs or caches were added. Decimal arithmetic semantics are unchanged.

## Tests actually run

- `cargo test --workspace` — PASS: nineteen tests total.
- `cargo clippy --workspace --all-targets -- -D warnings` — PASS.
- `cargo fmt --all -- --check` — PASS.

New tests cover default, partial, and complete overrides; preserved defaults and
revisions; model edits/deletion; shared-context concurrent invocations; isolated
division-by-zero failures; selected and duplicate outputs; empty selections;
foreign IDs; computed-node overrides; and duplicate bindings. Rejected bindings
do not affect later calculations.

Numeric code did not change; the cross-language decimal suite and .NET builds were
not rerun for this Rust-only slice.

## Decisions made or proposed

- **Accepted:** snapshot immutability and calculation-local input bindings applied.
- **Implementation choices:** sequence-based request, request validation before
  execution, and retained override metadata in results. No new architectural
  decision requires acceptance.

## Open issues / risks

Values are decimal-only. Explicit public model input/output ports, required-input
declarations, isolated policy selection, persistence, IPC, serialization, GUI, and
incremental caching remain outside this slice. Snapshot validation is still global.
Result lifetime retains the full immutable snapshot. Process-local IDs are not a
wire or persistence contract.

## Questions for ChatGPT / owner

No blocking questions. Named interface design should determine required/default
input semantics and which internal nodes are exposed as public outputs.

## Next recommended slice

Design explicit named model interfaces before defining an external invocation
protocol. Do not infer protocol or transport decisions from this in-process API.
