//! In-process authoring foundation. Caller-supplied principals must come from a
//! trusted authentication boundary. No connection owns a resource. Exclusive
//! access serializes commits; an embedder can put this service behind a mutex.
use pivot_core::model::WorkingModel;
use std::fmt;
use uuid::Uuid;

macro_rules! portable_id {
    ($name:ident) => {
        #[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
        pub struct $name(Uuid);
        impl $name {
            fn allocate() -> Self {
                Self(Uuid::new_v4())
            }
        }
        impl fmt::Display for $name {
            fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
                self.0.fmt(f)
            }
        }
        impl std::str::FromStr for $name {
            type Err = uuid::Error;
            fn from_str(value: &str) -> Result<Self, Self::Err> {
                value.parse().map(Self)
            }
        }
    };
}
portable_id!(ServiceInstanceId);
portable_id!(WorkingModelId);
portable_id!(AttachmentId);
portable_id!(AuthoringNodeId);
portable_id!(AuthoringPortId);
portable_id!(CalculationSnapshotId);
portable_id!(PublishedModelId);
portable_id!(PublishedRevisionId);
mod editing;
pub use editing::*;
mod calculation;
pub use calculation::*;

/// Distinct from core revisions, calculation snapshots and published revisions.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct WorkingRevision {
    pub instance: ServiceInstanceId,
    pub model: WorkingModelId,
    pub sequence: u64,
}
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Durability {
    MemoryOnly,
}
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Limits {
    pub resources: usize,
    pub attachments: usize,
    pub receipts: usize,
}
impl Default for Limits {
    fn default() -> Self {
        Self {
            resources: 128,
            attachments: 256,
            receipts: 1024,
        }
    }
}
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ServiceInfo {
    pub instance: ServiceInstanceId,
    pub durability: Durability,
    pub limits: Limits,
    pub edit_limits: EditLimits,
    pub calculation_limits: CalculationLimits,
}
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Action {
    Create,
    List,
    Inspect,
    Open,
    Close,
    Edit,
    Recover,
    Validate,
    Evaluate,
    Publish,
}
/// Separate from invocation authorization. Identity/attachment IDs confer no
/// permission. Policy must support checks at request and commit boundaries.
pub trait Authorizer: Send + Sync {
    fn allowed(&self, principal: &str, action: Action, model: Option<WorkingModelId>) -> bool;
}
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Error {
    InvalidArgument,
    StaleInstance,
    Denied,
    UnknownModel,
    InvalidAttachment,
    ResourceLimit,
    AttachmentLimit,
    ReceiptLimit,
    AllocationFailed,
    OperationIdConflict,
    CoreInitializationFailed,
    RevisionConflict,
    InvalidEdit,
    ModelLimit,
    BatchLimit,
    IdentityExhausted,
    CaptureLimit,
    WorkLimit,
    PublishedLimit,
    InvalidInput,
    InvalidOutput,
}
impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{self:?}")
    }
}
impl std::error::Error for Error {}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Description {
    pub id: WorkingModelId,
    pub name: String,
    pub revision: WorkingRevision,
    pub node_count: usize,
    pub input_count: usize,
    pub output_count: usize,
}
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Created {
    pub id: WorkingModelId,
    pub revision: WorkingRevision,
}
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CreateRequest {
    pub instance: ServiceInstanceId,
    pub operation_id: String,
    pub name: String,
}
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum OperationOutcome {
    Unknown,
    Committed(Created),
    Rejected(Error),
    Edited(EditResult),
    Published(PublicationResult),
    PublicationRejected(PublicationError),
}
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Attachment {
    pub id: AttachmentId,
    pub model: WorkingModelId,
    pub revision: WorkingRevision,
}
struct Resource {
    id: WorkingModelId,
    graph: WorkingModel,
    revision: WorkingRevision,
    identities: editing::Identities,
    lineage: PublishedModelId,
}
struct Receipt {
    principal: String,
    operation: String,
    name: String,
    result: Result<Created, Error>,
}
struct HeldAttachment {
    attachment: Attachment,
    principal: String,
}

