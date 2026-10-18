//! Transactional authoring. All core handles stay behind this module's boundary.
use super::*;
use pivot_core::{
    interface::{InputRequirement, PortId},
    model::{DraftDefinition, NodeId, SnapshotError},
};
use pivot_types::PivotDecimal;
use std::collections::{BTreeMap, BTreeSet};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct EditLimits {
    pub operations: usize,
    pub batch_bytes: usize,
    pub nodes: usize,
    pub ports: usize,
    pub source_bytes: usize,
    /// Cumulative committed identities, including deleted-reference tombstones.
    pub node_identities: usize,
    pub port_identities: usize,
}
impl Default for EditLimits {
    fn default() -> Self {
        Self {
            operations: 128,
            batch_bytes: 65536,
            nodes: 1024,
            ports: 256,
            source_bytes: 262144,
            node_identities: 4096,
            port_identities: 1024,
        }
    }
}
impl EditLimits {
    pub(super) fn validate(self) -> Result<(), Error> {
        if [
            self.operations,
            self.batch_bytes,
            self.nodes,
            self.ports,
            self.source_bytes,
            self.node_identities,
            self.port_identities,
        ]
        .contains(&0)
        {
            Err(Error::InvalidArgument)
        } else {
            Ok(())
        }
    }
}
#[derive(Debug, Clone)]
pub enum NodeDefinition {
    Input(PivotDecimal),
    Formula(String),
}
impl PartialEq for NodeDefinition {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (Self::Input(a), Self::Input(b)) => a.parts() == b.parts(),
            (Self::Formula(a), Self::Formula(b)) => a == b,
            _ => false,
        }
    }
}
impl Eq for NodeDefinition {}
#[derive(Debug, Clone)]
pub enum Requirement {
    Required,
    Optional(PivotDecimal),
}
impl PartialEq for Requirement {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (Self::Required, Self::Required) => true,
            (Self::Optional(a), Self::Optional(b)) => a.parts() == b.parts(),
            _ => false,
        }
    }
}
impl Eq for Requirement {}
impl Requirement {
    fn core(&self) -> InputRequirement {
        match self {
            Self::Required => InputRequirement::Required,
            Self::Optional(v) => InputRequirement::Optional {
                default: (*v).into(),
            },
        }
    }
    fn from_core(value: &InputRequirement) -> Self {
        match value {
            InputRequirement::Required => Self::Required,
            InputRequirement::Optional {
                default: pivot_types::PivotValue::Decimal(v),
            } => Self::Optional(*v),
            _ => unreachable!("service accepts decimal values only"),
        }
    }
}
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum NodeRef {
    Existing(AuthoringNodeId),
    Temporary(String),
}
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PortRef {
    Existing(AuthoringPortId),
    Temporary(String),
}
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Edit {
    AddNode {
        temporary: String,
        name: String,
        definition: NodeDefinition,
    },
    RemoveNode(NodeRef),
    RenameNode {
        node: NodeRef,
        name: String,
    },
    SetNode {
        node: NodeRef,
        definition: NodeDefinition,
    },
    AddInputPort {
        temporary: String,
        name: String,
        node: NodeRef,
        requirement: Requirement,
    },
    AddOutputPort {
        temporary: String,
        name: String,
        node: NodeRef,
    },
    RemovePort(PortRef),
    RenamePort {
        port: PortRef,
        name: String,
    },
    SetInputPort {
        port: PortRef,
        node: NodeRef,
        requirement: Requirement,
    },
    SetOutputPort {
        port: PortRef,
        node: NodeRef,
    },
}
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ApplyEdits {
    pub instance: ServiceInstanceId,
    pub model: WorkingModelId,
    pub expected: WorkingRevision,
    pub operation_id: String,
    pub edits: Vec<Edit>,
}
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EditResult {
    pub revision: WorkingRevision,
    pub nodes: BTreeMap<String, AuthoringNodeId>,
    pub ports: BTreeMap<String, AuthoringPortId>,
    pub diagnostics: Vec<Diagnostic>,
}
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct NodeView {
    pub id: AuthoringNodeId,
    pub name: String,
    pub definition: NodeDefinition,
    /// Bound identities, including deleted targets. Never a runtime name lookup.
    pub dependencies: Vec<AuthoringNodeId>,
}
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PortDefinition {
    Input {
        node: AuthoringNodeId,
        requirement: Requirement,
    },
    Output {
        node: AuthoringNodeId,
    },
}
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PortView {
    pub id: AuthoringPortId,
    pub name: String,
    pub definition: PortDefinition,
}
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum DiagnosticKind {
    Parse { offset: usize, message: String },
    UnknownName(String),
    MissingReference(AuthoringNodeId),
    Unorderable,
    InvalidPortBinding,
}
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Diagnostic {
    pub node: Option<AuthoringNodeId>,
    pub port: Option<AuthoringPortId>,
    pub kind: DiagnosticKind,
}
/// Owned raw draft view; does not require an executable snapshot or retain history.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DraftView {
    pub revision: WorkingRevision,
    pub nodes: Vec<NodeView>,
    pub ports: Vec<PortView>,
    pub diagnostics: Vec<Diagnostic>,
}
#[derive(Clone, Default)]
pub(super) struct Identities {
    nodes: BTreeMap<AuthoringNodeId, NodeId>,
    ports: BTreeMap<AuthoringPortId, PortId>,
}
pub(super) struct EditReceipt {
    pub principal: String,
    pub request: ApplyEdits,
    pub result: Result<EditResult, Error>,
}
impl WorkingModelService {
    pub fn read_draft(
        &self,
        principal: &str,
        expected: WorkingRevision,
    ) -> Result<DraftView, Error> {
        self.check(
            expected.instance,
            principal,
            Action::Inspect,
            Some(expected.model),
        )?;
        let resource = self
            .resources
            .iter()
            .find(|r| r.id == expected.model)
            .ok_or(Error::UnknownModel)?;
        if resource.revision != expected {
            return Err(Error::RevisionConflict);
        }
        Ok(draft(resource))
    }
    pub fn apply_edits(
        &mut self,
        principal: &str,
        request: ApplyEdits,
    ) -> Result<EditResult, Error> {
        self.check(
            request.instance,
            principal,
            Action::Edit,
            Some(request.model),
        )?;
        validate_request(&request, self.info.edit_limits)?;
        // Complete typed intent includes exact decimal layout, source bytes,
        // ordered commands, temporary references and expected revision.
        if self
            .receipts
            .iter()
            .any(|r| r.principal == principal && r.operation == request.operation_id)
        {
            return Err(Error::OperationIdConflict);
        }
        if let Some(r) = self
            .edit_receipts
            .iter()
            .find(|r| r.principal == principal && r.request.operation_id == request.operation_id)
        {
            return if r.request == request {
                r.result.clone()
            } else {
                Err(Error::OperationIdConflict)
            };
        }
        if self.receipts.len() + self.edit_receipts.len() >= self.info.limits.receipts {
            return Err(Error::ReceiptLimit);
        }
        self.edit_receipts
            .try_reserve(1)
            .map_err(|_| Error::AllocationFailed)?;
        let principal_owned = principal.to_owned();
        let candidate = (|| {
            let index = self
                .resources
                .iter()
                .position(|r| r.id == request.model)
                .ok_or(Error::UnknownModel)?;
            let original = &self.resources[index];
            if original.revision != request.expected {
                return Err(Error::RevisionConflict);
            }
            let sequence = original
                .revision
                .sequence
                .checked_add(1)
                .ok_or(Error::IdentityExhausted)?;
            let mut staged = Resource {
                id: original.id,
                graph: original.graph.stage(),
                revision: WorkingRevision {
                    sequence,
                    ..original.revision
                },
                identities: original.identities.clone(),
            };
            let result = stage(&mut staged, &request.edits, self.info.edit_limits)?;
            self.check(
                request.instance,
                principal,
                Action::Edit,
                Some(request.model),
            )?;
            // Explicit final comparison. Exclusive service access currently
            // prevents intervening writers; keep the commit invariant here.
            if self.resources[index].revision != request.expected {
                return Err(Error::RevisionConflict);
            }
            Ok((index, staged, result))
        })();
        // Allocate caller/receipt result copies before installing anything.
        let result = candidate
            .as_ref()
            .map(|(_, _, result)| result.clone())
            .map_err(|e| *e);
        let receipt = EditReceipt {
            principal: principal_owned,
            request,
            result: result.clone(),
        };
        if let Ok((index, resource, _)) = candidate {
            self.resources[index] = resource;
        }
        self.edit_receipts.push(receipt);
        result
    }
}

