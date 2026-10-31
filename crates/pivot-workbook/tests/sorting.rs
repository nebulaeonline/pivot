use pivot_core::model::integration::{CancellationToken, IntegrationRuntime};
use pivot_types::PivotValue;
use pivot_workbook::{formatting::*, sorting::*, *};
fn p(row: u32, column: u32) -> Position {
    Position::new(row, column)
}
fn entry(row: u32, column: u32, source: &str) -> Edit {
    Edit::SetEntry {
        position: p(row, column),
        source: source.into(),
    }
}
fn options(last: u32, columns: u32) -> Options {
    Options {
        rectangle: Rectangle {
            first: p(1, 1),
            last: p(last, columns),
        },
        exclude_header: false,
        keys: vec![Key {
            column: 1,
            descending: false,
        }],
        move_formatting: true,
    }
}
fn value(b: &Workbook, row: u32, column: u32) -> std::result::Result<PivotValue, CellError> {
    let c = b.capture_local(b.expected()).unwrap();
    c.evaluate_values(&[c.cell_at(p(row, column)).unwrap()])
        .unwrap()
        .values[0]
        .1
        .clone()
}
fn number(b: &Workbook, row: u32, column: u32) -> i64 {
    match value(b, row, column).unwrap() {
        PivotValue::Decimal(n) => n.to_string().parse().unwrap(),
        v => panic!("{v:?}"),
    }
}
fn finish(b: Workbook, rt: IntegrationRuntime) {
    drop(b);
    let memory = rt.memory();
    assert!(rt.finish().is_ok());
    assert_eq!(memory.stats().charged, 0);
}

