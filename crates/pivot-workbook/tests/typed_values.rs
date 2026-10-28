use pivot_types::PivotValue as V;
use pivot_workbook::*;
fn p(r: u32, c: u32) -> Position {
    Position::new(r, c)
}
fn enter(b: &mut Workbook, r: u32, c: u32, s: &str) {
    b.apply(
        b.expected(),
        &[Edit::SetEntry {
            position: p(r, c),
            source: s.into(),
        }],
    )
    .unwrap();
}
fn value(b: &Workbook, r: u32, c: u32) -> std::result::Result<V, CellError> {
    let cap = b.capture_local(b.expected()).unwrap();
    let cell = cap.cell_at(p(r, c)).unwrap();
    cap.evaluate_values(&[cell]).unwrap().values.remove(0).1
}
#[test]
fn canonical_short_circuit_does_not_execute_unused_dependency() {
    let mut b = Workbook::new("demand", Limits::default()).unwrap();
    enter(&mut b, 1, 1, "=1/0");
    enter(&mut b, 1, 2, "=IF(FALSE,A1,\"ok\")");
    let cap = b.capture_local(b.expected()).unwrap();
    let cell = cap.cell_at(p(1, 2)).unwrap();
    let result = cap.evaluate_values(&[cell]).unwrap();
    assert_eq!(result.values[0].1, Ok(V::String("ok".into())));
    assert_eq!(
        result.evaluated_nodes, 1,
        "unused A1 must not execute in the canonical graph"
    );
    drop(cap);
    enter(&mut b, 2, 1, "=AND(FALSE,A1)");
    assert_eq!(value(&b, 2, 1), Ok(V::Boolean(false)));
    enter(&mut b, 2, 2, "=OR(TRUE,A1)");
    assert_eq!(value(&b, 2, 2), Ok(V::Boolean(true)));
    enter(&mut b, 3, 1, "=IF(FALSE,A3,1)");
    let cap = b.capture_local(b.expected()).unwrap();
    let cell = cap.cell_at(p(3, 1)).unwrap();
    assert!(
        cap.evaluate_values(&[cell]).is_err(),
        "inactive structural cycles remain invalid"
    );
}
#[test]
fn strings_booleans_unicode_aggregates_and_blank_are_distinct() {
    let mut b = Workbook::new("types", Limits::default()).unwrap();
    for (r, s) in [
        "TRUE",
        "'TRUE",
        "'",
        "=\"\"",
        "=LEN(\"😀a\")",
        "=MID(\"😀ab\",2,99)",
        "=\"a\"&\"b\"",
        "=COUNT(\"42\",TRUE,7)",
        "=\"é\"=\"é\"",
        "=IF(1,2,3)",
        "=1+2*3=7",
        "=SUM(\"1\")",
    ]
    .iter()
    .enumerate()
    {
        enter(&mut b, r as u32 + 1, 1, s);
    }
    assert_eq!(value(&b, 1, 1), Ok(V::Boolean(true)));
    assert_eq!(value(&b, 2, 1), Ok(V::String("TRUE".into())));
    assert_eq!(value(&b, 3, 1), Ok(V::String(String::new())));
    assert_eq!(value(&b, 4, 1), Ok(V::String(String::new())));
    assert_eq!(value(&b, 5, 1), Ok(V::Decimal(2.into())));
    assert_eq!(value(&b, 6, 1), Ok(V::String("ab".into())));
    assert_eq!(value(&b, 7, 1), Ok(V::String("ab".into())));
    assert_eq!(value(&b, 8, 1), Ok(V::Decimal(1.into())));
    assert_eq!(value(&b, 9, 1), Ok(V::Boolean(false)));
    assert!(matches!(value(&b, 10, 1), Err(CellError::Typed(_))));
    assert_eq!(value(&b, 11, 1), Ok(V::Boolean(true)));
    assert!(matches!(value(&b, 12, 1), Err(CellError::Typed(_))));
    enter(&mut b, 13, 1, "=B13");
    assert_eq!(value(&b, 13, 1), Ok(V::Decimal(0.into())));
}

#[test]
fn cross_sheet_typed_binding_copy_fill_captures_and_limits() {
    let mut b = Workbook::new("cross types", Limits::default()).unwrap();
    let home = b.worksheet();
    let input = b.add_sheet(b.expected(), Some("Input")).unwrap();
    b.apply_sheet(
        input,
        b.expected(),
        &[
            Edit::SetEntry {
                position: p(1, 1),
                source: "hello".into(),
            },
            Edit::SetEntry {
                position: p(2, 1),
                source: "TRUE".into(),
            },
        ],
    )
    .unwrap();
    enter(&mut b, 1, 1, "=IF(Input!$A$2,Input!A1&\"!\",\"no\")");
    let old = b.capture_local(b.expected()).unwrap();
    let output = old.cell_at(p(1, 1)).unwrap();
    b.rename_sheet(b.expected(), input, "O'Brien Input")
        .unwrap();
    let copy = b
        .copy_sheet(
            home,
            b.expected(),
            Rectangle {
                first: p(1, 1),
                last: p(1, 1),
            },
        )
        .unwrap();
    assert!(copy.text.contains("'O''Brien Input'!"));
    b.paste_sheet(
        home,
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
        value(&b, 2, 1),
        Err(CellError::Typed(pivot_core::typed::Error::TypeMismatch))
    ); // shifted A2 is Boolean, never implicitly text.
    b.apply_sheet(
        input,
        b.expected(),
        &[Edit::SetEntry {
            position: p(1, 1),
            source: "changed".into(),
        }],
    )
    .unwrap();
    assert_eq!(value(&b, 1, 1), Ok(V::String("changed!".into())));
    assert_eq!(
        old.evaluate_values(&[output]).unwrap().values[0].1,
        Ok(V::String("hello!".into()))
    );
    drop(old);
    let long = "🧾".repeat(1024);
    enter(&mut b, 4, 1, &long);
    enter(&mut b, 5, 1, "=A4&A4");
    assert_eq!(
        value(&b, 5, 1),
        Err(CellError::Typed(pivot_core::typed::Error::TextTooLong))
    );
    let before = b.expected();
    assert!(
        b.apply(
            before,
            &[Edit::SetText {
                position: p(6, 1),
                text: long + "x"
            }]
        )
        .is_err()
    );
    assert_eq!(b.expected(), before);
    let cap = b.capture_local(b.expected()).unwrap();
    let cell = cap.cell_at(p(1, 1)).unwrap();
    let cancel = pivot_core::model::integration::CancellationToken::default();
    cancel.cancel();
    assert!(cap.evaluate_values_cancelled(&[cell], &cancel).is_err());
    assert_eq!(b.expected(), before);
    assert!(cap.evaluate_values(&[cell]).is_ok());
}
