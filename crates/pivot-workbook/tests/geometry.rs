use pivot_types::PivotDecimal;
use pivot_workbook::{geometry::*, structure::*, *};
fn p(r: u32, c: u32) -> Position {
    Position::new(r, c)
}
fn r(a: u32, b: u32, c: u32, d: u32) -> Rectangle {
    Rectangle {
        first: p(a, b),
        last: p(c, d),
    }
}
fn mapped(book: &Workbook, scope: GridScopeId) -> Map {
    book.capture_scope(scope, book.expected())
        .unwrap()
        .geometry_map(None, &Default::default())
        .unwrap()
}
#[test]
fn geometry_only_publication_pins_old_roots_and_never_allocates_cells() {
    let mut book = Workbook::new("geometry", Limits::default()).unwrap();
    let scope = book
        .create_grid_window(book.expected(), "Invoice", 36, 8)
        .unwrap()
        .into();
    let before = book.expected();
    let old = book.capture_scope(scope, before).unwrap();
    let axes = old.axes_at(p(5, 1)).unwrap();
    let after = book
        .edit_geometry(
            scope,
            before,
            &[Command::AddBreak {
                horizontal: true,
                before: 5,
            }],
            &Default::default(),
        )
        .unwrap();
    assert_ne!(before.state, after.state);
    assert_eq!(before.graph, after.graph);
    assert_eq!(book.stats().live_cells, 0);
    assert_eq!(book.stats().live_nodes, 0);
    assert_eq!(
        mapped(&book, scope)
            .section_closure(r(4, 1, 5, 3))
            .unwrap()
            .effective,
        r(1, 1, 36, 8)
    );
    assert_eq!(
        old.geometry_map(None, &Default::default())
            .unwrap()
            .section_closure(r(4, 1, 5, 3))
            .unwrap()
            .effective,
        r(4, 1, 5, 3)
    );
    book.restructure(
        scope,
        after,
        Change {
            axis: AxisKind::Rows,
            action: Action::Insert,
            start: 5,
            count: 2,
        },
        &Default::default(),
    )
    .unwrap();
    assert_eq!(book.axes_at_sheet(scope, p(7, 1)).unwrap(), axes);
    assert_eq!(
        mapped(&book, scope)
            .section_closure(r(6, 1, 7, 3))
            .unwrap()
            .effective,
        r(1, 1, 36, 8)
    );
    assert_eq!(
        mapped(&book, scope)
            .section_closure(r(4, 1, 5, 3))
            .unwrap()
            .effective,
        r(4, 1, 5, 3)
    );
    book.restructure(
        scope,
        book.expected(),
        Change {
            axis: AxisKind::Rows,
            action: Action::Delete,
            start: 7,
            count: 2,
        },
        &Default::default(),
    )
    .unwrap();
    let image = book.geometry_images().next().unwrap();
    assert!(
        matches!(image.horizontal[1].start,Some(Anchor::Authored(id)) if id==book.axes_at_sheet(scope,p(7,1)).unwrap().0)
    );
    assert_eq!(old.axes_at(p(5, 1)).unwrap(), axes);
}
#[test]
fn collision_batch_is_atomic_and_valid_shared_seam_batch_succeeds() {
    let mut book = Workbook::new("geometry", Limits::default()).unwrap();
    let scope = book
        .create_grid_window(book.expected(), "Grid", 2, 2)
        .unwrap()
        .into();
    book.edit_geometry(
        scope,
        book.expected(),
        &[
            Command::AddBreak {
                horizontal: true,
                before: 2,
            },
            Command::AddBreak {
                horizontal: false,
                before: 2,
            },
        ],
        &Default::default(),
    )
    .unwrap();
    let profile = |horizontal, at, a: i64, b: i64| Command::SetProfile {
        horizontal,
        at,
        sizes: vec![
            SizeRun {
                first: 1,
                last: 1,
                size: a * 64,
            },
            SizeRun {
                first: 2,
                last: 2,
                size: b * 64,
            },
        ],
    };
    // Publish the specified valid gap map in a single normal preview/commit batch.
    book.edit_geometry(
        scope,
        book.expected(),
        &[
            profile(true, 1, 2, 1),
            profile(true, 2, 1, 1),
            profile(false, 1, 1, 1),
            profile(false, 2, 2, 1),
        ],
        &Default::default(),
    )
    .unwrap();
    assert_eq!(mapped(&book, scope).hit(96, 96), None);
    let before = book.expected();
    let images: Vec<_> = book.geometry_images().collect();
    assert!(
        book.edit_geometry(
            scope,
            before,
            &[profile(true, 1, 1, 1), profile(true, 2, 2, 1)],
            &Default::default()
        )
        .is_err()
    );
    assert_eq!(book.expected(), before);
    assert_eq!(book.geometry_images().collect::<Vec<_>>(), images);
    assert!(
        book.edit_geometry(
            scope,
            Expected {
                state: before.state - 1,
                ..before
            },
            &[Command::ResetAll],
            &Default::default()
        )
        .is_err()
    );
    book.edit_geometry(scope, before, &[Command::ResetAll], &Default::default())
        .unwrap();
    assert_eq!(
        mapped(&book, scope)
            .section_closure(r(1, 1, 2, 2))
            .unwrap()
            .sections,
        1
    );
    assert!(book.geometry_images().next().unwrap().next_id > images[0].next_id);
}

