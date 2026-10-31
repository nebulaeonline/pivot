//! Portable bounded clipboard images. No graph identities cross this boundary.
use super::*;
use expression::ReferenceSyntax;
use pivot_core::model::integration::CancellationToken;

pub const PRESERVED_REJECTION: &str = "copied rejected draft; explicitly resubmit to execute";
pub const MAX_BYTES: usize = 256 * 1024;
const MAX_SOURCE: usize = 128 * 1024;
const MAX_TOKENS: usize = 16_384;
const MAX_REFERENCES: usize = 8192;
const MAGIC: &[u8] = b"PIVOTCB5";
const VISIBLE_MAGIC: &[u8] = b"PIVOTCB6";

#[derive(Debug)]
pub struct ClipboardBlock {
    pub native: Vec<u8>,
    pub text: String,
    pub text_available: bool,
}
/// Ordered direct-write footprint; visibility qualification belongs to the session.
pub struct VisibleDestination<'a> {
    pub rectangle: Rectangle,
    pub rows: &'a [u32],
}
#[derive(Clone)]
enum Slot {
    Boolean(bool),
    Text(String),
    Blank,
    Number(DecimalInput),
    Formula {
        source: String,
        effective: Vec<Position>,
        rejected: bool,
        faults: Vec<String>,
    },
}
struct Image {
    source_rows: Option<Vec<u32>>,
    first: Position,
    height: u32,
    width: u32,
    slots: Vec<Slot>,
    styles: Vec<formatting::Style>,
}

