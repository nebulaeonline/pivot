//! Feature-only complete checkpoints and identity ledger. No history cursor/journal.
use super::*;
use pivot_core::model::{
    integration::{
        IntegrationRuntime,
        history_codec::{self, Blob, Reader, RecordKey, Writer},
    },
    selection::{Rectangle, SelectionId, SpaceId},
};
use sha2::{Digest, Sha256};
use std::collections::{BTreeMap, BTreeSet};
fn bad() -> Error {
    Error::Graph("malformed cold workbook history".into())
}
fn id(r: &mut Reader<'_>) -> Result<Uuid> {
    Ok(Uuid::from_bytes(r.id().map_err(graph_error)?))
}
pub(super) fn pos(w: &mut Writer, p: Position) {
    w.n(p.row.into());
    w.n(p.column.into());
}
fn read_pos(r: &mut Reader<'_>) -> Result<Position> {
    Ok(Position::new(
        r.u32().map_err(graph_error)?,
        r.u32().map_err(graph_error)?,
    ))
}
fn rev(w: &mut Writer, r: Revision) {
    let (a, b) = r.provenance();
    w.n(a);
    w.n(b);
}
fn read_rev(r: &mut Reader<'_>) -> Result<Revision> {
    Ok(Revision::from_provenance(
        r.n().map_err(graph_error)?,
        r.n().map_err(graph_error)?,
    ))
}
fn rect(w: &mut Writer, r: Rectangle) {
    w.raw(&r.space.0);
    for n in [r.first_row, r.last_row, r.first_column, r.last_column] {
        w.n(n.into());
    }
}
fn read_rect(r: &mut Reader<'_>) -> Result<Rectangle> {
    let x = Rectangle {
        space: SpaceId(r.id().map_err(graph_error)?),
        first_row: r.u32().map_err(graph_error)?,
        last_row: r.u32().map_err(graph_error)?,
        first_column: r.u32().map_err(graph_error)?,
        last_column: r.u32().map_err(graph_error)?,
    };
    if !x.valid() {
        return Err(bad());
    }
    Ok(x)
}
fn kind(w: &mut Writer, k: expression::AggregateKind) {
    use expression::AggregateKind::*;
    w.n(match k {
        Sum => 0,
        Min => 1,
        Max => 2,
        Average => 3,
        Count => 4,
    });
}
fn read_kind(r: &mut Reader<'_>) -> Result<expression::AggregateKind> {
    use expression::AggregateKind::*;
    Ok(match r.n().map_err(graph_error)? {
        0 => Sum,
        1 => Min,
        2 => Max,
        3 => Average,
        4 => Count,
        _ => return Err(bad()),
    })
}
fn span(w: &mut Writer, s: &std::ops::Range<usize>) {
    w.n(s.start as u64);
    w.n(s.end as u64);
}
fn read_span(r: &mut Reader<'_>) -> Result<std::ops::Range<usize>> {
    let start = r.n().map_err(graph_error)?.try_into().map_err(|_| bad())?;
    let end = r.n().map_err(graph_error)?.try_into().map_err(|_| bad())?;
    if start > end || end > 4096 {
        return Err(bad());
    }
    Ok(start..end)
}
fn tokens(w: &mut Writer, ts: &[Token]) -> Result<()> {
    w.n(ts.len() as u64);
    for t in ts {
        match t {
            Token::Control(c) => {
                w.n(5);
                w.n(c.code().into());
            }
            Token::Decimal(d) => {
                w.n(0);
                let (a, b, c, s, e) = d.parts();
                for n in [a, b, c, u32::from(s), e] {
                    w.n(n.into());
                }
            }
            Token::Cell(c) => {
                w.n(1);
                w.raw(c.worksheet.uuid().as_bytes());
                w.raw(c.cell.uuid().as_bytes());
            }
            Token::Unary(op) => {
                w.n(2);
                w.n(match op {
                    UnaryOp::Plus => 0,
                    UnaryOp::Minus => 1,
                });
            }
            Token::Binary(op) => {
                w.n(3);
                w.n(match op {
                    BinaryOp::Add => 0,
                    BinaryOp::Subtract => 1,
                    BinaryOp::Multiply => 2,
                    BinaryOp::Divide => 3,
                });
            }
            Token::Sum {
                kind: k,
                id,
                rectangle,
                first,
                last,
                span: s,
            } => {
                w.n(4);
                kind(w, *k);
                w.raw(&id.0);
                rect(w, *rectangle);
                pos(w, *first);
                pos(w, *last);
                span(w, s);
            }
            _ => return Err(bad()),
        }
    }
    Ok(())
}
fn read_decimal_input(r: &mut Reader<'_>) -> Result<DecimalInput> {
    let a = r.u32().map_err(graph_error)?;
    let b = r.u32().map_err(graph_error)?;
    let c = r.u32().map_err(graph_error)?;
    let sign = r.flag().map_err(graph_error)?;
    let scale = r.u32().map_err(graph_error)?;
    DecimalInput::from_parts(a, b, c, sign, scale)
}
fn read_tokens(r: &mut Reader<'_>) -> Result<Vec<Token>> {
    let mut out = Vec::new();
    for _ in 0..r.count(4096).map_err(graph_error)? {
        out.push(match r.n().map_err(graph_error)? {
            5 => Token::Control(
                pivot_core::expression::AggregateControl::from_code(
                    u8::try_from(r.n().map_err(graph_error)?).map_err(|_| bad())?,
                )
                .ok_or_else(bad)?,
            ),
            0 => Token::Decimal(read_decimal_input(r)?),
            1 => Token::Cell(CellRef {
                worksheet: WorksheetId::from_uuid(id(r)?),
                cell: CellId::from_uuid(id(r)?),
            }),
            2 => Token::Unary(match r.n().map_err(graph_error)? {
                0 => UnaryOp::Plus,
                1 => UnaryOp::Minus,
                _ => return Err(bad()),
            }),
            3 => Token::Binary(match r.n().map_err(graph_error)? {
                0 => BinaryOp::Add,
                1 => BinaryOp::Subtract,
                2 => BinaryOp::Multiply,
                3 => BinaryOp::Divide,
                _ => return Err(bad()),
            }),
            4 => Token::Sum {
                kind: read_kind(r)?,
                id: SelectionId(r.id().map_err(graph_error)?),
                rectangle: read_rect(r)?,
                first: read_pos(r)?,
                last: read_pos(r)?,
                span: read_span(r)?,
            },
            _ => return Err(bad()),
        });
    }
    Ok(out)
}
pub(super) fn content(w: &mut Writer, c: &Content) -> Result<()> {
    match c {
        Content::Boolean(_) => return Err(bad()),
        Content::Blank => w.n(0),
        Content::Decimal(d) => {
            w.n(1);
            let (a, b, c, s, e) = d.parts();
            for n in [a, b, c, u32::from(s), e] {
                w.n(n.into());
            }
        }
        Content::LiteralText(s) => {
            w.n(2);
            w.text(s);
        }
        Content::Formula { source, tokens: ts } => {
            w.n(3);
            w.text(source);
            tokens(w, ts)?;
        }
        Content::TextFormula(f) => {
            w.n(4);
            w.text(&f.original_source);
            tokens(w, &f.tokens)?;
            w.n(f.references.len() as u64);
            for x in &f.references {
                span(w, &x.span);
                pos(w, x.submitted_position);
                w.raw(x.target.worksheet.uuid().as_bytes());
                w.raw(x.target.cell.uuid().as_bytes());
                w.raw(x.node.uuid().as_bytes());
            }
            w.flag(f.diagnostic.is_some());
            if let Some(e) = &f.diagnostic {
                w.parse_error(e);
            }
            w.raw(f.transaction_base.session.as_bytes());
            w.n(f.transaction_base.state);
            rev(w, f.transaction_base.graph);
            w.n(f.edit_ordinal as u64);
            rev(w, f.resolved_graph);
        }
        _ => return Err(bad()),
    }
    Ok(())
}
fn read_content(r: &mut Reader<'_>) -> Result<Content> {
    Ok(match r.n().map_err(graph_error)? {
        0 => Content::Blank,
        1 => Content::Decimal(read_decimal_input(r)?),
        2 => Content::LiteralText(r.text(4096).map_err(graph_error)?),
        3 => Content::Formula {
            source: r.text(4096).map_err(graph_error)?,
            tokens: read_tokens(r)?,
        },
        4 => {
            let original_source = r.text(4096).map_err(graph_error)?;
            let tokens = read_tokens(r)?;
            let mut references = Vec::new();
            for _ in 0..r.count(4096).map_err(graph_error)? {
                references.push(FormulaReference {
                    span: read_span(r)?,
                    submitted_position: read_pos(r)?,
                    target: CellRef {
                        worksheet: WorksheetId::from_uuid(id(r)?),
                        cell: CellId::from_uuid(id(r)?),
                    },
                    node: GraphNodeId::from_uuid(id(r)?),
                });
            }
            let diagnostic = if r.flag().map_err(graph_error)? {
                Some(r.parse_error().map_err(graph_error)?)
            } else {
                None
            };
            let transaction_base = Expected {
                session: id(r)?,
                state: r.n().map_err(graph_error)?,
                graph: read_rev(r)?,
            };
            let edit_ordinal = r.n().map_err(graph_error)?.try_into().map_err(|_| bad())?;
            let resolved_graph = read_rev(r)?;
            Content::TextFormula(TextFormula {
                original_source,
                tokens,
                references,
                diagnostic,
                transaction_base,
                edit_ordinal,
                resolved_graph,
            })
        }
        _ => return Err(bad()),
    })
}
/// Research identity authority outside undoable content. Observe every committed state.
/// Persistence/commit ordering is deliberately left to later journal experiments.
pub struct ColdLedger {
    /// Ephemeral source certification, never serialized. Established only after
    /// full identity validation/observation or verified cold restoration.
    pub(super) validated_content: Option<Uuid>,
    pub(super) instance: Uuid,
    pub(super) session: Uuid,
    pub(super) worksheet: WorksheetId,
    pub(super) graph: Uuid,
    pub(super) floor: u64,
    pub(super) selection_floor: usize,
    pub(super) identities: BTreeMap<u64, (CellId, GraphNodeId)>,
    pub(super) selections: BTreeSet<SelectionId>,
    _charge: Charge,
}
impl ColdLedger {
    /// Full-state differential oracle only. This O(L) clone is not used by the
    /// mutation-local producer or any persisted decision protocol.
    pub fn clone_for_oracle(
        &self,
        memory: &pivot_core::model::integration::resources::Memory,
    ) -> Result<Self> {
        let charge = memory
            .reserve(self.identities.len() * 512 + self.selections.len() * 128 + 4096)
            .map_err(|_| Error::Limit)?;
        Ok(Self {
            validated_content: self.validated_content,
            instance: self.instance,
            session: self.session,
            worksheet: self.worksheet,
            graph: self.graph,
            floor: self.floor,
            selection_floor: self.selection_floor,
            identities: self.identities.clone(),
            selections: self.selections.clone(),
            _charge: charge,
        })
    }
    pub fn new(book: &Workbook) -> Result<Self> {
        let mut x = Self {
            validated_content: None,
            instance: Uuid::new_v4(),
            session: book.session,
            worksheet: book.worksheet,
            graph: book.graph_id,
            floor: 0,
            selection_floor: 0,
            identities: BTreeMap::new(),
            selections: BTreeSet::new(),
            _charge: book
                .core
                .allocation_domain()
                .map_err(graph_error)?
                .reserve(4096)
                .map_err(|_| Error::Limit)?,
        };
        x.observe(book)?;
        Ok(x)
    }
    pub fn observe(&mut self, book: &Workbook) -> Result<()> {
        if self.session != book.session
            || self.worksheet != book.worksheet
            || self.graph != book.graph_id
        {
            return Err(Error::StaleSession);
        }
        let m = book.history_metrics()?;
        if m.next_node > 8192 {
            return Err(Error::Limit);
        }
        let memory = book.core.allocation_domain().map_err(graph_error)?;
        let _scratch = memory
            .reserve(
                (m.next_node as usize)
                    .saturating_mul(384)
                    .saturating_add(m.lifetime_selections.saturating_mul(128))
                    .saturating_add(4096),
            )
            .map_err(|_| Error::Limit)?;
        let mut additions = Vec::new();
        let mut cells: BTreeSet<_> = self.identities.values().map(|x| x.0).collect();
        let mut nodes: BTreeSet<_> = self.identities.values().map(|x| x.1).collect();
        for (cell, b) in persistent::state(book).bindings.iter() {
            let n = book
                .core
                .history_cold_ordinal(b.runtime)
                .map_err(graph_error)?;
            match self.identities.get(&n) {
                Some(old) if *old != (*cell, b.portable) => return Err(bad()),
                Some(_) => {}
                None => {
                    if !cells.insert(*cell) || !nodes.insert(b.portable) {
                        return Err(bad());
                    }
                    additions.push((n, (*cell, b.portable)));
                }
            }
        }
        let ids: Vec<_> = book
            .core
            .selection_image()
            .identities
            .into_iter()
            .map(|x| x.0)
            .collect();
        let floor = self.floor.max(m.next_node);
        let selection_floor = self.selection_floor.max(m.lifetime_selections);
        let new_selections = ids
            .iter()
            .filter(|id| !self.selections.contains(id))
            .count();
        // An old root cannot recreate reservations that its caller failed to retain.
        if (self.identities.len() + additions.len()) as u64 != floor
            || self.selections.len() + new_selections != selection_floor
        {
            return Err(bad());
        }
        let charge = memory
            .reserve(
                (floor as usize)
                    .saturating_mul(512)
                    .saturating_add(selection_floor.saturating_mul(128))
                    .saturating_add(4096),
            )
            .map_err(|_| Error::Limit)?;
        for (n, pair) in additions {
            self.identities.insert(n, pair);
        }
        self.selections.extend(ids);
        self.floor = floor;
        self.selection_floor = selection_floor;
        self._charge = charge;
        self.validated_content = Some(book.history_content_id());
        Ok(())
    }
    /// Feature-only O(changes log L) append after a sealed local candidate. No
    /// journal/receipt/ACK is created; persisted reconstruction remains unchanged.
    pub(super) fn append_local_records(
        &mut self,
        book: &Workbook,
        next_content: Uuid,
        change: &super::changes::Reservations,
    ) -> Result<()> {
        if self.session != book.session
            || self.worksheet != book.worksheet
            || self.graph != book.graph_id
            || self.instance != change.instance
            || self.validated_content != Some(book.history_content_id())
            || self.floor != change.nodes.0
            || self.selection_floor != change.selections.0
            || change.nodes.0.checked_add(change.added_nodes.len() as u64) != Some(change.nodes.1)
            || change
                .selections
                .0
                .checked_add(change.added_selections.len())
                != Some(change.selections.1)
        {
            return Err(bad());
        }
        for (index, (n, _)) in change.added_nodes.iter().enumerate() {
            if *n != self.floor + index as u64 || self.identities.contains_key(n) {
                return Err(bad());
            }
        }
        if change.added_selections.windows(2).any(|v| v[0] >= v[1])
            || change
                .added_selections
                .iter()
                .any(|id| self.selections.contains(id))
        {
            return Err(bad());
        }
        let memory = book.core.allocation_domain().map_err(graph_error)?;
        let charge = if self.floor != change.nodes.1 || self.selection_floor != change.selections.1
        {
            Some(
                memory
                    .reserve(
                        (change.nodes.1 as usize)
                            .saturating_mul(512)
                            .saturating_add(change.selections.1.saturating_mul(128))
                            .saturating_add(4096),
                    )
                    .map_err(|_| Error::Limit)?,
            )
        } else {
            None
        };
        for (n, pair) in &change.added_nodes {
            self.identities.insert(*n, *pair);
        }
        self.selections
            .extend(change.added_selections.iter().copied());
        self.floor = change.nodes.1;
        self.selection_floor = change.selections.1;
        if let Some(charge) = charge {
            self._charge = charge;
        }
        self.validated_content = Some(next_content);
        Ok(())
    }
    pub fn allocated_floor(&self) -> u64 {
        self.floor
    }
    fn write(&self, w: &mut Writer) {
        w.boundary(RecordKey::new(1, &[]));
        w.raw(self.instance.as_bytes());
        w.n(self.floor);
        w.n(self.selection_floor as u64);
        w.n(self.identities.len() as u64);
        for (n, (c, p)) in &self.identities {
            w.boundary(RecordKey::number(2, *n));
            w.n(*n);
            w.raw(c.uuid().as_bytes());
            w.raw(p.uuid().as_bytes());
        }
        w.boundary(RecordKey::new(3, &[]));
        w.n(self.selections.len() as u64);
        for s in &self.selections {
            w.boundary(RecordKey::new(4, &s.0));
            w.raw(&s.0);
        }
    }
    fn read(
        r: &mut Reader<'_>,
        book: &Workbook,
        worksheet: WorksheetId,
        graph: Uuid,
    ) -> Result<Self> {
        let instance = id(r)?;
        let floor = r.n().map_err(graph_error)?;
        let selection_floor = usize::try_from(r.n().map_err(graph_error)?).map_err(|_| bad())?;
        if floor > book.limits.allocated_cells as u64 || selection_floor > 8192 {
            return Err(Error::Limit);
        }
        let memory = book.core.allocation_domain().map_err(graph_error)?;
        let charge = memory
            .reserve(
                (floor as usize)
                    .saturating_mul(512)
                    .saturating_add(selection_floor.saturating_mul(128))
                    .saturating_add(4096),
            )
            .map_err(|_| Error::Limit)?;
        let _scratch = memory
            .reserve(
                (floor as usize)
                    .saturating_mul(256)
                    .saturating_add(selection_floor.saturating_mul(64))
                    .saturating_add(2048),
            )
            .map_err(|_| Error::Limit)?;
        let mut identities = BTreeMap::new();
        let mut cells = BTreeSet::new();
        let mut nodes = BTreeSet::new();
        for _ in 0..r.count(8192).map_err(graph_error)? {
            let n = r.n().map_err(graph_error)?;
            let c = CellId::from_uuid(id(r)?);
            let p = GraphNodeId::from_uuid(id(r)?);
            if n >= floor
                || identities.insert(n, (c, p)).is_some()
                || !cells.insert(c)
                || !nodes.insert(p)
            {
                return Err(bad());
            }
        }
        if identities.len() as u64 != floor {
            return Err(bad());
        }
        let mut selections = BTreeSet::new();
        for _ in 0..r.count(8192).map_err(graph_error)? {
            if !selections.insert(SelectionId(r.id().map_err(graph_error)?)) {
                return Err(bad());
            }
        }
        if selections.len() != selection_floor {
            return Err(bad());
        }
        Ok(Self {
            validated_content: None,
            instance,
            session: book.session,
            worksheet,
            graph,
            floor,
            selection_floor,
            identities,
            selections,
            _charge: charge,
        })
    }
    fn includes(&self, other: &Self) -> bool {
        self.instance == other.instance
            && self.worksheet == other.worksheet
            && self.graph == other.graph
            && self.floor >= other.floor
            && self.selection_floor >= other.selection_floor
            && other
                .identities
                .iter()
                .all(|(n, p)| self.identities.get(n) == Some(p))
            && other.selections.is_subset(&self.selections)
    }
}
fn frame(bytes: &[u8]) -> Result<&[u8]> {
    if bytes.len() < 40 || bytes.len() > history_codec::MAX_BYTES {
        return Err(bad());
    }
    let (body, digest) = bytes.split_at(bytes.len() - 32);
    if Sha256::digest(body).as_slice() != digest {
        return Err(bad());
    }
    Ok(body)
}
fn header<'a>(
    bytes: &'a [u8],
    book: &Workbook,
) -> Result<(Reader<'a>, WorksheetId, Uuid, Axis, Axis, Uuid, ColdLedger)> {
    let mut r = Reader::new(frame(bytes)?).map_err(graph_error)?;
    if r.take(8).map_err(graph_error)? != b"PIVHWB01" {
        return Err(bad());
    }
    let worksheet = WorksheetId::from_uuid(id(&mut r)?);
    let graph = id(&mut r)?;
    let rows = Axis {
        namespace: id(&mut r)?,
        length: r.u32().map_err(graph_error)?,
    };
    let columns = Axis {
        namespace: id(&mut r)?,
        length: r.u32().map_err(graph_error)?,
    };
    let content = id(&mut r)?;
    if rows.namespace == columns.namespace
        || rows.length != book.limits.rows
        || columns.length != book.limits.columns
    {
        return Err(bad());
    }
    let ledger = ColdLedger::read(&mut r, book, worksheet, graph)?;
    Ok((r, worksheet, graph, rows, columns, content, ledger))
}
impl Workbook {
    pub fn history_cold_export(&self, expected: Expected, ledger: &ColdLedger) -> Result<Blob> {
        self.check(expected)?;
        self.history_weight()?;
        if ledger.session != self.session
            || ledger.worksheet != self.worksheet
            || ledger.graph != self.graph_id
        {
            return Err(Error::StaleSession);
        }
        let m = self.history_metrics()?;
        if m.next_node > ledger.floor
            || m.lifetime_selections > ledger.selection_floor
            || ledger.identities.len() as u64 != ledger.floor
            || ledger.selections.len() != ledger.selection_floor
        {
            return Err(bad());
        }
        let memory = self.core.allocation_domain().map_err(graph_error)?;
        let _scratch = memory.reserve(2048).map_err(|_| Error::Limit)?;
        let mut w = Writer::new(&memory).map_err(graph_error)?;
        w.raw(b"PIVHWB01");
        w.raw(self.worksheet.uuid().as_bytes());
        w.raw(self.graph_id.as_bytes());
        w.raw(self.rows.namespace.as_bytes());
        w.n(self.rows.length.into());
        w.raw(self.columns.namespace.as_bytes());
        w.n(self.columns.length.into());
        w.raw(self.history_content.as_bytes());
        ledger.write(&mut w);
        let core = self.core.history_cold_export().map_err(graph_error)?;
        w.boundary(RecordKey::new(5, &[]));
        w.n(core.bytes.len() as u64);
        w.append_blob(&core);
        let d = persistent::state(self);
        w.boundary(RecordKey::new(15, &[]));
        w.n(d.records.len() as u64);
        for (cell, r) in d.records.iter() {
            w.boundary(RecordKey::new(16, cell.uuid().as_bytes()));
            let b = d.bindings.get(cell).ok_or_else(bad)?;
            let n = self
                .core
                .history_cold_ordinal(b.runtime)
                .map_err(graph_error)?;
            if ledger.identities.get(&n) != Some(&(*cell, b.portable)) {
                return Err(bad());
            }
            w.raw(cell.uuid().as_bytes());
            w.raw(b.portable.uuid().as_bytes());
            w.n(n);
            w.flag(r.position.is_some());
            if let (Some(p), Some(c)) = (r.position, &r.content) {
                pos(&mut w, p);
                content(&mut w, c)?;
            } else if r.position.is_some() || r.content.is_some() {
                return Err(bad());
            }
        }
        let mut blob = w.finish().map_err(graph_error)?;
        let digest = Sha256::digest(&blob.bytes);
        if blob.bytes.len() + 32 > history_codec::MAX_BYTES {
            return Err(Error::Limit);
        }
        blob.append(RecordKey::new(17, &[]), &digest)
            .map_err(graph_error)?;
        Ok(blob)
    }
    fn cold_candidate(
        &self,
        bytes: &[u8],
        authority: &ColdLedger,
        cancel: &CancellationToken,
        new_instance: bool,
    ) -> Result<Self> {
        if cancel.is_cancelled() {
            return Err(bad());
        }
        self.history_weight()?;
        if authority.session != self.session {
            return Err(Error::StaleSession);
        }
        let current = self.history_metrics()?;
        if current.next_node > authority.floor
            || current.lifetime_selections > authority.selection_floor
        {
            return Err(bad());
        }
        let memory = self.core.allocation_domain().map_err(graph_error)?;
        let _scratch = memory
            .reserve(bytes.len().saturating_mul(8).saturating_add(4096))
            .map_err(|_| Error::Limit)?;
        let (mut r, worksheet, graph, rows, columns, content, embedded) = header(bytes, self)?;
        if !authority.includes(&embedded)
            || (!new_instance
                && (worksheet != self.worksheet
                    || graph != self.graph_id
                    || rows.namespace != self.rows.namespace
                    || columns.namespace != self.columns.namespace))
        {
            return Err(bad());
        }
        let core_bytes = r.bytes(history_codec::MAX_BYTES).map_err(graph_error)?;
        let base = self
            .core
            .history_cold_with_floors(authority.floor, authority.selection_floor)
            .map_err(graph_error)?;
        let core = base
            .history_cold_restore(core_bytes, cancel)
            .map_err(graph_error)?;
        let metrics = core.history_metrics().map_err(graph_error)?;
        if metrics.next_node != authority.floor
            || metrics.lifetime_selections != authority.selection_floor
        {
            return Err(bad());
        }
        let mut data = core
            .prepare_resource(persistent::Data::new(memory))
            .map_err(graph_error)?;
        data.install_sheets(vec![SheetInfo {
            id: worksheet,
            name: "Sheet1".into(),
            rows: rows.clone(),
            columns: columns.clone(),
        }])?;
        let mut ordinals = BTreeMap::new();
        let mut portable = BTreeSet::new();
        for _ in 0..r.count(self.limits.allocated_cells).map_err(graph_error)? {
            if cancel.is_cancelled() {
                return Err(bad());
            }
            let cell = CellId::from_uuid(id(&mut r)?);
            let node = GraphNodeId::from_uuid(id(&mut r)?);
            let ordinal = r.n().map_err(graph_error)?;
            if authority.identities.get(&ordinal) != Some(&(cell, node))
                || data.bindings.contains_key(&cell)
                || !portable.insert(node)
                || ordinals.contains_key(&ordinal)
            {
                return Err(bad());
            }
            let live = r.flag().map_err(graph_error)?;
            let (position, value) = if live {
                let p = read_pos(&mut r)?;
                rows.at(p.row)?;
                columns.at(p.column)?;
                if data.positions.contains_key(&sheets::at(worksheet, p)) {
                    return Err(bad());
                }
                data.positions
                    .insert_private(sheets::at(worksheet, p), cell)
                    .map_err(|_| Error::Limit)?;
                data.live += 1;
                (Some(p), Some(read_content(&mut r)?))
            } else {
                (None, None)
            };
            let runtime = core.history_cold_node(ordinal).map_err(graph_error)?;
            data.record(
                cell,
                Record {
                    worksheet,
                    position,
                    content: value,
                },
            )?;
            data.bindings
                .insert_private(
                    cell,
                    Binding {
                        portable: node,
                        runtime,
                    },
                )
                .map_err(|_| Error::Limit)?;
            data.nodes
                .insert_private(runtime, cell)
                .map_err(|_| Error::Limit)?;
            ordinals.insert(ordinal, live);
        }
        r.end().map_err(graph_error)?;
        if core
            .history_cold_nodes()
            .into_iter()
            .collect::<BTreeMap<_, _>>()
            != ordinals
        {
            return Err(bad());
        }
        for (s, _) in core.selection_image().identities {
            if !authority.selections.contains(&s) {
                return Err(bad());
            }
        }
        let stats = data.stats();
        if stats.live_cells > self.limits.live_cells
            || stats.formula_tokens > self.limits.formula_tokens
            || stats.source_bytes > self.limits.source_bytes
            || stats.estimated_owned_bytes > self.limits.bytes
        {
            return Err(Error::Limit);
        }
        let selection_rectangles: BTreeMap<_, _> =
            core.selection_image().identities.into_iter().collect();
        // Compare the two authoritative halves without parsing or name resolution.
        for (cell, record) in data.records.iter() {
            use pivot_core::model::integration::ColdDefinition;
            let Some(value) = &record.content else {
                continue;
            };
            let mut resolved = Vec::new();
            let formula = match value {
                Content::Formula { source, tokens } => {
                    Some((source.as_str(), tokens.as_slice(), false))
                }
                Content::TextFormula(f) => Some((
                    f.original_source.as_str(),
                    f.tokens.as_slice(),
                    f.diagnostic.is_some(),
                )),
                _ => None,
            };
            let expected = if let Some((source, tokens, rejected)) = formula {
                for token in tokens {
                    resolved.push(match token {
                        Token::Decimal(d) => {
                            pivot_core::model::transfer::ResolvedToken::Decimal(d.value())
                        }
                        Token::Cell(c) => pivot_core::model::transfer::ResolvedToken::Node(
                            data.bindings.get(&c.cell).ok_or_else(bad)?.runtime,
                        ),
                        Token::Unary(op) => pivot_core::model::transfer::ResolvedToken::Unary(*op),
                        Token::Control(c) => {
                            pivot_core::model::transfer::ResolvedToken::Control(*c)
                        }
                        Token::Binary(op) => {
                            pivot_core::model::transfer::ResolvedToken::Binary(*op)
                        }
                        Token::Sum {
                            kind,
                            id,
                            rectangle,
                            ..
                        } => {
                            if selection_rectangles.get(id) != Some(&Some(*rectangle)) {
                                return Err(bad());
                            }
                            if *kind == expression::AggregateKind::Sum {
                                pivot_core::model::transfer::ResolvedToken::Sum(*id)
                            } else {
                                pivot_core::model::transfer::ResolvedToken::Aggregate(*kind, *id)
                            }
                        }
                        _ => return Err(bad()),
                    });
                }
                if rejected && !tokens.is_empty() {
                    return Err(bad());
                }
                ColdDefinition::Formula(
                    source,
                    if rejected {
                        None
                    } else {
                        Some(resolved.as_slice())
                    },
                )
            } else {
                match value {
                    Content::Blank => ColdDefinition::Blank,
                    Content::LiteralText(text) => ColdDefinition::LiteralString(text),
                    Content::Decimal(d) => ColdDefinition::Input(d.value()),
                    _ => return Err(bad()),
                }
            };
            core.history_cold_check_definition(
                data.bindings.get(cell).ok_or_else(bad)?.runtime,
                expected,
            )
            .map_err(graph_error)?;
        }
        // Check logical reference coverage without parsing/rebinding the submitted source.
        for (_, record) in data.records.iter() {
            let (source, ts) = match &record.content {
                Some(Content::Formula { source, tokens }) => (source.as_str(), tokens.as_slice()),
                Some(Content::TextFormula(f)) => {
                    for x in &f.references {
                        if x.target.worksheet != worksheet
                            || data
                                .bindings
                                .get(&x.target.cell)
                                .is_none_or(|b| b.portable != x.node)
                            || x.span.end > f.original_source.len()
                            || !f.original_source.is_char_boundary(x.span.start)
                            || !f.original_source.is_char_boundary(x.span.end)
                        {
                            return Err(bad());
                        }
                    }
                    if f.diagnostic.as_ref().is_some_and(|e| {
                        e.offset > f.original_source.len()
                            || !f.original_source.is_char_boundary(e.offset)
                    }) {
                        return Err(bad());
                    }
                    (&*f.original_source, f.tokens.as_slice())
                }
                _ => continue,
            };
            for t in ts {
                match t {
                    Token::Cell(c) => {
                        if c.worksheet != worksheet || !data.bindings.contains_key(&c.cell) {
                            return Err(bad());
                        }
                    }
                    Token::Sum {
                        id,
                        rectangle,
                        first,
                        last,
                        span,
                        ..
                    } => {
                        if !authority.selections.contains(id)
                            || rectangle.space.0 != *worksheet.uuid().as_bytes()
                            || span.start >= span.end
                            || rectangle.first_row != first.row.min(last.row)
                            || rectangle.last_row != first.row.max(last.row)
                            || rectangle.first_column != first.column.min(last.column)
                            || rectangle.last_column != first.column.max(last.column)
                            || span.end > source.len()
                            || !source.is_char_boundary(span.start)
                            || !source.is_char_boundary(span.end)
                        {
                            return Err(bad());
                        }
                        rows.at(first.row)?;
                        rows.at(last.row)?;
                        columns.at(first.column)?;
                        columns.at(last.column)?;
                    }
                    _ => {}
                }
            }
        }
        let persistent = data.freeze();
        let candidate = Self {
            session: self.session,
            state: self.state.checked_add(1).ok_or(Error::Exhausted)?,
            worksheet,
            graph_id: graph,
            rows,
            columns,
            persistent,
            core,
            limits: self.limits,
            history_content: content,
            history_active: true,
            #[cfg(feature = "history-experiment")]
            history_recording: None,
        };
        if cancel.is_cancelled() {
            return Err(bad());
        }
        Ok(candidate)
    }
    pub fn history_cold_activate(
        &mut self,
        expected: Expected,
        bytes: &[u8],
        ledger: &ColdLedger,
        cancel: &CancellationToken,
    ) -> Result<Expected> {
        self.check(expected)?;
        let candidate = self.cold_candidate(bytes, ledger, cancel, false)?;
        *self = candidate;
        Ok(self.expected())
    }
    /// Latest committed authority capsule and chosen content capsule are separate inputs.
    /// Caller selection/commit consistency is a later journal gate, not inferred here.
    pub fn history_cold_reopen(
        authority_bytes: &[u8],
        state_bytes: &[u8],
        limits: Limits,
        runtime: &IntegrationRuntime,
    ) -> Result<(Self, ColdLedger)> {
        let baseline = Self::new_in("cold reopen", limits, runtime)?;
        let (_, _, _, _, _, _, mut ledger) = header(authority_bytes, &baseline)?;
        let book =
            baseline.cold_candidate(state_bytes, &ledger, &CancellationToken::default(), true)?;
        ledger.validated_content = Some(book.history_content_id());
        Ok((book, ledger))
    }
}