#[test]
fn bound_geometry_is_positional_dormant_and_detaches_into_fresh_axes() {
    use pivot_workbook::producers::{Fixture, KeyPolicy};
    let mut book = Workbook::new("bound geometry", Limits::default()).unwrap();
    let producer = book
        .create_fixture_producer(book.expected(), Fixture::Ledger, KeyPolicy::Durable)
        .unwrap();
    let window = book
        .create_bound_window(book.expected(), "Bound", producer)
        .unwrap();
    let scope = window.into();
    book.edit_geometry(
        scope,
        book.expected(),
        &[
            Command::AddBreak {
                horizontal: true,
                before: 3,
            },
            Command::SetProfile {
                horizontal: true,
                at: 1,
                sizes: vec![
                    SizeRun {
                        first: 1,
                        last: 1,
                        size: 200 * 64,
                    },
                    SizeRun {
                        first: 2,
                        last: MAX_COLUMNS,
                        size: DEFAULT_WIDTH,
                    },
                ],
            },
        ],
        &Default::default(),
    )
    .unwrap();
    let absent = book.capture_scope(scope, book.expected()).unwrap();
    assert!(absent.calculation.positional_geometry(scope).unwrap());
    assert_eq!(
        absent
            .geometry_map(None, &Default::default())
            .unwrap()
            .hit(0, 0),
        None
    );
    let image = book.geometry_images().next().unwrap();
    assert_eq!(image.horizontal[1].start, Some(Anchor::Ordinal(3)));
    let ticket = book.begin_refresh(book.expected(), producer).unwrap();
    book.refresh_fixture(&ticket, 0).unwrap();
    drop(ticket);
    let current = mapped(&book, scope);
    assert_eq!(current.cell_rect(p(1, 1)).unwrap().right, 200 * 64);
    assert_eq!(book.geometry_images().next().unwrap(), image);
    assert!(
        absent
            .geometry_map(None, &Default::default())
            .unwrap()
            .cell_rect(p(1, 1))
            .is_none()
    );
    let copy = book
        .detach_bound_window(book.expected(), window, "Detached", &Default::default())
        .unwrap();
    let copied = mapped(&book, copy.into());
    assert_eq!(copied.cell_rect(p(1, 1)), current.cell_rect(p(1, 1)));
    let detached = book
        .geometry_images()
        .find(|g| g.scope == copy.into())
        .unwrap();
    assert!(
        detached
            .horizontal
            .iter()
            .filter_map(|s| s.start)
            .all(|a| matches!(a, Anchor::Authored(_)))
    );
    assert!(
        detached
            .profiles
            .iter()
            .flat_map(|p| &p.sizes)
            .all(|s| matches!(s.first, Anchor::Authored(_)))
    );
}

