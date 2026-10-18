use crate::v1::*;
use prost::Message;
use std::collections::{BTreeMap, BTreeSet};

pub const MAX_MESSAGE_BYTES: usize = 1024 * 1024;
pub const MAX_PORTS: usize = 1024;
pub const CAPABILITIES: &[&str] = &["decimal96", "model-discovery", "synchronous-invocation"];

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ValidationError {
    pub code: ErrorCode,
    pub field: String,
}
impl std::fmt::Display for ValidationError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}: {}", self.code, self.field)
    }
}
impl std::error::Error for ValidationError {}
fn err(code: ErrorCode, field: &str) -> ValidationError {
    ValidationError {
        code,
        field: field.into(),
    }
}
fn invalid(field: &str) -> ValidationError {
    err(ErrorCode::InvalidArgument, field)
}
fn id(value: &str, field: &str) -> Result<(), ValidationError> {
    if value.is_empty()
        || value.len() > 128
        || !value
            .bytes()
            .all(|b| b.is_ascii_alphanumeric() || b"._-".contains(&b))
    {
        return Err(invalid(field));
    }
    Ok(())
}
pub fn validate_decimal(value: &Decimal128) -> Result<(), ValidationError> {
    if value.scale > 28 {
        return Err(invalid("decimal.scale"));
    }
    Ok(())
}
pub fn validate_value(value: &Value) -> Result<(), ValidationError> {
    match &value.kind {
        Some(value::Kind::Decimal(value)) => validate_decimal(value),
        _ => Err(err(ErrorCode::UnsupportedValueType, "value.kind")),
    }
}
/// Mathematical decimal equality, independent of scale and signed zero.
/// Derived message equality remains available for exact representation checks.
pub fn numeric_equal(left: &Value, right: &Value) -> Result<bool, ValidationError> {
    validate_value(left)?;
    validate_value(right)?;
    fn canonical(value: &Value) -> (u128, u32, bool) {
        let Some(value::Kind::Decimal(d)) = &value.kind else {
            unreachable!()
        };
        let mut coefficient =
            u128::from(d.lo) | (u128::from(d.mid) << 32) | (u128::from(d.hi) << 64);
        let mut scale = d.scale;
        if coefficient == 0 {
            return (0, 0, false);
        }
        while scale > 0 && coefficient % 10 == 0 {
            coefficient /= 10;
            scale -= 1;
        }
        (coefficient, scale, d.negative)
    }
    Ok(canonical(left) == canonical(right))
}
fn model(value: Option<&ModelRef>) -> Result<(), ValidationError> {
    let value = value.ok_or_else(|| invalid("model"))?;
    id(&value.model_id, "model_id")?;
    id(&value.revision_id, "revision_id")
}
pub fn negotiate(hello: &Handshake) -> Result<HandshakeAccepted, ValidationError> {
    if hello.encoded_len() > MAX_MESSAGE_BYTES {
        return Err(err(ErrorCode::ResourceExhausted, "handshake"));
    }
    if hello.supported_versions.len() > 32
        || hello.required_capabilities.len() + hello.optional_capabilities.len() > 64
    {
        return Err(err(ErrorCode::ResourceExhausted, "handshake"));
    }
    if !hello
        .supported_versions
        .iter()
        .any(|v| v.major == 1 && v.minor == 0)
    {
        return Err(err(ErrorCode::UnsupportedVersion, "supported_versions"));
    }
    for required in &hello.required_capabilities {
        if !CAPABILITIES.contains(&required.as_str()) {
            return Err(err(
                ErrorCode::UnsupportedCapability,
                "required_capabilities",
            ));
        }
    }
    // Advertise supported capabilities; the client enables only those it understands.
    Ok(HandshakeAccepted {
        selected_version: Some(Version { major: 1, minor: 0 }),
        capabilities: CAPABILITIES.iter().map(|s| s.to_string()).collect(),
        max_message_bytes: MAX_MESSAGE_BYTES as u32,
        max_inputs: MAX_PORTS as u32,
        max_outputs: MAX_PORTS as u32,
    })
}
pub fn decode_request(bytes: &[u8]) -> Result<Request, ValidationError> {
    if bytes.len() > MAX_MESSAGE_BYTES {
        return Err(err(ErrorCode::ResourceExhausted, "message"));
    }
    let request = Request::decode(bytes).map_err(|_| invalid("protobuf"))?;
    validate_request(&request)?;
    Ok(request)
}
pub fn validate_request(request: &Request) -> Result<(), ValidationError> {
    if request.encoded_len() > MAX_MESSAGE_BYTES {
        return Err(err(ErrorCode::ResourceExhausted, "message"));
    }
    id(&request.request_id, "request_id")?;
    match &request.operation {
        Some(request::Operation::Handshake(hello)) => {
            negotiate(hello)?;
        }
        Some(operation) => {
            if request.version != Some(Version { major: 1, minor: 0 }) {
                return Err(err(ErrorCode::UnsupportedVersion, "version"));
            }
            match operation {
                request::Operation::ListModels(page) => {
                    if page.page_size > 100 || page.continuation_token.len() > 512 {
                        return Err(invalid("pagination"));
                    }
                }
                request::Operation::DescribeModel(value) => model(value.model.as_ref())?,
                request::Operation::InvokeModel(value) => {
                    model(value.model.as_ref())?;
                    if value.inputs.len() > MAX_PORTS || value.output_port_ids.len() > MAX_PORTS {
                        return Err(err(ErrorCode::ResourceExhausted, "ports"));
                    }
                    let mut seen = BTreeSet::new();
                    for binding in &value.inputs {
                        id(&binding.port_id, "input.port_id")?;
                        if binding.source != InputSource::Unspecified as i32 {
                            return Err(invalid("input.source"));
                        }
                        if !seen.insert(&binding.port_id) {
                            return Err(err(ErrorCode::DuplicateBinding, "inputs"));
                        }
                        validate_value(
                            binding
                                .value
                                .as_ref()
                                .ok_or_else(|| invalid("input.value"))?,
                        )?;
                    }
                    let mut seen = BTreeSet::new();
                    for output in &value.output_port_ids {
                        id(output, "output.port_id")?;
                        if !seen.insert(output) {
                            return Err(err(ErrorCode::DuplicateBinding, "outputs"));
                        }
                    }
                }
                request::Operation::GetInvocationResult(_) => {
                    return Err(err(ErrorCode::UnsupportedCapability, "result-lookup"));
                }
                request::Operation::Handshake(_) => unreachable!(),
            }
        }
        None => return Err(invalid("operation")),
    }
    Ok(())
}
pub fn validate_description(description: &ModelDescription) -> Result<(), ValidationError> {
    if description.encoded_len() > MAX_MESSAGE_BYTES {
        return Err(err(ErrorCode::ResourceExhausted, "description"));
    }
    model(description.model.as_ref())?;
    if description.name.is_empty() || description.name.len() > 256 {
        return Err(invalid("model.name"));
    }
    if description.inputs.len() > MAX_PORTS || description.outputs.len() > MAX_PORTS {
        return Err(err(ErrorCode::ResourceExhausted, "ports"));
    }
    let mut ids = BTreeSet::new();
    let mut names = BTreeSet::new();
    for (port_id, name, ty) in description
        .inputs
        .iter()
        .map(|p| (&p.port_id, &p.name, p.value_type))
        .chain(
            description
                .outputs
                .iter()
                .map(|p| (&p.port_id, &p.name, p.value_type)),
        )
    {
        id(port_id, "port_id")?;
        if name.is_empty() || name.len() > 128 || !ids.insert(port_id) || !names.insert(name) {
            return Err(invalid("port.declaration"));
        }
        if ty != ValueType::Decimal as i32 {
            return Err(err(ErrorCode::UnsupportedValueType, "port.type"));
        }
    }
    for input in &description.inputs {
        match &input.requirement {
            Some(input_port::Requirement::Required(true)) => (),
            Some(input_port::Requirement::DefaultValue(value)) => validate_value(value)?,
            _ => return Err(invalid("input.requirement")),
        }
    }
    Ok(())
}
/// Validate against the exact revision's public interface; no runtime NodeIds.
pub fn validate_invocation(
    value: &InvokeModel,
    description: &ModelDescription,
) -> Result<(), ValidationError> {
    validate_description(description)?;
    validate_request(&Request {
        request_id: "validation".into(),
        version: Some(Version { major: 1, minor: 0 }),
        operation: Some(request::Operation::InvokeModel(value.clone())),
    })?;
    if value.model != description.model {
        return Err(err(ErrorCode::RevisionUnavailable, "model"));
    }
    let ports: BTreeMap<_, _> = description.inputs.iter().map(|p| (&p.port_id, p)).collect();
    for input in &value.inputs {
        if !ports.contains_key(&input.port_id) {
            return Err(err(ErrorCode::UnknownPort, &input.port_id));
        }
    }
    for port in &description.inputs {
        if matches!(
            port.requirement,
            Some(input_port::Requirement::Required(true))
        ) && !value.inputs.iter().any(|b| b.port_id == port.port_id)
        {
            return Err(err(ErrorCode::MissingInput, &port.port_id));
        }
    }
    for output in &value.output_port_ids {
        if !description.outputs.iter().any(|p| p.port_id == *output) {
            return Err(err(ErrorCode::UnknownPort, output));
        }
    }
    Ok(())
}

