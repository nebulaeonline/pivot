use pivot_core::model::integration::{CancellationToken, IntegrationRuntime};
use pivot_workbook::{formatting::*, visible_edit::ClearMode, *};
fn p(r: u32, c: u32) -> Position {
    Position::new(r, c)
}
fn rect(a: Position, b: Position) -> Rectangle {
    Rectangle { first: a, last: b }
}
fn entry(r: u32, c: u32, s: &str) -> Edit {
    Edit::SetEntry {
        position: p(r, c),
        source: s.into(),
    }
}
fn run(limits: Limits, test: impl FnOnce(&mut Workbook)) {
    let rt = IntegrationRuntime::standalone().unwrap();
    let mut b = Workbook::new_in("Visible edit", limits, &rt).unwrap();
    test(&mut b);
    drop(b);
    let m = rt.memory();
    assert!(rt.finish().is_ok());
    assert_eq!(m.stats().charged, 0);
}
#[test]
fn independent_masks_and_all_directions_use_original_visible_edges() {
    for mask in 1..8 {
        let rows: Vec<_> = [2, 5, 12]
            .into_iter()
            .enumerate()
            .filter(|(i, _)| mask & (1 << i) != 0)
            .map(|(_, r)| r)
            .collect();
        for direction in [
            FillDirection::Down,
            FillDirection::Up,
            FillDirection::Left,
            FillDirection::Right,
        ] {
            run(Limits::capacity(), |b| {
                let edits = (1..=13)
                    .flat_map(|r| (2..=4).map(move |c| entry(r, c, &format!("'00{r}:{c}"))))
                    .collect::<Vec<_>>();
                b.apply(b.expected(), &edits).unwrap();
                let before = (1..=13)
                    .flat_map(|r| (2..=4).map(move |c| p(r, c)))
                    .map(|p| b.cell_at(p).unwrap().unwrap())
                    .collect::<Vec<_>>();
                b.fill_visible_sheet(
                    b.worksheet(),
                    b.expected(),
                    &rows,
                    (2, 4),
                    direction,
                    &Default::default(),
                )
                .unwrap();
                for r in 1..=13 {
                    for c in 2..=4 {
                        let mut source = (r, c);
                        if rows.contains(&r) {
                            source = match direction {
                                FillDirection::Down => (rows[0], c),
                                FillDirection::Up => (*rows.last().unwrap(), c),
                                FillDirection::Right => (r, 2),
                                FillDirection::Left => (r, 4),
                            };
                        }
                        let actual = b.cell_at(p(r, c)).unwrap().unwrap();
                        let expected = &before[((source.0 - 1) * 3 + source.1 - 2) as usize];
                        assert_eq!(
                            actual.content, expected.content,
                            "mask {mask}, {direction:?}, {r}:{c}"
                        );
                        assert_eq!(
                            actual.reference,
                            before[((r - 1) * 3 + c - 2) as usize].reference
                        );
                    }
                }
            });
        }
    }
}
#[test]
fn clear_modes_overwrite_exceptions_preserve_hidden_identities_and_explicit_defaults() {
    for mode in [ClearMode::Contents, ClearMode::Formatting, ClearMode::Both] {
        run(Limits::capacity(), |b| {
            let sheet = b.worksheet();
            b.apply(
                b.expected(),
                &[
                    entry(2, 2, "'001234"),
                    entry(3, 2, "'hidden"),
                    entry(5, 2, "=B2"),
                    entry(6, 3, "=B5"),
                ],
            )
            .unwrap();
            b.set_worksheet_default(
                sheet,
                b.expected(),
                StylePatch {
                    background: Field::Set(Some(0xffff00)),
                    ..Default::default()
                },
                &Default::default(),
            )
            .unwrap();
            b.format_sheet(
                sheet,
                b.expected(),
                rect(p(1, 1), p(12, 4)),
                StylePatch {
                    bold: Field::Set(true),
                    ..Default::default()
                },
                false,
                &Default::default(),
            )
            .unwrap();
            b.format_sheet(
                sheet,
                b.expected(),
                rect(p(5, 2), p(5, 2)),
                StylePatch {
                    italic: Field::Set(true),
                    ..Default::default()
                },
                false,
                &Default::default(),
            )
            .unwrap();
            let hidden = b.cell_at(p(3, 2)).unwrap().unwrap();
            let hs = b.style_at(sheet, p(3, 2)).unwrap();
            let ids = [2, 5].map(|r| b.cell_at(p(r, 2)).unwrap().unwrap().reference);
            let before = b.expected();
            let cells = b.stats().live_cells;
            b.clear_visible_sheet(
                sheet,
                before,
                &[2, 5, 12],
                (2, 2),
                mode,
                &Default::default(),
            )
            .unwrap();
            assert_eq!(b.stats().live_cells, cells);
            assert!(b.cell_at(p(12, 2)).unwrap().is_none());
            assert_eq!(b.cell_at(p(3, 2)).unwrap().unwrap(), hidden);
            assert_eq!(b.style_at(sheet, p(3, 2)).unwrap(), hs);
            for (i, r) in [2, 5].into_iter().enumerate() {
                let cell = b.cell_at(p(r, 2)).unwrap().unwrap();
                assert_eq!(cell.reference, ids[i]);
                if mode != ClearMode::Formatting {
                    assert_eq!(cell.content, Content::Blank);
                }
            }
            if mode == ClearMode::Formatting {
                assert_eq!(b.expected().graph, before.graph);
            }
            if mode != ClearMode::Contents {
                for r in [2, 5, 12] {
                    assert_eq!(b.style_at(sheet, p(r, 2)).unwrap(), Style::default());
                }
            }
            let same = b.expected();
            b.clear_visible_sheet(sheet, same, &[2, 5, 12], (2, 2), mode, &Default::default())
                .unwrap();
            assert_eq!(b.expected(), same);
        });
    }
    // Equal appearance is still a canonical explicit assignment change.
    run(Limits::capacity(), |b| {
        let sheet = b.worksheet();
        let before = b.expected();
        b.clear_visible_sheet(
            sheet,
            before,
            &[2],
            (2, 2),
            ClearMode::Both,
            &Default::default(),
        )
        .unwrap();
        assert_ne!(b.expected(), before);
        assert_eq!(b.expected().graph, before.graph);
        assert_eq!(b.stats().live_cells, 0);
    });
}
#[test]
fn boundary_cancel_and_late_style_refusal_are_atomic() {
    run(Limits::capacity(), |b| {
        let sheet = b.worksheet();
        let rows = (1..=4096).map(|i| i * 16).collect::<Vec<_>>();
        b.clear_visible_sheet(
            sheet,
            b.expected(),
            &rows,
            (1, 1),
            ClearMode::Both,
            &Default::default(),
        )
        .unwrap();
        let same = b.expected();
        b.clear_visible_sheet(
            sheet,
            same,
            &rows,
            (1, 1),
            ClearMode::Both,
            &Default::default(),
        )
        .unwrap();
        assert_eq!(b.expected(), same);
        assert_eq!(b.stats().live_cells, 0);
        let mut too_many = rows;
        too_many.insert(0, 1);
        assert!(
            b.clear_visible_sheet(
                sheet,
                same,
                &too_many,
                (1, 1),
                ClearMode::Contents,
                &Default::default()
            )
            .is_err()
        );
        assert_eq!(b.expected(), same);
        let cancel = CancellationToken::default();
        cancel.cancel();
        assert!(
            b.clear_visible_sheet(sheet, same, &[1], (1, 1), ClearMode::Both, &cancel)
                .is_err()
        );
        assert_eq!(b.expected(), same);
        assert!(
            b.fill_visible_sheet(
                sheet,
                same,
                &[1],
                (1, 257),
                FillDirection::Down,
                &Default::default()
            )
            .is_err()
        );
    });
    let mut limits = Limits::capacity();
    limits.styles.positions = 1;
    run(limits, |b| {
        b.apply(b.expected(), &[entry(2, 1, "'keep")]).unwrap();
        let before = b.expected();
        let cell = b.cell_at(p(2, 1)).unwrap();
        assert!(
            b.clear_visible_sheet(
                b.worksheet(),
                before,
                &[2],
                (1, 1),
                ClearMode::Both,
                &Default::default()
            )
            .is_err()
        );
        assert_eq!(b.expected(), before);
        assert_eq!(b.cell_at(p(2, 1)).unwrap(), cell);
    });
}

