//! Immutable captures and coordinated in-memory publication. No transport API.
use super::*;
use pivot_core::{
    evaluation::{EvaluationContext, EvaluationError, InvocationRequest},
    model::{ModelSnapshot, NodeId, SnapshotDefinition},
};
use pivot_types::{DecimalError, PivotDecimal};
use std::{
    collections::{BTreeMap, BTreeSet},
    sync::{
        Arc,
        atomic::{AtomicUsize, Ordering},
    },
};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct CalculationLimits {
    pub captures: usize,
    pub inputs: usize,
    pub outputs: usize,
    pub work: usize,
    pub published_revisions: usize,
}
impl Default for CalculationLimits {
    fn default() -> Self {
        Self {
            captures: 32,
            inputs: 1024,
            outputs: 1024,
            work: 100_000,
            published_revisions: 256,
        }
    }
}
impl CalculationLimits {
    pub(super) fn validate(self) -> Result<(), Error> {
        if [
            self.captures,
            self.inputs,
            self.outputs,
            self.work,
            self.published_revisions,
        ]
        .contains(&0)
        {
            Err(Error::InvalidArgument)
        } else {
            Ok(())
        }
    }
}
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct SnapshotProvenance {
    pub working: WorkingRevision,
    pub snapshot: CalculationSnapshotId,
}
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ValidationFailure {
    pub revision: WorkingRevision,
    pub diagnostics: Vec<Diagnostic>,
}
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum CaptureError {
    Service(Error),
    Validation(ValidationFailure),
}
impl fmt::Display for CaptureError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{self:?}")
    }
}
impl std::error::Error for CaptureError {}
impl From<Error> for CaptureError {
    fn from(e: Error) -> Self {
        Self::Service(e)
    }
}
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ValidationReport {
    pub revision: WorkingRevision,
    pub diagnostics: Vec<Diagnostic>,
    pub valid: bool,
}
#[derive(Debug, Clone, Default)]
pub struct EvaluationRequest {
    pub inputs: Vec<(AuthoringNodeId, PivotDecimal)>,
    pub outputs: Vec<AuthoringNodeId>,
}
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum CalculationError {
    Arithmetic(DecimalError),
    DependencyFailed {
        dependency: AuthoringNodeId,
        origin: AuthoringNodeId,
    },
    InvalidExpression,
}
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EffectiveSource {
    Captured,
    Override,
}
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EffectiveInput {
    pub value: PivotDecimal,
    pub source: EffectiveSource,
}
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EvaluationResult {
    pub provenance: SnapshotProvenance,
    pub outputs: BTreeMap<AuthoringNodeId, Result<PivotDecimal, CalculationError>>,
    pub effective_inputs: BTreeMap<AuthoringNodeId, EffectiveInput>,
}
struct Lease(Arc<AtomicUsize>);
impl Drop for Lease {
    fn drop(&mut self) {
        self.0.fetch_sub(1, Ordering::AcqRel);
    }
}
/// Owns captured state; run evaluate after releasing the management-service lock.
/// No core snapshot or runtime handle escapes. Not implicitly cloneable.
pub struct CalculationCapture {
    provenance: SnapshotProvenance,
    snapshot: ModelSnapshot,
    nodes: BTreeMap<AuthoringNodeId, NodeId>,
    reverse: BTreeMap<NodeId, AuthoringNodeId>,
    limits: CalculationLimits,
    policy: Arc<dyn Authorizer>,
    principal: String,
    _lease: Lease,
}
impl CalculationCapture {
    pub fn provenance(&self) -> SnapshotProvenance {
        self.provenance
    }
    fn permitted(&self) -> Result<(), Error> {
        if self.policy.allowed(
            &self.principal,
            Action::Evaluate,
            Some(self.provenance.working.model),
        ) {
            Ok(())
        } else {
            Err(Error::Denied)
        }
    }
    pub fn evaluate(&self, request: EvaluationRequest) -> Result<EvaluationResult, Error> {
        self.permitted()?;
        if request.inputs.len() > self.limits.inputs || request.outputs.len() > self.limits.outputs
        {
            return Err(Error::WorkLimit);
        }
        if request.outputs.is_empty() {
            return Err(Error::InvalidOutput);
        }
        let mut inputs = Vec::new();
        let mut seen = BTreeSet::new();
        for &(id, value) in &request.inputs {
            let core = *self.nodes.get(&id).ok_or(Error::InvalidInput)?;
            if !seen.insert(id)
                || !matches!(
                    self.snapshot.node(core).unwrap().definition(),
                    SnapshotDefinition::Input(_)
                )
            {
                return Err(Error::InvalidInput);
            }
            inputs.push((core, value));
        }
        let mut outputs = Vec::new();
        let mut seen = BTreeSet::new();
        for &id in &request.outputs {
            if !seen.insert(id) {
                return Err(Error::InvalidOutput);
            }
            outputs.push(*self.nodes.get(&id).ok_or(Error::InvalidOutput)?);
        }
        let result = EvaluationContext::new(self.snapshot.clone())
            .invoke(&InvocationRequest { inputs, outputs })
            .map_err(|_| Error::InvalidArgument)?;
        let outputs = request
            .outputs
            .iter()
            .map(|&id| {
                let value = result
                    .value(self.nodes[&id])
                    .expect("selected output evaluated")
                    .clone()
                    .map_err(|e| match e {
                        EvaluationError::Arithmetic { error, .. } => {
                            CalculationError::Arithmetic(error)
                        }
                        EvaluationError::DependencyFailed {
                            dependency, origin, ..
                        } => CalculationError::DependencyFailed {
                            dependency: self.reverse[&dependency],
                            origin: self.reverse[&origin],
                        },
                        _ => CalculationError::InvalidExpression,
                    });
                (id, value)
            })
            .collect();
        let mut effective_inputs = BTreeMap::new();
        for (&id, &core) in &self.nodes {
            if let SnapshotDefinition::Input(default) =
                self.snapshot.node(core).unwrap().definition()
            {
                let submitted = request.inputs.iter().find(|(target, _)| *target == id);
                effective_inputs.insert(
                    id,
                    EffectiveInput {
                        value: submitted.map_or(*default, |(_, v)| *v),
                        source: if submitted.is_some() {
                            EffectiveSource::Override
                        } else {
                            EffectiveSource::Captured
                        },
                    },
                );
            }
        }
        self.permitted()?; // delayed result delivery obeys current policy too
        Ok(EvaluationResult {
            provenance: self.provenance,
            outputs,
            effective_inputs,
        })
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct PublishedModelRef {
    pub model: PublishedModelId,
    pub revision: PublishedRevisionId,
}
impl PublishedModelRef {
    pub fn wire(&self) -> pivot_protocol::v1::ModelRef {
        pivot_protocol::v1::ModelRef {
            model_id: self.model.to_string(),
            revision_id: self.revision.to_string(),
        }
    }
}
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PublishRequest {
    pub instance: ServiceInstanceId,
    pub model: WorkingModelId,
    pub expected: WorkingRevision,
    pub operation_id: String,
}
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PublicationResult {
    pub provenance: SnapshotProvenance,
    pub published: PublishedModelRef,
}
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PublicationError {
    Service(Error),
    Validation(ValidationFailure),
    RegistryRejected,
}
impl fmt::Display for PublicationError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{self:?}")
    }
}
impl std::error::Error for PublicationError {}
impl From<Error> for PublicationError {
    fn from(e: Error) -> Self {
        Self::Service(e)
    }
}

