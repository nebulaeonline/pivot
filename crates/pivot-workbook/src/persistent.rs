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
    #[cfg(feature = "history-experiment")]
    pub(super) history_payload: usize,
    pub(crate) directory: Option<structured::Directory>,
    pub(crate) adapter: Option<Shared<structured::Adapter>>,
    annotations: Index<CellId, structured::StructuredContent>,
    pub(super) positions: Index<sheets::SheetPosition, CellId>,
    pub(super) sheets: Option<Shared<sheets::Directory>>,
    pub(super) records: Index<CellId, Shared<Record>>,
    pub(super) bindings: Index<CellId, Binding>,
    pub(super) nodes: Index<NodeId, CellId>,
    pub(super) memory: Memory,
    tokens: usize,
    source: usize,
    metadata: usize,
    pub(super) live: usize,
}
impl Key for Position {}
impl Key for CellId {}
impl Footprint for CellId {}
impl Footprint for Binding {}
fn content_size(content: &Option<Content>) -> (usize, usize, usize) {
    match content {
        Some(Content::LiteralText(text)) => (0, text.len(), 0),
        Some(Content::StructuredFormula(f)) => (f.tokens().len(), f.source().len(), 0),
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
            Some(Content::LiteralText(text)) => text.capacity(),
            Some(Content::Formula { source, tokens }) => {
                source.capacity()
                    + tokens.capacity() * std::mem::size_of::<Token>()
                    + tokens
                        .iter()
                        .map(|t| {
                            if let Token::Typed(op, _) = t {
                                op.heap_bytes()
                            } else {
                                0
                            }
                        })
                        .sum::<usize>()
            }
            Some(Content::TextFormula(f)) => {
                f.original_source.capacity()
                    + f.tokens.capacity() * std::mem::size_of::<Token>()
                    + f.tokens
                        .iter()
                        .map(|t| {
                            if let Token::Typed(op, _) = t {
                                op.heap_bytes()
                            } else {
                                0
                            }
                        })
                        .sum::<usize>()
                    + f.references.capacity() * std::mem::size_of::<FormulaReference>()
            }
            _ => 0,
        }
    }
}
impl Data {
    pub(super) fn new(memory: Memory) -> Self {
        Self {
            #[cfg(feature = "history-experiment")]
            history_payload: 0,
            directory: None,
            adapter: None,
            annotations: Index::new(memory.clone()),
            sheets: None,
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
                + self.metadata
                + self.sheets.as_ref().map_or(0, |s| s.heap_bytes()),
        }
    }
    pub(super) fn record(&mut self, id: CellId, record: Record) -> Result<()> {
        let _profile = profile::scope(3);
        let old = self
            .records
            .get(&id)
            .map_or((0, 0, 0), |r| content_size(&r.content));
        let new = content_size(&record.content);
        if let Some(Content::StructuredFormula(formula)) = &record.content {
            if !self.annotations.contains_key(&id) && self.annotations.len() >= 32 {
                return Err(Error::Limit);
            }
            self.annotations
                .insert_private(id, formula.clone())
                .map_err(|_| Error::Limit)?;
        } else {
            self.annotations.remove(&id).map_err(|_| Error::Limit)?;
        }
        #[cfg(feature = "history-experiment")]
        {
            let old = self.records.get(&id).map_or(0, |r| (**r).heap_bytes());
            self.history_payload = self.history_payload - old + record.heap_bytes();
        }
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
        let worksheet = WorksheetId::fresh();
        let rows = Axis {
            namespace: Uuid::new_v4(),
            length: limits.rows,
        };
        let columns = Axis {
            namespace: Uuid::new_v4(),
            length: limits.columns,
        };
        let mut initial = Data::new(runtime.memory());
        initial.install_sheets(vec![SheetInfo {
            id: worksheet,
            name: "Sheet1".into(),
            rows: rows.clone(),
            columns: columns.clone(),
        }])?;
        if initial.stats().estimated_owned_bytes > limits.bytes {
            return Err(Error::Limit);
        }
        let data = core.retain_resource(initial).map_err(graph_error)?;
        Ok(Self {
            #[cfg(feature = "history-experiment")]
            history_content: Uuid::new_v4(),
            #[cfg(feature = "history-experiment")]
            history_active: false,
            #[cfg(feature = "history-experiment")]
            history_recording: None,
            session: Uuid::new_v4(),
            state: 0,
            worksheet,
            graph_id: Uuid::new_v4(),
            rows,
            columns,
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
pub(crate) fn state(book: &Workbook) -> &Data {
    &book.persistent
}
fn cell_ref(book: &Workbook, id: CellId) -> CellRef {
    CellRef {
        worksheet: book
            .persistent
            .records
            .get(&id)
            .map_or(book.worksheet, |r| r.worksheet),
        cell: id,
    }
}
fn view_heap(record: &Record) -> usize {
    match &record.content {
        Some(Content::StructuredFormula(f)) => f.source().len() + std::mem::size_of_val(f.tokens()),
        _ => record.heap_bytes(),
    }
}
fn owned_content(record: &Record) -> Option<Content> {
    record.content.as_ref().map(|content| match content {
        Content::StructuredFormula(f) => Content::Formula {
            source: f.source().into(),
            tokens: f.tokens().to_vec(),
        },
        other => other.clone(),
    })
}
fn view(book: &Workbook, p: &Data, id: CellId) -> Option<CellView> {
    let record = p.records.get(&id)?;
    Some(CellView {
        reference: cell_ref(book, id),
        node: p.bindings.get(&id)?.portable,
        position: record.position?,
        content: owned_content(record)?,
    })
}
pub(super) fn cell_at(book: &Workbook, position: Position) -> Result<Option<CellView>> {
    book.axes_at(position)?;
    let p = state(book);
    let Some(id) = p.positions.get(&sheets::at(book.worksheet, position)) else {
        return Ok(None);
    };
    let _copy = p
        .memory
        .reserve(4096 + std::mem::size_of::<CellView>() + view_heap(p.records.get(id).unwrap()))
        .map_err(|_| Error::Limit)?;
    Ok(view(book, p, *id))
}
pub(super) fn node_for(book: &Workbook, r: CellRef) -> Result<GraphNodeId> {
    if state(book)
        .records
        .get(&r.cell)
        .is_none_or(|record| record.worksheet != r.worksheet)
    {
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
    positions: BTreeSet<sheets::SheetPosition>,
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
        self.anchor_on(self.book.worksheet, position)
    }
    fn anchor_on(&mut self, worksheet: WorksheetId, position: Position) -> Result<CellId> {
        self.book.sheet_info(worksheet)?.rows.at(position.row)?;
        self.book
            .sheet_info(worksheet)?
            .columns
            .at(position.column)?;
        if let Some(id) = self.data.positions.get(&sheets::at(worksheet, position)) {
            return Ok(*id);
        }
        #[cfg(feature = "history-experiment")]
        if self.book.history_active
            && self.batch.history_next_node() >= self.book.limits.allocated_cells as u64
        {
            return Err(Error::Limit);
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
        self.batch.set_blank(node).map_err(graph_error)?;
        drop(identity_profile);
        let _profile = profile::scope(3);
        self.journal.cells.insert(id);
        self.journal
            .positions
            .insert(sheets::at(worksheet, position));
        self.data.record(
            id,
            Record {
                worksheet,
                position: Some(position),
                content: Some(Content::Blank),
            },
        )?;
        self.data
            .positions
            .insert_private(sheets::at(worksheet, position), id)
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
        if let Some(position) = position {
            self.journal
                .positions
                .insert(sheets::at(self.book.worksheet, position));
        }
        let worksheet = self.data.records.get(&id).unwrap().worksheet;
        self.data.record(
            id,
            Record {
                worksheet,
                position,
                content: Some(content),
            },
        )
    }
    fn resolve(
        &mut self,
        source: &str,
        tokens: &[Token],
    ) -> Result<(Vec<ResolvedToken>, Vec<Token>)> {
        let _profile = profile::scope(5);
        let mut bound = Vec::with_capacity(tokens.len());
        let mut normalized = Vec::with_capacity(tokens.len());
        let mut qualified = None;
        for token in tokens {
            match token {
                Token::QualifierAt { name, span } => {
                    if qualified.is_some() {
                        return Err(Error::InvalidTarget);
                    }
                    let worksheet = self
                        .book
                        .sheet_by_name(name)
                        .ok_or_else(|| Error::Graph("unknown worksheet".into()))?;
                    qualified = Some(worksheet);
                    normalized.push(Token::Qualifier {
                        worksheet,
                        span: span.clone(),
                    });
                    continue;
                }
                Token::Qualifier { worksheet, span } => {
                    self.book.sheet_info(*worksheet)?;
                    if qualified.is_some() || source.get(span.clone()).is_none() {
                        return Err(Error::InvalidTarget);
                    }
                    qualified = Some(*worksheet);
                    normalized.push(token.clone());
                    continue;
                }
                _ => (),
            }
            if let Token::Sum { span, .. } | Token::SumAt { span, .. } = token
                && (span.start >= span.end || source.get(span.clone()).is_none())
            {
                return Err(Error::InvalidTarget);
            }
            let t = match token {
                Token::SumAt {
                    kind,
                    first,
                    last,
                    span,
                } => {
                    let worksheet = qualified.take().unwrap_or(self.book.worksheet);
                    self.book.sheet_info(worksheet)?.rows.at(first.row)?;
                    self.book.sheet_info(worksheet)?.columns.at(last.column)?;
                    let rectangle = pivot_core::model::selection::Rectangle {
                        space: pivot_core::model::selection::SpaceId(*worksheet.uuid().as_bytes()),
                        first_row: first.row.min(last.row),
                        last_row: first.row.max(last.row),
                        first_column: first.column.min(last.column),
                        last_column: first.column.max(last.column),
                    };
                    let id = pivot_core::model::selection::SelectionId(*Uuid::new_v4().as_bytes());
                    self.batch
                        .define_selection(id, rectangle)
                        .map_err(graph_error)?;
                    Token::Sum {
                        kind: *kind,
                        id,
                        rectangle,
                        first: *first,
                        last: *last,
                        span: span.clone(),
                    }
                }
                Token::Sum {
                    id,
                    rectangle,
                    first,
                    last,
                    ..
                } => {
                    if rectangle.first_row != first.row.min(last.row)
                        || rectangle.last_row != first.row.max(last.row)
                        || rectangle.first_column != first.column.min(last.column)
                        || rectangle.last_column != first.column.max(last.column)
                    {
                        return Err(Error::InvalidTarget);
                    }
                    let worksheet = WorksheetId::from_uuid(Uuid::from_bytes(rectangle.space.0));
                    if qualified.take().unwrap_or(self.book.worksheet) != worksheet
                        || self.book.sheet_info(worksheet).is_err()
                    {
                        return Err(Error::InvalidTarget);
                    }
                    self.book
                        .axes_at(Position::new(rectangle.first_row, rectangle.first_column))?;
                    self.book
                        .axes_at(Position::new(rectangle.last_row, rectangle.last_column))?;
                    self.batch
                        .define_selection(*id, *rectangle)
                        .map_err(graph_error)?;
                    token.clone()
                }
                Token::At(p) => {
                    let worksheet = qualified.take().unwrap_or(self.book.worksheet);
                    let id = self.anchor_on(worksheet, *p)?;
                    Token::Cell(CellRef {
                        worksheet,
                        cell: id,
                    })
                }
                Token::Cell(r) => {
                    if qualified.take().unwrap_or(self.book.worksheet) != r.worksheet
                        || self
                            .data
                            .records
                            .get(&r.cell)
                            .is_none_or(|record| record.worksheet != r.worksheet)
                    {
                        return Err(Error::InvalidTarget);
                    }
                    Token::Cell(*r)
                }
                other => other.clone(),
            };
            bound.push(match &t {
                Token::Output(binding) => {
                    let directory = self.data.directory.as_ref().ok_or(Error::InvalidTarget)?;
                    if directory.binding != *binding {
                        return Err(Error::InvalidTarget);
                    }
                    let adapter = match &self.data.adapter {
                        Some(a) if a.binding == *binding => a.node,
                        Some(_) => return Err(Error::Limit),
                        None => {
                            let identity = Uuid::new_v4();
                            let (node, slot) = self
                                .batch
                                .add_captured_scalar(
                                    format!("SC_{}", identity.simple()),
                                    *identity.as_bytes(),
                                )
                                .map_err(graph_error)?;
                            let a = structured::Adapter::new(directory, *binding, node, slot)?;
                            self.data.adapter =
                                Some(self.data.memory.alloc(a).map_err(|_| Error::Limit)?);
                            node
                        }
                    };
                    ResolvedToken::Node(adapter)
                }
                Token::Sum { kind, id, .. } => {
                    if *kind == pivot_core::expression::AggregateKind::Sum {
                        ResolvedToken::Sum(*id)
                    } else {
                        ResolvedToken::Aggregate(*kind, *id)
                    }
                }
                Token::Decimal(v) => ResolvedToken::Decimal(v.value()),
                Token::Cell(r) => {
                    ResolvedToken::Node(self.data.bindings.get(&r.cell).unwrap().runtime)
                }
                Token::Typed(op, n) => ResolvedToken::Typed(op.clone(), *n),
                Token::Unary(op) => ResolvedToken::Unary(*op),
                Token::Control(c) => ResolvedToken::Control(*c),
                Token::Binary(op) => ResolvedToken::Binary(*op),
                Token::At(_)
                | Token::SumAt { .. }
                | Token::QualifierAt { .. }
                | Token::Qualifier { .. } => unreachable!(),
            });
            normalized.push(t);
        }
        if qualified.is_some() {
            return Err(Error::InvalidTarget);
        }
        Ok((bound, normalized))
    }
    fn edit(&mut self, edit: &Edit, expected: Expected, ordinal: usize) -> Result<()> {
        match edit {
            Edit::SetBoolean { position, value } => {
                let id = self.anchor(*position)?;
                let content = Content::Boolean(*value);
                if self.data.records.get(&id).unwrap().content.as_ref() != Some(&content) {
                    self.batch
                        .set_resolved(
                            self.data.bindings.get(&id).unwrap().runtime,
                            "",
                            vec![ResolvedToken::Typed(
                                pivot_core::typed::Op::Boolean(*value),
                                0,
                            )],
                        )
                        .map_err(graph_error)?;
                    self.set(id, content)?;
                }
            }
            Edit::SetEntry { position, source } => {
                let classified =
                    crate::entry::classify(*position, source, self.book.limits.source_bytes)?;
                self.edit(&classified, expected, ordinal)?;
            }
            Edit::SetText { position, text } => {
                crate::entry::validate_text(text, self.book.limits.source_bytes)?;
                let id = self.anchor(*position)?;
                let content = Content::LiteralText(text.clone());
                if self.data.records.get(&id).unwrap().content.as_ref() != Some(&content) {
                    self.batch
                        .set_resolved(
                            self.data.bindings.get(&id).unwrap().runtime,
                            "",
                            vec![ResolvedToken::Typed(
                                pivot_core::typed::Op::String(text.clone()),
                                0,
                            )],
                        )
                        .map_err(graph_error)?;
                    self.set(id, content)?;
                }
            }
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
                let (bound, tokens) = self.resolve(source, tokens)?;
                let content = if tokens.iter().any(|t| matches!(t, Token::Output(_))) {
                    if let Some(Content::StructuredFormula(old)) =
                        self.data.records.get(&id).unwrap().content.as_ref()
                        && old.source() == source
                        && old.tokens() == tokens.as_slice()
                    {
                        Content::StructuredFormula(old.clone())
                    } else {
                        if !self.data.annotations.contains_key(&id)
                            && self.data.annotations.len() >= 32
                        {
                            return Err(Error::Limit);
                        }
                        let scope = &self
                            .data
                            .directory
                            .as_ref()
                            .ok_or(Error::InvalidTarget)?
                            .scope;
                        Content::StructuredFormula(structured::StructuredContent::new(
                            &self.data.memory,
                            scope,
                            source,
                            tokens,
                        )?)
                    }
                } else {
                    Content::Formula {
                        source: source.clone(),
                        tokens,
                    }
                };
                if self.data.records.get(&id).unwrap().content.as_ref() != Some(&content) {
                    self.batch
                        .set_resolved(self.data.bindings.get(&id).unwrap().runtime, source, bound)
                        .map_err(graph_error)?;
                    self.set(id, content)?;
                }
            }
            Edit::SetFormulaText { position, source }
            | Edit::PreserveRejectedFormula { position, source } => {
                self.book.axes_at(*position)?;
                let parse_profile = profile::scope(4);
                let parsed = if matches!(edit, Edit::PreserveRejectedFormula { .. }) {
                    Err(ParseError {
                        offset: 0,
                        message: clipboard::PRESERVED_REJECTION,
                    })
                } else {
                    expression::parse_spreadsheet(source)
                };
                let (mut tokens, references, mut diagnostic) = self.book.lower_text(&parsed);
                if diagnostic.is_none()
                    && let Some(span) = tokens.iter().find_map(|t| {
                        if let Token::QualifierAt { name, span } = t {
                            self.book
                                .sheet_by_name(name)
                                .is_none()
                                .then_some(span.clone())
                        } else {
                            None
                        }
                    })
                {
                    diagnostic = Some(ParseError {
                        offset: span.start,
                        message: "unknown worksheet",
                    });
                    tokens.clear();
                }
                if matches!(edit, Edit::PreserveRejectedFormula { .. }) {
                    diagnostic = Some(ParseError {
                        offset: 0,
                        message: clipboard::PRESERVED_REJECTION,
                    });
                }
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
                        .set_invalid(
                            node,
                            source,
                            ParseError {
                                offset: error.offset,
                                message: "unexpected trailing input",
                            },
                        )
                        .map_err(graph_error)?;
                } else {
                    let (bound, normalized) = self.resolve(source, &tokens)?;
                    tokens = normalized;
                    resolved_graph = self.batch.revision().map_err(graph_error)?;
                    self.batch
                        .set_resolved(node, source, bound)
                        .map_err(graph_error)?;
                    let targets: Vec<_> = tokens
                        .iter()
                        .filter_map(|t| {
                            if let Token::Cell(r) = t {
                                Some(*r)
                            } else {
                                None
                            }
                        })
                        .collect();
                    for ((span, pos), target) in references.into_iter().zip(targets) {
                        bound_references.push(FormulaReference {
                            span,
                            submitted_position: pos,
                            target,
                            node: self.data.bindings.get(&target.cell).unwrap().portable,
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
                if let Some(id) = self
                    .data
                    .positions
                    .get(&sheets::at(self.book.worksheet, *position))
                    .copied()
                    && self.data.records.get(&id).unwrap().content != Some(Content::Blank)
                {
                    self.batch
                        .set_blank(self.data.bindings.get(&id).unwrap().runtime)
                        .map_err(graph_error)?;
                    self.set(id, Content::Blank)?;
                }
            }
            Edit::Move { from, to } => {
                self.book.axes_at(*from)?;
                self.book.axes_at(*to)?;
                let id = *self
                    .data
                    .positions
                    .get(&sheets::at(self.book.worksheet, *from))
                    .ok_or(Error::InvalidTarget)?;
                if from == to {
                    return Ok(());
                }
                if self
                    .data
                    .positions
                    .contains_key(&sheets::at(self.book.worksheet, *to))
                {
                    return Err(Error::Occupied);
                }
                self.journal.cells.insert(id);
                self.journal.positions.extend([
                    sheets::at(self.book.worksheet, *from),
                    sheets::at(self.book.worksheet, *to),
                ]);
                self.data
                    .positions
                    .remove(&sheets::at(self.book.worksheet, *from))
                    .map_err(|_| Error::Limit)?;
                self.data
                    .positions
                    .insert_private(sheets::at(self.book.worksheet, *to), id)
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
                    .get(&sheets::at(self.book.worksheet, *position))
                    .ok_or(Error::InvalidTarget)?;
                self.batch
                    .retire(self.data.bindings.get(&id).unwrap().runtime)
                    .map_err(graph_error)?;
                self.journal.cells.insert(id);
                self.journal
                    .positions
                    .insert(sheets::at(self.book.worksheet, *position));
                self.data
                    .positions
                    .remove(&sheets::at(self.book.worksheet, *position))
                    .map_err(|_| Error::Limit)?;
                self.data.record(
                    id,
                    Record {
                        worksheet: self.book.worksheet,
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
pub(super) fn apply_fill(
    book: &mut Workbook,
    expected: Expected,
    edits: &[Edit],
    cancel: &CancellationToken,
) -> Result<Expected> {
    apply_mode(book, expected, edits, cancel, true)
}
fn apply(
    book: &mut Workbook,
    expected: Expected,
    edits: &[Edit],
    cancel: &CancellationToken,
) -> Result<Expected> {
    apply_mode(book, expected, edits, cancel, false)
}
fn apply_mode(
    book: &mut Workbook,
    expected: Expected,
    edits: &[Edit],
    cancel: &CancellationToken,
    reject_cycles: bool,
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
            Edit::SetEntry { source, .. }
            | Edit::SetText { text: source, .. }
            | Edit::SetFormulaText { source, .. }
            | Edit::PreserveRejectedFormula { source, .. } => {
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
    units = units
        .saturating_add(edits.len().saturating_mul(2))
        .saturating_add(text / 4);
    let core = book.core.stage();
    #[cfg(feature = "history-experiment")]
    let core = if book.history_recording.is_some() {
        core.history_recording_stage(true).map_err(graph_error)?
    } else {
        core
    };
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
    for position in tx.journal.positions.iter() {
        let node = tx.data.positions.get(position).and_then(|id| {
            let record = tx.data.records.get(id)?;
            if matches!(record.content, Some(Content::Blank) | None) {
                None
            } else {
                Some(tx.data.bindings.get(id).unwrap().runtime)
            }
        });
        tx.batch
            .place(
                pivot_core::model::selection::SpaceId(*position.worksheet.uuid().as_bytes()),
                position.row,
                position.column,
                node,
            )
            .map_err(graph_error)?;
    }
    if tx.data.annotations.is_empty()
        && let Some(adapter) = tx.data.adapter.take()
    {
        tx.batch.retire(adapter.node).map_err(graph_error)?;
    }
    let next = book.state.checked_add(1).ok_or(Error::Exhausted)?;
    let cycle_outputs = if reject_cycles {
        tx.journal
            .positions
            .iter()
            .filter_map(|p| {
                tx.data
                    .positions
                    .get(p)
                    .and_then(|id| tx.data.bindings.get(id))
                    .map(|b| b.runtime)
            })
            .collect::<Vec<_>>()
    } else {
        Vec::new()
    };
    #[cfg(feature = "history-experiment")]
    let recording = if let Some(keys) = &book.history_recording {
        let mut keys = keys.clone();
        for id in &tx.journal.cells {
            keys.insert(*id, ()).map_err(|_| Error::Limit)?;
        }
        Some(keys)
    } else {
        None
    };
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
    if reject_cycles {
        core.capture_local()
            .map_err(graph_error)?
            .validate_acyclic(&cycle_outputs, cancel)
            .map_err(graph_error)?;
    }
    let data = data.freeze();
    book.core = core;
    book.persistent = data;
    book.state = next;
    #[cfg(feature = "history-experiment")]
    {
        book.history_content = Uuid::new_v4();
        book.history_recording = recording;
    }
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
        let start = sheets::at(book.worksheet, Position::new(row, rectangle.first.column));
        for (pos, id) in p.positions.iter_from(&start) {
            if pos.worksheet != book.worksheet
                || pos.row != row
                || pos.column > rectangle.last.column
            {
                break;
            }
            if ids.len() == max {
                return Err(Error::Limit);
            }
            bytes = bytes
                .checked_add(view_heap(p.records.get(id).unwrap()))
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
        scalar_context: None,
        source_fault: None,
    })
}
pub(super) fn evaluate(capture: &CalculationCapture, outputs: &[CellRef]) -> Result<Calculation> {
    let result = evaluate_values(capture, outputs)?;
    Ok(Calculation {
        expected: result.expected,
        generation: result.generation,
        values: result
            .values
            .into_iter()
            .map(|(id, value)| {
                (
                    id,
                    value
                        .map_err(|e| {
                            if matches!(e, CellError::Typed(pivot_core::typed::Error::TypeMismatch))
                            {
                                CellError::NonNumericInput
                            } else {
                                e
                            }
                        })
                        .and_then(|v| match v {
                            pivot_types::PivotValue::Decimal(d) => Ok(d),
                            pivot_types::PivotValue::Blank => Ok(PivotDecimal::ZERO),
                            _ => Err(CellError::NonNumericInput),
                        }),
                )
            })
            .collect(),
    })
}
pub(super) fn evaluate_values(
    capture: &CalculationCapture,
    outputs: &[CellRef],
) -> Result<TypedCalculation> {
    evaluate_values_cancelled(capture, outputs, &CancellationToken::default())
}
pub(super) fn evaluate_values_cancelled(
    capture: &CalculationCapture,
    outputs: &[CellRef],
    cancel: &CancellationToken,
) -> Result<TypedCalculation> {
    if outputs.len() > capture.limit {
        return Err(Error::Limit);
    }
    let nodes = outputs
        .iter()
        .map(|r| {
            if capture
                .pinned
                .records
                .get(&r.cell)
                .is_none_or(|record| record.worksheet != r.worksheet)
            {
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
    let result = capture
        .local_core
        .evaluate_values(
            &nodes,
            capture.scalar_context.as_ref().map(|c| &c.core),
            cancel,
        )
        .map_err(graph_error)?;
    let values = outputs
        .iter()
        .zip(nodes)
        .map(|(r, node)| {
            (
                *r,
                result.values()[&node]
                    .as_ref()
                    .cloned()
                    .map_err(|e| match e {
                        EvaluationError::CapturedSourceFailed { fault_id, .. }
                        | EvaluationError::DependencyFailed {
                            fault_id: Some(fault_id),
                            ..
                        } => match capture.source_fault {
                            Some((id, kind)) if id == *fault_id => {
                                CellError::CapturedSourceFailed {
                                    fault_id: *fault_id,
                                    kind,
                                }
                            }
                            _ => CellError::InvalidExpression,
                        },
                        EvaluationError::NoNumericValues { operation, .. }
                        | EvaluationError::NoNumericArguments { operation, .. } => {
                            CellError::NoNumericValues(*operation)
                        }
                        EvaluationError::NonNumericInput { .. } => CellError::NonNumericInput,
                        EvaluationError::Typed { error, .. } => CellError::Typed(error.clone()),
                        EvaluationError::Arithmetic { error, .. } => CellError::Arithmetic(*error),
                        EvaluationError::DependencyFailed { origin, .. } => {
                            if let Some(Err(EvaluationError::Typed { error, .. })) =
                                result.values().get(origin)
                            {
                                return CellError::Typed(error.clone());
                            }
                            if matches!(
                                result.values().get(origin),
                                Some(Err(EvaluationError::NonNumericInput { .. }))
                            ) {
                                CellError::NonNumericInput
                            } else {
                                CellError::DependencyFailed
                            }
                        }
                        _ => CellError::InvalidExpression,
                    }),
            )
        })
        .collect();
    Ok(TypedCalculation {
        evaluated_nodes: result.evaluated_nodes(),
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
            Some(Content::Formula { tokens, .. }) => Some(tokens.as_slice()),
            Some(Content::StructuredFormula(f)) => Some(f.tokens()),
            Some(Content::TextFormula(f)) => Some(f.tokens.as_slice()),
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
    match book.core.capture_calculation() {
        Ok(_) => (),
        Err(pivot_core::model::integration::CaptureFailure::Invalid(
            SnapshotError::Unorderable { nodes },
        )) => diagnostics.push(Diagnostic::Unorderable {
            cells: nodes
                .iter()
                .map(|n| cell_ref(book, *p.nodes.get(n).unwrap()))
                .collect(),
        }),
        Err(pivot_core::model::integration::CaptureFailure::Invalid(
            SnapshotError::MissingReference { .. } | SnapshotError::Parse { .. },
        )) if !diagnostics.is_empty() => {}
        Err(_) => diagnostics.push(Diagnostic::CoreInvalid),
    }
    Ok(diagnostics)
}

impl CalculationCapture {
    /// Bounded point definition read from the persistent capture's original generation.
    pub fn pinned_content(&self, position: Position) -> Result<Option<CellView>> {
        let p = &self.pinned;
        let Some(id) = p.positions.get(&sheets::at(self.worksheet, position)) else {
            return Ok(None);
        };
        let record = p.records.get(id).unwrap();
        let _copy = p
            .memory
            .reserve(4096 + std::mem::size_of::<CellView>() + view_heap(record))
            .map_err(|_| Error::Limit)?;
        Ok(Some(CellView {
            reference: CellRef {
                worksheet: self.worksheet,
                cell: *id,
            },
            node: p.bindings.get(id).unwrap().portable,
            position,
            content: owned_content(record).unwrap(),
        }))
    }
}

#[cfg(feature = "history-experiment")]
pub(super) fn history_weight(book: &Workbook) -> Result<usize> {
    let d = state(book);
    if book.sheets().len() != 1
        || book.sheets()[0].name != "Sheet1"
        || d.records.iter().any(|(_, r)| match &r.content {
            Some(Content::TextFormula(f)) => f
                .tokens
                .iter()
                .any(|t| matches!(t, Token::Typed(..) | Token::Qualifier { .. })),
            Some(Content::Formula { tokens, .. }) => tokens
                .iter()
                .any(|t| matches!(t, Token::Typed(..) | Token::Qualifier { .. })),
            Some(Content::Boolean(_)) => true,
            _ => false,
        })
        || d.directory.is_some()
        || d.adapter.is_some()
        || !d.annotations.is_empty()
    {
        return Err(Error::InvalidTarget);
    }
    let bytes = 4096u128
        + d.history_payload as u128
        + d.positions.len() as u128
            * Index::<sheets::SheetPosition, CellId>::history_entry_header() as u128
        + d.records.len() as u128
            * (Index::<CellId, Shared<Record>>::history_entry_header()
                + pivot_core::model::integration::resources::history_allocation_header::<Record>())
                as u128
        + d.bindings.len() as u128 * Index::<CellId, Binding>::history_entry_header() as u128
        + d.nodes.len() as u128 * Index::<NodeId, CellId>::history_entry_header() as u128;
    usize::try_from(bytes).map_err(|_| Error::Exhausted)
}
