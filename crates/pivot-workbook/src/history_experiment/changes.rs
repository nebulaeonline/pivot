//! Feature-only mutation-local authoritative records; no journal integration.
use super::*;
use pivot_core::model::integration::{
    CancellationToken,
    history_codec::{Blob, RecordKey, Writer},
    resources::{Charge, Index, Memory},
};
use sha2::{Digest, Sha256};
use std::{
    collections::{BTreeMap, BTreeSet},
    time::Instant,
};
fn bad() -> Error {
    Error::Graph("invalid mutation-local record candidate".into())
}
fn poll(cancel: &CancellationToken) -> Result<()> {
    if cancel.is_cancelled() {
        Err(bad())
    } else {
        Ok(())
    }
}
/// Authoring input is consumed only to stage the normal private transaction. It is
/// never recorded/replayed: operations below hold resolved canonical record bytes.
pub enum Mutation<'a> {
    Edits(&'a [Edit]),
    Fill(Rectangle, FillDirection),
    Paste {
        rectangle: Rectangle,
        bytes: &'a [u8],
        native: bool,
    },
    Duplicate(&'a [(Position, Position)]),
}
#[derive(Default, Debug, Clone)]
pub struct Metrics {
    /// Selected original/final cell record reads (not AVL traversal nodes).
    pub workbook_keys_visited: usize,
    /// Selected original/final core record lookups, including fixed metadata.
    pub core_keys_visited: usize,
    pub mutation_writes: usize,
    pub mutation_record_reads: usize,
    pub mutation_record_scans: usize,
    pub membership_index_visits: u64,
    pub selection_candidates: u64,
    pub selection_members: u64,
    pub emitted_records: usize,
    pub compared_bytes: usize,
    pub emitted_bytes: usize,
    pub local_ns: u128,
    pub scratch_admission: usize,
}
/// One final operation per stable logical record key, relative to the original
/// committed state. Encodings are existing typed cold records, never commands.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum RecordKind {
    WorkbookAuthority,
    Ledger,
    NodeReservation,
    SelectionLedger,
    SelectionReservation,
    CoreMetadata,
    Definition,
    SelectionMetadata,
    Selection,
    PlacementMetadata,
    Placement,
    ValidationMetadata,
    Validation,
    Invalidation,
    CellDirectory,
    Cell,
}
pub struct RecordOperation {
    key: RecordKey,
    before: Option<Vec<u8>>,
    after: Option<Vec<u8>>,
}
impl RecordOperation {
    pub fn kind(&self) -> RecordKind {
        use RecordKind::*;
        match self.key.0[0] {
            0 => WorkbookAuthority,
            1 => Ledger,
            2 => NodeReservation,
            3 => SelectionLedger,
            4 => SelectionReservation,
            6 => CoreMetadata,
            7 => Definition,
            8 => SelectionMetadata,
            9 => Selection,
            10 => PlacementMetadata,
            11 => Placement,
            12 => ValidationMetadata,
            13 => Validation,
            14 => Invalidation,
            15 => CellDirectory,
            16 => Cell,
            _ => unreachable!("sealed semantic record key"),
        }
    }

