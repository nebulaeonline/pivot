use pivot_core::{
    evaluation::{EvaluationContext, InvocationRequest},
    interface::{InputRequirement, NamedInvocationRequest},
    model::{
        DraftDefinition, ModelError, NodeId, SnapshotDefinition, SnapshotError, WorkingModel,
        integration::{CalculationFailure, CancellationToken, CaptureFailure, IntegrationRuntime},
        transfer,
    },
};
use pivot_types::{PivotDecimal, PivotValue};

fn runtime() -> IntegrationRuntime {
    IntegrationRuntime::new(256 * 1024 * 1024, 16).unwrap()
}
fn finish(r: IntegrationRuntime) {
    assert!(r.finish().is_ok());
}
fn pair(r: &IntegrationRuntime) -> (WorkingModel, WorkingModel) {
    let a = WorkingModel::new_in("Stage 2", r).unwrap();
    let b = a.stage();

    (a, b)
}
fn drafts(m: &WorkingModel) -> Vec<(NodeId, String, String)> {
    m.draft_nodes()
        .map(|(id, name, d)| {
            (
                id,
                name.into(),
                match d {
                    DraftDefinition::NonNumeric => "nonnumeric".into(),
                    DraftDefinition::Input(v) => format!("{:?}", v.parts()),
                    DraftDefinition::Computed { source, binding } => {
                        format!("{source}:{binding:?}")
                    }
                },
            )
        })
        .collect()
}
fn compare(a: &WorkingModel, b: &WorkingModel) {
    assert_eq!(a.revision(), b.revision());
    assert_eq!(drafts(a), drafts(b));
    match (a.snapshot(), b.snapshot()) {
        (Err(a), Err(b)) => assert_eq!(a, b),
        (Ok(a), Ok(b)) => {
            assert_eq!(a.revision(), b.revision());
            assert_eq!(a.dependency_order(), b.dependency_order());
            let expected = EvaluationContext::new(a.clone()).evaluate_all();
            let result = EvaluationContext::new(b.clone()).evaluate_all();
            assert_eq!(expected.values(), result.values());
            assert_eq!(b.revision(), result.revision());
        }
        _ => panic!("snapshot validity differs"),
    }
}

#[test]
fn every_port_operation_default_override_and_noop_preserves_exact_revisions() {
    let r = runtime();
    let (mut a, mut b) = pair(&r);
    let x = a.add_input("x", 3.into()).unwrap();
    b.add_input("x", 3.into()).unwrap();
    let y = a.add_input("y", 7.into()).unwrap();
    b.add_input("y", 7.into()).unwrap();
    let out = a.add_computed("out", "x+y").unwrap();
    b.add_computed("out", "x+y").unwrap();
    let input = a
        .add_input_port("input", x, InputRequirement::Required)
        .unwrap();
    assert_eq!(
        input,
        b.add_input_port("input", x, InputRequirement::Required)
            .unwrap()
    );
    let optional = a
        .add_input_port(
            "optional",
            y,
            InputRequirement::Optional {
                default: PivotValue::Decimal(11.into()),
            },
        )
        .unwrap();
    assert_eq!(
        optional,
        b.add_input_port(
            "optional",
            y,
            InputRequirement::Optional {
                default: PivotValue::Decimal(11.into())
            }
        )
        .unwrap()
    );
    let output = a.add_output_port("result", out).unwrap();
    assert_eq!(output, b.add_output_port("result", out).unwrap());
    for request in [
        NamedInvocationRequest {
            inputs: vec![],
            outputs: vec!["result".into()],
        },
        NamedInvocationRequest {
            inputs: vec![("input".into(), PivotValue::Decimal(5.into()))],
            outputs: vec!["result".into()],
        },
        NamedInvocationRequest {
            inputs: vec![
                ("input".into(), PivotValue::Decimal(5.into())),
                ("optional".into(), PivotValue::Decimal(2.into())),
            ],
            outputs: vec!["result".into()],
        },
    ] {
        let left = a.capture_calculation().unwrap().invoke_named(&request);
        let right = b.capture_local().unwrap().invoke_named(&request);
        match (left, right) {
            (Ok(a), Ok(b)) => {
                assert_eq!(a.outputs(), b.outputs());
                assert_eq!(a.inputs(), b.inputs());
                assert_eq!(a.revision(), b.revision());
            }
            (Err(a), Err(b)) => assert_eq!(a, b),
            _ => panic!("invocation mismatch"),
        }
    }
    let old = b.capture_local().unwrap();
    for m in [&mut a, &mut b] {
        let before = m.revision();
        m.rename_port(input, "input").unwrap();
        m.patch_input_port(input, None, None).unwrap();
        assert_eq!(before, m.revision());
        m.rename_port(input, "renamed").unwrap();
        m.patch_input_port(
            input,
            None,
            Some(InputRequirement::Optional {
                default: PivotValue::Decimal(13.into()),
            }),
        )
        .unwrap();
        assert!(matches!(
            m.update_input_port(input, y, InputRequirement::Required),
            Err(ModelError::Interface(_))
        ));
        m.update_output_port(output, x).unwrap();
        m.remove_port(optional).unwrap();
        m.update_input_port(input, y, InputRequirement::Required)
            .unwrap();
    }
    compare(&a, &b);
    assert_eq!(
        old.invoke_named(&NamedInvocationRequest {
            inputs: vec![("input".into(), PivotValue::Decimal(1.into()))],
            outputs: vec!["result".into()]
        })
        .unwrap()
        .outputs()["result"],
        Ok(PivotValue::Decimal(12.into()))
    );
    drop((a, b, old));
    finish(r);
}

