//! Native producer records. Integrity checks do not replace relationship validation.
use super::*;
const MAX_BYTES: usize = 64 * 1024 * 1024;
fn bad() -> Error {
    Error::Graph("malformed producer authority".into())
}
struct W(Vec<u8>);
impl W {
    fn n(&mut self, n: u64) {
        self.0.extend(n.to_le_bytes());
    }
    fn id(&mut self, id: Uuid) {
        self.0.extend(id.as_bytes());
    }
    fn text(&mut self, s: &str) {
        self.n(s.len() as u64);
        self.0.extend(s.as_bytes());
    }
    fn pos(&mut self, p: Position) {
        self.n(p.row.into());
        self.n(p.column.into());
    }
    fn rectangle(&mut self, r: Option<Rectangle>) {
        self.n(u64::from(r.is_some()));
        if let Some(r) = r {
            self.pos(r.first);
            self.pos(r.last);
        }
    }
    fn dec(&mut self, d: DecimalInput) {
        let (a, b, c, s, e) = d.parts();
        for n in [a, b, c, u32::from(s), e] {
            self.n(n.into());
        }
    }
    fn scope(&mut self, s: GridScopeId) {
        self.n(u64::from(s.is_window()));
        self.id(s.uuid());
    }
}
struct R<'a> {
    bytes: &'a [u8],
    at: usize,
}
impl<'a> R<'a> {
    fn bytes(&mut self, n: usize) -> Result<&'a [u8]> {
        let end = self.at.checked_add(n).ok_or_else(bad)?;
        let b = self.bytes.get(self.at..end).ok_or_else(bad)?;
        self.at = end;
        Ok(b)
    }
    fn n(&mut self) -> Result<u64> {
        Ok(u64::from_le_bytes(
            self.bytes(8)?.try_into().map_err(|_| bad())?,
        ))
    }
    fn count(&mut self, max: usize) -> Result<usize> {
        let n = usize::try_from(self.n()?).map_err(|_| bad())?;
        if n > max {
            return Err(Error::Limit);
        }
        Ok(n)
    }
    fn id(&mut self) -> Result<Uuid> {
        Ok(Uuid::from_bytes(
            self.bytes(16)?.try_into().map_err(|_| bad())?,
        ))
    }
    fn flag(&mut self) -> Result<bool> {
        match self.n()? {
            0 => Ok(false),
            1 => Ok(true),
            _ => Err(bad()),
        }
    }
    fn text(&mut self, max: usize) -> Result<String> {
        let n = self.count(max)?;
        Ok(std::str::from_utf8(self.bytes(n)?)
            .map_err(|_| bad())?
            .to_owned())
    }
    fn u32(&mut self) -> Result<u32> {
        u32::try_from(self.n()?).map_err(|_| bad())
    }
    fn pos(&mut self) -> Result<Position> {
        Ok(Position::new(self.u32()?, self.u32()?))
    }
    fn rectangle(&mut self) -> Result<Option<Rectangle>> {
        if !self.flag()? {
            return Ok(None);
        }
        let r = Rectangle {
            first: self.pos()?,
            last: self.pos()?,
        };
        if r.first.row == 0
            || r.first.column == 0
            || r.first.row > r.last.row
            || r.first.column > r.last.column
            || r.last.row > MAX_ROWS
            || r.last.column > MAX_COLUMNS
        {
            return Err(bad());
        }
        Ok(Some(r))
    }
    fn dec(&mut self) -> Result<DecimalInput> {
        let a = self.u32()?;
        let b = self.u32()?;
        let c = self.u32()?;
        let sign = self.flag()?;
        let scale = self.u32()?;
        DecimalInput::from_parts(a, b, c, sign, scale)
    }
    fn scope(&mut self) -> Result<GridScopeId> {
        let window = self.flag()?;
        let id = self.id()?;
        Ok(if window {
            WindowId::from_uuid(id).into()
        } else {
            WorksheetId::from_uuid(id).into()
        })
    }
}
pub fn encode(image: &Image) -> Result<Vec<u8>> {
    let mut w = W(Vec::new());
    w.n(image.allocated_identities as u64);
    w.n(image.producers.len() as u64);
    for p in &image.producers {
        w.id(p.id.0);
        w.n(p.configuration);
        w.n(u64::from(p.retired));
        match &p.source {
            SourceRecord::Worksheet {
                worksheet,
                selection,
                rectangle,
            } => {
                w.n(0);
                w.id(worksheet.uuid());
                w.id(Uuid::from_bytes(selection.0));
                w.rectangle(*rectangle);
            }
            SourceRecord::Fixture {
                fixture,
                schema,
                policy,
                selected,
                last_status,
            } => {
                w.n(1);
                w.n(*fixture as u64);
                w.n(*policy as u64);
                w.n(*last_status as u64);
                w.id(schema.id.0);
                w.n(schema.fields.len() as u64);
                for f in &schema.fields {
                    w.id(f.id.0);
                    w.text(&f.label);
                    w.n(f.kind as u64);
                    w.n(u64::from(f.nullable));
                }
                w.n(u64::from(selected.is_some()));
                if let Some(s) = selected {
                    w.id(s.generation.0);
                    w.0.extend(s.digest);
                    w.n(s.rows.len() as u64);
                    for row in &s.rows {
                        w.text(&row.key);
                        w.n(row.values.len() as u64);
                        for v in &row.values {
                            match v {
                                Value::Blank => w.n(0),
                                Value::Decimal(d) => {
                                    w.n(1);
                                    w.dec(*d);
                                }
                                Value::String(s) => {
                                    w.n(2);
                                    w.text(s);
                                }
                                Value::Boolean(b) => {
                                    w.n(3);
                                    w.n(u64::from(*b));
                                }
                            }
                        }
                    }
                }
            }
        }
    }
    w.n(image.windows.len() as u64);
    for b in &image.windows {
        w.id(b.window.uuid());
        w.id(b.producer.0);
    }
    w.n(image.guards.len() as u64);
    for g in &image.guards {
        w.id(g.id.0);
        w.id(g.window.uuid());
        w.id(g.producer.0);
        w.n(g.configuration);
        w.n(u64::from(g.active));
        w.n(g.copy_flags.into());
        match &g.target {
            GuardTarget::Cell(c) => {
                w.n(0);
                w.scope(c.worksheet);
                w.id(c.cell.uuid());
            }
            GuardTarget::External {
                key,
                field,
                generation,
            } => {
                w.n(1);
                w.text(key);
                w.id(field.0);
                w.n(u64::from(generation.is_some()));
                if let Some(id) = generation {
                    w.id(id.0);
                }
            }
            GuardTarget::Range {
                selection,
                rectangle,
                kind,
            } => {
                w.n(2);
                w.id(Uuid::from_bytes(selection.0));
                w.rectangle(*rectangle);
                w.n(kind.map_or(0, |k| k as u64 + 1));
            }
        }
    }
    w.n(image.outputs.len() as u64);
    for o in &image.outputs {
        let k = &o.key;
        w.id(k.producer.0);
        w.n(k.configuration);
        w.id(k.field.0);
        w.text(&k.key);
        w.n(u64::from(k.generation.is_some()));
        if let Some(id) = k.generation {
            w.id(id.0);
        }
        w.id(o.portable.uuid());
        w.n(o.slot as u64);
    }
    if w.0.len() > MAX_BYTES {
        return Err(Error::Limit);
    }
    Ok(w.0)
}
pub fn decode(bytes: &[u8]) -> Result<Image> {
    if bytes.len() > MAX_BYTES {
        return Err(Error::Limit);
    }
    let mut r = R { bytes, at: 0 };
    let allocated_identities = r.count(262_144)?;
    let mut producers = Vec::new();
    let mut slots = 0usize;
    for _ in 0..r.count(262_144)? {
        let id = ProducerId(r.id()?);
        let configuration = r.n()?;
        let retired = r.flag()?;
        let source = match r.n()? {
            0 => SourceRecord::Worksheet {
                worksheet: WorksheetId::from_uuid(r.id()?),
                selection: pivot_core::model::selection::SelectionId(*r.id()?.as_bytes()),
                rectangle: r.rectangle()?,
            },
            1 => {
                let fixture = match r.n()? {
                    0 => Fixture::Ledger,
                    1 => Fixture::Typed,
                    _ => return Err(bad()),
                };
                let policy = match r.n()? {
                    0 => KeyPolicy::Durable,
                    1 => KeyPolicy::GenerationQualified,
                    _ => return Err(bad()),
                };
                let last_status = match r.n()? {
                    0 => AttemptStatus::Never,
                    1 => AttemptStatus::InProgress,
                    2 => AttemptStatus::Succeeded,
                    3 => AttemptStatus::Failed,
                    4 => AttemptStatus::Cancelled,
                    5 => AttemptStatus::Disconnected,
                    6 => AttemptStatus::Interrupted,
                    _ => return Err(bad()),
                };
                let schema_id = SchemaId(r.id()?);
                let mut fields = Vec::new();
                for _ in 0..r.count(MAX_FIELDS)? {
                    fields.push(Field {
                        id: FieldId(r.id()?),
                        label: r.text(256)?,
                        kind: match r.n()? {
                            0 => ValueKind::Decimal,
                            1 => ValueKind::String,
                            2 => ValueKind::Boolean,
                            _ => return Err(bad()),
                        },
                        nullable: r.flag()?,
                    });
                }
                let schema = Schema {
                    id: schema_id,
                    fields,
                };
                schema.validate()?;
                let selected = if r.flag()? {
                    let generation = GenerationId(r.id()?);
                    let digest = r.bytes(32)?.try_into().map_err(|_| bad())?;
                    let mut rows = Vec::new();
                    let n = r.count(MAX_SLOTS)?;
                    slots = slots
                        .checked_add(n.checked_mul(schema.fields.len()).ok_or(Error::Limit)?)
                        .ok_or(Error::Limit)?;
                    if slots > MAX_SLOTS {
                        return Err(Error::Limit);
                    }
                    for _ in 0..n {
                        let key = r.text(MAX_KEY_BYTES)?;
                        let n = r.count(MAX_FIELDS)?;
                        if n != schema.fields.len() {
                            return Err(bad());
                        }
                        let mut values = Vec::new();
                        for _ in 0..n {
                            values.push(match r.n()? {
                                0 => Value::Blank,
                                1 => Value::Decimal(r.dec()?),
                                2 => Value::String(r.text(MAX_STRING_BYTES)?),
                                3 => Value::Boolean(r.flag()?),
                                _ => return Err(bad()),
                            });
                        }
                        rows.push(Row { key, values });
                    }
                    Some(SnapshotRecord {
                        generation,
                        digest,
                        rows,
                    })
                } else {
                    None
                };
                SourceRecord::Fixture {
                    fixture,
                    schema,
                    policy,
                    selected,
                    last_status,
                }
            }
            _ => return Err(bad()),
        };
        producers.push(ProducerRecord {
            id,
            configuration,
            retired,
            source,
        });
    }
    let mut windows = Vec::new();
    for _ in 0..r.count(262_144)? {
        windows.push(WindowBinding {
            window: WindowId::from_uuid(r.id()?),
            producer: ProducerId(r.id()?),
        });
    }
    let mut guards = Vec::new();
    for _ in 0..r.count(262_144)? {
        let id = GuardId(r.id()?);
        let window = WindowId::from_uuid(r.id()?);
        let producer = ProducerId(r.id()?);
        let configuration = r.n()?;
        let active = r.flag()?;
        let copy_flags = r.count(63)? as u8;
        let target = match r.n()? {
            0 => GuardTarget::Cell(CellRef {
                worksheet: r.scope()?,
                cell: CellId::from_uuid(r.id()?),
            }),
            1 => {
                let key = r.text(MAX_KEY_BYTES)?;
                let field = FieldId(r.id()?);
                let generation = if r.flag()? {
                    Some(GenerationId(r.id()?))
                } else {
                    None
                };
                GuardTarget::External {
                    key,
                    field,
                    generation,
                }
            }
            2 => {
                let selection = pivot_core::model::selection::SelectionId(*r.id()?.as_bytes());
                let rectangle = r.rectangle()?;
                let kind = match r.n()? {
                    0 => None,
                    1 => Some(expression::AggregateKind::Sum),
                    2 => Some(expression::AggregateKind::Min),
                    3 => Some(expression::AggregateKind::Max),
                    4 => Some(expression::AggregateKind::Average),
                    5 => Some(expression::AggregateKind::Count),
                    _ => return Err(bad()),
                };
                GuardTarget::Range {
                    selection,
                    rectangle,
                    kind,
                }
            }
            _ => return Err(bad()),
        };
        guards.push(Guard {
            id,
            window,
            producer,
            configuration,
            target,
            active,
            copy_flags,
        });
    }
    let mut outputs = Vec::new();
    for _ in 0..r.count(65_536)? {
        let producer = ProducerId(r.id()?);
        let configuration = r.n()?;
        let field = FieldId(r.id()?);
        let key = r.text(MAX_KEY_BYTES)?;
        let generation = if r.flag()? {
            Some(GenerationId(r.id()?))
        } else {
            None
        };
        outputs.push(OutputRecord {
            key: OutputKey {
                producer,
                configuration,
                field,
                key,
                generation,
            },
            portable: GraphNodeId::from_uuid(r.id()?),
            slot: r.count(262_144)?,
        });
    }
    if r.at != bytes.len() {
        return Err(bad());
    }
    Ok(Image {
        producers,
        windows,
        guards,
        outputs,
        allocated_identities,
    })
}