fn valid_rows(rows: &[u32]) -> Result<()> {
    if rows.is_empty()
        || rows.len() > 256
        || rows[0] == 0
        || *rows.last().unwrap() > crate::MAX_ROWS
        || rows.windows(2).any(|r| r[0] >= r[1])
    {
        return Err(invalid());
    }
    Ok(())
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
pub(crate) fn references(source: &str) -> Result<(Vec<ReferenceSyntax>, usize)> {
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
                qualifier,
            } => refs.push(ReferenceSyntax {
                qualifier: qualifier.clone(),
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
            Expr::AggregateCall { arguments, .. } => {
                stack.extend(arguments.iter().rev().map(|a| a.expression()))
            }
            Expr::Unary { operand, .. } => stack.push(operand),
            Expr::Binary { left, right, .. } => {
                stack.push(right);
                stack.push(left);
            }
            Expr::Decimal(_) => (),
            Expr::Typed { arguments, .. } => stack.extend(arguments.iter()),
            Expr::CapturedOutput { .. } if proof => (),
            Expr::CapturedOutput { .. } | Expr::Name(_) => return Err(invalid()),
        }
    }
    refs.sort_by_key(|r| r.span.start);
    Ok((refs, work))
}
pub(crate) fn address(p: Position, r: &ReferenceSyntax) -> String {
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
        if p.row == 0 || p.row > crate::MAX_ROWS || p.column == 0 || p.column > crate::MAX_COLUMNS {
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
        let text_available = self.source_rows.is_none()
            || !self.slots.iter().any(|slot| match slot {
                Slot::Text(text) | Slot::Formula { source: text, .. } => {
                    text.contains(['\r', '\n', '\t'])
                }
                _ => false,
            });
        let mut native = if self.source_rows.is_some() {
            VISIBLE_MAGIC
        } else {
            MAGIC
        }
        .to_vec();
        for v in [self.first.row, self.first.column, self.height, self.width] {
            n(&mut native, v);
        }
        if let Some(rows) = &self.source_rows {
            valid_rows(rows)?;
            if rows.len() != self.height as usize || rows[0] != self.first.row {
                return Err(invalid());
            }
            for row in rows {
                n(&mut native, *row);
            }
        }
        let mut output = String::new();
        let mut source_work = 0;
        let mut token_work = 0;
        let mut reference_work = 0;
        for (i, slot) in self.slots.iter().enumerate() {
            native.extend(self.styles[i].encode());
            if i > 0 {
                output.push(if i % self.width as usize == 0 {
                    '\n'
                } else {
                    '\t'
                });
            }
            match slot {
                Slot::Boolean(value) => {
                    native.push(5);
                    native.push(u8::from(*value));
                    output.push_str(if *value { "TRUE" } else { "FALSE" });
                }
                Slot::Text(value) => {
                    crate::entry::validate_value_text(value, 4096)?;
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
                    faults,
                } => {
                    native.push(if *rejected { 3 } else { 2 });
                    text(&mut native, source);
                    n(&mut native, effective.len() as u32);
                    for p in effective {
                        n(&mut native, p.row);
                        n(&mut native, p.column);
                    }
                    n(&mut native, faults.len() as u32);
                    for origin in faults {
                        crate::structure::FaultOrigin::decode(origin)?;
                        text(&mut native, origin);
                        source_work += origin.len();
                    }
                    let rendered = if *rejected {
                        source.clone()
                    } else {
                        let (text, tokens, refs) = transform(
                            source,
                            effective,
                            0,
                            0,
                            crate::MAX_ROWS,
                            crate::MAX_COLUMNS,
                        )?;
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
            if !text_available {
                output.clear();
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
            text_available,
        })
    }
    fn decode(bytes: &[u8]) -> Result<Self> {
        if bytes.len() > MAX_BYTES {
            return Err(Error::Limit);
        }
        let mut r = Reader { bytes, at: 0 };
        let magic = r.take(MAGIC.len())?;
        if magic != MAGIC && magic != VISIBLE_MAGIC {
            return Err(invalid());
        }
        let visible = magic == VISIBLE_MAGIC;
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
        if last.row > crate::MAX_ROWS || last.column > crate::MAX_COLUMNS {
            return Err(invalid());
        }
        let source_rows = if visible {
            let rows = (0..height).map(|_| r.n()).collect::<Result<Vec<_>>>()?;
            valid_rows(&rows)?;
            if rows[0] != first.row {
                return Err(invalid());
            }
            Some(rows)
        } else {
            None
        };
        let mut slots = Vec::new();
        let mut styles = Vec::new();
        for _ in 0..height * width {
            styles.push(formatting::Style::decode(r.take(17)?)?);
            let tag = r.take(1)?[0];
            slots.push(match tag {
                4 => {
                    let value = r.text()?;
                    crate::entry::validate_value_text(&value, 4096)?;
                    Slot::Text(value)
                }
                5 => {
                    let b = r.take(1)?[0];
                    if b > 1 {
                        return Err(invalid());
                    }
                    Slot::Boolean(b == 1)
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
                    let fault_count = r.n()? as usize;
                    if fault_count > MAX_TOKENS {
                        return Err(invalid());
                    }
                    let mut faults = Vec::new();
                    for _ in 0..fault_count {
                        let origin = r.text()?;
                        crate::structure::FaultOrigin::decode(&origin)?;
                        faults.push(origin);
                    }
                    if (tag == 3 && !faults.is_empty())
                        || (tag == 2
                            && faults.len() != crate::structure::literal_fault_spans(&source).len())
                    {
                        return Err(invalid());
                    }
                    if tag == 3 {
                        // A constrained, intentionally non-executable state, never an asserted parser diagnostic.
                        if count != 0 {
                            return Err(invalid());
                        }
                    } else {
                        transform(
                            &source,
                            &effective,
                            0,
                            0,
                            crate::MAX_ROWS,
                            crate::MAX_COLUMNS,
                        )?;
                    }
                    Slot::Formula {
                        source,
                        effective,
                        rejected: tag == 3,
                        faults,
                    }
                }
                _ => return Err(invalid()),
            });
        }
        if r.at != bytes.len() {
            return Err(invalid());
        }
        let image = Self {
            source_rows,
            first,
            height,
            width,
            slots,
            styles,
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
                        Edit::SetBoolean { value, .. } => Slot::Boolean(value),
                        Edit::Clear(_) => Slot::Blank,
                        Edit::SetDecimal { value, .. } => Slot::Number(value),
                        Edit::SetText { text, .. } => Slot::Text(text),
                        Edit::SetFormulaText { source, .. } => Slot::Formula {
                            source,
                            effective: Vec::new(),
                            rejected: false,
                            faults: Vec::new(),
                        },
                        _ => unreachable!(),
                    },
                );
            }
        }
        Ok(Self {
            source_rows: None,
            first: Position::new(1, 1),
            height: rows.len() as u32,
            width: width.unwrap() as u32,
            styles: vec![formatting::Style::default(); slots.len()],
            slots,
        })
    }
}
impl Workbook {
    pub fn clipboard_shape(
        &self,
        expected: Expected,
        bytes: &[u8],
        native: bool,
    ) -> Result<(u32, u32)> {
        self.check(expected)?;
        let _scratch = self
            .persistent
            .memory
            .reserve(8 * 1024 * 1024)
            .map_err(|_| Error::Limit)?;
        let image = if native {
            Image::decode(bytes)?
        } else {
            Image::tsv(std::str::from_utf8(bytes).map_err(|_| invalid())?)?
        };
        Ok((image.height, image.width))
    }
    pub fn copy_visible_sheet(
        &self,
        sheet: WorksheetId,
        expected: Expected,
        rows: &[u32],
        first_column: u32,
        last_column: u32,
        cancel: &CancellationToken,
    ) -> Result<ClipboardBlock> {
        self.request_view(sheet)?
            .copy_rows(expected, rows, first_column, last_column, true, cancel)
    }
    pub fn paste_visible_sheet(
        &mut self,
        sheet: WorksheetId,
        expected: Expected,
        destination: VisibleDestination<'_>,
        bytes: &[u8],
        native: bool,
        cancel: &CancellationToken,
    ) -> Result<Rectangle> {
        self.mutate_sheet(sheet, |view| {
            view.paste_planned(
                expected,
                destination.rectangle,
                bytes,
                native,
                cancel,
                Some(destination.rows),
            )
        })
    }
    /// Exact addressed destination from the immutable payload, for operation review.
    pub fn paste_extent(
        &self,
        sheet: WorksheetId,
        expected: Expected,
        rectangle: Rectangle,
        bytes: &[u8],
        native: bool,
    ) -> Result<Rectangle> {
        self.check(expected)?;
        self.axes_at_sheet(sheet, rectangle.first)?;
        self.axes_at_sheet(sheet, rectangle.last)?;
        let _lease = self
            .persistent
            .memory
            .reserve(8 * 1024 * 1024)
            .map_err(|_| Error::Limit)?;
        let image = if native {
            Image::decode(bytes)?
        } else {
            Image::tsv(std::str::from_utf8(bytes).map_err(|_| invalid())?)?
        };
        let shape = area(rectangle.first, rectangle.last)?;
        if shape != (1, 1) && shape != (image.height, image.width) {
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
        self.axes_at_sheet(sheet, last)?;
        Ok(Rectangle {
            first: rectangle.first,
            last,
        })
    }
    /// Copy from exactly one immutable generation; returned bytes own no source roots.
    pub fn copy_block(&self, expected: Expected, rectangle: Rectangle) -> Result<ClipboardBlock> {
        area(rectangle.first, rectangle.last)?;
        self.copy_rows(
            expected,
            &(rectangle.first.row..=rectangle.last.row).collect::<Vec<_>>(),
            rectangle.first.column,
            rectangle.last.column,
            false,
            &CancellationToken::default(),
        )
    }
    fn copy_rows(
        &self,
        expected: Expected,
        rows: &[u32],
        first_column: u32,
        last_column: u32,
        visible: bool,
        cancel: &CancellationToken,
    ) -> Result<ClipboardBlock> {
        self.check(expected)?;
        valid_rows(rows)?;
        let first = Position::new(rows[0], first_column);
        let last = Position::new(*rows.last().unwrap(), last_column);
        self.axes_at(first)?;
        self.axes_at(last)?;
        if last_column < first_column {
            return Err(Error::Position);
        }
        let height = rows.len() as u32;
        let width = last_column - first_column + 1;
        if u64::from(height) * u64::from(width) > 256 {
            return Err(Error::Limit);
        }
        let _scratch = self
            .core
            .stage()
            .allocation_domain()
            .map_err(graph_error)?
            .reserve(8 * 1024 * 1024)
            .map_err(|_| Error::Limit)?;
        let capture = self.capture_local(expected)?;
        let mut slots = Vec::new();
        for &row in rows {
            if cancel.is_cancelled() {
                return Err(Error::Graph("cancelled".into()));
            }
            for column in first_column..=last_column {
                let content = capture
                    .pinned_content(Position::new(row, column))?
                    .map(|c| c.content);
                let slot=match content {
                    None|Some(Content::Blank)=>Slot::Blank,
                    Some(Content::Boolean(v))=>Slot::Boolean(v),Some(Content::LiteralText(v))=>Slot::Text(v),Some(Content::Decimal(v))=>Slot::Number(v),
                    Some(Content::StructuredFormula(_))=>return Err(Error::Graph("Structured formulas cannot use OS Copy/Cut; use same-document Duplicate instead.".into())),
                    Some(content)=>{
                        let (source,tokens,rejected)=match &content {Content::Formula{source,tokens}=>(source.as_str(),tokens.as_slice(),false),Content::TextFormula(f)=>(f.original_source.as_str(),f.tokens.as_slice(),f.diagnostic.is_some()),_=>return Err(invalid())};
                        if tokens.iter().any(|t|matches!(t,Token::Output(_))){return Err(Error::Graph("Structured formulas cannot use OS Copy/Cut; use same-document Duplicate instead.".into()));}
                        if tokens.iter().any(|t|matches!(t,Token::Cell(target) if capture.pinned.records.get(&target.cell).is_none_or(|r|r.position.is_none()))){return Err(Error::Graph("Cannot copy a retired or unplaced bound target.".into()));}
                        if rejected {Slot::Formula{source:source.into(),effective:Vec::new(),rejected:true,faults:Vec::new()}}
                        else {
                            let (source,faults)=crate::structure::render(source,tokens,&capture.pinned,self.sheets(),self.limits.source_bytes)?;
                            let effective=references(&source)?.0.into_iter().map(|r|Position::new(r.row,r.column)).collect();
                            Slot::Formula{source,effective,rejected:false,faults}
                        }
                    }
                };
                slots.push(slot);
            }
        }
        Image {
            source_rows: visible.then(|| rows.to_vec()),
            first,
            height,
            width,
            styles: rows
                .iter()
                .copied()
                .flat_map(|r| (first_column..=last_column).map(move |c| Position::new(r, c)))
                .map(|p| self.style_at(self.worksheet, p))
                .collect::<Result<Vec<_>>>()?,
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
        self.paste_planned(expected, rectangle, bytes, native, cancel, None)
    }
    fn paste_planned(
        &mut self,
        expected: Expected,
        rectangle: Rectangle,
        bytes: &[u8],
        native: bool,
        cancel: &CancellationToken,
        destination_rows: Option<&[u32]>,
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
        let last = if let Some(rows) = destination_rows {
            valid_rows(rows)?;
            if rows.len() != image.height as usize
                || rows[0] != rectangle.first.row
                || *rows.last().unwrap() != rectangle.last.row
                || rectangle
                    .last
                    .column
                    .checked_sub(rectangle.first.column)
                    .and_then(|v| v.checked_add(1))
                    != Some(image.width)
            {
                return Err(invalid());
            }
            rectangle.last
        } else {
            let (height, width) = area(rectangle.first, rectangle.last)?;
            if (height, width) != (1, 1) && (height, width) != (image.height, image.width) {
                return Err(invalid());
            }
            Position::new(
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
            )
        };
        self.axes_at(last)?;
        let dc = i64::from(rectangle.first.column) - i64::from(image.first.column);
        let mut edits = Vec::new();
        let mut styles = Vec::new();
        let mut source_work = 0;
        let mut token_work = 0;
        let mut reference_work = 0;
        for (i, slot) in image.slots.into_iter().enumerate() {
            if cancel.is_cancelled() {
                return Err(Error::Graph("cancelled".into()));
            }
            let position = Position::new(
                destination_rows.map_or(rectangle.first.row + i as u32 / image.width, |rows| {
                    rows[i / image.width as usize]
                }),
                rectangle.first.column + i as u32 % image.width,
            );
            let source_row = image
                .source_rows
                .as_ref()
                .map_or(image.first.row + i as u32 / image.width, |rows| {
                    rows[i / image.width as usize]
                });
            let dr = i64::from(position.row) - i64::from(source_row);
            if native {
                styles.push((position, image.styles[i]));
            }
            edits.push(match slot {
                Slot::Text(text) => {
                    source_work += text.len() + 1;
                    if source_work > MAX_SOURCE {
                        return Err(Error::Limit);
                    }
                    Edit::SetText { position, text }
                }
                Slot::Boolean(value) => Edit::SetBoolean { position, value },
                Slot::Blank => Edit::Clear(position),
                Slot::Number(value) => Edit::SetDecimal { position, value },
                Slot::Formula {
                    source,
                    effective,
                    rejected,
                    faults,
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
                        if let Ok(parsed) = expression::parse_spreadsheet(&source) {
                            let (tokens, _, _) = Self::lower_text_bounds(
                                self.rows.length,
                                self.columns.length,
                                &Ok(parsed),
                            );
                            if let Some(name) = tokens.iter().find_map(|t| {
                                if let Token::QualifierAt { name, .. } = t {
                                    self.sheet_by_name(name).is_none().then_some(name)
                                } else {
                                    None
                                }
                            }) {
                                return Err(Error::Graph(format!(
                                    "unknown destination worksheet: {name}"
                                )));
                            }
                        }
                        if native && !faults.is_empty() {
                            let (mut tokens, _, diagnostic) = Self::lower_text_bounds(
                                self.rows.length,
                                self.columns.length,
                                &expression::parse_spreadsheet(&source),
                            );
                            if diagnostic.is_some() {
                                return Err(invalid());
                            }
                            crate::structure::attach_faults(&mut tokens, &faults, true)?;
                            Edit::SetFormula {
                                position,
                                source,
                                tokens,
                            }
                        } else {
                            Edit::SetFormulaText { position, source }
                        }
                    }
                }
            });
        }
        persistent::apply_styled(self, expected, &edits, &styles, cancel, false)?;
        Ok(Rectangle {
            first: rectangle.first,
            last,
        })
    }
}
