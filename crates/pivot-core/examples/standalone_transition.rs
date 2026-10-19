//! Production-facade comparison; no research Graph API used here.
use pivot_core::model::{NodeId, WorkingModel, integration::IntegrationRuntime};
use pivot_core::{
    interface::{InputRequirement, NamedInvocationRequest},
    model::{
        integration::{CalculationFailure, CancellationToken},
        transfer,
    },
};
use pivot_types::PivotValue;
use std::sync::{Arc, atomic::AtomicBool};
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

fn measure(name: &str, samples: usize, mut f: impl FnMut(usize)) {
    for i in 0..3 {
        f(i);
    }
    let mut values = Vec::with_capacity(samples);
    let bytes = TOTAL.load(Relaxed);
    let calls = CALLS.load(Relaxed);
    for i in 0..samples {
        let start = Instant::now();
        f(i + 3);
        values.push(start.elapsed().as_nanos());
    }
    let bytes = TOTAL.load(Relaxed) - bytes;
    let calls = CALLS.load(Relaxed) - calls;
    values.sort_unstable();
    let p95 = values[(samples * 95).div_ceil(100) - 1];
    println!(
        "ROW,{name},{samples},{},{},{},{},{}",
        values[samples / 2],
        p95,
        values[samples - 1],
        bytes / samples,
        calls / samples
    );
}
fn check(model: &WorkingModel, out: NodeId, expected: i64) {
    let capture = model.capture_local().unwrap();
    let result = capture.evaluate(&[out]).unwrap();
    assert_eq!(result.values()[&out], Ok(expected.into()));
}
fn invalid(model: &WorkingModel, out: NodeId) {
    let capture = model.capture_local().unwrap();
    assert!(matches!(
        capture.evaluate(&[out]),
        Err(CalculationFailure::LocalInvalid(_))
    ));
}
fn fixture(
    size: usize,
    name: &str,
    runtime: Option<&IntegrationRuntime>,
) -> (WorkingModel, Vec<NodeId>, usize) {
    let mut model = match runtime {
        Some(r) => WorkingModel::new_in(name, r),
        None => WorkingModel::new_reference(name),
    }
    .unwrap();
    let nodes: Vec<_> = (0..size)
        .map(|i| model.add_input(format!("n{i}"), 1.into()).unwrap())
        .collect();
    let count = 1000.min(size - 3);
    let start = size - count;
    for (index, &id) in nodes.iter().enumerate().skip(start) {
        let dependency = if index == start { 0 } else { index - 1 };
        model
            .set_expression(id, format!("n{dependency}+1"))
            .unwrap();
    }
    (model, nodes, count)
}
fn main() {
    let args: Vec<_> = std::env::args().collect();
    let size: usize = args[1].parse().unwrap();
    let integrated = match args[2].as_str() {
        "integrated" => true,
        "reference" => false,
        _ => panic!("backend"),
    };
    let samples: usize = args[3].parse().unwrap();
    let scenario = &args[4];
    assert!(size >= 1000 && samples > 0);
    let runtime = IntegrationRuntime::new(2 * 1024 * 1024 * 1024, 64).unwrap();
    let small_start = Instant::now();
    let mut small = WorkingModel::new("Independent small default").unwrap();
    assert_eq!(small.backend(), pivot_core::model::BackendKind::Persistent);
    let small_input = small.add_input("x", 4.into()).unwrap();
    let small_output = small.add_computed("y", "x*2+1").unwrap();
    check(&small, small_output, 9);
    assert!(small.contains_node(small_input));
    drop(small);
    println!("SMALL_DEFAULT_NS,{}", small_start.elapsed().as_nanos());
    let small_cleanup = Instant::now();
    let small_memory = IntegrationRuntime::with_convenience(|r| {
        r.flush();
        r.memory()
    })
    .unwrap();
    IntegrationRuntime::shutdown_convenience().unwrap();
    assert_eq!(small_memory.stats().charged, 0);
    println!(
        "SMALL_CLEANUP_NS,{},{}",
        small_cleanup.elapsed().as_nanos(),
        small_memory.stats().charged
    );
    let construction = Instant::now();
    let (mut model, mut nodes, count) = fixture(
        size,
        "Standalone default benchmark",
        integrated.then_some(&runtime),
    );
    println!("CONSTRUCTION_NS,{}", construction.elapsed().as_nanos());
    let out = nodes[size - 1];
    let start = size - count;
    let required = model
        .add_input_port("required", nodes[0], InputRequirement::Required)
        .unwrap();
    let optional = model
        .add_input_port(
            "optional",
            nodes[1],
            InputRequirement::Optional {
                default: PivotValue::Decimal(7.into()),
            },
        )
        .unwrap();
    let output = model.add_output_port("result", out).unwrap();
    let optional_output = model.add_output_port("optional_value", nodes[1]).unwrap();
    let mut ports = vec![required, optional, output, optional_output];
    if scenario == "ports-wide" {
        for &node in nodes.iter().take(start.min(1000)).skip(2) {
            let index = ports.len();
            ports.push(
                model
                    .add_input_port(
                        format!("wide{index}"),
                        node,
                        InputRequirement::Optional {
                            default: PivotValue::Decimal(7.into()),
                        },
                    )
                    .unwrap(),
            );
        }
    }
    if scenario == "invalid-import" {
        model.set_expression(out, "(").unwrap();
    }
    let timer = Instant::now();
    assert_eq!(
        model.backend(),
        if integrated {
            pivot_core::model::BackendKind::Persistent
        } else {
            pivot_core::model::BackendKind::Reference
        }
    );
    runtime.flush();
    println!("POST_CONSTRUCTION_DRAIN_NS,{}", timer.elapsed().as_nanos());
    let mut input = 1i64;
    let mut increments = vec![1i64; count];
    if scenario == "edits" {
        measure("unchanged_capture", samples, |_| {
            drop(model.capture_local().unwrap())
        });
        measure("input_capture", samples, |i| {
            let mut next = model.stage_expected(model.revision()).unwrap();
            next.set_input(nodes[0], (i as i64).into()).unwrap();
            let capture = next.capture_local().unwrap();
            model = next;
            drop(capture);
            input = i as i64;
        });
        measure("input_chain", samples, |i| {
            let mut next = model.stage_expected(model.revision()).unwrap();
            next.set_input(nodes[0], (i as i64).into()).unwrap();
            check(&next, out, i as i64 + increments.iter().sum::<i64>());
            model = next;
            input = i as i64;
        });
        for (label, position) in [
            ("formula_front", 0),
            ("formula_middle", count / 2),
            ("formula_tail", count - 1),
        ] {
            measure(label, samples, |i| {
                let index = start + position;
                let previous = if position == 0 { 0 } else { index - 1 };
                let increment = 1 + (i % 2) as i64;
                let mut next = model.stage();
                next.set_expression(nodes[index], format!("n{previous}+{increment}"))
                    .unwrap();
                increments[position] = increment;
                check(&next, out, input + increments.iter().sum::<i64>());
                model = next;
            });
        }
        measure("rename", samples, |i| {
            let mut next = model.stage();
            next.rename(nodes[2], format!("label{i}")).unwrap();
            check(&next, out, input + increments.iter().sum::<i64>());
            model = next;
        });
        measure("add_retire", samples, |i| {
            let mut next = model.stage();
            let id = next.add_input(format!("retired{i}"), 1.into()).unwrap();
            next.remove(id).unwrap();
            check(&next, out, input + increments.iter().sum::<i64>());
            model = next;
            nodes.push(id);
        });
        measure("invalid_draft_repair", samples, |i| {
            let mut next = model.stage();
            next.set_expression(out, "(").unwrap();
            invalid(&next, out);
            let increment = 1 + (i % 2) as i64;
            next.set_expression(out, format!("n{}+{increment}", size - 2))
                .unwrap();
            increments[count - 1] = increment;
            check(&next, out, input + increments.iter().sum::<i64>());
            model = next;
        });
        measure("named_add_formula", samples, |i| {
            let mut next = model.stage();
            let a = next
                .add_input(format!("extra{i}"), (i as i64).into())
                .unwrap();
            let b = next
                .add_computed(format!("derived{i}"), format!("extra{i}*2+1"))
                .unwrap();
            check(&next, b, 2 * i as i64 + 1);
            model = next;
            nodes.extend([a, b]);
        });
        for keep in [16, 32] {
            runtime.flush();
            let before = LIVE.load(Relaxed);
            let mut captures = Vec::with_capacity(keep);
            let timer = Instant::now();
            for i in 0..keep {
                let mut next = model.stage();
                next.set_expression(out, format!("n{}+{}", size - 2, 1 + i % 2))
                    .unwrap();
                captures.push(next.capture_local().unwrap());
                model = next;
            }
            let elapsed = timer.elapsed().as_nanos();
            runtime.flush();
            let retained = LIVE.load(Relaxed).saturating_sub(before);
            let timer = Instant::now();
            drop(captures);
            let enqueue = timer.elapsed().as_nanos();
            let timer = Instant::now();
            runtime.flush();
            println!(
                "RETAIN,{keep},{elapsed},{retained},{enqueue},{}",
                timer.elapsed().as_nanos()
            );
        }
    } else if scenario == "ports-wide" {
        let width = model.interface().inputs().count();
        println!("PORT_WIDTH,{width}");
        measure("port_update_invoke", samples, |i| {
            let mut next = model.stage();
            next.patch_input_port(
                optional,
                None,
                Some(InputRequirement::Optional {
                    default: PivotValue::Decimal((i as i64).into()),
                }),
            )
            .unwrap();
            let result = next
                .capture_local()
                .unwrap()
                .invoke_named(&NamedInvocationRequest {
                    inputs: vec![("required".into(), PivotValue::Decimal(5.into()))],
                    outputs: vec!["result".into(), "optional_value".into()],
                })
                .unwrap();
            assert_eq!(
                result.outputs()["result"],
                Ok(PivotValue::Decimal((5 + count as i64).into()))
            );
            assert_eq!(
                result.outputs()["optional_value"],
                Ok(PivotValue::Decimal((i as i64).into()))
            );
            assert_eq!(result.inputs().len(), width);
            model = next;
        });
        measure("port_create_rename_remove", samples, |i| {
            let mut next = model.stage();
            let port = next.add_output_port(format!("temporary{i}"), out).unwrap();
            next.rename_port(port, format!("renamed{i}")).unwrap();
            next.update_output_port(port, nodes[0]).unwrap();
            next.remove_port(port).unwrap();
            check(&next, out, 1 + count as i64);
            model = next;
            ports.push(port);
        });
    } else if scenario == "invalid-import" {
        measure("initially_invalid_repair", samples, |i| {
            let mut next = model.stage();
            next.set_expression(out, "(").unwrap();
            invalid(&next, out);
            let increment = 1 + (i % 2) as i64;
            next.set_expression(out, format!("n{}+{increment}", size - 2))
                .unwrap();
            check(&next, out, count as i64 + increment);
            model = next;
        });
    } else if scenario == "inspection" {
        measure("inspection_capture_page", samples, |_| {
            let checksum = if integrated {
                let view = model.inspect().unwrap();
                let page = view.nodes(None, 256, 4096).unwrap();
                page.nodes().iter().map(|n| n.name.len()).sum::<usize>()
            } else {
                let copy = model.stage();
                copy.draft_nodes()
                    .take(256)
                    .map(|(_, name, _)| name.len())
                    .sum()
            };
            assert!(checksum > 0);
        });
        measure("reverse_edge_page", samples, |_| {
            let values = if integrated {
                model
                    .inspect()
                    .unwrap()
                    .edges(nodes[0], true, None, 256)
                    .unwrap()
                    .nodes()
                    .to_vec()
            } else {
                let copy = model.stage();
                copy.draft_nodes()
                    .filter(|(_, _, d)| match d {
                        pivot_core::model::DraftDefinition::Computed {
                            binding: Ok(deps), ..
                        } => deps.contains(&nodes[0]),
                        _ => false,
                    })
                    .map(|(id, _, _)| id)
                    .take(256)
                    .collect::<Vec<_>>()
            };
            assert_eq!(values, vec![nodes[start]]);
        });
        measure("full_snapshot", samples, |_| {
            let snapshot = model.snapshot().unwrap();
            assert_eq!(snapshot.revision(), model.revision());
            assert_eq!(snapshot.dependency_order().len(), size);
        });
        measure("complete_diagnosis", samples, |_| {
            if integrated {
                model
                    .inspect()
                    .unwrap()
                    .validate_complete(&CancellationToken::default())
                    .unwrap();
            } else {
                model.snapshot().unwrap();
            }
        });
        if size <= 4096 {
            measure("portable_export", samples, |_| {
                let image = transfer::export(&model, &nodes, &ports).unwrap();
                assert_eq!(image.nodes.len(), size);
            });
            let image = transfer::export(&model, &nodes, &ports).unwrap();
            measure("portable_import_calculate", samples, |_| {
                let imported = if integrated {
                    transfer::import_persistent(image.clone(), &runtime).unwrap()
                } else {
                    transfer::import(image.clone()).unwrap()
                };
                check(&imported.model, imported.nodes[size - 1], 1 + count as i64);
            });
        } else {
            println!(
                "SKIP,portable_transfer,frozen live-node/ledger limits; not admitted at {size}"
            );
            measure("portable_reject", samples, |_| {
                assert!(transfer::export(&model, &nodes, &ports).is_err())
            });
        }
    } else if scenario == "contention" {
        let (background, _, _) = fixture(
            size,
            "Independent background",
            integrated.then_some(&runtime),
        );
        runtime.flush();
        let stop = Arc::new(AtomicBool::new(false));
        let busy = Arc::new(AtomicUsize::new(0));
        let completed = Arc::new(AtomicUsize::new(0));
        let (ready, started) = std::sync::mpsc::channel();
        std::thread::scope(|scope| {
            for _ in 0..2 {
                let stop = stop.clone();
                let busy = busy.clone();
                let completed = completed.clone();
                let ready = ready.clone();
                let background = &background;
                scope.spawn(move || {
                    let mut first = true;
                    while !stop.load(Relaxed) {
                        busy.fetch_add(1, Relaxed);
                        let snapshot = background.snapshot().unwrap();
                        assert_eq!(snapshot.dependency_order().len(), size);
                        drop(snapshot);
                        busy.fetch_sub(1, Relaxed);
                        completed.fetch_add(1, Relaxed);
                        if first {
                            ready.send(()).unwrap();
                            first = false;
                        }
                    }
                });
            }
            started.recv().unwrap();
            started.recv().unwrap();
            let wait = Instant::now();
            while busy.load(Relaxed) < 2 && wait.elapsed().as_secs() < 5 {
                std::thread::yield_now();
            }
            assert!(completed.load(Relaxed) >= 2);
            measure("input_capture_contended", samples, |i| {
                let mut next = model.stage();
                next.set_input(nodes[0], (i as i64).into()).unwrap();
                let capture = next.capture_local().unwrap();
                model = next;
                drop(capture);
            });
            stop.store(true, Relaxed);
        });
        println!("BACKGROUND_COMPLETED,{}", completed.load(Relaxed));
        drop(background);
        runtime.flush();
    } else {
        panic!("scenario");
    }
    runtime.flush();
    let stats = runtime.stats();
    println!(
        "CLEANUP,{},{},{},{}",
        stats.completed_cleanup, stats.total_drop_ns, stats.max_drop_ns, stats.peak_pending_cleanup
    );
    println!(
        "HEAP,{},{},{}",
        LIVE.load(Relaxed),
        PEAK.load(Relaxed),
        runtime.memory().stats().peak
    );
    let timer = Instant::now();
    drop(model);
    let enqueue = timer.elapsed().as_nanos();
    let timer = Instant::now();
    runtime.flush();
    let drain = timer.elapsed().as_nanos();
    let stats = runtime.stats();
    let memory = runtime.memory();
    let timer = Instant::now();
    assert!(runtime.finish().is_ok());
    println!(
        "FINAL_DROP,{enqueue},{drain},{},{}",
        timer.elapsed().as_nanos(),
        memory.stats().charged
    );
    println!(
        "TOTAL_CLEANUP,{},{},{},{}",
        stats.completed_cleanup, stats.total_drop_ns, stats.max_drop_ns, stats.peak_pending_cleanup
    );
    assert_eq!(memory.stats().charged, 0);
}
