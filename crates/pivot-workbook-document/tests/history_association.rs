#![cfg(feature = "history-experiment")]
use pivot_core::model::integration::IntegrationRuntime;
use pivot_storage::experimental_file::SavePoint;
use pivot_workbook::{
    history_experiment::{ColdLedger, durability::Candidate},
    *,
};
use pivot_workbook_document::history_association::*;
use std::{fs, path::PathBuf};
use uuid::Uuid;
fn dir() -> PathBuf {
    let p = std::env::temp_dir().join(format!("pivot-stage4-{}", Uuid::new_v4()));
    fs::create_dir(&p).unwrap();
    p.canonicalize().unwrap()
}
fn edit(b: &mut Workbook, s: &str) {
    b.apply(
        b.expected(),
        &[Edit::SetEntry {
            position: Position::new(1, 1),
            source: s.into(),
        }],
    )
    .unwrap();
}
fn finish(root: PathBuf, rt: IntegrationRuntime) {
    rt.flush();
    let m = rt.memory();
    assert!(rt.finish().is_ok());
    assert_eq!(m.stats().charged, 0);
    let resolved = root.canonicalize().unwrap();
    assert!(resolved.starts_with(std::env::temp_dir().canonicalize().unwrap()));
    assert!(
        resolved
            .file_name()
            .unwrap()
            .to_string_lossy()
            .starts_with("pivot-stage4-")
    );
    fs::remove_dir_all(resolved).unwrap();
}
#[test]
fn ordinary_open_recovery_clear_and_historical_save_keep_separate_authorities() {
    let rt = IntegrationRuntime::standalone().unwrap();
    let root = dir();
    let path = root.join("book.pivot");
    let mut b = Workbook::new_in("association", Limits::default(), &rt).unwrap();
    edit(&mut b, "10");
    let mut l = ColdLedger::new(&b).unwrap();
    let mut a = Association::create(&root.join("history"), &b, &l, Policy::Retained, &rt).unwrap();
    let ca = b.history_content_id();
    let t = a.prepare(&b, &path, Hook::default()).unwrap();
    a.finish(&t, &rt, Hook::default(), |_| Ok(())).unwrap();
    drop(t);
    edit(&mut b, "20");
    b.apply(
        b.expected(),
        &[Edit::SetEntry {
            position: Position::new(2, 1),
            source: "B-only".into(),
        }],
    )
    .unwrap();
    l.observe(&b).unwrap();
    let floor = b.history_metrics().unwrap().next_node;
    let c = Candidate::capture(&b, &l).unwrap();
    a.edit(&c, &rt).unwrap();
    drop(c);
    let cb = b.history_content_id();
    let t = a.prepare(&b, &path, Hook::default()).unwrap();
    let checkpoint = t.content_checkpoint();
    a.undo().unwrap();
    a.clear(&rt, Hook::default()).unwrap();
    assert!(a.redo().is_err());
    let (live, ledger) = a.restore(&rt).unwrap();
    assert_eq!(live.history_content_id(), ca);
    assert_eq!(live.history_metrics().unwrap().next_node, floor);
    let copied = root.join("copy.pivot");
    fs::copy(&path, &copied).unwrap();
    assert!(a.verify_attachment(&copied, &rt).is_err());
    let (ordinary, _) = open_saved(&copied, &rt).unwrap();
    drop(ordinary);
    a.finish(&t, &rt, Hook::default(), |_| Ok(())).unwrap();
    assert_eq!(a.saved().unwrap().unwrap().content, cb);
    assert_eq!(open_saved(&path, &rt).unwrap().1, checkpoint);
    assert!(!a.is_clean(&live, &path, &rt).unwrap());
    drop(t);
    drop(a);
    let mut a = Association::open(&root.join("history"), &rt).unwrap();
    a.verify_attachment(&path, &rt).unwrap();
    let (recovered, cl) = a.recover_matching(&path, &rt).unwrap();
    assert_eq!(recovered.history_content_id(), ca);
    assert_eq!(recovered.history_metrics().unwrap().next_node, floor);
    drop((a, recovered, cl, live, ledger, b, l));
    finish(root, rt);
}
#[test]
fn uncertain_installation_and_control_decisions_require_explicit_reconciliation() {
    let rt = IntegrationRuntime::standalone().unwrap();
    let root = dir();
    let path = root.join("book.pivot");
    let mut b = Workbook::new_in("association", Limits::default(), &rt).unwrap();
    edit(&mut b, "10");
    let mut l = ColdLedger::new(&b).unwrap();
    let mut a = Association::create(&root.join("history"), &b, &l, Policy::Retained, &rt).unwrap();
    let t = a.prepare(&b, &path, Hook::default()).unwrap();
    a.finish(&t, &rt, Hook::default(), |_| Ok(())).unwrap();
    drop(t);
    edit(&mut b, "20");
    l.observe(&b).unwrap();
    let c = Candidate::capture(&b, &l).unwrap();
    a.edit(&c, &rt).unwrap();
    drop(c);
    let t = a.prepare(&b, &path, Hook::default()).unwrap();
    assert!(
        a.finish(&t, &rt, Hook::default(), |p| {
            if p == SavePoint::AfterInstall {
                Err(std::io::Error::other("uncertain"))
            } else {
                Ok(())
            }
        })
        .is_err()
    );
    assert!(a.clear(&rt, Hook::default()).is_err());
    assert!(a.break_retention(&rt, Hook::default()).is_err());
    assert!(a.undo().is_err());
    assert!(a.close(false, true, Hook::default()).is_err());
    a.reconcile_install(t.id(), &rt, Hook::default()).unwrap();
    assert!(a.is_clean(&b, &path, &rt).unwrap());
    drop(t);
    for (point, committed) in [
        (Point::PartialControlDecision, false),
        (Point::AfterControlDecision, true),
    ] {
        assert!(a.clear(&rt, Hook(Some((point, Action::Error)))).is_err());
        let id = a.control_pending().unwrap();
        assert!(a.saved().is_err());
        assert!(a.restore(&rt).is_err());
        assert!(a.break_retention(&rt, Hook::default()).is_err());
        assert!(a.reconcile_control(Uuid::new_v4(), &rt).is_err());
        assert_eq!(a.reconcile_control(id, &rt).unwrap(), committed);
    }
    assert!(a.is_clean(&b, &path, &rt).unwrap());
    drop((a, b, l));
    finish(root, rt);
}
