//! Controlled integration through WorkingModel, not a second model facade.
//! Full inspection snapshots, transfer authoring and port edits are not enabled.
use super::*;
use crate::evaluation::{CalculationResult, EvaluationContext, EvaluationError};
use std::{any::Any, sync::mpsc, thread};
pub use structural::memory::{Memory as AllocationDomain, MemoryStats as AllocationStats};
use structural::{Calculation, Capture, ComputeError, Edit, Graph, Limits, memory::Memory};

type Job = Box<dyn Any + Send>;

fn on_worker<T: Send + 'static>(
    queue: &mpsc::SyncSender<Job>,
    task: impl FnOnce() -> T + Send + 'static,
) -> Result<T, &'static str> {
    struct Task<F: FnOnce()>(Option<F>);
    impl<F: FnOnce()> Drop for Task<F> {
        fn drop(&mut self) {
            if let Some(f) = self.0.take() {
                f();
            }
        }
    }
    let (reply, receive) = mpsc::sync_channel(1);
    queue
        .send(Box::new(Task(Some(move || {
            let _ = reply.send(task());
        }))))
        .map_err(|_| "worker disconnected")?;
    receive.recv().map_err(|_| "worker task failed")
}

/// Explicit runtime owner. Drop disconnects its handle without joining; call
/// finish after disposing all models, branches, captures and results to drain.
pub struct IntegrationRuntime {
    queue: Arc<mpsc::SyncSender<Job>>,
    worker: Option<thread::JoinHandle<()>>,
    memory: Memory,
}
impl IntegrationRuntime {
    pub fn new(budget: usize, queue_capacity: usize) -> Result<Self, ModelError> {
        Self::with_domain(Memory::new(budget), queue_capacity)
    }
    /// Several runtimes may share a domain for aggregate known-allocation
    /// admission. Caller buffers and allocator RSS remain outside this ledger.
    pub fn with_domain(
        memory: AllocationDomain,
        queue_capacity: usize,
    ) -> Result<Self, ModelError> {
        let charge = memory
            .reserve(256 * 1024 + 4096 + queue_capacity.saturating_mul(128))
            .map_err(|_| ModelError::Backend("allocation admission"))?;
        let (queue, receiver) = mpsc::sync_channel::<Job>(queue_capacity);
        let worker = thread::Builder::new()
            .name("pivot-core-release".into())
            .stack_size(256 * 1024)
            .spawn(move || {
                let _charge = charge;
                while let Ok(job) = receiver.recv() {
                    // A destructor panic must not disconnect later root owners.
                    let _ = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| drop(job)));
                }
            })
            .map_err(|_| ModelError::Backend("reclamation worker unavailable"))?;
        Ok(Self {
            queue: Arc::new(queue),
            worker: Some(worker),
            memory,
        })
    }
    pub fn memory(&self) -> Memory {
        self.memory.clone()
    }
    /// Wait for already-enqueued disposal; keep this off interactive threads.
    pub fn flush(&self) {
        struct Barrier(mpsc::Sender<()>);
        impl Drop for Barrier {
            fn drop(&mut self) {
                let _ = self.0.send(());
            }
        }
        let (sender, receiver) = mpsc::channel();
        if self.queue.send(Box::new(Barrier(sender))).is_ok() {
            let _ = receiver.recv();
        }
    }
    /// Consuming drain; returns the runtime if any live resource still owns its
    /// queue, instead of joining while that resource could keep it open.
    pub fn finish(mut self) -> Result<(), Self> {
        if Arc::strong_count(&self.queue) != 1 {
            return Err(self);
        }
        drop(self.queue);
        self.worker
            .take()
            .unwrap()
            .join()
            .expect("reclamation worker panicked");
        Ok(())
    }
}

struct Owned<T: Send + 'static> {
    value: Option<T>,
    queue: Arc<mpsc::SyncSender<Job>>,
}
impl<T: Send + 'static> Owned<T> {
    fn new(value: T, queue: Arc<mpsc::SyncSender<Job>>) -> Self {
        Self {
            value: Some(value),
            queue,
        }
    }
    fn get(&self) -> &T {
        self.value.as_ref().unwrap()
    }
}
impl<T: Clone + Send + 'static> Clone for Owned<T> {
    fn clone(&self) -> Self {
        Self::new(self.get().clone(), self.queue.clone())
    }
}
impl<T: Send + 'static> Drop for Owned<T> {
    fn drop(&mut self) {
        if let Some(value) = self.value.take() {
            // Bounded backpressure preserves ownership when the queue is full.
            // If the worker is unavailable, the returned job is safely dropped
            // here: synchronous emergency cleanup, never a leak or lost owner.
            let _ = self.queue.send(Box::new(value));
        }
    }
}

