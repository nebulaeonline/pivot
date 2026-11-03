use pivot_core::model::integration::IntegrationRuntime;
use pivot_workbook::{geometry::*, structure::*, *};
use pivot_workbook_document::Document;
fn p(r: u32, c: u32) -> Position {
    Position::new(r, c)
}
#[test]
fn wb16_restores_stable_geometry_and_rejects_forged_authority() {
    let rt = IntegrationRuntime::standalone().unwrap();
    let mut b = Workbook::new_in("Geometry", Limits::default(), &rt).unwrap();
    let scope = b
        .create_grid_window(b.expected(), "Invoice", 36, 8)
        .unwrap()
        .into();
    b.apply_scope(
        scope,
        b.expected(),
        &[Edit::SetText {
            position: p(1, 1),
            text: "001234".into(),
        }],
    )
    .unwrap();
    b.edit_geometry(
        scope,
        b.expected(),
        &[
            Command::AddBreak {
                horizontal: true,
                before: 5,
            },
            Command::AddBreak {
                horizontal: true,
                before: 29,
            },
            Command::SetProfile {
                horizontal: true,
                at: 5,
                sizes: vec![
                    SizeRun {
                        first: 1,
                        last: 1,
                        size: 240 * 64,
                    },
                    SizeRun {
                        first: 2,
                        last: 8,
                        size: 48 * 64,
                    },
                ],
            },
        ],
        &Default::default(),
    )
    .unwrap();
    b.restructure(
        scope,
        b.expected(),
        Change {
            axis: AxisKind::Rows,
            action: Action::Insert,
            start: 5,
            count: 2,
        },
        &Default::default(),
    )
    .unwrap();
    let images: Vec<_> = b.geometry_images().collect();
    let export = Document::export_document(&b, b.expected()).unwrap();
    assert!(export.bytes.starts_with(b"PIVOTWB16"));
    let (restored, _) = Document::open_document(&export.bytes, Limits::default(), &rt).unwrap();
    assert_eq!(restored.geometry_images().collect::<Vec<_>>(), images);
    let map = restored
        .capture_scope(scope, restored.expected())
        .unwrap()
        .geometry_map(None, &Default::default())
        .unwrap();
    assert_eq!(map.cell_rect(p(7, 1)).unwrap().right, 240 * 64);
    let capture = restored.capture_scope(scope, restored.expected()).unwrap();
    let cell = capture.calculation.cell_at(p(1, 1)).unwrap();
    assert_eq!(
        capture.calculation.evaluate_values(&[cell]).unwrap().values[0].1,
        Ok(pivot_types::PivotValue::String("001234".into()))
    );
    drop(capture);
    for corruption in 0..6 {
        let mut portable = b.export_portable(b.expected()).unwrap();
        let g = &mut portable.geometry[0];
        match corruption {
            0 => g.next_id = 1,
            1 => g.horizontal[1].start = Some(Anchor::Ordinal(7)),
            2 => g.horizontal[1].id = g.horizontal[0].id,
            3 => g.profiles[0].default = 0,
            4 => g.horizontal[1].profile = u64::MAX,
            _ => g.scope = b.worksheet(),
        }
        // Restore is a private candidate; a failure cannot change the source book.
        assert!(Workbook::restore_portable(portable, Limits::default(), &rt).is_err());
        assert_eq!(b.geometry_images().collect::<Vec<_>>(), images);
    }
    let mut obsolete = export.bytes.clone();
    obsolete[..9].copy_from_slice(b"PIVOTWB15");
    assert!(Document::open_document(&obsolete, Limits::default(), &rt).is_err());
    drop(map);
    drop(restored);
    drop(b);
    let memory = rt.memory();
    assert!(rt.finish().is_ok());
    assert_eq!(memory.stats().charged, 0);
}

