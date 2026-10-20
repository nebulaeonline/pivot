use pivot_core::model::{
    SnapshotError,
    integration::{CancellationToken, IntegrationRuntime},
};
fn finish(r: IntegrationRuntime) {
    let memory = r.memory();
    assert!(r.finish().is_ok());
    assert_eq!(memory.stats().charged, 0);
}
#[test]
fn final_directory_forward_binding_logical_revisions_and_exact_source() {
    let r = IntegrationRuntime::standalone().unwrap();
    let mut m = r.create_model("batch").unwrap();
    let mut oracle = m.stage();
    oracle.add_input("first", 0.into()).unwrap();
    oracle.add_input("second", 0.into()).unwrap();
    let mut b = m.begin_batch(2, 128).unwrap();
    let a = b.add_formula("a", "b+1").unwrap();
    let target = b.add_input("b", 41.into()).unwrap();
    let stats = r.stats();
    m.apply_batch(b, &Default::default()).unwrap();
    assert_eq!(r.stats().mutations.accepted, stats.mutations.accepted + 1);
    assert_eq!(m.revision(), oracle.revision());
    assert_eq!(m.node_id("b"), Some(target));
    let result = m.capture_local().unwrap().evaluate(&[a]).unwrap();
    assert_eq!(result.values()[&a], Ok(42.into()));
    let snapshot = m.snapshot().unwrap();
    assert_eq!(snapshot.revision(), m.revision());
    let s = m.capture_local().unwrap();
    let before = m.revision();
    let mut bad = m.begin_batch(2, 32).unwrap();
    bad.add_input("temporary", 3.into()).unwrap();
    bad.add_input("b", 4.into()).unwrap();
    assert!(m.apply_batch(bad, &Default::default()).is_err());
    assert_eq!(m.revision(), before);
    assert!(m.node_id("temporary").is_none());
    let mut update = m.begin_batch(1, 0).unwrap();
    update.replace_input(target, 50.into()).unwrap();
    m.apply_batch(update, &Default::default()).unwrap();
    assert_eq!(s.evaluate(&[a]).unwrap().values()[&a], Ok(42.into()));
    drop((m, oracle, result, snapshot, s));
    finish(r);
}
#[test]
fn stale_root_not_just_equal_revision_and_cancelled_batch_are_atomic() {
    let r = IntegrationRuntime::standalone().unwrap();
    let mut a = r.create_model("base").unwrap();
    let x = a.add_input("x", 1.into()).unwrap();
    let mut b = a.stage();
    a.set_input(x, 2.into()).unwrap();
    b.set_input(x, 3.into()).unwrap();
    assert_eq!(a.revision(), b.revision());
    let mut patch = a.begin_batch(1, 0).unwrap();
    patch.replace_input(x, 4.into()).unwrap();
    assert!(b.apply_batch(patch, &Default::default()).is_err());
    let original = b.revision();
    let mut patch = b.begin_batch(1, 16).unwrap();
    patch.add_input("hidden", 8.into()).unwrap();
    let cancel = CancellationToken::default();
    cancel.cancel();
    assert!(b.apply_batch(patch, &cancel).is_err());
    assert_eq!(b.revision(), original);
    assert!(b.node_id("hidden").is_none());
    drop((a, b));
    finish(r);
}
#[test]
fn cycles_and_bad_drafts_stay_editable_but_never_certify_global_validity() {
    let r = IntegrationRuntime::standalone().unwrap();
    let mut m = r.create_model("drafts").unwrap();
    let mut b = m.begin_batch(3, 64).unwrap();
    let a = b.add_formula("a", "b+1").unwrap();
    b.add_formula("b", "a+1").unwrap();
    let x = b.add_input("x", 9.into()).unwrap();
    m.apply_batch(b, &Default::default()).unwrap();
    assert_eq!(
        m.capture_local().unwrap().evaluate(&[x]).unwrap().values()[&x],
        Ok(9.into())
    );
    assert!(m.capture_local().unwrap().evaluate(&[a]).is_err());
    assert!(matches!(
        m.snapshot(),
        Err(SnapshotError::Unorderable { .. })
    ));
    let mut repair = m.begin_batch(1, 0).unwrap();
    repair.replace_input(a, 5.into()).unwrap();
    m.apply_batch(repair, &Default::default()).unwrap();
    assert!(m.snapshot().is_ok());
    drop(m);
    finish(r);
}
#[test]
fn memory_refusal_and_foreign_identity_leave_writer_and_ledger_unchanged() {
    let r = IntegrationRuntime::standalone().unwrap();
    let mut m = r.create_model("admission").unwrap();
    let foreign = r.create_model("foreign").unwrap();
    let before = m.revision();
    let mut patch = m.begin_batch(1, 0).unwrap();
    let mut foreign = foreign;
    let x = foreign.add_input("x", 1.into()).unwrap();
    patch.replace_input(x, 2.into()).unwrap();
    assert!(m.apply_batch(patch, &Default::default()).is_err());
    assert_eq!(m.revision(), before);
    r.flush();
    let memory = r.memory();
    let hold = memory
        .reserve(memory.stats().limit - memory.stats().charged)
        .unwrap();
    assert!(m.begin_batch(1, 1).is_err());
    drop(hold);
    let mut patch = m.begin_batch(1, 32).unwrap();
    patch.add_input("accepted", 3.into()).unwrap();
    m.apply_batch(patch, &Default::default()).unwrap();
    drop((m, foreign));
    finish(r);
}

#[test]
fn preparation_failure_poisoning_prevents_accidental_prefix_installation() {
    let r = IntegrationRuntime::standalone().unwrap();
    let mut m = r.create_model("prefix").unwrap();
    let original = m.revision();
    let mut b = m.begin_batch(1, 32).unwrap();
    b.add_input("private", 3.into()).unwrap();
    assert!(b.add_input("too_many", 4.into()).is_err());
    assert!(m.apply_batch(b, &Default::default()).is_err());
    assert_eq!(m.revision(), original);
    assert!(m.node_id("private").is_none());
    drop(m);
    finish(r);
}
