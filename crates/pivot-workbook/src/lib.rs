//! Bounded single-sheet projection with Rust scalar formula parsing and binding.
use pivot_core::{
    evaluation::EvaluationError,
    expression::{self, BinaryOp, Expr, ParseError, UnaryOp},
    model::{NodeId, Revision, SnapshotError, WorkingModel, transfer::ResolvedToken},
};
pub mod entry;
#[doc(hidden)]
pub mod projection;
use pivot_types::{DecimalError, PivotDecimal};
use std::collections::BTreeSet;
use uuid::Uuid;
pub mod clipboard;
pub mod csv;
mod fill;
pub mod formatting;
#[cfg(feature = "history-experiment")]
pub mod history_experiment;
mod persistent;
mod sheets;
pub use fill::FillDirection;
pub use sheets::SheetInfo;
#[doc(hidden)]
pub mod structured;
pub mod transfer;

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
#[derive(Clone, Debug, PartialEq, Eq)]
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
/// Typed source is provenance; SetFormulaText uses the shared spreadsheet parser.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Token {
    Typed(pivot_core::typed::Op, usize),
    QualifierAt {
        name: String,
        span: std::ops::Range<usize>,
    },
    Qualifier {
        worksheet: WorksheetId,
        span: std::ops::Range<usize>,
    },
    Control(pivot_core::expression::AggregateControl),
    #[doc(hidden)]
    Output(structured::OutputBinding),
    SumAt {
        kind: pivot_core::expression::AggregateKind,
        first: Position,
        last: Position,
        span: std::ops::Range<usize>,
    },
    Sum {
        kind: pivot_core::expression::AggregateKind,
        id: pivot_core::model::selection::SelectionId,
        rectangle: pivot_core::model::selection::Rectangle,
        first: Position,
        last: Position,
        span: std::ops::Range<usize>,
    },
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
    pub tokens: Vec<Token>,
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Content {
    Boolean(bool),
    LiteralText(String),
    TextFormula(TextFormula),
    Blank,
    Decimal(DecimalInput),
    Formula {
        source: String,
        tokens: Vec<Token>,
    },
    #[doc(hidden)]
    StructuredFormula(structured::StructuredContent),
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
    SetBoolean {
        position: Position,
        value: bool,
    },
    SetText {
        position: Position,
        text: String,
    },
    SetEntry {
        position: Position,
        source: String,
    },
    /// Preserve a non-executable clipboard draft; only explicit SetFormulaText resubmits it.
    #[doc(hidden)]
    PreserveRejectedFormula {
        position: Position,
        source: String,
    },
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
    pub styles: formatting::StyleLimits,
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
            styles: formatting::StyleLimits::default(),
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
    worksheet: WorksheetId,
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
            worksheet: self.worksheet,
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

pub type LoweredFormula = (
    Vec<Token>,
    Vec<(std::ops::Range<usize>, Position)>,
    Option<ParseError>,
);

pub struct Workbook {
    #[cfg(feature = "history-experiment")]
    history_content: Uuid,
    #[cfg(feature = "history-experiment")]
    history_active: bool,
    #[cfg(feature = "history-experiment")]
    history_recording: Option<pivot_core::model::integration::resources::Index<CellId, ()>>,
    session: Uuid,
    state: u64,
    worksheet: WorksheetId,
    graph_id: Uuid,
    rows: Axis,
    columns: Axis,
    persistent: persistent::State,
    core: WorkingModel,
    limits: Limits,
}
impl Workbook {
    /// Convenience workbooks share bounded runtime resources, never one pool per workbook.
    pub fn new(name: &str, limits: Limits) -> Result<Self> {
        pivot_core::model::integration::IntegrationRuntime::with_convenience(|runtime| {
            Self::new_in(name, limits, runtime)
        })
        .map_err(graph_error)?
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
    fn preflight(&self, edits: &[Edit]) -> Result<()> {
        if edits.len() > self.limits.edits {
            return Err(Error::Limit);
        }
        let mut work = 0usize;
        for edit in edits {
            if let Edit::SetText { text: source, .. }
            | Edit::SetEntry { source, .. }
            | Edit::SetFormulaText { source, .. }
            | Edit::PreserveRejectedFormula { source, .. } = edit
            {
                let payload = if matches!(edit, Edit::SetEntry { .. }) {
                    source.strip_prefix('\'').unwrap_or(source)
                } else {
                    source
                };
                if payload.len() > self.limits.source_bytes {
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
    fn lower_text(&self, parsed: &std::result::Result<Expr, ParseError>) -> LoweredFormula {
        Self::lower_text_axes(&self.rows, &self.columns, parsed, None)
    }
    pub fn lower_text_bounds(
        rows: u32,
        columns: u32,
        parsed: &std::result::Result<Expr, ParseError>,
    ) -> LoweredFormula {
        Self::lower_text_axes(
            &Axis {
                namespace: Uuid::nil(),
                length: rows,
            },
            &Axis {
                namespace: Uuid::nil(),
                length: columns,
            },
            parsed,
            None,
        )
    }
    fn lower_text_axes(
        rows: &Axis,
        columns: &Axis,
        parsed: &std::result::Result<Expr, ParseError>,
        output_binding: Option<structured::OutputBinding>,
    ) -> LoweredFormula {
        let mut references = vec![];
        let mut tokens = vec![];
        // Validate all coordinate syntax/bounds before materializing any targets.
        let mut diagnostic = parsed.as_ref().err().cloned();
        if let Ok(expr) = &parsed {
            let mut stack = vec![(expr, 0u8)];
            while let Some((expr, visited)) = stack.pop() {
                if visited >= 2 {
                    use pivot_core::expression::AggregateControl as C;
                    tokens.push(Token::Control(match visited {
                        2 => C::Range,
                        3 => C::Value,
                        _ => C::End,
                    }));
                    continue;
                }
                match expr {
                    Expr::Typed { op, arguments } => {
                        if visited == 1 {
                            tokens.push(Token::Typed(op.clone(), arguments.len()));
                        } else {
                            stack.push((expr, 1));
                            for argument in arguments.iter().rev() {
                                stack.push((argument, 0));
                            }
                        }
                    }
                    Expr::AggregateCall { kind, arguments } => {
                        use pivot_core::expression::AggregateControl as C;
                        tokens.push(Token::Control(C::Start(*kind)));
                        stack.push((expr, 4));
                        for arg in arguments.iter().rev() {
                            stack.push((
                                arg.expression(),
                                if matches!(
                                    arg,
                                    pivot_core::expression::AggregateArgument::Range(_)
                                ) {
                                    2
                                } else {
                                    3
                                },
                            ));
                            stack.push((arg.expression(), 0));
                        }
                    }
                    Expr::SumRange {
                        kind,
                        first,
                        last,
                        span,
                        first_reference,
                        ..
                    } => {
                        if let Some(q) = &first_reference.qualifier {
                            tokens.push(Token::QualifierAt {
                                name: q.name.clone(),
                                span: q.span.clone(),
                            });
                        }
                        let first = Position::new(first.0, first.1);
                        let last = Position::new(last.0, last.1);
                        if rows.at(first.row).is_err()
                            || columns.at(first.column).is_err()
                            || rows.at(last.row).is_err()
                            || columns.at(last.column).is_err()
                        {
                            diagnostic = Some(ParseError {
                                offset: span.start,
                                message: "cell reference outside worksheet",
                            });
                            break;
                        }
                        tokens.push(Token::SumAt {
                            kind: *kind,
                            first,
                            last,
                            span: span.clone(),
                        });
                    }
                    Expr::Decimal(v) => tokens.push(Token::Decimal(DecimalInput::from_decimal(*v))),
                    Expr::CellReference {
                        row,
                        column,
                        span,
                        qualifier,
                        ..
                    } => {
                        if let Some(q) = qualifier {
                            tokens.push(Token::QualifierAt {
                                name: q.name.clone(),
                                span: q.span.clone(),
                            });
                        }
                        let pos = Position::new(*row, *column);
                        if rows.at(pos.row).is_err() || columns.at(pos.column).is_err() {
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
                        if visited == 1 {
                            tokens.push(Token::Unary(*op));
                        } else {
                            stack.push((expr, 1));
                            stack.push((operand, 0));
                        }
                    }
                    Expr::Binary { op, left, right } => {
                        if visited == 1 {
                            tokens.push(Token::Binary(*op));
                        } else {
                            stack.push((expr, 1));
                            stack.push((right, 0));
                            stack.push((left, 0));
                        }
                    }
                    Expr::CapturedOutput { span, .. } => {
                        if let Some(binding) = output_binding {
                            tokens.push(Token::Output(binding));
                        } else {
                            diagnostic = Some(ParseError {
                                offset: span.start,
                                message: "structured binding unavailable",
                            });
                            break;
                        }
                    }
                    Expr::Name(_) => {
                        unreachable!("ordinary spreadsheet parser emits typed references")
                    }
                }
            }
        }
        (tokens, references, diagnostic)
    }
    pub fn cell_at(&self, position: Position) -> Result<Option<CellView>> {
        persistent::cell_at(self, position)
    }
    pub fn node_for(&self, reference: CellRef) -> Result<GraphNodeId> {
        persistent::node_for(self, reference)
    }
    pub fn apply(&mut self, expected: Expected, edits: &[Edit]) -> Result<Expected> {
        self.apply_cancelled(expected, edits, &Default::default())
    }
    pub fn validate(&self, expected: Expected) -> Result<Vec<Diagnostic>> {
        persistent::validate(self, expected)
    }
    pub fn capture(&self, expected: Expected) -> Result<CalculationCapture> {
        persistent::capture(self, expected, false)
    }
    pub fn read(
        &self,
        expected: Expected,
        rectangle: Rectangle,
        max_records: usize,
    ) -> Result<RectangleRead> {
        persistent::read(self, expected, rectangle, max_records)
    }
    pub fn selection_metrics(&self) -> pivot_core::model::selection::Metrics {
        self.core.selection_metrics()
    }
    pub fn invalidated_cells(&self) -> Vec<CellRef> {
        self.core
            .invalidated_selection_nodes()
            .iter()
            .filter_map(|id| {
                let cell = self.persistent.nodes.get(id)?;
                let record = self.persistent.records.get(cell)?;
                Some(CellRef {
                    worksheet: record.worksheet,
                    cell: *cell,
                })
            })
            .collect()
    }
    pub fn stats(&self) -> Stats {
        self.persistent.stats()
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
    pinned: persistent::State,
    local_core: pivot_core::model::integration::CalculationCapture,
    scalar_context: Option<structured::CapturedAdapterContext>,
    source_fault: Option<(
        pivot_core::captured::CapturedFaultId,
        structured::SourceFault,
    )>,
    limit: usize,
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum CellError {
    Typed(pivot_core::typed::Error),
    NoNumericValues(pivot_core::expression::AggregateKind),
    CapturedSourceFailed {
        fault_id: pivot_core::captured::CapturedFaultId,
        kind: structured::SourceFault,
    },
    NonNumericInput,
    Arithmetic(DecimalError),
    DependencyFailed,
    InvalidExpression,
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct TypedCalculation {
    pub expected: Expected,
    pub generation: Uuid,
    pub values: Vec<(
        CellRef,
        std::result::Result<pivot_types::PivotValue, CellError>,
    )>,
    pub evaluated_nodes: usize,
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
        self.pinned
            .positions
            .get(&sheets::at(self.worksheet, position))
            .map(|id| CellRef {
                worksheet: self.worksheet,
                cell: *id,
            })
    }
    pub fn evaluate_values(&self, outputs: &[CellRef]) -> Result<TypedCalculation> {
        persistent::evaluate_values(self, outputs)
    }
    pub fn evaluate_values_cancelled(
        &self,
        outputs: &[CellRef],
        cancel: &pivot_core::model::integration::CancellationToken,
    ) -> Result<TypedCalculation> {
        persistent::evaluate_values_cancelled(self, outputs, cancel)
    }
    pub fn evaluate(&self, outputs: &[CellRef]) -> Result<Calculation> {
        persistent::evaluate(self, outputs)
    }
}
