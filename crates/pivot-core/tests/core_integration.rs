#![cfg(feature = "core-integration")]
use pivot_core::{
    evaluation::EvaluationError,
    model::{
        DraftDefinition, ModelError, SnapshotError, WorkingModel,
        integration::{CaptureFailure, IntegrationRuntime},
    },
};
use pivot_types::PivotDecimal;

fn runtime() -> IntegrationRuntime {
    IntegrationRuntime::new(256 * 1024 * 1024, 8).unwrap()
}
fn finish(r: IntegrationRuntime) {
    assert!(r.finish().is_ok());
}
fn pair(r: &IntegrationRuntime) -> (WorkingModel, WorkingModel) {
    let old = WorkingModel::new_reference("integration").unwrap();
    let mut new = old.stage();
    new.enable_persistent(r).unwrap();
    (old, new)
}
fn equal(old: &WorkingModel, new: &WorkingModel, outputs: &[pivot_core::model::NodeId]) {
    assert_eq!(old.revision(), new.revision());
    let a = old.capture_calculation().ok().unwrap();
    let b = new.capture_calculation().ok().unwrap();
    assert_eq!(a.revision(), b.revision());
    let a = a.evaluate(outputs).ok().unwrap();
    let b = b.evaluate(outputs).ok().unwrap();
    assert_eq!(a.values(), b.values());
    assert_eq!(a.revision(), b.revision());
}

#[test]
fn facade_revisions_staging_and_independent_arithmetic_oracle() {
    let r = runtime();
    let (mut old, mut new) = pair(&r);
    let a = old.add_input("a", 3.into()).unwrap();
    assert_eq!(a, new.add_input("a", 3.into()).unwrap());
    let mut outputs = vec![a];
    for i in 0..128 {
        let source = if i == 0 {
            "a*2+1".into()
        } else {
            format!("n{}+a", i - 1)
        };
        let id = old.add_computed(format!("n{i}"), &source).unwrap();
        assert_eq!(id, new.add_computed(format!("n{i}"), source).unwrap());
        outputs.push(id);
    }
    equal(&old, &new, &outputs);
    let retained = new.capture_calculation().ok().unwrap();
    let expected = new.revision();
    let mut branch = new.stage_expected(expected).unwrap();
    let mut reference = old.stage_expected(expected).unwrap();
    for m in [&mut branch, &mut reference] {
        m.set_input(a, 9.into()).unwrap();
        m.set_expression(outputs[1], "a*4").unwrap();
    }
    equal(&reference, &branch, &outputs);
    assert!(branch.stage_expected(expected).is_err());
    assert!(new.stage_expected(expected).is_ok());
    let result = retained.evaluate(&outputs).ok().unwrap();
    assert_eq!(
        result.values()[outputs.last().unwrap()],
        Ok(PivotDecimal::from(388))
    );
    drop((retained, result, branch, reference, old, new));
    finish(r);
}

#[test]
fn failed_edits_equal_assignments_empty_stages_and_resource_rejection() {
    let r = runtime();
    let (mut old, mut new) = pair(&r);
    let a = old.add_input("a", 1.into()).unwrap();
    new.add_input("a", 1.into()).unwrap();
    for m in [&mut old, &mut new] {
        let before = m.revision();
        assert_eq!(
            m.add_input("a", 2.into()),
            Err(ModelError::DuplicateName("a".into()))
        );
        assert_eq!(
            m.add_input("1bad", 2.into()),
            Err(ModelError::InvalidName("1bad".into()))
        );
        assert_eq!(before, m.revision());
        assert_eq!(before, m.stage().revision());
        m.set_input(a, 1.into()).unwrap(); // Accepted behavior advances even if equal.
        assert_ne!(before, m.revision());
    }
    equal(&old, &new, &[a]);
    r.flush(); // No queued obsolete roots may release headroom during this check.
    let memory = r.memory();
    let headroom = memory.stats().limit - memory.stats().charged - 64;
    let hold = memory.reserve(headroom).unwrap();
    let before = new.revision();
    assert!(matches!(
        new.capture_calculation(),
        Err(CaptureFailure::Resource(_))
    ));
    assert!(matches!(
        new.set_input(a, 8.into()),
        Err(ModelError::Backend(_))
    ));
    assert_eq!(before, new.revision());
    let mut source = old.stage();
    assert!(matches!(
        source.enable_persistent(&r),
        Err(ModelError::Backend(_))
    ));
    assert_eq!(source.revision(), old.revision());
    assert_eq!(source.node_id("a"), Some(a));
    assert!(source.snapshot().is_ok());
    drop(source);
    drop(hold);
    equal(&old, &new, &[a]);
    drop((old, new));
    finish(r);
    assert_eq!(memory.stats().charged, 0);
}

