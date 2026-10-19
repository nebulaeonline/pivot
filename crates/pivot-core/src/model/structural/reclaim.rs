//! Explicit bounded handoff of obsolete owners to a reclamation worker.
//! Shutdown joins the worker; callers must keep shutdown off interactive threads.
use super::memory::{Admission, Charge, Memory};
use std::{
    any::Any,
    sync::mpsc::{self, SyncSender, TrySendError},
    thread::{self, JoinHandle},
};
struct Job {
    value: Box<dyn Any + Send>,
    _charge: Charge,
}
pub struct Reclaimer {
    sender: Option<SyncSender<Job>>,
    worker: Option<JoinHandle<()>>,
    _charge: Charge,
    memory: Memory,
}
pub struct Rejected<T> {
    pub value: T,
    pub reason: &'static str,
}
impl Reclaimer {
    pub fn new(memory: Memory, capacity: usize) -> Result<Self, Admission> {
        let charge =
            memory.reserve(256 * 1024 + 4096 + capacity.checked_mul(128).ok_or(Admission)?)?;
        let (sender, receiver) = mpsc::sync_channel::<Job>(capacity);
        let worker = thread::Builder::new()
            .name("pivot-reclaim".into())
            .stack_size(256 * 1024)
            .spawn(move || {
                while let Ok(job) = receiver.recv() {
                    drop(job);
                }
            })
            .map_err(|_| Admission)?;
        Ok(Self {
            sender: Some(sender),
            worker: Some(worker),
            _charge: charge,
            memory,
        })
    }
    pub fn retire<T: Send + 'static>(&self, value: T) -> Result<(), Rejected<T>> {
        let charge = match self.memory.reserve(std::mem::size_of::<T>() + 128) {
            Ok(c) => c,
            Err(_) => {
                return Err(Rejected {
                    value,
                    reason: "admission",
                });
            }
        };
        let job = Job {
            value: Box::new(value),
            _charge: charge,
        };
        match self.sender.as_ref().unwrap().try_send(job) {
            Ok(()) => Ok(()),
            Err(error) => {
                let (job, reason) = match error {
                    TrySendError::Full(job) => (job, "queue full"),
                    TrySendError::Disconnected(job) => (job, "worker disconnected"),
                };
                let value = *job.value.downcast::<T>().unwrap_or_else(|_| unreachable!());
                Err(Rejected { value, reason })
            }
        }
    }
}
impl Drop for Reclaimer {
    fn drop(&mut self) {
        drop(self.sender.take());
        if let Some(worker) = self.worker.take() {
            worker.join().expect("reclaimer panicked");
        }
    }
}
