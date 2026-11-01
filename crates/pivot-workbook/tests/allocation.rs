use pivot_core::model::integration::IntegrationRuntime;
use pivot_core::typed::Error as T;
use pivot_types::{DecimalError, PivotDecimal, PivotValue as V};
use pivot_workbook::*;
fn p(r: u32, c: u32) -> Position {
    Position::new(r, c)
}
fn entry(r: u32, c: u32, text: &str) -> Edit {
    Edit::SetEntry {
        position: p(r, c),
        source: text.into(),
    }
}
fn value(b: &Workbook, r: u32, c: u32) -> std::result::Result<V, CellError> {
    let cap = b.capture_local(b.expected()).unwrap();
    cap.evaluate_values(&[cap.cell_at(p(r, c)).unwrap()])
        .unwrap()
        .values
        .remove(0)
        .1
}
fn group(b: &mut Workbook, total: &str, n: u32, places: u32) {
    let mut edits = (1..=n)
        .map(|i| entry(i, 1, &format!("=ALLOCATE({total},{n},{i},{places})")))
        .collect::<Vec<_>>();
    let forward = (1..=n).map(|i| format!("A{i}")).collect::<Vec<_>>();
    let reverse = forward.iter().rev().cloned().collect::<Vec<_>>();
    let mut mixed = forward.clone();
    mixed.rotate_left((n / 2) as usize);
    for (i, order) in [forward, reverse, mixed].iter().enumerate() {
        edits.push(entry(
            100 + i as u32,
            1,
            &format!("=SUM({})", order.join(",")),
        ));
    }
    b.apply(b.expected(), &edits).unwrap();
    let expected = Ok(V::Decimal(
        total.parse::<PivotDecimal>().unwrap().normalize(),
    ));
    for r in 100..=102 {
        assert_eq!(
            value(b, r, 1),
            expected,
            "{total}, {n}, {places}, order {r}"
        );
    }
}
#[test]
fn production_sum_small_groups_and_extreme_normalizing_final_additions() {
    let rt = IntegrationRuntime::standalone().unwrap();
    let mut b = Workbook::new_in("Allocation", Limits::default(), &rt).unwrap();
    for cents in -40i32..=40 {
        let total = format!(
            "{}{}.{:02}",
            if cents < 0 { "-" } else { "" },
            cents.abs() / 100,
            cents.abs() % 100
        );
        for n in 1..=7 {
            group(&mut b, &total, n, 2);
        }
    }
    for total in [
        "79228162514264337593543950335",
        "-79228162514264337593543950335",
    ] {
        for n in [1, 3] {
            group(&mut b, total, n, 28);
        }
    }
    for total in ["10", "-10", "8", "-8", "0.0000000000000000000000000001"] {
        group(&mut b, total, 3, 28);
    }
    drop(b);
    let memory = rt.memory();
    assert!(rt.finish().is_ok());
    assert_eq!(memory.stats().charged, 0);
}
#[test]
fn group_refusal_types_argument_order_and_laziness() {
    let mut b = Workbook::new("Allocation", Limits::default()).unwrap();
    for (text, n, pn) in [
        ("79228162514264337593543950335", 2, 1),
        ("79228162514264337593543950335", 10, 1),
        ("79228162514264337593543950326", 11, 2),
    ] {
        for i in 1..=n {
            b.apply(
                b.expected(),
                &[entry(1, 1, &format!("=ALLOCATE({text},{n},{i},{pn})"))],
            )
            .unwrap();
            assert_eq!(
                value(&b, 1, 1),
                Err(CellError::Arithmetic(DecimalError::Overflow))
            );
        }
    }
    for (source, error) in [
        ("=ALLOCATE(10.005,3,1,2)", T::InvalidArgument),
        ("=ALLOCATE(0,0,1,2)", T::InvalidArgument),
        ("=ALLOCATE(0,4294967296,1,2)", T::InvalidArgument),
        ("=ALLOCATE(0,3,4,2)", T::InvalidArgument),
        ("=ALLOCATE(0,3,1,29)", T::InvalidArgument),
        ("=ALLOCATE(0,3.5,1,2)", T::InvalidArgument),
        ("=ALLOCATE(A200,3,1,2)", T::TypeMismatch),
        ("=ALLOCATE(10,3,1,A200)", T::TypeMismatch),
        ("=ALLOCATE(TRUE,3,1,2)", T::TypeMismatch),
        ("=ALLOCATE(10,3,1,\"2\")", T::TypeMismatch),
        ("=ALLOCATE(TRUE,1/0,1,2)", T::TypeMismatch),
        ("=ALLOCATE(10,0,1/0,2)", T::InvalidArgument),
    ] {
        b.apply(b.expected(), &[entry(1, 1, source)]).unwrap();
        assert_eq!(value(&b, 1, 1), Err(CellError::Typed(error)), "{source}");
    }
    b.apply(b.expected(), &[entry(1, 1, "=ALLOCATE(10,2,3,1/0)")])
        .unwrap();
    assert_eq!(
        value(&b, 1, 1),
        Err(CellError::Arithmetic(DecimalError::DivisionByZero))
    );
    for source in [
        "=ALLOCATE(10,3,1)",
        "=ALLOCATE(10,3,1,2,0)",
        "=ALLOCATE(10,3,1,)",
    ] {
        assert!(pivot_core::expression::parse_spreadsheet(source).is_err());
    }
    for source in [
        "=IF(FALSE,ALLOCATE(10.005,3,1,2),7)",
        "=IF(TRUE,7,ALLOCATE(A200,0,1/0,2))",
    ] {
        b.apply(b.expected(), &[entry(1, 1, source)]).unwrap();
        assert_eq!(value(&b, 1, 1), Ok(V::Decimal(7.into())));
    }
    b.apply(b.expected(), &[entry(1, 1, "=ALLOCATE(10,3.00,1.0,2.000)")])
        .unwrap();
    assert_eq!(value(&b, 1, 1), Ok(V::Decimal("3.34".parse().unwrap())));
}
#[test]
fn cross_sheet_captures_native_copy_and_visible_fill_preserve_explicit_indices() {
    let mut b = Workbook::new("Allocation", Limits::default()).unwrap();
    let sheet = b.worksheet();
    let values = b.add_sheet(b.expected(), Some("Inputs")).unwrap();
    b.apply_sheet(values, b.expected(), &[entry(1, 1, "10")])
        .unwrap();
    b.apply_sheet(
        sheet,
        b.expected(),
        &[entry(1, 1, "=ALLOCATE(Inputs!$A$1,3,1,2)")],
    )
    .unwrap();
    let old = b.capture_sheet(sheet, b.expected()).unwrap();
    let out = old.cell_at(p(1, 1)).unwrap();
    b.apply_sheet(values, b.expected(), &[entry(1, 1, "20")])
        .unwrap();
    assert_eq!(
        old.evaluate_values(&[out]).unwrap().values[0].1,
        Ok(V::Decimal("3.34".parse().unwrap()))
    );
    let image = b
        .copy_sheet(
            sheet,
            b.expected(),
            Rectangle {
                first: p(1, 1),
                last: p(1, 1),
            },
        )
        .unwrap();
    assert!(image.native.starts_with(b"PIVOTCB5"));
    b.paste_sheet(
        sheet,
        b.expected(),
        Rectangle {
            first: p(2, 1),
            last: p(2, 1),
        },
        &image.native,
        true,
        &Default::default(),
    )
    .unwrap();
    b.fill_visible_sheet(
        sheet,
        b.expected(),
        &[1, 5, 12],
        (1, 1),
        FillDirection::Down,
        &Default::default(),
    )
    .unwrap();
    let capture = b.capture_sheet(sheet, b.expected()).unwrap();
    for r in [1, 2, 5, 12] {
        let out = capture.cell_at(p(r, 1)).unwrap();
        assert_eq!(
            capture.evaluate_values(&[out]).unwrap().values[0].1,
            Ok(V::Decimal("6.67".parse().unwrap()))
        );
        assert_eq!(
            b.display_formula(out).unwrap().unwrap(),
            "=ALLOCATE(Inputs!$A$1,3,1,2)"
        );
    }
    assert!(b.cell_at_sheet(sheet, p(6, 1)).unwrap().is_none());
}
