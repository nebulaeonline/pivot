//! Owned logical workbook transfer, independent of byte encodings and files.
use super::*;
use pivot_core::model::{
    integration::{CancellationToken, IntegrationRuntime},
    transfer,
};
#[derive(Clone)]
pub struct CellRecord {
    pub position: Option<Position>,
    pub content: Option<Content>,
}
pub struct PortableWorkbook {
    pub worksheet: WorksheetId,
    pub graph_id: Uuid,
    pub rows: Uuid,
    pub columns: Uuid,
    pub row_count: u32,
    pub column_count: u32,
    pub state: u64,
    pub cells: Vec<(CellId, GraphNodeId, CellRecord)>,
    pub graph: transfer::Image,
}
impl Workbook {
    pub fn export_portable(&self, expected: Expected) -> Result<PortableWorkbook> {
        self.check(expected)?;
        let data = &self.persistent;
        let mut pairs: Vec<_> = data
            .bindings
            .iter()
            .map(|(id, b)| (*id, b.clone()))
            .collect();
        pairs.sort_by_key(|(_, b)| b.portable);
        let graph = transfer::export(
            &self.core,
            &pairs.iter().map(|(_, b)| b.runtime).collect::<Vec<_>>(),
            &[],
        )
        .map_err(graph_error)?;
        let cells = pairs
            .into_iter()
            .map(|(id, b)| {
                let r = data.records.get(&id).unwrap();
                (
                    id,
                    b.portable,
                    CellRecord {
                        position: r.position,
                        content: r.content.clone(),
                    },
                )
            })
            .collect();
        Ok(PortableWorkbook {
            worksheet: self.worksheet,
            graph_id: self.graph_id,
            rows: self.rows.namespace,
            columns: self.columns.namespace,
            row_count: self.rows.length,
            column_count: self.columns.length,
            state: self.state,
            cells,
            graph,
        })
    }
    /// Restore a validated logical projection with already-bound canonical graph definitions.
    /// The document layer must additionally validate source/binding provenance before calling.
    #[doc(hidden)]
    pub fn restore_portable(
        image: PortableWorkbook,
        limits: Limits,
        runtime: &IntegrationRuntime,
    ) -> Result<Self> {
        if image.cells.len() > limits.allocated_cells
            || image.graph.node_slots != image.cells.len()
            || !image.graph.ports.is_empty()
            || image.graph.port_slots != 0
            || image.row_count != limits.rows
            || image.column_count != limits.columns
            || image.rows.is_nil()
            || image.columns.is_nil()
            || image.rows == image.columns
            || image.graph_id.is_nil()
            || image.worksheet.uuid().is_nil()
        {
            return Err(Error::InvalidTarget);
        }
        let mut data = persistent::Data::new(runtime.memory());
        let mut cells = BTreeSet::new();
        let mut ids = BTreeSet::new();
        for (slot, (id, node, r)) in image.cells.iter().enumerate() {
            if id.uuid().is_nil()
                || node.uuid().is_nil()
                || !cells.insert(*id)
                || !ids.insert(*node)
                || r.position.is_some() != r.content.is_some()
            {
                return Err(Error::InvalidTarget);
            }
            let live = image.graph.nodes.iter().any(|n| n.slot == slot);
            if live != r.position.is_some() {
                return Err(Error::InvalidTarget);
            }
            if let Some(p) = r.position {
                if p.row == 0
                    || p.row > limits.rows
                    || p.column == 0
                    || p.column > limits.columns
                    || data.positions.get(&p).is_some()
                {
                    return Err(Error::Position);
                }
                data.positions
                    .insert_private(p, *id)
                    .map_err(|_| Error::Limit)?;
                data.live += 1;
            }
            data.record(
                *id,
                Record {
                    position: r.position,
                    content: r.content.clone(),
                },
            )?;
        }
        let stats = data.stats();
        if stats.live_cells > limits.live_cells
            || stats.formula_tokens > limits.formula_tokens
            || stats.source_bytes > limits.source_bytes
            || stats.estimated_owned_bytes > limits.bytes
        {
            return Err(Error::Limit);
        }
        let imported = runtime
            .import_portable(image.graph, &CancellationToken::default())
            .map_err(graph_error)?;
        for ((id, portable, _), runtime_id) in image.cells.into_iter().zip(imported.nodes) {
            data.bindings
                .insert_private(
                    id,
                    Binding {
                        portable,
                        runtime: runtime_id,
                    },
                )
                .map_err(|_| Error::Limit)?;
            data.nodes
                .insert_private(runtime_id, id)
                .map_err(|_| Error::Limit)?;
        }
        let persistent = imported.model.retain_resource(data).map_err(graph_error)?;
        Ok(Self {
            session: Uuid::new_v4(),
            state: image.state,
            worksheet: image.worksheet,
            graph_id: image.graph_id,
            rows: Axis {
                namespace: image.rows,
                length: image.row_count,
            },
            columns: Axis {
                namespace: image.columns,
                length: image.column_count,
            },
            persistent,
            core: imported.model,
            limits,
        })
    }
}
