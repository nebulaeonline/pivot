use pivot_core::model::integration::IntegrationRuntime;
use pivot_workbook::*;
use pivot_workbook_document::Document;
fn p(r: u32, c: u32) -> Position {
    Position::new(r, c)
}
fn e(r: u32, c: u32, s: &str) -> Edit {
    Edit::SetEntry {
        position: p(r, c),
        source: s.into(),
    }
}
#[test]
fn wb7_restores_directory_bindings_and_authored_and_effective_names() {
    let rt = IntegrationRuntime::standalone().unwrap();
    let mut b = Workbook::new_in("saved", Limits::default(), &rt).unwrap();
    let home = b.worksheet();
    let source = b.add_sheet(b.expected(), Some("Revenue")).unwrap();
    b.apply_sheet(source, b.expected(), &[e(1, 1, "10"), e(2, 1, "20")])
        .unwrap();
    b.apply_sheet(
        home,
        b.expected(),
        &[
            e(1, 1, "2"),
            e(1, 2, "=SUM(Revenue!A1:A2,A1,MAX(Revenue!A1,5))"),
        ],
    )
    .unwrap();
    let formula = b.cell_at_sheet(home, p(1, 2)).unwrap().unwrap();
    b.rename_sheet(b.expected(), source, "O'Brien Sales")
        .unwrap();
    let saved = Document::export_document(&b, b.expected()).unwrap();
    assert_eq!(&saved.bytes[..8], b"PIVOTWB7");
    let (mut reopened, _) = Document::open_document(&saved.bytes, Limits::default(), &rt).unwrap();
    assert_eq!(reopened.sheets(), b.sheets());
    assert_eq!(
        reopened.cell_at_sheet(home, p(1, 2)).unwrap().unwrap(),
        formula
    );
    assert_eq!(
        reopened
            .display_formula(formula.reference)
            .unwrap()
            .unwrap(),
        "=SUM('O''Brien Sales'!A1:A2,A1,MAX('O''Brien Sales'!A1,5))"
    );
    reopened
        .apply_sheet(source, reopened.expected(), &[e(1, 1, "100")])
        .unwrap();
    let cap = reopened.capture_sheet(home, reopened.expected()).unwrap();
    assert_eq!(
        cap.evaluate(&[formula.reference]).unwrap().values[0].1,
        Ok(222.into())
    );
    drop((cap, reopened, b));
    assert!(rt.finish().is_ok());
}
#[test]
fn unknown_sheet_draft_is_not_rebound_by_create_or_reopen() {
    let rt = IntegrationRuntime::standalone().unwrap();
    let mut b = Workbook::new_in("draft", Limits::default(), &rt).unwrap();
    b.apply(b.expected(), &[e(1, 1, "=Missing!A1")]).unwrap();
    b.add_sheet(b.expected(), Some("Missing")).unwrap();
    let saved = Document::export_document(&b, b.expected()).unwrap();
    let (opened, _) = Document::open_document(&saved.bytes, Limits::default(), &rt).unwrap();
    assert!(matches!(
        opened.cell_at(p(1, 1)).unwrap().unwrap().content,
        Content::TextFormula(TextFormula {
            diagnostic: Some(_),
            ..
        })
    ));
    drop((opened, b));
    assert!(rt.finish().is_ok());
}
