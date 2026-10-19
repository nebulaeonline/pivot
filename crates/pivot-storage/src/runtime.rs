use crate::*;
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

use crate::graph::{GraphHeader, capture_graph, capture_graph_snapshot, restore_hosted_graph};
use pivot_core::{
    interface::PortId,
    model::{ModelSnapshot, NodeId, WorkingModel},
};
use std::collections::BTreeMap;

pub struct RestoredModel {
    record: Record,
    graph: EditableGraph,
    limits: Limits,
}
impl Record {
    pub fn graph_payload(&self, limits: Limits) -> Result<GraphPayload> {
        self.validate(limits)?;
        Ok(GraphPayload {
            version: 1,
            numeric: self.numeric,
            expression: self.expression,
            model: self.model,
            revision: self.revision,
            name: self.name.clone(),
            node_ids: self.node_ids.clone(),
            port_ids: self.port_ids.clone(),
            nodes: self.nodes.clone(),
            ports: self.ports.clone(),
        })
    }
}
fn hosted(g: GraphPayload, h: Header) -> Record {
    Record {
        codec: 1,
        numeric: g.numeric,
        expression: g.expression,
        store: h.store,
        epoch: h.epoch,
        model: g.model,
        revision: g.revision,
        sequence: h.sequence,
        name: g.name,
        node_ids: g.node_ids,
        port_ids: g.port_ids,
        nodes: g.nodes,
        ports: g.ports,
        publication: h.publication,
    }
}
pub fn capture(
    core: &WorkingModel,
    nodes: &BTreeMap<Uuid, NodeId>,
    ports: &BTreeMap<Uuid, PortId>,
    h: Header,
    limits: Limits,
) -> Result<Record> {
    let graph = capture_graph(
        core,
        nodes,
        ports,
        GraphHeader {
            model: h.model,
            revision: h.revision,
        },
        limits,
    )?;
    let record = hosted(graph, h);
    encode(&record, limits)?;
    if record.publication.is_some() {
        restore(record.clone(), limits)?;
    }
    Ok(record)
}
pub fn capture_snapshot(
    core: &ModelSnapshot,
    nodes: &BTreeMap<Uuid, NodeId>,
    ports: &BTreeMap<Uuid, PortId>,
    h: Header,
    limits: Limits,
) -> Result<Record> {
    let graph = capture_graph_snapshot(
        core,
        nodes,
        ports,
        GraphHeader {
            model: h.model,
            revision: h.revision,
        },
        limits,
    )?;
    let record = hosted(graph, h);
    encode(&record, limits)?;
    if record.publication.is_some() {
        restore(record.clone(), limits)?;
    }
    Ok(record)
}
pub fn restore(record: Record, limits: Limits) -> Result<RestoredModel> {
    encode(&record, limits)?;
    let graph = restore_hosted_graph(record.graph_payload(limits)?, limits)?;
    if record.publication.is_some() {
        graph
            .validate_snapshot()
            .map_err(|_| Error::InvalidPublished)?;
    }
    Ok(RestoredModel {
        record,
        graph,
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

impl RestoredModel {
    pub fn record(&self) -> &Record {
        &self.record
    }
    pub fn evaluate(&self, outputs: &[Uuid]) -> Result<Values> {
        self.graph.evaluate(outputs)
    }
    pub fn diagnostics(&self) -> Vec<Diagnostic> {
        self.graph.diagnostics()
    }
    pub fn invoke_public(
        &self,
        inputs: &[(String, Decimal96)],
        outputs: &[String],
    ) -> Result<NamedValues> {
        let publication = self
            .record
            .publication
            .as_ref()
            .ok_or(Error::InvalidPublished)?;
        let lookup = |external: &str| -> Result<String> {
            let (id, _) = publication
                .public_ports
                .iter()
                .find(|(_, s)| s == external)
                .ok_or(Error::Identity)?;
            Ok(self
                .record
                .ports
                .iter()
                .find(|p| p.id == *id)
                .ok_or(Error::Identity)?
                .name
                .clone())
        };
        if inputs.len() > self.limits.ports || outputs.len() > self.limits.ports {
            return Err(Error::Limit);
        }
        let inputs = inputs
            .iter()
            .map(|(id, v)| Ok((lookup(id)?, *v)))
            .collect::<Result<Vec<_>>>()?;
        let named = outputs
            .iter()
            .map(|id| lookup(id))
            .collect::<Result<Vec<_>>>()?;
        let result = self.graph.invoke_public(&inputs, &named).map_err(|e| {
            if e == Error::InvalidDraft {
                Error::InvalidPublished
            } else {
                e
            }
        })?;
        Ok(outputs
            .iter()
            .zip(result)
            .map(|(id, (_, v))| (id.clone(), v))
            .collect())
    }
    fn edit(&mut self, apply: impl FnOnce(&mut EditableGraph) -> Result<()>) -> Result<()> {
        if self.record.publication.is_some() {
            return Err(Error::InvalidPublished);
        }
        let sequence = self.record.sequence.checked_add(1).ok_or(Error::Limit)?;
        let mut graph = self.graph.stage();
        apply(&mut graph)?;
        let record = hosted(
            graph.record().clone(),
            Header {
                store: self.record.store,
                epoch: self.record.epoch,
                model: self.record.model,
                revision: graph.record().revision,
                sequence,
                publication: None,
            },
        );
        encode(&record, self.limits)?;
        self.record = record;
        self.graph = graph;
        Ok(())
    }
    pub fn rename(&mut self, id: Uuid, name: &str) -> Result<()> {
        self.edit(|g| g.rename(id, name))
    }
    pub fn remove(&mut self, id: Uuid) -> Result<()> {
        self.edit(|g| g.remove(id))
    }
    pub fn set_port_target(&mut self, id: Uuid, node: Uuid) -> Result<()> {
        self.edit(|g| g.set_port_target(id, node))
    }
    pub fn set_formula(&mut self, id: Uuid, source: &str) -> Result<()> {
        self.edit(|g| g.set_formula(id, source))
    }
    pub fn set_input(&mut self, id: Uuid, value: Decimal96) -> Result<()> {
        self.edit(|g| g.set_input(id, value))
    }
    pub fn add_input(&mut self, id: Uuid, name: &str, value: Decimal96) -> Result<()> {
        self.edit(|g| g.add_input(id, name, value))
    }
}
