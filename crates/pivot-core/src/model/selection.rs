//! Immutable generic coordinate selections. No worksheet callbacks or formula parsing.
use super::{
    NodeId,
    structural::{
        Index, Key,
        memory::{Admission, Footprint, Memory, Shared},
    },
};
use std::sync::atomic::{AtomicU64, Ordering};
pub const MAX_SELECTIONS: usize = 128;
pub const MAX_IDENTITIES: usize = 8192;
pub const MAX_POINTS: usize = 4096;
pub const MAX_CONSUMERS: usize = 4096;
pub const MAX_MEMBER_EDGES_PER_FORMULA: usize = 65_536;
pub const MAX_INVALIDATION_NODES: usize = 65_536;
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct SelectionId(pub [u8; 16]);
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct SpaceId(pub [u8; 16]);
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Rectangle {
    pub space: SpaceId,
    pub first_row: u32,
    pub last_row: u32,
    pub first_column: u32,
    pub last_column: u32,
}
impl Rectangle {
    pub fn valid(self) -> bool {
        self.space.0 != [0; 16]
            && self.first_row > 0
            && self.first_column > 0
            && self.first_row <= self.last_row
            && self.first_column <= self.last_column
    }
}
impl Key for SpaceId {}
impl Key for (SpaceId, u32, u32) {}
impl Key for SelectionId {}
impl Footprint for Rectangle {}
impl Footprint for Option<Rectangle> {}
impl Key for (SpaceId, u32) {}
impl Key for u32 {}
impl Footprint for SelectionId {}
#[derive(Clone)]
struct Columns(Index<u32, NodeId>);
impl Footprint for Columns {}
#[derive(Clone)]
struct Locations(Index<(SpaceId, u32, u32), ()>);
impl Footprint for Locations {}
#[derive(Clone)]
struct Consumers(Index<NodeId, ()>);
impl Footprint for Consumers {}
#[derive(Default)]
struct Counters {
    nodes: AtomicU64,
    candidates: AtomicU64,
    members: AtomicU64,
}
impl Footprint for Counters {}
#[derive(Clone, Copy, Debug, Default)]
pub struct Metrics {
    pub visited_index_nodes: u64,
    pub selection_candidates: u64,
    pub emitted_members: u64,
    pub points: usize,
    pub selections: usize,
    pub identities: usize,
    pub consumers: usize,
    pub invalidated_nodes: usize,
}
#[derive(Clone)]
pub(crate) struct State {
    #[cfg(feature = "history-experiment")]
    record_trace: Option<Index<super::integration::history_codec::RecordKey, ()>>,
    #[cfg(feature = "history-experiment")]
    record_writes: usize,
    #[cfg(feature = "history-experiment")]
    record_scans: usize,
    rows: Index<(SpaceId, u32), Shared<Columns>>,
    locations: Index<NodeId, Shared<Locations>>,
    coverage: Index<SpaceId, Shared<Coverage>>,
    descriptors: Index<SelectionId, Option<Rectangle>>,
    consumers: Index<SelectionId, Shared<Consumers>>,
    points: usize,
    point_limit: usize,
    live: usize,
    uses: usize,
    memory: Memory,
    counters: Shared<Counters>,
}
#[derive(Debug)]
pub enum Error {
    Quota,
    Invalid,
    Admission,
    Cancelled,
}
impl From<Admission> for Error {
    fn from(_: Admission) -> Self {
        Self::Admission
    }
}
impl State {
    pub fn new(memory: Memory) -> Result<Self, Error> {
        Self::with_point_limit(memory, MAX_POINTS)
    }
    pub(crate) fn with_point_limit(memory: Memory, point_limit: usize) -> Result<Self, Error> {
        Ok(Self {
            #[cfg(feature = "history-experiment")]
            record_trace: None,
            #[cfg(feature = "history-experiment")]
            record_writes: 0,
            #[cfg(feature = "history-experiment")]
            record_scans: 0,
            rows: Index::new(memory.clone()),
            locations: Index::new(memory.clone()),
            coverage: Index::new(memory.clone()),
            descriptors: Index::new(memory.clone()),
            consumers: Index::new(memory.clone()),
            points: 0,
            point_limit,
            live: 0,
            uses: 0,
            counters: memory.alloc(Counters::default())?,
            memory,
        })
    }
    pub fn define(&mut self, id: SelectionId, rectangle: Rectangle) -> Result<(), Error> {
        if id.0 == [0; 16] || !rectangle.valid() {
            return Err(Error::Invalid);
        }
        if let Some(old) = self.descriptors.get(&id) {
            return if *old == Some(rectangle) {
                Ok(())
            } else {
                Err(Error::Invalid)
            };
        }
        if self.live >= MAX_SELECTIONS * 2 || self.descriptors.len() >= MAX_IDENTITIES {
            return Err(Error::Quota);
        }
        let mut coverage = self
            .coverage
            .get(&rectangle.space)
            .map(|c| (**c).clone())
            .unwrap_or_default();
        coverage.root = interval_insert(&self.memory, &coverage.root, id, rectangle)?;
        self.coverage
            .insert(rectangle.space, self.memory.alloc(coverage)?)?;
        #[cfg(feature = "history-experiment")]
        self.history_note(super::integration::history_codec::RecordKey::new(9, &id.0))?;
        self.descriptors.insert(id, Some(rectangle))?;
        self.live += 1;
        Ok(())
    }
    pub fn retired_identity(&mut self, id: SelectionId) -> Result<(), Error> {
        if id.0 == [0; 16] || self.descriptors.contains_key(&id) {
            return Err(Error::Invalid);
        }
        if self.descriptors.len() >= MAX_IDENTITIES {
            return Err(Error::Quota);
        }
        #[cfg(feature = "history-experiment")]
        self.history_note(super::integration::history_codec::RecordKey::new(9, &id.0))?;
        self.descriptors.insert(id, None)?;
        Ok(())
    }
    #[cfg(feature = "history-experiment")]
    pub(crate) fn history_has_identity(&self, id: SelectionId) -> bool {
        self.descriptors.contains_key(&id)
    }
    #[cfg(feature = "history-experiment")]
    pub(crate) fn history_weight(&self) -> u128 {
        use super::structural::memory::history_allocation_header as h;
        let rows = self.rows.len() as u128;
        let loc = self.locations.len() as u128;
        let consumers = self.consumers.len() as u128;
        let coverage = self.coverage.len() as u128;
        h::<Self>() as u128
            + h::<Counters>() as u128
            + rows
                * (Index::<(SpaceId, u32), Shared<Columns>>::history_entry_header()
                    + h::<Columns>()) as u128
            + self.points as u128 * Index::<u32, NodeId>::history_entry_header() as u128
            + loc
                * (Index::<NodeId, Shared<Locations>>::history_entry_header() + h::<Locations>())
                    as u128
            + self.points as u128 * Index::<(SpaceId, u32, u32), ()>::history_entry_header() as u128
            + self.descriptors.len() as u128
                * Index::<SelectionId, Option<Rectangle>>::history_entry_header() as u128
            + consumers
                * (Index::<SelectionId, Shared<Consumers>>::history_entry_header()
                    + h::<Consumers>()) as u128
            + self.uses as u128 * Index::<NodeId, ()>::history_entry_header() as u128
            + coverage
                * (Index::<SpaceId, Shared<Coverage>>::history_entry_header() + h::<Coverage>())
                    as u128
            + self.live as u128 * h::<Interval>() as u128
    }
    pub fn descriptor(&self, id: SelectionId) -> Option<Rectangle> {
        self.descriptors.get(&id).copied().flatten()
    }
    pub fn identities(&self) -> impl Iterator<Item = (SelectionId, Option<Rectangle>)> + '_ {
        self.descriptors.iter().map(|(id, r)| (*id, *r))
    }
    pub fn placement(
        &mut self,
        space: SpaceId,
        row: u32,
        column: u32,
        node: Option<NodeId>,
    ) -> Result<bool, Error> {
        if space.0 == [0; 16] || row == 0 || column == 0 {
            return Err(Error::Invalid);
        }
        let key = (space, row);
        let mut cols = self
            .rows
            .get(&key)
            .map(|c| (**c).clone())
            .unwrap_or_else(|| Columns(Index::new(self.memory.clone())));
        let old = cols.0.get(&column).copied();
        if old == node {
            return Ok(false);
        }
        for (value, add) in [(old, false), (node, true)] {
            if let Some(value) = value {
                let mut locations = self
                    .locations
                    .get(&value)
                    .map(|p| (**p).clone())
                    .unwrap_or_else(|| Locations(Index::new(self.memory.clone())));
                if add {
                    locations.0.insert((space, row, column), ())?;
                } else {
                    locations.0.remove(&(space, row, column))?;
                }
                if locations.0.is_empty() {
                    self.locations.remove(&value)?;
                } else {
                    self.locations
                        .insert(value, self.memory.alloc(locations)?)?;
                }
            }
        }
        #[cfg(feature = "history-experiment")]
        self.history_note(history_point_key(space, row, column))?;
        let exists = old.is_some();
        match node {
            Some(node) => {
                if !exists && self.points >= self.point_limit {
                    return Err(Error::Quota);
                }
                cols.0.insert(column, node)?;
                if !exists {
                    self.points += 1
                }
            }
            None => {
                cols.0.remove(&column)?;
                if exists {
                    self.points -= 1
                }
            }
        }
        if cols.0.is_empty() {
            self.rows.remove(&key)?;
        } else {
            self.rows.insert(key, self.memory.alloc(cols)?)?;
        }
        Ok(true)
    }
    pub fn consumer(&mut self, id: SelectionId, node: NodeId, add: bool) -> Result<(), Error> {
        if self.descriptor(id).is_none() {
            return Err(Error::Invalid);
        }
        let mut users = self
            .consumers
            .get(&id)
            .map(|c| (**c).clone())
            .unwrap_or_else(|| Consumers(Index::new(self.memory.clone())));
        if add && !users.0.contains_key(&node) {
            if self.uses >= MAX_CONSUMERS {
                return Err(Error::Quota);
            }
            users.0.insert(node, ())?;
            self.uses += 1;
        }
        if !add && users.0.contains_key(&node) {
            users.0.remove(&node)?;
            self.uses -= 1;
        }
        if users.0.is_empty() {
            self.consumers.remove(&id)?;
        } else {
            self.consumers.insert(id, self.memory.alloc(users)?)?;
        }
        Ok(())
    }
    pub fn retire_unused(&mut self) -> Result<(), Error> {
        #[cfg(feature = "history-experiment")]
        if self.record_trace.is_some() {
            self.record_scans += self.descriptors.len();
        }
        let ids: Vec<_> = self
            .descriptors
            .iter()
            .filter(|(id, r)| r.is_some() && !self.consumers.contains_key(id))
            .map(|(id, _)| *id)
            .collect();
        for id in ids {
            let rectangle = self.descriptor(id).ok_or(Error::Invalid)?;
            let mut coverage =
                (**self.coverage.get(&rectangle.space).ok_or(Error::Invalid)?).clone();
            coverage.root =
                interval_remove(&self.memory, &coverage.root, (rectangle.first_row, id))?;
            if coverage.root.is_none() {
                self.coverage.remove(&rectangle.space)?;
            } else {
                self.coverage
                    .insert(rectangle.space, self.memory.alloc(coverage)?)?;
            }
            #[cfg(feature = "history-experiment")]
            self.history_note(super::integration::history_codec::RecordKey::new(9, &id.0))?;
            self.descriptors.insert(id, None)?;
            self.live -= 1;
        }
        if self.live > MAX_SELECTIONS {
            return Err(Error::Quota);
        }
        Ok(())
    }
    pub fn members(
        &self,
        id: SelectionId,
        budget: &mut usize,
        cancel: &dyn Fn() -> bool,
    ) -> Result<Vec<NodeId>, Error> {
        if cancel() {
            return Err(Error::Cancelled);
        }
        let r = self.descriptor(id).ok_or(Error::Invalid)?;
        let mut out = Vec::new();
        let mut failed = false;
        let rows = self.rows.visit_range(
            &(r.space, r.first_row),
            &(r.space, r.last_row),
            |_, cols| {
                let columns = cols
                    .0
                    .visit_range(&r.first_column, &r.last_column, |_, node| {
                        if *budget == 0 || cancel() {
                            failed = true;
                            return false;
                        }
                        *budget -= 1;
                        out.push(*node);
                        true
                    });
                self.counters
                    .nodes
                    .fetch_add(columns as u64, Ordering::Relaxed);
                !failed
            },
        );
        self.counters
            .nodes
            .fetch_add(rows as u64, Ordering::Relaxed);
        self.counters
            .members
            .fetch_add(out.len() as u64, Ordering::Relaxed);
        if cancel() {
            Err(Error::Cancelled)
        } else if failed {
            Err(Error::Quota)
        } else {
            Ok(out)
        }
    }
    pub fn covering(&self, space: SpaceId, row: u32, column: u32) -> Vec<SelectionId> {
        let mut out = Vec::new();
        let mut nodes = 0;
        let mut candidates = 0;
        if let Some(coverage) = self.coverage.get(&space) {
            interval_query(
                &coverage.root,
                row,
                column,
                &mut out,
                &mut nodes,
                &mut candidates,
            );
        }
        self.counters.nodes.fetch_add(nodes, Ordering::Relaxed);
        self.counters
            .candidates
            .fetch_add(candidates, Ordering::Relaxed);
        out
    }
    pub fn consumers_of(&self, selection: SelectionId) -> Vec<NodeId> {
        self.consumers
            .get(&selection)
            .map(|u| u.0.iter().map(|(n, _)| *n).collect())
            .unwrap_or_default()
    }
    pub fn containing_node(&self, node: NodeId) -> Vec<SelectionId> {
        let mut out = std::collections::BTreeSet::new();
        if let Some(locations) = self.locations.get(&node) {
            for ((space, row, column), _) in locations.0.iter() {
                out.extend(self.covering(*space, *row, *column));
            }
        }
        out.into_iter().collect()
    }
    pub fn points(&self) -> impl Iterator<Item = (SpaceId, u32, u32, NodeId)> + '_ {
        self.rows.iter().flat_map(|((space, row), cols)| {
            cols.0
                .iter()
                .map(move |(col, node)| (*space, *row, *col, *node))
        })
    }
    pub fn metrics(&self) -> Metrics {
        Metrics {
            visited_index_nodes: self.counters.nodes.load(Ordering::Relaxed),
            selection_candidates: self.counters.candidates.load(Ordering::Relaxed),
            emitted_members: self.counters.members.load(Ordering::Relaxed),
            points: self.points,
            selections: self.live,
            identities: self.descriptors.len(),
            consumers: self.uses,
            invalidated_nodes: 0,
        }
    }
}

