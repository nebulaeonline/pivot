//! Opt-in structural feasibility laboratory. Not a production model API.
//! Fixed-radix directory + 64-slot immutable pages; IDs never encode addresses.
use super::*;
use crate::evaluation::{EvaluationError, evaluate_expression};

const PAGE: usize = 64;
type Link<T> = Option<Arc<Tree<T>>>;
enum Tree<T> {
    Branch([Link<T>; 16]),
    Leaf(Arc<T>),
}
pub struct Pages<T> {
    root: Link<[Option<Arc<T>>; PAGE]>,
}
impl<T> Clone for Pages<T> {
    fn clone(&self) -> Self {
        Self {
            root: self.root.clone(),
        }
    }
}
impl<T> Default for Pages<T> {
    fn default() -> Self {
        Self { root: None }
    }
}
fn lookup<T>(root: &Link<T>, key: u64) -> Option<&Arc<T>> {
    let mut current = root.as_deref()?;
    for shift in (0..16).rev() {
        match current {
            Tree::Branch(children) => {
                current = children[((key >> (shift * 4)) & 15) as usize].as_deref()?
            }
            Tree::Leaf(_) => unreachable!(),
        }
    }
    match current {
        Tree::Leaf(value) => Some(value),
        _ => unreachable!(),
    }
}
fn insert<T>(root: &mut Link<T>, key: u64, depth: usize, value: Arc<T>) {
    if depth == 16 {
        *root = Some(Arc::new(Tree::Leaf(value)));
        return;
    }
    if root.is_none() {
        *root = Some(Arc::new(Tree::Branch(std::array::from_fn(|_| None))));
    }
    // Unique candidate branches mutate in place; shared branches alone are copied.
    if Arc::strong_count(root.as_ref().unwrap()) != 1 {
        let Tree::Branch(children) = root.as_deref().unwrap() else {
            unreachable!()
        };
        crate::experiment_profile::copy(std::mem::size_of::<Tree<T>>());
        *root = Some(Arc::new(Tree::Branch(children.clone())));
    }
    let Tree::Branch(children) = Arc::get_mut(root.as_mut().unwrap()).unwrap() else {
        unreachable!()
    };
    insert(
        &mut children[((key >> ((15 - depth) * 4)) & 15) as usize],
        key,
        depth + 1,
        value,
    );
}
impl<T> Pages<T> {
    pub fn get(&self, key: u64) -> Option<&Arc<T>> {
        lookup(&self.root, key / PAGE as u64)?[(key % PAGE as u64) as usize].as_ref()
    }
    pub fn set(&mut self, key: u64, value: Arc<T>) {
        let page_key = key / PAGE as u64;
        // Take the root's page reference only after copying its directory path.
        let mut page = lookup(&self.root, page_key)
            .cloned()
            .unwrap_or_else(|| Arc::new(std::array::from_fn(|_| None)));
        if Arc::strong_count(&page) > 1 {
            crate::experiment_profile::copy(std::mem::size_of::<[Option<Arc<T>>; PAGE]>());
        }
        Arc::make_mut(&mut page)[(key % PAGE as u64) as usize] = Some(value);
        insert(&mut self.root, page_key, 0, page);
    }
}

