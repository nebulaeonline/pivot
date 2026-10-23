//! Controlled integration through WorkingModel, not a second model facade.
//! Explicit local capture, bounded inspection and complete standalone operations.
use super::*;
use crate::evaluation::EvaluationError;
use std::{any::Any, sync::mpsc, thread};
pub use structural::memory::{Memory as AllocationDomain, MemoryStats as AllocationStats};
use structural::{Calculation, Capture, ComputeError, Edit, Graph, Limits, memory::Memory};

#[doc(hidden)]
pub mod batch;
mod inspection;
#[doc(hidden)]
pub mod resources;
mod runtime;
mod standalone;
pub use inspection::{
    DefinitionView, EdgeCursor, EdgePage, InspectionCursor, InspectionPage, NodeView,
};
pub use runtime::{CancellationToken, IntegrationRuntime, RuntimeStats, WorkLaneStats};
use runtime::{Owned, RuntimeState, Ticket};
pub use standalone::RuntimeConfig;
pub(super) use standalone::create_default;
#[derive(Clone)]
pub(super) struct Integrated {
    graph: Owned<Graph>,
    runtime: Arc<RuntimeState>,
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
    Invocation(crate::evaluation::InvocationError),
    Interface(InterfaceError),
    LocalInvalid(SnapshotError),
    Cancelled,
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

#[derive(Clone)]
struct PersistentCapture {
    value: Owned<Capture>,
    runtime: Arc<RuntimeState>,
}

/// Calculation state is separate from the explicit complete-inspection snapshot API.
pub struct CalculationCapture {
    capture_id: u64,
    revision: Revision,
    state: PersistentCapture,
}

pub struct CapturedResult {
    revision: Revision,
    state: Owned<Calculation>,
}
impl CapturedResult {
    pub fn revision(&self) -> Revision {
        self.revision
    }
    pub fn values(&self) -> &BTreeMap<NodeId, Result<PivotDecimal, EvaluationError>> {
        self.state.get().values()
    }
}
impl Clone for CalculationCapture {
    fn clone(&self) -> Self {
        Self {
            capture_id: self.capture_id,
            revision: self.revision,
            state: self.state.clone(),
        }
    }
}
impl CalculationCapture {
    pub fn revision(&self) -> Revision {
        self.revision
    }
    /// Cycle-only validation for a staged document transaction; does not run numeric reducers.
    pub fn validate_acyclic(
        &self,
        outputs: &[NodeId],
        cancel: &CancellationToken,
    ) -> Result<(), CalculationFailure> {
        self.state
            .value
            .get()
            .validate_acyclic(outputs, || cancel.is_cancelled())
            .map_err(compute_failure)
    }
    pub fn evaluate(&self, outputs: &[NodeId]) -> Result<CapturedResult, CalculationFailure> {
        let _copy = {
            let s = &self.state;
            Some(
                s.value
                    .get()
                    .memory()
                    .reserve(
                        outputs
                            .len()
                            .saturating_mul(std::mem::size_of::<NodeId>())
                            .saturating_add(128),
                    )
                    .map_err(|_| CalculationFailure::Resource("request copy admission"))?,
            )
        };
        self.invoke(&crate::evaluation::InvocationRequest {
            inputs: Vec::new(),
            outputs: outputs.to_vec(),
        })
    }
    pub fn invoke(
        &self,
        request: &crate::evaluation::InvocationRequest,
    ) -> Result<CapturedResult, CalculationFailure> {
        self.invoke_cancelled(request, &CancellationToken::default())
    }
    pub fn invoke_cancelled(
        &self,
        request: &crate::evaluation::InvocationRequest,
        cancel: &CancellationToken,
    ) -> Result<CapturedResult, CalculationFailure> {
        self.invoke_captured(request, None, cancel)
    }
    fn invoke_captured(
        &self,
        request: &crate::evaluation::InvocationRequest,
        sources: Option<&ScalarContext>,
        cancel: &CancellationToken,
    ) -> Result<CapturedResult, CalculationFailure> {
        if sources.is_some_and(|s| s.capture != self.capture_id) {
            return Err(CalculationFailure::Invocation(
                crate::evaluation::InvocationError::CapturedContext(
                    crate::captured::ContextError::Snapshot,
                ),
            ));
        }
        if cancel.is_cancelled() {
            return Err(CalculationFailure::Cancelled);
        }
        let state = {
            let s = &self.state;
            let result = s
                .runtime
                .cleanup
                .reserve()
                .map_err(CalculationFailure::Resource)?;
            let scratch = s
                .runtime
                .cleanup
                .reserve()
                .map_err(CalculationFailure::Resource)?;
            let value = s
                .value
                .get()
                .invoke_sources_retiring_cancelled(
                    request,
                    sources.map(|s| &s.values),
                    |v| scratch.retire(v),
                    || cancel.is_cancelled(),
                )
                .map_err(compute_failure)?;
            if cancel.is_cancelled() {
                result.retire(value);
                return Err(CalculationFailure::Cancelled);
            }
            result.own(value)
        };
        Ok(CapturedResult {
            revision: self.revision,
            state,
        })
    }
}
fn compute_failure(e: ComputeError) -> CalculationFailure {
    match e {
        ComputeError::Invocation(crate::evaluation::InvocationError::Evaluation(e)) => {
            CalculationFailure::Evaluation(e)
        }
        ComputeError::Invocation(e) => CalculationFailure::Invocation(e),
        ComputeError::Invalid(e) => CalculationFailure::LocalInvalid(e),
        ComputeError::Cancelled => CalculationFailure::Cancelled,
        _ => CalculationFailure::Resource("calculation admission or work budget"),
    }
}

impl WorkingModel {
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
        let id = self.integrated.graph().next_identity();
        self.integrated_edit(match value {
            Some(v) => Edit::AddInput(name, v),
            None => Edit::AddFormula(name, source.unwrap()),
        })?;
        Ok(id)
    }
    pub(super) fn integrated_request_charge(
        &self,
        bytes: usize,
    ) -> Result<structural::memory::Charge, ModelError> {
        self.integrated
            .graph()
            .memory()
            .reserve(bytes)
            .map_err(|_| ModelError::Backend("request copy admission"))
    }
    pub(super) fn integrated_add_port(
        &mut self,
        edit: structural::PortEdit,
    ) -> Result<PortId, ModelError> {
        let id = self.integrated.graph().next_port_identity()?;
        self.integrated_edit(Edit::Port(edit))?;
        Ok(id)
    }
    pub(super) fn integrated_edit(&mut self, edit: Edit) -> Result<(), ModelError> {
        let i = &mut self.integrated;
        let state = i.runtime.clone();
        let root = state.cleanup.reserve().map_err(ModelError::Backend)?;
        let mut candidate = i.graph().clone();
        let before = candidate.version();
        let (candidate, outcome) = state
            .mutations
            .run(&state.cleanup, &CancellationToken::default(), move || {
                let outcome = candidate.apply(candidate.version(), vec![edit]);
                (candidate, outcome)
            })
            .map_err(ModelError::Backend)?;
        let after = candidate.version();
        if outcome.is_ok() {
            let next = if before != after {
                match self.revision.version.checked_add(1) {
                    Some(v) => v,
                    None => {
                        root.retire(candidate);
                        return Err(ModelError::IdentityExhausted);
                    }
                }
            } else {
                self.revision.version
            };
            i.graph = root.own(candidate);
            self.revision.version = next;
        } else {
            root.retire(candidate);
        }
        outcome.map_err(map_error)?;
        Ok(())
    }
    /// Locally efficient immutable capture for the integrated subset. Invalid
    /// drafts use the explicitly guarded O(N) compatibility diagnostic bridge,
    /// preserving the accepted full Kahn error and first-error ordering.
    pub fn capture_calculation(&self) -> Result<CalculationCapture, CaptureFailure> {
        let i = &self.integrated;
        let state = {
            if !i.graph().certified() {
                let graph = i.graph().clone();
                let diagnosis = i
                    .runtime
                    .heavy
                    .run(
                        &i.runtime.cleanup,
                        &CancellationToken::default(),
                        move || graph.diagnose(),
                    )
                    .map_err(CaptureFailure::Resource)?
                    .map_err(|_| CaptureFailure::Resource("full diagnostic admission"))?;
                if let Err(e) = diagnosis {
                    return Err(CaptureFailure::Invalid(e));
                }
                i.graph().certify();
            }
            let capture = i
                .graph()
                .capture_local()
                .map_err(|_| CaptureFailure::Resource("capture admission"))?;
            i.graph()
                .validate_ports()
                .map_err(CaptureFailure::Invalid)?;
            PersistentCapture {
                value: i
                    .runtime
                    .cleanup
                    .reserve()
                    .map_err(CaptureFailure::Resource)?
                    .own(capture),
                runtime: i.runtime.clone(),
            }
        };
        Ok(CalculationCapture {
            capture_id: next_capture_identity()?,
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
        structural::Error::Cancelled => ModelError::Backend("cancelled"),
        structural::Error::Exhausted => ModelError::IdentityExhausted,
        structural::Error::InvalidTransfer => {
            ModelError::InvalidName("invalid graph transfer".into())
        }
        _ => ModelError::Backend("persistent admission, quota or work budget"),
    }
}

impl WorkingModel {
    /// Interactive capture: unrelated drafts do not reject selected evaluation.
    /// Requested closures are checked during evaluation; full global diagnosis
    /// remains available through capture_calculation and snapshot.
    pub fn capture_local(&self) -> Result<CalculationCapture, CaptureFailure> {
        let i = &self.integrated;
        let ticket = i
            .runtime
            .cleanup
            .reserve()
            .map_err(CaptureFailure::Resource)?;
        let capture = i
            .graph()
            .capture_local()
            .map_err(|_| CaptureFailure::Resource("capture admission"))?;
        Ok(CalculationCapture {
            capture_id: next_capture_identity()?,
            revision: self.revision,
            state: PersistentCapture {
                value: ticket.own(capture),
                runtime: i.runtime.clone(),
            },
        })
    }
}

pub(super) struct SnapshotLease {
    ticket: Ticket,
    charge: structural::memory::Charge,
}
impl fmt::Debug for SnapshotLease {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("AdmittedSnapshot")
    }
}
impl SnapshotLease {
    pub(super) fn release<T: Send + 'static>(self, value: T) {
        self.ticket.retire((value, self.charge));
    }
}
impl WorkingModel {
    pub(super) fn integrated_snapshot(
        &self,
        cancel: &CancellationToken,
    ) -> Result<ModelSnapshot, SnapshotError> {
        let i = &self.integrated;
        materialize_snapshot(i.graph().clone(), self.revision, &i.runtime, cancel)
    }