#[test]
fn filter_and_geometry_candidates_both_refuse_compaction_collisions_atomically() {
    use pivot_workbook::filtering::{Condition, Definition, Literal, Operator, Predicate};
    let mut b = Workbook::new("filtered geometry", Limits::default()).unwrap();
    let scope = b
        .create_grid_window(b.expected(), "Grid", 3, 2)
        .unwrap()
        .into();
    b.apply_scope(
        scope,
        b.expected(),
        &[
            Edit::SetBoolean {
                position: p(1, 1),
                value: true,
            },
            Edit::SetBoolean {
                position: p(2, 1),
                value: false,
            },
            Edit::SetBoolean {
                position: p(3, 1),
                value: true,
            },
        ],
    )
    .unwrap();
    let sizes = |values: &[i64]| {
        values
            .iter()
            .enumerate()
            .map(|(i, n)| SizeRun {
                first: i as u32 + 1,
                last: i as u32 + 1,
                size: n * 64,
            })
            .collect()
    };
    b.edit_geometry(
        scope,
        b.expected(),
        &[
            Command::AddBreak {
                horizontal: true,
                before: 3,
            },
            Command::AddBreak {
                horizontal: false,
                before: 2,
            },
            Command::SetProfile {
                horizontal: true,
                at: 1,
                sizes: sizes(&[1, 1]),
            },
            Command::SetProfile {
                horizontal: true,
                at: 3,
                sizes: sizes(&[2, 1]),
            },
            Command::SetProfile {
                horizontal: false,
                at: 1,
                sizes: sizes(&[1, 2, 1]),
            },
            Command::SetProfile {
                horizontal: false,
                at: 2,
                sizes: sizes(&[2, 1, 1]),
            },
        ],
        &Default::default(),
    )
    .unwrap();
    let definition = Definition {
        rectangle: r(1, 1, 3, 2),
        exclude_header: false,
        needs_review: false,
        conditions: vec![Condition {
            column: 1,
            either: false,
            predicates: vec![Predicate {
                operator: Operator::Equal,
                literal: Some(Literal::Boolean(true)),
            }],
        }],
    };
    let before = b.expected();
    assert!(
        b.set_filter(scope, before, Some(&definition), &Default::default())
            .is_err()
    );
    assert_eq!(b.expected(), before);
    assert!(b.filter(scope).unwrap().is_none());
    b.edit_geometry(
        scope,
        before,
        &[Command::SetProfile {
            horizontal: true,
            at: 3,
            sizes: sizes(&[1, 1]),
        }],
        &Default::default(),
    )
    .unwrap();
    b.set_filter(scope, b.expected(), Some(&definition), &Default::default())
        .unwrap();
    let before = b.expected();
    let roots: Vec<_> = b.geometry_images().collect();
    assert!(
        b.edit_geometry(
            scope,
            before,
            &[Command::SetProfile {
                horizontal: true,
                at: 3,
                sizes: sizes(&[2, 1])
            }],
            &Default::default()
        )
        .is_err()
    );
    assert_eq!(b.expected(), before);
    assert_eq!(b.geometry_images().collect::<Vec<_>>(), roots);
}

#[test]
fn geometry_ownership_refuses_unsupported_experimental_activation() {
    let mut b = Workbook::new("unsupported geometry conversion", Limits::default()).unwrap();
    b.edit_geometry(
        b.worksheet(),
        b.expected(),
        &[Command::AddBreak {
            horizontal: true,
            before: 5,
        }],
        &Default::default(),
    )
    .unwrap();
    let before = b.expected();
    assert!(
        b.enable_structured_proof(uuid::Uuid::new_v4(), uuid::Uuid::new_v4())
            .is_err()
    );
    #[cfg(feature = "history-experiment")]
    assert!(b.pin_history_root(before).is_err());
    assert_eq!(b.expected(), before);
}

