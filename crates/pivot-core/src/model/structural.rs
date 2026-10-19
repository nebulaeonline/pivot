//! Opt-in structural feasibility laboratory. Not a production model API.
//! Fixed-radix directory + 64-slot immutable pages; IDs never encode addresses.
use super::*;
use crate::evaluation::{EvaluationError, evaluate_expression};

mod compatibility;
mod index;
pub mod memory;
pub mod reclaim;
pub use compatibility::{Materialized, NamedCalculation, NamedError, Restored, benchmark_model};
use index::Index;
use memory::{Admission, Charge, Footprint, Memory, Shared};
impl From<Admission> for Error {
    fn from(_: Admission) -> Self {
        Self::Admission
    }
}
impl From<ModelError> for Error {
    fn from(e: ModelError) -> Self {
        Self::Model(e)
    }
}
impl Footprint for NodeId {}
impl Footprint for () {}
impl Footprint for String {
    fn heap_bytes(&self) -> usize {
        self.capacity()
    }
}
impl Footprint for SnapshotError {
    fn heap_bytes(&self) -> usize {
        match self {
            Self::UnknownName { name, .. } => name.capacity(),
            Self::Unorderable { nodes } => nodes.capacity() * std::mem::size_of::<NodeId>(),
            Self::Interface(e) => interface_bytes(e),
            _ => 0,
        }
    }
}
fn interface_bytes(e: &InterfaceError) -> usize {
    match e {
        InterfaceError::InvalidName(n)
        | InterfaceError::DuplicateName(n)
        | InterfaceError::UnknownInput(n)
        | InterfaceError::DuplicateInput(n)
        | InterfaceError::MissingRequired(n)
        | InterfaceError::UnknownOutput(n)
        | InterfaceError::DuplicateOutput(n) => n.capacity(),
        InterfaceError::TypeMismatch { name, .. } => name.capacity(),
        _ => 0,
    }
}
impl Footprint for ModelInterface {
    fn heap_bytes(&self) -> usize {
        usize::from(!self.inputs.is_empty()) * 2048
            + usize::from(!self.outputs.is_empty()) * 2048
            + (self.inputs.len() + self.outputs.len()) * 256
            + self
                .inputs
                .iter()
                .map(|(n, p)| n.capacity() + p.name.capacity())
                .sum::<usize>()
            + self
                .outputs
                .iter()
                .map(|(n, p)| n.capacity() + p.name.capacity())
                .sum::<usize>()
    }
}
fn error_node(e: &SnapshotError) -> Option<NodeId> {
    match e {
        SnapshotError::Parse { node, .. }
        | SnapshotError::UnknownName { node, .. }
        | SnapshotError::MissingReference { node, .. } => Some(*node),
        SnapshotError::Unorderable { nodes } => nodes.first().copied(),
        _ => None,
    }
}
const PAGE: usize = 64;
type Link<T> = Option<Shared<Tree<T>>>;
enum Tree<T> {
    Branch([Link<T>; 16]),
    Leaf(Shared<T>),
}
impl<T> Footprint for Tree<T> {}
struct Page<T>([Option<Shared<T>>; PAGE]);
impl<T> Footprint for Page<T> {}
pub struct Pages<T> {
    root: Link<Page<T>>,
    memory: Memory,
}
impl<T> Clone for Pages<T> {
    fn clone(&self) -> Self {
        Self {
            root: self.root.clone(),
            memory: self.memory.clone(),
        }
    }
}
impl<T> Default for Pages<T> {
    fn default() -> Self {
        Self::new(Memory::new(usize::MAX))
    }
}
fn lookup<T>(root: &Link<T>, key: u64) -> Option<&Shared<T>> {
    let mut current = root.as_ref()?;
    for shift in (0..16).rev() {
        match &**current {
            Tree::Branch(children) => {
                current = children[((key >> (shift * 4)) & 15) as usize].as_ref()?
            }
            _ => unreachable!(),
        }
    }
    match &**current {
        Tree::Leaf(v) => Some(v),
        _ => unreachable!(),
    }
}
fn insert<T>(
    memory: &Memory,
    root: &mut Link<T>,
    key: u64,
    depth: usize,
    value: Shared<T>,
) -> Result<(), Admission> {
    if depth == 16 {
        *root = Some(memory.alloc(Tree::Leaf(value))?);
        return Ok(());
    }
    if root.is_none() {
        *root = Some(memory.alloc(Tree::Branch(std::array::from_fn(|_| None)))?);
    }
    if root.as_mut().unwrap().get_mut().is_none() {
        let Tree::Branch(children) = &**root.as_ref().unwrap() else {
            unreachable!()
        };
        crate::experiment_profile::copy(std::mem::size_of::<Tree<T>>());
        *root = Some(memory.alloc(Tree::Branch(children.clone()))?);
    }
    let Tree::Branch(children) = root.as_mut().unwrap().get_mut().unwrap() else {
        unreachable!()
    };
    insert(
        memory,
        &mut children[((key >> ((15 - depth) * 4)) & 15) as usize],
        key,
        depth + 1,
        value,
    )
}
impl<T> Pages<T> {
    pub fn new(memory: Memory) -> Self {
        Self { root: None, memory }
    }
    pub fn get(&self, key: u64) -> Option<&Shared<T>> {
        lookup(&self.root, key / PAGE as u64)?.0[(key % PAGE as u64) as usize].as_ref()
    }
    pub fn iter(&self) -> impl Iterator<Item = (u64, &T)> {
        let mut stack: Vec<_> = self
            .root
            .as_ref()
            .map(|n| vec![(0u64, n)])
            .unwrap_or_default();
        let mut values = Vec::new().into_iter();
        std::iter::from_fn(move || {
            loop {
                if let Some(v) = values.next() {
                    return Some(v);
                }
                let (prefix, n) = stack.pop()?;
                match &**n {
                    Tree::Branch(children) => {
                        for (i, c) in children.iter().enumerate().rev() {
                            if let Some(c) = c {
                                stack.push(((prefix << 4) | i as u64, c));
                            }
                        }
                    }
                    Tree::Leaf(page) => {
                        values = page
                            .0
                            .iter()
                            .enumerate()
                            .filter_map(|(i, v)| {
                                v.as_ref().map(|v| (prefix * PAGE as u64 + i as u64, &**v))
                            })
                            .collect::<Vec<_>>()
                            .into_iter();
                    }
                }
            }
        })
    }
}
impl<T: Footprint> Pages<T> {
    pub fn set(&mut self, key: u64, value: T) -> Result<(), Admission> {
        let value = self.memory.alloc(value)?;
        let page_key = key / PAGE as u64;
        let mut page = lookup(&self.root, page_key)
            .map(|p| p.0.clone())
            .unwrap_or_else(|| std::array::from_fn(|_| None));
        crate::experiment_profile::copy(std::mem::size_of::<Page<T>>());
        page[(key % PAGE as u64) as usize] = Some(value);
        insert(
            &self.memory,
            &mut self.root,
            page_key,
            0,
            self.memory.alloc(Page(page))?,
        )
    }
}

