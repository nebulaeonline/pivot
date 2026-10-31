//! Document-owned typed criteria; derived visibility never changes computational placement.
use super::*;
use pivot_core::model::integration::{
    CancellationToken,
    resources::{Charge, Footprint, Key},
};
use pivot_types::PivotValue;
use std::cmp::Ordering;

pub const MAX_DEFINITION_BYTES: usize = 1024 * 1024;
impl Key for WorksheetId {}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[repr(u8)]
pub enum Operator {
    Equal,
    NotEqual,
    Less,
    LessEqual,
    Greater,
    GreaterEqual,
    Contains,
    StartsWith,
    EndsWith,
    Blank,
    NotBlank,
    Error,
    NotError,
}
impl TryFrom<u8> for Operator {
    type Error = Error;
    fn try_from(code: u8) -> Result<Self> {
        [
            Self::Equal,
            Self::NotEqual,
            Self::Less,
            Self::LessEqual,
            Self::Greater,
            Self::GreaterEqual,
            Self::Contains,
            Self::StartsWith,
            Self::EndsWith,
            Self::Blank,
            Self::NotBlank,
            Self::Error,
            Self::NotError,
        ]
        .get(code as usize)
        .copied()
        .ok_or(Error::InvalidTarget)
    }
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Literal {
    Decimal(PivotDecimal),
    Text(String),
    Boolean(bool),
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Predicate {
    pub operator: Operator,
    pub literal: Option<Literal>,
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Condition {
    pub column: u32,
    pub either: bool,
    pub predicates: Vec<Predicate>,
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Definition {
    pub rectangle: Rectangle,
    pub exclude_header: bool,
    pub needs_review: bool,
    pub conditions: Vec<Condition>,
}
impl Footprint for Definition {
    fn heap_bytes(&self) -> usize {
        self.conditions.capacity() * std::mem::size_of::<Condition>()
            + self
                .conditions
                .iter()
                .map(|c| {
                    c.predicates.capacity() * std::mem::size_of::<Predicate>()
                        + c.predicates
                            .iter()
                            .map(|p| match &p.literal {
                                Some(Literal::Text(s)) => s.capacity(),
                                _ => 0,
                            })
                            .sum::<usize>()
                })
                .sum::<usize>()
    }
}
pub(crate) fn definition_bytes(d: &Definition) -> usize {
    d.storage_bytes()
}
impl Definition {
    /// Stable conservative quota units; allocator spare capacity is charged separately
    /// by Footprint and must not make a valid saved definition fail on reopen.
    pub fn storage_bytes(&self) -> usize {
        256 + self.conditions.len() * 64
            + self
                .conditions
                .iter()
                .map(|c| {
                    c.predicates.len() * 80
                        + c.predicates
                            .iter()
                            .map(|p| match &p.literal {
                                Some(Literal::Text(s)) => s.len(),
                                _ => 0,
                            })
                            .sum::<usize>()
                })
                .sum::<usize>()
    }
    pub fn validate(&self, rows: u32, columns: u32) -> Result<()> {
        let r = self.rectangle;
        if r.first.row == 0
            || r.first.column == 0
            || r.first.row > r.last.row
            || r.first.column > r.last.column
            || r.last.row > rows
            || r.last.column > columns
            || self.conditions.len() > (r.last.column - r.first.column + 1) as usize
        {
            return Err(Error::InvalidTarget);
        }
        if definition_bytes(self) > MAX_DEFINITION_BYTES {
            return Err(Error::Capacity {
                resource: "filter definitions",
                limit: MAX_DEFINITION_BYTES,
            });
        }
        let mut seen = BTreeSet::new();
        for c in &self.conditions {
            if c.column < r.first.column
                || c.column > r.last.column
                || !seen.insert(c.column)
                || c.predicates.is_empty()
                || c.predicates.len() > 2
                || (c.predicates.len() == 1 && c.either)
            {
                return Err(Error::InvalidTarget);
            }
            for p in &c.predicates {
                let code = p.operator as u8;
                match &p.literal {
                    None if code >= Operator::Blank as u8 => (),
                    Some(Literal::Decimal(_)) if code <= Operator::GreaterEqual as u8 => (),
                    Some(Literal::Boolean(_)) if code <= Operator::NotEqual as u8 => (),
                    Some(Literal::Text(s)) if code <= Operator::EndsWith as u8 => {
                        entry::validate_value_text(s, 4096)?
                    }
                    _ => return Err(Error::InvalidTarget),
                }
            }
        }
        Ok(())
    }
}
struct Work<'a> {
    used: usize,
    cancel: &'a CancellationToken,
}
impl Work<'_> {
    fn add(&mut self, n: usize) -> Result<()> {
        if self.cancel.is_cancelled() {
            return Err(Error::Graph("cancelled filter evaluation".into()));
        }
        self.used = self.used.checked_add(n).ok_or(Error::Exhausted)?;
        if self.used > structure::MAX_WORK {
            return Err(Error::Capacity {
                resource: "filter work",
                limit: structure::MAX_WORK,
            });
        }
        Ok(())
    }
}
type Value = std::result::Result<PivotValue, EvaluationError>;
const BLANK: Value = Ok(PivotValue::Blank);
fn ordered(op: Operator, order: Ordering) -> bool {
    match op {
        Operator::Equal => order == Ordering::Equal,
        Operator::NotEqual => order != Ordering::Equal,
        Operator::Less => order == Ordering::Less,
        Operator::LessEqual => order != Ordering::Greater,
        Operator::Greater => order == Ordering::Greater,
        Operator::GreaterEqual => order != Ordering::Less,
        _ => false,
    }
}
fn predicate(p: &Predicate, v: &Value, work: &mut Work<'_>) -> Result<bool> {
    work.add(1)?;
    match p.operator {
        Operator::Blank => return Ok(matches!(v, Ok(PivotValue::Blank))),
        Operator::NotBlank => return Ok(!matches!(v, Ok(PivotValue::Blank))),
        Operator::Error => return Ok(v.is_err()),
        Operator::NotError => return Ok(v.is_ok()),
        _ => (),
    }
    Ok(match (&p.literal, v) {
        (Some(Literal::Decimal(a)), Ok(PivotValue::Decimal(b))) => ordered(p.operator, b.cmp(a)),
        (Some(Literal::Boolean(a)), Ok(PivotValue::Boolean(b))) => ordered(p.operator, b.cmp(a)),
        (Some(Literal::Text(a)), Ok(PivotValue::String(b))) => match p.operator {
            Operator::Contains => {
                if a.is_empty() {
                    true
                } else {
                    let mut found = false;
                    for start in 0..=b.len().saturating_sub(a.len()) {
                        work.add(1)?;
                        if start + a.len() > b.len() {
                            break;
                        }
                        let mut equal = true;
                        for (x, y) in b.as_bytes()[start..start + a.len()]
                            .iter()
                            .zip(a.as_bytes())
                        {
                            work.add(1)?;
                            if x != y {
                                equal = false;
                                break;
                            }
                        }
                        if equal {
                            found = true;
                            break;
                        }
                    }
                    found
                }
            }
            Operator::StartsWith | Operator::EndsWith => {
                work.add(a.len().min(b.len()) + 1)?;
                if p.operator == Operator::StartsWith {
                    b.starts_with(a)
                } else {
                    b.ends_with(a)
                }
            }
            _ => {
                let mut order = Ordering::Equal;
                for (x, y) in b.as_bytes().iter().zip(a.as_bytes()) {
                    work.add(1)?;
                    order = x.cmp(y);
                    if order != Ordering::Equal {
                        break;
                    }
                }
                if order == Ordering::Equal {
                    order = b.len().cmp(&a.len());
                }
                ordered(p.operator, order)
            }
        },
        _ => false,
    })
}
fn condition(c: &Condition, v: &Value, work: &mut Work<'_>) -> Result<bool> {
    let first = predicate(&c.predicates[0], v, work)?;
    if c.predicates.len() == 1 {
        return Ok(first);
    }
    if c.either && first {
        return Ok(true);
    }
    if !c.either && !first {
        return Ok(false);
    }
    predicate(&c.predicates[1], v, work)
}
/// Session-owned charged map. Expected qualifies both values and visibility.
pub struct Visibility {
    pub expected: Expected,
    pub worksheet: WorksheetId,
    pub shown_data_rows: u32,
    pub total_data_rows: u32,
    pub work: usize,
    rows: Vec<u32>,
    _charge: Charge,
}
impl Visibility {
    pub fn rows(&self) -> &[u32] {
        &self.rows
    }
    pub fn hidden_in(&self, first: u32, last: u32) -> Result<u32> {
        if first == 0 || first > last || last > MAX_ROWS {
            return Err(Error::Position);
        }
        let shown =
            self.rows.partition_point(|r| *r <= last) - self.rows.partition_point(|r| *r < first);
        Ok(last - first + 1 - shown as u32)
    }
    pub fn ordinal(&self, row: u32) -> Option<usize> {
        self.rows.binary_search(&row).ok()
    }
}
impl Workbook {
    pub fn filter(&self, sheet: WorksheetId) -> Result<Option<&Definition>> {
        self.sheet_info(sheet)?;
        Ok(self.persistent.filters.get(&sheet).map(|d| &**d))
    }
    pub fn filters(&self) -> impl Iterator<Item = (WorksheetId, &Definition)> {
        self.persistent.filters.iter().map(|(s, d)| (*s, &**d))
    }
    /// Compute a candidate before changing persistent criteria. Transient evaluation failures never dirty a book.
    pub fn set_filter(
        &mut self,
        sheet: WorksheetId,
        expected: Expected,
        definition: Option<&Definition>,
        cancel: &CancellationToken,
    ) -> Result<Expected> {
        self.set_filter_with_visibility(sheet, expected, definition, cancel)
            .map(|(expected, _)| expected)
    }
    pub fn set_filter_with_visibility(
        &mut self,
        sheet: WorksheetId,
        expected: Expected,
        definition: Option<&Definition>,
        cancel: &CancellationToken,
    ) -> Result<(Expected, Option<Visibility>)> {
        self.check(expected)?;
        self.sheet_info(sheet)?;
        if self.is_structured_proof() {
            return Err(Error::InvalidTarget);
        }
        #[cfg(feature = "history-experiment")]
        if self.history_active || self.history_recording.is_some() {
            return Err(Error::InvalidTarget);
        }
        if let Some(d) = definition {
            d.validate(
                self.sheet_info(sheet)?.row_count(),
                self.sheet_info(sheet)?.column_count(),
            )?;
        }
        let _scratch = self
            .persistent
            .memory
            .reserve(definition.map_or(0, Definition::storage_bytes) * 2 + 8192)
            .map_err(|_| Error::Limit)?;
        let mut candidate = definition.cloned();
        let mut visibility = None;
        if let Some(d) = &mut candidate {
            d.validate(
                self.sheet_info(sheet)?.row_count(),
                self.sheet_info(sheet)?.column_count(),
            )?;
            d.conditions.sort_by_key(|c| c.column);
            if !d.needs_review {
                visibility = Some(self.filter_visibility_for(sheet, expected, Some(d), cancel)?);
            }
        }
        if self.filter(sheet)? == candidate.as_ref() {
            return Ok((expected, visibility));
        }
        let total = self
            .filters()
            .filter(|(id, _)| *id != sheet)
            .map(|(_, d)| definition_bytes(d))
            .sum::<usize>()
            + candidate.as_ref().map_or(0, definition_bytes);
        if total > MAX_DEFINITION_BYTES {
            return Err(Error::Capacity {
                resource: "filter definitions",
                limit: MAX_DEFINITION_BYTES,
            });
        }
        let mut data = self
            .core
            .stage_resource(&self.persistent)
            .map_err(graph_error)?;
        match candidate {
            Some(d) => {
                let d = data.memory.alloc(d).map_err(|_| Error::Limit)?;
                data.filters
                    .insert_private(sheet, d)
                    .map_err(|_| Error::Limit)?;
            }
            None => {
                data.filters.remove(&sheet).map_err(|_| Error::Limit)?;
            }
        }
        if data.stats().estimated_owned_bytes > self.limits.bytes {
            return Err(Error::Limit);
        }
        if cancel.is_cancelled() {
            return Err(Error::Graph("cancelled filter definition".into()));
        }
        let next = self.state.checked_add(1).ok_or(Error::Exhausted)?;
        self.persistent = data.freeze();
        self.state = next;
        if let Some(map) = &mut visibility {
            map.expected = self.expected();
        }
        Ok((self.expected(), visibility))
    }
    pub fn preview_filter(
        &self,
        sheet: WorksheetId,
        expected: Expected,
        definition: &Definition,
        cancel: &CancellationToken,
    ) -> Result<Visibility> {
        self.filter_visibility_for(sheet, expected, Some(definition), cancel)
    }
    pub fn filter_visibility(
        &self,
        sheet: WorksheetId,
        expected: Expected,
        cancel: &CancellationToken,
    ) -> Result<Visibility> {
        self.filter_visibility_for(sheet, expected, self.filter(sheet)?, cancel)
    }
    fn filter_visibility_for(
        &self,
        sheet: WorksheetId,
        expected: Expected,
        definition: Option<&Definition>,
        cancel: &CancellationToken,
    ) -> Result<Visibility> {
        self.check(expected)?;
        let capture = self.capture_sheet(sheet, expected)?;
        capture.filter_visibility_for(definition, cancel)
    }
}
impl CalculationCapture {
    pub fn filter_definition(&self) -> Option<&Definition> {
        self.pinned.filters.get(&self.worksheet).map(|d| &**d)
    }
    pub fn filter_visibility(&self, cancel: &CancellationToken) -> Result<Visibility> {
        self.filter_visibility_for(self.filter_definition(), cancel)
    }
    fn filter_visibility_for(
        &self,
        definition: Option<&Definition>,
        cancel: &CancellationToken,
    ) -> Result<Visibility> {
        let expected = self.expected;
        let sheet = self.worksheet;
        let info = self
            .pinned
            .sheets
            .as_ref()
            .and_then(|s| s.0.iter().find(|s| s.id == sheet))
            .ok_or(Error::InvalidTarget)?;
        let extent = info.rows.length;
        let charge = self
            .pinned
            .memory
            .reserve(extent as usize * 4 + 512)
            .map_err(|_| Error::Limit)?;
        let _scratch = self
            .pinned
            .memory
            .reserve(self.pinned.positions.len() * 128 + extent as usize * 8 + 32768)
            .map_err(|_| Error::Limit)?;
        let mut work = Work { used: 0, cancel };
        work.add(1)?;
        let mut rows = Vec::with_capacity(extent as usize);
        let Some(d) = definition.filter(|d| !d.needs_review) else {
            for row in 1..=extent {
                work.add(1)?;
                rows.push(row);
            }
            return Ok(Visibility {
                expected,
                worksheet: sheet,
                shown_data_rows: 0,
                total_data_rows: 0,
                work: work.used,
                rows,
                _charge: charge,
            });
        };
        d.validate(extent, info.columns.length)?;
        let first = d.rectangle.first.row + u32::from(d.exclude_header);
        let last = d.rectangle.last.row;
        let mut levels = vec![None; MAX_COLUMNS as usize + 1];
        let mut defaults = Vec::with_capacity(d.conditions.len());
        let mut failures = 0isize;
        for (i, c) in d.conditions.iter().enumerate() {
            levels[c.column as usize] = Some(i);
            let yes = condition(c, &BLANK, &mut work)?;
            defaults.push(yes);
            failures += isize::from(!yes);
        }
        let mut keys = Vec::new();
        for (at, id) in self.pinned.positions.iter() {
            work.add(1)?;
            if at.worksheet == sheet
                && at.position.row >= first
                && at.position.row <= last
                && let Some(level) = levels[at.position.column as usize]
            {
                keys.push((
                    at.position.row,
                    level,
                    self.pinned
                        .bindings
                        .get(id)
                        .ok_or(Error::InvalidTarget)?
                        .runtime,
                ));
            }
        }
        work.add(sorting::sort_move_units(keys.len()))?;
        sorting::sort_by(&mut keys, |a, b| {
            work.add(1)?;
            Ok((a.0, a.1).cmp(&(b.0, b.1)))
        })?;
        let nodes = keys.iter().map(|k| k.2).collect::<Vec<_>>();
        let calculation = self
            .local_core
            .evaluate_values(&nodes, None, cancel)
            .map_err(graph_error)?;
        let mut index = 0;
        let mut shown = 0;
        for row in 1..=extent {
            work.add(1)?;
            if row < first || row > last {
                rows.push(row);
                continue;
            }
            let mut failed = failures;
            while index < keys.len() && keys[index].0 == row {
                let (_, level, node) = keys[index];
                work.add(
                    1 + (usize::BITS - calculation.values().len().max(1).leading_zeros()) as usize,
                )?;
                let value = calculation
                    .values()
                    .get(&node)
                    .ok_or(Error::InvalidTarget)?;
                if matches!(
                    value,
                    Err(EvaluationError::InvalidExpression { .. }
                        | EvaluationError::UnknownNode(_)
                        | EvaluationError::MissingCapturedContext { .. })
                ) {
                    return Err(Error::InvalidDraft);
                }
                if !matches!(
                    value,
                    Ok(PivotValue::Blank
                        | PivotValue::Decimal(_)
                        | PivotValue::String(_)
                        | PivotValue::Boolean(_))
                        | Err(_)
                ) {
                    return Err(Error::InvalidTarget);
                }
                let yes = condition(&d.conditions[level], value, &mut work)?;
                failed += isize::from(!yes) - isize::from(!defaults[level]);
                index += 1;
            }
            if failed == 0 {
                rows.push(row);
                shown += 1;
            }
        }
        Ok(Visibility {
            expected,
            worksheet: sheet,
            shown_data_rows: shown,
            total_data_rows: last.saturating_add(1).saturating_sub(first),
            work: work.used,
            rows,
            _charge: charge,
        })
    }
}
impl persistent::Data {
    pub(super) fn review_filter_after_structure(&mut self, sheet: WorksheetId) -> Result<()> {
        if let Some(old) = self.filters.get(&sheet)
            && !old.needs_review
        {
            let mut d = (**old).clone();
            d.needs_review = true;
            let shared = self.memory.alloc(d).map_err(|_| Error::Limit)?;
            self.filters
                .insert_private(sheet, shared)
                .map_err(|_| Error::Limit)?;
        }
        Ok(())
    }
}
