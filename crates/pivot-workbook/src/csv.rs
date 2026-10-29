//! Bounded CSV interchange. Quotation preserves Strings; no formula authoring.
use crate::{
    CalculationCapture, DecimalInput, Edit, Limits, Position, Rectangle, Workbook, WorksheetId,
};
use pivot_core::model::integration::IntegrationRuntime;
use pivot_types::{PivotDecimal, PivotValue};

/// Interchange admission policy, independent of the workbook's computational limits.
#[derive(Clone, Copy, Debug)]
pub struct Policy {
    pub bytes: usize,
    pub fields: usize,
}
impl Default for Policy {
    fn default() -> Self {
        Self {
            bytes: 4 * 1024 * 1024,
            fields: 65_536,
        }
    }
}
#[derive(Clone, Copy, Debug)]
pub struct Options {
    pub typed: bool,
    pub preserve_leading_zeros: bool,
    /// Explicit quoted-field policy, independent of leading-zero preservation.
    pub recognize_quoted: bool,
}
impl Default for Options {
    fn default() -> Self {
        Self {
            typed: false,
            preserve_leading_zeros: true,
            recognize_quoted: false,
        }
    }
}
#[derive(Debug)]
struct Field {
    text: String,
    quoted: bool,
}
#[derive(Debug)]
pub struct Table {
    fields: Vec<Field>,
    pub rows: u32,
    pub columns: u32,
    pub all_quoted: bool,
}
type Result<T> = std::result::Result<T, String>;
fn at(row: usize, column: usize, message: &str) -> String {
    format!("CSV row {row}, column {column}: {message}")
}

/// Deliberately strict dialect; retain quotation provenance for conversion.
pub fn parse(bytes: &[u8], policy: Policy, limits: Limits) -> Result<Table> {
    if bytes.len() > policy.bytes {
        return Err(format!("CSV file-size limit: {} bytes", policy.bytes));
    }
    let source = std::str::from_utf8(bytes)
        .map_err(|_| "CSV encoding: expected strict UTF-8".to_string())?;
    let source = source.strip_prefix('\u{feff}').unwrap_or(source);
    if source.is_empty() {
        return Err("CSV contains no records".into());
    }
    let b = source.as_bytes();
    let mut fields = Vec::new();
    let (mut i, mut row, mut column, mut width) = (0, 1usize, 1usize, None);
    let mut all_quoted = true;
    loop {
        if fields.len() >= policy.fields {
            return Err(format!("CSV logical-field limit: {}", policy.fields));
        }
        if row > limits.rows as usize || column > limits.columns as usize {
            return Err(at(row, column, "worksheet coordinate limit exceeded"));
        }
        let quoted = b.get(i) == Some(&b'"');
        all_quoted &= quoted;
        let mut text = Vec::new();
        if quoted {
            i += 1;
            loop {
                match b.get(i) {
                    None => return Err(at(row, column, "CSV syntax: unclosed quoted field")),
                    Some(b'"') if b.get(i + 1) == Some(&b'"') => {
                        text.push(b'"');
                        i += 2;
                    }
                    Some(b'"') => {
                        i += 1;
                        break;
                    }
                    Some(byte) => {
                        text.push(*byte);
                        i += 1;
                    }
                }
                if text.len() > limits.source_bytes {
                    return Err(at(
                        row,
                        column,
                        "workbook String/source byte limit exceeded",
                    ));
                }
            }
        } else {
            while let Some(byte) = b.get(i) {
                if matches!(byte, b',' | b'\r' | b'\n') {
                    break;
                }
                if *byte == b'"' {
                    return Err(at(row, column, "CSV syntax: quote in unquoted field"));
                }
                text.push(*byte);
                i += 1;
                if text.len() > limits.source_bytes {
                    return Err(at(
                        row,
                        column,
                        "workbook String/source byte limit exceeded",
                    ));
                }
            }
        }
        // UTF-8 boundaries are unchanged by ASCII quote unescaping.
        fields.push(Field {
            text: String::from_utf8(text).map_err(|_| "CSV encoding: invalid UTF-8".to_string())?,
            quoted,
        });
        match b.get(i) {
            Some(b',') => {
                i += 1;
                column += 1;
                continue;
            }
            Some(b'\r') if b.get(i + 1) == Some(&b'\n') => i += 2,
            Some(b'\n') => i += 1,
            Some(_) => {
                return Err(at(
                    row,
                    column,
                    "CSV syntax: expected comma, CRLF, LF or end of file",
                ));
            }
            None => (),
        }
        if width.is_some_and(|w| w != column) {
            return Err(at(row, column, "CSV shape: unequal column counts"));
        }
        width = Some(column);
        if i == b.len() {
            break;
        }
        row += 1;
        column = 1;
    }
    Ok(Table {
        fields,
        rows: row as u32,
        columns: width.unwrap() as u32,
        all_quoted,
    })
}

