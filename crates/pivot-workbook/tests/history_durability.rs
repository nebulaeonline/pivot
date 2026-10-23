#![cfg(feature = "history-experiment")]
use pivot_core::model::integration::IntegrationRuntime;
use pivot_workbook::{
    history_experiment::{ColdLedger, durability::*},
    *,
};
use std::{fs, path::PathBuf};
use uuid::Uuid;
fn directory() -> PathBuf {
    let p = std::env::temp_dir().join(format!("pivot-stage3-{}", Uuid::new_v4()));
    fs::create_dir(&p).unwrap();
    p
}
fn cleanup(dir: PathBuf, rt: IntegrationRuntime) {
    rt.flush();
    let m = rt.memory();
    assert!(rt.finish().is_ok());
    assert_eq!(m.stats().charged, 0);
    let dir = dir.canonicalize().unwrap();
    assert!(dir.starts_with(std::env::temp_dir().canonicalize().unwrap()));
    assert!(
        dir.file_name()
            .unwrap()
            .to_string_lossy()
            .starts_with("pivot-stage3-")
    );
    fs::remove_dir_all(dir).unwrap();
}
fn edit(book: &mut Workbook, text: &str) {
    book.apply(
        book.expected(),
        &[Edit::SetEntry {
            position: Position::new(1, 1),
            source: text.into(),
        }],
    )
    .unwrap();
}

#[test]
fn acknowledged_cursor_branch_and_duplicate_operations_survive_reopen() {
    let rt = IntegrationRuntime::standalone().unwrap();
    let dir = directory();
    let path = dir.join("journal");
    let mut j = Journal::create(&path, Config::default(), &rt).unwrap();
    assert!(Journal::open(&path, &rt).is_err()); // Exclusive ownership of the actual journal file.
    let mut book = Workbook::new_in("journal", Limits::default(), &rt).unwrap();
    edit(&mut book, "10");
    let mut ledger = ColdLedger::new(&book).unwrap();
    let a = Candidate::capture(&book, &ledger).unwrap();
    let a_id = Uuid::new_v4();
    j.commit(a_id, &a, &rt, &mut Fault::default()).unwrap();
    edit(&mut book, "20");
    ledger.observe(&book).unwrap();
    let b = Candidate::capture(&book, &ledger).unwrap();
    let b_id = Uuid::new_v4();
    j.commit(b_id, &b, &rt, &mut Fault::default()).unwrap();
    let undo = Uuid::new_v4();
    j.undo(undo, &mut Fault::default()).unwrap();
    let old = j.receipt().unwrap();
    assert_eq!(j.current_bytes().unwrap().bytes, a.bytes());
    let (mut restored, mut l) = j.restore(&rt).unwrap();
    let unchanged = Candidate::capture(&restored, &l).unwrap();
    let bytes = fs::metadata(&path).unwrap().len();
    assert!(
        j.commit(Uuid::new_v4(), &unchanged, &rt, &mut Fault::default())
            .is_err()
    );
    assert_eq!(fs::metadata(&path).unwrap().len(), bytes);
    assert_eq!(j.receipt().unwrap().redo, old.redo);
    assert!(j.undo(b_id, &mut Fault::default()).is_err());
    j.redo(Uuid::new_v4(), &mut Fault::default()).unwrap();
    assert_eq!(j.current_bytes().unwrap().bytes, b.bytes());
    j.undo(Uuid::new_v4(), &mut Fault::default()).unwrap();
    edit(&mut restored, "30");
    l.observe(&restored).unwrap();
    let c = Candidate::capture(&restored, &l).unwrap();
    let c_id = Uuid::new_v4();
    j.commit(c_id, &c, &rt, &mut Fault::default()).unwrap();
    assert_eq!(j.receipt().unwrap().redo, 0);
    assert!(j.redo(Uuid::new_v4(), &mut Fault::default()).is_err());
    drop(j);
    let mut j = Journal::open(&path, &rt).unwrap();
    assert_eq!(j.current_bytes().unwrap().bytes, c.bytes());
    assert!(matches!(j.lookup(c_id).unwrap(), Outcome::Committed(_)));
    assert!(matches!(j.lookup(b_id).unwrap(), Outcome::Committed(_))); // Historical decision retained, no replay/resurrection of its Redo branch.
    assert!(j.commit(c_id, &c, &rt, &mut Fault::default()).is_err());
    drop((j, book, ledger, restored, l, unchanged, a, b, c));
    cleanup(dir, rt);
}
