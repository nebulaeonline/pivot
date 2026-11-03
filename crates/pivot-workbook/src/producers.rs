//! Producer authority and immutable payloads. Reads never execute a provider.
pub mod codec;
use super::*;
use pivot_core::model::integration::CancellationToken;
use pivot_core::model::integration::resources::Charge;
use pivot_core::model::integration::resources::{Footprint, Memory, Shared};
use sha2::{Digest, Sha256};
use std::collections::BTreeMap;
use std::sync::atomic::{AtomicUsize, Ordering};

pub const MAX_PRODUCERS: usize = 32;
pub const MAX_FIELDS: usize = 64;
pub const MAX_VIEW_ROWS: usize = 100_000;
pub const MAX_SLOTS: usize = 65_536;
pub const MAX_KEY_BYTES: usize = 256;
pub const MAX_STRING_BYTES: usize = 4096;

macro_rules! ids {
    ($($id:ident),+) => {$(
        #[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
        pub struct $id(pub Uuid);
        impl $id { fn fresh() -> Self { Self(Uuid::new_v4()) } }
    )+};
}
ids!(ProducerId, SchemaId, FieldId, GenerationId, GuardId);
impl pivot_core::model::integration::resources::Key for GuardId {}
impl Footprint for GuardId {}
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum GuardTarget {
    Cell(CellRef),
    External {
        key: String,
        field: FieldId,
        generation: Option<GenerationId>,
    },
    Range {
        selection: pivot_core::model::selection::SelectionId,
        rectangle: Option<Rectangle>,
        kind: Option<expression::AggregateKind>,
    },
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Guard {
    pub id: GuardId,
    pub window: WindowId,
    pub producer: ProducerId,
    pub configuration: u64,
    pub target: GuardTarget,
    pub active: bool,
    /// First/last row/column absolute bits (0..3), reversed row/column bits (4..5).
    pub copy_flags: u8,
}
impl Footprint for Guard {
    fn heap_bytes(&self) -> usize {
        match &self.target {
            GuardTarget::External { key, .. } => key.capacity(),
            _ => 0,
        }
    }
}
#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct OutputKey {
    pub producer: ProducerId,
    pub configuration: u64,
    pub field: FieldId,
    pub key: String,
    pub generation: Option<GenerationId>,
}
impl pivot_core::model::integration::resources::Key for OutputKey {
    fn heap_bytes(&self) -> usize {
        self.key.capacity()
    }
}
#[derive(Clone, Copy)]
pub(crate) struct Output {
    pub node: NodeId,
    pub slot: pivot_core::captured::CapturedScalarId,
    pub portable: GraphNodeId,
}
impl Footprint for Output {}
impl Guard {
    pub(crate) fn output_key(&self) -> Option<OutputKey> {
        let GuardTarget::External {
            key,
            field,
            generation,
        } = &self.target
        else {
            return None;
        };
        Some(OutputKey {
            producer: self.producer,
            configuration: self.configuration,
            field: *field,
            key: key.clone(),
            generation: *generation,
        })
    }
}
pub(crate) fn external_location(
    data: &persistent::Data,
    scope: GridScopeId,
    position: Position,
) -> Result<Option<(WindowBinding, OutputKey)>> {
    let Some(directory) = &data.producers else {
        return Ok(None);
    };
    let Some(b) = directory.binding(scope) else {
        return Ok(None);
    };
    let p = directory.producer(b.producer)?;
    let Source::Fixture {
        schema,
        policy,
        selected,
        ..
    } = &p.source
    else {
        return Ok(None);
    };
    if p.retired {
        return Err(Error::InvalidTarget);
    }
    let s = selected
        .as_ref()
        .ok_or_else(|| Error::Graph("Refresh this source before binding a value.".into()))?;
    let row = s
        .rows
        .get(position.row.checked_sub(1).ok_or(Error::Position)? as usize)
        .ok_or(Error::Position)?;
    let field = schema
        .fields
        .get(position.column.checked_sub(1).ok_or(Error::Position)? as usize)
        .ok_or(Error::Position)?;
    if field.kind != ValueKind::Decimal {
        return Err(Error::Graph(
            "External non-Decimal fields are presentation-only.".into(),
        ));
    }
    Ok(Some((
        b,
        OutputKey {
            producer: p.id,
            configuration: p.configuration,
            field: field.id,
            key: row.key.clone(),
            generation: (*policy == KeyPolicy::GenerationQualified).then_some(s.generation),
        },
    )))
}
type WorksheetLocation = (WindowBinding, ProducerId, u64, GridScopeId, Position);
pub(crate) fn worksheet_location(
    data: &persistent::Data,
    scope: GridScopeId,
    position: Position,
) -> Result<Option<WorksheetLocation>> {
    let Some(directory) = &data.producers else {
        return Ok(None);
    };
    let Some(binding) = directory.binding(scope) else {
        return Ok(None);
    };
    let p = directory.producer(binding.producer)?;
    if p.retired {
        return Err(Error::InvalidTarget);
    }
    let Source::Worksheet {
        worksheet,
        rectangle: Some(r),
        ..
    } = p.source
    else {
        return Err(Error::Graph(
            "This bound source cannot supply a worksheet cell.".into(),
        ));
    };
    if position.row == 0
        || position.column == 0
        || position.row > r.last.row - r.first.row + 1
        || position.column > r.last.column - r.first.column + 1
    {
        return Err(Error::Position);
    }
    Ok(Some((
        binding,
        p.id,
        p.configuration,
        worksheet.into(),
        Position::new(
            r.first.row + position.row - 1,
            r.first.column + position.column - 1,
        ),
    )))
}
pub(crate) fn install_guard(
    data: &mut persistent::Data,
    window: WindowId,
    producer: ProducerId,
    configuration: u64,
    target: GuardTarget,
    copy_flags: u8,
) -> Result<GuardId> {
    if data.stats().allocated_nodes >= Limits::capacity().allocated_cells {
        return Err(Error::Limit);
    }
    let id = GuardId::fresh();
    let guard = data
        .memory
        .alloc(Guard {
            id,
            window,
            producer,
            configuration,
            target,
            active: true,
            copy_flags,
        })
        .map_err(|_| Error::Limit)?;
    data.guards.insert(id, guard).map_err(|_| Error::Limit)?;
    Ok(id)
}
pub(crate) fn guard_active(data: &persistent::Data, guard: &Guard) -> bool {
    if !data.sheets.as_ref().is_some_and(|s| {
        s.0.iter()
            .any(|s| s.id == guard.window.into() && !s.retired)
    }) {
        return false;
    }
    let Some(directory) = &data.producers else {
        return false;
    };
    let Ok(p) = directory.producer(guard.producer) else {
        return false;
    };
    if p.retired || p.configuration != guard.configuration {
        return false;
    }
    if let Source::Fixture {
        schema,
        policy,
        selected: Some(s),
        ..
    } = &p.source
    {
        let GuardTarget::External {
            key,
            field,
            generation,
        } = &guard.target
        else {
            return false;
        };
        return s.row(key).is_some()
            && schema
                .fields
                .iter()
                .any(|f| f.id == *field && f.kind == ValueKind::Decimal)
            && match policy {
                KeyPolicy::Durable => generation.is_none(),
                KeyPolicy::GenerationQualified => *generation == Some(s.generation),
            };
    }
    let Source::Worksheet {
        worksheet,
        rectangle: Some(r),
        ..
    } = p.source
    else {
        return false;
    };
    let contains = |at: Position| {
        at.row >= r.first.row
            && at.row <= r.last.row
            && at.column >= r.first.column
            && at.column <= r.last.column
    };
    match &guard.target {
        GuardTarget::Cell(c) => {
            c.worksheet == worksheet.into()
                && data
                    .records
                    .get(&c.cell)
                    .and_then(|c| c.position)
                    .is_some_and(contains)
        }
        GuardTarget::Range {
            rectangle: Some(range),
            ..
        } => contains(range.first) && contains(range.last),
        _ => false,
    }
}
pub(crate) fn resolved_guard(data: &persistent::Data, id: GuardId) -> Result<ResolvedToken> {
    let guard = data.guards.get(&id).ok_or(Error::InvalidTarget)?;
    if !guard_active(data, guard) {
        return Ok(ResolvedToken::Typed(
            pivot_core::typed::Op::ReferenceDeleted(String::new()),
            0,
        ));
    }
    Ok(match &guard.target {
        GuardTarget::Cell(c) => ResolvedToken::Node(
            data.bindings
                .get(&c.cell)
                .ok_or(Error::InvalidTarget)?
                .runtime,
        ),
        GuardTarget::External { .. } => ResolvedToken::Node(
            data.outputs
                .get(&guard.output_key().ok_or(Error::InvalidTarget)?)
                .ok_or(Error::InvalidTarget)?
                .node,
        ),
        GuardTarget::Range {
            selection,
            kind: Some(kind),
            ..
        } => {
            if *kind == expression::AggregateKind::Sum {
                ResolvedToken::Sum(*selection)
            } else {
                ResolvedToken::Aggregate(*kind, *selection)
            }
        }
        GuardTarget::Range {
            selection,
            kind: None,
            ..
        } => ResolvedToken::Range(*selection),
    })
}
pub(crate) fn guard_position(data: &persistent::Data, guard: &Guard) -> Option<Position> {
    if !guard_active(data, guard) {
        return None;
    }
    let p = data.producers.as_ref()?.producer(guard.producer).ok()?;
    if let Source::Fixture {
        schema,
        selected: Some(s),
        ..
    } = &p.source
    {
        let GuardTarget::External { key, field, .. } = &guard.target else {
            return None;
        };
        return Some(Position::new(
            s.ordinal(key)? as u32 + 1,
            schema.fields.iter().position(|f| f.id == *field)? as u32 + 1,
        ));
    }
    let Source::Worksheet {
        rectangle: Some(r), ..
    } = p.source
    else {
        return None;
    };
    let GuardTarget::Cell(c) = guard.target else {
        return None;
    };
    let at = data.records.get(&c.cell)?.position?;
    Some(Position::new(
        at.row - r.first.row + 1,
        at.column - r.first.column + 1,
    ))
}
pub(crate) fn guard_reference_text(data: &persistent::Data, g: &Guard) -> Option<String> {
    if !guard_active(data, g) {
        return None;
    }
    let name = &data
        .sheets
        .as_ref()?
        .0
        .iter()
        .find(|s| s.id == g.window.into())?
        .name;
    let address = |p: Position, shift: u8| {
        clipboard::address(
            p,
            &expression::ReferenceSyntax {
                qualifier: None,
                row: p.row,
                column: p.column,
                absolute_row: g.copy_flags & (1 << shift) != 0,
                absolute_column: g.copy_flags & (2 << shift) != 0,
                span: 0..0,
            },
        )
    };
    let text = if let GuardTarget::Range {
        rectangle: Some(r), ..
    } = g.target
    {
        let p = data.producers.as_ref()?.producer(g.producer).ok()?;
        let Source::Worksheet {
            rectangle: Some(source),
            ..
        } = p.source
        else {
            return None;
        };
        let a = Position::new(
            r.first.row - source.first.row + 1,
            r.first.column - source.first.column + 1,
        );
        let z = Position::new(
            r.last.row - source.first.row + 1,
            r.last.column - source.first.column + 1,
        );
        let first = Position::new(
            if g.copy_flags & 16 == 0 { a.row } else { z.row },
            if g.copy_flags & 32 == 0 {
                a.column
            } else {
                z.column
            },
        );
        let last = Position::new(
            if g.copy_flags & 16 == 0 { z.row } else { a.row },
            if g.copy_flags & 32 == 0 {
                z.column
            } else {
                a.column
            },
        );
        format!("{}:{}", address(first, 0), address(last, 2))
    } else {
        address(guard_position(data, g)?, 0)
    };
    Some(format!("{}|{text}", sheets::quoted_name(name)))
}
pub(crate) fn resolved_tokens(
    data: &persistent::Data,
    tokens: &[Token],
) -> Result<Vec<ResolvedToken>> {
    let mut out = Vec::with_capacity(tokens.len());
    for (index, t) in tokens.iter().enumerate() {
        let token =
            match t {
                Token::Bound(id) => {
                    let g = data.guards.get(id).ok_or(Error::InvalidTarget)?;
                    let _ = g;
                    resolved_guard(data, *id)?
                }
                Token::Qualifier { .. } => continue,
                Token::Cell(c) => ResolvedToken::Node(
                    data.bindings
                        .get(&c.cell)
                        .ok_or(Error::InvalidTarget)?
                        .runtime,
                ),
                Token::Decimal(d) => ResolvedToken::Decimal(d.value()),
                Token::Typed(op, n) => ResolvedToken::Typed(op.clone(), *n),
                Token::Control(c) => {
                    let previous = index.checked_sub(1).and_then(|i| tokens.get(i));
                    let c = if let Some(Token::Bound(id)) = previous
                        && data.guards.get(id).is_some_and(|g| {
                            matches!(g.target, GuardTarget::Range { kind: None, .. })
                        }) {
                        if guard_active(data, data.guards.get(id).ok_or(Error::InvalidTarget)?) {
                            expression::AggregateControl::Range
                        } else {
                            expression::AggregateControl::Value
                        }
                    } else {
                        *c
                    };
                    ResolvedToken::Control(c)
                }
                Token::WeightRange { id, .. } => ResolvedToken::Range(*id),
                Token::Sum { kind, id, .. } => {
                    if *kind == expression::AggregateKind::Sum {
                        ResolvedToken::Sum(*id)
                    } else {
                        ResolvedToken::Aggregate(*kind, *id)
                    }
                }
                Token::Unary(op) => ResolvedToken::Unary(*op),
                Token::Binary(op) => ResolvedToken::Binary(*op),
                Token::Output(_) => {
                    ResolvedToken::Node(data.adapter.as_ref().ok_or(Error::InvalidTarget)?.node)
                }
                _ => return Err(Error::InvalidTarget),
            };
        out.push(token);
    }
    Ok(out)
}
pub(crate) fn validate_guard_source(
    data: &persistent::Data,
    owner: GridScopeId,
    source: &str,
    tokens: &[Token],
) -> Result<()> {
    let parsed = expression::parse_spreadsheet(source);
    let (syntax, _, diagnostic) = Workbook::lower_text_bounds(MAX_ROWS, MAX_COLUMNS, &parsed);
    if diagnostic.is_some() || syntax.len() != tokens.len() {
        return Err(Error::InvalidTarget);
    }
    let mut scope = None;
    for (index, (syntax, bound)) in syntax.iter().zip(tokens).enumerate() {
        match (syntax, bound) {
            (Token::Typed(pivot_core::typed::Op::ReferenceDeleted(_), 0), Token::Bound(id)) => {
                if !data.guards.contains_key(id) {
                    return Err(Error::InvalidTarget);
                }
                scope.take();
            }
            (
                Token::QualifierAt { window, span, .. },
                Token::Qualifier {
                    worksheet,
                    span: actual,
                },
            ) if *window == worksheet.is_window() && span == actual => {
                scope = Some(*worksheet);
            }
            (Token::At(_), Token::Bound(id)) => {
                let g = data.guards.get(id).ok_or(Error::InvalidTarget)?;
                if scope.take().unwrap_or(owner) != g.window.into()
                    || matches!(g.target, GuardTarget::Range { .. })
                {
                    return Err(Error::InvalidTarget);
                }
            }
            (Token::SumAt { kind, .. }, Token::Bound(id)) => {
                let g = data.guards.get(id).ok_or(Error::InvalidTarget)?;
                if scope.take().unwrap_or(owner) != g.window.into()
                    || !matches!(g.target,GuardTarget::Range{kind:Some(k),..} if k==*kind)
                {
                    return Err(Error::InvalidTarget);
                }
            }
            (Token::WeightRangeAt { .. }, Token::Bound(id)) => {
                let g = data.guards.get(id).ok_or(Error::InvalidTarget)?;
                if scope.take().unwrap_or(owner) != g.window.into()
                    || !matches!(g.target, GuardTarget::Range { kind: None, .. })
                {
                    return Err(Error::InvalidTarget);
                }
            }
            (Token::At(_), Token::Cell(c)) => {
                if scope.take().unwrap_or(owner) != c.worksheet
                    || data
                        .records
                        .get(&c.cell)
                        .is_none_or(|r| r.worksheet != c.worksheet)
                {
                    return Err(Error::InvalidTarget);
                }
            }
            (Token::WeightRangeAt { .. }, Token::WeightRange { rectangle, .. })
            | (Token::SumAt { .. }, Token::Sum { rectangle, .. }) => {
                if rectangle.space.0 != *scope.take().unwrap_or(owner).uuid().as_bytes() {
                    return Err(Error::InvalidTarget);
                }
            }
            _ if syntax == bound => (),
            _ if structure::structural_syntax(
                syntax,
                bound,
                index.checked_sub(1).and_then(|i| tokens.get(i)),
            )? =>
            {
                scope.take();
            }
            _ => return Err(Error::InvalidTarget),
        }
    }
    if scope.is_some() {
        return Err(Error::InvalidTarget);
    }
    Ok(())
}
pub(crate) fn validate_authority(
    data: &persistent::Data,
    selections: &[(
        pivot_core::model::selection::SelectionId,
        Option<pivot_core::model::selection::Rectangle>,
    )],
    limits: Limits,
) -> Result<()> {
    let Some(directory) = &data.producers else {
        if !data.guards.is_empty() || !data.outputs.is_empty() {
            return Err(Error::InvalidTarget);
        }
        return Ok(());
    };
    directory.validate(data, limits)?;
    for binding in &directory.windows {
        let scope: GridScopeId = binding.window.into();
        if data.filters.get(&scope).is_some()
            || data.styles.regions().any(|region| region.sheet == scope)
            || data.styles.defaults().any(|(sheet, _)| sheet == scope)
        {
            return Err(Error::InvalidTarget);
        }
    }
    let selections: BTreeMap<_, _> = selections.iter().copied().collect();
    for p in &directory.producers {
        if let Source::Worksheet {
            worksheet,
            selection,
            rectangle,
        } = p.source
        {
            let expected = rectangle.map(|r| pivot_core::model::selection::Rectangle {
                space: pivot_core::model::selection::SpaceId(*worksheet.uuid().as_bytes()),
                first_row: r.first.row,
                last_row: r.last.row,
                first_column: r.first.column,
                last_column: r.last.column,
            });
            if selections.get(&selection) != Some(&expected) {
                return Err(Error::InvalidTarget);
            }
        }
    }
    for (_, g) in data.guards.iter() {
        let p = directory.producer(g.producer)?;
        if directory
            .binding(g.window.into())
            .is_none_or(|b| b.producer != g.producer)
            || g.configuration != p.configuration
            || g.copy_flags > 63
            || g.active != guard_active(data, g)
        {
            return Err(Error::InvalidTarget);
        }
        match (&g.target, &p.source) {
            (GuardTarget::Cell(c), Source::Worksheet { worksheet, .. }) => {
                if c.worksheet != (*worksheet).into()
                    || data
                        .records
                        .get(&c.cell)
                        .is_none_or(|r| r.worksheet != c.worksheet)
                {
                    return Err(Error::InvalidTarget);
                }
            }
            (
                GuardTarget::Range {
                    selection,
                    rectangle,
                    ..
                },
                Source::Worksheet { worksheet, .. },
            ) => {
                let expected = rectangle.map(|r| pivot_core::model::selection::Rectangle {
                    space: pivot_core::model::selection::SpaceId(*worksheet.uuid().as_bytes()),
                    first_row: r.first.row,
                    last_row: r.last.row,
                    first_column: r.first.column,
                    last_column: r.last.column,
                });
                if selections.get(selection) != Some(&expected) {
                    return Err(Error::InvalidTarget);
                }
            }
            (GuardTarget::External { .. }, Source::Fixture { .. }) => {
                if !data
                    .outputs
                    .contains_key(&g.output_key().ok_or(Error::InvalidTarget)?)
                {
                    return Err(Error::InvalidTarget);
                }
            }
            _ => return Err(Error::InvalidTarget),
        }
    }
    for (key, _) in data.outputs.iter() {
        let p = directory.producer(key.producer)?;
        let Source::Fixture { schema, policy, .. } = &p.source else {
            return Err(Error::InvalidTarget);
        };
        if key.configuration != p.configuration
            || key.key.is_empty()
            || key.key.len() > MAX_KEY_BYTES
            || !schema
                .fields
                .iter()
                .any(|f| f.id == key.field && f.kind == ValueKind::Decimal)
            || (key.generation.is_some() != (*policy == KeyPolicy::GenerationQualified))
            || key.generation.is_some_and(|g| g.0.is_nil())
        {
            return Err(Error::InvalidTarget);
        }
    }
    for (_, r) in data.records.iter() {
        if directory.binding(r.worksheet).is_some() {
            return Err(Error::InvalidTarget);
        }
        let (source, tokens) = match &r.content {
            Some(Content::Formula { source, tokens }) => (source.as_str(), tokens.as_slice()),
            Some(Content::TextFormula(f)) if f.diagnostic.is_none() => {
                (f.original_source.as_str(), f.tokens.as_slice())
            }
            _ => continue,
        };
        if tokens.iter().any(|t| matches!(t, Token::Bound(_))) {
            validate_guard_source(data, r.worksheet, source, tokens)?;
        }
    }
    Ok(())
}
pub(crate) fn maintain_guards(
    core: &mut WorkingModel,
    data: &mut persistent::Data,
    cancel: &pivot_core::model::integration::CancellationToken,
) -> Result<()> {
    if data.guards.is_empty() {
        return Ok(());
    }
    let _scratch = data
        .memory
        .reserve(data.guards.len().saturating_mul(256) + data.records.len().saturating_mul(64))
        .map_err(|_| Error::Limit)?;
    let mut changed = BTreeSet::new();
    for (id, g) in data.guards.iter() {
        if cancel.is_cancelled() {
            return Err(Error::Graph("cancelled".into()));
        }
        if g.active != guard_active(data, g) {
            changed.insert(*id);
        }
    }
    for id in &changed {
        let mut g = (**data.guards.get(id).ok_or(Error::InvalidTarget)?).clone();
        g.active = guard_active(data, &g);
        data.guards
            .insert(*id, data.memory.alloc(g).map_err(|_| Error::Limit)?)
            .map_err(|_| Error::Limit)?;
    }
    if changed.is_empty() {
        return Ok(());
    }
    let mut updates = Vec::new();
    let mut bytes = 0usize;
    for (id, record) in data.records.iter() {
        if record.position.is_none() {
            continue;
        }
        let (source, tokens) = match &record.content {
            Some(Content::Formula { source, tokens }) => (source.as_str(), tokens.as_slice()),
            Some(Content::TextFormula(f)) if f.diagnostic.is_none() => {
                (f.original_source.as_str(), f.tokens.as_slice())
            }
            _ => continue,
        };
        if tokens
            .iter()
            .any(|t| matches!(t,Token::Bound(g) if changed.contains(g)))
        {
            bytes = bytes.saturating_add(source.len());
            updates.push((*id, source.to_owned(), resolved_tokens(data, tokens)?));
        }
    }
    let mut batch = core
        .begin_batch(updates.len(), bytes)
        .map_err(graph_error)?;
    for (id, source, tokens) in updates {
        batch
            .set_resolved(
                data.bindings.get(&id).ok_or(Error::InvalidTarget)?.runtime,
                &source,
                tokens,
            )
            .map_err(graph_error)?;
    }
    if !batch.is_empty() {
        core.apply_batch(batch, cancel).map_err(graph_error)?;
    }
    Ok(())
}

