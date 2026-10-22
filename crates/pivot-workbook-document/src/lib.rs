//! Versioned native workbook envelope. No cached results or allocation layouts.
use pivot_core::{
    expression::{self, BinaryOp, UnaryOp},
    model::Revision,
};
use pivot_types::PivotDecimal;
use pivot_workbook::transfer::{CellRecord as Record, PortableWorkbook};
use pivot_workbook::*;
use std::collections::BTreeSet;
use uuid::Uuid;
fn graph_error(e: impl std::fmt::Display) -> Error {
    Error::Graph(e.to_string())
}
struct Axis {
    namespace: Uuid,
    length: u32,
}
impl Axis {
    fn at(&self, n: u32) -> Result<()> {
        if n == 0 || n > self.length {
            Err(Error::Position)
        } else {
            Ok(())
        }
    }
}
use pivot_core::model::integration::IntegrationRuntime;
use pivot_core::model::{
    selection::{Rectangle as SelectionRectangle, SelectionId, SpaceId},
    transfer::{SelectionFormula, SelectionImage, SelectionToken},
};
use pivot_storage::{self as storage, document::ModelDocument};
use sha2::{Digest, Sha256};
use std::collections::BTreeMap;

pub const MAX_BYTES: usize = 4 * 1024 * 1024;
const MAGIC: &[u8; 8] = b"PIVOTWB4";
fn malformed() -> Error {
    Error::Graph("malformed workbook document".into())
}
fn limits() -> storage::Limits {
    storage::Limits::default()
}
struct Writer(Vec<u8>);
impl Writer {
    fn n(&mut self, n: u64) {
        self.0.extend(n.to_le_bytes());
    }
    fn id(&mut self, id: Uuid) {
        self.0.extend(id.as_bytes());
    }
    fn bytes(&mut self, b: &[u8]) {
        self.n(b.len() as u64);
        self.0.extend(b);
    }
    fn text(&mut self, s: &str) {
        self.bytes(s.as_bytes());
    }
    fn pos(&mut self, p: Position) {
        self.n(p.row.into());
        self.n(p.column.into());
    }
    fn dec(&mut self, d: DecimalInput) {
        let (a, b, c, s, e) = d.parts();
        for v in [a, b, c, u32::from(s), e] {
            self.n(v.into());
        }
    }
    fn rev(&mut self, r: Revision) {
        let (a, b) = r.provenance();
        self.n(a);
        self.n(b);
    }
    fn rectangle(&mut self, r: SelectionRectangle) {
        self.id(Uuid::from_bytes(r.space.0));
        for n in [r.first_row, r.last_row, r.first_column, r.last_column] {
            self.n(n.into());
        }
    }
    fn graph_tokens(&mut self, tokens: &[SelectionToken]) {
        self.n(tokens.len() as u64);
        for token in tokens {
            match token {
                SelectionToken::Decimal(d) => {
                    self.n(0);
                    self.dec(DecimalInput::from_decimal(*d));
                }
                SelectionToken::Node(slot) => {
                    self.n(1);
                    self.n(*slot as u64);
                }
                SelectionToken::Sum(id) => {
                    self.n(2);
                    self.id(Uuid::from_bytes(id.0));
                }
                SelectionToken::Unary(op) => {
                    self.n(3);
                    self.n(u64::from(*op == UnaryOp::Minus));
                }
                SelectionToken::Binary(op) => {
                    self.n(4);
                    self.n(match op {
                        BinaryOp::Add => 0,
                        BinaryOp::Subtract => 1,
                        BinaryOp::Multiply => 2,
                        BinaryOp::Divide => 3,
                    });
                }
            }
        }
    }
    fn tokens(&mut self, tokens: &[Token]) {
        self.n(tokens.len() as u64);
        for t in tokens {
            match t {
                Token::Sum {
                    id,
                    rectangle,
                    first,
                    last,
                    span,
                } => {
                    self.n(4);
                    self.id(Uuid::from_bytes(id.0));
                    self.rectangle(*rectangle);
                    self.pos(*first);
                    self.pos(*last);
                    self.n(span.start as u64);
                    self.n(span.end as u64);
                }
                Token::Decimal(d) => {
                    self.n(0);
                    self.dec(*d);
                }
                Token::Cell(c) => {
                    self.n(1);
                    self.id(c.worksheet.uuid());
                    self.id(c.cell.uuid());
                }
                Token::Unary(op) => {
                    self.n(2);
                    self.n(u64::from(*op == UnaryOp::Minus));
                }
                Token::Binary(op) => {
                    self.n(3);
                    self.n(match op {
                        BinaryOp::Add => 0,
                        BinaryOp::Subtract => 1,
                        BinaryOp::Multiply => 2,
                        BinaryOp::Divide => 3,
                    });
                }
                Token::Output(_) | Token::At(_) | Token::SumAt { .. } => {
                    unreachable!("validated ordinary document formulas are bound")
                }
            }
        }
    }
}
struct Reader<'a> {
    bytes: &'a [u8],
    at: usize,
}
impl<'a> Reader<'a> {
    fn take(&mut self, n: usize) -> Result<&'a [u8]> {
        let end = self.at.checked_add(n).ok_or_else(malformed)?;
        let b = self.bytes.get(self.at..end).ok_or_else(malformed)?;
        self.at = end;
        Ok(b)
    }
    fn n(&mut self) -> Result<u64> {
        Ok(u64::from_le_bytes(self.take(8)?.try_into().unwrap()))
    }
    fn count(&mut self, max: usize) -> Result<usize> {
        let n = usize::try_from(self.n()?).map_err(|_| malformed())?;
        if n > max { Err(Error::Limit) } else { Ok(n) }
    }
    fn u32(&mut self) -> Result<u32> {
        u32::try_from(self.n()?).map_err(|_| malformed())
    }
    fn id(&mut self) -> Result<Uuid> {
        let id = Uuid::from_slice(self.take(16)?).map_err(|_| malformed())?;
        if id.is_nil() {
            Err(malformed())
        } else {
            Ok(id)
        }
    }
    fn bytes(&mut self, max: usize) -> Result<&'a [u8]> {
        let n = self.count(max)?;
        self.take(n)
    }
    fn text(&mut self) -> Result<String> {
        String::from_utf8(self.bytes(4096)?.to_vec()).map_err(|_| malformed())
    }
    fn pos(&mut self) -> Result<Position> {
        Ok(Position::new(self.u32()?, self.u32()?))
    }
    fn dec(&mut self) -> Result<DecimalInput> {
        let (a, b, c, s, e) = (
            self.u32()?,
            self.u32()?,
            self.u32()?,
            self.u32()?,
            self.u32()?,
        );
        if s > 1 {
            return Err(malformed());
        }
        DecimalInput::from_parts(a, b, c, s != 0, e)
    }
    fn rev(&mut self) -> Result<Revision> {
        Ok(Revision::from_provenance(self.n()?, self.n()?))
    }
    fn rectangle(&mut self) -> Result<SelectionRectangle> {
        let r = SelectionRectangle {
            space: SpaceId(*self.id()?.as_bytes()),
            first_row: self.u32()?,
            last_row: self.u32()?,
            first_column: self.u32()?,
            last_column: self.u32()?,
        };
        if !r.valid() {
            return Err(malformed());
        }
        Ok(r)
    }
    fn graph_tokens(&mut self) -> Result<Vec<SelectionToken>> {
        let n = self.count(4096)?;
        let mut out = Vec::with_capacity(n);
        for _ in 0..n {
            out.push(match self.n()? {
                0 => SelectionToken::Decimal(self.dec()?.value()),
                1 => SelectionToken::Node(self.count(8191)?),
                2 => SelectionToken::Sum(SelectionId(*self.id()?.as_bytes())),
                3 => SelectionToken::Unary(match self.n()? {
                    0 => UnaryOp::Plus,
                    1 => UnaryOp::Minus,
                    _ => return Err(malformed()),
                }),
                4 => SelectionToken::Binary(match self.n()? {
                    0 => BinaryOp::Add,
                    1 => BinaryOp::Subtract,
                    2 => BinaryOp::Multiply,
                    3 => BinaryOp::Divide,
                    _ => return Err(malformed()),
                }),
                _ => return Err(malformed()),
            });
        }
        Ok(out)
    }
    fn tokens(&mut self) -> Result<Vec<Token>> {
        let n = self.count(4096)?;
        let mut out = Vec::with_capacity(n);
        for _ in 0..n {
            out.push(match self.n()? {
                4 => {
                    let id = SelectionId(*self.id()?.as_bytes());
                    let rectangle = self.rectangle()?;
                    let first = self.pos()?;
                    let last = self.pos()?;
                    let start = self.count(4096)?;
                    let end = self.count(4096)?;
                    Token::Sum {
                        id,
                        rectangle,
                        first,
                        last,
                        span: start..end,
                    }
                }
                0 => Token::Decimal(self.dec()?),
                1 => Token::Cell(CellRef {
                    worksheet: WorksheetId::from_uuid(self.id()?),
                    cell: CellId::from_uuid(self.id()?),
                }),
                2 => Token::Unary(match self.n()? {
                    0 => UnaryOp::Plus,
                    1 => UnaryOp::Minus,
                    _ => return Err(malformed()),
                }),
                3 => Token::Binary(match self.n()? {
                    0 => BinaryOp::Add,
                    1 => BinaryOp::Subtract,
                    2 => BinaryOp::Multiply,
                    3 => BinaryOp::Divide,
                    _ => return Err(malformed()),
                }),
                _ => return Err(malformed()),
            });
        }
        Ok(out)
    }
}
/// Owned coherent export, independent of subsequent edits. Bytes are caller-owned.
#[derive(Default)]
pub struct Timings {
    pub capture_ns: u64,
    pub encode_ns: u64,
    pub decode_ns: u64,
    pub restore_ns: u64,
}
pub struct Export {
    pub timings: Timings,
    pub bytes: Vec<u8>,
    pub checkpoint: Uuid,
    pub expected: Expected,
}
pub struct Document;
impl Document {
    pub fn export_document(book: &Workbook, expected: Expected) -> Result<Export> {
        if book.is_structured_proof() {
            return Err(malformed());
        }
        if book.expected() != expected {
            return Err(Error::StaleRevision);
        }
        let started = std::time::Instant::now();
        let projection = book.export_portable(expected)?;
        let pairs = &projection.cells;
        let checkpoint = Uuid::new_v4();
        let graph = storage::from_image(
            projection.graph.scalar_skeleton,
            pairs.iter().map(|(_, n, _)| n.uuid()).collect(),
            vec![],
            storage::GraphHeader {
                model: projection.graph_id,
                revision: checkpoint,
            },
            limits(),
        )
        .map_err(graph_error)?;
        let capture_ns = started.elapsed().as_nanos() as u64;
        let encoded = std::time::Instant::now();
        let graph = storage::encode_document(
            &ModelDocument {
                format: 1,
                document_id: projection.graph_id,
                state_id: checkpoint,
                graph,
            },
            limits(),
        )
        .map_err(graph_error)?;
        let mut w = Writer(vec![]);
        w.id(projection.worksheet.uuid());
        w.id(projection.rows);
        w.n(projection.row_count.into());
        w.id(projection.columns);
        w.n(projection.column_count.into());
        w.n(projection.state);
        w.bytes(&graph);
        w.n(1); // Explicit coordinate-area / checked row-major SUM policy.
        w.n(projection.graph.identities.len() as u64);
        for (id, rectangle) in &projection.graph.identities {
            w.id(Uuid::from_bytes(id.0));
            w.n(u64::from(rectangle.is_some()));
            if let Some(rectangle) = rectangle {
                w.rectangle(*rectangle);
            }
        }
        w.n(projection.graph.formulas.len() as u64);
        for formula in &projection.graph.formulas {
            w.n(formula.slot as u64);
            w.text(&formula.source);
            w.graph_tokens(&formula.tokens);
        }
        w.n(pairs.len() as u64);
        for (id, node, record) in pairs {
            w.id(id.uuid());
            w.id(node.uuid());
            if let (Some(p), Some(c)) = (record.position, record.content.as_ref()) {
                w.n(1);
                w.pos(p);
                match c {
                    Content::StructuredFormula(_) => return Err(malformed()),
                    Content::LiteralText(text) => {
                        w.n(4);
                        w.text(text);
                    }
                    Content::Blank => w.n(0),
                    Content::Decimal(d) => {
                        w.n(1);
                        w.dec(*d);
                    }
                    Content::Formula { source, tokens } => {
                        w.n(2);
                        w.text(source);
                        w.tokens(tokens);
                    }
                    Content::TextFormula(f) => {
                        w.n(3);
                        w.text(&f.original_source);
                        w.tokens(&f.tokens);
                        w.n(match &f.diagnostic {
                            None => 0,
                            Some(error)
                                if error.message
                                    == pivot_workbook::clipboard::PRESERVED_REJECTION =>
                            {
                                2
                            }
                            Some(_) => 1,
                        });
                        w.id(f.transaction_base.session);
                        w.n(f.transaction_base.state);
                        w.rev(f.transaction_base.graph);
                        w.n(f.edit_ordinal as u64);
                        w.rev(f.resolved_graph);
                        w.n(f.references.len() as u64);
                        for r in &f.references {
                            w.n(r.span.start as u64);
                            w.n(r.span.end as u64);
                            w.pos(r.submitted_position);
                            w.id(r.target.worksheet.uuid());
                            w.id(r.target.cell.uuid());
                            w.id(r.node.uuid());
                        }
                    }
                }
            } else {
                w.n(0);
            }
        }
        if w.0.len() > MAX_BYTES - 48 {
            return Err(Error::Limit);
        }
        let mut bytes = MAGIC.to_vec();
        bytes.extend((w.0.len() as u64).to_le_bytes());
        bytes.extend(Sha256::digest(&w.0));
        bytes.extend(w.0);
        Ok(Export {
            timings: Timings {
                capture_ns,
                encode_ns: encoded.elapsed().as_nanos() as u64,
                ..Default::default()
            },
            bytes,
            checkpoint,
            expected,
        })
    }
    pub fn open_document(
        bytes: &[u8],
        limits: Limits,
        runtime: &IntegrationRuntime,
    ) -> Result<(Workbook, Uuid)> {
        Self::open_document_measured(bytes, limits, runtime, &mut Timings::default())
    }
    pub fn open_document_measured(
        bytes: &[u8],
        limits: Limits,
        runtime: &IntegrationRuntime,
        timings: &mut Timings,
    ) -> Result<(Workbook, Uuid)> {
        let started = std::time::Instant::now();
        if bytes.len() > MAX_BYTES {
            return Err(Error::Limit);
        }
        for version in ["WB1", "WB2", "WB3"] {
            if bytes.starts_with(format!("PIVOT{version}").as_bytes()) {
                return Err(Error::Graph(format!(
                    "unsupported native workbook version {version}; WB4 required"
                )));
            }
        }
        if bytes.len() < 48 || &bytes[..8] != MAGIC {
            return Err(malformed());
        }
        if u64::from_le_bytes(bytes[8..16].try_into().unwrap()) != (bytes.len() - 48) as u64
            || Sha256::digest(&bytes[48..])[..] != bytes[16..48]
        {
            return Err(malformed());
        }
        let mut r = Reader {
            bytes: &bytes[48..],
            at: 0,
        };
        let worksheet = WorksheetId::from_uuid(r.id()?);
        let rows = Axis {
            namespace: r.id()?,
            length: r.u32()?,
        };
        let columns = Axis {
            namespace: r.id()?,
            length: r.u32()?,
        };
        let state = r.n()?;
        if rows.length != limits.rows
            || columns.length != limits.columns
            || rows.namespace == columns.namespace
        {
            return Err(malformed());
        }
        let document =
            storage::decode_document(r.bytes(MAX_BYTES)?, self::limits()).map_err(graph_error)?;
        if !document.graph.ports.is_empty()
            || !document.graph.port_ids.is_empty()
            || document.document_id != document.graph.model
            || document.state_id != document.graph.revision
        {
            return Err(malformed());
        }
        if r.n()? != 1 {
            return Err(Error::Graph("unsupported range semantics policy".into()));
        }
        let selection_count = r.count(pivot_core::model::selection::MAX_IDENTITIES)?;
        let mut identities = Vec::with_capacity(selection_count);
        let mut selection_map = BTreeMap::new();
        for _ in 0..selection_count {
            let id = SelectionId(*r.id()?.as_bytes());
            let rectangle = match r.n()? {
                0 => None,
                1 => Some(r.rectangle()?),
                _ => return Err(malformed()),
            };
            if let Some(rectangle) = rectangle
                && (rectangle.space.0 != *worksheet.uuid().as_bytes()
                    || rectangle.last_row > rows.length
                    || rectangle.last_column > columns.length)
            {
                return Err(malformed());
            }
            if selection_map.insert(id, rectangle).is_some() {
                return Err(malformed());
            }
            identities.push((id, rectangle));
        }
        let formula_count = r.count(pivot_core::model::selection::MAX_CONSUMERS)?;
        let mut formulas = Vec::with_capacity(formula_count);
        let mut overlay = BTreeMap::new();
        for _ in 0..formula_count {
            let f = SelectionFormula {
                slot: r.count(8191)?,
                source: r.text()?,
                tokens: r.graph_tokens()?,
            };
            if overlay.insert(f.slot, formulas.len()).is_some() {
                return Err(malformed());
            }
            formulas.push(f);
        }
        let count = r.count(limits.allocated_cells.min(8192))?;
        if count != document.graph.node_ids.len() {
            return Err(malformed());
        }
        let mut records = BTreeMap::new();
        let mut ids = Vec::new();
        let mut positions = BTreeSet::new();
        let mut live_ids = BTreeSet::new();
        for expected_node in &document.graph.node_ids {
            let id = CellId::from_uuid(r.id()?);
            let node = GraphNodeId::from_uuid(r.id()?);
            if node.uuid() != *expected_node {
                return Err(malformed());
            }
            let record = match r.n()? {
                0 => Record {
                    position: None,
                    content: None,
                },
                1 => {
                    let p = r.pos()?;
                    rows.at(p.row)?;
                    columns.at(p.column)?;
                    if !positions.insert(p) {
                        return Err(malformed());
                    }
                    let content = match r.n()? {
                        4 => Content::LiteralText(r.text()?),
                        0 => Content::Blank,
                        1 => Content::Decimal(r.dec()?),
                        2 => Content::Formula {
                            source: r.text()?,
                            tokens: r.tokens()?,
                        },
                        3 => {
                            let original_source = r.text()?;
                            let tokens = r.tokens()?;
                            let invalid = r.n()?;
                            let diagnostic = match invalid {
                                0 => None,
                                1 => Some(
                                    Workbook::lower_text_bounds(
                                        rows.length,
                                        columns.length,
                                        &expression::parse_spreadsheet(&original_source),
                                    )
                                    .2
                                    .ok_or_else(malformed)?,
                                ),
                                2 => Some(expression::ParseError {
                                    offset: 0,
                                    message: pivot_workbook::clipboard::PRESERVED_REJECTION,
                                }),
                                _ => return Err(malformed()),
                            };
                            let transaction_base = Expected {
                                session: r.id()?,
                                state: r.n()?,
                                graph: r.rev()?,
                            };
                            let edit_ordinal = r.count(255)?;
                            let resolved_graph = r.rev()?;
                            let n = r.count(4096)?;
                            let mut references = Vec::with_capacity(n);
                            for _ in 0..n {
                                let start = r.count(4096)?;
                                let end = r.count(4096)?;
                                let submitted_position = r.pos()?;
                                rows.at(submitted_position.row)?;
                                columns.at(submitted_position.column)?;
                                if start >= end || original_source.get(start..end).is_none() {
                                    return Err(malformed());
                                }
                                references.push(FormulaReference {
                                    span: start..end,
                                    submitted_position,
                                    target: CellRef {
                                        worksheet: WorksheetId::from_uuid(r.id()?),
                                        cell: CellId::from_uuid(r.id()?),
                                    },
                                    node: GraphNodeId::from_uuid(r.id()?),
                                });
                            }
                            Content::TextFormula(TextFormula {
                                original_source,
                                tokens,
                                diagnostic,
                                transaction_base,
                                edit_ordinal,
                                resolved_graph,
                                references,
                            })
                        }
                        _ => return Err(malformed()),
                    };
                    live_ids.insert(node.uuid());
                    Record {
                        position: Some(p),
                        content: Some(content),
                    }
                }
                _ => return Err(malformed()),
            };
            if records.insert(id, record).is_some() {
                return Err(malformed());
            }
            ids.push((id, node));
        }
        if r.at != r.bytes.len()
            || positions.len() > limits.live_cells
            || live_ids != document.graph.nodes.iter().map(|n| n.id).collect()
        {
            return Err(malformed());
        }
        let portable: BTreeMap<_, _> = ids.iter().map(|(c, n)| (*c, *n)).collect();
        for node in &document.graph.nodes {
            let ordinal = node
                .name
                .strip_prefix('N')
                .and_then(|n| n.parse::<usize>().ok())
                .ok_or_else(malformed)?;
            if ordinal >= count || node.name != format!("N{ordinal}") {
                return Err(malformed());
            }
        }
        // Cross-check projection and computational definitions without historical A1 rebinding.
        for (id, node) in &ids {
            let Some(content) = records[id].content.as_ref() else {
                continue;
            };
            let definition = &document
                .graph
                .nodes
                .iter()
                .find(|n| n.id == node.uuid())
                .ok_or_else(malformed)?
                .definition;
            let (source, tokens, invalid): (&str, &[Token], bool) = match content {
                Content::StructuredFormula(_) => return Err(malformed()),
                Content::LiteralText(text) => {
                    pivot_workbook::entry::classify(Position::new(1, 1), &format!("'{text}"), 4097)
                        .map_err(|_| malformed())?;
                    if text.is_empty()
                        || text.len() > 4096
                        || *definition != storage::Definition::NonNumeric
                    {
                        return Err(malformed());
                    }
                    continue;
                }
                Content::Blank => {
                    if *definition
                        != storage::Definition::Input(storage::Decimal96::from_runtime(
                            PivotDecimal::ZERO,
                        ))
                    {
                        return Err(malformed());
                    }
                    continue;
                }
                Content::Decimal(d) => {
                    let expected = storage::Decimal96::from_runtime(d.value());
                    if *definition != storage::Definition::Input(expected) {
                        return Err(malformed());
                    }
                    continue;
                }
                Content::Formula { source, tokens } => (source, tokens, false),
                Content::TextFormula(f) => {
                    if f.diagnostic.is_some() && (!f.tokens.is_empty() || !f.references.is_empty())
                    {
                        return Err(malformed());
                    }
                    for reference in &f.references {
                        if reference.target.worksheet != worksheet
                            || portable.get(&reference.target.cell) != Some(&reference.node)
                        {
                            return Err(malformed());
                        }
                    }
                    // Validate source/provenance shape, without resolving coordinates against today's grid.
                    if f.diagnostic.is_none() {
                        let parsed = expression::parse_spreadsheet(&f.original_source)
                            .map_err(graph_error)?;
                        let (syntax, spans, diagnostic) =
                            Workbook::lower_text_bounds(rows.length, columns.length, &Ok(parsed));
                        if diagnostic.is_some()
                            || spans.len() != f.references.len()
                            || syntax.len() != f.tokens.len()
                        {
                            return Err(malformed());
                        }
                        for ((span, position), reference) in spans.iter().zip(&f.references) {
                            if *span != reference.span || *position != reference.submitted_position
                            {
                                return Err(malformed());
                            }
                        }
                        let mut refs = f.references.iter();
                        for (syntax, bound) in syntax.iter().zip(&f.tokens) {
                            match syntax {
                                Token::SumAt { first, last, span } => {
                                    if let Token::Sum {
                                        first: a,
                                        last: b,
                                        span: s,
                                        ..
                                    } = bound
                                    {
                                        if first != a || last != b || span != s {
                                            return Err(malformed());
                                        }
                                    } else {
                                        return Err(malformed());
                                    }
                                }
                                Token::At(_) => {
                                    if Some(bound)
                                        != refs.next().map(|r| Token::Cell(r.target)).as_ref()
                                    {
                                        return Err(malformed());
                                    }
                                }
                                _ => {
                                    if syntax != bound {
                                        return Err(malformed());
                                    }
                                }
                            }
                        }
                    }
                    (&f.original_source, &f.tokens, f.diagnostic.is_some())
                }
            };
            if invalid {
                if let storage::Definition::ParseFailed { source: s, .. } = definition {
                    if s != source {
                        return Err(malformed());
                    }
                } else {
                    return Err(malformed());
                }
            } else {
                let mut selection_tokens = Vec::with_capacity(tokens.len());
                let node_slot = document
                    .graph
                    .node_ids
                    .iter()
                    .position(|n| *n == node.uuid())
                    .ok_or_else(malformed)?;
                for t in tokens {
                    selection_tokens.push(match t {
                        Token::Sum {
                            id,
                            rectangle,
                            first,
                            last,
                            span,
                        } => {
                            if selection_map.get(id) != Some(&Some(*rectangle))
                                || rectangle.first_row != first.row.min(last.row)
                                || rectangle.last_row != first.row.max(last.row)
                                || rectangle.first_column != first.column.min(last.column)
                                || rectangle.last_column != first.column.max(last.column)
                                || span.start >= span.end
                                || source.get(span.clone()).is_none()
                            {
                                return Err(malformed());
                            }
                            SelectionToken::Sum(*id)
                        }
                        Token::Decimal(d) => SelectionToken::Decimal(d.value()),
                        Token::Cell(c) => {
                            if c.worksheet != worksheet {
                                return Err(malformed());
                            }
                            let id = portable.get(&c.cell).ok_or_else(malformed)?.uuid();
                            SelectionToken::Node(
                                document
                                    .graph
                                    .node_ids
                                    .iter()
                                    .position(|n| *n == id)
                                    .ok_or_else(malformed)?,
                            )
                        }
                        Token::Unary(op) => SelectionToken::Unary(*op),
                        Token::Binary(op) => SelectionToken::Binary(*op),
                        Token::Output(_) | Token::At(_) | Token::SumAt { .. } => {
                            return Err(malformed());
                        }
                    });
                }
                let has_sum = selection_tokens
                    .iter()
                    .any(|t| matches!(t, SelectionToken::Sum(_)));
                if let Some(index) = overlay.get(&node_slot) {
                    if !has_sum
                        || formulas[*index].source != *source
                        || formulas[*index].tokens != selection_tokens
                    {
                        return Err(malformed());
                    }
                    continue;
                } else if has_sum {
                    return Err(malformed());
                }
                let bound = tokens
                    .iter()
                    .map(|t| {
                        Ok(match t {
                            Token::Decimal(d) => {
                                storage::Token::Decimal(storage::Decimal96::from_runtime(d.value()))
                            }
                            Token::Cell(c) => {
                                if c.worksheet != worksheet {
                                    return Err(malformed());
                                }
                                storage::Token::Node {
                                    model: document.graph.model,
                                    node: portable.get(&c.cell).ok_or_else(malformed)?.uuid(),
                                }
                            }
                            Token::Unary(op) => {
                                storage::Token::Unary(u8::from(*op == UnaryOp::Minus))
                            }
                            Token::Binary(op) => storage::Token::Binary(match op {
                                BinaryOp::Add => 0,
                                BinaryOp::Subtract => 1,
                                BinaryOp::Multiply => 2,
                                BinaryOp::Divide => 3,
                            }),
                            Token::Output(_)
                            | Token::At(_)
                            | Token::SumAt { .. }
                            | Token::Sum { .. } => {
                                return Err(malformed());
                            }
                        })
                    })
                    .collect::<Result<Vec<_>>>()?;
                if *definition
                    != (storage::Definition::Bound {
                        source: source.to_string(),
                        tokens: bound,
                    })
                {
                    return Err(malformed());
                }
            }
        }
        for f in &formulas {
            let (id, _) = ids.get(f.slot).ok_or_else(malformed)?;
            let content = records
                .get(id)
                .and_then(|r| r.content.as_ref())
                .ok_or_else(malformed)?;
            let tokens: &[Token] = match content {
                Content::TextFormula(f) if f.diagnostic.is_none() => &f.tokens,
                Content::Formula { tokens, .. } => tokens,
                _ => return Err(malformed()),
            };
            if !tokens.iter().any(|t| matches!(t, Token::Sum { .. })) {
                return Err(malformed());
            }
        }
        timings.decode_ns = started.elapsed().as_nanos() as u64;
        let restoring = std::time::Instant::now();
        let scalar_skeleton =
            storage::to_image(&document.graph, self::limits()).map_err(graph_error)?;
        let points = ids
            .iter()
            .enumerate()
            .filter_map(|(slot, (id, _))| {
                let record = &records[id];
                if matches!(record.content, Some(Content::Blank) | None) {
                    return None;
                }
                let p = record.position?;
                Some((SpaceId(*worksheet.uuid().as_bytes()), p.row, p.column, slot))
            })
            .collect();
        let graph = SelectionImage {
            version: 2,
            scalar_skeleton,
            formulas,
            identities,
            points,
        };
        let cells = ids
            .into_iter()
            .map(|(id, node)| {
                let record = records.remove(&id).unwrap();
                (id, node, record)
            })
            .collect();
        let book = Workbook::restore_portable(
            PortableWorkbook {
                worksheet,
                graph_id: document.graph.model,
                rows: rows.namespace,
                columns: columns.namespace,
                row_count: rows.length,
                column_count: columns.length,
                state,
                cells,
                graph,
            },
            limits,
            runtime,
        )?;
        timings.restore_ns = restoring.elapsed().as_nanos() as u64;
        Ok((book, document.state_id))
    }
}
