use pivot_core::model::{
    WorkingModel,
    integration::{CancellationToken, IntegrationRuntime},
    selection::{Dependency, Rectangle, SelectionId, SpaceId},
    transfer::ResolvedToken,
};
fn rectangle() -> Rectangle {
    Rectangle {
        space: SpaceId([1; 16]),
        first_row: 1,
        last_row: 1024,
        first_column: 1,
        last_column: 1024,
    }
}
#[test]
fn canonical_selection_closure_cycles_and_immutable_membership() {
    let runtime = IntegrationRuntime::standalone().unwrap();
    let mut m = WorkingModel::new_in("Ranges", &runtime).unwrap();
    let mut batch = m.begin_batch(10, 4096).unwrap();
    let a = batch.add_input("A", "2".parse().unwrap()).unwrap();
    let b = batch.add_input("B", "3".parse().unwrap()).unwrap();
    let sum = batch.add_input("Sum", "0".parse().unwrap()).unwrap();
    let id = SelectionId([2; 16]);
    batch.define_selection(id, rectangle()).unwrap();
    batch.place(rectangle().space, 2, 2, Some(a)).unwrap();
    batch.place(rectangle().space, 1000, 1000, Some(b)).unwrap();
    batch
        .set_resolved(sum, "SUM", vec![ResolvedToken::Sum(id)])
        .unwrap();
    m.apply_batch(batch, &CancellationToken::default()).unwrap();
    assert_eq!(
        m.typed_dependencies(sum).unwrap(),
        vec![Dependency::Selection(id)]
    );
    let image = m.export_selections(&[a, b, sum], &[]).unwrap();
    let mut malformed = image.clone();
    malformed.identities.clear();
    assert!(
        runtime
            .import_selections(malformed, &CancellationToken::default())
            .is_err()
    );
    let restored = runtime
        .import_selections(image, &CancellationToken::default())
        .unwrap();
    let restored_sum = restored.nodes[2];
    assert_eq!(
        *restored
            .model
            .capture_local()
            .unwrap()
            .evaluate(&[restored_sum])
            .unwrap()
            .values()[&restored_sum]
            .as_ref()
            .unwrap(),
        "5".parse().unwrap()
    );
    drop(restored);
    let s = m.capture_local().unwrap();
    assert_eq!(
        *s.evaluate(&[sum]).unwrap().values()[&sum].as_ref().unwrap(),
        "5".parse().unwrap()
    );
    let mut batch = m.begin_batch(2, 4096).unwrap();
    batch.place(rectangle().space, 2, 2, None).unwrap();
    m.apply_batch(batch, &CancellationToken::default()).unwrap();
    assert_eq!(
        *m.capture_local()
            .unwrap()
            .evaluate(&[sum])
            .unwrap()
            .values()[&sum]
            .as_ref()
            .unwrap(),
        "3".parse().unwrap()
    );
    assert_eq!(
        *s.evaluate(&[sum]).unwrap().values()[&sum].as_ref().unwrap(),
        "5".parse().unwrap()
    );
    let metrics = m.selection_metrics();
    assert_eq!(metrics.points, 1);
    assert_eq!(metrics.selections, 1);
    assert!(metrics.visited_index_nodes < 100);
    let mut batch = m.begin_batch(1, 4096).unwrap();
    batch.place(rectangle().space, 1, 1, Some(sum)).unwrap();
    m.apply_batch(batch, &CancellationToken::default()).unwrap();
    assert!(m.capture_calculation().is_err());
    assert!(m.capture_local().unwrap().evaluate(&[sum]).is_err());
    assert!(pivot_core::model::transfer::export(&m, &[a, b, sum], &[]).is_err());
    drop(s);
    drop(m);
    let memory = runtime.memory();
    assert!(runtime.finish().is_ok());
    assert_eq!(memory.stats().charged, 0);
}

#[test]
fn overlapping_sparse_rectangles_match_independent_integer_oracle() {
    let runtime = IntegrationRuntime::standalone().unwrap();
    let mut m = WorkingModel::new_in("Overlap", &runtime).unwrap();
    let space = SpaceId([7; 16]);
    let mut batch = m.begin_batch(320, 65536).unwrap();
    let mut points = Vec::new();
    for i in 0..80u32 {
        let row = 1 + (i * 13) % 1024;
        let column = 1 + (i * 7) % 1024;
        let value = i as i64 - 40;
        let node = batch
            .add_input(format!("P{i}"), value.to_string().parse().unwrap())
            .unwrap();
        batch.place(space, row, column, Some(node)).unwrap();
        points.push((row, column, value, node));
    }
    let mut sums = Vec::new();
    for i in 0..40u32 {
        let rect = Rectangle {
            space,
            first_row: 1 + i * 9,
            last_row: 500 + i * 9,
            first_column: 1 + i * 3,
            last_column: 400 + i * 5,
        };
        let id = SelectionId((i as u128 + 1).to_le_bytes());
        batch.define_selection(id, rect).unwrap();
        let node = batch
            .add_input(format!("S{i}"), "0".parse().unwrap())
            .unwrap();
        batch
            .set_resolved(node, "SUM", vec![ResolvedToken::Sum(id)])
            .unwrap();
        let expected: i64 = points
            .iter()
            .filter(|(r, c, _, _)| {
                *r >= rect.first_row
                    && *r <= rect.last_row
                    && *c >= rect.first_column
                    && *c <= rect.last_column
            })
            .map(|(_, _, v, _)| *v)
            .sum();
        sums.push((node, expected, rect));
    }
    m.apply_batch(batch, &CancellationToken::default()).unwrap();
    let outputs = sums.iter().map(|(n, _, _)| *n).collect::<Vec<_>>();
    let captured = m.capture_local().unwrap();
    let values = captured.evaluate(&outputs).unwrap();
    for (node, expected, _) in &sums {
        assert_eq!(
            *values.values()[node].as_ref().unwrap(),
            expected.to_string().parse().unwrap()
        );
    }
    let (row, column, _, _) = points[20];
    let mut batch = m.begin_batch(1, 4096).unwrap();
    batch.place(space, row, column, None).unwrap();
    m.apply_batch(batch, &CancellationToken::default()).unwrap();
    let invalidated = m.invalidated_selection_nodes();
    for (sum, _, r) in sums {
        assert_eq!(
            invalidated.contains(&sum),
            row >= r.first_row
                && row <= r.last_row
                && column >= r.first_column
                && column <= r.last_column
        );
    }
    assert_eq!(m.selection_metrics().points, 79);
    drop(values);
    drop(captured);
    drop(m);
    let memory = runtime.memory();
    assert!(runtime.finish().is_ok());
    assert_eq!(memory.stats().charged, 0);
}