    /// Explicit full compatibility materialization, cooperative cancellation.
    pub fn snapshot_cancelled(
        &self,
        cancel: &CancellationToken,
    ) -> Result<ModelSnapshot, SnapshotError> {
        self.integrated_snapshot(cancel)
    }
    pub(super) fn integrated_export_selections(
        &self,
        nodes: &[NodeId],
        ports: &[PortId],
    ) -> Result<transfer::SelectionImage, ModelError> {
        let i = &self.integrated;
        if nodes.len() > 8192 || ports.len() > 8192 || i.graph().live_count() > 4096 {
            return Err(ModelError::Backend("selection transfer limit"));
        }
        let _request = i
            .graph()
            .memory()
            .reserve(65536 + nodes.len().saturating_add(ports.len()) * 256)
            .map_err(|_| ModelError::Backend("selection transfer admission"))?;
        let graph = i.graph.clone();
        let nodes = nodes.to_vec();
        let ports = ports.to_vec();
        i.runtime
            .heavy
            .run(
                &i.runtime.cleanup,
                &CancellationToken::default(),
                move || {
                    let _copy = graph.get().copy_reservation()?;
                    graph.get().export_selections(&nodes, &ports)
                },
            )
            .map_err(ModelError::Backend)?
            .map_err(map_error)
    }
    pub(super) fn integrated_export(
        &self,
        nodes: &[NodeId],
        ports: &[PortId],
    ) -> Result<transfer::Image, ModelError> {
        let i = &self.integrated;
        if i.graph().live_count() > 4096 || nodes.len() > 8192 || ports.len() > 8192 {
            return Err(ModelError::InvalidName("invalid graph transfer".into()));
        }
        let _request = i
            .graph()
            .memory()
            .reserve(65536 + nodes.len().saturating_add(ports.len()).saturating_mul(256))
            .map_err(|_| ModelError::Backend("transfer ledger admission"))?;
        let graph = i.graph().clone();
        let nodes = nodes.to_vec();
        let ports = ports.to_vec();
        i.runtime
            .heavy
            .run(
                &i.runtime.cleanup,
                &CancellationToken::default(),
                move || {
                    let _copy = graph.copy_reservation()?;
                    graph.export(&nodes, &ports)
                },
            )
            .map_err(ModelError::Backend)?
            .map_err(map_error)
    }
}
impl IntegrationRuntime {
    pub fn import_portable(
        &self,
        image: transfer::Image,
        cancel: &CancellationToken,
    ) -> Result<transfer::Imported, ModelError> {
        transfer::check_header(&image)?;
        let state = self.state().clone();
        let root = state.cleanup.reserve().map_err(ModelError::Backend)?;
        let runtime = state.clone();
        let memory = self.memory();
        state
            .heavy
            .run(&state.cleanup, cancel, move || {
                // Existing transfer caps are checked by the one accepted importer.
                let mut bytes = 65536usize
                    .saturating_add(image.name.len().saturating_mul(4))
                    .saturating_add(
                        image
                            .node_slots
                            .saturating_add(image.port_slots)
                            .saturating_mul(256),
                    );
                for node in &image.nodes {
                    bytes = bytes
                        .saturating_add(node.name.len().saturating_mul(4))
                        .saturating_add(2048);
                    if let transfer::NodeDefinition::Computed { source, binding } = &node.definition
                    {
                        bytes = bytes
                            .saturating_add(source.len().saturating_mul(4))
                            .saturating_add(match binding {
                                transfer::Binding::Bound(tokens) => {
                                    tokens.len().saturating_mul(128)
                                }
                                transfer::Binding::Unresolved(name) => {
                                    name.len().saturating_mul(4).saturating_add(1024)
                                }
                                _ => 1024,
                            });
                    }
                }
                for port in &image.ports {
                    bytes = bytes
                        .saturating_add(port.name.len().saturating_mul(4))
                        .saturating_add(2048);
                }
                let _copy = memory
                    .reserve(bytes)
                    .map_err(|_| ModelError::Backend("transfer copy-in admission"))?;
                let imported = transfer::decode(image)?;
                let mut graph = Graph::from_image_in(&imported.model, Limits::default(), memory)
                    .map_err(map_error)?;
                graph.enable_local_validation();
                let model = WorkingModel {
                    integrated: Integrated {
                        graph: root.own(graph),
                        runtime,
                    },
                    revision: imported.model.revision,
                };
                Ok(transfer::Imported {
                    model,
                    nodes: imported.nodes,
                    ports: imported.ports,
                })
            })
            .map_err(ModelError::Backend)?
    }
}

