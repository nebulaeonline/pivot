use super::*;
use structural::memory::Charge;
pub struct NodeView<'a> {
    pub id: NodeId,
    pub name: &'a str,
    pub definition: DraftDefinition<'a>,
    pub tokens: Option<&'a [BoundToken]>,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct InspectionCursor {
    view: u64,
    revision: Revision,
    slot: u64,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct EdgeCursor {
    view: u64,
    revision: Revision,
    node: NodeId,
    reverse: bool,
    last: NodeId,
}
pub struct InspectionPage<'a> {
    nodes: Vec<NodeView<'a>>,
    next: Option<InspectionCursor>,
    _charge: Charge,
}
impl<'a> InspectionPage<'a> {
    pub fn nodes(&self) -> &[NodeView<'a>] {
        &self.nodes
    }
    pub fn next(&self) -> Option<InspectionCursor> {
        self.next
    }
}
pub struct EdgePage {
    nodes: Vec<NodeId>,
    next: Option<EdgeCursor>,
    _charge: Charge,
}
impl EdgePage {
    pub fn nodes(&self) -> &[NodeId] {
        &self.nodes
    }
    pub fn next(&self) -> Option<EdgeCursor> {
        self.next
    }
}
/// Immutable version-pinned borrowed definitions, without ordered whole-graph arrays.
#[derive(Clone)]
pub struct DefinitionView {
    view: u64,
    graph: Owned<Graph>,
    runtime: Arc<RuntimeState>,
    revision: Revision,
}
impl DefinitionView {
    pub fn revision(&self) -> Revision {
        self.revision
    }
    pub fn name(&self) -> &str {
        self.graph.get().name()
    }
    pub fn interface(&self) -> &ModelInterface {
        self.graph.get().interface()
    }
    pub fn node(&self, id: NodeId) -> Option<NodeView<'_>> {
        self.graph.get().view_node(id)
    }
    /// At most 256 borrowed node headers and 4096 scanned records, plus one lookahead.
    pub fn nodes(
        &self,
        after: Option<InspectionCursor>,
        limit: usize,
        scan: usize,
    ) -> Result<InspectionPage<'_>, ModelError> {
        if limit == 0 || limit > 256 || scan == 0 || scan > 4096 {
            return Err(ModelError::Backend("inspection page limit"));
        }
        if after.is_some_and(|a| a.view != self.view || a.revision != self.revision) {
            return Err(ModelError::Backend("stale inspection cursor"));
        }
        let charge = self
            .graph
            .get()
            .memory()
            .reserve(4096 + limit * std::mem::size_of::<NodeView<'_>>())
            .map_err(|_| ModelError::Backend("inspection copy-out admission"))?;
        let (nodes, next) = self
            .graph
            .get()
            .view_nodes(after.map_or(0, |a| a.slot), limit, scan);
        Ok(InspectionPage {
            nodes,
            next: next.map(|slot| InspectionCursor {
                view: self.view,
                revision: self.revision,
                slot,
            }),
            _charge: charge,
        })
    }
    pub fn edges(
        &self,
        node: NodeId,
        reverse: bool,
        after: Option<EdgeCursor>,
        limit: usize,
    ) -> Result<EdgePage, ModelError> {
        if limit == 0 || limit > 256 {
            return Err(ModelError::Backend("edge page limit"));
        }
        if after.is_some_and(|a| {
            a.view != self.view
                || a.revision != self.revision
                || a.node != node
                || a.reverse != reverse
        }) {
            return Err(ModelError::Backend("stale or mismatched edge cursor"));
        }
        let charge = self
            .graph
            .get()
            .memory()
            .reserve(4096 + 2 * (limit + 1) * std::mem::size_of::<NodeId>())
            .map_err(|_| ModelError::Backend("edge copy-out admission"))?;
        let (nodes, next) =
            self.graph
                .get()
                .view_edges(node, reverse, after.map(|a| a.last), limit)?;
        Ok(EdgePage {
            nodes,
            next: next.map(|last| EdgeCursor {
                view: self.view,
                revision: self.revision,
                node,
                reverse,
                last,
            }),
            _charge: charge,
        })
    }
    pub fn snapshot(&self, cancel: &CancellationToken) -> Result<ModelSnapshot, SnapshotError> {
        materialize_snapshot(
            self.graph.get().clone(),
            self.revision,
            &self.runtime,
            cancel,
        )
    }
    pub fn validate_complete(&self, cancel: &CancellationToken) -> Result<(), SnapshotError> {
        self.snapshot(cancel).map(|_| ())
    }
    pub fn capture_local(&self) -> Result<CalculationCapture, CaptureFailure> {
        let ticket = self
            .runtime
            .cleanup
            .reserve()
            .map_err(CaptureFailure::Resource)?;
        let value = self
            .graph
            .get()
            .capture_local()
            .map_err(|_| CaptureFailure::Resource("capture admission"))?;
        Ok(CalculationCapture {
            revision: self.revision,
            state: PersistentCapture {
                value: ticket.own(value),
                runtime: self.runtime.clone(),
            },
        })
    }
}
impl WorkingModel {
    /// Pin the current canonical definitions without whole-graph materialization.
    pub fn inspect(&self) -> Result<DefinitionView, ModelError> {
        let i = &self.integrated;
        static NEXT_VIEW: std::sync::atomic::AtomicU64 = std::sync::atomic::AtomicU64::new(1);
        let view = NEXT_VIEW
            .try_update(
                std::sync::atomic::Ordering::Relaxed,
                std::sync::atomic::Ordering::Relaxed,
                |n| n.checked_add(1),
            )
            .map_err(|_| ModelError::IdentityExhausted)?;
        Ok(DefinitionView {
            view,
            graph: i.graph.clone(),
            runtime: i.runtime.clone(),
            revision: self.revision,
        })
    }
}
