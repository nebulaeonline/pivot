//! Controlled integration through WorkingModel, not a second model facade.
//! Explicit local capture, bounded inspection and complete standalone operations.
use super::*;
use crate::evaluation::{CalculationResult, EvaluationContext, EvaluationError};
use std::{any::Any, sync::mpsc, thread};
pub use structural::memory::{Memory as AllocationDomain, MemoryStats as AllocationStats};
use structural::{Calculation, Capture, ComputeError, Edit, Graph, Limits, memory::Memory};

#[doc(hidden)]
pub mod batch;
mod inspection;
mod reference;
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
enum State {
    Reference(ModelSnapshot),
    Draft(Arc<WorkingModel>),
    Persistent(PersistentCapture),
}
/// Calculation state is separate from the legacy full-inspection snapshot API.
pub struct CalculationCapture {
    revision: Revision,
    state: State,
}
enum ResultState {
    Reference(CalculationResult),
    Draft {
        value: CalculationResult,
        _capture: Arc<WorkingModel>,
    },
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
            ResultState::Draft { value, .. } => value.values(),
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
                State::Draft(s) => State::Draft(s.clone()),
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
        let _copy = match &self.state {
            State::Persistent(s) => Some(
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
            ),
            _ => None,
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
        if cancel.is_cancelled() {
            return Err(CalculationFailure::Cancelled);
        }
        let state = match &self.state {
            State::Draft(source) => ResultState::Draft {
                value: reference::evaluate(source, request, cancel)?,
                _capture: source.clone(),
            },
            State::Reference(s) => {
                ResultState::Reference(EvaluationContext::new(s.clone()).invoke(request).map_err(
                    |e| match e {
                        crate::evaluation::InvocationError::Evaluation(e) => {
                            CalculationFailure::Evaluation(e)
                        }
                        other => CalculationFailure::Invocation(other),
                    },
                )?)
            }
            State::Persistent(s) => {
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
                    .invoke_retiring_cancelled(
                        request,
                        |v| scratch.retire(v),
                        || cancel.is_cancelled(),
                    )
                    .map_err(compute_failure)?;
                if cancel.is_cancelled() {
                    result.retire(value);
                    return Err(CalculationFailure::Cancelled);
                }
                ResultState::Persistent(result.own(value))
            }
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
    /// One-time O(N) activation/import bridge. Existing bound tokens and local
    /// identities survive; no formulas are rebound. Intended for explicit reference
    /// fallback models. New standalone models start persistent without this bridge.
    /// Already-persistent models must belong to the same runtime; no rehoming.
    pub fn enable_persistent(&mut self, runtime: &IntegrationRuntime) -> Result<(), ModelError> {
        if let Some(current) = &self.integrated {
            return if Arc::ptr_eq(&current.runtime, runtime.state()) {
                Ok(())
            } else {
                Err(ModelError::Backend(
                    "model belongs to another runtime; construct with new_in",
                ))
            };
        }
        let state = runtime.state().clone();
        let root = state.cleanup.reserve().map_err(ModelError::Backend)?;
        let original = state.cleanup.reserve().map_err(ModelError::Backend)?;
        // Move the source into a shared rollback guard; no full source clone.
        let shell = WorkingModel {
            integrated: None,
            name: String::new(),
            revision: self.revision,
            nodes: BTreeMap::new(),
            names: BTreeMap::new(),
            next_node: self.next_node,
            next_port: self.next_port,
            interface: ModelInterface::default(),
        };
        let source = Arc::new(std::mem::replace(self, shell));
        let borrowed = source.clone();
        let memory = runtime.memory();
        let outcome = state
            .heavy
            .run(&state.cleanup, &CancellationToken::default(), move || {
                Graph::from_model_in(&borrowed, Limits::default(), memory).map(|mut g| {
                    g.enable_local_validation();
                    g
                })
            });
        match outcome {
            Ok(Ok(graph)) => {
                self.integrated = Some(Integrated {
                    graph: root.own(graph),
                    runtime: state.clone(),
                });
                original.retire(source);
            }
            other => {
                // A rejected queued task may still own an Arc to the source.
                // Activation is an explicit heavy operation; drain that cleanup
                // before restoring the exact original model on failure.
                state.cleanup.wait_payloads();
                *self = Arc::try_unwrap(source)
                    .unwrap_or_else(|_| unreachable!("activation source still borrowed"));
                return Err(match other {
                    Ok(Err(e)) => map_error(e),
                    Err(e) => ModelError::Backend(e),
                    _ => unreachable!(),
                });
            }
        }
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
    pub(super) fn integrated_request_charge(
        &self,
        bytes: usize,
    ) -> Result<structural::memory::Charge, ModelError> {
        self.integrated
            .as_ref()
            .unwrap()
            .graph()
            .memory()
            .reserve(bytes)
            .map_err(|_| ModelError::Backend("request copy admission"))
    }
    pub(super) fn integrated_add_port(
        &mut self,
        edit: structural::PortEdit,
    ) -> Result<PortId, ModelError> {
        let id = self
            .integrated
            .as_ref()
            .unwrap()
            .graph()
            .next_port_identity()?;
        self.integrated_edit(Edit::Port(edit))?;
        Ok(id)
    }
    pub(super) fn integrated_edit(&mut self, edit: Edit) -> Result<(), ModelError> {
        let i = self.integrated.as_mut().unwrap();
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
        let state = if let Some(i) = &self.integrated {
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
            State::Persistent(PersistentCapture {
                value: i
                    .runtime
                    .cleanup
                    .reserve()
                    .map_err(CaptureFailure::Resource)?
                    .own(capture),
                runtime: i.runtime.clone(),
            })
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
        if let Some(i) = &self.integrated {
            let ticket = i
                .runtime
                .cleanup
                .reserve()
                .map_err(CaptureFailure::Resource)?;
            let capture = i
                .graph()
                .capture_local()
                .map_err(|_| CaptureFailure::Resource("capture admission"))?;
            return Ok(CalculationCapture {
                revision: self.revision,
                state: State::Persistent(PersistentCapture {
                    value: ticket.own(capture),
                    runtime: i.runtime.clone(),
                }),
            });
        }
        Ok(CalculationCapture {
            revision: self.revision,
            state: State::Draft(Arc::new(self.stage())),
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
        let i = self.integrated.as_ref().unwrap();
        materialize_snapshot(i.graph().clone(), self.revision, &i.runtime, cancel)
    }

    /// Explicit full compatibility materialization, cooperative cancellation.
    pub fn snapshot_cancelled(
        &self,
        cancel: &CancellationToken,
    ) -> Result<ModelSnapshot, SnapshotError> {
        if self.integrated.is_some() {
            self.integrated_snapshot(cancel)
        } else {
            self.snapshot_cancel_check(|| cancel.is_cancelled())
        }
    }
    pub(super) fn integrated_export(
        &self,
        nodes: &[NodeId],
        ports: &[PortId],
    ) -> Result<transfer::Image, ModelError> {
        let i = self.integrated.as_ref().unwrap();
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
                let mut imported = transfer::import(image)?;
                let mut graph = Graph::from_model_in(&imported.model, Limits::default(), memory)
                    .map_err(map_error)?;
                graph.enable_local_validation();
                let shell = WorkingModel {
                    integrated: Some(Integrated {
                        graph: root.own(graph),
                        runtime,
                    }),
                    name: String::new(),
                    revision: imported.model.revision,
                    nodes: BTreeMap::new(),
                    names: BTreeMap::new(),
                    next_node: imported.model.next_node,
                    next_port: imported.model.next_port,
                    interface: ModelInterface::default(),
                };
                drop(std::mem::replace(&mut imported.model, shell));
                Ok(imported)
            })
            .map_err(ModelError::Backend)?
    }
}

enum NamedState {
    Reference(crate::interface::NamedCalculationResult),
    Draft {
        value: crate::interface::NamedCalculationResult,
        _capture: Arc<WorkingModel>,
    },
    Persistent(Owned<structural::NamedCalculation>),
}
pub struct CapturedNamedResult {
    revision: Revision,
    state: NamedState,
}
impl CapturedNamedResult {
    pub fn revision(&self) -> Revision {
        self.revision
    }
    pub fn outputs(
        &self,
    ) -> &BTreeMap<String, Result<pivot_types::PivotValue, crate::interface::PublicEvaluationError>>
    {
        match &self.state {
            NamedState::Reference(r) => r.outputs(),
            NamedState::Draft { value, .. } => value.outputs(),
            NamedState::Persistent(r) => r.get().outputs(),
        }
    }
    pub fn inputs(&self) -> &BTreeMap<String, pivot_types::PivotValue> {
        match &self.state {
            NamedState::Reference(r) => r.inputs(),
            NamedState::Draft { value, .. } => value.inputs(),
            NamedState::Persistent(r) => r.get().bindings(),
        }
    }
}
impl CapturedResult {
    pub fn input_overrides(&self) -> &BTreeMap<NodeId, PivotDecimal> {
        match &self.state {
            ResultState::Reference(r) => r.input_overrides(),
            ResultState::Draft { value, .. } => value.input_overrides(),
            ResultState::Persistent(r) => r.get().input_overrides(),
        }
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
        let state = match &self.state {
            State::Draft(source) => {
                source
                    .interface
                    .validate_kinds(|id| {
                        source
                            .nodes
                            .get(&id)
                            .map(|node| matches!(node.definition, Definition::Input(_)))
                    })
                    .map_err(CalculationFailure::Interface)?;
                let prepared = crate::interface::prepare_named(&source.interface, request)
                    .map_err(CalculationFailure::Interface)?;
                let value = reference::evaluate(source, &prepared.request, cancel)?;
                NamedState::Draft {
                    value: crate::interface::named_result(
                        value,
                        request,
                        &source.interface,
                        prepared.bindings,
                    ),
                    _capture: source.clone(),
                }
            }
            State::Reference(s) => NamedState::Reference(
                EvaluationContext::new(s.clone())
                    .invoke_named(request)
                    .map_err(CalculationFailure::Interface)?,
            ),
            State::Persistent(s) => {
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
                NamedState::Persistent(result.own(value))
            }
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
                .to_model_cancelled(|| token.is_cancelled())
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

#[cfg(test)]
mod cancellation_tests {
    use super::*;
    #[test]
    fn cancellation_after_partial_traversal_reclaims_scratch_and_preserves_s_t() {
        let runtime = IntegrationRuntime::new(64 * 1024 * 1024, 8).unwrap();
        let mut model = WorkingModel::new_reference("cancelled closure").unwrap();
        let nodes: Vec<_> = (0..1024)
            .map(|i| model.add_input(format!("n{i}"), 1.into()).unwrap())
            .collect();
        for (index, &id) in nodes.iter().enumerate().skip(1) {
            model
                .set_expression(id, format!("n{}+1", index - 1))
                .unwrap();
        }
        model.enable_persistent(&runtime).unwrap();
        let capture = model.capture_local().unwrap();
        let revision = model.revision();
        runtime.flush();
        let baseline = runtime.memory().stats().charged;
        let State::Persistent(state) = &capture.state else {
            unreachable!()
        };
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
