use pivot_workbook::*;
fn p(r: u32, c: u32) -> Position {
    Position::new(r, c)
}
fn e(r: u32, c: u32, s: &str) -> Edit {
    Edit::SetEntry {
        position: p(r, c),
        source: s.into(),
    }
}
fn value(b: &Workbook, s: WorksheetId, r: u32, c: u32) -> String {
    let cap = b.capture_sheet(s, b.expected()).unwrap();
    let cell = cap.cell_at(p(r, c)).unwrap();
    cap.evaluate(&[cell]).unwrap().values[0]
        .1
        .clone()
        .unwrap()
        .to_string()
}
#[test]
fn independent_positions_common_graph_and_effective_names() {
    let mut b = Workbook::new("sheets", Limits::default()).unwrap();
    let first = b.worksheet();
    b.rename_sheet(b.expected(), first, "Revenue").unwrap();
    let costs = b.add_sheet(b.expected(), Some("Costs")).unwrap();
    b.apply_sheet(first, b.expected(), &[e(1, 1, "10"), e(2, 1, "20")])
        .unwrap();
    b.apply_sheet(
        costs,
        b.expected(),
        &[e(1, 1, "3"), e(1, 2, "=SUM(Revenue!A1:A2,Revenue!A1)-A1")],
    )
    .unwrap();
    assert_eq!(value(&b, costs, 1, 2), "37");
    assert_eq!(value(&b, first, 1, 1), "10");
    let cap = b.capture_sheet(costs, b.expected()).unwrap();
    let output = cap.cell_at(p(1, 2)).unwrap();
    let before = b.cell_at_sheet(costs, p(1, 2)).unwrap().unwrap();
    b.rename_sheet(b.expected(), first, "O'Brien Sales")
        .unwrap();
    assert_eq!(b.cell_at_sheet(costs, p(1, 2)).unwrap().unwrap(), before);
    assert_eq!(
        b.display_formula(before.reference).unwrap().unwrap(),
        "=SUM('O''Brien Sales'!A1:A2,'O''Brien Sales'!A1)-A1"
    );
    b.apply_sheet(first, b.expected(), &[e(1, 1, "100")])
        .unwrap();
    assert_eq!(value(&b, costs, 1, 2), "217");
    assert_eq!(cap.evaluate(&[output]).unwrap().values[0].1, Ok(37.into()));
    assert_eq!(b.stats().live_cells, 4);
}
#[test]
fn names_quoting_caps_and_failed_operations() {
    let mut b = Workbook::new("names", Limits::default()).unwrap();
    let sheet = b.add_sheet(b.expected(), Some("O'Brien Costs")).unwrap();
    assert!(b.add_sheet(b.expected(), Some("o'brien costs")).is_err());
    b.apply_sheet(sheet, b.expected(), &[e(1, 1, "9")]).unwrap();
    let home = b.worksheet();
    b.apply_sheet(home, b.expected(), &[e(1, 1, "='O''Brien Costs'!A1")])
        .unwrap();
    assert_eq!(value(&b, home, 1, 1), "9");
    let n = b.add_sheet(b.expected(), Some("Straße")).unwrap();
    let expected = b.expected();
    assert!(b.rename_sheet(expected, n, "STRASSE").is_ok());
    assert!(b.add_sheet(b.expected(), Some("straße")).is_err());
    let before = b.expected();
    assert!(b.rename_sheet(before, n, "bad!name").is_err());
    assert_eq!(b.expected(), before);
    while b.sheets().len() < 16 {
        b.add_sheet(b.expected(), None).unwrap();
    }
    let before = b.expected();
    assert!(b.add_sheet(before, None).is_err());
    assert_eq!(b.expected(), before);
}
#[test]
fn qualified_copy_fill_and_missing_destination_are_atomic() {
    let mut b = Workbook::new("copy", Limits::default()).unwrap();
    let local = b.worksheet();
    let source = b.add_sheet(b.expected(), Some("Revenue")).unwrap();
    b.apply_sheet(source, b.expected(), &[e(1, 1, "10"), e(2, 1, "20")])
        .unwrap();
    b.apply_sheet(
        local,
        b.expected(),
        &[
            e(1, 1, "2"),
            e(2, 1, "4"),
            e(1, 3, "=SUM(Revenue!$A$1:A2,A1)"),
        ],
    )
    .unwrap();
    b.rename_sheet(b.expected(), source, "O'Brien Sales")
        .unwrap();
    let rect = Rectangle {
        first: p(1, 3),
        last: p(1, 3),
    };
    let copied = b.copy_sheet(local, b.expected(), rect).unwrap();
    assert!(copied.text.contains("'O''Brien Sales'!"));
    b.paste_sheet(
        local,
        b.expected(),
        Rectangle {
            first: p(2, 3),
            last: p(2, 3),
        },
        &copied.native,
        true,
        &Default::default(),
    )
    .unwrap();
    assert_eq!(value(&b, local, 2, 3), "34");
    b.fill_sheet(
        local,
        b.expected(),
        Rectangle {
            first: p(1, 3),
            last: p(3, 3),
        },
        FillDirection::Down,
        &Default::default(),
    )
    .unwrap();
    assert_eq!(value(&b, local, 2, 3), "34");
    assert_eq!(value(&b, local, 3, 3), "30");
    let before = b.expected();
    b.fill_sheet(
        local,
        before,
        Rectangle {
            first: p(1, 3),
            last: p(3, 3),
        },
        FillDirection::Down,
        &Default::default(),
    )
    .unwrap();
    assert_eq!(b.expected(), before);
    let mut other = Workbook::new("destination", Limits::default()).unwrap();
    let target = other.worksheet();
    let before = other.expected();
    assert!(
        other
            .paste_sheet(
                target,
                before,
                rect,
                &copied.native,
                true,
                &Default::default()
            )
            .is_err()
    );
    assert_eq!(other.expected(), before);
}

