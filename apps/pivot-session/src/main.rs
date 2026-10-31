//! Private application child, one workbook and one explicitly owned runtime.
use pivot_core::model::integration::IntegrationRuntime;
use pivot_protocol::{application::v1 as wire, v1::Decimal128};
use pivot_types::PivotDecimal;
use pivot_workbook::{
    CalculationCapture, Content, DecimalInput, Edit, Limits, Position, Rectangle, Workbook,
    WorksheetId,
};
use prost::Message;
use std::{
    collections::BTreeMap,
    io::{self, Read, Write},
    time::Instant,
};
use uuid::Uuid;

mod csv;
mod filtering;
#[cfg(test)]
mod filtering_tests;
mod projections;
mod structured;

const MAX_FRAME: usize = 1_048_576;
const MAX_CELLS: usize = 256;
struct Capture {
    computational_state: String,
    proof: Option<pivot_workbook::projection::Capture>,
    value: CalculationCapture,
    revision: String,
}
struct PendingSave {
    ticket: String,
    revision: String,
    export: pivot_workbook_document::Export,
    location: pivot_storage::experimental_file::native::Location,
}
#[derive(Default)]
struct Session {
    filter: filtering::Presentation,
    filter_confirmation: Option<filtering::Confirmation>,
    computational_state: String,
    proof: Option<pivot_workbook::projection::State>,
    proof_frame: Option<pivot_workbook::projection::Frame>,
    location: Option<pivot_storage::experimental_file::native::Location>,
    saved: Option<pivot_workbook::Expected>,
    checkpoint: String,
    pending: Option<PendingSave>,
    csv_import: Option<csv::PendingImport>,
    uncertain: bool,
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
fn set_value(cell: &mut wire::Cell, value: pivot_types::PivotValue) {
    use pivot_types::PivotValue as V;
    use wire::typed_value::Kind;
    cell.result = Some(wire::TypedValue {
        kind: Some(match value {
            V::Decimal(d) => {
                cell.value = Some(decimal(d));
                Kind::Decimal(decimal(d))
            }
            V::Boolean(b) => Kind::Boolean(b),
            V::String(s) => Kind::Text(s),
            V::Blank => Kind::Blank(true),
            _ => unreachable!("closed current value kinds"),
        }),
    });
}
fn error_code(error: &pivot_workbook::CellError) -> String {
    use pivot_workbook::CellError as E;
    match error {
        E::Typed(pivot_core::typed::Error::ReferenceDeleted) => "ReferenceDeleted",
        E::Typed(pivot_core::typed::Error::TypeMismatch) => "TypeMismatch",
        E::Typed(pivot_core::typed::Error::InvalidArgument) => "InvalidArgument",
        E::Typed(pivot_core::typed::Error::TextTooLong) => "TextTooLong",
        E::Arithmetic(pivot_types::DecimalError::DivisionByZero) => "DivisionByZero",
        E::Arithmetic(_) => "NumericError",
        E::NoNumericValues(_) => "NoNumericValues",
        E::NonNumericInput => "TypeMismatch",
        E::DependencyFailed => "DependencyFailed",
        E::CapturedSourceFailed { .. } => "CapturedSourceFailed",
        _ => "InvalidExpression",
    }
    .into()
}
fn location(p: Position) -> wire::Position {
    wire::Position {
        row: p.row,
        column: p.column,
    }
}
fn readable_error(error: impl std::fmt::Display) -> String {
    let detail = error.to_string();
    if detail.contains("CapturedSourceFailed") {
        for reason in [
            "MissingProducer",
            "MissingOutput",
            "UnavailableResult",
            "SchemaMismatch",
            "TypeMismatch",
            "NumericOverflow",
        ] {
            if detail.contains(reason) {
                return format!("{reason}: the structured numeric output could not be used.");
            }
        }
        "Structured source failed.".into()
    } else if detail.contains("NonNumericInput") {
        "Nonnumeric input: a numeric formula cannot consume text.".into()
    } else if detail.contains("Unorderable") {
        "Circular formula reference.".into()
    } else if detail.contains("MissingReference") {
        "A referenced cell is no longer available.".into()
    } else if detail.contains("Parse") || detail.contains("InvalidDraft") {
        "Invalid formula; edit it to repair this cell.".into()
    } else if detail.contains("DivisionByZero") {
        "Division by zero.".into()
    } else if detail.contains("NoNumericValues") {
        "No numeric values in this range.".into()
    } else if detail.contains("Overflow") {
        "Decimal overflow.".into()
    } else {
        "Formula could not be calculated.".into()
    }
}
fn file_error(error: pivot_storage::experimental_file::Error) -> String {
    use pivot_storage::experimental_file::Error;
    match error {
        Error::Conflict=>"The file changed outside Pivot or is not a supported regular file. Open it to review changes, or Save As to a new file.".into(),
        Error::Busy=>"Another Pivot writer is using this directory. Wait for it to finish, then save again.".into(),
        Error::Uncertain(cause)=>format!("Save installation is uncertain ({cause}). Reopen to verify the file, or Save As to a new file to preserve current work."),
        Error::Io(cause)=>format!("Could not access the workbook file: {cause}. Check the path and permissions."),
        error=>format!("Workbook file operation failed: {error}. Choose a supported local file or a different destination."),
    }
}
impl Session {
    fn new() -> Result<Self, String> {
        Ok(Self {
            computational_state: Uuid::new_v4().to_string(),
            proof: None,
            proof_frame: None,
            location: None,
            saved: None,
            checkpoint: String::new(),
            pending: None,
            csv_import: None,
            uncertain: false,
            runtime: Some(IntegrationRuntime::standalone().map_err(|e| e.to_string())?),
            book: None,
            revision: String::new(),
            captures: BTreeMap::new(),
            filter: Default::default(),
            filter_confirmation: None,
        })
    }
    fn close(&mut self) -> Result<u64, String> {
        self.proof_frame.take();
        self.pending.take();
        self.csv_import.take();
        self.captures.clear();
        self.proof.take();
        self.filter_confirmation = None;
        self.filter = Default::default();
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
        if q.version != 11 {
            return Err("unsupported application version".into());
        }
        let operation = Op::try_from(q.operation).map_err(|_| "unknown operation")?;
        if matches!(operation, Op::ProofStart | Op::BindingStart)
            && self
                .book
                .as_ref()
                .is_some_and(|b| b.filters().next().is_some())
        {
            return Err("Filters are ordinary worksheet state; clear them before entering an experimental proof.".into());
        }
        if operation == Op::PrepareFilteredOperation {
            return self.prepare_filtered_operation(q, r);
        }
        let single_edit = operation == Op::Apply
            && q.edits.len() == 1
            && !matches!(q.edits[0].action, Some(wire::edit::Action::MoveTo(_)));
        if filtering::is_bulk(operation)
            && (!single_edit || !q.filter_confirmation.is_empty())
            && self.book.is_some()
        {
            if q.revision != self.revision {
                r.conflict = true;
                return Err("expected-state conflict".into());
            }
            self.guard_filtered_operation(q)?;
        }
        if matches!(
            operation,
            Op::PrepareCsvImport
                | Op::ImportCsv
                | Op::ExportCsv
                | Op::CancelCsvImport
                | Op::CsvBounds
        ) {
            return self.handle_csv(q, r, operation);
        }
        if matches!(operation, Op::AddSheet | Op::RenameSheet) && self.proof.is_some() {
            return Err("worksheet metadata is unavailable in experimental proof documents".into());
        }
        if matches!(operation, Op::ProofStart | Op::BindingStart)
            && self.book.as_ref().is_some_and(|b| b.sheets().len() != 1)
        {
            return Err("experimental proof modes require a separate single-sheet document".into());
        }
        self.proof_frame.take();
        if (self.book.as_ref().is_some_and(|b| b.is_structured_proof())
            || operation == wire::request::Operation::BindingStart
            || operation == wire::request::Operation::Fill)
            && !matches!(
                operation,
                wire::request::Operation::Read
                    | wire::request::Operation::ProofRead
                    | wire::request::Operation::Calculate
                    | wire::request::Operation::ProofEvaluate
                    | wire::request::Operation::Release
                    | wire::request::Operation::Close
            )
            && q.computational_state != self.computational_state
        {
            r.conflict = true;
            return Err("computational state conflict".into());
        }
        if (20..=30).contains(&q.operation) {
            return self.handle_structured(q, r, operation);
        }

        if operation == Op::ProofPublish
            && self.book.as_ref().is_some_and(|b| b.is_structured_proof())
        {
            let binding = self
                .book
                .as_ref()
                .and_then(|b| b.current_structured_binding())
                .ok_or("MissingProducer: no active producer")?;
            self.handle_proof(q, r, operation)?;
            self.proof
                .as_mut()
                .ok_or("missing proof")?
                .adopt_output_identity(binding.producer, binding.output);
            return Ok(());
        }
        if matches!(
            operation,
            Op::ProofStart | Op::ProofPublish | Op::ProofLayout | Op::ProofRead | Op::ProofEvaluate
        ) {
            return self.handle_proof(q, r, operation);
        }
        if self.proof.is_some() && matches!(operation, Op::PrepareSave | Op::Save) {
            return Err(
                "Experimental projection document cannot be saved; discard or cancel.".into(),
            );
        }

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
            // Clean baseline for an untouched untitled document; no disk checkpoint.
            self.saved = self.book.as_ref().map(Workbook::expected);
            self.revision = Uuid::new_v4().to_string();
            r.revision = self.revision.clone();
            return Ok(());
        }
        if matches!(
            operation,
            Op::New | Op::Open | Op::CloseDocument | Op::PrepareSave | Op::Save
        ) {
            use pivot_storage::experimental_file::native;
            let captured_authority = operation == Op::Save
                && self
                    .pending
                    .as_ref()
                    .is_some_and(|p| p.ticket == q.save_ticket && p.revision == q.revision);
            if q.revision != self.revision && !captured_authority {
                r.conflict = true;
                return Err("expected-state conflict".into());
            }
            if q.path.len() > 4096 {
                return Err("path too long".into());
            }
            if matches!(operation, Op::New | Op::Open | Op::CloseDocument) {
                if (self
                    .book
                    .as_ref()
                    .is_some_and(|b| Some(b.expected()) != self.saved)
                    || self.proof.as_ref().is_some_and(|p| p.changed))
                    && !q.discard
                {
                    return Err("Unsaved changes: save or explicitly discard before replacing the workbook.".into());
                }
                // Validate the entire candidate before changing the current document.
                let candidate = if operation == Op::Open {
                    let start = Instant::now();
                    let (bytes, location) =
                        native::read(std::path::Path::new(&q.path)).map_err(|e| e.to_string())?;
                    r.file_ns = start.elapsed().as_nanos() as u64;
                    let mut timings = pivot_workbook_document::Timings::default();
                    let (book, checkpoint) =
                        pivot_workbook_document::Document::open_document_measured(
                            &bytes,
                            Limits::default(),
                            self.runtime.as_ref().ok_or("closed")?,
                            &mut timings,
                        )
                        .map_err(|e| e.to_string())?;
                    r.decode_ns = timings.decode_ns;
                    r.restore_ns = timings.restore_ns;
                    Some((book, Some(location), checkpoint.to_string()))
                } else if operation == Op::New {
                    Some((
                        Workbook::new_in(
                            "Pivot",
                            Limits::default(),
                            self.runtime.as_ref().ok_or("closed")?,
                        )
                        .map_err(|e| e.to_string())?,
                        None,
                        String::new(),
                    ))
                } else {
                    None
                };
                self.pending.take();
                self.csv_import.take();
                self.captures.clear();
                self.proof.take();
                self.book.take();
                self.location = None;
                self.saved = None;
                self.checkpoint.clear();
                self.uncertain = false;
                if let Some((book, location, checkpoint)) = candidate {
                    self.saved = Some(book.expected());
                    self.book = Some(book);
                    self.location = location;
                    self.checkpoint = checkpoint;
                }
                self.revision = Uuid::new_v4().to_string();
                r.revision = self.revision.clone();
                return Ok(());
            }
            if operation == Op::PrepareSave {
                if self.pending.is_some() {
                    return Err("A save is already prepared; finish or cancel it first.".into());
                }
                let location = if q.path.is_empty() {
                    self.location.clone().ok_or("Save As requires a path")?
                } else {
                    {
                        let observed = native::destination(std::path::Path::new(&q.path))
                            .map_err(file_error)?;
                        if let Some(current) =
                            self.location.as_ref().filter(|l| l.path == observed.path)
                        {
                            current.clone()
                        } else {
                            observed
                        }
                    }
                };
                if self.uncertain
                    && (location.fingerprint.is_some()
                        || self
                            .location
                            .as_ref()
                            .is_some_and(|l| l.path == location.path))
                {
                    return Err("Previous save installation is uncertain. Reopen to verify it, or Save As to a new file to preserve current work.".into());
                }
                let book = self.book.as_ref().ok_or("No workbook open")?;
                let start = Instant::now();
                let export =
                    pivot_workbook_document::Document::export_document(book, book.expected())
                        .map_err(|e| e.to_string())?;
                r.export_ns = start.elapsed().as_nanos() as u64;
                r.capture_ns = export.timings.capture_ns;
                r.encode_ns = export.timings.encode_ns;
                let ticket = Uuid::new_v4().to_string();
                r.save_ticket = ticket.clone();
                self.pending = Some(PendingSave {
                    revision: self.revision.clone(),
                    ticket,
                    export,
                    location,
                });
                return Ok(());
            }
            if q.save_ticket.is_empty()
                || self
                    .pending
                    .as_ref()
                    .is_none_or(|p| p.ticket != q.save_ticket)
            {
                return Err("Unknown save ticket".into());
            }
            let pending = self.pending.take().unwrap();
            if q.discard {
                return Ok(());
            } // Explicitly abandon a prepared save, without writing.
            let start = Instant::now();
            let outcome = native::install(&pending.location, &pending.export.bytes);
            r.file_ns = start.elapsed().as_nanos() as u64;
            match outcome {
                Ok(location) => {
                    self.uncertain = false;
                    self.location = Some(location);
                    self.saved = Some(pending.export.expected);
                    self.checkpoint = pending.export.checkpoint.to_string();
                }
                Err(e) => {
                    self.uncertain |=
                        matches!(e, pivot_storage::experimental_file::Error::Uncertain(_));
                    r.uncertain = self.uncertain;
                    return Err(file_error(e));
                }
            }
            return Ok(());
        }
        if matches!(
            operation,
            Op::SetFilter
                | Op::ReadFiltered
                | Op::ReapplyFilter
                | Op::ValidateFilter
                | Op::GetFilter
        ) {
            return self.handle_filter(q, r, operation);
        }
        let book = self.book.as_mut().ok_or("create workbook first")?;
        if !matches!(operation, Op::Calculate | Op::Release) && q.revision != self.revision {
            r.conflict = true;
            return Err("expected-state conflict".into());
        }
        let sheet = if matches!(
            operation,
            Op::Fill
                | Op::Structure
                | Op::Sort
                | Op::SortBounds
                | Op::CopyBlock
                | Op::PasteBlock
                | Op::Apply
                | Op::Read
                | Op::Capture
                | Op::Calculate
                | Op::RenameSheet
                | Op::Format
                | Op::ReadStyles
        ) {
            let id = WorksheetId::from_uuid(
                Uuid::parse_str(&q.worksheet).map_err(|_| "worksheet ID required")?,
            );
            if !book.sheets().iter().any(|s| s.id == id) {
                return Err("unknown worksheet ID".into());
            }
            r.worksheet = id.uuid().to_string();
            Some(id)
        } else {
            None
        };
        match operation {
            Op::ListSheets => (),
            Op::AddSheet => {
                let id = book
                    .add_sheet(
                        book.expected(),
                        (!q.sheet_name.is_empty()).then_some(q.sheet_name.as_str()),
                    )
                    .map_err(|e| e.to_string())?;
                r.worksheet = id.uuid().to_string();
                self.revision = Uuid::new_v4().to_string();
                r.revision = self.revision.clone();
            }
            Op::RenameSheet => {
                let before = book.expected();
                book.rename_sheet(before, sheet.unwrap(), &q.sheet_name)
                    .map_err(|e| e.to_string())?;
                if book.expected() != before {
                    self.revision = Uuid::new_v4().to_string();
                }
                r.revision = self.revision.clone();
            }
            Op::Fill => {
                let direction = match wire::request::FillDirection::try_from(q.fill_direction) {
                    Ok(wire::request::FillDirection::Up) => pivot_workbook::FillDirection::Up,
                    Ok(wire::request::FillDirection::Down) => pivot_workbook::FillDirection::Down,
                    Ok(wire::request::FillDirection::Left) => pivot_workbook::FillDirection::Left,
                    Ok(wire::request::FillDirection::Right) => pivot_workbook::FillDirection::Right,
                    _ => return Err("Specify Fill Up, Down, Left or Right".into()),
                };
                let before = book.expected();
                book.fill_sheet(
                    sheet.unwrap(),
                    before,
                    Rectangle {
                        first: pos(q.first.as_ref())?,
                        last: pos(q.last.as_ref())?,
                    },
                    direction,
                    &Default::default(),
                )
                .map_err(|e| {
                    format!("Fill rejected ({e}); check references and the 256-position limit.")
                })?;
                if book.expected() != before {
                    self.revision = Uuid::new_v4().to_string();
                }
                r.revision = self.revision.clone();
            }
            Op::CopyBlock => {
                let rectangle = Rectangle {
                    first: pos(q.first.as_ref())?,
                    last: pos(q.last.as_ref())?,
                };
                let copied = book
                    .copy_sheet(sheet.unwrap(), book.expected(), rectangle)
                    .map_err(|e| e.to_string())?;
                r.clipboard = copied.native;
                r.clipboard_text = copied.text;
            }
            Op::PasteBlock => {
                let rectangle = Rectangle {
                    first: pos(q.first.as_ref())?,
                    last: pos(q.last.as_ref())?,
                };
                let before = book.expected();
                let pasted = book
                    .paste_sheet(
                        sheet.unwrap(),
                        before,
                        rectangle,
                        &q.clipboard,
                        q.native_clipboard,
                        &Default::default(),
                    )
                    .map_err(|e| e.to_string())?;
                if book.expected() != before {
                    self.revision = Uuid::new_v4().to_string();
                }
                r.revision = self.revision.clone();
                r.pasted_first = Some(location(pasted.first));
                r.pasted_last = Some(location(pasted.last));
            }
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
                            wire::edit::Action::Boolean(value) => Edit::SetBoolean {
                                position,
                                value: *value,
                            },
                            wire::edit::Action::Text(text) => Edit::SetText {
                                position,
                                text: text.clone(),
                            },
                            wire::edit::Action::Entry(source) => Edit::SetEntry {
                                position,
                                source: source.clone(),
                            },
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
                let after = book
                    .apply_sheet(sheet.unwrap(), before, &edits)
                    .map_err(|e| e.to_string())?;
                if before != after && q.operation != wire::request::Operation::Format as i32 {
                    self.revision = Uuid::new_v4().to_string()
                }
                r.revision = self.revision.clone();
            }
            Op::SortBounds => {
                let suggested = book
                    .sort_bounds(
                        sheet.unwrap(),
                        book.expected(),
                        Rectangle {
                            first: pos(q.first.as_ref())?,
                            last: pos(q.last.as_ref())?,
                        },
                        &Default::default(),
                    )
                    .map_err(|e| e.to_string())?;
                r.sort_suggested_first = Some(wire::Position {
                    row: suggested.first.row,
                    column: suggested.first.column,
                });
                r.sort_suggested_last = Some(wire::Position {
                    row: suggested.last.row,
                    column: suggested.last.column,
                });
            }
            Op::Sort => {
                let outcome = book
                    .sort_rows(
                        sheet.unwrap(),
                        book.expected(),
                        &pivot_workbook::sorting::Options {
                            rectangle: Rectangle {
                                first: pos(q.first.as_ref())?,
                                last: pos(q.last.as_ref())?,
                            },
                            exclude_header: q.sort_exclude_header,
                            move_formatting: q.sort_move_formatting,
                            keys: q
                                .sort_keys
                                .iter()
                                .map(|k| pivot_workbook::sorting::Key {
                                    column: k.column,
                                    descending: k.descending,
                                })
                                .collect(),
                        },
                        &Default::default(),
                    )
                    .map_err(|e| e.to_string())?;
                if outcome.changed {
                    self.revision = Uuid::new_v4().to_string();
                }
                r.revision = self.revision.clone();
                r.sort_changed = outcome.changed;
                r.sort_moved_rows = outcome.moved_rows as u32;
                r.sort_moved_cells = outcome.moved_cells as u32;
                r.sort_work = outcome.work as u64;
            }
            Op::Structure => {
                use pivot_workbook::structure::{Action, AxisKind, Change};
                let outcome = book
                    .restructure(
                        sheet.unwrap(),
                        book.expected(),
                        Change {
                            axis: if q.structure_columns {
                                AxisKind::Columns
                            } else {
                                AxisKind::Rows
                            },
                            action: if q.structure_delete {
                                Action::Delete
                            } else {
                                Action::Insert
                            },
                            start: q.structure_start,
                            count: q.structure_count,
                        },
                        &Default::default(),
                    )
                    .map_err(|e| e.to_string())?;
                if outcome.changed {
                    self.revision = Uuid::new_v4().to_string();
                }
                r.revision = self.revision.clone();
                r.structure_changed = outcome.changed;
                r.formatting_clipped = outcome.formatting_clipped;
                r.structure_moved = outcome.moved as u32;
                r.structure_retired = outcome.retired as u32;
                r.structure_work = outcome.work as u64;
            }
            Op::ReadStyles => {
                let first = pos(q.first.as_ref())?;
                let last = pos(q.last.as_ref())?;
                book.axes_at_sheet(sheet.unwrap(), first)
                    .map_err(|e| e.to_string())?;
                book.axes_at_sheet(sheet.unwrap(), last)
                    .map_err(|e| e.to_string())?;
                let rectangle = Rectangle { first, last };
                if q.style_summary {
                    let (style, mixed) = book
                        .style_summary(sheet.unwrap(), rectangle)
                        .map_err(|e| e.to_string())?;
                    r.summary_style = style.encode().to_vec();
                    r.mixed_style_fields = mixed;
                    r.worksheet_default_style = book
                        .worksheet_default(sheet.unwrap())
                        .map_err(|e| e.to_string())?
                        .encode()
                        .to_vec();
                } else {
                    for (p, style) in book
                        .style_viewport(sheet.unwrap(), rectangle)
                        .map_err(|e| e.to_string())?
                    {
                        r.styles.push(wire::StyledPosition {
                            at: Some(location(p)),
                            style: style.encode().to_vec(),
                        });
                    }
                }
            }

