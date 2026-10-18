//! Editable definitions and validated immutable snapshots. No evaluation cache
//! lives in either type. Storage and synchronization are implementation details.
use crate::expression::{self, BinaryOp, Expr, ParseError, UnaryOp};
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
    Parse {
        node: NodeId,
        error: ParseError,
    },
    UnknownName {
        node: NodeId,
        name: String,
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
    Computed(String),
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
}
impl WorkingModel {
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
        self.add(name.into(), Definition::Computed(source.into()))
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
        self.nodes.get_mut(&id).unwrap().definition = Definition::Computed(source);
        Ok(())
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
                Definition::Computed(source) => {
                    let expr = expression::parse(source)
                        .map_err(|error| SnapshotError::Parse { node: id, error })?;
                    let (tokens, dependencies) = bind(id, &expr, &self.names)?;
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
        Ok(ModelSnapshot(Arc::new(SnapshotData {
            name: self.name.clone(),
            revision: self.revision,
            nodes,
            names: self.names.clone(),
            order,
        })))
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
