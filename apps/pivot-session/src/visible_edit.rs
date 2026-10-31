//! Qualified exact Clear/Fill plans sharing the single-use visible confirmation slot.
use super::*;
use wire::request::Operation as O;
impl Session {
    fn edit_plan(
        &mut self,
        q: &wire::Request,
    ) -> Result<(WorksheetId, Vec<u32>, Rectangle), String> {
        if !matches!(
            O::try_from(q.operation),
            Ok(O::ClearVisible | O::FillVisible)
        ) {
            return Err("visible Clear/Fill required".into());
        }
        if q.operation == O::ClearVisible as i32 {
            if !matches!(q.clear_mode, 1..=3) || q.fill_direction != 0 {
                return Err("explicit Clear mode required".into());
            }
        } else if !matches!(q.fill_direction, 1..=4) || q.clear_mode != 0 {
            return Err("explicit Fill direction required".into());
        }
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
            return Err("invalid visible edit rectangle".into());
        }
        let count = visible
            .iter()
            .filter(|&&r| r >= first.row && r <= last.row)
            .count();
        if count == 0 {
            return Err("No visible cells selected; Review, Reapply or Clear Filter".into());
        }
        let limit = if q.operation == O::ClearVisible as i32 {
            4096
        } else {
            256
        };
        if count as u64 * u64::from(last.column - first.column + 1) > limit {
            return Err(format!(
                "Visible {} exceeds {limit} addressed slots (including Blank, absent and source cells); select a smaller area",
                if q.operation == O::ClearVisible as i32 {
                    "Clear"
                } else {
                    "Fill"
                }
            ));
        }
        let rows = visible
            .into_iter()
            .filter(|&r| r >= first.row && r <= last.row)
            .collect();
        Ok((sheet, rows, Rectangle { first, last }))
    }
    pub(super) fn handle_visible_edit(
        &mut self,
        q: &wire::Request,
        r: &mut wire::Response,
        op: O,
    ) -> Result<(), String> {
        if op == O::PrepareVisibleEdit {
            self.visible_confirmation = None;
            self.filter_confirmation = None;
            let operation =
                wire::Request::decode(q.filter_operation.as_slice()).map_err(|e| e.to_string())?;
            if operation.worksheet != q.worksheet
                || operation.revision != q.revision
                || operation.computational_state != q.computational_state
            {
                return Err("exact visible edit state required".into());
            }
            let (_, rows, rectangle) = self.edit_plan(&operation)?;
            let bytes = visible::canonical(&operation);
            let charge = self
                .runtime
                .as_ref()
                .ok_or("closed")?
                .memory()
                .reserve(bytes.len() * 3 + rows.len() * 16 + 8192)
                .map_err(|_| "visible edit plan admission")?;
            let ticket = Uuid::new_v4().to_string();
            r.filter_confirmation = ticket.clone();
            r.worksheet = q.worksheet.clone();
            r.visible_rows = rows.clone();
            r.visible_first_column = rectangle.first.column;
            r.visible_width = rectangle.last.column - rectangle.first.column + 1;
            r.visible_skipped_rows =
                rectangle.last.row - rectangle.first.row + 1 - rows.len() as u32;
            r.filter_generation = self.filter.generation.clone();
            self.visible_confirmation = Some(visible::Confirmation {
                ticket,
                operation: bytes,
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
            .ok_or("visible edit confirmation missing or consumed")?;
        let (sheet, rows, rectangle) = self.edit_plan(q)?;
        let book = self.book.as_mut().unwrap();
        if q.filter_confirmation != confirmed.ticket
            || visible::canonical(q) != confirmed.operation
            || book.expected() != confirmed.expected
            || self.filter.generation != confirmed.generation
            || rows != confirmed.rows
            || rectangle != confirmed.rectangle
        {
            return Err("visible edit confirmation stale or changed; no replay".into());
        }
        let before = book.expected();
        let columns = (rectangle.first.column, rectangle.last.column);
        if op == O::ClearVisible {
            use pivot_workbook::visible_edit::ClearMode;
            let mode = match q.clear_mode {
                1 => ClearMode::Contents,
                2 => ClearMode::Formatting,
                3 => ClearMode::Both,
                _ => unreachable!(),
            };
            book.clear_visible_sheet(sheet, before, &rows, columns, mode, &Default::default())
        } else {
            let direction = match q.fill_direction {
                1 => pivot_workbook::FillDirection::Up,
                2 => pivot_workbook::FillDirection::Down,
                3 => pivot_workbook::FillDirection::Left,
                4 => pivot_workbook::FillDirection::Right,
                _ => unreachable!(),
            };
            book.fill_visible_sheet(
                sheet,
                before,
                &rows,
                columns,
                direction,
                &Default::default(),
            )
        }
        .map_err(|e| e.to_string())?;
        if book.expected() != before {
            self.revision = Uuid::new_v4().to_string();
        }
        r.revision = self.revision.clone();
        r.worksheet = q.worksheet.clone();
        r.visible_rows = rows;
        r.visible_first_column = columns.0;
        r.visible_width = columns.1 - columns.0 + 1;
        Ok(())
    }
}
