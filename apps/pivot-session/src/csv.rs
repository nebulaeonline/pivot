//! CSV lifecycle stays separate from native workbook checkpoints.
use super::*;
use pivot_core::model::integration::resources::Charge;
use pivot_workbook::csv::{self, Options, Policy, Table};

pub(super) struct PendingImport {
    ticket: String,
    table: Table,
    _charge: Charge,
}
impl Session {
    pub(super) fn handle_csv(
        &mut self,
        q: &wire::Request,
        r: &mut wire::Response,
        op: wire::request::Operation,
    ) -> Result<(), String> {
        use pivot_storage::experimental_file::native;
        use wire::request::Operation as Op;
        if q.revision != self.revision {
            r.conflict = true;
            return Err("expected-state conflict".into());
        }
        if self.proof.is_some() {
            return Err("CSV is unavailable in experimental proof documents".into());
        }
        if q.path.len() > 4096 {
            return Err("CSV path too long".into());
        }
        let policy = Policy::default();
        let runtime = self.runtime.as_ref().ok_or("closed")?;
        match op {
            Op::CsvBounds => {
                let book = self.book.as_ref().ok_or("No workbook open")?;
                let sheet = WorksheetId::from_uuid(
                    Uuid::parse_str(&q.worksheet).map_err(|_| "CSV worksheet required")?,
                );
                r.csv_has_content = book.csv_content_bounds(sheet)?.is_some();
            }
            Op::PrepareCsvImport => {
                if self.csv_import.is_some() {
                    return Err(
                        "A CSV import is already prepared; finish or cancel it first".into(),
                    );
                }
                // Conservative reservation covers input, parsed fields, conversion and copies.
                let charge = runtime
                    .memory()
                    .reserve(policy.bytes * 4 + policy.fields * 128)
                    .map_err(|_| "CSV runtime memory admission exceeded")?;
                let file = std::fs::File::open(&q.path).map_err(|e| format!("CSV read: {e}"))?;
                if !file
                    .metadata()
                    .map_err(|e| format!("CSV read: {e}"))?
                    .is_file()
                {
                    return Err("CSV input must be a regular file".into());
                }
                let mut bytes = Vec::new();
                file.take(policy.bytes as u64 + 1)
                    .read_to_end(&mut bytes)
                    .map_err(|e| format!("CSV read: {e}"))?;
                let table = csv::parse(&bytes, policy, Limits::default())?;
                let ticket = Uuid::new_v4().to_string();
                r.csv_ticket = ticket.clone();
                r.csv_all_quoted = table.all_quoted;
                self.csv_import = Some(PendingImport {
                    ticket,
                    table,
                    _charge: charge,
                });
            }
            Op::CancelCsvImport => {
                if self
                    .csv_import
                    .as_ref()
                    .is_none_or(|p| p.ticket != q.csv_ticket)
                {
                    return Err("Unknown CSV import ticket".into());
                }
                self.csv_import.take();
            }
            Op::ImportCsv => {
                if self
                    .csv_import
                    .as_ref()
                    .is_none_or(|p| p.ticket != q.csv_ticket)
                {
                    return Err("Unknown CSV import ticket".into());
                }
                let pending = self.csv_import.take().unwrap();
                if self
                    .book
                    .as_ref()
                    .is_some_and(|b| Some(b.expected()) != self.saved)
                    && !q.discard
                {
                    return Err(
                        "Unsaved changes: save or explicitly discard before importing CSV".into(),
                    );
                }
                let book = pending.table.workbook(
                    Options {
                        typed: q.csv_typed,
                        preserve_leading_zeros: !q.csv_convert_leading_zeros,
                        recognize_quoted: q.csv_recognize_quoted,
                    },
                    Limits::default(),
                    runtime,
                )?;
                // No state changes before the full candidate is admitted.
                self.pending.take();
                self.captures.clear();
                self.proof_frame.take();
                self.book = Some(book);
                self.location = None;
                self.saved = None;
                self.checkpoint.clear();
                self.uncertain = false;
                self.revision = Uuid::new_v4().to_string();
                r.revision = self.revision.clone();
            }
            Op::ExportCsv => {
                let _charge = runtime
                    .memory()
                    .reserve(policy.bytes * 4)
                    .map_err(|_| "CSV runtime memory admission exceeded")?;
                let book = self.book.as_ref().ok_or("No workbook open")?;
                let sheet = WorksheetId::from_uuid(
                    Uuid::parse_str(&q.worksheet).map_err(|_| "CSV worksheet required")?,
                );
                let rect = if q.csv_whole_sheet {
                    book.csv_content_bounds(sheet)?
                        .ok_or("CSV active sheet has no content")?
                } else {
                    Rectangle {
                        first: pos(q.first.as_ref())?,
                        last: pos(q.last.as_ref())?,
                    }
                };
                let destination = native::destination(std::path::Path::new(&q.path))
                    .map_err(|e| format!("CSV destination: {e}"))?;
                if self
                    .location
                    .as_ref()
                    .is_some_and(|l| l.path == destination.path)
                {
                    return Err("CSV export cannot overwrite the open native workbook; choose a different destination".into());
                }
                let capture = book
                    .capture_sheet(sheet, book.expected())
                    .map_err(|e| format!("CSV capture: {e}"))?;
                let bytes = csv::export(&capture, rect, policy)?;
                native::install(&destination, &bytes)
                    .map_err(|e| format!("CSV file installation: {e}"))?;
            }
            _ => unreachable!(),
        }
        Ok(())
    }
}