#[test]
fn raw_invocation_isolated_overrides_errors_and_retained_capture_are_equal() {
    let r = runtime();
    let (mut a, mut b) = pair(&r);
    let x = a.add_input("x", 2.into()).unwrap();
    b.add_input("x", 2.into()).unwrap();
    let y = a.add_computed("y", "x*3").unwrap();
    b.add_computed("y", "x*3").unwrap();
    let ca = a.capture_calculation().unwrap();
    let cb = b.capture_local().unwrap();
    for request in [
        InvocationRequest {
            inputs: vec![(x, 5.into())],
            outputs: vec![y],
        },
        InvocationRequest {
            inputs: vec![(x, 5.into()), (x, 2.into())],
            outputs: vec![y],
        },
        InvocationRequest {
            inputs: vec![(y, 5.into())],
            outputs: vec![x],
        },
    ] {
        match (ca.invoke(&request), cb.invoke(&request)) {
            (Ok(a), Ok(b)) => {
                assert_eq!(a.values(), b.values());
                assert_eq!(a.input_overrides(), b.input_overrides());
            }
            (Err(a), Err(b)) => assert_eq!(a, b),
            _ => panic!("override mismatch"),
        }
    }
    b.set_input(x, 9.into()).unwrap();
    assert_eq!(cb.evaluate(&[y]).unwrap().values()[&y], Ok(6.into()));
    assert_eq!(
        b.capture_local().unwrap().evaluate(&[y]).unwrap().values()[&y],
        Ok(27.into())
    );
    drop((a, b, ca, cb));
    finish(r);
}

#[test]
fn local_draft_capture_checks_selected_closure_and_full_errors_remain_exact() {
    let r = runtime();
    let (mut a, mut b) = pair(&r);
    let x = a.add_input("x", 2.into()).unwrap();
    b.add_input("x", 2.into()).unwrap();
    let bad = a.add_computed("bad", "(").unwrap();
    b.add_computed("bad", "(").unwrap();
    let cycle = a.add_computed("cycle", "cycle+1").unwrap();
    b.add_computed("cycle", "cycle+1").unwrap();
    for m in [&mut a, &mut b] {
        m.add_computed("blocked", "cycle+1").unwrap();
    }
    let local = b.capture_local().unwrap();
    let oracle = a.capture_local().unwrap();
    assert_eq!(
        oracle.evaluate(&[x]).unwrap().values(),
        local.evaluate(&[x]).unwrap().values()
    );
    assert_eq!(local.evaluate(&[x]).unwrap().values()[&x], Ok(2.into()));
    assert!(matches!(
        local.evaluate(&[bad]),
        Err(CalculationFailure::LocalInvalid(
            SnapshotError::Parse { .. }
        ))
    ));
    assert!(matches!(
        local.evaluate(&[cycle]),
        Err(CalculationFailure::LocalInvalid(
            SnapshotError::Unorderable { .. }
        ))
    ));
    assert_eq!(
        b.capture_calculation().err(),
        Some(CaptureFailure::Invalid(a.snapshot().unwrap_err()))
    );
    for m in [&mut a, &mut b] {
        m.set_expression(bad, "x+1").unwrap();
    }
    assert_eq!(a.snapshot().unwrap_err(), b.snapshot().unwrap_err());
    let before = b.revision();
    for m in [&mut a, &mut b] {
        m.set_expression(cycle, "x*2").unwrap();
    }
    assert_ne!(before, b.revision());
    compare(&a, &b);
    assert!(matches!(
        local.evaluate(&[bad]),
        Err(CalculationFailure::LocalInvalid(_))
    ));
    drop((a, b, local, oracle));
    finish(r);
}

