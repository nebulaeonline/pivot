use pivot_core::model::integration::{CancellationToken, IntegrationRuntime};
use pivot_workbook::*;
fn p(row: u32, column: u32) -> Position {
    Position::new(row, column)
}
fn rect(first: Position, last: Position) -> Rectangle {
    Rectangle { first, last }
}
fn entry(row: u32, column: u32, text: &str) -> Edit {
    Edit::SetEntry {
        position: p(row, column),
        source: text.into(),
    }
}
fn run(test: impl FnOnce(&mut Workbook)) {
    let rt = IntegrationRuntime::standalone().unwrap();
    let mut book = Workbook::new_in("Visible", Limits::default(), &rt).unwrap();
    test(&mut book);
    drop(book);
    let memory = rt.memory();
    assert!(rt.finish().is_ok());
    assert_eq!(memory.stats().charged, 0);
}
fn formula(book: &Workbook, row: u32, column: u32) -> String {
    let Content::TextFormula(f) = book.cell_at(p(row, column)).unwrap().unwrap().content else {
        panic!("formula required")
    };
    f.original_source
}
#[test]
fn independent_small_masks_preserve_order_origins_values_and_hidden_cells() {
    for sm in 1u32..8 {
        for dm in 1u32..8 {
            let source: Vec<_> = [2, 5, 9]
                .into_iter()
                .enumerate()
                .filter(|(i, _)| sm & (1 << i) != 0)
                .map(|(_, r)| r)
                .collect();
            let destination: Vec<_> = [20, 23, 28]
                .into_iter()
                .enumerate()
                .filter(|(i, _)| dm & (1 << i) != 0)
                .map(|(_, r)| r)
                .collect();
            if source.len() != destination.len() {
                continue;
            }
            run(|b| {
                let mut edits = vec![entry(22, 3, "'hidden sentinel")];
                for &r in &source {
                    edits.push(entry(r, 1, &format!("'00{r}")));
                    edits.push(entry(r, 2, &format!("=A{r}")));
                }
                b.apply(b.expected(), &edits).unwrap();
                let hidden = b.cell_at(p(22, 3)).unwrap().unwrap();
                let image = b
                    .copy_visible_sheet(
                        b.worksheet(),
                        b.expected(),
                        &source,
                        1,
                        2,
                        &Default::default(),
                    )
                    .unwrap();
                assert!(image.native.starts_with(b"PIVOTCB6"));
                assert!(image.text_available);
                b.paste_visible_sheet(
                    b.worksheet(),
                    b.expected(),
                    pivot_workbook::clipboard::VisibleDestination {
                        rectangle: rect(p(destination[0], 3), p(*destination.last().unwrap(), 4)),
                        rows: &destination,
                    },
                    &image.native,
                    true,
                    &Default::default(),
                )
                .unwrap();
                for (&sr, &dr) in source.iter().zip(&destination) {
                    assert_eq!(formula(b, dr, 4), format!("=C{dr}"));
                    assert!(
                        matches!(b.cell_at(p(dr,3)).unwrap().unwrap().content,Content::LiteralText(v) if v==format!("00{sr}"))
                    );
                }
                assert_eq!(b.cell_at(p(22, 3)).unwrap().unwrap(), hidden);
            });
        }
    }
}
#[test]
fn cross_record_references_are_geometric_not_rewired_and_cb6_packs_ordinary_paste() {
    run(|b| {
        b.apply(
            b.expected(),
            &[
                entry(2, 1, "7"),
                entry(2, 2, "=A2"),
                entry(5, 1, "11"),
                entry(5, 2, "=A2"),
                entry(9, 3, "42"),
            ],
        )
        .unwrap();
        let image = b
            .copy_visible_sheet(
                b.worksheet(),
                b.expected(),
                &[2, 5],
                1,
                2,
                &Default::default(),
            )
            .unwrap();
        b.paste_visible_sheet(
            b.worksheet(),
            b.expected(),
            pivot_workbook::clipboard::VisibleDestination {
                rectangle: rect(p(10, 3), p(12, 4)),
                rows: &[10, 12],
            },
            &image.native,
            true,
            &Default::default(),
        )
        .unwrap();
        assert_eq!(formula(b, 10, 4), "=C10");
        assert_eq!(formula(b, 12, 4), "=C9");
        b.paste_block(
            b.expected(),
            rect(p(30, 3), p(30, 3)),
            &image.native,
            true,
            &Default::default(),
        )
        .unwrap();
        assert_eq!(formula(b, 31, 4), "=C28");
    });
}
#[test]
fn native_only_strings_and_untrusted_row_metadata_are_atomic() {
    run(|b| {
        b.apply(
            b.expected(),
            &[
                Edit::SetText {
                    position: p(2, 1),
                    text: "001234\tline\nnext".into(),
                },
                entry(5, 1, "'001234"),
            ],
        )
        .unwrap();
        let image = b
            .copy_visible_sheet(
                b.worksheet(),
                b.expected(),
                &[2, 5],
                1,
                1,
                &Default::default(),
            )
            .unwrap();
        assert!(!image.text_available);
        assert!(image.text.is_empty());
        let before = b.expected();
        let mut duplicate = image.native.clone();
        duplicate[28..32].copy_from_slice(&2u32.to_le_bytes());
        for bad in [
            duplicate,
            image.native[..image.native.len() - 1].to_vec(),
            [image.native.clone(), vec![0]].concat(),
        ] {
            assert!(
                b.paste_visible_sheet(
                    b.worksheet(),
                    before,
                    pivot_workbook::clipboard::VisibleDestination {
                        rectangle: rect(p(10, 1), p(12, 1)),
                        rows: &[10, 12]
                    },
                    &bad,
                    true,
                    &Default::default()
                )
                .is_err()
            );
            assert_eq!(b.expected(), before);
        }
        let cancel = CancellationToken::default();
        cancel.cancel();
        assert!(
            b.paste_visible_sheet(
                b.worksheet(),
                before,
                pivot_workbook::clipboard::VisibleDestination {
                    rectangle: rect(p(10, 1), p(12, 1)),
                    rows: &[10, 12]
                },
                &image.native,
                true,
                &cancel
            )
            .is_err()
        );
        assert_eq!(b.expected(), before);
        b.paste_visible_sheet(
            b.worksheet(),
            before,
            pivot_workbook::clipboard::VisibleDestination {
                rectangle: rect(p(10, 1), p(12, 1)),
                rows: &[10, 12],
            },
            &image.native,
            true,
            &Default::default(),
        )
        .unwrap();
        assert!(
            matches!(b.cell_at(p(10,1)).unwrap().unwrap().content,Content::LiteralText(v) if v=="001234\tline\nnext")
        );
    });
}
#[test]
fn mapped_style_overwrites_match_source_even_on_blank_slots_and_preserve_gaps() {
    use pivot_workbook::formatting::{Field, StylePatch};
    run(|b| {
        let source = [2, 5, 9, 12];
        let destination = [20, 23, 25, 40];
        let sheet = b.worksheet();
        b.apply(b.expected(), &[entry(2, 1, "'001234"), entry(5, 2, "TRUE")])
            .unwrap();
        b.format_sheet(
            sheet,
            b.expected(),
            rect(p(1, 3), p(65536, 4)),
            StylePatch {
                bold: Field::Set(true),
                background: Field::Set(Some(0x112233)),
                ..Default::default()
            },
            false,
            &Default::default(),
        )
        .unwrap();
        b.format_sheet(
            sheet,
            b.expected(),
            rect(p(5, 1), p(9, 2)),
            StylePatch {
                italic: Field::Set(true),
                foreground: Field::Set(Some(0xabcdef)),
                ..Default::default()
            },
            false,
            &Default::default(),
        )
        .unwrap();
        let hidden = b.style_at(sheet, p(22, 3)).unwrap();
        let expected = source
            .iter()
            .flat_map(|row| (1..=2).map(move |column| p(*row, column)))
            .map(|p| b.style_at(sheet, p).unwrap())
            .collect::<Vec<_>>();
        let image = b
            .copy_visible_sheet(sheet, b.expected(), &source, 1, 2, &Default::default())
            .unwrap();
        b.paste_visible_sheet(
            sheet,
            b.expected(),
            pivot_workbook::clipboard::VisibleDestination {
                rectangle: rect(p(20, 3), p(40, 4)),
                rows: &destination,
            },
            &image.native,
            true,
            &Default::default(),
        )
        .unwrap();
        for (index, at) in destination
            .iter()
            .flat_map(|row| (3..=4).map(move |column| p(*row, column)))
            .enumerate()
        {
            assert_eq!(b.style_at(sheet, at).unwrap(), expected[index]);
        }
        assert_eq!(b.style_at(sheet, p(22, 3)).unwrap(), hidden);
        assert!(b.cell_at(p(22, 3)).unwrap().is_none());
        let before = b.expected();
        b.paste_visible_sheet(
            sheet,
            before,
            pivot_workbook::clipboard::VisibleDestination {
                rectangle: rect(p(20, 3), p(40, 4)),
                rows: &destination,
            },
            &image.native,
            true,
            &Default::default(),
        )
        .unwrap();
        assert_eq!(b.expected(), before);
    });
}
#[test]
fn mapped_late_cell_and_style_quota_failures_publish_nothing() {
    for style_quota in [false, true] {
        let rt = IntegrationRuntime::standalone().unwrap();
        let mut limits = Limits::default();
        if style_quota {
            limits.styles.positions = 1;
        } else {
            limits.live_cells = 2;
        }
        let mut b = Workbook::new_in("atomic mapped admission", limits, &rt).unwrap();
        b.apply(b.expected(), &[entry(1, 1, "7"), entry(1, 2, "8")])
            .unwrap();
        let before = b.expected();
        let stats = b.stats();
        let old = b.cell_at(p(1, 2)).unwrap();
        let image = b
            .copy_visible_sheet(b.worksheet(), before, &[1], 1, 1, &Default::default())
            .unwrap();
        let result = if style_quota {
            b.paste_visible_sheet(
                b.worksheet(),
                before,
                pivot_workbook::clipboard::VisibleDestination {
                    rectangle: rect(p(1, 2), p(1, 2)),
                    rows: &[1],
                },
                &image.native,
                true,
                &Default::default(),
            )
        } else {
            b.paste_visible_sheet(
                b.worksheet(),
                before,
                pivot_workbook::clipboard::VisibleDestination {
                    rectangle: rect(p(1, 2), p(5, 2)),
                    rows: &[1, 5],
                },
                b"99\n100",
                false,
                &Default::default(),
            )
        };
        assert!(result.is_err());
        assert_eq!(b.expected(), before);
        assert_eq!(b.stats(), stats);
        assert_eq!(b.cell_at(p(1, 2)).unwrap(), old);
        assert!(b.cell_at(p(5, 2)).unwrap().is_none());
        drop(b);
        assert!(rt.finish().is_ok());
    }
}