pub fn validate_result(
    result: &InvocationResult,
    invocation: &InvokeModel,
    description: &ModelDescription,
) -> Result<(), ValidationError> {
    if result.encoded_len() > MAX_MESSAGE_BYTES {
        return Err(err(ErrorCode::ResourceExhausted, "result"));
    }
    validate_invocation(invocation, description)?;
    id(&result.invocation_id, "invocation_id")?;
    if result.model != invocation.model {
        return Err(err(ErrorCode::RevisionUnavailable, "result.model"));
    }
    let expected: BTreeSet<_> = invocation.output_port_ids.iter().collect();
    let mut seen = BTreeSet::new();
    for output in &result.outputs {
        if !expected.contains(&output.port_id) || !seen.insert(&output.port_id) {
            return Err(invalid("result.outputs"));
        }
        match &output.outcome {
            Some(output_result::Outcome::Value(value)) => validate_value(value)?,
            Some(output_result::Outcome::Error(error))
                if matches!(
                    CalculationErrorCode::try_from(error.code),
                    Ok(CalculationErrorCode::Overflow
                        | CalculationErrorCode::DivisionByZero
                        | CalculationErrorCode::DependencyFailed
                        | CalculationErrorCode::InvalidExpression)
                ) && error.message.len() <= 1024 => {}
            _ => return Err(invalid("output.outcome")),
        }
    }
    if seen != expected {
        return Err(invalid("result.outputs"));
    }
    let mut bindings = BTreeMap::new();
    for binding in &result.effective_inputs {
        if bindings
            .insert(&binding.port_id, binding.value.as_ref())
            .is_some()
        {
            return Err(err(ErrorCode::DuplicateBinding, "effective_inputs"));
        }
    }
    if bindings.len() != description.inputs.len() {
        return Err(invalid("effective_inputs"));
    }
    for port in &description.inputs {
        let expected = invocation
            .inputs
            .iter()
            .find(|b| b.port_id == port.port_id)
            .and_then(|b| b.value.as_ref())
            .or(match &port.requirement {
                Some(input_port::Requirement::DefaultValue(value)) => Some(value),
                _ => None,
            });
        let actual_binding = result
            .effective_inputs
            .iter()
            .find(|b| b.port_id == port.port_id)
            .ok_or_else(|| invalid("effective_inputs"))?;
        let source = if invocation.inputs.iter().any(|b| b.port_id == port.port_id) {
            InputSource::Submitted
        } else {
            InputSource::Defaulted
        };
        if actual_binding.source != source as i32 {
            return Err(invalid("effective_inputs.source"));
        }
        let actual = actual_binding
            .value
            .as_ref()
            .ok_or_else(|| invalid("effective_inputs.value"))?;
        if !numeric_equal(actual, expected.ok_or_else(|| invalid("effective_inputs"))?)? {
            return Err(invalid("effective_inputs"));
        }
    }
    Ok(())
}

