//! Portable bounded clipboard images. No graph identities cross this boundary.
use super::*;
use expression::ReferenceSyntax;
use pivot_core::model::integration::CancellationToken;

pub const PRESERVED_REJECTION: &str = "copied rejected draft; explicitly resubmit to execute";
pub const MAX_BYTES: usize = 256 * 1024;
const MAX_SOURCE: usize = 128 * 1024;
const MAX_TOKENS: usize = 16_384;
const MAX_REFERENCES: usize = 8192;
const MAGIC: &[u8] = b"PIVOTCB2";

#[derive(Debug)]
pub struct ClipboardBlock {
    pub native: Vec<u8>,
    pub text: String,
}
#[derive(Clone)]
enum Slot {
    Text(String),
    Blank,
    Number(DecimalInput),
    Formula {
        source: String,
        effective: Vec<Position>,
        rejected: bool,
    },
}
struct Image {
    first: Position,
    height: u32,
    width: u32,
    slots: Vec<Slot>,
}
fn invalid() -> Error {
    Error::Graph("invalid or unsupported clipboard block".into())
}
fn area(first: Position, last: Position) -> Result<(u32, u32)> {
    if first.row == 0 || first.column == 0 || last.row < first.row || last.column < first.column {
        return Err(Error::Position);
    }
    let height = last
        .row
        .checked_sub(first.row)
        .and_then(|v| v.checked_add(1))
        .ok_or(Error::Limit)?;
    let width = last
        .column
        .checked_sub(first.column)
        .and_then(|v| v.checked_add(1))
        .ok_or(Error::Limit)?;
    if u64::from(height) * u64::from(width) > 256 {
        return Err(Error::Limit);
    }
    Ok((height, width))
}
fn references(source: &str) -> Result<(Vec<ReferenceSyntax>, usize)> {
    references_mode(source, false)
}
fn references_mode(source: &str, proof: bool) -> Result<(Vec<ReferenceSyntax>, usize)> {
    let (expr, work) = if proof {
        (expression::parse_structured_proof(source), source.len())
    } else {
        expression::parse_spreadsheet_counted(source)
    };
    let expr = expr.map_err(|_| invalid())?;
    let mut stack = vec![&expr];
    let mut refs = Vec::new();
    while let Some(expr) = stack.pop() {
        match expr {
            Expr::CellReference {
                row,
                column,
                absolute_row,
                absolute_column,
                span,
            } => refs.push(ReferenceSyntax {
                row: *row,
                column: *column,
                absolute_row: *absolute_row,
                absolute_column: *absolute_column,
                span: span.clone(),
            }),
            Expr::SumRange {
                first_reference,
                last_reference,
                ..
            } => {
                refs.push(first_reference.clone());
                refs.push(last_reference.clone());
            }
            Expr::Unary { operand, .. } => stack.push(operand),
            Expr::Binary { left, right, .. } => {
                stack.push(right);
                stack.push(left);
            }
            Expr::Decimal(_) => (),
            Expr::CapturedOutput { .. } if proof => (),
            Expr::CapturedOutput { .. } | Expr::Name(_) => return Err(invalid()),
        }
    }
    refs.sort_by_key(|r| r.span.start);
    Ok((refs, work))
}
fn address(p: Position, r: &ReferenceSyntax) -> String {
    let mut column = p.column;
    let mut letters = Vec::new();
    while column > 0 {
        column -= 1;
        letters.push(b'A' + (column % 26) as u8);
        column /= 26;
    }
    letters.reverse();
    format!(
        "{}{}{}{}",
        if r.absolute_column { "$" } else { "" },
        String::from_utf8(letters).unwrap(),
        if r.absolute_row { "$" } else { "" },
        p.row
    )
}
fn transform(
    source: &str,
    effective: &[Position],
    dr: i64,
    dc: i64,
    rows: u32,
    columns: u32,
) -> Result<(String, usize, usize)> {
    transform_mode(source, effective, dr, dc, rows, columns, false)
}
pub(crate) fn transform_mode(
    source: &str,
    effective: &[Position],
    dr: i64,
    dc: i64,
    rows: u32,
    columns: u32,
    proof: bool,
) -> Result<(String, usize, usize)> {
    let (refs, tokens) = references_mode(source, proof)?;
    if refs.len() != effective.len() {
        return Err(invalid());
    }
    let mut out = String::new();
    let mut cursor = 0;
    for (r, p) in refs.iter().zip(effective) {
        if p.row == 0 || p.row > 1024 || p.column == 0 || p.column > 1024 {
            return Err(invalid());
        }
        let row = i64::from(p.row) + if r.absolute_row { 0 } else { dr };
        let column = i64::from(p.column) + if r.absolute_column { 0 } else { dc };
        if row < 1 || row > i64::from(rows) || column < 1 || column > i64::from(columns) {
            return Err(Error::Position);
        }
        let next = Position::new(row as u32, column as u32);
        out.push_str(&source[cursor..r.span.start]);
        if next == Position::new(r.row, r.column) {
            out.push_str(&source[r.span.clone()]);
        } else {
            out.push_str(&address(next, r));
        }
        cursor = r.span.end;
        if out.len() > 4096 {
            return Err(Error::Limit);
        }
    }
    out.push_str(&source[cursor..]);
    if out.len() > 4096 {
        return Err(Error::Limit);
    }
    Ok((out, tokens, refs.len()))
}
fn n(out: &mut Vec<u8>, v: u32) {
    out.extend(v.to_le_bytes());
}
fn text(out: &mut Vec<u8>, s: &str) {
    n(out, s.len() as u32);
    out.extend(s.as_bytes());
}
struct Reader<'a> {
    bytes: &'a [u8],
    at: usize,
}
impl Reader<'_> {
    fn take(&mut self, len: usize) -> Result<&[u8]> {
        let end = self.at.checked_add(len).ok_or_else(invalid)?;
        let bytes = self.bytes.get(self.at..end).ok_or_else(invalid)?;
        self.at = end;
        Ok(bytes)
    }
    fn n(&mut self) -> Result<u32> {
        Ok(u32::from_le_bytes(self.take(4)?.try_into().unwrap()))
    }
    fn text(&mut self) -> Result<String> {
        let len = self.n()? as usize;
        if len > 4096 {
            return Err(Error::Limit);
        }
        String::from_utf8(self.take(len)?.to_vec()).map_err(|_| invalid())
    }
}
impl Image {
    fn encode(&self) -> Result<ClipboardBlock> {
        let mut native = MAGIC.to_vec();
        for v in [self.first.row, self.first.column, self.height, self.width] {
            n(&mut native, v);
        }
        let mut output = String::new();
        let mut source_work = 0;
        let mut token_work = 0;
        let mut reference_work = 0;
        for (i, slot) in self.slots.iter().enumerate() {
            if i > 0 {
                output.push(if i % self.width as usize == 0 {
                    '\n'
                } else {
                    '\t'
                });
            }
            match slot {
                Slot::Text(value) => {
                    crate::entry::validate_text(value, 4096)?;
                    source_work += value.len() + 1;
                    native.push(4);
                    text(&mut native, value);
                    output.push('\'');
                    output.push_str(value);
                }
                Slot::Blank => native.push(0),
                Slot::Number(value) => {
                    native.push(1);
                    let (lo, mid, hi, negative, scale) = value.parts();
                    for v in [lo, mid, hi, u32::from(negative), scale] {
                        n(&mut native, v);
                    }
                    let mut rendered = value.value().to_string();
                    if negative && lo == 0 && mid == 0 && hi == 0 && !rendered.starts_with('-') {
                        rendered.insert(0, '-');
                    }
                    output.push_str(&rendered);
                }
                Slot::Formula {
                    source,
                    effective,
                    rejected,
                } => {
                    native.push(if *rejected { 3 } else { 2 });
                    text(&mut native, source);
                    n(&mut native, effective.len() as u32);
                    for p in effective {
                        n(&mut native, p.row);
                        n(&mut native, p.column);
                    }
                    let rendered = if *rejected {
                        source.clone()
                    } else {
                        let (text, tokens, refs) = transform(source, effective, 0, 0, 1024, 1024)?;
                        token_work += tokens;
                        reference_work += refs;
                        text
                    };
                    source_work += rendered.len();
                    for ch in rendered.chars() {
                        output.push(if matches!(ch, '\r' | '\n' | '\t') {
                            ' '
                        } else {
                            ch
                        });
                    }
                }
            }
            if native.len() + output.len() > MAX_BYTES
                || source_work > MAX_SOURCE
                || token_work > MAX_TOKENS
                || reference_work > MAX_REFERENCES
            {
                return Err(Error::Limit);
            }
        }
        Ok(ClipboardBlock {
            native,
            text: output,
        })
    }
    fn decode(bytes: &[u8]) -> Result<Self> {
        if bytes.len() > MAX_BYTES {
            return Err(Error::Limit);
        }
        let mut r = Reader { bytes, at: 0 };
        if r.take(MAGIC.len())? != MAGIC {
            return Err(invalid());
        }
        let first = Position::new(r.n()?, r.n()?);
        let height = r.n()?;
        let width = r.n()?;
        if height == 0 || width == 0 {
            return Err(invalid());
        }
        let last = Position::new(
            first.row.checked_add(height - 1).ok_or_else(invalid)?,
            first.column.checked_add(width - 1).ok_or_else(invalid)?,
        );
        area(first, last)?;
        if last.row > 1024 || last.column > 1024 {
            return Err(invalid());
        }
        let mut slots = Vec::new();
        for _ in 0..height * width {
            let tag = r.take(1)?[0];
            slots.push(match tag {
                4 => {
                    let value = r.text()?;
                    crate::entry::validate_text(&value, 4096)?;
                    Slot::Text(value)
                }
                0 => Slot::Blank,
                1 => {
                    let lo = r.n()?;
                    let mid = r.n()?;
                    let hi = r.n()?;
                    let negative = r.n()?;
                    let scale = r.n()?;
                    if negative > 1 {
                        return Err(invalid());
                    }
                    Slot::Number(DecimalInput::from_parts(lo, mid, hi, negative == 1, scale)?)
                }
                2 | 3 => {
                    let source = r.text()?;
                    let count = r.n()? as usize;
                    if count > MAX_REFERENCES || count > r.bytes.len().saturating_sub(r.at) / 8 {
                        return Err(invalid());
                    }
                    let mut effective = Vec::new();
                    for _ in 0..count {
                        effective.push(Position::new(r.n()?, r.n()?));
                    }
                    if tag == 3 {
                        // A constrained, intentionally non-executable state, never an asserted parser diagnostic.
                        if count != 0 {
                            return Err(invalid());
                        }
                    } else {
                        transform(&source, &effective, 0, 0, 1024, 1024)?;
                    }
                    Slot::Formula {
                        source,
                        effective,
                        rejected: tag == 3,
                    }
                }
                _ => return Err(invalid()),
            });
        }
        if r.at != bytes.len() {
            return Err(invalid());
        }
        let image = Self {
            first,
            height,
            width,
            slots,
        };
        image.encode()?;
        Ok(image)
    }
    fn tsv(source: &str) -> Result<Self> {
        if source.len() > MAX_BYTES {
            return Err(Error::Limit);
        }
        let source = source
            .strip_suffix("\r\n")
            .or_else(|| source.strip_suffix('\n'))
            .unwrap_or(source);
        let rows = source.split('\n').collect::<Vec<_>>();
        if rows.len() > 256 {
            return Err(Error::Limit);
        }
        let mut slots = Vec::new();
        let mut width = None;
        for (index, row) in rows.iter().enumerate() {
            let row = if index + 1 < rows.len() {
                row.strip_suffix('\r').unwrap_or(row)
            } else {
                row
            };
            if row.contains('\r') {
                return Err(invalid());
            }
            let fields = row.split('\t').collect::<Vec<_>>();
            if width.is_some_and(|w| w != fields.len()) {
                return Err(invalid());
            }
            width = Some(fields.len());
            if fields.len() > 256 || slots.len() + fields.len() > 256 {
                return Err(Error::Limit);
            }
            for field in fields {
                slots.push(
                    match crate::entry::classify(Position::new(1, 1), field, 4096)? {
                        Edit::Clear(_) => Slot::Blank,
                        Edit::SetDecimal { value, .. } => Slot::Number(value),
                        Edit::SetText { text, .. } => Slot::Text(text),
                        Edit::SetFormulaText { source, .. } => Slot::Formula {
                            source,
                            effective: Vec::new(),
                            rejected: false,
                        },
                        _ => unreachable!(),
                    },
                );
            }
        }
        Ok(Self {
            first: Position::new(1, 1),
            height: rows.len() as u32,
            width: width.unwrap() as u32,
            slots,
        })
    }
}
impl Workbook {
    /// Copy from exactly one immutable generation; returned bytes own no source roots.
    pub fn copy_block(&self, expected: Expected, rectangle: Rectangle) -> Result<ClipboardBlock> {
        self.check(expected)?;
        self.axes_at(rectangle.first)?;
        self.axes_at(rectangle.last)?;
        let (height, width) = area(rectangle.first, rectangle.last)?;
        let _scratch = self
            .core
            .stage()
            .allocation_domain()
            .map_err(graph_error)?
            .reserve(8 * 1024 * 1024)
            .map_err(|_| Error::Limit)?;
        let capture = self.capture_local(expected)?;
        let mut slots = Vec::new();
        for row in rectangle.first.row..=rectangle.last.row {
            for column in rectangle.first.column..=rectangle.last.column {
                let cell = capture.pinned_content(Position::new(row, column))?;
                let content = match cell.map(|c| c.content) {
                    Some(Content::StructuredFormula(_))=>return Err(Error::Graph("Structured formulas cannot use OS Copy/Cut; use same-document Duplicate instead.".into())),
                    Some(Content::Formula { source, tokens }) => {
                        if tokens.iter().any(|t| matches!(t, Token::Output(_))) {
                            return Err(Error::Graph("Structured formulas cannot use OS Copy/Cut; use same-document Duplicate instead.".into()));
                        }
                        let (syntax, spans, diagnostic) = Workbook::lower_text_bounds(
                            self.rows.length,
                            self.columns.length,
                            &expression::parse_spreadsheet(&source),
                        );
                        if diagnostic.is_some() || syntax.len() != tokens.len() {
                            return Err(invalid());
                        }
                        let mut submitted = spans.into_iter();
                        let mut references = Vec::new();
                        for (syntax, bound) in syntax.iter().zip(tokens.iter()) {
                            match (syntax, bound) {
                                (Token::At(_), Token::Cell(target)) => {
                                    if target.worksheet != self.worksheet {
                                        return Err(invalid());
                                    }
                                    let (span, submitted_position) =
                                        submitted.next().ok_or_else(invalid)?;
                                    let node = capture
                                        .pinned
                                        .bindings
                                        .get(&target.cell)
                                        .ok_or_else(invalid)?
                                        .portable;
                                    references.push(FormulaReference {
                                        span,
                                        submitted_position,
                                        target: *target,
                                        node,
                                    });
                                }
                                (
                                    Token::SumAt { kind, first, last, span },
                                    Token::Sum {
                                        kind:k,
                                        first: a,
                                        last: b,
                                        span: s,
                                        ..
                                    },
                                ) if kind==k && first == a && last == b && span == s => (),
                                (a, b) if a == b => (),
                                _ => return Err(invalid()),
                            }
                        }
                        Some(Content::TextFormula(TextFormula {
                            original_source: source,
                            tokens,
                            references,
                            diagnostic: None,
                            transaction_base: expected,
                            edit_ordinal: 0,
                            resolved_graph: expected.graph,
                        }))
                    }
                    content => content,
                };
                slots.push(match content {
                    Some(Content::LiteralText(value)) => Slot::Text(value),
                    None | Some(Content::Blank) => Slot::Blank,
                    Some(Content::Decimal(value)) => Slot::Number(value),
                    Some(Content::Formula { .. } | Content::StructuredFormula(_)) => return Err(invalid()),
                    Some(Content::TextFormula(f)) => {
                        let mut annotations = Vec::new();
                        if f.diagnostic.is_none() {
                            for reference in &f.references {
                                let p = capture
                                    .pinned
                                    .records
                                    .get(&reference.target.cell)
                                    .and_then(|r| r.position)
                                    .ok_or_else(||Error::Graph(format!("Cannot copy R{row}C{column}: a bound target is retired or unplaced.")))?;
                                annotations.push((reference.span.start, p));
                            }
                            let (refs, _) = references(&f.original_source)?;
                            let mut ranges = f.tokens.iter().filter_map(|t| {
                                if let Token::Sum { first, last, .. } = t {
                                    Some((*first, *last))
                                } else {
                                    None
                                }
                            });
                            let parsed = expression::parse_spreadsheet(&f.original_source)
                                .map_err(|_| invalid())?;
                            let mut stack = vec![&parsed];
                            while let Some(expr) = stack.pop() {
                                match expr {
                                    Expr::SumRange {
                                        first_reference,
                                        last_reference,
                                        ..
                                    } => {
                                        let (first, last) = ranges.next().ok_or_else(invalid)?;
                                        annotations.push((first_reference.span.start, first));
                                        annotations.push((last_reference.span.start, last));
                                    }
                                    Expr::Unary { operand, .. } => stack.push(operand),
                                    Expr::Binary { left, right, .. } => {
                                        stack.push(right);
                                        stack.push(left);
                                    }
                                    _ => (),
                                }
                            }
                            annotations.sort_by_key(|(span, _)| *span);
                            if annotations.len() != refs.len() {
                                return Err(invalid());
                            }
                        }
                        Slot::Formula {
                            source: f.original_source,
                            effective: annotations.into_iter().map(|(_, p)| p).collect(),
                            rejected: f.diagnostic.is_some(),
                        }
                    }
                });
            }
        }
        Image {
            first: rectangle.first,
            height,
            width,
            slots,
        }
        .encode()
    }
    /// Decode, plan, and install all slots in one existing atomic workbook transaction.
    pub fn paste_block(
        &mut self,
        expected: Expected,
        rectangle: Rectangle,
        bytes: &[u8],
        native: bool,
        cancel: &CancellationToken,
    ) -> Result<Rectangle> {
        self.check(expected)?;
        self.axes_at(rectangle.first)?;
        self.axes_at(rectangle.last)?;
        let _scratch = self
            .core
            .stage()
            .allocation_domain()
            .map_err(graph_error)?
            .reserve(8 * 1024 * 1024)
            .map_err(|_| Error::Limit)?;
        if cancel.is_cancelled() {
            return Err(Error::Graph("cancelled".into()));
        }
        let image = if native {
            Image::decode(bytes)?
        } else {
            Image::tsv(std::str::from_utf8(bytes).map_err(|_| invalid())?)?
        };
        if self.is_structured_proof()
            && image
                .slots
                .iter()
                .any(|slot| matches!(slot,Slot::Formula{source,..} if source.contains("@{")))
        {
            return Err(Error::Graph("Structured formula paste requires explicit typed authoring; the rectangle was not changed.".into()));
        }
        let (height, width) = area(rectangle.first, rectangle.last)?;
        if (height, width) != (1, 1) && (height, width) != (image.height, image.width) {
            return Err(invalid());
        }
        let last = Position::new(
            rectangle
                .first
                .row
                .checked_add(image.height - 1)
                .ok_or(Error::Position)?,
            rectangle
                .first
                .column
                .checked_add(image.width - 1)
                .ok_or(Error::Position)?,
        );
        self.axes_at(last)?;
        let dr = i64::from(rectangle.first.row) - i64::from(image.first.row);
        let dc = i64::from(rectangle.first.column) - i64::from(image.first.column);
        let mut edits = Vec::new();
        let mut source_work = 0;
        let mut token_work = 0;
        let mut reference_work = 0;
        for (i, slot) in image.slots.into_iter().enumerate() {
            let position = Position::new(
                rectangle.first.row + i as u32 / image.width,
                rectangle.first.column + i as u32 % image.width,
            );
            edits.push(match slot {
                Slot::Text(text) => {
                    source_work += text.len() + 1;
                    if source_work > MAX_SOURCE {
                        return Err(Error::Limit);
                    }
                    Edit::SetText { position, text }
                }
                Slot::Blank => Edit::Clear(position),
                Slot::Number(value) => Edit::SetDecimal { position, value },
                Slot::Formula {
                    source,
                    effective,
                    rejected,
                } => {
                    let source = if native && !rejected {
                        let (source, tokens, refs) = transform(
                            &source,
                            &effective,
                            dr,
                            dc,
                            self.rows.length,
                            self.columns.length,
                        )?;
                        token_work += tokens;
                        reference_work += refs;
                        source
                    } else {
                        if !native {
                            let (_, work) = expression::parse_spreadsheet_counted(&source);
                            token_work += work;
                            if let Ok((refs, _)) = references(&source) {
                                reference_work += refs.len();
                            }
                        }
                        source
                    };
                    source_work += source.len();
                    if source_work > MAX_SOURCE
                        || token_work > MAX_TOKENS
                        || reference_work > MAX_REFERENCES
                    {
                        return Err(Error::Limit);
                    }
                    if native && rejected {
                        Edit::PreserveRejectedFormula { position, source }
                    } else {
                        Edit::SetFormulaText { position, source }
                    }
                }
            });
        }
        self.apply_cancelled(expected, &edits, cancel)?;
        Ok(Rectangle {
            first: rectangle.first,
            last,
        })
    }
}
