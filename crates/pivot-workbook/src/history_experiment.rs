//! Feature-only root pins and activation feasibility. No history stack or production controls.
use crate::*;
use pivot_core::model::integration::{CancellationToken, HistoryMetrics, resources::Charge};
pub mod changes;
mod cold;
pub use cold::ColdLedger;
pub mod durability;
pub mod representation;
pub struct HistoricalRoot {
    session: Uuid,
    worksheet: WorksheetId,
    graph: Uuid,
    core: WorkingModel,
    data: persistent::State,
    content: Uuid,
    weight: usize,
    _charge: Charge,
}
impl HistoricalRoot {
    pub fn content_id(&self) -> Uuid {
        self.content
    }
    pub fn weight(&self) -> usize {
        self.weight
    }
}
impl Workbook {
    pub fn history_runtime_node(&self, position: Position) -> Result<NodeId> {
        let c = self.cell_at(position)?.ok_or(Error::InvalidTarget)?;
        persistent::state(self)
            .bindings
            .get(&c.reference.cell)
            .map(|b| b.runtime)
            .ok_or(Error::InvalidTarget)
    }
    pub fn history_content_id(&self) -> Uuid {
        self.history_content
    }
    pub fn history_metrics(&self) -> Result<HistoryMetrics> {
        self.core.history_metrics().map_err(graph_error)
    }
    pub fn history_weight(&self) -> Result<usize> {
        self.history_metrics()?
            .weight
            .checked_add(persistent::history_weight(self)?)
            .and_then(|v| v.checked_add(std::mem::size_of::<HistoricalRoot>() + 256))
            .ok_or(Error::Exhausted)
    }
    pub fn pin_history_root(&self, expected: Expected) -> Result<HistoricalRoot> {
        self.check(expected)?;
        let weight = self.history_weight()?;
        let charge = self
            .core
            .allocation_domain()
            .map_err(graph_error)?
            .reserve(std::mem::size_of::<HistoricalRoot>() + 256)
            .map_err(|_| Error::Limit)?;
        Ok(HistoricalRoot {
            session: self.session,
            worksheet: self.worksheet,
            graph: self.graph_id,
            core: self.core.stage(),
            data: self.persistent.clone(),
            content: self.history_content,
            weight,
            _charge: charge,
        })
    }
    pub fn activate_history_root(
        &mut self,
        expected: Expected,
        root: &HistoricalRoot,
        cancel: &CancellationToken,
    ) -> Result<Expected> {
        self.check(expected)?;
        self.history_weight()?;
        if root.session != self.session
            || root.worksheet != self.worksheet
            || root.graph != self.graph_id
        {
            return Err(Error::StaleSession);
        }
        let state = self.state.checked_add(1).ok_or(Error::Exhausted)?;
        let core = self
            .core
            .history_activate(&root.core, cancel)
            .map_err(graph_error)?;
        if cancel.is_cancelled() {
            return Err(Error::Graph("cancelled".into()));
        }
        self.core = core;
        self.persistent = root.data.clone();
        self.state = state;
        self.history_content = root.content;
        self.history_active = true;
        Ok(self.expected())
    }
    /// Private staging owner for an admission experiment, not an Undo/Redo coordinator.
    pub fn stage_history_candidate(&self, expected: Expected) -> Result<Self> {
        self.check(expected)?;
        self.history_weight()?;
        Ok(Self {
            session: self.session,
            state: self.state,
            worksheet: self.worksheet,
            graph_id: self.graph_id,
            rows: self.rows.clone(),
            columns: self.columns.clone(),
            persistent: self.persistent.clone(),
            core: self.core.stage(),
            limits: self.limits,
            history_content: self.history_content,
            history_active: self.history_active,
            #[cfg(feature = "history-experiment")]
            history_recording: None,
        })
    }
}
