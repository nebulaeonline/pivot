//! Stage A: serialized in-process authoring. No daemon or IPC integration.
use super::*;
use hmac::{Hmac, Mac};
use pivot_protocol::{authoring::v1 as a, foundation, v1 as p};
use prost::Message;
use sha2::{Digest, Sha256};
use std::{
    collections::BTreeMap,
    sync::{
        Arc, Mutex, MutexGuard,
        atomic::{AtomicUsize, Ordering},
    },
    time::Instant,
};
mod wire;
use wire::*;
type Failure = Box<a::AuthoringFailure>;
fn failure_outcome(e: Failure) -> Outcome {
    Outcome::Error(*e)
}
type Outcome = a::authoring_response::Outcome;
type SharedService = Arc<Mutex<WorkingModelService>>;
type HmacSha256 = Hmac<Sha256>;

/// Implementations must return monotonically nondecreasing seconds.
pub trait Clock: Send + Sync {
    fn seconds(&self) -> u64;
}
struct MonotonicClock(Instant);
impl Clock for MonotonicClock {
    fn seconds(&self) -> u64 {
        self.0.elapsed().as_secs()
    }
}
#[derive(Clone, Copy)]
pub struct AdapterLimits {
    pub preflight: foundation::Limits,
    pub response_bytes: usize,
    pub page_target_bytes: usize,
    pub page_entries: usize,
    pub default_page_entries: usize,
    pub cursor_seconds: u64,
    pub attachment_ids_per_session: usize,
    pub sessions: usize,
}
impl Default for AdapterLimits {
    fn default() -> Self {
        Self {
            preflight: foundation::Limits::default(),
            response_bytes: 1024 * 1024,
            page_target_bytes: 512 * 1024,
            page_entries: 128,
            default_page_entries: 64,
            cursor_seconds: 60,
            attachment_ids_per_session: 256,
            sessions: 16,
        }
    }
}
pub struct AuthoringAdapter {
    service: SharedService,
    limits: AdapterLimits,
    clock: Arc<dyn Clock>,
    key: [u8; 32],
    sessions: AtomicUsize,
}
impl AuthoringAdapter {
    pub(super) fn request_limits(&self) -> foundation::Limits {
        foundation::Limits {
            bytes: self.limits.response_bytes,
            ..self.limits.preflight
        }
    }
    pub fn new(service: SharedService, limits: AdapterLimits) -> Result<Arc<Self>, Error> {
        Self::with_clock(service, limits, Arc::new(MonotonicClock(Instant::now())))
    }
    pub fn with_clock(
        service: SharedService,
        mut limits: AdapterLimits,
        clock: Arc<dyn Clock>,
    ) -> Result<Arc<Self>, Error> {
        if !(512..=pivot_protocol::validation::MAX_MESSAGE_BYTES).contains(&limits.preflight.bytes)
            || !(512..=pivot_protocol::validation::MAX_MESSAGE_BYTES)
                .contains(&limits.response_bytes)
            || limits.page_target_bytes < 512
            || limits.page_target_bytes > limits.response_bytes
            || limits.page_entries == 0
            || limits.page_entries > u32::MAX as usize
            || limits.default_page_entries == 0
            || limits.default_page_entries > limits.page_entries
            || limits.cursor_seconds == 0
            || limits.attachment_ids_per_session == 0
            || limits.sessions == 0
            || limits.preflight.cursor < 194
            || limits.preflight.cursor > 512
            || limits.preflight.depth == 0
            || limits.preflight.fields == 0
            || limits.preflight.edits == 0
            || limits.preflight.text == 0
        {
            return Err(Error::InvalidArgument);
        }
        // The handshake advertises a single ceiling for both directions.
        limits.response_bytes = limits.response_bytes.min(limits.preflight.bytes);
        limits.page_target_bytes = limits.page_target_bytes.min(limits.response_bytes);
        let mut key = [0; 32];
        key[..16].copy_from_slice(Uuid::new_v4().as_bytes());
        key[16..].copy_from_slice(Uuid::new_v4().as_bytes());
        Ok(Arc::new(Self {
            service,
            limits,
            clock,
            key,
            sessions: AtomicUsize::new(0),
        }))
    }
    /// Principal is supplied by the trusted embedder, never by a wire payload.
    pub fn open_session(self: &Arc<Self>, principal: &str) -> Result<AuthoringSession, Error> {
        if principal.is_empty() || principal.len() > 256 {
            return Err(Error::InvalidArgument);
        }
        let mut n = self.sessions.load(Ordering::Acquire);
        loop {
            if n >= self.limits.sessions {
                return Err(Error::ResourceLimit);
            }
            match self
                .sessions
                .compare_exchange_weak(n, n + 1, Ordering::AcqRel, Ordering::Acquire)
            {
                Ok(_) => break,
                Err(current) => n = current,
            }
        }
        Ok(AuthoringSession {
            adapter: self.clone(),
            principal: principal.into(),
            id: Uuid::new_v4(),
            negotiated: false,
            closed_session: false,
            live: BTreeMap::new(),
            closed: BTreeMap::new(),
        })
    }
    fn lock(&self) -> Result<MutexGuard<'_, WorkingModelService>, Failure> {
        self.service
            .lock()
            .map_err(|_| fail(a::AuthoringErrorCode::Internal))
    }
}
pub struct AuthoringSession {
    adapter: Arc<AuthoringAdapter>,
    principal: String,
    id: Uuid,
    negotiated: bool,
    closed_session: bool,
    live: BTreeMap<AttachmentId, WorkingModelId>,
    closed: BTreeMap<AttachmentId, WorkingModelId>,
}
enum Permit {
    Action(Action, Option<WorkingModelId>),
    Diagnostics(WorkingModelId),
    Receipt(ServiceInstanceId, String),
}
impl AuthoringSession {
    fn decode_limits(&self) -> foundation::Limits {
        foundation::Limits {
            bytes: self
                .adapter
                .limits
                .preflight
                .bytes
                .min(self.adapter.limits.response_bytes),
            ..self.adapter.limits.preflight
        }
    }
    pub fn close(&mut self) {
        if self.closed_session {
            return;
        }
        // Disconnect cleanup is ownership-based, even if Close policy was revoked.
        let mut service = self
            .adapter
            .service
            .lock()
            .unwrap_or_else(|e| e.into_inner());
        service
            .attachments
            .retain(|a| a.principal != self.principal || !self.live.contains_key(&a.attachment.id));
        self.live.clear();
        self.closed.clear();
        self.negotiated = false;
        self.closed_session = true;
        self.adapter.sessions.fetch_sub(1, Ordering::AcqRel);
    }
    pub fn dispatch(&mut self, bytes: &[u8]) -> Vec<u8> {
        if self.closed_session {
            return self.envelope_error("", p::ErrorCode::HandshakeRequired);
        }
        if !self.negotiated {
            let request = match foundation::decode_handshake(bytes, self.decode_limits()) {
                Ok(r) => r,
                Err(e) => {
                    return self.envelope_error(
                        "",
                        if e.code == p::ErrorCode::ResourceExhausted {
                            e.code
                        } else {
                            p::ErrorCode::HandshakeRequired
                        },
                    );
                }
            };
            let Some(p::request::Operation::Handshake(hello)) = request.operation else {
                unreachable!()
            };
            return match foundation::negotiate(&hello) {
                Ok(mut accepted)
                    if accepted.selected_version == Some(p::Version { major: 1, minor: 1 }) =>
                {
                    accepted.max_message_bytes =
                        self.adapter
                            .limits
                            .preflight
                            .bytes
                            .min(self.adapter.limits.response_bytes) as u32;
                    self.negotiated = true;
                    p::Response {
                        request_id: request.request_id,
                        version: Some(p::Version { major: 1, minor: 1 }),
                        outcome: Some(p::response::Outcome::Handshake(accepted)),
                    }
                    .encode_to_vec()
                }
                Ok(_) => self.envelope_error(&request.request_id, p::ErrorCode::UnsupportedVersion),
                Err(e) => self.envelope_error(&request.request_id, e.code),
            };
        }
        let request = match foundation::decode_envelope(bytes, self.decode_limits()) {
            Ok(r) => r,
            Err(e) => {
                let code = match e.code {
                    p::ErrorCode::ResourceExhausted => a::AuthoringErrorCode::ResourceExhausted,
                    p::ErrorCode::UnsupportedValueType => {
                        a::AuthoringErrorCode::UnsupportedValueType
                    }
                    _ => a::AuthoringErrorCode::InvalidArgument,
                };
                // No generated decode of a rejected mutation merely to recover correlation.
                return self
                    .response("", failure_outcome(fail(code)))
                    .encode_to_vec();
            }
        };
        let Some(p::request::Operation::Authoring(authoring)) = &request.operation else {
            unreachable!()
        };
        let Some(op) = authoring.operation.as_ref() else {
            unreachable!()
        };
        let permit = self.permit(op);
        let result = match &permit {
            Ok(_) => self
                .mutation_budget(&request, op)
                .and_then(|_| self.execute(op)),
            Err(e) => Err(e.clone()),
        };
        let outcome = result.unwrap_or_else(failure_outcome);
        let mut response = self.response(&request.request_id, outcome);
        if response.encoded_len() > self.adapter.limits.response_bytes {
            response = self.response(
                &request.request_id,
                failure_outcome(fail(a::AuthoringErrorCode::ResourceExhausted)),
            );
        }
        let encoded = response.encode_to_vec();
        // Final authorization after conversion/serialization, including errors with
        // privileged validation summaries. A committed mutation is not rolled back.
        if let Ok(permit) = permit
            && let Err(e) = self.authorize(&permit, &response)
        {
            return self
                .response(&request.request_id, failure_outcome(e))
                .encode_to_vec();
        }
        encoded
    }
    fn response(&self, id: &str, outcome: Outcome) -> p::Response {
        p::Response {
            request_id: id.into(),
            version: Some(p::Version { major: 1, minor: 1 }),
            outcome: Some(p::response::Outcome::Authoring(a::AuthoringResponse {
                outcome: Some(outcome),
            })),
        }
    }
    fn envelope_error(&self, id: &str, code: p::ErrorCode) -> Vec<u8> {
        p::Response {
            request_id: id.into(),
            version: Some(p::Version { major: 1, minor: 1 }),
            outcome: Some(p::response::Outcome::Error(p::ProtocolError {
                code: code as i32,
                message: "request rejected".into(),
                port_id: String::new(),
            })),
        }
        .encode_to_vec()
    }
    fn permit(&self, op: &a::authoring_request::Operation) -> Result<Permit, Failure> {
        use a::authoring_request::Operation::*;
        let action = match op {
            GetServiceInfo(_) => (Action::Info, None),
            CreateWorking(_) => (Action::Create, None),
            ListWorking(_) => (Action::List, None),
            OpenWorking(v) => (Action::Open, Some(parse(&v.working_model_id)?)),
            CloseAttachment(v) => {
                let id = parse(&v.attachment_id)?;
                let model = self
                    .live
                    .get(&id)
                    .or_else(|| self.closed.get(&id))
                    .ok_or_else(|| fail(a::AuthoringErrorCode::NotFound))?;
                (Action::Close, Some(*model))
            }
            ReadDraft(v) => (
                Action::Inspect,
                Some(revision(v.revision.as_ref().unwrap())?.model),
            ),
            ReadDiagnostics(v) => {
                return Ok(Permit::Diagnostics(
                    revision(v.working_revision.as_ref().unwrap())?.model,
                ));
            }
            ApplyEdits(v) => (
                Action::Edit,
                Some(parse(&v.mutation.as_ref().unwrap().working_model_id)?),
            ),
            ValidateWorking(v) => (
                Action::Validate,
                Some(revision(v.revision.as_ref().unwrap())?.model),
            ),
            EvaluateWorking(v) => (
                Action::Evaluate,
                Some(revision(v.revision.as_ref().unwrap())?.model),
            ),
            PublishWorking(v) => (
                Action::Publish,
                Some(parse(&v.mutation.as_ref().unwrap().working_model_id)?),
            ),
            GetOperationOutcome(v) => {
                return Ok(Permit::Receipt(
                    parse(&v.service_instance_id)?,
                    v.operation_id.clone(),
                ));
            }
        };
        Ok(Permit::Action(action.0, action.1))
    }
    fn authorize(&self, permit: &Permit, response: &p::Response) -> Result<(), Failure> {
        let service = self.adapter.lock()?;
        let instance = service.info.instance;
        match permit {
            Permit::Action(action, model) => service
                .check(instance, &self.principal, *action, *model)
                .map_err(service_error)?,
            Permit::Diagnostics(model) => service
                .diagnostic_permission(instance, &self.principal, *model)
                .map_err(service_error)?,
            Permit::Receipt(instance, op) => {
                service
                    .operation_outcome(*instance, &self.principal, op)
                    .map_err(service_error)?;
            }
        }
        if let Some(p::response::Outcome::Authoring(a::AuthoringResponse {
            outcome: Some(Outcome::WorkingModels(list)),
        })) = &response.outcome
        {
            for entry in &list.models {
                service
                    .check(
                        instance,
                        &self.principal,
                        Action::Inspect,
                        Some(parse(&entry.working_model_id)?),
                    )
                    .map_err(service_error)?;
            }
        }
        Ok(())
    }
    fn execute(&mut self, op: &a::authoring_request::Operation) -> Result<Outcome, Failure> {
        use a::authoring_request::Operation::*;
        let who = &self.principal;
        match op {
            GetServiceInfo(_) => {
                let s = self.adapter.lock()?;
                s.check(s.info.instance, who, Action::Info, None)
                    .map_err(service_error)?;
                Ok(Outcome::ServiceInfo(self.info(s.info())))
            }
            CreateWorking(v) => Ok(Outcome::Created(created(
                self.adapter
                    .lock()?
                    .create(
                        who,
                        CreateRequest {
                            instance: parse(&v.service_instance_id)?,
                            operation_id: v.operation_id.clone(),
                            name: v.name.clone(),
                        },
                    )
                    .map_err(service_error)?,
            ))),
            ListWorking(v) => self.list_page(v),
            OpenWorking(v) => {
                if self.live.len() + self.closed.len()
                    >= self.adapter.limits.attachment_ids_per_session
                {
                    return Err(fail(a::AuthoringErrorCode::ResourceExhausted));
                }
                let attachment = self
                    .adapter
                    .lock()?
                    .open(
                        parse(&v.service_instance_id)?,
                        who,
                        parse(&v.working_model_id)?,
                    )
                    .map_err(service_error)?;
                self.live.insert(attachment.id, attachment.model);
                Ok(Outcome::Opened(a::Opened {
                    attachment_id: attachment.id.to_string(),
                    revision: Some(rev(attachment.revision)),
                }))
            }
            CloseAttachment(v) => {
                let instance = parse(&v.service_instance_id)?;
                let id = parse(&v.attachment_id)?;
                let model = *self
                    .live
                    .get(&id)
                    .or_else(|| self.closed.get(&id))
                    .ok_or_else(|| fail(a::AuthoringErrorCode::NotFound))?;
                let mut s = self.adapter.lock()?;
                s.check(instance, who, Action::Close, Some(model))
                    .map_err(service_error)?;
                if self.live.contains_key(&id) {
                    s.close(instance, who, id).map_err(service_error)?;
                    self.live.remove(&id);
                    self.closed.insert(id, model);
                }
                Ok(Outcome::Closed(a::Closed {}))
            }
            ReadDraft(v) => self.draft_page(v),
            ReadDiagnostics(v) => self.diagnostics_page(v),
            ApplyEdits(v) => {
                let converted = edits(v)?;
                Ok(Outcome::Edited(committed(
                    self.adapter
                        .lock()?
                        .apply_edits_intent(who, converted, Some(v.clone()))
                        .map_err(service_error)?,
                )))
            }
            ValidateWorking(v) => {
                let report = self
                    .adapter
                    .lock()?
                    .validate_working(who, revision(v.revision.as_ref().unwrap())?)
                    .map_err(service_error)?;
                Ok(Outcome::Validation(a::ValidationReport {
                    revision: Some(rev(report.revision)),
                    valid: report.valid,
                    diagnostics: Some(summary(report.revision, &report.diagnostics)),
                }))
            }
            EvaluateWorking(v) => {
                let capture = {
                    self.adapter
                        .lock()?
                        .capture_evaluation(who, revision(v.revision.as_ref().unwrap())?)
                        .map_err(|e| match e {
                            CaptureError::Service(e) => service_error(e),
                            CaptureError::Validation(v) => {
                                let mut e = fail(a::AuthoringErrorCode::ValidationFailed);
                                e.validation_diagnostics =
                                    Some(summary(v.revision, &v.diagnostics));
                                e
                            }
                        })?
                };
                let inputs = v
                    .inputs
                    .iter()
                    .map(|i| Ok((parse(&i.node_id)?, value(i.value.as_ref().unwrap())?)))
                    .collect::<Result<_, Failure>>()?;
                let outputs = v
                    .output_node_ids
                    .iter()
                    .map(|s| parse(s))
                    .collect::<Result<_, _>>()?;
                // This owned capture has no management MutexGuard.
                Ok(Outcome::Evaluation(evaluated(
                    capture
                        .evaluate(EvaluationRequest { inputs, outputs })
                        .map_err(service_error)?,
                )))
            }
            PublishWorking(v) => {
                let m = v.mutation.as_ref().unwrap();
                let request = PublishRequest {
                    instance: parse(&m.service_instance_id)?,
                    model: parse(&m.working_model_id)?,
                    expected: revision(m.expected_revision.as_ref().unwrap())?,
                    operation_id: m.operation_id.clone(),
                };
                let prepared = {
                    self.adapter
                        .lock()?
                        .prepare_publication(who, request)
                        .map_err(publication_error)?
                };
                Ok(Outcome::Published(published(
                    self.adapter
                        .lock()?
                        .commit_publication(who, prepared)
                        .map_err(publication_error)?,
                )))
            }
            GetOperationOutcome(v) => Ok(Outcome::OperationOutcome(outcome(
                self.adapter
                    .lock()?
                    .operation_outcome(parse(&v.service_instance_id)?, who, &v.operation_id)
                    .map_err(service_error)?,
            ))),
        }
    }
    fn info(&self, v: ServiceInfo) -> a::ServiceInfo {
        let e = v.edit_limits;
        let c = v.calculation_limits;
        let l = self.adapter.limits;
        a::ServiceInfo {
            service_instance_id: v.instance.to_string(),
            durability: a::Durability::MemoryOnly as i32,
            receipt_scope: a::ReceiptScope::ServiceInstance as i32,
            receipts_retained_without_eviction: true,
            attachments_bound_to_connection: true,
            limits: Some(a::ResourceLimits {
                working_resources: v.limits.resources as u64,
                attachments: v.limits.attachments as u64,
                receipts: v.limits.receipts as u64,
                edits_per_batch: e.operations.min(l.preflight.edits) as u64,
                batch_accounting_bytes: e.batch_bytes as u64,
                live_nodes: e.nodes as u64,
                live_ports: e.ports as u64,
                stored_source_bytes: e.source_bytes as u64,
                cumulative_node_ids: e.node_identities as u64,
                cumulative_port_ids: e.port_identities as u64,
                live_captures: c.captures as u64,
                evaluation_inputs: c.inputs.min(l.preflight.inputs) as u64,
                evaluation_outputs: c.outputs.min(l.preflight.outputs) as u64,
                snapshot_work: c.work as u64,
                published_revisions: c.published_revisions as u64,
                page_entries: l.page_entries as u64,
                page_target_bytes: l.page_target_bytes as u64,
                message_bytes: l.preflight.bytes.min(l.response_bytes) as u64,
                cursor_bytes: l.preflight.cursor as u64,
                cursor_lifetime_seconds: l.cursor_seconds,
                name_bytes: 128.min(l.preflight.name) as u64,
                formula_bytes: 4096.min(l.preflight.text) as u64,
                temporary_id_bytes: 64,
                closed_attachment_ids_per_connection: l.attachment_ids_per_session as u64,
                authoring_sessions: l.sessions as u64,
            }),
        }
    }
    fn mutation_budget(
        &self,
        request: &p::Request,
        op: &a::authoring_request::Operation,
    ) -> Result<(), Failure> {
        use a::authoring_request::Operation::*;
        let id = "00000000-0000-0000-0000-000000000000";
        let revision = a::WorkingRevision {
            service_instance_id: id.into(),
            working_model_id: id.into(),
            sequence: u64::MAX,
        };
        let diagnostics = a::DiagnosticSummary {
            revision: Some(revision.clone()),
            total: u64::MAX,
            counts: (1..=5)
                .map(|code| a::DiagnosticCount {
                    code,
                    count: u64::MAX,
                })
                .collect(),
        };
        let success = match op {
            CreateWorking(_) => Outcome::Created(a::Created {
                working_model_id: id.into(),
                revision: Some(revision.clone()),
            }),
            ApplyEdits(v) => {
                let mut nodes = Vec::new();
                let mut ports = Vec::new();
                for e in &v.edits {
                    match &e.kind {
                        Some(a::edit::Kind::AddNode(v)) => nodes.push(a::NodeMapping {
                            temporary_id: v.temporary_id.clone(),
                            node_id: id.into(),
                        }),
                        Some(a::edit::Kind::AddInputPort(v)) => ports.push(a::PortMapping {
                            temporary_id: v.temporary_id.clone(),
                            port_id: id.into(),
                        }),
                        Some(a::edit::Kind::AddOutputPort(v)) => ports.push(a::PortMapping {
                            temporary_id: v.temporary_id.clone(),
                            port_id: id.into(),
                        }),
                        _ => {}
                    }
                }
                Outcome::Edited(a::EditCommitted {
                    revision: Some(revision.clone()),
                    nodes,
                    ports,
                    diagnostics: Some(diagnostics.clone()),
                })
            }
            PublishWorking(_) => Outcome::Published(a::Published {
                provenance: Some(a::Provenance {
                    working: Some(revision),
                    snapshot_id: id.into(),
                }),
                published: Some(p::ModelRef {
                    model_id: id.into(),
                    revision_id: id.into(),
                }),
            }),
            _ => return Ok(()),
        };
        let mut error = fail(a::AuthoringErrorCode::ValidationFailed);
        error.validation_diagnostics = Some(diagnostics);
        if self
            .response(&request.request_id, success)
            .encoded_len()
            .max(
                self.response(&request.request_id, failure_outcome(error))
                    .encoded_len(),
            )
            > self.adapter.limits.response_bytes
        {
            return Err(fail(a::AuthoringErrorCode::ResourceExhausted));
        }
        Ok(())
    }
}
impl Drop for AuthoringSession {
    fn drop(&mut self) {
        self.close();
    }
}

