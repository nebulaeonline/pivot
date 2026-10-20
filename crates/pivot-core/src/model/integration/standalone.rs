//! Standalone selection; no model registry and no runtime per model.
use super::*;
use std::sync::{Mutex, OnceLock};

/// Conservative admission policy, not an operating-system RSS cap.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct RuntimeConfig {
    pub allocation_budget: usize,
    pub queue_capacity: usize,
}
impl Default for RuntimeConfig {
    fn default() -> Self {
        Self {
            allocation_budget: 2 * 1024 * 1024 * 1024,
            queue_capacity: 64,
        }
    }
}
impl IntegrationRuntime {
    /// Application/session-owned runtime with ordinary defaults.
    pub fn standalone() -> Result<Self, ModelError> {
        Self::configured(RuntimeConfig::default())
    }
    pub fn configured(config: RuntimeConfig) -> Result<Self, ModelError> {
        Self::new(config.allocation_budget, config.queue_capacity)
    }
    /// Start directly on persistent storage. Multiple models share this runtime's
    /// bounded lanes and aggregate allocation domain. No legacy activation occurs.
    pub fn create_model(&self, name: impl Into<String>) -> Result<WorkingModel, ModelError> {
        let state = self.state().clone();
        let ticket = state.cleanup.reserve().map_err(ModelError::Backend)?;
        let revision = fresh_revision()?;
        let graph = Graph::empty(name.into(), revision, self.memory()).map_err(map_error)?;
        Ok(WorkingModel {
            integrated: Integrated {
                graph: ticket.own(graph),
                runtime: state,
            },
            revision,
        })
    }
}
impl WorkingModel {
    /// Preferred application-owned standalone construction.
    pub fn new_in(
        name: impl Into<String>,
        runtime: &IntegrationRuntime,
    ) -> Result<Self, ModelError> {
        runtime.create_model(name)
    }
}
// Only shared execution resources, never a model registry.
static CONVENIENCE: OnceLock<Mutex<Option<IntegrationRuntime>>> = OnceLock::new();
impl IntegrationRuntime {
    /// Inspect, flush or recover the convenience runtime. The callback holds its
    /// control lock: do not recursively construct a default model from it.
    /// Applications needing custom policy should own a runtime instead.
    pub fn with_convenience<T>(f: impl FnOnce(&mut Self) -> T) -> Result<T, ModelError> {
        let mut slot = CONVENIENCE
            .get_or_init(|| Mutex::new(None))
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        if slot.is_none() {
            *slot = Some(Self::standalone()?);
        }
        Ok(f(slot.as_mut().unwrap()))
    }
    /// Deterministic teardown when no default models/captures/results remain.
    /// Refusal retains the runtime. A later default constructor may initialize
    /// a fresh runtime after successful shutdown.
    pub fn shutdown_convenience() -> Result<(), ModelError> {
        let Some(control) = CONVENIENCE.get() else {
            return Ok(());
        };
        let mut slot = control
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        let Some(runtime) = slot.take() else {
            return Ok(());
        };
        match runtime.finish() {
            Ok(()) => Ok(()),
            Err(runtime) => {
                *slot = Some(runtime);
                Err(ModelError::Backend(
                    "convenience runtime still in use or requires recovery",
                ))
            }
        }
    }
}
pub(in crate::model) fn create_default(
    name: impl Into<String>,
) -> Result<WorkingModel, ModelError> {
    IntegrationRuntime::with_convenience(|r| r.create_model(name))?
}