    pub fn key(&self) -> RecordKey {
        self.key
    }
    pub fn before(&self) -> Option<&[u8]> {
        self.before.as_deref()
    }
    pub fn after(&self) -> Option<&[u8]> {
        self.after.as_deref()
    }
}
pub(super) struct Reservations {
    pub instance: Uuid,
    pub nodes: (u64, u64),
    pub selections: (usize, usize),
    pub added_nodes: Vec<(u64, (CellId, GraphNodeId))>,
    pub added_selections: Vec<selection::SelectionId>,
}
pub struct PreparedRecordChanges {
    base: Expected,
    before_content: Uuid,
    candidate: Workbook,
    operations: Vec<RecordOperation>,
    guards: Vec<(RecordKey, [u8; 32])>,
    reservations: Reservations,
    metrics: Metrics,
    _charge: Charge,
}
impl PreparedRecordChanges {
    pub fn metrics(&self) -> &Metrics {
        &self.metrics
    }
    pub fn candidate(&self) -> &Workbook {
        &self.candidate
    }
    pub fn operations(&self) -> &[RecordOperation] {
        &self.operations
    }
    /// Feature-only in-memory pair publication, not a persisted history decision.
    /// All fallible admission/validation precedes either authority change.
    pub fn publish(self, book: &mut Workbook, ledger: &mut ColdLedger) -> Result<()> {
        book.check(self.base)?;
        if book.history_content_id() != self.before_content {
            return Err(bad());
        }
        ledger.append_local_records(
            book,
            self.candidate.history_content_id(),
            &self.reservations,
        )?;
        *book = self.candidate;
        Ok(())
    }
}
fn spans(blob: &Blob) -> BTreeMap<RecordKey, &[u8]> {
    blob.records
        .iter()
        .map(|r| (r.key, &blob.bytes[r.range.clone()]))
        .collect()
}
fn book_records(
    book: &Workbook,
    ledger: &ColdLedger,
    keys: &[RecordKey],
    additions: &BTreeMap<u64, (CellId, GraphNodeId)>,
    selections: &BTreeSet<selection::SelectionId>,
    after: bool,
) -> Result<Blob> {
    let memory = book.core.allocation_domain().map_err(graph_error)?;
    let mut w = Writer::new(&memory).map_err(graph_error)?;
    let metrics = book.history_metrics()?;
    for key in keys {
        match key.0[0] {
            0 => {
                w.boundary(*key);
                w.raw(b"PIVHWB01");
                w.raw(book.worksheet.uuid().as_bytes());
                w.raw(book.graph_id.as_bytes());
                w.raw(book.rows.namespace.as_bytes());
                w.n(book.rows.length.into());
                w.raw(book.columns.namespace.as_bytes());
                w.n(book.columns.length.into());
                w.raw(book.history_content.as_bytes());
            }
            1 => {
                w.boundary(*key);
                w.raw(ledger.instance.as_bytes());
                let floor = if after {
                    metrics.next_node
                } else {
                    ledger.floor
                };
                w.n(floor);
                w.n(if after {
                    metrics.lifetime_selections as u64
                } else {
                    ledger.selection_floor as u64
                });
                w.n(floor);
            }
            2 => {
                let n = u64::from_be_bytes(key.0[1..9].try_into().unwrap());
                if let Some((cell, node)) = ledger
                    .identities
                    .get(&n)
                    .or_else(|| if after { additions.get(&n) } else { None })
                {
                    w.boundary(*key);
                    w.n(n);
                    w.raw(cell.uuid().as_bytes());
                    w.raw(node.uuid().as_bytes());
                }
            }
            3 => {
                w.boundary(*key);
                w.n(if after {
                    metrics.lifetime_selections as u64
                } else {
                    ledger.selection_floor as u64
                });
            }
            4 => {
                let id = selection::SelectionId(key.0[1..17].try_into().unwrap());
                if ledger.selections.contains(&id) || (after && selections.contains(&id)) {
                    w.boundary(*key);
                    w.raw(&id.0);
                }
            }
            15 => {
                w.boundary(*key);
                w.n(book.persistent.records.len() as u64);
            }
            16 => {
                let id = CellId::from_uuid(Uuid::from_bytes(key.0[1..17].try_into().unwrap()));
                if let Some(record) = book.persistent.records.get(&id) {
                    let binding = book.persistent.bindings.get(&id).ok_or_else(bad)?;
                    let n = book
                        .core
                        .history_cold_ordinal(binding.runtime)
                        .map_err(graph_error)?;
                    if ledger.identities.get(&n).or_else(|| additions.get(&n))
                        != Some(&(id, binding.portable))
                    {
                        return Err(bad());
                    }
                    w.boundary(*key);
                    w.raw(id.uuid().as_bytes());
                    w.raw(binding.portable.uuid().as_bytes());
                    w.n(n);
                    w.flag(record.position.is_some());
                    if let (Some(position), Some(content)) = (record.position, &record.content) {
                        super::cold::pos(&mut w, position);
                        super::cold::content(&mut w, content)?;
                    } else if record.position.is_some() || record.content.is_some() {
                        return Err(bad());
                    }
                }
            }
            _ => (),
        }
    }
    w.finish().map_err(graph_error)
}
use pivot_core::model::selection;
impl Workbook {
    pub fn prepare_record_changes(
        &self,
        expected: Expected,
        ledger: &ColdLedger,
        mutation: Mutation<'_>,
        cancel: &CancellationToken,
    ) -> Result<Option<PreparedRecordChanges>> {
        self.check(expected)?;
        poll(cancel)?;
        self.history_weight()?;
        let before = self.history_metrics()?;
        if ledger.validated_content != Some(self.history_content_id())
            || ledger.session != self.session
            || ledger.worksheet != self.worksheet
            || ledger.graph != self.graph_id
            || ledger.floor != before.next_node
            || ledger.selection_floor != before.lifetime_selections
        {
            return Err(bad());
        }
        let select_before = self.core.selection_metrics();
        let start = Instant::now();
        let memory = self.core.allocation_domain().map_err(graph_error)?;
        let mut candidate = self.stage_history_candidate(expected)?;
        candidate.history_recording = Some(Index::new(memory.clone()));
        match mutation {
            Mutation::Edits(edits) => {
                candidate.apply_cancelled(expected, edits, cancel)?;
            }
            Mutation::Fill(rectangle, direction) => {
                candidate.fill(expected, rectangle, direction, cancel)?;
            }
            Mutation::Paste {
                rectangle,
                bytes,
                native,
            } => {
                candidate.paste_block(expected, rectangle, bytes, native, cancel)?;
            }
            Mutation::Duplicate(pairs) => {
                candidate.duplicate_pairs(expected, pairs.iter().copied(), cancel)?;
            }
        }
        poll(cancel)?;
        if candidate.expected() == expected {
            return Ok(None);
        }
        candidate.history_weight()?;
        let cells = candidate.history_recording.take().ok_or_else(bad)?;
        let core_keys = candidate.core.history_record_keys().map_err(graph_error)?;
        let scratch_bytes =
            (cells.len() * 2 + core_keys.entries.len() + 4).saturating_mul(512) + 8192;
        let _scratch = memory.reserve(scratch_bytes).map_err(|_| Error::Limit)?;
        let mut keys: BTreeSet<_> = core_keys.entries.iter().copied().collect();
        keys.extend([0, 1, 3, 15].map(|tag| RecordKey::new(tag, &[])));
        let mut additions = BTreeMap::new();
        let mut claims = BTreeSet::new();
        for (cell, _) in cells.iter() {
            poll(cancel)?;
            keys.insert(RecordKey::new(16, cell.uuid().as_bytes()));
            let binding = candidate.persistent.bindings.get(cell).ok_or_else(bad)?;
            let n = candidate
                .core
                .history_cold_ordinal(binding.runtime)
                .map_err(graph_error)?;
            keys.insert(RecordKey::number(2, n));
            match ledger.identities.get(&n) {
                Some(old) if *old != (*cell, binding.portable) => return Err(bad()),
                Some(_) => (),
                None => {
                    additions.insert(n, (*cell, binding.portable));
                }
            }
        }
        for key in &core_keys.entries {
            if key.0[0] == 9 {
                let id = selection::SelectionId(key.0[1..17].try_into().unwrap());
                keys.insert(RecordKey::new(4, &id.0));
                if !ledger.selections.contains(&id) {
                    claims.insert(id);
                }
            }
        }
        let after = candidate.history_metrics()?;
        if after.next_node != ledger.floor + additions.len() as u64
            || after.lifetime_selections != ledger.selection_floor + claims.len()
        {
            return Err(bad());
        }
        let keys: Vec<_> = keys.into_iter().collect();
        let old_core = self
            .core
            .history_export_records(&core_keys.entries)
            .map_err(graph_error)?;
        let new_core = candidate
            .core
            .history_export_records(&core_keys.entries)
            .map_err(graph_error)?;
        let old_book = book_records(self, ledger, &keys, &additions, &claims, false)?;
        let new_book = book_records(&candidate, ledger, &keys, &additions, &claims, true)?;
        let total = old_core.bytes.len()
            + new_core.bytes.len()
            + old_book.bytes.len()
            + new_book.bytes.len();
        let charge = memory
            .reserve(total.saturating_mul(2) + keys.len() * 256 + 4096)
            .map_err(|_| Error::Limit)?;
        let mut old = spans(&old_core);
        old.extend(spans(&old_book));
        let mut new = spans(&new_core);
        new.extend(spans(&new_book));
        let guards = [0, 1, 3, 6]
            .into_iter()
            .map(|tag| {
                let key = RecordKey::new(tag, &[]);
                Ok((key, Sha256::digest(old.get(&key).ok_or_else(bad)?).into()))
            })
            .collect::<Result<Vec<_>>>()?;
        let mut operations = Vec::new();
        let mut emitted_bytes = 0;
        for key in keys {
            poll(cancel)?;
            let a = old.get(&key).copied();
            let b = new.get(&key).copied();
            if a != b {
                emitted_bytes += b.map_or(0, |v| v.len());
                operations.push(RecordOperation {
                    key,
                    before: a.map(<[u8]>::to_vec),
                    after: b.map(<[u8]>::to_vec),
                });
            }
        }
        candidate.core = candidate
            .core
            .history_recording_stage(false)
            .map_err(graph_error)?;
        poll(cancel)?;
        let select_after = candidate.core.selection_metrics();
        let metrics = Metrics {
            workbook_keys_visited: cells.len() * 2,
            core_keys_visited: core_keys.entries.len() * 2,
            mutation_writes: core_keys.mutation_writes,
            mutation_record_reads: core_keys.mutation_record_reads,
            mutation_record_scans: core_keys.mutation_record_scans,
            membership_index_visits: select_after
                .visited_index_nodes
                .saturating_sub(select_before.visited_index_nodes),
            selection_candidates: select_after
                .selection_candidates
                .saturating_sub(select_before.selection_candidates),
            selection_members: select_after
                .emitted_members
                .saturating_sub(select_before.emitted_members),
            emitted_records: operations.len(),
            compared_bytes: total,
            emitted_bytes,
            local_ns: start.elapsed().as_nanos(),
            scratch_admission: scratch_bytes,
        };
        let reservations = Reservations {
            instance: ledger.instance,
            nodes: (ledger.floor, after.next_node),
            selections: (ledger.selection_floor, after.lifetime_selections),
            added_nodes: additions.into_iter().collect(),
            added_selections: claims.into_iter().collect(),
        };
        Ok(Some(PreparedRecordChanges {
            base: expected,
            before_content: self.history_content_id(),
            candidate,
            operations,
            guards,
            reservations,
            metrics,
            _charge: charge,
        }))
    }
}

