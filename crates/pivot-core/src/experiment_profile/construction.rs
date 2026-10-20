//! Exclusive phase wall time across threads; opt-in instrumentation is not a CPU profiler.
use std::{
    cell::Cell,
    sync::atomic::{AtomicBool, AtomicU64, Ordering::Relaxed},
    time::Instant,
};
pub const NAMES: [&str; 15] = [
    "other",
    "request",
    "identity",
    "sheet_indexes",
    "parse",
    "binding",
    "core_install",
    "validation",
    "admission",
    "publication",
    "reclamation",
    "worksheet",
    "core_dispatch_wait",
    "graph_pages",
    "graph_names",
];
const N: usize = NAMES.len();
static ENABLED: AtomicBool = AtomicBool::new(false);
static NS: [AtomicU64; N] = [const { AtomicU64::new(0) }; N];
static BYTES: [AtomicU64; N] = [const { AtomicU64::new(0) }; N];
static CALLS: [AtomicU64; N] = [const { AtomicU64::new(0) }; N];
thread_local! { static CURRENT: Cell<usize> = const { Cell::new(0) }; static STAMP: Cell<Option<Instant>> = const { Cell::new(None) }; }
pub fn start() {
    for a in [&NS, &BYTES, &CALLS] {
        for v in a {
            v.store(0, Relaxed);
        }
    }
    ENABLED.store(true, Relaxed);
}
pub fn stop() -> Vec<(&'static str, u64, u64, u64)> {
    ENABLED.store(false, Relaxed);
    (0..N)
        .map(|i| {
            (
                NAMES[i],
                NS[i].load(Relaxed),
                BYTES[i].load(Relaxed),
                CALLS[i].load(Relaxed),
            )
        })
        .collect()
}
fn mark(next: usize) -> usize {
    let now = Instant::now();
    let old = CURRENT.with(|v| v.replace(next));
    STAMP.with(|s| {
        if let Some(t) = s.replace((next != 0).then_some(now)) {
            NS[old].fetch_add(now.duration_since(t).as_nanos() as u64, Relaxed);
        }
    });
    old
}
pub struct Scope(Option<usize>);
pub fn scope(phase: usize) -> Scope {
    Scope(ENABLED.load(Relaxed).then(|| mark(phase)))
}
impl Drop for Scope {
    fn drop(&mut self) {
        if let Some(old) = self.0 {
            mark(old);
        }
    }
}
/// Called by the isolated counting allocator; performs no allocation itself.
pub fn allocation(bytes: usize) {
    if ENABLED.load(Relaxed) {
        let phase = CURRENT.try_with(Cell::get).unwrap_or(0);
        BYTES[phase].fetch_add(bytes as u64, Relaxed);
        CALLS[phase].fetch_add(1, Relaxed);
    }
}
