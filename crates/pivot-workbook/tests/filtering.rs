use pivot_core::model::integration::{CancellationToken, IntegrationRuntime};
use pivot_types::PivotValue;
use pivot_workbook::{filtering::*, *};
fn p(r: u32, c: u32) -> Position {
    Position::new(r, c)
}
fn entry(r: u32, c: u32, s: &str) -> Edit {
    Edit::SetEntry {
        position: p(r, c),
        source: s.into(),
    }
}
fn pred(operator: Operator, literal: Option<Literal>) -> Predicate {
    Predicate { operator, literal }
}
fn definition(last: u32, predicate: Predicate) -> Definition {
    Definition {
        rectangle: Rectangle {
            first: p(1, 1),
            last: p(last, 2),
        },
        exclude_header: false,
        needs_review: false,
        conditions: vec![Condition {
            column: 1,
            either: false,
            predicates: vec![predicate],
        }],
    }
}
fn finish(b: Workbook, rt: IntegrationRuntime) {
    drop(b);
    let memory = rt.memory();
    assert!(rt.finish().is_ok());
    assert_eq!(memory.stats().charged, 0);
}
#[test]
fn exact_strings_typed_negatives_blank_empty_and_completed_errors_preserve_all_identities() {
    let rt = IntegrationRuntime::standalone().unwrap();
    let mut b = Workbook::new_in("filter", Limits::default(), &rt).unwrap();
    let sheet = b.worksheet();
    b.apply(
        b.expected(),
        &[
            entry(1, 1, "'001234"),
            entry(2, 1, "1234"),
            entry(3, 1, "'"),
            entry(4, 1, "TRUE"),
            entry(5, 1, "=1/0"),
            entry(7, 1, "'1234"),
            entry(1, 2, "=A2"),
            entry(2, 2, "=SUM(A2:A2)"),
        ],
    )
    .unwrap();
    let old = b.capture_local(b.expected()).unwrap();
    let graph = b.expected().graph;
    let ids = (1..=7)
        .map(|r| b.cell_at(p(r, 1)).unwrap())
        .collect::<Vec<_>>();
    for (predicate, expected) in [
        (
            pred(Operator::Equal, Some(Literal::Text("001234".into()))),
            vec![1],
        ),
        (
            pred(Operator::Equal, Some(Literal::Decimal(1234.into()))),
            vec![2],
        ),
        (
            pred(Operator::NotEqual, Some(Literal::Text("001234".into()))),
            vec![3, 7],
        ),
        (
            pred(Operator::Equal, Some(Literal::Text("".into()))),
            vec![3],
        ),
        (pred(Operator::Blank, None), vec![6]),
        (pred(Operator::NotBlank, None), vec![1, 2, 3, 4, 5, 7]),
        (pred(Operator::Error, None), vec![5]),
        (pred(Operator::NotError, None), vec![1, 2, 3, 4, 6, 7]),
        (pred(Operator::Equal, Some(Literal::Boolean(true))), vec![4]),
        (
            pred(Operator::NotEqual, Some(Literal::Boolean(false))),
            vec![4],
        ),
        (
            pred(Operator::Contains, Some(Literal::Text("".into()))),
            vec![1, 3, 7],
        ),
    ] {
        let d = definition(7, predicate);
        b.set_filter(sheet, b.expected(), Some(&d), &Default::default())
            .unwrap();
        let map = b
            .filter_visibility(sheet, b.expected(), &Default::default())
            .unwrap();
        assert_eq!(
            map.rows()
                .iter()
                .copied()
                .filter(|r| *r <= 7)
                .collect::<Vec<_>>(),
            expected
        );
        assert_eq!(b.expected().graph, graph);
        assert_eq!(
            (1..=7)
                .map(|r| b.cell_at(p(r, 1)).unwrap())
                .collect::<Vec<_>>(),
            ids
        );
        for (i, row) in map.rows().iter().enumerate() {
            assert_eq!(map.ordinal(*row), Some(i));
        }
    }
    assert_eq!(
        old.evaluate_values(&[old.cell_at(p(1, 2)).unwrap(), old.cell_at(p(2, 2)).unwrap()])
            .unwrap()
            .values
            .iter()
            .map(|x| x.1.clone())
            .collect::<Vec<_>>(),
        vec![Ok(PivotValue::Decimal(1234.into())); 2]
    );
    drop(old);
    finish(b, rt);
}
#[test]
fn headers_cross_column_and_or_and_generated_sparse_rows_match_independent_integer_oracle() {
    let rt = IntegrationRuntime::standalone().unwrap();
    let mut b = Workbook::new_in("oracle", Limits::default(), &rt).unwrap();
    let sheet = b.worksheet();
    let mut edits = vec![entry(1, 1, "'Header")];
    for r in 2..=40 {
        if r % 4 != 0 {
            edits.push(entry(r, 1, &(r as i32 - 15).to_string()));
        }
        edits.push(entry(r, 2, if r % 3 == 0 { "TRUE" } else { "FALSE" }));
    }
    b.apply(b.expected(), &edits).unwrap();
    for threshold in [-10, 0, 10, 20] {
        for either in [false, true] {
            let d = Definition {
                rectangle: Rectangle {
                    first: p(1, 1),
                    last: p(40, 2),
                },
                exclude_header: true,
                needs_review: false,
                conditions: vec![
                    Condition {
                        column: 1,
                        either,
                        predicates: vec![
                            pred(
                                Operator::GreaterEqual,
                                Some(Literal::Decimal(threshold.into())),
                            ),
                            pred(Operator::Blank, None),
                        ],
                    },
                    Condition {
                        column: 2,
                        either: false,
                        predicates: vec![pred(Operator::Equal, Some(Literal::Boolean(true)))],
                    },
                ],
            };
            b.set_filter(sheet, b.expected(), Some(&d), &Default::default())
                .unwrap();
            let map = b
                .filter_visibility(sheet, b.expected(), &Default::default())
                .unwrap();
            let expected = (1..=40)
                .filter(|r| {
                    if *r == 1 {
                        return true;
                    }
                    let blank = r % 4 == 0;
                    let numeric = !blank && (*r as i64 - 15) >= threshold;
                    let a = if either {
                        numeric || blank
                    } else {
                        numeric && blank
                    };
                    a && r % 3 == 0
                })
                .collect::<Vec<_>>();
            assert_eq!(
                map.rows()
                    .iter()
                    .copied()
                    .filter(|r| *r <= 40)
                    .collect::<Vec<_>>(),
                expected
            );
            assert_eq!(map.hidden_in(1, 40).unwrap(), 40 - expected.len() as u32);
        }
    }
    finish(b, rt);
}
#[test]
fn sparse_virtual_full_extent_zero_graph_nodes_noops_stale_cancel_and_exhaustion_are_atomic() {
    let rt = IntegrationRuntime::standalone().unwrap();
    let memory = rt.memory();
    let mut b = Workbook::new_in("sparse", Limits::default(), &rt).unwrap();
    let sheet = b.worksheet();
    let mut d = definition(MAX_ROWS, pred(Operator::Blank, None));
    d.rectangle.last.column = MAX_COLUMNS;
    let before = b.expected();
    b.set_filter(sheet, before, Some(&d), &Default::default())
        .unwrap();
    assert_eq!(b.stats().allocated_nodes, 0);
    let expected = b.expected();
    assert_eq!(
        b.set_filter(sheet, expected, Some(&d), &Default::default())
            .unwrap(),
        expected
    );
    let map = b
        .filter_visibility(sheet, expected, &Default::default())
        .unwrap();
    assert_eq!(map.rows().len(), MAX_ROWS as usize);
    assert!(map.work < MAX_ROWS as usize + 100);
    drop(map);
    assert!(
        b.set_filter(sheet, before, None, &Default::default())
            .is_err()
    );
    let cancel = CancellationToken::default();
    cancel.cancel();
    assert!(b.filter_visibility(sheet, expected, &cancel).is_err());
    assert!(b.set_filter(sheet, expected, None, &cancel).is_err());
    let pressure = memory
        .reserve(memory.stats().limit - memory.stats().charged)
        .unwrap();
    assert!(
        b.filter_visibility(sheet, expected, &Default::default())
            .is_err()
    );
    drop(pressure);
    assert_eq!(b.expected(), expected);
    assert_eq!(b.filter(sheet).unwrap(), Some(&d));
    finish(b, rt);
}
#[test]
fn actual_structure_suspends_definition_without_retargeting_and_sort_does_not() {
    let rt = IntegrationRuntime::standalone().unwrap();
    let mut b = Workbook::new_in("review", Limits::default(), &rt).unwrap();
    let sheet = b.worksheet();
    b.apply(b.expected(), &[entry(1, 1, "2"), entry(2, 1, "1")])
        .unwrap();
    let d = definition(2, pred(Operator::Greater, Some(Literal::Decimal(1.into()))));
    b.set_filter(sheet, b.expected(), Some(&d), &Default::default())
        .unwrap();
    let before = b.expected();
    let noop = b
        .restructure(
            sheet,
            before,
            structure::Change {
                axis: structure::AxisKind::Rows,
                action: structure::Action::Insert,
                start: MAX_ROWS,
                count: 1,
            },
            &Default::default(),
        )
        .unwrap();
    assert!(!noop.changed);
    assert_eq!(b.expected(), before);
    assert!(!b.filter(sheet).unwrap().unwrap().needs_review);
    b.sort_rows(
        sheet,
        b.expected(),
        &sorting::Options {
            rectangle: d.rectangle,
            exclude_header: false,
            keys: vec![sorting::Key {
                column: 1,
                descending: false,
            }],
            move_formatting: true,
        },
        &Default::default(),
    )
    .unwrap();
    assert!(!b.filter(sheet).unwrap().unwrap().needs_review);
    b.restructure(
        sheet,
        b.expected(),
        structure::Change {
            axis: structure::AxisKind::Rows,
            action: structure::Action::Insert,
            start: 1,
            count: 1,
        },
        &Default::default(),
    )
    .unwrap();
    let stored = b.filter(sheet).unwrap().unwrap();
    assert!(stored.needs_review);
    assert_eq!(stored.rectangle, d.rectangle);
    let map = b
        .filter_visibility(sheet, b.expected(), &Default::default())
        .unwrap();
    assert_eq!(map.rows().len(), MAX_ROWS as usize);
    drop(map);
    finish(b, rt);
}

