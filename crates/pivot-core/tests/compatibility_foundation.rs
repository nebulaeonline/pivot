#![cfg(feature = "scaling-experiment")]
use pivot_core::{
    evaluation::{EvaluationContext, InvocationError, InvocationRequest},
    interface::{InputRequirement, InterfaceError, NamedInvocationRequest},
    model::{SnapshotError, WorkingModel, structural::*, transfer},
};
use pivot_types::{PivotDecimal, PivotValue};
fn model() -> (
    WorkingModel,
    pivot_core::model::NodeId,
    pivot_core::model::NodeId,
    pivot_core::model::NodeId,
) {
    let mut m = WorkingModel::new("Foundation").unwrap();
    let a = m.add_input("a", 2.into()).unwrap();
    let b = m.add_input("b", 3.into()).unwrap();
    let c = m.add_computed("c", "a+b").unwrap();
    (m, a, b, c)
}
fn value(c: &Capture, id: pivot_core::model::NodeId) -> PivotDecimal {
    c.evaluate(&[id]).unwrap()[&id].clone().unwrap()
}
#[test]
fn public_ports_defaults_required_overrides_and_saved_capture_are_compatible() {
    let (mut m, a, b, c) = model();
    let input = m
        .add_input_port("required", a, InputRequirement::Required)
        .unwrap();
    m.add_input_port(
        "optional",
        b,
        InputRequirement::Optional {
            default: PivotValue::Decimal(7.into()),
        },
    )
    .unwrap();
    let output = m.add_output_port("result", c).unwrap();
    let mut g = Graph::from_model(&m, Limits::default()).unwrap();
    let s = g.capture().unwrap();
    let request = NamedInvocationRequest {
        inputs: vec![("required".into(), PivotValue::Decimal(5.into()))],
        outputs: vec!["result".into()],
    };
    let baseline = EvaluationContext::new(m.snapshot().unwrap())
        .invoke_named(&request)
        .unwrap();
    let actual = s.invoke_named(&request).unwrap();
    assert_eq!(actual.outputs(), baseline.outputs());
    assert_eq!(actual.bindings(), baseline.inputs());
    assert_eq!(
        actual.outputs()["result"],
        Ok(PivotValue::Decimal(12.into()))
    );
    assert!(matches!(
        s.invoke_named(&NamedInvocationRequest {
            inputs: vec![],
            outputs: vec![]
        }),
        Err(NamedError::Interface(InterfaceError::MissingRequired(_)))
    ));
    let invalid = InvocationRequest {
        inputs: vec![(c, 1.into())],
        outputs: vec![c],
    };
    assert!(
        matches!(s.invoke(&invalid),Err(ComputeError::Invocation(InvocationError::NotInput(n))) if n==c)
    );
    g.apply(
        g.version(),
        vec![
            Edit::Port(PortEdit::Rename(input, "renamed".into())),
            Edit::Port(PortEdit::Output(output, a)),
            Edit::Input(a, 99.into()),
        ],
    )
    .unwrap();
    assert_eq!(
        s.invoke_named(&request).unwrap().outputs()["result"],
        Ok(PivotValue::Decimal(12.into()))
    );
    assert_eq!(value(&s, c), 5.into());
    assert_eq!(value(&g.capture().unwrap(), c), 102.into());
    g.apply(
        g.version(),
        vec![Edit::Port(PortEdit::Input(
            input,
            None,
            Some(InputRequirement::Optional {
                default: PivotValue::Decimal(11.into()),
            }),
        ))],
    )
    .unwrap();
    let r = g
        .capture()
        .unwrap()
        .invoke_named(&NamedInvocationRequest {
            inputs: vec![],
            outputs: vec!["result".into()],
        })
        .unwrap();
    assert_eq!(r.outputs()["result"], Ok(PivotValue::Decimal(11.into())));
}
#[test]
fn transfer_round_trip_never_rebinds_historical_source_and_preserves_drafts_ports_and_parts() {
    let (mut m, a, b, c) = model();
    let port = m.add_output_port("out", c).unwrap();
    m.rename(a, "old").unwrap();
    m.remove(b).unwrap();
    let replacement = m.add_input("b", 99.into()).unwrap();
    let bad = m.add_computed("bad", "(").unwrap();
    let exact = PivotDecimal::from_parts(0, 0, 0, true, 28).unwrap();
    let d = m.add_input("exact", exact).unwrap();
    let ids = [a, b, c, replacement, bad, d];
    let ports = [port];
    let image = transfer::export(&m, &ids, &ports).unwrap();
    let restored = Graph::restore(image.clone(), Limits::default(), 16 * 1024 * 1024).unwrap();
    let exported = restored
        .graph
        .export(&restored.nodes, &restored.ports)
        .unwrap();
    assert_eq!(format!("{image:?}"), format!("{exported:?}"));
    assert_eq!(
        restored.graph.dependencies(restored.nodes[2]).unwrap(),
        &[restored.nodes[0], restored.nodes[1]]
    );
    assert!(
        matches!(restored.graph.diagnose().unwrap(),Err(SnapshotError::MissingReference {target,..}) if target==restored.nodes[1])
    );
    let mut g = restored.graph;
    g.apply(
        g.version(),
        vec![
            Edit::Formula(restored.nodes[2], "old+b".into()),
            Edit::Formula(restored.nodes[4], "1".into()),
        ],
    )
    .unwrap();
    assert_eq!(value(&g.capture().unwrap(), restored.nodes[2]), 101.into());
    assert_eq!(
        g.to_model()
            .unwrap()
            .draft_nodes()
            .find(|(id, _, _)| *id == restored.nodes[5])
            .map(|(_, _, d)| match d {
                pivot_core::model::DraftDefinition::Input(v) => v.parts(),
                _ => unreachable!(),
            })
            .unwrap(),
        exact.parts()
    );
}
#[test]
fn invalid_import_multiple_cycles_and_affected_dependents_have_full_diagnostics() {
    let (mut m, a, b, c) = model();
    m.set_expression(a, "c+1").unwrap();
    m.set_expression(b, "b+1").unwrap();
    let mut g = Graph::from_model(&m, Limits::default()).unwrap();
    assert_eq!(g.diagnose().unwrap(), m.snapshot().map(|_| ()));
    g.apply(g.version(), vec![Edit::ReplaceInput(a, 2.into())])
        .unwrap();
    assert!(g.capture().is_err());
    m.replace_input(a, 2.into()).unwrap();
    assert_eq!(g.diagnose().unwrap(), m.snapshot().map(|_| ()));
    g.apply(g.version(), vec![Edit::ReplaceInput(b, 3.into())])
        .unwrap();
    assert_eq!(value(&g.capture().unwrap(), c), 5.into());
    let s = g.capture().unwrap();
    g.apply(g.version(), vec![Edit::Formula(a, "c+1".into())])
        .unwrap();
    let error = g.diagnose().unwrap().unwrap_err();
    assert!(matches!(error,SnapshotError::Unorderable {nodes} if nodes==vec![a,c]));
    assert_eq!(value(&s, c), 5.into());
}
#[test]
fn rename_add_retire_and_explicit_repair_preserve_stable_ids() {
    let (m, a, b, c) = model();
    let mut g = Graph::from_model(&m, Limits::default()).unwrap();
    let s = g.capture().unwrap();
    g.apply(
        g.version(),
        vec![
            Edit::Rename(a, "renamed".into()),
            Edit::AddInput("a".into(), 40.into()),
            Edit::Retire(b),
        ],
    )
    .unwrap();
    assert_eq!(g.dependencies(c).unwrap(), &[a, b]);
    assert!(g.capture().is_err());
    g.apply(g.version(), vec![Edit::Formula(c, "a+renamed".into())])
        .unwrap();
    assert_eq!(value(&g.capture().unwrap(), c), 42.into());
    assert_eq!(value(&s, c), 5.into());
    assert_ne!(g.node_id("a").unwrap(), a);
}
#[test]
fn invalid_port_drafts_and_atomic_port_rejections_reuse_accepted_rules() {
    let (mut m, a, b, c) = model();
    let p = m
        .add_input_port("in", a, InputRequirement::Required)
        .unwrap();
    let mut g = Graph::from_model(&m, Limits::default()).unwrap();
    let v = g.version();
    assert_eq!(
        g.apply(
            v,
            vec![
                Edit::Rename(a, "a".into()),
                Edit::Port(PortEdit::Rename(p, "in".into())),
                Edit::Port(PortEdit::Input(p, None, None))
            ]
        )
        .unwrap(),
        v
    );
    assert!(matches!(
        g.apply(
            v,
            vec![
                Edit::Input(b, 99.into()),
                Edit::Port(PortEdit::AddInput(
                    "other".into(),
                    a,
                    InputRequirement::Required
                ))
            ]
        ),
        Err(Error::Model(pivot_core::model::ModelError::Interface(
            InterfaceError::DuplicateInputTarget(_)
        )))
    ));
    assert_eq!(g.version(), v);
    assert_eq!(value(&g.capture().unwrap(), c), 5.into());
    g.apply(v, vec![Edit::Formula(a, "b+1".into())]).unwrap();
    assert!(
        matches!(g.capture(),Err(CaptureError::Invalid(SnapshotError::Interface(InterfaceError::InvalidBinding(id)))) if id==p)
    );
    g.apply(
        g.version(),
        vec![Edit::Port(PortEdit::Input(
            p,
            None,
            Some(InputRequirement::Required),
        ))],
    )
    .unwrap();
    assert!(g.capture().is_err());
    g.apply(
        g.version(),
        vec![
            Edit::Port(PortEdit::Input(p, Some(b), None)),
            Edit::Port(PortEdit::AddOutput("out".into(), c)),
        ],
    )
    .unwrap();
    assert!(g.capture().is_ok());
    let output = g.interface().outputs().next().unwrap().id();
    g.apply(g.version(), vec![Edit::Port(PortEdit::Remove(output))])
        .unwrap();
    assert_eq!(g.interface().outputs().count(), 0);
}
#[test]
fn memory_admission_rejects_candidates_without_leaks_and_charges_shared_owners_once() {
    let (m, a, _, c) = model();
    let mut g = Graph::from_model_with_budget(&m, Limits::default(), 2 * 1024 * 1024).unwrap();
    let memory = g.memory();
    let allocated_before = memory.stats().allocated;
    let s = g.capture().unwrap();
    assert!(memory.stats().allocated > allocated_before);
    let before = memory.stats();
    let cloned = s.clone();
    assert_eq!(memory.stats().allocated, before.allocated);
    let held = memory
        .reserve(memory.stats().limit - memory.stats().charged - 1024)
        .unwrap();
    let v = g.version();
    assert_eq!(
        g.apply(v, vec![Edit::Input(a, 90.into())]),
        Err(Error::Admission)
    );
    assert_eq!(g.version(), v);
    drop(held);
    assert_eq!(memory.stats().charged, before.charged);
    g.apply(v, vec![Edit::Formula(c, "a+9".into())]).unwrap();
    let result = g.capture().unwrap().evaluate(&[c]).unwrap();
    assert_eq!(result.version(), g.version());
    assert_eq!(result[&c], Ok(11.into()));
    assert_eq!(value(&s, c), 5.into());
    drop(result);
    drop(g);
    assert!(memory.stats().allocated > 0);
    drop(s);
    drop(cloned);
    assert_eq!(memory.stats().charged, 0);
    assert!(matches!(
        Graph::from_model_with_budget(&m, Limits::default(), 64),
        Err(Error::Admission)
    ));
}
#[test]
fn background_reclamation_releases_owners_and_bounded_queue_rejections_preserve_values() {
    let (m, _, _, c) = model();
    let g = Graph::from_model_with_budget(&m, Limits::default(), 4 * 1024 * 1024).unwrap();
    let memory = g.memory();
    let capture = g.capture().unwrap();
    let worker = reclaim::Reclaimer::new(memory.clone(), 4).unwrap();
    assert!(worker.retire((g, capture)).is_ok());
    drop(worker);
    assert_eq!(memory.stats().charged, 0);
    let tiny = memory::Memory::new(64);
    assert!(reclaim::Reclaimer::new(tiny, 4).is_err());
    let g = Graph::from_model(&m, Limits::default()).unwrap();
    let s = g.capture().unwrap();
    assert_eq!(value(&s, c), 5.into());

    struct Gate(std::sync::mpsc::Sender<()>, std::sync::mpsc::Receiver<()>);
    impl Drop for Gate {
        fn drop(&mut self) {
            let _ = self.0.send(());
            let _ = self.1.recv_timeout(std::time::Duration::from_secs(5));
        }
    }
    let memory = memory::Memory::new(2 * 1024 * 1024);
    let worker = reclaim::Reclaimer::new(memory.clone(), 1).unwrap();
    let (entered_tx, entered) = std::sync::mpsc::channel();
    let (release, release_rx) = std::sync::mpsc::channel();
    assert!(worker.retire(Gate(entered_tx, release_rx)).is_ok());
    entered
        .recv_timeout(std::time::Duration::from_secs(3))
        .unwrap();
    assert!(worker.retire(17u32).is_ok());
    let rejected = worker.retire("still owned".to_string()).err().unwrap();
    assert_eq!(rejected.reason, "queue full");
    assert_eq!(rejected.value, "still owned");
    release.send(()).unwrap();
    drop(worker);
    assert_eq!(memory.stats().charged, 0);
}
#[test]
fn differential_generated_dag_edits_and_invocation_errors_have_independent_oracles() {
    let (mut m, a, b, _) = model();
    let mut g = Graph::from_model(&m, Limits::default()).unwrap();
    for i in 0..128 {
        let name = format!("f{i}");
        let source = format!("a*2+b+{i}");
        let id = m.add_computed(&name, &source).unwrap();
        g.apply(g.version(), vec![Edit::AddFormula(name.clone(), source)])
            .unwrap();
        let paged = g.node_id(&name).unwrap();
        assert_eq!(id, paged);
        let actual = g.capture().unwrap().evaluate(&[id]).unwrap();
        assert_eq!(actual[&id], Ok((7 + i as i64).into()));
        let expected = EvaluationContext::new(m.snapshot().unwrap())
            .evaluate(&[id])
            .unwrap();
        assert_eq!(actual[&id], *expected.value(id).unwrap());
    }
    let duplicate = InvocationRequest {
        inputs: vec![(a, 4.into()), (a, 5.into())],
        outputs: vec![b],
    };
    assert!(
        matches!(g.capture().unwrap().invoke(&duplicate),Err(ComputeError::Invocation(InvocationError::DuplicateInput(n))) if n==a)
    );
}

