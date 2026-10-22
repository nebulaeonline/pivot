use pivot_core::{evaluation::*, model::*};
use pivot_types::{DecimalError, PivotDecimal};
fn d(s: &str) -> PivotDecimal {
    s.parse().unwrap()
}

#[test]
fn financial_model_and_results_survive_edits_and_context_drop() {
    let mut model = WorkingModel::new("Profit").unwrap();
    let revenue = model.add_input("Revenue", d("250000")).unwrap();
    model.add_input("Costs", d("85000")).unwrap();
    let net = model.add_computed("Net", "Revenue - Costs").unwrap();
    let margin = model.add_computed("Margin", "Net / Revenue").unwrap();
    let context = EvaluationContext::new(model.snapshot().unwrap());
    let old = context.evaluate(&[margin]).unwrap();
    assert_eq!(old.value(net), Some(&Ok(d("165000"))));
    assert_eq!(old.value(margin), Some(&Ok(d("0.66"))));
    model.set_input(revenue, d("300000")).unwrap();
    model.set_expression(net, "Revenue - Costs * 2").unwrap();
    let new = EvaluationContext::new(model.snapshot().unwrap()).evaluate_all();
    assert_eq!(new.value(net), Some(&Ok(d("130000"))));
    assert_ne!(old.revision(), new.revision());
    assert_eq!(
        context.evaluate(&[margin]).unwrap().value(margin),
        old.value(margin)
    );
    drop(context);
    drop(model);
    assert_eq!(old.value(net), Some(&Ok(d("165000"))));
    assert_eq!(old.snapshot().node_id("Net"), Some(net));
}

#[test]
fn arithmetic_failures_propagate_without_poisoning_other_outputs() {
    let mut m = WorkingModel::new("Errors").unwrap();
    let bad = m.add_computed("Bad", "1 / 0").unwrap();
    let dependent = m.add_computed("Dependent", "Bad + 1").unwrap();
    let downstream = m.add_computed("Downstream", "Dependent * 2").unwrap();
    let overflow = m
        .add_computed("Overflow", "79228162514264337593543950335 + 1")
        .unwrap();
    let good = m.add_computed("Good", "-(2 + 3) * +4 / 2 - 1").unwrap();
    let context = EvaluationContext::new(m.snapshot().unwrap());
    let all = context.evaluate_all();
    assert_eq!(
        all.value(bad),
        Some(&Err(EvaluationError::Arithmetic {
            node: bad,
            error: DecimalError::DivisionByZero
        }))
    );
    assert_eq!(
        all.value(overflow),
        Some(&Err(EvaluationError::Arithmetic {
            node: overflow,
            error: DecimalError::Overflow
        }))
    );
    assert_eq!(
        all.value(downstream),
        Some(&Err(EvaluationError::DependencyFailed {
            node: downstream,
            dependency: dependent,
            origin: bad,
            fault_id: None,
        }))
    );
    assert_eq!(all.value(good), Some(&Ok(d("-11"))));
    let selected = context.evaluate(&[good, good]).unwrap();
    assert_eq!(selected.outputs(), &[good]);
    assert_eq!(selected.evaluated_count(), 1);
    assert_eq!(selected.value(bad), None);
    assert_eq!(context.evaluate(&[]).unwrap().evaluated_count(), 0);
    let mut other = WorkingModel::new("Other").unwrap();
    let foreign = other.add_input("X", d("1")).unwrap();
    assert!(
        matches!(context.evaluate(&[foreign]), Err(EvaluationError::UnknownNode(id)) if id == foreign)
    );
}

#[test]
fn repeated_and_independent_evaluations_preserve_decimal_semantics() {
    let mut m = WorkingModel::new("Precision").unwrap();
    let exact = m.add_computed("Exact", "0.1 + 0.2").unwrap();
    let third = m.add_computed("Third", "1 / 3").unwrap();
    let tiny = m
        .add_computed("Tiny", "0.0000000000000000000000000001 * 0.1")
        .unwrap();
    let snapshot = m.snapshot().unwrap();
    let first = EvaluationContext::new(snapshot.clone()).evaluate_all();
    let worker = std::thread::spawn(move || EvaluationContext::new(snapshot).evaluate_all());
    let second = worker.join().unwrap();
    assert_eq!(first.value(exact), Some(&Ok(d("0.3"))));
    assert_eq!(
        first.value(third),
        Some(&Ok(d("0.3333333333333333333333333333")))
    );
    assert_eq!(first.value(tiny), Some(&Ok(PivotDecimal::ZERO)));
    for id in first.outputs() {
        assert_eq!(first.value(*id), second.value(*id));
    }
    assert_eq!(first.revision(), second.revision());
}

#[test]
fn shared_dependencies_and_long_chains_are_evaluated_once_iteratively() {
    let mut m = WorkingModel::new("Chain").unwrap();
    let input = m.add_input("Input", d("1")).unwrap();
    let mut previous = "Input".to_string();
    let mut last = input;
    for i in 0..1000 {
        let name = format!("N{i}");
        last = m.add_computed(&name, format!("{previous} + 1")).unwrap();
        previous = name;
    }
    let output = m
        .add_computed("Output", format!("{previous} + {previous}"))
        .unwrap();
    let result = EvaluationContext::new(m.snapshot().unwrap())
        .evaluate(&[output])
        .unwrap();
    assert_eq!(result.value(last), Some(&Ok(d("1001"))));
    assert_eq!(result.value(output), Some(&Ok(d("2002"))));
    assert_eq!(result.evaluated_count(), 1002);
}
