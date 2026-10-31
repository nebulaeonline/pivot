//! One snapshot-based, identity-preserving permutation of row segments.
use super::*;
use pivot_core::model::{integration::CancellationToken, selection};
use pivot_types::PivotValue;
use std::cmp::Ordering;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Key {
    pub column: u32,
    pub descending: bool,
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Options {
    pub rectangle: Rectangle,
    pub exclude_header: bool,
    pub keys: Vec<Key>,
    pub move_formatting: bool,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Outcome {
    pub expected: Expected,
    pub changed: bool,
    pub moved_rows: usize,
    pub moved_cells: usize,
    pub work: usize,
}
pub(crate) struct Work<'a> {
    pub(crate) used: usize,
    pub(crate) cancel: &'a CancellationToken,
}
impl Work<'_> {
    pub(crate) fn add(&mut self, n: usize) -> Result<()> {
        if self.cancel.is_cancelled() {
            return Err(Error::Graph("cancelled".into()));
        }
        self.used = self.used.checked_add(n).ok_or(Error::Exhausted)?;
        if self.used > structure::MAX_WORK {
            return Err(Error::Capacity {
                resource: "sorting work",
                limit: structure::MAX_WORK,
            });
        }
        Ok(())
    }
}
/// Stable, bounded and immediately fallible, including cancellation in comparators.
pub(crate) fn sort_move_units(len: usize) -> usize {
    if len < 2 {
        0
    } else {
        len * (usize::BITS - (len - 1).leading_zeros()) as usize * 2
    }
}
pub(crate) fn sort_by<T: Copy>(
    items: &mut [T],
    mut cmp: impl FnMut(T, T) -> Result<Ordering>,
) -> Result<()> {
    if items.len() < 2 {
        return Ok(());
    }
    let mut scratch = items.to_vec();
    let mut width = 1;
    while width < items.len() {
        for start in (0..items.len()).step_by(width * 2) {
            let middle = (start + width).min(items.len());
            let end = (middle + width).min(items.len());
            let (mut left, mut right) = (start, middle);
            for output in &mut scratch[start..end] {
                if left < middle
                    && (right == end || cmp(items[left], items[right])? != Ordering::Greater)
                {
                    *output = items[left];
                    left += 1;
                } else {
                    *output = items[right];
                    right += 1;
                }
            }
        }
        items.copy_from_slice(&scratch);
        width *= 2;
    }
    Ok(())
}
type Value = std::result::Result<PivotValue, EvaluationError>;
const BLANK: Value = Ok(PivotValue::Blank);
fn rank(value: &Value) -> u8 {
    match value {
        Ok(PivotValue::Decimal(_)) => 0,
        Ok(PivotValue::String(_)) => 1,
        Ok(PivotValue::Boolean(_)) => 2,
        Err(_) => 3,
        Ok(PivotValue::Blank) => 4,
        _ => 5,
    }
}
fn compare(a: &Value, b: &Value, descending: bool, work: &mut Work<'_>) -> Result<Ordering> {
    work.add(1)?;
    let (ar, br) = (rank(a), rank(b));
    if ar >= 3 || br >= 3 {
        return Ok(ar.cmp(&br));
    }
    let order = match (a, b) {
        (Ok(PivotValue::Decimal(a)), Ok(PivotValue::Decimal(b))) => a.cmp(b),
        (Ok(PivotValue::Boolean(a)), Ok(PivotValue::Boolean(b))) => a.cmp(b),
        (Ok(PivotValue::String(a)), Ok(PivotValue::String(b))) => {
            // UTF-8 preserves scalar order; count every examined byte and poll long prefixes.
            let mut order = Ordering::Equal;
            for (a, b) in a.bytes().zip(b.bytes()) {
                work.add(1)?;
                order = a.cmp(&b);
                if order != Ordering::Equal {
                    break;
                }
            }
            if order == Ordering::Equal {
                a.len().cmp(&b.len())
            } else {
                order
            }
        }
        _ => ar.cmp(&br),
    };
    Ok(if descending { order.reverse() } else { order })
}
#[derive(Clone, Copy)]
struct KeyCell {
    row: u32,
    level: usize,
    node: NodeId,
}
impl Workbook {
    /// Explicit adjacent-column suggestion only; never changes the selected rectangle.
    pub fn sort_bounds(
        &self,
        sheet: WorksheetId,
        expected: Expected,
        rectangle: Rectangle,
        cancel: &CancellationToken,
    ) -> Result<Rectangle> {
        self.check(expected)?;
        self.axes_at_sheet(sheet, rectangle.first)?;
        self.axes_at_sheet(sheet, rectangle.last)?;
        if rectangle.first.row > rectangle.last.row
            || rectangle.first.column > rectangle.last.column
        {
            return Err(Error::Position);
        }
        let _scratch = self
            .persistent
            .memory
            .reserve(65536)
            .map_err(|_| Error::Limit)?;
        let mut work = Work { used: 0, cancel };
        let mut columns = BTreeSet::new();
        for (at, id) in self.persistent.positions.iter() {
            work.add(1)?;
            if at.worksheet == sheet
                && at.position.row >= rectangle.first.row
                && at.position.row <= rectangle.last.row
                && self
                    .persistent
                    .records
                    .get(id)
                    .is_some_and(|r| !matches!(r.content, None | Some(Content::Blank)))
            {
                columns.insert(at.position.column);
            }
        }
        let mut suggested = rectangle;
        while suggested.first.column > 1 && columns.contains(&(suggested.first.column - 1)) {
            work.add(1)?;
            suggested.first.column -= 1;
        }
        let limit = self.sheet_info(sheet)?.column_count();
        while suggested.last.column < limit && columns.contains(&(suggested.last.column + 1)) {
            work.add(1)?;
            suggested.last.column += 1;
        }
        Ok(suggested)
    }
    fn sort_permutation(
        &self,
        sheet: WorksheetId,
        r: Rectangle,
        keys: &[Key],
        work: &mut Work<'_>,
    ) -> Result<Vec<u32>> {
        let height = (r.last.row - r.first.row + 1) as usize;
        let mut levels = vec![None; self.sheet_info(sheet)?.column_count() as usize + 1];
        for (i, key) in keys.iter().enumerate() {
            levels[key.column as usize] = Some(i);
        }
        let mut cells = Vec::new();
        for (at, id) in self.persistent.positions.iter() {
            work.add(1)?;
            if at.worksheet != sheet
                || at.position.row < r.first.row
                || at.position.row > r.last.row
            {
                continue;
            }
            if let Some(level) = levels[at.position.column as usize] {
                let node = self
                    .persistent
                    .bindings
                    .get(id)
                    .ok_or(Error::InvalidTarget)?
                    .runtime;
                cells.push(KeyCell {
                    row: at.position.row,
                    level,
                    node,
                });
            }
        }
        work.add(sort_move_units(cells.len()))?;
        sort_by(&mut cells, |a, b| {
            work.add(1)?;
            Ok((a.row, a.level).cmp(&(b.row, b.level)))
        })?;
        let capture = self.capture_sheet(sheet, self.expected())?;
        let nodes = cells.iter().map(|c| c.node).collect::<Vec<_>>();
        let calculation = capture
            .local_core
            .evaluate_values(&nodes, None, work.cancel)
            .map_err(graph_error)?;
        let mut values = Vec::with_capacity(cells.len());
        let mut offsets = Vec::with_capacity(height + 1);
        let mut index = 0;
        for row in r.first.row..=r.last.row {
            work.add(1)?;
            offsets.push(values.len());
            while index < cells.len() && cells[index].row == row {
                work.add(
                    1 + (usize::BITS - calculation.values().len().max(1).leading_zeros()) as usize,
                )?;
                let value = calculation
                    .values()
                    .get(&cells[index].node)
                    .ok_or(Error::InvalidTarget)?;
                if matches!(
                    value,
                    Err(EvaluationError::InvalidExpression { .. }
                        | EvaluationError::UnknownNode(_)
                        | EvaluationError::MissingCapturedContext { .. })
                ) {
                    return Err(Error::InvalidDraft);
                }
                if rank(value) == 5 {
                    return Err(Error::InvalidTarget);
                }
                values.push((cells[index].level, value));
                index += 1;
            }
        }
        offsets.push(values.len());
        let mut rows = (0..height as u32).collect::<Vec<_>>();
        work.add(sort_move_units(rows.len()))?;
        sort_by(&mut rows, |a, b| {
            work.add(1)?;
            let mut av = &values[offsets[a as usize]..offsets[a as usize + 1]];
            let mut bv = &values[offsets[b as usize]..offsets[b as usize + 1]];
            while !av.is_empty() || !bv.is_empty() {
                let level = av
                    .first()
                    .map_or(usize::MAX, |v| v.0)
                    .min(bv.first().map_or(usize::MAX, |v| v.0));
                let aa = if av.first().is_some_and(|v| v.0 == level) {
                    let v = av[0].1;
                    av = &av[1..];
                    v
                } else {
                    &BLANK
                };
                let bb = if bv.first().is_some_and(|v| v.0 == level) {
                    let v = bv[0].1;
                    bv = &bv[1..];
                    v
                } else {
                    &BLANK
                };
                let order = compare(aa, bb, keys[level].descending, work)?;
                if order != Ordering::Equal {
                    return Ok(order);
                }
            }
            Ok(a.cmp(&b))
        })?;
        let mut permutation = vec![0; height];
        for (new, old) in rows.into_iter().enumerate() {
            work.add(1)?;
            permutation[old as usize] = r.first.row + new as u32;
        }
        Ok(permutation)
    }
    pub fn sort_rows(
        &mut self,
        sheet: WorksheetId,
        expected: Expected,
        options: &Options,
        cancel: &CancellationToken,
    ) -> Result<Outcome> {
        self.check(expected)?;
        let mut work = Work { used: 0, cancel };
        work.add(0)?;
        if options.rectangle.first.row > options.rectangle.last.row
            || options.rectangle.first.column > options.rectangle.last.column
        {
            return Err(Error::Position);
        }
        self.axes_at_sheet(sheet, options.rectangle.first)?;
        self.axes_at_sheet(sheet, options.rectangle.last)?;
        if self.is_structured_proof() {
            return Err(Error::InvalidTarget);
        }
        #[cfg(feature = "history-experiment")]
        if self.history_active || self.history_recording.is_some() {
            return Err(Error::InvalidTarget);
        }
        let mut seen = BTreeSet::new();
        if options.keys.is_empty() {
            return Err(Error::InvalidTarget);
        }
        for key in &options.keys {
            work.add(1)?;
            if key.column < options.rectangle.first.column
                || key.column > options.rectangle.last.column
                || !seen.insert(key.column)
            {
                return Err(Error::InvalidTarget);
            }
        }
        let mut r = options.rectangle;
        if options.exclude_header {
            r.first.row += 1;
        }
        let unchanged = |work| Outcome {
            expected,
            changed: false,
            moved_rows: 0,
            moved_cells: 0,
            work,
        };
        if r.first.row >= r.last.row {
            return Ok(unchanged(work.used));
        }
        let height = (r.last.row - r.first.row + 1) as usize;
        let _scratch = self
            .persistent
            .memory
            .reserve(self.persistent.positions.len().saturating_mul(256) + height * 48 + 8192)
            .map_err(|_| Error::Limit)?;
        let permutation = self.sort_permutation(sheet, r, &options.keys, &mut work)?;
        let moved_rows = permutation
            .iter()
            .enumerate()
            .filter(|(i, row)| **row != r.first.row + *i as u32)
            .count();
        if moved_rows == 0 {
            return Ok(unchanged(work.used));
        }
        let mut data = self
            .core
            .stage_resource(&self.persistent)
            .map_err(graph_error)?;
        let mut placements = Vec::new();
        for (at, id) in self.persistent.positions.iter() {
            work.add(1)?;
            let old = at.position;
            if at.worksheet != sheet
                || old.row < r.first.row
                || old.row > r.last.row
                || old.column < r.first.column
                || old.column > r.last.column
            {
                continue;
            }
            let new = Position::new(permutation[(old.row - r.first.row) as usize], old.column);
            if new != old {
                placements.push((*id, old, new));
            }
        }
        for (_, old, _) in &placements {
            work.add(1)?;
            data.positions
                .remove(&sheets::at(sheet, *old))
                .map_err(|_| Error::Limit)?;
        }
        for (id, _, new) in &placements {
            work.add(1)?;
            data.positions
                .insert_private(sheets::at(sheet, *new), *id)
                .map_err(|_| Error::Limit)?;
            let mut record = (**self
                .persistent
                .records
                .get(id)
                .ok_or(Error::InvalidTarget)?)
            .clone();
            record.position = Some(*new);
            data.record(*id, record)?;
        }
        let style_changed = if options.move_formatting {
            let (changed, used) = data.styles.permute_rows(
                sheet,
                r,
                &permutation,
                self.limits.styles,
                structure::MAX_WORK - work.used,
                cancel,
            )?;
            work.add(used)?;
            changed
        } else {
            false
        };
        if placements.is_empty() && !style_changed {
            return Ok(unchanged(work.used));
        }
        if data.stats().estimated_owned_bytes > self.limits.bytes {
            return Err(Error::Limit);
        }
        let next = self.state.checked_add(1).ok_or(Error::Exhausted)?;
        let mut core = self.core.stage();
        let mut batch = core
            .begin_batch(placements.len() * 2, placements.len() * 128)
            .map_err(graph_error)?;
        let space = selection::SpaceId(*sheet.uuid().as_bytes());
        for (_, old, _) in &placements {
            work.add(1)?;
            batch
                .place(space, old.row, old.column, None)
                .map_err(graph_error)?;
        }
        for (id, _, new) in &placements {
            work.add(1)?;
            batch
                .place(
                    space,
                    new.row,
                    new.column,
                    Some(
                        self.persistent
                            .bindings
                            .get(id)
                            .ok_or(Error::InvalidTarget)?
                            .runtime,
                    ),
                )
                .map_err(graph_error)?;
        }
        if !batch.is_empty() {
            core.apply_batch(batch, cancel).map_err(graph_error)?;
        }
        work.add(0)?;
        self.core = core;
        self.persistent = data.freeze();
        self.state = next;
        #[cfg(feature = "history-experiment")]
        {
            self.history_content = Uuid::new_v4();
        }
        Ok(Outcome {
            expected: self.expected(),
            changed: true,
            moved_rows,
            moved_cells: placements.len(),
            work: work.used,
        })
    }
}