#[test]
fn complete_region_moves_preserve_identity_styles_blanks_and_atomic_refusals() {
    let mut b = Workbook::new("region operations", Limits::default()).unwrap();
    let scope = b.worksheet();
    b.apply(
        b.expected(),
        &[
            Edit::SetText {
                position: p(1, 1),
                text: "001234".into(),
            },
            Edit::SetDecimal {
                position: p(1, 2),
                value: DecimalInput::from_decimal(7.into()),
            },
            Edit::SetEntry {
                position: p(4, 1),
                source: "=A1".into(),
            },
        ],
    )
    .unwrap();
    let identity = b.cell_at(p(1, 1)).unwrap().unwrap().reference;
    b.format_sheet(
        scope,
        b.expected(),
        r(1, 1, 2, 2),
        formatting::StylePatch {
            bold: formatting::Field::Set(true),
            ..Default::default()
        },
        false,
        &Default::default(),
    )
    .unwrap();
    let allocated = b.stats().allocated_nodes;
    b.move_region_sheet(
        scope,
        b.expected(),
        r(1, 1, 2, 2),
        p(1, 2),
        &Default::default(),
    )
    .unwrap();
    assert_eq!(b.cell_at(p(1, 2)).unwrap().unwrap().reference, identity);
    assert_eq!(
        b.cell_at(p(1, 3)).unwrap().unwrap().content,
        Content::Decimal(DecimalInput::from_decimal(7.into()))
    );
    assert!(b.cell_at(p(2, 3)).unwrap().is_none());
    assert_eq!(b.stats().allocated_nodes, allocated);
    assert!(b.style_at(scope, p(2, 3)).unwrap().bold);
    assert!(!b.style_at(scope, p(2, 1)).unwrap().bold);
    let capture = b.capture_local(b.expected()).unwrap();
    let out = capture.cell_at(p(4, 1)).unwrap();
    assert_eq!(
        capture.evaluate_values(&[out]).unwrap().values[0].1,
        Ok(pivot_types::PivotValue::String("001234".into()))
    );
    let before = b.expected();
    assert!(
        b.move_region_sheet(scope, before, r(1, 2, 2, 3), p(4, 1), &Default::default())
            .is_err()
    );
    assert_eq!(b.expected(), before);
    b.clear_region_sheet(
        scope,
        b.expected(),
        r(1, 2, 2, 3),
        visible_edit::ClearMode::Contents,
        &Default::default(),
    )
    .unwrap();
    assert_eq!(b.cell_at(p(1, 2)).unwrap().unwrap().reference, identity);
    assert!(b.style_at(scope, p(2, 3)).unwrap().bold);
}

