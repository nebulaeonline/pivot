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
#[derive(Debug, Clone)]
struct WorkingNode {
    name: String,
    definition: Definition,
}

/// Mutations require exclusive access. Invalid formulas may be saved while
/// editing; validation occurs at snapshot capture. Not implicitly cloneable.
#[derive(Debug)]
pub struct WorkingModel {
    name: String,
    revision: Revision,
    nodes: BTreeMap<NodeId, WorkingNode>,
    names: BTreeMap<String, NodeId>,
    next_node: u64,
    next_port: u64,
    interface: ModelInterface,
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
        Self {
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
        self.nodes.iter().map(|(&id, n)| {
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
        })
    }
    pub fn contains_node(&self, id: NodeId) -> bool {
        self.nodes.contains_key(&id)
    }
    /// Replace either definition kind without changing node identity. Existing
    /// input ports may become invalid drafts when converting to computed.
    pub fn replace_input(&mut self, id: NodeId, value: PivotDecimal) -> Result<(), ModelError> {
        if !self.nodes.contains_key(&id) {
            return Err(ModelError::UnknownNode(id));
        }
        self.advance()?;
        self.nodes.get_mut(&id).unwrap().definition = Definition::Input(value);
        Ok(())
    }
    /// Update an input port in place, preserving its core and public identity.
    pub fn update_input_port(
        &mut self,
        id: PortId,
        node: NodeId,
        requirement: InputRequirement,
    ) -> Result<(), ModelError> {
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
    pub fn new(name: impl Into<String>) -> Result<Self, ModelError> {
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
            name: name.into(),
            revision: Revision { model, version: 0 },
            nodes: BTreeMap::new(),
            names: BTreeMap::new(),
            next_node: 0,
            next_port: 0,
            interface: ModelInterface::default(),
        })
    }
    pub fn name(&self) -> &str {
        &self.name
    }
    pub fn revision(&self) -> Revision {
        self.revision
    }
    pub fn node_id(&self, name: &str) -> Option<NodeId> {
        self.names.get(name).copied()
    }
    pub fn interface(&self) -> &ModelInterface {
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
        self.add(name.into(), Definition::Input(value))
    }
    pub fn add_computed(
        &mut self,
        name: impl Into<String>,
        source: impl Into<String>,
    ) -> Result<NodeId, ModelError> {
        let name = name.into();
        let source = source.into();
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
        let mut nodes = BTreeMap::new();
        for (&id, node) in &self.nodes {
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
        for &id in self.nodes.keys() {
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
        let snapshot = ModelSnapshot(Arc::new(SnapshotData {
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
    interface: ModelInterface,
    name: String,
    revision: Revision,
    nodes: BTreeMap<NodeId, SnapshotNode>,
    names: BTreeMap<String, NodeId>,
    order: Vec<NodeId>,
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
    let mut stack = vec![(expr, false)];
    let mut tokens = Vec::new();
    let mut dependencies = BTreeSet::new();
    while let Some((expr, visited)) = stack.pop() {
        match expr {
            Expr::Decimal(value) => tokens.push(BoundToken::Decimal(*value)),
            Expr::Name(name) => {
                let target = *names.get(name).ok_or_else(|| SnapshotError::UnknownName {
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
