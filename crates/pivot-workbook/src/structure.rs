//! Whole-axis structural transformation; no row lineage or history coordinator.
use super::*;
use pivot_core::model::{integration::CancellationToken, selection};
pub const MAX_WORK: usize = 33_554_432;
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum AxisKind {
    Rows,
    Columns,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Action {
    Insert,
    Delete,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Change {
    pub axis: AxisKind,
    pub action: Action,
    pub start: u32,
    pub count: u32,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Outcome {
    pub expected: Expected,
    pub changed: bool,
    pub formatting_clipped: bool,
    pub moved: usize,
    pub retired: usize,
    pub work: usize,
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct FaultOrigin {
    pub kind: u8,
    pub worksheet: WorksheetId,
    pub target: Uuid,
    pub node: Uuid,
    pub first: Position,
    pub last: Position,
}
impl FaultOrigin {
    pub fn encode(&self) -> String {
        let mut bytes = vec![1, self.kind];
        bytes.extend(self.worksheet.uuid().as_bytes());
        bytes.extend(self.target.as_bytes());
        bytes.extend(self.node.as_bytes());
        for n in [
            self.first.row,
            self.first.column,
            self.last.row,
            self.last.column,
        ] {
            bytes.extend(n.to_le_bytes());
        }
        bytes.iter().map(|b| format!("{b:02x}")).collect()
    }
    pub fn decode(text: &str) -> Result<Option<Self>> {
        if text.is_empty() {
            return Ok(None);
        }
        if text.len() != 132 || !text.bytes().all(|b| b.is_ascii_hexdigit()) {
            return Err(Error::InvalidTarget);
        }
        let bytes = (0..text.len())
            .step_by(2)
            .map(|i| u8::from_str_radix(&text[i..i + 2], 16).map_err(|_| Error::InvalidTarget))
            .collect::<Result<Vec<_>>>()?;
        if bytes[0] != 1 || !(1..=4).contains(&bytes[1]) {
            return Err(Error::InvalidTarget);
        }
        let first = Position::new(
            u32::from_le_bytes(bytes[50..54].try_into().unwrap()),
            u32::from_le_bytes(bytes[54..58].try_into().unwrap()),
        );
        let last = Position::new(
            u32::from_le_bytes(bytes[58..62].try_into().unwrap()),
            u32::from_le_bytes(bytes[62..66].try_into().unwrap()),
        );
        let origin = Self {
            kind: bytes[1],
            worksheet: WorksheetId::from_uuid(Uuid::from_slice(&bytes[2..18]).unwrap()),
            target: Uuid::from_slice(&bytes[18..34]).unwrap(),
            node: Uuid::from_slice(&bytes[34..50]).unwrap(),
            first,
            last,
        };
        if origin.worksheet.uuid().is_nil()
            || origin.target.is_nil()
            || first.row == 0
            || first.column == 0
            || last.row < first.row
            || last.column < first.column
            || last.row > MAX_ROWS
            || last.column > MAX_COLUMNS
            || (origin.kind % 2 == 1 && (first != last || origin.node.is_nil()))
            || (origin.kind % 2 == 0 && !origin.node.is_nil())
        {
            return Err(Error::InvalidTarget);
        }
        Ok(Some(origin))
    }
    pub fn transferred(mut self) -> Self {
        if self.kind <= 2 {
            self.kind += 2;
        }
        self
    }
}
pub(crate) fn fault_op(origin: FaultOrigin) -> pivot_core::typed::Op {
    pivot_core::typed::Op::ReferenceDeleted(origin.encode())
}
pub fn origin_of(token: &Token) -> Result<Option<FaultOrigin>> {
    if let Token::Typed(pivot_core::typed::Op::ReferenceDeleted(origin), 0) = token {
        FaultOrigin::decode(origin)
    } else {
        Ok(None)
    }
}
pub(crate) fn attach_faults(
    tokens: &mut [Token],
    origins: &[String],
    transferred: bool,
) -> Result<()> {
    let mut origins = origins.iter();
    for token in tokens {
        if let Token::Typed(pivot_core::typed::Op::ReferenceDeleted(value), 0) = token {
            let origin = origins.next().ok_or(Error::InvalidTarget)?;
            *value = if transferred {
                FaultOrigin::decode(origin)?.map_or_else(String::new, |o| o.transferred().encode())
            } else {
                FaultOrigin::decode(origin)?;
                origin.clone()
            };
        }
    }
    if origins.next().is_some() {
        return Err(Error::InvalidTarget);
    }
    Ok(())
}
/// Source is provenance; these are the explicitly allowed structural differences.
pub fn structural_syntax(syntax: &Token, bound: &Token, previous: Option<&Token>) -> Result<bool> {
    let origin = origin_of(bound)?;
    Ok(match syntax {
        Token::At(_) => origin.is_some_and(|o| o.kind == 1),
        Token::SumAt { .. } => origin.is_some_and(|o| o.kind == 2),
        Token::Typed(pivot_core::typed::Op::ReferenceDeleted(_), 0) => matches!(
            bound,
            Token::Typed(pivot_core::typed::Op::ReferenceDeleted(_), 0)
        ),
        Token::Control(expression::AggregateControl::Range) => {
            *bound == Token::Control(expression::AggregateControl::Value)
                && previous
                    .is_some_and(|t| origin_of(t).ok().flatten().is_some_and(|o| o.kind == 2))
        }
        _ => false,
    })
}
pub(crate) fn literal_fault_spans(source: &str) -> Vec<std::ops::Range<usize>> {
    let bytes = source.as_bytes();
    let mut spans = Vec::new();
    let mut i = 0;
    let mut quote = None;
    while i < bytes.len() {
        let b = bytes[i];
        if let Some(q) = quote {
            if b == q {
                if bytes.get(i + 1) == Some(&q) {
                    i += 2;
                    continue;
                }
                quote = None;
            }
            i += 1;
            continue;
        }
        if b == b'\'' || b == b'"' {
            quote = Some(b);
            i += 1;
            continue;
        }
        if bytes
            .get(i..i + 5)
            .is_some_and(|s| s.eq_ignore_ascii_case(b"#REF!"))
        {
            spans.push(i..i + 5);
            i += 5;
        } else {
            i += 1;
        }
    }
    spans
}
pub(crate) fn render(
    source: &str,
    tokens: &[Token],
    data: &persistent::Data,
    sheets: &[SheetInfo],
    limit: usize,
) -> Result<(String, Vec<String>)> {
    let parsed = expression::parse_spreadsheet(source);
    if parsed.is_err() {
        return Ok((
            sheets::render_source(source, tokens, sheets, limit)?,
            Vec::new(),
        ));
    }
    let (syntax, references, diagnostic) =
        Workbook::lower_text_bounds(MAX_ROWS, MAX_COLUMNS, &parsed);
    if diagnostic.is_some() || syntax.len() != tokens.len() {
        return Err(Error::InvalidTarget);
    }
    let lexical = clipboard::references(source)?.0;
    let mut scalar = references.into_iter();
    let mut literal = literal_fault_spans(source).into_iter();
    let mut qualifier = None;
    let mut replacements: Vec<(std::ops::Range<usize>, String, Option<String>)> = Vec::new();
    for (syntax, bound) in syntax.iter().zip(tokens) {
        if let Token::Qualifier { worksheet, span } = bound {
            qualifier = Some((*worksheet, span.clone()));
            continue;
        }
        let fault = if let Token::Typed(pivot_core::typed::Op::ReferenceDeleted(origin), 0) = bound
        {
            Some(origin.clone())
        } else {
            None
        };
        let span = match syntax {
            Token::At(_) => Some(scalar.next().ok_or(Error::InvalidTarget)?.0),
            Token::SumAt { span, .. } => Some(span.clone()),
            Token::Typed(pivot_core::typed::Op::ReferenceDeleted(_), 0) => {
                Some(literal.next().ok_or(Error::InvalidTarget)?)
            }
            _ => None,
        };
        if let Some(origin) = fault {
            let mut span = span.ok_or(Error::InvalidTarget)?;
            if let Some((_, q)) = qualifier.take() {
                span.start = q.start;
            }
            replacements.push((span, "#REF!".into(), Some(origin)));
            continue;
        }
        if let Some((sheet, span)) = qualifier.take() {
            let name = &sheets
                .iter()
                .find(|s| s.id == sheet)
                .ok_or(Error::InvalidTarget)?
                .name;
            replacements.push((span, sheets::quoted_name(name), None));
        }
        match (syntax, bound, span) {
            (Token::At(_), Token::Cell(target), Some(span)) => {
                let p = data.records.get(&target.cell).and_then(|r| r.position);
                let text = if let Some(p) = p {
                    clipboard::address(
                        p,
                        lexical
                            .iter()
                            .find(|r| r.span == span)
                            .ok_or(Error::InvalidTarget)?,
                    )
                } else {
                    "#REF!".into()
                };
                replacements.push((span, text, None));
            }
            (Token::SumAt { .. }, Token::Sum { first, last, .. }, Some(span)) => {
                let refs: Vec<_> = lexical
                    .iter()
                    .filter(|r| r.span.start >= span.start && r.span.end <= span.end)
                    .collect();
                if refs.len() != 2 {
                    return Err(Error::InvalidTarget);
                }
                replacements.push((
                    refs[0].span.clone(),
                    clipboard::address(*first, refs[0]),
                    None,
                ));
                replacements.push((
                    refs[1].span.clone(),
                    clipboard::address(*last, refs[1]),
                    None,
                ));
            }
            _ => (),
        }
    }
    replacements.sort_by_key(|r| r.0.start);
    let mut at = 0;
    let mut text = String::new();
    let mut faults = Vec::new();
    for (span, value, fault) in replacements {
        if span.start < at {
            return Err(Error::InvalidTarget);
        }
        text.push_str(source.get(at..span.start).ok_or(Error::InvalidTarget)?);
        text.push_str(&value);
        if let Some(origin) = fault {
            faults.push(origin);
        }
        at = span.end;
        if text.len() > limit {
            return Err(Error::Limit);
        }
    }
    text.push_str(source.get(at..).ok_or(Error::InvalidTarget)?);
    if text.len() > limit {
        return Err(Error::Limit);
    }
    Ok((text, faults))
}
impl Change {
    pub(crate) fn coordinate(self, p: Position) -> u32 {
        match self.axis {
            AxisKind::Rows => p.row,
            AxisKind::Columns => p.column,
        }
    }
    pub(crate) fn limit(self, rows: u32, columns: u32) -> u32 {
        match self.axis {
            AxisKind::Rows => rows,
            AxisKind::Columns => columns,
        }
    }
    pub(crate) fn map(self, x: u32, limit: u32) -> Option<u32> {
        match self.action {
            Action::Insert => {
                if x < self.start {
                    Some(x)
                } else {
                    x.checked_add(self.count).filter(|x| *x <= limit)
                }
            }
            Action::Delete => {
                if x < self.start {
                    Some(x)
                } else if x < self.start + self.count {
                    None
                } else {
                    Some(x - self.count)
                }
            }
        }
    }
    pub(crate) fn position(self, p: Position, limit: u32) -> Option<Position> {
        self.map(self.coordinate(p), limit)
            .map(|x| match self.axis {
                AxisKind::Rows => Position::new(x, p.column),
                AxisKind::Columns => Position::new(p.row, x),
            })
    }
    pub(crate) fn range(self, a: u32, b: u32, limit: u32) -> Option<(u32, u32)> {
        match self.action {
            Action::Insert => {
                let first = if self.start <= a {
                    a.checked_add(self.count)?
                } else {
                    a
                };
                let last = if self.start <= b {
                    b.checked_add(self.count)?.min(limit)
                } else {
                    b
                };
                (first <= last).then_some((first, last))
            }
            Action::Delete => {
                let end = self.start + self.count;
                let first = if a < self.start {
                    a
                } else if a >= end {
                    a - self.count
                } else if b >= end {
                    self.start
                } else {
                    return None;
                };
                let last = if b < self.start {
                    b
                } else if b >= end {
                    b - self.count
                } else if a < self.start {
                    self.start - 1
                } else {
                    return None;
                };
                (first <= last).then_some((first, last))
            }
        }
    }
}
fn cancelled(cancel: &CancellationToken) -> Result<()> {
    if cancel.is_cancelled() {
        Err(Error::Graph("cancelled".into()))
    } else {
        Ok(())
    }
}
fn units(work: &mut usize, n: usize, cancel: &CancellationToken) -> Result<()> {
    cancelled(cancel)?;
    *work = work.checked_add(n).ok_or(Error::Exhausted)?;
    if *work > MAX_WORK {
        Err(Error::Capacity {
            resource: "structural work",
            limit: MAX_WORK,
        })
    } else {
        Ok(())
    }
}
impl Workbook {
    pub fn restructure(
        &mut self,
        sheet: WorksheetId,
        expected: Expected,
        change: Change,
        cancel: &CancellationToken,
    ) -> Result<Outcome> {
        use std::collections::{BTreeMap, BTreeSet};
        self.check(expected)?;
        cancelled(cancel)?;
        if self.is_structured_proof() {
            return Err(Error::InvalidTarget);
        }
        #[cfg(feature = "history-experiment")]
        if self.history_active || self.history_recording.is_some() {
            return Err(Error::InvalidTarget);
        }
        let info = self.sheet_info(sheet)?;
        let limit = change.limit(info.row_count(), info.column_count());
        if change.start == 0
            || change.start > limit
            || change.count == 0
            || change.count > limit - change.start + 1
        {
            return Err(Error::Position);
        }
        let _scratch = self
            .persistent
            .memory
            .reserve(
                self.persistent.records.len().saturating_mul(512)
                    + self.persistent.stats().estimated_owned_bytes
                    + 8192,
            )
            .map_err(|_| Error::Limit)?;
        let mut data = self
            .core
            .stage_resource(&self.persistent)
            .map_err(graph_error)?;
        let mut work = 0;
        let mut updates: BTreeMap<CellId, Record> = BTreeMap::new();
        let mut deleted = BTreeSet::new();
        let mut moved = 0;
        let mut placements = Vec::new();
        for (at, id) in self.persistent.positions.iter() {
            units(&mut work, 1, cancel)?;
            if at.worksheet != sheet {
                continue;
            }
            let old = at.position;
            let new = change.position(old, limit);
            if new == Some(old) {
                continue;
            }
            if new.is_none() && change.action == Action::Insert {
                return Err(Error::Graph(format!(
                    "Cannot insert: positioned cell R{}C{} would leave the worksheet; cleared cells and Blank references retain identities.",
                    old.row, old.column
                )));
            }
            let mut record = (**self
                .persistent
                .records
                .get(id)
                .ok_or(Error::InvalidTarget)?)
            .clone();
            record.position = new;
            if new.is_none() {
                record.set_content(&data.memory, None)?;
                deleted.insert(*id);
            } else {
                moved += 1;
            }
            updates.insert(*id, record);
            placements.push((*id, old, new));
        }
        let mut reframes = BTreeMap::new();
        let mut formulas = BTreeMap::new();
        for (id, record) in self.persistent.records.iter() {
            units(&mut work, 1, cancel)?;
            if record.position.is_none() || deleted.contains(id) {
                continue;
            }
            let mut content = record.content.clone();
            let (source, tokens) = match &mut content {
                Some(Content::Formula { source, tokens }) => (source.as_str(), tokens),
                Some(Content::TextFormula(f)) if f.diagnostic.is_none() => {
                    (f.original_source.as_str(), &mut f.tokens)
                }
                _ => continue,
            };
            let mut changed = false;
            for index in 0..tokens.len() {
                units(&mut work, 1, cancel)?;
                match tokens[index].clone() {
                    Token::Cell(target) if deleted.contains(&target.cell) => {
                        let target_record = self
                            .persistent
                            .records
                            .get(&target.cell)
                            .ok_or(Error::InvalidTarget)?;
                        let p = target_record.position.ok_or(Error::InvalidTarget)?;
                        let origin = FaultOrigin {
                            kind: 1,
                            worksheet: target.worksheet,
                            target: target.cell.uuid(),
                            node: self
                                .persistent
                                .bindings
                                .get(&target.cell)
                                .ok_or(Error::InvalidTarget)?
                                .portable
                                .uuid(),
                            first: p,
                            last: p,
                        };
                        tokens[index] = Token::Typed(fault_op(origin), 0);
                        changed = true;
                    }
                    Token::Sum {
                        kind,
                        id: selection,
                        mut rectangle,
                        mut first,
                        mut last,
                        span,
                    } if rectangle.space.0 == *sheet.uuid().as_bytes() => {
                        let (a, b) = match change.axis {
                            AxisKind::Rows => (rectangle.first_row, rectangle.last_row),
                            AxisKind::Columns => (rectangle.first_column, rectangle.last_column),
                        };
                        if let Some((new_a, new_b)) = change.range(a, b, limit) {
                            if (a, b) == (new_a, new_b) {
                                continue;
                            }
                            match change.axis {
                                AxisKind::Rows => {
                                    rectangle.first_row = new_a;
                                    rectangle.last_row = new_b;
                                    if first.row <= last.row {
                                        first.row = new_a;
                                        last.row = new_b;
                                    } else {
                                        first.row = new_b;
                                        last.row = new_a;
                                    }
                                }
                                AxisKind::Columns => {
                                    rectangle.first_column = new_a;
                                    rectangle.last_column = new_b;
                                    if first.column <= last.column {
                                        first.column = new_a;
                                        last.column = new_b;
                                    } else {
                                        first.column = new_b;
                                        last.column = new_a;
                                    }
                                }
                            }
                            reframes.insert(selection, rectangle);
                            tokens[index] = Token::Sum {
                                kind,
                                id: selection,
                                rectangle,
                                first,
                                last,
                                span,
                            };
                            changed = true;
                        } else {
                            let origin = FaultOrigin {
                                kind: 2,
                                worksheet: sheet,
                                target: Uuid::from_bytes(selection.0),
                                node: Uuid::nil(),
                                first: Position::new(rectangle.first_row, rectangle.first_column),
                                last: Position::new(rectangle.last_row, rectangle.last_column),
                            };
                            tokens[index] = Token::Typed(fault_op(origin), 0);
                            if tokens.get(index + 1)
                                == Some(&Token::Control(expression::AggregateControl::Range))
                            {
                                tokens[index + 1] =
                                    Token::Control(expression::AggregateControl::Value);
                            }
                            changed = true;
                        }
                    }
                    _ => (),
                }
            }
            if changed {
                let mut resolved = Vec::new();
                for token in tokens.iter() {
                    units(&mut work, 1, cancel)?;
                    resolved.push(match token {
                        Token::Qualifier { .. } => continue,
                        Token::Typed(op, n) => ResolvedToken::Typed(op.clone(), *n),
                        Token::Control(c) => ResolvedToken::Control(*c),
                        Token::Decimal(d) => ResolvedToken::Decimal(d.value()),
                        Token::Cell(target) => ResolvedToken::Node(
                            self.persistent
                                .bindings
                                .get(&target.cell)
                                .ok_or(Error::InvalidTarget)?
                                .runtime,
                        ),
                        Token::Sum { kind, id, .. } => {
                            if *kind == expression::AggregateKind::Sum {
                                ResolvedToken::Sum(*id)
                            } else {
                                ResolvedToken::Aggregate(*kind, *id)
                            }
                        }
                        Token::Unary(op) => ResolvedToken::Unary(*op),
                        Token::Binary(op) => ResolvedToken::Binary(*op),
                        _ => return Err(Error::InvalidTarget),
                    });
                }
                formulas.insert(*id, (source.to_owned(), resolved));
                updates
                    .entry(*id)
                    .or_insert_with(|| (**record).clone())
                    .set_content(&data.memory, content)?;
            }
        }
        // Remove every source key before inserting any destination key: collisions
        // between surviving cells are transient, not user-visible occupied errors.
        for (id, old, _) in &placements {
            units(&mut work, 1, cancel)?;
            data.positions
                .remove(&sheets::at(sheet, *old))
                .map_err(|_| Error::Limit)?;
            if deleted.contains(id) {
                data.live -= 1;
            }
        }
        for (id, _, new) in &placements {
            if let Some(p) = new {
                units(&mut work, 1, cancel)?;
                data.positions
                    .insert_private(sheets::at(sheet, *p), *id)
                    .map_err(|_| Error::Limit)?;
            }
        }
        for (id, record) in updates {
            data.record(id, record)?;
        }
        let (style_changed, clipped, style_work) =
            data.styles
                .structural(sheet, change, limit, self.limits.styles, cancel)?;
        units(&mut work, style_work, cancel)?;
        let changed =
            !placements.is_empty() || !formulas.is_empty() || !reframes.is_empty() || style_changed;
        if !changed {
            return Ok(Outcome {
                expected,
                changed: false,
                formatting_clipped: false,
                moved: 0,
                retired: 0,
                work,
            });
        }
        let next = self.state.checked_add(1).ok_or(Error::Exhausted)?;
        let stats = data.stats();
        if stats.estimated_owned_bytes > self.limits.bytes
            || stats.source_bytes > self.limits.total_source_bytes
            || stats.formula_tokens > self.limits.total_formula_tokens
        {
            return Err(Error::Limit);
        }
        let operations = placements.len() + moved + deleted.len() + formulas.len() + reframes.len();
        units(&mut work, operations, cancel)?;
        let bytes = placements.len() * 64
            + moved * 64
            + reframes.len() * 128
            + formulas
                .values()
                .map(|(s, t)| {
                    s.len()
                        + t.iter()
                            .map(|t| {
                                if let ResolvedToken::Typed(op, _) = t {
                                    op.heap_bytes()
                                } else {
                                    0
                                }
                            })
                            .sum::<usize>()
                })
                .sum::<usize>();
        let mut core = self.core.stage();
        let mut batch = core.begin_batch(operations, bytes).map_err(graph_error)?;
        let space = selection::SpaceId(*sheet.uuid().as_bytes());
        for (_, old, _) in &placements {
            batch
                .place(space, old.row, old.column, None)
                .map_err(graph_error)?;
        }
        for (id, rectangle) in reframes {
            batch
                .reframe_selection(id, rectangle)
                .map_err(graph_error)?;
        }
        for (id, (source, tokens)) in formulas {
            batch
                .set_resolved(
                    self.persistent
                        .bindings
                        .get(&id)
                        .ok_or(Error::InvalidTarget)?
                        .runtime,
                    &source,
                    tokens,
                )
                .map_err(graph_error)?;
        }
        for id in &deleted {
            batch
                .retire(
                    self.persistent
                        .bindings
                        .get(id)
                        .ok_or(Error::InvalidTarget)?
                        .runtime,
                )
                .map_err(graph_error)?;
        }
        for (id, _, new) in placements {
            if let Some(p) = new {
                batch
                    .place(
                        space,
                        p.row,
                        p.column,
                        Some(
                            self.persistent
                                .bindings
                                .get(&id)
                                .ok_or(Error::InvalidTarget)?
                                .runtime,
                        ),
                    )
                    .map_err(graph_error)?;
            }
        }
        if !batch.is_empty() {
            core.apply_batch(batch, cancel).map_err(graph_error)?;
        }
        cancelled(cancel)?;
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
            formatting_clipped: clipped,
            moved,
            retired: deleted.len(),
            work,
        })
    }
}
