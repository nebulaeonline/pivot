use crate::*;
use pivot_core::{
    evaluation::{EvaluationContext, EvaluationError},
    expression::{BinaryOp, UnaryOp},
    interface::{InputRequirement, NamedInvocationRequest, PortId, PublicEvaluationError},
    model::{ModelSnapshot, NodeId, SnapshotError, WorkingModel, transfer as t},
};
use std::collections::BTreeMap;

/// Hosting-independent computational graph, not a native file format.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct GraphPayload {
    pub version: u32,
    pub numeric: u32,
    pub expression: u32,
    pub model: Uuid,
    pub revision: Uuid,
    pub name: String,
    pub node_ids: Vec<Uuid>,
    pub port_ids: Vec<Uuid>,
    pub nodes: Vec<Node>,
    pub ports: Vec<Port>,
}
#[derive(Clone, Copy)]
pub struct GraphHeader {
    pub model: Uuid,
    pub revision: Uuid,
}
pub type NamedValues = Vec<(
    String,
    std::result::Result<Decimal96, PublicEvaluationError>,
)>;
impl GraphPayload {
    pub fn validate(&self, limits: Limits) -> Result<()> {
        if self.version != 1 || self.numeric != 1 || self.expression != 1 {
            return Err(Error::UnsupportedVersion);
        }
        self.validate_semantics(limits)?;
        // Logical resource accounting, not a document byte encoding. Check before
        // cloning/importing; also retain independent core transfer hard ceilings.
        let mut bytes = 64usize + self.name.len();
        let mut items = 8usize;
        let mut charge = |b: usize, i: usize| -> Result<()> {
            bytes = bytes.checked_add(b).ok_or(Error::Limit)?;
            items = items.checked_add(i).ok_or(Error::Limit)?;
            if bytes > limits.bytes || items > limits.items {
                return Err(Error::Limit);
            }
            Ok(())
        };
        charge(
            16 * (self.node_ids.len() + self.port_ids.len()),
            self.node_ids.len() + self.port_ids.len(),
        )?;
        for n in &self.nodes {
            charge(32 + n.name.len(), 4)?;
            match &n.definition {
                Definition::Input(_) => charge(16, 2)?,
                Definition::Bound { source, tokens } => {
                    charge(source.len(), 3)?;
                    for t in tokens {
                        charge(
                            match t {
                                Token::Node { .. } => 40,
                                Token::Decimal(_) => 24,
                                _ => 8,
                            },
                            4,
                        )?;
                    }
                }
                Definition::ParseFailed { source, .. } => charge(source.len() + 16, 4)?,
                Definition::BindFailed { source, name } => charge(source.len() + name.len(), 3)?,
            }
        }
        for p in &self.ports {
            charge(80 + p.name.len(), 8)?;
        }
        Ok(())
    }
    fn validate_semantics(&self, limits: Limits) -> Result<()> {
        GraphView {
            model: self.model,
            revision: self.revision,
            name: &self.name,
            node_ids: &self.node_ids,
            port_ids: &self.port_ids,
            nodes: &self.nodes,
            ports: &self.ports,
        }
        .validate(limits)?;
        Ok(())
    }
}