struct Cursor {
    index: u64,
    last: Uuid,
    expiry: u64,
}
impl AuthoringSession {
    fn page(
        &self,
        page: Option<&a::Page>,
        query: &a::AuthoringRequest,
    ) -> Result<(usize, Cursor, [u8; 32]), Failure> {
        let size = page.map_or(0, |p| p.page_size) as usize;
        let size = if size == 0 {
            self.adapter.limits.default_page_entries
        } else {
            size
        };
        if size > self.adapter.limits.page_entries {
            return Err(fail(a::AuthoringErrorCode::ResourceExhausted));
        }
        let mut normalized = query.clone();
        let page_mut = match normalized.operation.as_mut().unwrap() {
            a::authoring_request::Operation::ListWorking(v) => &mut v.page,
            a::authoring_request::Operation::ReadDraft(v) => &mut v.page,
            a::authoring_request::Operation::ReadDiagnostics(v) => &mut v.page,
            _ => unreachable!(),
        };
        *page_mut = Some(a::Page {
            page_size: size as u32,
            continuation_token: String::new(),
        });
        let query: [u8; 32] = Sha256::digest(normalized.encode_to_vec()).into();
        let now = self.adapter.clock.seconds();
        let token = page.map_or("", |p| p.continuation_token.as_str());
        let cursor = if token.is_empty() {
            Cursor {
                index: 0,
                last: Uuid::nil(),
                expiry: now
                    .checked_add(self.adapter.limits.cursor_seconds)
                    .ok_or_else(|| fail(a::AuthoringErrorCode::ResourceExhausted))?,
            }
        } else {
            let bytes = unhex(token).ok_or_else(|| fail(a::AuthoringErrorCode::InvalidArgument))?;
            if bytes.len() != 97 || bytes[0] != 1 {
                return Err(fail(a::AuthoringErrorCode::InvalidArgument));
            }
            let mut mac = self.mac();
            mac.update(&bytes[..65]);
            mac.verify_slice(&bytes[65..])
                .map_err(|_| fail(a::AuthoringErrorCode::InvalidArgument))?;
            if bytes[33..65] != query {
                return Err(fail(a::AuthoringErrorCode::InvalidArgument));
            }
            let expiry = u64::from_le_bytes(bytes[1..9].try_into().unwrap());
            if now >= expiry {
                return Err(fail(a::AuthoringErrorCode::ViewExpired));
            }
            Cursor {
                expiry,
                index: u64::from_le_bytes(bytes[9..17].try_into().unwrap()),
                last: Uuid::from_slice(&bytes[17..33]).unwrap(),
            }
        };
        Ok((size, cursor, query))
    }
    fn mac(&self) -> HmacSha256 {
        let mut mac = HmacSha256::new_from_slice(&self.adapter.key).expect("fixed key length");
        mac.update(self.id.as_bytes());
        mac.update(&(self.principal.len() as u32).to_le_bytes());
        mac.update(self.principal.as_bytes());
        mac
    }
    fn cursor(&self, cursor: &Cursor, query: &[u8; 32]) -> String {
        let mut bytes = vec![1];
        bytes.extend(cursor.expiry.to_le_bytes());
        bytes.extend(cursor.index.to_le_bytes());
        bytes.extend(cursor.last.as_bytes());
        bytes.extend(query);
        let mut mac = self.mac();
        mac.update(&bytes);
        bytes.extend(mac.finalize().into_bytes());
        hex(&bytes)
    }
    fn fits(&self, outcome: Outcome, empty: bool) -> bool {
        let n = self.response(&"r".repeat(128), outcome).encoded_len();
        n <= self.adapter.limits.response_bytes
            && (empty || n <= self.adapter.limits.page_target_bytes)
    }
    fn list_page(&self, v: &a::ListWorking) -> Result<Outcome, Failure> {
        let query = a::AuthoringRequest {
            operation: Some(a::authoring_request::Operation::ListWorking(v.clone())),
        };
        let (size, mut cursor, key) = self.page(v.page.as_ref(), &query)?;
        let mut entries = self
            .adapter
            .lock()?
            .list(parse(&v.service_instance_id)?, &self.principal)
            .map_err(service_error)?;
        entries.sort_by_key(|r| r.id);
        let entries: Vec<_> = entries
            .into_iter()
            .filter(|r| r.id.0 > cursor.last)
            .collect();
        let mut result = a::WorkingList::default();
        for r in &entries {
            if result.models.len() >= size {
                break;
            }
            let old_last = cursor.last;
            cursor.last = r.id.0;
            let entry = a::WorkingSummary {
                working_model_id: r.id.to_string(),
                name: r.name.clone(),
                revision: Some(rev(r.revision)),
                nodes: r.node_count as u64,
                inputs: r.input_count as u64,
                outputs: r.output_count as u64,
            };
            let empty = result.models.is_empty();
            result.models.push(entry);
            result.continuation_token = self.cursor(&cursor, &key);
            if !self.fits(Outcome::WorkingModels(result.clone()), empty) {
                result.models.pop();
                cursor.last = old_last;
                if empty {
                    return Err(fail(a::AuthoringErrorCode::ResourceExhausted));
                }
                break;
            }
        }
        result.continuation_token = if result.models.len() < entries.len() {
            self.cursor(&cursor, &key)
        } else {
            String::new()
        };
        Ok(Outcome::WorkingModels(result))
    }
    fn draft_page(&self, v: &a::ReadDraft) -> Result<Outcome, Failure> {
        let query = a::AuthoringRequest {
            operation: Some(a::authoring_request::Operation::ReadDraft(v.clone())),
        };
        let (size, mut cursor, key) = self.page(v.page.as_ref(), &query)?;
        let expected = revision(v.revision.as_ref().unwrap())?;
        let mut view = self
            .adapter
            .lock()?
            .read_draft(&self.principal, expected)
            .map_err(|e| self.read_error(e, v.page.as_ref()))?;
        view.nodes.sort_by_key(|n| n.id);
        view.ports.sort_by_key(|p| p.id);
        let is_nodes = v.section == a::DraftSection::Nodes as i32;
        let total = if is_nodes {
            view.nodes.len()
        } else {
            view.ports.len()
        };
        let start = usize::try_from(cursor.index)
            .map_err(|_| fail(a::AuthoringErrorCode::InvalidArgument))?;
        if start > total {
            return Err(fail(a::AuthoringErrorCode::InvalidArgument));
        }
        let mut result = a::DraftPage {
            revision: Some(rev(expected)),
            section: v.section,
            ..Default::default()
        };
        for index in start..total {
            if index - start >= size {
                break;
            }
            if is_nodes {
                let n = &view.nodes[index];
                result.nodes.push(a::RawNode {
                    node_id: n.id.to_string(),
                    name: n.name.clone(),
                    definition: Some(node_definition(&n.definition)),
                    bound_dependency_ids: n.dependencies.iter().map(ToString::to_string).collect(),
                });
            } else {
                let p = &view.ports[index];
                result.ports.push(a::RawPort {
                    port_id: p.id.to_string(),
                    name: p.name.clone(),
                    kind: Some(match &p.definition {
                        PortDefinition::Input { node, requirement } => {
                            a::raw_port::Kind::Input(a::RawInputPort {
                                node_id: node.to_string(),
                                requirement: Some(port_requirement(requirement)),
                            })
                        }
                        PortDefinition::Output { node } => {
                            a::raw_port::Kind::Output(a::RawOutputPort {
                                node_id: node.to_string(),
                            })
                        }
                    }),
                });
            }
            cursor.index = index as u64 + 1;
            result.continuation_token = self.cursor(&cursor, &key);
            if !self.fits(Outcome::Draft(result.clone()), index == start) {
                if is_nodes {
                    result.nodes.pop();
                } else {
                    result.ports.pop();
                }
                cursor.index = index as u64;
                if index == start {
                    return Err(fail(a::AuthoringErrorCode::ResourceExhausted));
                }
                break;
            }
        }
        result.continuation_token = if cursor.index < total as u64 {
            self.cursor(&cursor, &key)
        } else {
            String::new()
        };
        Ok(Outcome::Draft(result))
    }
    fn diagnostics_page(&self, v: &a::ReadDiagnostics) -> Result<Outcome, Failure> {
        let query = a::AuthoringRequest {
            operation: Some(a::authoring_request::Operation::ReadDiagnostics(v.clone())),
        };
        let (size, mut cursor, key) = self.page(v.page.as_ref(), &query)?;
        let expected = revision(v.working_revision.as_ref().unwrap())?;
        if expected.instance != parse(&v.service_instance_id)? {
            return Err(fail(a::AuthoringErrorCode::InvalidArgument));
        }
        let diagnostics = self
            .adapter
            .lock()?
            .read_diagnostics(&self.principal, expected)
            .map_err(|e| self.read_error(e, v.page.as_ref()))?;
        let summary = summary(expected, &diagnostics);
        let mut diagnostics: Vec<_> = diagnostics.iter().map(diagnostic).collect();
        diagnostics.sort_by_key(Message::encode_to_vec);
        let start = usize::try_from(cursor.index)
            .map_err(|_| fail(a::AuthoringErrorCode::InvalidArgument))?;
        if start > diagnostics.len() {
            return Err(fail(a::AuthoringErrorCode::InvalidArgument));
        }
        let mut result = a::DiagnosticsPage {
            summary: Some(summary),
            ..Default::default()
        };
        for (index, d) in diagnostics.iter().enumerate().skip(start).take(size) {
            result.diagnostics.push(d.clone());
            cursor.index = index as u64 + 1;
            result.continuation_token = self.cursor(&cursor, &key);
            if !self.fits(Outcome::Diagnostics(result.clone()), index == start) {
                result.diagnostics.pop();
                cursor.index = index as u64;
                if index == start {
                    return Err(fail(a::AuthoringErrorCode::ResourceExhausted));
                }
                break;
            }
        }
        result.continuation_token = if cursor.index < diagnostics.len() as u64 {
            self.cursor(&cursor, &key)
        } else {
            String::new()
        };
        Ok(Outcome::Diagnostics(result))
    }
    fn read_error(&self, e: Error, page: Option<&a::Page>) -> Failure {
        if e == Error::RevisionConflict && page.is_some_and(|p| !p.continuation_token.is_empty()) {
            fail(a::AuthoringErrorCode::ViewExpired)
        } else {
            service_error(e)
        }
    }
}
fn hex(bytes: &[u8]) -> String {
    const DIGITS: &[u8] = b"0123456789abcdef";
    let mut s = String::with_capacity(bytes.len() * 2);
    for b in bytes {
        s.push(DIGITS[(b >> 4) as usize] as char);
        s.push(DIGITS[(b & 15) as usize] as char);
    }
    s
}
fn unhex(s: &str) -> Option<Vec<u8>> {
    if s.len() != 194 {
        return None;
    }
    fn nibble(b: u8) -> Option<u8> {
        match b {
            b'0'..=b'9' => Some(b - b'0'),
            b'a'..=b'f' => Some(b - b'a' + 10),
            _ => None,
        }
    }
    s.as_bytes()
        .as_chunks::<2>()
        .0
        .iter()
        .map(|b| Some(nibble(b[0])? * 16 + nibble(b[1])?))
        .collect()
}
