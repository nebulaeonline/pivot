#![cfg(feature = "history-experiment")]
//! Stage-one boundary probes, not a cold codec or a disk-history implementation.
use pivot_core::model::integration::IntegrationRuntime;
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
fn finish(runtime: IntegrationRuntime) {
    runtime.flush();
    let memory = runtime.memory();
    assert!(runtime.finish().is_ok());
    assert_eq!(memory.stats().charged, 0);
}

#[test]
fn portable_state_preserves_cell_data_but_is_not_a_history_identity_capsule() {
    let runtime = IntegrationRuntime::standalone().unwrap();
    let mut book = Workbook::new_in("inventory", Limits::default(), &runtime).unwrap();
    book.apply(
        book.expected(),
        &[
            entry(1, 1, "10"),
            entry(1, 2, "=A1+1"),
            entry(1, 3, "=SUM(A1:A2)"),
            entry(1, 4, "=("),
            entry(1, 5, "=E1"),
            entry(1, 6, "heading"),
        ],
    )
    .unwrap();
    book.apply(
        book.expected(),
        &[Edit::Move {
            from: p(1, 1),
            to: p(2, 1),
        }],
    )
    .unwrap();
    let before = book.expected();
    let content = book.history_content_id();
    let runtime_node = book.history_runtime_node(p(1, 2)).unwrap();
    let cells: Vec<_> = [p(2, 1), p(1, 2), p(1, 3), p(1, 4), p(1, 5), p(1, 6)]
        .into_iter()
        .map(|position| book.cell_at(position).unwrap().unwrap())
        .collect();
    let image = book.export_portable(before).unwrap();
    let restored = Workbook::restore_portable(image, Limits::default(), &runtime).unwrap();
    for cell in &cells {
        assert_eq!(
            restored.cell_at(cell.position).unwrap().as_ref(),
            Some(cell)
        );
    }
    assert_ne!(restored.expected().session, before.session);
    assert_ne!(restored.history_content_id(), content);
    assert_ne!(
        restored.history_runtime_node(p(1, 2)).unwrap(),
        runtime_node
    );
    assert_ne!(
        restored.expected().graph.provenance().0,
        before.graph.provenance().0
    );
    let capture = restored.capture_local(restored.expected()).unwrap();
    let scalar = capture.cell_at(p(1, 2)).unwrap();
    assert_eq!(
        capture.evaluate(&[scalar]).unwrap().values[0].1,
        Ok(11.into())
    );
    println!(
        "portable inventory: exact cell content/provenance preserved; ContentStateId and runtime namespace replaced"
    );
    drop((capture, restored, book));
    finish(runtime);
}

#[test]
fn portable_export_rejects_a_historical_root_with_a_later_branch_floor() {
    let runtime = IntegrationRuntime::standalone().unwrap();
    let limits = Limits {
        allocated_cells: 2,
        ..Limits::default()
    };
    let mut book = Workbook::new_in("floor inventory", limits, &runtime).unwrap();
    book.apply(book.expected(), &[entry(1, 1, "1")]).unwrap();
    let a = book.pin_history_root(book.expected()).unwrap();
    book.apply(book.expected(), &[entry(2, 1, "2")]).unwrap();
    let b_floor = book.history_metrics().unwrap().next_node;
    book.activate_history_root(book.expected(), &a, &Default::default())
        .unwrap();
    assert_eq!(book.history_metrics().unwrap().next_node, b_floor);
    assert!(book.apply(book.expected(), &[entry(3, 1, "3")]).is_err());
    let expected = book.expected();
    let content = book.history_content_id();
    assert!(book.export_portable(expected).is_err());
    assert_eq!(book.expected(), expected);
    assert_eq!(book.history_content_id(), content);
    assert_eq!(book.history_metrics().unwrap().next_node, b_floor);
    println!(
        "portable floor inventory: historical root has one cell and floor {b_floor}; transfer rejects the non-contiguous ledger instead of restoring it"
    );
    drop((a, book));
    finish(runtime);
}

#[test]
fn process_local_model_namespace_restarts_in_independent_processes() {
    let child = || {
        let output = std::process::Command::new(std::env::current_exe().unwrap())
            .args(["--exact", "process_identity_probe_child", "--nocapture"])
            .env("PIVOT_HISTORY_IDENTITY_PROBE", "1")
            .output()
            .unwrap();
        assert!(
            output.status.success(),
            "{}",
            String::from_utf8_lossy(&output.stderr)
        );
        let stdout = String::from_utf8(output.stdout).unwrap();
        stdout
            .lines()
            .find_map(|line| line.strip_prefix("HISTORY_PROCESS_IDENTITY "))
            .unwrap()
            .to_owned()
    };
    let first = child();
    let second = child();
    assert_eq!(first, second);
    println!(
        "independent process namespaces: {first}; {second}; equality of these bits is not durable identity"
    );
}

#[test]
fn process_identity_probe_child() {
    if std::env::var_os("PIVOT_HISTORY_IDENTITY_PROBE").is_none() {
        return;
    }
    let runtime = IntegrationRuntime::standalone().unwrap();
    let mut book = Workbook::new_in("process identity", Limits::default(), &runtime).unwrap();
    book.apply(book.expected(), &[entry(1, 1, "1")]).unwrap();
    println!(
        "HISTORY_PROCESS_IDENTITY {:?}",
        book.history_runtime_node(p(1, 1)).unwrap()
    );
    drop(book);
    finish(runtime);
}
