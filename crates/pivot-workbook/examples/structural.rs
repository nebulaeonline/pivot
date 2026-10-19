//! Optimized comparative harness; opt-in only. See docs/performance.
use pivot_core::experiment_profile as profile;
use pivot_workbook::{
    scaling::position,
    structural::{self, StructuralWorkbook},
    *,
};
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
    size: usize,
    shape: String,
    implementation: String,
    samples: usize,
}
fn measure<T>(run: &Run, name: &str, samples: usize, mut f: impl FnMut(usize) -> T) -> T {
    let mut times = Vec::with_capacity(samples);
    let mut bytes = 0;
    let mut calls = 0;
    let mut copies = 0;
    let mut peak = 0;
    let mut phase = [0u128; profile::PHASES];
    let mut phase_bytes = [0u128; profile::PHASES];
    let mut last = None;
    let initial = LIVE.load(Relaxed);
    for i in 0..samples + usize::from(samples > 1) * 3 {
        drop(last.take()); // Previous returned capture reclamation is separately measured.
        let warmup = samples > 1 && i < 3;
        profile::reset();
        let before = (ALLOC.load(Relaxed), CALLS.load(Relaxed), LIVE.load(Relaxed));
        let pb: [u64; profile::PHASES] = std::array::from_fn(|j| PHASE_BYTES[j].load(Relaxed));
        PEAK.store(before.2, Relaxed);
        let start = Instant::now();
        let value = black_box(f(i));
        let elapsed = start.elapsed().as_nanos();
        if !warmup {
            times.push(elapsed);
            bytes += ALLOC.load(Relaxed) - before.0;
            calls += CALLS.load(Relaxed) - before.1;
            peak = peak.max(PEAK.load(Relaxed).saturating_sub(before.2));
            let (pt, cp) = profile::metrics();
            copies += cp;
            for j in 0..profile::PHASES {
                phase[j] += pt[j] as u128;
                phase_bytes[j] += (PHASE_BYTES[j].load(Relaxed) - pb[j]) as u128;
            }
        }
        last = Some(value);
    }
    times.sort_unstable();
    println!(
        "ROW,{},{},{},{},{},{},{},{},{},{},{},{},{}",
        run.implementation,
        run.size,
        run.shape,
        name,
        samples,
        times[samples / 2],
        times[(samples * 95).div_ceil(100) - 1],
        times[samples - 1],
        bytes / samples as u64,
        calls / samples as u64,
        copies / samples as u64,
        LIVE.load(Relaxed) as i128 - initial as i128,
        peak
    );
    for j in 0..profile::PHASES {
        if phase[j] != 0 || phase_bytes[j] != 0 {
            println!(
                "PHASE,{name},{},{},{}",
                profile::NAMES[j],
                phase[j] / samples as u128,
                phase_bytes[j] / samples as u128
            );
        }
    }
    last.unwrap()
}
fn arg(name: &str, fallback: &str) -> String {
    std::env::args()
        .skip_while(|a| a != name)
        .nth(1)
        .unwrap_or_else(|| fallback.into())
}
fn main() {
    if cfg!(debug_assertions) {
        panic!("run with --release");
    }
    let profiling = std::env::args().any(|a| a == "--profile");
    TRACK.store(profiling, Relaxed);
    profile::enable(profiling);
    let run = Run {
        size: arg("--size", "1000").parse().unwrap(),
        shape: arg("--shape", "independent"),
        implementation: arg("--implementation", "paged"),
        samples: arg("--samples", "101").parse().unwrap(),
    };
    assert!(run.samples > 0 && run.samples <= 101);
    println!(
        "HEADER,implementation,cells,shape,operation,samples,p50_ns,p95_ns,worst_ns,allocated_mean,calls_mean,logical_copy_mean,retained_delta,peak_extra"
    );
    let book = measure(&run, "construction", 1, |_| {
        structural::build(run.size, &run.shape)
    });
    if run.implementation == "baseline" {
        baseline(&run, book);
    } else {
        assert_eq!(run.implementation, "paged");
        paged(&run, book);
    }
}
fn selected(run: &Run) -> Position {
    position(if run.shape == "chain" {
        (run.size - 1).min(1000)
    } else {
        run.size - 1
    })
}
fn input(i: usize) -> Edit {
    Edit::SetDecimal {
        position: position(0),
        value: DecimalInput::from_decimal((i as i64 + 42).into()),
    }
}
fn input_expected(run: &Run, i: usize) -> pivot_types::PivotDecimal {
    let value = match run.shape.as_str() {
        "chain" => i + 42 + (run.size - 1).min(1000),
        "mixed" => 2,
        _ => i + 43,
    };
    (value as i64).into()
}
fn formula(run: &Run, i: usize) -> (Position, Edit) {
    let pos = structural::formula_position(run.size, &run.shape, i);
    let source = if i % 2 == 0 {
        format!("=A1+{}", i + 2)
    } else {
        format!("={}", i + 3)
    };
    (
        pos,
        Edit::SetFormulaText {
            position: pos,
            source,
        },
    )
}
fn baseline(run: &Run, mut book: Workbook) {
    let old = book.capture(book.expected()).unwrap();
    let selected_ref = book.cell_at(selected(run)).unwrap().unwrap().reference;
    let old_value = old.evaluate(&[selected_ref]).unwrap().values[0]
        .1
        .clone()
        .unwrap();
    measure(run, "input_edit", run.samples, |i| {
        book.apply(book.expected(), &[input(i)]).unwrap()
    });
    drop(measure(run, "capture", run.samples, |_| {
        book.capture(book.expected()).unwrap()
    }));
    drop(measure(run, "input_capture_evaluate", run.samples, |i| {
        book.apply(book.expected(), &[input(i)]).unwrap();
        let captured = book.capture(book.expected()).unwrap();
        assert_eq!(
            captured.evaluate(&[selected_ref]).unwrap().values[0]
                .1
                .clone()
                .unwrap(),
            input_expected(run, i)
        );
        captured
    }));
    // Normalize current A1 to 1 before matched formula controls.
    book.apply(
        book.expected(),
        &[Edit::SetDecimal {
            position: position(0),
            value: DecimalInput::from_decimal(1.into()),
        }],
    )
    .unwrap();
    drop(measure(
        run,
        "changing_formula_pipeline",
        run.samples,
        |i| {
            let (pos, edit) = formula(run, i);
            book.apply(book.expected(), &[edit]).unwrap();
            let captured = book.capture(book.expected()).unwrap();
            let output = book.cell_at(pos).unwrap().unwrap().reference;
            assert_eq!(
                captured.evaluate(&[output]).unwrap().values[0]
                    .1
                    .clone()
                    .unwrap(),
                (i as i64 + 3).into()
            );
            captured
        },
    ));
    let mut retained = Vec::new();
    measure(run, "retain_16_structural", 1, |_| {
        for i in 0..16 {
            let (_, edit) = formula(run, i + 200);
            book.apply(book.expected(), &[edit]).unwrap();
            retained.push(book.capture(book.expected()).unwrap());
        }
    });
    measure(run, "cleanup_16_structural", 1, |_| {
        drop(std::mem::take(&mut retained))
    });
    assert_eq!(
        old.evaluate(&[selected_ref]).unwrap().values[0]
            .1
            .clone()
            .unwrap(),
        old_value
    );
    println!("STATE,requested_live_bytes,{}", LIVE.load(Relaxed));
}
fn paged(run: &Run, book: Workbook) {
    let mut fixture = Some(book);
    let mut book = measure(run, "compile_paged", 1, |_| {
        StructuralWorkbook::new(fixture.take().unwrap()).unwrap()
    });
    // The original fixture is dropped before measurements; conversion peak is still in RSS.
    paged_operations(run, &mut book);
}
fn paged_operations(run: &Run, book: &mut StructuralWorkbook) {
    let old = book.capture().unwrap();
    let old_value = old.value(selected(run)).unwrap();
    measure(run, "input_edit", run.samples, |i| {
        book.apply(book.version(), &[input(i)]).unwrap()
    });
    drop(measure(run, "capture", run.samples, |_| {
        book.capture().unwrap()
    }));
    drop(measure(run, "input_capture_evaluate", run.samples, |i| {
        book.apply(book.version(), &[input(i)]).unwrap();
        let captured = book.capture().unwrap();
        assert_eq!(
            captured.value(selected(run)).unwrap(),
            input_expected(run, i)
        );
        captured
    }));
    book.apply(
        book.version(),
        &[Edit::SetDecimal {
            position: position(0),
            value: DecimalInput::from_decimal(1.into()),
        }],
    )
    .unwrap();
    drop(measure(
        run,
        "changing_formula_pipeline",
        run.samples,
        |i| {
            let (pos, edit) = formula(run, i);
            book.apply(book.version(), &[edit]).unwrap();
            let captured = book.capture().unwrap();
            assert_eq!(captured.value(pos).unwrap(), (i as i64 + 3).into());
            captured
        },
    ));
    let mut retained = Vec::new();
    measure(run, "retain_16_structural", 1, |_| {
        for i in 0..16 {
            let (_, edit) = formula(run, i + 200);
            book.apply(book.version(), &[edit]).unwrap();
            retained.push(book.capture().unwrap());
        }
    });
    measure(run, "cleanup_16_structural", 1, |_| {
        drop(std::mem::take(&mut retained))
    });
    assert_eq!(old.value(selected(run)).unwrap(), old_value);
    println!("STATE,requested_live_bytes,{}", LIVE.load(Relaxed));
}
