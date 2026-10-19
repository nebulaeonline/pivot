//! Optimized comparative harness; opt-in only. See docs/performance.
use pivot_core::experiment_profile as profile;
use pivot_workbook::{scaling::*, *};
use std::{
    alloc::{GlobalAlloc, Layout, System},
    hint::black_box,
    sync::atomic::{AtomicBool, AtomicU64, Ordering::Relaxed},
    time::Instant,
};
static TRACK: AtomicBool = AtomicBool::new(false);
static LIVE: AtomicU64 = AtomicU64::new(0);
static PEAK: AtomicU64 = AtomicU64::new(0);
static ALLOC: AtomicU64 = AtomicU64::new(0);
static CALLS: AtomicU64 = AtomicU64::new(0);
static PHASE_BYTES: [AtomicU64; profile::PHASES] = [const { AtomicU64::new(0) }; profile::PHASES];
struct Counting;
fn allocated(size: usize) {
    ALLOC.fetch_add(size as u64, Relaxed);
    CALLS.fetch_add(1, Relaxed);
    PHASE_BYTES[profile::phase()].fetch_add(size as u64, Relaxed);
    let live = LIVE.fetch_add(size as u64, Relaxed) + size as u64;
    PEAK.fetch_max(live, Relaxed);
}
// SAFETY: all operations delegate unchanged pointer/layout pairs to System.
// Counters and phase lookup neither allocate nor touch the allocated memory.
unsafe impl GlobalAlloc for Counting {
    unsafe fn alloc(&self, l: Layout) -> *mut u8 {
        let p = unsafe { System.alloc(l) };
        if !p.is_null() && TRACK.load(Relaxed) {
            allocated(l.size());
        }
        p
    }
    unsafe fn alloc_zeroed(&self, l: Layout) -> *mut u8 {
        let p = unsafe { System.alloc_zeroed(l) };
        if !p.is_null() && TRACK.load(Relaxed) {
            allocated(l.size());
        }
        p
    }
    unsafe fn dealloc(&self, p: *mut u8, l: Layout) {
        if TRACK.load(Relaxed) {
            LIVE.fetch_sub(l.size() as u64, Relaxed);
        }
        unsafe { System.dealloc(p, l) }
    }
    unsafe fn realloc(&self, p: *mut u8, l: Layout, n: usize) -> *mut u8 {
        let next = unsafe { System.realloc(p, l, n) };
        if !next.is_null() && TRACK.load(Relaxed) {
            LIVE.fetch_sub(l.size() as u64, Relaxed);
            allocated(n);
        }
        next
    }
}
#[global_allocator]
static ALLOCATOR: Counting = Counting;
struct Run {
    shape: String,
    size: usize,
    implementation: &'static str,
    samples: usize,
    profiling: bool,
}
fn measure<T>(r: &Run, op: &str, samples: usize, mut f: impl FnMut() -> T) -> T {
    let mut times = Vec::with_capacity(samples);
    let mut bytes = 0;
    let mut calls = 0;
    let mut copies = 0;
    let mut peak = 0;
    let mut retained = 0i128;
    let mut phase_times = [0u128; profile::PHASES];
    let mut phase_bytes = [0u128; profile::PHASES];
    let mut last = None;
    if samples > 1 {
        for _ in 0..3 {
            drop(black_box(f()));
        }
    }
    for _ in 0..samples {
        drop(last.take());
        profile::reset();
        let a = ALLOC.load(Relaxed);
        let c = CALLS.load(Relaxed);
        let live = LIVE.load(Relaxed);
        PEAK.store(live, Relaxed);
        let pb: [u64; profile::PHASES] = std::array::from_fn(|i| PHASE_BYTES[i].load(Relaxed));
        let start = Instant::now();
        let result = black_box(f());
        let ns = start.elapsed().as_nanos() as u64;
        times.push(ns);
        bytes += ALLOC.load(Relaxed) - a;
        calls += CALLS.load(Relaxed) - c;
        peak = peak.max(PEAK.load(Relaxed).saturating_sub(live));
        retained += LIVE.load(Relaxed) as i128 - live as i128;
        let (pt, cp) = profile::metrics();
        copies += cp;
        for i in 0..profile::PHASES {
            phase_times[i] += pt[i] as u128;
            phase_bytes[i] += (PHASE_BYTES[i].load(Relaxed) - pb[i]) as u128;
        }
        last = Some(result);
    }
    times.sort_unstable();
    let p95 = times[(samples * 95).div_ceil(100) - 1];
    println!(
        "ROW,{},{},{},{},{},{},{},{},{},{},{},{},{},{},{}",
        r.implementation,
        r.shape,
        r.size,
        op,
        if r.profiling { "profile" } else { "latency" },
        samples,
        times[samples / 2],
        p95,
        times[samples - 1],
        bytes / samples as u64,
        calls / samples as u64,
        copies / samples as u64,
        peak,
        retained / samples as i128,
        LIVE.load(Relaxed)
    );
    if r.profiling {
        for i in 1..profile::PHASES {
            if phase_times[i] > 0 {
                println!(
                    "PHASE,{},{},{},{},{},{},{}",
                    r.implementation,
                    r.shape,
                    r.size,
                    op,
                    profile::NAMES[i],
                    phase_times[i] / samples as u128,
                    phase_bytes[i] / samples as u128
                );
            }
        }
    }
    last.unwrap()
}
fn main() {
    let args: Vec<_> = std::env::args().collect();
    let flag = |name: &str| {
        args.iter()
            .position(|x| x == name)
            .and_then(|i| args.get(i + 1))
            .cloned()
    };
    let profiling = args.iter().any(|x| x == "--profile");
    // Enable before any workload allocation; existing runtime allocations are not
    // tracked, but their later frees occur only at process teardown after reports.
    TRACK.store(profiling, Relaxed);
    profile::enable(profiling);
    let samples = flag("--samples")
        .map(|s| s.parse().unwrap())
        .unwrap_or(if profiling { 5 } else { 101 });
    assert!(samples > 0);
    let sizes: Vec<usize> = flag("--sizes")
        .unwrap_or("1000,10000,100000".into())
        .split(',')
        .map(|s| s.parse().unwrap())
        .collect();
    let shapes: Vec<String> = flag("--shapes")
        .unwrap_or("independent,chain,fanout,mixed".into())
        .split(',')
        .map(str::to_owned)
        .collect();
    println!(
        "META,os={},arch={},profile={},samples={},release={}",
        std::env::consts::OS,
        std::env::consts::ARCH,
        profiling,
        samples,
        !cfg!(debug_assertions)
    );
    if cfg!(debug_assertions) {
        panic!("run with --release");
    }
    println!(
        "HEADER,implementation,shape,cells,operation,mode,samples,p50_ns,p95_ns,worst_ns,allocated_bytes_mean,allocation_calls_mean,instrumented_payload_copy_bytes_mean,peak_extra_heap_bytes,retained_delta_mean_bytes,live_heap_bytes"
    );
    for size in sizes {
        for shape in &shapes {
            assert!(["independent", "chain", "fanout", "mixed"].contains(&shape.as_str()));
            let r = Run {
                shape: shape.clone(),
                size,
                implementation: "baseline",
                samples,
                profiling,
            };
            let mut w = measure(&r, "bulk_construct", 1, || build(size, shape));
            let input = position(0);
            let formula = position(if shape == "chain" {
                size.min(1001) - 1
            } else {
                size - 1
            });
            // Ensure every shape has a selected computed output, independent of bulk mix.
            let source = format!(
                "={}+1",
                address(position(if shape == "chain" {
                    size.min(1001) - 2
                } else if shape == "mixed" {
                    size - 2
                } else {
                    0
                }))
            );
            w.apply(
                w.expected(),
                &[Edit::SetFormulaText {
                    position: formula,
                    source: source.clone(),
                }],
            )
            .unwrap();
            let output = w.cell_at(formula).unwrap().unwrap().reference;
            let captured = w.capture(w.expected()).unwrap();
            for _ in 0..3 {
                black_box(captured.evaluate(&[output]).unwrap());
            }
            measure(&r, "selected_evaluate", r.samples, || {
                captured.evaluate(&[output]).unwrap()
            });
            drop(captured);
            let affected: Vec<_> = if shape == "fanout" {
                (1..size)
                    .map(|i| w.cell_at(position(i)).unwrap().unwrap().reference)
                    .collect()
            } else {
                vec![output]
            };
            measure(&r, "affected_capture_evaluate", r.samples.min(31), || {
                w.capture(w.expected())
                    .unwrap()
                    .evaluate(&affected)
                    .unwrap()
            });
            let mut v = 1u32;
            measure(&r, "isolated_input_edit", r.samples, || {
                v = 3 - v;
                w.apply(
                    w.expected(),
                    &[Edit::SetDecimal {
                        position: input,
                        value: DecimalInput::from_parts(v, 0, 0, false, 0).unwrap(),
                    }],
                )
                .unwrap()
            });
            measure(&r, "capture", r.samples, || {
                w.capture(w.expected()).unwrap()
            });
            measure(&r, "formula_replace", r.samples, || {
                w.apply(
                    w.expected(),
                    &[Edit::SetFormulaText {
                        position: formula,
                        source: source.clone(),
                    }],
                )
                .unwrap()
            });
            measure(&r, "stage_only", r.samples, || stage(&w));
            measure(&r, "quota_only", r.samples, || quota(&w));
            let viewport = Rectangle {
                first: Position::new(1, 1),
                last: Position::new(100, 50),
            };
            measure(&r, "viewport_100x50", r.samples, || {
                w.read(w.expected(), viewport, 5000).unwrap()
            });
            measure(&r, "update_capture_evaluate", r.samples, || {
                v = 3 - v;
                w.apply(
                    w.expected(),
                    &[Edit::SetDecimal {
                        position: input,
                        value: DecimalInput::from_parts(v, 0, 0, false, 0).unwrap(),
                    }],
                )
                .unwrap();
                w.capture(w.expected())
                    .unwrap()
                    .evaluate(&[output])
                    .unwrap()
            });
            measure(&r, "retain_16_captures", 1, || {
                let mut states = Vec::new();
                for i in 0..16 {
                    w.apply(
                        w.expected(),
                        &[Edit::SetDecimal {
                            position: input,
                            value: DecimalInput::from_parts(i + 1, 0, 0, false, 0).unwrap(),
                        }],
                    )
                    .unwrap();
                    states.push(w.capture(w.expected()).unwrap());
                }
                states
            });
            let r = Run {
                implementation: "shared",
                ..r
            };
            let mut input_book = Some(w);
            let mut shared = measure(&r, "compile_shared_topology", 1, || {
                SharedWorkbook::new(input_book.take().unwrap())
            });
            let old = shared.capture().unwrap();
            let old_value = old.value(output).unwrap();
            let old_version = old.version;
            let mut v = 1u32;
            measure(&r, "isolated_input_edit", r.samples, || {
                v = 3 - v;
                shared
                    .apply(
                        shared.expected(),
                        &[Edit::SetDecimal {
                            position: input,
                            value: DecimalInput::from_parts(v, 0, 0, false, 0).unwrap(),
                        }],
                    )
                    .unwrap()
            });
            assert_eq!(old.value(output).unwrap(), old_value);
            assert_eq!(old.version, old_version);
            measure(&r, "capture_1000", r.samples, || {
                for _ in 0..1000 {
                    black_box(shared.capture().unwrap());
                }
            });
            let captured = shared.capture().unwrap();
            measure(&r, "selected_evaluate", r.samples, || {
                captured.evaluate(&[output]).unwrap()
            });
            drop(captured);
            measure(&r, "affected_capture_evaluate", r.samples.min(31), || {
                shared.capture().unwrap().evaluate(&affected).unwrap()
            });
            measure(&r, "viewport_100x50", r.samples, || {
                shared.read(viewport, 5000).unwrap()
            });
            measure(&r, "update_capture_evaluate", r.samples, || {
                v = 3 - v;
                shared
                    .apply(
                        shared.expected(),
                        &[Edit::SetDecimal {
                            position: input,
                            value: DecimalInput::from_parts(v, 0, 0, false, 0).unwrap(),
                        }],
                    )
                    .unwrap();
                shared.capture().unwrap().evaluate(&[output]).unwrap()
            });
            measure(&r, "retain_16_captures", 1, || {
                let mut states = Vec::new();
                for i in 0..16 {
                    shared
                        .apply(
                            shared.expected(),
                            &[Edit::SetDecimal {
                                position: input,
                                value: DecimalInput::from_parts(i + 1, 0, 0, false, 0).unwrap(),
                            }],
                        )
                        .unwrap();
                    states.push(shared.capture().unwrap());
                }
                states
            });
            // Structural edits remain deliberately expensive. This measures that cost.
            measure(&r, "formula_replace", r.samples, || {
                shared
                    .apply(
                        shared.expected(),
                        &[Edit::SetFormulaText {
                            position: formula,
                            source: source.clone(),
                        }],
                    )
                    .unwrap()
            });
            // Populate the overlay at every eligible input, outside single-edit timing.
            measure(&r, "populate_overlay", 1, || {
                for i in 0..size {
                    let p = position(i);
                    if matches!(
                        shared.cell_at(p).unwrap().unwrap().content,
                        Content::Decimal(_) | Content::Blank
                    ) {
                        shared
                            .apply(
                                shared.expected(),
                                &[Edit::SetDecimal {
                                    position: p,
                                    value: DecimalInput::from_parts(42, 0, 0, false, 0).unwrap(),
                                }],
                            )
                            .unwrap();
                    }
                }
            });
            measure(&r, "full_overlay_input_edit", r.samples, || {
                v = 3 - v;
                shared
                    .apply(
                        shared.expected(),
                        &[Edit::SetDecimal {
                            position: input,
                            value: DecimalInput::from_parts(v, 0, 0, false, 0).unwrap(),
                        }],
                    )
                    .unwrap()
            });
            measure(&r, "full_overlay_capture_1000", r.samples, || {
                for _ in 0..1000 {
                    black_box(shared.capture().unwrap());
                }
            });
            let expected = if shape == "chain" {
                (size.min(1001) - 1) as u32 + v
            } else if shape == "mixed" {
                43
            } else {
                v + 1
            };
            assert_eq!(
                shared.capture().unwrap().value(output).unwrap(),
                pivot_types::PivotDecimal::from(expected as i64)
            );
        }
    }
}
