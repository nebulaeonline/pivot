//! Workbook-owned unbound Grid scopes. Editor selection is never server state.
use super::*;
use pivot_core::model::integration::CancellationToken;

pub(crate) fn validate_scope_graph(
    core: &WorkingModel,
    data: &persistent::Data,
    cancel: &CancellationToken,
) -> Result<()> {
    crate::producers::validate_feedback(data, cancel)?;
    if !data
        .sheets
        .as_ref()
        .is_some_and(|s| s.0.iter().any(|s| s.id.is_window() && !s.retired))
    {
        return Ok(());
    }
    let _scratch = data
        .memory
        .reserve(data.records.len().saturating_mul(32))
        .map_err(|_| Error::Limit)?;
    let outputs = data
        .records
        .iter()
        .filter(|(_, r)| r.position.is_some())
        .map(|(id, _)| {
            data.bindings
                .get(id)
                .map(|b| b.runtime)
                .ok_or(Error::InvalidTarget)
        })
        .collect::<Result<Vec<_>>>()?;
    core.capture_local()
        .map_err(graph_error)?
        .validate_acyclic(&outputs, cancel)
        .map_err(graph_error)
}

pub struct ScopeCapture {
    pub scope: GridScopeId,
    pub expected: Expected,
    pub calculation: CalculationCapture,
}
impl ScopeCapture {
    pub fn axes_at(&self, position: Position) -> Result<(AxisId, AxisId)> {
        if self
            .calculation
            .pinned
            .producers
            .as_ref()
            .is_some_and(|p| p.binding(self.scope).is_some())
        {
            return Err(Error::InvalidTarget);
        }
        let info = self
            .calculation
            .pinned
            .sheets
            .as_ref()
            .ok_or(Error::InvalidTarget)?
            .0
            .iter()
            .find(|s| s.id == self.scope && !s.retired)
            .ok_or(Error::InvalidTarget)?;
        Ok((
            info.rows.at(position.row)?,
            info.columns.at(position.column)?,
        ))
    }
}

