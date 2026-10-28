use pivot_core::{expression::AggregateKind as K, model::integration::IntegrationRuntime};
use pivot_workbook::*;
fn p(r: u32, c: u32) -> Position {
    Position::new(r, c)
}
fn entry(r: u32, c: u32, s: &str) -> Edit {
    Edit::SetEntry {
        position: p(r, c),
        source: s.into(),
    }
}
fn result(b: &Workbook, r: u32, c: u32) -> std::result::Result<String, CellError> {
    let capture = b.capture_local(b.expected()).unwrap();
    let cell = capture.cell_at(p(r, c)).unwrap();
    capture.evaluate(&[cell]).unwrap().values[0]
        .1
        .clone()
        .map(|v| v.to_string())
}
#[test]
fn member_kinds_invalidation_and_old_captures() {
    let runtime = IntegrationRuntime::standalone().unwrap();
    let mut b = Workbook::new_in("aggregates", Limits::default(), &runtime).unwrap();
    b.apply(
        b.expected(),
        &[
            entry(1, 1, "2"),
            entry(2, 1, "4"),
            entry(3, 1, "Label"),
            entry(1, 3, "=MIN(A1:A4)"),
            entry(2, 3, "=MAX(A1:A4)"),
            entry(3, 3, "=AVERAGE(A1:A4)"),
            entry(4, 3, "=COUNT(A1:A4)"),
            entry(5, 3, "=SUM(A1:A4)"),
        ],
    )
    .unwrap();
    for (row, value) in [(1, "2"), (2, "4"), (3, "3"), (4, "2")] {
        assert_eq!(result(&b, row, 3), Ok(value.into()));
    }
    assert_eq!(result(&b, 5, 3), Err(CellError::NonNumericInput));
    b.apply(b.expected(), &[entry(1, 5, "=A4")]).unwrap(); // Materializing a blank anchor cannot add a number.
    assert_eq!(result(&b, 4, 3), Ok("2".into()));
    let blank = b.cell_at(p(4, 1)).unwrap().unwrap().reference;
    let old = b.capture_local(b.expected()).unwrap();
    let count = old.cell_at(p(4, 3)).unwrap();
    b.apply(b.expected(), &[entry(4, 1, "0")]).unwrap();
    assert_eq!(b.cell_at(p(4, 1)).unwrap().unwrap().reference, blank);
    assert_eq!(result(&b, 4, 3), Ok("3".into()));
    assert_eq!(result(&b, 3, 3), Ok("2".into()));
    assert_eq!(old.evaluate(&[count]).unwrap().values[0].1, Ok(2.into()));
    b.apply(b.expected(), &[Edit::Clear(p(4, 1))]).unwrap();
    assert_eq!(result(&b, 4, 3), Ok("2".into()));
    b.apply(b.expected(), &[entry(4, 1, "=E2")]).unwrap(); // Successful formula over blank is numeric zero.
    assert_eq!(result(&b, 4, 3), Ok("3".into()));
    b.apply(b.expected(), &[entry(4, 1, "=A3")]).unwrap();
    // Calculated strings are successful values under ADR-0025 and COUNT ignores them.
    assert_eq!(result(&b, 4, 3), Ok("2".into()));
    let typed = b.capture_local(b.expected()).unwrap();
    assert!(matches!(
        &typed
            .evaluate_values(&[typed.cell_at(p(4, 1)).unwrap()])
            .unwrap()
            .values[0]
            .1,
        Ok(pivot_types::PivotValue::String(_))
    ));
    drop(typed);
    b.enable_structured_proof(uuid::Uuid::new_v4(), uuid::Uuid::new_v4())
        .unwrap();
    let source = "=@{InvoiceLines.TotalAmount}";
    let occurrences = b.resolve_structured(source).unwrap();
    b.apply_structured(b.expected(), p(1, 1), source, &occurrences)
        .unwrap();
    let mut failed = b.capture_local(b.expected()).unwrap();
    failed
        .prepare_structured_failure(pivot_workbook::structured::SourceFault::UnavailableResult)
        .unwrap();
    let consumers = [
        failed.cell_at(p(3, 3)).unwrap(),
        failed.cell_at(p(4, 3)).unwrap(),
    ];
    let errors = failed.evaluate(&consumers).unwrap();
    let first = match errors.values[0].1 {
        Err(CellError::CapturedSourceFailed {
            fault_id,
            kind: pivot_workbook::structured::SourceFault::UnavailableResult,
        }) => fault_id,
        ref other => panic!("{other:?}"),
    };
    assert!(
        matches!(errors.values[1].1,Err(CellError::CapturedSourceFailed{fault_id,..}) if fault_id==first)
    );
    drop((errors, failed));
    drop((old, b));
    let memory = runtime.memory();
    assert!(runtime.finish().is_ok());
    assert_eq!(memory.stats().charged, 0);
}
#[test]
fn no_values_precedence_and_canonical_extrema() {
    let runtime = IntegrationRuntime::standalone().unwrap();
    let mut b = Workbook::new_in("errors", Limits::default(), &runtime).unwrap();
    b.apply(
        b.expected(),
        &[
            entry(1, 3, "=MIN(A1:A4)"),
            entry(2, 3, "=MAX(A1:A4)"),
            entry(3, 3, "=AVERAGE(A1:A4)"),
            entry(4, 3, "=COUNT(A1:A4)"),
        ],
    )
    .unwrap();
    for (row, kind) in [(1, K::Min), (2, K::Max), (3, K::Average)] {
        assert_eq!(result(&b, row, 3), Err(CellError::NoNumericValues(kind)));
    }
    assert_eq!(result(&b, 4, 3), Ok("0".into()));
    b.apply(b.expected(), &[entry(1, 1, "1.00"), entry(2, 1, "1.0")])
        .unwrap();
    assert_eq!(result(&b, 1, 3), Ok("1".into()));
    assert_eq!(result(&b, 2, 3), Ok("1".into()));
    b.apply(b.expected(), &[entry(1, 1, "-0.00"), entry(2, 1, "0.000")])
        .unwrap();
    assert_eq!(result(&b, 1, 3), Ok("0".into()));
    assert_eq!(result(&b, 2, 3), Ok("0".into()));
    b.apply(
        b.expected(),
        &[
            entry(1, 1, "=1/0"),
            entry(2, 1, "Label"),
            entry(3, 1, "=A2"),
        ],
    )
    .unwrap();
    assert!(matches!(result(&b, 4, 3), Err(CellError::DependencyFailed)));
    b.apply(
        b.expected(),
        &[Edit::Move {
            from: p(1, 1),
            to: p(5, 1),
        }],
    )
    .unwrap();
    assert_eq!(result(&b, 4, 3), Ok("0".into())); // Only successful String/Blank members remain.
    drop(b);
    let memory = runtime.memory();
    assert!(runtime.finish().is_ok());
    assert_eq!(memory.stats().charged, 0);
}
#[test]
fn copy_axes_large_ranges_and_exact_mean() {
    let runtime = IntegrationRuntime::standalone().unwrap();
    let mut b = Workbook::new_in("copy", Limits::default(), &runtime).unwrap();
    b.apply(
        b.expected(),
        &[
            entry(1, 1, "79228162514264337593543950335"),
            entry(2, 1, "79228162514264337593543950335"),
            entry(1, 3, "=AVERAGE(A1:A2)"),
            entry(2, 3, "=min($A2:A$1)+COUNT(A1:A2)"),
        ],
    )
    .unwrap();
    assert_eq!(result(&b, 1, 3), Ok("79228162514264337593543950335".into()));
    let rect = Rectangle {
        first: p(1, 3),
        last: p(2, 3),
    };
    let copied = b.copy_block(b.expected(), rect).unwrap();
    b.paste_block(
        b.expected(),
        Rectangle {
            first: p(3, 4),
            last: p(3, 4),
        },
        &copied.native,
        true,
        &Default::default(),
    )
    .unwrap();
    let source = match b.cell_at(p(4, 4)).unwrap().unwrap().content {
        Content::TextFormula(f) => f.original_source,
        _ => panic!(),
    };
    assert_eq!(source, "=min($A4:B$1)+COUNT(B3:B4)");
    for start in [10, 160] {
        let edits: Vec<_> = (start..start + 150).map(|r| entry(r, 8, "1")).collect();
        b.apply(b.expected(), &edits).unwrap();
    }
    b.apply(
        b.expected(),
        &[
            entry(1, 10, "=COUNT(H10:H309)"),
            entry(2, 10, "=AVERAGE(H10:H309)"),
            entry(3, 10, "=MAX(H10:H309)-MIN(H10:H309)"),
        ],
    )
    .unwrap();
    assert_eq!(result(&b, 1, 10), Ok("300".into()));
    assert_eq!(result(&b, 2, 10), Ok("1".into()));
    assert_eq!(result(&b, 3, 10), Ok("0".into()));
    drop(b);
    let memory = runtime.memory();
    assert!(runtime.finish().is_ok());
    assert_eq!(memory.stats().charged, 0);
}
