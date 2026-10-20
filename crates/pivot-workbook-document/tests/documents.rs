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
    version[7] = b'4';
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

#[test]
fn sum_document_keeps_bound_geometry_identity_and_member_affinity() {
    let runtime = IntegrationRuntime::standalone().unwrap();
    let mut book = Workbook::new_in("SUM", Limits::default(), &runtime).unwrap();
    book.apply(
        book.expected(),
        &[
            number(p(1, 1), "10"),
            number(p(2, 1), "20"),
            Edit::SetFormulaText {
                position: p(1, 4),
                source: "=A1+SUM(A3:A1)".into(),
            },
        ],
    )
    .unwrap();
    book.apply(
        book.expected(),
        &[
            Edit::Move {
                from: p(1, 1),
                to: p(5, 1),
            },
            number(p(1, 1), "7"),
        ],
    )
    .unwrap();
    let formula = book.cell_at(p(1, 4)).unwrap().unwrap();
    let bytes = pivot_workbook_document::Document::export_document(&book, book.expected()).unwrap();
    assert_eq!(&bytes.bytes[..8], b"PIVOTWB3");
    let (mut restored, _) =
        pivot_workbook_document::Document::open_document(&bytes.bytes, Limits::default(), &runtime)
            .unwrap();
    assert_eq!(restored.cell_at(p(1, 4)).unwrap().unwrap(), formula);
    let s = restored.capture_local(restored.expected()).unwrap();
    let output = s.cell_at(p(1, 4)).unwrap();
    assert_eq!(
        s.evaluate(&[output]).unwrap().values[0]
            .1
            .as_ref()
            .unwrap()
            .to_string(),
        "37"
    );
    restored
        .apply(restored.expected(), &[number(p(3, 1), "4")])
        .unwrap();
    let t = restored.capture_local(restored.expected()).unwrap();
    assert_eq!(
        t.evaluate(&[output]).unwrap().values[0]
            .1
            .as_ref()
            .unwrap()
            .to_string(),
        "41"
    );
    assert_eq!(
        s.evaluate(&[output]).unwrap().values[0]
            .1
            .as_ref()
            .unwrap()
            .to_string(),
        "37"
    );
    drop(s);
    drop(t);
    drop(book);
    drop(restored);
    let memory = runtime.memory();
    assert!(runtime.finish().is_ok());
    assert_eq!(memory.stats().charged, 0);
}
#[test]
fn wb1_is_an_explicit_unsupported_version_not_a_migration_reader() {
    let text = include_str!("fixtures/wb1.hex").trim();
    let bytes = (0..text.len())
        .step_by(2)
        .map(|i| u8::from_str_radix(&text[i..i + 2], 16).unwrap())
        .collect::<Vec<_>>();
    let runtime = IntegrationRuntime::standalone().unwrap();
    let error =
        match pivot_workbook_document::Document::open_document(&bytes, Limits::default(), &runtime)
        {
            Err(e) => e,
            Ok(_) => panic!("WB1 must be unsupported"),
        };
    assert!(
        error
            .to_string()
            .contains("unsupported native workbook version WB1")
    );
    assert!(runtime.finish().is_ok());
}

#[test]
fn wb3_rejects_checksum_valid_unknown_policy_and_selection_corruption() {
    use sha2::{Digest, Sha256};
    let runtime = IntegrationRuntime::standalone().unwrap();
    let mut book = Workbook::new_in("Corruption", Limits::default(), &runtime).unwrap();
    book.apply(
        book.expected(),
        &[
            number(p(1, 1), "3"),
            Edit::SetFormulaText {
                position: p(1, 3),
                source: "=SUM(A1:A2)".into(),
            },
        ],
    )
    .unwrap();
    let bytes = pivot_workbook_document::Document::export_document(&book, book.expected())
        .unwrap()
        .bytes;
    // WB3 fixed worksheet/axis/state header, then length-prefixed graph.
    let graph_length = u64::from_le_bytes(bytes[120..128].try_into().unwrap()) as usize;
    let policy = 128 + graph_length;
    for (at, value) in [(policy, 2u64), (policy + 8, 8193), (policy + 8 + 8 + 16, 9)] {
        let mut corrupt = bytes.clone();
        corrupt[at..at + 8].copy_from_slice(&value.to_le_bytes());
        let digest = Sha256::digest(&corrupt[48..]);
        corrupt[16..48].copy_from_slice(&digest);
        assert!(
            pivot_workbook_document::Document::open_document(&corrupt, Limits::default(), &runtime)
                .is_err()
        );
    }
    drop(book);
    let memory = runtime.memory();
    assert!(runtime.finish().is_ok());
    assert_eq!(memory.stats().charged, 0);
}

#[test]
fn wb2_is_explicitly_rejected() {
    let text = include_str!("fixtures/wb2.hex").trim();
    let bytes = (0..text.len())
        .step_by(2)
        .map(|i| u8::from_str_radix(&text[i..i + 2], 16).unwrap())
        .collect::<Vec<_>>();
    let runtime = IntegrationRuntime::standalone().unwrap();
    let error =
        match pivot_workbook_document::Document::open_document(&bytes, Limits::default(), &runtime)
        {
            Err(error) => error,
            Ok(_) => panic!("WB2 must be unsupported"),
        };
    assert!(
        error
            .to_string()
            .contains("unsupported native workbook version WB2")
    );
    assert!(runtime.finish().is_ok());
}
