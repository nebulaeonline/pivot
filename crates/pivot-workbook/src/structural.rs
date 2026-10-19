//! Fixed-projection adapter for the paged graph experiment. No default API changes.
//! Existing anchors only: insertion, moves, retirement and axis edits are not optimized here.
use super::*;
use pivot_core::model::structural::{self as graph, Pages, Version};
use std::sync::Arc;

/// Trusted benchmark-only bulk preparation; not an optimized user import path.
pub fn build(size: usize, shape: &str) -> Workbook {
    let mut book = crate::scaling::build(size, shape);
    if shape == "independent" {
        for i in size - 128..size {
            book.edit_text(crate::scaling::position(i), "=A1+1", book.expected(), 0)
                .unwrap();
        }
        book.budget().unwrap();
    }
    book
}
pub fn formula_position(size: usize, shape: &str, i: usize) -> Position {
    let index = match shape {
        "chain" => 1 + i % (size - 1).min(1000),
        "mixed" => 4 + 5 * (i % (size / 5)),
        "fanout" => 1 + i % (size - 1),
        _ => size - 1 - i % 128,
    };
    crate::scaling::position(index)
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Metadata {
    Original(Content),
    Decimal(DecimalInput),
    Formula {
        source: String,
        references: Vec<FormulaReference>,
        diagnostic: Option<ParseError>,
    },
}
#[derive(Clone)]
struct Projection {
    book: Arc<Workbook>,
    slots: Arc<BTreeMap<CellId, u64>>,
    metadata: Pages<Metadata>,
    bytes: usize,
}
#[derive(Clone)]
pub struct StructuralWorkbook {
    projection: Projection,
    graph: graph::Graph,
}
#[derive(Debug, PartialEq, Eq)]
pub enum Failure {
    Workbook(Error),
    Graph(graph::Error),
    Snapshot(SnapshotError),
}
impl StructuralWorkbook {
    pub fn new(mut book: Workbook) -> std::result::Result<Self, Failure> {
        let graph = graph::Graph::from_model(&book.core, graph::Limits::default())
            .map_err(Failure::Graph)?;
        // The frozen projection needs identities/content, never the old mutable core.
        book.core = WorkingModel::new("Unused projection shell")
            .map_err(|e| Failure::Workbook(Error::Graph(e.to_string())))?;
        let slots = book
            .data
            .bindings
            .keys()
            .enumerate()
            .map(|(i, &c)| (c, i as u64))
            .collect();
        let bytes = book
            .data
            .records
            .values()
            .map(|r| r.content.as_ref().map_or(0, content_bytes))
            .sum();
        Ok(Self {
            graph,
            projection: Projection {
                book: Arc::new(book),
                slots: Arc::new(slots),
                metadata: Pages::default(),
                bytes,
            },
        })
    }
    pub fn version(&self) -> Version {
        self.graph.version()
    }
    pub fn metadata(&self, position: Position) -> std::result::Result<Metadata, Failure> {
        let p = &self.projection;
        let cell = p
            .book
            .cell_at(position)
            .map_err(Failure::Workbook)?
            .ok_or(Failure::Workbook(Error::InvalidTarget))?;
        Ok(p.metadata
            .get(p.slots[&cell.reference.cell])
            .map_or(Metadata::Original(cell.content), |m| (**m).clone()))
    }
    pub fn apply(
        &mut self,
        expected: Version,
        edits: &[Edit],
    ) -> std::result::Result<Version, Failure> {
        if expected != self.version() {
            return Err(Failure::Graph(graph::Error::Stale));
        }
        if edits.len() > self.projection.book.limits.edits {
            return Err(Failure::Workbook(Error::Limit));
        }
        let mut candidate = {
            let _phase = pivot_core::experiment_profile::scope(2);
            self.clone()
        };
        let mut graph_edits = Vec::with_capacity(edits.len());
        for edit in edits {
            let (position, source) = match edit {
                Edit::SetFormulaText { position, source } => (*position, Some(source)),
                Edit::SetDecimal { position, .. } => (*position, None),
                _ => return Err(Failure::Workbook(Error::InvalidTarget)),
            };
            let p = &mut candidate.projection;
            let cell = p
                .book
                .cell_at(position)
                .map_err(Failure::Workbook)?
                .ok_or(Failure::Workbook(Error::InvalidTarget))?;
            let id = p.book.data.bindings[&cell.reference.cell].runtime;
            let slot = p.slots[&cell.reference.cell];
            let old_bytes = p
                .metadata
                .get(slot)
                .map_or_else(|| content_bytes(&cell.content), |m| metadata_bytes(m));
            let metadata = if let Some(source) = source {
                let _phase = pivot_core::experiment_profile::scope(5);
                if source.len() > p.book.limits.source_bytes {
                    return Err(Failure::Workbook(Error::Limit));
                }
                let parsed = expression::parse_spreadsheet(source);
                let (tokens, references, diagnostic) = p.book.lower_text(&parsed);
                if tokens.len() > p.book.limits.formula_tokens {
                    return Err(Failure::Workbook(Error::Limit));
                }
                let mut bound = Vec::new();
                let mut bound_refs = Vec::new();
                if let Some(error) = &diagnostic {
                    graph_edits.push(graph::Edit::Invalid(id, source.clone(), error.clone()));
                } else {
                    for token in tokens {
                        bound.push(match token {
                            Token::Decimal(v) => ResolvedToken::Decimal(v.value()),
                            Token::Unary(op) => ResolvedToken::Unary(op),
                            Token::Binary(op) => ResolvedToken::Binary(op),
                            Token::At(pos) => {
                                let target = p
                                    .book
                                    .cell_at(pos)
                                    .map_err(Failure::Workbook)?
                                    .ok_or(Failure::Workbook(Error::InvalidTarget))?;
                                ResolvedToken::Node(
                                    p.book.data.bindings[&target.reference.cell].runtime,
                                )
                            }
                            Token::Cell(_) => unreachable!("text lowering emits positions"),
                        });
                    }
                    for (span, pos) in references {
                        let target = p
                            .book
                            .cell_at(pos)
                            .map_err(Failure::Workbook)?
                            .ok_or(Failure::Workbook(Error::InvalidTarget))?;
                        bound_refs.push(FormulaReference {
                            span,
                            submitted_position: pos,
                            target: target.reference,
                            node: target.node,
                        });
                    }
                    graph_edits.push(graph::Edit::Resolved(id, source.clone(), bound));
                }
                Metadata::Formula {
                    source: source.clone(),
                    references: bound_refs,
                    diagnostic,
                }
            } else {
                let Edit::SetDecimal { value, .. } = edit else {
                    unreachable!()
                };
                graph_edits.push(graph::Edit::ReplaceInput(id, value.value()));
                Metadata::Decimal(*value)
            };
            p.bytes = p.bytes - old_bytes + metadata_bytes(&metadata);
            if p.bytes > p.book.limits.bytes {
                return Err(Failure::Workbook(Error::Limit));
            }
            p.metadata.set(slot, Arc::new(metadata));
        }
        let version = candidate
            .graph
            .apply(expected, graph_edits)
            .map_err(Failure::Graph)?;
        *self = candidate;
        Ok(version)
    }
    pub fn capture(&self) -> std::result::Result<Capture, Failure> {
        Ok(Capture {
            version: self.version(),
            graph: self.graph.capture().map_err(Failure::Snapshot)?,
            projection: self.projection.clone(),
        })
    }
}
fn content_bytes(content: &Content) -> usize {
    match content {
        Content::TextFormula(f) => {
            f.original_source.len() + f.references.len() * std::mem::size_of::<FormulaReference>()
        }
        Content::Formula { source, tokens } => {
            source.len() + tokens.len() * std::mem::size_of::<Token>()
        }
        _ => std::mem::size_of::<DecimalInput>(),
    }
}
fn metadata_bytes(metadata: &Metadata) -> usize {
    match metadata {
        Metadata::Original(c) => content_bytes(c),
        Metadata::Decimal(_) => std::mem::size_of::<DecimalInput>(),
        Metadata::Formula {
            source, references, ..
        } => source.len() + references.len() * std::mem::size_of::<FormulaReference>(),
    }
}
#[derive(Clone)]
pub struct Capture {
    pub version: Version,
    graph: graph::Capture,
    projection: Projection,
}
impl Capture {
    pub fn metadata(&self, pos: Position) -> std::result::Result<Metadata, Failure> {
        let p = &self.projection;
        let cell = p
            .book
            .cell_at(pos)
            .map_err(Failure::Workbook)?
            .ok_or(Failure::Workbook(Error::InvalidTarget))?;
        Ok(p.metadata
            .get(p.slots[&cell.reference.cell])
            .map_or(Metadata::Original(cell.content), |m| (**m).clone()))
    }
    pub fn value(&self, pos: Position) -> std::result::Result<PivotDecimal, Failure> {
        let cell = self
            .projection
            .book
            .cell_at(pos)
            .map_err(Failure::Workbook)?
            .ok_or(Failure::Workbook(Error::InvalidTarget))?;
        let id = self.projection.book.data.bindings[&cell.reference.cell].runtime;
        self.graph
            .evaluate(&[id])
            .map_err(|_| Failure::Workbook(Error::InvalidTarget))?[&id]
            .clone()
            .map_err(|e| Failure::Workbook(Error::Graph(format!("{e}"))))
    }
}
