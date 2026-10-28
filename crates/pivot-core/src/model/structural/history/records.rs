//! Feature-only mutation-local key capture. No checkpoint enumeration or journal writes.
use super::*;
use codec::{Blob, RecordKey, Writer, malformed};
#[derive(Default)]
pub(in crate::model::structural) struct TraceWork {
    pub reads: std::sync::atomic::AtomicUsize,
    pub scans: std::sync::atomic::AtomicUsize,
}
impl Footprint for TraceWork {}
pub struct RecordKeys {
    pub entries: Vec<RecordKey>,
    pub mutation_writes: usize,
    pub mutation_record_reads: usize,
    pub mutation_record_scans: usize,
    _charge: Charge,
}
impl Graph {
    pub(in crate::model) fn history_begin_records(&mut self) -> Result<(), Error> {
        self.history_metrics()?;
        if self.next_port != 0 || self.next > 8192 {
            return Err(Error::InvalidTransfer);
        }
        self.history.record_trace = Some(Index::new(self.memory.clone()));
        self.history.record_writes = 0;
        self.history.record_work = Some(self.memory.alloc(TraceWork::default())?);
        if let Some(state) = &self.selections {
            let mut state = (**state).clone();
            state.history_start_records();
            self.selections = Some(self.memory.alloc(state)?);
        }
        Ok(())
    }
    pub(in crate::model) fn history_end_records(&mut self) -> Result<(), Error> {
        self.history.record_trace = None;
        self.history.record_writes = 0;
        self.history.record_work = None;
        if let Some(state) = &self.selections {
            let mut state = (**state).clone();
            state.history_end_records();
            self.selections = Some(self.memory.alloc(state)?);
        }
        Ok(())
    }
    pub(in crate::model) fn history_note_record(&mut self, key: RecordKey) -> Result<(), Error> {
        if let Some(trace) = &mut self.history.record_trace {
            trace.insert(key, ())?;
            self.history.record_writes += 1;
        }
        Ok(())
    }
    pub(in crate::model) fn history_note_edit(&mut self, edit: &Edit) -> Result<(), Error> {
        if self.history.record_trace.is_none() {
            return Ok(());
        }
        let node = match edit {
            Edit::CapturedScalar(id, _)
            | Edit::Blank(id)
            | Edit::NonNumeric(id)
            | Edit::Input(id, _)
            | Edit::ReplaceInput(id, _)
            | Edit::Resolved(id, _, _)
            | Edit::Invalid(id, _, _)
            | Edit::Formula(id, _)
            | Edit::Retire(id)
            | Edit::Rename(id, _) => Some(id.node),
            Edit::AddInput(_, _) | Edit::AddFormula(_, _) => Some(self.next),
            _ => None,
        };
        if let Some(node) = node {
            self.history_note_record(RecordKey::number(7, node))?;
        }
        Ok(())
    }
    pub(in crate::model) fn history_record_keys(&self) -> Result<RecordKeys, ModelError> {
        self.history_metrics().map_err(|_| malformed())?;
        if self.next_port != 0 || self.next > 8192 {
            return Err(malformed());
        }
        let trace = self.history.record_trace.as_ref().ok_or_else(malformed)?;
        let selection_count = self
            .selections
            .as_ref()
            .map_or(0, |s| s.history_record_keys().count());
        let charge = self
            .memory
            .reserve((trace.len() + selection_count + 5).saturating_mul(64) + 4096)
            .map_err(|_| malformed())?;
        let mut entries = Vec::with_capacity(trace.len() + selection_count + 5);
        entries.extend(trace.iter().map(|(k, _)| *k));
        entries.extend([6, 8, 10, 12, 14].map(|tag| RecordKey::new(tag, &[])));
        if let Some(s) = &self.selections {
            entries.extend(s.history_record_keys());
        }
        entries.sort_unstable();
        entries.dedup();
        Ok(RecordKeys {
            entries,
            mutation_writes: self.history.record_writes
                + self
                    .selections
                    .as_ref()
                    .map_or(0, |s| s.history_record_writes()),
            mutation_record_reads: self
                .history
                .record_work
                .as_ref()
                .map_or(0, |w| w.reads.load(std::sync::atomic::Ordering::Relaxed)),
            mutation_record_scans: self
                .history
                .record_work
                .as_ref()
                .map_or(0, |w| w.scans.load(std::sync::atomic::Ordering::Relaxed))
                + self
                    .selections
                    .as_ref()
                    .map_or(0, |s| s.history_record_scans()),
            _charge: charge,
        })
    }
    pub(in crate::model) fn history_export_records(
        &self,
        keys: &[RecordKey],
    ) -> Result<Blob, ModelError> {
        self.history_metrics().map_err(|_| malformed())?;
        if self.next_port != 0 || self.next > 8192 {
            return Err(malformed());
        }
        let mut w = Writer::new(&self.memory)?;
        for key in keys {
            let ordinal = u64::from_be_bytes(key.0[1..9].try_into().unwrap());
            match key.0[0] {
                6 => {
                    w.boundary(*key);
                    w.raw(b"PIVHGC01");
                    w.text(&self.model_name);
                    w.n(self.next);
                    w.n(self.history.selections as u64);
                    w.n(self.history.records as u64);
                }
                7 => {
                    if let Some(r) = self.records.get(ordinal) {
                        w.boundary(*key);
                        w.n(ordinal);
                        w.flag(r.live);
                        w.text(&r.name);
                        match &*r.definition {
                            Compiled::Blank => w.n(0),
                            Compiled::NonNumeric => w.n(1),
                            Compiled::Input => {
                                w.n(2);
                                w.decimal(**self.values.get(ordinal).ok_or_else(malformed)?);
                            }
                            Compiled::Formula { source, binding } => {
                                w.n(3);
                                w.text(source);
                                w.flag(binding.is_ok());
                                match binding {
                                    Ok((tokens, _)) => {
                                        w.n(tokens.len() as u64);
                                        for t in tokens {
                                            super::cold::token(&mut w, t)?;
                                        }
                                    }
                                    Err(e) => super::cold::write_error(&mut w, e)?,
                                }
                            }
                            Compiled::CapturedScalar(_) => return Err(malformed()),
                        }
                    }
                }
                8 => {
                    w.boundary(*key);
                    w.n(self.selection_metrics().identities as u64);
                }
                9 => {
                    let id = selection::SelectionId(key.0[1..17].try_into().unwrap());
                    if let Some(rect) = self
                        .selections
                        .as_ref()
                        .and_then(|s| s.history_descriptor(id))
                    {
                        w.boundary(*key);
                        w.raw(&id.0);
                        w.flag(rect.is_some());
                        if let Some(rect) = rect {
                            w.raw(&rect.space.0);
                            for v in [
                                rect.first_row,
                                rect.last_row,
                                rect.first_column,
                                rect.last_column,
                            ] {
                                w.n(v.into());
                            }
                        }
                    }
                }
                10 => {
                    w.boundary(*key);
                    w.n(self.selection_metrics().points as u64);
                }
                11 => {
                    let space = selection::SpaceId(key.0[1..17].try_into().unwrap());
                    let row = u32::from_be_bytes(key.0[17..21].try_into().unwrap());
                    let col = u32::from_be_bytes(key.0[21..25].try_into().unwrap());
                    if let Some(n) = self
                        .selections
                        .as_ref()
                        .and_then(|s| s.history_point(space, row, col))
                    {
                        w.boundary(*key);
                        w.raw(&space.0);
                        w.n(row.into());
                        w.n(col.into());
                        w.n(n.node);
                    }
                }
                12 => {
                    w.boundary(*key);
                    w.n(self.invalid.len() as u64);
                }
                13 => {
                    let id = NodeId {
                        model: self.version.model,
                        node: ordinal,
                    };
                    if let Some(e) = self.invalid.get(&id) {
                        w.boundary(*key);
                        w.n(ordinal);
                        super::cold::write_error(&mut w, e)?;
                    }
                }
                14 => {
                    w.boundary(*key);
                    let nodes = self.invalidated_selection_nodes();
                    w.n(nodes.len() as u64);
                    for n in nodes {
                        w.n(n.node);
                    }
                }
                _ => return Err(malformed()),
            }
        }
        w.finish()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn cancelled_private_edits_publish_neither_records_nor_allocations() {
        let memory = Memory::new(16 * 1024 * 1024);
        let image = GraphImage::empty("G2 cancel").unwrap();
        let mut graph = Graph::empty("G2 cancel".into(), image.revision, memory.clone()).unwrap();
        graph
            .apply(
                graph.version(),
                vec![Edit::AddInput("base".into(), 10.into())],
            )
            .unwrap();
        let before = graph.cold_export().unwrap();
        let version = graph.version();
        let floor = graph.next;
        graph.history_begin_records().unwrap();
        let polls = std::cell::Cell::new(0usize);
        let edits = (0..128)
            .map(|i| Edit::AddInput(format!("n{i}"), i.into()))
            .collect();
        assert!(matches!(
            graph.apply_bulk(
                version,
                edits,
                &|| {
                    polls.set(polls.get() + 1);
                    polls.get() > 136
                },
                drop
            ),
            Err(Error::Cancelled)
        ));
        assert!(polls.get() > 136);
        assert_eq!(graph.version(), version);
        assert_eq!(graph.next, floor);
        assert_eq!(graph.cold_export().unwrap().bytes, before.bytes);
        let keys = graph.history_record_keys().unwrap();
        assert!(keys.entries.iter().all(|k| k.0[0] != 7));
        drop(keys);
        graph.history_end_records().unwrap();
        drop((before, graph));
        assert_eq!(memory.stats().charged, 0);
    }
}
