use crate::*;
use pivot_core::{
    evaluation::{EvaluationContext, EvaluationError},
    expression::{BinaryOp, UnaryOp},
    interface::{InputRequirement, NamedInvocationRequest, PortId, PublicEvaluationError},
    model::{ModelSnapshot, NodeId, SnapshotError, WorkingModel, transfer as t},
};
use std::collections::BTreeMap;

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum DiagnosticKind {
    Parse { offset: u32, code: u8 },
    Unresolved(String),
    Missing(Uuid),
    InvalidPort,
    Unorderable,
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Diagnostic {
    pub node: Option<Uuid>,
    pub port: Option<Uuid>,
    pub kind: DiagnosticKind,
}
pub struct RestoredModel {
    record: Record,
    core: WorkingModel,
    nodes: BTreeMap<Uuid, NodeId>,
    ports: BTreeMap<Uuid, PortId>,
    limits: Limits,
}
/// Internal export bridge. Caller owns opaque core handles and complete portable
/// identity maps. No handles appear in returned records or encoded data.
pub fn capture(
    core: &WorkingModel,
    nodes: &BTreeMap<Uuid, NodeId>,
    ports: &BTreeMap<Uuid, PortId>,
    header: Header,
    limits: Limits,
) -> Result<Record> {
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
pub fn capture_snapshot(
    core: &ModelSnapshot,
    nodes: &BTreeMap<Uuid, NodeId>,
    ports: &BTreeMap<Uuid, PortId>,
    header: Header,
    limits: Limits,
) -> Result<Record> {
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
fn from_image(
    image: t::Image,
    node_ids: Vec<Uuid>,
    port_ids: Vec<Uuid>,
    h: Header,
    limits: Limits,
) -> Result<Record> {
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
    let r = Record {
        codec: 1,
        numeric: 1,
        expression: 1,
        store: h.store,
        epoch: h.epoch,
        model: h.model,
        revision: h.revision,
        sequence: h.sequence,
        name: image.name,
        node_ids,
        port_ids,
        nodes,
        ports,
        publication: h.publication,
    };
    encode(&r, limits)?;
    if r.publication.is_some() {
        restore(r.clone(), limits)?;
    }
    Ok(r)
}
pub fn restore(record: Record, limits: Limits) -> Result<RestoredModel> {
    encode(&record, limits)?;
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
    let imported = t::import(t::Image {
        name: record.name.clone(),
        node_slots: record.node_ids.len(),
        port_slots: record.port_ids.len(),
        nodes,
        ports,
    })
    .map_err(|_| Error::Malformed)?;
    if record.publication.is_some() {
        imported
            .model
            .snapshot()
            .map_err(|_| Error::InvalidPublished)?;
    }
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
    Ok(RestoredModel {
        record,
        core: imported.model,
        nodes,
        ports,
        limits,
    })
}
/// Complete-set import: no partially recovered collection escapes on error.
pub fn restore_all(records: Vec<Record>, limits: Limits) -> Result<Vec<RestoredModel>> {
    if records.len() > limits.nodes {
        return Err(Error::Limit);
    }
    let mut identity = BTreeMap::new();
    let mut keys = BTreeSet::new();
    let mut context = None;
    let mut publications = BTreeSet::new();
    let mut snapshots = BTreeSet::new();
    let mut lineages = BTreeMap::new();
    let mut public_ids = BTreeMap::new();
    let mut public_ports = BTreeMap::new();
    let mut directions = BTreeMap::new();
    let mut bytes = 0usize;
    for r in &records {
        if context.is_some_and(|c| c != r.store) {
            return Err(Error::Identity);
        }
        context = Some(r.store);
        if !keys.insert((
            r.model,
            r.revision,
            r.publication.as_ref().map(|p| (p.model, p.revision)),
        )) {
            return Err(Error::Identity);
        }
        if let Some(p) = &r.publication {
            if !publications.insert((p.model, p.revision))
                || !snapshots.insert(p.snapshot)
                || lineages
                    .insert(p.model, r.model)
                    .is_some_and(|m| m != r.model)
            {
                return Err(Error::Identity);
            }
            for (port, external) in &p.public_ports {
                if public_ids
                    .insert((p.model, external.clone()), *port)
                    .is_some_and(|old| old != *port)
                    || public_ports
                        .insert((p.model, *port), external.clone())
                        .is_some_and(|old| old != *external)
                {
                    return Err(Error::Identity);
                }
            }
        }
        for p in &r.ports {
            let output = matches!(p.kind, PortKind::Output);
            if directions
                .insert((r.model, p.id), output)
                .is_some_and(|old| old != output)
            {
                return Err(Error::Identity);
            }
        }
        for (id, kind) in r
            .node_ids
            .iter()
            .map(|id| (id, 0u8))
            .chain(r.port_ids.iter().map(|id| (id, 1u8)))
        {
            if identity
                .insert(*id, (r.model, kind))
                .is_some_and(|m| m != (r.model, kind))
            {
                return Err(Error::Identity);
            }
        }
        bytes = bytes
            .checked_add(encode(r, limits)?.len())
            .ok_or(Error::Limit)?;
        if bytes > limits.bytes {
            return Err(Error::Limit);
        }
    }
    records.into_iter().map(|r| restore(r, limits)).collect()
}
fn public_error(e: &EvaluationError) -> PublicEvaluationError {
    match e {
        EvaluationError::Arithmetic { error, .. } => PublicEvaluationError::Arithmetic(*error),
        EvaluationError::DependencyFailed { .. } => PublicEvaluationError::DependencyFailed,
        _ => PublicEvaluationError::InvalidExpression,
    }
}
pub type Values = Vec<(Uuid, std::result::Result<Decimal96, PublicEvaluationError>)>;
impl RestoredModel {
    pub fn record(&self) -> &Record {
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
        let p = self
            .record
            .publication
            .as_ref()
            .ok_or(Error::InvalidPublished)?;
        if inputs.len() > self.limits.ports || outputs.len() > self.limits.ports {
            return Err(Error::Limit);
        }
        let lookup = |id: &str| -> Result<&str> {
            let (key, _) = p
                .public_ports
                .iter()
                .find(|(_, value)| value == id)
                .ok_or(Error::Identity)?;
            Ok(self
                .record
                .ports
                .iter()
                .find(|port| port.id == *key)
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
            .map_err(|_| Error::InvalidPublished)?;
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
        if self.record.publication.is_some() {
            return Err(Error::InvalidPublished);
        }
        let sequence = self.record.sequence.checked_add(1).ok_or(Error::Limit)?;
        let mut candidate = self.core.stage();
        apply(&mut candidate, &self.nodes)?;
        let header = Header {
            store: self.record.store,
            epoch: self.record.epoch,
            model: self.record.model,
            revision: Uuid::new_v4(),
            sequence,
            publication: None,
        };
        let mut record = capture(&candidate, &self.nodes, &self.ports, header, self.limits)?;
        // Retain exact representations untouched by this edit, even if runtime
        // construction normalized signed zero or other representation details.
        for n in &mut record.nodes {
            if Some(n.id) != changed
                && let Some(old) = self.record.nodes.iter().find(|v| v.id == n.id)
            {
                n.definition = old.definition.clone();
            }
        }
        for p in &mut record.ports {
            if let Some(old) = self.record.ports.iter().find(|v| v.id == p.id) {
                p.kind = old.kind.clone();
            }
        }
        encode(&record, self.limits)?;
        self.record = record;
        self.core = candidate;
        Ok(())
    }
    pub fn rename(&mut self, id: Uuid, name: &str) -> Result<()> {
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
        if self.record.publication.is_some() {
            return Err(Error::InvalidPublished);
        }
        if id.is_nil() || self.nodes.contains_key(&id) || self.ports.contains_key(&id) {
            return Err(Error::Identity);
        }
        let sequence = self.record.sequence.checked_add(1).ok_or(Error::Limit)?;
        let mut candidate = self.core.stage();
        let handle = candidate
            .add_input(name, value.runtime()?)
            .map_err(|_| Error::Runtime)?;
        let mut identities = self.nodes.clone();
        identities.insert(id, handle);
        let h = Header {
            store: self.record.store,
            epoch: self.record.epoch,
            model: self.record.model,
            revision: Uuid::new_v4(),
            sequence,
            publication: None,
        };
        let mut record = capture(&candidate, &identities, &self.ports, h, self.limits)?;
        for n in &mut record.nodes {
            if let Some(old) = self.record.nodes.iter().find(|v| v.id == n.id) {
                n.definition = old.definition.clone();
            } else {
                n.definition = Definition::Input(value);
            }
        }
        for p in &mut record.ports {
            if let Some(old) = self.record.ports.iter().find(|v| v.id == p.id) {
                p.kind = old.kind.clone();
            }
        }
        encode(&record, self.limits)?;
        self.core = candidate;
        self.nodes = identities;
        self.record = record;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn repeated_import_allocates_independent_handles() {
        let mut core = WorkingModel::new("Example").unwrap();
        let n = core.add_input("N", "1".parse().unwrap()).unwrap();
        let id = Uuid::new_v4();
        let port = core.add_output_port("Output", n).unwrap();
        let public = Uuid::new_v4();
        let r = capture(
            &core,
            &BTreeMap::from([(id, n)]),
            &BTreeMap::from([(public, port)]),
            Header {
                store: Uuid::new_v4(),
                epoch: Uuid::new_v4(),
                model: Uuid::new_v4(),
                revision: Uuid::new_v4(),
                sequence: 9,
                publication: None,
            },
            Limits::default(),
        )
        .unwrap();
        let a = restore(r.clone(), Limits::default()).unwrap();
        let b = restore(r, Limits::default()).unwrap();
        assert_ne!(a.nodes[&id], b.nodes[&id]);
        assert_ne!(a.nodes[&id], n);
        assert_ne!(a.ports[&public], b.ports[&public]);
        assert_ne!(a.ports[&public], port);
        assert_eq!(core.node_id("N"), Some(n));
        assert_eq!(core.revision(), core.snapshot().unwrap().revision());
    }
}
