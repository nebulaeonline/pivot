use pivot_core::model::integration::{CancellationToken, IntegrationRuntime};
use pivot_workbook::*;
fn p(r: u32, c: u32) -> Position {
    Position::new(r, c)
}
fn entry(r: u32, c: u32, s: &str) -> Edit {
    Edit::SetEntry {
        position: p(r, c),
        source: s.into(),
    }
}
fn run(test: impl FnOnce(&mut Workbook)) {
    let runtime = IntegrationRuntime::standalone().unwrap();
    let mut book = Workbook::new_in("Text", Limits::default(), &runtime).unwrap();
    test(&mut book);
    drop(book);
    let memory = runtime.memory();
    assert!(runtime.finish().is_ok());
    assert_eq!(memory.stats().charged, 0);
}
fn value(
    book: &Workbook,
    at: Position,
) -> std::result::Result<pivot_types::PivotDecimal, CellError> {
    let c = book.capture_local(book.expected()).unwrap();
    c.evaluate(&[c.cell_at(at).unwrap()]).unwrap().values[0]
        .1
        .clone()
}
#[test]
fn classification_utf8_and_atomic_rejections() {
    run(|b| {
        for (i, (source, text)) in [
            (" Invoice ", " Invoice "),
            ("'001.20", "001.20"),
            ("'=A1", "=A1"),
            ("''Hello", "'Hello"),
            ("e\u{301} 🧾", "e\u{301} 🧾"),
            ("1e3", "1e3"),
            ("   ", "   "),
        ]
        .iter()
        .enumerate()
        {
            let at = p(1, i as u32 + 1);
            b.apply(b.expected(), &[entry(at.row, at.column, source)])
                .unwrap();
            assert_eq!(
                b.cell_at(at).unwrap().unwrap().content,
                Content::LiteralText((*text).into())
            );
        }
        for source in [
            "79228162514264337593543950336",
            "0.00000000000000000000000000001",
            "bad\ttext",
            "bad\ntext",
            "bad\u{0}text",
            "bad\u{85}text",
            "bad\u{2028}text",
            "bad\u{2029}text",
        ] {
            let expected = b.expected();
            assert!(
                b.apply(
                    expected,
                    &[entry(2, 1, "would install"), entry(2, 2, source)]
                )
                .is_err()
            );
            assert_eq!(b.expected(), expected);
            assert!(b.cell_at(p(2, 1)).unwrap().is_none());
        }
        b.apply(b.expected(), &[entry(3, 1, &"é".repeat(2048))])
            .unwrap();
        assert!(
            b.apply(b.expected(), &[entry(3, 1, &"é".repeat(2049))])
                .is_err()
        );
        let maximum = b
            .copy_block(
                b.expected(),
                Rectangle {
                    first: p(3, 1),
                    last: p(3, 1),
                },
            )
            .unwrap();
        b.paste_block(
            b.expected(),
            Rectangle {
                first: p(3, 2),
                last: p(3, 2),
            },
            maximum.text.as_bytes(),
            false,
            &CancellationToken::default(),
        )
        .unwrap();
        assert_eq!(
            b.cell_at(p(3, 1)).unwrap().unwrap().content,
            b.cell_at(p(3, 2)).unwrap().unwrap().content
        );
        b.apply(b.expected(), &[entry(3, 1, "'")]).unwrap();
        assert_eq!(
            b.cell_at(p(3, 1)).unwrap().unwrap().content,
            Content::LiteralText(String::new())
        );
        b.apply(b.expected(), &[entry(3, 1, "")]).unwrap();
        assert_eq!(b.cell_at(p(3, 1)).unwrap().unwrap().content, Content::Blank);
        b.apply(
            b.expected(),
            &[
                entry(4, 1, "-0.000"),
                entry(4, 2, "001.20"),
                entry(4, 3, "=("),
            ],
        )
        .unwrap();
        let Content::Decimal(d) = b.cell_at(p(4, 1)).unwrap().unwrap().content else {
            panic!()
        };
        assert_eq!(d.parts(), (0, 0, 0, true, 3));
        assert!(matches!(
            b.cell_at(p(4, 3)).unwrap().unwrap().content,
            Content::TextFormula(_)
        ));
    });
}
#[test]
fn type_errors_invalidate_scalar_ranges_and_preserve_captures_and_identity() {
    run(|b| {
        b.apply(
            b.expected(),
            &[
                entry(1, 1, "2"),
                entry(1, 2, "=A1+1"),
                entry(1, 3, "=SUM(A1:A2)"),
                entry(1, 4, "=B1*2"),
            ],
        )
        .unwrap();
        let old = b.capture_local(b.expected()).unwrap();
        let id = b.cell_at(p(1, 1)).unwrap().unwrap().reference;
        b.apply(b.expected(), &[entry(1, 1, "Label")]).unwrap();
        assert_eq!(b.cell_at(p(1, 1)).unwrap().unwrap().reference, id);
        for c in 1..=4 {
            assert_eq!(value(b, p(1, c)), Err(CellError::NonNumericInput));
        }
        assert_eq!(
            old.evaluate(&[old.cell_at(p(1, 4)).unwrap()])
                .unwrap()
                .values[0]
                .1
                .as_ref()
                .unwrap()
                .to_string(),
            "6"
        );
        let text_capture = b.capture(b.expected()).unwrap();
        b.apply(b.expected(), &[entry(1, 1, "5")]).unwrap();
        assert_eq!(
            text_capture.evaluate(&[id]).unwrap().values[0].1,
            Err(CellError::NonNumericInput)
        );
        assert_eq!(value(b, p(1, 4)).unwrap().to_string(), "12");
        assert_eq!(value(b, p(1, 3)).unwrap().to_string(), "5");
        b.apply(
            b.expected(),
            &[
                entry(1, 1, "Label"),
                Edit::Move {
                    from: p(1, 1),
                    to: p(2, 2),
                },
            ],
        )
        .unwrap();
        assert_eq!(b.cell_at(p(2, 2)).unwrap().unwrap().reference, id);
        assert_eq!(value(b, p(1, 2)), Err(CellError::NonNumericInput));
        b.apply(b.expected(), &[Edit::Clear(p(2, 2))]).unwrap();
        assert_eq!(value(b, p(1, 2)).unwrap().to_string(), "1");
    });
}
#[test]
fn mixed_native_and_tsv_roundtrip_overlap_and_bad_version() {
    run(|b| {
        b.apply(
            b.expected(),
            &[
                entry(1, 1, "''Hello"),
                entry(1, 2, "'001.20"),
                entry(1, 3, "'=A1"),
                entry(1, 4, "2"),
                entry(1, 5, "=D1+1"),
            ],
        )
        .unwrap();
        let block = b
            .copy_block(
                b.expected(),
                Rectangle {
                    first: p(1, 1),
                    last: p(1, 5),
                },
            )
            .unwrap();
        assert!(block.native.starts_with(b"PIVOTCB5"));
        assert_eq!(block.text, "''Hello\t'001.20\t'=A1\t2\t=D1+1");
        for (row, bytes, native) in [
            (2, block.native.as_slice(), true),
            (3, block.text.as_bytes(), false),
        ] {
            b.paste_block(
                b.expected(),
                Rectangle {
                    first: p(row, 1),
                    last: p(row, 1),
                },
                bytes,
                native,
                &CancellationToken::default(),
            )
            .unwrap();
            for c in 1..=4 {
                assert_eq!(
                    b.cell_at(p(1, c)).unwrap().unwrap().content,
                    b.cell_at(p(row, c)).unwrap().unwrap().content
                );
            }
        }
        let expected = b.expected();
        let mut old = block.native.clone();
        old[7] = b'1';
        assert!(
            b.paste_block(
                expected,
                Rectangle {
                    first: p(4, 1),
                    last: p(4, 1)
                },
                &old,
                true,
                &CancellationToken::default()
            )
            .is_err()
        );
        assert_eq!(b.expected(), expected);
        let overlap = b
            .copy_block(
                b.expected(),
                Rectangle {
                    first: p(1, 1),
                    last: p(1, 3),
                },
            )
            .unwrap();
        b.paste_block(
            b.expected(),
            Rectangle {
                first: p(1, 2),
                last: p(1, 2),
            },
            &overlap.native,
            true,
            &CancellationToken::default(),
        )
        .unwrap();
        assert_eq!(
            b.cell_at(p(1, 2)).unwrap().unwrap().content,
            Content::LiteralText("'Hello".into())
        );
    });
}
