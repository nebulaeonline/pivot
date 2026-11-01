use pivot_core::typed::Error as T;
use pivot_types::PivotValue as V;
use pivot_workbook::*;
fn p(r: u32, c: u32) -> Position {
    Position::new(r, c)
}
fn entry(r: u32, c: u32, s: impl Into<String>) -> Edit {
    Edit::SetEntry {
        position: p(r, c),
        source: s.into(),
    }
}
fn value(b: &Workbook) -> std::result::Result<V, CellError> {
    let cap = b.capture_local(b.expected()).unwrap();
    cap.evaluate_values(&[cap.cell_at(p(1, 1)).unwrap()])
        .unwrap()
        .values
        .remove(0)
        .1
}
#[test]
fn positional_weights_strict_slots_and_reversed_endpoints() {
    let mut b = Workbook::new("Range", Limits::default()).unwrap();
    b.apply(
        b.expected(),
        &[
            entry(1, 2, "3"),
            entry(2, 2, "1"),
            entry(1, 1, "=ALLOCATEWEIGHTEDRANGE(10,1,2,B2:B1)"),
        ],
    )
    .unwrap();
    assert_eq!(value(&b), Ok(V::Decimal("7.5".parse().unwrap())));
    for source in [
        "=ALLOCATEWEIGHTEDRANGE(10,1,2,B1:B3)",
        "=ALLOCATEWEIGHTEDRANGE(10,1,2,C1:C1)",
    ] {
        b.apply(b.expected(), &[entry(1, 1, source)]).unwrap();
        assert_eq!(
            value(&b),
            Err(CellError::Typed(T::TypeMismatch)),
            "{source}"
        );
    }
    for source in [
        "=ALLOCATEWEIGHTEDRANGE(10,1,2,B1:C2)",
        "=ALLOCATEWEIGHTEDRANGE(10,1,2,B1:B65)",
    ] {
        b.apply(b.expected(), &[entry(1, 1, source)]).unwrap();
        assert_eq!(
            value(&b),
            Err(CellError::Typed(T::InvalidArgument)),
            "{source}"
        );
    }
    b.apply(
        b.expected(),
        &[entry(
            1,
            1,
            "=IF(FALSE,ALLOCATEWEIGHTEDRANGE(10,1,2,C1:C1),7)",
        )],
    )
    .unwrap();
    assert_eq!(value(&b), Ok(V::Decimal(7.into())));
}
#[test]
fn literal_range_syntax_is_required() {
    for source in [
        "=ALLOCATEWEIGHTEDRANGE(10,1,2,B1)",
        "=ALLOCATEWEIGHTEDRANGE(10,1,2,1)",
        "=ALLOCATEWEIGHTEDRANGE(10,1,2,B1:B2,3)",
    ] {
        assert!(
            pivot_core::expression::parse_spreadsheet(source).is_err(),
            "{source}"
        );
    }
}

