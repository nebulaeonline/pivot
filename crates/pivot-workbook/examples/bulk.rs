use pivot_core::model::{
    NodeId, WorkingModel,
    integration::{CalculationCapture, IntegrationRuntime},
};
use pivot_workbook::{DecimalInput, Edit, Limits, Position, Rectangle, Workbook};
use std::{
    alloc::{GlobalAlloc, Layout, System},
    sync::atomic::{AtomicUsize, Ordering::Relaxed},
    time::Instant,
};
static LIVE: AtomicUsize = AtomicUsize::new(0);
static TOTAL: AtomicUsize = AtomicUsize::new(0);
static CALLS: AtomicUsize = AtomicUsize::new(0);
static PEAK: AtomicUsize = AtomicUsize::new(0);
struct Counting;
fn allocated(n: usize) {
    pivot_core::experiment_profile::construction::allocation(n);
    TOTAL.fetch_add(n, Relaxed);
    CALLS.fetch_add(1, Relaxed);
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

fn dispatches(runtime: &IntegrationRuntime) -> u64 {
    let s = runtime.stats();
    s.mutations.accepted + s.heavy.accepted
}
fn measure(name: &str, mut f: impl FnMut(usize)) {
    for i in 0..3 {
        f(i);
    }
    let bytes = TOTAL.load(Relaxed);
    let calls = CALLS.load(Relaxed);
    let mut times = Vec::new();
    for i in 3..34 {
        let start = Instant::now();
        f(i);
        times.push(start.elapsed().as_nanos());
    }
    times.sort_unstable();
    println!(
        "ROW,{name},31,{},{},{},{},{}",
        times[15],
        times[29],
        times[30],
        (TOTAL.load(Relaxed) - bytes) / 31,
        (CALLS.load(Relaxed) - calls) / 31
    );
}
fn formula(n: usize, i: usize, offset: usize, shape: &str) -> Option<String> {
    match shape {
        "mixed" if i % 5 == 4 => Some("n0+1".into()),
        "fanout" if i > 0 => Some("n0+1".into()),
        "long-chain" if i > 0 => Some(format!("n{}+1", if i == 1 { 0 } else { offset + i - 1 })),
        "chain" if i >= n - 1000.min(n - 1) => Some(format!(
            "n{}+1",
            if i == n - 1000.min(n - 1) {
                0
            } else {
                offset + i - 1
            }
        )),
        _ => None,
    }
}
fn depth(n: usize, shape: &str) -> i64 {
    match shape {
        "chain" => 1000.min(n - 1) as i64,
        "long-chain" => (n - 1) as i64,
        "mixed" | "fanout" => 1,
        _ => 0,
    }
}
fn populate(
    model: &mut WorkingModel,
    n: usize,
    offset: usize,
    backend: &str,
    shape: &str,
) -> Vec<NodeId> {
    if backend == "batch" {
        let formulas = match shape {
            "mixed" => n / 5,
            "chain" => 1000.min(n - 1),
            "long-chain" => n - 1,
            "fanout" => n - 1,
            _ => 0,
        };
        let mut batch = model.begin_batch(n + formulas, n * 64).unwrap();
        let nodes = (0..n)
            .map(|i| {
                batch
                    .add_input(format!("n{}", offset + i), 1.into())
                    .unwrap()
            })
            .collect::<Vec<_>>();
        for (i, &node) in nodes.iter().enumerate() {
            if let Some(source) = formula(n, i, offset, shape) {
                batch.set_expression(node, source).unwrap();
            }
        }
        model.apply_batch(batch, &Default::default()).unwrap();
        nodes
    } else {
        let nodes = (0..n)
            .map(|i| {
                model
                    .add_input(format!("n{}", offset + i), 1.into())
                    .unwrap()
            })
            .collect::<Vec<_>>();
        for (i, &node) in nodes.iter().enumerate() {
            if let Some(source) = formula(n, i, offset, shape) {
                model.set_expression(node, source).unwrap();
            }
        }
        nodes
    }
}
fn check(capture: &CalculationCapture, out: NodeId, value: i64) {
    assert_eq!(
        capture.evaluate(&[out]).unwrap().values()[&out],
        Ok(value.into())
    );
}
fn core(
    runtime: &IntegrationRuntime,
    n: usize,
    backend: &str,
    shape: &str,
    existing: bool,
    samples: usize,
) {
    let mut final_model = None;
    for sample in 0..samples {
        drop(final_model.take());
        runtime.flush();
        let mut model = runtime.create_model("bulk").unwrap();
        let old = if existing {
            populate(&mut model, n, 0, "batch", "independent");
            Some(model.capture_local().unwrap())
        } else {
            None
        };
        let dispatch = dispatches(runtime);
        let bytes = TOTAL.load(Relaxed);
        let calls = CALLS.load(Relaxed);
        let start = Instant::now();
        let nodes = populate(&mut model, n, if existing { n } else { 0 }, backend, shape);
        let mutation = start.elapsed().as_nanos();
        let root = model.node_id("n0").unwrap();
        let out = if shape == "independent" {
            root
        } else {
            nodes[n - 1]
        };
        let begin = Instant::now();
        let capture = model.capture_calculation().unwrap();
        let captured = begin.elapsed().as_nanos();
        let begin = Instant::now();
        check(&capture, out, 1 + depth(n, shape));
        let evaluated = begin.elapsed().as_nanos();
        println!(
            "CONSTRUCT,{sample},{mutation},{captured},{evaluated},{},{},{},{}",
            start.elapsed().as_nanos(),
            TOTAL.load(Relaxed) - bytes,
            CALLS.load(Relaxed) - calls,
            dispatches(runtime) - dispatch
        );
        if let Some(old) = old {
            check(&old, root, 1);
        }
        drop(capture);
        final_model = Some((model, root, out));
    }
    let (mut model, root, out) = final_model.take().unwrap();
    println!(
        "POST_CONSTRUCTION_NODES,{}",
        n * if existing { 2 } else { 1 }
    );
    measure("input_capture", |i| {
        let mut next = model.stage();
        next.set_input(root, (i as i64).into()).unwrap();
        let c = next.capture_local().unwrap();
        model = next;
        drop(c);
    });
    measure("input_chain", |i| {
        let mut next = model.stage();
        next.set_input(root, (i as i64).into()).unwrap();
        let c = next.capture_local().unwrap();
        check(&c, out, i as i64 + depth(n, shape));
        model = next;
    });
    let scratch = model.add_computed("scratch", "n0+1").unwrap();
    measure("formula_edit", |i| {
        let mut next = model.stage();
        next.set_expression(scratch, format!("n0+{}", i + 1))
            .unwrap();
        check(&next.capture_local().unwrap(), scratch, 33 + i as i64 + 1);
        model = next;
    });
    {
        measure("bounded_inspection", |_| {
            let v = model.inspect().unwrap();
            assert!(!v.nodes(None, 256, 4096).unwrap().nodes().is_empty());
        });
    }
    model.set_input(root, 1.into()).unwrap();
    runtime.flush();
    let initial = LIVE.load(Relaxed);
    let mut retained = Vec::new();
    for i in 0..32 {
        model.set_expression(scratch, format!("n0+{i}")).unwrap();
        retained.push((model.capture_local().unwrap(), i as i64 + 1));
        if i == 15 || i == 31 {
            runtime.flush();
            println!(
                "RETAIN,{},{}",
                i + 1,
                LIVE.load(Relaxed).saturating_sub(initial)
            );
        }
    }
    for (c, value) in &retained {
        check(c, scratch, *value);
    }
    let start = Instant::now();
    drop(retained);
    let enqueue = start.elapsed().as_nanos();
    let start = Instant::now();
    runtime.flush();
    println!("RETAIN_DROP,{enqueue},{}", start.elapsed().as_nanos());
    let start = Instant::now();
    drop(model);
    println!("ROOT_DROP,{}", start.elapsed().as_nanos());
}
fn position(i: usize) -> Position {
    Position::new((i / 100 * 2 + 1) as u32, (i % 100 * 2 + 1) as u32)
}
fn wb_formula(n: usize, i: usize, shape: &str) -> Option<String> {
    formula(n, i, 0, shape).map(|source| {
        let id: usize = source
            .strip_prefix('n')
            .unwrap()
            .strip_suffix("+1")
            .unwrap()
            .parse()
            .unwrap();
        format!("={}+1", address(position(id)))
    })
}
fn input(i: usize, value: u32) -> Edit {
    Edit::SetDecimal {
        position: position(i),
        value: DecimalInput::from_parts(value, 0, 0, false, 0).unwrap(),
    }
}
fn workbook(runtime: &IntegrationRuntime, n: usize, _backend: &str, shape: &str, samples: usize) {
    let limits = Limits {
        rows: ((n + 60000) / 100 * 2 + 10) as u32,
        columns: 202,
        live_cells: n + 50000,
        allocated_cells: n + 60000,
        edits: 262144,
        bytes: 512 * 1024 * 1024,
        ..Limits::default()
    };
    let mut final_book = None;
    for sample in 0..samples {
        drop(final_book.take());
        runtime.flush();
        let dispatch = dispatches(runtime);
        let bytes = TOTAL.load(Relaxed);
        let calls = CALLS.load(Relaxed);
        let profiling = std::env::var_os("PIVOT_CONSTRUCTION_PROFILE").is_some();
        if profiling {
            pivot_core::experiment_profile::construction::start();
        }
        let start = Instant::now();
        let request_profile = pivot_core::experiment_profile::construction::scope(1);
        let mut book = Workbook::new_persistent_experiment("bulk", limits, runtime).unwrap();
        let edits = (0..n)
            .map(|i| {
                if let Some(source) = wb_formula(n, i, shape) {
                    Edit::SetFormulaText {
                        position: position(i),
                        source,
                    }
                } else {
                    input(i, 1)
                }
            })
            .collect::<Vec<_>>();
        drop(request_profile);
        book.apply(book.expected(), &edits).unwrap();
        let mutation = start.elapsed().as_nanos();
        if profiling {
            runtime.flush();
            for (name, ns, bytes, calls) in pivot_core::experiment_profile::construction::stop() {
                println!("PROFILE,{sample},{name},{ns},{bytes},{calls}");
            }
        }
        let begin = Instant::now();
        let capture = book.capture(book.expected()).unwrap();
        let captured = begin.elapsed().as_nanos();
        let output = capture
            .cell_at(position(if shape == "independent" { 0 } else { n - 1 }))
            .unwrap();
        let begin = Instant::now();
        assert_eq!(
            capture.evaluate(&[output]).unwrap().values[0].1,
            Ok((1 + depth(n, shape)).into())
        );
        let evaluated = begin.elapsed().as_nanos();
        println!(
            "CONSTRUCT,{sample},{mutation},{captured},{evaluated},{},{},{},{}",
            start.elapsed().as_nanos(),
            TOTAL.load(Relaxed) - bytes,
            CALLS.load(Relaxed) - calls,
            dispatches(runtime) - dispatch
        );
        drop((capture, edits));
        final_book = Some((book, output));
    }
    let (mut book, out) = final_book.take().unwrap();
    let old = book.capture_local(book.expected()).unwrap();
    let mut extra = n;
    for size in [100usize, 1000, 10000] {
        for sample in 0..3 {
            let edits = (extra..extra + size)
                .map(|i| input(i, 7))
                .collect::<Vec<_>>();
            extra += size;
            let dispatch = dispatches(runtime);
            let bytes = TOTAL.load(Relaxed);
            let calls = CALLS.load(Relaxed);
            let start = Instant::now();
            book.apply(book.expected(), &edits).unwrap();
            let capture = book.capture_local(book.expected()).unwrap();
            assert_eq!(
                capture.evaluate(&[out]).unwrap().values[0].1,
                Ok((1 + depth(n, shape)).into())
            );
            println!(
                "ADD,{size},{sample},{},{},{},{}",
                start.elapsed().as_nanos(),
                TOTAL.load(Relaxed) - bytes,
                CALLS.load(Relaxed) - calls,
                dispatches(runtime) - dispatch
            );
        }
    }
    for size in [100usize, 1000, 10000] {
        for sample in 0..3 {
            let edits = (n..n + size)
                .map(|i| input(i, 9 + sample))
                .collect::<Vec<_>>();
            let count = dispatches(runtime);
            let start = Instant::now();
            book.apply(book.expected(), &edits).unwrap();
            let c = book.capture_local(book.expected()).unwrap();
            let target = c.cell_at(position(n)).unwrap();
            assert_eq!(
                c.evaluate(&[target]).unwrap().values[0].1,
                Ok((9 + sample as i64).into())
            );
            println!(
                "UPDATE,{size},{sample},{},{}",
                start.elapsed().as_nanos(),
                dispatches(runtime) - count
            );
        }
    }
    let before_formula = book.capture_local(book.expected()).unwrap();
    let target = before_formula.cell_at(position(n)).unwrap();
    for sample in 0..3 {
        let edits = (n..n + 10000)
            .map(|i| Edit::SetFormulaText {
                position: position(i),
                source: format!("=A1+{}", sample + 3),
            })
            .collect::<Vec<_>>();
        let count = dispatches(runtime);
        let start = Instant::now();
        book.apply(book.expected(), &edits).unwrap();
        let c = book.capture_local(book.expected()).unwrap();
        assert_eq!(
            c.evaluate(&[target]).unwrap().values[0].1,
            Ok((sample as i64 + 4).into())
        );
        println!(
            "FORMULAS,10000,{sample},{},{}",
            start.elapsed().as_nanos(),
            dispatches(runtime) - count
        );
    }
    assert_eq!(
        before_formula.evaluate(&[target]).unwrap().values[0].1,
        Ok(11.into())
    );
    drop(before_formula);
    println!("POST_ADDITION_CELLS,{}", book.stats().live_cells);
    measure("input_capture", |i| {
        book.apply(book.expected(), &[input(0, i as u32)]).unwrap();
        drop(book.capture_local(book.expected()).unwrap());
    });
    measure("input_chain", |i| {
        book.apply(book.expected(), &[input(0, i as u32)]).unwrap();
        let c = book.capture_local(book.expected()).unwrap();
        assert_eq!(
            c.evaluate(&[out]).unwrap().values[0].1,
            Ok((i as i64 + depth(n, shape)).into())
        );
    });
    let scratch = position(extra);
    book.apply(
        book.expected(),
        &[Edit::SetFormulaText {
            position: scratch,
            source: "=A1+1".into(),
        }],
    )
    .unwrap();
    let scratch_ref = book.cell_at(scratch).unwrap().unwrap().reference;
    measure("formula_edit", |i| {
        book.apply(
            book.expected(),
            &[Edit::SetFormulaText {
                position: scratch,
                source: format!("=A1+{}", i + 1),
            }],
        )
        .unwrap();
        assert_eq!(
            book.capture_local(book.expected())
                .unwrap()
                .evaluate(&[scratch_ref])
                .unwrap()
                .values[0]
                .1,
            Ok((33 + i as i64 + 1).into())
        );
    });
    measure("bounded_inspection", |_| {
        let view = book
            .read(
                book.expected(),
                Rectangle {
                    first: Position::new(1, 1),
                    last: Position::new(63, 63),
                },
                1024,
            )
            .unwrap();
        assert!(!view.cells.is_empty());
    });
    assert_eq!(
        old.evaluate(&[out]).unwrap().values[0].1,
        Ok((1 + depth(n, shape)).into())
    );
    drop(old);
    book.apply(book.expected(), &[input(0, 1)]).unwrap();
    runtime.flush();
    let initial = LIVE.load(Relaxed);
    let mut held = Vec::new();
    for i in 0..16 {
        book.apply(
            book.expected(),
            &[Edit::SetFormulaText {
                position: scratch,
                source: format!("=A1+{i}"),
            }],
        )
        .unwrap();
        held.push((book.capture_local(book.expected()).unwrap(), i as i64 + 1));
    }
    runtime.flush();
    println!("RETAIN,16,{}", LIVE.load(Relaxed).saturating_sub(initial));
    for (c, value) in &held {
        assert_eq!(
            c.evaluate(&[scratch_ref]).unwrap().values[0].1,
            Ok((*value).into())
        );
    }
    let start = Instant::now();
    drop(held);
    let enqueue = start.elapsed().as_nanos();
    let start = Instant::now();
    runtime.flush();
    println!("RETAIN_DROP,{enqueue},{}", start.elapsed().as_nanos());
    let start = Instant::now();
    drop(book);
    println!("ROOT_DROP,{}", start.elapsed().as_nanos());
}
fn main() {
    let args = std::env::args().collect::<Vec<_>>();
    let n: usize = args[1].parse().unwrap();
    let backend = &args[2];
    let shape = &args[3];
    let layer = &args[4];
    let samples: usize = args[5].parse().unwrap();
    assert!(samples > 0 && (layer == "small" || (n >= 1000 && n % 100 == 0)));
    assert!(["independent", "mixed", "chain", "fanout", "long-chain"].contains(&shape.as_str()));
    assert!(if layer == "workbook" {
        ["persistent"].contains(&backend.as_str())
    } else {
        ["individual", "batch"].contains(&backend.as_str())
    });
    let runtime = IntegrationRuntime::standalone().unwrap();
    if layer == "small" {
        small(&runtime, n);
    } else if layer == "workbook" {
        workbook(&runtime, n, backend, shape, samples)
    } else {
        core(&runtime, n, backend, shape, layer == "existing", samples)
    }
    let start = Instant::now();
    runtime.flush();
    let drain = start.elapsed().as_nanos();
    let memory = runtime.memory();
    let stats = runtime.stats();
    println!(
        "HEAP,{},{},{}",
        LIVE.load(Relaxed),
        PEAK.load(Relaxed),
        memory.stats().peak
    );
    println!(
        "RUNTIME,{},{},{},{},{},{},{},{},{},{}",
        stats.mutations.peak_queued,
        stats.mutations.accepted,
        stats.mutations.rejected,
        stats.heavy.peak_queued,
        stats.heavy.accepted,
        stats.heavy.rejected,
        stats.completed_cleanup,
        stats.total_drop_ns,
        stats.max_drop_ns,
        stats.peak_pending_cleanup
    );
    let start = Instant::now();
    assert!(runtime.finish().is_ok());
    assert_eq!(memory.stats().charged, 0);
    println!(
        "FINAL,{drain},{},{}",
        start.elapsed().as_nanos(),
        memory.stats().charged
    );
}

pub fn address(pos: Position) -> String {
    let mut c = pos.column;
    let mut letters = Vec::new();
    while c > 0 {
        c -= 1;
        letters.push((b'A' + (c % 26) as u8) as char);
        c /= 26;
    }
    format!(
        "{}{}",
        letters.into_iter().rev().collect::<String>(),
        pos.row
    )
}

fn small(runtime: &IntegrationRuntime, n: usize) {
    assert!((1..=32).contains(&n));
    measure("small_model_create_capture_evaluate_drop", |_| {
        let mut model = WorkingModel::new_in("small", runtime).unwrap();
        let mut output = model.add_input("x", 3.into()).unwrap();
        for i in 1..n {
            output = model
                .add_computed(
                    format!("n{i}"),
                    if i == 1 {
                        "x+1".to_string()
                    } else {
                        format!("n{}+1", i - 1)
                    },
                )
                .unwrap();
        }
        let capture = model.capture_local().unwrap();
        assert_eq!(
            capture.evaluate(&[output]).unwrap().values()[&output],
            Ok((n as i64 + 2).into())
        );
    });
}
