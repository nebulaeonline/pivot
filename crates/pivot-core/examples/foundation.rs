//! Optimized comparative harness; opt-in only. See docs/performance.
use pivot_core::experiment_profile as profile;
use pivot_core::{
    evaluation::EvaluationContext,
    interface::{NamedInvocationRequest, PortId},
    model::{
        ModelSnapshot, NodeId, WorkingModel,
        structural::{self, Edit, Graph, PortEdit},
    },
};
use pivot_types::PivotValue;
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
    let profiling = std::env::args().any(|s| s == "--profile");
    TRACK.store(profiling, Relaxed);
    profile::enable(profiling);
    let size = arg("--size", "1000").parse::<usize>().unwrap();
    let degree = arg("--degree", "1").parse::<usize>().unwrap().min(size - 2);
    let run = Run {
        size,
        shape: format!("degree{degree}"),
        implementation: arg("--implementation", "paged"),
        samples: arg("--samples", "31").parse().unwrap(),
    };
    assert!((1..=101).contains(&run.samples));
    println!(
        "HEADER,implementation,cells,shape,operation,samples,p50_ns,p95_ns,worst_ns,allocated_mean,calls_mean,logical_copy_mean,retained_delta,peak_extra"
    );
    let (model, nodes, port) = measure(&run, "construction", 1, |_| {
        structural::benchmark_model(size, degree)
    });
    if run.implementation == "baseline" {
        baseline(&run, model, &nodes, port, degree);
    } else {
        assert_eq!(run.implementation, "paged");
        let mut fixture = Some(model);
        let graph = measure(&run, "compile_paged", 1, |_| {
            Graph::from_model(fixture.as_ref().unwrap(), structural::Limits::default()).unwrap()
        });
        drop(fixture.take());
        paged(&run, graph, &nodes, port, degree);
    }
}
fn commit(m: &mut WorkingModel, edit: impl FnOnce(&mut WorkingModel)) -> ModelSnapshot {
    let mut candidate = m.stage();
    edit(&mut candidate);
    let snapshot = candidate.snapshot().unwrap();
    *m = candidate;
    snapshot
}
fn base_value(snapshot: ModelSnapshot, id: NodeId) -> pivot_types::PivotDecimal {
    EvaluationContext::new(snapshot)
        .evaluate(&[id])
        .unwrap()
        .value(id)
        .unwrap()
        .clone()
        .unwrap()
}
fn node_edit(run: &Run, nodes: &[NodeId], degree: usize, i: usize) -> (NodeId, String) {
    let _ = run;
    let id = nodes[2 + i % degree];
    let input = 1 - (i / degree) % 2;
    (id, format!("n{input}+{}", i + 2))
}
fn baseline(run: &Run, mut model: WorkingModel, nodes: &[NodeId], port: PortId, degree: usize) {
    let old = model.snapshot().unwrap();
    let stale = model.revision();
    let mut other = WorkingModel::new_reference("Other").unwrap();
    let foreign = other.add_input("foreign", 1.into()).unwrap();
    drop(measure(run, "capture", run.samples, |_| {
        model.snapshot().unwrap()
    }));
    drop(measure(run, "input_pipeline", run.samples, |i| {
        let snap = commit(&mut model, |m| {
            m.set_input(nodes[0], (i as i64 + 2).into()).unwrap()
        });
        assert_eq!(base_value(snap.clone(), nodes[2]), (i as i64 + 3).into());
        snap
    }));
    model.set_input(nodes[0], 1.into()).unwrap();
    drop(measure(run, "changing_edge_pipeline", run.samples, |i| {
        let (id, source) = node_edit(run, nodes, degree, i);
        let snap = commit(&mut model, |m| m.set_expression(id, source).unwrap());
        assert_eq!(base_value(snap.clone(), id), (i as i64 + 3).into());
        snap
    }));
    drop(measure(run, "name_add_pipeline", run.samples, |i| {
        let mut id = None;
        let snap = commit(&mut model, |m| {
            id = Some(m.add_input(format!("added{i}"), 2.into()).unwrap())
        });
        assert_eq!(base_value(snap.clone(), id.unwrap()), 2.into());
        snap
    }));
    drop(measure(run, "name_rename_pipeline", run.samples, |i| {
        let snap = commit(&mut model, |m| {
            m.rename(nodes[run.size - 1], format!("renamed{i}"))
                .unwrap()
        });
        black_box(base_value(snap.clone(), nodes[run.size - 1]));
        snap
    }));
    drop(measure(run, "add_retire_pipeline", run.samples, |i| {
        let mut id = None;
        drop(commit(&mut model, |m| {
            id = Some(m.add_input(format!("retired{i}"), 1.into()).unwrap())
        }));
        let snap = commit(&mut model, |m| m.remove(id.unwrap()).unwrap());
        black_box(base_value(snap.clone(), nodes[2]));
        snap
    }));
    drop(measure(run, "formula_add_pipeline", run.samples, |i| {
        let mut id = None;
        let snap = commit(&mut model, |m| {
            id = Some(m.add_computed(format!("formula{i}"), "n0+2").unwrap())
        });
        assert_eq!(base_value(snap.clone(), id.unwrap()), 3.into());
        snap
    }));
    drop(measure(run, "draft_repair_pipeline", run.samples, |_| {
        let mut draft = model.stage();
        draft.set_expression(nodes[2], "missing+1").unwrap();
        model = draft;
        let snap = commit(&mut model, |m| m.set_expression(nodes[2], "n0+5").unwrap());
        assert_eq!(base_value(snap.clone(), nodes[2]), 6.into());
        snap
    }));
    drop(measure(run, "port_update_invoke", run.samples, |i| {
        let snap = commit(&mut model, |m| {
            m.update_output_port(port, nodes[i % 2]).unwrap()
        });
        let context = EvaluationContext::new(snap.clone());
        let r = context.invoke_named(&request()).unwrap();
        assert_eq!(
            r.outputs()["out"],
            Ok(PivotValue::Decimal(if i % 2 == 0 {
                5.into()
            } else {
                7.into()
            }))
        );
        snap
    }));
    measure(run, "candidate_reject_unknown", run.samples, |_| {
        let revision = model.revision();
        let mut candidate = model.stage();
        candidate.set_input(nodes[0], 99.into()).unwrap();
        assert!(candidate.set_input(foreign, 99.into()).is_err());
        drop(candidate);
        assert_eq!(model.revision(), revision);
    });
    measure(run, "stale_reject", run.samples, |_| {
        assert_ne!(model.revision(), stale)
    });
    let mut retained = Vec::new();
    measure(run, "retain_32_structural", 1, |_| {
        for i in 0..32 {
            retained.push(commit(&mut model, |m| {
                m.set_expression(nodes[2], format!("n0+{}", i + 200))
                    .unwrap()
            }));
        }
    });
    assert_eq!(base_value(retained[0].clone(), nodes[2]), 201.into());
    measure(run, "cleanup_32_structural", 1, |_| {
        drop(std::mem::take(&mut retained))
    });
    measure(run, "raw_churn_1024", 1, |_| {
        for _ in 0..1024 {
            let id = model.add_input("churn", 1.into()).unwrap();
            model.remove(id).unwrap();
        }
    });
    assert_eq!(base_value(old.clone(), nodes[2]), 2.into());
    let mut final_owners = Some((model, old));
    measure(run, "last_owner_inline", 1, |_| drop(final_owners.take()));
}
fn request() -> NamedInvocationRequest {
    NamedInvocationRequest {
        inputs: vec![("required".into(), PivotValue::Decimal(5.into()))],
        outputs: vec!["out".into()],
    }
}
fn paged(run: &Run, mut graph: Graph, nodes: &[NodeId], port: PortId, degree: usize) {
    let old = graph.capture().unwrap();
    let memory = graph.memory();
    let mut other = WorkingModel::new_reference("Other").unwrap();
    let foreign = other.add_input("foreign", 1.into()).unwrap();
    drop(measure(run, "capture", run.samples, |_| {
        graph.capture().unwrap()
    }));
    drop(measure(run, "input_pipeline", run.samples, |i| {
        graph
            .apply(
                graph.version(),
                vec![Edit::Input(nodes[0], (i as i64 + 2).into())],
            )
            .unwrap();
        let snap = graph.capture().unwrap();
        assert_eq!(
            snap.evaluate(&[nodes[2]]).unwrap()[&nodes[2]],
            Ok((i as i64 + 3).into())
        );
        snap
    }));
    graph
        .apply(graph.version(), vec![Edit::Input(nodes[0], 1.into())])
        .unwrap();
    drop(measure(run, "changing_edge_pipeline", run.samples, |i| {
        let (id, source) = node_edit(run, nodes, degree, i);
        graph
            .apply(graph.version(), vec![Edit::Formula(id, source)])
            .unwrap();
        let snap = graph.capture().unwrap();
        assert_eq!(
            snap.evaluate(&[id]).unwrap()[&id],
            Ok((i as i64 + 3).into())
        );
        snap
    }));
    drop(measure(run, "name_add_pipeline", run.samples, |i| {
        let name = format!("added{i}");
        graph
            .apply(
                graph.version(),
                vec![Edit::AddInput(name.clone(), 2.into())],
            )
            .unwrap();
        let id = graph.node_id(&name).unwrap();
        let snap = graph.capture().unwrap();
        assert_eq!(snap.evaluate(&[id]).unwrap()[&id], Ok(2.into()));
        snap
    }));
    drop(measure(run, "name_rename_pipeline", run.samples, |i| {
        graph
            .apply(
                graph.version(),
                vec![Edit::Rename(nodes[run.size - 1], format!("renamed{i}"))],
            )
            .unwrap();
        let snap = graph.capture().unwrap();
        black_box(snap.evaluate(&[nodes[run.size - 1]]).unwrap());
        snap
    }));
    drop(measure(run, "add_retire_pipeline", run.samples, |i| {
        let name = format!("retired{i}");
        graph
            .apply(
                graph.version(),
                vec![Edit::AddInput(name.clone(), 1.into())],
            )
            .unwrap();
        drop(graph.capture().unwrap());
        let id = graph.node_id(&name).unwrap();
        graph
            .apply(graph.version(), vec![Edit::Retire(id)])
            .unwrap();
        let snap = graph.capture().unwrap();
        black_box(snap.evaluate(&[nodes[2]]).unwrap());
        snap
    }));
    drop(measure(run, "formula_add_pipeline", run.samples, |i| {
        let name = format!("formula{i}");
        graph
            .apply(
                graph.version(),
                vec![Edit::AddFormula(name.clone(), "n0+2".into())],
            )
            .unwrap();
        let id = graph.node_id(&name).unwrap();
        let snap = graph.capture().unwrap();
        assert_eq!(snap.evaluate(&[id]).unwrap()[&id], Ok(3.into()));
        snap
    }));
    drop(measure(run, "draft_repair_pipeline", run.samples, |_| {
        graph
            .apply(
                graph.version(),
                vec![Edit::Formula(nodes[2], "missing+1".into())],
            )
            .unwrap();
        graph
            .apply(
                graph.version(),
                vec![Edit::Formula(nodes[2], "n0+5".into())],
            )
            .unwrap();
        let snap = graph.capture().unwrap();
        assert_eq!(snap.evaluate(&[nodes[2]]).unwrap()[&nodes[2]], Ok(6.into()));
        snap
    }));
    drop(measure(run, "port_update_invoke", run.samples, |i| {
        graph
            .apply(
                graph.version(),
                vec![Edit::Port(PortEdit::Output(port, nodes[i % 2]))],
            )
            .unwrap();
        let snap = graph.capture().unwrap();
        let r = snap.invoke_named(&request()).unwrap();
        assert_eq!(
            r.outputs()["out"],
            Ok(PivotValue::Decimal(if i % 2 == 0 {
                5.into()
            } else {
                7.into()
            }))
        );
        snap
    }));
    measure(run, "candidate_reject_unknown", run.samples, |_| {
        let v = graph.version();
        let charged = memory.stats().charged;
        assert!(
            graph
                .apply(
                    v,
                    vec![
                        Edit::Input(nodes[0], 99.into()),
                        Edit::Input(foreign, 99.into())
                    ]
                )
                .is_err()
        );
        assert_eq!(graph.version(), v);
        assert_eq!(memory.stats().charged, charged);
    });
    measure(run, "stale_reject", run.samples, |_| {
        assert_eq!(
            graph.apply(old.version(), vec![Edit::Input(nodes[0], 99.into())]),
            Err(structural::Error::Stale)
        )
    });
    let held = memory
        .reserve(memory.stats().limit - memory.stats().charged - 1024)
        .unwrap();
    measure(run, "admission_reject", run.samples, |_| {
        assert_eq!(
            graph.apply(graph.version(), vec![Edit::Input(nodes[0], 99.into())]),
            Err(structural::Error::Admission)
        )
    });
    drop(held);
    let mut retained = Vec::new();
    measure(run, "retain_32_structural", 1, |_| {
        for i in 0..32 {
            graph
                .apply(
                    graph.version(),
                    vec![Edit::Formula(nodes[2], format!("n0+{}", i + 200))],
                )
                .unwrap();
            retained.push(graph.capture().unwrap());
        }
    });
    assert_eq!(
        retained[0].evaluate(&[nodes[2]]).unwrap()[&nodes[2]],
        Ok(201.into())
    );
    measure(run, "cleanup_32_structural", 1, |_| {
        drop(std::mem::take(&mut retained))
    });
    measure(run, "raw_churn_1024", 1, |_| {
        for _ in 0..1024 {
            graph
                .apply(
                    graph.version(),
                    vec![Edit::AddInput("churn".into(), 1.into())],
                )
                .unwrap();
            let id = graph.node_id("churn").unwrap();
            graph
                .apply(graph.version(), vec![Edit::Retire(id)])
                .unwrap();
        }
    });
    assert_eq!(old.evaluate(&[nodes[2]]).unwrap()[&nodes[2]], Ok(2.into()));
    let stats = memory.stats();
    println!(
        "MEMORY,allocated,{},reserved,{},admission_peak,{}",
        stats.allocated, stats.reserved, stats.peak
    );
    let worker = structural::reclaim::Reclaimer::new(memory.clone(), 4).unwrap();
    let mut owners = Some((graph, old));
    measure(run, "last_owner_enqueue", 1, |_| {
        assert!(worker.retire(owners.take().unwrap()).is_ok())
    });
    let mut worker = Some(worker);
    measure(run, "last_owner_worker_join", 1, |_| drop(worker.take()));
    assert_eq!(memory.stats().charged, 0);
}
