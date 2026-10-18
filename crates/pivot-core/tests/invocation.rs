use pivot_core::{evaluation::*, model::*};
use pivot_types::{DecimalError, PivotDecimal};
fn d(s: &str) -> PivotDecimal {
    s.parse().unwrap()
}
fn setup() -> (WorkingModel, NodeId, NodeId, NodeId, NodeId) {
    let mut m = WorkingModel::new("Profit").unwrap();
    let revenue = m.add_input("Revenue", d("250000")).unwrap();
    let costs = m.add_input("Costs", d("85000")).unwrap();
    let net = m.add_computed("Net", "Revenue - Costs").unwrap();
    let margin = m.add_computed("Margin", "Net / Revenue").unwrap();
    (m, revenue, costs, net, margin)
}

#[test]
fn defaults_partial_and_complete_bindings_preserve_snapshot_and_revision() {
    let (mut m, revenue, costs, net, margin) = setup();
    let revision = m.revision();
    let context = EvaluationContext::new(m.snapshot().unwrap());
    let defaults = context
        .invoke(&InvocationRequest {
            inputs: vec![],
            outputs: vec![margin],
        })
        .unwrap();
    assert_eq!(defaults.value(margin), Some(&Ok(d("0.66"))));
    let partial = context
        .invoke(&InvocationRequest {
            inputs: vec![(costs, d("100000"))],
            outputs: vec![net, margin],
        })
        .unwrap();
    assert_eq!(partial.value(net), Some(&Ok(d("150000"))));
    assert_eq!(partial.value(margin), Some(&Ok(d("0.6"))));
    let complete = context
        .invoke(&InvocationRequest {
            inputs: vec![(revenue, d("500")), (costs, d("200"))],
            outputs: vec![margin],
        })
        .unwrap();
    assert_eq!(complete.value(margin), Some(&Ok(d("0.6"))));
    assert_eq!(complete.value(revenue), Some(&Ok(d("500"))));
    assert_eq!(complete.input_overrides().get(&costs), Some(&d("200")));
    for result in [&defaults, &partial, &complete] {
        assert_eq!(result.revision(), revision);
    }
    assert_eq!(m.revision(), revision);
    assert!(
        matches!(complete.snapshot().node(revenue).unwrap().definition(), SnapshotDefinition::Input(v) if *v == d("250000"))
    );
    assert_eq!(
        context.evaluate(&[margin]).unwrap().value(margin),
        defaults.value(margin)
    );
    m.set_input(revenue, d("1")).unwrap();
    assert_eq!(complete.revision(), revision);
}

#[test]
fn invalid_overrides_are_typed_and_checked_even_outside_output_closure() {
    let (m, revenue, _, net, margin) = setup();
    let context = EvaluationContext::new(m.snapshot().unwrap());
    let mut other = WorkingModel::new("Other").unwrap();
    let foreign = other.add_input("X", d("1")).unwrap();
    for (inputs, error) in [
        (
            vec![(foreign, d("1"))],
            InvocationError::UnknownInput(foreign),
        ),
        (vec![(net, d("1"))], InvocationError::NotInput(net)),
        (
            vec![(revenue, d("1")), (revenue, d("1"))],
            InvocationError::DuplicateInput(revenue),
        ),
    ] {
        assert_eq!(
            context
                .invoke(&InvocationRequest {
                    inputs,
                    outputs: vec![]
                })
                .unwrap_err(),
            error
        );
    }
    assert_eq!(
        context
            .invoke(&InvocationRequest {
                inputs: vec![],
                outputs: vec![foreign]
            })
            .unwrap_err(),
        InvocationError::Evaluation(EvaluationError::UnknownNode(foreign))
    );
    assert_eq!(
        context.evaluate(&[margin]).unwrap().value(margin),
        Some(&Ok(d("0.66")))
    );
    let empty = context
        .invoke(&InvocationRequest {
            inputs: vec![(revenue, d("1"))],
            outputs: vec![],
        })
        .unwrap();
    assert_eq!(empty.evaluated_count(), 0);
    assert_eq!(empty.input_overrides().get(&revenue), Some(&d("1")));
}

#[test]
fn concurrent_invocations_after_model_deletion_are_independent() {
    let (m, revenue, costs, net, margin) = setup();
    let context = EvaluationContext::new(m.snapshot().unwrap());
    let revision = m.revision();
    drop(m);
    std::thread::scope(|scope| {
        let first = scope.spawn(|| {
            context
                .invoke(&InvocationRequest {
                    inputs: vec![(revenue, d("100")), (costs, d("25"))],
                    outputs: vec![net, margin],
                })
                .unwrap()
        });
        let second = scope.spawn(|| {
            context
                .invoke(&InvocationRequest {
                    inputs: vec![(revenue, d("200")), (costs, d("100"))],
                    outputs: vec![net, margin],
                })
                .unwrap()
        });
        let a = first.join().unwrap();
        let b = second.join().unwrap();
        assert_eq!(a.value(net), Some(&Ok(d("75"))));
        assert_eq!(a.value(margin), Some(&Ok(d("0.75"))));
        assert_eq!(b.value(net), Some(&Ok(d("100"))));
        assert_eq!(b.value(margin), Some(&Ok(d("0.5"))));
        assert_eq!(a.revision(), revision);
        assert_eq!(b.revision(), revision);
    });
    assert_eq!(
        context.evaluate(&[net]).unwrap().value(net),
        Some(&Ok(d("165000")))
    );
}

#[test]
fn arithmetic_errors_and_selected_outputs_stay_invocation_local() {
    let (m, revenue, _, net, margin) = setup();
    let context = EvaluationContext::new(m.snapshot().unwrap());
    let bad = context
        .invoke(&InvocationRequest {
            inputs: vec![(revenue, d("0"))],
            outputs: vec![net, margin],
        })
        .unwrap();
    assert_eq!(bad.value(net), Some(&Ok(d("-85000"))));
    assert_eq!(
        bad.value(margin),
        Some(&Err(EvaluationError::Arithmetic {
            node: margin,
            error: DecimalError::DivisionByZero
        }))
    );
    let selected = context
        .invoke(&InvocationRequest {
            inputs: vec![(revenue, d("0"))],
            outputs: vec![net, net],
        })
        .unwrap();
    assert_eq!(selected.outputs(), &[net]);
    assert_eq!(selected.value(margin), None);
    assert_eq!(
        context
            .invoke(&InvocationRequest {
                inputs: vec![],
                outputs: vec![margin]
            })
            .unwrap()
            .value(margin),
        Some(&Ok(d("0.66")))
    );
}