#[derive(Clone)]
pub(super) struct Integrated {
    graph: Owned<Graph>,
}
impl fmt::Debug for Integrated {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("PersistentCore")
    }
}
impl Integrated {
    pub(super) fn graph(&self) -> &Graph {
        self.graph.get()
    }
}

#[derive(Debug, PartialEq, Eq)]
pub enum CaptureFailure {
    Invalid(SnapshotError),
    Resource(&'static str),
}
#[derive(Debug, PartialEq, Eq)]
pub enum CalculationFailure {
    Evaluation(EvaluationError),
    Resource(&'static str),
}
impl fmt::Display for CaptureFailure {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{self:?}")
    }
}
impl std::error::Error for CaptureFailure {}
impl fmt::Display for CalculationFailure {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{self:?}")
    }
}
impl std::error::Error for CalculationFailure {}

enum State {
    Reference(ModelSnapshot),
    Persistent(Owned<Capture>),
}
/// Calculation state is separate from the legacy full-inspection snapshot API.
pub struct CalculationCapture {
    revision: Revision,
    state: State,
}
enum ResultState {
    Reference(CalculationResult),
    Persistent(Owned<Calculation>),
}
pub struct CapturedResult {
    revision: Revision,
    state: ResultState,
}
impl CapturedResult {
    pub fn revision(&self) -> Revision {
        self.revision
    }
    pub fn values(&self) -> &BTreeMap<NodeId, Result<PivotDecimal, EvaluationError>> {
        match &self.state {
            ResultState::Reference(r) => r.values(),
            ResultState::Persistent(r) => r.get().values(),
        }
    }
}
impl Clone for CalculationCapture {
    fn clone(&self) -> Self {
        Self {
            revision: self.revision,
            state: match &self.state {
                State::Reference(s) => State::Reference(s.clone()),
                State::Persistent(s) => State::Persistent(s.clone()),
            },
        }
    }
}
impl CalculationCapture {
    pub fn revision(&self) -> Revision {
        self.revision
    }
    pub fn evaluate(&self, outputs: &[NodeId]) -> Result<CapturedResult, CalculationFailure> {
        let state = match &self.state {
            State::Reference(s) => ResultState::Reference(
                EvaluationContext::new(s.clone())
                    .evaluate(outputs)
                    .map_err(CalculationFailure::Evaluation)?,
            ),
            State::Persistent(s) => {
                let queue = s.queue.clone();
                let r = s
                    .get()
                    .evaluate_retiring(outputs, |scratch| drop(Owned::new(scratch, queue)))
                    .map_err(|e| match e {
                        ComputeError::Invocation(
                            crate::evaluation::InvocationError::Evaluation(e),
                        ) => CalculationFailure::Evaluation(e),
                        _ => CalculationFailure::Resource("calculation admission or work budget"),
                    })?;
                ResultState::Persistent(Owned::new(r, s.queue.clone()))
            }
        };
        Ok(CapturedResult {
            revision: self.revision,
            state,
        })
    }
}

