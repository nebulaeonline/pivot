//! One-graph, in-memory documents. No files, hosted registry or wire contract.
use crate::{
    Decimal96, Diagnostic, EditableGraph, GraphPayload, Limits, NamedValues, PortKind, Uuid, Values,
};
use std::sync::Arc;

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Error {
    Graph(crate::Error),
    UnsupportedVersion,
    Identity,
    StaleSession,
    StaleRevision,
    StaleCheckpoint,
    Exhausted,
}
impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "document {self:?}")
    }
}
impl std::error::Error for Error {}
impl From<crate::Error> for Error {
    fn from(value: crate::Error) -> Self {
        Self::Graph(value)
    }
}
pub type Result<T> = std::result::Result<T, Error>;

/// Owned logical representation; not a native document byte format.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ModelDocument {
    pub format: u32,
    /// Logical ancestry, not a physical file or a loaded session identity.
    pub document_id: Uuid,
    pub state_id: Uuid,
    pub graph: GraphPayload,
}
impl ModelDocument {
    pub fn validate(&self, limits: Limits) -> Result<()> {
        if self.format != 1 {
            return Err(Error::UnsupportedVersion);
        }
        if self.document_id.is_nil() || self.state_id.is_nil() {
            return Err(Error::Identity);
        }
        self.graph.validate(limits)?;
        Ok(())
    }
}

/// A caller must retain all three tokens, not just a logical DocumentId.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Expected {
    pub session: Uuid,
    pub state: Uuid,
    pub graph_revision: Uuid,
}

/// Private provenance prevents another session's export from marking this one saved.
#[derive(Clone, Debug)]
pub struct ExportCapture {
    document: Arc<ModelDocument>,
    session: Uuid,
    sequence: u64,
}
impl ExportCapture {
    pub fn document(&self) -> &ModelDocument {
        &self.document
    }
}

#[derive(Clone, Debug)]
pub struct Calculation<T> {
    pub expected: Expected,
    pub generation: Uuid,
    pub values: T,
}

