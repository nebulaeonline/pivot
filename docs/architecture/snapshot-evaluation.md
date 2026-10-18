# Snapshot evaluation

`pivot_core::evaluation::EvaluationContext` owns an immutable model snapshot.
`evaluate(&[NodeId])` computes selected outputs and their transitive dependencies;
`evaluate_all()` computes all nodes. An empty selection computes nothing, and
duplicate outputs are deduplicated. Outputs are returned in stable node-ID order.

Execution is deterministic and single-threaded. Nodes run in the snapshot's
dependency order, once per call. Bound postfix expressions use decimal literals,
node-ID references, unary signs, and binary arithmetic. Stack evaluation and
dependency traversal are iterative. No expression names are resolved at runtime.

Each call creates a private result map and temporary expression stack. Contexts
hold no mutable cross-call cache. Repeated evaluations and separate contexts over
one snapshot cannot change each other's values. No incremental recalculation or
parallel scheduler is introduced.

## Isolated invocation

`invoke(&InvocationRequest { inputs, outputs })` evaluates the same bound snapshot
with temporary decimal overrides keyed by `NodeId`. Unspecified inputs use captured
defaults. The request uses a sequence of bindings so duplicates are detectable.
Only input nodes may be overridden; foreign, removed, or otherwise absent IDs are
rejected. Presence is checked against the captured snapshot, not the current model.

All bindings are validated, including bindings outside the selected dependency
closure. Validation follows binding order, checking identity, input kind, then
duplicates, and returns the first error. An empty output selection still validates
bindings. Unknown output IDs are wrapped as invocation evaluation-request errors.
Arithmetic failures remain node-level result data, just as for ordinary evaluation.

Invocation reuses the snapshot's bound expressions and dependency order. It does
not parse, rebind, edit the model, or advance its revision. Results retain accepted
overrides through `input_overrides()`, including unused bindings. `snapshot()` still
describes captured defaults; `value(id)` describes the evaluated invocation value.
These two views deliberately differ when an input is overridden.

Model editing changes working state; snapshot capture freezes it; evaluation uses
captured values; invocation temporarily substitutes input values for one execution.
The working model need not remain alive for evaluation or invocation.

## Results and errors

`CalculationResult` retains the originating snapshot and its model-scoped revision.
It remains usable after the working model changes or is dropped. `outputs()` lists
requested IDs; `value(id)` also exposes computed dependency results. `None` means
the ID was not evaluated. `evaluated_count()` includes inputs and dependencies.

Unknown output IDs reject a request before evaluation. Arithmetic failures are
per-node result data, including decimal overflow and division by zero. Failed
references produce `DependencyFailed` with the immediate dependency and original
failing node. The root error remains accessible in the same result. Evaluation
continues for independent nodes. Expression errors stop that expression at the
first failing token; error order is deterministic, not an aggregate diagnostic.
`InvalidExpression` defensively represents a violated bound-expression invariant.

All arithmetic uses the existing `PivotDecimal` operations: canonical arithmetic
scales, rounded finite-precision division, and possible underflow to zero. Unary
plus preserves its operand; unary minus uses checked negation. The explicit
rounding API is unchanged, and no extra rounding is applied during evaluation.

## Example

```rust
use pivot_core::{evaluation::EvaluationContext, model::WorkingModel};

let mut model = WorkingModel::new("Profit")?;
model.add_input("Revenue", "250000".parse()?)?;
model.add_input("Costs", "85000".parse()?)?;
let net = model.add_computed("Net", "Revenue - Costs")?;
let margin = model.add_computed("Margin", "Net / Revenue")?;
let result = EvaluationContext::new(model.snapshot()?).evaluate(&[margin])?;
// result.value(net) is Some(Ok(165000)); margin is Some(Ok(0.66)).
# Ok::<(), Box<dyn std::error::Error>>(())
```

## Limits and reserved decisions

Snapshot validation still rejects invalid syntax, missing bindings, and cycles
anywhere in a model. Selected-output evaluation does not yet allow capture of
partially invalid models. Distinguish this limit from independent arithmetic
failures during execution; partial validation remains a future option.

No configurable calculation policy, cancellation,
global resource budgets, persistence, or published version identity is added.
Results hold snapshots in memory and may be expensive to retain in large numbers.
Stable-ID references now survive working-model renames; see ADR-0003.