impl WorkingModel {
    /// One-time O(N) activation/import bridge. Existing bound tokens and local
    /// identities survive; no formulas are rebound. The default constructor
    /// remains the reference backend. No implicit migration in either direction.
    pub fn enable_persistent(&mut self, runtime: &IntegrationRuntime) -> Result<(), ModelError> {
        if self.integrated.is_some() {
            return Ok(());
        }
        // The source stays borrowed and intact until activation succeeds. An
        // admitted scoped worker owns temporary import/legacy-snapshot cleanup;
        // no full source copy or destructive move is required for this bridge.
        let _stack = runtime
            .memory
            .reserve(256 * 1024 + 4096)
            .map_err(|_| ModelError::Backend("activation stack admission"))?;
        let graph = thread::scope(|scope| {
            let worker = thread::Builder::new()
                .name("pivot-core-import".into())
                .stack_size(256 * 1024)
                .spawn_scoped(scope, || {
                    Graph::from_model_in(self, Limits::default(), runtime.memory.clone())
                })
                .map_err(|_| ModelError::Backend("activation worker unavailable"))?;
            worker
                .join()
                .map_err(|_| ModelError::Backend("activation worker failed"))?
                .map_err(map_error)
        })?;
        self.integrated = Some(Integrated {
            graph: Owned::new(graph, runtime.queue.clone()),
        });
        // Dispose the original imported representation on the worker too.
        let old_nodes = std::mem::take(&mut self.nodes);
        let old_names = std::mem::take(&mut self.names);
        let old_name = std::mem::take(&mut self.name);
        let old_interface = std::mem::take(&mut self.interface);
        drop(Owned::new(
            (old_nodes, old_names, old_name, old_interface),
            runtime.queue.clone(),
        ));
        Ok(())
    }
    /// Compare the real core revision, independently of hosted publication IDs.
    /// A branch keeps ordinary per-operation revision increments; installation
    /// remains serialized by its owner, as with the accepted stage method.
    pub fn stage_expected(&self, expected: Revision) -> Result<Self, ModelError> {
        if expected != self.revision {
            return Err(ModelError::Backend("stale core revision"));
        }
        Ok(self.stage())
    }
    pub(super) fn integrated_add(
        &mut self,
        name: String,
        value: Option<PivotDecimal>,
        source: Option<String>,
    ) -> Result<NodeId, ModelError> {
        let id = self.integrated.as_ref().unwrap().graph().next_identity();
        self.integrated_edit(match value {
            Some(v) => Edit::AddInput(name, v),
            None => Edit::AddFormula(name, source.unwrap()),
        })?;
        Ok(id)
    }
    pub(super) fn integrated_edit(&mut self, edit: Edit) -> Result<(), ModelError> {
        let next = self
            .revision
            .version
            .checked_add(1)
            .ok_or(ModelError::IdentityExhausted)?;
        let i = self.integrated.as_mut().unwrap();
        // Graph::apply is the canonical persistent transaction implementation.
        // No shadow WorkingModel or duplicate edit/evaluation implementation.
        let graph = i.graph.value.as_mut().unwrap();
        let queue = i.graph.queue.clone();
        // All private edit preparation and failed validation scratch live on
        // the worker. It uses the canonical graph transaction unchanged.
        let mut candidate = graph.clone();
        let (candidate, outcome) = on_worker(&queue, move || {
            let outcome = candidate.apply(candidate.version(), vec![edit]);
            (candidate, outcome)
        })
        .map_err(ModelError::Backend)?;
        let obsolete = if outcome.is_ok() {
            std::mem::replace(graph, candidate)
        } else {
            candidate
        };
        drop(Owned::new(obsolete, queue));
        outcome.map_err(map_error)?;
        self.revision.version = next;
        Ok(())
    }
    /// Locally efficient immutable capture for the integrated subset. Invalid
    /// drafts use the explicitly guarded O(N) compatibility diagnostic bridge,
    /// preserving the accepted full Kahn error and first-error ordering.
    pub fn capture_calculation(&self) -> Result<CalculationCapture, CaptureFailure> {
        let state = if let Some(i) = &self.integrated {
            if i.graph().has_drafts() {
                let graph = i.graph().clone();
                let diagnosis = on_worker(&i.graph.queue, move || graph.diagnose())
                    .map_err(CaptureFailure::Resource)?
                    .map_err(|_| CaptureFailure::Resource("full diagnostic admission"))?;
                if let Err(e) = diagnosis {
                    return Err(CaptureFailure::Invalid(e));
                }
            }
            let capture = i.graph().capture().map_err(|e| match e {
                structural::CaptureError::Invalid(e) => CaptureFailure::Invalid(e),
                structural::CaptureError::Admission => {
                    CaptureFailure::Resource("capture admission")
                }
            })?;
            State::Persistent(Owned::new(capture, i.graph.queue.clone()))
        } else {
            State::Reference(self.snapshot().map_err(CaptureFailure::Invalid)?)
        };
        Ok(CalculationCapture {
            revision: self.revision,
            state,
        })
    }
}

fn map_error(error: structural::Error) -> ModelError {
    match error {
        structural::Error::Model(e) => e,
        structural::Error::Unknown(id) => ModelError::UnknownNode(id),
        structural::Error::NotInput(id) => ModelError::NotInput(id),
        structural::Error::Exhausted => ModelError::IdentityExhausted,
        _ => ModelError::Backend("persistent admission, quota or work budget"),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn full_queue_backpressure_preserves_owner_and_runs_drop_on_receiver() {
        let (sender, receiver) = mpsc::sync_channel::<Job>(0);
        let queue = Arc::new(sender);
        let (released, observed) = mpsc::channel();
        struct Probe(mpsc::Sender<thread::ThreadId>);
        impl Drop for Probe {
            fn drop(&mut self) {
                self.0.send(thread::current().id()).unwrap();
            }
        }
        let owner = Owned::new(Probe(released), queue.clone());
        let submitter = thread::spawn(move || drop(owner));
        assert!(observed.try_recv().is_err());
        let job = receiver.recv().unwrap();
        submitter.join().unwrap();
        assert!(observed.try_recv().is_err());
        drop(job);
        assert_eq!(observed.recv().unwrap(), thread::current().id());
    }
    #[test]
    fn disconnected_queue_preserves_safe_emergency_cleanup() {
        let (sender, receiver) = mpsc::sync_channel::<Job>(1);
        drop(receiver);
        let (tx, rx) = mpsc::channel();
        struct Probe(mpsc::Sender<()>);
        impl Drop for Probe {
            fn drop(&mut self) {
                self.0.send(()).unwrap();
            }
        }
        drop(Owned::new(Probe(tx), Arc::new(sender)));
        rx.recv().unwrap();
    }
}
