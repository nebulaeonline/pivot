use pivot_core::model::integration::{CancellationToken, IntegrationRuntime};
use pivot_workbook::*;
fn p(r: u32, c: u32) -> Position {
    Position::new(r, c)
}
fn rect(a: Position, b: Position) -> Rectangle {
    Rectangle { first: a, last: b }
}
fn e(r: u32, c: u32, s: &str) -> Edit {
    Edit::SetEntry {
        position: p(r, c),
        source: s.into(),
    }
}
fn value(b: &Workbook, r: u32, c: u32) -> std::result::Result<String, CellError> {
    let s = b.capture_local(b.expected()).unwrap();
    let id = s.cell_at(p(r, c)).unwrap();
    s.evaluate(&[id]).unwrap().values[0]
        .1
        .clone()
        .map(|v| v.to_string())
}
fn finish(rt: IntegrationRuntime, b: Workbook) {
    drop(b);
    let m = rt.memory();
    assert!(rt.finish().is_ok());
    assert_eq!(m.stats().charged, 0);
}
#[test]
fn all_directions_translate_original_edges_and_repeat_without_identity_churn() {
    let rt = IntegrationRuntime::standalone().unwrap();
    let mut b = Workbook::new_in("fill", Limits::default(), &rt).unwrap();
    for (d, source) in [
        (FillDirection::Down, p(3, 3)),
        (FillDirection::Up, p(5, 3)),
        (FillDirection::Right, p(3, 3)),
        (FillDirection::Left, p(3, 5)),
    ] {
        let range = if matches!(d, FillDirection::Down | FillDirection::Up) {
            rect(p(3, 3), p(5, 3))
        } else {
            rect(p(3, 3), p(3, 5))
        };
        b.apply(b.expected(),&[e(source.row,source.column,"=SUM($A$1:$A$2)+MIN($A$1:$A$2)+MAX($A$1:$A$2)+AVERAGE($A$1:$A$2)+COUNT($A$1:$A$2)"),e(1,1,"2"),e(2,1,"4")]).unwrap();
        let old = b.capture_local(b.expected()).unwrap();
        let src = b.cell_at(source).unwrap().unwrap();
        b.fill(b.expected(), range, d, &Default::default()).unwrap();
        assert_eq!(b.cell_at(source).unwrap().unwrap(), src);
        for r in range.first.row..=range.last.row {
            for c in range.first.column..=range.last.column {
                assert_eq!(value(&b, r, c), Ok("17".into()));
            }
        }
        let before = b.expected();
        let image = b.export_portable(before).unwrap();
        let stats = b.stats();
        assert_eq!(
            b.fill(before, range, d, &Default::default()).unwrap(),
            before
        );
        let after = b.export_portable(before).unwrap();
        assert_eq!(format!("{:?}", image.graph), format!("{:?}", after.graph));
        assert_eq!(b.stats(), stats);
        drop((old, image, after));
    }
    b.apply(b.expected(), &[e(5, 6, "=A5+$B$1")]).unwrap();
    b.fill(
        b.expected(),
        rect(p(3, 6), p(5, 6)),
        FillDirection::Up,
        &Default::default(),
    )
    .unwrap();
    assert!(
        matches!(b.cell_at(p(3,6)).unwrap().unwrap().content,Content::Formula{source,..} if source=="=A3+$B$1")
    );
    finish(rt, b);
}
#[test]
fn blank_fill_preserves_referenced_destination_id_and_numeric_classification() {
    let rt = IntegrationRuntime::standalone().unwrap();
    let mut b = Workbook::new_in("blank", Limits::default(), &rt).unwrap();
    b.apply(
        b.expected(),
        &[
            e(2, 1, "0"),
            e(3, 1, "4"),
            e(1, 3, "=A2"),
            e(2, 3, "=COUNT(A2:A3)"),
            e(3, 3, "=AVERAGE(A2:A3)"),
        ],
    )
    .unwrap();
    let id = b.cell_at(p(2, 1)).unwrap().unwrap().reference;
    let old = b.capture_local(b.expected()).unwrap();
    let count = old.cell_at(p(2, 3)).unwrap();
    b.fill(
        b.expected(),
        rect(p(1, 1), p(2, 1)),
        FillDirection::Down,
        &Default::default(),
    )
    .unwrap();
    assert_eq!(b.cell_at(p(2, 1)).unwrap().unwrap().reference, id);
    assert!(b.cell_at(p(1, 1)).unwrap().is_none());
    assert_eq!(value(&b, 1, 3), Ok("0".into()));
    assert_eq!(value(&b, 2, 3), Ok("1".into()));
    assert_eq!(value(&b, 3, 3), Ok("4".into()));
    assert_eq!(old.evaluate(&[count]).unwrap().values[0].1, Ok(2.into()));
    drop(old);
    let before = b.expected();
    let stats = b.stats();
    b.fill(
        before,
        rect(p(7, 1), p(9, 1)),
        FillDirection::Down,
        &Default::default(),
    )
    .unwrap();
    assert_eq!(b.expected().graph, before.graph);
    assert_eq!(b.stats().allocated_nodes, stats.allocated_nodes);
    let snapshot = b.expected();
    b.fill(
        snapshot,
        rect(p(7, 1), p(9, 1)),
        FillDirection::Down,
        &Default::default(),
    )
    .unwrap();
    assert_eq!(b.expected(), snapshot);
    finish(rt, b);
}
#[test]
fn typed_content_multi_cell_edges_and_moved_targets() {
    let rt = IntegrationRuntime::standalone().unwrap();
    let mut b = Workbook::new_in("content", Limits::default(), &rt).unwrap();
    b.apply(
        b.expected(),
        &[
            e(1, 1, "'001.20"),
            e(1, 2, "=("),
            Edit::SetDecimal {
                position: p(1, 3),
                value: DecimalInput::from_decimal("-0.000".parse().unwrap()),
            },
            e(1, 4, "=H1"),
        ],
    )
    .unwrap();
    b.apply(
        b.expected(),
        &[Edit::Move {
            from: p(1, 8),
            to: p(5, 8),
        }],
    )
    .unwrap();
    b.fill(
        b.expected(),
        rect(p(1, 1), p(3, 4)),
        FillDirection::Down,
        &Default::default(),
    )
    .unwrap();
    for r in 2..=3 {
        assert_eq!(
            b.cell_at(p(r, 1)).unwrap().unwrap().content,
            Content::LiteralText("001.20".into())
        );
        assert!(
            matches!(b.cell_at(p(r,2)).unwrap().unwrap().content,Content::TextFormula(f) if f.original_source=="=("&&f.diagnostic.is_some())
        );
        assert_eq!(
            b.cell_at(p(r, 3)).unwrap().unwrap().content,
            b.cell_at(p(1, 3)).unwrap().unwrap().content
        );
    }
    assert!(
        matches!(b.cell_at(p(3,4)).unwrap().unwrap().content,Content::Formula{source,..} if source=="=H7")
    );
    finish(rt, b);
}
#[test]
fn late_invalid_reference_quota_cycle_stale_and_cancelled_fill_are_atomic() {
    let rt = IntegrationRuntime::standalone().unwrap();
    let mut b = Workbook::new_in("rollback", Limits::default(), &rt).unwrap();
    b.apply(
        b.expected(),
        &[e(1, 1, "8"), e(1, 2, "=C65535"), e(2, 1, "9")],
    )
    .unwrap();
    let before = b.expected();
    let stats = b.stats();
    assert!(
        b.fill(
            before,
            rect(p(1, 1), p(3, 2)),
            FillDirection::Down,
            &Default::default()
        )
        .is_err()
    );
    assert_eq!(b.expected(), before);
    assert_eq!(b.stats(), stats);
    assert_eq!(value(&b, 2, 1), Ok("9".into()));
    assert!(
        b.fill(
            before,
            rect(p(1, 1), p(257, 1)),
            FillDirection::Down,
            &Default::default()
        )
        .is_err()
    );
    let cancel = CancellationToken::default();
    cancel.cancel();
    assert!(
        b.fill(before, rect(p(1, 1), p(2, 1)), FillDirection::Down, &cancel)
            .is_err()
    );
    assert_eq!(
        b.fill(
            before,
            rect(p(1, 1), p(1, 1024)),
            FillDirection::Up,
            &Default::default()
        )
        .unwrap(),
        before
    );
    b.apply(before, &[e(8, 8, "1")]).unwrap();
    assert!(
        b.fill(
            before,
            rect(p(1, 1), p(1, 1)),
            FillDirection::Down,
            &Default::default()
        )
        .is_err()
    );
    b.fill(
        b.expected(),
        rect(p(10, 1), p(265, 1)),
        FillDirection::Down,
        &Default::default(),
    )
    .unwrap();
    b.apply(b.expected(), &[e(1, 4, "=$E$1")]).unwrap();
    let before = b.expected();
    let stats = b.stats();
    assert!(
        b.fill(
            before,
            rect(p(1, 4), p(1, 5)),
            FillDirection::Right,
            &Default::default()
        )
        .is_err()
    );
    assert_eq!(b.expected(), before);
    assert_eq!(b.stats(), stats);
    b.apply(b.expected(), &[e(1, 6, "=SUM($G$1:$G$2)"), e(20, 20, "=(")])
        .unwrap();
    let before = b.expected();
    assert!(
        b.fill(
            before,
            rect(p(1, 6), p(1, 7)),
            FillDirection::Right,
            &Default::default()
        )
        .is_err()
    );
    assert_eq!(b.expected(), before);
    finish(rt, b);
}