impl Workbook {
    pub(crate) fn authoring_diagnostic(&self, tokens: &[Token]) -> Option<ParseError> {
        let mut qualified = None;
        for token in tokens {
            if let Token::QualifierAt { window, name, span } = token {
                let Some(scope) = self.scope_by_name(*window, name) else {
                    return Some(ParseError {
                        offset: span.start,
                        message: "unknown Grid scope",
                    });
                };
                qualified = Some(scope);
                continue;
            }
            let (first, last, offset) = match token {
                Token::At(p) => (*p, *p, 0),
                Token::SumAt {
                    first, last, span, ..
                }
                | Token::WeightRangeAt { first, last, span } => (*first, *last, span.start),
                _ => continue,
            };
            let info = self
                .sheet_info(qualified.take().unwrap_or(self.worksheet))
                .ok()?;
            if first.row > info.row_count()
                || last.row > info.row_count()
                || first.column > info.column_count()
                || last.column > info.column_count()
            {
                return Some(ParseError {
                    offset,
                    message: "cell reference outside Grid scope",
                });
            }
        }
        None
    }
    pub fn window_inbound_references(
        &self,
        id: WindowId,
        cancel: &CancellationToken,
    ) -> Result<usize> {
        let scope = GridScopeId::Window(id);
        self.sheet_info(scope)?;
        let mut count = 0usize;
        let mut work = 0usize;
        for (_, record) in self.persistent.records.iter() {
            if cancel.is_cancelled() {
                return Err(Error::Graph("cancelled".into()));
            }
            if record.worksheet == scope || record.position.is_none() {
                continue;
            }
            let tokens = match &record.content {
                Some(Content::Formula { tokens, .. }) => tokens.as_slice(),
                Some(Content::TextFormula(f)) => &f.tokens,
                _ => continue,
            };
            for token in tokens {
                work += 1;
                if work > structure::MAX_WORK {
                    return Err(Error::Limit);
                }
                if match token {
                    Token::Bound(guard) => self
                        .persistent
                        .guards
                        .get(guard)
                        .is_some_and(|g| g.window == id),
                    Token::Cell(c) => c.worksheet == scope,
                    Token::Sum { rectangle, .. } | Token::WeightRange { rectangle, .. } => {
                        rectangle.space.0 == *scope.uuid().as_bytes()
                    }
                    _ => false,
                } {
                    count += 1;
                }
            }
        }
        Ok(count)
    }
    pub fn create_grid_window(
        &mut self,
        expected: Expected,
        name: &str,
        rows: u32,
        columns: u32,
    ) -> Result<WindowId> {
        self.check(expected)?;
        if self.is_structured_proof()
            || self.windows().filter(|s| !s.retired).count() >= 64
            || rows == 0
            || columns == 0
            || rows > self.limits.rows
            || columns > self.limits.columns
        {
            return Err(Error::Limit);
        }
        #[cfg(feature = "history-experiment")]
        if self.history_active || self.history_recording.is_some() {
            return Err(Error::InvalidTarget);
        }
        let name = sheets::normalize_name(name)?;
        if name.contains('|') || self.scope_by_name(true, &name).is_some() {
            return Err(Error::InvalidTarget);
        }
        let id = WindowId::fresh();
        let _directory_copy = self.reserve_scope_copy()?;
        let mut scopes = self.scopes().to_vec();
        scopes.push(SheetInfo {
            retired: false,
            id: id.into(),
            name,
            rows: Axis {
                layout: None,
                namespace: Uuid::new_v4(),
                length: rows,
            },
            columns: Axis {
                layout: None,
                namespace: Uuid::new_v4(),
                length: columns,
            },
        });
        self.publish_sheets(scopes)?;
        Ok(id)
    }
    pub fn rename_window(
        &mut self,
        expected: Expected,
        id: WindowId,
        name: &str,
    ) -> Result<Expected> {
        if name.contains('|') {
            return Err(Error::InvalidTarget);
        }
        self.rename_sheet(expected, id.into(), name)
    }
    pub fn apply_scope(
        &mut self,
        scope: GridScopeId,
        expected: Expected,
        edits: &[Edit],
    ) -> Result<Expected> {
        self.apply_sheet(scope, expected, edits)
    }
    pub fn read_scope(
        &self,
        scope: GridScopeId,
        expected: Expected,
        rectangle: Rectangle,
        max_records: usize,
    ) -> Result<RectangleRead> {
        self.read_sheet(scope, expected, rectangle, max_records)
    }
    pub fn capture_scope(&self, scope: GridScopeId, expected: Expected) -> Result<ScopeCapture> {
        let calculation = self.capture_sheet(scope, expected)?;
        Ok(ScopeCapture {
            scope,
            expected,
            calculation,
        })
    }
    pub fn delete_window(
        &mut self,
        expected: Expected,
        id: WindowId,
        cancel: &CancellationToken,
    ) -> Result<Expected> {
        self.check(expected)?;
        let scope = GridScopeId::Window(id);
        let rows = self.sheet_info(scope)?.row_count();
        if self.window_binding(scope).is_some() {
            if cancel.is_cancelled() {
                return Err(Error::Graph("cancelled".into()));
            }
            let _copy = self.reserve_scope_copy()?;
            let mut scopes = self.scopes().to_vec();
            scopes
                .iter_mut()
                .find(|s| s.id == scope)
                .ok_or(Error::InvalidTarget)?
                .retired = true;
            let mut candidate = self.request_view(self.worksheet)?;
            candidate.publish_sheets(scopes)?;
            candidate.retire_geometry(scope)?;
            if cancel.is_cancelled() {
                return Err(Error::Graph("cancelled".into()));
            }
            self.core = candidate.core;
            self.persistent = candidate.persistent;
            self.state = candidate.state;
            return Ok(self.expected());
        }
        let mut candidate = self.request_view(self.worksheet)?;
        candidate
            .restructure(
                scope,
                expected,
                structure::Change {
                    axis: structure::AxisKind::Rows,
                    action: structure::Action::Delete,
                    start: 1,
                    count: rows,
                },
                cancel,
            )
            .map_err(|e| Error::Graph(format!("window retirement: {e}")))?;
        let _directory_copy = candidate.reserve_scope_copy()?;
        let mut scopes = candidate.scopes().to_vec();
        scopes
            .iter_mut()
            .find(|s| s.id == scope)
            .ok_or(Error::InvalidTarget)?
            .retired = true;
        candidate
            .publish_sheets(scopes)
            .map_err(|e| Error::Graph(format!("window directory retirement: {e}")))?;
        candidate.retire_geometry(scope)?;
        if cancel.is_cancelled() {
            return Err(Error::Graph("cancelled".into()));
        }
        candidate.state = self.state.checked_add(1).ok_or(Error::Exhausted)?;
        self.core = candidate.core;
        self.persistent = candidate.persistent;
        self.state = candidate.state;
        Ok(self.expected())
    }
}