fn validate_request(request: &ApplyEdits, limits: EditLimits) -> Result<(), Error> {
    if !valid_operation(&request.operation_id) || request.edits.is_empty() {
        return Err(Error::InvalidArgument);
    }
    if request.expected.instance != request.instance {
        return Err(Error::StaleInstance);
    }
    if request.edits.len() > limits.operations {
        return Err(Error::BatchLimit);
    }
    let mut bytes = request
        .edits
        .len()
        .checked_mul(512)
        .ok_or(Error::BatchLimit)?;
    for edit in &request.edits {
        let (name, temporary, definition) = match edit {
            Edit::AddNode {
                temporary,
                name,
                definition,
            } => (Some(name), Some(temporary), Some(definition)),
            Edit::RenameNode { name, .. } | Edit::RenamePort { name, .. } => {
                (Some(name), None, None)
            }
            Edit::SetNode { definition, .. } => (None, None, Some(definition)),
            Edit::AddInputPort {
                temporary, name, ..
            }
            | Edit::AddOutputPort {
                temporary, name, ..
            } => (Some(name), Some(temporary), None),
            _ => (None, None, None),
        };
        if name.is_some_and(|s| s.len() > 128) || temporary.is_some_and(|s| !valid_temporary(s)) {
            return Err(Error::InvalidArgument);
        }
        if let Some(NodeDefinition::Formula(source)) = definition {
            if source.len() > 4096 {
                return Err(Error::BatchLimit);
            }
            bytes = bytes.checked_add(source.len()).ok_or(Error::BatchLimit)?;
        }
        let (node, port) = match edit {
            Edit::RemoveNode(n)
            | Edit::RenameNode { node: n, .. }
            | Edit::SetNode { node: n, .. }
            | Edit::AddInputPort { node: n, .. }
            | Edit::AddOutputPort { node: n, .. } => (Some(n), None),
            Edit::SetInputPort { node, port, .. } | Edit::SetOutputPort { node, port } => {
                (Some(node), Some(port))
            }
            Edit::RemovePort(p) | Edit::RenamePort { port: p, .. } => (None, Some(p)),
            _ => (None, None),
        };
        if node.is_some_and(|n| matches!(n, NodeRef::Temporary(s) if !valid_temporary(s)))
            || port.is_some_and(|p| matches!(p, PortRef::Temporary(s) if !valid_temporary(s)))
        {
            return Err(Error::InvalidArgument);
        }
    }
    if bytes > limits.batch_bytes {
        return Err(Error::BatchLimit);
    }
    Ok(())
}
fn valid_temporary(s: &str) -> bool {
    s.len() <= 64 && valid_operation(s)
}
fn node(
    r: &Resource,
    reference: &NodeRef,
    temps: &BTreeMap<String, AuthoringNodeId>,
) -> Result<NodeId, Error> {
    let id = match reference {
        NodeRef::Existing(id) => id,
        NodeRef::Temporary(t) => temps.get(t).ok_or(Error::InvalidEdit)?,
    };
    r.identities
        .nodes
        .get(id)
        .copied()
        .filter(|&id| r.graph.contains_node(id))
        .ok_or(Error::InvalidEdit)
}
fn port(
    r: &Resource,
    reference: &PortRef,
    temps: &BTreeMap<String, AuthoringPortId>,
) -> Result<PortId, Error> {
    let id = match reference {
        PortRef::Existing(id) => id,
        PortRef::Temporary(t) => temps.get(t).ok_or(Error::InvalidEdit)?,
    };
    let core = r
        .identities
        .ports
        .get(id)
        .copied()
        .ok_or(Error::InvalidEdit)?;
    if r.graph.interface().inputs().any(|p| p.id() == core)
        || r.graph.interface().outputs().any(|p| p.id() == core)
    {
        Ok(core)
    } else {
        Err(Error::InvalidEdit)
    }
}
fn stage(r: &mut Resource, edits: &[Edit], limits: EditLimits) -> Result<EditResult, Error> {
    let mut nodes = BTreeMap::new();
    let mut ports = BTreeMap::new();
    let mut submitted = BTreeMap::new();
    // One temporary namespace across node and port allocations, never reusable
    // even if a later edit removes the object within the same batch.
    let mut used = BTreeSet::new();
    for edit in edits {
        match edit {
            Edit::AddNode {
                temporary,
                name,
                definition,
            } => {
                if !used.insert(temporary.clone()) {
                    return Err(Error::InvalidEdit);
                }
                if r.identities.nodes.len() >= limits.node_identities {
                    return Err(Error::ModelLimit);
                }
                let id = match definition {
                    NodeDefinition::Input(v) => r.graph.add_input(name, *v),
                    NodeDefinition::Formula(source) => r.graph.add_computed(name, source),
                }
                .map_err(|_| Error::InvalidEdit)?;
                let portable = AuthoringNodeId::allocate();
                r.identities.nodes.insert(portable, id);
                nodes.insert(temporary.clone(), portable);
                if let NodeDefinition::Formula(source) = definition {
                    submitted.insert(id, source.clone());
                }
            }
            Edit::RemoveNode(reference) => {
                let id = node(r, reference, &nodes)?;
                r.graph.remove(id).map_err(|_| Error::InvalidEdit)?;
                submitted.remove(&id);
            }
            Edit::RenameNode {
                node: reference,
                name,
            } => {
                let id = node(r, reference, &nodes)?;
                r.graph.rename(id, name).map_err(|_| Error::InvalidEdit)?;
            }
            Edit::SetNode {
                node: reference,
                definition,
            } => {
                let id = node(r, reference, &nodes)?;
                match definition {
                    NodeDefinition::Input(v) => {
                        r.graph
                            .replace_input(id, *v)
                            .map_err(|_| Error::InvalidEdit)?;
                        submitted.remove(&id);
                    }
                    NodeDefinition::Formula(source) => {
                        r.graph
                            .set_expression(id, source)
                            .map_err(|_| Error::InvalidEdit)?;
                        submitted.insert(id, source.clone());
                    }
                }
            }
            Edit::AddInputPort {
                temporary,
                name,
                node: reference,
                requirement,
            } => {
                if !used.insert(temporary.clone()) {
                    return Err(Error::InvalidEdit);
                }
                if r.identities.ports.len() >= limits.port_identities {
                    return Err(Error::ModelLimit);
                }
                let id = node(r, reference, &nodes)?;
                let core = r
                    .graph
                    .add_input_port(name, id, requirement.core())
                    .map_err(|_| Error::InvalidEdit)?;
                let portable = AuthoringPortId::allocate();
                r.identities.ports.insert(portable, core);
                ports.insert(temporary.clone(), portable);
            }
            Edit::AddOutputPort {
                temporary,
                name,
                node: reference,
            } => {
                if !used.insert(temporary.clone()) {
                    return Err(Error::InvalidEdit);
                }
                if r.identities.ports.len() >= limits.port_identities {
                    return Err(Error::ModelLimit);
                }
                let id = node(r, reference, &nodes)?;
                let core = r
                    .graph
                    .add_output_port(name, id)
                    .map_err(|_| Error::InvalidEdit)?;
                let portable = AuthoringPortId::allocate();
                r.identities.ports.insert(portable, core);
                ports.insert(temporary.clone(), portable);
            }
            Edit::RemovePort(reference) => {
                let id = port(r, reference, &ports)?;
                r.graph.remove_port(id).map_err(|_| Error::InvalidEdit)?;
            }
            Edit::RenamePort {
                port: reference,
                name,
            } => {
                let id = port(r, reference, &ports)?;
                r.graph
                    .rename_port(id, name)
                    .map_err(|_| Error::InvalidEdit)?;
            }
            Edit::SetInputPort {
                port: reference,
                node: n,
                requirement,
            } => {
                let id = port(r, reference, &ports)?;
                let target = node(r, n, &nodes)?;
                r.graph
                    .update_input_port(id, target, requirement.core())
                    .map_err(|_| Error::InvalidEdit)?;
            }
            Edit::SetOutputPort {
                port: reference,
                node: n,
            } => {
                let id = port(r, reference, &ports)?;
                let target = node(r, n, &nodes)?;
                r.graph
                    .update_output_port(id, target)
                    .map_err(|_| Error::InvalidEdit)?;
            }
        }
        if r.graph.draft_nodes().count() > limits.nodes
            || r.graph.interface().inputs().count() + r.graph.interface().outputs().count()
                > limits.ports
        {
            return Err(Error::ModelLimit);
        }
    }
    // Only explicitly submitted definitions bind to the FINAL directory. Existing
    // bound formulas and unresolved drafts are copied byte-for-byte by staging.
    for (id, source) in submitted {
        r.graph
            .set_expression(id, source)
            .map_err(|_| Error::InvalidEdit)?;
    }
    let source_bytes: usize = r
        .graph
        .draft_nodes()
        .map(|(_, _, d)| match d {
            DraftDefinition::Computed { source, .. } => source.len(),
            _ => 0,
        })
        .sum();
    if source_bytes > limits.source_bytes {
        return Err(Error::ModelLimit);
    }
    let diagnostics = draft(r).diagnostics;
    nodes.retain(|_, id| r.graph.contains_node(r.identities.nodes[id]));
    ports.retain(|_, id| {
        let core = r.identities.ports[id];
        r.graph.interface().inputs().any(|p| p.id() == core)
            || r.graph.interface().outputs().any(|p| p.id() == core)
    });
    Ok(EditResult {
        revision: r.revision,
        nodes,
        ports,
        diagnostics,
    })
}