/// Independent full-state oracle image. Its full-map scans/clones are deliberately
/// excluded from the mutation-local producer above and measured separately.
pub struct RecordImage {
    map: BTreeMap<RecordKey, Vec<u8>>,
    _charge: Charge,
}
impl RecordImage {
    pub fn from_checkpoint(blob: &Blob, memory: &Memory) -> Result<Self> {
        if blob.bytes.len() < 32
            || Sha256::digest(&blob.bytes[..blob.bytes.len() - 32]).as_slice()
                != &blob.bytes[blob.bytes.len() - 32..]
        {
            return Err(bad());
        }
        let charge = memory
            .reserve(blob.bytes.len().saturating_mul(2) + blob.records.len() * 256 + 4096)
            .map_err(|_| Error::Limit)?;
        let mut map = BTreeMap::new();
        let mut at = 0;
        let mut previous = None;
        for record in &blob.records {
            if record.range.start != at
                || record.range.end > blob.bytes.len()
                || previous.is_some_and(|p| p >= record.key)
            {
                return Err(bad());
            }
            at = record.range.end;
            previous = Some(record.key);
            if !matches!(record.key.0[0], 5 | 17) {
                map.insert(record.key, blob.bytes[record.range.clone()].to_vec());
            }
        }
        if at != blob.bytes.len() {
            return Err(bad());
        }
        Ok(Self {
            map,
            _charge: charge,
        })
    }
    pub fn records(&self) -> usize {
        self.map.len()
    }
    pub fn apply(&self, changes: &PreparedRecordChanges, memory: &Memory) -> Result<Self> {
        let bytes = self.map.values().map(Vec::len).sum::<usize>() + changes.metrics.emitted_bytes;
        let charge = memory
            .reserve(
                bytes.saturating_mul(2) + (self.map.len() + changes.operations.len()) * 256 + 4096,
            )
            .map_err(|_| Error::Limit)?;
        for (key, digest) in &changes.guards {
            if <[u8; 32]>::from(Sha256::digest(self.map.get(key).ok_or_else(bad)?)) != *digest {
                return Err(bad());
            }
        }
        let mut previous = None;
        for op in &changes.operations {
            if previous.is_some_and(|k| k >= op.key)
                || self.map.get(&op.key).map(Vec::as_slice) != op.before()
            {
                return Err(bad());
            }
            previous = Some(op.key);
        }
        let mut map = self.map.clone();
        for op in &changes.operations {
            match &op.after {
                Some(v) => {
                    map.insert(op.key, v.clone());
                }
                None => {
                    map.remove(&op.key);
                }
            }
        }
        Ok(Self {
            map,
            _charge: charge,
        })
    }
    pub fn checkpoint(&self, memory: &Memory) -> Result<Blob> {
        let core_bytes = self
            .map
            .iter()
            .filter(|(k, _)| (6..=14).contains(&k.0[0]))
            .map(|(_, v)| v.len())
            .sum::<usize>();
        let mut w = Writer::new(memory).map_err(graph_error)?;
        for (key, value) in &self.map {
            if key.0[0] == 6 {
                w.boundary(RecordKey::new(5, &[]));
                w.n(core_bytes as u64);
            }
            w.boundary(*key);
            w.raw(value);
        }
        let mut blob = w.finish().map_err(graph_error)?;
        let digest = Sha256::digest(&blob.bytes);
        blob.append(RecordKey::new(17, &[]), &digest)
            .map_err(graph_error)?;
        Ok(blob)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use pivot_core::model::integration::IntegrationRuntime;
    #[test]
    fn independent_full_checkpoint_oracle_detects_omitted_coupled_records() {
        let rt = IntegrationRuntime::standalone().unwrap();
        let mut b = Workbook::new_in("G2 omission", Limits::default(), &rt).unwrap();
        b.apply(
            b.expected(),
            &[
                Edit::SetEntry {
                    position: Position::new(1, 1),
                    source: "1".into(),
                },
                Edit::SetEntry {
                    position: Position::new(1, 2),
                    source: "=SUM(A1:A3)".into(),
                },
            ],
        )
        .unwrap();
        let ledger = ColdLedger::new(&b).unwrap();
        let before = b.history_cold_export(b.expected(), &ledger).unwrap();
        let image = RecordImage::from_checkpoint(&before, &rt.memory()).unwrap();
        let edits = [Edit::SetEntry {
            position: Position::new(1, 2),
            source: "=SUM(A2:A4)".into(),
        }];
        let mut delta = b
            .prepare_record_changes(
                b.expected(),
                &ledger,
                Mutation::Edits(&edits),
                &Default::default(),
            )
            .unwrap()
            .unwrap();
        let mut next = ledger.clone_for_oracle(&rt.memory()).unwrap();
        next.observe(delta.candidate()).unwrap();
        let full = delta
            .candidate()
            .history_cold_export(delta.candidate().expected(), &next)
            .unwrap();
        assert_eq!(
            image
                .apply(&delta, &rt.memory())
                .unwrap()
                .checkpoint(&rt.memory())
                .unwrap()
                .bytes,
            full.bytes
        );
        let length = delta.operations.len();
        for index in 0..length {
            let omitted = delta.operations.remove(index);
            let reconstruction = image
                .apply(&delta, &rt.memory())
                .unwrap()
                .checkpoint(&rt.memory())
                .unwrap();
            assert_ne!(
                reconstruction.bytes,
                full.bytes,
                "missing {:?}",
                omitted.kind()
            );
            delta.operations.insert(index, omitted);
        }
        drop((delta, image, b, ledger, next, before, full));
        rt.flush();
        let memory = rt.memory();
        assert!(rt.finish().is_ok());
        assert_eq!(memory.stats().charged, 0);
    }
}
