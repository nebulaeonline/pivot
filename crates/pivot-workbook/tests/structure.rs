use pivot_core::model::integration::IntegrationRuntime;
use pivot_workbook::{formatting::*, structure::*, *};
fn p(row: u32, column: u32) -> Position {
    Position::new(row, column)
}
fn e(row: u32, column: u32, source: &str) -> Edit {
    Edit::SetEntry {
        position: p(row, column),
        source: source.into(),
    }
}
#[test]
fn inherited_profiles_keep_explicit_defaults_distinct_and_cleared_tail_anchors_are_protected() {
    let rt = IntegrationRuntime::standalone().unwrap();
    let mut b = Workbook::new_in("explicit defaults", Limits::default(), &rt).unwrap();
    let sheet = b.worksheet();
    b.format_sheet(
        sheet,
        b.expected(),
        Rectangle {
            first: p(2, 1),
            last: p(2, 1),
        },
        StylePatch {
            bold: Field::Set(false),
            ..Default::default()
        },
        false,
        &Default::default(),
    )
    .unwrap();
    edit(&mut b, AxisKind::Rows, Action::Insert, 3, 1);
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
    assert!(!b.style_at(sheet, p(2, 1)).unwrap().bold);
    assert!(!b.style_at(sheet, p(3, 1)).unwrap().bold);
    assert!(b.style_at(sheet, p(4, 1)).unwrap().bold);
    b.apply(b.expected(), &[e(1, 2, "=A65536")]).unwrap();
    let blank = b.cell_at(p(65536, 1)).unwrap().unwrap();
    let expected = b.expected();
    let stats = b.stats();
    assert!(
        b.restructure(
            sheet,
            expected,
            Change {
                axis: AxisKind::Rows,
                action: Action::Insert,
                start: 2,
                count: 1
            },
            &Default::default()
        )
        .is_err()
    );
    assert_eq!(b.expected(), expected);
    assert_eq!(b.stats().allocated_nodes, stats.allocated_nodes);
    assert_eq!(b.cell_at(p(65536, 1)).unwrap().unwrap(), blank);
    drop(b);
    let memory = rt.memory();
    assert!(rt.finish().is_ok());
    assert_eq!(memory.stats().charged, 0);
}
#[test]
fn cross_sheet_reversed_mixed_absolute_ranges_match_surviving_member_sums_on_both_axes() {
    for columns in [false, true] {
        for delete in [false, true] {
            for (start, count) in [(1, 1), (2, 2), (3, 1), (3, 2), (5, 3), (7, 2), (2, 5)] {
                let rt = IntegrationRuntime::standalone().unwrap();
                let mut b = Workbook::new_in(
                    "range oracle",
                    Limits {
                        rows: 12,
                        columns: 12,
                        ..Default::default()
                    },
                    &rt,
                )
                .unwrap();
                let data = b.worksheet();
                let data_name = b.sheets()[0].name.clone();
                let edits = (1..=6)
                    .map(|i| {
                        e(
                            if columns { 2 } else { i },
                            if columns { i } else { 2 },
                            &i.to_string(),
                        )
                    })
                    .collect::<Vec<_>>();
                b.apply(b.expected(), &edits).unwrap();
                let results = b.add_sheet(b.expected(), Some("Results")).unwrap();
                let source = if columns {
                    format!("=SUM('{data_name}'!$F$2:B$2,'{data_name}'!C2:D2)")
                } else {
                    format!("=SUM('{data_name}'!$B$6:B$2,'{data_name}'!B3:B4)")
                };
                b.apply_sheet(results, b.expected(), &[e(1, 1, &source)])
                    .unwrap();
                let formula = b
                    .cell_at_sheet(results, p(1, 1))
                    .unwrap()
                    .unwrap()
                    .reference;
                let ids = b.selection_metrics().identities;
                b.restructure(
                    data,
                    b.expected(),
                    Change {
                        axis: if columns {
                            AxisKind::Columns
                        } else {
                            AxisKind::Rows
                        },
                        action: if delete {
                            Action::Delete
                        } else {
                            Action::Insert
                        },
                        start,
                        count,
                    },
                    &Default::default(),
                )
                .unwrap();
                assert_eq!(b.selection_metrics().identities, ids);
                let kept = |a, b| {
                    (a..=b)
                        .filter(|i| !delete || *i < start || *i >= start + count)
                        .collect::<Vec<u32>>()
                };
                let first = kept(2, 6);
                let second = kept(3, 4);
                let capture = b.capture_sheet(results, b.expected()).unwrap();
                let actual = capture.evaluate_values(&[formula]).unwrap().values[0]
                    .1
                    .clone();
                let expected = if first.is_empty() || second.is_empty() {
                    Err(CellError::Typed(pivot_core::typed::Error::ReferenceDeleted))
                } else {
                    Ok(pivot_types::PivotValue::Decimal(
                        (first.iter().chain(&second).sum::<u32>() as i64).into(),
                    ))
                };
                assert_eq!(
                    actual, expected,
                    "columns={columns}, delete={delete}, start={start}, count={count}"
                );
                let Content::TextFormula(authored) =
                    b.cell_at_sheet(results, p(1, 1)).unwrap().unwrap().content
                else {
                    panic!()
                };
                assert_eq!(authored.original_source, source);
                drop((capture, b));
                let memory = rt.memory();
                assert!(rt.finish().is_ok());
                assert_eq!(memory.stats().charged, 0);
            }
        }
    }
}
#[test]
fn both_axes_match_an_independent_dense_ten_property_formatting_oracle() {
    let rt = IntegrationRuntime::standalone().unwrap();
    let limits = Limits {
        rows: 12,
        columns: 10,
        ..Limits::default()
    };
    let mut b = Workbook::new_in("dense structural oracle", limits, &rt).unwrap();
    let sheet = b.worksheet();
    let mut dense = vec![vec![Style::default(); 10]; 12];
    for row in 1..=12 {
        for col in 1..=10 {
            let n: u32 = row * 13 + col * 7;
            let mut bytes = Style::default().encode();
            bytes[..8].copy_from_slice(&[
                (n % 4) as u8,
                (n % 29) as u8,
                (n % 7) as u8,
                (n % 2) as u8,
                (n % 3) as u8,
                ((n / 2) % 2) as u8,
                ((n / 3) % 2) as u8,
                ((n / 5) % 4) as u8,
            ]);
            bytes[9..13].copy_from_slice(&(n * 31).to_le_bytes());
            bytes[13..17].copy_from_slice(&(n * 37).to_le_bytes());
            let s = Style::decode(&bytes).unwrap();
            dense[row as usize - 1][col as usize - 1] = s;
            b.format_sheet(
                sheet,
                b.expected(),
                Rectangle {
                    first: p(row, col),
                    last: p(row, col),
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
    let mut seed = 7281u32;
    for step in 0..48 {
        seed = seed.wrapping_mul(1664525).wrapping_add(1013904223);
        let columns = step % 2 == 0;
        let length = if columns { 10 } else { 12 };
        let start = if step % 11 == 0 {
            0
        } else {
            seed as usize % length
        };
        let count = if step % 11 == 0 {
            length
        } else {
            1 + (seed as usize / 17) % (length - start)
        };
        let insert = step % 3 != 0;
        let old = dense.clone();
        for (r, row) in dense.iter_mut().enumerate() {
            for (c, cell) in row.iter_mut().enumerate() {
                let x = if columns { c } else { r };
                let origin = if insert {
                    Some(if x < start {
                        x
                    } else if x < start + count {
                        start.saturating_sub(1)
                    } else {
                        x - count
                    })
                } else if x < start {
                    Some(x)
                } else if x < length - count {
                    Some(x + count)
                } else if count == length {
                    None
                } else {
                    Some(if start + count == length {
                        start - 1
                    } else {
                        length - 1
                    })
                };
                *cell =
                    origin.map_or_else(
                        Style::default,
                        |i| if columns { old[r][i] } else { old[i][c] },
                    );
            }
        }
        edit(
            &mut b,
            if columns {
                AxisKind::Columns
            } else {
                AxisKind::Rows
            },
            if insert {
                Action::Insert
            } else {
                Action::Delete
            },
            start as u32 + 1,
            count as u32,
        );
        for (r, row) in dense.iter().enumerate() {
            for (c, expected) in row.iter().enumerate() {
                assert_eq!(
                    b.style_at(sheet, p(r as u32 + 1, c as u32 + 1)).unwrap(),
                    *expected,
                    "step {step}, row {r}, column {c}"
                );
            }
        }
        assert_eq!(b.stats().allocated_nodes, 0);
    }
    drop(b);
    let memory = rt.memory();
    assert!(rt.finish().is_ok());
    assert_eq!(memory.stats().charged, 0);
}
fn edit(b: &mut Workbook, axis: AxisKind, action: Action, start: u32, count: u32) -> Outcome {
    b.restructure(
        b.worksheet(),
        b.expected(),
        Change {
            axis,
            action,
            start,
            count,
        },
        &Default::default(),
    )
    .unwrap()
}
fn value(
    b: &Workbook,
    row: u32,
    column: u32,
) -> std::result::Result<pivot_types::PivotValue, CellError> {
    let c = b.capture_local(b.expected()).unwrap();
    c.evaluate_values(&[c.cell_at(p(row, column)).unwrap()])
        .unwrap()
        .values[0]
        .1
        .clone()
}
#[test]
fn structural_references_preserve_identity_and_deletion_is_a_local_lazy_fault() {
    let rt = IntegrationRuntime::standalone().unwrap();
    let mut b = Workbook::new_in("structure", Limits::default(), &rt).unwrap();
    b.apply(
        b.expected(),
        &[
            e(2, 1, "7"),
            e(5, 2, "=$A$2*2"),
            e(5, 3, "=IF(FALSE,A2,1)"),
            e(7, 4, "=1+2"),
        ],
    )
    .unwrap();
    let target = b.cell_at(p(2, 1)).unwrap().unwrap().reference;
    let formula = b.cell_at(p(5, 2)).unwrap().unwrap().reference;
    let old = b.capture_local(b.expected()).unwrap();
    let result = edit(&mut b, AxisKind::Rows, Action::Insert, 2, 2);
    assert!(result.changed);
    assert_eq!(b.cell_at(p(4, 1)).unwrap().unwrap().reference, target);
    assert_eq!(b.display_formula(formula).unwrap().unwrap(), "=$A$4*2");
    assert_eq!(
        value(&b, 7, 2),
        Ok(pivot_types::PivotValue::Decimal(14.into()))
    );
    edit(&mut b, AxisKind::Rows, Action::Delete, 4, 1);
    assert_eq!(b.display_formula(formula).unwrap().unwrap(), "=#REF!*2");
    assert_eq!(
        value(&b, 6, 2),
        Err(CellError::Typed(pivot_core::typed::Error::ReferenceDeleted))
    );
    assert_eq!(
        value(&b, 6, 3),
        Ok(pivot_types::PivotValue::Decimal(1.into()))
    );
    assert_eq!(
        value(&b, 8, 4),
        Ok(pivot_types::PivotValue::Decimal(3.into()))
    );
    b.apply(b.expected(), &[e(4, 1, "100")]).unwrap();
    assert_eq!(
        value(&b, 6, 2),
        Err(CellError::Typed(pivot_core::typed::Error::ReferenceDeleted))
    );
    b.apply(b.expected(), &[e(6, 2, "=A4*2")]).unwrap();
    assert_eq!(
        value(&b, 6, 2),
        Ok(pivot_types::PivotValue::Decimal(200.into()))
    );
    assert_eq!(old.evaluate(&[target]).unwrap().values[0].1, Ok(7.into()));
    drop((old, b));
    let memory = rt.memory();
    assert!(rt.finish().is_ok());
    assert_eq!(memory.stats().charged, 0);
}
#[test]
fn range_affinity_reframes_without_new_identities_and_fully_deleted_range_faults() {
    let rt = IntegrationRuntime::standalone().unwrap();
    let mut b = Workbook::new_in("range", Limits::default(), &rt).unwrap();
    b.apply(
        b.expected(),
        &[
            e(2, 1, "2"),
            e(3, 1, "3"),
            e(4, 1, "4"),
            e(5, 1, "5"),
            e(1, 2, "=SUM(A2:A5)"),
            e(1, 3, "=SUM(A2:A5,1)"),
        ],
    )
    .unwrap();
    let ids = b.selection_metrics().identities;
    edit(&mut b, AxisKind::Rows, Action::Insert, 4, 1);
    assert_eq!(b.selection_metrics().identities, ids);
    assert_eq!(
        b.display_formula(b.cell_at(p(1, 2)).unwrap().unwrap().reference)
            .unwrap()
            .unwrap(),
        "=SUM(A2:A6)"
    );
    assert_eq!(
        value(&b, 1, 2),
        Ok(pivot_types::PivotValue::Decimal(14.into()))
    );
    edit(&mut b, AxisKind::Rows, Action::Delete, 2, 5);
    assert_eq!(
        value(&b, 1, 2),
        Err(CellError::Typed(pivot_core::typed::Error::ReferenceDeleted))
    );
    assert_eq!(
        value(&b, 1, 3),
        Err(CellError::Typed(pivot_core::typed::Error::ReferenceDeleted))
    );
    assert_eq!(
        b.display_formula(b.cell_at(p(1, 3)).unwrap().unwrap().reference)
            .unwrap()
            .unwrap(),
        "=SUM(#REF!,1)"
    );
    drop(b);
    let m = rt.memory();
    assert!(rt.finish().is_ok());
    assert_eq!(m.stats().charged, 0);
}
#[test]
fn style_profiles_clipping_protected_tail_and_complete_state_noops() {
    let rt = IntegrationRuntime::standalone().unwrap();
    let mut b = Workbook::new_in("styles", Limits::default(), &rt).unwrap();
    let sheet = b.worksheet();
    let before = b.expected();
    assert!(!edit(&mut b, AxisKind::Rows, Action::Insert, 2, 1).changed);
    assert_eq!(b.expected(), before);
    b.format_sheet(
        sheet,
        b.expected(),
        Rectangle {
            first: p(2, 1),
            last: p(4, 10),
        },
        StylePatch {
            bold: Field::Set(true),
            ..Default::default()
        },
        false,
        &Default::default(),
    )
    .unwrap();
    let count = b.stats().allocated_nodes;
    assert!(edit(&mut b, AxisKind::Rows, Action::Insert, 3, 2).changed);
    assert!(b.style_at(sheet, p(3, 1)).unwrap().bold);
    assert!(b.style_at(sheet, p(6, 10)).unwrap().bold);
    assert_eq!(b.stats().allocated_nodes, count);
    b.format_sheet(
        sheet,
        b.expected(),
        Rectangle {
            first: p(65536, 1),
            last: p(65536, 1),
        },
        StylePatch {
            italic: Field::Set(true),
            ..Default::default()
        },
        false,
        &Default::default(),
    )
    .unwrap();
    assert!(edit(&mut b, AxisKind::Rows, Action::Insert, 1, 1).formatting_clipped);
    b.apply(b.expected(), &[e(65536, 1, "001234")]).unwrap();
    let before = b.expected();
    assert!(
        b.restructure(
            sheet,
            before,
            Change {
                axis: AxisKind::Rows,
                action: Action::Insert,
                start: 1,
                count: 1
            },
            &Default::default()
        )
        .is_err()
    );
    assert_eq!(b.expected(), before);
    let cancel = pivot_core::model::integration::CancellationToken::default();
    cancel.cancel();
    assert!(
        b.restructure(
            sheet,
            before,
            Change {
                axis: AxisKind::Columns,
                action: Action::Delete,
                start: 1,
                count: 1
            },
            &cancel
        )
        .is_err()
    );
    assert_eq!(b.expected(), before);
    drop(b);
    let m = rt.memory();
    assert!(rt.finish().is_ok());
    assert_eq!(m.stats().charged, 0);
}
