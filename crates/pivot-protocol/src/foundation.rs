//! Schema-aware bounded preflight before generated decoding. No service dispatch.
use crate::{
    authoring::v1 as a,
    v1,
    validation::{ValidationError, validate_value},
};
use prost::Message;
use prost_types::{DescriptorProto, FieldDescriptorProto, FileDescriptorSet};
use std::{
    collections::{BTreeMap, BTreeSet},
    sync::OnceLock,
};

#[derive(Clone, Copy)]
pub struct Limits {
    pub bytes: usize,
    pub depth: usize,
    pub fields: usize,
    pub edits: usize,
    pub text: usize,
    pub inputs: usize,
    pub outputs: usize,
    pub name: usize,
    pub cursor: usize,
}
impl Default for Limits {
    fn default() -> Self {
        Self {
            bytes: 1024 * 1024,
            depth: 32,
            fields: 65536,
            edits: 128,
            text: 4096,
            inputs: 1024,
            outputs: 1024,
            name: 128,
            cursor: 512,
        }
    }
}
fn bad(field: &str) -> ValidationError {
    ValidationError {
        code: v1::ErrorCode::InvalidArgument,
        field: field.into(),
    }
}
fn exhausted() -> ValidationError {
    ValidationError {
        code: v1::ErrorCode::ResourceExhausted,
        field: "authoring limits".into(),
    }
}
struct Schema {
    messages: BTreeMap<String, DescriptorProto>,
    enums: BTreeMap<String, BTreeSet<i32>>,
}
fn schema() -> &'static Schema {
    static SCHEMA: OnceLock<Schema> = OnceLock::new();
    SCHEMA.get_or_init(|| {
        let mut s = Schema {
            messages: BTreeMap::new(),
            enums: BTreeMap::new(),
        };
        fn add(s: &mut Schema, prefix: &str, d: DescriptorProto) {
            let name = format!("{prefix}.{}", d.name());
            for n in &d.nested_type {
                add(s, &name, n.clone());
            }
            for e in &d.enum_type {
                s.enums.insert(
                    format!("{name}.{}", e.name()),
                    e.value.iter().map(|v| v.number()).collect(),
                );
            }
            s.messages.insert(name, d);
        }
        for f in FileDescriptorSet::decode(crate::DESCRIPTORS).unwrap().file {
            let prefix = format!(".{}", f.package());
            for e in &f.enum_type {
                s.enums.insert(
                    format!("{prefix}.{}", e.name()),
                    e.value.iter().map(|v| v.number()).collect(),
                );
            }
            for d in f.message_type {
                add(&mut s, &prefix, d);
            }
        }
        s
    })
}
fn varint(bytes: &[u8], position: &mut usize) -> Result<u64, ValidationError> {
    let mut value = 0;
    for shift in (0..70).step_by(7) {
        let b = *bytes
            .get(*position)
            .ok_or_else(|| bad("truncated varint"))?;
        *position += 1;
        if shift == 63 && b > 1 {
            return Err(bad("varint overflow"));
        }
        value |= u64::from(b & 127) << shift;
        if b < 128 {
            return Ok(value);
        }
    }
    Err(bad("varint"))
}
fn uuid(s: &str) -> bool {
    s.len() == 36
        && s.bytes().enumerate().all(|(i, b)| {
            if [8, 13, 18, 23].contains(&i) {
                b == b'-'
            } else {
                b.is_ascii_digit() || (b'a'..=b'f').contains(&b)
            }
        })
}
fn identifier(s: &str) -> bool {
    !s.is_empty()
        && s.len() <= 128
        && s.bytes()
            .all(|b| b.is_ascii_alphanumeric() || b"._-".contains(&b))
}
fn required(name: &str) -> &'static [i32] {
    match name {
        "Request" => &[1, 2],
        "WorkingRevision" | "ModelRef" => &[1, 2],
        "Provenance" => &[1, 2],
        "MutationContext" => &[1, 2, 3, 4],
        "CreateWorking" => &[1, 2, 3],
        "Created" => &[1, 2],
        "ListWorking" => &[1],
        "OpenWorking" | "CloseAttachment" | "GetOperationOutcome" => &[1, 2],
        "Opened" => &[1, 2],
        "ReadDraft" => &[1, 2],
        "ReadDiagnostics" => &[1, 2],
        "ApplyEdits" => &[1, 2],
        "ValidateWorking" | "PublishWorking" => &[1],
        "EvaluateWorking" => &[1, 3],
        "AddNode" => &[1, 2, 3],
        "RemoveNode" | "RemovePort" => &[1],
        "RenameNode" | "RenamePort" | "SetNode" => &[1, 2],
        "AddInputPort" => &[1, 2, 3, 4],
        "AddOutputPort" => &[1, 2, 3],
        "SetInputPort" => &[1],
        "SetOutputPort" => &[1, 2],
        "Override" | "NodeMapping" | "PortMapping" => &[1, 2],
        "ServiceInfo" => &[1, 2, 3, 4],
        "WorkingSummary" => &[1, 2, 3],
        "EditCommitted" => &[1, 4],
        "RawNode" => &[1, 2, 3],
        "RawPort" => &[1, 2],
        "RawInputPort" => &[1, 2],
        "RawOutputPort" => &[1],
        "DraftPage" => &[1, 2],
        "Diagnostic" => &[1],
        "DiagnosticSummary" => &[1],
        "DiagnosticCount" => &[1, 2],
        "DiagnosticsPage" => &[1],
        "ValidationReport" => &[1, 3],
        "EffectiveInput" => &[1, 2, 3],
        "CalculationFailure" => &[1],
        "NodeResult" => &[1],
        "EvaluationResult" => &[1, 3],
        "Published" => &[1, 2],
        "AuthoringFailure" => &[1],
        _ => &[],
    }
}
fn walk(
    name: &str,
    bytes: &[u8],
    limits: Limits,
    depth: usize,
    remaining: &mut usize,
) -> Result<(), ValidationError> {
    if depth > limits.depth {
        return Err(exhausted());
    }
    let d = schema().messages.get(name).ok_or_else(|| bad("schema"))?;
    let short = d.name();
    let mut seen = BTreeMap::<i32, usize>::new();
    let mut oneofs = BTreeSet::new();
    let mut p = 0;
    while p < bytes.len() {
        *remaining = remaining.checked_sub(1).ok_or_else(exhausted)?;
        let key = varint(bytes, &mut p)?;
        let number = i32::try_from(key >> 3).map_err(|_| bad("field number"))?;
        let f = d
            .field
            .iter()
            .find(|f| f.number() == number)
            .ok_or_else(|| bad("unknown field"))?;
        let count = seen.entry(number).or_default();
        *count += 1;
        if f.label() != prost_types::field_descriptor_proto::Label::Repeated && *count > 1 {
            return Err(bad("duplicate singular"));
        }
        if let Some(index) = f.oneof_index
            && !oneofs.insert(index)
        {
            return Err(bad("conflicting oneof"));
        }
        if short == "ApplyEdits" && number == 2 && *count > limits.edits {
            return Err(exhausted());
        }
        if short == "EvaluateWorking"
            && ((number == 2 && *count > limits.inputs) || (number == 3 && *count > limits.outputs))
        {
            return Err(exhausted());
        }
        let wire = (key & 7) as u8;
        use prost_types::field_descriptor_proto::Type;
        let expected = match f.r#type() {
            Type::Fixed32 | Type::Sfixed32 | Type::Float => 5,
            Type::Fixed64 | Type::Sfixed64 | Type::Double => 1,
            Type::String | Type::Bytes | Type::Message => 2,
            _ => 0,
        };
        if wire != expected {
            return Err(bad("wire type"));
        }
        match wire {
            0 => {
                let v = varint(bytes, &mut p)?;
                if matches!(f.r#type(), Type::Uint32) && v > u32::MAX as u64 {
                    return Err(bad("uint32"));
                }
                if f.r#type() == Type::Bool && v > 1 {
                    return Err(bad("boolean"));
                }
                if f.r#type() == Type::Enum
                    && (!schema().enums[f.type_name()].contains(&(v as i32))
                        || v == 0
                        || v > i32::MAX as u64)
                {
                    return Err(bad("enum"));
                }
                if short == "Decimal128" && f.name() == "scale" && v > 28 {
                    return Err(bad("scale"));
                }
            }
            1 | 5 => {
                p = p
                    .checked_add(if wire == 1 { 8 } else { 4 })
                    .ok_or_else(exhausted)?;
                if p > bytes.len() {
                    return Err(bad("truncated fixed"));
                }
            }
            2 => {
                let len = usize::try_from(varint(bytes, &mut p)?).map_err(|_| exhausted())?;
                let end = p.checked_add(len).ok_or_else(exhausted)?;
                let value = bytes.get(p..end).ok_or_else(|| bad("truncated field"))?;
                p = end;
                if f.r#type() == Type::Message {
                    walk(f.type_name(), value, limits, depth + 1, remaining)?;
                } else {
                    if len > limits.text {
                        return Err(exhausted());
                    }
                    let s = std::str::from_utf8(value).map_err(|_| bad("utf8"))?;
                    validate_text(name, f, s)?;
                    if f.name() == "name"
                        && short != "CreateWorking"
                        && short != "WorkingSummary"
                        && s.len() > limits.name
                    {
                        return Err(exhausted());
                    }
                    if f.name() == "continuation_token" && s.len() > limits.cursor {
                        return Err(exhausted());
                    }
                }
            }
            _ => return Err(bad("wire type")),
        }
    }
    for field in required(short) {
        if !seen.contains_key(field) {
            return Err(bad("missing field"));
        }
    }
    // Every real oneof is semantically required; optional proto3 scalars use
    // synthetic oneofs and are allowed to be absent (offset zero is meaningful).
    for (index, _) in d.oneof_decl.iter().enumerate() {
        let synthetic = d
            .field
            .iter()
            .any(|f| f.oneof_index == Some(index as i32) && f.proto3_optional());
        if !synthetic && !oneofs.contains(&(index as i32)) {
            return Err(bad("missing alternative"));
        }
    }
    if short == "SetInputPort" && !seen.contains_key(&2) && !seen.contains_key(&3) {
        return Err(bad("empty port patch"));
    }
    if short == "Value" {
        validate_value(&v1::Value::decode(bytes).map_err(|_| bad("value"))?)?;
    }
    Ok(())
}
fn validate_text(message: &str, f: &FieldDescriptorProto, s: &str) -> Result<(), ValidationError> {
    let name = f.name();
    if message == ".pivot.v1.ModelRef" {
        if !identifier(s) {
            return Err(bad(name));
        }
        return Ok(());
    }
    if name.ends_with("_id") || name.ends_with("_ids") {
        if ["operation_id", "temporary_id", "request_id"].contains(&name) {
            if !identifier(s) {
                return Err(bad(name));
            }
        } else if !uuid(s) {
            return Err(bad(name));
        }
    }
    if name == "name" && s.is_empty() {
        return Err(bad("name"));
    }
    if name == "name"
        && ![
            ".pivot.authoring.v1.CreateWorking",
            ".pivot.authoring.v1.WorkingSummary",
        ]
        .contains(&message)
    {
        let mut b = s.bytes();
        if !b
            .next()
            .is_some_and(|b| b.is_ascii_alphabetic() || b == b'_')
            || !b.all(|b| b.is_ascii_alphanumeric() || b == b'_')
        {
            return Err(bad("identifier name"));
        }
    }
    Ok(())
}
pub fn decode_authoring(
    bytes: &[u8],
    limits: Limits,
) -> Result<a::AuthoringRequest, ValidationError> {
    if bytes.len() > limits.bytes {
        return Err(exhausted());
    }
    let mut remaining = limits.fields;
    walk(
        ".pivot.authoring.v1.AuthoringRequest",
        bytes,
        limits,
        0,
        &mut remaining,
    )?;
    let request = a::AuthoringRequest::decode(bytes).map_err(|_| bad("decode"))?;
    use a::authoring_request::Operation;
    let mutation = match &request.operation {
        Some(Operation::ApplyEdits(x)) => x.mutation.as_ref(),
        Some(Operation::PublishWorking(x)) => x.mutation.as_ref(),
        _ => None,
    };
    if let Some(m) = mutation {
        let r = m
            .expected_revision
            .as_ref()
            .ok_or_else(|| bad("expected"))?;
        if r.service_instance_id != m.service_instance_id
            || r.working_model_id != m.working_model_id
        {
            return Err(bad("revision scope"));
        }
    }
    if let Some(Operation::EvaluateWorking(v)) = &request.operation
        && (v
            .inputs
            .iter()
            .map(|v| &v.node_id)
            .collect::<BTreeSet<_>>()
            .len()
            != v.inputs.len()
            || v.output_node_ids.iter().collect::<BTreeSet<_>>().len() != v.output_node_ids.len())
    {
        return Err(bad("duplicate calculation binding"));
    }
    Ok(request)
}
pub fn validate_authoring_response(
    response: &a::AuthoringResponse,
    limits: Limits,
) -> Result<(), ValidationError> {
    let bytes = response.encode_to_vec();
    if bytes.len() > limits.bytes {
        return Err(exhausted());
    }
    let mut remaining = limits.fields;
    walk(
        ".pivot.authoring.v1.AuthoringResponse",
        &bytes,
        limits,
        0,
        &mut remaining,
    )
}
/// Structural fragment checks for codecs/tests, not authorization or admission.
pub fn validate_message(name: &str, bytes: &[u8], limits: Limits) -> Result<(), ValidationError> {
    if !name.starts_with(".pivot.authoring.v1.") || bytes.len() > limits.bytes {
        return Err(exhausted());
    }
    let mut remaining = limits.fields;
    walk(name, bytes, limits, 0, &mut remaining)
}
pub fn decode_envelope(bytes: &[u8], limits: Limits) -> Result<v1::Request, ValidationError> {
    if bytes.len() > limits.bytes {
        return Err(exhausted());
    }
    let mut remaining = limits.fields;
    walk(".pivot.v1.Request", bytes, limits, 0, &mut remaining)?;
    let request = v1::Request::decode(bytes).map_err(|_| bad("envelope"))?;
    if request.version != Some(v1::Version { major: 1, minor: 1 }) {
        return Err(bad("authoring version"));
    }
    let Some(v1::request::Operation::Authoring(a)) = &request.operation else {
        return Err(bad("authoring operation"));
    };
    decode_authoring(&a.encode_to_vec(), limits)?;
    Ok(request)
}
/// Read-only response decoding tolerates additive unknown fields, while checking
/// correlation, result alternative and exact snapshot provenance.
pub fn decode_response(
    bytes: &[u8],
    request: &v1::Request,
    limits: Limits,
) -> Result<v1::Response, ValidationError> {
    if bytes.len() > limits.bytes {
        return Err(exhausted());
    }
    let response = v1::Response::decode(bytes).map_err(|_| bad("response"))?;
    if response.request_id != request.request_id || response.version != request.version {
        return Err(bad("response correlation"));
    }
    let Some(v1::request::Operation::Authoring(req)) = &request.operation else {
        return Err(bad("authoring request"));
    };
    let Some(v1::response::Outcome::Authoring(res)) = &response.outcome else {
        return Err(bad("authoring response"));
    };
    validate_authoring_response(res, limits)?;
    use a::{authoring_request::Operation as Q, authoring_response::Outcome as R};
    let matched = matches!(
        (&req.operation, &res.outcome),
        (_, Some(R::Error(_)))
            | (Some(Q::GetServiceInfo(_)), Some(R::ServiceInfo(_)))
            | (Some(Q::CreateWorking(_)), Some(R::Created(_)))
            | (Some(Q::ListWorking(_)), Some(R::WorkingModels(_)))
            | (Some(Q::OpenWorking(_)), Some(R::Opened(_)))
            | (Some(Q::CloseAttachment(_)), Some(R::Closed(_)))
            | (Some(Q::ReadDraft(_)), Some(R::Draft(_)))
            | (Some(Q::ReadDiagnostics(_)), Some(R::Diagnostics(_)))
            | (Some(Q::ApplyEdits(_)), Some(R::Edited(_)))
            | (Some(Q::ValidateWorking(_)), Some(R::Validation(_)))
            | (Some(Q::EvaluateWorking(_)), Some(R::Evaluation(_)))
            | (Some(Q::PublishWorking(_)), Some(R::Published(_)))
            | (
                Some(Q::GetOperationOutcome(_)),
                Some(R::OperationOutcome(_))
            )
    );
    if !matched {
        return Err(bad("response alternative"));
    }
    let (expected, actual) = match (&req.operation, &res.outcome) {
        (Some(Q::ReadDraft(q)), Some(R::Draft(r))) => (q.revision.as_ref(), r.revision.as_ref()),
        (Some(Q::ReadDiagnostics(q)), Some(R::Diagnostics(r))) => (
            q.working_revision.as_ref(),
            r.summary.as_ref().and_then(|s| s.revision.as_ref()),
        ),
        (Some(Q::ValidateWorking(q)), Some(R::Validation(r))) => {
            (q.revision.as_ref(), r.revision.as_ref())
        }
        (Some(Q::EvaluateWorking(q)), Some(R::Evaluation(r))) => (
            q.revision.as_ref(),
            r.provenance.as_ref().and_then(|p| p.working.as_ref()),
        ),
        (Some(Q::PublishWorking(q)), Some(R::Published(r))) => (
            q.mutation
                .as_ref()
                .and_then(|m| m.expected_revision.as_ref()),
            r.provenance.as_ref().and_then(|p| p.working.as_ref()),
        ),
        _ => (None, None),
    };
    if expected != actual {
        return Err(bad("response revision"));
    }
    if let (Some(Q::EvaluateWorking(q)), Some(R::Evaluation(r))) = (&req.operation, &res.outcome) {
        let selected: BTreeSet<_> = q.output_node_ids.iter().collect();
        let returned: BTreeSet<_> = r.outputs.iter().map(|o| &o.node_id).collect();
        if selected != returned || returned.len() != r.outputs.len() {
            return Err(bad("output set"));
        }
    }
    Ok(response)
}
#[derive(Debug, Clone, PartialEq)]
pub enum MutationIntent {
    Create(a::CreateWorking),
    Edit(a::ApplyEdits),
    Publish(a::PublishWorking),
}
/// Use only after raw preflight; generated decoding alone discards ambiguity.
pub fn mutation_intent(request: &a::AuthoringRequest) -> Option<MutationIntent> {
    use a::authoring_request::Operation;
    match request.operation.as_ref()? {
        Operation::CreateWorking(v) => Some(MutationIntent::Create(v.clone())),
        Operation::ApplyEdits(v) => Some(MutationIntent::Edit(v.clone())),
        Operation::PublishWorking(v) => Some(MutationIntent::Publish(v.clone())),
        _ => None,
    }
}
/// Opt-in foundation negotiation. Existing daemon calls negotiate(), which never
/// advertises authoring or selects 1.1 before adapter integration is authorized.
pub fn negotiate(hello: &v1::Handshake) -> Result<v1::HandshakeAccepted, ValidationError> {
    if hello.encoded_len() > crate::validation::MAX_MESSAGE_BYTES
        || hello.supported_versions.len() > 32
        || hello.required_capabilities.len() + hello.optional_capabilities.len() > 64
    {
        return Err(exhausted());
    }
    if !hello
        .supported_versions
        .iter()
        .any(|v| v.major == 1 && v.minor == 1)
    {
        return crate::validation::negotiate(hello);
    }
    let mut base = hello.clone();
    base.supported_versions = vec![v1::Version { major: 1, minor: 0 }];
    base.required_capabilities
        .retain(|s| s != "working-models-v1");
    let mut accepted = crate::validation::negotiate(&base)?;
    accepted.selected_version = Some(v1::Version { major: 1, minor: 1 });
    accepted.capabilities.push("working-models-v1".into());
    Ok(accepted)
}