#[test]
fn records_keep_scalar_bindings_while_ranges_read_current_occupants_and_old_captures_stay_pinned() {
    let rt = IntegrationRuntime::standalone().unwrap();
    let mut b = Workbook::new_in("sort", Limits::default(), &rt).unwrap();
    b.apply(
        b.expected(),
        &[
            entry(1, 1, "2"),
            entry(2, 1, "1"),
            entry(1, 2, "=A1*10"),
            entry(2, 2, "=A2*10"),
            entry(1, 4, "=A1"),
            entry(2, 4, "=SUM(A1:A1)"),
        ],
    )
    .unwrap();
    let first = b.cell_at(p(1, 1)).unwrap().unwrap();
    let second = b.cell_at(p(2, 1)).unwrap().unwrap();
    let capture = b.capture_local(b.expected()).unwrap();
    let stats = b.stats();
    let selections = b.selection_metrics().identities;
    let out = b
        .sort_rows(
            b.worksheet(),
            b.expected(),
            &options(2, 2),
            &Default::default(),
        )
        .unwrap();
    assert!(out.changed);
    assert_eq!(out.moved_rows, 2);
    assert_eq!(out.moved_cells, 4);
    assert_eq!(
        b.cell_at(p(1, 1)).unwrap().unwrap().reference,
        second.reference
    );
    assert_eq!(b.cell_at(p(2, 1)).unwrap().unwrap().node, first.node);
    assert_eq!(
        (
            number(&b, 1, 2),
            number(&b, 2, 2),
            number(&b, 1, 4),
            number(&b, 2, 4)
        ),
        (10, 20, 2, 1)
    );
    let external = b.cell_at(p(1, 4)).unwrap().unwrap().reference;
    assert_eq!(b.display_formula(external).unwrap().as_deref(), Some("=A2"));
    assert_eq!(
        capture
            .evaluate_values(&[capture.cell_at(p(2, 4)).unwrap()])
            .unwrap()
            .values[0]
            .1,
        Ok(PivotValue::Decimal(2.into()))
    );
    assert_eq!(b.stats().allocated_nodes, stats.allocated_nodes);
    assert_eq!(b.selection_metrics().identities, selections);
    assert!(
        !b.sort_rows(
            b.worksheet(),
            b.expected(),
            &options(2, 2),
            &Default::default()
        )
        .unwrap()
        .changed
    );
    drop(capture);
    finish(b, rt);
}
#[test]
fn stable_mixed_kind_order_keeps_errors_and_blanks_last_and_strings_exact() {
    for descending in [false, true] {
        let rt = IntegrationRuntime::standalone().unwrap();
        let mut b = Workbook::new_in("types", Limits::default(), &rt).unwrap();
        let input = [
            "TRUE", "'001234", "2", "=1/0", "'", "FALSE", "'1234", "1", "=#REF!", "",
        ];
        let mut edits = Vec::new();
        for (i, s) in input.iter().enumerate() {
            if !s.is_empty() {
                edits.push(entry(i as u32 + 1, 1, s));
            }
            edits.push(entry(i as u32 + 1, 2, &(i + 1).to_string()));
        }
        b.apply(b.expected(), &edits).unwrap();
        let before = b.stats();
        let mut o = options(10, 2);
        o.keys[0].descending = descending;
        b.sort_rows(b.worksheet(), b.expected(), &o, &Default::default())
            .unwrap();
        let expected = if descending {
            vec![1, 6, 7, 2, 5, 3, 8, 4, 9, 10]
        } else {
            vec![8, 3, 5, 2, 7, 6, 1, 4, 9, 10]
        };
        assert_eq!(
            (1..=10).map(|r| number(&b, r, 2)).collect::<Vec<_>>(),
            expected
        );
        let zero_row = 4;
        assert_eq!(
            value(&b, zero_row, 1),
            Ok(PivotValue::String("001234".into()))
        );
        assert_eq!(b.stats().allocated_nodes, before.allocated_nodes);
        finish(b, rt);
    }
}
#[test]
fn one_pre_sort_calculation_does_not_iterate_after_range_keys_recalculate() {
    let rt = IntegrationRuntime::standalone().unwrap();
    let mut b = Workbook::new_in("snapshot", Limits::default(), &rt).unwrap();
    b.apply(
        b.expected(),
        &[
            entry(1, 1, "=SUM(B1:B1)"),
            entry(2, 1, "=SUM(B2:B2)"),
            entry(1, 2, "2"),
            entry(2, 2, "1"),
        ],
    )
    .unwrap();
    let first = b.cell_at(p(1, 1)).unwrap().unwrap().reference;
    b.sort_rows(
        b.worksheet(),
        b.expected(),
        &options(2, 2),
        &Default::default(),
    )
    .unwrap();
    assert_eq!(b.cell_at(p(2, 1)).unwrap().unwrap().reference, first);
    assert_eq!(
        (number(&b, 1, 1), number(&b, 2, 1), number(&b, 1, 2)),
        (2, 1, 1)
    );
    finish(b, rt);
}
#[test]
fn multikey_header_and_formatting_modes_preserve_explicit_defaults_and_inheritance() {
    for move_formatting in [false, true] {
        let rt = IntegrationRuntime::standalone().unwrap();
        let mut b = Workbook::new_in("format", Limits::default(), &rt).unwrap();
        let sheet = b.worksheet();
        b.apply(
            b.expected(),
            &[
                entry(1, 1, "'Header"),
                entry(2, 1, "1"),
                entry(3, 1, "1"),
                entry(4, 1, "0"),
                entry(2, 2, "2"),
                entry(3, 2, "1"),
                entry(4, 2, "3"),
            ],
        )
        .unwrap();
        let header = b.cell_at(p(1, 1)).unwrap().unwrap().reference;
        b.format_sheet(
            sheet,
            b.expected(),
            Rectangle {
                first: p(2, 1),
                last: p(2, 2),
            },
            StylePatch {
                bold: Field::Set(false),
                ..Default::default()
            },
            false,
            &Default::default(),
        )
        .unwrap();
        b.format_sheet(
            sheet,
            b.expected(),
            Rectangle {
                first: p(1, 2),
                last: p(MAX_ROWS, 2),
            },
            StylePatch {
                italic: Field::Set(true),
                ..Default::default()
            },
            false,
            &Default::default(),
        )
        .unwrap();
        let mut o = options(4, 2);
        o.exclude_header = true;
        o.keys.push(Key {
            column: 2,
            descending: false,
        });
        o.move_formatting = move_formatting;
        b.sort_rows(sheet, b.expected(), &o, &Default::default())
            .unwrap();
        assert_eq!(b.cell_at(p(1, 1)).unwrap().unwrap().reference, header);
        assert_eq!(
            (2..=4).map(|r| number(&b, r, 2)).collect::<Vec<_>>(),
            vec![3, 1, 2]
        );
        b.set_worksheet_default(
            sheet,
            b.expected(),
            StylePatch {
                bold: Field::Set(true),
                ..Default::default()
            },
            &Default::default(),
        )
        .unwrap();
        for row in 2..=4 {
            assert_eq!(
                b.style_at(sheet, p(row, 1)).unwrap().bold,
                row != if move_formatting { 4 } else { 2 }
            );
            assert!(b.style_at(sheet, p(row, 2)).unwrap().italic);
        }
        finish(b, rt);
    }
}
#[test]
fn sparse_whole_sheet_has_no_area_allocation_and_failures_leave_complete_state_unchanged() {
    let rt = IntegrationRuntime::standalone().unwrap();
    let mut b = Workbook::new_in("sparse", Limits::default(), &rt).unwrap();
    b.apply(
        b.expected(),
        &[
            entry(100, 1, "2"),
            entry(MAX_ROWS, 1, "1"),
            entry(2, 2, "=A300"),
        ],
    )
    .unwrap();
    let mut o = options(MAX_ROWS, MAX_COLUMNS);
    o.move_formatting = false;
    let old = b.expected();
    let stats = b.stats();
    let cancel = CancellationToken::default();
    cancel.cancel();
    assert!(b.sort_rows(b.worksheet(), old, &o, &cancel).is_err());
    assert_eq!(b.expected(), old);
    let mut bad = o.clone();
    bad.keys.push(bad.keys[0]);
    assert!(
        b.sort_rows(b.worksheet(), old, &bad, &Default::default())
            .is_err()
    );
    assert_eq!(b.expected(), old);
    let outcome = b
        .sort_rows(b.worksheet(), old, &o, &Default::default())
        .unwrap();
    assert!(outcome.work < 5_000_000);
    assert_eq!((number(&b, 1, 1), number(&b, 2, 1)), (1, 2));
    assert_eq!(b.stats().allocated_nodes, stats.allocated_nodes);
    assert!(
        b.sort_rows(b.worksheet(), old, &o, &Default::default())
            .is_err()
    );
    finish(b, rt);
}