#[derive(Clone, Debug, Default)]
pub struct Image {
    pub identities: Vec<(SelectionId, Option<Rectangle>)>,
    pub points: Vec<(SpaceId, u32, u32, NodeId)>,
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Dependency {
    Scalar(NodeId),
    Selection(SelectionId),
}
impl super::WorkingModel {
    pub fn selection_image(&self) -> Image {
        self.integrated.graph().selection_image()
    }
    pub fn selection_metrics(&self) -> Metrics {
        self.integrated.graph().selection_metrics()
    }
    pub fn invalidated_selection_nodes(&self) -> &[NodeId] {
        self.integrated.graph().invalidated_selection_nodes()
    }
    pub fn typed_dependencies(&self, node: NodeId) -> Result<Vec<Dependency>, super::ModelError> {
        self.integrated
            .graph()
            .typed_dependencies(node)
            .map_err(|_| super::ModelError::Backend("typed dependency inspection"))
    }
}

// Persistent augmented AVL interval index. One coverage record per descriptor,
// independent of rectangle area; the outer index separates generic spaces.
type IntervalLink = Option<Shared<Interval>>;
struct Interval {
    id: SelectionId,
    rectangle: Rectangle,
    left: IntervalLink,
    right: IntervalLink,
    height: i16,
    max_end: u32,
}
impl Footprint for Interval {}
#[derive(Clone, Default)]
struct Coverage {
    root: IntervalLink,
}
impl Footprint for Coverage {}
fn height(link: &IntervalLink) -> i16 {
    link.as_ref().map_or(0, |n| n.height)
}
fn interval_node(
    memory: &Memory,
    id: SelectionId,
    r: Rectangle,
    left: IntervalLink,
    right: IntervalLink,
) -> Result<Shared<Interval>, Admission> {
    let max_end = r
        .last_row
        .max(left.as_ref().map_or(0, |n| n.max_end))
        .max(right.as_ref().map_or(0, |n| n.max_end));
    let h = 1 + height(&left).max(height(&right));
    memory.alloc(Interval {
        id,
        rectangle: r,
        left,
        right,
        height: h,
        max_end,
    })
}
fn balance(memory: &Memory, node: Shared<Interval>) -> Result<Shared<Interval>, Admission> {
    if height(&node.left) - height(&node.right) > 1 {
        let l = node.left.as_ref().unwrap();
        if height(&l.left) < height(&l.right) {
            let lr = l.right.as_ref().unwrap();
            let left = interval_node(memory, l.id, l.rectangle, l.left.clone(), lr.left.clone())?;
            let right = interval_node(
                memory,
                node.id,
                node.rectangle,
                lr.right.clone(),
                node.right.clone(),
            )?;
            return interval_node(memory, lr.id, lr.rectangle, Some(left), Some(right));
        }
        let right = interval_node(
            memory,
            node.id,
            node.rectangle,
            l.right.clone(),
            node.right.clone(),
        )?;
        return interval_node(memory, l.id, l.rectangle, l.left.clone(), Some(right));
    }
    if height(&node.right) - height(&node.left) > 1 {
        let r = node.right.as_ref().unwrap();
        if height(&r.right) < height(&r.left) {
            let rl = r.left.as_ref().unwrap();
            let left = interval_node(
                memory,
                node.id,
                node.rectangle,
                node.left.clone(),
                rl.left.clone(),
            )?;
            let right =
                interval_node(memory, r.id, r.rectangle, rl.right.clone(), r.right.clone())?;
            return interval_node(memory, rl.id, rl.rectangle, Some(left), Some(right));
        }
        let left = interval_node(
            memory,
            node.id,
            node.rectangle,
            node.left.clone(),
            r.left.clone(),
        )?;
        return interval_node(memory, r.id, r.rectangle, Some(left), r.right.clone());
    }
    Ok(node)
}
fn interval_insert(
    memory: &Memory,
    root: &IntervalLink,
    id: SelectionId,
    r: Rectangle,
) -> Result<IntervalLink, Admission> {
    let Some(n) = root else {
        return Ok(Some(interval_node(memory, id, r, None, None)?));
    };
    let (left, right) = if (r.first_row, id) < (n.rectangle.first_row, n.id) {
        (interval_insert(memory, &n.left, id, r)?, n.right.clone())
    } else {
        (n.left.clone(), interval_insert(memory, &n.right, id, r)?)
    };
    Ok(Some(balance(
        memory,
        interval_node(memory, n.id, n.rectangle, left, right)?,
    )?))
}
fn interval_remove(
    memory: &Memory,
    root: &IntervalLink,
    key: (u32, SelectionId),
) -> Result<IntervalLink, Admission> {
    let Some(n) = root else { return Ok(None) };
    let current = (n.rectangle.first_row, n.id);
    let updated = if key < current {
        interval_node(
            memory,
            n.id,
            n.rectangle,
            interval_remove(memory, &n.left, key)?,
            n.right.clone(),
        )?
    } else if key > current {
        interval_node(
            memory,
            n.id,
            n.rectangle,
            n.left.clone(),
            interval_remove(memory, &n.right, key)?,
        )?
    } else {
        if n.left.is_none() {
            return Ok(n.right.clone());
        }
        if n.right.is_none() {
            return Ok(n.left.clone());
        }
        let mut next = n.right.as_ref().unwrap();
        while let Some(left) = &next.left {
            next = left;
        }
        interval_node(
            memory,
            next.id,
            next.rectangle,
            n.left.clone(),
            interval_remove(memory, &n.right, (next.rectangle.first_row, next.id))?,
        )?
    };
    Ok(Some(balance(memory, updated)?))
}
fn interval_query(
    root: &IntervalLink,
    row: u32,
    column: u32,
    out: &mut Vec<SelectionId>,
    nodes: &mut u64,
    candidates: &mut u64,
) {
    let Some(n) = root else { return };
    *nodes += 1;
    if n.max_end < row {
        return;
    }
    interval_query(&n.left, row, column, out, nodes, candidates);
    if n.rectangle.first_row <= row {
        if row <= n.rectangle.last_row {
            *candidates += 1;
            if n.rectangle.first_column <= column && column <= n.rectangle.last_column {
                out.push(n.id)
            }
        }
        interval_query(&n.right, row, column, out, nodes, candidates);
    }
}

#[cfg(feature = "history-experiment")]
pub(crate) fn history_point_key(
    space: SpaceId,
    row: u32,
    column: u32,
) -> super::integration::history_codec::RecordKey {
    let mut key = [0; 24];
    key[..16].copy_from_slice(&space.0);
    key[16..20].copy_from_slice(&row.to_be_bytes());
    key[20..].copy_from_slice(&column.to_be_bytes());
    super::integration::history_codec::RecordKey::new(11, &key)
}
#[cfg(feature = "history-experiment")]
impl State {
    pub(crate) fn history_start_records(&mut self) {
        self.record_trace = Some(Index::new(self.memory.clone()));
        self.record_writes = 0;
        self.record_scans = 0;
    }
    pub(crate) fn history_enable_records(&mut self) {
        if self.record_trace.is_none() {
            self.history_start_records();
        }
    }
    pub(crate) fn history_end_records(&mut self) {
        self.record_trace = None;
        self.record_writes = 0;
        self.record_scans = 0;
    }
    fn history_note(
        &mut self,
        key: super::integration::history_codec::RecordKey,
    ) -> Result<(), Error> {
        if let Some(trace) = &mut self.record_trace {
            trace.insert(key, ())?;
            self.record_writes += 1;
        }
        Ok(())
    }
    pub(crate) fn history_record_scans(&self) -> usize {
        self.record_scans
    }
    pub(crate) fn history_record_writes(&self) -> usize {
        self.record_writes
    }
    pub(crate) fn history_record_keys(
        &self,
    ) -> impl Iterator<Item = super::integration::history_codec::RecordKey> + '_ {
        self.record_trace
            .iter()
            .flat_map(|t| t.iter().map(|(k, _)| *k))
    }
    pub(crate) fn history_descriptor(&self, id: SelectionId) -> Option<Option<Rectangle>> {
        self.descriptors.get(&id).copied()
    }
    pub(crate) fn history_point(&self, space: SpaceId, row: u32, col: u32) -> Option<NodeId> {
        self.rows
            .get(&(space, row))
            .and_then(|r| r.0.get(&col))
            .copied()
    }
}
