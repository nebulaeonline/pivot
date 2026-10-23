//! Identity-budget measurement only; not a disk store or history coordinator.
use pivot_core::model::integration::IntegrationRuntime;
use pivot_workbook::*;
use std::time::Instant;

fn edit(row: u32, value: usize) -> Edit {
    Edit::SetEntry {
        position: Position::new(row, 1),
        source: value.to_string(),
    }
}
fn main() {
    let case = std::env::args().nth(1).expect("replace or branch");
    assert!(case == "replace" || case == "branch");
    let runtime = IntegrationRuntime::standalone().unwrap();
    let limits = Limits::default();
    let mut book = Workbook::new_in("identity budget", limits, &runtime).unwrap();
    book.apply(book.expected(), &[edit(1, 1)]).unwrap();
    let original = book.pin_history_root(book.expected()).unwrap();
    let mut committed = 0usize;
    let mut rejected = None;
    let start = Instant::now();
    for i in 0..10_000 {
        let before = book.expected();
        let content = book.history_content_id();
        match book.apply(
            before,
            &[edit(if case == "replace" { 1 } else { 2 }, i + 2)],
        ) {
            Ok(_) => {
                committed += 1;
                if case == "branch" {
                    book.activate_history_root(book.expected(), &original, &Default::default())
                        .unwrap();
                    assert!(book.cell_at(Position::new(2, 1)).unwrap().is_none());
                }
            }
            Err(error) => {
                assert_eq!(book.expected(), before);
                assert_eq!(book.history_content_id(), content);
                rejected = Some(format!("{error:?}"));
                break;
            }
        }
        runtime.flush();
    }
    let elapsed = start.elapsed().as_millis();
    let metrics = book.history_metrics().unwrap();
    let floor = metrics.next_node;
    if case == "replace" {
        assert_eq!(committed, 10_000);
        assert_eq!(floor, 1);
        assert!(rejected.is_none());
    } else {
        assert_eq!(committed, limits.allocated_cells - 1);
        assert_eq!(floor, limits.allocated_cells as u64);
        assert_eq!(rejected.as_deref(), Some("Limit"));
    }
    drop((original, book));
    runtime.flush();
    let memory = runtime.memory();
    assert!(runtime.finish().is_ok());
    let charge = memory.stats().charged;
    assert_eq!(charge, 0);
    println!(
        "{{\"case\":\"{case}\",\"attempt_target\":10000,\"committed_edits\":{committed},\"initial_identities\":1,\"allocated_node_floor\":{floor},\"active_cells_at_end\":1,\"first_rejection\":{},\"elapsed_ms\":{elapsed},\"remaining_charge\":{charge},\"disk_history\":false}}",
        rejected.map_or("null".into(), |reason| format!("\"{reason}\""))
    );
}
