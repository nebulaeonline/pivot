//! Fixed work classes and pre-admitted last-owner reclamation.
use super::*;
use std::sync::{
    Condvar, Mutex, MutexGuard,
    atomic::{AtomicBool, Ordering},
};
use structural::memory::Charge;

fn lock<T>(value: &Mutex<T>) -> MutexGuard<'_, T> {
    value
        .lock()
        .unwrap_or_else(std::sync::PoisonError::into_inner)
}

#[derive(Clone, Default)]
pub struct CancellationToken(Arc<AtomicBool>);
impl CancellationToken {
    pub fn cancel(&self) {
        self.0.store(true, Ordering::Release);
    }
    pub fn is_cancelled(&self) -> bool {
        self.0.load(Ordering::Acquire)
    }
}

type Garbage = Box<dyn Any + Send>;
struct Retired {
    _value: Garbage,
    _charge: Charge,
}
struct Pool {
    free: Vec<usize>,
    live: usize,
    pending: usize,
    failed: bool,
    recovery_queued: bool,
    completed: u64,
    drop_ns: u128,
    max_drop_ns: u128,
    peak_pending: usize,
}
struct Slots {
    values: Vec<Mutex<Option<Retired>>>,
    pool: Mutex<Pool>,
    changed: Condvar,
    _charge: Charge,
}
impl Slots {
    fn release(&self, index: usize) {
        let retired = { lock(&self.values[index]).take() };
        if let Some(value) = retired {
            let _profile = crate::experiment_profile::construction::scope(10);
            let start = std::time::Instant::now();
            let _ = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| drop(value)));
            let elapsed = start.elapsed().as_nanos();
            drop(_profile);
            let mut pool = lock(&self.pool);
            pool.completed += 1;
            pool.drop_ns += elapsed;
            pool.max_drop_ns = pool.max_drop_ns.max(elapsed);
            pool.pending -= 1;
            pool.free.push(index);
            self.changed.notify_all();
        }
    }
}
struct Notifications {
    primary: mpsc::SyncSender<usize>,
    fallback: mpsc::SyncSender<usize>,
}
static RECOVERY: Mutex<Option<mpsc::SyncSender<Arc<Slots>>>> = Mutex::new(None);
fn recovery_sender() -> Result<mpsc::SyncSender<Arc<Slots>>, ModelError> {
    let mut existing = lock(&RECOVERY);
    if let Some(sender) = &*existing {
        return Ok(sender.clone());
    }
    let (sender, receiver) = mpsc::sync_channel::<Arc<Slots>>(64);
    thread::Builder::new()
        .name("pivot-cleanup-recovery".into())
        .stack_size(256 * 1024)
        .spawn(move || {
            while let Ok(slots) = receiver.recv() {
                loop {
                    for index in 0..slots.values.len() {
                        slots.release(index);
                    }
                    let mut pool = lock(&slots.pool);
                    if pool.pending == 0 {
                        pool.recovery_queued = false;
                        slots.changed.notify_all();
                        break;
                    }
                    if !slots.values.iter().any(|value| lock(value).is_some()) {
                        pool = slots
                            .changed
                            .wait(pool)
                            .unwrap_or_else(std::sync::PoisonError::into_inner);
                    }
                    drop(pool);
                }
            }
        })
        .map_err(|_| ModelError::Backend("process recovery worker unavailable"))?;
    *existing = Some(sender.clone());
    Ok(sender)
}
fn cleanup_workers(
    memory: &Memory,
    slots: &Arc<Slots>,
) -> Result<(Notifications, Vec<thread::JoinHandle<()>>), ModelError> {
    let mut workers = Vec::new();
    let mut senders = Vec::new();
    let outcome = (|| {
        for name in ["pivot-reclaim", "pivot-reclaim-reserve"] {
            let stack = memory
                .reserve(256 * 1024 + 4096)
                .map_err(|_| ModelError::Backend("cleanup stack admission"))?;
            let (sender, receiver) = mpsc::sync_channel::<usize>(slots.values.len());
            let slots = slots.clone();
            let worker = thread::Builder::new()
                .name(name.into())
                .stack_size(256 * 1024)
                .spawn(move || {
                    let _stack = stack;
                    while let Ok(index) = receiver.recv() {
                        slots.release(index);
                    }
                })
                .map_err(|_| ModelError::Backend("cleanup worker unavailable"))?;
            workers.push(worker);
            senders.push(sender);
        }
        Ok::<(), ModelError>(())
    })();
    if let Err(error) = outcome {
        drop(senders);
        for worker in workers {
            let _ = worker.join();
        }
        return Err(error);
    }
    let fallback = senders.pop().unwrap();
    let primary = senders.pop().unwrap();
    Ok((Notifications { primary, fallback }, workers))
}
pub(super) struct Cleanup {
    notify: Mutex<Notifications>,
    recovery: mpsc::SyncSender<Arc<Slots>>,
    slots: Arc<Slots>,
    memory: Memory,
}
impl Cleanup {
    fn new(
        memory: Memory,
        capacity: usize,
        workers: &mut Vec<thread::JoinHandle<()>>,
    ) -> Result<Arc<Self>, ModelError> {
        let count = capacity
            .checked_mul(16)
            .and_then(|n| n.checked_add(64))
            .filter(|n| *n <= 65600)
            .ok_or(ModelError::Backend(
                "runtime capacity exceeds supported bound",
            ))?;
        let charge = memory
            .reserve(4096 + count * 128)
            .map_err(|_| ModelError::Backend("cleanup metadata admission"))?;
        let slots = Arc::new(Slots {
            values: (0..count).map(|_| Mutex::new(None)).collect(),
            pool: Mutex::new(Pool {
                free: (0..count).rev().collect(),
                live: 0,
                pending: 0,
                failed: false,
                recovery_queued: false,
                completed: 0,
                drop_ns: 0,
                max_drop_ns: 0,
                peak_pending: 0,
            }),
            changed: Condvar::new(),
            _charge: charge,
        });
        let recovery = recovery_sender()?;
        let (notify, created) = cleanup_workers(&memory, &slots)?;
        workers.extend(created);
        Ok(Arc::new(Self {
            notify: Mutex::new(notify),
            recovery,
            slots,
            memory,
        }))
    }

