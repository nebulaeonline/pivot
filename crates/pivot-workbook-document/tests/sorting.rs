use pivot_core::model::integration::IntegrationRuntime;
use pivot_workbook::{formatting::*, sorting::*, *};
use pivot_workbook_document::Document;
fn p(r: u32, c: u32) -> Position {
    Position::new(r, c)
}
fn e(r: u32, c: u32, text: &str) -> Edit {
    Edit::SetEntry {
        position: p(r, c),
        source: text.into(),
    }
}
#[test]
fn wb11_and_cb5_preserve_sorted_identities_ranges_exact_strings_and_fault_provenance() {
    let rt = IntegrationRuntime::standalone().unwrap();
    let mut b = Workbook::new_in("sorting persistence", Limits::default(), &rt).unwrap();
    let sheet = b.worksheet();
    b.apply(
        b.expected(),
        &[
            e(1, 1, "2"),
            e(2, 1, "1"),
            e(1, 2, "'001234"),
            e(2, 2, "'000042"),
            e(1, 3, "=A1*2"),
            e(2, 3, "=SUM(A1:A1)"),
            e(1, 5, "=A1"),
            e(2, 5, "=A10"),
        ],
    )
    .unwrap();
    b.restructure(
        sheet,
        b.expected(),
        structure::Change {
            axis: structure::AxisKind::Rows,
            action: structure::Action::Delete,
            start: 10,
            count: 1,
        },
        &Default::default(),
    )
    .unwrap();
    b.format_sheet(
        sheet,
        b.expected(),
        Rectangle {
            first: p(1, 1),
            last: p(1, 2),
        },
        StylePatch {
            bold: Field::Set(false),
            ..Default::default()
        },
        false,
        &Default::default(),
    )
    .unwrap();
    let original = b.cell_at(p(1, 1)).unwrap().unwrap().reference;
    let fault = b.cell_at(p(2, 5)).unwrap().unwrap();
    b.sort_rows(
        sheet,
        b.expected(),
        &Options {
            rectangle: Rectangle {
                first: p(1, 1),
                last: p(2, 3),
            },
            exclude_header: false,
            keys: vec![Key {
                column: 1,
                descending: false,
            }],
            move_formatting: true,
        },
        &Default::default(),
    )
    .unwrap();
    let bytes = Document::export_document(&b, b.expected()).unwrap().bytes;
    assert!(bytes.starts_with(b"PIVOTWB11"));
    let mut reopened = Document::open_document(&bytes, Limits::default(), &rt)
        .unwrap()
        .0;
    for r in 1..=2 {
        for c in 1..=5 {
            assert_eq!(
                reopened.cell_at(p(r, c)).unwrap(),
                b.cell_at(p(r, c)).unwrap()
            );
            assert_eq!(
                reopened.style_at(sheet, p(r, c)).unwrap(),
                b.style_at(sheet, p(r, c)).unwrap()
            );
        }
    }
    assert_eq!(
        reopened.cell_at(p(2, 1)).unwrap().unwrap().reference,
        original
    );
    assert_eq!(reopened.cell_at(p(2, 5)).unwrap().unwrap(), fault);
    assert_eq!(
        reopened
            .display_formula(reopened.cell_at(p(1, 5)).unwrap().unwrap().reference)
            .unwrap()
            .as_deref(),
        Some("=A2")
    );
    assert_eq!(
        reopened
            .display_formula(reopened.cell_at(p(1, 3)).unwrap().unwrap().reference)
            .unwrap()
            .as_deref(),
        Some("=SUM(A1:A1)")
    );
    let block = reopened
        .copy_sheet(
            sheet,
            reopened.expected(),
            Rectangle {
                first: p(2, 2),
                last: p(2, 2),
            },
        )
        .unwrap();
    assert!(block.native.starts_with(b"PIVOTCB5"));
    reopened
        .paste_sheet(
            sheet,
            reopened.expected(),
            Rectangle {
                first: p(4, 2),
                last: p(4, 2),
            },
            &block.native,
            true,
            &Default::default(),
        )
        .unwrap();
    let cap = reopened.capture_local(reopened.expected()).unwrap();
    assert_eq!(
        cap.evaluate_values(&[cap.cell_at(p(4, 2)).unwrap()])
            .unwrap()
            .values[0]
            .1,
        Ok(pivot_types::PivotValue::String("001234".into()))
    );
    drop(cap);
    drop(reopened);
    drop(b);
    let mem = rt.memory();
    assert!(rt.finish().is_ok());
    assert_eq!(mem.stats().charged, 0);
}
