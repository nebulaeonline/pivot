use pivot_core::model::integration::IntegrationRuntime;
use pivot_workbook::{structure::*, *};
use pivot_workbook_document::Document;
fn p(row: u32, col: u32) -> Position {
    Position::new(row, col)
}
fn e(row: u32, col: u32, text: &str) -> Edit {
    Edit::SetEntry {
        position: p(row, col),
        source: text.into(),
    }
}
#[test]
fn structural_coordinates_fault_provenance_and_rejection_roundtrip_without_rebinding() {
    let rt = IntegrationRuntime::standalone().unwrap();
    let mut b = Workbook::new_in("WB11", Limits::default(), &rt).unwrap();
    b.apply(
        b.expected(),
        &[
            e(2, 1, "7"),
            e(1, 2, "=A2*2"),
            e(1, 3, "=SUM(A2:A4,1)"),
            e(1, 4, "=IF(FALSE,A2,1)"),
            e(1, 5, "=#REF!+1"),
        ],
    )
    .unwrap();
    for action in [Action::Insert, Action::Delete] {
        b.restructure(
            b.worksheet(),
            b.expected(),
            Change {
                axis: AxisKind::Rows,
                action,
                start: 2,
                count: if action == Action::Delete { 6 } else { 3 },
            },
            &Default::default(),
        )
        .unwrap();
        let export = Document::export_document(&b, b.expected()).unwrap();
        assert!(export.bytes.starts_with(b"PIVOTWB12"));
        let mut restored = Document::open_document(&export.bytes, Limits::default(), &rt)
            .unwrap()
            .0;
        assert_eq!(
            restored.cell_at(p(1, 2)).unwrap(),
            b.cell_at(p(1, 2)).unwrap()
        );
        assert_eq!(
            restored
                .display_formula(restored.cell_at(p(1, 3)).unwrap().unwrap().reference)
                .unwrap(),
            b.display_formula(b.cell_at(p(1, 3)).unwrap().unwrap().reference)
                .unwrap()
        );
        if action == Action::Delete {
            restored
                .apply(restored.expected(), &[e(2, 1, "100")])
                .unwrap();
            let c = restored.capture_local(restored.expected()).unwrap();
            assert_eq!(
                c.evaluate_values(&[c.cell_at(p(1, 2)).unwrap()])
                    .unwrap()
                    .values[0]
                    .1,
                Err(CellError::Typed(pivot_core::typed::Error::ReferenceDeleted))
            );
        }
        drop(restored);
    }
    drop(b);
    let m = rt.memory();
    assert!(rt.finish().is_ok());
    assert_eq!(m.stats().charged, 0);
}