#[test]
fn trusted_token_and_port_payloads_reserve_headroom_before_construction() {
    let (m, _, _, c) = model();
    let mut limited =
        Graph::from_model_with_budget(&m, Limits::default(), 2 * 1024 * 1024).unwrap();
    let version = limited.version();
    let tokens = || {
        std::iter::once(transfer::ResolvedToken::Decimal(1.into()))
            .chain(std::iter::repeat_n(
                transfer::ResolvedToken::Unary(pivot_core::expression::UnaryOp::Plus),
                100_000,
            ))
            .collect()
    };
    assert_eq!(
        limited.apply(
            version,
            vec![Edit::Resolved(c, "provenance".into(), tokens())]
        ),
        Err(Error::Admission)
    );
    assert_eq!(
        limited.apply(
            version,
            vec![Edit::Port(PortEdit::AddOutput("p".repeat(100_000), c))]
        ),
        Err(Error::Admission)
    );
    assert_eq!(limited.version(), version);
    assert_eq!(value(&limited.capture().unwrap(), c), 5.into());
    let mut admitted =
        Graph::from_model_with_budget(&m, Limits::default(), 64 * 1024 * 1024).unwrap();
    admitted
        .apply(
            admitted.version(),
            vec![Edit::Resolved(c, "provenance".into(), tokens())],
        )
        .unwrap();
    assert_eq!(value(&admitted.capture().unwrap(), c), 1.into());
}

