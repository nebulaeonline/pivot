//! Bounded opt-in worksheet transactions using canonical persistent graph/indexes.
use super::*;
use pivot_core::experiment_profile::construction as profile;
use pivot_core::model::integration::{
    CancellationToken, IntegrationRuntime,
    batch::Batch,
    resources::{Charge, Footprint, Index, Key, Memory, Reclaimable, Retained, Shared},
};
pub(super) type State = Retained<Data>;
#[derive(Clone)]
pub(super) struct Data {
    pub(super) positions: Index<Position, CellId>,
    records: Index<CellId, Shared<Record>>,
    bindings: Index<CellId, Binding>,
    nodes: Index<NodeId, CellId>,
    memory: Memory,
    tokens: usize,
    source: usize,
    metadata: usize,
    live: usize,
}
impl Key for Position {}
impl Key for CellId {}
impl Footprint for CellId {}
impl Footprint for Binding {}
fn content_size(content: &Option<Content>) -> (usize, usize, usize) {
    match content {
        Some(Content::Formula { source, tokens }) => (tokens.len(), source.len(), 0),
        Some(Content::TextFormula(f)) => (
            f.tokens.len(),
            f.original_source.len(),
            f.references.len() * std::mem::size_of::<FormulaReference>(),
        ),
        _ => (0, 0, 0),
    }
}
impl Footprint for Record {
    fn heap_bytes(&self) -> usize {
        match &self.content {
            Some(Content::Formula { source, tokens }) => {
                source.capacity() + tokens.capacity() * std::mem::size_of::<Token>()
            }
            Some(Content::TextFormula(f)) => {
                f.original_source.capacity()
                    + f.tokens.capacity() * std::mem::size_of::<Token>()
                    + f.references.capacity() * std::mem::size_of::<FormulaReference>()
            }
            _ => 0,
        }
    }
}
impl Data {
    fn new(memory: Memory) -> Self {
        Self {
            positions: Index::new(memory.clone()),
            records: Index::new(memory.clone()),
            bindings: Index::new(memory.clone()),
            nodes: Index::new(memory.clone()),
            memory,
            tokens: 0,
            source: 0,
            metadata: 0,
            live: 0,
        }
    }
    pub(super) fn stats(&self) -> Stats {
        Stats {
            live_cells: self.positions.len(),
            live_nodes: self.live,
            allocated_nodes: self.records.len(),
            formula_tokens: self.tokens,
            source_bytes: self.source,
            estimated_owned_bytes: self.records.len() * 256
                + self.positions.len() * 64
                + self.tokens * std::mem::size_of::<Token>()
                + self.source
                + self.metadata,
        }
    }
    fn record(&mut self, id: CellId, record: Record) -> Result<()> {
        let _profile = profile::scope(3);
        let old = self
            .records
            .get(&id)
            .map_or((0, 0, 0), |r| content_size(&r.content));
        let new = content_size(&record.content);
        let shared = self.memory.alloc(record).map_err(|_| Error::Limit)?;
        self.records
            .insert_private(id, shared)
            .map_err(|_| Error::Limit)?;
        self.tokens = self.tokens - old.0 + new.0;
        self.source = self.source - old.1 + new.1;
        self.metadata = self.metadata - old.2 + new.2;
        Ok(())
    }
}
impl Workbook {
    /// Application-owned bounded runtime.
    pub fn new_in(name: &str, limits: Limits, runtime: &IntegrationRuntime) -> Result<Self> {
        Self::persistent_checked(name, limits, runtime, false)
    }
    #[cfg(feature = "scaling-experiment")]
    #[doc(hidden)]
    pub fn new_persistent_experiment(
        name: &str,
        limits: Limits,
        runtime: &IntegrationRuntime,
    ) -> Result<Self> {
        Self::persistent_checked(name, limits, runtime, true)
    }
    fn persistent_checked(
        name: &str,
        limits: Limits,
        runtime: &IntegrationRuntime,
        benchmark: bool,
    ) -> Result<Self> {
        if limits.rows == 0
            || limits.columns == 0
            || name.len() > 128
            || (!benchmark
                && (limits.live_cells > 4096
                    || limits.allocated_cells > 8192
                    || limits.bytes > 1_048_576))
            || limits.source_bytes > 4096
            || limits.formula_tokens > 4096
        {
            return Err(Error::Limit);
        }
        let core = WorkingModel::new_in(name, runtime).map_err(graph_error)?;
        let data = core
            .retain_resource(Data::new(runtime.memory()))
            .map_err(graph_error)?;
        Ok(Self {
            session: Uuid::new_v4(),
            state: 0,
            worksheet: WorksheetId::fresh(),
            graph_id: Uuid::new_v4(),
            rows: Axis {
                namespace: Uuid::new_v4(),
                length: limits.rows,
            },
            columns: Axis {
                namespace: Uuid::new_v4(),
                length: limits.columns,
            },
            persistent: data,
            core,
            limits,
        })
    }
    /// Requested-closure capture, not certification that the whole worksheet is valid.
    pub fn capture_local(&self, expected: Expected) -> Result<CalculationCapture> {
        capture(self, expected, true)
    }
    pub fn apply_cancelled(
        &mut self,
        expected: Expected,
        edits: &[Edit],
        cancel: &CancellationToken,
    ) -> Result<Expected> {
        apply(self, expected, edits, cancel)
    }
}
fn state(book: &Workbook) -> &Data {
    &book.persistent
}
fn cell_ref(book: &Workbook, id: CellId) -> CellRef {
    CellRef {
        worksheet: book.worksheet,
        cell: id,
    }
}
fn view(book: &Workbook, p: &Data, id: CellId) -> Option<CellView> {
    let record = p.records.get(&id)?;
    Some(CellView {
        reference: cell_ref(book, id),
        node: p.bindings.get(&id)?.portable,
        position: record.position?,
        content: record.content.as_ref()?.clone(),
    })
}
pub(super) fn cell_at(book: &Workbook, position: Position) -> Result<Option<CellView>> {
    book.axes_at(position)?;
    let p = state(book);
    let Some(id) = p.positions.get(&position) else {
        return Ok(None);
    };
    let _copy = p
        .memory
        .reserve(
            4096 + std::mem::size_of::<CellView>() + (**p.records.get(id).unwrap()).heap_bytes(),
        )
        .map_err(|_| Error::Limit)?;
    Ok(view(book, p, *id))
}
pub(super) fn node_for(book: &Workbook, r: CellRef) -> Result<GraphNodeId> {
    if r.worksheet != book.worksheet {
        return Err(Error::InvalidTarget);
    }
    state(book)
        .bindings
        .get(&r.cell)
        .map(|b| b.portable)
        .ok_or(Error::InvalidTarget)
}
struct Journal {
    cells: BTreeSet<CellId>,
    positions: BTreeSet<Position>,
    _charge: Charge,
}
struct Transaction<'a> {
    book: &'a Workbook,
    data: Reclaimable<Data>,
    batch: Batch,
    journal: Reclaimable<Journal>,
}
impl Transaction<'_> {
    fn anchor(&mut self, position: Position) -> Result<CellId> {
        self.book.axes_at(position)?;
        if let Some(id) = self.data.positions.get(&position) {
            return Ok(*id);
        }
        if self.data.positions.len() >= self.book.limits.live_cells
            || self.data.records.len() >= self.book.limits.allocated_cells
        {
            return Err(Error::Limit);
        }
        let identity_profile = profile::scope(2);
        let id = CellId::fresh();
        let node = self
            .batch
            .add_input(format!("N{}", self.data.records.len()), PivotDecimal::ZERO)
            .map_err(graph_error)?;
        drop(identity_profile);
        let _profile = profile::scope(3);
        self.journal.cells.insert(id);
        self.journal.positions.insert(position);
        self.data.record(
            id,
            Record {
                position: Some(position),
                content: Some(Content::Blank),
            },
        )?;
        self.data
            .positions
            .insert_private(position, id)
            .map_err(|_| Error::Limit)?;
        let portable = {
            let _profile = profile::scope(2);
            GraphNodeId::fresh()
        };
        self.data
            .bindings
            .insert_private(
                id,
                Binding {
                    portable,
                    runtime: node,
                },
            )
            .map_err(|_| Error::Limit)?;
        self.data
            .nodes
            .insert_private(node, id)
            .map_err(|_| Error::Limit)?;
        self.data.live += 1;
        Ok(id)
    }
    fn set(&mut self, id: CellId, content: Content) -> Result<()> {
        self.journal.cells.insert(id);
        let position = self.data.records.get(&id).unwrap().position;
        self.data.record(
            id,
            Record {
                position,
                content: Some(content),
            },
        )
    }
    fn resolve(&mut self, tokens: &[Token]) -> Result<(Vec<ResolvedToken>, Vec<Token>)> {
        let _profile = profile::scope(5);
        let mut bound = Vec::with_capacity(tokens.len());
        let mut normalized = Vec::with_capacity(tokens.len());
        for token in tokens {
            let t = match token {
                Token::At(p) => {
                    let id = self.anchor(*p)?;
                    Token::Cell(cell_ref(self.book, id))
                }
                Token::Cell(r) => {
                    if r.worksheet != self.book.worksheet
                        || !self.data.records.contains_key(&r.cell)
                    {
                        return Err(Error::InvalidTarget);
                    }
                    Token::Cell(*r)
                }
                other => other.clone(),
            };
            bound.push(match &t {
                Token::Decimal(v) => ResolvedToken::Decimal(v.value()),
                Token::Cell(r) => {
                    ResolvedToken::Node(self.data.bindings.get(&r.cell).unwrap().runtime)
                }
                Token::Unary(op) => ResolvedToken::Unary(*op),
                Token::Binary(op) => ResolvedToken::Binary(*op),
                Token::At(_) => unreachable!(),
            });
            normalized.push(t);
        }
        Ok((bound, normalized))
    }
    fn edit(&mut self, edit: &Edit, expected: Expected, ordinal: usize) -> Result<()> {
        match edit {
            Edit::SetDecimal { position, value } => {
                let id = self.anchor(*position)?;
                let content = Content::Decimal(*value);
                if self.data.records.get(&id).unwrap().content.as_ref() != Some(&content) {
                    self.batch
                        .replace_input(self.data.bindings.get(&id).unwrap().runtime, value.value())
                        .map_err(graph_error)?;
                    self.set(id, content)?;
                }
            }
            Edit::SetFormula {
                position,
                source,
                tokens,
            } => {
                let id = self.anchor(*position)?;
                let (bound, tokens) = self.resolve(tokens)?;
                let content = Content::Formula {
                    source: source.clone(),
                    tokens,
                };
                if self.data.records.get(&id).unwrap().content.as_ref() != Some(&content) {
                    self.batch
                        .set_resolved(self.data.bindings.get(&id).unwrap().runtime, source, bound)
                        .map_err(graph_error)?;
                    self.set(id, content)?;
                }
            }
            Edit::SetFormulaText { position, source } => {
                self.book.axes_at(*position)?;
                let parse_profile = profile::scope(4);
                let parsed = expression::parse_spreadsheet(source);
                let (mut tokens, references, diagnostic) = self.book.lower_text(&parsed);
                drop(parse_profile);
                if tokens.len() > self.book.limits.formula_tokens {
                    return Err(Error::Limit);
                }
                let id = self.anchor(*position)?;
                let node = self.data.bindings.get(&id).unwrap().runtime;
                let mut bound_references = Vec::new();
                let resolved_graph;
                if let Some(error) = &diagnostic {
                    tokens.clear();
                    resolved_graph = self.batch.revision().map_err(graph_error)?;
                    self.batch
                        .set_invalid(node, source, error.clone())
                        .map_err(graph_error)?;
                } else {
                    let (bound, normalized) = self.resolve(&tokens)?;
                    tokens = normalized;
                    resolved_graph = self.batch.revision().map_err(graph_error)?;
                    self.batch
                        .set_resolved(node, source, bound)
                        .map_err(graph_error)?;
                    for (span, pos) in references {
                        let target = *self.data.positions.get(&pos).unwrap();
                        bound_references.push(FormulaReference {
                            span,
                            submitted_position: pos,
                            target: cell_ref(self.book, target),
                            node: self.data.bindings.get(&target).unwrap().portable,
                        });
                    }
                }
                self.set(
                    id,
                    Content::TextFormula(TextFormula {
                        original_source: source.clone(),
                        references: bound_references,
                        diagnostic,
                        transaction_base: expected,
                        edit_ordinal: ordinal,
                        resolved_graph,
                        tokens,
                    }),
                )?;
            }
            Edit::Clear(position) => {
                self.book.axes_at(*position)?;
                if let Some(id) = self.data.positions.get(position).copied()
                    && self.data.records.get(&id).unwrap().content != Some(Content::Blank)
                {
                    self.batch
                        .replace_input(
                            self.data.bindings.get(&id).unwrap().runtime,
                            PivotDecimal::ZERO,
                        )
                        .map_err(graph_error)?;
                    self.set(id, Content::Blank)?;
                }
            }
            Edit::Move { from, to } => {
                self.book.axes_at(*from)?;
                self.book.axes_at(*to)?;
                let id = *self.data.positions.get(from).ok_or(Error::InvalidTarget)?;
                if from == to {
                    return Ok(());
                }
                if self.data.positions.contains_key(to) {
                    return Err(Error::Occupied);
                }
                self.journal.cells.insert(id);
                self.journal.positions.extend([*from, *to]);
                self.data.positions.remove(from).map_err(|_| Error::Limit)?;
                self.data
                    .positions
                    .insert_private(*to, id)
                    .map_err(|_| Error::Limit)?;
                let mut record = (**self.data.records.get(&id).unwrap()).clone();
                record.position = Some(*to);
                self.data.record(id, record)?;
            }
            Edit::Retire(position) => {
                self.book.axes_at(*position)?;
                let id = *self
                    .data
                    .positions
                    .get(position)
                    .ok_or(Error::InvalidTarget)?;
                self.batch
                    .retire(self.data.bindings.get(&id).unwrap().runtime)
                    .map_err(graph_error)?;
                self.journal.cells.insert(id);
                self.journal.positions.insert(*position);
                self.data
                    .positions
                    .remove(position)
                    .map_err(|_| Error::Limit)?;
                self.data.record(
                    id,
                    Record {
                        position: None,
                        content: None,
                    },
                )?;
                self.data.live -= 1;
            }
        }
        let _profile = profile::scope(8);
        let stats = self.data.stats();
        if stats.estimated_owned_bytes > self.book.limits.bytes {
            return Err(Error::Limit);
        }
        Ok(())
    }
    fn same(&self) -> bool {
        let original = state(self.book);
        self.journal.cells.iter().all(|id| {
            self.data.records.get(id).map(|r| &**r) == original.records.get(id).map(|r| &**r)
                && self.data.bindings.get(id) == original.bindings.get(id)
        }) && self
            .journal
            .positions
            .iter()
            .all(|p| self.data.positions.get(p) == original.positions.get(p))
    }
}
fn apply(
    book: &mut Workbook,
    expected: Expected,
    edits: &[Edit],
    cancel: &CancellationToken,
) -> Result<Expected> {
    let _profile = profile::scope(11);
    book.check(expected)?;
    {
        let _profile = profile::scope(8);
        book.preflight(edits)?;
    }
    if cancel.is_cancelled() {
        return Err(Error::Graph("cancelled".into()));
    }
    let mut units = edits.len().saturating_mul(2);
    let mut text = 0usize;
    let mut work = edits.len().saturating_mul(512);
    for edit in edits {
        match edit {
            Edit::SetFormulaText { source, .. } => {
                units = units.saturating_add(source.len());
                text = text.saturating_add(source.len());
                work = work.saturating_add(source.len().saturating_mul(512));
            }
            Edit::SetFormula { source, tokens, .. } => {
                units = units.saturating_add(tokens.len());
                text = text.saturating_add(source.len());
                work = work
                    .saturating_add(source.len().saturating_mul(512))
                    .saturating_add(tokens.len().saturating_mul(128));
            }
            _ => (),
        }
    }
    let available = book
        .limits
        .allocated_cells
        .saturating_sub(state(book).records.len());
    units = edits
        .len()
        .saturating_add(units.saturating_sub(edits.len()).min(available));
    let core = book.core.stage();
    let charge = core
        .allocation_domain()
        .map_err(graph_error)?
        .reserve(work + 4096)
        .map_err(|_| Error::Limit)?;
    let mut tx = Transaction {
        book,
        data: core.stage_resource(&book.persistent).map_err(graph_error)?,
        batch: core
            .begin_batch(units, units.saturating_mul(32).saturating_add(text))
            .map_err(graph_error)?,
        journal: core
            .prepare_resource(Journal {
                cells: BTreeSet::new(),
                positions: BTreeSet::new(),
                _charge: charge,
            })
            .map_err(graph_error)?,
    };
    for (ordinal, edit) in edits.iter().enumerate() {
        if cancel.is_cancelled() {
            return Err(Error::Graph("cancelled".into()));
        }
        tx.edit(edit, expected, ordinal)?;
    }
    if tx.same() {
        return Ok(expected);
    }
    let next = book.state.checked_add(1).ok_or(Error::Exhausted)?;
    let Transaction { data, batch, .. } = tx;
    let mut core = core;
    {
        let _profile = profile::scope(12);
        core.apply_batch(batch, cancel).map_err(graph_error)?;
    }
    if cancel.is_cancelled() {
        return Err(Error::Graph("cancelled".into()));
    }
    let _profile = profile::scope(9);
    let data = data.freeze();
    book.core = core;
    book.persistent = data;
    book.state = next;
    Ok(book.expected())
}
pub(super) fn read(
    book: &Workbook,
    expected: Expected,
    rectangle: Rectangle,
    max: usize,
) -> Result<RectangleRead> {
    book.check(expected)?;
    book.axes_at(rectangle.first)?;
    book.axes_at(rectangle.last)?;
    if rectangle.first.row > rectangle.last.row || rectangle.first.column > rectangle.last.column {
        return Err(Error::Position);
    }
    let area = (rectangle.last.row as u64 - rectangle.first.row as u64 + 1)
        * (rectangle.last.column as u64 - rectangle.first.column as u64 + 1);
    if area > book.limits.read_area || max > book.limits.read_records {
        return Err(Error::Limit);
    }
    let p = state(book);
    let headers = max
        .checked_mul(std::mem::size_of::<CellView>() + std::mem::size_of::<CellId>())
        .and_then(|n| n.checked_add(4096))
        .ok_or(Error::Limit)?;
    let _headers = p.memory.reserve(headers).map_err(|_| Error::Limit)?;
    let mut ids = Vec::new();
    let mut bytes = 0usize;
    for row in rectangle.first.row..=rectangle.last.row {
        let start = Position::new(row, rectangle.first.column);
        for (pos, id) in p.positions.iter_from(&start) {
            if pos.row != row || pos.column > rectangle.last.column {
                break;
            }
            if ids.len() == max {
                return Err(Error::Limit);
            }
            bytes = bytes
                .checked_add((**p.records.get(id).unwrap()).heap_bytes())
                .ok_or(Error::Limit)?;
            ids.push(*id);
        }
    }
    let _payload = p.memory.reserve(bytes).map_err(|_| Error::Limit)?;
    let cells = ids
        .into_iter()
        .map(|id| view(book, p, id).unwrap())
        .collect();
    Ok(RectangleRead {
        expected,
        rectangle,
        cells,
    })
}
pub(super) fn capture(
    book: &Workbook,
    expected: Expected,
    local: bool,
) -> Result<CalculationCapture> {
    book.check(expected)?;
    let core = if local {
        book.core.capture_local()
    } else {
        book.core.capture_calculation()
    }
    .map_err(|_| Error::InvalidDraft)?;
    Ok(CalculationCapture {
        expected,
        worksheet: book.worksheet,
        limit: book.limits.read_records,
        pinned: book.persistent.clone(),
        local_core: core,
    })
}
pub(super) fn evaluate(capture: &CalculationCapture, outputs: &[CellRef]) -> Result<Calculation> {
    if outputs.len() > capture.limit {
        return Err(Error::Limit);
    }
    let nodes = outputs
        .iter()
        .map(|r| {
            if r.worksheet != capture.worksheet {
                return Err(Error::InvalidTarget);
            }
            let p = &capture.pinned;

            if p.records
                .get(&r.cell)
                .is_none_or(|record| record.position.is_none())
            {
                return Err(Error::InvalidTarget);
            }
            p.bindings
                .get(&r.cell)
                .map(|b| b.runtime)
                .ok_or(Error::InvalidTarget)
        })
        .collect::<Result<Vec<_>>>()?;
    let result = capture.local_core.evaluate(&nodes).map_err(graph_error)?;
    let values = outputs
        .iter()
        .zip(nodes)
        .map(|(r, node)| {
            (
                *r,
                result.values()[&node]
                    .as_ref()
                    .copied()
                    .map_err(|e| match e {
                        EvaluationError::Arithmetic { error, .. } => CellError::Arithmetic(*error),
                        EvaluationError::DependencyFailed { .. } => CellError::DependencyFailed,
                        _ => CellError::InvalidExpression,
                    }),
            )
        })
        .collect();
    Ok(Calculation {
        expected: capture.expected,
        generation: Uuid::new_v4(),
        values,
    })
}
pub(super) fn validate(book: &Workbook, expected: Expected) -> Result<Vec<Diagnostic>> {
    book.check(expected)?;
    let p = state(book);
    let mut diagnostics = Vec::new();
    for (id, record) in p.records.iter() {
        if let Some(Content::TextFormula(f)) = &record.content
            && let Some(error) = &f.diagnostic
        {
            diagnostics.push(Diagnostic::Formula {
                cell: cell_ref(book, *id),
                error: error.clone(),
            });
        }
        if let Some(Content::TextFormula(f)) = &record.content {
            for reference in &f.references {
                if p.records
                    .get(&reference.target.cell)
                    .unwrap()
                    .position
                    .is_none()
                {
                    diagnostics.push(Diagnostic::Formula {
                        cell: cell_ref(book, *id),
                        error: ParseError {
                            offset: reference.span.start,
                            message: "retired cell reference",
                        },
                    });
                }
            }
        }
        let tokens = match &record.content {
            Some(Content::Formula { tokens, .. }) => Some(tokens),
            Some(Content::TextFormula(f)) => Some(&f.tokens),
            _ => None,
        };
        if let Some(tokens) = tokens {
            let mut missing = BTreeSet::new();
            for token in tokens {
                if let Token::Cell(target) = token
                    && p.records.get(&target.cell).unwrap().position.is_none()
                {
                    missing.insert(target.cell);
                }
            }
            for target in missing {
                diagnostics.push(Diagnostic::MissingReference {
                    cell: cell_ref(book, *id),
                    target: cell_ref(book, target),
                });
            }
        }
    }
    match book.core.snapshot() {
        Ok(_) => (),
        Err(SnapshotError::Unorderable { nodes }) => diagnostics.push(Diagnostic::Unorderable {
            cells: nodes
                .iter()
                .map(|n| cell_ref(book, *p.nodes.get(n).unwrap()))
                .collect(),
        }),
        Err(SnapshotError::MissingReference { .. } | SnapshotError::Parse { .. })
            if !diagnostics.is_empty() => {}
        Err(_) => diagnostics.push(Diagnostic::CoreInvalid),
    }
    Ok(diagnostics)
}

impl CalculationCapture {
    /// Bounded point definition read from the persistent capture's original generation.
    pub fn pinned_content(&self, position: Position) -> Result<Option<CellView>> {
        let p = &self.pinned;
        let Some(id) = p.positions.get(&position) else {
            return Ok(None);
        };
        let record = p.records.get(id).unwrap();
        let _copy = p
            .memory
            .reserve(4096 + std::mem::size_of::<CellView>() + (**record).heap_bytes())
            .map_err(|_| Error::Limit)?;
        Ok(Some(CellView {
            reference: CellRef {
                worksheet: self.worksheet,
                cell: *id,
            },
            node: p.bindings.get(id).unwrap().portable,
            position,
            content: record.content.as_ref().unwrap().clone(),
        }))
    }
}