#[derive(Clone)]
struct Record {
    live: bool,
    name: String,
    definition: Shared<Compiled>,
    reverse: Index<NodeId, ()>,
}
impl Footprint for Record {
    fn heap_bytes(&self) -> usize {
        self.name.capacity()
    }
}
enum Compiled {
    Input,
    Formula {
        source: String,
        binding: Result<(Vec<BoundToken>, Vec<NodeId>), SnapshotError>,
    },
}
impl Footprint for Compiled {
    fn heap_bytes(&self) -> usize {
        match self {
            Self::Input => 0,
            Self::Formula { source, binding } => {
                source.capacity()
                    + match binding {
                        Ok((t, d)) => {
                            t.capacity() * std::mem::size_of::<BoundToken>()
                                + d.capacity() * std::mem::size_of::<NodeId>()
                        }
                        Err(e) => e.heap_bytes(),
                    }
            }
        }
    }
}
#[derive(Clone)]
pub enum PortEdit {
    AddInput(String, NodeId, InputRequirement),
    AddOutput(String, NodeId),
    Rename(PortId, String),
    Remove(PortId),
    Input(PortId, Option<NodeId>, Option<InputRequirement>),
    Output(PortId, NodeId),
}
impl Compiled {
    fn deps(&self) -> &[NodeId] {
        match self {
            Self::Formula {
                binding: Ok((_, deps)),
                ..
            } => deps,
            _ => &[],
        }
    }
    fn bytes(&self) -> usize {
        match self {
            Self::Input => 0,
            Self::Formula { source, binding } => {
                source.len()
                    + binding.as_ref().map_or(0, |(t, d)| {
                        t.len() * std::mem::size_of::<BoundToken>()
                            + d.len() * std::mem::size_of::<NodeId>()
                    })
            }
        }
    }
}
/// One coherent version; topology and input counters have different invalidation roles.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Version {
    pub model: u64,
    pub commit: u64,
    pub topology: u64,
    pub inputs: u64,
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Error {
    Stale,
    Unknown(NodeId),
    NotInput(NodeId),
    InvalidTransfer,
    Quota,
    ValidationBudget,
    Exhausted,
    Admission,
    Model(ModelError),
}
#[derive(Clone, Copy)]
pub struct Limits {
    pub nodes: usize,
    pub definition_bytes: usize,
    pub validation_visits: usize,
}
impl Default for Limits {
    fn default() -> Self {
        Self {
            nodes: 1_002_048,
            definition_bytes: 512 * 1024 * 1024,
            validation_visits: 2_000_000,
        }
    }
}
/// Typed references are stable IDs, including allocated retired IDs. No name rebinding.
pub enum Edit {
    Input(NodeId, PivotDecimal),
    ReplaceInput(NodeId, PivotDecimal),
    Formula(NodeId, String),
    Resolved(NodeId, String, Vec<transfer::ResolvedToken>),
    Invalid(NodeId, String, ParseError),
    Retire(NodeId),
    AddFormula(String, String),
    AddInput(String, PivotDecimal),
    Rename(NodeId, String),
    Port(PortEdit),
}
#[derive(Clone)]
pub struct Graph {
    records: Pages<Record>,
    values: Pages<PivotDecimal>,
    names: Index<String, NodeId>,
    invalid: Index<NodeId, Shared<SnapshotError>>,
    version: Version,
    next: u64,
    live: usize,
    bytes: usize,
    limits: Limits,
    memory: Memory,
    model_name: Shared<String>,
    ports: Shared<ModelInterface>,
    next_port: u64,
    revision_origin: u64,
    import_invalid: bool,
    max_name: usize,
}
impl Graph {
    pub fn from_model(model: &WorkingModel, limits: Limits) -> Result<Self, Error> {
        Self::from_model_with_budget(model, limits, 2 * 1024 * 1024 * 1024)
    }
    pub fn from_model_with_budget(
        model: &WorkingModel,
        limits: Limits,
        budget: usize,
    ) -> Result<Self, Error> {
        Self::from_model_in(model, limits, Memory::new(budget))
    }
    pub(super) fn from_model_in(
        model: &WorkingModel,
        limits: Limits,
        memory: Memory,
    ) -> Result<Self, Error> {
        let payload = model
            .nodes
            .values()
            .map(|node| match &node.definition {
                Definition::Input(_) => 0,
                Definition::Computed { source, binding } => {
                    source.capacity()
                        + binding.as_ref().map_or(0, |(t, d)| {
                            t.capacity() * std::mem::size_of::<BoundToken>()
                                + d.capacity() * std::mem::size_of::<NodeId>()
                        })
                }
            })
            .sum::<usize>();
        if model.next_node as usize > limits.nodes || payload > limits.definition_bytes {
            return Err(Error::Quota);
        }
        let names = model.names.keys().map(String::len).sum::<usize>();
        let _import_scratch = memory.reserve(
            model
                .nodes
                .len()
                .saturating_mul(1024)
                .saturating_add(payload.saturating_mul(4))
                .saturating_add(names.saturating_mul(4))
                .saturating_add(model.interface.heap_bytes().saturating_mul(4)),
        )?;
        let initial_error = model.snapshot().err();
        let mut graph = Self {
            records: Pages::new(memory.clone()),
            values: Pages::new(memory.clone()),
            names: Index::new(memory.clone()),
            invalid: Index::new(memory.clone()),
            version: Version {
                model: model.revision.model,
                commit: 0,
                topology: 0,
                inputs: 0,
            },
            next: model.next_node,
            live: model.nodes.len(),
            bytes: 0,
            limits,
            model_name: memory.alloc(model.name.clone())?,
            ports: memory.alloc(model.interface.clone())?,
            next_port: model.next_port,
            revision_origin: model.revision.version,
            import_invalid: initial_error.is_some(),
            memory: memory.clone(),
            max_name: model.names.keys().map(String::len).max().unwrap_or(0),
        };
        for (&id, node) in &model.nodes {
            let definition = match &node.definition {
                Definition::Input(v) => {
                    graph.values.set(id.node, *v)?;
                    Compiled::Input
                }
                Definition::Computed { source, binding } => Compiled::Formula {
                    source: source.clone(),
                    binding: binding.clone(),
                },
            };
            graph.bytes += definition.bytes();
            graph.names.insert(node.name.clone(), id)?;
            graph.records.set(
                id.node,
                Record {
                    live: true,
                    name: node.name.clone(),
                    definition: memory.alloc(definition)?,
                    reverse: Index::new(memory.clone()),
                },
            )?;
        }
        for (&id, node) in &model.nodes {
            if let Definition::Computed {
                binding: Ok((_, deps)),
                ..
            } = &node.definition
            {
                for &target in deps {
                    let mut record =
                        graph
                            .record(target)
                            .map(|r| (**r).clone())
                            .unwrap_or(Record {
                                live: false,
                                name: String::new(),
                                definition: memory.alloc(Compiled::Input)?,
                                reverse: Index::new(memory.clone()),
                            });
                    record.reverse.insert(id, ())?;
                    graph.records.set(target.node, record)?;
                }
            }
        }
        if let Some(error) = initial_error {
            let id = error_node(&error).unwrap_or(NodeId {
                model: model.revision.model,
                node: 0,
            });
            graph.invalid.insert(id, graph.memory.alloc(error)?)?;
        }
        graph.check_quota()?;
        Ok(graph)
    }
    pub fn memory(&self) -> Memory {
        self.memory.clone()
    }
    pub fn version(&self) -> Version {
        self.version
    }
    pub fn node_id(&self, name: &str) -> Option<NodeId> {
        self.names.get(name).copied()
    }
    fn record(&self, id: NodeId) -> Option<&Shared<Record>> {
        (id.model == self.version.model)
            .then(|| self.records.get(id.node))
            .flatten()
    }
    fn live_record(&self, id: NodeId) -> Result<&Shared<Record>, Error> {
        self.record(id).filter(|r| r.live).ok_or(Error::Unknown(id))
    }
    pub fn dependencies(&self, id: NodeId) -> Result<&[NodeId], Error> {
        Ok(self.live_record(id)?.definition.deps())
    }
    pub fn dependents(&self, id: NodeId) -> Result<BTreeSet<NodeId>, Error> {
        Ok(self
            .live_record(id)?
            .reverse
            .iter()
            .map(|(n, _)| *n)
            .collect())
    }
    pub fn dependent_count(&self, id: NodeId) -> Result<usize, Error> {
        Ok(self.live_record(id)?.reverse.len())
    }
    pub fn has_drafts(&self) -> bool {
        !self.invalid.is_empty()
    }
    pub fn same_definition(&self, other: &Self, id: NodeId) -> bool {
        match (self.record(id), other.record(id)) {
            (Some(a), Some(b)) => Shared::ptr_eq(&a.definition, &b.definition),
            _ => false,
        }
    }
    fn check_quota(&self) -> Result<(), Error> {
        // Allocated identities (including tombstones) are deliberately bounded.
        if self.next as usize > self.limits.nodes || self.bytes > self.limits.definition_bytes {
            Err(Error::Quota)
        } else {
            Ok(())
        }
    }
    fn compile(&self, id: NodeId, source: String) -> Compiled {
        let _phase = crate::experiment_profile::scope(5);
        let binding = expression::parse(&source)
            .map_err(|error| SnapshotError::Parse { node: id, error })
            .and_then(|expr| bind_with(id, &expr, |name| self.names.get(name).copied()));
        Compiled::Formula { source, binding }
    }
    fn replace(&mut self, id: NodeId, definition: Compiled) -> Result<(), Error> {
        let _phase = crate::experiment_profile::scope(4);
        let old = self.live_record(id)?.clone();
        self.bytes = self.bytes - old.definition.bytes() + definition.bytes();
        let before: BTreeSet<_> = old.definition.deps().iter().copied().collect();
        let after: BTreeSet<_> = definition.deps().iter().copied().collect();
        for target in before.symmetric_difference(&after) {
            if let Some(record) = self.record(*target) {
                let mut record = (**record).clone();
                if after.contains(target) {
                    record.reverse.insert(id, ())?;
                } else {
                    record.reverse.remove(&id)?;
                }
                self.records.set(target.node, record)?;
            }
        }
        // Fetch again: self-reference may have updated this record's reverse index.
        let mut record = (**self.live_record(id)?).clone();
        record.definition = self.memory.alloc(definition)?;
        self.records.set(id.node, record)?;
        Ok(())
    }
    /// Private branch, incremental indexes, bounded validation, single atomic installation.
    /// Invalid source/missing targets/cycles commit as editable drafts; hard failures install nothing.
    pub fn apply(&mut self, expected: Version, edits: Vec<Edit>) -> Result<Version, Error> {
        if expected != self.version {
            return Err(Error::Stale);
        }
        if edits.is_empty() {
            return Ok(self.version);
        }
        let source_bytes = edits
            .iter()
            .map(|e| match e {
                Edit::Formula(_, s)
                | Edit::Invalid(_, s, _)
                | Edit::Resolved(_, s, _)
                | Edit::Rename(_, s) => s.len(),
                Edit::AddFormula(n, s) => n.len() + s.len(),
                Edit::AddInput(n, _) => n.len(),
                Edit::Port(
                    PortEdit::AddInput(n, _, _)
                    | PortEdit::AddOutput(n, _)
                    | PortEdit::Rename(_, n),
                ) => n.len(),
                Edit::Port(_) => 256,
                _ => 0,
            })
            .sum::<usize>();
        let mut transfer_bytes = 0usize;
        let mut structural_work = self
            .invalid
            .len()
            .saturating_mul(128)
            .saturating_add(edits.len().saturating_mul(256));
        for edit in &edits {
            let target = match edit {
                Edit::Formula(id, _)
                | Edit::Resolved(id, _, _)
                | Edit::Invalid(id, _, _)
                | Edit::ReplaceInput(id, _)
                | Edit::Retire(id) => Some(*id),
                _ => None,
            };
            if let Some(record) = target.and_then(|id| self.record(id)) {
                structural_work = structural_work
                    .saturating_add(record.definition.deps().len().saturating_mul(128));
                if matches!(edit, Edit::Retire(_)) {
                    structural_work =
                        structural_work.saturating_add(record.reverse.len().saturating_mul(128));
                }
            }
            if let Edit::Resolved(_, _, tokens) = edit {
                if tokens.len() > 1_048_576 {
                    return Err(Error::InvalidTransfer);
                }
                transfer_bytes = transfer_bytes.saturating_add(tokens.len().saturating_mul(128));
            }
        }
        let _scratch = self.memory.reserve(
            8192usize
                .saturating_add(self.max_name.saturating_mul(256))
                .saturating_add(source_bytes.saturating_mul(512))
                .saturating_add(transfer_bytes)
                .saturating_add(structural_work),
        )?;
        let mut candidate = {
            let _phase = crate::experiment_profile::scope(1);
            self.clone()
        };
        let mut changed = BTreeSet::new();
        let mut topology = false;
        let mut inputs = false;
        for edit in edits {
            match edit {
                Edit::ReplaceInput(id, value) => {
                    if !matches!(*candidate.live_record(id)?.definition, Compiled::Input) {
                        candidate.replace(id, Compiled::Input)?;
                        changed.insert(id);
                        topology = true;
                    }
                    candidate.values.set(id.node, value)?;
                    inputs = true;
                }
                Edit::Input(id, value) => {
                    if !matches!(*candidate.live_record(id)?.definition, Compiled::Input) {
                        return Err(Error::NotInput(id));
                    }
                    candidate.values.set(id.node, value)?;
                    inputs = true;
                }
                Edit::Formula(id, source) => {
                    let definition = candidate.compile(id, source);
                    candidate.replace(id, definition)?;
                    changed.insert(id);
                    topology = true;
                }
                Edit::Invalid(id, source, error) => {
                    if source.len() > 4096
                        || error.offset > source.len()
                        || !source.is_char_boundary(error.offset)
                    {
                        return Err(Error::InvalidTransfer);
                    }
                    candidate.replace(
                        id,
                        Compiled::Formula {
                            source,
                            binding: Err(SnapshotError::Parse { node: id, error }),
                        },
                    )?;
                    changed.insert(id);
                    topology = true;
                }
                Edit::Resolved(id, source, tokens) => {
                    if source.len() > 4096 {
                        return Err(Error::InvalidTransfer);
                    }
                    let mut bound = Vec::with_capacity(tokens.len());
                    for token in tokens {
                        bound.push(match token {
                            transfer::ResolvedToken::Decimal(v) => BoundToken::Decimal(v),
                            transfer::ResolvedToken::Node(n) => {
                                if n.model != candidate.version.model || n.node >= candidate.next {
                                    return Err(Error::InvalidTransfer);
                                }
                                BoundToken::Node(n)
                            }
                            transfer::ResolvedToken::Unary(op) => BoundToken::Unary(op),
                            transfer::ResolvedToken::Binary(op) => BoundToken::Binary(op),
                        });
                    }
                    let deps = transfer::checked_dependencies(&bound)
                        .map_err(|_| Error::InvalidTransfer)?;
                    candidate.replace(
                        id,
                        Compiled::Formula {
                            source,
                            binding: Ok((bound, deps)),
                        },
                    )?;
                    changed.insert(id);
                    topology = true;
                }
                Edit::Retire(id) => {
                    let record = candidate.live_record(id)?.clone();
                    // Preserve tombstone reverse links so historical references never rebind.
                    for target in record.definition.deps() {
                        if let Some(r) = candidate.record(*target) {
                            let mut r = (**r).clone();
                            r.reverse.remove(&id)?;
                            candidate.records.set(target.node, r)?;
                        }
                    }
                    changed.extend(record.reverse.iter().map(|(n, _)| *n));
                    let mut retired = (*record).clone();
                    retired.live = false;
                    candidate.records.set(id.node, retired)?;
                    candidate.names.remove(&record.name)?;
                    candidate.invalid.remove(&id)?;
                    candidate.live -= 1;
                    topology = true;
                }
                Edit::AddFormula(name, source) => {
                    let id = candidate.add_input(name, PivotDecimal::ZERO)?;
                    let definition = candidate.compile(id, source);
                    candidate.replace(id, definition)?;
                    changed.insert(id);
                    topology = true;
                }
                Edit::AddInput(name, value) => {
                    candidate.add_input(name, value)?;
                    topology = true;
                    inputs = true;
                }
                Edit::Rename(id, name) => {
                    let old = candidate.live_record(id)?.clone();
                    if old.name == name {
                        continue;
                    }
                    candidate.check_name(&name)?;
                    candidate.max_name = candidate.max_name.max(name.len());
                    candidate.names.remove(&old.name)?;
                    candidate.names.insert(name.clone(), id)?;
                    let mut record = (*old).clone();
                    record.name = name;
                    candidate.records.set(id.node, record)?;
                    topology = true;
                }
                Edit::Port(edit) => {
                    topology |= candidate.edit_port(edit)?;
                }
            }
        }
        candidate.check_quota()?;
        if candidate.import_invalid && topology {
            let restored = candidate.to_model()?;
            candidate.invalid = Index::new(candidate.memory.clone());
            if let Err(error) = restored.snapshot() {
                let id = error_node(&error).unwrap_or(NodeId {
                    model: candidate.version.model,
                    node: 0,
                });
                candidate
                    .invalid
                    .insert(id, candidate.memory.alloc(error)?)?;
            } else {
                candidate.import_invalid = false;
            }
        } else if topology {
            let _phase = crate::experiment_profile::scope(7);
            changed.extend(candidate.invalid.iter().map(|(id, _)| *id));
            let mut budget = candidate.limits.validation_visits;
            for id in changed {
                if candidate.live_record(id).is_err() {
                    continue;
                }
                let error = candidate.validate_from(id, &mut budget)?;
                if candidate.invalid.get(&id).map(|e| &**e) != error.as_ref() {
                    if let Some(error) = error {
                        candidate
                            .invalid
                            .insert(id, candidate.memory.alloc(error)?)?;
                    } else {
                        candidate.invalid.remove(&id)?;
                    }
                }
            }
        }
        if !topology && !inputs {
            return Ok(self.version);
        }
        candidate.version.commit = candidate
            .version
            .commit
            .checked_add(1)
            .ok_or(Error::Exhausted)?;
        candidate.version.topology += u64::from(topology);
        candidate.version.inputs += u64::from(inputs);
        {
            let _phase = crate::experiment_profile::scope(10);
            *self = candidate;
        }
        Ok(self.version)
    }
    fn validate_from(
        &self,
        start: NodeId,
        budget: &mut usize,
    ) -> Result<Option<SnapshotError>, Error> {
        let mut work = Work::new(self.memory.clone());
        let mut active = BTreeSet::new();
        let mut done = BTreeSet::new();
        let mut pending = vec![(start, false)];
        while let Some((id, exit)) = pending.pop() {
            *budget = budget.checked_sub(1).ok_or(Error::ValidationBudget)?;
            if exit {
                active.remove(&id);
                done.insert(id);
                continue;
            }
            if done.contains(&id) {
                continue;
            }
            work.add(512)?;
            if !active.insert(id) {
                return Ok(Some(SnapshotError::Unorderable {
                    nodes: active.into_iter().collect(),
                }));
            }
            let record = self.live_record(id).map_err(|_| Error::Unknown(id))?;
            if let Compiled::Formula {
                binding: Err(error),
                ..
            } = &*record.definition
            {
                return Ok(Some(error.clone()));
            }
            work.add(record.definition.deps().len().saturating_mul(64))?;
            pending.push((id, true));
            for &target in record.definition.deps() {
                if self.live_record(target).is_err() {
                    return Ok(Some(SnapshotError::MissingReference { node: id, target }));
                }
                pending.push((target, false));
            }
        }
        Ok(None)
    }
    pub fn capture(&self) -> Result<Capture, CaptureError> {
        let _phase = crate::experiment_profile::scope(6);
        if let Some(error) = self.invalid.iter().next().map(|(_, e)| e) {
            return Err(CaptureError::Invalid((**error).clone()));
        }
        self.validate_ports().map_err(CaptureError::Invalid)?;
        Ok(Capture(
            self.memory
                .alloc(self.clone())
                .map_err(|_| CaptureError::Admission)?,
        ))
    }
}
#[derive(Clone)]
pub struct Capture(Shared<Graph>);
impl Footprint for Graph {}
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum CaptureError {
    Admission,
    Invalid(SnapshotError),
}
impl Capture {
    pub fn version(&self) -> Version {
        self.0.version
    }
    pub fn evaluate(&self, outputs: &[NodeId]) -> Result<Calculation, ComputeError> {
        let _request = self.0.memory.reserve(
            outputs
                .len()
                .saturating_mul(std::mem::size_of::<NodeId>())
                .saturating_add(128),
        )?;
        self.invoke(&crate::evaluation::InvocationRequest {
            inputs: Vec::new(),
            outputs: outputs.to_vec(),
        })
    }
    pub fn invoke(
        &self,
        request: &crate::evaluation::InvocationRequest,
    ) -> Result<Calculation, ComputeError> {
        let _phase = crate::experiment_profile::scope(8);
        let mut work = Work::new(self.0.memory.clone());
        work.add(4096 + request.inputs.len() * 512 + request.outputs.len() * 128)?;
        let inputs = crate::evaluation::prepare_inputs(request, |id| {
            self.0
                .record(id)
                .filter(|r| r.live)
                .map(|r| matches!(*r.definition, Compiled::Input))
        })?;
        let mut required = BTreeSet::new();
        let mut order = Vec::new();
        let mut pending: Vec<_> = request.outputs.iter().map(|&n| (n, false)).collect();
        let mut visits = self.0.limits.validation_visits;
        while let Some((id, exit)) = pending.pop() {
            visits = visits.checked_sub(1).ok_or(ComputeError::WorkBudget)?;
            let record = self.0.live_record(id).map_err(|_| {
                crate::evaluation::InvocationError::Evaluation(EvaluationError::UnknownNode(id))
            })?;
            if exit {
                order.push(id);
            } else {
                work.add(512 + record.definition.deps().len() * 64)?;
                if required.insert(id) {
                    pending.push((id, true));
                    pending.extend(record.definition.deps().iter().map(|&d| (d, false)));
                }
            }
        }
        let mut values = BTreeMap::new();
        for id in order {
            let record = self.0.live_record(id).unwrap();
            let value = match &*record.definition {
                Compiled::Input => Ok(inputs
                    .get(&id)
                    .copied()
                    .unwrap_or(**self.0.values.get(id.node).unwrap())),
                Compiled::Formula {
                    binding: Ok((tokens, _)),
                    ..
                } => {
                    let _stack = self.0.memory.reserve(tokens.len().saturating_mul(64))?;
                    evaluate_expression(id, tokens, &values)
                }
                _ => unreachable!("capture rejects invalid drafts"),
            };
            values.insert(id, value);
        }
        Ok(Calculation {
            version: self.version(),
            values,
            input_overrides: inputs,
            _capture: self.clone(),
            _work: work,
        })
    }
}
pub struct Calculation {
    version: Version,
    values: BTreeMap<NodeId, Result<PivotDecimal, EvaluationError>>,
    input_overrides: BTreeMap<NodeId, PivotDecimal>,
    _capture: Capture,
    _work: Work,
}
impl Calculation {
    pub fn version(&self) -> Version {
        self.version
    }
    pub fn values(&self) -> &BTreeMap<NodeId, Result<PivotDecimal, EvaluationError>> {
        &self.values
    }
    pub fn input_overrides(&self) -> &BTreeMap<NodeId, PivotDecimal> {
        &self.input_overrides
    }
}
impl std::ops::Deref for Calculation {
    type Target = BTreeMap<NodeId, Result<PivotDecimal, EvaluationError>>;
    fn deref(&self) -> &Self::Target {
        &self.values
    }
}
#[derive(Debug, PartialEq, Eq)]
pub enum ComputeError {
    Admission,
    WorkBudget,
    Invocation(crate::evaluation::InvocationError),
}
impl From<Admission> for ComputeError {
    fn from(_: Admission) -> Self {
        Self::Admission
    }
}
impl From<crate::evaluation::InvocationError> for ComputeError {
    fn from(e: crate::evaluation::InvocationError) -> Self {
        Self::Invocation(e)
    }
}
struct Work {
    memory: Memory,
    charges: Vec<Charge>,
}
impl Work {
    fn new(memory: Memory) -> Self {
        Self {
            memory,
            charges: Vec::new(),
        }
    }
    fn add(&mut self, bytes: usize) -> Result<(), Admission> {
        self.charges.push(
            self.memory
                .reserve(bytes.saturating_add(2 * std::mem::size_of::<Charge>()))?,
        );
        Ok(())
    }
}