#[test]
fn numeric_and_balanced_edits_preview_atomically_without_changing_roots() {
    let mut book = Workbook::new("sizing", Limits::default()).unwrap();
    let scope = book
        .create_grid_window(book.expected(), "Invoice", 36, 8)
        .unwrap()
        .into();
    book.edit_geometry(
        scope,
        book.expected(),
        &[Command::AddBreak {
            horizontal: true,
            before: 5,
        }],
        &Default::default(),
    )
    .unwrap();
    let before = book.expected();
    let old = book.capture_scope(scope, before).unwrap();
    let command = Command::SetSizes {
        horizontal: true,
        at: 5,
        first: 1,
        last: 1,
        size: DEFAULT_WIDTH + 640,
        paired: true,
        shared: false,
    };
    let images: Vec<_> = book.geometry_images().collect();
    assert_eq!(
        book.preview_geometry(
            scope,
            before,
            std::slice::from_ref(&command),
            &Default::default()
        )
        .unwrap(),
        before
    );
    assert_eq!(book.geometry_images().collect::<Vec<_>>(), images);
    book.edit_geometry(scope, before, &[command], &Default::default())
        .unwrap();
    let map = mapped(&book, scope);
    let first = map.cell_rect(p(5, 1)).unwrap();
    let second = map.cell_rect(p(5, 2)).unwrap();
    assert_eq!(first.right - first.left, DEFAULT_WIDTH + 640);
    assert_eq!(second.right - second.left, DEFAULT_WIDTH - 640);
    assert_eq!(second.right, DEFAULT_WIDTH * 2);
    assert_eq!(map.cell_rect(p(1, 1)).unwrap().right, DEFAULT_WIDTH);
    assert_eq!(
        old.geometry_map(None, &Default::default())
            .unwrap()
            .cell_rect(p(5, 1))
            .unwrap()
            .right,
        DEFAULT_WIDTH
    );
    let shared = Command::SetSizes {
        horizontal: true,
        at: 1,
        first: 1,
        last: 1,
        size: DEFAULT_WIDTH + 320,
        paired: true,
        shared: true,
    };
    book.edit_geometry(scope, book.expected(), &[shared], &Default::default())
        .unwrap();
    assert_eq!(
        mapped(&book, scope).cell_rect(p(1, 1)).unwrap().right,
        DEFAULT_WIDTH + 320
    );
    assert_eq!(
        mapped(&book, scope).cell_rect(p(5, 1)).unwrap().right,
        DEFAULT_WIDTH + 960
    );
    let before = book.expected();
    let images: Vec<_> = book.geometry_images().collect();
    let invalid = Command::SetSizes {
        horizontal: true,
        at: 1,
        first: 1,
        last: 1,
        size: MAX_SIZE,
        paired: true,
        shared: true,
    };
    assert!(
        book.preview_geometry(
            scope,
            before,
            std::slice::from_ref(&invalid),
            &Default::default()
        )
        .is_err()
    );
    assert!(
        book.edit_geometry(scope, before, &[invalid], &Default::default())
            .is_err()
    );
    assert_eq!(book.expected(), before);
    assert_eq!(book.geometry_images().collect::<Vec<_>>(), images);
    let edge = Command::SetSizes {
        horizontal: false,
        at: 1,
        first: 36,
        last: 36,
        size: 64,
        paired: true,
        shared: false,
    };
    assert!(
        book.edit_geometry(scope, before, &[edge], &Default::default())
            .is_err()
    );
}

