//! Production-facade comparison; no research Graph API used here.
use pivot_core::model::{NodeId, WorkingModel, integration::IntegrationRuntime};
use pivot_types::PivotDecimal;
use std::{
    alloc::{GlobalAlloc, Layout, System},
    sync::atomic::{AtomicUsize, Ordering::Relaxed},
    time::Instant,
};
static LIVE: AtomicUsize = AtomicUsize::new(0);
static PEAK: AtomicUsize = AtomicUsize::new(0);
struct Counting;
fn allocated(n: usize) {
    let live = LIVE.fetch_add(n, Relaxed) + n;
    PEAK.fetch_max(live, Relaxed);
}
// SAFETY: pointer/layout operations are delegated unchanged to System. Atomic
// counters do not allocate, dereference or otherwise modify caller storage.
unsafe impl GlobalAlloc for Counting {
    unsafe fn alloc(&self, l: Layout) -> *mut u8 {
        let p = unsafe { System.alloc(l) };
        if !p.is_null() {
            allocated(l.size())
        }
        p
    }
    unsafe fn alloc_zeroed(&self, l: Layout) -> *mut u8 {
        let p = unsafe { System.alloc_zeroed(l) };
        if !p.is_null() {
            allocated(l.size())
        }
        p
    }
    unsafe fn dealloc(&self, p: *mut u8, l: Layout) {
        LIVE.fetch_sub(l.size(), Relaxed);
        unsafe { System.dealloc(p, l) }
    }
    unsafe fn realloc(&self, p: *mut u8, l: Layout, n: usize) -> *mut u8 {
        let q = unsafe { System.realloc(p, l, n) };
        if !q.is_null() {
            LIVE.fetch_sub(l.size(), Relaxed);
            allocated(n)
        }
        q
    }
}
#[global_allocator]
static ALLOCATOR: Counting = Counting;
fn measure(name: &str, samples: usize, mut f: impl FnMut(usize)) {
    for i in 0..3 {
        f(i);
    }
    let mut values = Vec::with_capacity(samples);
    for i in 0..samples {
        let start = Instant::now();
        f(i + 3);
        values.push(start.elapsed().as_nanos());
    }
    values.sort_unstable();
    let p95 = values[(samples * 95).div_ceil(100) - 1];
    println!(
        "ROW,{name},{samples},{},{},{}",
        values[samples / 2],
        p95,
        values[samples - 1]
    );
}
fn check(m: &WorkingModel, out: NodeId, expected: i64) {
    let capture = m.capture_calculation().expect("valid capture");
    let result = capture.evaluate(&[out]).expect("evaluation");
    assert_eq!(result.values()[&out], Ok(PivotDecimal::from(expected)));
}
fn main() {
    let args: Vec<_> = std::env::args().collect();
    let size: usize = args[1].parse().unwrap();
    let implementation = &args[2];
    let samples: usize = args[3].parse().unwrap();
    assert!(size >= 1000 && samples > 0);
    let runtime = IntegrationRuntime::new(2 * 1024 * 1024 * 1024, 64).unwrap();
    let mut model = WorkingModel::new("Facade benchmark").unwrap();
    let nodes: Vec<_> = (0..size)
        .map(|i| model.add_input(format!("n{i}"), 1.into()).unwrap())
        .collect();
    let count = 1000.min(size - 1);
    let start = size - count;
    for (index, &id) in nodes.iter().enumerate().skip(start) {
        let dependency = if index == start { 0 } else { index - 1 };
        model
            .set_expression(id, format!("n{dependency}+1"))
            .unwrap();
    }
    let activation = Instant::now();
    if implementation == "integrated" {
        model.enable_persistent(&runtime).unwrap();
    } else {
        assert_eq!(implementation, "reference");
    }
    runtime.flush();
    println!("ACTIVATION_NS,{}", activation.elapsed().as_nanos());
    measure("unchanged_capture", samples, |_| {
        drop(model.capture_calculation().unwrap());
    });
    let output = nodes[size - 1];
    let mut input = 1i64;
    let mut increments = vec![1i64; count];
    measure("input_capture", samples, |i| {
        let mut candidate = model.stage_expected(model.revision()).unwrap();
        candidate.set_input(nodes[0], (i as i64).into()).unwrap();
        let capture = candidate.capture_calculation().ok().unwrap();
        model = candidate;
        drop(capture);
        input = i as i64;
    });
    measure("input_chain", samples, |i| {
        let mut candidate = model.stage_expected(model.revision()).unwrap();
        candidate.set_input(nodes[0], (i as i64).into()).unwrap();
        check(
            &candidate,
            output,
            i as i64 + increments.iter().sum::<i64>(),
        );
        model = candidate;
        input = i as i64;
    });
    for (label, position) in [
        ("formula_front", 0),
        ("formula_middle", count / 2),
        ("formula_tail", count - 1),
    ] {
        measure(label, samples, |i| {
            let index = start + position;
            let dependency = if position == 0 { 0 } else { index - 1 };
            let increment = 1 + (i % 2) as i64;
            let mut candidate = model.stage_expected(model.revision()).unwrap();
            candidate
                .set_expression(nodes[index], format!("n{dependency}+{increment}"))
                .unwrap();
            increments[position] = increment;
            check(&candidate, output, input + increments.iter().sum::<i64>());
            model = candidate;
        });
    }
    measure("invalid_draft_repair", samples, |_| {
        let mut candidate = model.stage_expected(model.revision()).unwrap();
        candidate.set_expression(output, "missing+1").unwrap();
        assert!(candidate.capture_calculation().is_err());
        let previous = if count == 1 { 0 } else { size - 2 };
        candidate
            .set_expression(output, format!("n{previous}+{}", increments[count - 1]))
            .unwrap();
        check(&candidate, output, input + increments.iter().sum::<i64>());
        model = candidate;
    });
    measure("named_add_formula", samples, |i| {
        let mut candidate = model.stage_expected(model.revision()).unwrap();
        candidate
            .add_input(format!("added{i}"), (i as i64).into())
            .unwrap();
        let out = candidate
            .add_computed(format!("computed{i}"), format!("added{i}*2+1"))
            .unwrap();
        check(&candidate, out, 2 * i as i64 + 1);
        model = candidate;
    });
    for keep in [16, 32] {
        runtime.flush();
        let before = LIVE.load(Relaxed);
        let mut captures = Vec::with_capacity(keep);
        let timer = Instant::now();
        for i in 0..keep {
            let mut candidate = model.stage();
            candidate
                .set_expression(output, format!("n{}+{}", size - 2, 1 + i % 2))
                .unwrap();
            captures.push(candidate.capture_calculation().ok().unwrap());
            model = candidate;
        }
        let elapsed = timer.elapsed().as_nanos();
        runtime.flush();
        let retained = LIVE.load(Relaxed).saturating_sub(before);
        let timer = Instant::now();
        drop(captures);
        let enqueue = timer.elapsed().as_nanos();
        let timer = Instant::now();
        runtime.flush();
        let drain = timer.elapsed().as_nanos();
        println!("RETAIN,{keep},{elapsed},{retained},{enqueue},{drain}");
    }
    runtime.flush();
    println!(
        "HEAP,{},{},{}",
        LIVE.load(Relaxed),
        PEAK.load(Relaxed),
        runtime.memory().stats().peak
    );
    let timer = Instant::now();
    drop(model);
    let enqueue = timer.elapsed().as_nanos();
    let memory = runtime.memory();
    let timer = Instant::now();
    assert!(runtime.finish().is_ok());
    println!(
        "FINAL_DROP,{enqueue},{},{}",
        timer.elapsed().as_nanos(),
        memory.stats().charged
    );
    assert_eq!(memory.stats().charged, 0);
}
