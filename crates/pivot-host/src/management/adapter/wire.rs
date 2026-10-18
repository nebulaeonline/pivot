use super::*;
pub(super) fn parse<T: std::str::FromStr>(s: &str) -> Result<T, Failure> {
    s.parse()
        .map_err(|_| fail(a::AuthoringErrorCode::InvalidArgument))
}
pub(super) fn revision(r: &a::WorkingRevision) -> Result<WorkingRevision, Failure> {
    Ok(WorkingRevision {
        instance: parse(&r.service_instance_id)?,
        model: parse(&r.working_model_id)?,
        sequence: r.sequence,
    })
}
pub(super) fn rev(r: WorkingRevision) -> a::WorkingRevision {
    a::WorkingRevision {
        service_instance_id: r.instance.to_string(),
        working_model_id: r.model.to_string(),
        sequence: r.sequence,
    }
}
pub(super) fn value(v: &p::Value) -> Result<pivot_types::PivotDecimal, Failure> {
    let Some(p::value::Kind::Decimal(d)) = &v.kind else {
        return Err(fail(a::AuthoringErrorCode::UnsupportedValueType));
    };
    pivot_types::PivotDecimal::from_parts(d.lo, d.mid, d.hi, d.negative, d.scale)
        .map_err(|_| fail(a::AuthoringErrorCode::InvalidArgument))
}
pub(super) fn number(v: pivot_types::PivotDecimal) -> p::Value {
    let (lo, mid, hi, negative, scale) = v.parts();
    p::Value {
        kind: Some(p::value::Kind::Decimal(p::Decimal128 {
            lo,
            mid,
            hi,
            negative,
            scale,
        })),
    }
}
fn definition(v: &a::NodeDefinition) -> Result<NodeDefinition, Failure> {
    match v.kind.as_ref() {
        Some(a::node_definition::Kind::Input(v)) => Ok(NodeDefinition::Input(value(v)?)),
        Some(a::node_definition::Kind::FormulaSource(s)) => Ok(NodeDefinition::Formula(s.clone())),
        _ => Err(fail(a::AuthoringErrorCode::InvalidArgument)),
    }
}
pub(super) fn node_definition(v: &NodeDefinition) -> a::NodeDefinition {
    a::NodeDefinition {
        kind: Some(match v {
            NodeDefinition::Input(v) => a::node_definition::Kind::Input(number(*v)),
            NodeDefinition::Formula(s) => a::node_definition::Kind::FormulaSource(s.clone()),
        }),
    }
}
fn requirement(v: &a::Requirement) -> Result<Requirement, Failure> {
    match v.kind.as_ref() {
        Some(a::requirement::Kind::Required(_)) => Ok(Requirement::Required),
        Some(a::requirement::Kind::OptionalDefault(v)) => Ok(Requirement::Optional(value(v)?)),
        _ => Err(fail(a::AuthoringErrorCode::InvalidArgument)),
    }
}
pub(super) fn port_requirement(v: &Requirement) -> a::Requirement {
    a::Requirement {
        kind: Some(match v {
            Requirement::Required => a::requirement::Kind::Required(a::Required {}),
            Requirement::Optional(v) => a::requirement::Kind::OptionalDefault(number(*v)),
        }),
    }
}
fn node(v: &a::NodeReference) -> Result<NodeRef, Failure> {
    match v.kind.as_ref() {
        Some(a::node_reference::Kind::NodeId(id)) => Ok(NodeRef::Existing(parse(id)?)),
        Some(a::node_reference::Kind::TemporaryId(id)) => Ok(NodeRef::Temporary(id.clone())),
        _ => Err(fail(a::AuthoringErrorCode::InvalidArgument)),
    }
}
fn port(v: &a::PortReference) -> Result<PortRef, Failure> {
    match v.kind.as_ref() {
        Some(a::port_reference::Kind::PortId(id)) => Ok(PortRef::Existing(parse(id)?)),
        Some(a::port_reference::Kind::TemporaryId(id)) => Ok(PortRef::Temporary(id.clone())),
        _ => Err(fail(a::AuthoringErrorCode::InvalidArgument)),
    }
}
fn needed<T>(v: Option<&T>) -> Result<&T, Failure> {
    v.ok_or_else(|| fail(a::AuthoringErrorCode::InvalidArgument))
}
pub(super) fn edits(v: &a::ApplyEdits) -> Result<ApplyEdits, Failure> {
    let m = needed(v.mutation.as_ref())?;
    let edits = v
        .edits
        .iter()
        .map(|e| {
            use a::edit::Kind::*;
            Ok(match needed(e.kind.as_ref())? {
                AddNode(v) => Edit::AddNode {
                    temporary: v.temporary_id.clone(),
                    name: v.name.clone(),
                    definition: definition(needed(v.definition.as_ref())?)?,
                },
                RemoveNode(v) => Edit::RemoveNode(node(needed(v.node.as_ref())?)?),
                RenameNode(v) => Edit::RenameNode {
                    node: node(needed(v.node.as_ref())?)?,
                    name: v.name.clone(),
                },
                SetNode(v) => Edit::SetNode {
                    node: node(needed(v.node.as_ref())?)?,
                    definition: definition(needed(v.definition.as_ref())?)?,
                },
                AddInputPort(v) => Edit::AddInputPort {
                    temporary: v.temporary_id.clone(),
                    name: v.name.clone(),
                    node: node(needed(v.node.as_ref())?)?,
                    requirement: requirement(needed(v.requirement.as_ref())?)?,
                },
                AddOutputPort(v) => Edit::AddOutputPort {
                    temporary: v.temporary_id.clone(),
                    name: v.name.clone(),
                    node: node(needed(v.node.as_ref())?)?,
                },
                RemovePort(v) => Edit::RemovePort(port(needed(v.port.as_ref())?)?),
                RenamePort(v) => Edit::RenamePort {
                    port: port(needed(v.port.as_ref())?)?,
                    name: v.name.clone(),
                },
                SetInputPort(v) => Edit::PatchInputPort {
                    port: port(needed(v.port.as_ref())?)?,
                    node: v.node.as_ref().map(node).transpose()?,
                    requirement: v.requirement.as_ref().map(requirement).transpose()?,
                },
                SetOutputPort(v) => Edit::SetOutputPort {
                    port: port(needed(v.port.as_ref())?)?,
                    node: node(needed(v.node.as_ref())?)?,
                },
            })
        })
        .collect::<Result<_, Failure>>()?;
    Ok(ApplyEdits {
        instance: parse(&m.service_instance_id)?,
        model: parse(&m.working_model_id)?,
        expected: revision(needed(m.expected_revision.as_ref())?)?,
        operation_id: m.operation_id.clone(),
        edits,
    })
}
pub(super) fn summary(r: WorkingRevision, diags: &[Diagnostic]) -> a::DiagnosticSummary {
    let mut counts = BTreeMap::<i32, u64>::new();
    for d in diags {
        *counts.entry(diagnostic(d).code).or_default() += 1;
    }
    a::DiagnosticSummary {
        revision: Some(rev(r)),
        total: diags.len() as u64,
        counts: counts
            .into_iter()
            .map(|(code, count)| a::DiagnosticCount { code, count })
            .collect(),
    }
}
pub(super) fn diagnostic(d: &Diagnostic) -> a::Diagnostic {
    use DiagnosticKind::*;
    let (code, message, offset, name, missing) = match &d.kind {
        Parse { offset, message } => (
            a::DiagnosticCode::Parse,
            message.clone(),
            Some(*offset as u32),
            String::new(),
            String::new(),
        ),
        UnknownName(s) => (
            a::DiagnosticCode::UnresolvedName,
            "unresolved name".into(),
            None,
            s.clone(),
            String::new(),
        ),
        MissingReference(id) => (
            a::DiagnosticCode::MissingReference,
            "missing bound reference".into(),
            None,
            String::new(),
            id.to_string(),
        ),
        Unorderable => (
            a::DiagnosticCode::Unorderable,
            "cycle or dependent on cycle".into(),
            None,
            String::new(),
            String::new(),
        ),
        InvalidPortBinding => (
            a::DiagnosticCode::InvalidPortBinding,
            "invalid port binding".into(),
            None,
            String::new(),
            String::new(),
        ),
    };
    a::Diagnostic {
        code: code as i32,
        message,
        location: Some(a::Location {
            node_id: d.node.map_or_else(String::new, |id| id.to_string()),
            port_id: d.port.map_or_else(String::new, |id| id.to_string()),
            expression_byte_offset: offset,
            edit_index: None,
        }),
        unresolved_name: name,
        missing_node_id: missing,
    }
}
pub(super) fn created(v: Created) -> a::Created {
    a::Created {
        working_model_id: v.id.to_string(),
        revision: Some(rev(v.revision)),
    }
}
pub(super) fn committed(v: EditResult) -> a::EditCommitted {
    a::EditCommitted {
        revision: Some(rev(v.revision)),
        nodes: v
            .nodes
            .into_iter()
            .map(|(temporary_id, id)| a::NodeMapping {
                temporary_id,
                node_id: id.to_string(),
            })
            .collect(),
        ports: v
            .ports
            .into_iter()
            .map(|(temporary_id, id)| a::PortMapping {
                temporary_id,
                port_id: id.to_string(),
            })
            .collect(),
        diagnostics: Some(summary(v.revision, &v.diagnostics)),
    }
}
pub(super) fn provenance(v: SnapshotProvenance) -> a::Provenance {
    a::Provenance {
        working: Some(rev(v.working)),
        snapshot_id: v.snapshot.to_string(),
    }
}
pub(super) fn published(v: PublicationResult) -> a::Published {
    a::Published {
        provenance: Some(provenance(v.provenance)),
        published: Some(v.published.wire()),
    }
}
pub(super) fn evaluated(v: EvaluationResult) -> a::EvaluationResult {
    a::EvaluationResult {
        provenance: Some(provenance(v.provenance)),
        effective_inputs: v
            .effective_inputs
            .into_iter()
            .map(|(id, v)| a::EffectiveInput {
                node_id: id.to_string(),
                value: Some(number(v.value)),
                source: match v.source {
                    EffectiveSource::Captured => a::WorkingInputSource::Captured,
                    EffectiveSource::Override => a::WorkingInputSource::Submitted,
                } as i32,
            })
            .collect(),
        outputs: v
            .outputs
            .into_iter()
            .map(|(id, v)| a::NodeResult {
                node_id: id.to_string(),
                outcome: Some(match v {
                    Ok(v) => a::node_result::Outcome::Value(number(v)),
                    Err(e) => {
                        let (code, dependency, origin) = match e {
                            CalculationError::Arithmetic(e) => (
                                match e {
                                    pivot_types::DecimalError::DivisionByZero => {
                                        p::CalculationErrorCode::DivisionByZero
                                    }
                                    _ => p::CalculationErrorCode::Overflow,
                                },
                                String::new(),
                                id.to_string(),
                            ),
                            CalculationError::DependencyFailed { dependency, origin } => (
                                p::CalculationErrorCode::DependencyFailed,
                                dependency.to_string(),
                                origin.to_string(),
                            ),
                            CalculationError::InvalidExpression => (
                                p::CalculationErrorCode::InvalidExpression,
                                String::new(),
                                id.to_string(),
                            ),
                        };
                        a::node_result::Outcome::Error(a::CalculationFailure {
                            code: code as i32,
                            dependency_node_id: dependency,
                            origin_node_id: origin,
                        })
                    }
                }),
            })
            .collect(),
    }
}
pub(super) fn fail(code: a::AuthoringErrorCode) -> Failure {
    Box::new(a::AuthoringFailure {
        code: code as i32,
        message: match code {
            a::AuthoringErrorCode::Unauthorized => "not authorized",
            a::AuthoringErrorCode::RevisionConflict => "working revision conflict",
            a::AuthoringErrorCode::ViewExpired => "read view expired",
            a::AuthoringErrorCode::ResourceExhausted => "resource limit reached",
            _ => "request rejected",
        }
        .into(),
        ..Default::default()
    })
}
pub(super) fn service_error(e: Error) -> Failure {
    use a::AuthoringErrorCode as C;
    let (code, limit) = match e {
        Error::Denied => (C::Unauthorized, ""),
        Error::StaleInstance => (C::StaleInstance, ""),
        Error::UnknownModel | Error::InvalidAttachment => (C::NotFound, ""),
        Error::RevisionConflict => (C::RevisionConflict, ""),
        Error::OperationIdConflict => (C::OperationIdConflict, ""),
        Error::InvalidEdit | Error::InvalidInput | Error::InvalidOutput => {
            (C::InvalidReference, "")
        }
        Error::ResourceLimit => (C::ResourceExhausted, "resources"),
        Error::AttachmentLimit => (C::ResourceExhausted, "attachments"),
        Error::ReceiptLimit => (C::ResourceExhausted, "receipts"),
        Error::CaptureLimit => (C::ResourceExhausted, "captures"),
        Error::WorkLimit => (C::ResourceExhausted, "work"),
        Error::PublishedLimit => (C::ResourceExhausted, "publications"),
        Error::ModelLimit => (C::ResourceExhausted, "model"),
        Error::BatchLimit => (C::ResourceExhausted, "batch"),
        Error::AllocationFailed | Error::IdentityExhausted | Error::CoreInitializationFailed => {
            (C::Internal, "")
        }
        Error::InvalidArgument => (C::InvalidArgument, ""),
    };
    let mut f = fail(code);
    f.limit_category = limit.into();
    f
}
pub(super) fn publication_error(e: PublicationError) -> Failure {
    match e {
        PublicationError::Service(e) => service_error(e),
        PublicationError::RegistryRejected => fail(a::AuthoringErrorCode::RegistryRejected),
        PublicationError::Validation(v) => {
            let mut f = fail(a::AuthoringErrorCode::ValidationFailed);
            f.validation_diagnostics = Some(summary(v.revision, &v.diagnostics));
            f
        }
    }
}
pub(super) fn outcome(v: OperationOutcome) -> a::OperationOutcome {
    use a::operation_outcome::Outcome;
    a::OperationOutcome {
        outcome: Some(match v {
            OperationOutcome::Unknown => Outcome::Unknown(a::UnknownOperation {}),
            OperationOutcome::Committed(v) => Outcome::Created(created(v)),
            OperationOutcome::Edited(v) => Outcome::Edited(committed(v)),
            OperationOutcome::Published(v) => Outcome::Published(published(v)),
            OperationOutcome::Rejected(e) => Outcome::Rejected(*service_error(e)),
            OperationOutcome::PublicationRejected(e) => Outcome::Rejected(*publication_error(e)),
        }),
    }
}
