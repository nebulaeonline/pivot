use pivot_core::model::*;
use pivot_types::PivotDecimal;
fn d(s: &str) -> PivotDecimal {
    s.parse().unwrap()
}

#[test]
fn financial_model_binds_and_snapshots_inputs_and_definitions() {
    let mut model = WorkingModel::new("Profit").unwrap();
    let revenue = model.add_input("Revenue", d("250000")).unwrap();
    let costs = model.add_input("Costs", d("85000")).unwrap();
    let net = model.add_computed("Net", "Revenue - Costs").unwrap();
    let margin = model.add_computed("Margin", "Net / Revenue").unwrap();
    let first = model.snapshot().unwrap();
    assert_eq!(first.name(), "Profit");
    assert_eq!(first.node(net).unwrap().dependencies(), &[revenue, costs]);
    assert_eq!(first.node(margin).unwrap().dependencies(), &[revenue, net]);
    assert_eq!(first.node(revenue).unwrap().dependents(), &[net, margin]);
    assert_eq!(first.dependency_order(), &[revenue, costs, net, margin]);
    let SnapshotDefinition::Computed { tokens, .. } = first.node(net).unwrap().definition() else {
        panic!()
    };
    assert_eq!(
        tokens,
        &[
            BoundToken::Node(revenue),
            BoundToken::Node(costs),
            BoundToken::Binary(pivot_core::expression::BinaryOp::Subtract)
        ]
    );
    model.set_input(revenue, d("300000")).unwrap();
    model.set_expression(net, "Revenue - Costs * 2").unwrap();
    let second = model.snapshot().unwrap();
    assert_ne!(first.revision(), second.revision());
    assert_eq!(second.node_id("Net"), Some(net));
    assert!(
        matches!(first.node(revenue).unwrap().definition(), SnapshotDefinition::Input(v) if *v == d("250000"))
    );
    assert!(
        matches!(second.node(revenue).unwrap().definition(), SnapshotDefinition::Input(v) if *v == d("300000"))
    );
    assert!(
        matches!(first.node(net).unwrap().definition(), SnapshotDefinition::Computed { source, .. } if source == "Revenue - Costs")
    );
    model.remove(costs).unwrap();
    assert!(matches!(
        model.snapshot(),
        Err(SnapshotError::MissingReference { .. })
    ));
    assert!(first.node(costs).is_some());
    drop(model);
    assert_eq!(first.clone().node_id("Revenue"), Some(revenue));
}

#[test]
fn incomplete_edits_forward_references_and_cycles() {
    let mut model = WorkingModel::new("Editable").unwrap();
    let a = model.add_computed("A", "B + B").unwrap();
    assert!(
        matches!(model.snapshot(), Err(SnapshotError::UnknownName { node, name }) if node == a && name == "B")
    );
    let b = model.add_input("B", d("1")).unwrap();
    // Resolve text at the command boundary; snapshot capture never rebinds.
    model.set_expression(a, "B + B").unwrap();
    let valid = model.snapshot().unwrap();
    assert_eq!(valid.node(a).unwrap().dependencies(), &[b]);
    assert_eq!(valid.node(b).unwrap().dependents(), &[a]);
    assert_eq!(valid.dependency_order(), &[b, a]);
    model.set_expression(b, "A").unwrap();
    assert!(
        matches!(model.snapshot(), Err(SnapshotError::Unorderable { nodes }) if nodes == vec![a, b])
    );
    model.set_expression(a, "A").unwrap();
    assert!(matches!(
        model.snapshot(),
        Err(SnapshotError::Unorderable { .. })
    ));
    model.set_expression(a, "1 +").unwrap();
    assert!(matches!(model.snapshot(), Err(SnapshotError::Parse { node, .. }) if node == a));
    model.set_expression(a, "1").unwrap();
    assert!(model.snapshot().is_ok());
    assert_eq!(valid.node(a).unwrap().dependencies(), &[b]);
}

#[test]
fn identities_names_and_rejected_mutations() {
    let mut m = WorkingModel::new("one").unwrap();
    let mut other = WorkingModel::new("two").unwrap();
    let id = m.add_input("X", d("1")).unwrap();
    let foreign = other.add_input("X", d("1")).unwrap();
    assert_ne!(id, foreign);
    let rev = m.revision();
    assert!(matches!(
        m.set_input(foreign, d("2")),
        Err(ModelError::UnknownNode(_))
    ));
    assert!(matches!(
        m.add_input("X", d("2")),
        Err(ModelError::DuplicateName(_))
    ));
    for name in ["", "1bad", "bad name", "é"] {
        assert!(m.add_input(name, d("0")).is_err());
    }
    assert_eq!(rev, m.revision());
    m.rename(id, "Y").unwrap();
    assert_eq!(m.node_id("Y"), Some(id));
    assert_eq!(m.node_id("X"), None);
    m.remove(id).unwrap();
    let replacement = m.add_input("Y", d("1")).unwrap();
    assert_ne!(id, replacement);
    assert!(m.snapshot().unwrap().node(id).is_none());
    assert!(
        WorkingModel::new("empty")
            .unwrap()
            .snapshot()
            .unwrap()
            .dependency_order()
            .is_empty()
    );
}

#[test]
fn snapshot_can_be_read_elsewhere_while_working_model_changes() {
    let mut model = WorkingModel::new("Concurrent use").unwrap();
    let x = model.add_input("X", d("1")).unwrap();
    let snapshot = model.snapshot().unwrap();
    let worker = std::thread::spawn(move || {
        assert!(
            matches!(snapshot.node(x).unwrap().definition(), SnapshotDefinition::Input(v) if *v == d("1"))
        );
        snapshot
    });
    model.set_input(x, d("2")).unwrap();
    let old = worker.join().unwrap();
    assert_ne!(old.revision(), model.revision());
}