    pub(super) fn reserve(self: &Arc<Self>) -> Result<Ticket, &'static str> {
        let charge = self
            .memory
            .reserve(1024)
            .map_err(|_| "owner metadata admission")?;
        let mut pool = lock(&self.slots.pool);
        if pool.failed {
            return Err("cleanup unavailable; retain runtime and recover before shutdown");
        }
        let index = pool
            .free
            .pop()
            .ok_or("cleanup capacity exhausted; release captures or retry after reclamation")?;
        pool.live += 1;
        Ok(Ticket {
            index: Some(index),
            charge: Some(charge),
            cleanup: self.clone(),
        })
    }
    pub(super) fn flush(&self) -> Result<(), &'static str> {
        let mut pool = lock(&self.slots.pool);
        while pool.pending != 0 && !pool.failed {
            pool = self
                .slots
                .changed
                .wait(pool)
                .unwrap_or_else(std::sync::PoisonError::into_inner);
        }
        if pool.failed {
            Err("cleanup workers disconnected; owners remain parked")
        } else {
            Ok(())
        }
    }
    pub(super) fn wait_payloads(&self) {
        let mut pool = lock(&self.slots.pool);
        while pool.pending != 0 || pool.recovery_queued {
            pool = self
                .slots
                .changed
                .wait(pool)
                .unwrap_or_else(std::sync::PoisonError::into_inner);
        }
    }
    fn live(&self) -> usize {
        lock(&self.slots.pool).live
    }
}

