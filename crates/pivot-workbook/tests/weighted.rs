use pivot_core::{model::integration::IntegrationRuntime, typed::Error as T};
use pivot_types::{DecimalError, PivotDecimal, PivotValue as V};
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
fn value(b: &Workbook, r: u32, c: u32) -> std::result::Result<V, CellError> {
    let cap = b.capture_local(b.expected()).unwrap();
    cap.evaluate_values(&[cap.cell_at(p(r, c)).unwrap()])
        .unwrap()
        .values
        .remove(0)
        .1
}
fn group(b: &mut Workbook, total: &str, weights: &[&str], places: u32) {
    let n = weights.len();
    let list = weights.join(",");
    let mut edits = (1..=n)
        .map(|i| {
            entry(
                i as u32,
                1,
                format!("=ALLOCATEWEIGHTED({total},{i},{places},{list})"),
            )
        })
        .collect::<Vec<_>>();
    let refs = (1..=n).map(|i| format!("A{i}")).collect::<Vec<_>>();
    let reverse = refs.iter().rev().cloned().collect::<Vec<_>>();
    let mut mixed = refs.clone();
    mixed.rotate_left(n / 2);
    for (j, order) in [refs, reverse, mixed].iter().enumerate() {
        edits.push(entry(
            100 + j as u32,
            1,
            format!("=SUM({})", order.join(",")),
        ));
    }
    b.apply(b.expected(), &edits).unwrap();
    for r in 100..=102 {
        assert_eq!(
            value(b, r, 1),
            Ok(V::Decimal(
                total.parse::<PivotDecimal>().unwrap().normalize()
            )),
            "{total} {weights:?} p={places} order={r}"
        );
    }
}
#[test]
fn production_sum_exhaustive_small_groups_zero_orders_and_extreme_normalization() {
    let rt = IntegrationRuntime::standalone().unwrap();
    let mut b = Workbook::new_in("Weighted", Limits::default(), &rt).unwrap();
    for n in 1..=3u32 {
        for code in 1..3u32.pow(n) {
            let mut digits = code;
            let mut weights = Vec::new();
            for _ in 0..n {
                weights.push((digits % 3).to_string());
                digits /= 3;
            }
            let weights = weights.iter().map(String::as_str).collect::<Vec<_>>();
            for units in -2i32..=2 {
                group(&mut b, &units.to_string(), &weights, 0);
                let total = format!("{}0.{:02}", if units < 0 { "-" } else { "" }, units.abs());
                group(&mut b, &total, &weights, 2);
            }
        }
    }
    for total in ["10", "-10", "8"] {
        group(&mut b, total, &["1", "1", "1", "0"], 28);
        group(&mut b, total, &["0", "1", "1", "1"], 28);
        group(&mut b, total, &["1", "0", "1", "1"], 28);
    }
    let m = "79228162514264337593543950335";
    group(&mut b, m, &["0", "1", "0"], 28);
    group(&mut b, m, &["1", "2"], 28);
    group(&mut b, m, &[m, "0.0000000000000000000000000001"], 1);
    drop(b);
    let memory = rt.memory();
    assert!(rt.finish().is_ok());
    assert_eq!(memory.stats().charged, 0);
}
#[test]
fn strict_errors_lazy_demand_and_group_wide_final_award_admission() {
    let mut b = Workbook::new("Weighted", Limits::default()).unwrap();
    for (source, error) in [
        (
            "=ALLOCATEWEIGHTED(10,1,2,A200)",
            CellError::Typed(T::TypeMismatch),
        ),
        (
            "=ALLOCATEWEIGHTED(10,1,2,TRUE)",
            CellError::Typed(T::TypeMismatch),
        ),
        (
            "=ALLOCATEWEIGHTED(10,1,2,\"1\")",
            CellError::Typed(T::TypeMismatch),
        ),
        (
            "=ALLOCATEWEIGHTED(0,1,2,0,0)",
            CellError::Typed(T::InvalidArgument),
        ),
        (
            "=ALLOCATEWEIGHTED(10.005,1,2,1)",
            CellError::Typed(T::InvalidArgument),
        ),
        (
            "=ALLOCATEWEIGHTED(10,1,2,-1,1/0)",
            CellError::Typed(T::InvalidArgument),
        ),
        (
            "=ALLOCATEWEIGHTED(10,3,2,1,1/0)",
            CellError::Arithmetic(DecimalError::DivisionByZero),
        ),
    ] {
        b.apply(b.expected(), &[entry(1, 1, source)]).unwrap();
        assert_eq!(value(&b, 1, 1), Err(error), "{source}");
    }
    for source in [
        "=IF(FALSE,ALLOCATEWEIGHTED(10,1,2,1/0),7)",
        "=IF(TRUE,7,ALLOCATEWEIGHTED(10,1,2,0))",
    ] {
        b.apply(b.expected(), &[entry(1, 1, source)]).unwrap();
        assert_eq!(value(&b, 1, 1), Ok(V::Decimal(7.into())));
    }
    for n in [2, 10, 11] {
        let total = if n == 11 {
            "79228162514264337593543950326"
        } else {
            "79228162514264337593543950335"
        };
        let places = if n == 11 { 2 } else { 1 };
        let ws = vec!["1"; n].join(",");
        for i in 1..=n {
            b.apply(
                b.expected(),
                &[entry(
                    1,
                    1,
                    format!("=ALLOCATEWEIGHTED({total},{i},{places},{ws})"),
                )],
            )
            .unwrap();
            assert_eq!(
                value(&b, 1, 1),
                Err(CellError::Arithmetic(DecimalError::Overflow))
            );
        }
    }
    for n in [0, 65] {
        let list = vec!["1"; n].join(",");
        let source = if n == 0 {
            "=ALLOCATEWEIGHTED(10,1,2)".into()
        } else {
            format!("=ALLOCATEWEIGHTED(10,1,2,{list})")
        };
        assert!(pivot_core::expression::parse_spreadsheet(&source).is_err());
    }
}