#[test]
fn generated_record_permutations_match_dense_ten_property_and_stable_tuple_oracles() {
    for seed in 0..24u32 {
        let rt = IntegrationRuntime::standalone().unwrap();
        let mut b = Workbook::new_in(
            "dense sort oracle",
            Limits {
                rows: 12,
                columns: 8,
                ..Default::default()
            },
            &rt,
        )
        .unwrap();
        let sheet = b.worksheet();
        let mut tuples = Vec::new();
        let mut edits = Vec::new();
        let mut styles = vec![vec![Style::default(); 8]; 12];
        for r in 0..12u32 {
            let a = ((r * 7 + seed * 3) % 5) as i64;
            let z = ((r * 3 + seed) % 4) as i64;
            tuples.push((a, z, r));
            edits.push(entry(r + 1, 2, &a.to_string()));
            edits.push(entry(r + 1, 3, &z.to_string()));
            edits.push(entry(r + 1, 4, &r.to_string()));
            for c in 0..8u32 {
                let s = Style {
                    numeric: if (r + c) % 2 == 0 {
                        NumericFormat::Number
                    } else {
                        NumericFormat::Percentage
                    },
                    places: ((r + c) % 5) as u8,
                    rounding: if r % 2 == 0 {
                        pivot_types::RoundingMode::ToEven
                    } else {
                        pivot_types::RoundingMode::AwayFromZero
                    },
                    grouping: r % 2 == 0,
                    currency: if c % 2 == 0 {
                        Currency::Euro
                    } else {
                        Currency::Pound
                    },
                    bold: r % 3 == 0,
                    italic: c % 2 == 0,
                    alignment: if r % 2 == 0 {
                        Alignment::Left
                    } else {
                        Alignment::Right
                    },
                    foreground: Some(r * 977 + c),
                    background: Some(c * 1009 + r),
                };
                styles[r as usize][c as usize] = s;
                b.format_sheet(
                    sheet,
                    b.expected(),
                    Rectangle {
                        first: p(r + 1, c + 1),
                        last: p(r + 1, c + 1),
                    },
                    StylePatch {
                        numeric: Field::Set(s.numeric),
                        places: Field::Set(s.places),
                        rounding: Field::Set(s.rounding),
                        grouping: Field::Set(s.grouping),
                        currency: Field::Set(s.currency),
                        bold: Field::Set(s.bold),
                        italic: Field::Set(s.italic),
                        alignment: Field::Set(s.alignment),
                        foreground: Field::Set(s.foreground),
                        background: Field::Set(s.background),
                    },
                    false,
                    &Default::default(),
                )
                .unwrap();
            }
        }
        b.apply(b.expected(), &edits).unwrap();
        let header = seed % 3 == 0;
        let move_formatting = seed % 2 == 0;
        let descending = seed % 4 < 2;
        let mut expected = tuples[usize::from(header)..].to_vec();
        expected.sort_by(|a, b| {
            let primary = if descending {
                b.0.cmp(&a.0)
            } else {
                a.0.cmp(&b.0)
            };
            primary.then(a.1.cmp(&b.1))
        });
        let o = Options {
            rectangle: Rectangle {
                first: p(1, 2),
                last: p(12, 6),
            },
            exclude_header: header,
            keys: vec![
                Key {
                    column: 2,
                    descending,
                },
                Key {
                    column: 3,
                    descending: false,
                },
            ],
            move_formatting,
        };
        b.sort_rows(sheet, b.expected(), &o, &Default::default())
            .unwrap();
        for (index, record) in expected.iter().enumerate() {
            let row = index + usize::from(header);
            assert_eq!(number(&b, row as u32 + 1, 4), record.2 as i64);
            for (col, _) in styles[row].iter().enumerate() {
                let source = if move_formatting && (1..6).contains(&col) {
                    record.2 as usize
                } else {
                    row
                };
                assert_eq!(
                    b.style_at(sheet, p(row as u32 + 1, col as u32 + 1))
                        .unwrap(),
                    styles[source][col],
                    "seed={seed} row={row} column={col}"
                );
            }
        }
        finish(b, rt);
    }
}