#[test]
fn pinned_inspection_pages_reverse_edges_and_full_snapshot_preserve_provenance() {
    let r = runtime();
    let (mut a, mut b) = pair(&r);
    let root = a.add_input("root", 1.into()).unwrap();
    b.add_input("root", 1.into()).unwrap();
    let mut nodes = vec![root];
    for i in 0..300 {
        let name = format!("f{i}");
        nodes.push(a.add_computed(&name, "root+1").unwrap());
        b.add_computed(name, "root+1").unwrap();
    }
    let view = b.inspect().unwrap();
    let revision = view.revision();
    let mut all = Vec::new();
    let mut cursor = None;
    loop {
        let page = view.nodes(cursor, 17, 23).unwrap();
        all.extend(page.nodes().iter().map(|n| n.id));
        cursor = page.next();
        if cursor.is_none() {
            break;
        }
    }
    assert_eq!(all, nodes);
    let mut reverse = Vec::new();
    let mut after = None;
    loop {
        let page = view.edges(root, true, after, 7).unwrap();
        reverse.extend_from_slice(page.nodes());
        after = page.next();
        if after.is_none() {
            break;
        }
    }
    assert_eq!(reverse, nodes[1..]);
    assert_eq!(
        view.edges(nodes[1], false, None, 3).unwrap().nodes(),
        &[root]
    );
    let current = b.inspect().unwrap();
    let next = view.nodes(None, 1, 1).unwrap().next();
    assert!(current.nodes(next, 1, 1).is_err());
    assert!(view.nodes(None, 257, 1).is_err());
    let snapshot = a.snapshot().unwrap();
    for &id in &nodes {
        let n = view.node(id).unwrap();
        let expected = snapshot.node(id).unwrap();
        assert_eq!(n.name, expected.name());
        if let SnapshotDefinition::Computed { source, tokens } = expected.definition() {
            assert_eq!(n.tokens.unwrap(), tokens);
            match n.definition {
                DraftDefinition::Computed { source: s, .. } => assert_eq!(s, source),
                _ => panic!("kind"),
            }
        }
    }
    b.rename(root, "renamed").unwrap();
    b.set_input(root, 9.into()).unwrap();
    b.remove(nodes[4]).unwrap();
    let old = view.snapshot(&CancellationToken::default()).unwrap();
    assert_eq!(old.revision(), revision);
    assert_eq!(old.node_id("root"), Some(root));
    assert!(old.node(nodes[4]).is_some());
    assert_eq!(
        EvaluationContext::new(old.clone())
            .evaluate(&[nodes[1]])
            .unwrap()
            .value(nodes[1]),
        Some(&Ok(2.into()))
    );
    drop((a, b, view, current, old, snapshot));
    finish(r);
}

#[test]
fn transfer_preserves_retired_ledgers_binding_source_ports_and_exact_parts() {
    let r = runtime();
    let mut m = WorkingModel::new_in("portable", &r).unwrap();

    let value = PivotDecimal::from_parts(12300, 0, 0, true, 4).unwrap();
    let x = m.add_input("x", value).unwrap();
    let y = m.add_computed("y", "x+1").unwrap();
    let p = m
        .add_input_port(
            "input",
            x,
            InputRequirement::Optional {
                default: PivotValue::Decimal(value),
            },
        )
        .unwrap();
    let q = m.add_output_port("output", y).unwrap();
    m.remove_port(p).unwrap();
    m.rename(x, "renamed").unwrap();
    m.remove(x).unwrap();
    let reused = m.add_input("x", 9.into()).unwrap();
    let image = transfer::export(&m, &[x, y, reused], &[p, q]).unwrap();
    let imported = r
        .import_portable(image.clone(), &CancellationToken::default())
        .unwrap();
    let next = transfer::export(&imported.model, &imported.nodes, &imported.ports).unwrap();
    assert_eq!(format!("{image:?}"), format!("{next:?}"));
    assert!(
        matches!(imported.model.snapshot(),Err(SnapshotError::MissingReference{node,target}) if node==imported.nodes[1] && target==imported.nodes[0])
    );
    let mut restored = imported.model;
    restored.set_expression(imported.nodes[1], "x+1").unwrap();
    assert_eq!(
        restored
            .capture_local()
            .unwrap()
            .evaluate(&[imported.nodes[1]])
            .unwrap()
            .values()[&imported.nodes[1]],
        Ok(10.into())
    );
    let mut malformed = image.clone();
    malformed.nodes[1].slot = malformed.nodes[0].slot;
    assert!(
        r.import_portable(malformed, &CancellationToken::default())
            .is_err()
    );
    let cancelled = CancellationToken::default();
    cancelled.cancel();
    assert!(r.import_portable(image, &cancelled).is_err());
    drop((m, restored));
    finish(r);
}