#[test]
fn wide_existing_dependencies_and_retirement_reserve_structural_headroom() {
    let mut m = WorkingModel::new("Wide").unwrap();
    let inputs: Vec<_> = (0..5000)
        .map(|i| m.add_input(format!("n{i}"), 1.into()).unwrap())
        .collect();
    let output = m.add_input("out", 0.into()).unwrap();
    let mut g = Graph::from_model_with_budget(&m, Limits::default(), 16 * 1024 * 1024).unwrap();
    let mut tokens = vec![transfer::ResolvedToken::Node(inputs[0])];
    for &id in &inputs[1..] {
        tokens.push(transfer::ResolvedToken::Node(id));
        tokens.push(transfer::ResolvedToken::Binary(
            pivot_core::expression::BinaryOp::Add,
        ));
    }
    g.apply(
        g.version(),
        vec![Edit::Resolved(
            output,
            "bound wide definition".into(),
            tokens,
        )],
    )
    .unwrap();
    let memory = g.memory();
    let version = g.version();
    let held = memory
        .reserve(memory.stats().limit - memory.stats().charged - 128 * 1024)
        .unwrap();
    assert_eq!(
        g.apply(version, vec![Edit::Formula(output, "1".into())]),
        Err(Error::Admission)
    );
    drop(held);
    assert_eq!(g.version(), version);
    assert_eq!(value(&g.capture().unwrap(), output), 5000.into());
    let (m, nodes, _) = benchmark_model(1000, 998);
    let mut g = Graph::from_model(&m, Limits::default()).unwrap();
    let memory = g.memory();
    let version = g.version();
    let held = memory
        .reserve(memory.stats().limit - memory.stats().charged - 64 * 1024)
        .unwrap();
    assert_eq!(
        g.apply(version, vec![Edit::Retire(nodes[0])]),
        Err(Error::Admission)
    );
    drop(held);
    assert_eq!(g.version(), version);
    assert_eq!(value(&g.capture().unwrap(), nodes[2]), 2.into());
}
