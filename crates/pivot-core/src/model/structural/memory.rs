//! Charged allocation domain. This bounds known requested storage, not OS RSS.
use std::{
    ops::Deref,
    sync::{
        Arc,
        atomic::{AtomicUsize, Ordering::Relaxed},
    },
};
#[derive(Clone)]
pub struct Memory(Arc<Ledger>);
struct Ledger {
    limit: usize,
    used: AtomicUsize,
    allocated: AtomicUsize,
    reserved: AtomicUsize,
    peak: AtomicUsize,
    allocation_count: AtomicUsize,
    allocation_bytes: AtomicUsize,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct MemoryStats {
    pub limit: usize,
    pub charged: usize,
    pub allocated: usize,
    pub reserved: usize,
    pub peak: usize,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Admission;
pub trait Footprint {
    fn heap_bytes(&self) -> usize {
        0
    }
}
impl Footprint for pivot_types::PivotDecimal {}
impl Memory {
    /// Research measurement boundary; call only after quiescing this domain.
    #[cfg(feature = "history-experiment")]
    pub fn history_reset_peak(&self) {
        self.0.peak.store(self.0.used.load(Relaxed), Relaxed);
    }
    pub fn new(limit: usize) -> Self {
        Self(Arc::new(Ledger {
            limit,
            used: AtomicUsize::new(0),
            allocated: AtomicUsize::new(0),
            reserved: AtomicUsize::new(0),
            peak: AtomicUsize::new(0),
            allocation_count: AtomicUsize::new(0),
            allocation_bytes: AtomicUsize::new(0),
        }))
    }
    /// Cumulative successful domain-accounted allocations, not all process allocations.
    pub fn allocation_traffic(&self) -> (usize, usize) {
        (
            self.0.allocation_count.load(Relaxed),
            self.0.allocation_bytes.load(Relaxed),
        )
    }
    pub fn stats(&self) -> MemoryStats {
        MemoryStats {
            limit: self.0.limit,
            charged: self.0.used.load(Relaxed),
            allocated: self.0.allocated.load(Relaxed),
            reserved: self.0.reserved.load(Relaxed),
            peak: self.0.peak.load(Relaxed),
        }
    }
    pub fn reserve(&self, bytes: usize) -> Result<Charge, Admission> {
        let _profile = crate::experiment_profile::construction::scope(8);
        let mut old = self.0.used.load(Relaxed);
        loop {
            let next = old
                .checked_add(bytes)
                .filter(|&n| n <= self.0.limit)
                .ok_or(Admission)?;
            match self
                .0
                .used
                .compare_exchange_weak(old, next, Relaxed, Relaxed)
            {
                Ok(_) => {
                    self.0.peak.fetch_max(next, Relaxed);
                    self.0.reserved.fetch_add(bytes, Relaxed);
                    return Ok(Charge {
                        memory: self.clone(),
                        bytes,
                        allocation: false,
                    });
                }
                Err(current) => old = current,
            }
        }
    }
    pub fn alloc<T: Footprint>(&self, value: T) -> Result<Shared<T>, Admission> {
        let bytes = std::mem::size_of::<Charged<T>>()
            + 2 * std::mem::size_of::<usize>()
            + value.heap_bytes();
        let mut charge = self.reserve(bytes)?;
        charge.allocation = true;
        self.0.reserved.fetch_sub(bytes, Relaxed);
        self.0.allocated.fetch_add(bytes, Relaxed);
        self.0.allocation_count.fetch_add(1, Relaxed);
        self.0.allocation_bytes.fetch_add(bytes, Relaxed);
        Ok(Shared(Arc::new(Charged {
            value,
            _charge: charge,
        })))
    }
}
pub struct Charge {
    memory: Memory,
    bytes: usize,
    allocation: bool,
}
impl Drop for Charge {
    fn drop(&mut self) {
        if self.allocation {
            self.memory.0.allocated.fetch_sub(self.bytes, Relaxed);
        } else {
            self.memory.0.reserved.fetch_sub(self.bytes, Relaxed);
        }
        self.memory.0.used.fetch_sub(self.bytes, Relaxed);
    }
}
struct Charged<T> {
    value: T,
    _charge: Charge,
}
pub struct Shared<T>(Arc<Charged<T>>);
impl<T> Clone for Shared<T> {
    fn clone(&self) -> Self {
        Self(self.0.clone())
    }
}
impl<T> Footprint for Shared<T> {}
impl<T> Deref for Shared<T> {
    type Target = T;
    fn deref(&self) -> &T {
        &self.0.value
    }
}
impl<T> Shared<T> {
    pub fn ptr_eq(a: &Self, b: &Self) -> bool {
        Arc::ptr_eq(&a.0, &b.0)
    }
    pub(super) fn get_mut(&mut self) -> Option<&mut T> {
        Arc::get_mut(&mut self.0).map(|v| &mut v.value)
    }
}

#[cfg(feature = "history-experiment")]
pub fn history_allocation_header<T>() -> usize {
    std::mem::size_of::<Charged<T>>() + 2 * std::mem::size_of::<usize>()
}
