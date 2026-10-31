//! Exact mapped destinations, independent of the ordinary rectangular clipboard guard.
use super::*;
pub(super) struct Confirmation {
    ticket: String,
    operation: Vec<u8>,
    expected: pivot_workbook::Expected,
    generation: String,
    rows: Vec<u32>,
    rectangle: Rectangle,
    _charge: pivot_core::model::integration::resources::Charge,
}
fn canonical(q: &wire::Request) -> Vec<u8> {
    let mut q = q.clone();
    q.version = 0;
    q.id = 0;
    q.filter_confirmation.clear();
    q.encode_to_vec()
}
pub(super) fn validate_native_kind(q: &wire::Request) -> Result<(), String> {
    if !q.native_clipboard {
        if q.clipboard_kind != 0 {
            return Err("external clipboard kind must be zero".into());
        }
        return Ok(());
    }
    let magic = match q.clipboard_kind {
        0 => return Ok(()),
        5 => b"PIVOTCB5",
        6 => b"PIVOTCB6",
        _ => return Err("unsupported native clipboard kind".into()),
    };
    if !q.clipboard.starts_with(magic) {
        return Err("preferred native clipboard representation is malformed; no fallback".into());
    }
    Ok(())
}
impl Session {
    fn visible_rows(&mut self, q: &wire::Request) -> Result<(WorksheetId, Vec<u32>), String> {
        if self.proof.is_some() {
            return Err("Visible commands require an ordinary worksheet".into());
        }
        if q.revision != self.revision || q.computational_state != self.computational_state {
            return Err("expected-state conflict".into());
        }
        let sheet = WorksheetId::from_uuid(
            Uuid::parse_str(&q.worksheet).map_err(|_| "worksheet ID required")?,
        );
        let book = self.book.as_ref().ok_or("create workbook first")?;
        self.filter.ensure(book, sheet, false)?;
        let mut state = wire::Response::default();
        self.filter.describe(book, sheet, &mut state)?;
        if !matches!(state.filter_status, 0 | 1) {
            return Err("Visible operations require a qualified filter view; Review, Reapply or Clear Filter".into());
        }
        Ok((sheet, self.filter.rows(book, sheet)?))
    }
    fn visible_plan(
        &mut self,
        q: &wire::Request,
    ) -> Result<(WorksheetId, Vec<u32>, Rectangle), String> {
        validate_native_kind(q)?;
        let (sheet, visible) = self.visible_rows(q)?;
        let first = pos(q.first.as_ref())?;
        let last = pos(q.last.as_ref())?;
        if first.row == 0
            || first.column == 0
            || last.row < first.row
            || last.column < first.column
            || last.row > 65536
            || last.column > 1024
        {
            return Err("invalid visible destination rectangle".into());
        }
        let book = self.book.as_ref().unwrap();
        let (height, width) = book
            .clipboard_shape(book.expected(), &q.clipboard, q.native_clipboard)
            .map_err(|e| e.to_string())?;
        let rows: Vec<_> = if first == last {
            if visible.binary_search(&first.row).is_err() {
                return Err("destination anchor is hidden".into());
            }
            visible
                .into_iter()
                .filter(|row| *row >= first.row)
                .take(height as usize)
                .collect()
        } else {
            if last.column - first.column + 1 != width {
                return Err("visible destination width must match exactly".into());
            }
            if visible
                .iter()
                .filter(|row| **row >= first.row && **row <= last.row)
                .count()
                != height as usize
            {
                return Err("visible destination row count must match exactly".into());
            }
            visible
                .into_iter()
                .filter(|row| *row >= first.row && *row <= last.row)
                .collect()
        };
        if rows.len() != height as usize {
            return Err("not enough visible destination rows".into());
        }
        let column = first
            .column
            .checked_add(width - 1)
            .filter(|c| *c <= 1024)
            .ok_or("destination column overflow")?;
        let rectangle = Rectangle {
            first: Position::new(rows[0], first.column),
            last: Position::new(*rows.last().unwrap(), column),
        };
        Ok((sheet, rows, rectangle))
    }
    pub(super) fn handle_visible(
        &mut self,
        q: &wire::Request,
        r: &mut wire::Response,
        op: wire::request::Operation,
    ) -> Result<(), String> {
        use wire::request::Operation as O;
        if op == O::CopyVisible {
            let (sheet, visible) = self.visible_rows(q)?;
            let first = pos(q.first.as_ref())?;
            let last = pos(q.last.as_ref())?;
            if first.row == 0
                || first.column == 0
                || last.row < first.row
                || last.column < first.column
                || last.row > 65536
                || last.column > 1024
            {
                return Err("invalid visible source rectangle".into());
            }
            let count = visible
                .iter()
                .filter(|row| **row >= first.row && **row <= last.row)
                .count();
            if count == 0 {
                return Err("No visible cells selected; clipboard unchanged".into());
            }
            if count as u64 * u64::from(last.column - first.column + 1) > 256 {
                return Err("Visible Copy exceeds 256 addressed slots".into());
            }
            let rows = visible
                .into_iter()
                .filter(|row| *row >= first.row && *row <= last.row)
                .collect::<Vec<_>>();
            let book = self.book.as_ref().unwrap();
            let copied = book
                .copy_visible_sheet(
                    sheet,
                    book.expected(),
                    &rows,
                    first.column,
                    last.column,
                    &Default::default(),
                )
                .map_err(|e| e.to_string())?;
            r.worksheet = q.worksheet.clone();
            r.visible_rows = rows;
            r.visible_first_column = first.column;
            r.visible_width = last.column - first.column + 1;
            r.clipboard = copied.native;
            r.clipboard_text = copied.text;
            r.clipboard_text_available = copied.text_available;
            r.clipboard_kind = 6;
            return Ok(());
        }
        if op == O::PrepareVisiblePaste {
            self.visible_confirmation = None;
            self.filter_confirmation = None;
            let operation =
                wire::Request::decode(q.filter_operation.as_slice()).map_err(|e| e.to_string())?;
            if operation.operation != O::PasteVisible as i32
                || operation.worksheet != q.worksheet
                || operation.revision != q.revision
                || operation.computational_state != q.computational_state
            {
                return Err("exact visible paste required".into());
            }
            let (_, rows, rectangle) = self.visible_plan(&operation)?;
            let skipped = if operation.first == operation.last {
                rectangle.last.row - rectangle.first.row + 1
            } else {
                pos(operation.last.as_ref())?.row - pos(operation.first.as_ref())?.row + 1
            } - rows.len() as u32;
            let operation = canonical(&operation);
            let charge = self
                .runtime
                .as_ref()
                .ok_or("closed")?
                .memory()
                .reserve(operation.len() * 3 + 8192)
                .map_err(|_| "visible plan admission")?;
            let ticket = Uuid::new_v4().to_string();
            r.filter_confirmation = ticket.clone();
            r.worksheet = q.worksheet.clone();
            r.visible_rows = rows.clone();
            r.visible_first_column = rectangle.first.column;
            r.visible_width = rectangle.last.column - rectangle.first.column + 1;
            r.visible_skipped_rows = skipped;
            r.filter_generation = self.filter.generation.clone();
            self.visible_confirmation = Some(Confirmation {
                ticket,
                operation,
                expected: self.book.as_ref().unwrap().expected(),
                generation: self.filter.generation.clone(),
                rows,
                rectangle,
                _charge: charge,
            });
            return Ok(());
        }
        let confirmed = self
            .visible_confirmation
            .take()
            .ok_or("visible confirmation missing or consumed")?;
        let (sheet, rows, rectangle) = self.visible_plan(q)?;
        let book = self.book.as_mut().unwrap();
        if q.filter_confirmation != confirmed.ticket
            || canonical(q) != confirmed.operation
            || book.expected() != confirmed.expected
            || self.filter.generation != confirmed.generation
            || rows != confirmed.rows
            || rectangle != confirmed.rectangle
        {
            return Err("visible confirmation stale or changed; no operation replayed".into());
        }
        let before = book.expected();
        book.paste_visible_sheet(
            sheet,
            before,
            pivot_workbook::clipboard::VisibleDestination {
                rectangle,
                rows: &rows,
            },
            &q.clipboard,
            q.native_clipboard,
            &Default::default(),
        )
        .map_err(|e| e.to_string())?;
        if book.expected() != before {
            self.revision = Uuid::new_v4().to_string();
        }
        r.revision = self.revision.clone();
        r.worksheet = q.worksheet.clone();
        r.visible_rows = rows;
        r.visible_first_column = rectangle.first.column;
        r.visible_width = rectangle.last.column - rectangle.first.column + 1;
        Ok(())
    }
}
