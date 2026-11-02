use pivot_workbook::{structure::*, *};
fn p(row: u32, column: u32) -> Position {
    Position::new(row, column)
}
fn entry(row: u32, source: &str) -> Edit {
    Edit::SetEntry {
        position: p(row, 1),
        source: source.into(),
    }
}
fn value(book: &Workbook, scope: GridScopeId, row: u32) -> String {
    let capture = book.capture_scope(scope, book.expected()).unwrap();
    let cell = capture.calculation.cell_at(p(row, 1)).unwrap();
    capture.calculation.evaluate(&[cell]).unwrap().values[0]
        .1
        .clone()
        .unwrap()
        .to_string()
}
#[test]
fn same_names_distinct_namespaces_and_old_capture_after_retirement() {
    let mut b = Workbook::new("windows", Limits::default()).unwrap();
    let sheet = b.worksheet();
    b.rename_sheet(b.expected(), sheet, "Costs").unwrap();
    let id = b
        .create_grid_window(b.expected(), "Costs", 100, 10)
        .unwrap();
    let scope = id.into();
    b.apply_scope(scope, b.expected(), &[entry(1, "7"), entry(2, "3")])
        .unwrap();
    b.apply_sheet(sheet, b.expected(), &[entry(1, "=SUM(Costs|A1:A2)")])
        .unwrap();
    assert_eq!(value(&b, sheet, 1), "10");
    assert_eq!(value(&b, scope, 1), "7");
    assert_eq!(b.sheets().len(), 1);
    let old = b.capture_scope(sheet, b.expected()).unwrap();
    let cell = old.calculation.cell_at(p(1, 1)).unwrap();
    b.rename_window(b.expected(), id, "O'Brien").unwrap();
    assert!(
        b.display_formula(cell)
            .unwrap()
            .unwrap()
            .contains("'O''Brien'|")
    );
    b.delete_window(b.expected(), id, &Default::default())
        .unwrap();
    assert!(b.capture_scope(scope, b.expected()).is_err());
    assert!(b.csv_content_bounds(scope).is_err());
    assert_eq!(
        old.calculation.evaluate(&[cell]).unwrap().values[0].1,
        Ok(10.into())
    );
    b.create_grid_window(b.expected(), "O'Brien", 100, 10)
        .unwrap();
    let cap = b.capture_sheet(sheet, b.expected()).unwrap();
    assert!(cap.evaluate(&[cell]).unwrap().values[0].1.is_err());
}
#[test]
fn lazy_axis_identity_survives_insert_delete_and_capture() {
    let mut b = Workbook::new("axes", Limits::default()).unwrap();
    let id = b.create_grid_window(b.expected(), "Grid", 100, 10).unwrap();
    let scope = id.into();
    let old = b.capture_scope(scope, b.expected()).unwrap();
    let axis = old.axes_at(p(5, 1)).unwrap();
    b.restructure(
        scope,
        b.expected(),
        Change {
            axis: AxisKind::Rows,
            action: Action::Insert,
            start: 2,
            count: 3,
        },
        &Default::default(),
    )
    .unwrap();
    assert_eq!(b.axes_at_sheet(scope, p(8, 1)).unwrap(), axis);
    assert_ne!(b.axes_at_sheet(scope, p(5, 1)).unwrap().0, axis.0);
    b.restructure(
        scope,
        b.expected(),
        Change {
            axis: AxisKind::Rows,
            action: Action::Delete,
            start: 2,
            count: 3,
        },
        &Default::default(),
    )
    .unwrap();
    assert_eq!(b.axes_at_sheet(scope, p(5, 1)).unwrap(), axis);
    assert_eq!(old.axes_at(p(5, 1)).unwrap(), axis);
    assert_eq!(b.stats().live_cells, 0);
}
#[test]
fn retained_capture_owns_charged_axis_roots_across_delete_and_recreation() {
    use pivot_core::model::integration::IntegrationRuntime;
    let rt = IntegrationRuntime::standalone().unwrap();
    let memory = rt.memory();
    let mut b = Workbook::new_in("retention", Limits::default(), &rt).unwrap();
    let id = b.create_grid_window(b.expected(), "Grid", 100, 10).unwrap();
    let scope = id.into();
    b.apply_scope(scope, b.expected(), &[entry(5, "17")])
        .unwrap();
    b.restructure(
        scope,
        b.expected(),
        Change {
            axis: AxisKind::Rows,
            action: Action::Insert,
            start: 2,
            count: 1,
        },
        &Default::default(),
    )
    .unwrap();
    let capture = b.capture_scope(scope, b.expected()).unwrap();
    let axis = capture.axes_at(p(6, 1)).unwrap();
    let cell = capture.calculation.cell_at(p(6, 1)).unwrap();
    b.delete_window(b.expected(), id, &Default::default())
        .unwrap();
    let next = b.create_grid_window(b.expected(), "Grid", 100, 10).unwrap();
    assert_ne!(id, next);
    assert_ne!(b.axes_at_sheet(next.into(), p(6, 1)).unwrap(), axis);
    assert_eq!(
        capture.calculation.evaluate(&[cell]).unwrap().values[0].1,
        Ok(17.into())
    );
    drop(b);
    // Exhaust the remaining domain: the live capture's ownership cannot be reused.
    let stats = memory.stats();
    assert!(stats.charged > 0);
    let full = memory.reserve(stats.limit - stats.charged).unwrap();
    assert!(memory.reserve(1).is_err());
    drop(full);
    drop(capture);
    assert!(rt.finish().is_ok());
    assert_eq!(memory.stats().charged, 0);
}
#[test]
fn cross_scope_copy_binds_destination_and_native_clipboard_refuses_authority_loss() {
    let mut b = Workbook::new("copy", Limits::default()).unwrap();
    let id = b.create_grid_window(b.expected(), "Grid", 100, 10).unwrap();
    let scope = id.into();
    let sheet = b.worksheet();
    b.apply_scope(
        scope,
        b.expected(),
        &[
            entry(1, "9"),
            entry(2, "=A1+1"),
            Edit::SetText {
                position: p(3, 1),
                text: "001234".into(),
            },
        ],
    )
    .unwrap();
    b.copy_between_scopes(
        b.expected(),
        scope,
        Rectangle {
            first: p(1, 1),
            last: p(3, 1),
        },
        sheet,
        p(1, 1),
        &Default::default(),
    )
    .unwrap();
    assert_eq!(value(&b, sheet, 2), "10");
    b.apply_scope(scope, b.expected(), &[entry(1, "99")])
        .unwrap();
    assert_eq!(value(&b, sheet, 2), "10");
    let rect = Rectangle {
        first: p(1, 1),
        last: p(3, 1),
    };
    assert!(b.copy_sheet(scope, b.expected(), rect).is_err());
    let exported = b.copy_values_scope(scope, b.expected(), rect).unwrap();
    assert!(exported.native.is_empty());
    assert!(exported.text.contains("'001234"));
    b.apply_sheet(sheet, b.expected(), &[entry(4, "=Grid|A1")])
        .unwrap();
    assert!(
        b.copy_sheet(
            sheet,
            b.expected(),
            Rectangle {
                first: p(4, 1),
                last: p(4, 1)
            }
        )
        .is_err()
    );
    let native = b
        .copy_sheet(
            sheet,
            b.expected(),
            Rectangle {
                first: p(1, 1),
                last: p(1, 1),
            },
        )
        .unwrap();
    assert!(
        b.paste_sheet(
            scope,
            b.expected(),
            rect,
            &native.native,
            true,
            &Default::default()
        )
        .is_err()
    );
}
#[test]
fn empty_windows_are_sparse_and_cells_share_one_capacity() {
    use pivot_core::model::integration::IntegrationRuntime;
    let rt = IntegrationRuntime::standalone().unwrap();
    let mut b = Workbook::new_in("empty", Limits::capacity(), &rt).unwrap();
    let baseline = rt.memory().stats().charged;
    for i in 0..64 {
        b.create_grid_window(b.expected(), &format!("Grid{i}"), MAX_ROWS, MAX_COLUMNS)
            .unwrap();
    }
    assert_eq!(b.stats().allocated_nodes, 0);
    assert_eq!(b.sheets().len(), 1);
    assert_eq!(b.windows().count(), 64);
    assert_eq!(b.stats().axis_namespaces, 130); // One worksheet plus 64 Windows.
    assert_eq!(b.stats().explicit_axis_runs, 0);
    assert_eq!(b.stats().axis_layout_bytes, 0);
    assert!(rt.memory().stats().charged - baseline < 1_048_576);
    assert!(
        b.create_grid_window(b.expected(), "too many", 1, 1)
            .is_err()
    );
    let mut small = Workbook::new(
        "shared",
        Limits {
            live_cells: 1,
            allocated_cells: 1,
            ..Limits::default()
        },
    )
    .unwrap();
    let id = small
        .create_grid_window(small.expected(), "Grid", 1, 1)
        .unwrap();
    small
        .apply_scope(id.into(), small.expected(), &[entry(1, "1")])
        .unwrap();
    let before = small.expected();
    assert!(small.apply(before, &[entry(1, "2")]).is_err());
    assert_eq!(small.expected(), before);
}
#[test]
fn dense_axis_oracle_preserves_survivors_and_never_reuses_retired_sequences() {
    use std::collections::BTreeSet;
    for axis in [AxisKind::Rows, AxisKind::Columns] {
        let mut b = Workbook::new("axis oracle", Limits::default()).unwrap();
        let id = b.create_grid_window(b.expected(), "Grid", 20, 20).unwrap();
        let scope = id.into();
        let at = |book: &Workbook, n| {
            let pair = book
                .axes_at_sheet(
                    scope,
                    if axis == AxisKind::Rows {
                        p(n, 1)
                    } else {
                        p(1, n)
                    },
                )
                .unwrap();
            if axis == AxisKind::Rows {
                pair.0
            } else {
                pair.1
            }
        };
        let mut expected: Vec<_> = (1..=20).map(|n| Some(at(&b, n))).collect();
        let mut seen: BTreeSet<_> = expected.iter().map(|id| id.unwrap().sequence).collect();
        let old = b.capture_scope(scope, b.expected()).unwrap();
        let mut random = 12_345u32;
        for _ in 0..160 {
            random = random.wrapping_mul(1_664_525).wrapping_add(1_013_904_223);
            let count = 1 + random % 3;
            let start = 1 + (random >> 8) % (21 - count);
            let action = if random & 1 == 0 {
                Action::Insert
            } else {
                Action::Delete
            };
            if action == Action::Insert {
                expected.splice(
                    (start - 1) as usize..(start - 1) as usize,
                    vec![None; count as usize],
                );
                expected.truncate(20);
            } else {
                expected.drain((start - 1) as usize..(start - 1 + count) as usize);
                expected.extend(vec![None; count as usize]);
            }
            b.restructure(
                scope,
                b.expected(),
                Change {
                    axis,
                    action,
                    start,
                    count,
                },
                &Default::default(),
            )
            .unwrap();
            for (index, wanted) in expected.iter_mut().enumerate() {
                let actual = at(&b, index as u32 + 1);
                if let Some(wanted) = wanted {
                    assert_eq!(actual, *wanted);
                } else {
                    assert!(seen.insert(actual.sequence));
                    *wanted = Some(actual);
                }
            }
        }
        assert_eq!(old.axes_at(p(7, 7)).unwrap().0.sequence, 7);
        assert_eq!(old.axes_at(p(7, 7)).unwrap().1.sequence, 7);
        assert_eq!(b.stats().allocated_nodes, 0);
    }
}
#[test]
fn qualified_bounds_use_target_scope_and_cycles_refuse_atomically() {
    let mut b = Workbook::new("bounds", Limits::default()).unwrap();
    let id = b.create_grid_window(b.expected(), "Small", 10, 4).unwrap();
    let scope = id.into();
    b.apply(b.expected(), &[entry(200, "23")]).unwrap();
    b.apply_scope(scope, b.expected(), &[entry(1, "=Sheet1!A200")])
        .unwrap();
    assert_eq!(value(&b, scope, 1), "23");
    b.apply(
        b.expected(),
        &[entry(1, "=Small|A11"), entry(2, "=SUM(Small|A11:A1)")],
    )
    .unwrap();
    for row in [1, 2] {
        assert!(
            matches!(b.cell_at(p(row, 1)).unwrap().unwrap().content, Content::TextFormula(ref f) if f.diagnostic.is_some())
        );
    }
    let before = b.expected();
    assert!(b.apply(before, &[entry(200, "=Small|A1")]).is_err());
    assert_eq!(b.expected(), before);
}