/// Client-side response correlation/shape validation. Session/auth policy is separate.
pub fn decode_response(bytes: &[u8], request: &Request) -> Result<Response, ValidationError> {
    if bytes.len() > MAX_MESSAGE_BYTES {
        return Err(err(ErrorCode::ResourceExhausted, "message"));
    }
    let response = Response::decode(bytes).map_err(|_| invalid("protobuf"))?;
    if response.request_id != request.request_id {
        return Err(invalid("request_id"));
    }
    if response.version != Some(Version { major: 1, minor: 0 }) {
        return Err(err(ErrorCode::UnsupportedVersion, "version"));
    }
    match (&request.operation, &response.outcome) {
        (_, Some(response::Outcome::Error(error))) => {
            if error.code == 0
                || ErrorCode::try_from(error.code).is_err()
                || error.message.len() > 1024
                || error.port_id.len() > 128
            {
                return Err(invalid("error"));
            }
        }
        (
            Some(request::Operation::Handshake(hello)),
            Some(response::Outcome::Handshake(accepted)),
        ) => {
            let version = accepted
                .selected_version
                .as_ref()
                .ok_or_else(|| invalid("selected_version"))?;
            if !hello.supported_versions.contains(version)
                || accepted.selected_version != response.version
                || accepted.max_message_bytes == 0
                || accepted.max_message_bytes as usize > MAX_MESSAGE_BYTES
                || accepted.max_inputs == 0
                || accepted.max_inputs as usize > MAX_PORTS
                || accepted.max_outputs == 0
                || accepted.max_outputs as usize > MAX_PORTS
            {
                return Err(invalid("handshake"));
            }
            for capability in &hello.required_capabilities {
                if !accepted.capabilities.contains(capability) {
                    return Err(err(ErrorCode::UnsupportedCapability, "capabilities"));
                }
            }
        }
        (Some(request::Operation::ListModels(_)), Some(response::Outcome::Models(list))) => {
            if list.models.len() > 100 || list.continuation_token.len() > 512 {
                return Err(invalid("models"));
            }
            for entry in &list.models {
                model(entry.model.as_ref())?;
                if entry.name.is_empty() || entry.name.len() > 256 {
                    return Err(invalid("model.name"));
                }
            }
        }
        (
            Some(request::Operation::DescribeModel(describe)),
            Some(response::Outcome::Description(description)),
        ) => {
            validate_description(description)?;
            if describe.model != description.model {
                return Err(err(ErrorCode::RevisionUnavailable, "model"));
            }
        }
        (
            Some(request::Operation::InvokeModel(invocation)),
            Some(response::Outcome::Invocation(result)),
        ) => {
            id(&result.invocation_id, "invocation_id")?;
            if result.effective_inputs.len() > MAX_PORTS {
                return Err(err(ErrorCode::ResourceExhausted, "effective_inputs"));
            }
            let mut input_ids = BTreeSet::new();
            for binding in &result.effective_inputs {
                id(&binding.port_id, "input.port_id")?;
                if !input_ids.insert(&binding.port_id) {
                    return Err(err(ErrorCode::DuplicateBinding, "effective_inputs"));
                }
                if !matches!(
                    InputSource::try_from(binding.source),
                    Ok(InputSource::Submitted | InputSource::Defaulted)
                ) {
                    return Err(invalid("effective_inputs.source"));
                }
                validate_value(
                    binding
                        .value
                        .as_ref()
                        .ok_or_else(|| invalid("input.value"))?,
                )?;
            }
            if invocation.model != result.model {
                return Err(err(ErrorCode::RevisionUnavailable, "model"));
            }
            let expected: BTreeSet<_> = invocation.output_port_ids.iter().collect();
            let mut seen = BTreeSet::new();
            for output in &result.outputs {
                if !expected.contains(&output.port_id) || !seen.insert(&output.port_id) {
                    return Err(invalid("outputs"));
                }
                match &output.outcome {
                    Some(output_result::Outcome::Value(value)) => validate_value(value)?,
                    Some(output_result::Outcome::Error(error))
                        if error.code != 0
                            && CalculationErrorCode::try_from(error.code).is_ok()
                            && error.message.len() <= 1024 => {}
                    _ => return Err(invalid("outcome")),
                }
            }
            if seen != expected {
                return Err(invalid("outputs"));
            }
        }
        _ => return Err(invalid("response.outcome")),
    }
    Ok(response)
}
