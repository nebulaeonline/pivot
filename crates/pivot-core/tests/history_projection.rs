#![cfg(feature = "history-experiment")]
use pivot_core::model::{
    WorkingModel,
    integration::{CancellationToken, IntegrationRuntime},
};
#[test]
fn document_slots_are_exhaustive_unique_local_and_do_not_rewrite_live_ids() {
    let rt = IntegrationRuntime::standalone().unwrap();
    let mut m = WorkingModel::new_in("projection", &rt).unwrap();
    let a = m.add_input("a", 3.into()).unwrap();
    let b = m.add_computed("b", "a+1").unwrap();
    let old = m.stage();
    let future = m.add_input("future", 9.into()).unwrap();
    let historical = m.history_activate(&old, &Default::default()).unwrap();
    let guard = historical.revision();
    let ledger = historical.history_metrics().unwrap();
    let capture = historical.capture_calculation().unwrap();
    let mut other = WorkingModel::new_in("foreign", &rt).unwrap();
    let foreign = other.add_input("foreign", 0.into()).unwrap();
    for ids in [vec![a], vec![a, a, b], vec![a, b, foreign]] {
        assert!(
            historical
                .project_document_definitions(&ids, &Default::default())
                .is_err()
        );
    }
    assert!(historical.export_selections(&[a, b], &[]).is_err());
    let image = historical
        .project_document_definitions(&[b, a], &Default::default())
        .unwrap();
    assert_eq!(image.scalar_skeleton.node_slots, 2);
    let imported = rt.import_selections(image, &Default::default()).unwrap();
    assert_eq!(
        imported
            .model
            .capture_calculation()
            .unwrap()
            .evaluate(&[imported.nodes[0]])
            .unwrap()
            .values()[&imported.nodes[0]],
        Ok(4.into())
    );
    let cancel = CancellationToken::default();
    cancel.cancel();
    assert!(
        historical
            .project_document_definitions(&[a, b], &cancel)
            .is_err()
    );
    assert_eq!(historical.revision(), guard);
    assert_eq!(
        format!("{ledger:?}"),
        format!("{:?}", historical.history_metrics().unwrap())
    );
    assert_eq!(capture.evaluate(&[b]).unwrap().values()[&b], Ok(4.into()));
    assert_eq!(
        m.capture_calculation()
            .unwrap()
            .evaluate(&[future])
            .unwrap()
            .values()[&future],
        Ok(9.into())
    );
    drop((imported, capture, historical, old, m, other));
    rt.flush();
    let memory = rt.memory();
    assert!(rt.finish().is_ok());
    assert_eq!(memory.stats().charged, 0);
}
