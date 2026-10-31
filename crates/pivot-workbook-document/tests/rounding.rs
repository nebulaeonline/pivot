use pivot_core::model::integration::IntegrationRuntime;
use pivot_types::PivotValue;
use pivot_workbook::*;
use pivot_workbook_document::Document;
#[test]
fn wb9_rounding_instructions_roundtrip_without_format_cut() {
    let runtime = IntegrationRuntime::standalone().unwrap();
    let mut b = Workbook::new_in("rounding", Limits::default(), &runtime).unwrap();
    for (i, source) in [
        "=ROUND(12.55,1)",
        "=ROUNDUP(-12.51,1)",
        "=ROUNDDOWN(-125,-1)",
    ]
    .into_iter()
    .enumerate()
    {
        b.apply(
            b.expected(),
            &[Edit::SetEntry {
                position: Position::new(i as u32 + 1, 1),
                source: source.into(),
            }],
        )
        .unwrap();
    }
    let export = Document::export_document(&b, b.expected()).unwrap();
    assert_eq!(&export.bytes[..9], b"PIVOTWB11");
    let (opened, _) = Document::open_document(&export.bytes, Limits::default(), &runtime).unwrap();
    let capture = opened.capture_local(opened.expected()).unwrap();
    for (i, expected) in ["12.6", "-12.6", "-120"].into_iter().enumerate() {
        let cell = capture.cell_at(Position::new(i as u32 + 1, 1)).unwrap();
        assert_eq!(
            capture.evaluate_values(&[cell]).unwrap().values[0].1,
            Ok(PivotValue::Decimal(expected.parse().unwrap()))
        );
    }
    drop((capture, opened, b));
    assert!(runtime.finish().is_ok());
}