pub struct CapturedNamedResult {
    revision: Revision,
    state: Owned<structural::NamedCalculation>,
}
impl CapturedNamedResult {
    pub fn revision(&self) -> Revision {
        self.revision
    }
    pub fn outputs(
        &self,
    ) -> &BTreeMap<String, Result<pivot_types::PivotValue, crate::interface::PublicEvaluationError>>
    {
        self.state.get().outputs()
    }
    pub fn inputs(&self) -> &BTreeMap<String, pivot_types::PivotValue> {
        self.state.get().bindings()
    }
}
impl CapturedResult {
    pub fn input_overrides(&self) -> &BTreeMap<NodeId, PivotDecimal> {
        self.state.get().input_overrides()
    }
}
impl CalculationCapture {
    pub fn invoke_named(
        &self,
        request: &crate::interface::NamedInvocationRequest,
    ) -> Result<CapturedNamedResult, CalculationFailure> {
        self.invoke_named_cancelled(request, &CancellationToken::default())
    }
    pub fn invoke_named_cancelled(
        &self,
        request: &crate::interface::NamedInvocationRequest,
        cancel: &CancellationToken,
    ) -> Result<CapturedNamedResult, CalculationFailure> {
        if cancel.is_cancelled() {
            return Err(CalculationFailure::Cancelled);
        }
        let state = {
            let s = &self.state;
            let result = s
                .runtime
                .cleanup
                .reserve()
                .map_err(CalculationFailure::Resource)?;
            let scratch = s
                .runtime
                .cleanup
                .reserve()
                .map_err(CalculationFailure::Resource)?;
            let value = s
                .value
                .get()
                .invoke_named_retiring(request, |v| scratch.retire(v), || cancel.is_cancelled())
                .map_err(|e| match e {
                    structural::NamedError::Interface(e) => CalculationFailure::Interface(e),
                    structural::NamedError::Compute(e) => compute_failure(e),
                })?;
            if cancel.is_cancelled() {
                result.retire(value);
                return Err(CalculationFailure::Cancelled);
            }
            result.own(value)
        };
        Ok(CapturedNamedResult {
            revision: self.revision,
            state,
        })
    }
}

