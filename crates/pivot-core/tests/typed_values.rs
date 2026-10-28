use pivot_core::{
    evaluation::{EvaluationError, InvocationRequest},
    model::{
        WorkingModel,
        transfer::{self, ResolvedToken as T},
    },
    typed::Op,
};
#[test]
fn ordinary_canonical_decimal_adapter_uses_demand_and_scenario_inputs() {
    let mut model = WorkingModel::new("typed demand").unwrap();
    let input = model.add_input("Input", 0.into()).unwrap();
    let fail = model.add_computed("Unused", "1/0").unwrap();
    let output = model.add_computed("Output", "0").unwrap();
    transfer::set_resolved_expression(
        &mut model,
        output,
        "typed",
        &[
            T::Node(input),
            T::Decimal(0.into()),
            T::Typed(Op::Greater, 2),
            T::Node(fail),
            T::Decimal(7.into()),
            T::Typed(Op::If, 3),
        ],
    )
    .unwrap();
    let cap = model.capture_local().unwrap();
    let result = cap.evaluate(&[output]).unwrap();
    assert_eq!(result.values()[&output], Ok(7.into()));
    assert!(
        !result.values().contains_key(&fail),
        "unused dependency must not be evaluated through the canonical Decimal API either"
    );
    let scenario = cap
        .invoke(&InvocationRequest {
            inputs: vec![(input, 1.into())],
            outputs: vec![output],
        })
        .unwrap();
    assert!(matches!(
        scenario.values()[&output],
        Err(EvaluationError::DependencyFailed { .. })
    ));
    assert_eq!(
        cap.evaluate(&[output]).unwrap().values()[&output],
        Ok(7.into()),
        "scenario does not mutate the capture"
    );
}
