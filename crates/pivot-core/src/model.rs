//! Editable definitions and validated immutable snapshots. No evaluation cache
//! lives in either type. Storage and synchronization are implementation details.
use crate::expression::{self, BinaryOp, Expr, ParseError, UnaryOp};
use crate::interface::{
    InputPort, InputRequirement, InterfaceError, ModelInterface, OutputPort, PortId,
};
use pivot_types::PivotDecimal;
use std::{
    collections::{BTreeMap, BTreeSet},
    fmt,
    sync::{
        Arc,
        atomic::{AtomicU64, Ordering},
    },
};

static NEXT_MODEL: AtomicU64 = AtomicU64::new(1);
/// Persistent standalone backend and bounded shared runtime.
#[cfg(feature = "core-integration")]
pub mod integration;
/// Isolated paged structural-mutation research; never used by default models.
#[cfg(feature = "scaling-experiment")]
pub mod structural;
#[cfg(all(feature = "core-integration", not(feature = "scaling-experiment")))]
#[allow(dead_code, unused_imports)] // research-only adapters await later migration stages
mod structural;
/// Internal validated transfer boundary; not a persistent format or client API.
#[doc(hidden)]
pub mod transfer;

/// Process-local identity, not a persistence or wire identifier.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct NodeId {
    model: u64,
    node: u64,
}

/// Revision identity is scoped to a particular working model.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Revision {
    model: u64,
    version: u64,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ModelError {
    #[cfg(feature = "core-integration")]
    Backend(&'static str),
    Interface(InterfaceError),
    InvalidName(String),
    DuplicateName(String),
    UnknownNode(NodeId),
    NotInput(NodeId),
    IdentityExhausted,
}
impl fmt::Display for ModelError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{self:?}")
    }
}
impl std::error::Error for ModelError {}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SnapshotError {
    #[cfg(feature = "core-integration")]
    Backend(&'static str),
    Interface(InterfaceError),
    Parse {
        node: NodeId,
        error: ParseError,
    },
    UnknownName {
        node: NodeId,
        name: String,
    },
    MissingReference {
        node: NodeId,
        target: NodeId,
    },
    /// These nodes cannot be ordered: cycle members and nodes depending on them.
    Unorderable {
        nodes: Vec<NodeId>,
    },
}
impl fmt::Display for SnapshotError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{self:?}")
    }
}
impl std::error::Error for SnapshotError {}

#[derive(Debug, Clone)]
enum Definition {
    Input(PivotDecimal),
    Computed {
        source: String,
        binding: Result<(Vec<BoundToken>, Vec<NodeId>), SnapshotError>,
    },
}
#[derive(Debug)]
struct WorkingNode {
    name: String,
    definition: Definition,
}

impl Clone for WorkingNode {
    fn clone(&self) -> Self {
        #[cfg(feature = "scaling-experiment")]
        crate::experiment_profile::copy(
            std::mem::size_of::<Self>()
                + self.name.len()
                + match &self.definition {
                    Definition::Input(_) => 0,
                    Definition::Computed { source, binding } => {
                        source.len()
                            + binding
                                .as_ref()
                                .map(|(t, d)| {
                                    t.len() * std::mem::size_of::<BoundToken>()
                                        + d.len() * std::mem::size_of::<NodeId>()
                                })
                                .unwrap_or(0)
                    }
                },
        );
        Self {
            name: self.name.clone(),
            definition: self.definition.clone(),
        }
    }
}

