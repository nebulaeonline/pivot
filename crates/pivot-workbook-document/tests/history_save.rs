#![cfg(feature = "history-experiment")]
//! Prepared-export harness using the unchanged native file adapter; not session history integration.
use pivot_core::model::integration::IntegrationRuntime;
use pivot_storage::experimental_file::native;
use pivot_workbook::*;
use pivot_workbook_document::Document;
use uuid::Uuid;
#[test]
fn prepared_bytes_survive_root_release_and_saved_content_is_not_guard_or_file_authority() {
    let rt = IntegrationRuntime::standalone().unwrap();
    let mut book = Workbook::new_in("save proof", Limits::default(), &rt).unwrap();
    let p = Position::new(1, 1);
    book.apply(
        book.expected(),
        &[Edit::SetEntry {
            position: p,
            source: "10".into(),
        }],
    )
    .unwrap();
    let a = book.pin_history_root(book.expected()).unwrap();
    let original_guard = book.expected();
    let saved_a = book.history_content_id();
    let base = std::env::var_os("PIVOT_FILE_TEST_ROOT")
        .map(std::path::PathBuf::from)
        .unwrap_or_else(std::env::temp_dir);
    let directory = base.join(format!("pivot-history-proof-{}", Uuid::new_v4()));
    std::fs::create_dir_all(&directory).unwrap();
    let path = directory.canonicalize().unwrap().join("proof.pivot");
    let export_a = Document::export_document(&book, book.expected()).unwrap();
    let location_a =
        native::install(&native::destination(&path).unwrap(), &export_a.bytes).unwrap();
    book.apply(
        book.expected(),
        &[Edit::SetEntry {
            position: p,
            source: "20".into(),
        }],
    )
    .unwrap();
    let root_b = book.pin_history_root(book.expected()).unwrap();
    let saved_b = book.history_content_id();
    let prepared = Document::export_document(&book, book.expected()).unwrap();
    book.activate_history_root(book.expected(), &a, &Default::default())
        .unwrap();
    assert_eq!(book.history_content_id(), saved_a);
    assert_ne!(book.expected(), original_guard);
    drop(root_b);
    rt.flush();
    // The only retained B payload here is the immutable export, not a historical workbook root.
    let location_b = native::install(&location_a, &prepared.bytes).unwrap();
    assert_eq!(location_b.path, path);
    assert_ne!(book.history_content_id(), saved_b);
    assert_ne!(prepared.checkpoint, export_a.checkpoint);
    let (bytes, observed) = native::read(&path).unwrap();
    assert_eq!(bytes, prepared.bytes);
    let (restored, checkpoint) = Document::open_document(&bytes, Limits::default(), &rt).unwrap();
    assert_eq!(checkpoint, prepared.checkpoint);
    let s = restored.capture_local(restored.expected()).unwrap();
    let cell = s.cell_at(p).unwrap();
    assert_eq!(s.evaluate(&[cell]).unwrap().values[0].1, Ok(20.into()));
    let reopened = restored.pin_history_root(restored.expected()).unwrap();
    assert!(reopened.weight() > 0);
    // File association/checkpoint belong to the outer owner, not root activation.
    // Production ticket and uncertain-installation state are outside this harness.
    book.activate_history_root(book.expected(), &a, &Default::default())
        .unwrap();
    assert_eq!(observed.path, path);
    assert_eq!(native::read(&path).unwrap().0, bytes);
    drop((reopened, restored, s, a, book));
    rt.flush();
    let memory = rt.memory();
    assert!(rt.finish().is_ok());
    assert_eq!(memory.stats().charged, 0);
    std::fs::remove_file(&path).unwrap();
    let lock = directory.join(".pivot-experimental-write.lock");
    if lock.exists() {
        std::fs::remove_file(lock).unwrap();
    }
    std::fs::remove_dir(directory).unwrap();
}