fn decimal_spelling(s: &str) -> bool {
    let s = s.strip_prefix(['+', '-']).unwrap_or(s);
    let mut p = s.split('.');
    let whole = p.next().unwrap_or_default();
    !whole.is_empty()
        && whole.bytes().all(|c| c.is_ascii_digit())
        && p.next().is_none_or(|fraction| {
            !fraction.is_empty() && fraction.bytes().all(|c| c.is_ascii_digit())
        })
        && p.next().is_none()
}
fn value(field: &Field, options: Options) -> Result<PivotValue> {
    let s = &field.text;
    if s.is_empty() {
        return Ok(if field.quoted {
            PivotValue::String(String::new())
        } else {
            PivotValue::Blank
        });
    }
    if options.typed && (!field.quoted || options.recognize_quoted) {
        if s.eq_ignore_ascii_case("true") {
            return Ok(PivotValue::Boolean(true));
        }
        if s.eq_ignore_ascii_case("false") {
            return Ok(PivotValue::Boolean(false));
        }
        let identifier = options.preserve_leading_zeros
            && s.len() > 1
            && s.starts_with('0')
            && s.bytes().all(|c| c.is_ascii_digit());
        if !identifier && decimal_spelling(s) {
            return s
                .parse::<PivotDecimal>()
                .map(PivotValue::Decimal)
                .map_err(|_| {
                    "numeric conversion: value is not exactly representable as Decimal96".into()
                });
        }
    }
    Ok(PivotValue::String(s.clone()))
}
impl Table {
    /// All work is isolated in a candidate; the caller installs it only on success.
    pub fn workbook(
        &self,
        options: Options,
        limits: Limits,
        runtime: &IntegrationRuntime,
    ) -> Result<Workbook> {
        if self.rows > limits.rows || self.columns > limits.columns {
            return Err("CSV worksheet coordinate limit exceeded".into());
        }
        let mut book = Workbook::new_in("CSV import", limits, runtime)
            .map_err(|e| format!("CSV workbook admission: {e}"))?;
        let mut edits = Vec::new();
        let mut live = 0usize;
        for (index, field) in self.fields.iter().enumerate() {
            let row = index / self.columns as usize + 1;
            let column = index % self.columns as usize + 1;
            let position = Position::new(row as u32, column as u32);
            let edit = match value(field, options).map_err(|e| at(row, column, &e))? {
                PivotValue::Blank => continue,
                PivotValue::String(text) => {
                    crate::entry::validate_value_text(&text, limits.source_bytes)
                        .map_err(|e| at(row, column, &format!("workbook String admission: {e}")))?;
                    Edit::SetText { position, text }
                }
                PivotValue::Boolean(value) => Edit::SetBoolean { position, value },
                PivotValue::Decimal(d) => Edit::SetDecimal {
                    position,
                    value: DecimalInput::from_decimal(d),
                },
                _ => return Err("CSV unsupported value type".into()),
            };
            live += 1;
            if live > limits.live_cells {
                return Err(at(
                    row,
                    column,
                    &format!("workbook live-cell limit: {}", limits.live_cells),
                ));
            }
            edits.push(edit);
            if edits.len() == limits.edits.max(1) {
                book.apply(book.expected(), &edits)
                    .map_err(|e| format!("CSV workbook memory/allocation admission: {e}"))?;
                edits.clear();
            }
        }
        if !edits.is_empty() {
            book.apply(book.expected(), &edits)
                .map_err(|e| format!("CSV workbook memory/allocation admission: {e}"))?;
        }
        Ok(book)
    }
}