fn materialize_snapshot(
    graph: Graph,
    revision: Revision,
    runtime: &Arc<RuntimeState>,
    cancel: &CancellationToken,
) -> Result<ModelSnapshot, SnapshotError> {
    let ticket = runtime.cleanup.reserve().map_err(SnapshotError::Backend)?;
    let token = cancel.clone();
    runtime
        .heavy
        .run(&runtime.cleanup, cancel, move || {
            let charge = graph
                .copy_reservation()
                .map_err(|_| SnapshotError::Backend("snapshot copy-out admission"))?;
            let model = graph
                .materialize_cancelled(|| token.is_cancelled())
                .map_err(|error| match error {
                    structural::Error::Cancelled => SnapshotError::Backend("cancelled"),
                    _ => SnapshotError::Backend("snapshot materialization admission"),
                })?;
            let mut snapshot = model.snapshot_cancel_check(|| token.is_cancelled())?;
            let data = Arc::get_mut(&mut snapshot.0).unwrap();
            data.revision = revision;
            data.cleanup = Some(SnapshotLease { ticket, charge });
            graph.certify();
            Ok(snapshot)
        })
        .map_err(SnapshotError::Backend)?
}

/// Immutable integration-owned context; carries its domain charge until final owner drop.
#[doc(hidden)]
pub struct ScalarContext {
    _snapshot: PersistentCapture,
    capture: u64,
    values: BTreeMap<crate::captured::CapturedScalarId, crate::captured::ScalarValue>,
    _charge: resources::Charge,
}
impl CalculationCapture {
    #[doc(hidden)]
    pub fn scalar_context(
        &self,
        entries: &[(
            crate::captured::CapturedScalarId,
            crate::captured::ScalarValue,
        )],
        diagnostics: &[crate::captured::CapturedFaultId],
    ) -> Result<ScalarContext, crate::captured::ContextError> {
        use crate::captured::{ContextError as E, ScalarValue};
        if entries.len() > 1 || diagnostics.len() > 1 {
            return Err(E::Duplicate);
        }
        let charge = self
            .state
            .value
            .get()
            .memory()
            .reserve(4096)
            .map_err(|_| E::Resource)?;
        let slots = self.state.value.get().captured_slots();
        let mut values = BTreeMap::new();
        for &(id, value) in entries {
            if !slots.contains(&id) {
                return Err(E::Unknown);
            }
            if values.insert(id, value).is_some() {
                return Err(E::Duplicate);
            }
        }
        if slots.len() != values.len() {
            return Err(E::Missing);
        }
        let faults: Vec<_> = values
            .values()
            .filter_map(|v| {
                if let ScalarValue::Fault(id) = v {
                    Some(*id)
                } else {
                    None
                }
            })
            .collect();
        if faults != diagnostics {
            return Err(E::Fault);
        }
        Ok(ScalarContext {
            _snapshot: self.state.clone(),
            capture: self.capture_id,
            values,
            _charge: charge,
        })
    }
    #[doc(hidden)]
    pub fn evaluate_scalar_context(
        &self,
        outputs: &[NodeId],
        context: &ScalarContext,
    ) -> Result<CapturedResult, CalculationFailure> {
        self.invoke_captured(
            &crate::evaluation::InvocationRequest {
                inputs: vec![],
                outputs: outputs.to_vec(),
            },
            Some(context),
            &CancellationToken::default(),
        )
    }
}

