use pivot_core::model::integration::IntegrationRuntime;
use pivot_types::PivotValue;
use pivot_workbook::{structure::*, *};
use pivot_workbook_document::Document;
#[test]
fn wb12_weighted_roundtrip_and_structural_blank_anchors() {
    for (axis, action) in [
        (AxisKind::Rows, Action::Insert),
        (AxisKind::Columns, Action::Insert),
        (AxisKind::Rows, Action::Delete),
        (AxisKind::Columns, Action::Delete),
    ] {
        for formula in [
            "=ROUND(E200,2)",
            "=ALLOCATEWEIGHTED(10,1,E200,1,2,3)",
            "=SUM(ALLOCATEWEIGHTED(10,1,28,1,1,1),ALLOCATEWEIGHTED(10,2,28,1,1,1),ALLOCATEWEIGHTED(10,3,28,1,1,1))",
        ] {
            let rt = IntegrationRuntime::standalone().unwrap();
            let mut b = Workbook::new_in("Allocation", Limits::default(), &rt).unwrap();
            b.apply(
                b.expected(),
                &[Edit::SetEntry {
                    position: Position::new(3, 3),
                    source: formula.into(),
                }],
            )
            .unwrap();
            Document::export_document(&b, b.expected()).unwrap();
            b.restructure(
                b.worksheet(),
                b.expected(),
                Change {
                    axis,
                    action,
                    start: 1,
                    count: 1,
                },
                &Default::default(),
            )
            .unwrap();
            let export = Document::export_document(&b, b.expected()).unwrap();
            assert_eq!(&export.bytes[..9], b"PIVOTWB14");
            let (opened, _) =
                Document::open_document(&export.bytes, Limits::default(), &rt).unwrap();
            let c = opened.capture_local(opened.expected()).unwrap();
            let result = c
                .evaluate_values(&[c
                    .cell_at(Position::new(
                        if axis == AxisKind::Rows {
                            if action == Action::Insert { 4 } else { 2 }
                        } else {
                            3
                        },
                        if axis == AxisKind::Columns {
                            if action == Action::Insert { 4 } else { 2 }
                        } else {
                            3
                        },
                    ))
                    .unwrap()])
                .unwrap()
                .values
                .remove(0)
                .1;
            if formula.starts_with("=ROUND") {
                assert_eq!(result, Ok(PivotValue::Decimal(0.into())));
            } else if formula.starts_with("=SUM") {
                assert_eq!(result, Ok(PivotValue::Decimal(10.into())));
            } else {
                assert_eq!(
                    result,
                    Err(CellError::Typed(pivot_core::typed::Error::TypeMismatch))
                );
            }
            drop((c, opened, b));
            let memory = rt.memory();
            assert!(rt.finish().is_ok());
            assert_eq!(memory.stats().charged, 0);
        }
    }
}
