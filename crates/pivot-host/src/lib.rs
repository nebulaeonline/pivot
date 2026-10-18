//! Transport-independent, in-memory hosting. No persistent publishing policy.
pub mod management;
use pivot_core::{
    evaluation::EvaluationContext,
    interface::{InputRequirement, NamedInvocationRequest, PublicEvaluationError},
    model::ModelSnapshot,
};
use pivot_protocol::{
    v1 as wire,
    validation::{self, ValidationError},
};
use pivot_types::{DecimalError, PivotDecimal, PivotValue};
use prost::Message;
use std::{
    collections::{BTreeMap, BTreeSet},
    sync::{
        Arc,
        atomic::{AtomicU64, Ordering},
    },
};

fn error(code: wire::ErrorCode, field: &str) -> ValidationError {
    ValidationError {
        code,
        field: field.into(),
    }
}

#[derive(Debug, Clone, Copy)]
pub struct Limits {
    pub message_bytes: usize,
    pub inputs: usize,
    pub outputs: usize,
}
impl Default for Limits {
    fn default() -> Self {
        Self {
            message_bytes: validation::MAX_MESSAGE_BYTES,
            inputs: validation::MAX_PORTS,
            outputs: validation::MAX_PORTS,
        }
    }
}
impl Limits {
    fn validate(self) -> Result<Self, ValidationError> {
        if !(256..=validation::MAX_MESSAGE_BYTES).contains(&self.message_bytes)
            || self.inputs == 0
            || self.inputs > validation::MAX_PORTS
            || self.outputs == 0
            || self.outputs > validation::MAX_PORTS
        {
            return Err(error(wire::ErrorCode::InvalidArgument, "limits"));
        }
        Ok(self)
    }
}

#[derive(Clone, Copy)]
pub enum Action {
    Discover,
    Describe,
    Invoke,
}
/// The embedder supplies policy for its authenticated/trusted context. This is
/// a boundary, not an authentication mechanism. Deny before lookup or execution.
pub trait Authorizer: Send + Sync {
    fn allowed(&self, action: Action, model: &wire::ModelRef) -> bool;
}
pub struct AllowAll;
impl Authorizer for AllowAll {
    fn allowed(&self, _: Action, _: &wire::ModelRef) -> bool {
        true
    }
}

pub struct Session {
    limits: Limits,
    negotiated: bool,
    authorizer: Arc<dyn Authorizer>,
}
impl Session {
    pub fn new(limits: Limits, authorizer: Arc<dyn Authorizer>) -> Result<Self, ValidationError> {
        Ok(Self {
            limits: limits.validate()?,
            negotiated: false,
            authorizer,
        })
    }
    pub fn negotiated(&self) -> bool {
        self.negotiated
    }
}