#[test]
fn geometric_formulas_hidden_blank_demand_and_cycles() {
    run(Limits::capacity(), |b| {
        b.apply(
            b.expected(),
            &[
                entry(5, 2, "=A2"),
                entry(8, 2, "'hidden"),
                entry(12, 2, "99"),
            ],
        )
        .unwrap();
        let hidden = b.cell_at(p(8, 2)).unwrap();
        b.fill_visible_sheet(
            b.worksheet(),
            b.expected(),
            &[5, 12],
            (2, 2),
            FillDirection::Down,
            &Default::default(),
        )
        .unwrap();
        let target = b.cell_at(p(12, 2)).unwrap().unwrap();
        assert_eq!(b.display_formula(target.reference).unwrap().unwrap(), "=A9");
        assert_eq!(b.cell_at(p(8, 2)).unwrap(), hidden);
        assert_eq!(b.cell_at(p(9, 1)).unwrap().unwrap().content, Content::Blank);
        b.apply(b.expected(), &[entry(5, 2, "=$B$12")]).unwrap();
        let before = b.expected();
        let target = b.cell_at(p(12, 2)).unwrap();
        assert!(
            b.fill_visible_sheet(
                b.worksheet(),
                before,
                &[5, 12],
                (2, 2),
                FillDirection::Down,
                &Default::default()
            )
            .is_err()
        );
        assert_eq!(b.expected(), before);
        assert_eq!(b.cell_at(p(12, 2)).unwrap(), target);
    });
}

