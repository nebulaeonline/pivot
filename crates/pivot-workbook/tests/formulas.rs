fn build_book(
    name: &str,
    limits: pivot_workbook::Limits,
) -> pivot_workbook::Result<pivot_workbook::Workbook> {
    pivot_workbook::Workbook::new(name, limits)
}
include!("suites/formulas.rs");