#[test]
fn range_membership_reframes_and_new_slots_are_not_silently_zero() {
    use pivot_workbook::structure::*;
    let mut b = Workbook::new("Spatial", Limits::default()).unwrap();
    b.apply(
        b.expected(),
        &[
            entry(2, 2, "3"),
            entry(3, 2, "1"),
            entry(1, 1, "=ALLOCATEWEIGHTEDRANGE(10,1,2,$B$2:$B$3)"),
        ],
    )
    .unwrap();
    let old = b.capture_local(b.expected()).unwrap();
    let old_cell = old.cell_at(p(1, 1)).unwrap();
    // Inside insertion expands membership to include a new missing position.
    b.restructure(
        b.worksheet(),
        b.expected(),
        Change {
            axis: AxisKind::Rows,
            action: Action::Insert,
            start: 3,
            count: 1,
        },
        &Default::default(),
    )
    .unwrap();
    assert_eq!(value(&b), Err(CellError::Typed(T::TypeMismatch)));
    b.apply(b.expected(), &[entry(3, 2, "0")]).unwrap();
    assert_eq!(value(&b), Ok(V::Decimal("7.5".parse().unwrap())));
    // Deletion contracts, preserving the surviving geometric order.
    b.restructure(
        b.worksheet(),
        b.expected(),
        Change {
            axis: AxisKind::Rows,
            action: Action::Delete,
            start: 2,
            count: 1,
        },
        &Default::default(),
    )
    .unwrap();
    assert_eq!(value(&b), Ok(V::Decimal(0.into())));
    assert_eq!(
        old.evaluate_values(&[old_cell]).unwrap().values.remove(0).1,
        Ok(V::Decimal("7.5".parse().unwrap()))
    );
}
#[test]
fn conservation_matches_scalar_allocator_for_each_recipient() {
    let mut b = Workbook::new("Conservation", Limits::default()).unwrap();
    for weights in [
        vec!["1", "1", "1"],
        vec!["0", "1", "2"],
        vec!["3", "1"],
        vec!["1"],
    ] {
        let n = weights.len();
        for total in ["10", "-10", "0.01", "0"] {
            let mut edits = weights
                .iter()
                .enumerate()
                .map(|(i, w)| entry(i as u32 + 1, 2, *w))
                .collect::<Vec<_>>();
            for i in 1..=n {
                edits.push(entry(
                    i as u32,
                    1,
                    format!("=ALLOCATEWEIGHTEDRANGE({total},{i},2,$B$1:$B${n})"),
                ));
                edits.push(entry(
                    i as u32,
                    3,
                    format!("=ALLOCATEWEIGHTED({total},{i},2,{})", weights.join(",")),
                ));
            }
            edits.push(entry(100, 1, format!("=SUM(A1:A{n})")));
            b.apply(b.expected(), &edits).unwrap();
            let c = b.capture_local(b.expected()).unwrap();
            let ids = (1..=n)
                .flat_map(|i| {
                    [
                        c.cell_at(p(i as u32, 1)).unwrap(),
                        c.cell_at(p(i as u32, 3)).unwrap(),
                    ]
                })
                .collect::<Vec<_>>();
            let values = c.evaluate_values(&ids).unwrap().values;
            for pair in values.chunks(2) {
                assert_eq!(pair[0].1, pair[1].1);
            }
            let sum = c
                .evaluate_values(&[c.cell_at(p(100, 1)).unwrap()])
                .unwrap()
                .values
                .remove(0)
                .1;
            assert_eq!(sum, Ok(V::Decimal(total.parse().unwrap())));
        }
    }
}

#[test]
fn invalid_geometry_omits_member_edges_but_restoration_cycle_validates_atomically() {
    use pivot_workbook::structure::*;
    for (range, axis, start) in [
        ("A1:B2", AxisKind::Columns, 2),
        ("A1:A65", AxisKind::Rows, 65),
    ] {
        let mut b = Workbook::new("Cycle restoration", Limits::default()).unwrap();
        b.apply(
            b.expected(),
            &[entry(
                1,
                1,
                format!("=ALLOCATEWEIGHTEDRANGE(10,1,2,{range})"),
            )],
        )
        .unwrap();
        assert_eq!(value(&b), Err(CellError::Typed(T::InvalidArgument)));
        // An unrelated invalid draft must not mask the restored cycle.
        b.apply(b.expected(), &[entry(100, 3, "=1+")]).unwrap();
        let before = b.expected();
        assert!(
            b.restructure(
                b.worksheet(),
                before,
                Change {
                    axis,
                    action: Action::Delete,
                    start,
                    count: 1
                },
                &Default::default()
            )
            .is_err()
        );
        assert_eq!(b.expected(), before);
        assert_eq!(value(&b), Err(CellError::Typed(T::InvalidArgument)));
    }
}
#[test]
fn typed_slots_and_metadata_errors_remain_local_and_left_to_right() {
    let mut b = Workbook::new("Errors", Limits::default()).unwrap();
    for weight in ["TRUE", "'1", "=IF(TRUE,B100,1)", "'", ""] {
        b.apply(
            b.expected(),
            &[
                entry(1, 2, weight),
                entry(1, 1, "=ALLOCATEWEIGHTEDRANGE(10,1,2,B1:B1)"),
            ],
        )
        .unwrap();
        assert_eq!(
            value(&b),
            Err(CellError::Typed(T::TypeMismatch)),
            "{weight}"
        );
    }
    b.apply(
        b.expected(),
        &[
            entry(1, 2, "=1/0"),
            entry(1, 1, "=ALLOCATEWEIGHTEDRANGE(TRUE,1,2,B1:B1)"),
        ],
    )
    .unwrap();
    assert_eq!(value(&b), Err(CellError::Typed(T::TypeMismatch)));
    b.apply(
        b.expected(),
        &[entry(1, 1, "=ALLOCATEWEIGHTEDRANGE(10,1,29,B1:B1)")],
    )
    .unwrap();
    assert_eq!(value(&b), Err(CellError::Typed(T::InvalidArgument)));
}

