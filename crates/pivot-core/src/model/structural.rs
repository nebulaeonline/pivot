//! Canonical persistent graph; the independent research API is feature gated.
//! Fixed-radix directory + 64-slot immutable pages; IDs never encode addresses.
use super::*;
use crate::evaluation::EvaluationError;

mod index;
mod inspection;
mod materialization;
pub mod memory;
pub mod reclaim;
pub use index::{Index, Key};
pub use materialization::{MaterializedImage, NamedCalculation, NamedError, Restored};
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
impl Footprint for Vec<NodeId> {
    fn heap_bytes(&self) -> usize {
        self.capacity() * std::mem::size_of::<NodeId>()
    }
}
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
        self.iter_from(0)
    }
    pub fn iter_from(&self, start: u64) -> impl Iterator<Item = (u64, &T)> {
        let start_page = start / PAGE as u64;
        let mut stack: Vec<_> = self
            .root
            .as_ref()
            .map(|n| vec![(0u64, 0usize, n)])
            .unwrap_or_default();
        let mut values = Vec::new().into_iter();
        std::iter::from_fn(move || {
            loop {
                if let Some(value) = values.next() {
                    return Some(value);
                }
                let (prefix, depth, node) = stack.pop()?;
                match &**node {
                    Tree::Branch(children) => {
                        for (i, child) in children.iter().enumerate().rev() {
                            let next = (prefix << 4) | i as u64;
                            if next < (start_page >> ((15 - depth) * 4)) {
                                continue;
                            }
                            if let Some(child) = child {
                                stack.push((next, depth + 1, child));
                            }
                        }
                    }
                    Tree::Leaf(page) => {
                        values = page
                            .0
                            .iter()
                            .enumerate()
                            .filter_map(|(i, value)| {
                                let key = prefix * PAGE as u64 + i as u64;
                                if key < start {
                                    None
                                } else {
                                    value.as_ref().map(|v| (key, &**v))
                                }
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
        let _profile = crate::experiment_profile::construction::scope(13);
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
    Blank,
    NonNumeric,
    CapturedScalar(crate::captured::CapturedScalarId),
    Input,
    Formula {
        source: String,
        binding: Result<(Vec<BoundToken>, Vec<NodeId>), SnapshotError>,
    },
}
impl Footprint for Compiled {
    fn heap_bytes(&self) -> usize {
        match self {
            Self::Blank | Self::Input | Self::NonNumeric | Self::CapturedScalar(_) => 0,
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
    fn selection_ids(&self) -> BTreeSet<selection::SelectionId> {
        match self {
            Self::Formula {
                binding: Ok((tokens, _)),
                ..
            } => tokens
                .iter()
                .filter_map(|t| {
                    if let BoundToken::Sum(id) | BoundToken::Aggregate(_, id) = t {
                        Some(*id)
                    } else {
                        None
                    }
                })
                .collect(),
            _ => BTreeSet::new(),
        }
    }
    fn bytes(&self) -> usize {
        match self {
            Self::Blank | Self::Input | Self::NonNumeric | Self::CapturedScalar(_) => 0,
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
    Cancelled,
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
    RetiredSelection(selection::SelectionId),
    Selection(selection::SelectionId, selection::Rectangle),
    Placement(selection::SpaceId, u32, u32, Option<NodeId>),
    Input(NodeId, PivotDecimal),
    ReplaceInput(NodeId, PivotDecimal),
    Blank(NodeId),
    NonNumeric(NodeId),
    CapturedScalar(NodeId, crate::captured::CapturedScalarId),
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
    captured_scalar: Option<(NodeId, crate::captured::CapturedScalarId)>,
    pub(super) selections: Option<Shared<selection::State>>,
    selection_invalidated: Option<Shared<Vec<NodeId>>>,
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
    local_validation: bool,
    proof: Shared<Proof>,
}
// A bulk private root always has a reserved reclamation destination, including
// early errors and cooperative cancellation during construction/validation.
struct CandidateRoot {
    value: Option<Graph>,
    retire: Option<Box<dyn FnOnce(Graph) + Send>>,
}
impl CandidateRoot {
    fn take(mut self) -> Graph {
        self.value.take().unwrap()
    }
}
impl Drop for CandidateRoot {
    fn drop(&mut self) {
        if let Some(graph) = self.value.take() {
            match self.retire.take() {
                Some(retire) => retire(graph),
                None => drop(graph),
            }
        }
    }
}
struct DependencyScratch<'a> {
    items: std::borrow::Cow<'a, [NodeId]>,
    _charge: Option<Charge>,
}
impl Footprint for selection::State {}
struct Proof(std::sync::atomic::AtomicBool);
impl Footprint for Proof {}
impl Graph {
    /// Empty persistent root, with no legacy import or global snapshot bridge.
    pub(super) fn empty(name: String, revision: Revision, memory: Memory) -> Result<Self, Error> {
        Ok(Self {
            local_validation: true,
            captured_scalar: None,
            proof: memory.alloc(Proof(std::sync::atomic::AtomicBool::new(true)))?,
            selections: None,
            selection_invalidated: None,
            records: Pages::new(memory.clone()),
            values: Pages::new(memory.clone()),
            names: Index::new(memory.clone()),
            invalid: Index::new(memory.clone()),
            version: Version {
                model: revision.model,
                commit: 0,
                topology: 0,
                inputs: 0,
            },
            next: 0,
            live: 0,
            bytes: 0,
            limits: Limits::default(),
            model_name: memory.alloc(name)?,
            ports: memory.alloc(ModelInterface::default())?,
            next_port: 0,
            revision_origin: revision.version,
            import_invalid: false,
            memory,
            max_name: 0,
        })
    }
    pub(super) fn live_count(&self) -> usize {
        self.live
    }
    pub(super) fn node_name(&self, id: NodeId) -> Option<&str> {
        self.record(id).filter(|r| r.live).map(|r| r.name.as_str())
    }
    pub(super) fn next_port_identity(&self) -> Result<PortId, ModelError> {
        Ok(PortId {
            model: self.version.model,
            sequence: self
                .next_port
                .checked_add(1)
                .ok_or(ModelError::IdentityExhausted)?,
        })
    }
    pub(super) fn enable_local_validation(&mut self) {
        self.local_validation = true;
    }
    pub(super) fn certified(&self) -> bool {
        self.proof.0.load(Ordering::Acquire)
    }
    pub(super) fn certify(&self) {
        self.proof.0.store(true, Ordering::Release);
    }
    pub fn from_model(model: &WorkingModel, limits: Limits) -> Result<Self, Error> {
        let mut graph = model.integrated.graph().clone();
        graph.limits = limits;
        graph.check_quota()?;
        Ok(graph)
    }
    pub fn from_model_with_budget(
        model: &WorkingModel,
        limits: Limits,
        budget: usize,
    ) -> Result<Self, Error> {
        let image = model.integrated.graph().materialize()?;
        Self::from_image_in(&image, limits, Memory::new(budget))
    }
    pub(super) fn from_image_in(
        model: &GraphImage,
        limits: Limits,
        memory: Memory,
    ) -> Result<Self, Error> {
        let payload = model
            .nodes
            .values()
            .map(|node| match &node.definition {
                Definition::Blank
                | Definition::NonNumeric
                | Definition::CapturedScalar(_)
                | Definition::Input(_) => 0,
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
            local_validation: false,
            captured_scalar: None,
            proof: memory.alloc(Proof(std::sync::atomic::AtomicBool::new(
                initial_error.is_none(),
            )))?,
            selections: None,
            selection_invalidated: None,
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
                Definition::Blank => Compiled::Blank,
                Definition::NonNumeric => Compiled::NonNumeric,
                Definition::CapturedScalar(id) => Compiled::CapturedScalar(*id),
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
    pub(super) fn contains_node(&self, id: NodeId) -> bool {
        self.record(id).is_some_and(|r| r.live)
    }
    pub(super) fn next_identity(&self) -> NodeId {
        NodeId {
            model: self.version.model,
            node: self.next,
        }
    }
    pub(super) fn draft_nodes(&self) -> impl Iterator<Item = (NodeId, &str, DraftDefinition<'_>)> {
        self.records
            .iter()
            .filter(|(_, r)| r.live)
            .map(|(slot, r)| {
                let id = NodeId {
                    model: self.version.model,
                    node: slot,
                };
                let definition = match &*r.definition {
                    Compiled::Blank => DraftDefinition::Blank,
                    Compiled::NonNumeric => DraftDefinition::NonNumeric,
                    Compiled::CapturedScalar(id) => DraftDefinition::CapturedScalar(*id),
                    Compiled::Input => DraftDefinition::Input(**self.values.get(slot).unwrap()),
                    Compiled::Formula { source, binding } => DraftDefinition::Computed {
                        source,
                        binding: binding.as_ref().map(|(_, d)| d.as_slice()),
                    },
                };
                (id, r.name.as_str(), definition)
            })
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
    fn edit_selection(
        &mut self,
        edit: impl FnOnce(&mut selection::State) -> Result<(), selection::Error>,
    ) -> Result<(), Error> {
        let mut state = match &self.selections {
            Some(s) => (**s).clone(),
            None => selection::State::new(self.memory.clone()).map_err(|_| Error::Admission)?,
        };
        edit(&mut state).map_err(|e| match e {
            selection::Error::Quota => Error::Quota,
            selection::Error::Invalid => Error::InvalidTransfer,
            selection::Error::Admission => Error::Admission,
            selection::Error::Cancelled => Error::Cancelled,
        })?;
        self.selections = Some(self.memory.alloc(state)?);
        Ok(())
    }
    pub(super) fn has_selection_formulas(&self) -> bool {
        self.selections
            .as_ref()
            .is_some_and(|s| s.metrics().consumers != 0)
    }
    fn selection_dependencies(
        &self,
        id: NodeId,
        budget: &mut usize,
        cancel: &dyn Fn() -> bool,
    ) -> Result<DependencyScratch<'_>, Error> {
        let definition = &self.live_record(id)?.definition;
        let selections = definition.selection_ids();
        if selections.is_empty() {
            return Ok(DependencyScratch {
                items: std::borrow::Cow::Borrowed(definition.deps()),
                _charge: None,
            });
        }
        let state = self.selections.as_ref().ok_or(Error::InvalidTransfer)?;
        let upper = state
            .metrics()
            .points
            .saturating_mul(selections.len())
            .saturating_add(definition.deps().len())
            .min(selection::MAX_MEMBER_EDGES_PER_FORMULA);
        let charge = self.memory.reserve(upper.saturating_mul(64) + 4096)?;
        let mut members = BTreeSet::new();
        members.extend(definition.deps().iter().copied());
        let mut range_budget = (*budget).min(selection::MAX_MEMBER_EDGES_PER_FORMULA);
        let initial = range_budget;
        for selection in selections {
            members.extend(
                state
                    .members(selection, &mut range_budget, cancel)
                    .map_err(|e| {
                        if matches!(e, selection::Error::Cancelled) {
                            Error::Cancelled
                        } else {
                            Error::ValidationBudget
                        }
                    })?,
            );
        }
        *budget = budget
            .checked_sub(initial - range_budget)
            .ok_or(Error::ValidationBudget)?;
        Ok(DependencyScratch {
            items: std::borrow::Cow::Owned(members.into_iter().collect()),
            _charge: Some(charge),
        })
    }
    pub(super) fn selection_image(&self) -> selection::Image {
        self.selections
            .as_ref()
            .map(|s| selection::Image {
                identities: s.identities().collect(),
                points: s.points().collect(),
            })
            .unwrap_or_default()
    }
    pub(super) fn selection_metrics(&self) -> selection::Metrics {
        let mut metrics = self
            .selections
            .as_ref()
            .map(|s| s.metrics())
            .unwrap_or_default();
        metrics.invalidated_nodes = self.selection_invalidated.as_ref().map_or(0, |n| n.len());
        metrics
    }
    pub(super) fn invalidated_selection_nodes(&self) -> &[NodeId] {
        self.selection_invalidated
            .as_ref()
            .map_or(&[], |n| n.as_slice())
    }
    pub(super) fn typed_dependencies(
        &self,
        id: NodeId,
    ) -> Result<Vec<selection::Dependency>, Error> {
        let def = &self.live_record(id)?.definition;
        let mut edges = def
            .deps()
            .iter()
            .copied()
            .map(selection::Dependency::Scalar)
            .collect::<Vec<_>>();
        edges.extend(
            def.selection_ids()
                .into_iter()
                .map(selection::Dependency::Selection),
        );
        Ok(edges)
    }
    pub fn dependencies(&self, id: NodeId) -> Result<&[NodeId], Error> {
        if !self.live_record(id)?.definition.selection_ids().is_empty() {
            return Err(ModelError::Backend("use typed selection dependency inspection").into());
        }
        Ok(self.live_record(id)?.definition.deps())
    }
    pub fn dependents(&self, id: NodeId) -> Result<BTreeSet<NodeId>, Error> {
        let mut dependents: BTreeSet<_> = self
            .live_record(id)?
            .reverse
            .iter()
            .map(|(n, _)| *n)
            .collect();
        if let Some(state) = &self.selections {
            for selection in state.containing_node(id) {
                dependents.extend(state.consumers_of(selection));
            }
        }
        Ok(dependents)
    }
    pub fn dependent_count(&self, id: NodeId) -> Result<usize, Error> {
        if self.has_selection_formulas() {
            return Ok(self.dependents(id)?.len());
        }
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
        let _profile = crate::experiment_profile::construction::scope(5);
        let _phase = crate::experiment_profile::scope(4);
        let old = self.live_record(id)?.clone();
        if let Compiled::CapturedScalar(slot) = definition {
            if self.captured_scalar.is_some_and(|(node, _)| node != id) {
                return Err(Error::Quota);
            }
            self.captured_scalar = Some((id, slot));
        } else if self.captured_scalar.is_some_and(|(node, _)| node == id) {
            self.captured_scalar = None;
        }
        self.bytes = self.bytes - old.definition.bytes() + definition.bytes();
        let old_selections = old.definition.selection_ids();
        let new_selections = definition.selection_ids();
        for selection in old_selections.symmetric_difference(&new_selections) {
            self.edit_selection(|state| {
                state.consumer(*selection, id, new_selections.contains(selection))
            })?;
        }
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
        self.apply_mode(expected, edits, None, None)
    }
    pub(super) fn apply_bulk(
        &mut self,
        expected: Version,
        edits: Vec<Edit>,
        cancel: &dyn Fn() -> bool,
        retire: impl FnOnce(Graph) + Send + 'static,
    ) -> Result<Version, Error> {
        let _profile = crate::experiment_profile::construction::scope(6);
        // Reserve all new names/identities before compiling any submitted formula.
        let mut additions = Vec::new();
        let mut definitions = Vec::new();
        let mut next = self.next;
        for edit in edits {
            if cancel() {
                return Err(Error::Cancelled);
            }
            match edit {
                Edit::AddInput(n, v) => {
                    additions.push(Edit::AddInput(n, v));
                    next = next.checked_add(1).ok_or(Error::Exhausted)?;
                }
                Edit::AddFormula(n, source) => {
                    let id = NodeId {
                        model: self.version.model,
                        node: next,
                    };
                    next = next.checked_add(1).ok_or(Error::Exhausted)?;
                    additions.push(Edit::AddInput(n, PivotDecimal::ZERO));
                    definitions.push(Edit::Formula(id, source));
                }
                edit => definitions.push(edit),
            }
        }
        additions.extend(definitions);
        self.apply_mode(expected, additions, Some(cancel), Some(Box::new(retire)))
    }
    fn apply_mode(
        &mut self,
        expected: Version,
        edits: Vec<Edit>,
        bulk: Option<&dyn Fn() -> bool>,
        retire: Option<Box<dyn FnOnce(Graph) + Send>>,
    ) -> Result<Version, Error> {
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
        let max_source = edits
            .iter()
            .map(|e| match e {
                Edit::Formula(_, s)
                | Edit::Resolved(_, s, _)
                | Edit::Invalid(_, s, _)
                | Edit::AddFormula(_, s) => s.len(),
                _ => 0,
            })
            .max()
            .unwrap_or(0);
        let source_scratch = if bulk.is_some() {
            source_bytes
                .saturating_mul(4)
                .saturating_add(max_source.saturating_mul(512))
        } else {
            source_bytes.saturating_mul(512)
        };
        let _scratch = self.memory.reserve(
            8192usize
                .saturating_add(self.max_name.saturating_mul(256))
                .saturating_add(source_scratch)
                .saturating_add(transfer_bytes)
                .saturating_add(structural_work),
        )?;
        let mut guard = CandidateRoot {
            value: Some({
                let _phase = crate::experiment_profile::scope(1);
                self.clone()
            }),
            retire,
        };
        let candidate = guard.value.as_mut().unwrap();
        if candidate.local_validation && self.certified() {
            candidate.invalid = Index::new(candidate.memory.clone());
            candidate.import_invalid = false;
        }
        let mut changed = BTreeSet::new();
        let mut invalidation_sources = BTreeSet::new();
        let mut topology = false;
        let mut inputs = false;
        let install_profile = crate::experiment_profile::construction::scope(6);
        for edit in edits {
            if bulk.is_some_and(|check| check()) {
                return Err(Error::Cancelled);
            }
            match &edit {
                Edit::CapturedScalar(id, _)
                | Edit::Blank(id)
                | Edit::NonNumeric(id)
                | Edit::Input(id, _)
                | Edit::ReplaceInput(id, _)
                | Edit::Resolved(id, _, _)
                | Edit::Invalid(id, _, _)
                | Edit::Formula(id, _)
                | Edit::Retire(id) => {
                    invalidation_sources.insert(*id);
                }
                Edit::Placement(space, row, column, _) => {
                    for graph in [&*self, &*candidate] {
                        if let Some(state) = &graph.selections {
                            for selection in state.covering(*space, *row, *column) {
                                invalidation_sources.extend(state.consumers_of(selection));
                            }
                        }
                    }
                }
                _ => (),
            }
            match edit {
                Edit::RetiredSelection(id) => {
                    candidate.edit_selection(|state| state.retired_identity(id))?;
                    topology = true;
                }
                Edit::Selection(id, rectangle) => {
                    candidate.edit_selection(|state| state.define(id, rectangle))?;
                    topology = true;
                }
                Edit::Placement(space, row, column, node) => {
                    if node.is_some_and(|id| candidate.live_record(id).is_err()) {
                        return Err(Error::InvalidTransfer);
                    }
                    let mut moved = false;
                    candidate.edit_selection(|state| {
                        moved = state.placement(space, row, column, node)?;
                        Ok(())
                    })?;
                    topology |= moved;
                }
                Edit::CapturedScalar(id, slot) => {
                    candidate.replace(id, Compiled::CapturedScalar(slot))?;
                    changed.insert(id);
                    topology = true;
                }
                Edit::Blank(id) => {
                    candidate.replace(id, Compiled::Blank)?;
                    changed.insert(id);
                    topology = true;
                }
                Edit::NonNumeric(id) => {
                    candidate.replace(id, Compiled::NonNumeric)?;
                    changed.insert(id);
                    topology = true;
                }
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
                    let _profile = crate::experiment_profile::construction::scope(5);
                    if source.len() > 4096 {
                        return Err(Error::InvalidTransfer);
                    }
                    let mut bound = Vec::with_capacity(tokens.len());
                    for token in tokens {
                        bound.push(match token {
                            transfer::ResolvedToken::Aggregate(kind, id) => {
                                if kind == crate::expression::AggregateKind::Sum
                                    || candidate
                                        .selections
                                        .as_ref()
                                        .and_then(|s| s.descriptor(id))
                                        .is_none()
                                {
                                    return Err(Error::InvalidTransfer);
                                }
                                BoundToken::Aggregate(kind, id)
                            }
                            transfer::ResolvedToken::Sum(id) => {
                                if candidate
                                    .selections
                                    .as_ref()
                                    .and_then(|s| s.descriptor(id))
                                    .is_none()
                                {
                                    return Err(Error::InvalidTransfer);
                                }
                                BoundToken::Sum(id)
                            }
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
                    if candidate
                        .captured_scalar
                        .is_some_and(|(node, _)| node == id)
                    {
                        candidate.captured_scalar = None;
                    }
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
                    for selection in record.definition.selection_ids() {
                        candidate.edit_selection(|state| state.consumer(selection, id, false))?;
                    }
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
        drop(install_profile);
        let validation_profile = crate::experiment_profile::construction::scope(7);
        candidate.check_quota()?;
        if topology && candidate.selections.is_some() {
            candidate.edit_selection(|state| state.retire_unused())?;
        }
        if topology && candidate.has_selection_formulas() {
            // The scalar incremental proof cannot certify a graph with selection edges.
            // Validate the canonical dynamic relation; retain invalid drafts, not prefixes.
            changed.extend(
                candidate
                    .records
                    .iter()
                    .filter(|(_, r)| r.live)
                    .map(|(slot, _)| NodeId {
                        model: candidate.version.model,
                        node: slot,
                    }),
            );
            candidate.invalid = Index::new(candidate.memory.clone());
            candidate.import_invalid = false;
            if let Some((id, error)) =
                candidate.validate_many(&changed, bulk.unwrap_or(&|| false))?
            {
                candidate
                    .invalid
                    .insert(id, candidate.memory.alloc(error)?)?;
            }
        } else if candidate.import_invalid && topology && !candidate.local_validation {
            let restored = candidate.materialize()?;
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
        } else if topology && let Some(check) = bulk {
            if let Some((id, error)) = candidate.validate_many(&changed, check)? {
                candidate
                    .invalid
                    .insert(id, candidate.memory.alloc(error)?)?;
            }
        } else if topology {
            let _phase = crate::experiment_profile::scope(7);
            if !candidate.local_validation {
                changed.extend(candidate.invalid.iter().map(|(id, _)| *id));
            }
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
        if self.has_selection_formulas() || candidate.has_selection_formulas() {
            let _invalidation = self
                .memory
                .reserve(selection::MAX_INVALIDATION_NODES * 256)?;
            let mut seen = BTreeSet::new();
            let mut pending = invalidation_sources.into_iter().collect::<Vec<_>>();
            while let Some(node) = pending.pop() {
                if bulk.is_some_and(|cancel| cancel()) {
                    return Err(Error::Cancelled);
                }
                if !seen.insert(node) {
                    continue;
                }
                if seen.len() > selection::MAX_INVALIDATION_NODES
                    || pending.len() > selection::MAX_INVALIDATION_NODES
                {
                    return Err(Error::ValidationBudget);
                }
                for graph in [&*self, &*candidate] {
                    if let Ok(record) = graph.live_record(node) {
                        if pending.len().saturating_add(record.reverse.len())
                            > selection::MAX_INVALIDATION_NODES
                        {
                            return Err(Error::ValidationBudget);
                        }
                        pending.extend(record.reverse.iter().map(|(n, _)| *n));
                        if let Some(state) = &graph.selections {
                            for selection in state.containing_node(node) {
                                let consumers = state.consumers_of(selection);
                                if pending.len().saturating_add(consumers.len())
                                    > selection::MAX_INVALIDATION_NODES
                                {
                                    return Err(Error::ValidationBudget);
                                }
                                pending.extend(consumers);
                            }
                        }
                    }
                }
            }
            candidate.selection_invalidated =
                Some(candidate.memory.alloc(seen.into_iter().collect())?);
        }
        if !topology && !inputs {
            return Ok(self.version);
        }
        if topology {
            candidate.proof = candidate
                .memory
                .alloc(Proof(std::sync::atomic::AtomicBool::new(
                    !candidate.import_invalid && candidate.invalid.is_empty(),
                )))?;
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
            drop(validation_profile);
            let _profile = crate::experiment_profile::construction::scope(9);
            *self = guard.take();
        }
        Ok(self.version)
    }
    fn validate_many(
        &self,
        roots: &BTreeSet<NodeId>,
        cancel: &dyn Fn() -> bool,
    ) -> Result<Option<(NodeId, SnapshotError)>, Error> {
        let mut done = BTreeSet::new();
        let mut active = BTreeSet::new();
        let mut budget = self.limits.validation_visits;
        let mut work = Work::new(self.memory.clone());
        for &root in roots {
            if self.live_record(root).is_err() {
                continue;
            }
            let mut pending = vec![(root, false)];
            while let Some((id, exit)) = pending.pop() {
                if cancel() {
                    return Err(Error::Cancelled);
                }
                budget = budget.checked_sub(1).ok_or(Error::ValidationBudget)?;
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
                    return Ok(Some((
                        root,
                        SnapshotError::Unorderable {
                            nodes: active.into_iter().collect(),
                        },
                    )));
                }
                let record = self.live_record(id)?;
                if let Compiled::Formula {
                    binding: Err(error),
                    ..
                } = &*record.definition
                {
                    return Ok(Some((root, error.clone())));
                }
                let deps = self.selection_dependencies(id, &mut budget, cancel)?;
                work.add(deps.items.len().saturating_mul(64))?;
                pending.push((id, true));
                for &target in deps.items.iter() {
                    if self.live_record(target).is_err() {
                        return Ok(Some((
                            root,
                            SnapshotError::MissingReference { node: id, target },
                        )));
                    }
                    pending.push((target, false));
                }
            }
        }
        Ok(None)
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
            let deps = self.selection_dependencies(id, budget, &|| false)?;
            work.add(deps.items.len().saturating_mul(64))?;
            pending.push((id, true));
            for &target in deps.items.iter() {
                if self.live_record(target).is_err() {
                    return Ok(Some(SnapshotError::MissingReference { node: id, target }));
                }
                pending.push((target, false));
            }
        }
        Ok(None)
    }
    pub(super) fn capture_local(&self) -> Result<Capture, CaptureError> {
        Ok(Capture(
            self.memory
                .alloc(self.clone())
                .map_err(|_| CaptureError::Admission)?,
        ))
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
    pub(super) fn memory(&self) -> Memory {
        self.0.memory.clone()
    }
    pub fn version(&self) -> Version {
        self.0.version
    }
    pub(super) fn captured_slots(&self) -> Vec<crate::captured::CapturedScalarId> {
        self.0
            .captured_scalar
            .map(|(_, slot)| slot)
            .into_iter()
            .collect()
    }
    pub fn evaluate(&self, outputs: &[NodeId]) -> Result<Calculation, ComputeError> {
        self.evaluate_retiring(outputs, drop)
    }
    pub(super) fn evaluate_retiring(
        &self,
        outputs: &[NodeId],
        retire: impl FnOnce(CalculationScratch),
    ) -> Result<Calculation, ComputeError> {
        let _request = self.0.memory.reserve(
            outputs
                .len()
                .saturating_mul(std::mem::size_of::<NodeId>())
                .saturating_add(128),
        )?;
        self.invoke_retiring(
            &crate::evaluation::InvocationRequest {
                inputs: Vec::new(),
                outputs: outputs.to_vec(),
            },
            retire,
        )
    }
    pub fn invoke(
        &self,
        request: &crate::evaluation::InvocationRequest,
    ) -> Result<Calculation, ComputeError> {
        self.invoke_retiring(request, drop)
    }
    fn invoke_retiring(
        &self,
        request: &crate::evaluation::InvocationRequest,
        retire: impl FnOnce(CalculationScratch),
    ) -> Result<Calculation, ComputeError> {
        self.invoke_retiring_cancelled(request, retire, || false)
    }
    pub(super) fn invoke_retiring_cancelled(
        &self,
        request: &crate::evaluation::InvocationRequest,
        retire: impl FnOnce(CalculationScratch),
        cancelled: impl Fn() -> bool,
    ) -> Result<Calculation, ComputeError> {
        self.invoke_sources_retiring_cancelled(request, None, retire, cancelled)
    }
    pub(super) fn invoke_sources_retiring_cancelled(
        &self,
        request: &crate::evaluation::InvocationRequest,
        sources: Option<&BTreeMap<crate::captured::CapturedScalarId, crate::captured::ScalarValue>>,
        retire: impl FnOnce(CalculationScratch),
        cancelled: impl Fn() -> bool,
    ) -> Result<Calculation, ComputeError> {
        if cancelled() {
            return Err(ComputeError::Cancelled);
        }
        let slots = self.captured_slots();
        if sources.is_none() && !slots.is_empty() {
            return Err(crate::evaluation::InvocationError::CapturedContext(
                crate::captured::ContextError::Missing,
            )
            .into());
        }
        let _phase = crate::experiment_profile::scope(8);
        let mut work = Work::new(self.0.memory.clone());
        work.add(4096 + request.inputs.len() * 512 + request.outputs.len() * 128)?;
        let mut inputs = BTreeMap::new();
        let mut active = BTreeSet::new();
        let mut required = BTreeSet::new();
        let mut order = Vec::new();
        // Match the accepted selected-output normalization and unknown-output
        // precedence even when callers submit duplicates in arbitrary order.
        let outputs: BTreeSet<_> = request.outputs.iter().copied().collect();
        let mut pending: Vec<_> = outputs.iter().map(|&n| (n, false)).collect();
        let mut values: BTreeMap<NodeId, Result<PivotDecimal, EvaluationError>> = BTreeMap::new();
        let outcome = (|| -> Result<Calculation, ComputeError> {
            let prepared = crate::evaluation::prepare_inputs_into(
                request,
                |id| {
                    if cancelled() {
                        return None;
                    }
                    self.0
                        .record(id)
                        .filter(|r| r.live)
                        .map(|r| matches!(*r.definition, Compiled::Input))
                },
                &mut inputs,
            );
            if cancelled() {
                return Err(ComputeError::Cancelled);
            }
            prepared?;
            let mut visits = self.0.limits.validation_visits;
            while let Some((id, exit)) = pending.pop() {
                if cancelled() {
                    return Err(ComputeError::Cancelled);
                }
                visits = visits.checked_sub(1).ok_or(ComputeError::WorkBudget)?;
                let record = self.0.live_record(id).map_err(|_| {
                    crate::evaluation::InvocationError::Evaluation(EvaluationError::UnknownNode(id))
                })?;
                if exit {
                    active.remove(&id);
                    order.push(id);
                } else {
                    if active.contains(&id) {
                        return Err(ComputeError::Invalid(SnapshotError::Unorderable {
                            nodes: active.iter().copied().collect(),
                        }));
                    }
                    if let Compiled::Formula {
                        binding: Err(error),
                        ..
                    } = &*record.definition
                    {
                        return Err(ComputeError::Invalid(error.clone()));
                    }
                    let deps = self
                        .0
                        .selection_dependencies(id, &mut visits, &cancelled)
                        .map_err(|_| {
                            if cancelled() {
                                ComputeError::Cancelled
                            } else {
                                ComputeError::WorkBudget
                            }
                        })?;
                    for &target in deps.items.iter() {
                        if self.0.live_record(target).is_err() {
                            return Err(ComputeError::Invalid(SnapshotError::MissingReference {
                                node: id,
                                target,
                            }));
                        }
                    }
                    work.add(512 + deps.items.len() * 64)?;
                    if required.insert(id) {
                        active.insert(id);
                        pending.push((id, true));
                        pending.extend(deps.items.iter().map(|&d| (d, false)));
                    }
                }
            }
            for &id in &order {
                if cancelled() {
                    return Err(ComputeError::Cancelled);
                }
                let record = self.0.live_record(id).unwrap();
                let value = match &*record.definition {
                    Compiled::Blank => Ok(PivotDecimal::ZERO),
                    Compiled::NonNumeric => Err(EvaluationError::NonNumericInput { node: id }),
                    Compiled::CapturedScalar(slot) => match sources.and_then(|s| s.get(slot)) {
                        Some(crate::captured::ScalarValue::Decimal(value)) => Ok(*value),
                        Some(crate::captured::ScalarValue::Fault(fault_id)) => {
                            Err(EvaluationError::CapturedSourceFailed {
                                node: id,
                                fault_id: *fault_id,
                            })
                        }
                        None => Err(EvaluationError::MissingCapturedContext { node: id }),
                    },
                    Compiled::Input => Ok(inputs
                        .get(&id)
                        .copied()
                        .unwrap_or(**self.0.values.get(id.node).unwrap())),
                    Compiled::Formula {
                        binding: Ok((tokens, _)),
                        ..
                    } => {
                        let _stack = self.0.memory.reserve(tokens.len().saturating_mul(64))?;
                        // Operation-bearing keys, cached results and their tree metadata
                        // are separately reserved; pure SUM retains its existing admission.
                        let _aggregate_cache = if tokens
                            .iter()
                            .any(|t| matches!(t, BoundToken::Aggregate(_, _)))
                        {
                            Some(self.0.memory.reserve(tokens.len().saturating_mul(192))?)
                        } else {
                            None
                        };
                        let mut sums = BTreeMap::new();
                        if let Some(state) = self.0.selections.as_ref() {
                            let _members = self
                                .0
                                .memory
                                .reserve(state.metrics().points.saturating_mul(64) + 4096)?;
                            let operations: BTreeSet<_> = tokens
                                .iter()
                                .filter_map(|t| match t {
                                    BoundToken::Sum(id) => {
                                        Some((crate::expression::AggregateKind::Sum, *id))
                                    }
                                    BoundToken::Aggregate(kind, id) => Some((*kind, *id)),
                                    _ => None,
                                })
                                .collect();
                            for (kind, selection) in operations {
                                let members = state
                                    .members(selection, &mut visits, &cancelled)
                                    .map_err(|_| {
                                        if cancelled() {
                                            ComputeError::Cancelled
                                        } else {
                                            ComputeError::WorkBudget
                                        }
                                    })?;
                                let mut total = PivotDecimal::ZERO;
                                let mut extremum: Option<PivotDecimal> = None;
                                let mut count = 0u64;
                                let mut mean = crate::aggregate::Mean::default();
                                let mut failure = None;
                                for member in members {
                                    if cancelled() {
                                        return Err(ComputeError::Cancelled);
                                    }
                                    if kind != crate::expression::AggregateKind::Sum {
                                        work.add(64)?;
                                    }
                                    if kind != crate::expression::AggregateKind::Sum
                                        && self.0.live_record(member).is_ok_and(|r| {
                                            matches!(
                                                *r.definition,
                                                Compiled::Blank | Compiled::NonNumeric
                                            )
                                        })
                                    {
                                        continue;
                                    }
                                    let value = match values
                                        .get(&member)
                                        .ok_or(EvaluationError::InvalidExpression { node: id })
                                        .and_then(|v| {
                                            v.as_ref()
                                                .copied()
                                                .map_err(|e| e.dependency(id, member))
                                        }) {
                                        Ok(v) => v,
                                        Err(e) => {
                                            failure = Some(e);
                                            break;
                                        }
                                    };
                                    count = count.checked_add(1).ok_or(ComputeError::WorkBudget)?;
                                    use crate::expression::AggregateKind as K;
                                    let result = match kind {
                                        K::Sum => total.checked_add(value).map(|v| {
                                            total = v;
                                        }),
                                        K::Average => mean.push(value),
                                        K::Min => {
                                            extremum =
                                                Some(extremum.map_or(value, |v| v.min(value)));
                                            Ok(())
                                        }
                                        K::Max => {
                                            extremum =
                                                Some(extremum.map_or(value, |v| v.max(value)));
                                            Ok(())
                                        }
                                        K::Count => Ok(()),
                                    };
                                    if let Err(error) = result {
                                        failure =
                                            Some(EvaluationError::Arithmetic { node: id, error });
                                        break;
                                    }
                                }
                                use crate::expression::AggregateKind as K;
                                let result = if let Some(error) = failure {
                                    Err(error)
                                } else if kind != K::Sum && kind != K::Count && count == 0 {
                                    Err(EvaluationError::NoNumericValues {
                                        node: id,
                                        operation: kind,
                                        selection,
                                    })
                                } else {
                                    match kind {
                                        K::Sum => Ok(total),
                                        K::Count => Ok(PivotDecimal::from(count as i64)),
                                        K::Min | K::Max => Ok(extremum.unwrap().normalize()),
                                        K::Average => {
                                            work.add(8192)?;
                                            mean.finish().map_err(|error| {
                                                EvaluationError::Arithmetic { node: id, error }
                                            })
                                        }
                                    }
                                };
                                sums.insert((kind, selection), result);
                            }
                        }
                        crate::evaluation::evaluate_expression_with_selections(
                            id,
                            tokens,
                            &values,
                            |kind, selection| {
                                sums.get(&(kind, selection))
                                    .cloned()
                                    .unwrap_or(Err(EvaluationError::InvalidExpression { node: id }))
                            },
                        )
                    }
                    _ => unreachable!("capture rejects invalid drafts"),
                };
                values.insert(id, value);
            }
            Ok(Calculation {
                version: self.version(),
                values: std::mem::take(&mut values),
                input_overrides: std::mem::take(&mut inputs),
                _capture: self.clone(),
                _work: std::mem::replace(&mut work, Work::new(self.0.memory.clone())),
            })
        })();
        // Even a work-budget failure can own a large partial closure/result.
        // Keep its reservations with those buffers until actual reclamation.
        retire(CalculationScratch {
            _work: work,
            _inputs: inputs,
            _outputs: outputs,
            _required: required,
            _active: active,
            _order: order,
            _pending: pending,
            _values: values,
        });
        outcome
    }
}
pub(super) struct CalculationScratch {
    _inputs: BTreeMap<NodeId, PivotDecimal>,
    _outputs: BTreeSet<NodeId>,
    _required: BTreeSet<NodeId>,
    _active: BTreeSet<NodeId>,
    _order: Vec<NodeId>,
    _pending: Vec<(NodeId, bool)>,
    _values: BTreeMap<NodeId, Result<PivotDecimal, EvaluationError>>,
    _work: Work,
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
    Invalid(SnapshotError),
    Cancelled,
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

#[cfg(test)]
mod selection_transaction_tests {
    use super::*;
    #[test]
    fn late_selection_walk_cancellation_preserves_all_roots() {
        let runtime = integration::IntegrationRuntime::standalone().unwrap();
        let mut model = WorkingModel::new_in("Cancel", &runtime).unwrap();
        let mut batch = model.begin_batch(130, 32768).unwrap();
        let mut ids = Vec::new();
        for i in 0..60 {
            let id = batch.add_input(format!("P{i}"), 1.into()).unwrap();
            ids.push(id);
            batch
                .place(selection::SpaceId([1; 16]), i + 1, 1, Some(id))
                .unwrap();
        }
        let sum = batch.add_input("Sum", 0.into()).unwrap();
        let selected = selection::SelectionId([2; 16]);
        batch
            .define_selection(
                selected,
                selection::Rectangle {
                    space: selection::SpaceId([1; 16]),
                    first_row: 1,
                    last_row: 1000,
                    first_column: 1,
                    last_column: 2,
                },
            )
            .unwrap();
        batch
            .set_resolved(sum, "SUM", vec![transfer::ResolvedToken::Sum(selected)])
            .unwrap();
        model.apply_batch(batch, &Default::default()).unwrap();
        let mut graph = model.integrated.graph().clone();
        let before = graph.version;
        let points = graph.selection_image().points;
        let metrics = graph.selection_metrics();
        let calls = std::cell::Cell::new(0);
        let cancel = || {
            calls.set(calls.get() + 1);
            calls.get() > 25
        };
        let result = graph.apply_bulk(
            before,
            vec![Edit::Placement(
                selection::SpaceId([1; 16]),
                61,
                1,
                Some(ids[0]),
            )],
            &cancel,
            drop,
        );
        assert!(matches!(result, Err(Error::Cancelled)));
        assert!(calls.get() > 25);
        assert!(graph.selection_metrics().visited_index_nodes > metrics.visited_index_nodes);
        assert_eq!(graph.version, before);
        assert_eq!(graph.selection_image().points, points);
        // A late admission failure after preparing the candidate also preserves roots.
        let memory = runtime.memory();
        let hold = memory
            .reserve(memory.stats().limit - memory.stats().charged - 16_384)
            .unwrap();
        assert!(
            graph
                .apply_bulk(
                    before,
                    vec![Edit::Placement(
                        selection::SpaceId([1; 16]),
                        61,
                        1,
                        Some(ids[0])
                    )],
                    &|| false,
                    drop
                )
                .is_err()
        );
        assert_eq!(graph.version, before);
        assert_eq!(graph.selection_image().points, points);
        drop(hold);
        drop(graph);
        drop(model);
        assert!(runtime.finish().is_ok());
        assert_eq!(memory.stats().charged, 0);
    }
}