pub(super) struct Ticket {
    index: Option<usize>,
    charge: Option<Charge>,
    cleanup: Arc<Cleanup>,
}
impl Ticket {
    pub(super) fn retire<T: Send + 'static>(mut self, value: T) {
        let index = self.index.take().unwrap();
        let retired = Retired {
            _value: Box::new(value),
            _charge: self.charge.take().unwrap(),
        };
        let mut pool = lock(&self.cleanup.slots.pool);
        *lock(&self.cleanup.slots.values[index]) = Some(retired);
        pool.live -= 1;
        pool.pending += 1;
        pool.peak_pending = pool.peak_pending.max(pool.pending);
        let notify = lock(&self.cleanup.notify);
        let failed =
            notify.primary.try_send(index).is_err() && notify.fallback.try_send(index).is_err();
        let rescue = failed && !pool.recovery_queued;
        if failed {
            pool.failed = true;
            self.cleanup.slots.changed.notify_all();
        }
        if rescue {
            pool.recovery_queued = true;
        }
        drop(notify);
        drop(pool);
        // Normally notifications cannot fill: every owner reserved a distinct
        // slot. If both receivers are lost, one process-wide bounded recovery
        // worker retains and drains the slots, including nested cleanup owners.
        if rescue {
            self.cleanup
                .recovery
                .send(self.cleanup.slots.clone())
                .expect("process recovery worker disconnected");
        }
    }

    pub(super) fn own<T: Send + 'static>(self, value: T) -> Owned<T> {
        Owned(Arc::new(Released {
            value: Some(value),
            ticket: Some(self),
        }))
    }
}
impl Drop for Ticket {
    fn drop(&mut self) {
        if let Some(index) = self.index.take() {
            let mut pool = lock(&self.cleanup.slots.pool);
            pool.live -= 1;
            pool.free.push(index);
            self.cleanup.slots.changed.notify_all();
        }
    }
}
struct Released<T: Send + 'static> {
    value: Option<T>,
    ticket: Option<Ticket>,
}
impl<T: Send + 'static> Drop for Released<T> {
    fn drop(&mut self) {
        self.ticket
            .take()
            .unwrap()
            .retire(self.value.take().unwrap());
    }
}
pub(super) struct Owned<T: Send + 'static>(Arc<Released<T>>);
impl<T: Send + 'static> Owned<T> {
    pub(super) fn same(&self, other: &Self) -> bool {
        Arc::ptr_eq(&self.0, &other.0)
    }
    pub(super) fn get(&self) -> &T {
        self.0.value.as_ref().unwrap()
    }
}
impl<T: Send + 'static> Clone for Owned<T> {
    fn clone(&self) -> Self {
        Self(self.0.clone())
    }
}