#[test]
fn exact_diagnostics_multiple_cycles_blocked_nodes_and_explicit_repair() {
    let r = runtime();
    let (mut old, mut new) = pair(&r);
    let a = old.add_input("a", 1.into()).unwrap();
    new.add_input("a", 1.into()).unwrap();
    let b = old.add_computed("b", "a+1").unwrap();
    new.add_computed("b", "a+1").unwrap();
    let c = old.add_computed("c", "c+1").unwrap();
    new.add_computed("c", "c+1").unwrap();
    old.add_computed("downstream", "b+c").unwrap();
    new.add_computed("downstream", "b+c").unwrap();
    old.set_expression(a, "b").unwrap();
    new.set_expression(a, "b").unwrap();
    let expected = old.snapshot().unwrap_err();
    assert!(matches!(&expected, SnapshotError::Unorderable { nodes } if nodes.len() == 4));
    assert_eq!(
        new.capture_calculation().err(),
        Some(CaptureFailure::Invalid(expected))
    );
    for m in [&mut old, &mut new] {
        m.set_expression(a, "4").unwrap();
        m.set_expression(c, "2").unwrap();
    }
    equal(&old, &new, &[b, c]);
    for source in ["missing+1", "(", "a+1"] {
        old.set_expression(b, source).unwrap();
        new.set_expression(b, source).unwrap();
        match old.snapshot() {
            Err(e) => assert_eq!(
                new.capture_calculation().err(),
                Some(CaptureFailure::Invalid(e))
            ),
            Ok(_) => equal(&old, &new, &[b]),
        }
    }
    drop((old, new));
    finish(r);
}

#[test]
fn activation_preserves_renames_retired_ids_name_reuse_and_source() {
    let r = runtime();
    let mut old = WorkingModel::new_reference("history").unwrap();
    let a = old.add_input("a", 5.into()).unwrap();
    let b = old.add_computed("b", "a+2").unwrap();
    old.rename(a, "renamed").unwrap();
    let replacement = old.add_input("a", 100.into()).unwrap();
    let mut new = old.stage();
    new.enable_persistent(&r).unwrap();
    equal(&old, &new, &[b]);
    let sources: Vec<_> = new
        .draft_nodes()
        .filter_map(|(id, _, d)| match d {
            DraftDefinition::Computed { source, binding } => {
                Some((id, source.to_string(), binding.unwrap().to_vec()))
            }
            _ => None,
        })
        .collect();
    assert_eq!(sources, vec![(b, "a+2".into(), vec![a])]);
    // Retirement is explicitly outside Stage 1; import its accepted history.
    old.remove(a).unwrap();
    drop(new);
    let mut new = old.stage();
    new.enable_persistent(&r).unwrap();
    assert_eq!(
        new.capture_calculation().err(),
        Some(CaptureFailure::Invalid(old.snapshot().unwrap_err()))
    );
    old.set_expression(b, "a+2").unwrap();
    new.set_expression(b, "a+2").unwrap();
    equal(&old, &new, &[b, replacement]);
    let result = new
        .capture_calculation()
        .ok()
        .unwrap()
        .evaluate(&[b])
        .ok()
        .unwrap();
    assert_eq!(result.values()[&b], Ok(102.into()));
    drop((old, new, result));
    finish(r);
}