/// Mutations require exclusive access. Invalid formulas may be saved while
/// editing; validation occurs at snapshot capture. Not implicitly cloneable.
#[derive(Debug)]
pub struct WorkingModel {
    #[cfg(feature = "core-integration")]
    integrated: Option<integration::Integrated>,
    name: String,
    revision: Revision,
    nodes: BTreeMap<NodeId, WorkingNode>,
    names: BTreeMap<String, NodeId>,
    next_node: u64,
    next_port: u64,
    interface: ModelInterface,
}
/// Explicit backend identity for migration diagnostics.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BackendKind {
    Reference,
    Persistent,
}
/// Raw editable definitions, including invalid source. Core IDs remain local.
pub enum DraftDefinition<'a> {
    Input(PivotDecimal),
    Computed {
        source: &'a str,
        binding: Result<&'a [NodeId], &'a SnapshotError>,
    },
}
impl WorkingModel {
    /// Explicit private transaction branch preserving existing bound identities.
    /// The caller must serialize installation and discard failed branches. This
    /// is not a new independent model: do not publish competing branches as one
    /// mutable resource. Allocated branch IDs must not escape before commit.
    pub fn stage(&self) -> Self {
        #[cfg(feature = "scaling-experiment")]
        let _phase = crate::experiment_profile::scope(1);
        Self {
            #[cfg(feature = "core-integration")]
            integrated: self.integrated.clone(),
            name: self.name.clone(),
            revision: self.revision,
            nodes: self.nodes.clone(),
            names: self.names.clone(),
            next_node: self.next_node,
            next_port: self.next_port,
            interface: self.interface.clone(),
        }
    }
    pub fn draft_nodes(&self) -> impl Iterator<Item = (NodeId, &str, DraftDefinition<'_>)> {
        let legacy = self.nodes.iter().map(|(&id, n)| {
            (
                id,
                n.name.as_str(),
                match &n.definition {
                    Definition::Input(v) => DraftDefinition::Input(*v),
                    Definition::Computed { source, binding } => DraftDefinition::Computed {
                        source,
                        binding: binding.as_ref().map(|(_, deps)| deps.as_slice()),
                    },
                },
            )
        });
        #[cfg(feature = "core-integration")]
        let legacy = legacy.chain(self.integrated.iter().flat_map(|i| i.graph().draft_nodes()));
        legacy
    }
    pub fn contains_node(&self, id: NodeId) -> bool {
        #[cfg(feature = "core-integration")]
        if let Some(i) = &self.integrated {
            return i.graph().contains_node(id);
        }
        self.nodes.contains_key(&id)
    }
    /// Replace either definition kind without changing node identity. Existing
    /// input ports may become invalid drafts when converting to computed.
    pub fn replace_input(&mut self, id: NodeId, value: PivotDecimal) -> Result<(), ModelError> {
        #[cfg(feature = "core-integration")]
        if self.integrated.is_some() {
            return self.integrated_edit(structural::Edit::ReplaceInput(id, value));
        }
        if !self.nodes.contains_key(&id) {
            return Err(ModelError::UnknownNode(id));
        }
        self.advance()?;
        self.nodes.get_mut(&id).unwrap().definition = Definition::Input(value);
        Ok(())
    }
    /// Update an input port in place, preserving its core and public identity.
    /// Omitted fields stay unchanged, including an already-invalid saved target.
    pub fn patch_input_port(
        &mut self,
        id: PortId,
        node: Option<NodeId>,
        requirement: Option<InputRequirement>,
    ) -> Result<(), ModelError> {
        #[cfg(feature = "core-integration")]
        if self.integrated.is_some() {
            if node.is_none() && requirement.is_none() {
                if self.interface().inputs().any(|p| p.id() == id) {
                    return Ok(());
                }
                return Err(ModelError::Interface(InterfaceError::UnknownPort(id)));
            }
            return self.integrated_edit(structural::Edit::Port(structural::PortEdit::Input(
                id,
                node,
                requirement,
            )));
        }
        let old = self
            .interface
            .inputs
            .values()
            .find(|p| p.id == id)
            .ok_or(ModelError::Interface(InterfaceError::UnknownPort(id)))?;
        if let Some(node) = node {
            return self.update_input_port(
                id,
                node,
                requirement.unwrap_or_else(|| old.requirement.clone()),
            );
        }
        let name = old.name.clone();
        if let Some(requirement) = requirement {
            if let InputRequirement::Optional { default } = &requirement
                && default.value_type() != pivot_types::ValueType::Decimal
            {
                return Err(ModelError::Interface(InterfaceError::TypeMismatch {
                    name,
                    expected: pivot_types::ValueType::Decimal,
                    actual: default.value_type(),
                }));
            }
            self.advance()?;
            self.interface.inputs.get_mut(&name).unwrap().requirement = requirement;
        }
        Ok(())
    }
    /// Complete replacement, retained for existing in-process callers.
    pub fn update_input_port(
        &mut self,
        id: PortId,
        node: NodeId,
        requirement: InputRequirement,
    ) -> Result<(), ModelError> {
        #[cfg(feature = "core-integration")]
        if self.integrated.is_some() {
            return self.integrated_edit(structural::Edit::Port(structural::PortEdit::Input(
                id,
                Some(node),
                Some(requirement),
            )));
        }
        let old = self
            .interface
            .inputs
            .values()
            .find(|p| p.id == id)
            .ok_or(ModelError::Interface(InterfaceError::UnknownPort(id)))?;
        let name = old.name.clone();
        let target = self.nodes.get(&node).ok_or(ModelError::UnknownNode(node))?;
        if !matches!(target.definition, Definition::Input(_)) {
            return Err(ModelError::NotInput(node));
        }
        if self
            .interface
            .inputs
            .values()
            .any(|p| p.id != id && p.node == node)
        {
            return Err(ModelError::Interface(InterfaceError::DuplicateInputTarget(
                node,
            )));
        }
        if let InputRequirement::Optional { default } = &requirement
            && default.value_type() != pivot_types::ValueType::Decimal
        {
            return Err(ModelError::Interface(InterfaceError::TypeMismatch {
                name,
                expected: pivot_types::ValueType::Decimal,
                actual: default.value_type(),
            }));
        }
        self.advance()?;
        let port = self.interface.inputs.get_mut(&name).unwrap();
        port.node = node;
        port.requirement = requirement;
        Ok(())
    }
    pub fn update_output_port(&mut self, id: PortId, node: NodeId) -> Result<(), ModelError> {
        #[cfg(feature = "core-integration")]
        if self.integrated.is_some() {
            return self.integrated_edit(structural::Edit::Port(structural::PortEdit::Output(
                id, node,
            )));
        }
        let name = self
            .interface
            .outputs
            .values()
            .find(|p| p.id == id)
            .ok_or(ModelError::Interface(InterfaceError::UnknownPort(id)))?
            .name
            .clone();
        if !self.nodes.contains_key(&node) {
            return Err(ModelError::UnknownNode(node));
        }
        self.advance()?;
        self.interface.outputs.get_mut(&name).unwrap().node = node;
        Ok(())
    }
    /// Standalone default. With core-integration, shares the process convenience
    /// runtime; use new_in for application-owned budgets and deterministic shutdown.
    pub fn new(name: impl Into<String>) -> Result<Self, ModelError> {
        #[cfg(feature = "core-integration")]
        {
            integration::create_default(name)
        }
        #[cfg(not(feature = "core-integration"))]
        {
            Self::new_reference(name)
        }
    }
    /// Explicit migration fallback and downstream compatibility constructor.
    pub fn new_reference(name: impl Into<String>) -> Result<Self, ModelError> {
        let mut model = NEXT_MODEL.load(Ordering::Relaxed);
        loop {
            let next = model.checked_add(1).ok_or(ModelError::IdentityExhausted)?;
            match NEXT_MODEL.compare_exchange_weak(
                model,
                next,
                Ordering::Relaxed,
                Ordering::Relaxed,
            ) {
                Ok(_) => break,
                Err(current) => model = current,
            }
        }
        Ok(Self {
            #[cfg(feature = "core-integration")]
            integrated: None,
            name: name.into(),
            revision: Revision { model, version: 0 },
            nodes: BTreeMap::new(),
            names: BTreeMap::new(),
            next_node: 0,
            next_port: 0,
            interface: ModelInterface::default(),
        })
    }
    /// Observable backend selection, without exposing its representation.
    pub fn backend(&self) -> BackendKind {
        #[cfg(feature = "core-integration")]
        if self.integrated.is_some() {
            return BackendKind::Persistent;
        }
        BackendKind::Reference
    }
    pub fn name(&self) -> &str {
        #[cfg(feature = "core-integration")]
        if let Some(i) = &self.integrated {
            return i.graph().name();
        }
        &self.name
    }
    pub fn revision(&self) -> Revision {
        self.revision
    }
    pub fn node_id(&self, name: &str) -> Option<NodeId> {
        #[cfg(feature = "core-integration")]
        if let Some(i) = &self.integrated {
            return i.graph().node_id(name);
        }
        self.names.get(name).copied()
    }
    pub fn interface(&self) -> &ModelInterface {
        #[cfg(feature = "core-integration")]
        if let Some(i) = &self.integrated {
            return i.graph().interface();
        }
        &self.interface
    }
    fn port_name(&self, name: &str) -> Result<(), ModelError> {
        let mut bytes = name.bytes();
        if !bytes
            .next()
            .is_some_and(|b| b.is_ascii_alphabetic() || b == b'_')
            || !bytes.all(|b| b.is_ascii_alphanumeric() || b == b'_')
        {
            return Err(ModelError::Interface(InterfaceError::InvalidName(
                name.into(),
            )));
        }
        if self.interface.inputs.contains_key(name) || self.interface.outputs.contains_key(name) {
            return Err(ModelError::Interface(InterfaceError::DuplicateName(
                name.into(),
            )));
        }
        Ok(())
    }
    fn allocate_port(&mut self) -> Result<PortId, ModelError> {
        let sequence = self
            .next_port
            .checked_add(1)
            .ok_or(ModelError::IdentityExhausted)?;
        self.advance()?;
        self.next_port = sequence;
        Ok(PortId {
            model: self.revision.model,
            sequence,
        })
    }
    pub fn add_input_port(
        &mut self,
        name: impl Into<String>,
        node: NodeId,
        requirement: InputRequirement,
    ) -> Result<PortId, ModelError> {
        #[cfg(feature = "core-integration")]
        if self.integrated.is_some() {
            return self.integrated_add_port(structural::PortEdit::AddInput(
                name.into(),
                node,
                requirement,
            ));
        }
        let name = name.into();
        self.port_name(&name)?;
        let target = self.nodes.get(&node).ok_or(ModelError::UnknownNode(node))?;
        if !matches!(target.definition, Definition::Input(_)) {
            return Err(ModelError::NotInput(node));
        }
        if self.interface.inputs.values().any(|p| p.node == node) {
            return Err(ModelError::Interface(InterfaceError::DuplicateInputTarget(
                node,
            )));
        }
        if let InputRequirement::Optional { default } = &requirement
            && default.value_type() != pivot_types::ValueType::Decimal
        {
            return Err(ModelError::Interface(InterfaceError::TypeMismatch {
                name: name.clone(),
                expected: pivot_types::ValueType::Decimal,
                actual: default.value_type(),
            }));
        }
        let id = self.allocate_port()?;
        self.interface.inputs.insert(
            name.clone(),
            InputPort {
                id,
                name,
                node,
                requirement,
            },
        );
        Ok(id)
    }
    pub fn add_output_port(
        &mut self,
        name: impl Into<String>,
        node: NodeId,
    ) -> Result<PortId, ModelError> {
        #[cfg(feature = "core-integration")]
        if self.integrated.is_some() {
            return self.integrated_add_port(structural::PortEdit::AddOutput(name.into(), node));
        }
        let name = name.into();
        self.port_name(&name)?;
        if !self.nodes.contains_key(&node) {
            return Err(ModelError::UnknownNode(node));
        }
        let id = self.allocate_port()?;
        self.interface
            .outputs
            .insert(name.clone(), OutputPort { id, name, node });
        Ok(id)
    }
    pub fn rename_port(&mut self, id: PortId, name: impl Into<String>) -> Result<(), ModelError> {
        #[cfg(feature = "core-integration")]
        if self.integrated.is_some() {
            let name = name.into();
            let old = self
                .interface()
                .inputs()
                .map(|p| (p.id(), p.name()))
                .chain(self.interface().outputs().map(|p| (p.id(), p.name())))
                .find(|p| p.0 == id)
                .ok_or(ModelError::Interface(InterfaceError::UnknownPort(id)))?;
            if old.1 == name {
                return Ok(());
            }
            return self.integrated_edit(structural::Edit::Port(structural::PortEdit::Rename(
                id, name,
            )));
        }
        let old = self
            .interface
            .inputs
            .values()
            .find(|p| p.id == id)
            .map(|p| p.name.clone())
            .or_else(|| {
                self.interface
                    .outputs
                    .values()
                    .find(|p| p.id == id)
                    .map(|p| p.name.clone())
            })
            .ok_or(ModelError::Interface(InterfaceError::UnknownPort(id)))?;
        let name = name.into();
        if old == name {
            return Ok(());
        }
        self.port_name(&name)?;
        self.advance()?;
        if let Some(mut port) = self.interface.inputs.remove(&old) {
            port.name = name.clone();
            self.interface.inputs.insert(name, port);
        } else if let Some(mut port) = self.interface.outputs.remove(&old) {
            port.name = name.clone();
            self.interface.outputs.insert(name, port);
        }
        Ok(())
    }
    pub fn remove_port(&mut self, id: PortId) -> Result<(), ModelError> {
        #[cfg(feature = "core-integration")]
        if self.integrated.is_some() {
            return self.integrated_edit(structural::Edit::Port(structural::PortEdit::Remove(id)));
        }
        let name = self
            .interface
            .inputs
            .values()
            .find(|p| p.id == id)
            .map(|p| p.name.clone())
            .or_else(|| {
                self.interface
                    .outputs
                    .values()
                    .find(|p| p.id == id)
                    .map(|p| p.name.clone())
            })
            .ok_or(ModelError::Interface(InterfaceError::UnknownPort(id)))?;
        self.advance()?;
        self.interface.inputs.remove(&name);
        self.interface.outputs.remove(&name);
        Ok(())
    }
    fn advance(&mut self) -> Result<(), ModelError> {
        self.revision.version = self
            .revision
            .version
            .checked_add(1)
            .ok_or(ModelError::IdentityExhausted)?;
        Ok(())
    }
    fn validate_name(&self, name: &str) -> Result<(), ModelError> {
        let mut chars = name.bytes();
        if !chars
            .next()
            .is_some_and(|c| c.is_ascii_alphabetic() || c == b'_')
            || !chars.all(|c| c.is_ascii_alphanumeric() || c == b'_')
        {
            return Err(ModelError::InvalidName(name.into()));
        }
        if self.names.contains_key(name) {
            return Err(ModelError::DuplicateName(name.into()));
        }
        Ok(())
    }
    fn add(&mut self, name: String, definition: Definition) -> Result<NodeId, ModelError> {
        self.validate_name(&name)?;
        let next = self
            .next_node
            .checked_add(1)
            .ok_or(ModelError::IdentityExhausted)?;
        self.advance()?;
        let id = NodeId {
            model: self.revision.model,
            node: self.next_node,
        };
        self.next_node = next;
        self.names.insert(name.clone(), id);
        self.nodes.insert(id, WorkingNode { name, definition });
        Ok(id)
    }
    pub fn add_input(
        &mut self,
        name: impl Into<String>,
        value: PivotDecimal,
    ) -> Result<NodeId, ModelError> {
        let name = name.into();
        #[cfg(feature = "core-integration")]
        if self.integrated.is_some() {
            return self.integrated_add(name, Some(value), None);
        }
        self.add(name, Definition::Input(value))
    }
    pub fn add_computed(
        &mut self,
        name: impl Into<String>,
        source: impl Into<String>,
    ) -> Result<NodeId, ModelError> {
        let name = name.into();
        let source = source.into();
        #[cfg(feature = "core-integration")]
        if self.integrated.is_some() {
            return self.integrated_add(name, None, Some(source));
        }
        self.validate_name(&name)?;
        let id = NodeId {
            model: self.revision.model,
            node: self.next_node,
        };
        // Include self for self-reference binding, then use normal insertion.
        let mut directory = self.names.clone();
        directory.insert(name.clone(), id);
        let binding = expression::parse(&source)
            .map_err(|error| SnapshotError::Parse { node: id, error })
            .and_then(|expr| bind(id, &expr, &directory));
        self.add(name, Definition::Computed { source, binding })
    }