#[test]
fn actual_gap_translation_up_and_horizontal_edges() {
    for direction in [FillDirection::Up, FillDirection::Left, FillDirection::Right] {
        run(Limits::capacity(), |b| {
            let rows = [2, 5, 12];
            let columns = if direction == FillDirection::Up {
                (3, 3)
            } else {
                (3, 5)
            };
            let source_col = if direction == FillDirection::Left {
                5
            } else {
                3
            };
            let source_target = if direction == FillDirection::Left {
                "D"
            } else {
                "B"
            };
            let edits = rows
                .iter()
                .map(|&r| entry(r, source_col, &format!("={source_target}{}", r - 1)))
                .collect::<Vec<_>>();
            b.apply(b.expected(), &edits).unwrap();
            b.fill_visible_sheet(
                b.worksheet(),
                b.expected(),
                &rows,
                columns,
                direction,
                &Default::default(),
            )
            .unwrap();
            for &r in &rows {
                for c in columns.0..=columns.1 {
                    let cell = b.cell_at(p(r, c)).unwrap().unwrap();
                    let letter = char::from(b'A' + (c - 2) as u8);
                    assert_eq!(
                        b.display_formula(cell.reference).unwrap().unwrap(),
                        format!("={letter}{}", r - 1)
                    );
                }
            }
        });
    }
}

#[test]
fn wide_clear_spans_preserve_outside_style_and_accumulate_admission() {
    run(Limits::capacity(), |b| {
        let sheet = b.worksheet();
        b.format_sheet(
            sheet,
            b.expected(),
            rect(p(1, 1), p(65536, 1024)),
            StylePatch {
                bold: Field::Set(true),
                ..Default::default()
            },
            false,
            &Default::default(),
        )
        .unwrap();
        b.clear_visible_sheet(
            sheet,
            b.expected(),
            &[1, 5, 12, 65536],
            (1, 1024),
            ClearMode::Formatting,
            &Default::default(),
        )
        .unwrap();
        for row in [1, 5, 12, 65536] {
            for col in [1, 512, 1024] {
                assert_eq!(b.style_at(sheet, p(row, col)).unwrap(), Style::default());
            }
        }
        for row in [2, 4, 6, 11, 13, 65535] {
            assert!(b.style_at(sheet, p(row, 512)).unwrap().bold);
        }
        let before = b.expected();
        b.clear_visible_sheet(
            sheet,
            before,
            &[1, 5, 12, 65536],
            (1, 1024),
            ClearMode::Formatting,
            &Default::default(),
        )
        .unwrap();
        assert_eq!(b.expected(), before);
        assert_eq!(b.stats().live_cells, 0);
    });
}