struct HostedModel {
    snapshot: ModelSnapshot,
    description: wire::ModelDescription,
}
struct PreparedRegistration {
    key: (String, String),
    entry: HostedModel,
}
/// Entries are never replaced in place. Use a new external revision ID for edits.
pub struct ModelHost {
    entries: BTreeMap<(String, String), HostedModel>,
    invocation_namespace: String,
    next_invocation: AtomicU64,
    work_budget: usize,
}
impl ModelHost {
    /// Namespace must be unique for this host lifetime if IDs leave its scope.
    /// Caller allocation deliberately avoids establishing a durable registry policy.
    pub fn new(invocation_namespace: impl Into<String>) -> Result<Self, ValidationError> {
        let namespace = invocation_namespace.into();
        if namespace.is_empty()
            || namespace.len() > 96
            || !namespace
                .bytes()
                .all(|b| b.is_ascii_alphanumeric() || b"._-".contains(&b))
        {
            return Err(error(wire::ErrorCode::InvalidArgument, "namespace"));
        }
        Ok(Self {
            entries: BTreeMap::new(),
            invocation_namespace: namespace,
            next_invocation: AtomicU64::new(0),
            work_budget: 100_000,
        })
    }
    /// Conservative full-snapshot bound: nodes, edges, and bound expression tokens.
    /// This is independent of message size and intentionally not a wall-clock limit.
    pub fn set_work_budget(&mut self, maximum: usize) -> Result<(), ValidationError> {
        if maximum == 0 {
            return Err(error(wire::ErrorCode::InvalidArgument, "work budget"));
        }
        self.work_budget = maximum;
        Ok(())
    }
    fn check_work(&self, snapshot: &ModelSnapshot) -> Result<(), ValidationError> {
        let mut remaining = self.work_budget;
        for &id in snapshot.dependency_order() {
            let node = snapshot.node(id).expect("validated snapshot");
            let tokens = match node.definition() {
                pivot_core::model::SnapshotDefinition::Input(_) => 0,
                pivot_core::model::SnapshotDefinition::Computed { tokens, .. } => tokens.len(),
            };
            let cost = 1usize
                .checked_add(node.dependencies().len())
                .and_then(|c| c.checked_add(tokens))
                .ok_or_else(|| error(wire::ErrorCode::ResourceExhausted, "work budget"))?;
            remaining = remaining
                .checked_sub(cost)
                .ok_or_else(|| error(wire::ErrorCode::ResourceExhausted, "work budget"))?;
        }
        Ok(())
    }
    /// Map every public label to a caller-assigned portable port ID. No NodeIds
    /// or process-local PortIds are used to derive external identities.
    pub fn register(
        &mut self,
        model: wire::ModelRef,
        snapshot: ModelSnapshot,
        ports: BTreeMap<String, String>,
    ) -> Result<(), ValidationError> {
        let prepared = self.prepare_registration(model, snapshot, ports)?;
        self.install_registration(prepared);
        Ok(())
    }
    fn prepare_registration(
        &self,
        model: wire::ModelRef,
        snapshot: ModelSnapshot,
        ports: BTreeMap<String, String>,
    ) -> Result<PreparedRegistration, ValidationError> {
        let mut used = BTreeSet::new();
        let mut inputs = Vec::new();
        let mut outputs = Vec::new();
        for port in snapshot.interface().inputs() {
            let port_id = ports
                .get(port.name())
                .ok_or_else(|| error(wire::ErrorCode::InvalidArgument, "port mapping"))?
                .clone();
            used.insert(port.name().to_owned());
            let requirement = match port.requirement() {
                InputRequirement::Required => wire::input_port::Requirement::Required(true),
                InputRequirement::Optional { default } => {
                    wire::input_port::Requirement::DefaultValue(to_wire(default)?)
                }
            };
            inputs.push(wire::InputPort {
                port_id,
                name: port.name().into(),
                value_type: wire::ValueType::Decimal as i32,
                requirement: Some(requirement),
            });
        }
        for port in snapshot.interface().outputs() {
            let port_id = ports
                .get(port.name())
                .ok_or_else(|| error(wire::ErrorCode::InvalidArgument, "port mapping"))?
                .clone();
            used.insert(port.name().to_owned());
            outputs.push(wire::OutputPort {
                port_id,
                name: port.name().into(),
                value_type: wire::ValueType::Decimal as i32,
            });
        }
        if used.len() != ports.len() {
            return Err(error(wire::ErrorCode::InvalidArgument, "port mapping"));
        }
        let description = wire::ModelDescription {
            model: Some(model.clone()),
            name: snapshot.name().into(),
            inputs,
            outputs,
        };
        validation::validate_description(&description)?;
        let key = (model.model_id, model.revision_id);
        if self.entries.contains_key(&key) {
            return Err(error(
                wire::ErrorCode::InvalidArgument,
                "revision already registered",
            ));
        }
        // Existing port IDs must continue to refer to the same process-local port
        // within this live registry; deleted/recreated ports require new IDs.
        for ((model_id, _), existing) in &self.entries {
            if model_id != &key.0 {
                continue;
            }
            let old: BTreeMap<_, _> = existing
                .snapshot
                .interface()
                .inputs()
                .map(|p| (p.name(), p.id()))
                .chain(
                    existing
                        .snapshot
                        .interface()
                        .outputs()
                        .map(|p| (p.name(), p.id())),
                )
                .collect();
            let new: BTreeMap<_, _> = snapshot
                .interface()
                .inputs()
                .map(|p| (p.name(), p.id()))
                .chain(snapshot.interface().outputs().map(|p| (p.name(), p.id())))
                .collect();
            let old_ports = existing
                .description
                .inputs
                .iter()
                .map(|p| (&p.port_id, &p.name))
                .chain(
                    existing
                        .description
                        .outputs
                        .iter()
                        .map(|p| (&p.port_id, &p.name)),
                );
            let new_ports: Vec<_> = description
                .inputs
                .iter()
                .map(|p| (&p.port_id, &p.name))
                .chain(description.outputs.iter().map(|p| (&p.port_id, &p.name)))
                .collect();
            for (old_id, old_name) in old_ports {
                for (new_id, new_name) in &new_ports {
                    if (old_id == *new_id) != (old[old_name.as_str()] == new[new_name.as_str()]) {
                        return Err(error(
                            wire::ErrorCode::InvalidArgument,
                            "unstable port mapping",
                        ));
                    }
                }
            }
        }
        Ok(PreparedRegistration {
            key,
            entry: HostedModel {
                snapshot,
                description,
            },
        })
    }
    // Only install immediately after preparation under exclusive registry access.
    // No recoverable errors/callbacks here; allocator aborts are process-fatal.
    fn install_registration(&mut self, prepared: PreparedRegistration) {
        self.entries.insert(prepared.key, prepared.entry);
    }

