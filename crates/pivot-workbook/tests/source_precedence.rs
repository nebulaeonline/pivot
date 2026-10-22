//! ADR-0020 preflight: exercise the real worksheet/canonical evaluator before adding leaves.
use pivot_core::model::integration::IntegrationRuntime;
use pivot_types::DecimalError;
use pivot_workbook::*;
fn p(row: u32, column: u32) -> Position {
    Position::new(row, column)
}
fn entry(row: u32, column: u32, source: &str) -> Edit {
    Edit::SetEntry {
        position: p(row, column),
        source: source.into(),
    }
}
fn error(book: &Workbook, row: u32, column: u32) -> CellError {
    let capture = book.capture_local(book.expected()).unwrap();
    capture
        .evaluate(&[capture.cell_at(p(row, column)).unwrap()])
        .unwrap()
        .values[0]
        .1
        .clone()
        .unwrap_err()
}
#[test]
fn canonical_postfix_and_sum_error_order_before_source_extension() {
    let runtime = IntegrationRuntime::standalone().unwrap();
    let mut book = Workbook::new_in("precedence", Limits::default(), &runtime).unwrap();
    book.apply(
        book.expected(),
        &[
            entry(1, 1, "'failed numeric source stand-in"),
            entry(2, 1, "=1/0"),
            entry(1, 2, "=A1+(1/0)"),
            entry(2, 2, "=(1/0)+A1"),
            entry(3, 2, "=SUM(A1:A2)"),
        ],
    )
    .unwrap();
    assert_eq!(error(&book, 1, 2), CellError::NonNumericInput);
    assert_eq!(
        error(&book, 2, 2),
        CellError::Arithmetic(DecimalError::DivisionByZero)
    );
    assert_eq!(error(&book, 3, 2), CellError::NonNumericInput);
    book.apply(
        book.expected(),
        &[entry(1, 1, "=1/0"), entry(2, 1, "'later failure")],
    )
    .unwrap();
    assert_eq!(error(&book, 3, 2), CellError::DependencyFailed);
    drop(book);
    let memory = runtime.memory();
    assert!(runtime.finish().is_ok());
    assert_eq!(memory.stats().charged, 0);
}
