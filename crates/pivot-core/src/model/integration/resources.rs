//! Internal shared projection resources: canonical indexes and cleanup ownership.
use super::*;
use std::ops::{Deref, DerefMut};
pub use structural::memory::{Admission, Charge, Footprint, Memory, Shared};
pub use structural::{Index, Key};

pub struct Retained<T: Send + 'static>(pub(super) Owned<T>);
impl<T: Send + 'static> Clone for Retained<T> {
    fn clone(&self) -> Self {
        Self(self.0.clone())
    }
}
impl<T: Send + 'static> Deref for Retained<T> {
    type Target = T;
    fn deref(&self) -> &T {
        self.0.get()
    }
}
pub struct Reclaimable<T: Send + 'static> {
    value: Option<T>,
    ticket: Option<Ticket>,
}
impl<T: Send + 'static> Reclaimable<T> {
    pub(super) fn new(value: T, ticket: Ticket) -> Self {
        Self {
            value: Some(value),
            ticket: Some(ticket),
        }
    }
    pub fn freeze(mut self) -> Retained<T> {
        Retained(self.ticket.take().unwrap().own(self.value.take().unwrap()))
    }
    pub(super) fn into_inner(mut self) -> T {
        self.value.take().unwrap()
    }
}
impl<T: Send + 'static> Deref for Reclaimable<T> {
    type Target = T;
    fn deref(&self) -> &T {
        self.value.as_ref().unwrap()
    }
}
impl<T: Send + 'static> DerefMut for Reclaimable<T> {
    fn deref_mut(&mut self) -> &mut T {
        self.value.as_mut().unwrap()
    }
}
impl<T: Send + 'static> Drop for Reclaimable<T> {
    fn drop(&mut self) {
        if let Some(value) = self.value.take() {
            self.ticket.take().unwrap().retire(value);
        }
    }
}
impl WorkingModel {
    pub fn prepare_resource<T: Send + 'static>(
        &self,
        value: T,
    ) -> Result<Reclaimable<T>, ModelError> {
        let i = self
            .integrated
            .as_ref()
            .ok_or(ModelError::Backend("persistent model required"))?;
        let ticket = i.runtime.cleanup.reserve().map_err(ModelError::Backend)?;
        Ok(Reclaimable::new(value, ticket))
    }

    pub fn allocation_domain(&self) -> Result<Memory, ModelError> {
        self.integrated
            .as_ref()
            .map(|i| i.graph().memory())
            .ok_or(ModelError::Backend("persistent model required"))
    }
    pub fn retain_resource<T: Send + 'static>(&self, value: T) -> Result<Retained<T>, ModelError> {
        let i = self
            .integrated
            .as_ref()
            .ok_or(ModelError::Backend("persistent model required"))?;
        Ok(Retained(
            i.runtime
                .cleanup
                .reserve()
                .map_err(ModelError::Backend)?
                .own(value),
        ))
    }
    pub fn stage_resource<T: Clone + Send + 'static>(
        &self,
        value: &Retained<T>,
    ) -> Result<Reclaimable<T>, ModelError> {
        let i = self
            .integrated
            .as_ref()
            .ok_or(ModelError::Backend("persistent model required"))?;
        let ticket = i.runtime.cleanup.reserve().map_err(ModelError::Backend)?;
        Ok(Reclaimable::new((**value).clone(), ticket))
    }
}
