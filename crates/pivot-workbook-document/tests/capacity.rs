use pivot_core::model::integration::IntegrationRuntime;
use pivot_types::PivotValue;
use pivot_workbook::{Content, Edit, Error, Limits, Position, Rectangle, Workbook, csv};
use pivot_workbook_document::Document;
fn p(row: u32, column: u32) -> Position {
    Position::new(row, column)
}
fn entry(at: Position, source: &str) -> Edit {
    Edit::SetEntry {
        position: at,
        source: source.into(),
    }
}

#[test]
fn small_wb9_remains_readable_and_gains_sparse_row_admission_without_rebinding() {
    let rt = IntegrationRuntime::standalone().unwrap();
    let mut old = Workbook::new_in("small WB9", Limits::baseline(), &rt).unwrap();
    old.apply(
        old.expected(),
        &[entry(p(1, 1), "7"), entry(p(1, 2), "=A1+1")],
    )
    .unwrap();
    let original = old.cell_at(p(1, 1)).unwrap().unwrap().reference;
    let bytes = Document::export_document(&old, old.expected())
        .unwrap()
        .bytes;
    assert_eq!(&bytes[..8], b"PIVOTWB9");
    let (mut restored, _) = Document::open_document(&bytes, Limits::default(), &rt).unwrap();
    assert_eq!(
        restored.cell_at(p(1, 1)).unwrap().unwrap().reference,
        original
    );
    restored
        .apply(
            restored.expected(),
            &[entry(p(65536, 1), "3"), entry(p(65536, 2), "=A65536+$B$1")],
        )
        .unwrap();
    let cap = restored
        .capture_sheet(restored.worksheet(), restored.expected())
        .unwrap();
    assert_eq!(
        cap.evaluate_values(&[cap.cell_at(p(65536, 2)).unwrap()])
            .unwrap()
            .values[0]
            .1,
        Ok(PivotValue::Decimal(11.into()))
    );
    let block = restored
        .copy_block(
            restored.expected(),
            Rectangle {
                first: p(65536, 1),
                last: p(65536, 2),
            },
        )
        .unwrap();
    restored
        .paste_block(
            restored.expected(),
            Rectangle {
                first: p(65535, 1),
                last: p(65535, 1),
            },
            &block.native,
            true,
            &Default::default(),
        )
        .unwrap();
    assert!(matches!(
        restored.cell_at(p(65535, 2)).unwrap().unwrap().content,
        Content::TextFormula(_)
    ));
    let out = Document::export_document(&restored, restored.expected()).unwrap();
    let reopened = Document::open_document(&out.bytes, Limits::default(), &rt)
        .unwrap()
        .0;
    assert_eq!(
        reopened.cell_at(p(1, 1)).unwrap().unwrap().reference,
        original
    );
    drop((reopened, out, cap, restored, old));
    assert!(rt.finish().is_ok());
}

#[test]
fn positioned_and_allocated_limits_are_distinct_and_failure_is_atomic() {
    let rt = IntegrationRuntime::standalone().unwrap();
    let limits = Limits {
        live_cells: 2,
        allocated_cells: 4,
        ..Limits::default()
    };
    let mut book = Workbook::new_in("quotas", limits, &rt).unwrap();
    book.apply(book.expected(), &[entry(p(1, 1), "1"), entry(p(1, 2), "2")])
        .unwrap();
    let first = book.cell_at(p(1, 1)).unwrap().unwrap().reference;
    let before = book.expected();
    assert_eq!(
        book.apply(before, &[entry(p(1, 1), "9"), entry(p(1, 3), "3")]),
        Err(Error::Capacity {
            resource: "positioned cells",
            limit: 2
        })
    );
    assert_eq!(book.expected(), before);
    book.apply(book.expected(), &[Edit::Clear(p(1, 2))])
        .unwrap();
    assert_eq!(book.stats().live_cells, 2);
    book.apply(
        book.expected(),
        &[
            Edit::Retire(p(1, 2)),
            entry(p(1, 2), "3"),
            Edit::Retire(p(1, 2)),
            entry(p(1, 2), "4"),
            Edit::Retire(p(1, 2)),
        ],
    )
    .unwrap();
    let before = book.expected();
    assert_eq!(
        book.apply(before, &[entry(p(1, 2), "5")]),
        Err(Error::Capacity {
            resource: "allocated cell identities",
            limit: 4
        })
    );
    assert_eq!(book.expected(), before);
    assert_eq!(book.stats().allocated_nodes, 4);
    assert_eq!(book.cell_at(p(1, 1)).unwrap().unwrap().reference, first);
    drop(book);
    assert!(rt.finish().is_ok());
}