#[derive(Clone)]
struct Record {
    live: bool,
    definition: Arc<Compiled>,
    reverse: Arc<BTreeSet<NodeId>>,
}
enum Compiled {
    Input,
    Formula {
        source: String,
        binding: Result<(Vec<BoundToken>, Vec<NodeId>), SnapshotError>,
    },
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
}
#[derive(Clone)]
pub struct Graph {
    records: Pages<Record>,
    values: Pages<PivotDecimal>,
    names: Arc<BTreeMap<String, NodeId>>,
    invalid: Arc<BTreeMap<NodeId, SnapshotError>>,
    version: Version,
    next: u64,
    live: usize,
    bytes: usize,
    limits: Limits,
}
impl Graph {
    /// Import is intentionally O(N); refuses model ports until an interface adapter exists.
    pub fn from_model(model: &WorkingModel, limits: Limits) -> Result<Self, Error> {
        if model.interface.inputs().next().is_some() || model.interface.outputs().next().is_some() {
            return Err(Error::InvalidTransfer);
        }
        let snapshot = model.snapshot().map_err(|_| Error::InvalidTransfer)?;
        let mut graph = Self {
            records: Pages::default(),
            values: Pages::default(),
            names: Arc::new(model.names.clone()),
            invalid: Arc::new(BTreeMap::new()),
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
        };
        for (&id, node) in &snapshot.0.nodes {
            let definition = match &node.definition {
                SnapshotDefinition::Input(value) => {
                    graph.values.set(id.node, Arc::new(*value));
                    Compiled::Input
                }
                SnapshotDefinition::Computed { source, tokens } => Compiled::Formula {
                    source: source.clone(),
                    binding: Ok((tokens.clone(), node.dependencies.clone())),
                },
            };
            graph.bytes += definition.bytes();
            graph.records.set(
                id.node,
                Arc::new(Record {
                    live: true,
                    definition: Arc::new(definition),
                    reverse: Arc::new(node.dependents.iter().copied().collect()),
                }),
            );
        }
        graph.check_quota()?;
        Ok(graph)
    }
    pub fn version(&self) -> Version {
        self.version
    }
    pub fn node_id(&self, name: &str) -> Option<NodeId> {
        self.names.get(name).copied()
    }
    fn record(&self, id: NodeId) -> Option<&Arc<Record>> {
        (id.model == self.version.model)
            .then(|| self.records.get(id.node))
            .flatten()
    }
    fn live_record(&self, id: NodeId) -> Result<&Arc<Record>, Error> {
        self.record(id).filter(|r| r.live).ok_or(Error::Unknown(id))
    }
    pub fn dependencies(&self, id: NodeId) -> Result<&[NodeId], Error> {
        Ok(self.live_record(id)?.definition.deps())
    }
    pub fn dependents(&self, id: NodeId) -> Result<&BTreeSet<NodeId>, Error> {
        Ok(&self.live_record(id)?.reverse)
    }
    pub fn same_definition(&self, other: &Self, id: NodeId) -> bool {
        match (self.record(id), other.record(id)) {
            (Some(a), Some(b)) => Arc::ptr_eq(&a.definition, &b.definition),
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
            .and_then(|expr| bind(id, &expr, &self.names));
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
                crate::experiment_profile::copy(
                    record.reverse.len() * std::mem::size_of::<NodeId>(),
                );
                let reverse = Arc::make_mut(&mut record.reverse);
                if after.contains(target) {
                    reverse.insert(id);
                } else {
                    reverse.remove(&id);
                }
                self.records.set(target.node, Arc::new(record));
            }
        }
        // Fetch again: self-reference may have updated this record's reverse index.
        let mut record = (**self.live_record(id)?).clone();
        record.definition = Arc::new(definition);
        self.records.set(id.node, Arc::new(record));
        Ok(())
    }
    /// Private branch, incremental indexes, bounded validation, single atomic installation.
    /// Invalid source/missing targets/cycles commit as editable drafts; hard failures install nothing.
    pub fn apply(&mut self, expected: Version, edits: Vec<Edit>) -> Result<Version, Error> {
        if expected != self.version {
            return Err(Error::Stale);
        }
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
                    candidate.values.set(id.node, Arc::new(value));
                    inputs = true;
                }
                Edit::Input(id, value) => {
                    if !matches!(*candidate.live_record(id)?.definition, Compiled::Input) {
                        return Err(Error::NotInput(id));
                    }
                    candidate.values.set(id.node, Arc::new(value));
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
                            Arc::make_mut(&mut r.reverse).remove(&id);
                            candidate.records.set(target.node, Arc::new(r));
                        }
                    }
                    changed.extend(record.reverse.iter().copied());
                    let mut retired = (*record).clone();
                    retired.live = false;
                    candidate.records.set(id.node, Arc::new(retired));
                    Arc::make_mut(&mut candidate.names).retain(|_, v| *v != id);
                    Arc::make_mut(&mut candidate.invalid).remove(&id);
                    candidate.live -= 1;
                    topology = true;
                }
                Edit::AddFormula(name, source) => {
                    if name.is_empty()
                        || !name.chars().all(|c| c.is_ascii_alphanumeric() || c == '_')
                        || name.as_bytes()[0].is_ascii_digit()
                        || candidate.names.contains_key(&name)
                    {
                        return Err(Error::InvalidTransfer);
                    }
                    let id = NodeId {
                        model: candidate.version.model,
                        node: candidate.next,
                    };
                    candidate.next = candidate.next.checked_add(1).ok_or(Error::Exhausted)?;
                    Arc::make_mut(&mut candidate.names).insert(name, id);
                    candidate.records.set(
                        id.node,
                        Arc::new(Record {
                            live: true,
                            definition: Arc::new(Compiled::Input),
                            reverse: Arc::new(BTreeSet::new()),
                        }),
                    );
                    let definition = candidate.compile(id, source);
                    candidate.replace(id, definition)?;
                    candidate.live += 1;
                    changed.insert(id);
                    topology = true;
                }
            }
        }
        candidate.check_quota()?;
        if topology {
            let _phase = crate::experiment_profile::scope(7);
            changed.extend(candidate.invalid.keys().copied());
            let mut budget = candidate.limits.validation_visits;
            for id in changed {
                if candidate.live_record(id).is_err() {
                    continue;
                }
                let error = candidate.validate_from(id, &mut budget)?;
                if candidate.invalid.get(&id) != error.as_ref() {
                    let invalid = Arc::make_mut(&mut candidate.invalid);
                    if let Some(error) = error {
                        invalid.insert(id, error);
                    } else {
                        invalid.remove(&id);
                    }
                }
            }
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
    pub fn capture(&self) -> Result<Capture, SnapshotError> {
        let _phase = crate::experiment_profile::scope(6);
        if let Some(error) = self.invalid.values().next() {
            return Err(error.clone());
        }
        Ok(Capture(self.clone()))
    }
}
#[derive(Clone)]
pub struct Capture(Graph);
impl Capture {
    pub fn version(&self) -> Version {
        self.0.version
    }
    pub fn evaluate(
        &self,
        outputs: &[NodeId],
    ) -> Result<BTreeMap<NodeId, Result<PivotDecimal, EvaluationError>>, EvaluationError> {
        let _phase = crate::experiment_profile::scope(8);
        let mut required = BTreeSet::new();
        let mut order = Vec::new();
        let mut pending: Vec<_> = outputs.iter().map(|&n| (n, false)).collect();
        while let Some((id, exit)) = pending.pop() {
            let record = self
                .0
                .live_record(id)
                .map_err(|_| EvaluationError::UnknownNode(id))?;
            if exit {
                order.push(id);
            } else if required.insert(id) {
                pending.push((id, true));
                pending.extend(record.definition.deps().iter().map(|&d| (d, false)));
            }
        }
        let mut values = BTreeMap::new();
        for id in order {
            let record = self.0.live_record(id).unwrap();
            let value = match &*record.definition {
                Compiled::Input => Ok(**self.0.values.get(id.node).unwrap()),
                Compiled::Formula {
                    binding: Ok((tokens, _)),
                    ..
                } => evaluate_expression(id, tokens, &values),
                _ => unreachable!("captures reject invalid drafts"),
            };
            values.insert(id, value);
        }
        Ok(values)
    }
}