pub struct WorkingModelService {
    info: ServiceInfo,
    policy: std::sync::Arc<dyn Authorizer>,
    resources: Vec<Resource>,
    receipts: Vec<Receipt>,
    edit_receipts: Vec<editing::EditReceipt>,
    attachments: Vec<HeldAttachment>,
    publication_receipts: Vec<calculation::PublicationReceipt>,
    published: crate::ModelHost,
    captures: std::sync::Arc<std::sync::atomic::AtomicUsize>,
}
impl WorkingModelService {
    pub fn new(limits: Limits, policy: Box<dyn Authorizer>) -> Result<Self, Error> {
        Self::with_edit_limits(limits, EditLimits::default(), policy)
    }
    pub fn with_edit_limits(
        limits: Limits,
        edit_limits: EditLimits,
        policy: Box<dyn Authorizer>,
    ) -> Result<Self, Error> {
        Self::with_all_limits(limits, edit_limits, CalculationLimits::default(), policy)
    }
    pub fn with_all_limits(
        limits: Limits,
        edit_limits: EditLimits,
        calculation_limits: CalculationLimits,
        policy: Box<dyn Authorizer>,
    ) -> Result<Self, Error> {
        if limits.resources == 0 || limits.attachments == 0 || limits.receipts == 0 {
            return Err(Error::InvalidArgument);
        }
        edit_limits.validate()?;
        calculation_limits.validate()?;
        let instance = ServiceInstanceId::allocate();
        let mut published =
            crate::ModelHost::new(instance.to_string()).map_err(|_| Error::InvalidArgument)?;
        published
            .set_work_budget(calculation_limits.work)
            .map_err(|_| Error::InvalidArgument)?;
        Ok(Self {
            info: ServiceInfo {
                instance,
                durability: Durability::MemoryOnly,
                limits,
                edit_limits,
                calculation_limits,
            },
            policy: policy.into(),
            resources: Vec::new(),
            receipts: Vec::new(),
            edit_receipts: Vec::new(),
            attachments: Vec::new(),
            publication_receipts: Vec::new(),
            published,
            captures: std::sync::Arc::new(std::sync::atomic::AtomicUsize::new(0)),
        })
    }
    pub fn info(&self) -> ServiceInfo {
        self.info
    }
    fn context(&self, instance: ServiceInstanceId, principal: &str) -> Result<(), Error> {
        if instance != self.info.instance {
            return Err(Error::StaleInstance);
        }
        if principal.is_empty() || principal.len() > 256 {
            return Err(Error::InvalidArgument);
        }
        Ok(())
    }
    fn check(
        &self,
        instance: ServiceInstanceId,
        principal: &str,
        action: Action,
        model: Option<WorkingModelId>,
    ) -> Result<(), Error> {
        self.context(instance, principal)?;
        if !self.policy.allowed(principal, action, model) {
            return Err(Error::Denied);
        }
        Ok(())
    }
    pub fn create(&mut self, principal: &str, request: CreateRequest) -> Result<Created, Error> {
        self.create_prepared(principal, request, |name| {
            WorkingModel::new(name).map_err(|_| Error::CoreInitializationFailed)
        })
    }
    fn create_prepared(
        &mut self,
        principal: &str,
        request: CreateRequest,
        prepare: impl FnOnce(&str) -> Result<WorkingModel, Error>,
    ) -> Result<Created, Error> {
        self.check(request.instance, principal, Action::Create, None)?;
        if !valid_operation(&request.operation_id)
            || request.name.is_empty()
            || request.name.len() > 256
        {
            return Err(Error::InvalidArgument);
        }
        // Replay lookup precedes quotas, allocation and any future revision checks.
        if self.publication_operation_exists(principal, &request.operation_id) {
            return Err(Error::OperationIdConflict);
        }
        if self
            .edit_receipts
            .iter()
            .any(|r| r.principal == principal && r.request.operation_id == request.operation_id)
        {
            return Err(Error::OperationIdConflict);
        }
        if let Some(receipt) = self
            .receipts
            .iter()
            .find(|r| r.principal == principal && r.operation == request.operation_id)
        {
            return if receipt.name == request.name {
                receipt.result
            } else {
                Err(Error::OperationIdConflict)
            };
        }
        if self.receipt_count() >= self.info.limits.receipts {
            return Err(Error::ReceiptLimit);
        }
        self.receipts
            .try_reserve(1)
            .map_err(|_| Error::AllocationFailed)?;
        let mut receipt = Receipt {
            principal: principal.to_owned(),
            operation: request.operation_id,
            name: request.name,
            result: Err(Error::CoreInitializationFailed),
        };
        let candidate = (|| {
            if self.resources.len() >= self.info.limits.resources {
                return Err(Error::ResourceLimit);
            }
            self.resources
                .try_reserve(1)
                .map_err(|_| Error::AllocationFailed)?;
            let graph = prepare(&receipt.name)?;
            let id = WorkingModelId::allocate();
            let revision = WorkingRevision {
                instance: self.info.instance,
                model: id,
                sequence: 0,
            };
            self.check(request.instance, principal, Action::Create, None)?;
            Ok(Resource {
                id,
                graph,
                revision,
                identities: editing::Identities::default(),
                lineage: PublishedModelId::allocate(),
            })
        })();
        receipt.result = candidate
            .as_ref()
            .map(|r| Created {
                id: r.id,
                revision: r.revision,
            })
            .map_err(|e| *e);
        let result = receipt.result;
        // Both vectors have reserved capacity. No callbacks, fallible work or
        // observers inside this exclusive logical commit. Process crashes lose both.
        if let Ok(resource) = candidate {
            self.resources.push(resource);
        }
        self.receipts.push(receipt);
        result
    }
    /// Unknown is not proof an earlier request was never sent. Retry only the
    /// same operation/intent against this instance; never mint a fresh ID blindly.
    pub fn operation_outcome(
        &self,
        instance: ServiceInstanceId,
        principal: &str,
        operation: &str,
    ) -> Result<OperationOutcome, Error> {
        self.context(instance, principal)?;
        if !valid_operation(operation) {
            return Err(Error::InvalidArgument);
        }
        if let Some(result) = self.publication_outcome(instance, principal, operation)? {
            return Ok(result);
        }
        if let Some(r) = self
            .edit_receipts
            .iter()
            .find(|r| r.principal == principal && r.request.operation_id == operation)
        {
            self.check(instance, principal, Action::Edit, Some(r.request.model))?;
            return Ok(match &r.result {
                Ok(result) => OperationOutcome::Edited(result.clone()),
                Err(e) => OperationOutcome::Rejected(*e),
            });
        }
        if self
            .receipts
            .iter()
            .any(|r| r.principal == principal && r.operation == operation)
        {
            self.check(instance, principal, Action::Create, None)?;
        } else {
            self.check(instance, principal, Action::Recover, None)?;
        }
        Ok(
            match self
                .receipts
                .iter()
                .find(|r| r.principal == principal && r.operation == operation)
            {
                None => OperationOutcome::Unknown,
                Some(r) => match r.result {
                    Ok(c) => OperationOutcome::Committed(c),
                    Err(e) => OperationOutcome::Rejected(e),
                },
            },
        )
    }
    pub fn list(
        &self,
        instance: ServiceInstanceId,
        principal: &str,
    ) -> Result<Vec<Description>, Error> {
        self.check(instance, principal, Action::List, None)?;
        Ok(self
            .resources
            .iter()
            .filter(|r| self.policy.allowed(principal, Action::Inspect, Some(r.id)))
            .map(describe)
            .collect())
    }
    pub fn inspect(
        &self,
        instance: ServiceInstanceId,
        principal: &str,
        id: WorkingModelId,
    ) -> Result<Description, Error> {
        self.check(instance, principal, Action::Inspect, Some(id))?;
        self.resources
            .iter()
            .find(|r| r.id == id)
            .map(describe)
            .ok_or(Error::UnknownModel)
    }
    pub fn open(
        &mut self,
        instance: ServiceInstanceId,
        principal: &str,
        model: WorkingModelId,
    ) -> Result<Attachment, Error> {
        self.check(instance, principal, Action::Open, Some(model))?;
        let resource = self
            .resources
            .iter()
            .find(|r| r.id == model)
            .ok_or(Error::UnknownModel)?;
        if self.attachments.len() >= self.info.limits.attachments {
            return Err(Error::AttachmentLimit);
        }
        self.attachments
            .try_reserve(1)
            .map_err(|_| Error::AllocationFailed)?;
        let attachment = Attachment {
            id: AttachmentId::allocate(),
            model,
            revision: resource.revision,
        };
        self.attachments.push(HeldAttachment {
            attachment,
            principal: principal.into(),
        });
        Ok(attachment)
    }
    pub fn close(
        &mut self,
        instance: ServiceInstanceId,
        principal: &str,
        id: AttachmentId,
    ) -> Result<(), Error> {
        // Check instance/identity before any attachment lookup; never permit another principal's close.
        self.context(instance, principal)?;
        let index = self
            .attachments
            .iter()
            .position(|a| a.attachment.id == id && a.principal == principal)
            .ok_or(Error::InvalidAttachment)?;
        self.check(
            instance,
            principal,
            Action::Close,
            Some(self.attachments[index].attachment.model),
        )?;
        self.attachments.swap_remove(index);
        Ok(())
    }
}
fn valid_operation(value: &str) -> bool {
    !value.is_empty()
        && value.len() <= 128
        && value
            .bytes()
            .all(|b| b.is_ascii_alphanumeric() || b"._-".contains(&b))
}
fn describe(resource: &Resource) -> Description {
    Description {
        id: resource.id,
        name: resource.graph.name().into(),
        revision: resource.revision,
        node_count: resource.graph.draft_nodes().count(),
        input_count: resource.graph.interface().inputs().count(),
        output_count: resource.graph.interface().outputs().count(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::{
        Arc,
        atomic::{AtomicBool, Ordering},
    };

    struct Policy {
        deny: Arc<AtomicBool>,
    }
    impl Authorizer for Policy {
        fn allowed(&self, principal: &str, _: Action, _: Option<WorkingModelId>) -> bool {
            principal != "outsider" && !self.deny.load(Ordering::SeqCst)
        }
    }
    fn service(limits: Limits) -> WorkingModelService {
        WorkingModelService::new(
            limits,
            Box::new(Policy {
                deny: Arc::new(AtomicBool::new(false)),
            }),
        )
        .unwrap()
    }
    fn request(s: &WorkingModelService, op: &str, name: &str) -> CreateRequest {
        CreateRequest {
            instance: s.info().instance,
            operation_id: op.into(),
            name: name.into(),
        }
    }

    #[test]
    fn discovery_multiple_attachments_and_disconnect_do_not_own_model() {
        let mut s = service(Limits::default());
        let info = s.info();
        assert_eq!(info.durability, Durability::MemoryOnly);
        let created = s
            .create("alice", request(&s, "create-1", "Budget"))
            .unwrap();
        let a = s.open(info.instance, "alice", created.id).unwrap();
        let b = s.open(info.instance, "bob", created.id).unwrap();
        assert_ne!(a.id, b.id);
        assert_eq!(a.revision, b.revision);
        assert_eq!(
            s.close(info.instance, "bob", a.id),
            Err(Error::InvalidAttachment)
        );
        s.close(info.instance, "alice", a.id).unwrap();
        s.close(info.instance, "bob", b.id).unwrap();
        assert_eq!(
            s.close(info.instance, "bob", b.id),
            Err(Error::InvalidAttachment)
        );
        let listed = s.list(info.instance, "alice").unwrap();
        assert_eq!(listed.len(), 1);
        assert_eq!(
            listed[0],
            s.inspect(info.instance, "alice", created.id).unwrap()
        );
        assert_eq!(listed[0].revision, created.revision);
        assert_eq!(listed[0].name, "Budget");
        assert_eq!(listed[0].node_count, 0);
        assert_eq!(
            s.open(info.instance, "alice", created.id).unwrap().model,
            created.id
        );
    }

    #[test]
    fn lost_create_response_reconciles_before_quotas_and_never_allocates_twice() {
        let mut s = service(Limits {
            resources: 2,
            receipts: 1,
            attachments: 1,
        });
        let req = request(&s, "E42", "Budget");
        let committed = s.create("alice", req.clone()).unwrap(); // response discarded by caller
        assert_eq!(
            s.operation_outcome(s.info.instance, "alice", "E42"),
            Ok(OperationOutcome::Committed(committed))
        );
        for _ in 0..10 {
            assert_eq!(s.create("alice", req.clone()), Ok(committed));
        }
        assert_eq!(s.resources.len(), 1);
        assert_eq!(s.receipts.len(), 1);
        let mut changed = req.clone();
        changed.name = "Other".into();
        assert_eq!(s.create("alice", changed), Err(Error::OperationIdConflict));
        assert_eq!(
            s.create("alice", request(&s, "E43", "Budget")),
            Err(Error::ReceiptLimit)
        );
        assert_eq!(
            s.operation_outcome(s.info.instance, "bob", "E42"),
            Ok(OperationOutcome::Unknown)
        );
    }

    #[test]
    fn restart_rejects_all_old_contexts_even_replays() {
        let mut first = service(Limits::default());
        let req = request(&first, "one", "Budget");
        let model = first.create("alice", req.clone()).unwrap();
        let a = first.open(req.instance, "alice", model.id).unwrap();
        let mut next = service(Limits::default());
        assert_ne!(first.info.instance, next.info.instance);
        assert_eq!(next.create("alice", req.clone()), Err(Error::StaleInstance));
        assert_eq!(next.list(req.instance, "alice"), Err(Error::StaleInstance));
        assert_eq!(
            next.inspect(req.instance, "alice", model.id),
            Err(Error::StaleInstance)
        );
        assert_eq!(
            next.open(req.instance, "alice", model.id),
            Err(Error::StaleInstance)
        );
        assert_eq!(
            next.close(req.instance, "alice", a.id),
            Err(Error::StaleInstance)
        );
        assert_eq!(
            next.operation_outcome(req.instance, "alice", "one"),
            Err(Error::StaleInstance)
        );
        assert_eq!(
            next.inspect(next.info.instance, "alice", model.id),
            Err(Error::UnknownModel)
        );
        assert_eq!(
            next.close(next.info.instance, "alice", a.id),
            Err(Error::InvalidAttachment)
        );
    }

    #[test]
    fn quotas_do_not_evict_resources_attachments_or_receipts() {
        let mut s = service(Limits {
            resources: 1,
            receipts: 3,
            attachments: 1,
        });
        let c = s.create("alice", request(&s, "one", "A")).unwrap();
        let failed = request(&s, "two", "B");
        assert_eq!(s.create("alice", failed.clone()), Err(Error::ResourceLimit));
        assert_eq!(s.create("alice", failed), Err(Error::ResourceLimit));
        assert_eq!(
            s.operation_outcome(s.info.instance, "alice", "two"),
            Ok(OperationOutcome::Rejected(Error::ResourceLimit))
        );
        let a = s.open(s.info.instance, "alice", c.id).unwrap();
        assert_eq!(
            s.open(s.info.instance, "alice", c.id),
            Err(Error::AttachmentLimit)
        );
        s.close(s.info.instance, "alice", a.id).unwrap();
        assert!(s.open(s.info.instance, "alice", c.id).is_ok());
        assert_eq!(s.resources.len(), 1);
        assert_eq!(s.receipts.len(), 2);
    }

    #[test]
    fn failures_before_commit_leave_only_rejection_receipt() {
        let mut s = service(Limits::default());
        let req = request(&s, "failed", "Budget");
        assert_eq!(
            s.create_prepared("alice", req.clone(), |_| Err(
                Error::CoreInitializationFailed
            )),
            Err(Error::CoreInitializationFailed)
        );
        assert!(s.resources.is_empty());
        assert_eq!(s.create("alice", req), Err(Error::CoreInitializationFailed));
        assert!(s.resources.is_empty());
        let deny = Arc::new(AtomicBool::new(false));
        s.policy = std::sync::Arc::new(Policy { deny: deny.clone() });
        let req = request(&s, "revoked", "Other");
        assert_eq!(
            s.create_prepared("alice", req, |name| {
                deny.store(true, Ordering::SeqCst);
                WorkingModel::new(name).map_err(|_| Error::CoreInitializationFailed)
            }),
            Err(Error::Denied)
        );
        assert!(s.resources.is_empty());
        deny.store(false, Ordering::SeqCst);
        assert_eq!(
            s.operation_outcome(s.info.instance, "alice", "revoked"),
            Ok(OperationOutcome::Rejected(Error::Denied))
        );
        assert_eq!(s.receipts.len(), 2);
    }

    #[test]
    fn denied_requests_and_invalid_intent_allocate_nothing() {
        let mut s = service(Limits::default());
        let req = request(&s, "one", "Budget");
        assert_eq!(s.create("outsider", req), Err(Error::Denied));
        assert_eq!(s.list(s.info.instance, "outsider"), Err(Error::Denied));
        assert_eq!(
            s.create("alice", request(&s, "", "Budget")),
            Err(Error::InvalidArgument)
        );
        assert!(s.resources.is_empty());
        assert!(s.receipts.is_empty());
        let c = s.create("alice", request(&s, "one", "Budget")).unwrap();
        assert_eq!(
            s.inspect(s.info.instance, "outsider", c.id),
            Err(Error::Denied)
        );
        assert_eq!(
            s.open(s.info.instance, "outsider", c.id),
            Err(Error::Denied)
        );
        assert_eq!(
            s.operation_outcome(s.info.instance, "outsider", "one"),
            Err(Error::Denied)
        );
    }

    #[test]
    fn discovery_filters_per_resource_permissions() {
        struct Filter(WorkingModelId);
        impl Authorizer for Filter {
            fn allowed(&self, _: &str, action: Action, model: Option<WorkingModelId>) -> bool {
                !matches!(action, Action::Inspect | Action::Open | Action::Close)
                    || model == Some(self.0)
            }
        }
        let mut s = service(Limits::default());
        let visible = s.create("alice", request(&s, "one", "A")).unwrap();
        let hidden = s.create("alice", request(&s, "two", "B")).unwrap();
        s.policy = std::sync::Arc::new(Filter(visible.id));
        assert_eq!(s.list(s.info.instance, "alice").unwrap().len(), 1);
        assert_eq!(
            s.inspect(s.info.instance, "alice", hidden.id),
            Err(Error::Denied)
        );
        let a = s.open(s.info.instance, "alice", visible.id).unwrap();
        s.close(s.info.instance, "alice", a.id).unwrap();
    }
}