    pub fn set_input(&mut self, id: NodeId, value: PivotDecimal) -> Result<(), ModelError> {
        #[cfg(feature = "core-integration")]
        if self.integrated.is_some() {
            return self.integrated_edit(structural::Edit::Input(id, value));
        }
        let node = self.nodes.get(&id).ok_or(ModelError::UnknownNode(id))?;
        if !matches!(node.definition, Definition::Input(_)) {
            return Err(ModelError::NotInput(id));
        }
        self.advance()?;
        self.nodes.get_mut(&id).unwrap().definition = Definition::Input(value);
        Ok(())
    }
    /// Replace a definition, including converting an input to a computed node.
    pub fn set_expression(
        &mut self,
        id: NodeId,
        source: impl Into<String>,
    ) -> Result<(), ModelError> {
        #[cfg(feature = "core-integration")]
        if self.integrated.is_some() {
            return self.integrated_edit(structural::Edit::Formula(id, source.into()));
        }
        if !self.nodes.contains_key(&id) {
            return Err(ModelError::UnknownNode(id));
        }
        let source = source.into();
        self.advance()?;
        self.compile(id, source);
        Ok(())
    }
    fn compile(&mut self, id: NodeId, source: String) {
        let binding = expression::parse(&source)
            .map_err(|error| SnapshotError::Parse { node: id, error })
            .and_then(|expr| bind(id, &expr, &self.names));
        self.nodes.get_mut(&id).unwrap().definition = Definition::Computed { source, binding };
    }
    pub fn rename(&mut self, id: NodeId, name: impl Into<String>) -> Result<(), ModelError> {
        #[cfg(feature = "core-integration")]
        if self.integrated.is_some() {
            let name = name.into();
            let old = self
                .integrated
                .as_ref()
                .unwrap()
                .graph()
                .node_name(id)
                .ok_or(ModelError::UnknownNode(id))?;
            if old == name {
                return Ok(());
            }
            return self.integrated_edit(structural::Edit::Rename(id, name));
        }
        let old = self
            .nodes
            .get(&id)
            .ok_or(ModelError::UnknownNode(id))?
            .name
            .clone();
        let name = name.into();
        if old == name {
            return Ok(());
        }
        self.validate_name(&name)?;
        self.advance()?;
        self.names.remove(&old);
        self.names.insert(name.clone(), id);
        self.nodes.get_mut(&id).unwrap().name = name;
        Ok(())
    }
    pub fn remove(&mut self, id: NodeId) -> Result<(), ModelError> {
        #[cfg(feature = "core-integration")]
        if self.integrated.is_some() {
            return self.integrated_edit(structural::Edit::Retire(id));
        }
        if !self.nodes.contains_key(&id) {
            return Err(ModelError::UnknownNode(id));
        }
        self.advance()?;
        let node = self.nodes.remove(&id).unwrap();
        self.names.remove(&node.name);
        Ok(())
    }
    /// Captures definitions and inputs under one shared borrow. In safe Rust
    /// edits cannot overlap capture. After return, the working model is free
    /// to change while the owned snapshot is used elsewhere.
    pub fn snapshot(&self) -> Result<ModelSnapshot, SnapshotError> {
        #[cfg(feature = "core-integration")]
        if self.integrated.is_some() {
            return self.integrated_snapshot(&integration::CancellationToken::default());
        }
        self.snapshot_cancel_check(|| false)
    }
    fn snapshot_cancel_check(
        &self,
        _cancelled: impl Fn() -> bool,
    ) -> Result<ModelSnapshot, SnapshotError> {
        #[cfg(feature = "core-integration")]
        if _cancelled() {
            return Err(SnapshotError::Backend("cancelled"));
        }
        #[cfg(feature = "scaling-experiment")]
        let _phase = crate::experiment_profile::scope(6);
        let mut nodes = BTreeMap::new();
        for (&id, node) in &self.nodes {
            #[cfg(feature = "core-integration")]
            if _cancelled() {
                return Err(SnapshotError::Backend("cancelled"));
            }
            #[cfg(feature = "scaling-experiment")]
            crate::experiment_profile::copy(
                std::mem::size_of::<SnapshotNode>()
                    + node.name.len()
                    + match &node.definition {
                        Definition::Input(_) => 0,
                        Definition::Computed { source, binding } => {
                            source.len()
                                + binding
                                    .as_ref()
                                    .map(|(t, d)| {
                                        t.len() * std::mem::size_of::<BoundToken>()
                                            + d.len() * std::mem::size_of::<NodeId>()
                                    })
                                    .unwrap_or(0)
                        }
                    },
            );
            let (definition, dependencies) = match &node.definition {
                Definition::Input(value) => (SnapshotDefinition::Input(*value), Vec::new()),
                Definition::Computed { source, binding } => {
                    let (tokens, dependencies) = binding.clone()?;
                    for &target in &dependencies {
                        if !self.nodes.contains_key(&target) {
                            return Err(SnapshotError::MissingReference { node: id, target });
                        }
                    }
                    (
                        SnapshotDefinition::Computed {
                            source: source.clone(),
                            tokens,
                        },
                        dependencies,
                    )
                }
            };
            nodes.insert(
                id,
                SnapshotNode {
                    name: node.name.clone(),
                    definition,
                    dependencies,
                    dependents: Vec::new(),
                },
            );
        }
        #[cfg(feature = "scaling-experiment")]
        let order_phase = crate::experiment_profile::scope(7);
        for &id in self.nodes.keys() {
            #[cfg(feature = "core-integration")]
            if _cancelled() {
                return Err(SnapshotError::Backend("cancelled"));
            }
            let dependencies = nodes[&id].dependencies.clone();
            for dependency in dependencies {
                nodes.get_mut(&dependency).unwrap().dependents.push(id);
            }
        }
        let mut counts: BTreeMap<_, _> = nodes
            .iter()
            .map(|(&id, n)| (id, n.dependencies.len()))
            .collect();
        let mut ready: BTreeSet<_> = counts
            .iter()
            .filter_map(|(&id, &n)| (n == 0).then_some(id))
            .collect();
        let mut order = Vec::new();
        while let Some(id) = ready.pop_first() {
            #[cfg(feature = "core-integration")]
            if _cancelled() {
                return Err(SnapshotError::Backend("cancelled"));
            }
            order.push(id);
            for dependent in &nodes[&id].dependents {
                let count = counts.get_mut(dependent).unwrap();
                *count -= 1;
                if *count == 0 {
                    ready.insert(*dependent);
                }
            }
        }
        if order.len() != nodes.len() {
            return Err(SnapshotError::Unorderable {
                nodes: counts
                    .into_iter()
                    .filter_map(|(id, n)| (n != 0).then_some(id))
                    .collect(),
            });
        }
        #[cfg(feature = "scaling-experiment")]
        drop(order_phase);
        let snapshot = ModelSnapshot(Arc::new(SnapshotData {
            #[cfg(feature = "core-integration")]
            cleanup: None,
            name: self.name.clone(),
            revision: self.revision,
            nodes,
            names: self.names.clone(),
            order,
            interface: self.interface.clone(),
        }));
        snapshot
            .interface()
            .validate(&snapshot)
            .map_err(SnapshotError::Interface)?;
        Ok(snapshot)
    }
}

