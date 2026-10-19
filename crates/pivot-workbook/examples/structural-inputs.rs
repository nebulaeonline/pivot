//! Changing-key control for the paged prototype, separate from full RSS measurements.
use pivot_workbook::{
    scaling::{position, stage},
    structural::{self, Metadata, StructuralWorkbook},
    *,
};
use std::{hint::black_box, time::Instant};
fn sample(mut f: impl FnMut(usize)) -> (u128, u128, u128) {
    let mut times = Vec::with_capacity(101);
    for i in 0..104 {
        let start = Instant::now();
        f(i);
        let ns = start.elapsed().as_nanos();
        if i >= 3 {
            times.push(ns);
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
        let mut baseline = structural::build(size, "independent");
        let mut paged = StructuralWorkbook::new(stage(&baseline)).unwrap();
        let pick = |i| (65537usize * i + 13) % (size - 128);
        let edit = |i| Edit::SetDecimal {
            position: position(pick(i)),
            value: DecimalInput::from_decimal(43.into()),
        };
        let (p50, p95, worst) = sample(|i| {
            black_box(baseline.apply(baseline.expected(), &[edit(i)]).unwrap());
        });
        println!("baseline,{size},101,{p50},{p95},{worst}");
        let (p50, p95, worst) = sample(|i| {
            black_box(paged.apply(paged.version(), &[edit(i)]).unwrap());
        });
        println!("paged,{size},101,{p50},{p95},{worst}");
        for i in 0..size {
            let actual = paged.metadata(position(i)).unwrap();
            let expected = baseline.cell_at(position(i)).unwrap().unwrap().content;
            match actual {
                Metadata::Original(content) => assert_eq!(content, expected),
                Metadata::Decimal(value) => assert_eq!(Content::Decimal(value), expected),
                _ => unreachable!(),
            }
        }
    }
}
