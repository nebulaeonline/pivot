//! Separate ordinary WB5 baseline from the research SQLite history store.
use pivot_core::model::integration::IntegrationRuntime;
use pivot_workbook::{Content, Limits, Position, Workbook};
use pivot_workbook_document::Document;

fn main() {
    let args: Vec<_> = std::env::args().collect();
    let bytes = std::fs::read(&args[1]).unwrap();
    let rt = IntegrationRuntime::standalone().unwrap();
    let memory = rt.memory();
    let (book, ledger) =
        Workbook::history_cold_reopen(&bytes, &bytes, Limits::default(), &rt).unwrap();
    let exported = Document::export_document(&book, book.expected()).unwrap();
    let (restored, _) = Document::open_document(&exported.bytes, Limits::default(), &rt).unwrap();
    let a = book.capture_local(book.expected()).unwrap();
    let b = restored.capture_local(restored.expected()).unwrap();
    let mut cells = 0;
    for row in 1..=16 {
        for column in 1..=12 {
            let at = Position::new(row, column);
            let original = book.cell_at(at).unwrap();
            let reopened = restored.cell_at(at).unwrap();
            assert_eq!(original.is_some(), reopened.is_some());
            if let (Some(left), Some(right)) = (original, reopened) {
                cells += 1;
                assert_eq!(left.reference.cell, right.reference.cell);
                assert_eq!(left.content, right.content);
                if !matches!(left.content, Content::LiteralText(_)) {
                    assert_eq!(
                        a.evaluate(&[left.reference]).unwrap().values[0].1,
                        b.evaluate(&[right.reference]).unwrap().values[0].1
                    );
                }
            }
        }
    }
    std::fs::write(&args[2], &exported.bytes).unwrap();
    println!(
        "{{\"native_wb5_bytes\":{},\"cold_baseline_bytes\":{},\"cells\":{cells}}}",
        exported.bytes.len(),
        bytes.len()
    );
    drop((a, b, book, restored, ledger, exported));
    rt.flush();
    assert!(rt.finish().is_ok());
    assert_eq!(memory.stats().charged, 0);
    println!("FINISH 0");
}