#[test]
fn all_virtual_keys_are_noop_and_adjacent_suggestions_do_not_cross_empty_columns_or_styles() {
    let rt = IntegrationRuntime::standalone().unwrap();
    let mut b = Workbook::new_in("virtual sort", Limits::default(), &rt).unwrap();
    let sheet = b.worksheet();
    b.format_sheet(
        sheet,
        b.expected(),
        Rectangle {
            first: p(1, 1),
            last: p(MAX_ROWS, MAX_COLUMNS),
        },
        StylePatch {
            bold: Field::Set(true),
            ..Default::default()
        },
        false,
        &Default::default(),
    )
    .unwrap();
    let before = b.expected();
    let regions = b.regions().collect::<Vec<_>>();
    assert!(
        !b.sort_rows(
            sheet,
            before,
            &options(MAX_ROWS, MAX_COLUMNS),
            &Default::default()
        )
        .unwrap()
        .changed
    );
    assert_eq!(b.expected(), before);
    assert_eq!(b.regions().collect::<Vec<_>>(), regions);
    assert_eq!(b.stats().allocated_nodes, 0);
    b.apply(b.expected(), &[entry(2, 2, "'001234"), entry(2, 4, "7")])
        .unwrap();
    let selected = Rectangle {
        first: p(1, 1),
        last: p(4, 1),
    };
    let suggestion = b
        .sort_bounds(sheet, b.expected(), selected, &Default::default())
        .unwrap();
    assert_eq!(suggestion.last.column, 2);
    assert_eq!(suggestion.first, selected.first);
    assert_eq!(
        b.sort_bounds(sheet, b.expected(), suggestion, &Default::default())
            .unwrap(),
        suggestion
    );
    finish(b, rt);
}
#[test]
fn sorting_an_explicit_sheet_keeps_cross_sheet_absolute_scalar_and_fixed_range_bindings() {
    let rt = IntegrationRuntime::standalone().unwrap();
    let mut b = Workbook::new_in("sheet sort", Limits::default(), &rt).unwrap();
    let data = b.worksheet();
    let name = b.sheets()[0].name.clone();
    b.apply_sheet(data, b.expected(), &[entry(1, 1, "2"), entry(2, 1, "1")])
        .unwrap();
    let original = b.cell_at_sheet(data, p(1, 1)).unwrap().unwrap().reference;
    let other = b.add_sheet(b.expected(), Some("Results")).unwrap();
    b.apply_sheet(
        other,
        b.expected(),
        &[
            entry(1, 1, &format!("='{name}'!$A$1")),
            entry(2, 1, &format!("=SUM('{name}'!A1:A1)")),
        ],
    )
    .unwrap();
    let definition = b.cell_at_sheet(other, p(1, 1)).unwrap().unwrap().content;
    let selections = b.selection_metrics().identities;
    b.sort_rows(data, b.expected(), &options(2, 1), &Default::default())
        .unwrap();
    assert_eq!(
        b.cell_at_sheet(data, p(2, 1)).unwrap().unwrap().reference,
        original
    );
    assert_eq!(
        b.cell_at_sheet(other, p(1, 1)).unwrap().unwrap().content,
        definition
    );
    assert_eq!(b.selection_metrics().identities, selections);
    let capture = b.capture_sheet(other, b.expected()).unwrap();
    let values = capture
        .evaluate_values(&[
            capture.cell_at(p(1, 1)).unwrap(),
            capture.cell_at(p(2, 1)).unwrap(),
        ])
        .unwrap();
    assert_eq!(values.values[0].1, Ok(PivotValue::Decimal(2.into())));
    assert_eq!(values.values[1].1, Ok(PivotValue::Decimal(1.into())));
    assert!(
        b.display_formula(capture.cell_at(p(1, 1)).unwrap())
            .unwrap()
            .unwrap()
            .contains("$A$2")
    );
    drop(capture);
    finish(b, rt);
}

