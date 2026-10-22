use super::*;
use crate::model::integration::NodeView;
impl Graph {
    fn view_record<'a>(&self, id: NodeId, record: &'a Record) -> NodeView<'a> {
        let (definition, tokens) = match &*record.definition {
            Compiled::NonNumeric => (DraftDefinition::NonNumeric, None),
            Compiled::CapturedScalar(id) => (DraftDefinition::CapturedScalar(*id), None),
            Compiled::Input => (
                DraftDefinition::Input(**self.values.get(id.node).unwrap()),
                None,
            ),
            Compiled::Formula { source, binding } => (
                DraftDefinition::Computed {
                    source,
                    binding: binding.as_ref().map(|(_, d)| d.as_slice()),
                },
                binding.as_ref().ok().map(|(t, _)| t.as_slice()),
            ),
        };
        NodeView {
            id,
            name: &record.name,
            definition,
            tokens,
        }
    }
    pub(crate) fn view_node(&self, id: NodeId) -> Option<NodeView<'_>> {
        self.record(id)
            .filter(|r| r.live)
            .map(|r| self.view_record(id, r))
    }
    pub(crate) fn view_nodes(
        &self,
        start: u64,
        limit: usize,
        scan: usize,
    ) -> (Vec<NodeView<'_>>, Option<u64>) {
        let mut iter = self.records.iter_from(start);
        let mut result = Vec::with_capacity(limit);
        let mut next = start;
        for _ in 0..scan {
            let Some((slot, record)) = iter.next() else {
                return (result, None);
            };
            next = slot + 1;
            if record.live {
                result.push(self.view_record(
                    NodeId {
                        model: self.version.model,
                        node: slot,
                    },
                    record,
                ));
            }
            if result.len() == limit {
                break;
            }
        }
        let more = iter.next().is_some();
        (result, more.then_some(next))
    }
    pub(crate) fn view_edges(
        &self,
        id: NodeId,
        reverse: bool,
        after: Option<NodeId>,
        limit: usize,
    ) -> Result<(Vec<NodeId>, Option<NodeId>), ModelError> {
        if self.has_selection_formulas() {
            return Err(ModelError::Backend(
                "use typed selection dependency inspection",
            ));
        }
        let record = self
            .live_record(id)
            .map_err(|_| ModelError::UnknownNode(id))?;
        let mut values = if reverse {
            let first = after.unwrap_or(NodeId {
                model: self.version.model,
                node: 0,
            });
            record
                .reverse
                .iter_from(&first)
                .map(|(id, _)| *id)
                .filter(|id| after.is_none_or(|a| *id > a))
                .take(limit + 1)
                .collect::<Vec<_>>()
        } else {
            let deps = record.definition.deps();
            let start = after.map_or(0, |after| deps.partition_point(|id| *id <= after));
            deps[start..].iter().copied().take(limit + 1).collect()
        };
        let more = values.len() > limit;
        if more {
            values.pop();
        }
        let next = if more { values.last().copied() } else { None };
        Ok((values, next))
    }
}