pub struct EditableGraph {
    record: GraphPayload,
    core: WorkingModel,
    nodes: BTreeMap<Uuid, NodeId>,
    ports: BTreeMap<Uuid, PortId>,
    limits: Limits,
    logical_budget: bool,
}
/// Internal export bridge. Caller owns opaque core handles and complete portable
/// identity maps. No handles appear in returned records or encoded data.
pub(crate) fn capture_graph(
    core: &WorkingModel,
    nodes: &BTreeMap<Uuid, NodeId>,
    ports: &BTreeMap<Uuid, PortId>,
    header: GraphHeader,
    limits: Limits,
) -> Result<GraphPayload> {
    if nodes.len().saturating_add(ports.len()) > limits.identities.min(8192) {
        return Err(Error::Limit);
    }
    let image = t::export(
        core,
        &nodes.values().copied().collect::<Vec<_>>(),
        &ports.values().copied().collect::<Vec<_>>(),
    )
    .map_err(|_| Error::Runtime)?;
    from_image(
        image,
        nodes.keys().copied().collect(),
        ports.keys().copied().collect(),
        header,
        limits,
    )
}
pub(crate) fn capture_graph_snapshot(
    core: &ModelSnapshot,
    nodes: &BTreeMap<Uuid, NodeId>,
    ports: &BTreeMap<Uuid, PortId>,
    header: GraphHeader,
    limits: Limits,
) -> Result<GraphPayload> {
    if nodes.len().saturating_add(ports.len()) > limits.identities.min(8192)
        || core.dependency_order().len() > limits.nodes
        || core.interface().inputs().count() + core.interface().outputs().count() > limits.ports
    {
        return Err(Error::Limit);
    }
    let image = t::export_snapshot(
        core,
        &nodes.values().copied().collect::<Vec<_>>(),
        &ports.values().copied().collect::<Vec<_>>(),
    )
    .map_err(|_| Error::Runtime)?;
    from_image(
        image,
        nodes.keys().copied().collect(),
        ports.keys().copied().collect(),
        header,
        limits,
    )
}
pub fn from_image(
    image: t::Image,
    node_ids: Vec<Uuid>,
    port_ids: Vec<Uuid>,
    h: GraphHeader,
    limits: Limits,
) -> Result<GraphPayload> {
    let mut nodes = vec![];
    for n in image.nodes {
        let definition = match n.definition {
            t::NodeDefinition::Input(v) => Definition::Input(Decimal96::from_runtime(v)),
            t::NodeDefinition::Computed { source, binding } => match binding {
                t::Binding::Bound(tokens) => Definition::Bound {
                    source,
                    tokens: tokens
                        .into_iter()
                        .map(|t| {
                            Ok(match t {
                                t::Token::Decimal(v) => Token::Decimal(Decimal96::from_runtime(v)),
                                t::Token::Node(i) => Token::Node {
                                    model: h.model,
                                    node: *node_ids.get(i).ok_or(Error::Identity)?,
                                },
                                t::Token::Unary(op) => Token::Unary(match op {
                                    UnaryOp::Plus => 0,
                                    UnaryOp::Minus => 1,
                                }),
                                t::Token::Binary(op) => Token::Binary(match op {
                                    BinaryOp::Add => 0,
                                    BinaryOp::Subtract => 1,
                                    BinaryOp::Multiply => 2,
                                    BinaryOp::Divide => 3,
                                }),
                            })
                        })
                        .collect::<Result<_>>()?,
                },
                t::Binding::ParseFailed { offset, code } => Definition::ParseFailed {
                    source,
                    offset: offset.try_into().map_err(|_| Error::Limit)?,
                    code,
                },
                t::Binding::Unresolved(name) => Definition::BindFailed { source, name },
            },
        };
        nodes.push(Node {
            id: *node_ids.get(n.slot).ok_or(Error::Identity)?,
            name: n.name,
            definition,
        });
    }
    let mut ports = vec![];
    for p in image.ports {
        ports.push(Port {
            id: *port_ids.get(p.slot).ok_or(Error::Identity)?,
            name: p.name,
            model: h.model,
            node: *node_ids.get(p.node).ok_or(Error::Identity)?,
            kind: match p.requirement {
                None => PortKind::Output,
                Some(InputRequirement::Required) => PortKind::Required,
                Some(InputRequirement::Optional { default }) => {
                    let pivot_types::PivotValue::Decimal(v) = default else {
                        return Err(Error::UnsupportedVersion);
                    };
                    PortKind::Optional(Decimal96::from_runtime(v))
                }
            },
        });
    }
    nodes.sort_by_key(|n| n.id);
    ports.sort_by_key(|p| p.id);
    let r = GraphPayload {
        version: 1,
        numeric: 1,
        expression: 1,
        model: h.model,
        revision: h.revision,
        name: image.name,
        node_ids,
        port_ids,
        nodes,
        ports,
    };
    r.validate_semantics(limits)?;
    Ok(r)
}
pub(crate) fn restore_graph(record: GraphPayload, limits: Limits) -> Result<EditableGraph> {
    record.validate(limits)?;
    import_graph(record, limits, true)
}
// Hosted callers already enforce the exact frozen CBOR byte/item/depth budgets.
// Do not apply a different logical accounting profile to their accepted records.
pub(crate) fn restore_hosted_graph(record: GraphPayload, limits: Limits) -> Result<EditableGraph> {
    record.validate_semantics(limits)?;
    import_graph(record, limits, false)
}
fn import_graph(
    record: GraphPayload,
    limits: Limits,
    logical_budget: bool,
) -> Result<EditableGraph> {
    let imported = t::import(image_body(&record)?).map_err(|_| Error::Malformed)?;
    let nodes = record
        .node_ids
        .iter()
        .copied()
        .zip(imported.nodes)
        .collect();
    let ports = record
        .port_ids
        .iter()
        .copied()
        .zip(imported.ports)
        .collect();
    Ok(EditableGraph {
        record,
        logical_budget,
        core: imported.model,
        nodes,
        ports,
        limits,
    })
}
/// Validated portable transfer; importing remains the canonical core importer.
pub fn to_image(record: &GraphPayload, limits: Limits) -> Result<t::Image> {
    record.validate(limits)?;
    image_body(record)
}
fn image_body(record: &GraphPayload) -> Result<t::Image> {
    let slots: BTreeMap<_, _> = record
        .node_ids
        .iter()
        .enumerate()
        .map(|(i, id)| (*id, i))
        .collect();
    let port_slots: BTreeMap<_, _> = record
        .port_ids
        .iter()
        .enumerate()
        .map(|(i, id)| (*id, i))
        .collect();
    let nodes = record
        .nodes
        .iter()
        .map(|n| {
            Ok(t::Node {
                slot: slots[&n.id],
                name: n.name.clone(),
                definition: match &n.definition {
                    Definition::Input(v) => t::NodeDefinition::Input(v.runtime()?),
                    Definition::Bound { source, tokens } => t::NodeDefinition::Computed {
                        source: source.clone(),
                        binding: t::Binding::Bound(
                            tokens
                                .iter()
                                .map(|t| {
                                    Ok(match t {
                                        Token::Decimal(v) => t::Token::Decimal(v.runtime()?),
                                        Token::Node { node, .. } => t::Token::Node(slots[node]),
                                        Token::Unary(op) => t::Token::Unary(if *op == 0 {
                                            UnaryOp::Plus
                                        } else {
                                            UnaryOp::Minus
                                        }),
                                        Token::Binary(op) => t::Token::Binary(match op {
                                            0 => BinaryOp::Add,
                                            1 => BinaryOp::Subtract,
                                            2 => BinaryOp::Multiply,
                                            _ => BinaryOp::Divide,
                                        }),
                                    })
                                })
                                .collect::<Result<_>>()?,
                        ),
                    },
                    Definition::ParseFailed {
                        source,
                        offset,
                        code,
                    } => t::NodeDefinition::Computed {
                        source: source.clone(),
                        binding: t::Binding::ParseFailed {
                            offset: *offset as usize,
                            code: *code,
                        },
                    },
                    Definition::BindFailed { source, name } => t::NodeDefinition::Computed {
                        source: source.clone(),
                        binding: t::Binding::Unresolved(name.clone()),
                    },
                },
            })
        })
        .collect::<Result<_>>()?;
    let ports = record
        .ports
        .iter()
        .map(|p| {
            Ok(t::Port {
                slot: port_slots[&p.id],
                name: p.name.clone(),
                node: slots[&p.node],
                requirement: match p.kind {
                    PortKind::Output => None,
                    PortKind::Required => Some(InputRequirement::Required),
                    PortKind::Optional(v) => Some(InputRequirement::Optional {
                        default: v.runtime()?.into(),
                    }),
                },
            })
        })
        .collect::<Result<_>>()?;
    Ok(t::Image {
        name: record.name.clone(),
        node_slots: record.node_ids.len(),
        port_slots: record.port_ids.len(),
        nodes,
        ports,
    })
}

