#![cfg(feature = "scaling-experiment")]
use pivot_core::{
    evaluation::EvaluationContext,
    model::{SnapshotError, WorkingModel, structural::*, transfer::ResolvedToken},
};
use pivot_types::PivotDecimal;
fn setup() -> (
    WorkingModel,
    Graph,
    pivot_core::model::NodeId,
    pivot_core::model::NodeId,
    pivot_core::model::NodeId,
) {
    let mut m = WorkingModel::new("Structural").unwrap();
    let a = m.add_input("a", 2.into()).unwrap();
    let b = m.add_input("b", 3.into()).unwrap();
    let c = m.add_computed("c", "a+1").unwrap();
    let g = Graph::from_model(&m, Limits::default()).unwrap();
    (m, g, a, b, c)
}
fn value(g: &Capture, id: pivot_core::model::NodeId) -> PivotDecimal {
    g.evaluate(&[id]).unwrap()[&id].clone().unwrap()
}
#[test]
fn localized_edges_and_old_topology_match_baseline() {
    let (mut m, mut g, a, b, c) = setup();
    let old = g.capture().unwrap();
    let original = g.clone();
    g.apply(
        g.version(),
        vec![Edit::Formula(c, "b*2".into()), Edit::Input(a, 17.into())],
    )
    .unwrap();
    m.set_expression(c, "b*2").unwrap();
    m.set_input(a, 17.into()).unwrap();
    assert_eq!(g.dependencies(c).unwrap(), &[b]);
    assert!(!g.dependents(a).unwrap().contains(&c));
    assert!(g.dependents(b).unwrap().contains(&c));
    assert!(g.same_definition(&original, a));
    assert!(g.same_definition(&original, b));
    assert!(!g.same_definition(&original, c));
    assert_eq!(value(&old, c), 3.into());
    assert_eq!(value(&g.capture().unwrap(), c), 6.into());
    let expected = EvaluationContext::new(m.snapshot().unwrap())
        .evaluate(&[c])
        .unwrap();
    assert_eq!(
        g.capture().unwrap().evaluate(&[c]).unwrap()[&c],
        *expected.value(c).unwrap()
    );
    assert_eq!(old.version().commit, 0);
    assert_eq!(g.version().topology, 1);
    assert_eq!(g.version().inputs, 1);
}
#[test]
fn cycles_drafts_and_repairs_never_publish_calculable_cycles() {
    let (_, mut g, a, _, c) = setup();
    g.apply(g.version(), vec![Edit::Formula(a, "c+1".into())])
        .unwrap();
    assert!(matches!(
        g.capture(),
        Err(SnapshotError::Unorderable { .. })
    ));
    g.apply(g.version(), vec![Edit::ReplaceInput(a, 4.into())])
        .unwrap();
    assert_eq!(value(&g.capture().unwrap(), c), 5.into());
    g.apply(g.version(), vec![Edit::Formula(c, "(".into())])
        .unwrap();
    assert!(matches!(g.capture(), Err(SnapshotError::Parse { .. })));
    g.apply(g.version(), vec![Edit::Formula(c, "a/0".into())])
        .unwrap();
    assert!(g.capture().unwrap().evaluate(&[c]).unwrap()[&c].is_err());
}
#[test]
fn retirement_never_rebinds_and_explicit_resubmission_repairs() {
    let (_, mut g, a, _, c) = setup();
    let old = g.capture().unwrap();
    g.apply(
        g.version(),
        vec![Edit::Retire(a), Edit::AddFormula("a".into(), "40+2".into())],
    )
    .unwrap();
    let replacement = g.node_id("a").unwrap();
    assert_ne!(a, replacement);
    assert_eq!(g.dependencies(c).unwrap(), &[a]);
    assert!(
        matches!(g.capture(), Err(SnapshotError::MissingReference { target, .. }) if target == a)
    );
    assert_eq!(value(&old, c), 3.into());
    g.apply(g.version(), vec![Edit::Formula(c, "a+1".into())])
        .unwrap();
    assert_eq!(value(&g.capture().unwrap(), c), 43.into());
}
#[test]
fn hard_failures_staleness_and_validation_budget_are_atomic() {
    let (_, mut g, a, _, c) = setup();
    let version = g.version();
    assert_eq!(
        g.apply(
            version,
            vec![
                Edit::Input(a, 9.into()),
                Edit::AddFormula("a".into(), "1".into())
            ]
        ),
        Err(Error::InvalidTransfer)
    );
    assert_eq!(g.version(), version);
    assert_eq!(value(&g.capture().unwrap(), c), 3.into());
    g.apply(version, vec![Edit::Input(a, 7.into())]).unwrap();
    assert_eq!(
        g.apply(version, vec![Edit::Formula(c, "a+2".into())]),
        Err(Error::Stale)
    );
    let (m, _, _, _, c) = setup();
    let mut limited = Graph::from_model(
        &m,
        Limits {
            validation_visits: 1,
            ..Limits::default()
        },
    )
    .unwrap();
    let v = limited.version();
    assert_eq!(
        limited.apply(v, vec![Edit::Formula(c, "a+2".into())]),
        Err(Error::ValidationBudget)
    );
    assert_eq!(limited.version(), v);
    let mut quota = Graph::from_model(
        &m,
        Limits {
            nodes: 3,
            ..Limits::default()
        },
    )
    .unwrap();
    assert_eq!(
        quota.apply(
            quota.version(),
            vec![Edit::AddFormula("d".into(), "c+1".into())]
        ),
        Err(Error::Quota)
    );
    assert!(quota.node_id("d").is_none());
}
#[test]
fn resolved_transfer_uses_existing_executor_and_rejects_bad_stack_or_foreign_ids() {
    let (_, mut g, a, _, c) = setup();
    let (foreign, _, f, _, _) = setup();
    drop(foreign);
    let v = g.version();
    assert_eq!(
        g.apply(
            v,
            vec![Edit::Resolved(c, "x".into(), vec![ResolvedToken::Node(f)])]
        ),
        Err(Error::InvalidTransfer)
    );
    assert_eq!(
        g.apply(v, vec![Edit::Resolved(c, "x".into(), vec![])]),
        Err(Error::InvalidTransfer)
    );
    g.apply(
        v,
        vec![Edit::Resolved(
            c,
            "historical provenance".into(),
            vec![ResolvedToken::Node(a)],
        )],
    )
    .unwrap();
    assert_eq!(value(&g.capture().unwrap(), c), 2.into());
}
#[test]
fn retained_generations_many_pages_changing_formulas_and_independent_oracle() {
    let mut m = WorkingModel::new("Many").unwrap();
    let a = m.add_input("a", 3.into()).unwrap();
    let b = m.add_input("b", 7.into()).unwrap();
    let ids: Vec<_> = (0..256)
        .map(|i| m.add_computed(format!("f{i}"), "a+1").unwrap())
        .collect();
    let mut g = Graph::from_model(&m, Limits::default()).unwrap();
    let old = g.capture().unwrap();
    let mut retained = Vec::new();
    for (i, &id) in ids.iter().enumerate() {
        g.apply(g.version(), vec![Edit::Formula(id, format!("b+{i}"))])
            .unwrap();
        if i % 16 == 0 {
            retained.push((i, g.capture().unwrap()));
        }
    }
    for (i, &id) in ids.iter().enumerate() {
        assert_eq!(value(&old, id), 4.into());
        assert_eq!(value(&g.capture().unwrap(), id), (7 + i as i64).into());
        assert_eq!(g.dependencies(id).unwrap(), &[b]);
    }
    assert!(g.dependents(a).unwrap().is_empty());
    for (last, capture) in retained {
        assert_eq!(value(&capture, ids[last + 1]), 4.into());
    }
}