/// Bound postfix expression. Names have been replaced with concrete node IDs.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum BoundToken {
    Decimal(PivotDecimal),
    Node(NodeId),
    Unary(UnaryOp),
    Binary(BinaryOp),
}
#[derive(Debug)]
pub enum SnapshotDefinition {
    Input(PivotDecimal),
    Computed {
        source: String,
        tokens: Vec<BoundToken>,
    },
}
#[derive(Debug)]
pub struct SnapshotNode {
    name: String,
    definition: SnapshotDefinition,
    dependencies: Vec<NodeId>,
    dependents: Vec<NodeId>,
}
impl SnapshotNode {
    pub fn name(&self) -> &str {
        &self.name
    }
    pub fn definition(&self) -> &SnapshotDefinition {
        &self.definition
    }
    pub fn dependencies(&self) -> &[NodeId] {
        &self.dependencies
    }
    pub fn dependents(&self) -> &[NodeId] {
        &self.dependents
    }
}
#[derive(Debug)]
struct SnapshotData {
    #[cfg(feature = "core-integration")]
    cleanup: Option<integration::SnapshotLease>,
    interface: ModelInterface,
    name: String,
    revision: Revision,
    nodes: BTreeMap<NodeId, SnapshotNode>,
    names: BTreeMap<String, NodeId>,
    order: Vec<NodeId>,
}
#[cfg(feature = "core-integration")]
impl Drop for SnapshotData {
    fn drop(&mut self) {
        if let Some(lease) = self.cleanup.take() {
            lease.release((
                std::mem::take(&mut self.nodes),
                std::mem::take(&mut self.names),
                std::mem::take(&mut self.order),
                std::mem::take(&mut self.name),
                std::mem::take(&mut self.interface),
            ));
        }
    }
}
/// Immutable, owned snapshot; cheap to clone. Does not expose its storage layout.
#[derive(Debug, Clone)]
pub struct ModelSnapshot(Arc<SnapshotData>);
impl ModelSnapshot {
    pub fn interface(&self) -> &ModelInterface {
        &self.0.interface
    }
    pub fn name(&self) -> &str {
        &self.0.name
    }
    pub fn revision(&self) -> Revision {
        self.0.revision
    }
    pub fn node_id(&self, name: &str) -> Option<NodeId> {
        self.0.names.get(name).copied()
    }
    pub fn node(&self, id: NodeId) -> Option<&SnapshotNode> {
        self.0.nodes.get(&id)
    }
    pub fn dependency_order(&self) -> &[NodeId] {
        &self.0.order
    }
}

