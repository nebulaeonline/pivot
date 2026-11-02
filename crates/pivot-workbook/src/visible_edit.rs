//! Bounded trusted visible Clear/Fill; caller supplies a qualified immutable row plan.
use crate::*;
use pivot_core::model::integration::CancellationToken;
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ClearMode {
    Contents,
    Formatting,
    Both,
}
fn check_rows(
    book: &Workbook,
    rows: &[u32],
    first: u32,
    last: u32,
    limit: usize,
    cancel: &CancellationToken,
) -> Result<()> {
    #[cfg(feature = "history-experiment")]
    if book.history_active {
        return Err(Error::InvalidTarget);
    }
    if book.is_structured_proof() {
        return Err(Error::InvalidTarget);
    }
    if cancel.is_cancelled() {
        return Err(Error::Graph("cancelled".into()));
    }
    if rows.is_empty()
        || rows.len() > limit
        || first == 0
        || last < first
        || last > book.columns.length
    {
        return Err(Error::Position);
    }
    if rows.windows(2).any(|w| w[0] >= w[1])
        || rows[0] == 0
        || *rows.last().unwrap() > book.rows.length
    {
        return Err(Error::Position);
    }
    if rows.len() as u64 * u64::from(last - first + 1) > limit as u64 {
        return Err(Error::Limit);
    }
    Ok(())
}
impl Workbook {
    pub fn clear_visible_sheet(
        &mut self,
        sheet: GridScopeId,
        expected: Expected,
        rows: &[u32],
        columns: (u32, u32),
        mode: ClearMode,
        cancel: &CancellationToken,
    ) -> Result<Expected> {
        self.mutate_sheet(sheet, |book| {
            book.check(expected)?;
            check_rows(book, rows, columns.0, columns.1, 4096, cancel)?;
            let _scratch = book
                .core
                .allocation_domain()
                .map_err(graph_error)?
                .reserve(8 * 1024 * 1024)
                .map_err(|_| Error::Limit)?;
            let mut edits = Vec::new();
            if mode != ClearMode::Formatting {
                for &row in rows {
                    if cancel.is_cancelled() {
                        return Err(Error::Graph("cancelled".into()));
                    }
                    for column in columns.0..=columns.1 {
                        edits.push(Edit::Clear(Position::new(row, column)));
                    }
                }
            }
            persistent::clear_visible(
                book,
                expected,
                &edits,
                rows,
                columns,
                mode != ClearMode::Contents,
                cancel,
            )
        })
    }
    pub fn fill_visible_sheet(
        &mut self,
        sheet: GridScopeId,
        expected: Expected,
        rows: &[u32],
        columns: (u32, u32),
        direction: FillDirection,
        cancel: &CancellationToken,
    ) -> Result<Expected> {
        self.mutate_sheet(sheet, |book| {
            book.check(expected)?;
            check_rows(book, rows, columns.0, columns.1, 256, cancel)?;
            // Every pair is selected before duplicate_pairs captures/stages any writes.
            let pairs = rows
                .iter()
                .flat_map(|&row| {
                    (columns.0..=columns.1).filter_map(move |column| {
                        let at = Position::new(row, column);
                        let from = match direction {
                            FillDirection::Down => Position::new(rows[0], column),
                            FillDirection::Up => Position::new(*rows.last().unwrap(), column),
                            FillDirection::Right => Position::new(row, columns.0),
                            FillDirection::Left => Position::new(row, columns.1),
                        };
                        (from != at).then_some((from, at))
                    })
                })
                .collect::<Vec<_>>();
            if pairs.is_empty() {
                return Ok(expected);
            }
            book.duplicate_pairs(expected, pairs.into_iter(), cancel)
        })
    }
}
