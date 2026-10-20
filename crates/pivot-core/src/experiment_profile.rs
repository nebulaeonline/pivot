//! Opt-in phase instrumentation for the isolated scaling experiment.
use std::{
    cell::Cell,
    sync::atomic::{AtomicBool, Ordering::Relaxed},
    time::Instant,
};
static ENABLED: AtomicBool = AtomicBool::new(false);
pub fn enable(value: bool) {
    ENABLED.store(value, Relaxed);
}
pub const PHASES: usize = 12;
pub const NAMES: [&str; PHASES] = [
    "other",
    "core_stage",
    "sheet_stage",
    "quota",
    "apply",
    "bind",
    "snapshot",
    "dependency_order",
    "evaluate",
    "read",
    "commit",
    "prototype_edit",
];
thread_local! {
    static PHASE: Cell<usize> = const { Cell::new(0) };
    static TIMES: Cell<[u64; PHASES]> = const { Cell::new([0; PHASES]) };
    static COPIES: Cell<u64> = const { Cell::new(0) };
}
pub fn phase() -> usize {
    PHASE.try_with(Cell::get).unwrap_or(0)
}
pub fn copy(bytes: usize) {
    if !ENABLED.load(Relaxed) {
        return;
    }
    let _ = COPIES.try_with(|c| c.set(c.get() + bytes as u64));
}
pub fn reset() {
    TIMES.with(|c| c.set([0; PHASES]));
    COPIES.with(|c| c.set(0));
}
pub fn metrics() -> ([u64; PHASES], u64) {
    (TIMES.with(Cell::get), COPIES.with(Cell::get))
}
pub struct Scope {
    previous: usize,
    phase: usize,
    start: Option<Instant>,
}
pub fn scope(phase: usize) -> Scope {
    let previous = PHASE.with(|c| c.replace(phase));
    Scope {
        previous,
        phase,
        start: ENABLED.load(Relaxed).then(Instant::now),
    }
}
impl Drop for Scope {
    fn drop(&mut self) {
        let Some(start) = self.start else {
            PHASE.with(|c| c.set(self.previous));
            return;
        };
        TIMES.with(|c| {
            let mut t = c.get();
            t[self.phase] += start.elapsed().as_nanos() as u64;
            c.set(t)
        });
        PHASE.with(|c| c.set(self.previous));
    }
}

/// Opt-in cross-worker construction attribution, separate from legacy TLS totals.
pub mod construction;