fn public_error(e: &EvaluationError) -> PublicEvaluationError {
    match e {
        EvaluationError::Arithmetic { error, .. } => PublicEvaluationError::Arithmetic(*error),
        EvaluationError::DependencyFailed { .. } => PublicEvaluationError::DependencyFailed,
        _ => PublicEvaluationError::InvalidExpression,
    }
}
pub type Values = Vec<(Uuid, std::result::Result<Decimal96, PublicEvaluationError>)>;
impl EditableGraph {
    pub fn new(name: &str, limits: Limits) -> Result<Self> {
        if name.len() > 128 {
            return Err(Error::Limit);
        }
        let core = WorkingModel::new(name).map_err(|_| Error::Malformed)?;
        let record = capture_graph(
            &core,
            &BTreeMap::new(),
            &BTreeMap::new(),
            GraphHeader {
                model: Uuid::new_v4(),
                revision: Uuid::new_v4(),
            },
            limits,
        )?;
        record.validate(limits)?;
        Ok(Self {
            logical_budget: true,
            record,
            core,
            nodes: BTreeMap::new(),
            ports: BTreeMap::new(),
            limits,
        })
    }
    pub fn load(payload: GraphPayload, limits: Limits) -> Result<Self> {
        restore_graph(payload, limits)
    }
    pub(crate) fn stage(&self) -> Self {
        Self {
            logical_budget: self.logical_budget,
            record: self.record.clone(),
            core: self.core.stage(),
            nodes: self.nodes.clone(),
            ports: self.ports.clone(),
            limits: self.limits,
        }
    }
    pub fn validate_snapshot(&self) -> Result<()> {
        self.snapshot().map(|_| ())
    }
    pub fn add_formula(&mut self, id: Uuid, name: &str, source: &str) -> Result<()> {
        self.available(id)?;
        if source.len() > 4096 || name.len() > 128 {
            return Err(Error::Limit);
        }
        let mut core = self.core.stage();
        let handle = core
            .add_computed(name, source)
            .map_err(|_| Error::Runtime)?;
        let mut nodes = self.nodes.clone();
        nodes.insert(id, handle);
        self.install(core, nodes, self.ports.clone(), Some(id), None)
    }
    pub fn add_port(&mut self, id: Uuid, name: &str, node: Uuid, kind: PortKind) -> Result<()> {
        self.available(id)?;
        if name.len() > 128 {
            return Err(Error::Limit);
        }
        let target = *self.nodes.get(&node).ok_or(Error::Identity)?;
        let mut core = self.core.stage();
        let handle = match kind {
            PortKind::Output => core.add_output_port(name, target),
            PortKind::Required => core.add_input_port(name, target, InputRequirement::Required),
            PortKind::Optional(v) => core.add_input_port(
                name,
                target,
                InputRequirement::Optional {
                    default: v.runtime()?.into(),
                },
            ),
        }
        .map_err(|_| Error::Runtime)?;
        let mut ports = self.ports.clone();
        ports.insert(id, handle);
        self.install(core, self.nodes.clone(), ports, None, Some((id, kind)))
    }
    pub fn remove_port(&mut self, id: Uuid) -> Result<()> {
        let port = *self.ports.get(&id).ok_or(Error::Identity)?;
        self.change(None, |core, _| {
            core.remove_port(port).map_err(|_| Error::Runtime)
        })
    }
    fn available(&self, id: Uuid) -> Result<()> {
        if id.is_nil() || self.nodes.contains_key(&id) || self.ports.contains_key(&id) {
            return Err(Error::Identity);
        }
        Ok(())
    }
    fn install(
        &mut self,
        core: WorkingModel,
        nodes: BTreeMap<Uuid, NodeId>,
        ports: BTreeMap<Uuid, PortId>,
        changed: Option<Uuid>,
        exact_port: Option<(Uuid, PortKind)>,
    ) -> Result<()> {
        let mut record = capture_graph(
            &core,
            &nodes,
            &ports,
            GraphHeader {
                model: self.record.model,
                revision: Uuid::new_v4(),
            },
            self.limits,
        )?;
        for n in &mut record.nodes {
            if Some(n.id) != changed
                && let Some(old) = self.record.nodes.iter().find(|v| v.id == n.id)
            {
                n.definition = old.definition.clone();
            }
        }
        for p in &mut record.ports {
            if let Some((id, kind)) = &exact_port
                && p.id == *id
            {
                p.kind = kind.clone();
            } else if let Some(old) = self.record.ports.iter().find(|v| v.id == p.id) {
                p.kind = old.kind.clone();
            }
        }
        if self.logical_budget {
            record.validate(self.limits)?;
        } else {
            record.validate_semantics(self.limits)?;
        }
        self.record = record;
        self.core = core;
        self.nodes = nodes;
        self.ports = ports;
        Ok(())
    }
    pub fn record(&self) -> &GraphPayload {
        &self.record
    }
    fn snapshot(&self) -> Result<ModelSnapshot> {
        self.core.snapshot().map_err(|_| Error::InvalidDraft)
    }
    pub fn evaluate(&self, outputs: &[Uuid]) -> Result<Values> {
        if outputs.len() > self.limits.nodes {
            return Err(Error::Limit);
        }
        let ids = outputs
            .iter()
            .map(|id| {
                self.nodes
                    .get(id)
                    .copied()
                    .filter(|id| self.core.contains_node(*id))
                    .ok_or(Error::Identity)
            })
            .collect::<Result<Vec<_>>>()?;
        let result = EvaluationContext::new(self.snapshot()?)
            .evaluate(&ids)
            .map_err(|_| Error::Runtime)?;
        Ok(outputs
            .iter()
            .zip(ids)
            .map(|(id, node)| {
                (
                    *id,
                    result
                        .value(node)
                        .expect("requested output")
                        .as_ref()
                        .map(|v| Decimal96::from_runtime(*v))
                        .map_err(public_error),
                )
            })
            .collect())
    }
    pub fn invoke_public(
        &self,
        inputs: &[(String, Decimal96)],
        outputs: &[String],
    ) -> Result<
        Vec<(
            String,
            std::result::Result<Decimal96, PublicEvaluationError>,
        )>,
    > {
        if inputs.len() > self.limits.ports || outputs.len() > self.limits.ports {
            return Err(Error::Limit);
        }
        let lookup = |id: &str| -> Result<&str> {
            Ok(self
                .record
                .ports
                .iter()
                .find(|p| p.name == id)
                .ok_or(Error::Identity)?
                .name
                .as_str())
        };
        let request = NamedInvocationRequest {
            inputs: inputs
                .iter()
                .map(|(id, v)| Ok((lookup(id)?.into(), v.runtime()?.into())))
                .collect::<Result<_>>()?,
            outputs: outputs
                .iter()
                .map(|id| Ok(lookup(id)?.into()))
                .collect::<Result<_>>()?,
        };
        let result = EvaluationContext::new(self.snapshot()?)
            .invoke_named(&request)
            .map_err(|_| Error::InvalidDraft)?;
        outputs
            .iter()
            .map(|id| {
                let value = result
                    .outputs()
                    .get(lookup(id)?)
                    .ok_or(Error::Runtime)?
                    .as_ref()
                    .map(|v| {
                        let pivot_types::PivotValue::Decimal(v) = v else {
                            unreachable!()
                        };
                        Decimal96::from_runtime(*v)
                    })
                    .map_err(Clone::clone);
                Ok((id.clone(), value))
            })
            .collect()
    }
    pub fn diagnostics(&self) -> Vec<Diagnostic> {
        let mut out = vec![];
        let live: BTreeMap<_, _> = self
            .record
            .nodes
            .iter()
            .map(|n| (n.id, &n.definition))
            .collect();
        for n in &self.record.nodes {
            match &n.definition {
                Definition::ParseFailed { offset, code, .. } => out.push(Diagnostic {
                    node: Some(n.id),
                    port: None,
                    kind: DiagnosticKind::Parse {
                        offset: *offset,
                        code: *code,
                    },
                }),
                Definition::BindFailed { name, .. } => out.push(Diagnostic {
                    node: Some(n.id),
                    port: None,
                    kind: DiagnosticKind::Unresolved(name.clone()),
                }),
                Definition::Bound { tokens, .. } => {
                    let missing: BTreeSet<_> = tokens
                        .iter()
                        .filter_map(|t| match t {
                            Token::Node { node, .. } if !live.contains_key(node) => Some(*node),
                            _ => None,
                        })
                        .collect();
                    for id in missing {
                        out.push(Diagnostic {
                            node: Some(n.id),
                            port: None,
                            kind: DiagnosticKind::Missing(id),
                        });
                    }
                }
                _ => (),
            }
        }
        for p in &self.record.ports {
            if !live.contains_key(&p.node)
                || (!matches!(p.kind, PortKind::Output)
                    && !matches!(live.get(&p.node), Some(Definition::Input(_))))
            {
                out.push(Diagnostic {
                    node: None,
                    port: Some(p.id),
                    kind: DiagnosticKind::InvalidPort,
                });
            }
        }
        if let Err(SnapshotError::Unorderable { nodes }) = self.core.snapshot() {
            for node in nodes {
                let id = self.nodes.iter().find(|(_, n)| **n == node).unwrap().0;
                out.push(Diagnostic {
                    node: Some(*id),
                    port: None,
                    kind: DiagnosticKind::Unorderable,
                });
            }
        }
        out
    }
    fn change(
        &mut self,
        changed: Option<Uuid>,
        apply: impl FnOnce(&mut WorkingModel, &BTreeMap<Uuid, NodeId>) -> Result<()>,
    ) -> Result<()> {
        let mut candidate = self.core.stage();
        apply(&mut candidate, &self.nodes)?;
        self.install(
            candidate,
            self.nodes.clone(),
            self.ports.clone(),
            changed,
            None,
        )
    }
    pub fn rename(&mut self, id: Uuid, name: &str) -> Result<()> {
        if name.len() > 128 {
            return Err(Error::Limit);
        }
        self.change(None, |core, ids| {
            core.rename(*ids.get(&id).ok_or(Error::Identity)?, name)
                .map_err(|_| Error::Runtime)
        })
    }
    pub fn set_port_target(&mut self, id: Uuid, node: Uuid) -> Result<()> {
        let port = *self.ports.get(&id).ok_or(Error::Identity)?;
        let output = matches!(
            self.record
                .ports
                .iter()
                .find(|p| p.id == id)
                .ok_or(Error::Identity)?
                .kind,
            PortKind::Output
        );
        self.change(None, |core, nodes| {
            let node = *nodes.get(&node).ok_or(Error::Identity)?;
            if output {
                core.update_output_port(port, node)
            } else {
                core.patch_input_port(port, Some(node), None)
            }
            .map_err(|_| Error::Runtime)
        })
    }
    pub fn remove(&mut self, id: Uuid) -> Result<()> {
        self.change(None, |core, ids| {
            core.remove(*ids.get(&id).ok_or(Error::Identity)?)
                .map_err(|_| Error::Runtime)
        })
    }
    pub fn set_formula(&mut self, id: Uuid, source: &str) -> Result<()> {
        if source.len() > 4096 {
            return Err(Error::Limit);
        }
        self.change(Some(id), |core, ids| {
            core.set_expression(*ids.get(&id).ok_or(Error::Identity)?, source)
                .map_err(|_| Error::Runtime)
        })
    }
    pub fn set_input(&mut self, id: Uuid, value: Decimal96) -> Result<()> {
        self.change(Some(id), |core, ids| {
            core.replace_input(*ids.get(&id).ok_or(Error::Identity)?, value.runtime()?)
                .map_err(|_| Error::Runtime)
        })?;
        self.record
            .nodes
            .iter_mut()
            .find(|n| n.id == id)
            .ok_or(Error::Identity)?
            .definition = Definition::Input(value);
        Ok(())
    }
    pub fn add_input(&mut self, id: Uuid, name: &str, value: Decimal96) -> Result<()> {
        self.available(id)?;
        if name.len() > 128 {
            return Err(Error::Limit);
        }
        let mut core = self.core.stage();
        let handle = core
            .add_input(name, value.runtime()?)
            .map_err(|_| Error::Runtime)?;
        let mut nodes = self.nodes.clone();
        nodes.insert(id, handle);
        self.install(core, nodes, self.ports.clone(), Some(id), None)?;
        self.record
            .nodes
            .iter_mut()
            .find(|n| n.id == id)
            .expect("installed input")
            .definition = Definition::Input(value);
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn repeated_import_allocates_independent_node_and_port_handles() {
        let mut a = EditableGraph::new("Example", Limits::default()).unwrap();
        let node = Uuid::new_v4();
        let port = Uuid::new_v4();
        a.add_input(node, "N", Decimal96::from_parts(1, 0, 0, false, 0).unwrap())
            .unwrap();
        a.add_port(port, "Output", node, PortKind::Output).unwrap();
        let b = EditableGraph::load(a.record().clone(), Limits::default()).unwrap();
        let c = EditableGraph::load(a.record().clone(), Limits::default()).unwrap();
        assert_ne!(a.nodes[&node], b.nodes[&node]);
        assert_ne!(b.nodes[&node], c.nodes[&node]);
        assert_ne!(a.ports[&port], b.ports[&port]);
        assert_ne!(b.ports[&port], c.ports[&port]);
    }
}