fn draft(r: &Resource) -> DraftView {
    let node_ids: BTreeMap<_, _> = r.identities.nodes.iter().map(|(&a, &b)| (b, a)).collect();
    let port_ids: BTreeMap<_, _> = r.identities.ports.iter().map(|(&a, &b)| (b, a)).collect();
    let mut nodes = Vec::new();
    let mut ports = Vec::new();
    let mut diagnostics = Vec::new();
    for (id, name, definition) in r.graph.draft_nodes() {
        let portable = node_ids[&id];
        let mut dependencies = Vec::new();
        let definition = match definition {
            DraftDefinition::Input(v) => NodeDefinition::Input(v),
            DraftDefinition::Computed { source, binding } => {
                match binding {
                    Ok(deps) => {
                        for target in deps {
                            dependencies.push(node_ids[target]);
                            if !r.graph.contains_node(*target) {
                                diagnostics.push(Diagnostic {
                                    node: Some(portable),
                                    port: None,
                                    kind: DiagnosticKind::MissingReference(node_ids[target]),
                                });
                            }
                        }
                    }
                    Err(SnapshotError::Parse { error, .. }) => diagnostics.push(Diagnostic {
                        node: Some(portable),
                        port: None,
                        kind: DiagnosticKind::Parse {
                            offset: error.offset,
                            message: error.message.into(),
                        },
                    }),
                    Err(SnapshotError::UnknownName { name, .. }) => diagnostics.push(Diagnostic {
                        node: Some(portable),
                        port: None,
                        kind: DiagnosticKind::UnknownName(name.clone()),
                    }),
                    Err(_) => unreachable!("binding only reports parse or unresolved name errors"),
                }
                NodeDefinition::Formula(source.into())
            }
        };
        nodes.push(NodeView {
            id: portable,
            name: name.into(),
            definition,
            dependencies,
        });
    }
    for p in r.graph.interface().inputs() {
        let id = port_ids[&p.id()];
        if !nodes.iter().any(|n| {
            n.id == node_ids[&p.node()] && matches!(n.definition, NodeDefinition::Input(_))
        }) {
            diagnostics.push(Diagnostic {
                node: None,
                port: Some(id),
                kind: DiagnosticKind::InvalidPortBinding,
            });
        }
        ports.push(PortView {
            id,
            name: p.name().into(),
            definition: PortDefinition::Input {
                node: node_ids[&p.node()],
                requirement: Requirement::from_core(p.requirement()),
            },
        });
    }
    for p in r.graph.interface().outputs() {
        let id = port_ids[&p.id()];
        if !r.graph.contains_node(p.node()) {
            diagnostics.push(Diagnostic {
                node: None,
                port: Some(id),
                kind: DiagnosticKind::InvalidPortBinding,
            });
        }
        ports.push(PortView {
            id,
            name: p.name().into(),
            definition: PortDefinition::Output {
                node: node_ids[&p.node()],
            },
        });
    }
    // Core capture detects cycles/blocked nodes; it is never the stored draft.
    if let Err(SnapshotError::Unorderable { nodes }) = r.graph.snapshot() {
        for id in nodes {
            diagnostics.push(Diagnostic {
                node: Some(node_ids[&id]),
                port: None,
                kind: DiagnosticKind::Unorderable,
            });
        }
    }
    DraftView {
        revision: r.revision,
        nodes,
        ports,
        diagnostics,
    }
}
