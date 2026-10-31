use pivot_core::model::integration::IntegrationRuntime;
use pivot_types::PivotValue as V;
use pivot_workbook::*;
use pivot_workbook_document::Document;
#[test]
fn wb8_preserves_literal_and_calculated_kinds_and_demand_semantics() {
    let rt = IntegrationRuntime::standalone().unwrap();
    let mut b = Workbook::new_in("types", Limits::default(), &rt).unwrap();
    for (r, source) in [
        "TRUE",
        "hello",
        "'",
        "=IF(A1,B1&\"!\",1/0)",
        "=COUNT(A1:B3)",
    ]
    .into_iter()
    .enumerate()
    {
        b.apply(
            b.expected(),
            &[Edit::SetEntry {
                position: Position::new(r as u32 + 1, 1),
                source: source.into(),
            }],
        )
        .unwrap();
    }
    let bytes = Document::export_document(&b, b.expected()).unwrap().bytes;
    assert_eq!(&bytes[..9], b"PIVOTWB11");
    let (opened, _) = Document::open_document(&bytes, Limits::default(), &rt).unwrap();
    let cap = opened.capture_local(opened.expected()).unwrap();
    let refs = [1, 2, 3].map(|r| cap.cell_at(Position::new(r, 1)).unwrap());
    let v = cap.evaluate_values(&refs).unwrap();
    assert_eq!(v.values[0].1, Ok(V::Boolean(true)));
    assert_eq!(v.values[1].1, Ok(V::String("hello".into())));
    assert_eq!(v.values[2].1, Ok(V::String(String::new())));
    drop((cap, opened, b));
    assert!(rt.finish().is_ok());
}