#[test]
fn structured_fill_preserves_output_identity_under_rename_and_quota_rollback() {
    let rt = IntegrationRuntime::standalone().unwrap();
    let mut b = Workbook::new_in("structured", Limits::default(), &rt).unwrap();
    b.enable_structured_proof(uuid::Uuid::new_v4(), uuid::Uuid::new_v4())
        .unwrap();
    let source = "=@{InvoiceLines.TotalAmount}+$A$1";
    let occurrences = b.resolve_structured(source).unwrap();
    b.apply_structured(b.expected(), p(1, 2), source, &occurrences)
        .unwrap();
    b.rename_structured_producer(b.expected(), "Renamed")
        .unwrap();
    let binding = b.current_structured_binding();
    b.fill(
        b.expected(),
        rect(p(1, 2), p(3, 2)),
        FillDirection::Down,
        &Default::default(),
    )
    .unwrap();
    assert_eq!(b.current_structured_binding(), binding);
    let before = b.expected();
    rt.flush();
    let usage = b.structured_usage();
    b.fill(
        before,
        rect(p(1, 2), p(3, 2)),
        FillDirection::Down,
        &Default::default(),
    )
    .unwrap();
    assert_eq!(b.expected(), before);
    assert_eq!(b.structured_usage(), usage);
    assert!(
        b.fill(
            before,
            rect(p(1, 2), p(40, 2)),
            FillDirection::Down,
            &Default::default()
        )
        .is_err()
    );
    assert_eq!(b.expected(), before);
    rt.flush();
    assert_eq!(b.structured_usage(), usage);
    finish(rt, b);
}
#[test]
fn late_work_and_live_cell_admission_reject_without_partial_installation() {
    let rt = IntegrationRuntime::standalone().unwrap();
    let mut b = Workbook::new_in("work", Limits::default(), &rt).unwrap();
    b.apply(
        b.expected(),
        &[
            Edit::SetText {
                position: p(1, 1),
                text: "x".repeat(4096),
            },
            e(2, 1, "7"),
        ],
    )
    .unwrap();
    let before = b.expected();
    let stats = b.stats();
    assert!(
        b.fill(
            before,
            rect(p(1, 1), p(40, 1)),
            FillDirection::Down,
            &Default::default()
        )
        .is_err()
    );
    assert_eq!(b.expected(), before);
    assert_eq!(b.stats(), stats);
    assert_eq!(value(&b, 2, 1), Ok("7".into()));
    drop(b);
    let mut b = Workbook::new_in(
        "quota",
        Limits {
            live_cells: 2,
            ..Limits::default()
        },
        &rt,
    )
    .unwrap();
    b.apply(b.expected(), &[e(1, 1, "7")]).unwrap();
    let before = b.expected();
    let stats = b.stats();
    assert!(
        b.fill(
            before,
            rect(p(1, 1), p(3, 1)),
            FillDirection::Down,
            &Default::default()
        )
        .is_err()
    );
    assert_eq!(b.expected(), before);
    assert_eq!(b.stats(), stats);
    assert!(b.cell_at(p(2, 1)).unwrap().is_none());
    finish(rt, b);
}
