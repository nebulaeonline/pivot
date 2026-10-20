//! Restricted deterministic CBOR: minimal unsigneds, definite byte/text strings
//! and arrays. Fixed arity replaces field maps; no tags, floats or indefinite data.
use crate::*;
enum V {
    U(u64),
    B(Vec<u8>),
    T(String),
    A(Vec<V>),
}
impl V {
    fn array(self, n: usize) -> Result<Vec<V>> {
        if let Self::A(a) = self
            && a.len() == n
        {
            Ok(a)
        } else {
            Err(Error::Malformed)
        }
    }
    fn list(self) -> Result<Vec<V>> {
        if let Self::A(a) = self {
            Ok(a)
        } else {
            Err(Error::Malformed)
        }
    }
    fn u(self) -> Result<u64> {
        if let Self::U(v) = self {
            Ok(v)
        } else {
            Err(Error::Malformed)
        }
    }
    fn text(self) -> Result<String> {
        if let Self::T(v) = self {
            Ok(v)
        } else {
            Err(Error::Malformed)
        }
    }
    fn uuid(self) -> Result<Uuid> {
        if let Self::B(v) = self {
            Uuid::from_slice(&v).map_err(|_| Error::Malformed)
        } else {
            Err(Error::Malformed)
        }
    }
    fn decimal(self) -> Result<Decimal96> {
        if let Self::B(v) = self {
            Ok(Decimal96(v.try_into().map_err(|_| Error::Malformed)?))
        } else {
            Err(Error::Malformed)
        }
    }
}
fn head(major: u8, v: u64, out: &mut Vec<u8>) {
    let prefix = major << 5;
    match v {
        0..=23 => out.push(prefix | v as u8),
        24..=255 => {
            out.extend([prefix | 24, v as u8]);
        }
        256..=65535 => {
            out.push(prefix | 25);
            out.extend((v as u16).to_be_bytes());
        }
        65536..=0xffff_ffff => {
            out.push(prefix | 26);
            out.extend((v as u32).to_be_bytes());
        }
        _ => {
            out.push(prefix | 27);
            out.extend(v.to_be_bytes());
        }
    }
}
struct Writer {
    out: Vec<u8>,
    limits: Limits,
    depth: usize,
    items: usize,
}
impl Writer {
    fn item(&mut self, major: u8, n: u64, payload: &[u8]) -> Result<()> {
        if self.depth >= self.limits.depth || self.items == 0 {
            return Err(Error::Limit);
        }
        let header = match n {
            0..=23 => 1,
            24..=255 => 2,
            256..=65535 => 3,
            65536..=0xffff_ffff => 5,
            _ => 9,
        };
        let end = self
            .out
            .len()
            .checked_add(header)
            .and_then(|v| v.checked_add(payload.len()))
            .ok_or(Error::Limit)?;
        if end > self.limits.bytes {
            return Err(Error::Limit);
        }
        self.items -= 1;
        head(major, n, &mut self.out);
        self.out.extend_from_slice(payload);
        Ok(())
    }
    fn u(&mut self, n: impl Into<u64>) -> Result<()> {
        self.item(0, n.into(), &[])
    }
    fn b(&mut self, b: &[u8]) -> Result<()> {
        self.item(2, b.len() as u64, b)
    }
    fn text(&mut self, s: &str) -> Result<()> {
        self.item(3, s.len() as u64, s.as_bytes())
    }
    fn id(&mut self, id: Uuid) -> Result<()> {
        self.b(id.as_bytes())
    }
    fn dec(&mut self, v: Decimal96) -> Result<()> {
        self.b(&v.0)
    }
    fn a(&mut self, n: usize, body: impl FnOnce(&mut Self) -> Result<()>) -> Result<()> {
        self.item(4, n as u64, &[])?;
        self.depth += 1;
        let result = body(self);
        self.depth -= 1;
        result
    }
    fn definition(&mut self, definition: &Definition) -> Result<()> {
        match definition {
            Definition::Input(v) => self.a(2, |w| {
                w.u(0u8)?;
                w.dec(*v)
            }),
            Definition::Bound { source, tokens } => self.a(3, |w| {
                w.u(1u8)?;
                w.text(source)?;
                w.a(tokens.len(), |w| {
                    for token in tokens {
                        match token {
                            Token::Decimal(v) => w.a(2, |w| {
                                w.u(0u8)?;
                                w.dec(*v)
                            })?,
                            Token::Node { model, node } => w.a(3, |w| {
                                w.u(1u8)?;
                                w.id(*model)?;
                                w.id(*node)
                            })?,
                            Token::Unary(op) => w.a(2, |w| {
                                w.u(2u8)?;
                                w.u(*op)
                            })?,
                            Token::Binary(op) => w.a(2, |w| {
                                w.u(3u8)?;
                                w.u(*op)
                            })?,
                        }
                    }
                    Ok(())
                })
            }),
            Definition::ParseFailed {
                source,
                offset,
                code,
            } => self.a(4, |w| {
                w.u(2u8)?;
                w.text(source)?;
                w.u(*offset)?;
                w.u(*code)
            }),
            Definition::BindFailed { source, name } => self.a(3, |w| {
                w.u(3u8)?;
                w.text(source)?;
                w.text(name)
            }),
        }
    }
    fn graph_body(
        &mut self,
        name: &str,
        node_ids: &[Uuid],
        port_ids: &[Uuid],
        nodes: &[Node],
        ports: &[Port],
    ) -> Result<()> {
        let w = self;
        w.text(name)?;
        w.a(node_ids.len(), |w| {
            for id in node_ids {
                w.id(*id)?;
            }
            Ok(())
        })?;
        w.a(port_ids.len(), |w| {
            for id in port_ids {
                w.id(*id)?;
            }
            Ok(())
        })?;
        w.a(nodes.len(), |w| {
            for n in nodes {
                w.a(3, |w| {
                    w.id(n.id)?;
                    w.text(&n.name)?;
                    w.definition(&n.definition)
                })?;
            }
            Ok(())
        })?;
        w.a(ports.len(), |w| {
            for p in ports {
                w.a(5, |w| {
                    w.id(p.id)?;
                    w.text(&p.name)?;
                    w.id(p.model)?;
                    w.id(p.node)?;
                    match p.kind {
                        PortKind::Required => w.a(1, |w| w.u(0u8)),
                        PortKind::Optional(v) => w.a(2, |w| {
                            w.u(1u8)?;
                            w.dec(v)
                        }),
                        PortKind::Output => w.a(1, |w| w.u(2u8)),
                    }
                })?;
            }
            Ok(())
        })?;
        Ok(())
    }
    fn record(&mut self, r: &Record) -> Result<()> {
        self.a(14, |w| {
            w.u(r.codec)?;
            w.u(r.numeric)?;
            w.u(r.expression)?;
            w.id(r.store)?;
            w.id(r.epoch)?;
            w.id(r.model)?;
            w.id(r.revision)?;
            w.u(r.sequence)?;
            w.graph_body(&r.name, &r.node_ids, &r.port_ids, &r.nodes, &r.ports)?;
            if let Some(p) = &r.publication {
                w.a(4, |w| {
                    w.id(p.model)?;
                    w.id(p.revision)?;
                    w.id(p.snapshot)?;
                    w.a(p.public_ports.len(), |w| {
                        for (id, name) in &p.public_ports {
                            w.a(2, |w| {
                                w.id(*id)?;
                                w.text(name)
                            })?;
                        }
                        Ok(())
                    })
                })
            } else {
                w.a(0, |_| Ok(()))
            }
        })
    }
}
struct Reader<'a> {
    bytes: &'a [u8],
    pos: usize,
    items: usize,
    limits: Limits,
}
impl Reader<'_> {
    fn take(&mut self, n: usize) -> Result<&[u8]> {
        let end = self.pos.checked_add(n).ok_or(Error::Limit)?;
        let bytes = self.bytes.get(self.pos..end).ok_or(Error::Malformed)?;
        self.pos = end;
        Ok(bytes)
    }
    fn read(&mut self, depth: usize) -> Result<V> {
        if depth >= self.limits.depth || self.items == 0 {
            return Err(Error::Limit);
        }
        self.items -= 1;
        let h = self.take(1)?[0];
        let major = h >> 5;
        let extra = h & 31;
        let n = match extra {
            0..=23 => extra as u64,
            24..=27 => {
                let size = 1usize << (extra - 24);
                let mut data = [0; 8];
                data[8 - size..].copy_from_slice(self.take(size)?);
                let n = u64::from_be_bytes(data);
                let min = match size {
                    1 => 24,
                    2 => 256,
                    4 => 65536,
                    _ => 0x1_0000_0000,
                };
                if n < min {
                    return Err(Error::Malformed);
                }
                n
            }
            _ => return Err(Error::Malformed),
        };
        if major == 0 {
            return Ok(V::U(n));
        }
        let n = usize::try_from(n).map_err(|_| Error::Limit)?;
        match major {
            2 => Ok(V::B(self.take(n)?.to_vec())),
            3 => {
                if n > 4096 {
                    return Err(Error::Limit);
                }
                Ok(V::T(
                    std::str::from_utf8(self.take(n)?)
                        .map_err(|_| Error::Malformed)?
                        .into(),
                ))
            }
            4 => {
                if n > self.items || n > self.bytes.len().saturating_sub(self.pos) {
                    return Err(Error::Limit);
                }
                let mut values = Vec::new();
                for _ in 0..n {
                    values.push(self.read(depth + 1)?);
                }
                Ok(V::A(values))
            }
            _ => Err(Error::Malformed),
        }
    }
}
fn narrow<T: TryFrom<u64>>(v: V) -> Result<T> {
    T::try_from(v.u()?).map_err(|_| Error::Malformed)
}
fn next(values: &mut impl Iterator<Item = V>) -> Result<V> {
    values.next().ok_or(Error::Malformed)
}
type GraphBody = (String, Vec<Uuid>, Vec<Uuid>, Vec<Node>, Vec<Port>);
fn parse_graph_body(f: &mut impl Iterator<Item = V>) -> Result<GraphBody> {
    let name = next(f)?.text()?;
    let node_ids = next(f)?
        .list()?
        .into_iter()
        .map(V::uuid)
        .collect::<Result<_>>()?;
    let port_ids = next(f)?
        .list()?
        .into_iter()
        .map(V::uuid)
        .collect::<Result<_>>()?;
    let mut nodes = vec![];
    for n in next(f)?.list()? {
        let mut n = n.array(3)?.into_iter();
        let id = next(&mut n)?.uuid()?;
        let name = next(&mut n)?.text()?;
        let fields = next(&mut n)?.list()?;
        let mut d = fields.into_iter();
        let tag = next(&mut d)?.u()?;
        let definition = match tag {
            0 => Definition::Input(next(&mut d)?.decimal()?),
            1 => {
                let source = next(&mut d)?.text()?;
                let mut tokens = vec![];
                for t in next(&mut d)?.list()? {
                    let mut t = t.list()?.into_iter();
                    let token = match next(&mut t)?.u()? {
                        0 => Token::Decimal(next(&mut t)?.decimal()?),
                        1 => Token::Node {
                            model: next(&mut t)?.uuid()?,
                            node: next(&mut t)?.uuid()?,
                        },
                        2 => Token::Unary(narrow(next(&mut t)?)?),
                        3 => Token::Binary(narrow(next(&mut t)?)?),
                        _ => return Err(Error::UnsupportedVersion),
                    };
                    if t.next().is_some() {
                        return Err(Error::Malformed);
                    }
                    tokens.push(token);
                }
                Definition::Bound { source, tokens }
            }
            2 => Definition::ParseFailed {
                source: next(&mut d)?.text()?,
                offset: narrow(next(&mut d)?)?,
                code: narrow(next(&mut d)?)?,
            },
            3 => Definition::BindFailed {
                source: next(&mut d)?.text()?,
                name: next(&mut d)?.text()?,
            },
            _ => return Err(Error::UnsupportedVersion),
        };
        if d.next().is_some() {
            return Err(Error::Malformed);
        }
        nodes.push(Node {
            id,
            name,
            definition,
        });
    }
    let mut ports = vec![];
    for p in next(f)?.list()? {
        let mut p = p.array(5)?.into_iter();
        let id = next(&mut p)?.uuid()?;
        let name = next(&mut p)?.text()?;
        let model = next(&mut p)?.uuid()?;
        let node = next(&mut p)?.uuid()?;
        let mut k = next(&mut p)?.list()?.into_iter();
        let kind = match next(&mut k)?.u()? {
            0 => PortKind::Required,
            1 => PortKind::Optional(next(&mut k)?.decimal()?),
            2 => PortKind::Output,
            _ => return Err(Error::UnsupportedVersion),
        };
        if k.next().is_some() {
            return Err(Error::Malformed);
        }
        ports.push(Port {
            id,
            name,
            model,
            node,
            kind,
        });
    }
    Ok((name, node_ids, port_ids, nodes, ports))
}
fn parse(v: V) -> Result<Record> {
    let mut f = v.array(14)?.into_iter();
    let codec = narrow(next(&mut f)?)?;
    let numeric = narrow(next(&mut f)?)?;
    let expression = narrow(next(&mut f)?)?;
    let store = next(&mut f)?.uuid()?;
    let epoch = next(&mut f)?.uuid()?;
    let model = next(&mut f)?.uuid()?;
    let revision = next(&mut f)?.uuid()?;
    let sequence = next(&mut f)?.u()?;
    let (name, node_ids, port_ids, nodes, ports) = parse_graph_body(&mut f)?;
    let p = next(&mut f)?.list()?;
    let publication = if p.is_empty() {
        None
    } else {
        let mut p = V::A(p).array(4)?.into_iter();
        Some(Publication {
            model: next(&mut p)?.uuid()?,
            revision: next(&mut p)?.uuid()?,
            snapshot: next(&mut p)?.uuid()?,
            public_ports: next(&mut p)?
                .list()?
                .into_iter()
                .map(|v| {
                    let mut pair = v.array(2)?.into_iter();
                    Ok((next(&mut pair)?.uuid()?, next(&mut pair)?.text()?))
                })
                .collect::<Result<_>>()?,
        })
    };
    Ok(Record {
        codec,
        numeric,
        expression,
        store,
        epoch,
        model,
        revision,
        sequence,
        name,
        node_ids,
        port_ids,
        nodes,
        ports,
        publication,
    })
}
pub fn encode(record: &Record, limits: Limits) -> Result<Vec<u8>> {
    record.validate(limits)?;
    let mut writer = Writer {
        out: vec![],
        limits,
        depth: 0,
        items: limits.items,
    };
    writer.record(record)?;
    Ok(writer.out)
}
pub fn decode(bytes: &[u8], limits: Limits) -> Result<Record> {
    if bytes.len() > limits.bytes || limits.depth > 64 {
        return Err(Error::Limit);
    }
    let mut r = Reader {
        bytes,
        pos: 0,
        items: limits.items,
        limits,
    };
    let v = r.read(0)?;
    if r.pos != bytes.len() {
        return Err(Error::Malformed);
    }
    let record = parse(v)?;
    record.validate(limits)?;
    Ok(record)
}

