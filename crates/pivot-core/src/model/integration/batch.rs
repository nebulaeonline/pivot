//! Bounded internal batch contract; logical revisions count operations, not dispatches.
use super::*;
use resources::Reclaimable;
use structural::memory::Charge;
pub const MAX_OPERATIONS: usize = 262_144;
pub const MAX_SOURCE_BYTES: usize = 16 * 1024 * 1024;
pub struct Batch {
    base: Owned<Graph>,
    runtime: Arc<RuntimeState>,
    revision: Revision,
    next: NodeId,
    edits: Vec<Edit>,
    capacity: usize,
    source_limit: usize,
    source_used: usize,
    failed: bool,
    charge: Option<Charge>,
    token_charges: Vec<Charge>,
    ticket: Option<Ticket>,
}
impl Drop for Batch {
    fn drop(&mut self) {
        if let Some(ticket) = self.ticket.take() {
            ticket.retire((
                std::mem::take(&mut self.edits),
                self.charge.take(),
                std::mem::take(&mut self.token_charges),
            ));
        }
    }
}
impl Batch {
    fn fail<T>(&mut self, error: ModelError) -> Result<T, ModelError> {
        self.failed = true;
        Err(error)
    }
    fn push(&mut self, edit: Edit, bytes: usize) -> Result<(), ModelError> {
        if self.failed {
            return Err(ModelError::Backend("batch preparation failed"));
        }
        let used = self
            .source_used
            .checked_add(bytes)
            .ok_or(ModelError::Backend("batch source limit"))?;
        if self.edits.len() >= self.capacity || used > self.source_limit {
            return self.fail(ModelError::Backend("batch request limit"));
        }
        self.source_used = used;
        self.edits.push(edit);
        Ok(())
    }
    pub fn revision(&self) -> Result<Revision, ModelError> {
        Ok(Revision {
            model: self.revision.model,
            version: self
                .revision
                .version
                .checked_add(self.edits.len() as u64)
                .ok_or(ModelError::IdentityExhausted)?,
        })
    }
    pub fn add_input(
        &mut self,
        name: impl Into<String>,
        value: PivotDecimal,
    ) -> Result<NodeId, ModelError> {
        let name = name.into();
        let bytes = name.len();
        let id = self.next;
        let next = self
            .next
            .node
            .checked_add(1)
            .ok_or(ModelError::IdentityExhausted)?;
        self.push(Edit::AddInput(name, value), bytes)?;
        self.next.node = next;
        Ok(id)
    }
    pub fn add_formula(
        &mut self,
        name: impl Into<String>,
        source: impl Into<String>,
    ) -> Result<NodeId, ModelError> {
        let name = name.into();
        let source = source.into();
        if source.len() > 4096 {
            return self.fail(ModelError::Backend("batch formula limit"));
        }
        let bytes = name.len() + source.len();
        let id = self.next;
        let next = self
            .next
            .node
            .checked_add(1)
            .ok_or(ModelError::IdentityExhausted)?;
        self.push(Edit::AddFormula(name, source), bytes)?;
        self.next.node = next;
        Ok(id)
    }
    /// Replace a numeric definition with a valid nonnumeric worksheet marker.
    pub fn set_nonnumeric(&mut self, id: NodeId) -> Result<(), ModelError> {
        self.push(Edit::NonNumeric(id), 0)
    }
    /// Trusted workbook integration only; never a public invocation input.
    #[doc(hidden)]
    pub fn add_captured_scalar(
        &mut self,
        name: String,
        identity: [u8; 16],
    ) -> Result<(NodeId, crate::captured::CapturedScalarId), ModelError> {
        if identity == [0; 16] {
            return self.fail(ModelError::Backend("empty captured scalar identity"));
        }
        let slot = crate::captured::CapturedScalarId(identity);
        let id = self.add_input(name, PivotDecimal::ZERO)?;
        self.push(Edit::CapturedScalar(id, slot), 0)?;
        Ok((id, slot))
    }
    /// Blank scalar evaluates to zero, but is excluded by numeric-member reducers.
    pub fn set_blank(&mut self, id: NodeId) -> Result<(), ModelError> {
        self.push(Edit::Blank(id), 0)
    }
    pub fn replace_input(&mut self, id: NodeId, value: PivotDecimal) -> Result<(), ModelError> {
        self.push(Edit::ReplaceInput(id, value), 0)
    }
    pub fn set_expression(
        &mut self,
        id: NodeId,
        source: impl Into<String>,
    ) -> Result<(), ModelError> {
        let s = source.into();
        if s.len() > 4096 {
            return self.fail(ModelError::Backend("batch formula limit"));
        }
        let n = s.len();
        self.push(Edit::Formula(id, s), n)
    }
    pub fn set_resolved(
        &mut self,
        id: NodeId,
        source: &str,
        tokens: Vec<transfer::ResolvedToken>,
    ) -> Result<(), ModelError> {
        if source.len() > 4096 || tokens.len() > 1_048_576 {
            return self.fail(ModelError::Backend("batch formula limit"));
        }
        let payload = tokens
            .iter()
            .map(|t| {
                if let transfer::ResolvedToken::Typed(op, _) = t {
                    op.heap_bytes()
                } else {
                    0
                }
            })
            .sum::<usize>();
        let charge = self
            .base
            .get()
            .memory()
            .reserve(tokens.len().saturating_mul(128).saturating_add(payload))
            .map_err(|_| {
                self.failed = true;
                ModelError::Backend("batch token admission")
            })?;
        self.token_charges.push(charge);
        self.push(
            Edit::Resolved(id, source.into(), tokens),
            source.len().saturating_add(payload),
        )
    }
    pub fn set_invalid(
        &mut self,
        id: NodeId,
        source: &str,
        error: crate::expression::ParseError,
    ) -> Result<(), ModelError> {
        if source.len() > 4096 {
            return self.fail(ModelError::Backend("batch formula limit"));
        }
        self.push(Edit::Invalid(id, source.into(), error), source.len())
    }
    /// Trusted structural candidate only; ordinary definition keeps its conflict guard.
    pub fn reframe_selection(
        &mut self,
        id: selection::SelectionId,
        rectangle: selection::Rectangle,
    ) -> Result<(), ModelError> {
        self.push(Edit::ReframeSelection(id, rectangle), 128)
    }
    pub fn restore_retired_selection(
        &mut self,
        id: selection::SelectionId,
    ) -> Result<(), ModelError> {
        self.push(Edit::RetiredSelection(id), 64)
    }
    pub fn define_selection(
        &mut self,
        id: selection::SelectionId,
        rectangle: selection::Rectangle,
    ) -> Result<(), ModelError> {
        self.push(Edit::Selection(id, rectangle), 128)
    }
    pub fn place(
        &mut self,
        space: selection::SpaceId,
        row: u32,
        column: u32,
        node: Option<NodeId>,
    ) -> Result<(), ModelError> {
        self.push(Edit::Placement(space, row, column, node), 64)
    }
    pub fn retire(&mut self, id: NodeId) -> Result<(), ModelError> {
        self.push(Edit::Retire(id), 0)
    }
    #[cfg(feature = "history-experiment")]
    pub fn history_next_node(&self) -> u64 {
        self.next.node
    }
    pub fn len(&self) -> usize {
        self.edits.len()
    }
    pub fn is_empty(&self) -> bool {
        self.edits.is_empty()
    }
}
impl WorkingModel {
    pub fn begin_batch(&self, capacity: usize, source_bytes: usize) -> Result<Batch, ModelError> {
        if capacity > MAX_OPERATIONS || source_bytes > MAX_SOURCE_BYTES {
            return Err(ModelError::Backend("batch request limit"));
        }
        let i = &self.integrated;
        let ticket = i.runtime.cleanup.reserve().map_err(ModelError::Backend)?;
        let charge = i
            .graph()
            .memory()
            .reserve(4096 + capacity.saturating_mul(512) + source_bytes.saturating_mul(4))
            .map_err(|_| ModelError::Backend("batch request admission"))?;
        Ok(Batch {
            base: i.graph.clone(),
            runtime: i.runtime.clone(),
            revision: self.revision,
            next: i.graph().next_identity(),
            edits: Vec::with_capacity(capacity),
            capacity,
            source_limit: source_bytes,
            source_used: 0,
            failed: false,
            charge: Some(charge),
            token_charges: Vec::new(),
            ticket: Some(ticket),
        })
    }
    pub fn apply_batch(
        &mut self,
        mut batch: Batch,
        cancel: &CancellationToken,
    ) -> Result<Revision, ModelError> {
        let i = &mut self.integrated;
        if self.revision != batch.revision
            || !i.graph.same(&batch.base)
            || !Arc::ptr_eq(&i.runtime, &batch.runtime)
        {
            return Err(ModelError::Backend("stale batch root"));
        }
        if cancel.is_cancelled() {
            return Err(ModelError::Backend("cancelled"));
        }
        if batch.failed {
            return Err(ModelError::Backend("batch preparation failed"));
        }
        let next = batch.revision()?;
        if batch.is_empty() {
            return Ok(self.revision);
        }
        let root = i.runtime.cleanup.reserve().map_err(ModelError::Backend)?;
        let garbage = i.runtime.cleanup.reserve().map_err(ModelError::Backend)?;
        let heavy = batch.len() > 256
            || batch.source_used > 65536
            || batch
                .edits
                .iter()
                .any(|e| matches!(e,Edit::Resolved(_,_,tokens) if tokens.len()>256));
        let payload = Reclaimable::new(
            (
                std::mem::take(&mut batch.edits),
                batch.charge.take(),
                std::mem::take(&mut batch.token_charges),
            ),
            batch.ticket.take().unwrap(),
        );
        let mut graph = i.graph().clone();
        let token = cancel.clone();
        let lane = if heavy {
            &i.runtime.heavy
        } else {
            &i.runtime.mutations
        };
        let (candidate, result) = lane
            .run(&i.runtime.cleanup, cancel, move || {
                let (edits, _charge, _tokens) = payload.into_inner();
                let result = graph.apply_bulk(
                    graph.version(),
                    edits,
                    &|| token.is_cancelled(),
                    move |candidate| garbage.retire(candidate),
                );
                (root.own(graph), result)
            })
            .map_err(ModelError::Backend)?;
        result.map_err(map_error)?;
        if cancel.is_cancelled() {
            return Err(ModelError::Backend("cancelled"));
        }
        i.graph = candidate;
        self.revision = next;
        Ok(next)
    }
}

