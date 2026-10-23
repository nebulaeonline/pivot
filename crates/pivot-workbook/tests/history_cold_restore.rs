#![cfg(feature = "history-experiment")]
use pivot_core::model::integration::{CancellationToken, IntegrationRuntime};
use pivot_workbook::{history_experiment::ColdLedger, *};
use std::path::{Path, PathBuf};
fn p(r: u32, c: u32) -> Position {
    Position::new(r, c)
}
fn e(r: u32, c: u32, s: &str) -> Edit {
    Edit::SetEntry {
        position: p(r, c),
        source: s.into(),
    }
}
fn finish(rt: IntegrationRuntime) {
    rt.flush();
    let m = rt.memory();
    assert!(rt.finish().is_ok());
    assert_eq!(m.stats().charged, 0);
    println!(
        "cold finish {:?}: allocated={} charged={} peak_charged={}",
        std::thread::current().name(),
        m.stats().allocated,
        m.stats().charged,
        m.stats().peak
    );
}
fn fixture(rt: &IntegrationRuntime) -> Workbook {
    let mut b = Workbook::new_in("cold fixture", Limits::default(), rt).unwrap();
    b.apply(
        b.expected(),
        &[
            e(1, 1, "10.00"),
            e(1, 2, "=A1+1"),
            e(1, 3, "=SUM(A1:A2)"),
            e(1, 4, "=("),
            e(1, 5, "=E1"),
            e(1, 6, "Unicode λ"),
            e(1, 7, "=H1"),
            e(1, 10, "3"),
            e(1, 9, "=J1+1"),
            e(1, 11, "=MAX($A$2:A1)"),
            Edit::PreserveRejectedFormula {
                position: p(1, 12),
                source: "=1+2".into(),
            },
            Edit::SetDecimal {
                position: p(1, 13),
                value: DecimalInput::from_parts(0, 0, 0, true, 5).unwrap(),
            },
        ],
    )
    .unwrap();
    b.apply(
        b.expected(),
        &[
            Edit::Move {
                from: p(1, 1),
                to: p(2, 1),
            },
            Edit::Retire(p(1, 10)),
        ],
    )
    .unwrap();
    b
}
fn add_b(b: &mut Workbook) {
    b.apply(b.expected(), &[e(3, 1, "20"), e(3, 2, "=SUM(A2:A3)")])
        .unwrap();
}
fn add_c(b: &mut Workbook) {
    b.apply(b.expected(), &[e(3, 1, "7"), e(3, 2, "=COUNT(A2:A3)")])
        .unwrap();
}
fn oracle(b: &Workbook) -> String {
    format!(
        "{:?} diagnostics={:?} invalidated={:?} content={:?}",
        [
            p(2, 1),
            p(1, 2),
            p(1, 3),
            p(1, 4),
            p(1, 5),
            p(1, 6),
            p(1, 7),
            p(1, 8),
            p(1, 9),
            p(1, 10),
            p(1, 11),
            p(1, 12),
            p(1, 13),
            p(3, 1),
            p(3, 2)
        ]
        .into_iter()
        .map(|x| b.cell_at(x).unwrap())
        .collect::<Vec<_>>(),
        b.validate(b.expected()).unwrap(),
        b.invalidated_cells(),
        b.history_content_id()
    )
}
#[test]
fn cold_eviction_preserves_exact_live_runtime_ids_and_branch_capture() {
    let rt = IntegrationRuntime::standalone().unwrap();
    let mut b = fixture(&rt);
    let mut ledger = ColdLedger::new(&b).unwrap();
    let expected_a = b.expected();
    let content_a = b.history_content_id();
    let oracle_a = oracle(&b);
    let runtime_a = b.history_runtime_node(p(1, 2)).unwrap();
    let a = b.history_cold_export(b.expected(), &ledger).unwrap();
    add_b(&mut b);
    ledger.observe(&b).unwrap();
    let expected_b = b.expected();
    let content_b = b.history_content_id();
    let runtime_b = b.history_runtime_node(p(3, 2)).unwrap();
    let b_bytes = b.history_cold_export(b.expected(), &ledger).unwrap();
    let oracle_b = oracle(&b);
    let capture_b = b.capture_local(b.expected()).unwrap();
    let output = capture_b.cell_at(p(3, 2)).unwrap();
    assert_eq!(
        capture_b.evaluate(&[output]).unwrap().values[0].1,
        Ok(30.into())
    );
    // No pin_history_root or resident A wrapper is retained: only encoded bytes and an oracle.
    b.history_cold_activate(
        b.expected(),
        &a.bytes,
        &ledger,
        &CancellationToken::default(),
    )
    .unwrap();
    assert_eq!(oracle(&b), oracle_a);
    assert_eq!(b.history_content_id(), content_a);
    assert_eq!(b.history_runtime_node(p(1, 2)).unwrap(), runtime_a);
    assert_ne!(b.expected(), expected_a);
    assert!(b.apply(expected_a, &[e(9, 9, "1")]).is_err());
    add_c(&mut b);
    ledger.observe(&b).unwrap();
    assert!(b.history_runtime_node(p(3, 2)).unwrap() > runtime_b);
    assert_eq!(
        capture_b.evaluate(&[output]).unwrap().values[0].1,
        Ok(30.into())
    );
    let c = b.history_cold_export(b.expected(), &ledger).unwrap();
    println!(
        "research checkpoint bytes A={} B={} C={}; ledger_floor={}",
        a.bytes.len(),
        b_bytes.bytes.len(),
        c.bytes.len(),
        ledger.allocated_floor()
    );
    let oracle_c = oracle(&b);
    let runtime_c = b.history_runtime_node(p(3, 2)).unwrap();
    b.history_cold_activate(
        b.expected(),
        &b_bytes.bytes,
        &ledger,
        &CancellationToken::default(),
    )
    .unwrap();
    assert_eq!(oracle(&b), oracle_b);
    assert_eq!(b.history_content_id(), content_b);
    assert_eq!(b.history_runtime_node(p(3, 2)).unwrap(), runtime_b);
    assert!(b.apply(expected_b, &[e(9, 9, "1")]).is_err());
    b.history_cold_activate(
        b.expected(),
        &c.bytes,
        &ledger,
        &CancellationToken::default(),
    )
    .unwrap();
    assert_eq!(oracle(&b), oracle_c);
    assert_eq!(b.history_runtime_node(p(3, 2)).unwrap(), runtime_c);
    assert_eq!(
        capture_b.evaluate(&[output]).unwrap().values[0].1,
        Ok(30.into())
    );
    assert_eq!(
        b.history_metrics().unwrap().next_node,
        ledger.allocated_floor()
    );
    drop((a, b_bytes, c, ledger, capture_b, b));
    finish(rt);
}
#[test]
fn independent_copies_share_logical_ids_but_not_runtime_or_authority() {
    let rt = IntegrationRuntime::standalone().unwrap();
    // Also exercise a wholly valid graph, independent of the invalid/draft fixtures.
    let mut source = Workbook::new_in("valid independent copies", Limits::default(), &rt).unwrap();
    source
        .apply(source.expected(), &[e(2, 1, "10"), e(1, 2, "=A2+1")])
        .unwrap();
    let ledger = ColdLedger::new(&source).unwrap();
    let bytes = source
        .history_cold_export(source.expected(), &ledger)
        .unwrap();
    let (mut left, mut left_ledger) =
        Workbook::history_cold_reopen(&bytes.bytes, &bytes.bytes, Limits::default(), &rt).unwrap();
    let (mut right, right_ledger) =
        Workbook::history_cold_reopen(&bytes.bytes, &bytes.bytes, Limits::default(), &rt).unwrap();
    assert_eq!(oracle(&left), oracle(&right));
    assert_eq!(left.history_content_id(), right.history_content_id());
    assert_ne!(
        left.history_runtime_node(p(1, 2)).unwrap(),
        right.history_runtime_node(p(1, 2)).unwrap()
    );
    assert_ne!(left.expected().session, right.expected().session);
    assert!(left.apply(right.expected(), &[e(9, 9, "1")]).is_err());
    assert!(
        right
            .history_cold_activate(
                right.expected(),
                &bytes.bytes,
                &left_ledger,
                &CancellationToken::default()
            )
            .is_err()
    );
    let capture = right.capture_local(right.expected()).unwrap();
    let output = capture.cell_at(p(1, 2)).unwrap();
    left.apply(left.expected(), &[e(2, 1, "99")]).unwrap();
    left_ledger.observe(&left).unwrap();
    assert_eq!(
        capture.evaluate(&[output]).unwrap().values[0].1,
        Ok(11.into())
    );
    assert_eq!(
        right.cell_at(p(2, 1)).unwrap(),
        source.cell_at(p(2, 1)).unwrap()
    );
    drop((
        capture,
        left,
        right,
        left_ledger,
        right_ledger,
        ledger,
        bytes,
        source,
    ));
    finish(rt);
}
#[test]
fn cold_ledger_rejects_missing_branch_reservations_and_stale_authority() {
    let rt = IntegrationRuntime::standalone().unwrap();
    let mut b = fixture(&rt);
    let mut ledger = ColdLedger::new(&b).unwrap();
    let initial_floor = ledger.allocated_floor();
    let hot_a = b.pin_history_root(b.expected()).unwrap();
    let a = b.history_cold_export(b.expected(), &ledger).unwrap();
    add_b(&mut b);
    let expected = b.expected();
    let before = oracle(&b);
    assert!(
        b.history_cold_activate(expected, &a.bytes, &ledger, &CancellationToken::default())
            .is_err()
    );
    assert_eq!(b.expected(), expected);
    assert_eq!(oracle(&b), before);
    let hot_b = b.pin_history_root(b.expected()).unwrap();
    b.activate_history_root(b.expected(), &hot_a, &CancellationToken::default())
        .unwrap();
    assert!(ColdLedger::new(&b).is_err());
    assert!(ledger.observe(&b).is_err());
    assert_eq!(ledger.allocated_floor(), initial_floor);
    b.activate_history_root(b.expected(), &hot_b, &CancellationToken::default())
        .unwrap();
    ledger.observe(&b).unwrap();
    b.history_cold_activate(
        b.expected(),
        &a.bytes,
        &ledger,
        &CancellationToken::default(),
    )
    .unwrap();
    drop((hot_a, hot_b, a, ledger, b));
    finish(rt);
}
#[test]
fn malformed_cancelled_and_unadmitted_loads_preserve_live_state() {
    let rt = IntegrationRuntime::standalone().unwrap();
    let mut b = fixture(&rt);
    let ledger = ColdLedger::new(&b).unwrap();
    let bytes = b.history_cold_export(b.expected(), &ledger).unwrap();
    let expected = b.expected();
    let content = b.history_content_id();
    let before = oracle(&b);
    for cut in [0, 7, bytes.bytes.len() - 1] {
        assert!(
            b.history_cold_activate(
                expected,
                &bytes.bytes[..cut],
                &ledger,
                &CancellationToken::default()
            )
            .is_err()
        );
    }
    let mut broken = bytes.bytes.clone();
    broken[0] ^= 1;
    assert!(
        b.history_cold_activate(expected, &broken, &ledger, &CancellationToken::default())
            .is_err()
    );
    // Recomputed checksums must not bypass structural bounds or partial-root cleanup.
    use sha2::{Digest, Sha256};
    let n = |at: usize| u64::from_le_bytes(bytes.bytes[at..at + 8].try_into().unwrap()) as usize;
    let selection_count_at = 144 + n(136) * 40;
    let core_size_at = selection_count_at + 8 + n(selection_count_at) * 16;
    let records_at = core_size_at + 8 + n(core_size_at);
    rt.flush();
    let baseline_charge = rt.memory().stats().charged;
    for (at, value) in [(120, u64::MAX), (136, 8193), (records_at, 8193)] {
        let mut malformed = bytes.bytes.clone();
        malformed[at..at + 8].copy_from_slice(&value.to_le_bytes());
        let end = malformed.len() - 32;
        let digest = Sha256::digest(&malformed[..end]);
        malformed[end..].copy_from_slice(&digest);
        assert!(
            b.history_cold_activate(expected, &malformed, &ledger, &CancellationToken::default())
                .is_err()
        );
        rt.flush();
        assert_eq!(rt.memory().stats().charged, baseline_charge);
        assert_eq!(b.expected(), expected);
        assert_eq!(b.history_content_id(), content);
    }
    let cancel = CancellationToken::default();
    cancel.cancel();
    assert!(
        b.history_cold_activate(expected, &bytes.bytes, &ledger, &cancel)
            .is_err()
    );
    let memory = rt.memory();
    let pressure = memory
        .reserve(memory.stats().limit - memory.stats().charged)
        .unwrap();
    assert!(
        b.history_cold_activate(
            expected,
            &bytes.bytes,
            &ledger,
            &CancellationToken::default()
        )
        .is_err()
    );
    drop(pressure);
    assert_eq!(b.expected(), expected);
    assert_eq!(b.history_content_id(), content);
    assert_eq!(oracle(&b), before);
    drop((bytes, ledger, b));
    finish(rt);
}
fn directory() -> PathBuf {
    let base = std::env::var_os("PIVOT_FILE_TEST_ROOT")
        .map(PathBuf::from)
        .unwrap_or_else(std::env::temp_dir);
    base.join(format!("pivot-cold-stage1-{}", uuid::Uuid::new_v4()))
}
fn write(path: &Path, name: &str, bytes: &[u8]) {
    std::fs::write(path.join(name), bytes).unwrap();
}
#[test]
fn fresh_process_restores_durable_content_with_one_coherent_runtime_mapping() {
    let dir = directory();
    std::fs::create_dir_all(&dir).unwrap();
    let rt = IntegrationRuntime::standalone().unwrap();
    let mut b = fixture(&rt);
    let mut ledger = ColdLedger::new(&b).unwrap();
    let old = b.expected();
    write(
        &dir,
        "old.txt",
        format!(
            "{} {} {} {}",
            old.session,
            old.state,
            old.graph.provenance().0,
            old.graph.provenance().1
        )
        .as_bytes(),
    );
    let a = b.history_cold_export(b.expected(), &ledger).unwrap();
    write(&dir, "a.bin", &a.bytes);
    write(&dir, "a.oracle", oracle(&b).as_bytes());
    add_b(&mut b);
    ledger.observe(&b).unwrap();
    let b_bytes = b.history_cold_export(b.expected(), &ledger).unwrap();
    write(&dir, "b.bin", &b_bytes.bytes);
    write(&dir, "b.oracle", oracle(&b).as_bytes());
    b.history_cold_activate(
        b.expected(),
        &a.bytes,
        &ledger,
        &CancellationToken::default(),
    )
    .unwrap();
    add_c(&mut b);
    ledger.observe(&b).unwrap();
    let c = b.history_cold_export(b.expected(), &ledger).unwrap();
    write(&dir, "c.bin", &c.bytes);
    write(&dir, "c.oracle", oracle(&b).as_bytes());
    let floor = ledger.allocated_floor();
    write(&dir, "floor.txt", floor.to_string().as_bytes());
    drop((a, b_bytes, c, ledger, b));
    finish(rt);
    // Parent has no surviving model/runtime/capture. Child has only files and its fresh process.
    let result = std::process::Command::new(std::env::current_exe().unwrap())
        .args(["--exact", "cold_restart_child", "--nocapture"])
        .env("PIVOT_COLD_RESTART_DIR", &dir)
        .output()
        .unwrap();
    println!("{}", String::from_utf8_lossy(&result.stdout));
    assert!(
        result.status.success(),
        "{}",
        String::from_utf8_lossy(&result.stderr)
    );
    for name in [
        "old.txt",
        "a.bin",
        "a.oracle",
        "b.bin",
        "b.oracle",
        "c.bin",
        "c.oracle",
        "floor.txt",
    ] {
        std::fs::remove_file(dir.join(name)).unwrap();
    }
    std::fs::remove_dir(dir).unwrap();
}
#[test]
fn cold_restart_child() {
    let Some(dir) = std::env::var_os("PIVOT_COLD_RESTART_DIR") else {
        return;
    };
    let dir = PathBuf::from(dir);
    let rt = IntegrationRuntime::standalone().unwrap();
    let old = std::fs::read_to_string(dir.join("old.txt")).unwrap();
    let fields: Vec<_> = old.split_whitespace().collect();
    let original = Expected {
        session: uuid::Uuid::parse_str(fields[0]).unwrap(),
        state: fields[1].parse().unwrap(),
        graph: pivot_core::model::Revision::from_provenance(
            fields[2].parse().unwrap(),
            fields[3].parse().unwrap(),
        ),
    };
    let mut unrelated = Vec::new();
    for _ in 0..original.graph.provenance().0 + 2 {
        unrelated.push(pivot_core::model::WorkingModel::new_in("already open", &rt).unwrap());
    }
    let a = std::fs::read(dir.join("a.bin")).unwrap();
    let b = std::fs::read(dir.join("b.bin")).unwrap();
    let c = std::fs::read(dir.join("c.bin")).unwrap();
    let (mut book, mut ledger) =
        Workbook::history_cold_reopen(&c, &a, Limits::default(), &rt).unwrap();
    assert!(book.expected().graph.provenance().0 > original.graph.provenance().0);
    assert_ne!(book.expected().session, original.session);
    assert!(book.apply(original, &[e(9, 9, "1")]).is_err());
    let expected_a = std::fs::read_to_string(dir.join("a.oracle")).unwrap();
    let expected_b = std::fs::read_to_string(dir.join("b.oracle")).unwrap();
    let expected_c = std::fs::read_to_string(dir.join("c.oracle")).unwrap();
    assert_eq!(oracle(&book), expected_a);
    let node_a = book.history_runtime_node(p(1, 2)).unwrap();
    book.history_cold_activate(book.expected(), &b, &ledger, &CancellationToken::default())
        .unwrap();
    assert_eq!(oracle(&book), expected_b);
    let node_b = book.history_runtime_node(p(3, 2)).unwrap();
    let capture_b = book.capture_local(book.expected()).unwrap();
    let output = capture_b.cell_at(p(3, 2)).unwrap();
    book.history_cold_activate(book.expected(), &c, &ledger, &CancellationToken::default())
        .unwrap();
    assert_eq!(oracle(&book), expected_c);
    let node_c = book.history_runtime_node(p(3, 2)).unwrap();
    assert_ne!(node_b, node_c);
    book.history_cold_activate(book.expected(), &a, &ledger, &CancellationToken::default())
        .unwrap();
    assert_eq!(book.history_runtime_node(p(1, 2)).unwrap(), node_a);
    book.history_cold_activate(book.expected(), &b, &ledger, &CancellationToken::default())
        .unwrap();
    assert_eq!(book.history_runtime_node(p(3, 2)).unwrap(), node_b);
    assert_eq!(
        capture_b.evaluate(&[output]).unwrap().values[0].1,
        Ok(30.into())
    );
    let floor: u64 = std::fs::read_to_string(dir.join("floor.txt"))
        .unwrap()
        .parse()
        .unwrap();
    assert_eq!(ledger.allocated_floor(), floor);
    assert_eq!(book.history_metrics().unwrap().next_node, floor);
    book.history_cold_activate(book.expected(), &a, &ledger, &CancellationToken::default())
        .unwrap();
    add_c(&mut book);
    ledger.observe(&book).unwrap();
    assert!(book.history_runtime_node(p(3, 2)).unwrap() > node_c);
    assert_eq!(
        capture_b.evaluate(&[output]).unwrap().values[0].1,
        Ok(30.into())
    );
    println!(
        "restart oracle: durable A/B/C identities, provenance, coherent remapping, branch reservations and B capture passed"
    );
    drop((book, ledger, capture_b, unrelated));
    finish(rt);
}
