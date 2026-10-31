//! Ordinary worksheet visibility; no projection records or computational predicates.
use super::*;
use pivot_workbook::filtering::{Condition, Definition, Literal, Operator, Predicate, Visibility};
#[derive(Default)]
pub(super) struct Presentation {
    map: Option<Visibility>,
    generation: String,
    failures: BTreeMap<WorksheetId, String>,
    session: Option<Uuid>,
    job: Option<FilterJob>,
    status_charge: Option<pivot_core::model::integration::resources::Charge>,
}
pub(super) struct Confirmation {
    ticket: String,
    operation: Vec<u8>,
    expected: pivot_workbook::Expected,
    generation: String,
    hidden: u32,
    _charge: pivot_core::model::integration::resources::Charge,
}
fn canonical(q: &wire::Request) -> Vec<u8> {
    let mut q = q.clone();
    q.id = 0;
    q.version = 0;
    q.filter_confirmation.clear();
    q.encode_to_vec()
}
pub(super) fn is_bulk(op: wire::request::Operation) -> bool {
    use wire::request::Operation as O;
    matches!(
        op,
        O::Apply
            | O::CopyBlock
            | O::PasteBlock
            | O::Fill
            | O::Format
            | O::Sort
            | O::Structure
            | O::ExportCsv
    )
}
fn footprints(
    book: &Workbook,
    sheet: WorksheetId,
    q: &wire::Request,
) -> Result<Vec<Rectangle>, String> {
    use wire::request::Operation as O;
    let op = O::try_from(q.operation).map_err(|_| "unknown operation")?;
    let full = || Rectangle {
        first: Position::new(1, 1),
        last: Position::new(65536, 1024),
    };
    let rectangle = || {
        Ok::<_, String>(Rectangle {
            first: pos(q.first.as_ref())?,
            last: pos(q.last.as_ref())?,
        })
    };
    let out = match op {
        O::Apply => {
            if q.edits.is_empty() || q.edits.len() > 256 {
                return Err("1..256 edits required".into());
            }
            let mut out = Vec::new();
            for edit in &q.edits {
                let at = pos(edit.at.as_ref())?;
                out.push(Rectangle {
                    first: at,
                    last: at,
                });
                if let Some(wire::edit::Action::MoveTo(to)) = &edit.action {
                    let to = pos(Some(to))?;
                    out.push(Rectangle {
                        first: to,
                        last: to,
                    });
                }
            }
            out
        }
        O::PasteBlock => vec![
            book.paste_extent(
                sheet,
                book.expected(),
                rectangle()?,
                &q.clipboard,
                q.native_clipboard,
            )
            .map_err(|e| e.to_string())?,
        ],
        O::Format if q.worksheet_defaults => vec![full()],
        O::ExportCsv if q.csv_whole_sheet => book
            .csv_content_bounds(sheet)
            .map_err(|e| e.to_string())?
            .into_iter()
            .collect(),
        O::Structure => {
            if q.structure_start == 0 || q.structure_count == 0 {
                return Err("structure band required".into());
            }
            let end = q
                .structure_start
                .checked_add(q.structure_count - 1)
                .ok_or("structure band overflow")?;
            if end > if q.structure_columns { 1024 } else { 65536 } {
                return Err("structure band exceeds worksheet".into());
            }
            if q.structure_columns {
                vec![Rectangle {
                    first: Position::new(1, q.structure_start),
                    last: Position::new(65536, 1024),
                }]
            } else {
                vec![Rectangle {
                    first: Position::new(q.structure_start, 1),
                    last: Position::new(65536, 1024),
                }]
            }
        }
        O::Sort => {
            let mut r = rectangle()?;
            if q.sort_exclude_header {
                r.first.row = r.first.row.checked_add(1).ok_or("header overflow")?;
            }
            if r.first.row > r.last.row {
                vec![]
            } else {
                vec![r]
            }
        }
        O::Fill | O::Format | O::CopyBlock | O::ExportCsv => vec![rectangle()?],
        _ => return Err("not a filtered bulk operation".into()),
    };
    for r in &out {
        if r.first.row > r.last.row || r.first.column > r.last.column {
            return Err("invalid operation rectangle".into());
        }
        book.axes_at_sheet(sheet, r.first)
            .map_err(|e| e.to_string())?;
        book.axes_at_sheet(sheet, r.last)
            .map_err(|e| e.to_string())?;
    }
    Ok(out)
}
impl Session {
    fn hidden_footprint(&mut self, q: &wire::Request) -> Result<(Vec<Rectangle>, u32), String> {
        let book = self.book.as_ref().ok_or("no workbook")?;
        let sheet = WorksheetId::from_uuid(
            Uuid::parse_str(&q.worksheet).map_err(|_| "worksheet ID required")?,
        );
        let rects = footprints(book, sheet, q)?;
        self.filter.requalify(book, sheet);
        let Some(definition) = book
            .filter(sheet)
            .map_err(|e| e.to_string())?
            .filter(|d| !d.needs_review)
        else {
            return Ok((rects, 0));
        };
        let first = definition.rectangle.first.row + u32::from(definition.exclude_header);
        if rects
            .iter()
            .all(|r| r.last.row < first || r.first.row > definition.rectangle.last.row)
        {
            return Ok((rects, 0));
        }
        if self.filter.failures.contains_key(&sheet) {
            return Err("Filter is suspended. Reapply or Clear Filter before bulk actions.".into());
        }
        let map = self
            .filter
            .map
            .as_ref()
            .filter(|m| m.worksheet == sheet && m.expected == book.expected())
            .ok_or("Filter refresh is pending. Wait or Clear Filter before bulk actions.")?;
        let mut bands = rects
            .iter()
            .map(|r| (r.first.row, r.last.row))
            .collect::<Vec<_>>();
        bands.sort_unstable();
        let mut union: Vec<(u32, u32)> = Vec::new();
        for (first, last) in bands {
            if let Some(end) = union.last_mut().filter(|b| first <= b.1.saturating_add(1)) {
                end.1 = end.1.max(last);
            } else {
                union.push((first, last));
            }
        }
        let mut hidden = 0;
        for (first, last) in union {
            hidden += map.hidden_in(first, last).map_err(|e| e.to_string())?;
        }
        Ok((rects, hidden))
    }
    pub(super) fn prepare_filtered_operation(
        &mut self,
        q: &wire::Request,
        r: &mut wire::Response,
    ) -> Result<(), String> {
        if q.revision != self.revision {
            r.conflict = true;
            return Err("expected-state conflict".into());
        }
        self.filter_confirmation = None;
        let _scratch = self
            .runtime
            .as_ref()
            .ok_or("closed")?
            .memory()
            .reserve(q.filter_operation.len() * 4 + MAX_FRAME)
            .map_err(|_| "operation preview admission")?;
        let operation =
            wire::Request::decode(q.filter_operation.as_slice()).map_err(|e| e.to_string())?;
        if operation.revision != self.revision
            || operation.worksheet != q.worksheet
            || !is_bulk(
                wire::request::Operation::try_from(operation.operation).map_err(|_| "operation")?,
            )
        {
            return Err("exact current operation required".into());
        }
        let (rects, hidden) = self.hidden_footprint(&operation)?;
        r.filter_hidden_rows = hidden;
        r.filter_generation = self.filter.generation.clone();
        r.worksheet = q.worksheet.clone();
        r.filter_footprints = rects
            .iter()
            .map(|r| wire::FilterFootprint {
                first: Some(location(r.first)),
                last: Some(location(r.last)),
            })
            .collect();
        if hidden != 0 {
            let operation = canonical(&operation);
            let charge = self
                .runtime
                .as_ref()
                .ok_or("closed")?
                .memory()
                .reserve(operation.len() * 3 + 4096)
                .map_err(|_| "operation admission")?;
            let ticket = Uuid::new_v4().to_string();
            r.filter_confirmation = ticket.clone();
            self.filter_confirmation = Some(Confirmation {
                ticket,
                operation,
                expected: self.book.as_ref().unwrap().expected(),
                generation: self.filter.generation.clone(),
                hidden,
                _charge: charge,
            });
        }
        Ok(())
    }
    pub(super) fn guard_filtered_operation(&mut self, q: &wire::Request) -> Result<(), String> {
        if q.filter_confirmation.is_empty()
            && self
                .book
                .as_ref()
                .is_some_and(|b| b.filters().next().is_none())
        {
            return Ok(());
        }
        let (_, hidden) = self.hidden_footprint(q)?;
        if !q.filter_confirmation.is_empty() {
            let confirmed = self
                .filter_confirmation
                .take()
                .ok_or("confirmation missing or already consumed")?;
            if confirmed.ticket != q.filter_confirmation
                || confirmed.operation != canonical(q)
                || confirmed.expected != self.book.as_ref().ok_or("no workbook")?.expected()
                || confirmed.generation != self.filter.generation
                || confirmed.hidden != hidden
            {
                return Err(
                    "Hidden-row confirmation is stale; no operation was dispatched.".into(),
                );
            }
        } else if hidden != 0 {
            return Err(format!(
                "Operation includes {hidden} filtered-out rows; exact-operation confirmation required"
            ));
        }
        Ok(())
    }
}
fn bounded_error(error: impl std::fmt::Display) -> String {
    let mut text = error.to_string();
    let mut end = text.len().min(4096);
    while !text.is_char_boundary(end) {
        end -= 1;
    }
    text.truncate(end);
    text.shrink_to_fit();
    text
}
struct FilterJob {
    expected: pivot_workbook::Expected,
    sheet: WorksheetId,
    cancel: pivot_core::model::integration::CancellationToken,
    receive: std::sync::mpsc::Receiver<Result<Visibility, String>>,
    thread: Option<std::thread::JoinHandle<()>>,
}
impl Drop for FilterJob {
    fn drop(&mut self) {
        self.cancel.cancel();
        if let Some(thread) = self.thread.take() {
            let _ = thread.join();
        }
    }
}
impl Presentation {
    pub(super) fn requalify(&mut self, book: &Workbook, sheet: WorksheetId) {
        if book
            .filter(sheet)
            .ok()
            .flatten()
            .is_some_and(|d| !d.needs_review)
            && let Some(map) = &mut self.map
        {
            let expected = book.expected();
            if map.worksheet == sheet
                && map.expected.session == expected.session
                && map.expected.graph == expected.graph
            {
                map.expected = expected;
            }
        }
    }
    fn budget_status(
        &mut self,
        memory: &pivot_core::model::integration::resources::Memory,
    ) -> Result<(), String> {
        if self.status_charge.is_none() {
            self.status_charge = Some(
                memory
                    .reserve(16 * (4096 + 256) + 4096)
                    .map_err(|_| "filter status admission")?,
            );
        }
        Ok(())
    }
    fn reset(&mut self) {
        self.map = None;
        self.generation = Uuid::new_v4().to_string();
    }
    fn ensure(&mut self, book: &Workbook, sheet: WorksheetId, retry: bool) -> Result<(), String> {
        let expected = book.expected();
        if self.session != Some(expected.session) {
            self.job = None;
            self.reset();
            self.failures.clear();
            self.session = Some(expected.session);
        }
        if retry {
            self.failures.remove(&sheet);
            self.job = None;
            self.reset();
        }
        if self.failures.contains_key(&sheet) {
            return Ok(());
        }
        if book
            .filter(sheet)
            .map_err(|e| e.to_string())?
            .is_none_or(|d| d.needs_review)
        {
            if self.job.is_some() || self.map.is_some() {
                self.job = None;
                self.reset();
            }
            return Ok(());
        }
        if let Some(job) = &self.job {
            match job.receive.try_recv() {
                Ok(result) => {
                    let current = job.expected.session == expected.session
                        && job.expected.graph == expected.graph
                        && job.sheet == sheet;
                    self.job = None;
                    if current {
                        match result {
                            Ok(mut map) => {
                                map.expected = expected;
                                self.map = Some(map);
                                self.generation = Uuid::new_v4().to_string();
                            }
                            Err(error) => {
                                self.failures.insert(sheet, error);
                                self.generation = Uuid::new_v4().to_string();
                            }
                        }
                    }
                }
                Err(std::sync::mpsc::TryRecvError::Disconnected) => {
                    self.job = None;
                    self.failures
                        .insert(sheet, "Filter worker ended without a result".into());
                }
                Err(std::sync::mpsc::TryRecvError::Empty) => {
                    if job.expected.session != expected.session
                        || job.expected.graph != expected.graph
                        || job.sheet != sheet
                    {
                        job.cancel.cancel();
                        if self.map.is_some() {
                            self.reset();
                        }
                    }
                    return Ok(());
                }
            }
        }
        if self.failures.contains_key(&sheet) {
            return Ok(());
        }
        if let Some(map) = &mut self.map
            && map.worksheet == sheet
            && map.expected.session == expected.session
            && map.expected.graph == expected.graph
        {
            map.expected = expected;
            return Ok(());
        }
        self.reset();
        let capture = match book.capture_sheet(sheet, expected) {
            Ok(capture) => capture,
            Err(e) => {
                self.failures.insert(sheet, bounded_error(e));
                return Ok(());
            }
        };
        let cancel = pivot_core::model::integration::CancellationToken::default();
        let worker_cancel = cancel.clone();
        let (send, receive) = std::sync::mpsc::sync_channel(1);
        let thread = match std::thread::Builder::new()
            .name("pivot-filter".into())
            .spawn(move || {
                let result = capture
                    .filter_visibility(&worker_cancel)
                    .map_err(bounded_error);
                let _ = send.send(result);
            }) {
            Ok(thread) => thread,
            Err(error) => {
                self.failures.insert(sheet, bounded_error(error));
                return Ok(());
            }
        };
        self.job = Some(FilterJob {
            expected,
            sheet,
            cancel,
            receive,
            thread: Some(thread),
        });
        Ok(())
    }
    fn rows(&self, book: &Workbook, sheet: WorksheetId) -> Result<Vec<u32>, String> {
        if !self.failures.contains_key(&sheet)
            && let Some(map) = &self.map
            && map.worksheet == sheet
            && map.expected == book.expected()
        {
            return Ok(map.rows().to_vec());
        }
        let n = book
            .sheets()
            .iter()
            .find(|s| s.id == sheet)
            .ok_or("unknown worksheet")?
            .row_count();
        Ok((1..=n).collect())
    }
    pub(super) fn describe(
        &self,
        book: &Workbook,
        sheet: WorksheetId,
        r: &mut wire::Response,
    ) -> Result<(), String> {
        let definition = book.filter(sheet).map_err(|e| e.to_string())?;
        r.filter_summary = definition.map(|d| wire::FilterSummary {
            first: Some(location(d.rectangle.first)),
            last: Some(location(d.rectangle.last)),
            exclude_header: d.exclude_header,
            needs_review: d.needs_review,
            columns: d.conditions.iter().map(|c| c.column).collect(),
        });
        r.filter_generation = self.generation.clone();
        r.filter_status = if definition.is_none() {
            0
        } else if definition.is_some_and(|d| d.needs_review) {
            2
        } else if let Some(e) = self.failures.get(&sheet) {
            r.filter_error = e.clone();
            4
        } else if self
            .map
            .as_ref()
            .is_some_and(|m| m.expected == book.expected() && m.worksheet == sheet)
        {
            1
        } else {
            3
        };
        if let Some(map) = self
            .map
            .as_ref()
            .filter(|m| m.expected == book.expected() && m.worksheet == sheet)
        {
            r.filter_shown_data_rows = map.shown_data_rows;
            r.filter_total_data_rows = map.total_data_rows;
        }
        Ok(())
    }
}
pub(super) fn encode(d: &Definition) -> wire::FilterDefinition {
    wire::FilterDefinition {
        first: Some(location(d.rectangle.first)),
        last: Some(location(d.rectangle.last)),
        exclude_header: d.exclude_header,
        needs_review: d.needs_review,
        conditions: d
            .conditions
            .iter()
            .map(|c| wire::FilterCondition {
                column: c.column,
                either: c.either,
                predicates: c
                    .predicates
                    .iter()
                    .map(|p| wire::FilterPredicate {
                        operator: p.operator as u32,
                        literal: p.literal.as_ref().map(|v| match v {
                            Literal::Decimal(n) => {
                                wire::filter_predicate::Literal::Decimal(decimal(*n))
                            }
                            Literal::Text(s) => wire::filter_predicate::Literal::Text(s.clone()),
                            Literal::Boolean(b) => wire::filter_predicate::Literal::Boolean(*b),
                        }),
                    })
                    .collect(),
            })
            .collect(),
    }
}
fn decode(d: &wire::FilterDefinition) -> Result<Definition, String> {
    if d.conditions.len() > 1024 {
        return Err("too many filter columns".into());
    }
    Ok(Definition {
        rectangle: Rectangle {
            first: pos(d.first.as_ref())?,
            last: pos(d.last.as_ref())?,
        },
        exclude_header: d.exclude_header,
        needs_review: d.needs_review,
        conditions: d
            .conditions
            .iter()
            .map(|c| {
                if c.predicates.len() > 2 {
                    return Err("too many predicates".into());
                }
                Ok(Condition {
                    column: c.column,
                    either: c.either,
                    predicates: c
                        .predicates
                        .iter()
                        .map(|p| {
                            Ok(Predicate {
                                operator: Operator::try_from(
                                    u8::try_from(p.operator).map_err(|_| "filter operator")?,
                                )
                                .map_err(|e| e.to_string())?,
                                literal: match &p.literal {
                                    None => None,
                                    Some(wire::filter_predicate::Literal::DecimalText(s)) => {
                                        match pivot_workbook::entry::classify(
                                            Position::new(1, 1),
                                            s,
                                            4096,
                                        )
                                        .map_err(|e| e.to_string())?
                                        {
                                            Edit::SetDecimal { value, .. } => {
                                                Some(Literal::Decimal(value.value()))
                                            }
                                            _ => {
                                                return Err(
                                                    "An exact Decimal96 literal is required".into(),
                                                );
                                            }
                                        }
                                    }
                                    Some(wire::filter_predicate::Literal::Text(s)) => {
                                        Some(Literal::Text(s.clone()))
                                    }
                                    Some(wire::filter_predicate::Literal::Boolean(b)) => {
                                        Some(Literal::Boolean(*b))
                                    }
                                    Some(wire::filter_predicate::Literal::Decimal(d)) => {
                                        Some(Literal::Decimal(
                                            PivotDecimal::from_parts(
                                                d.lo, d.mid, d.hi, d.negative, d.scale,
                                            )
                                            .map_err(|e| e.to_string())?,
                                        ))
                                    }
                                },
                            })
                        })
                        .collect::<Result<_, String>>()?,
                })
            })
            .collect::<Result<_, String>>()?,
    })
}
impl Session {
    pub(super) fn handle_filter(
        &mut self,
        q: &wire::Request,
        r: &mut wire::Response,
        op: wire::request::Operation,
    ) -> Result<(), String> {
        use wire::request::Operation as Op;
        if q.revision != self.revision {
            r.conflict = true;
            return Err("expected-state conflict".into());
        }
        let book = self.book.as_mut().ok_or("no workbook")?;
        if self.proof.is_some() || book.is_structured_proof() {
            return Err("Filters require an ordinary workbook".into());
        }
        let sheet = WorksheetId::from_uuid(
            Uuid::parse_str(&q.worksheet).map_err(|_| "worksheet ID required")?,
        );
        book.filter(sheet).map_err(|e| e.to_string())?;
        r.worksheet = q.worksheet.clone();
        if q.filter_definition.is_some() || book.filter(sheet).map_err(|e| e.to_string())?.is_some()
        {
            self.filter
                .budget_status(&self.runtime.as_ref().ok_or("closed")?.memory())?;
        }
        if op == Op::GetFilter {
            r.filter_definition = book.filter(sheet).map_err(|e| e.to_string())?.map(encode);
            return Ok(());
        }
        if op == Op::ValidateFilter {
            let definition = decode(
                q.filter_definition
                    .as_ref()
                    .ok_or("filter definition required")?,
            )?;
            let info = book
                .sheets()
                .iter()
                .find(|s| s.id == sheet)
                .ok_or("unknown worksheet")?;
            definition
                .validate(info.row_count(), info.column_count())
                .map_err(|e| e.to_string())?;
            return Ok(());
        }
        if op == Op::SetFilter {
            self.filter.job = None;
            let definition = q.filter_definition.as_ref().map(decode).transpose()?;
            let before = book.expected();
            let (_, candidate) = book
                .set_filter_with_visibility(sheet, before, definition.as_ref(), &Default::default())
                .map_err(|e| e.to_string())?;
            if book.expected() != before {
                self.revision = Uuid::new_v4().to_string();
            }
            self.filter.failures.remove(&sheet);
            self.filter.reset();
            self.filter.session = Some(book.expected().session);
            self.filter.map = candidate;
            r.revision = self.revision.clone();
        }
        self.filter.ensure(book, sheet, op == Op::ReapplyFilter)?;
        self.filter.describe(book, sheet, r)?;
        if op != Op::ReadFiltered {
            return Ok(());
        }
        if q.filter_row_count > 20 || q.filter_row_count == 0 {
            return Err("filtered viewport requires 1..20 rows".into());
        }
        let first_column = pos(q.first.as_ref())?.column;
        let last_column = pos(q.last.as_ref())?.column;
        if first_column == 0
            || last_column < first_column
            || last_column > 1024
            || (last_column - first_column + 1) * q.filter_row_count > 200
        {
            return Err("filtered viewport exceeds 200 cells".into());
        }
        let rows = self.filter.rows(book, sheet)?;
        r.filter_total_visible_rows = rows.len() as u32;
        let start = q.filter_first_visible.max(1).min(
            (rows.len() as u32)
                .saturating_sub(q.filter_row_count)
                .saturating_add(1)
                .max(1),
        );
        r.filter_first_visible = start;
        r.filter_rows = rows
            .iter()
            .skip((start - 1) as usize)
            .take(q.filter_row_count as usize)
            .copied()
            .collect();
        r.filter_visible_mask = vec![0; 8192];
        for row in rows {
            r.filter_visible_mask[(row as usize - 1) / 8] |= 1 << ((row - 1) % 8);
        }
        let capture = book
            .capture_sheet(sheet, book.expected())
            .map_err(|e| e.to_string())?;
        r.capture_state = self.computational_state.clone();
        r.generation = Uuid::new_v4().to_string();
        r.capture_identity = r.generation.clone();
        let mut cells = Vec::new();
        for row in &r.filter_rows {
            let rect = Rectangle {
                first: Position::new(*row, first_column),
                last: Position::new(*row, last_column),
            };
            cells.extend(
                book.read_sheet(sheet, book.expected(), rect, 200)
                    .map_err(|e| e.to_string())?
                    .cells,
            );
            for (p, style) in book
                .style_viewport(sheet, rect)
                .map_err(|e| e.to_string())?
            {
                r.styles.push(wire::StyledPosition {
                    at: Some(location(p)),
                    style: style.encode().to_vec(),
                });
            }
        }
        let outputs = cells.iter().map(|c| c.reference).collect::<Vec<_>>();
        let values = capture.evaluate_values(&outputs);
        for (index, cell) in cells.into_iter().enumerate() {
            // Invalid authored formulas can refuse batch planning. Preserve the
            // existing independent-cell diagnostics using the same pinned root.
            let value = match &values {
                Ok(values) => Ok(values.values[index].1.clone()),
                Err(_) => capture
                    .evaluate_values(&[cell.reference])
                    .map(|values| values.values[0].1.clone())
                    .map_err(|error| error.to_string()),
            };
            let mut view = wire::Cell {
                at: Some(location(cell.position)),
                worksheet: q.worksheet.clone(),
                cell_id: cell.reference.cell.uuid().to_string(),
                node_id: cell.node.uuid().to_string(),
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
                Content::Decimal(d) => {
                    let (lo, mid, hi, negative, scale) = d.parts();
                    wire::cell::Content::Number(Decimal128 {
                        lo,
                        mid,
                        hi,
                        negative,
                        scale,
                    })
                }
                Content::Boolean(b) => wire::cell::Content::Boolean(b),
                Content::LiteralText(s) => wire::cell::Content::Text(s),
                Content::Blank => wire::cell::Content::Blank(true),
                Content::TextFormula(f) => {
                    view.authored_formula = f.original_source.clone();
                    wire::cell::Content::Formula(effective.unwrap_or(f.original_source))
                }
                Content::Formula { source, .. } => {
                    view.authored_formula = source.clone();
                    wire::cell::Content::Formula(effective.unwrap_or(source))
                }
                Content::StructuredFormula(formula) => {
                    wire::cell::Content::Formula(formula.source().into())
                }
            });
            match value {
                Ok(Ok(value)) => {
                    if let pivot_types::PivotValue::Decimal(d) = &value {
                        view.formatted_number = pivot_workbook::formatting::render_decimal(
                            *d,
                            book.style_at(sheet, cell.position)
                                .map_err(|e| e.to_string())?,
                        )
                        .map_err(|e| e.to_string())?;
                    }
                    set_value(&mut view, value);
                }
                Ok(Err(error)) => {
                    structured::set_cell_fault(&mut view, &error, &capture);
                    view.error_code = error_code(&error);
                    view.error = readable_error(format!("{error:?}"));
                }
                Err(error) => view.error = readable_error(error),
            }
            r.cells.push(view);
        }
        if r.encoded_len() > MAX_FRAME - 65536 {
            r.cells.clear();
            r.styles.clear();
            r.filter_rows.clear();
            return Err(
                "Viewport exceeds the existing response limit; request fewer rows or columns."
                    .into(),
            );
        }
        Ok(())
    }
}