// Experimental document payload only; hosted-v1 layout is unchanged.
pub fn encode_document(
    document: &crate::document::ModelDocument,
    limits: Limits,
) -> Result<Vec<u8>> {
    // Public envelope boundary validates graph limits independently of byte limits.
    let mut writer = Writer {
        out: vec![],
        limits,
        depth: 0,
        items: limits.items,
    };
    writer.a(4, |w| {
        w.u(document.format)?;
        w.id(document.document_id)?;
        w.id(document.state_id)?;
        let g = &document.graph;
        w.a(10, |w| {
            w.u(g.version)?;
            w.u(g.numeric)?;
            w.u(g.expression)?;
            w.id(g.model)?;
            w.id(g.revision)?;
            w.graph_body(&g.name, &g.node_ids, &g.port_ids, &g.nodes, &g.ports)
        })
    })?;
    Ok(writer.out)
}
pub fn decode_document(bytes: &[u8], limits: Limits) -> Result<crate::document::ModelDocument> {
    if bytes.len() > limits.bytes || limits.depth > 64 {
        return Err(Error::Limit);
    }
    let mut reader = Reader {
        bytes,
        pos: 0,
        items: limits.items,
        limits,
    };
    let v = reader.read(0)?;
    if reader.pos != bytes.len() {
        return Err(Error::Malformed);
    }
    let mut d = v.array(4)?.into_iter();
    let format = narrow(next(&mut d)?)?;
    let document_id = next(&mut d)?.uuid()?;
    let state_id = next(&mut d)?.uuid()?;
    let mut g = next(&mut d)?.array(10)?.into_iter();
    let version = narrow(next(&mut g)?)?;
    let numeric = narrow(next(&mut g)?)?;
    let expression = narrow(next(&mut g)?)?;
    let model = next(&mut g)?.uuid()?;
    let revision = next(&mut g)?.uuid()?;
    let (name, node_ids, port_ids, nodes, ports) = parse_graph_body(&mut g)?;
    let document = crate::document::ModelDocument {
        format,
        document_id,
        state_id,
        graph: GraphPayload {
            version,
            numeric,
            expression,
            model,
            revision,
            name,
            node_ids,
            port_ids,
            nodes,
            ports,
        },
    };
    // The envelope caller validates semantic/logical limits before installation.
    Ok(document)
}
