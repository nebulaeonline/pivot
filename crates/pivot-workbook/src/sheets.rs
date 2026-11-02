//! Sheet directory and explicit, short-lived request views over one canonical graph.
use super::*;
use pivot_core::model::integration::resources::Footprint;
use unicode_normalization::UnicodeNormalization;

pub const MAX_SHEETS: usize = 16;
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct SheetInfo {
    pub retired: bool,
    pub id: GridScopeId,
    pub name: String,
    pub(crate) rows: Axis,
    pub(crate) columns: Axis,
}
impl SheetInfo {
    pub fn row_count(&self) -> u32 {
        self.rows.length
    }
    pub fn column_count(&self) -> u32 {
        self.columns.length
    }
}
#[derive(Clone)]
pub(super) struct Directory(pub Vec<SheetInfo>);
impl Footprint for Directory {
    fn heap_bytes(&self) -> usize {
        self.0.capacity() * std::mem::size_of::<SheetInfo>()
            + self.0.iter().map(|s| s.name.capacity()).sum::<usize>()
    }
}
impl Directory {
    pub(super) fn axis_bytes(&self) -> usize {
        self.0
            .iter()
            .map(|s| {
                [&s.rows, &s.columns]
                    .into_iter()
                    .map(|axis| {
                        axis.layout.as_ref().map_or(0, |p| {
                            std::mem::size_of::<crate::axes::AxisLayout>() + p.heap_bytes()
                        })
                    })
                    .sum::<usize>()
            })
            .sum()
    }
}
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub(super) struct SheetPosition {
    pub worksheet: GridScopeId,
    pub position: Position,
}
impl std::ops::Deref for SheetPosition {
    type Target = Position;
    fn deref(&self) -> &Position {
        &self.position
    }
}
impl pivot_core::model::integration::resources::Key for SheetPosition {}
pub(super) fn at(worksheet: GridScopeId, position: Position) -> SheetPosition {
    SheetPosition {
        worksheet,
        position,
    }
}