pub(super) struct PublicationReceipt {
    pub principal: String,
    pub request: PublishRequest,
    pub result: Result<PublicationResult, PublicationError>,
}
enum Preparation {
    Ready(CalculationCapture),
    Invalid(ValidationFailure),
    Replay,
}
/// Opaque, one-use proposal. Preparation does not reserve a receipt or publish.
pub struct PreparedPublication {
    principal: String,
    request: PublishRequest,
    state: Preparation,
}

impl WorkingModelService {
    pub fn published_host(&self) -> &crate::ModelHost {
        &self.published
    }
    pub(super) fn receipt_count(&self) -> usize {
        self.receipts.len() + self.edit_receipts.len() + self.publication_receipts.len()
    }
    pub(super) fn publication_operation_exists(&self, principal: &str, operation: &str) -> bool {
        self.publication_receipts
            .iter()
            .any(|r| r.principal == principal && r.request.operation_id == operation)
    }
    pub(super) fn publication_outcome(
        &self,
        instance: ServiceInstanceId,
        principal: &str,
        operation: &str,
    ) -> Result<Option<OperationOutcome>, Error> {
        if let Some(r) = self
            .publication_receipts
            .iter()
            .find(|r| r.principal == principal && r.request.operation_id == operation)
        {
            self.check(instance, principal, Action::Publish, Some(r.request.model))?;
            return Ok(Some(match &r.result {
                Ok(result) => OperationOutcome::Published(result.clone()),
                Err(e) => OperationOutcome::PublicationRejected(e.clone()),
            }));
        }
        Ok(None)
    }
    fn exact(&self, expected: WorkingRevision) -> Result<&Resource, Error> {
        let r = self
            .resources
            .iter()
            .find(|r| r.id == expected.model)
            .ok_or(Error::UnknownModel)?;
        if r.revision != expected {
            return Err(Error::RevisionConflict);
        }
        Ok(r)
    }
    pub fn validate_working(
        &self,
        principal: &str,
        expected: WorkingRevision,
    ) -> Result<ValidationReport, Error> {
        self.check(
            expected.instance,
            principal,
            Action::Validate,
            Some(expected.model),
        )?;
        let r = self.exact(expected)?;
        let valid = match r.graph.snapshot() {
            Ok(snapshot) => {
                self.published
                    .check_work(&snapshot)
                    .map_err(|_| Error::WorkLimit)?;
                true
            }
            Err(_) => false,
        };
        let diagnostics = if valid {
            Vec::new()
        } else {
            editing::draft(r).diagnostics
        };
        self.check(
            expected.instance,
            principal,
            Action::Validate,
            Some(expected.model),
        )?;
        Ok(ValidationReport {
            revision: expected,
            diagnostics,
            valid,
        })
    }
    fn capture(
        &self,
        principal: &str,
        expected: WorkingRevision,
    ) -> Result<CalculationCapture, CaptureError> {
        let r = self.exact(expected)?;
        let mut count = self.captures.load(Ordering::Acquire);
        loop {
            if count >= self.info.calculation_limits.captures {
                return Err(Error::CaptureLimit.into());
            }
            match self.captures.compare_exchange_weak(
                count,
                count + 1,
                Ordering::AcqRel,
                Ordering::Acquire,
            ) {
                Ok(_) => break,
                Err(current) => count = current,
            }
        }
        let lease = Lease(self.captures.clone());
        let snapshot = r.graph.snapshot().map_err(|_| {
            CaptureError::Validation(ValidationFailure {
                revision: expected,
                diagnostics: editing::draft(r).diagnostics,
            })
        })?;
        self.published
            .check_work(&snapshot)
            .map_err(|_| Error::WorkLimit)?;
        let nodes: BTreeMap<_, _> = r
            .identities
            .nodes
            .iter()
            .filter(|(_, core)| snapshot.node(**core).is_some())
            .map(|(&a, &b)| (a, b))
            .collect();
        let reverse = nodes.iter().map(|(&a, &b)| (b, a)).collect();
        Ok(CalculationCapture {
            provenance: SnapshotProvenance {
                working: expected,
                snapshot: CalculationSnapshotId::allocate(),
            },
            snapshot,
            nodes,
            reverse,
            limits: self.info.calculation_limits,
            policy: self.policy.clone(),
            principal: principal.into(),
            _lease: lease,
        })
    }
    pub fn capture_evaluation(
        &self,
        principal: &str,
        expected: WorkingRevision,
    ) -> Result<CalculationCapture, CaptureError> {
        self.check(
            expected.instance,
            principal,
            Action::Evaluate,
            Some(expected.model),
        )?;
        let capture = self.capture(principal, expected)?;
        self.check(
            expected.instance,
            principal,
            Action::Evaluate,
            Some(expected.model),
        )?;
        Ok(capture)
    }
    fn publication_replay(
        &self,
        principal: &str,
        request: &PublishRequest,
    ) -> Result<Option<Result<PublicationResult, PublicationError>>, Error> {
        if self
            .receipts
            .iter()
            .any(|r| r.principal == principal && r.operation == request.operation_id)
            || self
                .edit_receipts
                .iter()
                .any(|r| r.principal == principal && r.request.operation_id == request.operation_id)
        {
            return Err(Error::OperationIdConflict);
        }
        if let Some(r) = self
            .publication_receipts
            .iter()
            .find(|r| r.principal == principal && r.request.operation_id == request.operation_id)
        {
            if r.request != *request {
                return Err(Error::OperationIdConflict);
            }
            return Ok(Some(r.result.clone()));
        }
        Ok(None)
    }
    pub fn prepare_publication(
        &self,
        principal: &str,
        request: PublishRequest,
    ) -> Result<PreparedPublication, PublicationError> {
        self.check(
            request.instance,
            principal,
            Action::Publish,
            Some(request.model),
        )?;
        if !valid_operation(&request.operation_id)
            || request.expected.model != request.model
            || request.expected.instance != request.instance
        {
            return Err(Error::InvalidArgument.into());
        }
        let state = if self.publication_replay(principal, &request)?.is_some() {
            Preparation::Replay
        } else {
            if self.receipt_count() >= self.info.limits.receipts {
                return Err(Error::ReceiptLimit.into());
            }
            match self.capture(principal, request.expected) {
                Ok(capture) => Preparation::Ready(capture),
                Err(CaptureError::Validation(failure)) => Preparation::Invalid(failure),
                Err(CaptureError::Service(e)) => return Err(e.into()),
            }
        };
        Ok(PreparedPublication {
            principal: principal.into(),
            request,
            state,
        })
    }
    pub fn publish(
        &mut self,
        principal: &str,
        request: PublishRequest,
    ) -> Result<PublicationResult, PublicationError> {
        let prepared = self.prepare_publication(principal, request)?;
        self.commit_publication(principal, prepared)
    }
    pub fn commit_publication(
        &mut self,
        principal: &str,
        prepared: PreparedPublication,
    ) -> Result<PublicationResult, PublicationError> {
        let PreparedPublication {
            principal: owner,
            request,
            state,
        } = prepared;
        self.check(
            request.instance,
            principal,
            Action::Publish,
            Some(request.model),
        )?;
        if owner != principal {
            return Err(Error::Denied.into());
        }
        if let Some(result) = self.publication_replay(principal, &request)? {
            return result;
        }
        if self.receipt_count() >= self.info.limits.receipts {
            return Err(Error::ReceiptLimit.into());
        }
        self.publication_receipts
            .try_reserve(1)
            .map_err(|_| Error::AllocationFailed)?;
        let principal = principal.to_owned();
        let candidate = (|| {
            let r = self.exact(request.expected)?;
            let capture = match state {
                Preparation::Ready(capture) => capture,
                Preparation::Invalid(failure) => return Err(PublicationError::Validation(failure)),
                Preparation::Replay => return Err(Error::OperationIdConflict.into()), // receipt cannot disappear
            };
            if self.published.entries.len() >= self.info.calculation_limits.published_revisions {
                return Err(Error::PublishedLimit.into());
            }
            let published = PublishedModelRef {
                model: r.lineage,
                revision: PublishedRevisionId::allocate(),
            };
            let port_reverse: BTreeMap<_, _> =
                r.identities.ports.iter().map(|(&a, &b)| (b, a)).collect();
            let ports = capture
                .snapshot
                .interface()
                .inputs()
                .map(|p| {
                    (
                        p.name().to_owned(),
                        format!("port-{}", port_reverse[&p.id()]),
                    )
                })
                .chain(capture.snapshot.interface().outputs().map(|p| {
                    (
                        p.name().to_owned(),
                        format!("port-{}", port_reverse[&p.id()]),
                    )
                }))
                .collect();
            let result = PublicationResult {
                provenance: capture.provenance,
                published,
            };
            let registry = self
                .published
                .prepare_registration(published.wire(), capture.snapshot.clone(), ports)
                .map_err(|_| PublicationError::RegistryRejected)?;
            self.check(
                request.instance,
                &principal,
                Action::Publish,
                Some(request.model),
            )?;
            self.exact(request.expected)?; // final head comparison, after registry preparation
            Ok((registry, result))
        })();
        let result = candidate
            .as_ref()
            .map(|(_, r)| r.clone())
            .map_err(Clone::clone);
        let receipt = PublicationReceipt {
            principal,
            request,
            result: result.clone(),
        };
        // Exclusive access to BOTH registry and ledger. No callbacks or fallible
        // operations between installation and receipt insertion. Crash loses both.
        if let Ok((registry, _)) = candidate {
            self.published.install_registration(registry);
        }
        self.publication_receipts.push(receipt);
        result
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    struct Permit;
    impl Authorizer for Permit {
        fn allowed(&self, _: &str, _: Action, _: Option<WorkingModelId>) -> bool {
            true
        }
    }
    #[test]
    fn registry_rejection_is_recorded_without_installing_the_candidate() {
        let mut s = WorkingModelService::new(Limits::default(), Box::new(Permit)).unwrap();
        let instance = s.info.instance;
        let c = s
            .create(
                "a",
                CreateRequest {
                    instance,
                    operation_id: "create".into(),
                    name: "Model".into(),
                },
            )
            .unwrap();
        let edited = s
            .apply_edits(
                "a",
                ApplyEdits {
                    instance,
                    model: c.id,
                    expected: c.revision,
                    operation_id: "edit".into(),
                    edits: vec![
                        Edit::AddNode {
                            temporary: "n".into(),
                            name: "Value".into(),
                            definition: NodeDefinition::Input(
                                PivotDecimal::from_parts(1, 0, 0, false, 0).unwrap(),
                            ),
                        },
                        Edit::AddOutputPort {
                            temporary: "p".into(),
                            name: "Output".into(),
                            node: NodeRef::Temporary("n".into()),
                        },
                    ],
                },
            )
            .unwrap();
        // Inject a preexisting incompatible lineage via the private registry to
        // exercise a real registration failure, not a mocked insert outcome.
        let lineage = s.resources[0].lineage;
        let mut foreign = WorkingModel::new("Foreign").unwrap();
        let n = foreign
            .add_input(
                "Other",
                PivotDecimal::from_parts(2, 0, 0, false, 0).unwrap(),
            )
            .unwrap();
        foreign.add_output_port("Output", n).unwrap();
        s.published
            .register(
                pivot_protocol::v1::ModelRef {
                    model_id: lineage.to_string(),
                    revision_id: "incompatible".into(),
                },
                foreign.snapshot().unwrap(),
                BTreeMap::from([("Output".into(), format!("port-{}", edited.ports["p"]))]),
            )
            .unwrap();
        let before = s.published.entries.len();
        let request = PublishRequest {
            instance,
            model: c.id,
            expected: edited.revision,
            operation_id: "failed-publish".into(),
        };
        assert_eq!(
            s.publish("a", request.clone()),
            Err(PublicationError::RegistryRejected)
        );
        assert_eq!(s.published.entries.len(), before);
        assert_eq!(s.publication_receipts.len(), 1);
        assert_eq!(
            s.publish("a", request),
            Err(PublicationError::RegistryRejected)
        );
        assert_eq!(s.published.entries.len(), before);
        assert_eq!(
            s.operation_outcome(instance, "a", "failed-publish"),
            Ok(OperationOutcome::PublicationRejected(
                PublicationError::RegistryRejected
            ))
        );
        assert_eq!(s.resources[0].revision, edited.revision);
    }
}