type Work = Box<dyn FnOnce() + Send>;
/// Accepted work not yet running, including up to two receiver-assigned tasks.
/// The channel itself retains its configured bound; these are not OS queue lengths.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct WorkLaneStats {
    pub queued: usize,
    pub peak_queued: usize,
    pub accepted: u64,
    pub rejected: u64,
}
struct Task {
    queued: bool,
    queue: Arc<Mutex<WorkLaneStats>>,
    work: Option<Work>,
    ticket: Option<Ticket>,
    pending: Arc<(Mutex<usize>, Condvar)>,
}
impl Task {
    fn run(mut self) {
        if self.queued {
            lock(&self.queue).queued -= 1;
            self.queued = false;
        }
        (self.work.take().unwrap())();
    }
}
impl Drop for Task {
    fn drop(&mut self) {
        if self.queued {
            lock(&self.queue).queued -= 1;
            self.queued = false;
        }
        if let Some(work) = self.work.take() {
            self.ticket.take().unwrap().retire(work);
        }
        let mut pending = lock(&self.pending.0);
        *pending -= 1;
        self.pending.1.notify_all();
    }
}
pub(super) struct Lane {
    queue: Arc<Mutex<WorkLaneStats>>,
    sender: mpsc::SyncSender<Task>,
    pending: Arc<(Mutex<usize>, Condvar)>,
}
impl Lane {
    fn new(
        name: &str,
        capacity: usize,
        memory: &Memory,
        workers: &mut Vec<thread::JoinHandle<()>>,
    ) -> Result<Self, ModelError> {
        let (sender, receiver) = mpsc::sync_channel::<Task>(capacity);
        let receiver = Arc::new(Mutex::new(receiver));
        for index in 0..2 {
            let stack = memory
                .reserve(256 * 1024 + 4096 + capacity * 128)
                .map_err(|_| ModelError::Backend("dispatch stack admission"))?;
            let receiver = receiver.clone();
            workers.push(
                thread::Builder::new()
                    .name(format!("pivot-{name}-{index}"))
                    .stack_size(256 * 1024)
                    .spawn(move || {
                        let _stack = stack;
                        loop {
                            let task = { lock(&receiver).recv() };
                            let Ok(task) = task else {
                                break;
                            };
                            let _ = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
                                task.run()
                            }));
                        }
                    })
                    .map_err(|_| ModelError::Backend("dispatch worker unavailable"))?,
            );
        }
        Ok(Self {
            queue: Arc::new(Mutex::new(WorkLaneStats::default())),
            sender,
            pending: Arc::new((Mutex::new(0), Condvar::new())),
        })
    }
    pub(super) fn run<T: Send + 'static>(
        &self,
        cleanup: &Arc<Cleanup>,
        cancel: &CancellationToken,
        f: impl FnOnce() -> T + Send + 'static,
    ) -> Result<T, &'static str> {
        if cancel.is_cancelled() {
            return Err("cancelled");
        }
        let ticket = cleanup.reserve()?;
        let (sender, receiver) = mpsc::sync_channel(1);
        let cancel = cancel.clone();
        let task = Task {
            queued: true,
            queue: self.queue.clone(),
            work: Some(Box::new(move || {
                if cancel.is_cancelled() {
                    let _ = sender.send(Err("cancelled"));
                    return;
                }
                let value = f();
                if cancel.is_cancelled() {
                    drop(value);
                    let _ = sender.send(Err("cancelled"));
                } else {
                    let _ = sender.send(Ok(value));
                }
            })),
            ticket: Some(ticket),
            pending: self.pending.clone(),
        };
        *lock(&self.pending.0) += 1;
        let outcome = {
            let mut queue = lock(&self.queue);
            queue.queued += 1;
            match self.sender.try_send(task) {
                Ok(()) => {
                    queue.accepted += 1;
                    queue.peak_queued = queue.peak_queued.max(queue.queued);
                    Ok(())
                }
                Err(error) => {
                    let (message, mut task) = match error {
                        mpsc::TrySendError::Full(task) => {
                            ("work queue busy; retry or cancel obsolete work", task)
                        }
                        mpsc::TrySendError::Disconnected(task) => {
                            ("work lane disconnected; runtime unavailable", task)
                        }
                    };
                    queue.queued -= 1;
                    queue.rejected += 1;
                    task.queued = false;
                    Err((message, task))
                }
            }
        };
        if let Err((message, task)) = outcome {
            drop(task);
            return Err(message);
        }
        receiver.recv().map_err(|_| "work task failed")?
    }
    fn flush(&self) {
        let mut count = lock(&self.pending.0);
        while *count != 0 {
            count = self
                .pending
                .1
                .wait(count)
                .unwrap_or_else(std::sync::PoisonError::into_inner);
        }
    }
}

pub(super) struct RuntimeState {
    pub(super) cleanup: Arc<Cleanup>,
    pub(super) mutations: Lane,
    pub(super) heavy: Lane,
}
pub struct IntegrationRuntime {
    state: Option<Arc<RuntimeState>>,
    workers: Vec<thread::JoinHandle<()>>,
    cleanup_workers: usize,
    memory: Memory,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct RuntimeStats {
    pub mutations: WorkLaneStats,
    pub heavy: WorkLaneStats,
    pub live_owners: usize,
    pub pending_cleanup: usize,
    pub completed_cleanup: u64,
    pub total_drop_ns: u128,
    pub max_drop_ns: u128,
    pub peak_pending_cleanup: usize,
    pub cleanup_failed: bool,
}
impl IntegrationRuntime {
    pub fn new(budget: usize, queue_capacity: usize) -> Result<Self, ModelError> {
        Self::with_domain(Memory::new(budget), queue_capacity)
    }
    pub fn with_domain(memory: AllocationDomain, capacity: usize) -> Result<Self, ModelError> {
        if capacity > 4096 {
            return Err(ModelError::Backend("runtime queue capacity exceeds 4096"));
        }
        let mut workers = Vec::new();
        let created = (|| {
            let cleanup = Cleanup::new(memory.clone(), capacity, &mut workers)?;
            let mutations = Lane::new("mutation", capacity, &memory, &mut workers)?;
            let heavy = Lane::new("heavy", capacity, &memory, &mut workers)?;
            Ok::<_, ModelError>(Arc::new(RuntimeState {
                cleanup,
                mutations,
                heavy,
            }))
        })();
        match created {
            Ok(state) => Ok(Self {
                state: Some(state),
                workers,
                cleanup_workers: 2,
                memory,
            }),
            Err(error) => {
                for worker in workers {
                    let _ = worker.join();
                }
                Err(error)
            }
        }
    }

