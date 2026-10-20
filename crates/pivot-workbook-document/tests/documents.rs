use pivot_core::model::integration::IntegrationRuntime;
use pivot_workbook::*;
fn p(r: u32, c: u32) -> Position {
    Position::new(r, c)
}
fn number(position: Position, value: &str) -> Edit {
    Edit::SetDecimal {
        position,
        value: DecimalInput::from_decimal(value.parse().unwrap()),
    }
}
#[test]
fn independent_restore_preserves_projection_bound_graph_and_retired_ledgers() {
    let runtime = IntegrationRuntime::standalone().unwrap();
    let mut book = Workbook::new_in("Saved", Limits::default(), &runtime).unwrap();
    book.apply(
        book.expected(),
        &[
            number(p(1, 1), "1.2300"),
            Edit::SetFormulaText {
                position: p(1, 2),
                source: " =A1+2 ".into(),
            },
            Edit::SetFormulaText {
                position: p(1, 3),
                source: "=(".into(),
            },
            number(p(5, 5), "9"),
        ],
    )
    .unwrap();
    book.apply(
        book.expected(),
        &[
            Edit::Retire(p(5, 5)),
            Edit::Move {
                from: p(1, 1),
                to: p(2, 1),
            },
            number(p(1, 1), "100"),
        ],
    )
    .unwrap();
    let stable = book.cell_at(p(2, 1)).unwrap().unwrap();
    let formula = book.cell_at(p(1, 2)).unwrap().unwrap();
    let axes = book.axes_at(p(2, 1)).unwrap();
    let export =
        pivot_workbook_document::Document::export_document(&book, book.expected()).unwrap();
    book.apply(book.expected(), &[number(p(2, 1), "8")])
        .unwrap();
    let (mut reopened, checkpoint) = pivot_workbook_document::Document::open_document(
        &export.bytes,
        Limits::default(),
        &runtime,
    )
    .unwrap();
    assert_eq!(checkpoint, export.checkpoint);
    assert_ne!(book.expected().session, reopened.expected().session);
    assert_eq!(reopened.cell_at(p(2, 1)).unwrap().unwrap(), stable);
    assert_eq!(reopened.cell_at(p(1, 2)).unwrap().unwrap(), formula);
    assert_eq!(axes, reopened.axes_at(p(2, 1)).unwrap());
    assert_eq!(reopened.stats().allocated_nodes, 5);
    let capture = reopened.capture_local(reopened.expected()).unwrap();
    assert_eq!(
        capture.evaluate(&[formula.reference]).unwrap().values[0]
            .1
            .as_ref()
            .unwrap()
            .to_string(),
        "3.23"
    );
    drop(capture);
    reopened
        .apply(
            reopened.expected(),
            &[
                number(p(5, 5), "4"),
                Edit::SetFormulaText {
                    position: p(1, 3),
                    source: "=B1*2".into(),
                },
            ],
        )
        .unwrap();
    assert_eq!(reopened.stats().allocated_nodes, 6);
    for len in [0, 7, 47, export.bytes.len() - 1] {
        assert!(
            pivot_workbook_document::Document::open_document(
                &export.bytes[..len],
                Limits::default(),
                &runtime
            )
            .is_err()
        );
    }
    let mut corrupt = export.bytes.clone();
    corrupt[50] ^= 1;
    assert!(
        pivot_workbook_document::Document::open_document(&corrupt, Limits::default(), &runtime)
            .is_err()
    );
    // Recompute integrity after malformed logical metadata: validation must still reject it.
    use sha2::{Digest, Sha256};
    for (offset, length) in [(48, 16), (80, 8), (104, 8)] {
        let mut bad = export.bytes.clone();
        bad[offset..offset + length].fill(0);
        let digest = Sha256::digest(&bad[48..]);
        bad[16..48].copy_from_slice(&digest);
        assert!(
            pivot_workbook_document::Document::open_document(&bad, Limits::default(), &runtime)
                .is_err()
        );
    }
    let mut version = export.bytes.clone();
    version[7] = b'2';
    assert!(
        pivot_workbook_document::Document::open_document(&version, Limits::default(), &runtime)
            .is_err()
    );
    drop(book);
    drop(reopened);
    let memory = runtime.memory();
    assert!(runtime.finish().is_ok());
    assert_eq!(memory.stats().charged, 0);
}

#[test]
fn referenced_retirement_and_out_of_bounds_drafts_roundtrip() {
    let runtime = IntegrationRuntime::standalone().unwrap();
    let mut book = Workbook::new_in("Retired", Limits::default(), &runtime).unwrap();
    book.apply(
        book.expected(),
        &[
            number(p(1, 1), "2"),
            Edit::SetFormulaText {
                position: p(1, 2),
                source: "=A1+1".into(),
            },
            Edit::SetFormulaText {
                position: p(1, 3),
                source: "=A99999".into(),
            },
        ],
    )
    .unwrap();
    book.apply(book.expected(), &[Edit::Retire(p(1, 1))])
        .unwrap();
    let saved = pivot_workbook_document::Document::export_document(&book, book.expected()).unwrap();
    let (restored, _) =
        pivot_workbook_document::Document::open_document(&saved.bytes, Limits::default(), &runtime)
            .unwrap();
    assert_eq!(
        restored.cell_at(p(1, 2)).unwrap(),
        book.cell_at(p(1, 2)).unwrap()
    );
    assert_eq!(
        restored.cell_at(p(1, 3)).unwrap(),
        book.cell_at(p(1, 3)).unwrap()
    );
    drop(restored);
    drop(book);
    assert!(runtime.finish().is_ok());
}