/// Private runtime state; no registry, retained mutation receipts or publication.
pub struct DocumentSession {
    id: Uuid,
    document_id: Uuid,
    state: Uuid,
    graph: EditableGraph,
    saved: Option<Uuid>,
    next_export: u64,
    acknowledged: Option<u64>,
}
impl DocumentSession {
    pub fn new(name: &str, limits: Limits) -> Result<Self> {
        Ok(Self::from_graph(
            Uuid::new_v4(),
            Uuid::new_v4(),
            EditableGraph::new(name, limits)?,
        ))
    }
    /// Loading logical content does not attest that any file has been saved.
    pub fn load(document: ModelDocument, limits: Limits) -> Result<Self> {
        document.validate(limits)?;
        Ok(Self::from_graph(
            document.document_id,
            document.state_id,
            EditableGraph::load(document.graph, limits)?,
        ))
    }
    pub fn load_graph(graph: GraphPayload, limits: Limits) -> Result<Self> {
        Ok(Self::from_graph(
            Uuid::new_v4(),
            Uuid::new_v4(),
            EditableGraph::load(graph, limits)?,
        ))
    }
    fn from_graph(document_id: Uuid, state: Uuid, graph: EditableGraph) -> Self {
        Self {
            id: Uuid::new_v4(),
            document_id,
            state,
            graph,
            saved: None,
            next_export: 0,
            acknowledged: None,
        }
    }
    pub fn expected(&self) -> Expected {
        Expected {
            session: self.id,
            state: self.state,
            graph_revision: self.graph.record().revision,
        }
    }
    pub fn document_id(&self) -> Uuid {
        self.document_id
    }
    pub fn graph(&self) -> &GraphPayload {
        self.graph.record()
    }
    pub fn is_dirty(&self) -> bool {
        self.saved != Some(self.state)
    }
    fn check(&self, expected: Expected) -> Result<()> {
        if expected.session != self.id {
            return Err(Error::StaleSession);
        }
        if expected.state != self.state || expected.graph_revision != self.graph.record().revision {
            return Err(Error::StaleRevision);
        }
        Ok(())
    }
    fn edit(
        &mut self,
        expected: Expected,
        apply: impl FnOnce(&mut EditableGraph) -> crate::Result<()>,
    ) -> Result<Expected> {
        self.check(expected)?;
        let mut candidate = self.graph.stage();
        apply(&mut candidate)?;
        // Ignore the newly assigned graph revision when detecting an actual no-op.
        let mut comparable = candidate.record().clone();
        comparable.revision = self.graph.record().revision;
        if comparable != *self.graph.record() {
            self.graph = candidate;
            self.state = Uuid::new_v4();
        }
        Ok(self.expected())
    }
    pub fn add_input(
        &mut self,
        expected: Expected,
        id: Uuid,
        name: &str,
        value: Decimal96,
    ) -> Result<Expected> {
        self.edit(expected, |g| g.add_input(id, name, value))
    }
    pub fn add_formula(
        &mut self,
        expected: Expected,
        id: Uuid,
        name: &str,
        source: &str,
    ) -> Result<Expected> {
        self.edit(expected, |g| g.add_formula(id, name, source))
    }
    pub fn set_input(
        &mut self,
        expected: Expected,
        id: Uuid,
        value: Decimal96,
    ) -> Result<Expected> {
        self.edit(expected, |g| g.set_input(id, value))
    }
    pub fn set_formula(&mut self, expected: Expected, id: Uuid, source: &str) -> Result<Expected> {
        self.edit(expected, |g| g.set_formula(id, source))
    }
    pub fn rename(&mut self, expected: Expected, id: Uuid, name: &str) -> Result<Expected> {
        self.edit(expected, |g| g.rename(id, name))
    }
    pub fn remove(&mut self, expected: Expected, id: Uuid) -> Result<Expected> {
        self.edit(expected, |g| g.remove(id))
    }
    pub fn add_port(
        &mut self,
        expected: Expected,
        id: Uuid,
        name: &str,
        node: Uuid,
        kind: PortKind,
    ) -> Result<Expected> {
        self.edit(expected, |g| g.add_port(id, name, node, kind))
    }
    pub fn remove_port(&mut self, expected: Expected, id: Uuid) -> Result<Expected> {
        self.edit(expected, |g| g.remove_port(id))
    }
    pub fn set_port_target(
        &mut self,
        expected: Expected,
        id: Uuid,
        node: Uuid,
    ) -> Result<Expected> {
        self.edit(expected, |g| g.set_port_target(id, node))
    }
    pub fn validate(&self, expected: Expected) -> Result<Vec<Diagnostic>> {
        self.check(expected)?;
        Ok(self.graph.diagnostics())
    }
    pub fn evaluate(&self, expected: Expected, outputs: &[Uuid]) -> Result<Calculation<Values>> {
        self.check(expected)?;
        Ok(Calculation {
            expected,
            generation: Uuid::new_v4(),
            values: self.graph.evaluate(outputs)?,
        })
    }
    pub fn invoke(
        &self,
        expected: Expected,
        inputs: &[(String, Decimal96)],
        outputs: &[String],
    ) -> Result<Calculation<NamedValues>> {
        self.check(expected)?;
        Ok(Calculation {
            expected,
            generation: Uuid::new_v4(),
            values: self.graph.invoke_public(inputs, outputs)?,
        })
    }
    pub fn export(&mut self, expected: Expected) -> Result<ExportCapture> {
        self.check(expected)?;
        let sequence = self.next_export.checked_add(1).ok_or(Error::Exhausted)?;
        self.next_export = sequence;
        Ok(ExportCapture {
            session: self.id,
            sequence,
            document: Arc::new(ModelDocument {
                format: 1,
                document_id: self.document_id,
                state_id: self.state,
                graph: self.graph.record().clone(),
            }),
        })
    }
    /// Trusted caller's in-memory attestation for this exact capture, not disk I/O.
    /// Future file brokers may call this only after resolving the actual save.
    pub fn acknowledge_saved(&mut self, capture: &ExportCapture) -> Result<()> {
        if capture.session != self.id {
            return Err(Error::StaleSession);
        }
        if self.acknowledged.is_some_and(|n| capture.sequence < n) {
            return Err(Error::StaleCheckpoint);
        }
        self.acknowledged = Some(capture.sequence);
        self.saved = Some(capture.document.state_id);
        Ok(())
    }
}
