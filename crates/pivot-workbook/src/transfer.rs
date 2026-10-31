//! Owned logical workbook transfer, independent of byte encodings and files.
use super::*;
use pivot_core::model::{
    integration::{CancellationToken, IntegrationRuntime},
    transfer,
};
use std::collections::BTreeMap;
#[derive(Clone)]
pub struct CellRecord {
    pub worksheet: WorksheetId,
    pub position: Option<Position>,
    pub content: Option<Content>,
}
#[derive(Clone, Debug)]
pub struct SheetRecord {
    pub id: WorksheetId,
    pub name: String,
    pub rows: Uuid,
    pub columns: Uuid,
    pub row_count: u32,
    pub column_count: u32,
}
pub struct PortableWorkbook {
    pub filters: Vec<(WorksheetId, filtering::Definition)>,
    pub styles: Vec<formatting::Region>,
    pub style_defaults: Vec<(WorksheetId, formatting::Style)>,
    pub sheets: Vec<SheetRecord>,
    pub worksheet: WorksheetId,
    pub graph_id: Uuid,
    pub rows: Uuid,
    pub columns: Uuid,
    pub row_count: u32,
    pub column_count: u32,
    pub state: u64,
    pub cells: Vec<(CellId, GraphNodeId, CellRecord)>,
    pub graph: transfer::SelectionImage,
}
pub fn validate_sheet_directory(sheets: &[SheetRecord], limits: Limits) -> Result<()> {
    if sheets.is_empty() || sheets.len() > 16 {
        return Err(Error::Limit);
    }
    let mut ids = BTreeSet::new();
    let mut names = BTreeSet::new();
    let mut axes = BTreeSet::new();
    for sheet in sheets {
        if sheet.id.uuid().is_nil()
            || !ids.insert(sheet.id)
            || sheets::normalize_name(&sheet.name)? != sheet.name
            || !names.insert(sheets::name_key(&sheet.name))
            || sheet.row_count == 0
            || sheet.column_count == 0
            || sheet.row_count > limits.rows
            || sheet.column_count > limits.columns
            || sheet.rows.is_nil()
            || sheet.columns.is_nil()
            || !axes.insert(sheet.rows)
            || !axes.insert(sheet.columns)
        {
            return Err(Error::InvalidTarget);
        }
    }
    Ok(())
}
impl Workbook {
    pub fn export_portable(&self, expected: Expected) -> Result<PortableWorkbook> {
        self.export_content(expected, None)
    }
    /// Internal native-document seam for current and historical content. The lease covers the owned projection
    /// through encoding; this does not grant authority to mutate or lower any ledger.
    #[doc(hidden)]
    pub fn project_document(
        &self,
        expected: Expected,
        cancel: &CancellationToken,
    ) -> Result<(
        PortableWorkbook,
        pivot_core::model::integration::resources::Charge,
    )> {
        self.check(expected)?;
        if cancel.is_cancelled() {
            return Err(Error::Graph("cancelled document projection".into()));
        }
        let selection_bytes = self.core.selection_metrics().identities.saturating_mul(256);
        // Admit owned transfer/codec copies, not the persistent indexes they replace.
        // GraphImage materialization has its own simultaneous core reservation.
        // Two cell/node representations cover logical and storage images; the
        // remaining fixed allowance covers verification maps, names and wire data.
        // Variable text/token/style payloads retain the conservative 8-copy bound.
        let stats = self.persistent.stats();
        let fixed = std::mem::size_of::<(CellId, GraphNodeId, CellRecord)>() * 2
            + std::mem::size_of::<transfer::Node>() * 2
            + 512;
        let payload = stats
            .estimated_owned_bytes
            .saturating_sub(self.persistent.records.len().saturating_mul(256))
            .saturating_sub(self.persistent.positions.len().saturating_mul(64));
        let lease = self
            .core
            .allocation_domain()
            .map_err(graph_error)?
            .reserve(
                payload
                    .saturating_mul(8)
                    .saturating_add(selection_bytes)
                    .saturating_add(self.persistent.records.len().saturating_mul(fixed))
                    .saturating_add(65536),
            )
            .map_err(|_| Error::Limit)?;
        let image = self.export_content(expected, Some(cancel))?;
        self.verify_document_projection(&image, cancel)?;
        Ok((image, lease))
    }
    fn export_content(
        &self,
        expected: Expected,
        cancel: Option<&CancellationToken>,
    ) -> Result<PortableWorkbook> {
        self.check(expected)?;
        if self.is_structured_proof() {
            return Err(Error::InvalidTarget);
        }
        let data = &self.persistent;
        if data.records.len() != data.bindings.len()
            || data.nodes.len() != data.bindings.len()
            || data.bindings.iter().any(|(id, b)| {
                !data.records.contains_key(id) || data.nodes.get(&b.runtime) != Some(id)
            })
        {
            return Err(Error::InvalidTarget);
        }
        let mut pairs: Vec<_> = data
            .bindings
            .iter()
            .map(|(id, b)| (*id, b.clone()))
            .collect();
        pairs.sort_by_key(|(_, b)| b.portable);
        let nodes = pairs.iter().map(|(_, b)| b.runtime).collect::<Vec<_>>();
        let graph = if let Some(cancel) = cancel {
            self.core
                .project_document_definitions(&nodes, cancel)
                .map_err(graph_error)?
        } else {
            self.core
                .export_selections(&nodes, &[])
                .map_err(graph_error)?
        };
        let cells = pairs
            .into_iter()
            .map(|(id, b)| {
                let r = data.records.get(&id).expect("validated workbook directory");
                (
                    id,
                    b.portable,
                    CellRecord {
                        worksheet: r.worksheet,
                        position: r.position,
                        content: r.content.clone(),
                    },
                )
            })
            .collect();
        Ok(PortableWorkbook {
            filters: self.filters().map(|(id, d)| (id, d.clone())).collect(),
            styles: self.regions().collect(),
            style_defaults: self.worksheet_defaults().collect(),
            sheets: self
                .sheets()
                .iter()
                .map(|s| SheetRecord {
                    id: s.id,
                    name: s.name.clone(),
                    rows: s.rows.namespace,
                    columns: s.columns.namespace,
                    row_count: s.rows.length,
                    column_count: s.columns.length,
                })
                .collect(),
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
    fn verify_document_projection(
        &self,
        image: &PortableWorkbook,
        cancel: &CancellationToken,
    ) -> Result<()> {
        use transfer::{Binding as B, NodeDefinition as D, SelectionToken as S};
        let bad = || Error::InvalidTarget;
        let graph = &image.graph;
        let slots: BTreeMap<_, _> = image
            .cells
            .iter()
            .enumerate()
            .map(|(slot, (id, _, _))| (*id, slot))
            .collect();
        let ids: BTreeSet<_> = image.cells.iter().map(|(_, node, _)| *node).collect();
        let nodes: BTreeMap<_, _> = graph
            .scalar_skeleton
            .nodes
            .iter()
            .map(|n| (n.slot, n))
            .collect();
        let overlays: BTreeMap<_, _> = graph.formulas.iter().map(|f| (f.slot, f)).collect();
        let selections: BTreeMap<_, _> = graph.identities.iter().copied().collect();
        let blanks: BTreeSet<_> = graph.blanks.iter().copied().collect();
        if slots.len() != image.cells.len()
            || ids.len() != image.cells.len()
            || ids.iter().any(|id| id.uuid().is_nil())
            || nodes.len() != graph.scalar_skeleton.nodes.len()
            || overlays.len() != graph.formulas.len()
            || selections.len() != graph.identities.len()
            || blanks.len() != graph.blanks.len()
            || graph.scalar_skeleton.node_slots != image.cells.len()
            || !graph.scalar_skeleton.ports.is_empty()
            || graph.scalar_skeleton.port_slots != 0
        {
            return Err(bad());
        }
        let mut expected_points = BTreeSet::new();
        let mut expected_blanks = BTreeSet::new();
        let mut expected_overlays = BTreeSet::new();
        for (slot, (id, _, record)) in image.cells.iter().enumerate() {
            if cancel.is_cancelled() {
                return Err(Error::Graph("cancelled document projection".into()));
            }
            if !self.sheets().iter().any(|s| s.id == record.worksheet)
                || id.uuid().is_nil()
                || record.position.is_some() != record.content.is_some()
                || nodes.contains_key(&slot) != record.content.is_some()
            {
                return Err(bad());
            }
            let Some(content) = &record.content else {
                continue;
            };
            let definition = &nodes.get(&slot).ok_or_else(bad)?.definition;
            let p = record.position.ok_or_else(bad)?;
            if self
                .persistent
                .positions
                .get(&sheets::at(record.worksheet, p))
                != Some(id)
            {
                return Err(bad());
            }
            if !matches!(content, Content::Blank) {
                expected_points.insert((
                    pivot_core::model::selection::SpaceId(*record.worksheet.uuid().as_bytes()),
                    p.row,
                    p.column,
                    slot,
                ));
            }
            let (source, tokens) = match content {
                Content::Blank => {
                    expected_blanks.insert(slot);
                    if !matches!(definition,D::Input(v) if v.parts()==PivotDecimal::ZERO.parts()) {
                        return Err(bad());
                    }
                    continue;
                }
                Content::Decimal(d) => {
                    if !matches!(definition,D::Input(v) if v.parts()==d.value().parts()) {
                        return Err(bad());
                    }
                    continue;
                }
                Content::LiteralText(_) | Content::Boolean(_) => {
                    let op = match content {
                        Content::LiteralText(s) => pivot_core::typed::Op::String(s.clone()),
                        Content::Boolean(b) => pivot_core::typed::Op::Boolean(*b),
                        _ => unreachable!(),
                    };
                    let overlay = overlays.get(&slot).ok_or_else(bad)?;
                    expected_overlays.insert(slot);
                    if !overlay.source.is_empty()
                        || !same_selection_tokens(&overlay.tokens, &[S::Typed(op, 0)])
                        || !matches!(definition,D::Computed{source,binding:B::ParseFailed{offset:0,code:1}} if source.is_empty())
                    {
                        return Err(bad());
                    }
                    continue;
                }
                Content::Formula { source, tokens } => (source, tokens),
                Content::TextFormula(f) => {
                    if let Some(diagnostic) = &f.diagnostic {
                        if !f.tokens.is_empty()
                            || !f.references.is_empty()
                            || !matches!(definition,D::Computed{source,binding:B::ParseFailed{offset,code:1}}
                                if source==&f.original_source && *offset==diagnostic.offset)
                        {
                            return Err(bad());
                        }
                        continue;
                    }
                    let mut references = f.references.iter();
                    for token in &f.tokens {
                        let target = match token {
                            Token::Cell(target) => Some(*target),
                            _ => crate::structure::origin_of(token)?
                                .filter(|o| o.kind == 1)
                                .map(|o| CellRef {
                                    worksheet: o.worksheet,
                                    cell: CellId::from_uuid(o.target),
                                }),
                        };
                        if let Some(target) = target {
                            let r = references.next().ok_or_else(bad)?;
                            let target_slot = *slots.get(&target.cell).ok_or_else(bad)?;
                            if r.target != target
                                || r.target.worksheet != image.cells[target_slot].2.worksheet
                                || r.node != image.cells[target_slot].1
                                || r.span.start >= r.span.end
                                || f.original_source.get(r.span.clone()).is_none()
                            {
                                return Err(bad());
                            }
                        }
                    }
                    if references.next().is_some() {
                        return Err(bad());
                    }
                    (&f.original_source, &f.tokens)
                }
                Content::StructuredFormula(_) => return Err(bad()),
            };
            for token in tokens {
                if let Some(o) = crate::structure::origin_of(token)? {
                    if o.kind == 1 {
                        let slot = *slots.get(&CellId::from_uuid(o.target)).ok_or_else(bad)?;
                        let (_, node, r) = &image.cells[slot];
                        if r.position.is_some()
                            || r.worksheet != o.worksheet
                            || node.uuid() != o.node
                        {
                            return Err(bad());
                        }
                    }
                    if o.kind == 2
                        && selections.get(&pivot_core::model::selection::SelectionId(
                            *o.target.as_bytes(),
                        )) != Some(&None)
                    {
                        return Err(bad());
                    }
                }
            }
            sheets::validate_scope(record.worksheet, source, tokens, self.sheets())?;
            let translated: Vec<_> = tokens
                .iter()
                .filter(|t| !matches!(t, Token::Qualifier { .. }))
                .map(|t| {
                    Ok(match t {
                        Token::Decimal(d) => S::Decimal(d.value()),
                        Token::Cell(c)
                            if slots.get(&c.cell).is_some_and(|slot| {
                                image.cells[*slot].2.worksheet == c.worksheet
                            }) =>
                        {
                            S::Node(*slots.get(&c.cell).ok_or_else(bad)?)
                        }
                        Token::Unary(op) => S::Unary(*op),
                        Token::Typed(op, n) => S::Typed(op.clone(), *n),
                        Token::Control(c) => S::Control(*c),
                        Token::Binary(op) => S::Binary(*op),
                        Token::Sum {
                            kind,
                            id,
                            rectangle,
                            first,
                            last,
                            span,
                        } => {
                            if selections.get(id) != Some(&Some(*rectangle))
                                || rectangle.first_row != first.row.min(last.row)
                                || rectangle.last_row != first.row.max(last.row)
                                || rectangle.first_column != first.column.min(last.column)
                                || rectangle.last_column != first.column.max(last.column)
                                || span.start >= span.end
                                || source.get(span.clone()).is_none()
                            {
                                return Err(bad());
                            }
                            if *kind == pivot_core::expression::AggregateKind::Sum {
                                S::Sum(*id)
                            } else {
                                S::Aggregate(*kind, *id)
                            }
                        }
                        _ => return Err(bad()),
                    })
                })
                .collect::<Result<_>>()?;
            if translated.iter().any(|t| {
                matches!(
                    t,
                    S::Typed(..) | S::Control(_) | S::Sum(_) | S::Aggregate(_, _)
                )
            }) {
                expected_overlays.insert(slot);
                let overlay = overlays.get(&slot).ok_or_else(bad)?;
                if overlay.source != *source
                    || !same_selection_tokens(&overlay.tokens, &translated)
                    || !matches!(definition,D::Computed{source:s,binding:B::ParseFailed{offset:0,code:1}} if s==source)
                {
                    return Err(bad());
                }
            } else {
                let D::Computed {
                    source: s,
                    binding: B::Bound(bound),
                } = definition
                else {
                    return Err(bad());
                };
                let actual: Vec<_> = bound
                    .iter()
                    .map(|t| match t {
                        transfer::Token::Decimal(v) => S::Decimal(*v),
                        transfer::Token::Node(n) => S::Node(*n),
                        transfer::Token::Unary(op) => S::Unary(*op),
                        transfer::Token::Binary(op) => S::Binary(*op),
                    })
                    .collect();
                if s != source || !same_selection_tokens(&actual, &translated) {
                    return Err(bad());
                }
            }
        }
        let points: BTreeSet<_> = graph.points.iter().copied().collect();
        if points.len() != graph.points.len()
            || points != expected_points
            || blanks != expected_blanks
            || overlays.keys().copied().collect::<BTreeSet<_>>() != expected_overlays
            || nodes.keys().any(|slot| *slot >= image.cells.len())
        {
            return Err(bad());
        }
        Ok(())
    }
    /// Restore a validated logical projection with already-bound canonical graph definitions.
    /// The document layer must additionally validate source/binding provenance before calling.
    #[doc(hidden)]
    pub fn restore_portable(
        image: PortableWorkbook,
        limits: Limits,
        runtime: &IntegrationRuntime,
    ) -> Result<Self> {
        validate_sheet_directory(&image.sheets, limits)?;
        if image.cells.len() > limits.allocated_cells
            || image.graph.scalar_skeleton.node_slots != image.cells.len()
            || !image.graph.scalar_skeleton.ports.is_empty()
            || image.graph.scalar_skeleton.port_slots != 0
            || image.row_count == 0
            || image.row_count > limits.rows
            || image.column_count == 0
            || image.column_count > limits.columns
            || image.rows.is_nil()
            || image.columns.is_nil()
            || image.rows == image.columns
            || image.graph_id.is_nil()
            || image.worksheet.uuid().is_nil()
        {
            return Err(Error::InvalidTarget);
        }
        if image.cells.iter().any(|(_, _, r)| match &r.content {
            Some(Content::StructuredFormula(_)) => true,
            Some(Content::Formula { tokens, .. }) => {
                tokens.iter().any(|t| matches!(t, Token::Output(_)))
            }
            Some(Content::TextFormula(f)) => f.tokens.iter().any(|t| matches!(t, Token::Output(_))),
            _ => false,
        }) {
            return Err(Error::InvalidTarget);
        }
        let blank_slots: BTreeSet<_> = image.graph.blanks.iter().copied().collect();
        let expected_blanks: BTreeSet<_> = image
            .cells
            .iter()
            .enumerate()
            .filter_map(|(slot, (_, _, r))| {
                matches!(r.content, Some(Content::Blank)).then_some(slot)
            })
            .collect();
        if blank_slots.len() != image.graph.blanks.len() || blank_slots != expected_blanks {
            return Err(Error::InvalidTarget);
        }
        let mut data = persistent::Data::new(runtime.memory());
        data.install_sheets(
            image
                .sheets
                .iter()
                .map(|s| SheetInfo {
                    id: s.id,
                    name: s.name.clone(),
                    rows: Axis {
                        namespace: s.rows,
                        length: limits.rows,
                    },
                    columns: Axis {
                        namespace: s.columns,
                        length: limits.columns,
                    },
                })
                .collect(),
        )?;
        if image.sheets.first().is_none_or(|s| {
            s.id != image.worksheet
                || s.rows != image.rows
                || s.columns != image.columns
                || s.row_count != image.row_count
                || s.column_count != image.column_count
        }) || image
            .sheets
            .iter()
            .any(|s| s.row_count > limits.rows || s.column_count > limits.columns)
        {
            return Err(Error::InvalidTarget);
        }
        let sheet_ids: BTreeSet<_> = image.sheets.iter().map(|s| s.id).collect();
        for &(sheet, style) in &image.style_defaults {
            if !sheet_ids.contains(&sheet) {
                return Err(Error::InvalidTarget);
            }
            style.validate()?;
        }
        for region in &image.styles {
            let r = region.rectangle;
            let sheet = image
                .sheets
                .iter()
                .find(|s| s.id == region.sheet)
                .ok_or(Error::InvalidTarget)?;
            if r.first.row == 0
                || r.first.column == 0
                || r.first.row > r.last.row
                || r.first.column > r.last.column
                || r.last.row > sheet.row_count
                || r.last.column > sheet.column_count
            {
                return Err(Error::InvalidTarget);
            }
        }
        data.styles
            .restore(&image.style_defaults, &image.styles, limits.styles)?;
        let mut filter_sheets = BTreeSet::new();
        let mut filter_bytes = 0usize;
        for (sheet, definition) in &image.filters {
            let info = image
                .sheets
                .iter()
                .find(|s| s.id == *sheet)
                .ok_or(Error::InvalidTarget)?;
            if !filter_sheets.insert(*sheet) {
                return Err(Error::InvalidTarget);
            }
            definition.validate(info.row_count, info.column_count)?;
            filter_bytes = filter_bytes
                .checked_add(filtering::definition_bytes(definition))
                .ok_or(Error::Limit)?;
            if filter_bytes > filtering::MAX_DEFINITION_BYTES {
                return Err(Error::Limit);
            }
            let mut definition = definition.clone();
            definition.conditions.sort_by_key(|c| c.column);
            let shared = data.memory.alloc(definition).map_err(|_| Error::Limit)?;
            data.filters
                .insert_private(*sheet, shared)
                .map_err(|_| Error::Limit)?;
        }
        let mut cells = BTreeSet::new();
        let mut ids = BTreeSet::new();
        let live_slots: BTreeSet<_> = image
            .graph
            .scalar_skeleton
            .nodes
            .iter()
            .map(|n| n.slot)
            .collect();
        if live_slots.len() != image.graph.scalar_skeleton.nodes.len() {
            return Err(Error::InvalidTarget);
        }
        // Complete graph import before allocating persistent cell/index copies.
        // Its bounded transient materialization must not overlap those copies;
        // both candidates remain private until the entire restore succeeds.
        let imported = runtime
            .import_selections_admitted(
                image.graph,
                &CancellationToken::default(),
                limits.document_admission(),
            )
            .map_err(graph_error)?;
        for (slot, (id, node, r)) in image.cells.iter().enumerate() {
            match &r.content {
                Some(Content::TextFormula(f)) => sheets::validate_scope(
                    r.worksheet,
                    &f.original_source,
                    &f.tokens,
                    &data.sheets.as_ref().unwrap().0,
                )?,
                Some(Content::Formula { source, tokens }) => sheets::validate_scope(
                    r.worksheet,
                    source,
                    tokens,
                    &data.sheets.as_ref().unwrap().0,
                )?,
                _ => (),
            }
            if !sheet_ids.contains(&r.worksheet)
                || id.uuid().is_nil()
                || node.uuid().is_nil()
                || !cells.insert(*id)
                || !ids.insert(*node)
                || r.position.is_some() != r.content.is_some()
            {
                return Err(Error::InvalidTarget);
            }
            let live = live_slots.contains(&slot);
            if live != r.position.is_some() {
                return Err(Error::InvalidTarget);
            }
            if let Some(p) = r.position {
                if p.row == 0
                    || p.row > limits.rows
                    || p.column == 0
                    || p.column > limits.columns
                    || data.positions.get(&sheets::at(r.worksheet, p)).is_some()
                {
                    return Err(Error::Position);
                }
                data.positions
                    .insert_private(sheets::at(r.worksheet, p), *id)
                    .map_err(|_| Error::Limit)?;
                data.live += 1;
            }
            data.record(
                *id,
                Record::new(&data.memory, r.worksheet, r.position, r.content.clone())?,
            )?;
        }
        let stats = data.stats();
        if stats.live_cells > limits.live_cells
            || stats.formula_tokens > limits.total_formula_tokens
            || stats.source_bytes > limits.total_source_bytes
            || stats.estimated_owned_bytes > limits.bytes
        {
            return Err(Error::Limit);
        }
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
            #[cfg(feature = "history-experiment")]
            history_content: Uuid::new_v4(),
            #[cfg(feature = "history-experiment")]
            history_active: false,
            #[cfg(feature = "history-experiment")]
            history_recording: None,
            session: Uuid::new_v4(),
            state: image.state,
            worksheet: image.worksheet,
            graph_id: image.graph_id,
            rows: Axis {
                namespace: image.rows,
                length: limits.rows,
            },
            columns: Axis {
                namespace: image.columns,
                length: limits.columns,
            },
            persistent,
            core: imported.model,
            limits,
        })
    }
}

fn same_selection_tokens(a: &[transfer::SelectionToken], b: &[transfer::SelectionToken]) -> bool {
    use transfer::SelectionToken as T;
    a.len() == b.len()
        && a.iter().zip(b).all(|(a, b)| match (a, b) {
            (T::Decimal(a), T::Decimal(b)) => a.parts() == b.parts(),
            (T::Node(a), T::Node(b)) => a == b,
            (T::Typed(a, n), T::Typed(b, m)) => a == b && n == m,
            (T::Control(a), T::Control(b)) => a == b,
            (T::Unary(a), T::Unary(b)) => a == b,
            (T::Binary(a), T::Binary(b)) => a == b,
            (T::Sum(a), T::Sum(b)) => a == b,
            (T::Aggregate(a, b), T::Aggregate(c, d)) => a == c && b == d,
            _ => false,
        })
}

#[cfg(all(test, feature = "history-experiment"))]
mod projection_tests {
    use super::*;
    #[test]
    fn inconsistent_directories_and_bound_targets_fail_closed() {
        let rt = IntegrationRuntime::standalone().unwrap();
        let mut book = Workbook::new_in("projection checks", Limits::default(), &rt).unwrap();
        book.apply(
            book.expected(),
            &[
                Edit::SetEntry {
                    position: Position::new(1, 1),
                    source: "10".into(),
                },
                Edit::SetEntry {
                    position: Position::new(1, 2),
                    source: "=A1".into(),
                },
                Edit::SetEntry {
                    position: Position::new(1, 3),
                    source: "=SUM(A1:B1)".into(),
                },
            ],
        )
        .unwrap();
        for damage in 0..8 {
            let (mut image, lease) = book
                .project_document(book.expected(), &Default::default())
                .unwrap();
            match damage {
                0 => {
                    image.cells.pop();
                }
                1 => {
                    image.cells[0].0 = image.cells[1].0;
                }
                2 => {
                    image.cells[0].1 = image.cells[1].1;
                }
                3 => {
                    image.graph.points.pop();
                }
                4 => {
                    image.graph.formulas.clear();
                }
                5 => {
                    image.graph.identities.clear();
                }
                6 => {
                    image.graph.scalar_skeleton.nodes[0].definition =
                        transfer::NodeDefinition::Input(999.into());
                }
                7 => {
                    image.graph.scalar_skeleton.nodes[0].slot = usize::MAX;
                }
                _ => unreachable!(),
            }
            assert!(
                book.verify_document_projection(&image, &Default::default())
                    .is_err(),
                "damage {damage}"
            );
            drop((image, lease));
        }
        // Corrupt the trusted directory itself: no missing/foreign binding may be inferred.
        let mut foreign = Workbook::new_in("foreign", Limits::default(), &rt).unwrap();
        foreign
            .apply(
                foreign.expected(),
                &[Edit::SetEntry {
                    position: Position::new(1, 1),
                    source: "1".into(),
                }],
            )
            .unwrap();
        let foreign_id = foreign.history_runtime_node(Position::new(1, 1)).unwrap();
        let mut data = (*book.persistent).clone();
        let id = *data.bindings.iter().next().unwrap().0;
        let mut binding = data.bindings.get(&id).unwrap().clone();
        binding.runtime = foreign_id;
        data.bindings.insert_private(id, binding).unwrap();
        book.persistent = book.core.retain_resource(data).unwrap();
        assert!(
            book.project_document(book.expected(), &Default::default())
                .is_err()
        );
        drop((book, foreign));
        rt.flush();
        let memory = rt.memory();
        assert!(rt.finish().is_ok());
        assert_eq!(memory.stats().charged, 0);
    }
}