    /// Dispatch one complete serialized message, never a partial frame. Bad
    /// encodings have no trustworthy request ID and receive an empty correlation.
    pub fn dispatch(&self, session: &mut Session, bytes: &[u8]) -> Vec<u8> {
        let mut request_id = String::new();
        let result = (|| {
            if bytes.len() > session.limits.message_bytes {
                return Err(error(wire::ErrorCode::ResourceExhausted, "message"));
            }
            let request = wire::Request::decode(bytes)
                .map_err(|_| error(wire::ErrorCode::InvalidArgument, "protobuf"))?;
            // Do not echo unchecked arbitrarily long or invalid identifiers.
            if request.request_id.len() <= 128
                && request
                    .request_id
                    .bytes()
                    .all(|b| b.is_ascii_alphanumeric() || b"._-".contains(&b))
            {
                request_id = request.request_id.clone();
            }
            if !session.negotiated
                && !matches!(
                    request.operation,
                    Some(wire::request::Operation::Handshake(_))
                )
            {
                return Err(error(wire::ErrorCode::HandshakeRequired, "handshake"));
            }
            validation::validate_request(&request)?;
            self.handle(session, request)
        })();
        let response = wire::Response {
            request_id: request_id.clone(),
            version: Some(wire::Version { major: 1, minor: 0 }),
            outcome: Some(match result {
                Ok(outcome) => outcome,
                Err(e) => wire::response::Outcome::Error(wire::ProtocolError {
                    code: e.code as i32,
                    message: "request rejected".into(),
                    port_id: String::new(),
                }),
            }),
        };
        if response.encoded_len() > session.limits.message_bytes {
            return wire::Response {
                request_id,
                version: response.version,
                outcome: Some(wire::response::Outcome::Error(wire::ProtocolError {
                    code: wire::ErrorCode::ResourceExhausted as i32,
                    message: "response limit exceeded".into(),
                    port_id: String::new(),
                })),
            }
            .encode_to_vec();
        }
        response.encode_to_vec()
    }
    fn lookup(&self, reference: &wire::ModelRef) -> Result<&HostedModel, ValidationError> {
        self.entries
            .get(&(reference.model_id.clone(), reference.revision_id.clone()))
            .ok_or_else(|| {
                error(
                    if self.entries.keys().any(|(m, _)| m == &reference.model_id) {
                        wire::ErrorCode::RevisionUnavailable
                    } else {
                        wire::ErrorCode::NotFound
                    },
                    "model",
                )
            })
    }
    fn handle(
        &self,
        session: &mut Session,
        request: wire::Request,
    ) -> Result<wire::response::Outcome, ValidationError> {
        use wire::{request::Operation, response::Outcome};
        match request.operation.expect("validated operation") {
            Operation::Handshake(hello) => {
                if session.negotiated {
                    return Err(error(
                        wire::ErrorCode::InvalidArgument,
                        "already negotiated",
                    ));
                }
                let mut accepted = validation::negotiate(&hello)?;
                accepted.max_message_bytes = session.limits.message_bytes as u32;
                accepted.max_inputs = session.limits.inputs as u32;
                accepted.max_outputs = session.limits.outputs as u32;
                session.negotiated = true;
                Ok(Outcome::Handshake(accepted))
            }
            Operation::ListModels(page) => {
                // Live registry is immutable while dispatch borrows it. Cursor
                // stability across registry edits is deliberately not promised.
                let start = if page.continuation_token.is_empty() {
                    0
                } else {
                    page.continuation_token
                        .parse::<usize>()
                        .map_err(|_| error(wire::ErrorCode::InvalidArgument, "cursor"))?
                };
                let entries: Vec<_> = self
                    .entries
                    .values()
                    .filter(|entry| {
                        session
                            .authorizer
                            .allowed(Action::Discover, entry.description.model.as_ref().unwrap())
                    })
                    .collect();
                if start > entries.len() {
                    return Err(error(wire::ErrorCode::InvalidArgument, "cursor"));
                }
                let end = (start
                    + if page.page_size == 0 {
                        100
                    } else {
                        page.page_size as usize
                    })
                .min(entries.len());
                Ok(Outcome::Models(wire::ModelList {
                    models: entries[start..end]
                        .iter()
                        .map(|entry| wire::ModelSummary {
                            model: entry.description.model.clone(),
                            name: entry.description.name.clone(),
                        })
                        .collect(),
                    continuation_token: if end < entries.len() {
                        end.to_string()
                    } else {
                        String::new()
                    },
                }))
            }
            Operation::DescribeModel(describe) => {
                let reference = describe.model.as_ref().expect("validated model");
                if !session.authorizer.allowed(Action::Describe, reference) {
                    return Err(error(wire::ErrorCode::Unauthorized, "model"));
                }
                Ok(Outcome::Description(
                    self.lookup(reference)?.description.clone(),
                ))
            }
            Operation::InvokeModel(call) => {
                let reference = call.model.as_ref().expect("validated model");
                if !session.authorizer.allowed(Action::Invoke, reference) {
                    return Err(error(wire::ErrorCode::Unauthorized, "model"));
                }
                if call.inputs.len() > session.limits.inputs
                    || call.output_port_ids.len() > session.limits.outputs
                {
                    return Err(error(wire::ErrorCode::ResourceExhausted, "ports"));
                }
                let hosted = self.lookup(reference)?;
                validation::validate_invocation(&call, &hosted.description)?;
                self.check_work(&hosted.snapshot)?;
                if hosted.description.inputs.len() > session.limits.inputs {
                    return Err(error(
                        wire::ErrorCode::ResourceExhausted,
                        "effective inputs",
                    ));
                }
                let mut inputs = Vec::new();
                for binding in &call.inputs {
                    let port = hosted
                        .description
                        .inputs
                        .iter()
                        .find(|p| p.port_id == binding.port_id)
                        .unwrap();
                    inputs.push((
                        port.name.clone(),
                        from_wire(binding.value.as_ref().unwrap())?,
                    ));
                }
                let outputs: Vec<_> = call
                    .output_port_ids
                    .iter()
                    .map(|id| {
                        hosted
                            .description
                            .outputs
                            .iter()
                            .find(|p| p.port_id == *id)
                            .unwrap()
                    })
                    .collect();
                let result = EvaluationContext::new(hosted.snapshot.clone())
                    .invoke_named(&NamedInvocationRequest {
                        inputs,
                        outputs: outputs.iter().map(|p| p.name.clone()).collect(),
                    })
                    .map_err(|_| error(wire::ErrorCode::Internal, "engine"))?;
                let mut effective_inputs = Vec::new();
                for port in &hosted.description.inputs {
                    effective_inputs.push(wire::InputBinding {
                        port_id: port.port_id.clone(),
                        value: Some(to_wire(&result.inputs()[&port.name])?),
                        source: if call.inputs.iter().any(|b| b.port_id == port.port_id) {
                            wire::InputSource::Submitted as i32
                        } else {
                            wire::InputSource::Defaulted as i32
                        },
                    });
                }
                let outputs = outputs
                    .iter()
                    .map(|port| {
                        let outcome = match &result.outputs()[&port.name] {
                            Ok(value) => to_wire(value).map(wire::output_result::Outcome::Value),
                            Err(e) => Ok(wire::output_result::Outcome::Error(
                                wire::CalculationError {
                                    code: match e {
                                        PublicEvaluationError::Arithmetic(
                                            DecimalError::Overflow,
                                        ) => wire::CalculationErrorCode::Overflow,
                                        PublicEvaluationError::Arithmetic(
                                            DecimalError::DivisionByZero,
                                        ) => wire::CalculationErrorCode::DivisionByZero,
                                        PublicEvaluationError::DependencyFailed => {
                                            wire::CalculationErrorCode::DependencyFailed
                                        }
                                        _ => wire::CalculationErrorCode::InvalidExpression,
                                    } as i32,
                                    message: "calculation failed".into(),
                                },
                            )),
                        }?;
                        Ok(wire::OutputResult {
                            port_id: port.port_id.clone(),
                            outcome: Some(outcome),
                        })
                    })
                    .collect::<Result<Vec<_>, ValidationError>>()?;
                let mut sequence = self.next_invocation.load(Ordering::Relaxed);
                loop {
                    let next = sequence.checked_add(1).ok_or_else(|| {
                        error(wire::ErrorCode::ResourceExhausted, "invocation ids")
                    })?;
                    match self.next_invocation.compare_exchange_weak(
                        sequence,
                        next,
                        Ordering::Relaxed,
                        Ordering::Relaxed,
                    ) {
                        Ok(_) => break,
                        Err(current) => sequence = current,
                    }
                }
                let result = wire::InvocationResult {
                    invocation_id: format!("{}-{sequence}", self.invocation_namespace),
                    model: call.model.clone(),
                    effective_inputs,
                    outputs,
                };
                validation::validate_result(&result, &call, &hosted.description)
                    .map_err(|_| error(wire::ErrorCode::Internal, "result"))?;
                Ok(Outcome::Invocation(result))
            }
            Operation::GetInvocationResult(_) => Err(error(
                wire::ErrorCode::UnsupportedCapability,
                "result lookup",
            )),
        }
    }
}

fn from_wire(value: &wire::Value) -> Result<PivotValue, ValidationError> {
    validation::validate_value(value)?;
    let Some(wire::value::Kind::Decimal(d)) = &value.kind else {
        unreachable!()
    };
    Ok(
        PivotDecimal::from_parts(d.lo, d.mid, d.hi, d.negative, d.scale)
            .map_err(|_| error(wire::ErrorCode::InvalidArgument, "decimal"))?
            .into(),
    )
}
fn to_wire(value: &PivotValue) -> Result<wire::Value, ValidationError> {
    match value {
        PivotValue::Decimal(d) => {
            let (lo, mid, hi, negative, scale) = d.parts();
            Ok(wire::Value {
                kind: Some(wire::value::Kind::Decimal(wire::Decimal128 {
                    lo,
                    mid,
                    hi,
                    negative,
                    scale,
                })),
            })
        }
        _ => Err(error(wire::ErrorCode::UnsupportedValueType, "value")),
    }
}
