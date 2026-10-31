use pivot_core::model::integration::{CancellationToken, IntegrationRuntime};
use pivot_workbook::*;
fn p(row: u32, column: u32) -> Position {
    Position::new(row, column)
}
fn rect(first: Position, last: Position) -> Rectangle {
    Rectangle { first, last }
}
fn one(at: Position) -> Rectangle {
    rect(at, at)
}
fn number(at: Position, value: &str) -> Edit {
    Edit::SetDecimal {
        position: at,
        value: DecimalInput::from_decimal(value.parse().unwrap()),
    }
}
fn formula(at: Position, source: &str) -> Edit {
    Edit::SetFormulaText {
        position: at,
        source: source.into(),
    }
}
fn source(book: &Workbook, at: Position) -> String {
    let Content::TextFormula(f) = book.cell_at(at).unwrap().unwrap().content else {
        panic!()
    };
    f.original_source
}
fn value(book: &Workbook, at: Position) -> String {
    let capture = book.capture_local(book.expected()).unwrap();
    capture
        .evaluate(&[capture.cell_at(at).unwrap()])
        .unwrap()
        .values[0]
        .1
        .as_ref()
        .unwrap()
        .to_string()
}
fn run(test: impl FnOnce(&mut Workbook)) {
    let runtime = IntegrationRuntime::standalone().unwrap();
    let mut book = Workbook::new_in("Clipboard", Limits::default(), &runtime).unwrap();
    test(&mut book);
    drop(book);
    let memory = runtime.memory();
    assert!(runtime.finish().is_ok());
    assert_eq!(memory.stats().charged, 0);
}
fn copy(book: &mut Workbook, from: Rectangle, to: Rectangle) -> Rectangle {
    let block = book.copy_block(book.expected(), from).unwrap();
    book.paste_block(
        book.expected(),
        to,
        &block.native,
        true,
        &CancellationToken::default(),
    )
    .unwrap()
}
#[test]
fn axes_moved_targets_and_origins_use_effective_bindings() {
    run(|book| {
        book.apply(
            book.expected(),
            &[
                number(p(2, 1), "10"),
                formula(p(2, 2), "= A2 + $A2 + A$2 + $A$2"),
            ],
        )
        .unwrap();
        copy(book, one(p(2, 2)), one(p(5, 4)));
        assert_eq!(source(book, p(5, 4)), "= C5 + $A5 + C$2 + $A$2");
        book.apply(
            book.expected(),
            &[
                formula(p(1, 2), "=A1+1"),
                Edit::Move {
                    from: p(1, 1),
                    to: p(9, 1),
                },
            ],
        )
        .unwrap();
        let block = book.copy_block(book.expected(), one(p(1, 2))).unwrap();
        assert_eq!(block.text, "=A9+1");
        book.paste_block(
            book.expected(),
            one(p(2, 2)),
            &block.native,
            true,
            &Default::default(),
        )
        .unwrap();
        assert_eq!(source(book, p(2, 2)), "=A10+1");
        book.apply(
            book.expected(),
            &[Edit::Move {
                from: p(1, 2),
                to: p(3, 2),
            }],
        )
        .unwrap();
        copy(book, one(p(3, 2)), one(p(4, 2)));
        assert_eq!(source(book, p(4, 2)), "=A10+1");
    });
}
#[test]
fn overlap_blank_clearing_and_destination_identities() {
    run(|book| {
        book.apply(
            book.expected(),
            &[
                number(p(1, 1), "1"),
                number(p(2, 1), "2"),
                number(p(3, 1), "3"),
            ],
        )
        .unwrap();
        let id = book.cell_at(p(2, 1)).unwrap().unwrap().reference;
        copy(book, rect(p(1, 1), p(2, 1)), one(p(2, 1)));
        assert_eq!(value(book, p(1, 1)), "1");
        assert_eq!(value(book, p(2, 1)), "1");
        assert_eq!(value(book, p(3, 1)), "2");
        assert_eq!(book.cell_at(p(2, 1)).unwrap().unwrap().reference, id);
        copy(book, one(p(9, 9)), one(p(2, 1)));
        assert!(matches!(
            book.cell_at(p(2, 1)).unwrap().unwrap().content,
            Content::Blank
        ));
        assert_eq!(book.cell_at(p(2, 1)).unwrap().unwrap().reference, id);
    });
}
#[test]
fn copied_formula_binds_later_created_destination_and_fresh_ids() {
    run(|book| {
        book.apply(
            book.expected(),
            &[formula(p(1, 1), "=B1+1"), number(p(1, 2), "7")],
        )
        .unwrap();
        let origin = book.cell_at(p(1, 2)).unwrap().unwrap().reference;
        copy(book, rect(p(1, 1), p(1, 2)), one(p(5, 5)));
        assert_eq!(source(book, p(5, 5)), "=F5+1");
        assert_eq!(value(book, p(5, 5)), "8");
        assert_ne!(book.cell_at(p(5, 6)).unwrap().unwrap().reference, origin);
        book.apply(book.expected(), &[number(p(5, 6), "10")])
            .unwrap();
        assert_eq!(value(book, p(5, 5)), "11");
    });
}
#[test]
fn reversed_sum_axes_fresh_selection_and_exact_source() {
    run(|book| {
        book.apply(
            book.expected(),
            &[formula(p(5, 5), "= sum( A3 : $B$1 ) + 1.00")],
        )
        .unwrap();
        copy(book, one(p(5, 5)), one(p(7, 6)));
        assert_eq!(source(book, p(7, 6)), "= sum( B5 : $B$1 ) + 1.00");
        let Content::TextFormula(a) = book.cell_at(p(5, 5)).unwrap().unwrap().content else {
            panic!()
        };
        let Content::TextFormula(b) = book.cell_at(p(7, 6)).unwrap().unwrap().content else {
            panic!()
        };
        let Token::Sum { id: a, .. } = a.tokens[0] else {
            panic!()
        };
        let Token::Sum { id: b, .. } = b.tokens[0] else {
            panic!()
        };
        assert_ne!(a, b);
    });
}
#[test]
fn external_tsv_shape_empty_rows_and_literal_formulas() {
    run(|book| {
        let paste = |book: &mut Workbook, text: &str| {
            book.paste_block(
                book.expected(),
                one(p(3, 3)),
                text.as_bytes(),
                false,
                &Default::default(),
            )
        };
        paste(book, "1\t2\r\n=A1+1\t\r\n").unwrap();
        assert_eq!(source(book, p(4, 3)), "=A1+1");
        assert_eq!(value(book, p(3, 4)), "2");
        assert_eq!(paste(book, "1\n\n").unwrap().last, p(4, 3));
        for bad in [
            "1\t2\n3",
            "1\r2",
            "1\r",
            "bad\u{0}text",
            "999999999999999999999999999999999",
        ] {
            let before = book.expected();
            assert!(paste(book, bad).is_err(), "{bad}");
            assert_eq!(book.expected(), before);
        }
        let before = book.expected();
        assert!(
            book.paste_block(
                before,
                rect(p(3, 3), p(4, 4)),
                b"1\t2",
                false,
                &Default::default()
            )
            .is_err()
        );
        assert_eq!(book.expected(), before);
    });
}
#[test]
fn portable_cross_document_exact_decimal_and_invalid_draft() {
    run(|book| {
        let zero = DecimalInput::from_parts(0, 0, 0, true, 4).unwrap();
        book.apply(
            book.expected(),
            &[
                Edit::SetDecimal {
                    position: p(1, 1),
                    value: zero,
                },
                formula(p(1, 2), "=("),
            ],
        )
        .unwrap();
        let block = book
            .copy_block(book.expected(), rect(p(1, 1), p(1, 2)))
            .unwrap();
        let mut other = Workbook::new("Other", Limits::default()).unwrap();
        other
            .paste_block(
                other.expected(),
                one(p(4, 4)),
                &block.native,
                true,
                &Default::default(),
            )
            .unwrap();
        assert_eq!(
            other.cell_at(p(4, 4)).unwrap().unwrap().content,
            Content::Decimal(zero)
        );
        let Content::TextFormula(f) = other.cell_at(p(4, 5)).unwrap().unwrap().content else {
            panic!()
        };
        assert!(f.diagnostic.is_some());
        assert!(f.tokens.is_empty());
    });
}
#[test]
fn rollback_bounds_late_quota_stale_and_cancellation() {
    run(|book| {
        book.apply(
            book.expected(),
            &[number(p(2, 1), "7"), formula(p(2, 2), "=A1")],
        )
        .unwrap();
        let block = book
            .copy_block(book.expected(), rect(p(2, 1), p(2, 2)))
            .unwrap();
        let before = book.expected();
        assert!(
            book.paste_block(
                before,
                one(p(1, 1)),
                &block.native,
                true,
                &Default::default()
            )
            .is_err()
        );
        assert_eq!(book.expected(), before);
        assert!(book.cell_at(p(1, 1)).unwrap().is_some());
        let cancel = CancellationToken::default();
        cancel.cancel();
        assert!(
            book.paste_block(before, one(p(5, 5)), &block.native, true, &cancel)
                .is_err()
        );
        assert_eq!(book.expected(), before);
        book.apply(before, &[number(p(8, 8), "1")]).unwrap();
        let after = book.expected();
        assert!(
            book.paste_block(
                before,
                one(p(5, 5)),
                &block.native,
                true,
                &Default::default()
            )
            .is_err()
        );
        assert_eq!(book.expected(), after);
        assert!(book.copy_block(after, rect(p(1, 1), p(17, 17))).is_err());
    });
}
#[test]
fn tampered_native_shape_types_annotations_and_trailing_bytes_reject_atomically() {
    run(|book| {
        book.apply(book.expected(), &[formula(p(1, 2), "=A1")])
            .unwrap();
        let bytes = book
            .copy_block(book.expected(), one(p(1, 2)))
            .unwrap()
            .native;
        let mut variants = Vec::new();
        for (offset, value) in [(16, 0u32), (20, 257u32)] {
            let mut bad = bytes.clone();
            bad[offset..offset + 4].copy_from_slice(&value.to_le_bytes());
            variants.push(bad);
        }
        let mut bad = bytes.clone();
        bad[24] = 255;
        variants.push(bad);
        let mut bad = bytes.clone();
        let len = bad.len();
        bad[len - 8..len - 4].copy_from_slice(&0u32.to_le_bytes());
        variants.push(bad);
        let mut bad = bytes.clone();
        bad.push(0);
        variants.push(bad);
        for bad in variants {
            let before = book.expected();
            assert!(
                book.paste_block(before, one(p(5, 5)), &bad, true, &Default::default())
                    .is_err()
            );
            assert_eq!(book.expected(), before);
            assert!(book.cell_at(p(5, 5)).unwrap().is_none());
        }
    });
}
#[test]
fn retired_reference_rejects_native_copy_instead_of_rebinding_spelling() {
    run(|book| {
        book.apply(
            book.expected(),
            &[
                formula(p(1, 2), "=A1"),
                Edit::Retire(p(1, 1)),
                number(p(1, 1), "9"),
            ],
        )
        .unwrap();
        assert!(book.copy_block(book.expected(), one(p(1, 2))).is_err());
    });
}