#[test]
fn activated_operations_never_edit_empty_reference_storage() {
    let r = runtime();
    let (_, mut m) = pair(&r);
    let a = m.add_input("a", 1.into()).unwrap();
    let before = m.revision();
    m.rename(a, "z").unwrap();
    assert_ne!(before, m.revision());
    let same = m.revision();
    m.rename(a, "z").unwrap();
    assert_eq!(same, m.revision());
    m.replace_input(a, 2.into()).unwrap();
    let snapshot = m.snapshot().unwrap();
    assert_eq!(snapshot.revision(), m.revision());
    assert!(m.contains_node(a));
    assert_eq!(m.node_id("z"), Some(a));
    assert_eq!(m.draft_nodes().count(), 1);
    assert!(pivot_core::model::transfer::export(&m, &[a], &[]).is_ok());
    m.remove(a).unwrap();
    assert!(!m.contains_node(a));
    assert_eq!(snapshot.node_id("z"), Some(a));
    drop((m, snapshot));
    finish(r);
}

#[test]
fn retained_generations_and_runtime_shutdown_ownership() {
    let r = runtime();
    let memory = r.memory();
    let (_, mut m) = pair(&r);
    let a = m.add_input("a", 1.into()).unwrap();
    let mut captures = Vec::new();
    for value in 0..32 {
        m.set_input(a, value.into()).unwrap();
        captures.push(m.capture_calculation().ok().unwrap());
    }
    let r = r.finish().expect_err("live roots prevent joining");
    drop(m);
    let results: Vec<_> = captures
        .iter()
        .map(|c| c.evaluate(&[a]).ok().unwrap())
        .collect();
    drop(captures);
    for (i, result) in results.iter().enumerate() {
        assert_eq!(result.values()[&a], Ok((i as i64).into()));
    }
    drop(results);
    finish(r);
    assert_eq!(memory.stats().charged, 0);
}

#[test]
fn arithmetic_failures_unknown_outputs_and_repeated_formula_revisions() {
    let r = runtime();
    let (mut old, mut new) = pair(&r);
    let a = old.add_input("a", 1.into()).unwrap();
    new.add_input("a", 1.into()).unwrap();
    let b = old.add_computed("b", "a/0").unwrap();
    new.add_computed("b", "a/0").unwrap();
    let c = old.add_computed("c", "b+1").unwrap();
    new.add_computed("c", "b+1").unwrap();
    equal(&old, &new, &[a, b, c]);
    let unknown = WorkingModel::new_reference("other")
        .unwrap()
        .add_input("x", 1.into())
        .unwrap();
    let x = old
        .capture_calculation()
        .ok()
        .unwrap()
        .evaluate(&[unknown])
        .err();
    let y = new
        .capture_calculation()
        .ok()
        .unwrap()
        .evaluate(&[unknown])
        .err();
    assert_eq!(x, y);
    let other = WorkingModel::new_reference("another")
        .unwrap()
        .add_input("y", 1.into())
        .unwrap();
    for outputs in [vec![other, unknown], vec![unknown, other, unknown]] {
        assert_eq!(
            old.capture_calculation().unwrap().evaluate(&outputs).err(),
            new.capture_calculation().unwrap().evaluate(&outputs).err()
        );
    }
    let result = new
        .capture_calculation()
        .ok()
        .unwrap()
        .evaluate(&[c])
        .ok()
        .unwrap();
    assert!(matches!(
        result.values()[&c],
        Err(EvaluationError::DependencyFailed { .. })
    ));
    for m in [&mut old, &mut new] {
        let before = m.revision();
        m.set_expression(b, "a/0").unwrap();
        assert_ne!(before, m.revision());
        assert_eq!(m.set_input(b, 2.into()), Err(ModelError::NotInput(b)));
    }
    equal(&old, &new, &[b]);
    drop((old, new, result));
    finish(r);
}

