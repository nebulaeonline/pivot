use pivot_core::{expression::AggregateKind as K, model::integration::IntegrationRuntime};
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
fn result(b: &Workbook, r: u32, c: u32) -> std::result::Result<String, CellError> {
    let s = b.capture_local(b.expected()).unwrap();
    let id = s.cell_at(p(r, c)).unwrap();
    s.evaluate(&[id]).unwrap().values[0]
        .1
        .clone()
        .map(|v| v.to_string())
}
#[test]
fn mixed_arguments_overlap_nested_and_invalidation() {
    let rt = IntegrationRuntime::standalone().unwrap();
    let mut b = Workbook::new_in("arguments", Limits::default(), &rt).unwrap();
    b.apply(
        b.expected(),
        &[
            e(1, 1, "2"),
            e(2, 1, "4"),
            e(1, 2, "8"),
            e(1, 4, "=SUM(A1:A2,B1:B2,5,A1*2)"),
            e(2, 4, "=MIN(A1:A2,1+2,MAX(B1:B2,9))"),
            e(3, 4, "=MAX(A1:A2,1+2,MIN(B1:B2,9))"),
            e(4, 4, "=AVERAGE(A1:A2,B1:B2,10)"),
            e(5, 4, "=COUNT(A1:A2,A1,0,B2,B2+0)"),
            e(6, 4, "=SUM(A1:A2,A1:A2)"),
            e(7, 4, "=SUM(1,AVERAGE(2,4),COUNT(0,3))"),
        ],
    )
    .unwrap();
    for (r, v) in [
        (1, "23"),
        (2, "2"),
        (3, "8"),
        (4, "6"),
        (5, "5"),
        (6, "12"),
        (7, "6"),
    ] {
        assert_eq!(result(&b, r, 4), Ok(v.into()), "row {r}");
    }
    let old = b.capture_local(b.expected()).unwrap();
    let id = old.cell_at(p(4, 4)).unwrap();
    b.apply(b.expected(), &[e(2, 2, "6")]).unwrap();
    assert_eq!(result(&b, 4, 4), Ok("6".into())); // new range occupant counts, too
    b.apply(b.expected(), &[e(2, 2, "16")]).unwrap();
    assert_eq!(result(&b, 4, 4), Ok("8".into()));
    assert_eq!(old.evaluate(&[id]).unwrap().values[0].1, Ok(6.into()));
    drop((old, b));
    let mem = rt.memory();
    assert!(rt.finish().is_ok());
    assert_eq!(mem.stats().charged, 0);
}
#[test]
fn blank_text_empty_errors_and_left_to_right_overflow() {
    let rt = IntegrationRuntime::standalone().unwrap();
    let mut b = Workbook::new_in("policy", Limits::default(), &rt).unwrap();
    b.apply(
        b.expected(),
        &[
            e(1, 1, "label"),
            e(1, 4, "=COUNT(A1,A2,A2+0)"),
            e(2, 4, "=AVERAGE(A1,A2)"),
            e(3, 4, "=MIN(A1:A3,B1:B3)"),
            e(4, 4, "=MAX(A1:A3,7)"),
            e(5, 4, "=SUM(0,A1)"),
            e(6, 4, "=SUM(79228162514264337593543950335,1,-1)"),
            e(
                7,
                4,
                "=AVERAGE(79228162514264337593543950335,79228162514264337593543950335)",
            ),
            e(8, 4, "=COUNT(1/0,A1:A3)"),
            e(9, 4, "=SUM(1/0,A1:A3)"),
            e(10, 4, "=SUM(A1:A3,1/0)"),
        ],
    )
    .unwrap();
    assert_eq!(result(&b, 1, 4), Ok("1".into()));
    assert_eq!(
        result(&b, 2, 4),
        Err(CellError::NoNumericValues(K::Average))
    );
    assert_eq!(result(&b, 3, 4), Err(CellError::NoNumericValues(K::Min)));
    assert_eq!(result(&b, 4, 4), Ok("7".into()));
    assert_eq!(result(&b, 5, 4), Err(CellError::NonNumericInput));
    assert!(matches!(result(&b, 6, 4), Err(CellError::Arithmetic(_))));
    assert_eq!(result(&b, 7, 4), Ok("79228162514264337593543950335".into()));
    assert!(matches!(result(&b, 8, 4), Err(CellError::Arithmetic(_))));
    assert!(matches!(result(&b, 9, 4), Err(CellError::Arithmetic(_))));
    assert_eq!(result(&b, 10, 4), Err(CellError::NonNumericInput));
    drop(b);
    assert!(rt.finish().is_ok());
}
#[test]
fn copy_and_fill_translate_every_argument_axis() {
    let rt = IntegrationRuntime::standalone().unwrap();
    let mut b = Workbook::new_in("copy", Limits::default(), &rt).unwrap();
    b.apply(
        b.expected(),
        &[
            e(1, 1, "2"),
            e(2, 1, "4"),
            e(1, 3, "=SUM($A$1:A2,MAX(A1,3),7)"),
        ],
    )
    .unwrap();
    let rect = Rectangle {
        first: p(1, 3),
        last: p(1, 3),
    };
    let copy = b.copy_block(b.expected(), rect).unwrap();
    b.paste_block(
        b.expected(),
        Rectangle {
            first: p(2, 4),
            last: p(2, 4),
        },
        &copy.native,
        true,
        &Default::default(),
    )
    .unwrap();
    let Content::TextFormula(f) = b.cell_at(p(2, 4)).unwrap().unwrap().content else {
        panic!()
    };
    assert_eq!(f.original_source, "=SUM($A$1:B3,MAX(B2,3),7)");
    assert_eq!(result(&b, 2, 4), Ok("16".into()));
    b.fill(
        b.expected(),
        Rectangle {
            first: p(1, 3),
            last: p(3, 3),
        },
        FillDirection::Down,
        &Default::default(),
    )
    .unwrap();
    assert_eq!(result(&b, 2, 3), Ok("17".into()));
    assert_eq!(result(&b, 3, 3), Ok("16".into()));
    drop(b);
    assert!(rt.finish().is_ok());
}
#[test]
fn mean_rounds_once_across_ranges_and_scalar_arguments() {
    let rt = IntegrationRuntime::standalone().unwrap();
    let mut b = Workbook::new_in("mean", Limits::default(), &rt).unwrap();
    b.apply(
        b.expected(),
        &[
            e(1, 1, "0.0000000000000000000000000001"),
            e(2, 1, "0"),
            e(1, 4, "=AVERAGE(A1:A2,0)"),
            e(2, 4, "=AVERAGE(A1:A2,1)"),
            e(3, 4, "=SUM(1,1/0,A1:A2)"),
            e(4, 4, "=COUNT(0,0,0)"),
        ],
    )
    .unwrap();
    assert_eq!(result(&b, 1, 4), Ok("0".into()));
    assert_eq!(
        result(&b, 2, 4),
        Ok("0.3333333333333333333333333334".into())
    );
    assert!(matches!(result(&b, 3, 4), Err(CellError::Arithmetic(_))));
    assert_eq!(result(&b, 4, 4), Ok("3".into()));
    drop(b);
    assert!(rt.finish().is_ok());
}

#[test]
fn nested_single_range_calls_are_scalar_results_not_outer_range_members() {
    let rt = IntegrationRuntime::standalone().unwrap();
    let mut b = Workbook::new_in("nested", Limits::default(), &rt).unwrap();
    b.apply(
        b.expected(),
        &[
            e(1, 1, "2"),
            e(2, 1, "4"),
            e(1, 4, "=MAX(SUM(A1:A2),5)"),
            e(2, 4, "=AVERAGE(AVERAGE(A1:A2),9)"),
            e(3, 4, "=COUNT(COUNT(A1:A2),0)"),
            e(4, 4, "=SUM(COUNT(A1:A2),MIN(A1:A2))"),
        ],
    )
    .unwrap();
    for (r, v) in [(1, "6"), (2, "6"), (3, "2"), (4, "4")] {
        assert_eq!(result(&b, r, 4), Ok(v.into()));
    }
    drop(b);
    assert!(rt.finish().is_ok());
}