#[test]
fn late_candidate_quota_failure_preserves_earlier_overwrite_and_revision() {
    let runtime = IntegrationRuntime::standalone().unwrap();
    let mut book = Workbook::new_in(
        "Quota",
        Limits {
            live_cells: 2,
            ..Limits::default()
        },
        &runtime,
    )
    .unwrap();
    book.apply(
        book.expected(),
        &[number(p(1, 1), "7"), number(p(1, 2), "8")],
    )
    .unwrap();
    let before = book.expected();
    let before_stats = book.stats();
    assert!(
        book.paste_block(before, one(p(1, 2)), b"99\t100", false, &Default::default())
            .is_err()
    );
    assert_eq!(book.expected(), before);
    assert_eq!(book.stats(), before_stats);
    assert_eq!(value(&book, p(1, 2)), "8");
    assert!(book.cell_at(p(1, 3)).unwrap().is_none());
    drop(book);
    let memory = runtime.memory();
    assert!(runtime.finish().is_ok());
    assert_eq!(memory.stats().charged, 0);
}
#[test]
fn source_snapshot_and_resource_profile_failures_are_atomic() {
    run(|book| {
        book.apply(book.expected(), &[number(p(1, 1), "1.2300")])
            .unwrap();
        let block = book.copy_block(book.expected(), one(p(1, 1))).unwrap();
        book.apply(book.expected(), &[number(p(1, 1), "99")])
            .unwrap();
        book.paste_block(
            book.expected(),
            one(p(2, 2)),
            &block.native,
            true,
            &Default::default(),
        )
        .unwrap();
        assert_eq!(value(book, p(2, 2)), "1.2300");
        for oversized in [vec![b'1'; 262145], vec![b'='; 4097]] {
            let before = book.expected();
            assert!(
                book.paste_block(before, one(p(3, 3)), &oversized, false, &Default::default())
                    .is_err()
            );
            assert_eq!(book.expected(), before);
        }
        let formula = format!("={}1", "1+".repeat(500));
        let many = vec![formula; 40].join("\t");
        let before = book.expected();
        assert!(
            book.paste_block(
                before,
                one(p(4, 4)),
                many.as_bytes(),
                false,
                &Default::default()
            )
            .is_err()
        );
        assert_eq!(book.expected(), before);
    });
}
#[test]
fn formula_whitespace_source_is_native_exact_and_tsv_has_unambiguous_shape() {
    run(|book| {
        book.apply(book.expected(), &[formula(p(2, 2), "= \tA1 +\n1.00")])
            .unwrap();
        let block = book.copy_block(book.expected(), one(p(2, 2))).unwrap();
        assert!(!block.text.contains(['\t', '\n', '\r']));
        book.paste_block(
            book.expected(),
            one(p(3, 3)),
            &block.native,
            true,
            &Default::default(),
        )
        .unwrap();
        assert_eq!(source(book, p(3, 3)), "= \tB2 +\n1.00");
    });
}
#[test]
fn copied_rejected_state_is_preserved_until_explicit_resubmission() {
    run(|book| {
        book.apply(
            book.expected(),
            &[Edit::PreserveRejectedFormula {
                position: p(1, 1),
                source: "=$B$2+1".into(),
            }],
        )
        .unwrap();
        copy(book, one(p(1, 1)), one(p(4, 4)));
        let Content::TextFormula(f) = book.cell_at(p(4, 4)).unwrap().unwrap().content else {
            panic!()
        };
        assert_eq!(f.original_source, "=$B$2+1");
        assert!(f.diagnostic.is_some());
        assert!(f.tokens.is_empty() && f.references.is_empty());
        assert_eq!(
            book.display_formula(book.cell_at(p(4, 4)).unwrap().unwrap().reference)
                .unwrap(),
            Some("=$B$2+1".into())
        );
        assert!(
            book.capture_local(book.expected())
                .unwrap()
                .evaluate(&[book.cell_at(p(4, 4)).unwrap().unwrap().reference])
                .is_err()
        );
        book.apply(book.expected(), &[formula(p(4, 4), "=$B$2+1")])
            .unwrap();
        assert_eq!(value(book, p(4, 4)), "1");
    });
}