#[test]
fn formatting_fragment_refusal_is_atomic_and_requires_explicit_keep_formatting_retry() {
    let rt = IntegrationRuntime::standalone().unwrap();
    let mut b = Workbook::new_in(
        "fragment refusal",
        Limits {
            styles: StyleLimits {
                positions: 8,
                ..Default::default()
            },
            ..Default::default()
        },
        &rt,
    )
    .unwrap();
    let sheet = b.worksheet();
    b.apply(
        b.expected(),
        &(1..=32)
            .map(|r| entry(r, 1, &((r * 17) % 32).to_string()))
            .collect::<Vec<_>>(),
    )
    .unwrap();
    b.format_sheet(
        sheet,
        b.expected(),
        Rectangle {
            first: p(1, 2),
            last: p(16, 2),
        },
        StylePatch {
            bold: Field::Set(true),
            ..Default::default()
        },
        false,
        &Default::default(),
    )
    .unwrap();
    let before = b.expected();
    let regions = b.regions().collect::<Vec<_>>();
    let first = b.cell_at(p(1, 1)).unwrap();
    let mut o = options(32, 2);
    assert!(b.sort_rows(sheet, before, &o, &Default::default()).is_err());
    assert_eq!(b.expected(), before);
    assert_eq!(b.regions().collect::<Vec<_>>(), regions);
    assert_eq!(b.cell_at(p(1, 1)).unwrap(), first);
    o.move_formatting = false;
    assert!(
        b.sort_rows(sheet, before, &o, &Default::default())
            .unwrap()
            .changed
    );
    assert_eq!(b.regions().collect::<Vec<_>>(), regions);
    finish(b, rt);
}

