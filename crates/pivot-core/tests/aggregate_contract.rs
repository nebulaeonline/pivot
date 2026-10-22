use pivot_core::{
    evaluation::EvaluationError,
    expression::AggregateKind as K,
    model::{
        SnapshotDefinition, WorkingModel,
        integration::{CancellationToken, IntegrationRuntime},
        selection::{Rectangle, SelectionId, SpaceId},
        transfer::{self, ResolvedToken},
    },
};
#[test]
fn blank_snapshot_and_frozen_exports_preserve_the_boundary() {
    let runtime = IntegrationRuntime::standalone().unwrap();
    let mut model = WorkingModel::new_in("blank", &runtime).unwrap();
    let node = model.add_input("blank", 0.into()).unwrap();
    let mut b = model.begin_batch(1, 128).unwrap();
    b.set_blank(node).unwrap();
    model.apply_batch(b, &CancellationToken::default()).unwrap();
    let snapshot = model.snapshot().unwrap();
    assert!(matches!(
        snapshot.node(node).unwrap().definition(),
        SnapshotDefinition::Blank
    ));
    let snapshot = pivot_core::evaluation::EvaluationContext::new(snapshot);
    assert_eq!(
        snapshot.evaluate(&[node]).unwrap().values()[&node],
        Ok(0.into())
    );
    assert!(transfer::export(&model, &[node], &[]).is_err());
    assert!(transfer::export_snapshot(snapshot.snapshot(), &[node], &[]).is_err());
    drop((snapshot, model));
    let memory = runtime.memory();
    assert!(runtime.finish().is_ok());
    assert_eq!(memory.stats().charged, 0);
}
#[test]
fn direct_blank_point_classification_definition_invalidation_and_transfer() {
    let runtime = IntegrationRuntime::standalone().unwrap();
    let mut model = WorkingModel::new_in("aggregate points", &runtime).unwrap();
    let mut b = model.begin_batch(24, 4096).unwrap();
    let blank = b.add_input("blank", 0.into()).unwrap();
    b.set_blank(blank).unwrap();
    let zero = b.add_input("zero", 0.into()).unwrap();
    let text = b.add_input("text", 0.into()).unwrap();
    b.set_nonnumeric(text).unwrap();
    let count = b.add_input("count", 0.into()).unwrap();
    let min = b.add_input("min", 0.into()).unwrap();
    let space = SpaceId([21; 16]);
    let id = SelectionId([22; 16]);
    b.define_selection(
        id,
        Rectangle {
            space,
            first_row: 1,
            last_row: 10,
            first_column: 1,
            last_column: 1,
        },
    )
    .unwrap();
    for (row, node) in [(1, blank), (2, zero), (3, text)] {
        b.place(space, row, 1, Some(node)).unwrap();
    }
    b.set_resolved(count, "COUNT", vec![ResolvedToken::Aggregate(K::Count, id)])
        .unwrap();
    b.set_resolved(min, "MIN", vec![ResolvedToken::Aggregate(K::Min, id)])
        .unwrap();
    model.apply_batch(b, &CancellationToken::default()).unwrap();
    let old = model.capture_local().unwrap();
    assert_eq!(
        old.evaluate(&[count]).unwrap().values()[&count],
        Ok(1.into())
    );
    let image = model
        .export_selections(&[blank, zero, text, count, min], &[])
        .unwrap();
    assert_eq!(image.blanks, vec![0]);
    let restored = runtime
        .import_selections(image, &CancellationToken::default())
        .unwrap();
    assert_eq!(
        restored
            .model
            .capture_local()
            .unwrap()
            .evaluate(&[restored.nodes[3]])
            .unwrap()
            .values()[&restored.nodes[3]],
        Ok(1.into())
    );
    drop(restored);
    let mut b = model.begin_batch(1, 128).unwrap();
    b.replace_input(blank, 0.into()).unwrap();
    model.apply_batch(b, &CancellationToken::default()).unwrap();
    assert_eq!(
        model
            .capture_local()
            .unwrap()
            .evaluate(&[count])
            .unwrap()
            .values()[&count],
        Ok(2.into())
    );
    assert_eq!(
        old.evaluate(&[count]).unwrap().values()[&count],
        Ok(1.into())
    );
    let mut b = model.begin_batch(2, 256).unwrap();
    b.place(space, 1, 1, None).unwrap();
    b.place(space, 2, 1, None).unwrap();
    model.apply_batch(b, &CancellationToken::default()).unwrap();
    let c = model.capture_local().unwrap();
    let r = c.evaluate(&[count, min]).unwrap();
    assert_eq!(r.values()[&count], Ok(0.into()));
    assert!(matches!(
        r.values()[&min],
        Err(EvaluationError::NoNumericValues {
            operation: K::Min,
            ..
        })
    ));
    drop((r, c, old, model));
    let memory = runtime.memory();
    assert!(runtime.finish().is_ok());
    assert_eq!(memory.stats().charged, 0);
}
