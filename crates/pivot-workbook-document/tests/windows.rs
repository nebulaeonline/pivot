use pivot_core::model::integration::IntegrationRuntime;
use pivot_workbook::{structure::*, *};
use pivot_workbook_document::Document;
#[test]
fn scope_tags_axis_runs_and_deleted_window_faults_round_trip() {
    let rt = IntegrationRuntime::standalone().unwrap();
    let mut b = Workbook::new_in("window save", Limits::default(), &rt).unwrap();
    let id = b
        .create_grid_window(b.expected(), "Lines", 100, 10)
        .unwrap();
    let scope = id.into();
    b.apply_scope(
        scope,
        b.expected(),
        &[Edit::SetEntry {
            position: Position::new(5, 1),
            source: "12".into(),
        }],
    )
    .unwrap();
    let original_axis = b.axes_at_sheet(scope, Position::new(5, 1)).unwrap();
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
    b.apply(
        b.expected(),
        &[Edit::SetEntry {
            position: Position::new(1, 1),
            source: "=Lines|A6".into(),
        }],
    )
    .unwrap();
    let export = Document::export_document(&b, b.expected()).unwrap();
    assert_eq!(&export.bytes[..9], b"PIVOTWB14");
    let (mut reopened, _) = Document::open_document(&export.bytes, Limits::default(), &rt).unwrap();
    assert_eq!(
        reopened.axes_at_sheet(scope, Position::new(6, 1)).unwrap(),
        original_axis
    );
    let cap = reopened.capture_local(reopened.expected()).unwrap();
    let out = cap.cell_at(Position::new(1, 1)).unwrap();
    assert_eq!(cap.evaluate(&[out]).unwrap().values[0].1, Ok(12.into()));
    reopened
        .delete_window(reopened.expected(), id, &Default::default())
        .unwrap();
    let export = Document::export_document(&reopened, reopened.expected()).unwrap();
    let (closed, _) = Document::open_document(&export.bytes, Limits::default(), &rt).unwrap();
    let cap = closed.capture_local(closed.expected()).unwrap();
    assert!(
        cap.evaluate(&[cap.cell_at(Position::new(1, 1)).unwrap()])
            .unwrap()
            .values[0]
            .1
            .is_err()
    );
}
#[test]
fn rejected_window_bounds_remain_editable_after_reopen_and_hostile_runs_refuse() {
    let rt = IntegrationRuntime::standalone().unwrap();
    let mut b = Workbook::new_in("invalid Window", Limits::default(), &rt).unwrap();
    let id = b.create_grid_window(b.expected(), "Small", 10, 4).unwrap();
    b.apply(
        b.expected(),
        &[Edit::SetEntry {
            position: Position::new(1, 1),
            source: "=Small|A11".into(),
        }],
    )
    .unwrap();
    let saved = Document::export_document(&b, b.expected()).unwrap();
    let (reopened, _) = Document::open_document(&saved.bytes, Limits::default(), &rt).unwrap();
    assert!(
        matches!(reopened.cell_at(Position::new(1, 1)).unwrap().unwrap().content, Content::TextFormula(ref f) if f.diagnostic.is_some())
    );
    let mut image = b.export_portable(b.expected()).unwrap();
    let window = image
        .sheets
        .iter_mut()
        .find(|s| s.id == GridScopeId::Window(id))
        .unwrap();
    window.row_axis.runs = vec![
        AxisRun { first: 1, count: 5 },
        AxisRun { first: 1, count: 5 },
    ];
    assert!(Workbook::restore_portable(image, Limits::default(), &rt).is_err());
    let mut image = b.export_portable(b.expected()).unwrap();
    image.sheets[0].id = id.into();
    assert!(Workbook::restore_portable(image, Limits::default(), &rt).is_err());
}