#[test]
fn exact_decimal_parts_and_imported_ports_survive_activation() {
    use pivot_core::interface::InputRequirement;
    let r = runtime();
    let mut old = WorkingModel::new_reference("parts").unwrap();
    let value = PivotDecimal::from_parts(0, 0, 0, true, 12).unwrap();
    let a = old.add_input("a", value).unwrap();
    let port = old
        .add_input_port("input", a, InputRequirement::Required)
        .unwrap();
    old.add_output_port("output", a).unwrap();
    let mut new = old.stage();
    new.enable_persistent(&r).unwrap();
    let result = new.capture_calculation().unwrap().evaluate(&[a]).unwrap();
    assert_eq!(result.values()[&a].as_ref().unwrap().parts(), value.parts());
    assert_eq!(new.interface().inputs().next().unwrap().id(), port);
    assert_eq!(new.interface().outputs().next().unwrap().name(), "output");
    let before = new.revision();
    new.rename_port(port, "input").unwrap();
    new.patch_input_port(port, None, None).unwrap();
    assert_eq!(before, new.revision());
    new.add_output_port("other", a).unwrap();
    assert_ne!(before, new.revision());
    drop((old, new, result));
    finish(r);
}

#[test]
fn import_invalid_parse_unknown_name_order_and_structural_retained_generations() {
    let r = runtime();
    let mut old = WorkingModel::new_reference("drafts").unwrap();
    let a = old.add_computed("a", "missing").unwrap();
    let b = old.add_computed("b", "(").unwrap();
    let mut new = old.stage();
    new.enable_persistent(&r).unwrap();
    assert_eq!(
        new.capture_calculation().err(),
        Some(CaptureFailure::Invalid(old.snapshot().unwrap_err()))
    );
    for m in [&mut old, &mut new] {
        m.set_expression(a, "1").unwrap();
    }
    assert_eq!(
        new.capture_calculation().err(),
        Some(CaptureFailure::Invalid(old.snapshot().unwrap_err()))
    );
    for m in [&mut old, &mut new] {
        m.set_expression(b, "a+1").unwrap();
    }
    equal(&old, &new, &[b]);
    let mut retained = Vec::new();
    for i in 0..32 {
        new.set_expression(b, format!("a+{i}")).unwrap();
        retained.push(new.capture_calculation().unwrap());
    }
    drop(new);
    for (i, capture) in retained.iter().enumerate() {
        assert_eq!(
            capture.evaluate(&[b]).unwrap().values()[&b],
            Ok((i as i64 + 1).into())
        );
    }
    drop((old, retained));
    finish(r);
}

#[test]
fn multiple_runtime_domains_share_known_allocation_admission() {
    use pivot_core::model::integration::AllocationDomain;
    let domain = AllocationDomain::new(8 * 1024 * 1024);
    let first = IntegrationRuntime::with_domain(domain.clone(), 4).unwrap();
    let second = IntegrationRuntime::with_domain(domain.clone(), 4).unwrap();
    let mut model = WorkingModel::new_reference("shared domain").unwrap();
    model.enable_persistent(&first).unwrap();
    model.add_input("x", 1.into()).unwrap();
    assert!(domain.stats().charged > 512 * 1024);
    assert!(IntegrationRuntime::with_domain(domain.clone(), 8192).is_err());
    drop(model);
    finish(first);
    finish(second);
    assert_eq!(domain.stats().charged, 0);
}

#[test]
fn partial_calculation_admission_failure_reclaims_scratch_without_editing_capture() {
    let r = runtime();
    let memory = r.memory();
    let mut model = WorkingModel::new_reference("partial calculation").unwrap();
    let nodes: Vec<_> = (0..1024)
        .map(|i| model.add_input(format!("n{i}"), 1.into()).unwrap())
        .collect();
    for (i, &id) in nodes.iter().enumerate().skip(1) {
        model.set_expression(id, format!("n{}+1", i - 1)).unwrap();
    }
    model.enable_persistent(&r).unwrap();
    let revision = model.revision();
    let capture = model.capture_calculation().unwrap();
    r.flush();
    let hold = memory
        .reserve(memory.stats().limit - memory.stats().charged - 32 * 1024)
        .unwrap();
    assert!(matches!(
        capture.evaluate(&[nodes[1023]]),
        Err(pivot_core::model::integration::CalculationFailure::Resource(_))
    ));
    assert_eq!(model.revision(), revision);
    drop(hold);
    r.flush();
    assert_eq!(
        capture.evaluate(&[nodes[1023]]).unwrap().values()[&nodes[1023]],
        Ok(1024.into())
    );
    drop((model, capture));
    finish(r);
    assert_eq!(memory.stats().charged, 0);
}