pub fn normalize_name(name: &str) -> Result<String> {
    if name.len() > 256 {
        return Err(Error::InvalidTarget);
    }
    let name: String = name.nfc().collect();
    if name.is_empty()
        || name.len() > 64
        || name.trim() != name
        || name
            .chars()
            .any(|c| c.is_control() || "[]:*?/\\!".contains(c))
    {
        return Err(Error::Graph("invalid worksheet name".into()));
    }
    Ok(name)
}
pub fn name_key(name: &str) -> String {
    caseless::default_case_fold_str(name).nfc().collect()
}
pub fn quoted_name(name: &str) -> String {
    let mut chars = name.chars();
    if chars
        .next()
        .is_some_and(|c| c.is_ascii_alphabetic() || c == '_')
        && chars.all(|c| c.is_ascii_alphanumeric() || c == '_')
    {
        name.into()
    } else {
        format!("'{}'", name.replace('\'', "''"))
    }
}
pub(crate) fn render_source(
    source: &str,
    tokens: &[Token],
    directory: &[SheetInfo],
    limit: usize,
) -> Result<String> {
    let mut qualifiers: Vec<_> = tokens
        .iter()
        .filter_map(|t| {
            if let Token::Qualifier { worksheet, span } = t {
                Some((*worksheet, span))
            } else {
                None
            }
        })
        .collect();
    qualifiers.sort_by_key(|(_, span)| span.start);
    let mut text = String::new();
    let mut at = 0;
    for (id, span) in qualifiers {
        if span.start < at || source.get(span.clone()).is_none() {
            return Err(Error::InvalidTarget);
        }
        let sheet = directory
            .iter()
            .find(|s| s.id == id)
            .ok_or(Error::InvalidTarget)?;
        text.push_str(source.get(at..span.start).ok_or(Error::InvalidTarget)?);
        text.push_str(&quoted_name(&sheet.name));
        at = span.end;
        if text.len() > limit {
            return Err(Error::Limit);
        }
    }
    text.push_str(source.get(at..).ok_or(Error::InvalidTarget)?);
    if text.len() > limit {
        return Err(Error::Limit);
    }
    Ok(text)
}
pub(crate) fn validate_scope(
    owner: GridScopeId,
    source: &str,
    tokens: &[Token],
    directory: &[SheetInfo],
) -> Result<()> {
    let mut pending = None;
    for token in tokens {
        match token {
            Token::Qualifier { worksheet, span } => {
                if pending.is_some()
                    || span.is_empty()
                    || source.get(span.clone()).is_none()
                    || !directory.iter().any(|s| s.id == *worksheet)
                    || source
                        .get(span.end..)
                        .and_then(|s| s.trim_start().chars().next())
                        != Some(if worksheet.is_window() { '|' } else { '!' })
                {
                    return Err(Error::InvalidTarget);
                }
                pending = Some(*worksheet);
            }
            Token::Cell(cell) => {
                if cell.worksheet != pending.take().unwrap_or(owner) {
                    return Err(Error::InvalidTarget);
                }
            }
            Token::Bound(_) => {
                pending.take();
            }
            Token::Typed(pivot_core::typed::Op::ReferenceDeleted(_), 0) => {
                pending.take();
            }
            Token::WeightRange { rectangle, .. } | Token::Sum { rectangle, .. } => {
                if rectangle.space.0 != *pending.take().unwrap_or(owner).uuid().as_bytes() {
                    return Err(Error::InvalidTarget);
                }
            }
            Token::WeightRangeAt { .. }
            | Token::QualifierAt { .. }
            | Token::At(_)
            | Token::SumAt { .. } => {
                return Err(Error::InvalidTarget);
            }
            _ if pending.is_some() => return Err(Error::InvalidTarget),
            _ => (),
        }
    }
    if pending.is_some() {
        return Err(Error::InvalidTarget);
    }
    Ok(())
}
impl persistent::Data {
    pub(super) fn install_sheets(&mut self, sheets: Vec<SheetInfo>) -> Result<()> {
        if sheets.first().is_none_or(|s| s.id.is_window() || s.retired)
            || sheets.len() > 262_144
            || sheets.iter().filter(|s| !s.id.is_window()).count() > MAX_SHEETS
            || sheets
                .iter()
                .filter(|s| s.id.is_window() && !s.retired)
                .count()
                > 64
        {
            return Err(Error::Limit);
        }
        let _validation = self
            .memory
            .reserve(sheets.len().saturating_mul(512))
            .map_err(|_| Error::Limit)?;
        let mut ids = BTreeSet::new();
        let mut names = BTreeSet::new();
        let mut axes = BTreeSet::new();
        let mut saw_window = false;
        for sheet in &sheets {
            if (saw_window && !sheet.id.is_window())
                || (!sheet.id.is_window() && sheet.retired)
                || (sheet.id.is_window() && sheet.name.contains('|'))
            {
                return Err(Error::InvalidTarget);
            }
            saw_window |= sheet.id.is_window();
            if sheet.id.uuid().is_nil()
                || !ids.insert(sheet.id.uuid())
                || normalize_name(&sheet.name)? != sheet.name
                || (!sheet.retired && !names.insert((sheet.id.is_window(), name_key(&sheet.name))))
                || sheet.rows.namespace.is_nil()
                || sheet.columns.namespace.is_nil()
                || !axes.insert(sheet.rows.namespace)
                || !axes.insert(sheet.columns.namespace)
            {
                return Err(Error::InvalidTarget);
            }
        }
        self.sheets = Some(
            self.memory
                .alloc(Directory(sheets))
                .map_err(|_| Error::Limit)?,
        );
        Ok(())
    }
}
impl Workbook {
    pub(crate) fn reserve_scope_copy(
        &self,
    ) -> Result<pivot_core::model::integration::resources::Charge> {
        self.persistent
            .memory
            .reserve(
                self.persistent
                    .sheets
                    .as_ref()
                    .map_or(0, |s| s.heap_bytes())
                    + 1024,
            )
            .map_err(|_| Error::Limit)
    }
    pub fn scopes(&self) -> &[SheetInfo] {
        &self
            .persistent
            .sheets
            .as_ref()
            .expect("validated sheet directory")
            .0
    }
    pub fn sheets(&self) -> &[SheetInfo] {
        let all = self.scopes();
        &all[..all.partition_point(|s| !s.id.is_window())]
    }
    pub fn windows(&self) -> impl Iterator<Item = &SheetInfo> {
        self.scopes()
            .iter()
            .filter(|s| s.id.is_window() && !s.retired)
    }
    pub fn scope_by_name(&self, window: bool, name: &str) -> Option<GridScopeId> {
        let normalized = normalize_name(name).ok()?;
        let key = name_key(&normalized);
        self.scopes()
            .iter()
            .find(|s| !s.retired && s.id.is_window() == window && name_key(&s.name) == key)
            .map(|s| s.id)
    }
    pub fn sheet_by_name(&self, name: &str) -> Option<GridScopeId> {
        let normalized = normalize_name(name).ok()?;
        let key = name_key(&normalized);
        self.sheets()
            .iter()
            .find(|s| name_key(&s.name) == key)
            .map(|s| s.id)
    }
    pub(crate) fn sheet_info(&self, id: GridScopeId) -> Result<&SheetInfo> {
        self.scopes()
            .iter()
            .find(|s| s.id == id && !s.retired)
            .ok_or(Error::InvalidTarget)
    }
    pub fn add_sheet(&mut self, expected: Expected, name: Option<&str>) -> Result<GridScopeId> {
        self.check(expected)?;
        if self.is_structured_proof() || self.sheets().len() >= MAX_SHEETS {
            return Err(Error::Limit);
        }
        #[cfg(feature = "history-experiment")]
        if self.history_active {
            return Err(Error::InvalidTarget);
        }
        let name = match name {
            Some(n) => normalize_name(n)?,
            None => (1..)
                .map(|n| format!("Sheet{n}"))
                .find(|n| self.sheet_by_name(n).is_none())
                .unwrap(),
        };
        if self.sheet_by_name(&name).is_some() {
            return Err(Error::Graph("duplicate worksheet name".into()));
        }
        let id = GridScopeId::fresh();
        let _directory_copy = self.reserve_scope_copy()?;
        let mut sheets = self.scopes().to_vec();
        sheets.insert(
            self.sheets().len(),
            SheetInfo {
                retired: false,
                id,
                name,
                rows: Axis {
                    layout: None,
                    namespace: Uuid::new_v4(),
                    length: self.limits.rows,
                },
                columns: Axis {
                    layout: None,
                    namespace: Uuid::new_v4(),
                    length: self.limits.columns,
                },
            },
        );
        self.publish_sheets(sheets)?;
        Ok(id)
    }
    pub fn rename_sheet(
        &mut self,
        expected: Expected,
        id: GridScopeId,
        name: &str,
    ) -> Result<Expected> {
        self.check(expected)?;
        let name = normalize_name(name)?;
        let old = self.sheet_info(id)?;
        if old.name == name {
            return Ok(expected);
        }
        if self.is_structured_proof() {
            return Err(Error::InvalidTarget);
        }
        #[cfg(feature = "history-experiment")]
        if self.history_active {
            return Err(Error::InvalidTarget);
        }
        if self
            .scope_by_name(id.is_window(), &name)
            .is_some_and(|other| other != id)
        {
            return Err(Error::Graph("duplicate worksheet name".into()));
        }
        let _directory_copy = self.reserve_scope_copy()?;
        let mut sheets = self.scopes().to_vec();
        sheets.iter_mut().find(|s| s.id == id).unwrap().name = name;
        self.publish_sheets(sheets)?;
        Ok(self.expected())
    }
    pub(crate) fn publish_sheets(&mut self, sheets: Vec<SheetInfo>) -> Result<()> {
        let next = self.state.checked_add(1).ok_or(Error::Exhausted)?;
        let _scratch = self
            .persistent
            .memory
            .reserve(8192)
            .map_err(|_| Error::Limit)?;
        for (_, record) in self.persistent.records.iter() {
            match &record.content {
                Some(Content::TextFormula(f)) => {
                    render_source(
                        &f.original_source,
                        &f.tokens,
                        &sheets,
                        self.limits.source_bytes,
                    )?;
                }
                Some(Content::Formula { source, tokens }) => {
                    render_source(source, tokens, &sheets, self.limits.source_bytes)?;
                }
                _ => (),
            }
        }
        let mut core = self.core.stage();
        let mut data = core.stage_resource(&self.persistent).map_err(graph_error)?;
        data.install_sheets(sheets)?;
        crate::producers::maintain_guards(&mut core, &mut data, &Default::default())?;
        crate::windows::validate_scope_graph(&core, &data, &Default::default())?;
        if data.stats().estimated_owned_bytes > self.limits.bytes {
            return Err(Error::Limit);
        }
        self.persistent = data.freeze();
        self.core = core;
        self.state = next;
        Ok(())
    }
    /// An explicit request scope, not a retained active-sheet setting or another engine.
    /// Graph/domain identity and every persistent root are shared until commit.
    pub(crate) fn request_view(&self, id: GridScopeId) -> Result<Self> {
        let sheet = self.sheet_info(id)?;
        Ok(Self {
            #[cfg(feature = "history-experiment")]
            history_content: self.history_content,
            #[cfg(feature = "history-experiment")]
            history_active: self.history_active,
            #[cfg(feature = "history-experiment")]
            history_recording: self.history_recording.clone(),
            session: self.session,
            state: self.state,
            worksheet: id,
            graph_id: self.graph_id,
            rows: sheet.rows.clone(),
            columns: sheet.columns.clone(),
            persistent: self.persistent.clone(),
            core: self.core.stage(),
            limits: self.limits,
        })
    }
    pub(crate) fn mutate_sheet<T>(
        &mut self,
        id: GridScopeId,
        call: impl FnOnce(&mut Self) -> Result<T>,
    ) -> Result<T> {
        self.require_editable(id)?;
        let mut view = self.request_view(id)?;
        let answer = call(&mut view)?;
        self.persistent = view.persistent;
        self.core = view.core;
        self.state = view.state;
        #[cfg(feature = "history-experiment")]
        {
            self.history_content = view.history_content;
            self.history_recording = view.history_recording;
        }
        Ok(answer)
    }
    pub fn apply_sheet(
        &mut self,
        id: GridScopeId,
        expected: Expected,
        edits: &[Edit],
    ) -> Result<Expected> {
        self.mutate_sheet(id, |v| v.apply(expected, edits))
    }
    pub fn axes_at_sheet(&self, id: GridScopeId, position: Position) -> Result<(AxisId, AxisId)> {
        if self.window_binding(id).is_some() {
            return Err(Error::Graph(
                "Bound positions use producer item/field authority; request a projected frame."
                    .into(),
            ));
        }
        self.request_view(id)?.axes_at(position)
    }
    pub fn display_formula(&self, cell: CellRef) -> Result<Option<String>> {
        let record = self
            .persistent
            .records
            .get(&cell.cell)
            .ok_or(Error::InvalidTarget)?;
        if record.worksheet != cell.worksheet {
            return Err(Error::InvalidTarget);
        }
        let _scratch = self
            .persistent
            .memory
            .reserve(8192)
            .map_err(|_| Error::Limit)?;
        match &record.content {
            Some(Content::TextFormula(f)) if f.diagnostic.is_some() => {
                Ok(Some(f.original_source.clone()))
            }
            Some(Content::TextFormula(f)) => crate::structure::render(
                &f.original_source,
                &f.tokens,
                &self.persistent,
                self.scopes(),
                self.limits.source_bytes,
            )
            .map(|v| Some(v.0)),
            Some(Content::Formula { source, tokens }) => crate::structure::render(
                source,
                tokens,
                &self.persistent,
                self.scopes(),
                self.limits.source_bytes,
            )
            .map(|v| Some(v.0)),
            _ => Ok(None),
        }
    }
    pub fn cell_at_sheet(&self, id: GridScopeId, position: Position) -> Result<Option<CellView>> {
        self.request_view(id)?.cell_at(position)
    }
    pub fn read_sheet(
        &self,
        id: GridScopeId,
        expected: Expected,
        rectangle: Rectangle,
        max: usize,
    ) -> Result<RectangleRead> {
        self.request_view(id)?.read(expected, rectangle, max)
    }
    pub fn capture_sheet(&self, id: GridScopeId, expected: Expected) -> Result<CalculationCapture> {
        self.request_view(id)?.capture_local(expected)
    }
    pub fn fill_sheet(
        &mut self,
        id: GridScopeId,
        expected: Expected,
        rectangle: Rectangle,
        direction: FillDirection,
        cancel: &pivot_core::model::integration::CancellationToken,
    ) -> Result<Expected> {
        self.mutate_sheet(id, |v| v.fill(expected, rectangle, direction, cancel))
    }
    pub fn copy_sheet(
        &self,
        id: GridScopeId,
        expected: Expected,
        rectangle: Rectangle,
    ) -> Result<clipboard::ClipboardBlock> {
        self.request_view(id)?.copy_block(expected, rectangle)
    }
    pub fn paste_sheet(
        &mut self,
        id: GridScopeId,
        expected: Expected,
        rectangle: Rectangle,
        bytes: &[u8],
        native: bool,
        cancel: &pivot_core::model::integration::CancellationToken,
    ) -> Result<Rectangle> {
        self.mutate_sheet(id, |v| {
            v.paste_block(expected, rectangle, bytes, native, cancel)
        })
    }
}
impl CalculationCapture {
    pub fn cell_at_sheet(&self, id: GridScopeId, position: Position) -> Option<CellRef> {
        self.pinned
            .positions
            .get(&at(id, position))
            .map(|cell| CellRef {
                worksheet: id,
                cell: *cell,
            })
    }
}
