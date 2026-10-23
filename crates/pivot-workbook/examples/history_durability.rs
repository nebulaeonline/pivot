//! Fresh-process Stage 3 fixture, fault worker, recovery probe and timing harness.
use pivot_core::model::integration::IntegrationRuntime;
use pivot_workbook::{
    history_experiment::{ColdLedger, durability::*},
    *,
};
use std::{
    fs,
    path::{Path, PathBuf},
    time::Instant,
};
use uuid::Uuid;
fn p(r: u32, c: u32) -> Position {
    Position::new(r, c)
}
fn edit(r: u32, c: u32, s: &str) -> Edit {
    Edit::SetEntry {
        position: p(r, c),
        source: s.into(),
    }
}
fn oracle(b: &Workbook) -> String {
    format!(
        "{:?}|{:?}|{:?}|{:?}",
        (1..=3)
            .flat_map(|r| (1..=8).map(move |c| p(r, c)))
            .map(|p| b.cell_at(p).unwrap())
            .collect::<Vec<_>>(),
        b.validate(b.expected()).unwrap(),
        b.invalidated_cells(),
        b.history_content_id()
    )
}
fn config(s: &str) -> Config {
    match s {
        "full" => Config {
            full: true,
            ..Config::default()
        },
        "delta16" => Config::default(),
        "delta64" => Config {
            interval: 64,
            delta_bytes: 16 * 1024 * 1024 * 64,
            delta_records: 65536 * 64,
            full: false,
        },
        "adaptive64" => Config {
            interval: 64,
            ..Config::default()
        },
        _ => panic!("bad configuration"),
    }
}
fn finish(rt: IntegrationRuntime) {
    rt.flush();
    let memory = rt.memory();
    assert!(rt.finish().is_ok());
    assert_eq!(memory.stats().charged, 0);
    println!("FINISH 0");
}
fn fixture(rt: &IntegrationRuntime) -> Workbook {
    let mut b = Workbook::new_in("durability fixture", Limits::default(), rt).unwrap();
    b.apply(
        b.expected(),
        &[
            edit(1, 1, "10.00"),
            edit(2, 1, "10.00"),
            edit(1, 2, "=A1"),
            edit(1, 3, "=SUM($A$2:A1)"),
            edit(1, 4, "=("),
            edit(1, 5, "=E1"),
            edit(1, 6, "Unicode λ"),
            Edit::SetDecimal {
                position: p(1, 7),
                value: DecimalInput::from_parts(0, 0, 0, true, 5).unwrap(),
            },
        ],
    )
    .unwrap();
    b
}
fn evidence(path: &Path) -> PathBuf {
    path.with_extension("oracles")
}
fn save(root: &Path, name: &str, book: &Workbook, candidate: &Candidate) {
    fs::write(root.join(format!("{name}.capsule")), candidate.bytes()).unwrap();
    fs::write(root.join(format!("{name}.oracle")), oracle(book)).unwrap();
}
fn check(j: &mut Journal, rt: &IntegrationRuntime, root: &Path, name: &str) {
    let bytes = j.current_bytes().unwrap();
    assert_eq!(
        bytes.bytes,
        fs::read(root.join(format!("{name}.capsule"))).unwrap()
    );
    let (book, ledger) = j.restore(rt).unwrap();
    assert_eq!(
        oracle(&book),
        fs::read_to_string(root.join(format!("{name}.oracle"))).unwrap()
    );
    assert_eq!(book.history_content_id(), j.receipt().unwrap().content);
    drop((book, ledger));
}
fn seed(path: &Path, mode: &str, at_b: bool) {
    let rt = IntegrationRuntime::standalone().unwrap();
    let root = evidence(path);
    fs::create_dir(&root).unwrap();
    let mut j = Journal::create(path, config(mode), &rt).unwrap();
    let mut b = fixture(&rt);
    let mut ledger = ColdLedger::new(&b).unwrap();
    let a = Candidate::capture(&b, &ledger).unwrap();
    save(&root, "a", &b, &a);
    j.commit(Uuid::new_v4(), &a, &rt, &mut Fault::default())
        .unwrap();
    drop(a);
    b.apply(
        b.expected(),
        &[
            edit(1, 2, "=A2"),
            edit(3, 1, "20"),
            edit(3, 2, "=SUM(A1:A3)"),
        ],
    )
    .unwrap();
    ledger.observe(&b).unwrap();
    let candidate = Candidate::capture(&b, &ledger).unwrap();
    save(&root, "b", &b, &candidate);
    j.commit(Uuid::new_v4(), &candidate, &rt, &mut Fault::default())
        .unwrap();
    drop(candidate);
    fs::write(
        root.join("b-only-id"),
        b.cell_at(p(3, 1))
            .unwrap()
            .unwrap()
            .reference
            .cell
            .uuid()
            .to_string(),
    )
    .unwrap();
    if !at_b {
        j.undo(Uuid::new_v4(), &mut Fault::default()).unwrap();
    }
    fs::write(
        root.join("baseline.tx"),
        j.receipt().unwrap().transaction.to_string(),
    )
    .unwrap();
    fs::write(
        root.join("baseline.receipt"),
        format!("{:?}", j.receipt().unwrap()),
    )
    .unwrap();
    fs::write(root.join("baseline.name"), if at_b { "b" } else { "a" }).unwrap();
    let (book, l) = j.restore(&rt).unwrap();
    fs::write(
        root.join("baseline.floor"),
        book.history_metrics().unwrap().next_node.to_string(),
    )
    .unwrap();
    check(&mut j, &rt, &root, if at_b { "b" } else { "a" });
    drop((book, l, ledger, b, j));
    finish(rt);
}
fn attempt(path: &Path, id: Uuid, operation: &str, point: &str, action: &str) {
    let rt = IntegrationRuntime::standalone().unwrap();
    let root = evidence(path);
    let mut j = Journal::open(path, &rt).unwrap();
    let (live, mut ledger) = j.restore(&rt).unwrap();
    let old = oracle(&live);
    let previous = j.receipt().unwrap();
    let mut candidate_book = live.stage_history_candidate(live.expected()).unwrap();
    let candidate = if operation == "edit" {
        candidate_book
            .apply(
                candidate_book.expected(),
                &[
                    edit(1, 2, "=A1"),
                    edit(3, 1, "7"),
                    edit(3, 2, "=COUNT(A1:A3)"),
                ],
            )
            .unwrap();
        ledger.observe(&candidate_book).unwrap();
        assert_ne!(
            candidate_book
                .cell_at(p(3, 1))
                .unwrap()
                .unwrap()
                .reference
                .cell
                .uuid()
                .to_string(),
            fs::read_to_string(root.join("b-only-id")).unwrap()
        );
        let c = Candidate::capture(&candidate_book, &ledger).unwrap();
        save(&root, "attempt", &candidate_book, &c);
        fs::write(
            root.join("attempt.floor"),
            candidate_book
                .history_metrics()
                .unwrap()
                .next_node
                .to_string(),
        )
        .unwrap();
        Some(c)
    } else {
        None
    };
    let target = match operation {
        "edit" => "attempt",
        "undo" => "a",
        "redo" => "b",
        _ => panic!("bad operation"),
    };
    fs::write(root.join("attempt.target"), target).unwrap();
    let mut fault = if action == "budget" {
        Fault {
            write_budget: Some(point.parse().unwrap()),
            ..Fault::default()
        }
    } else {
        Fault {
            at: Some((
                Point::parse(point).unwrap(),
                if action == "pause" {
                    Action::Pause
                } else {
                    Action::Error
                },
            )),
            ..Fault::default()
        }
    };
    let at = Instant::now();
    let result = match operation {
        "edit" => j.commit(id, candidate.as_ref().unwrap(), &rt, &mut fault),
        "undo" => j.undo(id, &mut fault),
        "redo" => j.redo(id, &mut fault),
        _ => unreachable!(),
    };
    assert_eq!(oracle(&live), old); // Publication cannot precede persisted success/reconciliation.
    match result {
        Ok(r) => {
            println!(
                "ACK {} {} {}",
                r.transaction,
                r.content,
                at.elapsed().as_nanos()
            );
            check(&mut j, &rt, &root, target);
        }
        Err(_) => {
            if j.pending().is_some() {
                assert!(j.undo(Uuid::new_v4(), &mut Fault::default()).is_err());
                assert!(j.redo(Uuid::new_v4(), &mut Fault::default()).is_err());
                if let Some(c) = candidate.as_ref() {
                    assert!(
                        j.commit(Uuid::new_v4(), c, &rt, &mut Fault::default())
                            .is_err()
                    );
                }
                assert!(j.reconcile(Uuid::new_v4(), &rt).is_err());
                match j.reconcile(id, &rt).unwrap() {
                    Outcome::Committed(r) => {
                        assert_eq!(r.transaction, id);
                        check(&mut j, &rt, &root, target);
                        println!("RECONCILED committed {id}");
                    }
                    Outcome::Uncommitted(t) => {
                        assert_eq!(t, id);
                        assert_eq!(
                            format!("{:?}", j.receipt().unwrap()),
                            format!("{previous:?}")
                        );
                        check(
                            &mut j,
                            &rt,
                            &root,
                            &fs::read_to_string(root.join("baseline.name")).unwrap(),
                        );
                        println!("RECONCILED uncommitted {id}");
                    }
                }
                assert_eq!(j.pending(), None);
            } else {
                assert!(matches!(j.lookup(id).unwrap(), Outcome::Uncommitted(_)));
                println!("DEFINITELY_UNCOMMITTED {id}");
            }
            // Same operation ID must never create another committed transition.
            if matches!(j.lookup(id).unwrap(), Outcome::Committed(_)) {
                assert!(j.undo(id, &mut Fault::default()).is_err());
            }
        }
    }
    println!("METRICS {:?}", j.metrics);
    drop((candidate, candidate_book, live, ledger, j));
    finish(rt);
}
fn probe(path: &Path, id: Uuid, expect: &str) {
    let rt = IntegrationRuntime::standalone().unwrap();
    let root = evidence(path);
    let opened = Journal::open(path, &rt);
    if expect == "reject" {
        assert!(opened.is_err());
        println!("REJECTED");
        drop(opened);
        finish(rt);
        return;
    }
    let mut j = opened.unwrap();
    let outcome = j.lookup(id).unwrap();
    let committed = matches!(outcome, Outcome::Committed(_));
    assert_eq!(committed, expect == "committed");
    if committed {
        assert_eq!(j.receipt().unwrap().transaction, id);
        check(
            &mut j,
            &rt,
            &root,
            &fs::read_to_string(root.join("attempt.target")).unwrap(),
        );
    } else {
        assert_eq!(
            format!("{:?}", j.receipt().unwrap()),
            fs::read_to_string(root.join("baseline.receipt")).unwrap()
        );
        check(
            &mut j,
            &rt,
            &root,
            &fs::read_to_string(root.join("baseline.name")).unwrap(),
        );
    }
    let target = fs::read_to_string(root.join("attempt.target")).unwrap();
    let authority = if committed && target == "attempt" {
        "attempt"
    } else {
        "b"
    };
    assert_eq!(
        j.authority_bytes().unwrap().bytes,
        fs::read(root.join(format!("{authority}.capsule"))).unwrap()
    );
    let expected_floor = if authority == "attempt" {
        "attempt.floor"
    } else {
        "baseline.floor"
    };
    let (book, ledger) = j.restore(&rt).unwrap();
    assert_eq!(
        book.history_metrics().unwrap().next_node,
        fs::read_to_string(root.join(expected_floor))
            .unwrap()
            .parse::<u64>()
            .unwrap()
    );
    // The discarded B allocation remains reserved in a fresh process after Undo/branch.
    if book.cell_at(p(3, 1)).unwrap().is_none() {
        let mut staged = book.stage_history_candidate(book.expected()).unwrap();
        staged.apply(staged.expected(), &[edit(3, 1, "9")]).unwrap();
        assert_ne!(
            staged
                .cell_at(p(3, 1))
                .unwrap()
                .unwrap()
                .reference
                .cell
                .uuid()
                .to_string(),
            fs::read_to_string(root.join("b-only-id")).unwrap()
        );
        drop(staged);
    }
    println!("RECOVERED {outcome:?}");
    drop((book, ledger, j));
    finish(rt);
}
fn series(path: &Path, mode: &str, count: usize, cells: u32) {
    let rt = IntegrationRuntime::standalone().unwrap();
    let memory = rt.memory();
    let mut j = Journal::create(path, config(mode), &rt).unwrap();
    let mut book = fixture(&rt);
    let additional = cells.saturating_sub(8);
    for base in (0..additional).step_by(256) {
        let edits = (base..(base + 256).min(additional))
            .map(|i| edit(i / 32 + 5, i % 32 + 1, "1"))
            .collect::<Vec<_>>();
        book.apply(book.expected(), &edits).unwrap();
    }
    let mut ledger = ColdLedger::new(&book).unwrap();
    let first = Candidate::capture(&book, &ledger).unwrap();
    j.commit(Uuid::new_v4(), &first, &rt, &mut Fault::default())
        .unwrap();
    drop(first);
    let mut export = 0;
    let mut total = 0;
    let mut max = 0;
    let mut last_id = Uuid::nil();
    for i in 0..count {
        let at = Instant::now();
        let mut staged = book.stage_history_candidate(book.expected()).unwrap();
        staged
            .apply(staged.expected(), &[edit(1, 1, &(i + 100).to_string())])
            .unwrap();
        ledger.observe(&staged).unwrap();
        let c = Candidate::capture(&staged, &ledger).unwrap();
        export += c.export_ns;
        last_id = Uuid::new_v4();
        j.commit(last_id, &c, &rt, &mut Fault::default()).unwrap();
        book = staged;
        let elapsed = at.elapsed().as_nanos();
        total += elapsed;
        max = max.max(elapsed);
    }
    let expected = oracle(&book);
    let expected_bytes = j.current_bytes().unwrap();
    let metrics = format!("{:?}", j.metrics);
    drop(j);
    let mut j = Journal::open(path, &rt).unwrap();
    assert!(matches!(j.lookup(last_id).unwrap(), Outcome::Committed(_)));
    let (restored, l) = j.restore(&rt).unwrap();
    assert_eq!(oracle(&restored), expected);
    assert_eq!(j.current_bytes().unwrap().bytes, expected_bytes.bytes);
    println!(
        "SERIES mode={mode} count={count} cells={cells} export_ns={export} acknowledgment_total_ns={total} acknowledgment_max_ns={max} idle_charge={} peak_charge={} bytes={} {metrics}",
        memory.stats().charged,
        memory.stats().peak,
        fs::metadata(path).unwrap().len()
    );
    drop((expected_bytes, book, ledger, restored, l, j));
    finish(rt);
}
fn main() {
    let args: Vec<_> = std::env::args().collect();
    let path = Path::new(&args[2]);
    match args[1].as_str() {
        "seed" => seed(path, &args[3], args[4] == "b"),
        "attempt" => attempt(
            path,
            Uuid::parse_str(&args[3]).unwrap(),
            &args[4],
            &args[5],
            &args[6],
        ),
        "probe" => probe(path, Uuid::parse_str(&args[3]).unwrap(), &args[4]),
        "series" => series(
            path,
            &args[3],
            args[4].parse().unwrap(),
            args[5].parse().unwrap(),
        ),
        _ => panic!("bad command"),
    }
}
