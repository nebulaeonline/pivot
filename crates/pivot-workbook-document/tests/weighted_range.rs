use pivot_core::model::integration::IntegrationRuntime;
use pivot_types::PivotValue as V;
use pivot_workbook::{structure::*, *};
use pivot_workbook_document::Document;
fn p(r: u32, c: u32) -> Position {
    Position::new(r, c)
}
fn e(r: u32, c: u32, s: &str) -> Edit {
    Edit::SetEntry {
        position: p(r, c),
        source: s.into(),
    }
}
fn result(b: &Workbook) -> std::result::Result<V, CellError> {
    let c = b.capture_local(b.expected()).unwrap();
    c.evaluate_values(&[c.cell_at(p(1, 1)).unwrap()])
        .unwrap()
        .values
        .remove(0)
        .1
}
#[test]
fn native_roundtrip_range_and_deleted_range_authority() {
    let rt = IntegrationRuntime::standalone().unwrap();
    let mut b = Workbook::new_in("Range", Limits::default(), &rt).unwrap();
    b.apply(
        b.expected(),
        &[
            e(1, 2, "3"),
            e(2, 2, "1"),
            e(1, 1, "=ALLOCATEWEIGHTEDRANGE(10,1,2,$B$1:$B$2)"),
        ],
    )
    .unwrap();
    for deleted in [false, true] {
        if deleted {
            b.restructure(
                b.worksheet(),
                b.expected(),
                Change {
                    axis: AxisKind::Columns,
                    action: Action::Delete,
                    start: 2,
                    count: 1,
                },
                &Default::default(),
            )
            .unwrap();
        }
        let saved = Document::export_document(&b, b.expected()).unwrap();
        assert!(saved.bytes.starts_with(b"PIVOTWB14"));
        let (opened, _) = Document::open_document(&saved.bytes, Limits::default(), &rt).unwrap();
        assert_eq!(result(&opened), result(&b));
        if !deleted {
            assert_eq!(result(&opened), Ok(V::Decimal("7.5".parse().unwrap())));
        } else {
            assert!(result(&opened).is_err());
        }
        let mut legacy = saved.bytes.clone();
        legacy[8] = b'2';
        assert!(Document::open_document(&legacy, Limits::default(), &rt).is_err());
    }
}