fn bind(
    id: NodeId,
    expr: &Expr,
    names: &BTreeMap<String, NodeId>,
) -> Result<(Vec<BoundToken>, Vec<NodeId>), SnapshotError> {
    bind_with(id, expr, |name| names.get(name).copied())
}
fn bind_with(
    id: NodeId,
    expr: &Expr,
    lookup: impl Fn(&str) -> Option<NodeId>,
) -> Result<(Vec<BoundToken>, Vec<NodeId>), SnapshotError> {
    let mut stack = vec![(expr, false)];
    let mut tokens = Vec::new();
    let mut dependencies = BTreeSet::new();
    while let Some((expr, visited)) = stack.pop() {
        match expr {
            Expr::Decimal(value) => tokens.push(BoundToken::Decimal(*value)),
            Expr::CellReference { span, .. } => {
                return Err(SnapshotError::Parse {
                    node: id,
                    error: ParseError {
                        offset: span.start,
                        message: "unexpected trailing input",
                    },
                });
            }
            Expr::Name(name) => {
                let target = lookup(name).ok_or_else(|| SnapshotError::UnknownName {
                    node: id,
                    name: name.clone(),
                })?;
                dependencies.insert(target);
                tokens.push(BoundToken::Node(target));
            }
            Expr::Unary { op, operand } => {
                if visited {
                    tokens.push(BoundToken::Unary(*op));
                } else {
                    stack.push((expr, true));
                    stack.push((operand, false));
                }
            }
            Expr::Binary { op, left, right } => {
                if visited {
                    tokens.push(BoundToken::Binary(*op));
                } else {
                    stack.push((expr, true));
                    stack.push((right, false));
                    stack.push((left, false));
                }
            }
        }
    }
    Ok((tokens, dependencies.into_iter().collect()))
}
