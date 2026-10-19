//! Additional cold-key control; uses the same accepted baseline and shared prototype.
use pivot_types::PivotDecimal;
use pivot_workbook::{scaling::*, *};
use std::{hint::black_box, time::Instant};
fn input(index: usize, n: i64) -> Edit {
    Edit::SetDecimal {
        position: position(index),
        value: DecimalInput::from_decimal(PivotDecimal::from(n)),
    }
}
fn sample(mut edit: impl FnMut(usize)) -> (u128, u128, u128) {
    let mut times = Vec::with_capacity(101);
    for i in 0..104 {
        let start = Instant::now();
        edit(i);
        let elapsed = start.elapsed().as_nanos();
        if i >= 3 {
            times.push(elapsed);
        }
    }
    times.sort_unstable();
    (times[50], times[95], times[100])
}
fn main() {
    if cfg!(debug_assertions) {
        panic!("run with --release");
    }
    println!("implementation,cells,samples,p50_ns,p95_ns,worst_ns");
    for size in [1000, 10000, 100000] {
        let mut w = build(size, "independent");
        let mut shared = SharedWorkbook::new(stage(&w));
        // Separate commits intentionally fill every input's patch entry and then
        // restore equal baseline values. No setup occurs inside sample timers.
        for i in 0..size - 1 {
            shared.apply(shared.expected(), &[input(i, 2)]).unwrap();
            shared.apply(shared.expected(), &[input(i, 1)]).unwrap();
        }
        let pick = |i| (65537usize * i + 13) % (size - 1);
        let (p50, p95, worst) = sample(|i| {
            black_box(w.apply(w.expected(), &[input(pick(i), 43)]).unwrap());
        });
        println!("baseline,{size},101,{p50},{p95},{worst}");
        let (p50, p95, worst) = sample(|i| {
            black_box(
                shared
                    .apply(shared.expected(), &[input(pick(i), 43)])
                    .unwrap(),
            );
        });
        println!("shared,{size},101,{p50},{p95},{worst}");
        for i in 0..size {
            assert_eq!(
                w.cell_at(position(i)).unwrap(),
                shared.cell_at(position(i)).unwrap()
            );
        }
    }
}
