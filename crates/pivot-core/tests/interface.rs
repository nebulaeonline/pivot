use pivot_core::{evaluation::EvaluationContext, interface::*, model::*};
use pivot_types::{PivotDecimal, PivotValue, ValueType};
fn d(s: &str) -> PivotDecimal {
    s.parse().unwrap()
}
fn v(s: &str) -> PivotValue {
    d(s).into()
}
fn setup() -> (WorkingModel, NodeId, NodeId, PortId) {
    let mut m = WorkingModel::new("Profit").unwrap();
    let sales = m.add_input("GrossSales", d("1")).unwrap();
    let cost = m.add_input("COGS", d("999")).unwrap();
    let net = m.add_computed("Net", "GrossSales - COGS").unwrap();
    let id = m
        .add_input_port("Revenue", sales, InputRequirement::Required)
        .unwrap();
    m.add_input_port(
        "FoodCost",
        cost,
        InputRequirement::Optional { default: v("25") },
    )
    .unwrap();
    m.add_output_port("Profit", net).unwrap();
    m.add_output_port("Sales", sales).unwrap();
    (m, sales, net, id)
}
#[test]
fn public_names_defaults_required_inputs_and_isolation() {
    let (mut m, sales, _, port) = setup();
    let snapshot = m.snapshot().unwrap();
    assert_eq!(
        snapshot.interface().inputs().next().unwrap().value_type(),
        ValueType::Decimal
    );
    let revision = snapshot.revision();
    let context = EvaluationContext::new(snapshot);
    let request = NamedInvocationRequest {
        inputs: vec![("Revenue".into(), v("100"))],
        outputs: vec!["Profit".into()],
    };
    let a = context.invoke_named(&request).unwrap();
    assert_eq!(a.outputs().get("Profit"), Some(&Ok(v("75"))));
    assert_eq!(a.inputs().get("FoodCost"), Some(&v("25")));
    assert_eq!(a.outputs().len(), 1);
    assert_eq!(a.revision(), revision);
    m.rename(sales, "NewSales").unwrap();
    m.rename_port(port, "NewRevenue").unwrap();
    assert_eq!(
        m.interface()
            .inputs()
            .find(|p| p.name() == "NewRevenue")
            .unwrap()
            .id(),
        port
    );
    assert!(m.snapshot().is_ok());
    drop(m);
    std::thread::scope(|scope| {
        let other = scope.spawn(|| {
            context
                .invoke_named(&NamedInvocationRequest {
                    inputs: vec![("Revenue".into(), v("200")), ("FoodCost".into(), v("50"))],
                    outputs: vec!["Profit".into(), "Sales".into()],
                })
                .unwrap()
        });
        assert_eq!(
            context.invoke_named(&request).unwrap().outputs(),
            a.outputs()
        );
        let b = other.join().unwrap();
        assert_eq!(b.outputs().get("Profit"), Some(&Ok(v("150"))));
        assert_eq!(b.outputs().get("Sales"), Some(&Ok(v("200"))));
        assert_eq!(b.revision(), revision);
    });
}
#[test]
fn public_validation_and_invalid_declarations() {
    let (mut m, sales, net, _) = setup();
    assert!(matches!(
        m.add_input_port("Bad", net, InputRequirement::Required),
        Err(ModelError::NotInput(_))
    ));
    assert!(matches!(
        m.add_output_port("Revenue", net),
        Err(ModelError::Interface(InterfaceError::DuplicateName(_)))
    ));
    assert!(matches!(
        m.add_input_port("Again", sales, InputRequirement::Required),
        Err(ModelError::Interface(InterfaceError::DuplicateInputTarget(
            _
        )))
    ));
    let context = EvaluationContext::new(m.snapshot().unwrap());
    for (inputs, outputs, expected) in [
        (
            vec![],
            vec![],
            InterfaceError::MissingRequired("Revenue".into()),
        ),
        (
            vec![("GrossSales".into(), v("1"))],
            vec![],
            InterfaceError::UnknownInput("GrossSales".into()),
        ),
        (
            vec![("Revenue".into(), v("1")), ("Revenue".into(), v("2"))],
            vec![],
            InterfaceError::DuplicateInput("Revenue".into()),
        ),
        (
            vec![("Revenue".into(), v("1"))],
            vec!["Net".into()],
            InterfaceError::UnknownOutput("Net".into()),
        ),
        (
            vec![("Revenue".into(), v("1"))],
            vec!["Profit".into(), "Profit".into()],
            InterfaceError::DuplicateOutput("Profit".into()),
        ),
    ] {
        assert_eq!(
            context
                .invoke_named(&NamedInvocationRequest { inputs, outputs })
                .unwrap_err(),
            expected
        );
    }
    m.remove(net).unwrap();
    assert!(matches!(
        m.snapshot(),
        Err(SnapshotError::Interface(InterfaceError::InvalidBinding(_)))
    ));
}
#[test]
fn bound_references_survive_rename_and_never_retarget_recreated_names() {
    let mut m = WorkingModel::new("Identity").unwrap();
    let x = m.add_input("X", d("10")).unwrap();
    let result = m.add_computed("Result", "X + 1").unwrap();
    m.rename(x, "Renamed").unwrap();
    let replacement = m.add_input("X", d("999")).unwrap();
    let snapshot = m.snapshot().unwrap();
    assert_eq!(snapshot.node(result).unwrap().dependencies(), &[x]);
    assert_eq!(
        EvaluationContext::new(snapshot.clone())
            .evaluate(&[result])
            .unwrap()
            .value(result),
        Some(&Ok(d("11")))
    );
    m.remove(x).unwrap();
    assert!(
        matches!(m.snapshot(), Err(SnapshotError::MissingReference { target, .. }) if target == x)
    );
    m.set_expression(result, "X + 1").unwrap();
    assert_eq!(
        m.snapshot().unwrap().node(result).unwrap().dependencies(),
        &[replacement]
    );
    assert_eq!(snapshot.node(result).unwrap().dependencies(), &[x]);
}
