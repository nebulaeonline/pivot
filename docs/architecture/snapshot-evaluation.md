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

No per-invocation input overrides, configurable calculation policy, cancellation,
global resource budgets, persistence, or published version identity is added.
Results hold snapshots in memory and may be expensive to retain in large numbers.
Stable-ID references across working-model renames remain an architectural
requirement for future editing work; snapshot-time name rebinding is provisional.
