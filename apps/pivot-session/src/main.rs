//! Private application child, one workbook and one explicitly owned runtime.
use pivot_core::model::integration::IntegrationRuntime;
use pivot_protocol::{application::v1 as wire, v1::Decimal128};
use pivot_types::PivotDecimal;
use pivot_workbook::{
    CalculationCapture, Content, DecimalInput, Edit, Limits, Position, Rectangle, Workbook,
};
use prost::Message;
use std::{
    collections::BTreeMap,
    io::{self, Read, Write},
    time::Instant,
};
use uuid::Uuid;

const MAX_FRAME: usize = 1_048_576;
const MAX_CELLS: usize = 256;
struct Capture {
    value: CalculationCapture,
    revision: String,
}
struct Session {
    runtime: Option<IntegrationRuntime>,
    book: Option<Workbook>,
    revision: String,
    captures: BTreeMap<String, Capture>,
}
fn pos(p: Option<&wire::Position>) -> Result<Position, String> {
    p.map(|p| Position::new(p.row, p.column))
        .ok_or("position required".into())
}
fn decimal(d: PivotDecimal) -> Decimal128 {
    let (lo, mid, hi, negative, scale) = d.parts();
    Decimal128 {
        lo,
        mid,
        hi,
        negative,
        scale,
    }
}
fn location(p: Position) -> wire::Position {
    wire::Position {
        row: p.row,
        column: p.column,
    }
}
fn readable_error(error: impl std::fmt::Display) -> String {
    let detail = error.to_string();
    if detail.contains("Unorderable") {
        "Circular formula reference.".into()
    } else if detail.contains("MissingReference") {
        "A referenced cell is no longer available.".into()
    } else if detail.contains("Parse") || detail.contains("InvalidDraft") {
        "Invalid formula; edit it to repair this cell.".into()
    } else if detail.contains("DivisionByZero") {
        "Division by zero.".into()
    } else if detail.contains("Overflow") {
        "Decimal overflow.".into()
    } else {
        "Formula could not be calculated.".into()
    }
}
impl Session {
    fn new() -> Result<Self, String> {
        Ok(Self {
            runtime: Some(IntegrationRuntime::standalone().map_err(|e| e.to_string())?),
            book: None,
            revision: String::new(),
            captures: BTreeMap::new(),
        })
    }
    fn close(&mut self) -> Result<u64, String> {
        self.captures.clear();
        self.book.take();
        if let Some(runtime) = self.runtime.take() {
            let memory = runtime.memory();
            runtime
                .finish()
                .map_err(|_| "runtime still owned".to_string())?;
            return Ok(memory.stats().charged as u64);
        }
        Ok(0)
    }
    fn handle(&mut self, q: &wire::Request, r: &mut wire::Response) -> Result<(), String> {
        use wire::request::Operation as Op;
        if q.version != 1 {
            return Err("unsupported application version".into());
        }
        let operation = Op::try_from(q.operation).map_err(|_| "unknown operation")?;
        if operation == Op::Close {
            r.remaining_charge = self.close()?;
            r.closed = true;
            return Ok(());
        }
        if operation == Op::Create {
            if self.book.is_some() {
                return Err("workbook already created".into());
            }
            self.book = Some(
                Workbook::new_in(
                    "Pivot",
                    Limits::default(),
                    self.runtime.as_ref().ok_or("closed")?,
                )
                .map_err(|e| e.to_string())?,
            );
            self.revision = Uuid::new_v4().to_string();
            r.revision = self.revision.clone();
            return Ok(());
        }
        let book = self.book.as_mut().ok_or("create workbook first")?;
        if !matches!(operation, Op::Calculate | Op::Release) && q.revision != self.revision {
            r.conflict = true;
            return Err("expected-state conflict".into());
        }
        match operation {
            Op::Apply => {
                if q.edits.is_empty() || q.edits.len() > 256 {
                    return Err("1..256 edits required".into());
                }
                let edits = q
                    .edits
                    .iter()
                    .map(|e| {
                        let position = pos(e.at.as_ref())?;
                        Ok(match e.action.as_ref().ok_or("edit action required")? {
                            wire::edit::Action::Number(d) => Edit::SetDecimal {
                                position,
                                value: DecimalInput::from_parts(
                                    d.lo, d.mid, d.hi, d.negative, d.scale,
                                )
                                .map_err(|e| e.to_string())?,
                            },
                            wire::edit::Action::Formula(source) => Edit::SetFormulaText {
                                position,
                                source: source.clone(),
                            },
                            wire::edit::Action::Clear(true) => Edit::Clear(position),
                            wire::edit::Action::MoveTo(p) => Edit::Move {
                                from: position,
                                to: pos(Some(p))?,
                            },
                            _ => return Err("invalid edit action".into()),
                        })
                    })
                    .collect::<Result<Vec<_>, String>>()?;
                let before = book.expected();
                let after = book.apply(before, &edits).map_err(|e| e.to_string())?;
                if before != after {
                    self.revision = Uuid::new_v4().to_string()
                }
                r.revision = self.revision.clone();
            }
            Op::Read => {
                let first = pos(q.first.as_ref())?;
                let last = pos(q.last.as_ref())?;
                let area = (last.row as u64).saturating_sub(first.row as u64) + 1;
                let area = area
                    .saturating_mul((last.column as u64).saturating_sub(first.column as u64) + 1);
                if area > MAX_CELLS as u64 {
                    return Err("viewport exceeds 256 cells".into());
                }
                let cells = book
                    .read(book.expected(), Rectangle { first, last }, MAX_CELLS)
                    .map_err(|e| e.to_string())?;
                let capture = book
                    .capture_local(book.expected())
                    .map_err(|e| e.to_string())?;
                r.generation = Uuid::new_v4().to_string();
                for cell in cells.cells {
                    let mut view = wire::Cell {
                        at: Some(location(cell.position)),
                        cell_id: cell.reference.cell.uuid().to_string(),
                        node_id: cell.node.uuid().to_string(),
                        ..Default::default()
                    };
                    view.content = Some(match cell.content {
                        Content::Decimal(d) => wire::cell::Content::Number({
                            let (lo, mid, hi, negative, scale) = d.parts();
                            Decimal128 {
                                lo,
                                mid,
                                hi,
                                negative,
                                scale,
                            }
                        }),
                        Content::TextFormula(f) => wire::cell::Content::Formula(f.original_source),
                        Content::Formula { source, .. } => wire::cell::Content::Formula(source),
                        Content::Blank => wire::cell::Content::Blank(true),
                    });
                    // One bounded viewport request, independent cell diagnostics from one pinned S.
                    match capture.evaluate(&[cell.reference]) {
                        Ok(result) => match &result.values[0].1 {
                            Ok(v) => view.value = Some(decimal(*v)),
                            Err(e) => view.error = readable_error(format!("{e:?}")),
                        },
                        Err(e) => view.error = readable_error(e),
                    }
                    r.cells.push(view);
                }
            }
            Op::Capture => {
                if self.captures.len() >= 4 {
                    return Err("capture limit (4)".into());
                }
                let id = Uuid::new_v4().to_string();
                self.captures.insert(
                    id.clone(),
                    Capture {
                        value: book
                            .capture_local(book.expected())
                            .map_err(|e| e.to_string())?,
                        revision: self.revision.clone(),
                    },
                );
                r.capture = id;
            }
            Op::Calculate => {
                if q.outputs.is_empty() || q.outputs.len() > MAX_CELLS {
                    return Err("1..256 outputs required".into());
                }
                let capture = self.captures.get(&q.capture).ok_or("unknown capture")?;
                r.revision = capture.revision.clone();
                let refs = q
                    .outputs
                    .iter()
                    .map(|p| {
                        capture
                            .value
                            .cell_at(pos(Some(p))?)
                            .ok_or("empty output".into())
                    })
                    .collect::<Result<Vec<_>, String>>()?;
                let values = capture.value.evaluate(&refs).map_err(|e| e.to_string())?;
                r.generation = values.generation.to_string();
                for (p, (id, value)) in q.outputs.iter().zip(values.values) {
                    let mut cell = wire::Cell {
                        at: Some(*p),
                        cell_id: id.cell.uuid().to_string(),
                        ..Default::default()
                    };
                    match value {
                        Ok(v) => cell.value = Some(decimal(v)),
                        Err(e) => cell.error = format!("{e:?}"),
                    };
                    r.cells.push(cell);
                }
            }
            Op::Release => {
                if self.captures.remove(&q.capture).is_none() {
                    return Err("unknown capture".into());
                }
            }
            _ => return Err("unsupported operation".into()),
        }
        Ok(())
    }
}
impl Drop for Session {
    fn drop(&mut self) {
        let _ = self.close();
    }
}
fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut session = Session::new().map_err(io::Error::other)?;
    let mut input = io::stdin().lock();
    let mut output = io::stdout().lock();
    loop {
        let mut header = [0; 4];
        if input.read(&mut header[..1])? == 0 {
            break;
        }
        input.read_exact(&mut header[1..])?;
        let length = u32::from_le_bytes(header) as usize;
        if length == 0 || length > MAX_FRAME {
            return Err("invalid frame size".into());
        }
        let mut bytes = vec![0; length];
        input.read_exact(&mut bytes)?;
        let q = wire::Request::decode(bytes.as_slice())?;
        let mut r = wire::Response {
            version: 1,
            id: q.id,
            revision: session.revision.clone(),
            ..Default::default()
        };
        let start = Instant::now();
        if let Err(e) = session.handle(&q, &mut r) {
            r.error = e
        }
        r.execution_ns = start.elapsed().as_nanos() as u64;
        let start = Instant::now();
        let _ = r.encode_to_vec();
        r.serialization_ns = start.elapsed().as_nanos() as u64;
        let bytes = r.encode_to_vec();
        if bytes.len() > MAX_FRAME {
            return Err("response too large".into());
        }
        output.write_all(&(bytes.len() as u32).to_le_bytes())?;
        output.write_all(&bytes)?;
        output.flush()?;
        if r.closed {
            break;
        }
    }
    Ok(())
}