#[test]
fn retired_geometry_keeps_allocation_floor_and_old_capture_ownership() {
    let rt = IntegrationRuntime::standalone().unwrap();
    let mut b = Workbook::new_in("retirement", Limits::default(), &rt).unwrap();
    let id = b.create_grid_window(b.expected(), "Grid", 10, 4).unwrap();
    b.edit_geometry(
        id.into(),
        b.expected(),
        &[Command::AddBreak {
            horizontal: true,
            before: 5,
        }],
        &Default::default(),
    )
    .unwrap();
    let floor = b.geometry_images().next().unwrap().next_id;
    let old = b.capture_scope(id.into(), b.expected()).unwrap();
    let old_axis = old.axes_at(p(5, 1)).unwrap();
    b.delete_window(b.expected(), id, &Default::default())
        .unwrap();
    let retired = b.geometry_images().next().unwrap();
    assert!(retired.next_id >= floor);
    assert!(
        retired.horizontal.is_empty() && retired.vertical.is_empty() && retired.profiles.is_empty()
    );
    let saved = Document::export_document(&b, b.expected()).unwrap();
    let (reopened, _) = Document::open_document(&saved.bytes, Limits::default(), &rt).unwrap();
    assert_eq!(reopened.geometry_images().next().unwrap(), retired);
    let replacement = b.create_grid_window(b.expected(), "Grid", 10, 4).unwrap();
    assert_ne!(replacement, id);
    assert_eq!(old.axes_at(p(5, 1)).unwrap(), old_axis);
    assert_eq!(
        old.geometry_map(None, &Default::default())
            .unwrap()
            .section_closure(Rectangle {
                first: p(4, 1),
                last: p(5, 2)
            })
            .unwrap()
            .sections,
        2
    );
    drop(reopened);
    drop(b);
    assert!(rt.memory().stats().charged > 0);
    drop(old);
    let memory = rt.memory();
    assert!(rt.finish().is_ok());
    assert_eq!(memory.stats().charged, 0);
}

#[test]
fn wb16_preserves_layout_acknowledgement_and_order_epoch() {
    use pivot_workbook::producers::{Fixture, KeyPolicy};
    let rt = IntegrationRuntime::standalone().unwrap();
    let mut book = Workbook::new_in("layout codec", Limits::default(), &rt).unwrap();
    let producer = book
        .create_fixture_producer(book.expected(), Fixture::Ledger, KeyPolicy::Durable)
        .unwrap();
    let ticket = book.begin_refresh(book.expected(), producer).unwrap();
    book.refresh_fixture(&ticket, 0).unwrap();
    drop(ticket);
    let scope = book
        .create_bound_window(book.expected(), "Bound", producer)
        .unwrap()
        .into();
    book.edit_geometry(
        scope,
        book.expected(),
        &[Command::AddBreak {
            horizontal: true,
            before: 3,
        }],
        &Default::default(),
    )
    .unwrap();
    let ticket = book.begin_refresh(book.expected(), producer).unwrap();
    book.refresh_fixture(&ticket, 1).unwrap();
    drop(ticket);
    let check = |book: &Workbook| {
        book.capture_scope(scope, book.expected())
            .unwrap()
            .calculation
            .geometry_layout_review(scope, &Default::default())
            .unwrap()
    };
    let pending = check(&book);
    assert!(pending.1);
    let export = Document::export_document(&book, book.expected()).unwrap();
    let (mut restored, _) = Document::open_document(&export.bytes, Limits::default(), &rt).unwrap();
    assert_eq!(check(&restored), pending);
    restored
        .edit_geometry(
            scope,
            restored.expected(),
            &[Command::AcknowledgeLayout {
                stamp: pending.0.unwrap(),
            }],
            &Default::default(),
        )
        .unwrap();
    let export = Document::export_document(&restored, restored.expected()).unwrap();
    let (reopened, _) = Document::open_document(&export.bytes, Limits::default(), &rt).unwrap();
    assert_eq!(check(&reopened), (pending.0, false));
    drop(reopened);
    drop(restored);
    drop(book);
    let memory = rt.memory();
    assert!(rt.finish().is_ok());
    assert_eq!(memory.stats().charged, 0);
}
