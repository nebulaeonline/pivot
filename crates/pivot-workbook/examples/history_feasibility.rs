//! Bounded measurement window, not production Undo/Redo or a general history stack.
use pivot_core::model::integration::IntegrationRuntime;
use pivot_workbook::*;
use std::{collections::VecDeque, time::Instant};
const BUDGET: usize = 16 * 1024 * 1024;
fn p(r: u32, c: u32) -> Position {
    Position::new(r, c)
}
fn e(r: u32, c: u32, s: String) -> Edit {
    Edit::SetEntry {
        position: p(r, c),
        source: s,
    }
}
fn main() {
    let case = std::env::args().nth(1).unwrap_or_else(|| "small".into());
    let (cells, columns, dense) = match case.as_str() {
        "small" => (16, 4, false),
        "medium" => (256, 32, false),
        "near" => (1024, 32, false),
        "formula-medium" => (256, 32, true),
        "tombstones" => (1, 32, false),
        _ => panic!("unknown case"),
    };
    let rt = IntegrationRuntime::standalone().unwrap();
    let baseline = rt.memory().stats();
    let mut b = Workbook::new_in("history measurement", Limits::default(), &rt).unwrap();
    for offset in (0..cells).step_by(256) {
        let edits = (offset..(offset + 256).min(cells))
            .map(|i| {
                let r = (i / columns + 1) as u32;
                let c = (i % columns + 1) as u32;
                e(
                    r,
                    c,
                    if dense && c > 1 {
                        format!("=$A$1+{c}")
                    } else {
                        (i + 1).to_string()
                    },
                )
            })
            .collect::<Vec<_>>();
        b.apply(b.expected(), &edits).unwrap();
    }
    let mut setup_rejections = 0;
    if case == "tombstones" {
        let mut remaining = 8191usize;
        let mut batch_size = 256;
        while remaining > 0 {
            let n = remaining.min(batch_size);
            let edits = (0..n)
                .map(|i| e((i / 32 + 2) as u32, (i % 32 + 1) as u32, "1".into()))
                .collect::<Vec<_>>();
            match b.apply(b.expected(), &edits) {
                Ok(_) => {}
                Err(Error::Limit) => {
                    setup_rejections += 1;
                    if batch_size == 1 {
                        break;
                    }
                    batch_size = (batch_size / 2).max(1);
                    continue;
                }
                Err(error) => panic!("setup: {error:?}"),
            }
            let edits = (0..n)
                .map(|i| Edit::Retire(p((i / 32 + 2) as u32, (i % 32 + 1) as u32)))
                .collect::<Vec<_>>();
            b.apply(b.expected(), &edits).unwrap();
            remaining -= n;
            rt.flush();
        }
    }
    rt.flush();
    let initial = b.history_weight().unwrap();
    let initial_allocated = rt.memory().stats().allocated - baseline.allocated;
    assert!(
        initial >= initial_allocated,
        "bound smaller than actual ordinary-root allocation"
    );
    let mut retained = VecDeque::new();
    retained.push_back(b.pin_history_root(b.expected()).unwrap());
    let mut sum = initial;
    let mut admitted = 0;
    let mut rejected = 0;
    let mut pruned = 0;
    let mut samples = Vec::new();
    for step in 0..32 {
        let before = b.expected();
        let mut candidate = b.stage_history_candidate(before).unwrap();
        let start = Instant::now();
        if step % 2 == 0 || case == "tombstones" {
            candidate
                .apply(before, &[e(1, 1, format!("{}", 1000 + step))])
                .unwrap();
        } else {
            candidate
                .fill(
                    before,
                    Rectangle {
                        first: p(1, 1),
                        last: p(if cells == 16 { 4 } else { 8 }, columns as u32),
                    },
                    FillDirection::Down,
                    &Default::default(),
                )
                .unwrap();
        }
        let changed = candidate.expected() != before;
        if !changed {
            continue;
        }
        let after = candidate.pin_history_root(candidate.expected()).unwrap();
        let weight = after.weight();
        let mut remove = 0;
        let mut planned = sum + weight;
        while retained.len() - remove > 1 && (planned > BUDGET || retained.len() - remove >= 33) {
            planned -= retained[remove].weight();
            remove += 1;
        }
        if planned > BUDGET {
            rejected += 1;
            drop((after, candidate));
            assert_eq!(b.expected(), before);
            continue;
        }
        for _ in 0..remove {
            sum -= retained.pop_front().unwrap().weight();
            pruned += 1;
        }
        sum += weight;
        retained.push_back(after);
        b = candidate;
        admitted += 1;
        samples.push(start.elapsed().as_micros() as u64);
    }
    let peak_states = retained.len();
    let logical = sum;
    rt.flush();
    let window = rt.memory().stats();
    // Branch replacement and pruning while a calculation, not history, retains B.
    let a = b.pin_history_root(b.expected()).unwrap();
    b.apply(b.expected(), &[e(1, 1, "123456".into())]).unwrap();
    let capture = b.capture_local(b.expected()).unwrap();
    let output = capture.cell_at(p(1, 1)).unwrap();
    b.activate_history_root(b.expected(), &a, &Default::default())
        .unwrap();
    retained.clear();
    rt.flush();
    assert_eq!(
        capture.evaluate(&[output]).unwrap().values[0].1,
        Ok(123456.into())
    );
    rt.flush();
    let with_capture = rt.memory().stats();
    drop(capture);
    rt.flush();
    let without_capture = rt.memory().stats();
    assert!(with_capture.charged > without_capture.charged);
    let query = Instant::now();
    for _ in 0..10_000 {
        std::hint::black_box(b.history_weight().unwrap());
    }
    let query_ns = query.elapsed().as_nanos() / 10_000;
    let activation = Instant::now();
    for _ in 0..100 {
        b.activate_history_root(b.expected(), &a, &Default::default())
            .unwrap();
    }
    let activation_ns = activation.elapsed().as_nanos() / 100;
    assert_eq!(b.history_metrics().unwrap().weight_scan_visits, 0);
    samples.sort_unstable();
    let p95 = if samples.is_empty() {
        0
    } else {
        samples[(samples.len() - 1) * 95 / 100]
    };
    let allocated = b.history_metrics().unwrap().next_node;
    drop((a, b, retained));
    rt.flush();
    let m = rt.memory();
    assert!(rt.finish().is_ok());
    let final_charge = m.stats().charged;
    assert_eq!(final_charge, 0);
    println!(
        "{{\"case\":\"{case}\",\"initial_cells\":{cells},\"setup_admission_rejections\":{setup_rejections},\"allocated_node_floor\":{allocated},\"initial_weight\":{initial},\"initial_actual_allocated\":{initial_allocated},\"admitted_sampled_transitions\":{admitted},\"rejected_sampled_transitions\":{rejected},\"pruned_sampled_states\":{pruned},\"retained_states\":{peak_states},\"logical_window_weight\":{logical},\"window_domain_charged\":{},\"window_domain_allocated\":{},\"domain_peak\":{},\"with_capture_after_prune\":{},\"without_capture_after_prune\":{},\"weight_query_ns\":{query_ns},\"activation_ns\":{activation_ns},\"candidate_p95_us\":{p95},\"weight_scan_visits\":0,\"remaining_charge\":{final_charge}}}",
        window.charged,
        window.allocated,
        window.peak,
        with_capture.charged,
        without_capture.charged
    );
}