#[test]
fn copy_fill_and_visible_transfer_preserve_literal_range_and_fault_authority() {
    use pivot_workbook::{clipboard::VisibleDestination, structure::*};
    let mut b = Workbook::new("Transfers", Limits::default()).unwrap();
    b.apply(
        b.expected(),
        &[
            entry(1, 2, "3"),
            entry(2, 2, "1"),
            entry(1, 1, "=ALLOCATEWEIGHTEDRANGE(10,1,2,$B$1:$B$2)"),
        ],
    )
    .unwrap();
    let one = |r| Rectangle {
        first: p(r, 1),
        last: p(r, 1),
    };
    let image = b.copy_block(b.expected(), one(1)).unwrap();
    b.paste_block(
        b.expected(),
        one(4),
        &image.native,
        true,
        &Default::default(),
    )
    .unwrap();
    b.fill(
        b.expected(),
        Rectangle {
            first: p(4, 1),
            last: p(5, 1),
        },
        FillDirection::Down,
        &Default::default(),
    )
    .unwrap();
    let visible = b
        .copy_visible_sheet(
            b.worksheet(),
            b.expected(),
            &[4, 5],
            1,
            1,
            &Default::default(),
        )
        .unwrap();
    b.paste_visible_sheet(
        b.worksheet(),
        b.expected(),
        VisibleDestination {
            rectangle: Rectangle {
                first: p(7, 1),
                last: p(9, 1),
            },
            rows: &[7, 9],
        },
        &visible.native,
        true,
        &Default::default(),
    )
    .unwrap();
    for r in [1, 4, 5, 7, 9] {
        let c = b.capture_local(b.expected()).unwrap();
        assert_eq!(
            c.evaluate_values(&[c.cell_at(p(r, 1)).unwrap()])
                .unwrap()
                .values
                .remove(0)
                .1,
            Ok(V::Decimal("7.5".parse().unwrap()))
        );
    }
    b.restructure(
        b.worksheet(),
        b.expected(),
        Change {
            axis: AxisKind::Columns,
            action: Action::Delete,
            start: 2,
            count: 1,
        },
        &Default::default(),
    )
    .unwrap();
    let image = b.copy_block(b.expected(), one(1)).unwrap();
    b.paste_block(
        b.expected(),
        one(12),
        &image.native,
        true,
        &Default::default(),
    )
    .unwrap();
    let visible = b
        .copy_visible_sheet(
            b.worksheet(),
            b.expected(),
            &[12],
            1,
            1,
            &Default::default(),
        )
        .unwrap();
    b.paste_visible_sheet(
        b.worksheet(),
        b.expected(),
        VisibleDestination {
            rectangle: one(14),
            rows: &[14],
        },
        &visible.native,
        true,
        &Default::default(),
    )
    .unwrap();
    for r in [12, 14] {
        let c = b.capture_local(b.expected()).unwrap();
        assert!(
            c.evaluate_values(&[c.cell_at(p(r, 1)).unwrap()])
                .unwrap()
                .values
                .remove(0)
                .1
                .is_err()
        );
    }
}