impl Workbook {
    /// Content only: excludes styled blanks and other sheets.
    pub fn csv_content_bounds(&self, sheet: WorksheetId) -> Result<Option<Rectangle>> {
        if !self.sheets().iter().any(|s| s.id == sheet) {
            return Err("CSV unknown worksheet".into());
        }
        let mut bound: Option<Rectangle> = None;
        for (at, id) in self.persistent.positions.iter() {
            if at.worksheet != sheet {
                continue;
            }
            if self.persistent.records.get(id).is_none_or(|r| {
                r.content.is_none() || matches!(r.content, Some(crate::Content::Blank))
            }) {
                continue;
            }
            bound = Some(match bound {
                None => Rectangle {
                    first: at.position,
                    last: at.position,
                },
                Some(r) => Rectangle {
                    first: Position::new(r.first.row.min(at.row), r.first.column.min(at.column)),
                    last: Position::new(r.last.row.max(at.row), r.last.column.max(at.column)),
                },
            });
        }
        Ok(bound)
    }
}

/// One immutable capture, bounded batches, row-major failure reporting.
pub fn export(
    capture: &CalculationCapture,
    rectangle: Rectangle,
    policy: Policy,
) -> Result<Vec<u8>> {
    let Rectangle { first, last } = rectangle;
    let sheet = capture
        .pinned
        .sheets
        .as_ref()
        .ok_or("CSV missing worksheet directory")?
        .0
        .iter()
        .find(|s| s.id == capture.worksheet)
        .ok_or("CSV unknown worksheet")?;
    if first.row == 0
        || first.column == 0
        || last.row < first.row
        || last.column < first.column
        || last.row > sheet.row_count()
        || last.column > sheet.column_count()
    {
        return Err("CSV worksheet coordinate bounds exceeded".into());
    }
    let width = (last.column - first.column + 1) as usize;
    let area = u64::from(last.row - first.row + 1) * width as u64;
    if area > policy.fields as u64 {
        return Err(format!("CSV logical-field limit: {}", policy.fields));
    }
    let mut output = Vec::new();
    for start in (0..area as usize).step_by(128) {
        let positions: Vec<_> = (start..(start + 128).min(area as usize))
            .map(|i| {
                Position::new(
                    first.row + (i / width) as u32,
                    first.column + (i % width) as u32,
                )
            })
            .collect();
        let refs: Vec<_> = positions
            .iter()
            .filter_map(|p| capture.cell_at(*p))
            .collect();
        let values = capture
            .evaluate_values(&refs)
            .map_err(|e| format!("CSV evaluation: {e}"))?;
        let mut values = values.values.into_iter();
        for (offset, p) in positions.iter().enumerate() {
            let value = if capture.cell_at(*p).is_some() {
                values
                    .next()
                    .ok_or("CSV missing calculation result")?
                    .1
                    .map_err(|e| format!("CSV evaluation at {}: {e:?}", address(*p)))?
            } else {
                PivotValue::Blank
            };
            let field = match value {
                PivotValue::Blank => String::new(),
                PivotValue::String(s) => format!("\"{}\"", s.replace('"', "\"\"")),
                PivotValue::Boolean(b) => if b { "TRUE" } else { "FALSE" }.into(),
                PivotValue::Decimal(d) => d.to_string(),
                _ => return Err("CSV unsupported value type".into()),
            };
            let ending: &[u8] = if (start + offset + 1) % width == 0 {
                b"\r\n"
            } else {
                b","
            };
            if output
                .len()
                .saturating_add(field.len())
                .saturating_add(ending.len())
                > policy.bytes
            {
                return Err(format!("CSV output-size limit: {} bytes", policy.bytes));
            }
            output.extend_from_slice(field.as_bytes());
            output.extend_from_slice(ending);
        }
    }
    Ok(output)
}
fn address(p: Position) -> String {
    let mut n = p.column;
    let mut letters = Vec::new();
    while n > 0 {
        n -= 1;
        letters.push(b'A' + (n % 26) as u8);
        n /= 26;
    }
    letters.reverse();
    format!("{}{}", String::from_utf8(letters).unwrap(), p.row)
}