#[test]
fn cross_sheet_cycles_repair_and_reused_names_keep_identity() {
    let mut b = Workbook::new("cycles", Limits::default()).unwrap();
    let first = b.worksheet();
    let other = b.add_sheet(b.expected(), Some("Other")).unwrap();
    b.apply_sheet(first, b.expected(), &[e(1, 1, "=Other!A1+1")])
        .unwrap();
    b.apply_sheet(other, b.expected(), &[e(1, 1, "=Sheet1!A1+1")])
        .unwrap();
    assert!(
        b.validate(b.expected())
            .unwrap()
            .iter()
            .any(|d| matches!(d, Diagnostic::Unorderable { .. }))
    );
    let capture = b.capture_sheet(first, b.expected()).unwrap();
    let output = capture.cell_at(p(1, 1)).unwrap();
    assert!(capture.evaluate(&[output]).is_err());
    drop(capture);
    b.apply_sheet(other, b.expected(), &[e(1, 1, "2")]).unwrap();
    assert_eq!(value(&b, first, 1, 1), "3");
    b.rename_sheet(b.expected(), other, "Renamed").unwrap();
    let replacement = b.add_sheet(b.expected(), Some("Other")).unwrap();
    b.apply_sheet(replacement, b.expected(), &[e(1, 1, "99")])
        .unwrap();
    assert_eq!(value(&b, first, 1, 1), "3");
    let cell = b.cell_at_sheet(first, p(1, 1)).unwrap().unwrap();
    assert_eq!(
        b.display_formula(cell.reference).unwrap().unwrap(),
        "=Renamed!A1+1"
    );
    let image = b.export_portable(b.expected()).unwrap();
    assert_ne!(image.sheets[0].rows, image.sheets[1].rows);
    assert_ne!(image.sheets[0].columns, image.sheets[1].columns);
}

#[test]
fn cell_limits_are_workbook_wide() {
    let mut b = Workbook::new(
        "quota",
        Limits {
            live_cells: 2,
            allocated_cells: 2,
            ..Limits::default()
        },
    )
    .unwrap();
    let first = b.worksheet();
    let other = b.add_sheet(b.expected(), None).unwrap();
    b.apply_sheet(first, b.expected(), &[e(1, 1, "1")]).unwrap();
    b.apply_sheet(other, b.expected(), &[e(1, 1, "2")]).unwrap();
    let before = b.expected();
    assert!(b.apply_sheet(other, before, &[e(2, 1, "3")]).is_err());
    assert_eq!(b.expected(), before);
    assert_eq!(b.stats().live_cells, 2);
    assert_eq!(value(&b, first, 1, 1), "1");
    assert_eq!(value(&b, other, 1, 1), "2");
}
