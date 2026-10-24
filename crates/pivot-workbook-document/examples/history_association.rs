//! Fresh-process association research workers; real WB5 and native file installation.
use pivot_core::model::integration::IntegrationRuntime;
use pivot_storage::experimental_file::native;
use pivot_workbook::{
    history_experiment::{ColdLedger, durability::Candidate},
    *,
};
use pivot_workbook_document::history_association::*;
use std::{fs, io::Write, path::Path};
use uuid::Uuid;
fn p(r: u32, c: u32) -> Position {
    Position::new(r, c)
}
fn edit(b: &mut Workbook, r: u32, c: u32, s: &str) {
    b.apply(
        b.expected(),
        &[Edit::SetEntry {
            position: p(r, c),
            source: s.into(),
        }],
    )
    .unwrap();
}
fn oracle(b: &Workbook) -> String {
    format!(
        "{:?}|{:?}|{:?}|{:?}",
        (1..=4)
            .flat_map(|r| (1..=7).map(move |c| p(r, c)))
            .map(|p| b.cell_at(p).unwrap())
            .collect::<Vec<_>>(),
        b.validate(b.expected()).unwrap(),
        b.invalidated_cells(),
        b.history_content_id()
    )
}
fn value(b: &Workbook) -> String {
    let c = b.capture_local(b.expected()).unwrap();
    let cell = c.cell_at(p(1, 1)).unwrap();
    format!(
        "{}",
        c.evaluate(&[cell]).unwrap().values[0].1.as_ref().unwrap()
    )
}
fn finish(rt: IntegrationRuntime) {
    rt.flush();
    let m = rt.memory();
    assert!(rt.finish().is_ok());
    assert_eq!(m.stats().charged, 0);
    println!("FINISH 0 peak={}", m.stats().peak);
}
fn save_oracle(base: &Path, name: &str, b: &Workbook, l: &ColdLedger) {
    let c = Candidate::capture(b, l).unwrap();
    fs::write(base.join(format!("{name}.capsule")), c.bytes()).unwrap();
    fs::write(base.join(format!("{name}.oracle")), oracle(b)).unwrap();
}
fn seed(base: &Path, prepared: bool) {
    fs::create_dir(base).unwrap();
    let rt = IntegrationRuntime::standalone().unwrap();
    let mut b = Workbook::new_in("association", Limits::default(), &rt).unwrap();
    for (r, c, s) in [
        (1, 1, "10"),
        (2, 1, "10"),
        (1, 2, "=A1"),
        (1, 3, "=SUM($A$2:A1)"),
        (1, 4, "=("),
        (1, 5, "=E1"),
        (1, 6, "text λ"),
    ] {
        edit(&mut b, r, c, s);
    }
    b.apply(
        b.expected(),
        &[Edit::SetDecimal {
            position: p(1, 7),
            value: DecimalInput::from_parts(0, 0, 0, true, 5).unwrap(),
        }],
    )
    .unwrap();
    let mut l = ColdLedger::new(&b).unwrap();
    save_oracle(base, "a", &b, &l);
    let mut a = Association::create(&base.join("history"), &b, &l, Policy::Retained, &rt).unwrap();
    let t = a
        .prepare(&b, &base.join("book.pivot"), Hook::default())
        .unwrap();
    a.finish(&t, &rt, Hook::default(), |_| Ok(())).unwrap();
    assert!(a.is_clean(&b, &base.join("book.pivot"), &rt).unwrap());
    drop(t);
    fs::write(
        base.join("saved-a.wb5"),
        native::read(&base.join("book.pivot")).unwrap().0,
    )
    .unwrap();
    for (name, num, row, text) in [("b", "20", 3, "B-only"), ("d", "40", 4, "D-only")] {
        edit(&mut b, 1, 1, num);
        edit(&mut b, row, 1, text);
        if name == "b" {
            edit(&mut b, 1, 2, "=A2");
        }
        l.observe(&b).unwrap();
        let c = Candidate::capture(&b, &l).unwrap();
        a.edit(&c, &rt).unwrap();
        drop(c);
        save_oracle(base, name, &b, &l);
        if prepared && name == "b" {
            let t = a
                .prepare(&b, &base.join("book.pivot"), Hook::default())
                .unwrap();
            fs::write(base.join("ticket"), t.id().to_string()).unwrap();
            drop(t);
        }
    }
    fs::write(
        base.join("floor"),
        b.history_metrics().unwrap().next_node.to_string(),
    )
    .unwrap();
    drop((a, b, l));
    finish(rt);
}
fn check(a: &mut Association, base: &Path, state: &str, rt: &IntegrationRuntime) {
    let (b, l) = a.restore(rt).unwrap();
    assert_eq!(
        oracle(&b),
        fs::read_to_string(base.join(format!("{state}.oracle"))).unwrap()
    );
    assert_eq!(
        b.history_metrics().unwrap().next_node,
        fs::read_to_string(base.join("floor"))
            .unwrap()
            .parse::<u64>()
            .unwrap()
    );
    if state == "d" {
        assert_eq!(
            a.history_bytes().unwrap().bytes,
            fs::read(base.join("d.capsule")).unwrap()
        );
    }
    drop((b, l));
}
fn point(s: &str) -> Point {
    match s {
        "BeforeControl" => Point::BeforeControl,
        "PartialControl" => Point::PartialControl,
        "BeforeControlDecision" => Point::BeforeControlDecision,
        "PartialControlDecision" => Point::PartialControlDecision,
        "AfterControlDecision" => Point::AfterControlDecision,
        "AfterInstall" => Point::AfterInstall,
        "BeforeSaveReceipt" => Point::BeforeSaveReceipt,
        "BeforeMove" => Point::BeforeMove,
        "AfterMove" => Point::AfterMove,
        _ => panic!("point"),
    }
}
fn pause(s: &str) {
    println!("STAGE4_READY {s}");
    std::io::stdout().flush().unwrap();
    loop {
        std::thread::park();
    }
}
fn operate(base: &Path, op: &str, at: &str, action: &str) {
    let rt = IntegrationRuntime::standalone().unwrap();
    let mut a = Association::open(&base.join("history"), &rt).unwrap();
    if op == "hold" {
        pause("locked");
    }
    let hook = if op == "native" {
        Hook::default()
    } else {
        Hook(Some((
            point(at),
            if action == "pause" {
                Action::Pause
            } else {
                Action::Error
            },
        )))
    };
    if op == "move" {
        let result = a.relocate(&base.join("moved-history"), &rt, hook);
        assert!(result.is_err());
        drop(result);
        finish(rt);
        return;
    }
    if op == "close" {
        a.set_policy(Policy::SessionOnly).unwrap();
        assert!(a.close(false, true, hook).is_err());
    } else {
        a.undo().unwrap();
        let id = Uuid::parse_str(&fs::read_to_string(base.join("ticket")).unwrap()).unwrap();
        let t = a.resume_ticket(id).unwrap();
        a.undo().unwrap();
        if op == "clear" {
            assert!(a.clear(&rt, hook).is_err());
        } else {
            assert!(
                a.finish(&t, &rt, hook, |p| {
                    if op == "native" && format!("{p:?}") == at {
                        if action == "pause" {
                            pause(at);
                        }
                        return Err(std::io::Error::other("native fault"));
                    }
                    Ok(())
                })
                .is_err()
            );
        }
        drop(t);
    }
    if let Some(id) = a.control_pending() {
        assert!(a.saved().is_err());
        assert!(a.restore(&rt).is_err());
        assert!(a.clear(&rt, Hook::default()).is_err());
        assert!(a.reconcile_control(Uuid::new_v4(), &rt).is_err());
        assert_eq!(
            a.reconcile_control(id, &rt).unwrap(),
            at == "AfterControlDecision"
        );
        println!("CONTROL_RECONCILED {id}");
    }
    drop(a);
    finish(rt);
}
fn probe(base: &Path, kind: &str, at: &str) {
    let rt = IntegrationRuntime::standalone().unwrap();
    let path = base.join("book.pivot");
    let (ordinary, cp) = open_saved(&path, &rt).unwrap();
    println!("ORDINARY {} {cp}", value(&ordinary));
    if kind == "decline" {
        assert_eq!(value(&ordinary), "10");
        drop(ordinary);
        finish(rt);
        return;
    }
    if kind == "unavailable" {
        assert!(Association::open(&base.join("history"), &rt).is_err());
        assert_eq!(value(&ordinary), "10");
        drop(ordinary);
        finish(rt);
        return;
    }
    let root = if base.join("moved-history").exists() {
        base.join("moved-history")
    } else {
        base.join("history")
    };
    if kind == "close" && at == "AfterControlDecision" {
        assert!(Association::open(&root, &rt).is_err());
        drop(ordinary);
        finish(rt);
        return;
    }
    let mut a = Association::open(&root, &rt).unwrap();
    match kind {
        "recover" | "close" => {
            a.verify_attachment(&path, &rt).unwrap();
            check(&mut a, base, "d", &rt);
            let (b, l) = a.recover_matching(&path, &rt).unwrap();
            assert_eq!(value(&b), "40");
            assert!(!a.is_clean(&b, &path, &rt).unwrap());
            drop((b, l));
        }
        "move" => {
            if root.ends_with("moved-history") {
                a.reconcile_relocation().unwrap();
            } else {
                assert!(a.restore(&rt).is_err());
                a.cancel_relocation().unwrap();
            }
            check(&mut a, base, "d", &rt);
            a.verify_attachment(&path, &rt).unwrap();
        }
        "save" | "native" => {
            let id = Uuid::parse_str(&fs::read_to_string(base.join("ticket")).unwrap()).unwrap();
            if value(&ordinary) == "20" {
                a.reconcile_install(id, &rt, Hook::default()).unwrap();
                a.reconcile_install(id, &rt, Hook::default()).unwrap();
                check(&mut a, base, "a", &rt);
                let (b, l) = a.restore(&rt).unwrap();
                assert!(!a.is_clean(&b, &path, &rt).unwrap());
                drop((b, l));
            } else {
                assert_eq!(value(&ordinary), "10");
                assert!(a.reconcile_install(id, &rt, Hook::default()).is_err());
                assert!(a.clear(&rt, Hook::default()).is_err());
                a.abort_uninstalled(id).unwrap();
                check(&mut a, base, "a", &rt);
                a.redo().unwrap();
                check(&mut a, base, "b", &rt);
            }
        }
        "clear" => {
            check(&mut a, base, "a", &rt);
            if at == "AfterControlDecision" {
                assert!(a.redo().is_err());
            } else {
                a.redo().unwrap();
                check(&mut a, base, "b", &rt);
                a.undo().unwrap();
            }
            let id = Uuid::parse_str(&fs::read_to_string(base.join("ticket")).unwrap()).unwrap();
            let t = a.resume_ticket(id).unwrap();
            a.finish(&t, &rt, Hook::default(), |_| Ok(())).unwrap();
            drop(t);
            check(&mut a, base, "a", &rt);
        }
        _ => panic!("probe"),
    }
    drop((a, ordinary));
    finish(rt);
}
fn exercise(base: &Path, case: &str) {
    seed(base, matches!(case, "late-save" | "late-clear"));
    let rt = IntegrationRuntime::standalone().unwrap();
    let path = base.join("book.pivot");
    let mut a = Association::open(&base.join("history"), &rt).unwrap();
    match case {
        "matching" => {
            a.verify_attachment(&path, &rt).unwrap();
            assert!(Association::open(&base.join("history"), &rt).is_err());
            check(&mut a, base, "d", &rt);
        }
        "late-save" | "late-clear" => {
            a.undo().unwrap();
            let id = Uuid::parse_str(&fs::read_to_string(base.join("ticket")).unwrap()).unwrap();
            let t = a.resume_ticket(id).unwrap();
            a.undo().unwrap();
            if case == "late-clear" {
                let cp = a.saved().unwrap().unwrap().checkpoint;
                a.clear(&rt, Hook::default()).unwrap();
                assert_eq!(a.saved().unwrap().unwrap().checkpoint, cp);
                assert!(a.redo().is_err());
            }
            a.finish(&t, &rt, Hook::default(), |_| Ok(())).unwrap();
            drop(t);
            check(&mut a, base, "a", &rt);
            let (b, l) = a.restore(&rt).unwrap();
            assert!(!a.is_clean(&b, &path, &rt).unwrap());
            drop((b, l));
            let (b, cp) = open_saved(&path, &rt).unwrap();
            assert_eq!(value(&b), "20");
            assert_eq!(a.saved().unwrap().unwrap().checkpoint, cp);
            drop(b);
        }
        "cancel" => {
            let (b, l) = a.restore(&rt).unwrap();
            let t = a.prepare(&b, &path, Hook::default()).unwrap();
            assert!(a.close(false, true, Hook::default()).is_err());
            a.close(true, false, Hook::default()).unwrap();
            a.cancel(t.id()).unwrap();
            assert!(a.finish(&t, &rt, Hook::default(), |_| Ok(())).is_err());
            drop((t, b, l));
            assert_eq!(
                native::read(&path).unwrap().0,
                fs::read(base.join("saved-a.wb5")).unwrap()
            );
            check(&mut a, base, "d", &rt);
        }
        "retained" => {
            a.close(false, false, Hook::default()).unwrap();
            drop(a);
            a = Association::open(&base.join("history"), &rt).unwrap();
            assert!(a.restore(&rt).is_err());
            a.reopen_retained().unwrap();
            check(&mut a, base, "d", &rt);
        }
        "session-only" => {
            a.set_policy(Policy::SessionOnly).unwrap();
            assert!(a.close(false, false, Hook::default()).is_err());
            a.close(true, false, Hook::default()).unwrap();
            check(&mut a, base, "d", &rt);
            a.close(false, true, Hook::default()).unwrap();
            drop(a);
            assert!(Association::open(&base.join("history"), &rt).is_err());
            let (b, _) = open_saved(&path, &rt).unwrap();
            assert_eq!(value(&b), "10");
            drop(b);
            finish(rt);
            return;
        }
        "rename" => {
            let moved = base.join("renamed.pivot");
            fs::rename(&path, &moved).unwrap();
            a.verify_attachment(&moved, &rt).unwrap();
            let (b, l) = a.recover_matching(&moved, &rt).unwrap();
            assert_eq!(value(&b), "40");
            drop((b, l));
        }
        "copy" => {
            let copied = base.join("copy.pivot");
            fs::copy(&path, &copied).unwrap();
            let (mut b, _) = open_saved(&copied, &rt).unwrap();
            assert!(a.verify_attachment(&copied, &rt).is_err());
            let mut l = ColdLedger::new(&b).unwrap();
            let mut fork =
                Association::create(&base.join("independent"), &b, &l, Policy::Retained, &rt)
                    .unwrap();
            edit(&mut b, 1, 1, "99");
            l.observe(&b).unwrap();
            let c = Candidate::capture(&b, &l).unwrap();
            fork.edit(&c, &rt).unwrap();
            let t = fork.prepare(&b, &copied, Hook::default()).unwrap();
            fork.finish(&t, &rt, Hook::default(), |_| Ok(())).unwrap();
            drop((t, c, fork, b, l));
            a.verify_attachment(&path, &rt).unwrap();
            check(&mut a, base, "d", &rt);
        }
        "replace" => {
            let copied = base.join("replacement.pivot");
            fs::copy(&path, &copied).unwrap();
            fs::rename(&copied, &path).unwrap();
            assert!(a.verify_attachment(&path, &rt).is_err());
            let (b, _) = open_saved(&path, &rt).unwrap();
            assert_eq!(value(&b), "10");
            drop(b);
        }
        "save-as" => {
            let other = base.join("new.pivot");
            let (b, l) = a.restore(&rt).unwrap();
            let t = a.prepare(&b, &other, Hook::default()).unwrap();
            a.finish(&t, &rt, Hook::default(), |_| Ok(())).unwrap();
            assert!(a.is_clean(&b, &other, &rt).unwrap());
            assert!(a.verify_attachment(&path, &rt).is_err());
            let (old, _) = open_saved(&path, &rt).unwrap();
            assert_eq!(value(&old), "10");
            drop((t, b, l, old));
        }
        "relocate" => {
            a = a
                .relocate(&base.join("moved-history"), &rt, Hook::default())
                .unwrap();
            a.verify_attachment(&path, &rt).unwrap();
            check(&mut a, base, "d", &rt);
        }
        "unretained" => {
            let (mut b, mut l) = a.break_retention(&rt, Hook::default()).unwrap();
            assert!(a.undo().is_err());
            edit(&mut b, 5, 1, "explicitly unretained");
            l.observe(&b).unwrap();
            assert!(
                b.history_metrics().unwrap().next_node
                    > fs::read_to_string(base.join("floor"))
                        .unwrap()
                        .parse::<u64>()
                        .unwrap()
            );
            drop((b, l, a));
            assert!(Association::open(&base.join("history"), &rt).is_err());
            let (b, _) = open_saved(&path, &rt).unwrap();
            assert_eq!(value(&b), "10");
            drop(b);
            finish(rt);
            return;
        }
        "historical-native-save" => {
            a.undo().unwrap();
            let (b, l) = a.restore(&rt).unwrap();
            let before = oracle(&b);
            let expected = b.expected();
            let ledger = format!("{:?}", b.history_metrics().unwrap());
            let capture = b.capture_local(expected).unwrap();
            // The original full-ledger contract continues to reject historical holes.
            assert!(b.export_portable(expected).is_err());
            let t = a.prepare(&b, &path, Hook::default()).unwrap();
            assert_eq!(b.expected(), expected);
            assert_eq!(oracle(&b), before);
            assert_eq!(format!("{:?}", b.history_metrics().unwrap()), ledger);
            a.finish(&t, &rt, Hook::default(), |_| Ok(())).unwrap();
            assert!(a.is_clean(&b, &path, &rt).unwrap());
            let (opened, checkpoint) = open_saved(&path, &rt).unwrap();
            assert_eq!(checkpoint, t.content_checkpoint());
            for r in 1..=4 {
                for c in 1..=7 {
                    assert_eq!(
                        opened.cell_at(p(r, c)).unwrap(),
                        b.cell_at(p(r, c)).unwrap()
                    );
                }
            }
            // Independent Open remaps runtime traversal order. With both a cycle
            // and a parse draft, the core first-failure choice is not an ordering
            // contract. Exact stored content/provenance is compared above; isolated
            // cycles have a complete diagnostic comparison in history_projection.
            let diagnostics = |book: &Workbook| {
                book.validate(book.expected())
                    .unwrap()
                    .into_iter()
                    .filter(|d| !matches!(d, Diagnostic::Unorderable { .. }))
                    .collect::<Vec<_>>()
            };
            assert_eq!(diagnostics(&opened), diagnostics(&b));
            for original in [&b, &opened] {
                let mut probe = original
                    .stage_history_candidate(original.expected())
                    .unwrap();
                probe
                    .apply(probe.expected(), &[Edit::Clear(p(1, 4))])
                    .unwrap();
                let cyclic = probe.cell_at(p(1, 5)).unwrap().unwrap().reference;
                assert_eq!(
                    probe.validate(probe.expected()).unwrap(),
                    vec![Diagnostic::Unorderable {
                        cells: vec![cyclic]
                    }]
                );
            }
            assert_eq!(value(&opened), value(&b));
            let cell = capture.cell_at(p(1, 1)).unwrap();
            assert_eq!(
                capture.evaluate(&[cell]).unwrap().values[0].1,
                Ok(20.into())
            );
            drop((t, capture, opened, b, l));
            a.redo().unwrap();
            check(&mut a, base, "d", &rt);
            let (d, dl) = a.restore(&rt).unwrap();
            assert!(!a.is_clean(&d, &path, &rt).unwrap());
            drop((d, dl));
            a.undo().unwrap();
            let (b, bl) = a.restore(&rt).unwrap();
            assert!(a.is_clean(&b, &path, &rt).unwrap());
            drop((b, bl));
            println!("HISTORICAL_NATIVE_SAVE_OPEN_REDO_AND_CLEANLINESS_PASSED");
        }
        "unavailable" => {
            let old = base.join("history");
            let unavailable = base.join("unavailable-history");
            match fs::rename(&old, &unavailable) {
                Ok(()) => {
                    assert!(a.undo().is_err());
                    assert!(a.clear(&rt, Hook::default()).is_err());
                    assert!(a.set_policy(Policy::SessionOnly).is_err());
                    assert!(a.break_retention(&rt, Hook::default()).is_err());
                    assert!(a.close(false, true, Hook::default()).is_err());
                }
                Err(e) => {
                    assert_eq!(e.kind(), std::io::ErrorKind::PermissionDenied);
                    println!("OWNER_LOCK_PREVENTED_LIVE_NAMESPACE_MOVE");
                    check(&mut a, base, "d", &rt);
                }
            }
            drop(a);
            if old.exists() {
                fs::rename(&old, &unavailable).unwrap();
            }
            assert!(Association::open(&old, &rt).is_err());
            let (b, _) = open_saved(&path, &rt).unwrap();
            assert_eq!(value(&b), "10");
            drop(b);
            fs::rename(&unavailable, &old).unwrap();
            a = Association::open(&old, &rt).unwrap();
            check(&mut a, base, "d", &rt);
        }
        _ => panic!("exercise"),
    }
    drop(a);
    finish(rt);
}
fn main() {
    let a: Vec<_> = std::env::args().collect();
    let b = Path::new(&a[2]);
    match a[1].as_str() {
        "seed" => seed(b, a.get(3).is_some_and(|s| s == "prepared")),
        "operate" => operate(b, &a[3], &a[4], &a[5]),
        "probe" => probe(b, &a[3], &a[4]),
        "exercise" => exercise(b, &a[3]),
        _ => panic!("command"),
    }
}