#[test]
fn deterministic_branch_edits_staging_rollback_and_invalid_models() {
    let r = runtime();
    let (mut a, mut b) = pair(&r);
    let mut nodes = Vec::new();
    for i in 0..32 {
        nodes.push(a.add_input(format!("n{i}"), (i as i64).into()).unwrap());
        b.add_input(format!("n{i}"), (i as i64).into()).unwrap();
    }
    let mut seed = 91u64;
    for step in 0..128 {
        seed = seed.wrapping_mul(6364136223846793005).wrapping_add(1);
        let id = nodes[(seed as usize >> 8) % nodes.len()];
        let action = (seed >> 32) % 6;
        let left = match action {
            0 => a.set_input(id, (step as i64).into()),
            1 => a.set_expression(id, format!("n0+{step}")),
            2 => a.replace_input(id, (step as i64).into()),
            3 => a.rename(id, format!("label{step}")),
            4 => a.remove(id),
            _ => a.set_expression(id, "("),
        };
        let right = match action {
            0 => b.set_input(id, (step as i64).into()),
            1 => b.set_expression(id, format!("n0+{step}")),
            2 => b.replace_input(id, (step as i64).into()),
            3 => b.rename(id, format!("label{step}")),
            4 => b.remove(id),
            _ => b.set_expression(id, "("),
        };
        assert_eq!(left, right);
        if step % 8 == 0 {
            let mut ca = a.stage();
            let mut cb = b.stage();
            let name = format!("added{step}");
            let x = ca.add_input(&name, 7.into()).unwrap();
            assert_eq!(x, cb.add_input(&name, 7.into()).unwrap());
            let y = ca
                .add_computed(format!("derived{step}"), format!("{name}+1"))
                .unwrap();
            assert_eq!(
                y,
                cb.add_computed(format!("derived{step}"), format!("{name}+1"))
                    .unwrap()
            );
            compare(&ca, &cb);
            if step % 16 == 0 {
                a = ca;
                b = cb;
                nodes.extend([x, y]);
            }
        }
        compare(&a, &b);
    }
    drop((a, b));
    finish(r);
}

#[test]
fn cancellation_and_failed_snapshot_admission_keep_writer_and_old_capture() {
    let r = runtime();
    let (_, mut m) = pair(&r);
    let x = m.add_input("x", 1.into()).unwrap();
    let capture = m.capture_local().unwrap();
    let view = m.inspect().unwrap();
    let before = m.revision();
    let cancel = CancellationToken::default();
    cancel.cancel();
    assert_eq!(
        capture
            .invoke_cancelled(
                &InvocationRequest {
                    inputs: vec![],
                    outputs: vec![x]
                },
                &cancel
            )
            .err(),
        Some(CalculationFailure::Cancelled)
    );
    assert_eq!(
        view.snapshot(&cancel).err(),
        Some(SnapshotError::Backend("cancelled"))
    );
    r.flush();
    let memory = r.memory();
    let hold = memory
        .reserve(memory.stats().limit - memory.stats().charged - 32 * 1024)
        .unwrap();
    assert!(m.snapshot().is_err());
    assert_eq!(m.revision(), before);
    drop(hold);
    r.flush();
    m.set_input(x, 7.into()).unwrap();
    assert_eq!(capture.evaluate(&[x]).unwrap().values()[&x], Ok(1.into()));
    assert_eq!(
        view.snapshot(&CancellationToken::default())
            .unwrap()
            .revision(),
        before
    );
    drop((m, capture, view));
    finish(r);
    assert_eq!(memory.stats().charged, 0);
}

#[test]
fn nonnumeric_marker_is_valid_but_frozen_scalar_exports_reject_it() {
    let r = runtime();
    let mut m = WorkingModel::new_in("Marker", &r).unwrap();
    let id = m.add_input("label", 1.into()).unwrap();
    let mut batch = m.begin_batch(1, 0).unwrap();
    batch.set_nonnumeric(id).unwrap();
    m.apply_batch(batch, &CancellationToken::default()).unwrap();
    let snapshot = m.snapshot().unwrap();
    assert!(matches!(
        EvaluationContext::new(snapshot.clone())
            .evaluate_all()
            .value(id),
        Some(Err(
            pivot_core::evaluation::EvaluationError::NonNumericInput { .. }
        ))
    ));
    assert!(transfer::export(&m, &[id], &[]).is_err());
    assert!(transfer::export_snapshot(&snapshot, &[id], &[]).is_err());
    drop(snapshot);
    drop(m);
    let memory = r.memory();
    finish(r);
    assert_eq!(memory.stats().charged, 0);
}
