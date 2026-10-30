use pivot_core::model::integration::IntegrationRuntime;
use pivot_workbook::*;
use pivot_workbook_document::Document;
fn p(r: u32, c: u32) -> Position {
    Position::new(r, c)
}
fn entry(r: u32, c: u32, s: &str) -> Edit {
    Edit::SetEntry {
        position: p(r, c),
        source: s.into(),
    }
}
fn values(b: &Workbook) -> Vec<String> {
    let s = b.capture_local(b.expected()).unwrap();
    let cells = [s.cell_at(p(1, 4)).unwrap(), s.cell_at(p(2, 4)).unwrap()];
    s.evaluate(&cells)
        .unwrap()
        .values
        .into_iter()
        .map(|(_, v)| v.unwrap().to_string())
        .collect()
}
#[test]
fn wb6_preserves_mixed_and_scalar_only_calls_and_stable_bindings() {
    let rt = IntegrationRuntime::standalone().unwrap();
    let mut b = Workbook::new_in("save arguments", Limits::default(), &rt).unwrap();
    b.apply(
        b.expected(),
        &[
            entry(1, 1, "2"),
            entry(2, 1, "4"),
            entry(1, 4, "=SUM(A1:A2,MAX(A1,3),7)"),
            entry(2, 4, "=AVERAGE(2,4,6)"),
        ],
    )
    .unwrap();
    let original = b.cell_at(p(1, 1)).unwrap().unwrap().reference;
    b.apply(
        b.expected(),
        &[
            Edit::Move {
                from: p(1, 1),
                to: p(3, 1),
            },
            entry(1, 1, "100"),
        ],
    )
    .unwrap();
    // The range sees current occupants (100+4); scalar A1 still means the old cell (2).
    assert_eq!(values(&b), vec!["114", "4"]);
    let export = Document::export_document(&b, b.expected()).unwrap();
    assert_eq!(&export.bytes[..9], b"PIVOTWB10");
    let (mut restored, checkpoint) =
        Document::open_document(&export.bytes, Limits::default(), &rt).unwrap();
    assert_eq!(checkpoint, export.checkpoint);
    assert_eq!(
        restored.cell_at(p(3, 1)).unwrap().unwrap().reference,
        original
    );
    assert_eq!(values(&restored), vec!["114", "4"]);
    restored
        .apply(restored.expected(), &[entry(3, 1, "20")])
        .unwrap();
    assert_eq!(values(&restored), vec!["131", "4"]);
    for version in b'1'..=b'6' {
        let mut obsolete = export.bytes.clone();
        obsolete.remove(8);
        obsolete[7] = version;
        let error = Document::open_document(&obsolete, Limits::default(), &rt)
            .err()
            .unwrap()
            .to_string();
        assert!(
            error.contains("unsupported native workbook version"),
            "{error}"
        );
    }
    drop((restored, b));
    let memory = rt.memory();
    assert!(rt.finish().is_ok());
    assert_eq!(memory.stats().charged, 0);
}
