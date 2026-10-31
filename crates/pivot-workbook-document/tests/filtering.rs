use pivot_core::model::integration::IntegrationRuntime;
use pivot_workbook::{filtering::*, *};
use pivot_workbook_document::Document;
#[test]
fn wb12_preserves_typed_definition_needs_review_and_exact_text_while_wb11_is_refused() {
    let rt = IntegrationRuntime::standalone().unwrap();
    let mut b = Workbook::new_in("persist filter", Limits::default(), &rt).unwrap();
    let sheet = b.worksheet();
    let p = |r| Position::new(r, 1);
    b.apply(
        b.expected(),
        &[Edit::SetEntry {
            position: p(1),
            source: "'001234".into(),
        }],
    )
    .unwrap();
    let d = Definition {
        rectangle: Rectangle {
            first: p(1),
            last: p(10),
        },
        exclude_header: false,
        needs_review: false,
        conditions: vec![Condition {
            column: 1,
            either: false,
            predicates: vec![Predicate {
                operator: Operator::Equal,
                literal: Some(Literal::Text("001234".into())),
            }],
        }],
    };
    b.set_filter(sheet, b.expected(), Some(&d), &Default::default())
        .unwrap();
    let id = b.cell_at(p(1)).unwrap();
    let saved = Document::export_document(&b, b.expected()).unwrap();
    assert!(saved.bytes.starts_with(b"PIVOTWB12"));
    let reopened = Document::open_document(&saved.bytes, Limits::default(), &rt)
        .unwrap()
        .0;
    assert_eq!(reopened.filter(sheet).unwrap(), Some(&d));
    assert_eq!(reopened.cell_at(p(1)).unwrap(), id);
    let map = reopened
        .filter_visibility(sheet, reopened.expected(), &Default::default())
        .unwrap();
    assert_eq!(map.rows()[0], 1);
    assert_eq!(map.rows()[1], 11);
    drop(map);
    let mut obsolete = saved.bytes.clone();
    obsolete[..9].copy_from_slice(b"PIVOTWB11");
    assert!(Document::open_document(&obsolete, Limits::default(), &rt).is_err());
    let mut image = b.export_portable(b.expected()).unwrap();
    image.filters[0].1.needs_review = true;
    let reviewed = Workbook::restore_portable(image, Limits::default(), &rt).unwrap();
    let saved2 = Document::export_document(&reviewed, reviewed.expected()).unwrap();
    let reopen2 = Document::open_document(&saved2.bytes, Limits::default(), &rt)
        .unwrap()
        .0;
    assert!(reopen2.filter(sheet).unwrap().unwrap().needs_review);
    drop((b, reopened, saved, reviewed, saved2, reopen2));
    let memory = rt.memory();
    assert!(rt.finish().is_ok());
    assert_eq!(memory.stats().charged, 0);
}
