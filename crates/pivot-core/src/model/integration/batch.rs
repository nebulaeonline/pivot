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
        let charge = self
            .base
            .get()
            .memory()
            .reserve(tokens.len().saturating_mul(128))
            .map_err(|_| {
                self.failed = true;
                ModelError::Backend("batch token admission")
            })?;
        self.token_charges.push(charge);
        self.push(Edit::Resolved(id, source.into(), tokens), source.len())
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
    pub fn retire(&mut self, id: NodeId) -> Result<(), ModelError> {
        self.push(Edit::Retire(id), 0)
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
        let i = self.integrated.as_ref().ok_or(ModelError::Backend(
            "persistent batch requires explicit runtime",
        ))?;
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
        let i = self
            .integrated
            .as_mut()
            .ok_or(ModelError::Backend("persistent batch required"))?;
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
        let mut graph = model.integrated.as_ref().unwrap().graph().clone();
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
