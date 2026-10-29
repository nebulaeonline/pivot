use pivot_types::PivotValue as V;
use pivot_workbook::*;
fn p(r: u32, c: u32) -> Position {
    Position::new(r, c)
}
fn enter(b: &mut Workbook, sheet: WorksheetId, r: u32, c: u32, s: &str) {
    b.apply_sheet(
        sheet,
        b.expected(),
        &[Edit::SetEntry {
            position: p(r, c),
            source: s.into(),
        }],
    )
    .unwrap();
}
fn value(b: &Workbook, sheet: WorksheetId, r: u32, c: u32) -> std::result::Result<V, CellError> {
    let capture = b.capture_sheet(sheet, b.expected()).unwrap();
    let cell = capture.cell_at(p(r, c)).unwrap();
    capture.evaluate_values(&[cell]).unwrap().values.remove(0).1
}
#[test]
fn rounding_functions_types_precision_errors_and_lazy_branches() {
    let mut b = Workbook::new("round", Limits::default()).unwrap();
    let sheet = b.sheets()[0].id;
    for (r, (source, expected)) in [
        ("=ROUND(2.5,0)", "3"),
        ("=ROUND(-2.5,0)", "-3"),
        ("=ROUNDUP(-2.1,0)", "-3"),
        ("=ROUNDDOWN(-2.9,0)", "-2"),
        ("=ROUND(125,-1)", "130"),
        (
            "=SUM(ROUND(1.25,1),ROUNDUP(1.21,1),ROUNDDOWN(1.29,1))",
            "3.8",
        ),
        ("=ROUND(ROUNDUP(1.231,2)+ROUNDDOWN(2.999,2),1)", "4.2"),
        ("=ROUND(1.23,79228162514264337593543950335)", "1.23"),
        ("=ROUND(1,-79228162514264337593543950335)", "0"),
        ("=ROUNDDOWN(1,-79228162514264337593543950335)", "0"),
        ("=ROUNDUP(0,-79228162514264337593543950335)", "0"),
        ("=ROUND(A100,A100)", "0"),
        ("=IF(FALSE,ROUNDUP(1,-100),7)", "7"),
    ]
    .into_iter()
    .enumerate()
    {
        enter(&mut b, sheet, r as u32 + 1, 1, source);
        assert_eq!(
            value(&b, sheet, r as u32 + 1, 1),
            Ok(V::Decimal(expected.parse().unwrap()))
        );
    }
    for source in [
        "=ROUND(TRUE,0)",
        r#"=ROUND(1,"2")"#,
        "=ROUND(1,0.5)",
        "=ROUNDUP(1,-79228162514264337593543950335)",
        "=ROUND(79228162514264337593543950335,-1)",
    ] {
        enter(&mut b, sheet, 30, 1, source);
        assert!(value(&b, sheet, 30, 1).is_err(), "{source}");
    }
    for source in ["=ROUND(1)", "=ROUND(1,2,3)"] {
        assert!(pivot_core::expression::parse_spreadsheet(source).is_err());
    }
    enter(&mut b, sheet, 30, 1, "=ROUND(TRUE,1/0)");
    assert_eq!(
        value(&b, sheet, 30, 1),
        Err(CellError::Typed(pivot_core::typed::Error::TypeMismatch))
    );
}
#[test]
fn cross_sheet_copy_fill_captures_and_formatting_are_independent() {
    let mut b = Workbook::new("round", Limits::default()).unwrap();
    let sheet = b.sheets()[0].id;
    let source = b.add_sheet(b.expected(), Some("Values")).unwrap();
    enter(&mut b, source, 1, 1, "1.255");
    enter(&mut b, sheet, 1, 1, "=ROUND(Values!A1,2)");
    let old = b.capture_sheet(sheet, b.expected()).unwrap();
    let out = old.cell_at(p(1, 1)).unwrap();
    assert_eq!(
        old.evaluate(&[out]).unwrap().values[0].1,
        Ok("1.26".parse().unwrap())
    );
    let copy = b
        .copy_sheet(
            sheet,
            b.expected(),
            Rectangle {
                first: p(1, 1),
                last: p(1, 1),
            },
        )
        .unwrap();
    b.paste_sheet(
        sheet,
        b.expected(),
        Rectangle {
            first: p(2, 1),
            last: p(2, 1),
        },
        &copy.native,
        true,
        &Default::default(),
    )
    .unwrap();
    assert_eq!(
        b.display_formula(b.cell_at_sheet(sheet, p(2, 1)).unwrap().unwrap().reference)
            .unwrap()
            .unwrap(),
        "=ROUND(Values!A2,2)"
    );
    b.fill_sheet(
        sheet,
        b.expected(),
        Rectangle {
            first: p(1, 1),
            last: p(1, 2),
        },
        FillDirection::Right,
        &Default::default(),
    )
    .unwrap();
    assert_eq!(value(&b, sheet, 1, 2), Ok(V::Decimal(0.into())));
    enter(&mut b, source, 1, 1, "2.345");
    assert_eq!(
        value(&b, sheet, 1, 1),
        Ok(V::Decimal("2.35".parse().unwrap()))
    );
    assert_eq!(
        old.evaluate_values(&[out]).unwrap().values[0].1,
        Ok(V::Decimal("1.26".parse().unwrap()))
    );
    use pivot_workbook::formatting::*;
    b.format_sheet(
        sheet,
        b.expected(),
        Rectangle {
            first: p(1, 1),
            last: p(1, 1),
        },
        StylePatch {
            places: Field::Set(0),
            numeric: Field::Set(NumericFormat::Number),
            ..Default::default()
        },
        false,
        &Default::default(),
    )
    .unwrap();
    assert_eq!(
        value(&b, sheet, 1, 1),
        Ok(V::Decimal("2.35".parse().unwrap()))
    );
}