#[test]
fn external_signed_zero_roundtrips_native_and_text_without_float_conversion() {
    run(|book| {
        book.paste_block(
            book.expected(),
            one(p(1, 1)),
            b"-0.000",
            false,
            &Default::default(),
        )
        .unwrap();
        assert_eq!(
            book.cell_at(p(1, 1)).unwrap().unwrap().content,
            Content::Decimal(DecimalInput::from_parts(0, 0, 0, true, 3).unwrap())
        );
        let block = book.copy_block(book.expected(), one(p(1, 1))).unwrap();
        assert_eq!(block.text, "-0.000");
        book.paste_block(
            book.expected(),
            one(p(2, 2)),
            block.text.as_bytes(),
            false,
            &Default::default(),
        )
        .unwrap();
        assert_eq!(
            book.cell_at(p(2, 2)).unwrap().unwrap().content,
            book.cell_at(p(1, 1)).unwrap().unwrap().content
        );
    });
}

#[test]
fn typed_formula_copy_requires_validated_spreadsheet_source_and_token_correspondence() {
    run(|book| {
        book.apply(
            book.expected(),
            &[
                number(p(1, 1), "5"),
                Edit::SetFormula {
                    position: p(1, 2),
                    source: "=A1+1".into(),
                    tokens: vec![
                        Token::At(p(1, 1)),
                        Token::Decimal(DecimalInput::from_decimal("1".parse().unwrap())),
                        Token::Binary(pivot_core::expression::BinaryOp::Add),
                    ],
                },
            ],
        )
        .unwrap();
        copy(book, one(p(1, 2)), one(p(2, 2)));
        assert_eq!(source(book, p(2, 2)), "=A2+1");
        book.apply(
            book.expected(),
            &[Edit::SetFormula {
                position: p(3, 3),
                source: "opaque provenance".into(),
                tokens: vec![Token::At(p(1, 1))],
            }],
        )
        .unwrap();
        assert!(book.copy_block(book.expected(), one(p(3, 3))).is_err());
    });
}

#[test]
fn malformed_external_formulas_cannot_bypass_aggregate_parser_work() {
    run(|book| {
        let malformed = format!("={} )", "1+".repeat(500));
        let payload = vec![malformed; 40].join("\t");
        let before = book.expected();
        assert!(
            book.paste_block(
                before,
                one(p(1, 1)),
                payload.as_bytes(),
                false,
                &Default::default()
            )
            .is_err()
        );
        assert_eq!(book.expected(), before);
        assert_eq!(book.stats().live_cells, 0);
    });
}