#[test]
fn fixed_range_permutation_keeps_canonical_editable_cycles_and_unrelated_evaluation() {
    let rt = IntegrationRuntime::standalone().unwrap();
    let mut b = Workbook::new_in("range cycle", Limits::default(), &rt).unwrap();
    b.apply(
        b.expected(),
        &[
            entry(1, 1, "=SUM(B1:B1)"),
            entry(2, 1, "=SUM(A1:A1)"),
            entry(1, 2, "2"),
            entry(2, 2, "1"),
            entry(1, 5, "7"),
        ],
    )
    .unwrap();
    let before = b.expected();
    let first = b.cell_at(p(1, 1)).unwrap().unwrap().reference;
    let second = b.cell_at(p(2, 1)).unwrap().unwrap().reference;
    let selections = b.selection_metrics().identities;
    let mut o = options(2, 2);
    o.keys[0].column = 2;
    assert!(
        b.sort_rows(b.worksheet(), before, &o, &Default::default())
            .unwrap()
            .changed
    );
    assert_eq!(b.cell_at(p(2, 1)).unwrap().unwrap().reference, first);
    assert_eq!(b.cell_at(p(1, 1)).unwrap().unwrap().reference, second);
    assert_eq!(b.selection_metrics().identities, selections);
    let capture = b.capture_local(b.expected()).unwrap();
    assert!(capture.evaluate_values(&[second]).is_err());
    assert_eq!(number(&b, 1, 5), 7);
    drop(capture);
    finish(b, rt);
}
#[test]
fn unicode_string_order_is_scalar_ordinal_and_work_exhaustion_and_midflight_cancel_are_atomic() {
    let rt = IntegrationRuntime::standalone().unwrap();
    let mut b = Workbook::new_in("bounded keys", Limits::default(), &rt).unwrap();
    let inputs = ["é", "e\u{301}", "😀", "Z", "a", "", "001234"];
    b.apply(
        b.expected(),
        &inputs
            .iter()
            .enumerate()
            .map(|(i, s)| Edit::SetText {
                position: p(i as u32 + 1, 1),
                text: (*s).into(),
            })
            .collect::<Vec<_>>(),
    )
    .unwrap();
    let mut expected = inputs.to_vec();
    expected.sort_by(|a, b| a.chars().cmp(b.chars()));
    b.sort_rows(
        b.worksheet(),
        b.expected(),
        &options(inputs.len() as u32, 1),
        &Default::default(),
    )
    .unwrap();
    for (i, s) in expected.iter().enumerate() {
        assert_eq!(
            value(&b, i as u32 + 1, 1),
            Ok(PivotValue::String((*s).into()))
        );
    }
    finish(b, rt);
    let rt = IntegrationRuntime::standalone().unwrap();
    let mut b = Workbook::new_in("long prefix", Limits::default(), &rt).unwrap();
    let prefix = "x".repeat(4000);
    let edits = (1..=2048)
        .map(|r| Edit::SetText {
            position: p(r, 1),
            text: format!("{prefix}{:04}", 2049 - r),
        })
        .collect::<Vec<_>>();
    for chunk in edits.chunks(16) {
        b.apply(b.expected(), chunk).unwrap();
    }
    drop(edits);
    let before = b.expected();
    let first = b.cell_at(p(1, 1)).unwrap();
    let o = options(2048, 1);
    assert!(matches!(
        b.sort_rows(b.worksheet(), before, &o, &Default::default()),
        Err(Error::Capacity {
            resource: "sorting work",
            ..
        })
    ));
    assert_eq!(b.expected(), before);
    assert_eq!(b.cell_at(p(1, 1)).unwrap(), first);
    let cancel = CancellationToken::default();
    let signal = cancel.clone();
    let thread = std::thread::spawn(move || {
        std::thread::sleep(std::time::Duration::from_millis(2));
        signal.cancel();
    });
    assert!(b.sort_rows(b.worksheet(), before, &o, &cancel).is_err());
    thread.join().unwrap();
    assert_eq!(b.expected(), before);
    assert_eq!(b.cell_at(p(1, 1)).unwrap(), first);
    finish(b, rt);
}
