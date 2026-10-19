//! Portable storage records and restoration only. No database or durability claim.
mod codec;
pub mod document;
mod graph;
mod runtime;
pub use codec::{decode, encode};
pub use graph::{EditableGraph, GraphPayload, NamedValues, Values};
pub use runtime::{
    Diagnostic, DiagnosticKind, RestoredModel, capture, capture_snapshot, restore, restore_all,
};
use std::{collections::BTreeSet, fmt};
pub use uuid::Uuid;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Error {
    Limit,
    Malformed,
    UnsupportedVersion,
    Identity,
    InvalidPublished,
    InvalidDraft,
    Runtime,
}
impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "storage {self:?}")
    }
}
impl std::error::Error for Error {}
pub type Result<T> = std::result::Result<T, Error>;

#[derive(Clone, Copy, Debug)]
pub struct Limits {
    pub bytes: usize,
    pub depth: usize,
    pub items: usize,
    pub nodes: usize,
    pub ports: usize,
    pub identities: usize,
    pub work: usize,
}
impl Default for Limits {
    fn default() -> Self {
        Self {
            bytes: 1_048_576,
            depth: 16,
            items: 100_000,
            nodes: 1024,
            ports: 256,
            identities: 8192,
            work: 1_048_576,
        }
    }
}
/// Exact portable representation, independent of runtime normalization.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Decimal96(pub [u8; 16]);
impl Decimal96 {
    pub fn from_parts(lo: u32, mid: u32, hi: u32, negative: bool, scale: u32) -> Result<Self> {
        if scale > 28 {
            return Err(Error::Malformed);
        }
        let mut bytes = [0; 16];
        for (i, word) in [
            lo,
            mid,
            hi,
            (scale << 16) | if negative { 1 << 31 } else { 0 },
        ]
        .iter()
        .enumerate()
        {
            bytes[i * 4..i * 4 + 4].copy_from_slice(&word.to_le_bytes());
        }
        Ok(Self(bytes))
    }
    pub fn parts(self) -> Result<(u32, u32, u32, bool, u32)> {
        let words: [u32; 4] = std::array::from_fn(|i| {
            u32::from_le_bytes(self.0[i * 4..i * 4 + 4].try_into().unwrap())
        });
        if words[3] & 0x7f00ffff != 0 || (words[3] >> 16) & 255 > 28 {
            return Err(Error::Malformed);
        }
        Ok((
            words[0],
            words[1],
            words[2],
            words[3] >> 31 != 0,
            (words[3] >> 16) & 255,
        ))
    }
    pub fn runtime(self) -> Result<pivot_types::PivotDecimal> {
        let (a, b, c, s, e) = self.parts()?;
        pivot_types::PivotDecimal::from_parts(a, b, c, s, e).map_err(|_| Error::Malformed)
    }
    pub fn from_runtime(v: pivot_types::PivotDecimal) -> Self {
        let (a, b, c, s, e) = v.parts();
        Self::from_parts(a, b, c, s, e).expect("valid runtime decimal")
    }
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Token {
    Decimal(Decimal96),
    Node { model: Uuid, node: Uuid },
    Unary(u8),
    Binary(u8),
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Definition {
    Input(Decimal96),
    Bound {
        source: String,
        tokens: Vec<Token>,
    },
    ParseFailed {
        source: String,
        offset: u32,
        code: u8,
    },
    BindFailed {
        source: String,
        name: String,
    },
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Node {
    pub id: Uuid,
    pub name: String,
    pub definition: Definition,
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum PortKind {
    Required,
    Optional(Decimal96),
    Output,
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Port {
    pub id: Uuid,
    pub name: String,
    pub model: Uuid,
    pub node: Uuid,
    pub kind: PortKind,
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Publication {
    pub model: Uuid,
    pub revision: Uuid,
    pub snapshot: Uuid,
    pub public_ports: Vec<(Uuid, String)>,
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Record {
    pub codec: u32,
    pub numeric: u32,
    pub expression: u32,
    pub store: Uuid,
    pub epoch: Uuid,
    pub model: Uuid,
    pub revision: Uuid,
    pub sequence: u64,
    pub name: String,
    pub node_ids: Vec<Uuid>,
    pub port_ids: Vec<Uuid>,
    pub nodes: Vec<Node>,
    pub ports: Vec<Port>,
    pub publication: Option<Publication>,
}
#[derive(Clone, Debug)]
pub struct Header {
    pub store: Uuid,
    pub epoch: Uuid,
    pub model: Uuid,
    pub revision: Uuid,
    pub sequence: u64,
    pub publication: Option<Publication>,
}
fn identifier(s: &str) -> bool {
    let mut bytes = s.bytes();
    bytes
        .next()
        .is_some_and(|b| b.is_ascii_alphabetic() || b == b'_')
        && s.len() <= 128
        && bytes.all(|b| b.is_ascii_alphanumeric() || b == b'_')
}
fn ordered(ids: impl IntoIterator<Item = Uuid>) -> Result<BTreeSet<Uuid>> {
    let mut set = BTreeSet::new();
    let mut previous = None;
    for id in ids {
        if id.is_nil() || previous.is_some_and(|p| p >= id) {
            return Err(Error::Identity);
        }
        previous = Some(id);
        set.insert(id);
    }
    Ok(set)
}
impl Record {
    pub fn validate(&self, limits: Limits) -> Result<()> {
        if self.codec != 1 || self.numeric != 1 || self.expression != 1 {
            return Err(Error::UnsupportedVersion);
        }
        if self.store.is_nil() || self.epoch.is_nil() {
            return Err(Error::Malformed);
        }
        GraphView {
            model: self.model,
            revision: self.revision,
            name: &self.name,
            node_ids: &self.node_ids,
            port_ids: &self.port_ids,
            nodes: &self.nodes,
            ports: &self.ports,
        }
        .validate(limits)?;
        if let Some(p) = &self.publication {
            if [p.model, p.revision, p.snapshot].iter().any(Uuid::is_nil) {
                return Err(Error::Identity);
            }
            let mapped = ordered(p.public_ports.iter().map(|(id, _)| *id))?;
            if mapped != self.ports.iter().map(|p| p.id).collect() {
                return Err(Error::Identity);
            }
            let mut labels = BTreeSet::new();
            for (_, id) in &p.public_ports {
                if id.is_empty()
                    || id.len() > 128
                    || !id
                        .bytes()
                        .all(|b| b.is_ascii_alphanumeric() || b"._-".contains(&b))
                    || !labels.insert(id)
                {
                    return Err(Error::Identity);
                }
            }
        }
        Ok(())
    }
}

pub(crate) struct GraphView<'a> {
    pub model: Uuid,
    pub revision: Uuid,
    pub name: &'a str,
    pub node_ids: &'a [Uuid],
    pub port_ids: &'a [Uuid],
    pub nodes: &'a [Node],
    pub ports: &'a [Port],
}
impl GraphView<'_> {
    fn validate(&self, limits: Limits) -> Result<()> {
        if limits.depth == 0
            || limits.depth > 64
            || self.nodes.len() > limits.nodes.min(4096)
            || self.ports.len() > limits.ports.min(4096)
            || self.node_ids.len().saturating_add(self.port_ids.len()) > limits.identities.min(8192)
        {
            return Err(Error::Limit);
        }
        if [self.model, self.revision].iter().any(Uuid::is_nil)
            || self.name.is_empty()
            || self.name.len() > 128
        {
            return Err(Error::Malformed);
        }
        let nodes = ordered(self.node_ids.iter().copied())?;
        let ports = ordered(self.port_ids.iter().copied())?;
        if !nodes.is_disjoint(&ports) {
            return Err(Error::Identity);
        }
        ordered(self.nodes.iter().map(|n| n.id))?;
        ordered(self.ports.iter().map(|p| p.id))?;
        let mut names = BTreeSet::new();
        let mut work = self
            .nodes
            .len()
            .checked_add(self.ports.len())
            .and_then(|n| n.checked_add(self.node_ids.len()))
            .and_then(|n| n.checked_add(self.port_ids.len()))
            .ok_or(Error::Limit)?;
        let work_limit = limits.work.min(limits.bytes);
        if work > work_limit {
            return Err(Error::Limit);
        }
        for n in self.nodes {
            if !nodes.contains(&n.id) || !identifier(&n.name) || !names.insert(&n.name) {
                return Err(Error::Identity);
            }
            let source = match &n.definition {
                Definition::Input(v) => {
                    v.parts()?;
                    continue;
                }
                Definition::Bound { source, .. }
                | Definition::ParseFailed { source, .. }
                | Definition::BindFailed { source, .. } => source,
            };
            if source.len() > 4096 {
                return Err(Error::Limit);
            }
            work = work.checked_add(source.len()).ok_or(Error::Limit)?;
            match &n.definition {
                Definition::Bound { tokens, .. } => {
                    if tokens.len() > limits.items {
                        return Err(Error::Limit);
                    }
                    work = work.checked_add(tokens.len()).ok_or(Error::Limit)?;
                    if work > work_limit {
                        return Err(Error::Limit);
                    }
                    let mut depth = 0usize;
                    for token in tokens {
                        match token {
                            Token::Decimal(v) => {
                                v.parts()?;
                                depth += 1;
                            }
                            Token::Node { model, node } => {
                                if *model != self.model || !nodes.contains(node) {
                                    return Err(Error::Identity);
                                }
                                depth += 1;
                            }
                            Token::Unary(op) => {
                                if *op > 1 {
                                    return Err(Error::UnsupportedVersion);
                                }
                                if depth < 1 {
                                    return Err(Error::Malformed);
                                }
                            }
                            Token::Binary(op) => {
                                if *op > 3 {
                                    return Err(Error::UnsupportedVersion);
                                }
                                if depth < 2 {
                                    return Err(Error::Malformed);
                                }
                                depth -= 1;
                            }
                        }
                    }
                    if depth != 1 {
                        return Err(Error::Malformed);
                    }
                }
                Definition::ParseFailed { offset, code, .. } => {
                    if *code > 5 {
                        return Err(Error::UnsupportedVersion);
                    }
                    if *offset as usize > source.len() || !source.is_char_boundary(*offset as usize)
                    {
                        return Err(Error::Malformed);
                    }
                }
                Definition::BindFailed { name, .. } => {
                    if !identifier(name) {
                        return Err(Error::Malformed);
                    }
                }
                _ => unreachable!(),
            }
            if work > work_limit {
                return Err(Error::Limit);
            }
        }
        let mut labels = BTreeSet::new();
        let mut inputs = BTreeSet::new();
        for p in self.ports {
            if !ports.contains(&p.id)
                || !identifier(&p.name)
                || !labels.insert(&p.name)
                || p.model != self.model
                || !nodes.contains(&p.node)
            {
                return Err(Error::Identity);
            }
            if !matches!(p.kind, PortKind::Output) && !inputs.insert(p.node) {
                return Err(Error::Malformed);
            }
            if let PortKind::Optional(v) = p.kind {
                v.parts()?;
            }
        }
        Ok(())
    }
}