#[test]
fn definition_quota_is_aggregate_canonical_and_failure_preserves_both_sheets() {
    let rt = IntegrationRuntime::standalone().unwrap();
    let mut b = Workbook::new_in("quota", Limits::default(), &rt).unwrap();
    let sheet = b.worksheet();
    let other = b.add_sheet(b.expected(), Some("Other")).unwrap();
    let make = |columns: u32| Definition {
        rectangle: Rectangle {
            first: p(1, 1),
            last: p(1, columns),
        },
        exclude_header: true,
        needs_review: false,
        conditions: (1..=columns)
            .map(|column| Condition {
                column,
                either: false,
                predicates: vec![pred(Operator::Equal, Some(Literal::Text("x".repeat(4096))))],
            })
            .collect(),
    };
    let d = make(240);
    assert!(d.storage_bytes() < MAX_DEFINITION_BYTES);
    b.set_filter(sheet, b.expected(), Some(&d), &Default::default())
        .unwrap();
    let expected = b.expected();
    let mut spare = d.clone();
    spare.conditions.reserve(512);
    assert_eq!(spare.storage_bytes(), d.storage_bytes());
    assert_eq!(
        b.set_filter(sheet, expected, Some(&spare), &Default::default())
            .unwrap(),
        expected
    );
    let excessive = make(16);
    assert!(d.storage_bytes() + excessive.storage_bytes() > MAX_DEFINITION_BYTES);
    assert!(
        b.set_filter(other, expected, Some(&excessive), &Default::default())
            .is_err()
    );
    assert_eq!(b.expected(), expected);
    assert_eq!(b.filter(sheet).unwrap(), Some(&d));
    assert!(b.filter(other).unwrap().is_none());
    assert_eq!(b.stats().allocated_nodes, 0);
    finish(b, rt);
}

