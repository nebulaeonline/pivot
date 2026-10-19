//! Bounded single-sheet projection with Rust scalar formula parsing and binding.
use pivot_core::{
    evaluation::{EvaluationContext, EvaluationError},
    expression::{self, BinaryOp, Expr, ParseError, UnaryOp},
    model::{
        NodeId, Revision, SnapshotError, WorkingModel,
        transfer::{self, ResolvedToken},
    },
};
use pivot_types::{DecimalError, PivotDecimal};
use std::collections::{BTreeMap, BTreeSet};
use uuid::Uuid;

macro_rules! identity {
    ($name:ident) => {
        #[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
        pub struct $name(Uuid);
        impl $name {
            pub fn from_uuid(id: Uuid) -> Self {
                Self(id)
            }
            pub fn uuid(self) -> Uuid {
                self.0
            }
            fn fresh() -> Self {
                Self(Uuid::new_v4())
            }
        }
    };
}
identity!(WorksheetId);
identity!(CellId);
identity!(GraphNodeId);

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct Position {
    pub row: u32,
    pub column: u32,
}
impl Position {
    pub const fn new(row: u32, column: u32) -> Self {
        Self { row, column }
    }
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct CellRef {
    pub worksheet: WorksheetId,
    pub cell: CellId,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct AxisId {
    pub namespace: Uuid,
    pub sequence: u32,
}
#[derive(Clone, Debug)]
struct Axis {
    namespace: Uuid,
    length: u32,
}
impl Axis {
    // One immutable ordered run, represented without allocating every row/column.
    fn at(&self, position: u32) -> Result<AxisId> {
        if position == 0 || position > self.length {
            return Err(Error::Position);
        }
        Ok(AxisId {
            namespace: self.namespace,
            sequence: position,
        })
    }
}

/// Exact submitted decimal representation, not a second arithmetic implementation.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct DecimalInput {
    lo: u32,
    mid: u32,
    hi: u32,
    negative: bool,
    scale: u32,
}
impl DecimalInput {
    pub fn from_parts(lo: u32, mid: u32, hi: u32, negative: bool, scale: u32) -> Result<Self> {
        PivotDecimal::from_parts(lo, mid, hi, negative, scale).map_err(Error::Numeric)?;
        Ok(Self {
            lo,
            mid,
            hi,
            negative,
            scale,
        })
    }
    pub fn from_decimal(value: PivotDecimal) -> Self {
        let (lo, mid, hi, negative, scale) = value.parts();
        Self {
            lo,
            mid,
            hi,
            negative,
            scale,
        }
    }
    pub fn parts(self) -> (u32, u32, u32, bool, u32) {
        (self.lo, self.mid, self.hi, self.negative, self.scale)
    }
    pub fn value(self) -> PivotDecimal {
        PivotDecimal::from_parts(self.lo, self.mid, self.hi, self.negative, self.scale)
            .expect("validated decimal")
    }
}

/// Typed postfix submission. At resolves once; committed formulas retain Cell refs.
/// Source is provenance only: this proof does not parse spreadsheet formula text.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Token {
    Decimal(DecimalInput),
    At(Position),
    Cell(CellRef),
    Unary(UnaryOp),
    Binary(BinaryOp),
}
/// Original text is provenance, not a rendering of current effective coordinates.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct FormulaReference {
    pub span: std::ops::Range<usize>,
    pub submitted_position: Position,
    pub target: CellRef,
    pub node: GraphNodeId,
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct TextFormula {
    pub original_source: String,
    pub references: Vec<FormulaReference>,
    pub diagnostic: Option<ParseError>,
    /// Transaction base plus zero-based edit ordinal and staged graph revision
    /// identify the exact sequential candidate on which this formula resolved.
    pub transaction_base: Expected,
    pub edit_ordinal: usize,
    pub resolved_graph: Revision,
    tokens: Vec<Token>,
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Content {
    TextFormula(TextFormula),
    Blank,
    Decimal(DecimalInput),
    Formula { source: String, tokens: Vec<Token> },
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct CellView {
    pub reference: CellRef,
    pub node: GraphNodeId,
    pub position: Position,
    pub content: Content,
}

#[derive(Clone, Debug)]
pub enum Edit {
    SetFormulaText {
        position: Position,
        source: String,
    },
    SetDecimal {
        position: Position,
        value: DecimalInput,
    },
    SetFormula {
        position: Position,
        source: String,
        tokens: Vec<Token>,
    },
    Clear(Position),
    Move {
        from: Position,
        to: Position,
    },
    Retire(Position),
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Expected {
    pub session: Uuid,
    pub state: u64,
    pub graph: Revision,
}
#[derive(Clone, Copy, Debug)]
pub struct Limits {
    pub rows: u32,
    pub columns: u32,
    pub live_cells: usize,
    pub allocated_cells: usize,
    pub edits: usize,
    pub formula_tokens: usize,
    pub source_bytes: usize,
    pub bytes: usize,
    pub read_area: u64,
    pub read_records: usize,
}
impl Default for Limits {
    fn default() -> Self {
        Self {
            rows: 1024,
            columns: 1024,
            live_cells: 1024,
            allocated_cells: 8192,
            edits: 256,
            formula_tokens: 4096,
            source_bytes: 4096,
            bytes: 1_048_576,
            read_area: 4096,
            read_records: 1024,
        }
    }
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Error {
    Position,
    InvalidTarget,
    Occupied,
    StaleSession,
    StaleRevision,
    Limit,
    Exhausted,
    Numeric(DecimalError),
    Graph(String),
    InvalidDraft,
}
impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "workbook {self:?}")
    }
}
impl std::error::Error for Error {}
pub type Result<T> = std::result::Result<T, Error>;
fn graph_error(e: impl std::fmt::Display) -> Error {
    Error::Graph(e.to_string())
}

#[derive(Debug, PartialEq, Eq)]
struct Record {
    position: Option<Position>,
    content: Option<Content>,
}
impl Clone for Record {
    fn clone(&self) -> Self {
        #[cfg(feature = "scaling-experiment")]
        pivot_core::experiment_profile::copy(
            std::mem::size_of::<Self>()
                + match &self.content {
                    Some(Content::Formula { source, tokens }) => {
                        source.len() + tokens.len() * std::mem::size_of::<Token>()
                    }
                    Some(Content::TextFormula(f)) => {
                        f.original_source.len()
                            + f.tokens.len() * std::mem::size_of::<Token>()
                            + f.references.len() * std::mem::size_of::<FormulaReference>()
                    }
                    _ => 0,
                },
        );
        Self {
            position: self.position,
            content: self.content.clone(),
        }
    }
}
#[derive(Clone, Debug, PartialEq, Eq)]
struct Binding {
    portable: GraphNodeId,
    runtime: NodeId,
}
#[derive(Clone, Debug, Default, PartialEq, Eq)]
struct SheetState {
    positions: BTreeMap<Position, CellId>,
    records: BTreeMap<CellId, Record>, // Includes small retired identity tombstones.
    bindings: BTreeMap<CellId, Binding>,
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Diagnostic {
    Formula { cell: CellRef, error: ParseError },
    MissingReference { cell: CellRef, target: CellRef },
    Unorderable { cells: Vec<CellRef> },
    CoreInvalid,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Stats {
    pub live_cells: usize,
    pub live_nodes: usize,
    pub allocated_nodes: usize,
    pub formula_tokens: usize,
    pub source_bytes: usize,
    pub estimated_owned_bytes: usize,
}

type LoweredFormula = (
    Vec<Token>,
    Vec<(std::ops::Range<usize>, Position)>,
    Option<ParseError>,
);

pub struct Workbook {
    session: Uuid,
    state: u64,
    worksheet: WorksheetId,
    graph_id: Uuid,
    rows: Axis,
    columns: Axis,
    data: SheetState,
    core: WorkingModel,
    limits: Limits,
}
impl Workbook {
    pub fn new(name: &str, limits: Limits) -> Result<Self> {
        Self::new_checked(name, limits, false)
    }
    fn new_checked(name: &str, limits: Limits, benchmark: bool) -> Result<Self> {
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
            data: SheetState::default(),
            core: WorkingModel::new(name).map_err(graph_error)?,
            limits,
        })
    }
    pub fn worksheet(&self) -> WorksheetId {
        self.worksheet
    }
    pub fn graph_id(&self) -> Uuid {
        self.graph_id
    }
    pub fn expected(&self) -> Expected {
        Expected {
            session: self.session,
            state: self.state,
            graph: self.core.revision(),
        }
    }
    pub fn axes_at(&self, position: Position) -> Result<(AxisId, AxisId)> {
        Ok((
            self.rows.at(position.row)?,
            self.columns.at(position.column)?,
        ))
    }
    fn check(&self, expected: Expected) -> Result<()> {
        if expected.session != self.session {
            return Err(Error::StaleSession);
        }
        if expected.state != self.state || expected.graph != self.core.revision() {
            return Err(Error::StaleRevision);
        }
        Ok(())
    }
    fn reference(&self, cell: CellId) -> CellRef {
        CellRef {
            worksheet: self.worksheet,
            cell,
        }
    }
    fn target(&self, reference: CellRef) -> Result<CellId> {
        if reference.worksheet != self.worksheet || !self.data.records.contains_key(&reference.cell)
        {
            return Err(Error::InvalidTarget);
        }
        Ok(reference.cell)
    }
    fn view(&self, id: CellId) -> Option<CellView> {
        let record = self.data.records.get(&id)?;
        Some(CellView {
            reference: self.reference(id),
            node: self.data.bindings[&id].portable,
            position: record.position?,
            content: record.content.as_ref()?.clone(),
        })
    }
    pub fn cell_at(&self, position: Position) -> Result<Option<CellView>> {
        self.axes_at(position)?;
        Ok(self
            .data
            .positions
            .get(&position)
            .and_then(|id| self.view(*id)))
    }
    pub fn node_for(&self, reference: CellRef) -> Result<GraphNodeId> {
        let id = self.target(reference)?;
        Ok(self.data.bindings[&id].portable)
    }
    pub fn stats(&self) -> Stats {
        let mut tokens = 0usize;
        let mut source = 0usize;
        let mut metadata = 0usize;
        for record in self.data.records.values() {
            match &record.content {
                Some(Content::Formula {
                    source: s,
                    tokens: t,
                }) => {
                    tokens += t.len();
                    source += s.len();
                }
                Some(Content::TextFormula(f)) => {
                    tokens += f.tokens.len();
                    source += f.original_source.len();
                    metadata += f.references.len() * std::mem::size_of::<FormulaReference>();
                }
                _ => (),
            }
        }
        // Logical estimate, not allocator/RSS measurement. BTree node overhead is
        // deliberately represented as a conservative per-entry allowance.
        Stats {
            live_cells: self.data.positions.len(),
            live_nodes: self.core.draft_nodes().count(),
            allocated_nodes: self.data.bindings.len(),
            formula_tokens: tokens,
            source_bytes: source,
            estimated_owned_bytes: self.data.records.len() * 256
                + self.data.positions.len() * 64
                + tokens * std::mem::size_of::<Token>()
                + source
                + metadata,
        }
    }
    fn budget(&self) -> Result<()> {
        #[cfg(feature = "scaling-experiment")]
        let _phase = pivot_core::experiment_profile::scope(3);
        let stats = self.stats();
        if stats.live_cells > self.limits.live_cells
            || stats.allocated_nodes > self.limits.allocated_cells
            || stats.estimated_owned_bytes > self.limits.bytes
        {
            return Err(Error::Limit);
        }
        Ok(())
    }
    fn ensure_anchor(&mut self, position: Position) -> Result<CellId> {
        self.axes_at(position)?;
        if let Some(id) = self.data.positions.get(&position) {
            return Ok(*id);
        }
        if self.data.positions.len() >= self.limits.live_cells
            || self.data.records.len() >= self.limits.allocated_cells
        {
            return Err(Error::Limit);
        }
        let id = CellId::fresh();
        // This is a diagnostic core label, never a fabricated formula identifier.
        let runtime = self
            .core
            .add_input(format!("N{}", self.data.records.len()), PivotDecimal::ZERO)
            .map_err(graph_error)?;
        self.data.records.insert(
            id,
            Record {
                position: Some(position),
                content: Some(Content::Blank),
            },
        );
        self.data.positions.insert(position, id);
        self.data.bindings.insert(
            id,
            Binding {
                portable: GraphNodeId::fresh(),
                runtime,
            },
        );
        Ok(id)
    }
    fn preflight(&self, edits: &[Edit]) -> Result<()> {
        if edits.len() > self.limits.edits {
            return Err(Error::Limit);
        }
        let mut work = 0usize;
        for edit in edits {
            if let Edit::SetFormulaText { source, .. } = edit {
                if source.len() > self.limits.source_bytes {
                    return Err(Error::Limit);
                }
                work = work.checked_add(source.len()).ok_or(Error::Limit)?;
                if work > self.limits.bytes {
                    return Err(Error::Limit);
                }
            }
            if let Edit::SetFormula { source, tokens, .. } = edit {
                if source.len() > self.limits.source_bytes
                    || tokens.len() > self.limits.formula_tokens
                {
                    return Err(Error::Limit);
                }
                work = work
                    .checked_add(source.len())
                    .and_then(|w| w.checked_add(tokens.len() * std::mem::size_of::<Token>()))
                    .ok_or(Error::Limit)?;
                if work > self.limits.bytes {
                    return Err(Error::Limit);
                }
            }
        }
        Ok(())
    }
    pub fn apply(&mut self, expected: Expected, edits: &[Edit]) -> Result<Expected> {
        #[cfg(feature = "scaling-experiment")]
        let _phase = pivot_core::experiment_profile::scope(4);
        self.check(expected)?;
        self.preflight(edits)?;
        #[cfg(feature = "scaling-experiment")]
        let stage_phase = pivot_core::experiment_profile::scope(2);
        let mut candidate = Self {
            session: self.session,
            state: self.state,
            worksheet: self.worksheet,
            graph_id: self.graph_id,
            rows: self.rows.clone(),
            columns: self.columns.clone(),
            data: self.data.clone(),
            core: self.core.stage(),
            limits: self.limits,
        };
        #[cfg(feature = "scaling-experiment")]
        drop(stage_phase);
        for (ordinal, edit) in edits.iter().enumerate() {
            if let Edit::SetFormulaText { position, source } = edit {
                candidate.edit_text(*position, source, expected, ordinal)?;
            } else {
                candidate.edit(edit)?;
            }
            candidate.budget()?;
        }
        if candidate.data == self.data {
            return Ok(self.expected());
        }
        candidate.state = self.state.checked_add(1).ok_or(Error::Exhausted)?;
        #[cfg(feature = "scaling-experiment")]
        let _commit_phase = pivot_core::experiment_profile::scope(10);
        *self = candidate;
        Ok(self.expected())
    }
    fn edit(&mut self, edit: &Edit) -> Result<()> {
        match edit {
            Edit::SetFormulaText { .. } => unreachable!("text routed through resolver"),
            Edit::SetDecimal { position, value } => {
                let id = self.ensure_anchor(*position)?;
                let content = Content::Decimal(*value);
                if self.data.records[&id].content.as_ref() != Some(&content) {
                    self.core
                        .replace_input(self.data.bindings[&id].runtime, value.value())
                        .map_err(graph_error)?;
                    self.data.records.get_mut(&id).unwrap().content = Some(content);
                }
            }
            Edit::SetFormula {
                position,
                source,
                tokens,
            } => {
                let id = self.ensure_anchor(*position)?;
                let (bound, normalized) = self.resolve_tokens(tokens)?;
                let content = Content::Formula {
                    source: source.clone(),
                    tokens: normalized,
                };
                if self.data.records[&id].content.as_ref() != Some(&content) {
                    transfer::set_resolved_expression(
                        &mut self.core,
                        self.data.bindings[&id].runtime,
                        source,
                        &bound,
                    )
                    .map_err(graph_error)?;
                    self.data.records.get_mut(&id).unwrap().content = Some(content);
                }
            }
            Edit::Clear(position) => {
                self.axes_at(*position)?;
                if let Some(id) = self.data.positions.get(position).copied()
                    && self.data.records[&id].content != Some(Content::Blank)
                {
                    self.core
                        .replace_input(self.data.bindings[&id].runtime, PivotDecimal::ZERO)
                        .map_err(graph_error)?;
                    self.data.records.get_mut(&id).unwrap().content = Some(Content::Blank);
                }
            }
            Edit::Move { from, to } => {
                self.axes_at(*from)?;
                self.axes_at(*to)?;
                let id = *self.data.positions.get(from).ok_or(Error::InvalidTarget)?;
                if from == to {
                    return Ok(());
                }
                // Even an explicit blank reference anchor occupies a destination.
                if self.data.positions.contains_key(to) {
                    return Err(Error::Occupied);
                }
                self.data.positions.remove(from);
                self.data.positions.insert(*to, id);
                self.data.records.get_mut(&id).unwrap().position = Some(*to);
            }
            Edit::Retire(position) => {
                self.axes_at(*position)?;
                let id = *self
                    .data
                    .positions
                    .get(position)
                    .ok_or(Error::InvalidTarget)?;
                self.core
                    .remove(self.data.bindings[&id].runtime)
                    .map_err(graph_error)?;
                self.data.positions.remove(position);
                self.data.records.insert(
                    id,
                    Record {
                        position: None,
                        content: None,
                    },
                );
            }
        }
        Ok(())
    }
    fn resolve_tokens(&mut self, tokens: &[Token]) -> Result<(Vec<ResolvedToken>, Vec<Token>)> {
        let mut bound = Vec::with_capacity(tokens.len());
        let mut normalized = Vec::with_capacity(tokens.len());
        for token in tokens {
            let normalized_token = match token {
                Token::At(position) => {
                    let target = self.ensure_anchor(*position)?;
                    Token::Cell(self.reference(target))
                }
                Token::Cell(reference) => {
                    self.target(*reference)?;
                    Token::Cell(*reference)
                }
                other => other.clone(),
            };
            bound.push(match &normalized_token {
                Token::Decimal(v) => ResolvedToken::Decimal(v.value()),
                Token::Cell(reference) => {
                    ResolvedToken::Node(self.data.bindings[&reference.cell].runtime)
                }
                Token::Unary(op) => ResolvedToken::Unary(*op),
                Token::Binary(op) => ResolvedToken::Binary(*op),
                Token::At(_) => unreachable!("normalized position"),
            });
            normalized.push(normalized_token);
        }
        Ok((bound, normalized))
    }
    fn lower_text(&self, parsed: &std::result::Result<Expr, ParseError>) -> LoweredFormula {
        let mut references = vec![];
        let mut tokens = vec![];
        // Validate all coordinate syntax/bounds before materializing any targets.
        let mut diagnostic = parsed.as_ref().err().cloned();
        if let Ok(expr) = &parsed {
            let mut stack = vec![(expr, false)];
            while let Some((expr, visited)) = stack.pop() {
                match expr {
                    Expr::Decimal(v) => tokens.push(Token::Decimal(DecimalInput::from_decimal(*v))),
                    Expr::CellReference { row, column, span } => {
                        let pos = Position::new(*row, *column);
                        if self.axes_at(pos).is_err() {
                            diagnostic = Some(ParseError {
                                offset: span.start,
                                message: "cell reference outside worksheet",
                            });
                            break;
                        }
                        tokens.push(Token::At(pos));
                        // Identity is filled only after the whole expression validates.
                        references.push((span.clone(), pos));
                    }
                    Expr::Unary { op, operand } => {
                        if visited {
                            tokens.push(Token::Unary(*op));
                        } else {
                            stack.push((expr, true));
                            stack.push((operand, false));
                        }
                    }
                    Expr::Binary { op, left, right } => {
                        if visited {
                            tokens.push(Token::Binary(*op));
                        } else {
                            stack.push((expr, true));
                            stack.push((right, false));
                            stack.push((left, false));
                        }
                    }
                    Expr::Name(_) => unreachable!("spreadsheet parser emits typed references"),
                }
            }
        }
        (tokens, references, diagnostic)
    }
    fn edit_text(
        &mut self,
        position: Position,
        source: &str,
        base: Expected,
        ordinal: usize,
    ) -> Result<()> {
        #[cfg(feature = "scaling-experiment")]
        let _phase = pivot_core::experiment_profile::scope(5);
        self.axes_at(position)?;
        let parsed = expression::parse_spreadsheet(source);
        let (mut tokens, references, diagnostic) = self.lower_text(&parsed);
        if tokens.len() > self.limits.formula_tokens {
            return Err(Error::Limit);
        }
        let id = self.ensure_anchor(position)?;
        let mut bound_references = vec![];
        let resolved_graph;
        if let Some(error) = &diagnostic {
            tokens.clear();
            resolved_graph = self.core.revision();
            transfer::set_invalid_expression(
                &mut self.core,
                self.data.bindings[&id].runtime,
                source,
                error.offset,
            )
            .map_err(graph_error)?;
        } else {
            let (bound, normalized) = self.resolve_tokens(&tokens)?;
            tokens = normalized;
            resolved_graph = self.core.revision();
            transfer::set_resolved_expression(
                &mut self.core,
                self.data.bindings[&id].runtime,
                source,
                &bound,
            )
            .map_err(graph_error)?;
            for (span, pos) in references {
                let target = self.reference(self.data.positions[&pos]);
                bound_references.push(FormulaReference {
                    span,
                    submitted_position: pos,
                    target,
                    node: self.node_for(target)?,
                });
            }
        }
        self.data.records.get_mut(&id).unwrap().content = Some(Content::TextFormula(TextFormula {
            original_source: source.into(),
            references: bound_references,
            diagnostic,
            transaction_base: base,
            edit_ordinal: ordinal,
            resolved_graph,
            tokens,
        }));
        Ok(())
    }
    pub fn validate(&self, expected: Expected) -> Result<Vec<Diagnostic>> {
        self.check(expected)?;
        let mut diagnostics = vec![];
        for (&id, record) in &self.data.records {
            if let Some(Content::TextFormula(f)) = &record.content
                && let Some(error) = &f.diagnostic
            {
                diagnostics.push(Diagnostic::Formula {
                    cell: self.reference(id),
                    error: error.clone(),
                });
            }
            if let Some(Content::TextFormula(f)) = &record.content {
                for reference in &f.references {
                    if self.data.records[&reference.target.cell].position.is_none() {
                        diagnostics.push(Diagnostic::Formula {
                            cell: self.reference(id),
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
                        && self.data.records[&target.cell].position.is_none()
                    {
                        missing.insert(target.cell);
                    }
                }
                for target in missing {
                    diagnostics.push(Diagnostic::MissingReference {
                        cell: self.reference(id),
                        target: self.reference(target),
                    });
                }
            }
        }
        match self.core.snapshot() {
            Ok(_) => (),
            Err(SnapshotError::Unorderable { nodes }) => {
                diagnostics.push(Diagnostic::Unorderable {
                    cells: nodes
                        .iter()
                        .map(|node| {
                            self.reference(
                                *self
                                    .data
                                    .bindings
                                    .iter()
                                    .find(|(_, b)| b.runtime == *node)
                                    .expect("owned node")
                                    .0,
                            )
                        })
                        .collect(),
                })
            }
            Err(SnapshotError::MissingReference { .. } | SnapshotError::Parse { .. })
                if !diagnostics.is_empty() => {}
            Err(_) => diagnostics.push(Diagnostic::CoreInvalid),
        }
        Ok(diagnostics)
    }
    pub fn capture(&self, expected: Expected) -> Result<CalculationCapture> {
        self.check(expected)?;
        let snapshot = self.core.snapshot().map_err(|_| Error::InvalidDraft)?;
        let cells = self
            .data
            .positions
            .values()
            .map(|id| (*id, self.data.bindings[id].runtime))
            .collect();
        let positions = self.data.positions.clone();
        Ok(CalculationCapture {
            expected,
            worksheet: self.worksheet,
            context: EvaluationContext::new(snapshot),
            cells,
            positions,
            limit: self.limits.read_records,
        })
    }
    pub fn read(
        &self,
        expected: Expected,
        rectangle: Rectangle,
        max_records: usize,
    ) -> Result<RectangleRead> {
        #[cfg(feature = "scaling-experiment")]
        let _phase = pivot_core::experiment_profile::scope(9);
        self.check(expected)?;
        self.axes_at(rectangle.first)?;
        self.axes_at(rectangle.last)?;
        if rectangle.first.row > rectangle.last.row
            || rectangle.first.column > rectangle.last.column
        {
            return Err(Error::Position);
        }
        let area = (rectangle.last.row as u64 - rectangle.first.row as u64 + 1)
            * (rectangle.last.column as u64 - rectangle.first.column as u64 + 1);
        if area > self.limits.read_area || max_records > self.limits.read_records {
            return Err(Error::Limit);
        }
        let mut cells = vec![];
        for (&position, &id) in self.data.positions.range(rectangle.first..=rectangle.last) {
            if position.column >= rectangle.first.column && position.column <= rectangle.last.column
            {
                if cells.len() == max_records {
                    return Err(Error::Limit);
                }
                cells.push(self.view(id).expect("live position"));
            }
        }
        Ok(RectangleRead {
            expected,
            rectangle,
            cells,
        })
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Rectangle {
    pub first: Position,
    pub last: Position,
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct RectangleRead {
    pub expected: Expected,
    pub rectangle: Rectangle,
    pub cells: Vec<CellView>,
}

pub struct CalculationCapture {
    expected: Expected,
    worksheet: WorksheetId,
    context: EvaluationContext,
    cells: BTreeMap<CellId, NodeId>,
    positions: BTreeMap<Position, CellId>,
    limit: usize,
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum CellError {
    Arithmetic(DecimalError),
    DependencyFailed,
    InvalidExpression,
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Calculation {
    pub expected: Expected,
    pub generation: Uuid,
    pub values: Vec<(CellRef, std::result::Result<PivotDecimal, CellError>)>,
}
impl CalculationCapture {
    pub fn expected(&self) -> Expected {
        self.expected
    }
    pub fn cell_at(&self, position: Position) -> Option<CellRef> {
        self.positions.get(&position).map(|id| CellRef {
            worksheet: self.worksheet,
            cell: *id,
        })
    }
    pub fn evaluate(&self, outputs: &[CellRef]) -> Result<Calculation> {
        if outputs.len() > self.limit {
            return Err(Error::Limit);
        }
        let nodes = outputs
            .iter()
            .map(|r| {
                if r.worksheet != self.worksheet {
                    return Err(Error::InvalidTarget);
                }
                self.cells.get(&r.cell).copied().ok_or(Error::InvalidTarget)
            })
            .collect::<Result<Vec<_>>>()?;
        let result = self.context.evaluate(&nodes).map_err(graph_error)?;
        let values = outputs
            .iter()
            .zip(nodes)
            .map(|(r, node)| {
                let value = result
                    .value(node)
                    .expect("requested node")
                    .as_ref()
                    .copied()
                    .map_err(|e| match e {
                        EvaluationError::Arithmetic { error, .. } => CellError::Arithmetic(*error),
                        EvaluationError::DependencyFailed { .. } => CellError::DependencyFailed,
                        _ => CellError::InvalidExpression,
                    });
                (*r, value)
            })
            .collect();
        Ok(Calculation {
            expected: self.expected,
            generation: Uuid::new_v4(),
            values,
        })
    }
}

#[cfg(test)]
mod formula_costs {
    use super::*;
    use std::{hint::black_box, time::Instant};
    #[test]
    fn report_text_formula_costs() {
        println!(
            "cells,parse_avg_ns,resolve_avg_ns,formula_edit_us,input_edit_us,capture_us,evaluate_us"
        );
        for count in [16u32, 128, 512] {
            let mut w = Workbook::new("Costs", Limits::default()).unwrap();
            let value = DecimalInput::from_parts(1, 0, 0, false, 0).unwrap();
            let edits: Vec<_> = (1..=count)
                .map(|column| Edit::SetDecimal {
                    position: Position::new(1, column),
                    value,
                })
                .collect();
            for batch in edits.chunks(256) {
                w.apply(w.expected(), batch).unwrap();
            }
            let source = "=A1+B1*2";
            let start = Instant::now();
            for _ in 0..500 {
                black_box(expression::parse_spreadsheet(black_box(source)).unwrap());
            }
            let parse = start.elapsed().as_nanos() / 500;
            let parsed = expression::parse_spreadsheet(source);
            let start = Instant::now();
            for _ in 0..500 {
                let (tokens, spans, error) = w.lower_text(black_box(&parsed));
                assert!(error.is_none());
                black_box(spans);
                black_box(w.resolve_tokens(&tokens).unwrap());
            }
            let resolve = start.elapsed().as_nanos() / 500;
            let output = Position::new(2, 1);
            let start = Instant::now();
            w.apply(
                w.expected(),
                &[Edit::SetFormulaText {
                    position: output,
                    source: source.into(),
                }],
            )
            .unwrap();
            let formula = start.elapsed().as_micros();
            let start = Instant::now();
            w.apply(
                w.expected(),
                &[Edit::SetDecimal {
                    position: Position::new(1, 1),
                    value: DecimalInput::from_parts(2, 0, 0, false, 0).unwrap(),
                }],
            )
            .unwrap();
            let input = start.elapsed().as_micros();
            let reference = w.cell_at(output).unwrap().unwrap().reference;
            let start = Instant::now();
            let captured = w.capture(w.expected()).unwrap();
            let capture = start.elapsed().as_micros();
            let start = Instant::now();
            let result = captured.evaluate(&[reference]).unwrap();
            let evaluate = start.elapsed().as_micros();
            assert_eq!(result.values[0].1.clone().unwrap(), "4".parse().unwrap());
            println!(
                "{},{parse},{resolve},{formula},{input},{capture},{evaluate}",
                count + 1
            );
        }
    }
}

/// Isolated benchmark prototype, excluded from default builds.
#[cfg(feature = "scaling-experiment")]
pub mod scaling;
#[cfg(feature = "scaling-experiment")]
pub mod structural;