#[test]
fn aggregate_source_admission_matches_reopen_and_preserves_individual_limits() {
    let rt = IntegrationRuntime::standalone().unwrap();
    let limits = Limits {
        total_source_bytes: 8,
        ..Limits::default()
    };
    let mut book = Workbook::new_in("source admission", limits, &rt).unwrap();
    book.apply(
        book.expected(),
        &[
            Edit::SetText {
                position: p(1, 1),
                text: "1234".into(),
            },
            Edit::SetText {
                position: p(1, 2),
                text: "5678".into(),
            },
        ],
    )
    .unwrap();
    let before = book.expected();
    assert_eq!(
        book.apply(
            before,
            &[Edit::SetText {
                position: p(1, 3),
                text: "9".into()
            }]
        ),
        Err(Error::Capacity {
            resource: "total source bytes",
            limit: 8
        })
    );
    assert_eq!(book.expected(), before);
    let exported = Document::export_document(&book, book.expected()).unwrap();
    let restored = Document::open_document(&exported.bytes, limits, &rt)
        .unwrap()
        .0;
    assert_eq!(restored.stats().source_bytes, 8);
    let table = csv::parse(b"\"001234\",TRUE,\"\",", csv::Policy::default(), limits).unwrap();
    let typed = table
        .workbook(
            csv::Options {
                typed: true,
                ..Default::default()
            },
            limits,
            &rt,
        )
        .unwrap();
    assert_eq!(typed.stats().live_cells, 3);
    drop((typed, restored, exported, book));
    assert!(rt.finish().is_ok());
}

#[test]
fn legacy_rejected_formula_stays_rejected_after_row_admission_grows_and_resave() {
    let rt = IntegrationRuntime::standalone().unwrap();
    let mut old = Workbook::new_in("legacy rejected draft", Limits::baseline(), &rt).unwrap();
    old.apply(old.expected(), &[entry(p(1, 1), "=A1025")])
        .unwrap();
    let original = old.cell_at(p(1, 1)).unwrap().unwrap().reference;
    let bytes = Document::export_document(&old, old.expected())
        .unwrap()
        .bytes;
    let upgraded = Document::open_document(&bytes, Limits::default(), &rt)
        .unwrap()
        .0;
    assert!(
        matches!(upgraded.cell_at(p(1,1)).unwrap().unwrap().content,Content::TextFormula(ref f) if f.diagnostic.is_some())
    );
    let saved = Document::export_document(&upgraded, upgraded.expected()).unwrap();
    let mut reopened = Document::open_document(&saved.bytes, Limits::default(), &rt)
        .unwrap()
        .0;
    assert_eq!(
        reopened.cell_at(p(1, 1)).unwrap().unwrap().reference,
        original
    );
    assert!(
        matches!(reopened.cell_at(p(1,1)).unwrap().unwrap().content,Content::TextFormula(ref f) if f.diagnostic.is_some())
    );
    reopened
        .apply(reopened.expected(), &[entry(p(1, 1), "=A1025")])
        .unwrap();
    assert!(
        matches!(reopened.cell_at(p(1,1)).unwrap().unwrap().content,Content::TextFormula(ref f) if f.diagnostic.is_none())
    );
    drop((reopened, saved, upgraded, old));
    assert!(rt.finish().is_ok());
}
