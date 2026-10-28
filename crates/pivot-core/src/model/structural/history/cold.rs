//! Separate research reconstruction seam; ordinary portable transfer is unchanged.
use super::*;
use crate::expression::AggregateKind;
use codec::{Blob, Reader, RecordKey, Writer, malformed};
fn node(r: &mut Reader<'_>, model: u64, floor: u64) -> Result<NodeId, ModelError> {
    let n = r.n()?;
    if n >= floor {
        return Err(malformed());
    }
    Ok(NodeId { model, node: n })
}
pub(super) fn write_error(w: &mut Writer, e: &SnapshotError) -> Result<(), ModelError> {
    match e {
        SnapshotError::Parse { node, error } => {
            w.n(0);
            w.n(node.node);
            w.parse_error(error);
        }
        SnapshotError::UnknownName { node, name } => {
            w.n(1);
            w.n(node.node);
            w.text(name);
        }
        SnapshotError::MissingReference { node, target } => {
            w.n(2);
            w.n(node.node);
            w.n(target.node);
        }
        SnapshotError::Unorderable { nodes } => {
            w.n(3);
            w.n(nodes.len() as u64);
            for n in nodes {
                w.n(n.node);
            }
        }
        _ => return Err(malformed()),
    }
    Ok(())
}
fn read_error(r: &mut Reader<'_>, model: u64, floor: u64) -> Result<SnapshotError, ModelError> {
    Ok(match r.n()? {
        0 => SnapshotError::Parse {
            node: node(r, model, floor)?,
            error: r.parse_error()?,
        },
        1 => SnapshotError::UnknownName {
            node: node(r, model, floor)?,
            name: r.text(128)?,
        },
        2 => SnapshotError::MissingReference {
            node: node(r, model, floor)?,
            target: node(r, model, floor)?,
        },
        3 => {
            let count = r.count(8192)?;
            let mut nodes = Vec::new();
            for _ in 0..count {
                nodes.push(node(r, model, floor)?);
            }
            SnapshotError::Unorderable { nodes }
        }
        _ => return Err(malformed()),
    })
}
pub(super) fn token(w: &mut Writer, t: &BoundToken) {
    match t {
        BoundToken::Control(c) => {
            w.n(6);
            w.n(c.code().into());
        }
        BoundToken::Decimal(d) => {
            w.n(0);
            w.decimal(*d);
        }
        BoundToken::Node(n) => {
            w.n(1);
            w.n(n.node);
        }
        BoundToken::Unary(op) => {
            w.n(2);
            w.n(match op {
                UnaryOp::Plus => 0,
                UnaryOp::Minus => 1,
            });
        }
        BoundToken::Binary(op) => {
            w.n(3);
            w.n(match op {
                BinaryOp::Add => 0,
                BinaryOp::Subtract => 1,
                BinaryOp::Multiply => 2,
                BinaryOp::Divide => 3,
            });
        }
        BoundToken::Sum(id) => {
            w.n(4);
            w.raw(&id.0);
        }
        BoundToken::Aggregate(k, id) => {
            w.n(5);
            w.n(match k {
                AggregateKind::Sum => 0,
                AggregateKind::Min => 1,
                AggregateKind::Max => 2,
                AggregateKind::Average => 3,
                AggregateKind::Count => 4,
            });
            w.raw(&id.0);
        }
    }
}
fn read_token(r: &mut Reader<'_>, model: u64, floor: u64) -> Result<BoundToken, ModelError> {
    Ok(match r.n()? {
        6 => BoundToken::Control(
            crate::expression::AggregateControl::from_code(
                u8::try_from(r.n()?).map_err(|_| malformed())?,
            )
            .ok_or_else(malformed)?,
        ),
        0 => BoundToken::Decimal(r.decimal()?),
        1 => BoundToken::Node(node(r, model, floor)?),
        2 => BoundToken::Unary(match r.n()? {
            0 => UnaryOp::Plus,
            1 => UnaryOp::Minus,
            _ => return Err(malformed()),
        }),
        3 => BoundToken::Binary(match r.n()? {
            0 => BinaryOp::Add,
            1 => BinaryOp::Subtract,
            2 => BinaryOp::Multiply,
            3 => BinaryOp::Divide,
            _ => return Err(malformed()),
        }),
        4 => BoundToken::Sum(selection::SelectionId(r.id()?)),
        5 => {
            let k = match r.n()? {
                0 => AggregateKind::Sum,
                1 => AggregateKind::Min,
                2 => AggregateKind::Max,
                3 => AggregateKind::Average,
                4 => AggregateKind::Count,
                _ => return Err(malformed()),
            };
            BoundToken::Aggregate(k, selection::SelectionId(r.id()?))
        }
        _ => return Err(malformed()),
    })
}
impl Graph {
    pub(in crate::model) fn cold_export(&self) -> Result<Blob, ModelError> {
        self.history_metrics().map_err(|_| malformed())?;
        if self.next > 8192 || self.next_port != 0 {
            return Err(malformed());
        }
        let _scratch = self
            .memory
            .reserve(
                self.selection_metrics()
                    .identities
                    .saturating_mul(128)
                    .saturating_add(self.selection_metrics().points.saturating_mul(128))
                    .saturating_add(4096),
            )
            .map_err(|_| malformed())?;
        let mut w = Writer::new(&self.memory)?;
        w.boundary(RecordKey::new(6, &[]));
        w.raw(b"PIVHGC01");
        w.text(&self.model_name);
        w.n(self.next);
        w.n(self.history.selections as u64);
        w.n(self.records.iter().count() as u64);
        for (slot, r) in self.records.iter() {
            w.boundary(RecordKey::number(7, slot));
            w.n(slot);
            w.flag(r.live);
            w.text(&r.name);
            match &*r.definition {
                Compiled::Blank => w.n(0),
                Compiled::NonNumeric => w.n(1),
                Compiled::Input => {
                    w.n(2);
                    w.decimal(**self.values.get(slot).ok_or_else(malformed)?);
                }
                Compiled::Formula { source, binding } => {
                    w.n(3);
                    w.text(source);
                    w.flag(binding.is_ok());
                    match binding {
                        Ok((tokens, _)) => {
                            w.n(tokens.len() as u64);
                            for t in tokens {
                                token(&mut w, t);
                            }
                        }
                        Err(e) => write_error(&mut w, e)?,
                    }
                }
                Compiled::CapturedScalar(_) => return Err(malformed()),
            }
        }
        let image = self.selection_image();
        w.boundary(RecordKey::new(8, &[]));
        w.n(image.identities.len() as u64);
        for (id, rect) in image.identities {
            w.boundary(RecordKey::new(9, &id.0));
            w.raw(&id.0);
            w.flag(rect.is_some());
            if let Some(r) = rect {
                w.raw(&r.space.0);
                for v in [r.first_row, r.last_row, r.first_column, r.last_column] {
                    w.n(v.into());
                }
            }
        }
        w.boundary(RecordKey::new(10, &[]));
        w.n(image.points.len() as u64);
        for (space, row, col, n) in image.points {
            let mut key = [0; 24];
            key[..16].copy_from_slice(&space.0);
            key[16..20].copy_from_slice(&row.to_be_bytes());
            key[20..].copy_from_slice(&col.to_be_bytes());
            w.boundary(RecordKey::new(11, &key));
            w.raw(&space.0);
            w.n(row.into());
            w.n(col.into());
            w.n(n.node);
        }
        w.boundary(RecordKey::new(12, &[]));
        w.n(self.invalid.len() as u64);
        for (id, e) in self.invalid.iter() {
            w.boundary(RecordKey::number(13, id.node));
            w.n(id.node);
            write_error(&mut w, e)?;
        }
        w.boundary(RecordKey::new(14, &[]));
        let invalidated = self.invalidated_selection_nodes();
        w.n(invalidated.len() as u64);
        for n in invalidated {
            w.n(n.node);
        }
        w.finish()
    }
    pub(in crate::model) fn cold_restore(
        &self,
        bytes: &[u8],
        cancel: &dyn Fn() -> bool,
        retire: impl FnOnce(Graph) + Send + 'static,
    ) -> Result<Self, Error> {
        let bad = || Error::InvalidTransfer;
        let _scratch = self
            .memory
            .reserve(bytes.len().saturating_mul(12).saturating_add(4096))?;
        let mut r = Reader::new(bytes)?;
        if r.take(8)? != b"PIVHGC01" {
            return Err(bad());
        }
        let name = r.text(128)?;
        let floor = r.n()?;
        let selections = usize::try_from(r.n()?).map_err(|_| bad())?;
        if floor > 8192
            || floor as usize > self.limits.nodes
            || selections > selection::MAX_IDENTITIES
        {
            return Err(bad());
        }
        let graph = Graph::empty(
            name,
            Revision {
                model: self.version.model,
                version: 0,
            },
            self.memory.clone(),
        )?;
        let mut owner = CandidateRoot {
            value: Some(graph),
            retire: Some(Box::new(retire)),
        };
        let root = owner.value.as_mut().unwrap();
        root.limits = self.limits;
        root.next = floor;
        root.history.selections = selections;
        let count = r.count(8192)?;
        let mut previous = None;
        for _ in 0..count {
            if cancel() {
                return Err(Error::Cancelled);
            }
            let id = node(&mut r, self.version.model, floor)?;
            if previous.is_some_and(|last| last >= id.node) {
                return Err(bad());
            }
            previous = Some(id.node);
            let live = r.flag()?;
            let name = r.text(128)?;
            let definition = match r.n()? {
                0 => Compiled::Blank,
                1 => Compiled::NonNumeric,
                2 => {
                    root.values.set(id.node, r.decimal()?)?;
                    Compiled::Input
                }
                3 => {
                    let source = r.text(4096)?;
                    let binding = if r.flag()? {
                        let mut tokens = Vec::new();
                        for _ in 0..r.count(4096)? {
                            tokens.push(read_token(&mut r, self.version.model, floor)?);
                        }
                        let dependencies = transfer::checked_dependencies(&tokens)?;
                        Ok((tokens, dependencies))
                    } else {
                        let e = read_error(&mut r, self.version.model, floor)?;
                        if error_node(&e) != Some(id) {
                            return Err(bad());
                        }
                        if let SnapshotError::Parse { error, .. } = &e
                            && (error.offset > source.len()
                                || !source.is_char_boundary(error.offset))
                        {
                            return Err(bad());
                        }
                        Err(e)
                    };
                    Compiled::Formula { source, binding }
                }
                _ => return Err(bad()),
            };
            root.history.payload += definition.heap_bytes();
            root.history.dependencies += definition.deps().len();
            root.history.names += name.capacity();
            root.history.records += 1;
            root.history.extent = id.node + 1;
            root.bytes += definition.bytes();
            root.max_name = root.max_name.max(name.len());
            if live {
                root.check_name(&name)?;
                root.names.insert(name.clone(), id)?;
                root.live += 1;
            }
            root.records.set(
                id.node,
                Record {
                    live,
                    name,
                    definition: root.memory.alloc(definition)?,
                    reverse: Index::new(root.memory.clone()),
                },
            )?;
        }
        let descriptors = r.count(selection::MAX_IDENTITIES)?;
        if descriptors > selections {
            return Err(bad());
        }
        let mut seen = BTreeSet::new();
        for _ in 0..descriptors {
            let id = selection::SelectionId(r.id()?);
            if !seen.insert(id) {
                return Err(bad());
            }
            if r.flag()? {
                let rect = selection::Rectangle {
                    space: selection::SpaceId(r.id()?),
                    first_row: r.u32()?,
                    last_row: r.u32()?,
                    first_column: r.u32()?,
                    last_column: r.u32()?,
                };
                root.edit_selection(|s| s.define(id, rect))?;
            } else {
                root.edit_selection(|s| s.retired_identity(id))?;
            }
        }
        let mut points = BTreeSet::new();
        for _ in 0..r.count(selection::MAX_POINTS)? {
            let space = selection::SpaceId(r.id()?);
            let row = r.u32()?;
            let col = r.u32()?;
            let n = node(&mut r, self.version.model, floor)?;
            if !points.insert((space, row, col)) || root.live_record(n).is_err() {
                return Err(bad());
            }
            root.edit_selection(|s| s.placement(space, row, col, Some(n)).map(|_| ()))?;
        }
        // Rebuild reverse edges and range consumers from canonical, already-bound definitions.
        let ids: Vec<_> = root
            .records
            .iter()
            .filter(|(_, record)| record.live)
            .map(|(slot, _)| NodeId {
                model: self.version.model,
                node: slot,
            })
            .collect();
        for id in ids {
            let definition = root.live_record(id)?.definition.clone();
            for target in definition.deps() {
                let mut record = (**root.record(*target).ok_or_else(bad)?).clone();
                record.reverse.insert(id, ())?;
                root.records.set(target.node, record)?;
            }
            for s in definition.selection_ids() {
                if !seen.contains(&s)
                    || root
                        .selections
                        .as_ref()
                        .and_then(|state| state.descriptor(s))
                        .is_none()
                {
                    return Err(bad());
                }
                root.edit_selection(|state| state.consumer(s, id, true))?;
            }
        }
        let mut invalid = BTreeSet::new();
        for _ in 0..r.count(8192)? {
            let id = node(&mut r, self.version.model, floor)?;
            if !invalid.insert(id) || root.live_record(id).is_err() {
                return Err(bad());
            }
            let e = read_error(&mut r, self.version.model, floor)?;
            root.invalid.insert(id, root.memory.alloc(e)?)?;
        }
        let mut invalidated = Vec::new();
        let mut seen_invalidated = BTreeSet::new();
        for _ in 0..r.count(selection::MAX_INVALIDATION_NODES)? {
            let n = node(&mut r, self.version.model, floor)?;
            if !seen_invalidated.insert(n) {
                return Err(bad());
            }
            invalidated.push(n);
        }
        if !invalidated.is_empty() {
            root.selection_invalidated = Some(root.memory.alloc(invalidated)?);
        }
        r.end()?;
        root.check_quota()?;
        // Serialized proof bits are never trusted; later edits revalidate using existing paths.
        root.proof = root
            .memory
            .alloc(Proof(std::sync::atomic::AtomicBool::new(false)))?;
        root.local_validation = false;
        root.import_invalid = true;
        root.history_activated(self)
    }
    pub(in crate::model) fn cold_node(&self, ordinal: u64) -> Result<NodeId, Error> {
        let id = NodeId {
            model: self.version.model,
            node: ordinal,
        };
        self.record(id).ok_or(Error::Unknown(id))?;
        Ok(id)
    }
    pub(in crate::model) fn cold_ordinal(&self, id: NodeId) -> Result<u64, Error> {
        self.record(id).ok_or(Error::Unknown(id))?;
        Ok(id.node)
    }
    pub(in crate::model) fn cold_nodes(&self) -> Vec<(u64, bool)> {
        self.records.iter().map(|(n, r)| (n, r.live)).collect()
    }
}