impl IntegrationRuntime {
    pub fn import_selections(
        &self,
        image: transfer::SelectionImage,
        cancel: &CancellationToken,
    ) -> Result<transfer::Imported, ModelError> {
        self.import_selections_admitted(image, cancel, transfer::AdmissionLimits::default())
    }
    pub fn import_selections_admitted(
        &self,
        image: transfer::SelectionImage,
        cancel: &CancellationToken,
        admission: transfer::AdmissionLimits,
    ) -> Result<transfer::Imported, ModelError> {
        use selection::{MAX_IDENTITIES, MAX_SELECTIONS};
        let invalid = || ModelError::Backend("invalid selection-aware transfer");
        if !admission.valid()
            || image.version != 3
            || image.identities.len() > MAX_IDENTITIES
            || image.points.len() > admission.points
            || image.formulas.len() > admission.nodes
        {
            return Err(invalid());
        }
        let ids: BTreeSet<_> = image.identities.iter().map(|(id, _)| *id).collect();
        if ids.len() != image.identities.len() {
            return Err(invalid());
        }
        let active: BTreeSet<_> = image
            .identities
            .iter()
            .filter(|(_, r)| r.is_some())
            .map(|(id, _)| *id)
            .collect();
        if active.len() > MAX_SELECTIONS {
            return Err(invalid());
        }
        let used: BTreeSet<_> = image
            .formulas
            .iter()
            .flat_map(|f| {
                f.tokens.iter().filter_map(|t| {
                    if let transfer::SelectionToken::Sum(id)
                    | transfer::SelectionToken::Aggregate(_, id) = t
                    {
                        Some(*id)
                    } else {
                        None
                    }
                })
            })
            .collect();
        if used != active {
            return Err(invalid());
        }
        let mut slots = BTreeSet::new();
        let mut point_keys = BTreeSet::new();
        let node_slots: BTreeMap<_, _> = image
            .scalar_skeleton
            .nodes
            .iter()
            .map(|n| (n.slot, n))
            .collect();
        if node_slots.len() != image.scalar_skeleton.nodes.len() {
            return Err(invalid());
        }
        for f in &image.formulas {
            if !slots.insert(f.slot)||!node_slots.get(&f.slot).is_some_and(|n|matches!(&n.definition,transfer::NodeDefinition::Computed{source,binding:transfer::Binding::ParseFailed{offset:0,code:1}} if *source==f.source)){return Err(invalid())}
            if f.tokens.len() > 4096 || f.source.len() > 4096 {
                return Err(invalid());
            }
        }
        for (space, row, column, slot) in &image.points {
            if !point_keys.insert((*space, *row, *column)) || !node_slots.contains_key(slot) {
                return Err(invalid());
            }
        }
        let mut blank_slots = BTreeSet::new();
        for slot in &image.blanks {
            if !blank_slots.insert(*slot) || !node_slots.get(slot).is_some_and(|n| matches!(n.definition,transfer::NodeDefinition::Input(v) if v==PivotDecimal::ZERO)){return Err(invalid());}
        }
        let mut imported =
            self.import_portable_admitted(image.scalar_skeleton, cancel, admission)?;
        // The imported model is private until every phase succeeds. Keep staging
        // bounded without exposing partially restored state to a live workbook.
        for chunk in image.blanks.chunks(256) {
            let mut batch = imported.model.begin_batch(chunk.len(), 0)?;
            for slot in chunk {
                batch.set_blank(*imported.nodes.get(*slot).ok_or_else(invalid)?)?;
            }
            imported.model.apply_batch(batch, cancel)?;
        }
        let descriptors: BTreeMap<_, _> = image.identities.into_iter().collect();
        let retired: Vec<_> = descriptors
            .iter()
            .filter_map(|(id, r)| r.is_none().then_some(*id))
            .collect();
        for chunk in retired.chunks(256) {
            let mut batch = imported.model.begin_batch(chunk.len(), chunk.len() * 64)?;
            for id in chunk {
                batch.restore_retired_selection(*id)?;
            }
            imported.model.apply_batch(batch, cancel)?;
        }
        for chunk in image.points.chunks(256) {
            let mut batch = imported.model.begin_batch(chunk.len(), chunk.len() * 64)?;
            for (space, row, column, slot) in chunk {
                batch.place(
                    *space,
                    *row,
                    *column,
                    Some(*imported.nodes.get(*slot).ok_or_else(invalid)?),
                )?;
            }
            imported.model.apply_batch(batch, cancel)?;
        }
        let mut defined = BTreeSet::new();
        let mut formulas = image.formulas.into_iter();
        loop {
            let chunk: Vec<_> = formulas.by_ref().take(256).collect();
            if chunk.is_empty() {
                break;
            }
            let mut new_ids = BTreeSet::new();
            let mut source_bytes = 0usize;
            for f in &chunk {
                source_bytes = source_bytes.saturating_add(f.source.len());
                for t in &f.tokens {
                    match t {
                        transfer::SelectionToken::Typed(op, _) => {
                            source_bytes = source_bytes.saturating_add(op.heap_bytes())
                        }
                        transfer::SelectionToken::Sum(id)
                        | transfer::SelectionToken::Aggregate(_, id)
                            if !defined.contains(id) =>
                        {
                            new_ids.insert(*id);
                        }
                        _ => (),
                    }
                }
            }
            let mut batch = imported.model.begin_batch(
                chunk.len() + new_ids.len(),
                source_bytes.saturating_add(new_ids.len() * 128),
            )?;
            // Install each live descriptor with its first consumer in the same
            // batch; otherwise retire_unused would correctly retire it early.
            for id in new_ids {
                batch.define_selection(
                    id,
                    descriptors
                        .get(&id)
                        .copied()
                        .flatten()
                        .ok_or_else(invalid)?,
                )?;
                defined.insert(id);
            }
            for f in chunk {
                let node = *imported.nodes.get(f.slot).ok_or_else(invalid)?;
                let tokens = f
                    .tokens
                    .into_iter()
                    .map(|t| {
                        Ok(match t {
                            transfer::SelectionToken::Typed(op, n) => {
                                transfer::ResolvedToken::Typed(op, n)
                            }
                            transfer::SelectionToken::Decimal(v) => {
                                transfer::ResolvedToken::Decimal(v)
                            }
                            transfer::SelectionToken::Node(slot) => transfer::ResolvedToken::Node(
                                *imported.nodes.get(slot).ok_or_else(invalid)?,
                            ),
                            transfer::SelectionToken::Aggregate(kind, id) => {
                                transfer::ResolvedToken::Aggregate(kind, id)
                            }
                            transfer::SelectionToken::Sum(id) => transfer::ResolvedToken::Sum(id),
                            transfer::SelectionToken::Unary(op) => {
                                transfer::ResolvedToken::Unary(op)
                            }
                            transfer::SelectionToken::Control(c) => {
                                transfer::ResolvedToken::Control(c)
                            }
                            transfer::SelectionToken::Binary(op) => {
                                transfer::ResolvedToken::Binary(op)
                            }
                        })
                    })
                    .collect::<Result<Vec<_>, ModelError>>()?;
                batch.set_resolved(node, &f.source, tokens)?;
            }
            imported.model.apply_batch(batch, cancel)?;
        }
        Ok(imported)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn partial_bulk_candidate_cancellation_reclaims_off_caller_without_installation() {
        struct Observed {
            _graph: Graph,
            sender: mpsc::Sender<thread::ThreadId>,
        }
        impl Drop for Observed {
            fn drop(&mut self) {
                self.sender.send(thread::current().id()).unwrap();
            }
        }
        let runtime = IntegrationRuntime::standalone().unwrap();
        let model = runtime.create_model("partial").unwrap();
        let mut graph = model.integrated.graph().clone();
        let before = graph.version();
        let edits = (0..10000)
            .map(|i| Edit::AddInput(format!("n{i}"), 1.into()))
            .collect();
        let ticket = runtime.state().cleanup.reserve().unwrap();
        let (sender, received) = mpsc::channel();
        let polls = std::sync::atomic::AtomicUsize::new(0);
        let result = graph.apply_bulk(
            before,
            edits,
            &|| polls.fetch_add(1, std::sync::atomic::Ordering::Relaxed) > 15000,
            move |candidate| {
                ticket.retire(Observed {
                    _graph: candidate,
                    sender,
                })
            },
        );
        assert_eq!(result, Err(structural::Error::Cancelled));
        assert_eq!(graph.version(), before);
        assert!(model.node_id("n0").is_none());
        assert_ne!(
            received
                .recv_timeout(std::time::Duration::from_secs(5))
                .unwrap(),
            thread::current().id()
        );
        drop((model, graph));
        let memory = runtime.memory();
        assert!(runtime.finish().is_ok());
        assert_eq!(memory.stats().charged, 0);
    }
}