fn next_capture_identity() -> Result<u64, CaptureFailure> {
    static NEXT: std::sync::atomic::AtomicU64 = std::sync::atomic::AtomicU64::new(1);
    NEXT.try_update(
        std::sync::atomic::Ordering::Relaxed,
        std::sync::atomic::Ordering::Relaxed,
        |n| n.checked_add(1),
    )
    .map_err(|_| CaptureFailure::Resource("capture identity exhausted"))
}

#[cfg(test)]
mod cancellation_tests {
    use super::*;
    #[test]
    fn cancellation_after_partial_traversal_reclaims_scratch_and_preserves_s_t() {
        let runtime = IntegrationRuntime::new(64 * 1024 * 1024, 8).unwrap();
        let mut model = WorkingModel::new_in("cancelled closure", &runtime).unwrap();
        let nodes: Vec<_> = (0..1024)
            .map(|i| model.add_input(format!("n{i}"), 1.into()).unwrap())
            .collect();
        for (index, &id) in nodes.iter().enumerate().skip(1) {
            model
                .set_expression(id, format!("n{}+1", index - 1))
                .unwrap();
        }

        let capture = model.capture_local().unwrap();
        let revision = model.revision();
        runtime.flush();
        let baseline = runtime.memory().stats().charged;
        let state = &capture.state;
        let ticket = state.runtime.cleanup.reserve().unwrap();
        let polls = std::sync::atomic::AtomicUsize::new(0);
        let result = state.value.get().invoke_retiring_cancelled(
            &crate::evaluation::InvocationRequest {
                inputs: vec![],
                outputs: vec![nodes[1023]],
            },
            |value| ticket.retire(value),
            || polls.fetch_add(1, std::sync::atomic::Ordering::Relaxed) >= 64,
        );
        assert!(matches!(result, Err(ComputeError::Cancelled)));
        assert_eq!(model.revision(), revision);
        runtime.flush();
        assert_eq!(runtime.memory().stats().charged, baseline);
        model.set_input(nodes[0], 9.into()).unwrap();
        assert_eq!(
            capture.evaluate(&[nodes[1023]]).unwrap().values()[&nodes[1023]],
            Ok(1024.into())
        );
        let memory = runtime.memory();
        drop((model, capture));
        assert!(runtime.finish().is_ok());
        assert_eq!(memory.stats().charged, 0);
    }
}
