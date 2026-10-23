//! Feature-only root activation/accounting feasibility, not an undo engine.
use super::*;
#[derive(Clone, Default)]
pub(super) struct Accounting {
    pub payload: usize,
    pub names: usize,
    pub dependencies: usize,
    pub records: usize,
    pub extent: u64,
    pub selections: usize,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct HistoryMetrics {
    pub weight: usize,
    pub next_node: u64,
    pub next_port: u64,
    pub lifetime_selections: usize,
    pub records: usize,
    pub weight_scan_visits: usize,
}
impl Graph {
    pub(super) fn history_claim_selection(
        &mut self,
        id: selection::SelectionId,
    ) -> Result<(), Error> {
        if self
            .selections
            .as_ref()
            .is_none_or(|s| !s.history_has_identity(id))
        {
            self.history.selections = self
                .history
                .selections
                .checked_add(1)
                .ok_or(Error::Exhausted)?;
            if self.history.selections > selection::MAX_IDENTITIES {
                return Err(Error::Quota);
            }
        }
        Ok(())
    }
    pub(in crate::model) fn history_activated(&self, current: &Self) -> Result<Self, Error> {
        if self.version.model != current.version.model
            || self.captured_scalar.is_some()
            || current.captured_scalar.is_some()
            || !self.ports.inputs.is_empty()
            || !self.ports.outputs.is_empty()
            || !current.ports.inputs.is_empty()
            || !current.ports.outputs.is_empty()
        {
            return Err(Error::InvalidTransfer);
        }
        let mut root = self.clone();
        root.next = root.next.max(current.next);
        root.next_port = root.next_port.max(current.next_port);
        root.history.selections = root.history.selections.max(current.history.selections);
        root.version.commit = root
            .version
            .commit
            .max(current.version.commit)
            .checked_add(1)
            .ok_or(Error::Exhausted)?;
        root.version.topology = root
            .version
            .topology
            .max(current.version.topology)
            .checked_add(1)
            .ok_or(Error::Exhausted)?;
        root.version.inputs = root
            .version
            .inputs
            .max(current.version.inputs)
            .checked_add(1)
            .ok_or(Error::Exhausted)?;
        root.check_quota()?;
        Ok(root)
    }
    pub(in crate::model) fn history_metrics(&self) -> Result<HistoryMetrics, Error> {
        if self.captured_scalar.is_some()
            || !self.ports.inputs.is_empty()
            || !self.ports.outputs.is_empty()
        {
            return Err(Error::InvalidTransfer);
        }
        let h = &self.history;
        let n = h.records as u128;
        let pages = (h.extent as u128).div_ceil(PAGE as u128);
        let mut branches = 0u128;
        let mut divisor = 16u128;
        for _ in 0..16 {
            branches += pages.div_ceil(divisor);
            divisor *= 16;
        }
        let header = memory::history_allocation_header::<Tree<Page<Record>>>() as u128;
        let page_headers = memory::history_allocation_header::<Page<Record>>() as u128
            + memory::history_allocation_header::<Page<PivotDecimal>>() as u128;
        let mut bytes = 4096u128
            + (self.model_name.capacity() as u128)
            + (branches + pages) * 2 * header
            + pages * page_headers;
        bytes += n
            * (memory::history_allocation_header::<Record>()
                + memory::history_allocation_header::<Compiled>()
                + memory::history_allocation_header::<PivotDecimal>()) as u128;
        bytes += h.payload as u128 + 2 * h.names as u128;
        bytes += self.names.len() as u128 * Index::<String, NodeId>::history_entry_header() as u128;
        bytes += h.dependencies as u128 * Index::<NodeId, ()>::history_entry_header() as u128;
        bytes += self.invalid.len() as u128
            * (Index::<NodeId, Shared<SnapshotError>>::history_entry_header()
                + memory::history_allocation_header::<SnapshotError>()) as u128;
        bytes += self.invalid.len() as u128
            * (h.payload as u128 + n * std::mem::size_of::<NodeId>() as u128);
        if let Some(s) = &self.selections {
            bytes += s.history_weight();
        }
        if let Some(v) = &self.selection_invalidated {
            bytes += memory::history_allocation_header::<Vec<NodeId>>() as u128
                + v.capacity() as u128 * std::mem::size_of::<NodeId>() as u128;
        }
        Ok(HistoryMetrics {
            weight: usize::try_from(bytes).map_err(|_| Error::Exhausted)?,
            next_node: self.next,
            next_port: self.next_port,
            lifetime_selections: h.selections,
            records: h.records,
            weight_scan_visits: 0,
        })
    }
}