    pub(super) fn state(&self) -> &Arc<RuntimeState> {
        self.state.as_ref().unwrap()
    }
    pub fn memory(&self) -> AllocationDomain {
        self.memory.clone()
    }
    pub fn stats(&self) -> RuntimeStats {
        let pool = lock(&self.state().cleanup.slots.pool);
        RuntimeStats {
            mutations: *lock(&self.state().mutations.queue),
            heavy: *lock(&self.state().heavy.queue),
            live_owners: pool.live,
            pending_cleanup: pool.pending,
            completed_cleanup: pool.completed,
            total_drop_ns: pool.drop_ns,
            max_drop_ns: pool.max_drop_ns,
            peak_pending_cleanup: pool.peak_pending,
            cleanup_failed: pool.failed,
        }
    }
    /// Explicit noninteractive drain of already-dispatched work and reclamation.
    pub fn flush(&self) {
        self.try_flush()
            .expect("cleanup unavailable; retain runtime and call recover_cleanup");
    }
    pub fn try_flush(&self) -> Result<(), ModelError> {
        let state = self.state();
        state.mutations.flush();
        state.heavy.flush();
        state.cleanup.flush().map_err(ModelError::Backend)
    }
    /// Explicit noninteractive recovery after both ordinary cleanup receivers
    /// failed. Parked owners are first drained by the bounded process reaper;
    /// only the fixed two receivers are restarted. Existing captures stay owned.
    pub fn recover_cleanup(&mut self) -> Result<(), ModelError> {
        let cleanup = self.state().cleanup.clone();
        if !lock(&cleanup.slots.pool).failed {
            return Ok(());
        }
        let rest = self.workers.split_off(self.cleanup_workers);
        for worker in self.workers.drain(..) {
            let _ = worker.join();
        }
        self.workers = rest;
        self.cleanup_workers = 0;
        let (notify, new_workers) = cleanup_workers(&self.memory, &cleanup.slots)?;
        let mut pool = lock(&cleanup.slots.pool);
        while pool.pending != 0 || pool.recovery_queued {
            pool = cleanup
                .slots
                .changed
                .wait(pool)
                .unwrap_or_else(std::sync::PoisonError::into_inner);
        }
        *lock(&cleanup.notify) = notify;
        pool.failed = false;
        drop(pool);
        let rest = std::mem::replace(&mut self.workers, new_workers);
        self.workers.extend(rest);
        self.cleanup_workers = 2;
        Ok(())
    }
    pub fn finish(mut self) -> Result<(), Self> {
        if Arc::strong_count(self.state()) != 1 {
            return Err(self);
        }
        if self.recover_cleanup().is_err() {
            return Err(self);
        }
        self.state().mutations.flush();
        self.state().heavy.flush();
        if self.state().cleanup.flush().is_err() || self.state().cleanup.live() != 0 {
            return Err(self);
        }
        drop(self.state.take());
        for worker in self.workers.drain(..) {
            worker.join().expect("runtime worker panicked");
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    struct Probe(mpsc::Sender<thread::ThreadId>);
    impl Drop for Probe {
        fn drop(&mut self) {
            let _ = self.0.send(thread::current().id());
        }
    }
    fn join(cleanup: Arc<Cleanup>, workers: Vec<thread::JoinHandle<()>>) {
        cleanup.flush().unwrap();
        drop(cleanup);
        for worker in workers {
            worker.join().unwrap();
        }
    }
    #[test]
    fn admitted_cleanup_is_nonblocking_and_disconnected_primary_uses_reserved_receiver() {
        let memory = Memory::new(16 * 1024 * 1024);
        let mut workers = Vec::new();
        let cleanup = Cleanup::new(memory.clone(), 0, &mut workers).unwrap();
        let (closed, receiver) = mpsc::sync_channel(1);
        drop(receiver);
        drop(std::mem::replace(
            &mut lock(&cleanup.notify).primary,
            closed,
        ));
        let (sender, receiver) = mpsc::channel();
        let owner = cleanup.reserve().unwrap().own(Probe(sender));
        drop(owner);
        assert_ne!(
            receiver
                .recv_timeout(std::time::Duration::from_secs(5))
                .unwrap(),
            thread::current().id()
        );
        join(cleanup, workers);
        assert_eq!(memory.stats().charged, 0);
    }
    #[test]
    fn rejected_work_never_runs_or_destroys_its_payload_on_submitter() {
        for disconnected in [false, true] {
            let memory = Memory::new(16 * 1024 * 1024);
            let mut workers = Vec::new();
            let cleanup = Cleanup::new(memory.clone(), 0, &mut workers).unwrap();
            let (sender, receiver) = mpsc::sync_channel(0);
            let receiver = if disconnected {
                drop(receiver);
                None
            } else {
                Some(receiver)
            };
            let lane = Lane {
                queue: Arc::new(Mutex::new(WorkLaneStats::default())),
                sender,
                pending: Arc::new((Mutex::new(0), Condvar::new())),
            };
            let (tx, rx) = mpsc::channel();
            let probe = Probe(tx);
            let ran = Arc::new(AtomicBool::new(false));
            let observed = ran.clone();
            assert!(
                lane.run(&cleanup, &CancellationToken::default(), move || {
                    let _probe = probe;
                    observed.store(true, Ordering::Release);
                })
                .is_err()
            );
            assert!(!ran.load(Ordering::Acquire));
            assert_ne!(
                rx.recv_timeout(std::time::Duration::from_secs(5)).unwrap(),
                thread::current().id()
            );
            assert_eq!(*lock(&lane.pending.0), 0);
            drop((lane, receiver));
            join(cleanup, workers);
            assert_eq!(memory.stats().charged, 0);
        }
    }
    #[test]
    fn live_owner_admission_is_bounded_before_any_drop_handoff() {
        let memory = Memory::new(16 * 1024 * 1024);
        let mut workers = Vec::new();
        let cleanup = Cleanup::new(memory.clone(), 0, &mut workers).unwrap();
        let mut owners = Vec::new();
        while let Ok(ticket) = cleanup.reserve() {
            owners.push(ticket.own(1u64));
        }
        assert_eq!(owners.len(), 64);
        assert!(cleanup.reserve().is_err());
        drop(owners);
        join(cleanup, workers);
        assert_eq!(memory.stats().charged, 0);
    }
    #[test]
    fn two_blocked_heavy_jobs_do_not_block_independent_model_mutations() {
        let runtime = IntegrationRuntime::new(64 * 1024 * 1024, 4).unwrap();
        let mut model = WorkingModel::new_reference("independent").unwrap();
        model.enable_persistent(&runtime).unwrap();
        let id = model.add_input("x", 1.into()).unwrap();
        let gate = Arc::new((Mutex::new(false), Condvar::new()));
        let (ready, started) = mpsc::channel();
        let (done, complete) = mpsc::channel();
        thread::scope(|scope| {
            for _ in 0..2 {
                let state = runtime.state().clone();
                let ready = ready.clone();
                let gate = gate.clone();
                scope.spawn(move || {
                    state
                        .heavy
                        .run(&state.cleanup, &CancellationToken::default(), move || {
                            ready.send(()).unwrap();
                            let mut open = lock(&gate.0);
                            while !*open {
                                open = gate.1.wait(open).unwrap();
                            }
                        })
                        .unwrap();
                });
            }
            started
                .recv_timeout(std::time::Duration::from_secs(5))
                .unwrap();
            started
                .recv_timeout(std::time::Duration::from_secs(5))
                .unwrap();
            scope.spawn(|| {
                let result = model.set_input(id, 9.into());
                done.send(result).unwrap();
            });
            let result = complete.recv_timeout(std::time::Duration::from_secs(5));
            *lock(&gate.0) = true;
            gate.1.notify_all();
            assert!(result.unwrap().is_ok());
        });
        assert_eq!(
            model
                .capture_local()
                .unwrap()
                .evaluate(&[id])
                .unwrap()
                .values()[&id],
            Ok(9.into())
        );
        drop(model);
        assert!(runtime.finish().is_ok());
    }
    #[test]
    fn loss_of_both_cleanup_receivers_preserves_owners_and_can_be_recovered() {
        let mut runtime = IntegrationRuntime::new(32 * 1024 * 1024, 4).unwrap();
        let mut model = WorkingModel::new_reference("recovery").unwrap();
        model.enable_persistent(&runtime).unwrap();
        let id = model.add_input("x", 1.into()).unwrap();
        runtime.flush();
        let cleanup = runtime.state().cleanup.clone();
        let (tx, rx) = mpsc::channel();
        let owner = cleanup.reserve().unwrap().own(Probe(tx));
        {
            let mut notify = lock(&cleanup.notify);
            let Notifications { primary, fallback } = &mut *notify;
            for sender in [primary, fallback] {
                let (closed, receiver) = mpsc::sync_channel(1);
                drop(receiver);
                drop(std::mem::replace(sender, closed));
            }
        }
        drop(owner);
        assert_ne!(
            rx.recv_timeout(std::time::Duration::from_secs(5)).unwrap(),
            thread::current().id()
        );
        let revision = model.revision();
        assert!(model.set_input(id, 2.into()).is_err());
        assert_eq!(model.revision(), revision);
        assert!(runtime.try_flush().is_err());
        runtime.recover_cleanup().unwrap();
        model.set_input(id, 2.into()).unwrap();
        assert_eq!(
            model
                .capture_local()
                .unwrap()
                .evaluate(&[id])
                .unwrap()
                .values()[&id],
            Ok(2.into())
        );
        let memory = runtime.memory();
        drop((cleanup, model));
        assert!(runtime.finish().is_ok());
        assert_eq!(memory.stats().charged, 0);
    }
    #[test]
    fn failed_runtime_initialization_joins_partial_workers_and_releases_domain() {
        let memory = Memory::new(1024 * 1024);
        assert!(IntegrationRuntime::with_domain(memory.clone(), 4).is_err());
        assert_eq!(memory.stats().charged, 0);
    }
    #[test]
    fn blocked_cleanup_does_not_occupy_mutation_workers() {
        let runtime = IntegrationRuntime::new(64 * 1024 * 1024, 4).unwrap();
        let mut model = WorkingModel::new_reference("edit during reclaim").unwrap();
        model.enable_persistent(&runtime).unwrap();
        let id = model.add_input("x", 1.into()).unwrap();
        runtime.flush();
        struct Gate {
            ready: mpsc::Sender<()>,
            open: Arc<(Mutex<bool>, Condvar)>,
        }
        impl Drop for Gate {
            fn drop(&mut self) {
                self.ready.send(()).unwrap();
                let mut open = lock(&self.open.0);
                while !*open {
                    open = self.open.1.wait(open).unwrap();
                }
            }
        }
        let gate = Arc::new((Mutex::new(false), Condvar::new()));
        let (tx, rx) = mpsc::channel();
        runtime.state().cleanup.reserve().unwrap().retire(Gate {
            ready: tx,
            open: gate.clone(),
        });
        rx.recv_timeout(std::time::Duration::from_secs(5)).unwrap();
        let (done, complete) = mpsc::channel();
        thread::scope(|scope| {
            scope.spawn(|| {
                done.send(model.set_input(id, 8.into())).unwrap();
            });
            let result = complete.recv_timeout(std::time::Duration::from_secs(5));
            *lock(&gate.0) = true;
            gate.1.notify_all();
            assert!(result.unwrap().is_ok());
        });
        assert_eq!(
            model
                .capture_local()
                .unwrap()
                .evaluate(&[id])
                .unwrap()
                .values()[&id],
            Ok(8.into())
        );
        drop(model);
        assert!(runtime.finish().is_ok());
    }

    #[test]
    fn queued_large_batch_cancellation_and_backpressure_preserve_owners_and_revision() {
        let runtime = IntegrationRuntime::new(64 * 1024 * 1024, 1).unwrap();
        let mut model = runtime.create_model("queued").unwrap();
        let mut rejected = runtime.create_model("rejected").unwrap();
        let before = model.revision();
        let rejected_before = rejected.revision();
        let mut batch = model.begin_batch(1024, 16384).unwrap();
        for n in 0..1024 {
            batch.add_input(format!("n{n}"), 1.into()).unwrap();
        }
        let mut reject = rejected.begin_batch(1024, 16384).unwrap();
        for n in 0..1024 {
            reject.add_input(format!("r{n}"), 2.into()).unwrap();
        }
        let gate = Arc::new((Mutex::new(false), Condvar::new()));
        let (ready, started) = mpsc::channel();
        let (done, complete) = mpsc::channel();
        let cancel = CancellationToken::default();
        struct ReleaseGate(Arc<(Mutex<bool>, Condvar)>);
        impl Drop for ReleaseGate {
            fn drop(&mut self) {
                *lock(&self.0.0) = true;
                self.0.1.notify_all();
            }
        }
        thread::scope(|scope| {
            let _release = ReleaseGate(gate.clone());
            for _ in 0..2 {
                let state = runtime.state().clone();
                let ready = ready.clone();
                let gate = gate.clone();
                scope.spawn(move || {
                    state
                        .heavy
                        .run(&state.cleanup, &Default::default(), move || {
                            ready.send(()).unwrap();
                            let mut open = lock(&gate.0);
                            while !*open {
                                open = gate.1.wait(open).unwrap();
                            }
                        })
                        .unwrap()
                });
                // Capacity one is intentional: wait for each receiver to start
                // before submitting the next held job, avoiding a setup rejection.
                started
                    .recv_timeout(std::time::Duration::from_secs(5))
                    .unwrap();
            }
            let token = cancel.clone();
            let writer = &mut model;
            scope.spawn(move || {
                done.send(writer.apply_batch(batch, &token)).unwrap();
            });
            let deadline = std::time::Instant::now() + std::time::Duration::from_secs(5);
            while runtime.stats().heavy.queued == 0 && std::time::Instant::now() < deadline {
                thread::sleep(std::time::Duration::from_millis(1));
            }
            let queued = runtime.stats().heavy.queued;
            let result = if queued > 0 {
                Some(rejected.apply_batch(reject, &Default::default()))
            } else {
                None
            };
            let mut interactive = runtime.create_model("interactive").unwrap();
            interactive.add_input("responsive", 1.into()).unwrap();
            drop(interactive);
            cancel.cancel();
            *lock(&gate.0) = true;
            gate.1.notify_all();
            assert_eq!(queued, 1);
            assert!(result.unwrap().is_err());
            assert!(
                complete
                    .recv_timeout(std::time::Duration::from_secs(5))
                    .unwrap()
                    .is_err()
            );
        });
        assert_eq!(model.revision(), before);
        assert_eq!(rejected.revision(), rejected_before);
        assert!(model.node_id("n0").is_none());
        runtime.flush();
        let stats = runtime.stats();
        assert_eq!(stats.heavy.queued, 0);
        assert_eq!(stats.heavy.rejected, 1);
        assert!(stats.heavy.peak_queued <= 3);
        drop((model, rejected));
        let memory = runtime.memory();
        assert!(runtime.finish().is_ok());
        assert_eq!(memory.stats().charged, 0);
    }
}