            Op::Format => {
                if self.proof.is_some() {
                    return Err("formatting unavailable in proof documents".into());
                }
                use pivot_workbook::formatting::{Field, Style, StylePatch};
                let bits = if q.reset_format { 1023 } else { q.style_fields };
                if bits & !1023 != 0 {
                    return Err("unknown style fields".into());
                }
                let v = if q.reset_format {
                    Style::default()
                } else {
                    Style::decode(&q.style).map_err(|e| e.to_string())?
                };
                let patch = StylePatch {
                    numeric: if bits & 1 != 0 {
                        if q.style_inherit {
                            Field::Inherit
                        } else {
                            Field::Set(v.numeric)
                        }
                    } else {
                        Field::Keep
                    },
                    places: if bits & 2 != 0 {
                        if q.style_inherit {
                            Field::Inherit
                        } else {
                            Field::Set(v.places)
                        }
                    } else {
                        Field::Keep
                    },
                    rounding: if bits & 4 != 0 {
                        if q.style_inherit {
                            Field::Inherit
                        } else {
                            Field::Set(v.rounding)
                        }
                    } else {
                        Field::Keep
                    },
                    grouping: if bits & 8 != 0 {
                        if q.style_inherit {
                            Field::Inherit
                        } else {
                            Field::Set(v.grouping)
                        }
                    } else {
                        Field::Keep
                    },
                    currency: if bits & 16 != 0 {
                        if q.style_inherit {
                            Field::Inherit
                        } else {
                            Field::Set(v.currency)
                        }
                    } else {
                        Field::Keep
                    },
                    bold: if bits & 32 != 0 {
                        if q.style_inherit {
                            Field::Inherit
                        } else {
                            Field::Set(v.bold)
                        }
                    } else {
                        Field::Keep
                    },
                    italic: if bits & 64 != 0 {
                        if q.style_inherit {
                            Field::Inherit
                        } else {
                            Field::Set(v.italic)
                        }
                    } else {
                        Field::Keep
                    },
                    alignment: if bits & 128 != 0 {
                        if q.style_inherit {
                            Field::Inherit
                        } else {
                            Field::Set(v.alignment)
                        }
                    } else {
                        Field::Keep
                    },
                    foreground: if bits & 256 != 0 {
                        if q.style_inherit {
                            Field::Inherit
                        } else {
                            Field::Set(v.foreground)
                        }
                    } else {
                        Field::Keep
                    },
                    background: if bits & 512 != 0 {
                        if q.style_inherit {
                            Field::Inherit
                        } else {
                            Field::Set(v.background)
                        }
                    } else {
                        Field::Keep
                    },
                };
                let before = book.expected();
                if q.worksheet_defaults {
                    book.set_worksheet_default(sheet.unwrap(), before, patch, &Default::default())
                } else {
                    book.format_sheet(
                        sheet.unwrap(),
                        before,
                        Rectangle {
                            first: pos(q.first.as_ref())?,
                            last: pos(q.last.as_ref())?,
                        },
                        patch,
                        q.reset_format,
                        &Default::default(),
                    )
                }
                .map_err(|e| e.to_string())?;
                if book.expected() != before {
                    self.revision = Uuid::new_v4().to_string();
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
                    .read_sheet(
                        sheet.unwrap(),
                        book.expected(),
                        Rectangle { first, last },
                        MAX_CELLS,
                    )
                    .map_err(|e| e.to_string())?;
                let mut capture = book
                    .capture_sheet(sheet.unwrap(), book.expected())
                    .map_err(|e| e.to_string())?;
                structured::prepare_capture(self.proof.as_ref(), &mut capture)?;
                r.capture_state = self.computational_state.clone();
                if let Some(source) = &self.proof {
                    r.captured_source_epoch = source.epoch;
                    r.captured_source_generation = source
                        .current_generation()
                        .map(|id| id.0.to_string())
                        .unwrap_or_default();
                }
                r.generation = Uuid::new_v4().to_string();
                r.capture_identity = r.generation.clone();
                for (p, style) in book
                    .style_viewport(sheet.unwrap(), Rectangle { first, last })
                    .map_err(|e| e.to_string())?
                {
                    r.styles.push(wire::StyledPosition {
                        at: Some(location(p)),
                        style: style.encode().to_vec(),
                    });
                }
                for cell in cells.cells {
                    let mut view = wire::Cell {
                        at: Some(location(cell.position)),
                        cell_id: cell.reference.cell.uuid().to_string(),
                        node_id: cell.node.uuid().to_string(),
                        worksheet: cell.reference.worksheet.uuid().to_string(),
                        ..Default::default()
                    };
                    if matches!(&cell.content, Content::StructuredFormula(_))
                        || matches!(&cell.content,Content::Formula{tokens,..} if tokens.iter().any(|t|matches!(t,pivot_workbook::Token::Output(_))))
                    {
                        structured::set_cell_binding(&mut view, &capture);
                    }
                    let effective = book
                        .display_formula(cell.reference)
                        .map_err(|e| e.to_string())?;
                    view.content = Some(match cell.content {
                        Content::Boolean(b) => wire::cell::Content::Boolean(b),
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
                        Content::TextFormula(f) => {
                            view.authored_formula = f.original_source.clone();
                            wire::cell::Content::Formula(effective.unwrap_or(f.original_source))
                        }
                        Content::Formula { source, .. } => {
                            view.authored_formula = source.clone();
                            wire::cell::Content::Formula(effective.unwrap_or(source))
                        }
                        Content::StructuredFormula(f) => {
                            wire::cell::Content::Formula(f.source().into())
                        }
                        Content::LiteralText(text) => wire::cell::Content::Text(text),
                        Content::Blank => wire::cell::Content::Blank(true),
                    });
                    // One bounded viewport request, independent cell diagnostics from one pinned S.
                    {
                        match capture.evaluate_values(&[cell.reference]) {
                            Ok(result) => match &result.values[0].1 {
                                Ok(v) => set_value(&mut view, v.clone()),
                                Err(e) => {
                                    structured::set_cell_fault(&mut view, e, &capture);
                                    view.error_code = error_code(e);
                                    view.error = readable_error(format!("{e:?}"));
                                }
                            },
                            Err(e) => view.error = readable_error(e),
                        }
                    }
                    if let Some(wire::TypedValue {
                        kind: Some(wire::typed_value::Kind::Decimal(d)),
                    }) = &view.result
                    {
                        let v = PivotDecimal::from_parts(d.lo, d.mid, d.hi, d.negative, d.scale)
                            .map_err(|e| e.to_string())?;
                        view.formatted_number = pivot_workbook::formatting::render_decimal(
                            v,
                            book.style_at(sheet.unwrap(), cell.position)
                                .map_err(|e| e.to_string())?,
                        )
                        .map_err(|e| e.to_string())?;
                    }
                    r.cells.push(view);
                }
            }
            Op::Capture => {
                if self
                    .proof
                    .as_ref()
                    .is_some_and(|p| q.proof_epoch != p.epoch)
                {
                    r.conflict = true;
                    return Err("result epoch conflict".into());
                }

                if self.captures.len() >= if self.proof.is_some() { 2 } else { 4 } {
                    return Err(if self.proof.is_some() {
                        "capture limit (2)"
                    } else {
                        "capture limit (4)"
                    }
                    .into());
                }
                let id = Uuid::new_v4().to_string();
                self.captures.insert(
                    id.clone(),
                    Capture {
                        computational_state: self.computational_state.clone(),
                        proof: self
                            .proof
                            .as_ref()
                            .map(|p| p.capture())
                            .transpose()
                            .map_err(|e| e.to_string())?,
                        value: {
                            let mut capture = book
                                .capture_sheet(sheet.unwrap(), book.expected())
                                .map_err(|e| e.to_string())?;
                            structured::prepare_capture(self.proof.as_ref(), &mut capture)?;
                            capture
                        },
                        revision: self.revision.clone(),
                    },
                );
                r.capture_identity = id.clone();
                r.capture_state = self.computational_state.clone();
                if let Some(source) = &self.proof {
                    r.captured_source_epoch = source.epoch;
                    r.captured_source_generation = source
                        .current_generation()
                        .map(|id| id.0.to_string())
                        .unwrap_or_default();
                }
                r.capture = id;
            }
            Op::Calculate => {
                if q.outputs.is_empty() || q.outputs.len() > MAX_CELLS {
                    return Err("1..256 outputs required".into());
                }
                let capture = self.captures.get(&q.capture).ok_or("unknown capture")?;
                r.capture_identity = q.capture.clone();
                r.capture_state = capture.computational_state.clone();
                if let Some(source) = &capture.proof {
                    r.captured_source_epoch = source.epoch;
                    r.captured_source_generation = source
                        .available_generation()
                        .map(|id| id.0.to_string())
                        .unwrap_or_default();
                }
                r.revision = capture.revision.clone();
                let refs = q
                    .outputs
                    .iter()
                    .map(|p| {
                        capture
                            .value
                            .cell_at_sheet(sheet.unwrap(), pos(Some(p))?)
                            .ok_or("empty output".into())
                    })
                    .collect::<Result<Vec<_>, String>>()?;
                let values = capture
                    .value
                    .evaluate_values(&refs)
                    .map_err(|e| e.to_string())?;
                r.generation = values.generation.to_string();
                for (p, (id, value)) in q.outputs.iter().zip(values.values) {
                    let mut cell = wire::Cell {
                        at: Some(*p),
                        cell_id: id.cell.uuid().to_string(),
                        worksheet: id.worksheet.uuid().to_string(),
                        ..Default::default()
                    };
                    match value {
                        Ok(v) => set_value(&mut cell, v),
                        Err(e) => {
                            structured::set_cell_fault(&mut cell, &e, &capture.value);
                            cell.error_code = error_code(&e);
                            cell.error = readable_error(format!("{e:?}"));
                        }
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
            version: 11,
            id: q.id,
            revision: session.revision.clone(),
            ..Default::default()
        };
        let start = Instant::now();
        let before = (
            session
                .book
                .as_ref()
                .map(|b| (b.expected().session, b.expected().graph)),
            session.proof.as_ref().map(|p| p.epoch),
            session
                .book
                .as_ref()
                .is_some_and(|b| b.is_structured_proof()),
        );
        // Retain admission through response encoding for style viewport records,
        // per-cell formatted strings and the additional encoded presentation bytes.
        let presentation_lease = if matches!(
            wire::request::Operation::try_from(q.operation),
            Ok(wire::request::Operation::Read
                | wire::request::Operation::ReadStyles
                | wire::request::Operation::ReadFiltered
                | wire::request::Operation::SetFilter
                | wire::request::Operation::ValidateFilter
                | wire::request::Operation::GetFilter)
        ) {
            session.runtime.as_ref().map(|runtime| {
                runtime.memory().reserve(
                    if matches!(
                        wire::request::Operation::try_from(q.operation),
                        Ok(wire::request::Operation::ReadFiltered
                            | wire::request::Operation::SetFilter
                            | wire::request::Operation::ValidateFilter
                            | wire::request::Operation::GetFilter)
                    ) {
                        MAX_FRAME * 4
                    } else {
                        MAX_FRAME
                    },
                )
            })
        } else {
            None
        };
        if presentation_lease
            .as_ref()
            .is_some_and(|lease| lease.is_err())
        {
            r.error = "presentation memory limit".into();
        } else if let Err(e) = session.handle(&q, &mut r) {
            r.error = e;
        }
        let after = (
            session
                .book
                .as_ref()
                .map(|b| (b.expected().session, b.expected().graph)),
            session.proof.as_ref().map(|p| p.epoch),
            session
                .book
                .as_ref()
                .is_some_and(|b| b.is_structured_proof()),
        );
        if before != after {
            session.computational_state = Uuid::new_v4().to_string();
        }
        r.computational_state = session.computational_state.clone();
        r.binding_mode = session
            .book
            .as_ref()
            .is_some_and(|b| b.is_structured_proof());
        r.document_id = session
            .book
            .as_ref()
            .map(|b| b.graph_id().to_string())
            .unwrap_or_default();
        if let Some(book) = &session.book {
            let stats = book.stats();
            let limits = book.admission_limits();
            r.positioned_cells = stats.live_cells as u32;
            r.allocated_cell_identities = stats.allocated_nodes as u32;
            r.format_regions = book.style_regions() as u32;
            r.format_region_limit = book.style_limits().positions as u32;
            r.format_bytes = book.style_bytes() as u64;
            r.positioned_cell_limit = limits.live_cells as u32;
            r.allocated_cell_identity_limit = limits.allocated_cells as u32;
            r.worksheets = book
                .sheets()
                .iter()
                .map(|s| wire::WorksheetInfo {
                    id: s.id.uuid().to_string(),
                    name: s.name.clone(),
                    rows: s.row_count(),
                    columns: s.column_count(),
                })
                .collect();
            if r.worksheet.is_empty() {
                r.worksheet = if book
                    .sheets()
                    .iter()
                    .any(|s| s.id.uuid().to_string() == q.worksheet)
                {
                    q.worksheet.clone()
                } else {
                    book.worksheet().uuid().to_string()
                };
            }
        }
        if let Some(book) = &session.book
            && let Ok(id) = Uuid::parse_str(&r.worksheet)
        {
            session.filter.requalify(book, WorksheetId::from_uuid(id));
            if let Err(e) = session
                .filter
                .describe(book, WorksheetId::from_uuid(id), &mut r)
                && r.error.is_empty()
            {
                r.error = e;
            }
        }
        r.path = session
            .location
            .as_ref()
            .map(|l| l.path.to_string_lossy().into_owned())
            .unwrap_or_default();
        r.saved_checkpoint = session.checkpoint.clone();
        r.proof_mode = session.proof.is_some();
        if let Some(proof) = &session.proof {
            r.proof_epoch = proof.epoch;
            r.proof_presentation = proof.presentation;
        }
        r.dirty = session
            .book
            .as_ref()
            .is_some_and(|b| Some(b.expected()) != session.saved)
            || session.proof.as_ref().is_some_and(|p| p.changed);
        r.uncertain = session.uncertain;
        if let (Some(book), Some(runtime)) = (&session.book, &session.runtime) {
            let metrics = book.selection_metrics();
            let memory = runtime.memory();
            let (allocations, bytes) = memory.allocation_traffic();
            let domain = memory.stats();
            r.selection_stats = Some(wire::SelectionStats {
                visited_index_nodes: metrics.visited_index_nodes,
                selection_candidates: metrics.selection_candidates,
                emitted_members: metrics.emitted_members,
                points: metrics.points as u64,
                selections: metrics.selections as u64,
                identities: metrics.identities as u64,
                consumers: metrics.consumers as u64,
                invalidated_nodes: metrics.invalidated_nodes as u64,
                allocation_count: allocations as u64,
                allocation_bytes: bytes as u64,
                domain_charge: domain.charged as u64,
                domain_peak: domain.peak as u64,
            });
        }
        r.execution_ns = start.elapsed().as_nanos() as u64;
        if r.encoded_len() > MAX_FRAME - 128
            && matches!(
                wire::request::Operation::try_from(q.operation),
                Ok(wire::request::Operation::Read
                    | wire::request::Operation::ReadStyles
                    | wire::request::Operation::ReadFiltered
                    | wire::request::Operation::Calculate
                    | wire::request::Operation::GetFilter)
            )
        {
            r.cells.clear();
            r.styles.clear();
            r.filter_definition = None;
            r.filter_rows.clear();
            r.error="Response exceeds the existing frame limit; request fewer cells. The workbook remains open.".into();
        }
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

#[cfg(test)]
mod protocol_version_tests {
    use super::*;
    #[test]
    fn incompatible_peers_reject_before_creating_state() {
        let mut session = Session::default();
        for version in 0..6 {
            let request = wire::Request {
                version,
                operation: wire::request::Operation::Create as i32,
                ..Default::default()
            };
            assert_eq!(
                session.handle(&request, &mut wire::Response::default()),
                Err("unsupported application version".into())
            );
            assert!(session.book.is_none() && session.runtime.is_none());
        }
    }
}