pub(crate) fn validate_feedback(
    data: &persistent::Data,
    cancel: &pivot_core::model::integration::CancellationToken,
) -> Result<()> {
    let Some(directory) = &data.producers else {
        return Ok(());
    };
    let _scratch = data
        .memory
        .reserve(
            data.records
                .len()
                .saturating_mul(128 + directory.producers.len() * std::mem::size_of::<CellId>()),
        )
        .map_err(|_| Error::Limit)?;
    let mut work = 0usize;
    let mut step = || -> Result<()> {
        work += 1;
        if work > structure::MAX_WORK {
            return Err(Error::Limit);
        }
        if cancel.is_cancelled() {
            return Err(Error::Graph("cancelled".into()));
        }
        Ok(())
    };
    let mut sources = BTreeMap::new();
    for p in &directory.producers {
        if p.retired {
            continue;
        }
        let Source::Worksheet {
            worksheet,
            rectangle: Some(r),
            ..
        } = p.source
        else {
            continue;
        };
        let mut cells = Vec::new();
        for (id, record) in data.records.iter() {
            step()?;
            if record.worksheet == worksheet.into()
                && record.position.is_some_and(|at| {
                    at.row >= r.first.row
                        && at.row <= r.last.row
                        && at.column >= r.first.column
                        && at.column <= r.last.column
                })
            {
                cells.push(*id);
            }
        }
        sources.insert(p.id, cells);
    }
    for (producer, cells) in &sources {
        let mut seen: BTreeSet<CellId> = cells.iter().copied().collect();
        let mut pending = cells.clone();
        while let Some(id) = pending.pop() {
            step()?;
            let Some(record) = data.records.get(&id) else {
                continue;
            };
            if record.position.is_none() {
                continue;
            }
            let tokens = match &record.content {
                Some(Content::Formula { tokens, .. }) => tokens.as_slice(),
                Some(Content::TextFormula(f)) => f.tokens.as_slice(),
                _ => continue,
            };
            for t in tokens {
                step()?;
                match t {
                    Token::Bound(id) => {
                        let g = data.guards.get(id).ok_or(Error::InvalidTarget)?;
                        if !data.sheets.as_ref().is_some_and(|s| {
                            s.0.iter().any(|s| s.id == g.window.into() && !s.retired)
                        }) {
                            continue;
                        }
                        if g.producer == *producer {
                            return Err(Error::Graph(
                                "Producer feedback cycle; candidate refused.".into(),
                            ));
                        }
                        if let Some(cells) = sources.get(&g.producer) {
                            for id in cells {
                                step()?;
                                if seen.insert(*id) {
                                    pending.push(*id);
                                }
                            }
                        }
                    }
                    Token::Cell(c) => {
                        if seen.insert(c.cell) {
                            pending.push(c.cell);
                        }
                    }
                    Token::Sum { rectangle, .. } | Token::WeightRange { rectangle, .. } => {
                        for (id, r) in data.records.iter() {
                            step()?;
                            if rectangle.space.0 == *r.worksheet.uuid().as_bytes()
                                && r.position.is_some_and(|at| {
                                    at.row >= rectangle.first_row
                                        && at.row <= rectangle.last_row
                                        && at.column >= rectangle.first_column
                                        && at.column <= rectangle.last_column
                                })
                                && seen.insert(*id)
                            {
                                pending.push(*id);
                            }
                        }
                    }
                    _ => (),
                }
            }
        }
    }
    Ok(())
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum KeyPolicy {
    Durable,
    GenerationQualified,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ValueKind {
    Decimal,
    String,
    Boolean,
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Value {
    Blank,
    Decimal(DecimalInput),
    String(String),
    Boolean(bool),
}
impl Value {
    fn bytes(&self) -> usize {
        if let Self::String(s) = self {
            s.capacity()
        } else {
            0
        }
    }
    pub fn typed(&self) -> pivot_types::PivotValue {
        match self {
            Self::Blank => pivot_types::PivotValue::Blank,
            Self::Decimal(d) => pivot_types::PivotValue::Decimal(d.value()),
            Self::String(s) => pivot_types::PivotValue::String(s.clone()),
            Self::Boolean(b) => pivot_types::PivotValue::Boolean(*b),
        }
    }
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Field {
    pub id: FieldId,
    pub label: String,
    pub kind: ValueKind,
    pub nullable: bool,
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Schema {
    pub id: SchemaId,
    pub fields: Vec<Field>,
}
impl Schema {
    pub fn validate(&self) -> Result<()> {
        let mut seen = BTreeSet::new();
        if self.id.0.is_nil() || self.fields.is_empty() || self.fields.len() > MAX_FIELDS {
            return Err(Error::InvalidTarget);
        }
        for f in &self.fields {
            if f.id.0.is_nil() || !seen.insert(f.id) || f.label.len() > 256 {
                return Err(Error::InvalidTarget);
            }
        }
        Ok(())
    }
}
impl Footprint for Schema {
    fn heap_bytes(&self) -> usize {
        self.fields.capacity() * std::mem::size_of::<Field>()
            + self
                .fields
                .iter()
                .map(|f| f.label.capacity())
                .sum::<usize>()
    }
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Row {
    pub key: String,
    pub values: Vec<Value>,
}
impl Row {
    fn bytes(&self) -> usize {
        self.key.capacity()
            + self.values.capacity() * std::mem::size_of::<Value>()
            + self.values.iter().map(Value::bytes).sum::<usize>()
    }
}
pub struct Snapshot {
    pub producer: ProducerId,
    pub configuration: u64,
    pub schema: SchemaId,
    pub generation: GenerationId,
    pub policy: KeyPolicy,
    pub digest: [u8; 32],
    pub rows: Vec<Row>,
    keys: BTreeMap<String, usize>,
}
impl Footprint for Snapshot {
    fn heap_bytes(&self) -> usize {
        self.rows.capacity() * std::mem::size_of::<Row>()
            + self.rows.iter().map(Row::bytes).sum::<usize>()
            // Conservative B-tree node allocation plus owned duplicate key bytes.
            + self.keys.len() * 128 + self.keys.keys().map(String::capacity).sum::<usize>()
    }
}
impl Snapshot {
    pub fn row(&self, key: &str) -> Option<&Row> {
        self.keys.get(key).and_then(|i| self.rows.get(*i))
    }
    pub fn ordinal(&self, key: &str) -> Option<usize> {
        self.keys.get(key).copied()
    }

    fn payload_digest(schema: SchemaId, policy: KeyPolicy, rows: &[Row]) -> [u8; 32] {
        let mut h = Sha256::new();
        h.update(b"Pivot producer payload v1");
        h.update(schema.0.as_bytes());
        h.update([policy as u8]);
        h.update((rows.len() as u64).to_le_bytes());
        for row in rows {
            h.update((row.key.len() as u64).to_le_bytes());
            h.update(row.key.as_bytes());
            h.update((row.values.len() as u64).to_le_bytes());
            for v in &row.values {
                match v {
                    Value::Blank => h.update([0]),
                    Value::Decimal(d) => {
                        h.update([1]);
                        let (lo, mid, hi, negative, scale) = d.parts();
                        for n in [lo, mid, hi, scale] {
                            h.update(n.to_le_bytes());
                        }
                        h.update([u8::from(negative)]);
                    }
                    Value::String(s) => {
                        h.update([2]);
                        h.update((s.len() as u64).to_le_bytes());
                        h.update(s.as_bytes());
                    }
                    Value::Boolean(b) => h.update([3, u8::from(*b)]),
                }
            }
        }
        h.finalize().into()
    }

    pub(crate) fn build(
        memory: &Memory,
        producer: ProducerId,
        configuration: u64,
        schema: &Schema,
        generation: GenerationId,
        policy: KeyPolicy,
        rows: Vec<Row>,
    ) -> Result<Shared<Self>> {
        schema.validate()?;
        if producer.0.is_nil()
            || configuration == 0
            || generation.0.is_nil()
            || rows
                .len()
                .checked_mul(schema.fields.len())
                .is_none_or(|n| n > MAX_SLOTS)
        {
            return Err(Error::InvalidTarget);
        }
        // Reserve duplicate-key validation/index storage before constructing it.
        let index_bytes =
            rows.len().saturating_mul(128) + rows.iter().map(|r| r.key.len()).sum::<usize>();
        let _scratch = memory.reserve(index_bytes).map_err(|_| Error::Limit)?;
        let mut keys = BTreeMap::new();
        for (i, row) in rows.iter().enumerate() {
            if row.key.is_empty()
                || row.key.len() > MAX_KEY_BYTES
                || row.values.len() != schema.fields.len()
                || keys.insert(row.key.clone(), i).is_some()
            {
                return Err(Error::InvalidTarget);
            }
            for (v, f) in row.values.iter().zip(&schema.fields) {
                let valid = match (v, f.kind) {
                    (Value::Blank, _) => f.nullable,
                    (Value::Decimal(_), ValueKind::Decimal)
                    | (Value::Boolean(_), ValueKind::Boolean) => true,
                    (Value::String(s), ValueKind::String) => s.len() <= MAX_STRING_BYTES,
                    _ => false,
                };
                if !valid {
                    return Err(Error::InvalidTarget);
                }
            }
        }
        let digest = Self::payload_digest(schema.id, policy, &rows);
        memory
            .alloc(Self {
                producer,
                configuration,
                schema: schema.id,
                generation,
                policy,
                digest,
                rows,
                keys,
            })
            .map_err(|_| Error::Limit)
    }
}

/// Installed deterministic providers. The catalog never accepts paths or executable code.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Fixture {
    Ledger,
    Typed,
}
impl Fixture {
    fn field_definitions(self) -> &'static [(&'static str, ValueKind, bool)] {
        const FIELDS: &[(&str, ValueKind, bool)] = &[
            ("Amount", ValueKind::Decimal, false),
            ("Identifier", ValueKind::String, false),
            ("Active", ValueKind::Boolean, false),
            ("Optional", ValueKind::Decimal, true),
        ];
        match self {
            Self::Ledger => &FIELDS[..2],
            Self::Typed => FIELDS,
        }
    }
    fn accepts_schema(self, schema: &Schema) -> bool {
        let expected = self.field_definitions();
        schema.fields.len() == expected.len()
            && schema
                .fields
                .iter()
                .zip(expected)
                .all(|(actual, (label, kind, nullable))| {
                    actual.label == *label && actual.kind == *kind && actual.nullable == *nullable
                })
    }
    pub fn name(self) -> &'static str {
        match self {
            Self::Ledger => "ledger",
            Self::Typed => "typed",
        }
    }
    pub fn schema(self) -> Schema {
        Schema {
            id: SchemaId::fresh(),
            fields: self
                .field_definitions()
                .iter()
                .map(|(label, kind, nullable)| Field {
                    id: FieldId::fresh(),
                    label: (*label).into(),
                    kind: *kind,
                    nullable: *nullable,
                })
                .collect(),
        }
    }
    pub fn rows(self, version: u32) -> Result<Vec<Row>> {
        if version > 2 {
            return Err(Error::InvalidTarget);
        }
        let mut rows = (0..128u32)
            .map(|i| {
                let key = format!("{:04}", i + 12);
                let mut values = vec![
                    Value::Decimal(DecimalInput::from_decimal(PivotDecimal::from(i64::from(
                        i + version + 1,
                    )))),
                    Value::String(key.clone()),
                ];
                if self == Self::Typed {
                    values.push(Value::Boolean(i % 2 == 0));
                    values.push(if i % 2 == 0 {
                        Value::Blank
                    } else {
                        Value::Decimal(DecimalInput::from_decimal(PivotDecimal::ZERO))
                    });
                }
                Row { key, values }
            })
            .collect::<Vec<_>>();
        if version == 1 {
            rows.reverse();
        }
        if version == 2 {
            rows.remove(0);
        }
        Ok(rows)
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum AttemptStatus {
    Never,
    InProgress,
    Succeeded,
    Failed,
    Cancelled,
    Disconnected,
    Interrupted,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Attempt {
    pub sequence: u64,
    pub epoch: u64,
    pub status: AttemptStatus,
    pub last_outcome: AttemptStatus,
    pub(crate) nonce: Uuid,
}
impl Default for Attempt {
    fn default() -> Self {
        Self {
            sequence: 0,
            epoch: 1,
            status: AttemptStatus::Never,
            last_outcome: AttemptStatus::Never,
            nonce: Uuid::nil(),
        }
    }
}
#[derive(Clone)]
pub enum Source {
    Worksheet {
        worksheet: WorksheetId,
        selection: pivot_core::model::selection::SelectionId,
        rectangle: Option<Rectangle>,
    },
    Fixture {
        fixture: Fixture,
        schema: Shared<Schema>,
        policy: KeyPolicy,
        selected: Option<Shared<Snapshot>>,
        attempt: Attempt,
    },
}
#[derive(Clone)]
pub struct Producer {
    pub id: ProducerId,
    pub configuration: u64,
    /// Presentation-only order epoch; does not qualify references or calculations.
    pub layout_order: u64,
    pub retired: bool,
    pub source: Source,
}
impl Producer {
    pub fn extent(&self) -> (u32, u32) {
        extent(self)
    }
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct WindowBinding {
    pub window: WindowId,
    pub producer: ProducerId,
}
#[derive(Clone, Default)]
pub(crate) struct Directory {
    pub producers: Vec<Producer>,
    pub windows: Vec<WindowBinding>,
    pub allocated_identities: usize,
    pool: Option<Shared<AttemptPool>>,
}
#[derive(Clone)]
pub struct SnapshotRecord {
    pub generation: GenerationId,
    pub digest: [u8; 32],
    pub rows: Vec<Row>,
}
#[derive(Clone)]
pub enum SourceRecord {
    Worksheet {
        worksheet: WorksheetId,
        selection: pivot_core::model::selection::SelectionId,
        rectangle: Option<Rectangle>,
    },
    Fixture {
        fixture: Fixture,
        schema: Schema,
        policy: KeyPolicy,
        selected: Option<SnapshotRecord>,
        last_status: AttemptStatus,
    },
}
#[derive(Clone)]
pub struct ProducerRecord {
    pub id: ProducerId,
    pub configuration: u64,
    /// Presentation-only order epoch; does not qualify references or calculations.
    pub layout_order: u64,
    pub retired: bool,
    pub source: SourceRecord,
}
#[derive(Clone)]
pub struct OutputRecord {
    pub key: OutputKey,
    pub portable: GraphNodeId,
    pub slot: usize,
}
#[derive(Clone, Default)]
pub struct Image {
    pub producers: Vec<ProducerRecord>,
    pub windows: Vec<WindowBinding>,
    pub guards: Vec<Guard>,
    pub outputs: Vec<OutputRecord>,
    pub allocated_identities: usize,
}
impl Image {
    pub(crate) fn from_data(data: &persistent::Data, cell_slots: usize) -> Self {
        let Some(d) = &data.producers else {
            return Self::default();
        };
        Self {
            allocated_identities: d.allocated_identities,
            producers: d
                .producers
                .iter()
                .map(|p| ProducerRecord {
                    id: p.id,
                    configuration: p.configuration,
                    layout_order: p.layout_order,
                    retired: p.retired,
                    source: match &p.source {
                        Source::Worksheet {
                            worksheet,
                            selection,
                            rectangle,
                        } => SourceRecord::Worksheet {
                            worksheet: *worksheet,
                            selection: *selection,
                            rectangle: *rectangle,
                        },
                        Source::Fixture {
                            fixture,
                            schema,
                            policy,
                            selected,
                            attempt,
                        } => SourceRecord::Fixture {
                            fixture: *fixture,
                            schema: (**schema).clone(),
                            policy: *policy,
                            last_status: if attempt.status == AttemptStatus::InProgress {
                                AttemptStatus::InProgress
                            } else {
                                attempt.last_outcome
                            },
                            selected: selected.as_ref().map(|s| SnapshotRecord {
                                generation: s.generation,
                                digest: s.digest,
                                rows: s.rows.clone(),
                            }),
                        },
                    },
                })
                .collect(),
            windows: d.windows.clone(),
            guards: data.guards.iter().map(|(_, g)| (**g).clone()).collect(),
            outputs: data
                .outputs
                .iter()
                .enumerate()
                .map(|(i, (key, o))| OutputRecord {
                    key: key.clone(),
                    portable: o.portable,
                    slot: cell_slots + i,
                })
                .collect(),
        }
    }
    pub fn owned_selections(&self) -> Vec<pivot_core::model::selection::SelectionId> {
        self.producers
            .iter()
            .filter_map(|p| {
                if let SourceRecord::Worksheet {
                    selection,
                    rectangle: Some(_),
                    ..
                } = p.source
                {
                    Some(selection)
                } else {
                    None
                }
            })
            .chain(self.guards.iter().filter_map(|g| {
                if let GuardTarget::Range {
                    selection,
                    rectangle: Some(_),
                    ..
                } = g.target
                {
                    Some(selection)
                } else {
                    None
                }
            }))
            .collect::<BTreeSet<_>>()
            .into_iter()
            .collect()
    }
    pub(crate) fn restore_directory(
        &self,
        data: &mut persistent::Data,
        limits: Limits,
    ) -> Result<()> {
        if self.producers.is_empty() {
            if !self.windows.is_empty()
                || !self.guards.is_empty()
                || !self.outputs.is_empty()
                || self.allocated_identities != 0
            {
                return Err(Error::InvalidTarget);
            }
            return Ok(());
        }
        if self.producers.len() > limits.allocated_cells
            || self.guards.len() + self.outputs.len() > limits.allocated_cells
        {
            return Err(Error::Limit);
        }
        let _scratch = data
            .memory
            .reserve(
                self.producers.len().saturating_mul(512) + self.guards.len().saturating_mul(256),
            )
            .map_err(|_| Error::Limit)?;
        let mut ids = BTreeSet::new();
        let mut minimum_ids = 0usize;
        let mut producers = Vec::new();
        for p in &self.producers {
            if !ids.insert(p.id.0) || p.id.0.is_nil() {
                return Err(Error::InvalidTarget);
            }
            minimum_ids += 1;
            let source = match &p.source {
                SourceRecord::Worksheet {
                    worksheet,
                    selection,
                    rectangle,
                } => {
                    if !data
                        .sheets
                        .as_ref()
                        .is_some_and(|s| s.0.iter().any(|s| s.id == (*worksheet).into()))
                        || selection.0 == [0; 16]
                    {
                        return Err(Error::InvalidTarget);
                    }
                    if let Some(r) = rectangle
                        && (r.first.row == 0
                            || r.first.column == 0
                            || r.last.row > limits.rows
                            || r.last.column > limits.columns
                            || r.first.row > r.last.row
                            || r.first.column > r.last.column)
                    {
                        return Err(Error::Position);
                    }
                    Source::Worksheet {
                        worksheet: *worksheet,
                        selection: *selection,
                        rectangle: *rectangle,
                    }
                }
                SourceRecord::Fixture {
                    fixture,
                    schema,
                    policy,
                    selected,
                    last_status,
                } => {
                    // A content digest does not authorize an installed provider schema.
                    // Field identities are durable workbook identities; definitions come
                    // from the fixed provider catalog.
                    if !fixture.accepts_schema(schema) {
                        return Err(Error::InvalidTarget);
                    }
                    if !ids.insert(schema.id.0) {
                        return Err(Error::InvalidTarget);
                    }
                    minimum_ids += 1 + schema.fields.len();
                    for field in &schema.fields {
                        if !ids.insert(field.id.0) {
                            return Err(Error::InvalidTarget);
                        }
                    }
                    let schema = data
                        .memory
                        .alloc(schema.clone())
                        .map_err(|_| Error::Limit)?;
                    let selected = if let Some(s) = selected {
                        if !ids.insert(s.generation.0) {
                            return Err(Error::InvalidTarget);
                        }
                        minimum_ids += 1;
                        let snapshot = Snapshot::build(
                            &data.memory,
                            p.id,
                            p.configuration,
                            &schema,
                            s.generation,
                            *policy,
                            s.rows.clone(),
                        )?;
                        if snapshot.digest != s.digest {
                            return Err(Error::InvalidTarget);
                        }
                        Some(snapshot)
                    } else {
                        None
                    };
                    Source::Fixture {
                        fixture: *fixture,
                        schema,
                        policy: *policy,
                        selected,
                        attempt: Attempt {
                            status: AttemptStatus::Disconnected,
                            last_outcome: if *last_status == AttemptStatus::InProgress {
                                AttemptStatus::Interrupted
                            } else {
                                *last_status
                            },
                            ..Default::default()
                        },
                    }
                }
            };
            producers.push(Producer {
                id: p.id,
                configuration: p.configuration,
                layout_order: p.layout_order,
                retired: p.retired,
                source,
            });
        }
        if self.allocated_identities < minimum_ids {
            return Err(Error::InvalidTarget);
        }
        let directory = Directory {
            producers,
            windows: self.windows.clone(),
            allocated_identities: self.allocated_identities,
            pool: None,
        };
        directory.validate(data, limits)?;
        data.producers = Some(data.memory.alloc(directory).map_err(|_| Error::Limit)?);
        for g in &self.guards {
            if g.id.0.is_nil() || !ids.insert(g.id.0) {
                return Err(Error::InvalidTarget);
            }
            data.guards
                .insert(
                    g.id,
                    data.memory.alloc(g.clone()).map_err(|_| Error::Limit)?,
                )
                .map_err(|_| Error::Limit)?;
        }
        Ok(())
    }
}
struct AttemptPool(AtomicUsize);
impl Footprint for AttemptPool {}
struct Permit {
    pool: Shared<AttemptPool>,
    _charge: Charge,
}
impl Drop for Permit {
    fn drop(&mut self) {
        self.pool.0.fetch_sub(1, Ordering::SeqCst);
    }
}
/// Session-local, one-use authority. It is deliberately not serializable or Clone.
pub struct RefreshTicket {
    session: Uuid,
    producer: ProducerId,
    configuration: u64,
    attempt: Attempt,
    _permit: Permit,
}
impl Footprint for Directory {
    fn heap_bytes(&self) -> usize {
        self.producers.capacity() * std::mem::size_of::<Producer>()
            + self.windows.capacity() * std::mem::size_of::<WindowBinding>()
    }
}
impl Directory {
    pub fn payload_bytes(&self) -> usize {
        self.producers
            .iter()
            .map(|p| match &p.source {
                Source::Fixture {
                    selected: Some(s), ..
                } => s.heap_bytes(),
                _ => 0,
            })
            .sum()
    }
    pub fn producer(&self, id: ProducerId) -> Result<&Producer> {
        self.producers
            .iter()
            .find(|p| p.id == id)
            .ok_or(Error::InvalidTarget)
    }
    pub fn binding(&self, scope: GridScopeId) -> Option<WindowBinding> {
        let GridScopeId::Window(id) = scope else {
            return None;
        };
        self.windows.iter().find(|b| b.window == id).copied()
    }
    pub fn owned_bytes(&self) -> usize {
        self.heap_bytes()
            + self
                .producers
                .iter()
                .map(|p| match &p.source {
                    Source::Fixture {
                        schema, selected, ..
                    } => {
                        std::mem::size_of::<Schema>()
                            + schema.heap_bytes()
                            + selected
                                .as_ref()
                                .map_or(0, |s| std::mem::size_of::<Snapshot>() + s.heap_bytes())
                    }
                    _ => 0,
                })
                .sum::<usize>()
    }
    pub fn validate(&self, data: &persistent::Data, limits: Limits) -> Result<()> {
        if self.producers.iter().filter(|p| !p.retired).count() > MAX_PRODUCERS
            || self.allocated_identities
                + data.records.len()
                + data.guards.len()
                + data.outputs.len()
                > limits.allocated_cells
        {
            return Err(Error::Limit);
        }
        let mut ids = BTreeSet::new();
        let mut slots = 0usize;
        for p in &self.producers {
            if p.id.0.is_nil() || p.configuration == 0 || !ids.insert(p.id) {
                return Err(Error::InvalidTarget);
            }
            if let Source::Fixture {
                fixture,
                schema,
                selected,
                ..
            } = &p.source
            {
                schema.validate()?;
                if !fixture.accepts_schema(schema) {
                    return Err(Error::InvalidTarget);
                }
                if let Some(s) = selected {
                    if s.producer != p.id
                        || s.configuration != p.configuration
                        || s.schema != schema.id
                    {
                        return Err(Error::InvalidTarget);
                    }
                    slots = slots
                        .checked_add(s.rows.len() * schema.fields.len())
                        .ok_or(Error::Limit)?;
                }
            }
        }
        if slots > MAX_SLOTS {
            return Err(Error::Limit);
        }
        let mut windows = BTreeSet::new();
        let mut rows = 0usize;
        for b in &self.windows {
            if !windows.insert(b.window) {
                return Err(Error::InvalidTarget);
            }
            let p = self.producer(b.producer)?;
            let info = data
                .sheets
                .as_ref()
                .and_then(|s| s.0.iter().find(|s| s.id == b.window.into()))
                .ok_or(Error::InvalidTarget)?;
            if info.retired {
                continue;
            }
            if !p.retired {
                rows += match &p.source {
                    Source::Worksheet { rectangle, .. } => {
                        rectangle.map_or(0, |r| (r.last.row - r.first.row + 1) as usize)
                    }
                    Source::Fixture { selected, .. } => {
                        selected.as_ref().map_or(0, |s| s.rows.len())
                    }
                };
            }
        }
        if rows > MAX_VIEW_ROWS
            || self.owned_bytes() > limits.bytes
            || data.stats().live_nodes > limits.live_cells
        {
            return Err(Error::Limit);
        }
        Ok(())
    }
}

pub(crate) fn sync_dimensions(data: &mut persistent::Data) -> Result<()> {
    let Some(producers) = &data.producers else {
        return Ok(());
    };
    let _scratch = data
        .memory
        .reserve(data.sheets.as_ref().map_or(0, |s| s.heap_bytes()) + 4096)
        .map_err(|_| Error::Limit)?;
    let mut scopes = data.sheets.as_ref().ok_or(Error::InvalidTarget)?.0.clone();
    let mut changed = false;
    for b in &producers.windows {
        let s = scopes
            .iter_mut()
            .find(|s| s.id == b.window.into())
            .ok_or(Error::InvalidTarget)?;
        if s.retired {
            continue;
        }
        let (rows, columns) = extent(producers.producer(b.producer)?);
        let (rows, columns) = (rows.max(1), columns.max(1));
        if s.rows.length != rows || s.columns.length != columns {
            s.rows.length = rows;
            s.columns.length = columns;
            changed = true;
        }
    }
    if changed {
        data.install_sheets(scopes)?;
    }
    Ok(())
}
fn extent(p: &Producer) -> (u32, u32) {
    if p.retired {
        return (0, 0);
    }
    match &p.source {
        Source::Worksheet {
            rectangle: Some(r), ..
        } => (
            r.last.row - r.first.row + 1,
            r.last.column - r.first.column + 1,
        ),
        Source::Fixture {
            schema, selected, ..
        } => (
            selected.as_ref().map_or(0, |s| s.rows.len() as u32),
            schema.fields.len() as u32,
        ),
        _ => (0, 0),
    }
}
pub(crate) fn transform_sources(
    data: &mut persistent::Data,
    sheet: GridScopeId,
    change: structure::Change,
    limit: u32,
) -> Result<
    Vec<(
        pivot_core::model::selection::SelectionId,
        Option<pivot_core::model::selection::Rectangle>,
    )>,
> {
    let Some(old) = &data.producers else {
        return Ok(Vec::new());
    };
    let _scratch = data
        .memory
        .reserve(old.heap_bytes() + 4096)
        .map_err(|_| Error::Limit)?;
    let mut directory = (**old).clone();
    let mut changed = Vec::new();
    for p in &mut directory.producers {
        if p.retired {
            continue;
        }
        let Source::Worksheet {
            worksheet,
            selection,
            rectangle,
        } = &mut p.source
        else {
            continue;
        };
        if GridScopeId::from(*worksheet) != sheet {
            continue;
        }
        let Some(mut r) = *rectangle else { continue };
        let (a, b) = match change.axis {
            structure::AxisKind::Rows => (r.first.row, r.last.row),
            structure::AxisKind::Columns => (r.first.column, r.last.column),
        };
        match change.range(a, b, limit) {
            Some((new_a, new_b)) => {
                if (a, b) == (new_a, new_b) {
                    continue;
                }
                match change.axis {
                    structure::AxisKind::Rows => {
                        r.first.row = new_a;
                        r.last.row = new_b;
                    }
                    structure::AxisKind::Columns => {
                        r.first.column = new_a;
                        r.last.column = new_b;
                    }
                }
                *rectangle = Some(r);
            }
            None => *rectangle = None,
        }
        changed.push((
            *selection,
            rectangle.map(|r| pivot_core::model::selection::Rectangle {
                space: pivot_core::model::selection::SpaceId(*worksheet.uuid().as_bytes()),
                first_row: r.first.row,
                last_row: r.last.row,
                first_column: r.first.column,
                last_column: r.last.column,
            }),
        ));
    }
    let _guard_scratch = data
        .memory
        .reserve(data.guards.len().saturating_mul(256))
        .map_err(|_| Error::Limit)?;
    let mut guard_updates = Vec::new();
    for (id, g) in data.guards.iter() {
        let p = directory.producer(g.producer)?;
        if !matches!(p.source,Source::Worksheet { worksheet,.. } if GridScopeId::from(worksheet)==sheet)
        {
            continue;
        }
        let GuardTarget::Range {
            selection,
            rectangle: Some(mut r),
            kind,
        } = g.target
        else {
            continue;
        };
        let (a, b) = match change.axis {
            structure::AxisKind::Rows => (r.first.row, r.last.row),
            structure::AxisKind::Columns => (r.first.column, r.last.column),
        };
        let mapped = change.range(a, b, limit);
        if mapped == Some((a, b)) {
            continue;
        }
        let rectangle = mapped.map(|(a, b)| {
            match change.axis {
                structure::AxisKind::Rows => {
                    r.first.row = a;
                    r.last.row = b;
                }
                structure::AxisKind::Columns => {
                    r.first.column = a;
                    r.last.column = b;
                }
            }
            r
        });
        let mut update = (**g).clone();
        update.target = GuardTarget::Range {
            selection,
            rectangle,
            kind,
        };
        guard_updates.push((*id, update));
        changed.push((
            selection,
            rectangle.map(|r| pivot_core::model::selection::Rectangle {
                space: pivot_core::model::selection::SpaceId(*sheet.uuid().as_bytes()),
                first_row: r.first.row,
                last_row: r.last.row,
                first_column: r.first.column,
                last_column: r.last.column,
            }),
        ));
    }
    for (id, g) in guard_updates {
        data.guards
            .insert(id, data.memory.alloc(g).map_err(|_| Error::Limit)?)
            .map_err(|_| Error::Limit)?;
    }
    if !changed.is_empty() {
        data.producers = Some(data.memory.alloc(directory).map_err(|_| Error::Limit)?);
        sync_dimensions(data)?;
    }
    Ok(changed)
}
pub struct ProjectedFrame {
    pub window: WindowId,
    pub expected: Expected,
    pub producer: ProducerId,
    pub configuration: u64,
    pub generation: Option<GenerationId>,
    pub rows: u32,
    pub columns: u32,
    pub attempt: Option<AttemptStatus>,
    pub cells: Vec<ProjectedCell>,
    _charge: Charge,
}
pub struct ProjectedCell {
    pub position: Position,
    pub source: Option<CellRef>,
    pub value: std::result::Result<pivot_types::PivotValue, CellError>,
    pub decimal: Option<DecimalInput>,
    pub style: formatting::Style,
    pub source_hidden: bool,
    pub item_key: String,
    pub field_id: String,
}
impl CalculationCapture {
    pub fn project_window(&self, window: WindowId, area: Rectangle) -> Result<ProjectedFrame> {
        let directory = self.pinned.producers.as_ref().ok_or(Error::InvalidTarget)?;
        let info = self
            .pinned
            .sheets
            .as_ref()
            .and_then(|s| s.0.iter().find(|s| s.id == window.into() && !s.retired))
            .ok_or(Error::InvalidTarget)?;
        let _ = info;
        let b = directory
            .binding(window.into())
            .ok_or(Error::InvalidTarget)?;
        let p = directory.producer(b.producer)?;
        let (rows, columns) = extent(p);
        let count = u64::from(
            area.last
                .row
                .checked_sub(area.first.row)
                .ok_or(Error::Position)?
                + 1,
        ) * u64::from(
            area.last
                .column
                .checked_sub(area.first.column)
                .ok_or(Error::Position)?
                + 1,
        );
        if area.first.row == 0 || area.first.column == 0 || count > self.limit as u64 {
            return Err(Error::Limit);
        }
        let (generation, attempt) = match &p.source {
            Source::Fixture {
                selected, attempt, ..
            } => (
                selected.as_ref().map(|s| s.generation),
                Some(attempt.status),
            ),
            _ => (None, None),
        };
        let charge = self
            .pinned
            .memory
            .reserve((count as usize).saturating_mul(MAX_STRING_BYTES + 256))
            .map_err(|_| Error::Limit)?;
        let mut frame = ProjectedFrame {
            window,
            expected: self.expected,
            producer: p.id,
            configuration: p.configuration,
            generation,
            rows,
            columns,
            attempt,
            cells: Vec::with_capacity(count as usize),
            _charge: charge,
        };
        if rows == 0 || columns == 0 {
            return Ok(frame);
        }
        if area.last.row > rows || area.last.column > columns {
            return Err(Error::Position);
        }
        let mut outputs = Vec::new();
        let source_visibility = if let Source::Worksheet { worksheet, .. } = p.source {
            self.pinned
                .filters
                .get(&worksheet.into())
                .map(|d| self.filter_visibility_on(worksheet.into(), Some(d), &Default::default()))
                .transpose()?
        } else {
            None
        };
        for row in area.first.row..=area.last.row {
            for column in area.first.column..=area.last.column {
                let position = Position::new(row, column);
                let mut cell = ProjectedCell {
                    position,
                    source: None,
                    value: Ok(pivot_types::PivotValue::Blank),
                    decimal: None,
                    style: Default::default(),
                    source_hidden: false,
                    item_key: String::new(),
                    field_id: String::new(),
                };
                match &p.source {
                    Source::Worksheet {
                        worksheet,
                        rectangle: Some(r),
                        ..
                    } => {
                        let source_at =
                            Position::new(r.first.row + row - 1, r.first.column + column - 1);
                        cell.style = self.style_at((*worksheet).into(), source_at)?;
                        cell.source_hidden = source_visibility
                            .as_ref()
                            .is_some_and(|v| v.rows().binary_search(&source_at.row).is_err());
                        let scope = self
                            .pinned
                            .sheets
                            .as_ref()
                            .and_then(|s| s.0.iter().find(|s| s.id == (*worksheet).into()))
                            .ok_or(Error::InvalidTarget)?;
                        let row_id = scope.rows.at(source_at.row)?;
                        let field = scope.columns.at(source_at.column)?;
                        cell.item_key = format!("axis:{}:{}", row_id.namespace, row_id.sequence);
                        cell.field_id = format!("axis:{}:{}", field.namespace, field.sequence);
                        let source = self.cell_at_sheet((*worksheet).into(), source_at);
                        cell.source = source;
                        if let Some(source) = source {
                            if let Some(record) = self.pinned.records.get(&source.cell)
                                && let Some(Content::Decimal(d)) = &record.content
                            {
                                cell.decimal = Some(*d);
                            }
                            outputs.push(source);
                        }
                    }
                    Source::Fixture {
                        schema,
                        selected: Some(s),
                        ..
                    } => {
                        cell.item_key = s.rows[(row - 1) as usize].key.clone();
                        cell.field_id = schema.fields[(column - 1) as usize].id.0.to_string();
                        let v = &s.rows[(row - 1) as usize].values[(column - 1) as usize];
                        cell.value = Ok(v.typed());
                        if let Value::Decimal(d) = v {
                            cell.decimal = Some(*d);
                        }
                    }
                    _ => (),
                }
                frame.cells.push(cell);
            }
        }
        let values = self.evaluate_values(&outputs)?.values;
        let mut values = values.into_iter();
        for cell in &mut frame.cells {
            if cell.source.is_some() {
                cell.value = values.next().ok_or(Error::InvalidTarget)?.1;
            }
        }
        Ok(frame)
    }
}

impl Workbook {
    pub fn producer_inbound_references(
        &self,
        id: ProducerId,
        cancel: &pivot_core::model::integration::CancellationToken,
    ) -> Result<usize> {
        self.persistent
            .producers
            .as_ref()
            .ok_or(Error::InvalidTarget)?
            .producer(id)?;
        let mut count = 0usize;
        let mut work = 0usize;
        for (_, r) in self.persistent.records.iter() {
            if r.position.is_none() {
                continue;
            }
            let tokens = match &r.content {
                Some(Content::Formula { tokens, .. }) => tokens.as_slice(),
                Some(Content::TextFormula(f)) => f.tokens.as_slice(),
                _ => continue,
            };
            for token in tokens {
                work += 1;
                if work > structure::MAX_WORK {
                    return Err(Error::Limit);
                }
                if cancel.is_cancelled() {
                    return Err(Error::Graph("cancelled".into()));
                }
                if let Token::Bound(g) = token
                    && self
                        .persistent
                        .guards
                        .get(g)
                        .is_some_and(|g| g.producer == id)
                {
                    count += 1;
                }
            }
        }
        Ok(count)
    }
    pub fn retire_producer(
        &mut self,
        expected: Expected,
        id: ProducerId,
        cancel: &pivot_core::model::integration::CancellationToken,
    ) -> Result<Expected> {
        self.check(expected)?;
        if cancel.is_cancelled() {
            return Err(Error::Graph("cancelled".into()));
        }
        let (mut directory, _scratch) = self.producer_directory_copy()?;
        let p = directory
            .producers
            .iter_mut()
            .find(|p| p.id == id && !p.retired)
            .ok_or(Error::InvalidTarget)?;
        p.retired = true;
        if let Source::Fixture { attempt, .. } = &mut p.source {
            attempt.epoch = attempt.epoch.checked_add(1).ok_or(Error::Exhausted)?;
            if attempt.status == AttemptStatus::InProgress {
                attempt.last_outcome = AttemptStatus::Cancelled;
            }
            attempt.status = AttemptStatus::Disconnected;
            attempt.nonce = Uuid::nil();
        }
        self.install_producers_cancellable(directory, cancel)
    }
    pub fn detach_bound_window(
        &mut self,
        expected: Expected,
        window: WindowId,
        name: &str,
        cancel: &pivot_core::model::integration::CancellationToken,
    ) -> Result<WindowId> {
        self.check(expected)?;
        let binding = self
            .window_binding(window.into())
            .ok_or(Error::InvalidTarget)?;
        let producer = self
            .persistent
            .producers
            .as_ref()
            .ok_or(Error::InvalidTarget)?
            .producer(binding.producer)?;
        let (rows, columns) = extent(producer);
        if rows == 0
            || columns == 0
            || u64::from(rows) * u64::from(columns) > self.limits.edits as u64
        {
            return Err(Error::Limit);
        }
        let capture = self.capture_local(expected)?;
        let frame = capture.project_window(
            window,
            Rectangle {
                first: Position::new(1, 1),
                last: Position::new(rows, columns),
            },
        )?;
        let mut edits = Vec::new();
        for cell in &frame.cells {
            if cancel.is_cancelled() {
                return Err(Error::Graph("cancelled".into()));
            }
            edits.push(match &cell.value {
                Ok(pivot_types::PivotValue::Blank) => Edit::Clear(cell.position),
                Ok(pivot_types::PivotValue::String(s)) => Edit::SetText {
                    position: cell.position,
                    text: s.clone(),
                },
                Ok(pivot_types::PivotValue::Boolean(value)) => Edit::SetBoolean {
                    position: cell.position,
                    value: *value,
                },
                Ok(pivot_types::PivotValue::Decimal(value)) => Edit::SetDecimal {
                    position: cell.position,
                    value: cell
                        .decimal
                        .unwrap_or_else(|| DecimalInput::from_decimal(*value)),
                },
                _ => {
                    return Err(Error::Graph(
                        "Detach requires exact supported values without errors.".into(),
                    ));
                }
            });
        }
        let mut candidate = self.request_view(self.worksheet)?;
        let id = candidate.create_grid_window(expected, name, rows, columns)?;
        candidate.apply_scope(id.into(), candidate.expected(), &edits)?;
        if candidate.persistent.geometry.contains_key(&window.into()) {
            let mut data = candidate
                .core
                .stage_resource(&candidate.persistent)
                .map_err(graph_error)?;
            data.detach_geometry(window.into(), id.into(), cancel)?;
            data.check_geometry_limits(candidate.limits)?;
            candidate.persistent = data.freeze();
            candidate.state = candidate.state.checked_add(1).ok_or(Error::Exhausted)?;
        }
        for cell in &frame.cells {
            if cell.style == formatting::Style::default() {
                continue;
            }
            use formatting::Field::Set;
            let s = cell.style;
            candidate.format_sheet(
                id.into(),
                candidate.expected(),
                Rectangle {
                    first: cell.position,
                    last: cell.position,
                },
                formatting::StylePatch {
                    numeric: Set(s.numeric),
                    places: Set(s.places),
                    rounding: Set(s.rounding),
                    grouping: Set(s.grouping),
                    currency: Set(s.currency),
                    bold: Set(s.bold),
                    italic: Set(s.italic),
                    alignment: Set(s.alignment),
                    foreground: Set(s.foreground),
                    background: Set(s.background),
                },
                false,
                cancel,
            )?;
        }
        if cancel.is_cancelled() {
            return Err(Error::Graph("cancelled".into()));
        }
        candidate.state = self.state.checked_add(1).ok_or(Error::Exhausted)?;
        self.core = candidate.core;
        self.persistent = candidate.persistent;
        self.state = candidate.state;
        Ok(id)
    }
    pub fn producers(&self) -> &[Producer] {
        self.persistent
            .producers
            .as_ref()
            .map_or(&[], |d| d.producers.as_slice())
    }
    pub fn window_binding(&self, scope: GridScopeId) -> Option<WindowBinding> {
        self.persistent.producers.as_ref()?.binding(scope)
    }
    pub(crate) fn require_editable(&self, scope: GridScopeId) -> Result<()> {
        if self.window_binding(scope).is_some() {
            return Err(Error::Graph(
                "Bound Grid Windows are read-only; edit the source worksheet instead.".into(),
            ));
        }
        Ok(())
    }
    fn producer_directory_copy(&self) -> Result<(Directory, Charge)> {
        let bytes = self
            .persistent
            .producers
            .as_ref()
            .map_or(4096, |p| p.heap_bytes() + 4096);
        let scratch = self
            .persistent
            .memory
            .reserve(bytes)
            .map_err(|_| Error::Limit)?;
        Ok((
            self.persistent
                .producers
                .as_ref()
                .map_or_else(Directory::default, |p| (**p).clone()),
            scratch,
        ))
    }
    fn install_producers(&mut self, directory: Directory) -> Result<Expected> {
        self.install_producers_cancellable(directory, &Default::default())
    }
    fn install_producers_cancellable(
        &mut self,
        directory: Directory,
        cancel: &pivot_core::model::integration::CancellationToken,
    ) -> Result<Expected> {
        if cancel.is_cancelled() {
            return Err(Error::Graph("cancelled".into()));
        }
        directory.validate(&self.persistent, self.limits)?;
        if directory.payload_bytes() + self.stats().source_bytes
            - self
                .persistent
                .producers
                .as_ref()
                .map_or(0, |p| p.payload_bytes())
            > self.limits.total_source_bytes
        {
            return Err(Error::Limit);
        }
        let next = self.state.checked_add(1).ok_or(Error::Exhausted)?;
        let mut core = self.core.stage();
        let mut data = core.stage_resource(&self.persistent).map_err(graph_error)?;
        let selected = |directory: Option<&Directory>, id: ProducerId| {
            directory.and_then(|d| d.producer(id).ok()).and_then(|p| {
                if let Source::Fixture { selected, .. } = &p.source {
                    Some((
                        p.retired,
                        p.configuration,
                        selected.as_ref().map(|s| s.generation),
                    ))
                } else {
                    None
                }
            })
        };
        let mut batch = core
            .begin_batch(data.outputs.len(), 0)
            .map_err(graph_error)?;
        for (key, output) in data.outputs.iter() {
            if selected(self.persistent.producers.as_deref(), key.producer)
                != selected(Some(&directory), key.producer)
            {
                batch
                    .replace_captured_scalar(output.node, output.slot)
                    .map_err(graph_error)?;
            }
        }
        if !batch.is_empty() {
            core.apply_batch(batch, cancel).map_err(graph_error)?;
        }
        data.producers = Some(data.memory.alloc(directory).map_err(|_| Error::Limit)?);
        sync_dimensions(&mut data)?;
        maintain_guards(&mut core, &mut data, cancel)?;
        crate::windows::validate_scope_graph(&core, &data, cancel)?;
        if data.stats().estimated_owned_bytes > self.limits.bytes {
            return Err(Error::Limit);
        }
        if cancel.is_cancelled() {
            return Err(Error::Graph("cancelled".into()));
        }
        self.core = core;
        self.persistent = data.freeze();
        self.state = next;
        Ok(self.expected())
    }
    pub fn create_fixture_producer(
        &mut self,
        expected: Expected,
        fixture: Fixture,
        policy: KeyPolicy,
    ) -> Result<ProducerId> {
        self.check(expected)?;
        #[cfg(feature = "history-experiment")]
        if self.history_active || self.history_recording.is_some() {
            return Err(Error::InvalidTarget);
        }
        if self.is_structured_proof() {
            return Err(Error::InvalidTarget);
        }
        let (mut directory, _scratch) = self.producer_directory_copy()?;
        let id = ProducerId::fresh();
        let schema = self
            .persistent
            .memory
            .alloc(fixture.schema())
            .map_err(|_| Error::Limit)?;
        directory.allocated_identities += 2 + schema.fields.len();
        directory.producers.push(Producer {
            id,
            configuration: 1,
            layout_order: 0,
            retired: false,
            source: Source::Fixture {
                fixture,
                schema,
                policy,
                selected: None,
                attempt: Attempt::default(),
            },
        });
        self.install_producers(directory)?;
        Ok(id)
    }
    pub fn create_worksheet_producer(
        &mut self,
        expected: Expected,
        worksheet: WorksheetId,
        rectangle: Rectangle,
    ) -> Result<ProducerId> {
        self.check(expected)?;
        #[cfg(feature = "history-experiment")]
        if self.history_active || self.history_recording.is_some() {
            return Err(Error::InvalidTarget);
        }
        let scope: GridScopeId = worksheet.into();
        self.axes_at_sheet(scope, rectangle.first)?;
        self.axes_at_sheet(scope, rectangle.last)?;
        if rectangle.first.row > rectangle.last.row
            || rectangle.first.column > rectangle.last.column
            || self.is_structured_proof()
        {
            return Err(Error::InvalidTarget);
        }
        let (mut directory, _scratch) = self.producer_directory_copy()?;
        let selection = pivot_core::model::selection::SelectionId(*Uuid::new_v4().as_bytes());
        let mut core = self.core.stage();
        let mut batch = core.begin_batch(2, 192).map_err(graph_error)?;
        let selection = batch
            .intern_selection(
                selection,
                pivot_core::model::selection::Rectangle {
                    space: pivot_core::model::selection::SpaceId(*worksheet.uuid().as_bytes()),
                    first_row: rectangle.first.row,
                    last_row: rectangle.last.row,
                    first_column: rectangle.first.column,
                    last_column: rectangle.last.column,
                },
            )
            .map_err(graph_error)?;
        batch
            .retain_selection(selection, true)
            .map_err(graph_error)?;
        core.apply_batch(batch, &Default::default())
            .map_err(graph_error)?;
        let id = ProducerId::fresh();
        directory.allocated_identities += 1;
        directory.producers.push(Producer {
            id,
            configuration: 1,
            layout_order: 0,
            retired: false,
            source: Source::Worksheet {
                worksheet,
                selection,
                rectangle: Some(rectangle),
            },
        });
        // Stage both directory and descriptor before publishing either.
        let mut candidate = self.request_view(self.worksheet)?;
        candidate.core = core;
        candidate.install_producers(directory)?;
        self.core = candidate.core;
        self.persistent = candidate.persistent;
        self.state = candidate.state;
        Ok(id)
    }
    pub fn create_bound_window(
        &mut self,
        expected: Expected,
        name: &str,
        producer: ProducerId,
    ) -> Result<WindowId> {
        self.check(expected)?;
        let (mut directory, _scratch) = self.producer_directory_copy()?;
        let p = directory.producer(producer)?;
        if p.retired {
            return Err(Error::InvalidTarget);
        }
        let (rows, columns) = match &p.source {
            Source::Worksheet {
                rectangle: Some(r), ..
            } => (
                r.last.row - r.first.row + 1,
                r.last.column - r.first.column + 1,
            ),
            Source::Fixture {
                schema, selected, ..
            } => (
                selected
                    .as_ref()
                    .map_or(1, |s| (s.rows.len() as u32).max(1)),
                schema.fields.len() as u32,
            ),
            _ => return Err(Error::InvalidTarget),
        };
        let mut candidate = self.request_view(self.worksheet)?;
        let id = candidate.create_grid_window(expected, name, rows, columns)?;
        directory.windows.push(WindowBinding {
            window: id,
            producer,
        });
        candidate.install_producers(directory)?;
        candidate.state = self.state.checked_add(1).ok_or(Error::Exhausted)?;
        self.core = candidate.core;
        self.persistent = candidate.persistent;
        self.state = candidate.state;
        Ok(id)
    }
    pub fn begin_refresh(
        &mut self,
        expected: Expected,
        producer: ProducerId,
    ) -> Result<RefreshTicket> {
        self.check(expected)?;
        let (mut directory, _scratch) = self.producer_directory_copy()?;
        if directory.pool.is_none() {
            directory.pool = Some(
                self.persistent
                    .memory
                    .alloc(AttemptPool(AtomicUsize::new(0)))
                    .map_err(|_| Error::Limit)?,
            );
        }
        let pool = directory.pool.as_ref().unwrap().clone();
        let charge = self
            .persistent
            .memory
            .reserve(8192)
            .map_err(|_| Error::Limit)?;
        pool.0
            .try_update(Ordering::SeqCst, Ordering::SeqCst, |n| {
                (n < 2).then_some(n + 1)
            })
            .map_err(|_| Error::Limit)?;
        let permit = Permit {
            pool,
            _charge: charge,
        };
        let p = directory
            .producers
            .iter_mut()
            .find(|p| p.id == producer && !p.retired)
            .ok_or(Error::InvalidTarget)?;
        let Source::Fixture { attempt, .. } = &mut p.source else {
            return Err(Error::InvalidTarget);
        };
        if attempt.status == AttemptStatus::Disconnected {
            return Err(Error::InvalidTarget);
        }
        attempt.sequence = attempt.sequence.checked_add(1).ok_or(Error::Exhausted)?;
        attempt.nonce = Uuid::new_v4();
        attempt.status = AttemptStatus::InProgress;
        let ticket = RefreshTicket {
            session: self.session,
            producer,
            configuration: p.configuration,
            attempt: *attempt,
            _permit: permit,
        };
        self.install_producers(directory)?;
        Ok(ticket)
    }
    fn refresh_matches(&self, ticket: &RefreshTicket, p: &Producer) -> bool {
        self.session == ticket.session
            && !p.retired
            && p.id == ticket.producer
            && p.configuration == ticket.configuration
            && matches!(&p.source, Source::Fixture { attempt, .. } if *attempt == ticket.attempt && attempt.status == AttemptStatus::InProgress)
    }
    pub fn finish_refresh(
        &mut self,
        ticket: &RefreshTicket,
        rows: Option<Vec<Row>>,
        cancelled: bool,
    ) -> Result<Expected> {
        let (mut directory, _scratch) = self.producer_directory_copy()?;
        let p = directory
            .producers
            .iter_mut()
            .find(|p| p.id == ticket.producer)
            .ok_or(Error::InvalidTarget)?;
        if !self.refresh_matches(ticket, p) {
            return Err(Error::StaleRevision);
        }
        let Source::Fixture {
            schema,
            policy,
            selected,
            attempt,
            ..
        } = &mut p.source
        else {
            return Err(Error::InvalidTarget);
        };
        if cancelled {
            attempt.status = AttemptStatus::Cancelled;
        } else if let Some(rows) = rows {
            let snapshot = Snapshot::build(
                &self.persistent.memory,
                p.id,
                p.configuration,
                schema,
                GenerationId::fresh(),
                *policy,
                rows,
            )?;
            *selected = Some(snapshot);
            directory.allocated_identities += 1;
            attempt.status = AttemptStatus::Succeeded;
        } else {
            attempt.status = AttemptStatus::Failed;
        }
        attempt.last_outcome = attempt.status;
        self.install_producers(directory)
    }
    pub fn refresh_fixture(&mut self, ticket: &RefreshTicket, version: u32) -> Result<Expected> {
        let p = self
            .persistent
            .producers
            .as_ref()
            .ok_or(Error::InvalidTarget)?
            .producer(ticket.producer)?;
        if !self.refresh_matches(ticket, p) {
            return Err(Error::StaleRevision);
        }
        let Source::Fixture { fixture, .. } = p.source else {
            return Err(Error::InvalidTarget);
        };
        let _provider_buffer = self
            .persistent
            .memory
            .reserve(128 * (4096 + 256))
            .map_err(|_| Error::Limit)?;
        self.finish_refresh(ticket, Some(fixture.rows(version)?), false)
    }
    pub fn disconnect_producer(&mut self, expected: Expected, id: ProducerId) -> Result<Expected> {
        self.check(expected)?;
        let (mut directory, _scratch) = self.producer_directory_copy()?;
        let p = directory
            .producers
            .iter_mut()
            .find(|p| p.id == id && !p.retired)
            .ok_or(Error::InvalidTarget)?;
        let Source::Fixture { attempt, .. } = &mut p.source else {
            return Err(Error::InvalidTarget);
        };
        attempt.epoch = attempt.epoch.checked_add(1).ok_or(Error::Exhausted)?;
        attempt.nonce = Uuid::nil();
        attempt.status = AttemptStatus::Disconnected;
        self.install_producers(directory)
    }
    pub fn reconnect_producer(
        &mut self,
        expected: Expected,
        id: ProducerId,
        fixture: Fixture,
    ) -> Result<Expected> {
        self.check(expected)?;
        let (mut directory, _scratch) = self.producer_directory_copy()?;
        let p = directory
            .producers
            .iter_mut()
            .find(|p| p.id == id && !p.retired)
            .ok_or(Error::InvalidTarget)?;
        let Source::Fixture {
            fixture: declared,
            attempt,
            ..
        } = &mut p.source
        else {
            return Err(Error::InvalidTarget);
        };
        if *declared != fixture || attempt.status != AttemptStatus::Disconnected {
            return Err(Error::InvalidTarget);
        }
        attempt.epoch = attempt.epoch.checked_add(1).ok_or(Error::Exhausted)?;
        attempt.status = AttemptStatus::Never;
        self.install_producers(directory)
    }
}

impl CalculationCapture {
    pub fn producer(&self, id: ProducerId) -> Option<&Producer> {
        self.pinned
            .producers
            .as_ref()?
            .producers
            .iter()
            .find(|p| p.id == id)
    }
    pub(crate) fn prepare_producer_context(&mut self) -> Result<()> {
        if self.pinned.outputs.is_empty() {
            return Ok(());
        }
        use pivot_core::captured::{CapturedFaultId, ScalarValue};
        let _scratch = self
            .pinned
            .memory
            .reserve(self.pinned.outputs.len().saturating_mul(256))
            .map_err(|_| Error::Limit)?;
        let directory = self.pinned.producers.as_ref().ok_or(Error::InvalidTarget)?;
        let mut values = Vec::with_capacity(self.pinned.outputs.len());
        let mut faults = Vec::new();
        for (key, output) in self.pinned.outputs.iter() {
            let p = directory.producer(key.producer)?;
            let decimal = if !p.retired && p.configuration == key.configuration {
                if let Source::Fixture {
                    schema,
                    selected: Some(s),
                    policy,
                    ..
                } = &p.source
                {
                    let matching_generation = match policy {
                        KeyPolicy::Durable => key.generation.is_none(),
                        KeyPolicy::GenerationQualified => key.generation == Some(s.generation),
                    };
                    schema
                        .fields
                        .iter()
                        .position(|f| f.id == key.field && f.kind == ValueKind::Decimal)
                        .filter(|_| matching_generation)
                        .and_then(|column| s.row(&key.key).and_then(|row| row.values.get(column)))
                        .and_then(|v| {
                            if let Value::Decimal(d) = v {
                                Some(d.value())
                            } else {
                                None
                            }
                        })
                } else {
                    None
                }
            } else {
                None
            };
            let value = if let Some(d) = decimal {
                ScalarValue::Decimal(d)
            } else {
                let fault = CapturedFaultId(*output.portable.uuid().as_bytes());
                faults.push(fault);
                ScalarValue::Fault(fault)
            };
            values.push((output.slot, value));
        }
        self.producer_context = Some(
            self.local_core
                .scalar_context(&values, &faults)
                .map_err(graph_error)?,
        );
        Ok(())
    }
}

/// Bounded presentation work: axis search levels and SHA-256 input blocks.
pub(crate) fn layout_work(data: &persistent::Data, scope: GridScopeId) -> Result<usize> {
    let Some(directory) = &data.producers else {
        return Ok(0);
    };
    let Some(binding) = directory.binding(scope) else {
        return Ok(0);
    };
    let producer = directory.producer(binding.producer)?;
    Ok(128
        + match &producer.source {
            Source::Fixture {
                schema, selected, ..
            } => {
                schema
                    .fields
                    .iter()
                    .map(|f| f.label.len().div_ceil(64) + 2)
                    .sum::<usize>()
                    + selected.as_ref().map_or(0, |snapshot| {
                        snapshot
                            .rows
                            .iter()
                            .map(|r| r.key.len().div_ceil(64) + 2)
                            .sum()
                    })
            }
            Source::Worksheet {
                worksheet,
                rectangle,
                ..
            } => {
                let Some(rect) = rectangle else {
                    return Ok(128);
                };
                let axes = data
                    .sheets
                    .as_ref()
                    .and_then(|d| d.0.iter().find(|s| s.id == GridScopeId::from(*worksheet)))
                    .ok_or(Error::InvalidTarget)?;
                let visits = |axis: &crate::axes::Axis| {
                    let n = axis.layout.as_ref().map_or(1, |p| p.runs.len());
                    (usize::BITS - n.leading_zeros()) as usize + 2
                };
                (rect.last.row - rect.first.row + 1) as usize * visits(&axes.rows)
                    + (rect.last.column - rect.first.column + 1) as usize * visits(&axes.columns)
            }
        })
}
/// Stream a presentation identity without values, refresh attempts or generation IDs.
/// Storage remains constant; source dimensions/keys are already independently bounded.
pub(crate) fn layout_stamp(
    data: &persistent::Data,
    scope: GridScopeId,
    cancel: &CancellationToken,
) -> Result<Option<[u8; 32]>> {
    let Some(directory) = &data.producers else {
        return Ok(None);
    };
    let Some(binding) = directory.binding(scope) else {
        return Ok(None);
    };
    let producer = directory.producer(binding.producer)?;
    let mut hash = Sha256::new();
    hash.update(b"Pivot positional layout v1");
    hash.update(producer.id.0.as_bytes());
    hash.update(producer.configuration.to_le_bytes());
    hash.update(producer.layout_order.to_le_bytes());
    match &producer.source {
        Source::Fixture {
            schema, selected, ..
        } => {
            hash.update([1]);
            hash.update(schema.id.0.as_bytes());
            for field in &schema.fields {
                hash.update(field.id.0.as_bytes());
                hash.update((field.label.len() as u64).to_le_bytes());
                hash.update(field.label.as_bytes());
                hash.update([field.kind as u8, u8::from(field.nullable)]);
            }
            hash.update([u8::from(selected.is_some())]);
            if let Some(snapshot) = selected {
                for row in &snapshot.rows {
                    if cancel.is_cancelled() {
                        return Err(Error::Graph("cancelled layout identity".into()));
                    }
                    hash.update((row.key.len() as u64).to_le_bytes());
                    hash.update(row.key.as_bytes());
                }
            }
        }
        Source::Worksheet {
            worksheet,
            rectangle,
            ..
        } => {
            hash.update([0]);
            hash.update(worksheet.uuid().as_bytes());
            hash.update([u8::from(rectangle.is_some())]);
            if let Some(rect) = rectangle {
                let axes = data
                    .sheets
                    .as_ref()
                    .and_then(|d| d.0.iter().find(|s| s.id == GridScopeId::from(*worksheet)))
                    .ok_or(Error::InvalidTarget)?;
                for (axis, first, last) in [
                    (&axes.rows, rect.first.row, rect.last.row),
                    (&axes.columns, rect.first.column, rect.last.column),
                ] {
                    hash.update((last - first + 1).to_le_bytes());
                    for rank in first..=last {
                        if cancel.is_cancelled() {
                            return Err(Error::Graph("cancelled layout identity".into()));
                        }
                        let id = axis.at(rank)?;
                        hash.update(id.namespace.as_bytes());
                        hash.update(id.sequence.to_le_bytes());
                    }
                }
            }
        }
    }
    Ok(Some(hash.finalize().into()))
}
/// A worksheet sort changes record order while authored axis identities remain stable.
pub(crate) fn note_layout_sort(
    data: &mut persistent::Data,
    scope: GridScopeId,
    rectangle: Rectangle,
) -> Result<()> {
    let Some(old) = &data.producers else {
        return Ok(());
    };
    let affected = |producer: &Producer| {
        matches!(producer.source, Source::Worksheet { worksheet, rectangle: Some(r), .. }
        if !producer.retired && GridScopeId::from(worksheet)==scope && r.first.row<=rectangle.last.row && r.last.row>=rectangle.first.row && r.first.column<=rectangle.last.column && r.last.column>=rectangle.first.column)
    };
    if !old.producers.iter().any(affected) {
        return Ok(());
    }
    let mut directory = (**old).clone();
    for producer in &mut directory.producers {
        if affected(producer) {
            producer.layout_order = producer
                .layout_order
                .checked_add(1)
                .ok_or(Error::Exhausted)?;
        }
    }
    data.producers = Some(data.memory.alloc(directory).map_err(|_| Error::Limit)?);
    Ok(())
}