#[test]
fn unicode_text_is_ordinal_exact_and_long_prefix_work_refuses_atomically() {
    let rt = IntegrationRuntime::standalone().unwrap();
    let mut b = Workbook::new_in("text", Limits::default(), &rt).unwrap();
    let sheet = b.worksheet();
    b.apply(
        b.expected(),
        &[
            entry(1, 1, "'e\u{301}"),
            entry(2, 1, "'\u{e9}"),
            entry(3, 1, "'E\u{301}"),
        ],
    )
    .unwrap();
    for (operator, text, expected) in [
        (Operator::Equal, "\u{e9}", vec![2]),
        (Operator::StartsWith, "e", vec![1]),
        (Operator::Contains, "\u{301}", vec![1, 3]),
        (Operator::Less, "\u{e9}", vec![1, 3]),
    ] {
        let d = definition(3, pred(operator, Some(Literal::Text(text.into()))));
        let map = b
            .preview_filter(sheet, b.expected(), &d, &Default::default())
            .unwrap();
        assert_eq!(
            map.rows()
                .iter()
                .copied()
                .filter(|r| *r <= 3)
                .collect::<Vec<_>>(),
            expected
        );
    }
    b.apply(
        b.expected(),
        &(1..=9)
            .map(|row| Edit::SetText {
                position: p(row, 1),
                text: "a".repeat(4096),
            })
            .collect::<Vec<_>>(),
    )
    .unwrap();
    let before = b.expected();
    let d = definition(
        9,
        pred(
            Operator::Contains,
            Some(Literal::Text("a".repeat(2048) + "b")),
        ),
    );
    assert!(matches!(
        b.set_filter(sheet, before, Some(&d), &Default::default()),
        Err(Error::Capacity {
            resource: "filter work",
            ..
        })
    ));
    assert_eq!(b.expected(), before);
    assert!(b.filter(sheet).unwrap().is_none());
    finish(b, rt);
}