#[test]
fn bound_layout_notice_is_persistent_generation_qualified_and_ignores_value_refreshes() {
    use pivot_workbook::producers::{Fixture, KeyPolicy, Value};
    let mut book = Workbook::new("layout", Limits::default()).unwrap();
    let producer = book
        .create_fixture_producer(book.expected(), Fixture::Ledger, KeyPolicy::Durable)
        .unwrap();
    let ticket = book.begin_refresh(book.expected(), producer).unwrap();
    book.refresh_fixture(&ticket, 0).unwrap();
    drop(ticket);
    let window = book
        .create_bound_window(book.expected(), "Layout", producer)
        .unwrap();
    let scope = window.into();
    book.edit_geometry(
        scope,
        book.expected(),
        &[Command::AddBreak {
            horizontal: true,
            before: 5,
        }],
        &Default::default(),
    )
    .unwrap();
    let review = |book: &Workbook| {
        book.capture_scope(scope, book.expected())
            .unwrap()
            .calculation
            .geometry_layout_review(scope, &Default::default())
            .unwrap()
    };
    let (original, needed) = review(&book);
    assert!(!needed);
    let old = book.capture_scope(scope, book.expected()).unwrap();
    let old_layout_work = old.calculation.geometry_layout_work(scope).unwrap();
    let ticket = book.begin_refresh(book.expected(), producer).unwrap();
    let mut values = Fixture::Ledger.rows(0).unwrap();
    values[0].values[0] = Value::Decimal(DecimalInput::from_decimal(PivotDecimal::from(999)));
    book.finish_refresh(&ticket, Some(values), false).unwrap();
    drop(ticket);
    assert_eq!(
        review(&book),
        (original, false),
        "Value-only publication does not request layout review"
    );
    let ticket = book.begin_refresh(book.expected(), producer).unwrap();
    book.refresh_fixture(&ticket, 1).unwrap();
    drop(ticket);
    let (reordered, needed) = review(&book);
    assert!(needed);
    assert_ne!(reordered, original);
    assert_eq!(
        old.calculation
            .geometry_layout_review(scope, &Default::default())
            .unwrap(),
        (original, false)
    );
    assert!(
        book.edit_geometry(
            scope,
            book.expected(),
            &[Command::AcknowledgeLayout {
                stamp: original.unwrap()
            }],
            &Default::default()
        )
        .is_err()
    );
    assert_eq!(review(&book), (reordered, true));
    book.edit_geometry(
        scope,
        book.expected(),
        &[Command::AcknowledgeLayout {
            stamp: reordered.unwrap(),
        }],
        &Default::default(),
    )
    .unwrap();
    assert_eq!(review(&book), (reordered, false));
    let ticket = book.begin_refresh(book.expected(), producer).unwrap();
    book.refresh_fixture(&ticket, 2).unwrap();
    drop(ticket);
    assert!(review(&book).1);
    book.edit_geometry(
        scope,
        book.expected(),
        &[Command::ResetAll],
        &Default::default(),
    )
    .unwrap();
    assert!(!review(&book).1);
    assert_eq!(
        old.calculation.geometry_layout_work(scope).unwrap(),
        old_layout_work
    );
    assert!(
        book.capture_scope(scope, book.expected())
            .unwrap()
            .calculation
            .geometry_layout_work(scope)
            .unwrap()
            < old_layout_work,
        "Work accounting follows the pinned producer generation, not the current book"
    );
}

#[test]
fn worksheet_bound_layout_detects_sort_without_treating_value_edits_as_schema_changes() {
    use pivot_workbook::sorting::{Key, Options};
    let mut book = Workbook::new("worksheet layout", Limits::default()).unwrap();
    let sheet = book.worksheet();
    book.apply(
        book.expected(),
        &[
            Edit::SetDecimal {
                position: p(1, 1),
                value: DecimalInput::from_decimal(PivotDecimal::from(2)),
            },
            Edit::SetDecimal {
                position: p(2, 1),
                value: DecimalInput::from_decimal(PivotDecimal::from(1)),
            },
        ],
    )
    .unwrap();
    let producer = book
        .create_worksheet_producer(
            book.expected(),
            WorksheetId::from_uuid(sheet.uuid()),
            r(1, 1, 2, 1),
        )
        .unwrap();
    let scope = book
        .create_bound_window(book.expected(), "Live", producer)
        .unwrap()
        .into();
    book.edit_geometry(
        scope,
        book.expected(),
        &[Command::AddBreak {
            horizontal: true,
            before: 2,
        }],
        &Default::default(),
    )
    .unwrap();
    let review = |book: &Workbook| {
        book.capture_scope(scope, book.expected())
            .unwrap()
            .calculation
            .geometry_layout_review(scope, &Default::default())
            .unwrap()
    };
    let original = review(&book);
    assert!(!original.1);
    book.apply(
        book.expected(),
        &[Edit::SetDecimal {
            position: p(1, 1),
            value: DecimalInput::from_decimal(PivotDecimal::from(3)),
        }],
    )
    .unwrap();
    assert_eq!(review(&book), original);
    book.sort_rows(
        sheet,
        book.expected(),
        &Options {
            rectangle: r(1, 1, 2, 1),
            exclude_header: false,
            keys: vec![Key {
                column: 1,
                descending: false,
            }],
            move_formatting: false,
        },
        &Default::default(),
    )
    .unwrap();
    assert!(review(&book).1);
    assert_ne!(review(&book).0, original.0);
}
