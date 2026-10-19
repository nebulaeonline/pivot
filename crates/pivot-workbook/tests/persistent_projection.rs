#![cfg(feature = "persistent-core")]
fn build_book(
    name: &str,
    limits: pivot_workbook::Limits,
) -> pivot_workbook::Result<pivot_workbook::Workbook> {
    use pivot_core::model::integration::IntegrationRuntime;
    static RUNTIME: std::sync::OnceLock<IntegrationRuntime> = std::sync::OnceLock::new();
    let runtime = RUNTIME.get_or_init(|| IntegrationRuntime::standalone().unwrap());
    pivot_workbook::Workbook::new_in(name, limits, runtime)
}
include!("suites/projection.rs");
