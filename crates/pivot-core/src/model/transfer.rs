//! Slot-based, owned graph transfer. No parsing or name binding on import.
use super::*;

#[derive(Clone, Debug)]
pub enum Token {
    Decimal(PivotDecimal),
    Node(usize),
    Unary(UnaryOp),
    Binary(BinaryOp),
}
#[derive(Clone, Debug)]
pub enum Binding {
    Bound(Vec<Token>),
    ParseFailed { offset: usize, code: u8 },
    Unresolved(String),
}
#[derive(Clone, Debug)]
pub enum NodeDefinition {
    /// Valid nonnumeric worksheet content; no string payload in the graph.
    NonNumeric,
    Input(PivotDecimal),
    Computed {
        source: String,
        binding: Binding,
    },
}
#[derive(Clone, Debug)]
pub struct Node {
    pub slot: usize,
    pub name: String,
    pub definition: NodeDefinition,
}
#[derive(Clone, Debug)]
pub struct Port {
    pub slot: usize,
    pub name: String,
    pub node: usize,
    pub requirement: Option<InputRequirement>,
}
#[derive(Clone, Debug)]
pub struct Image {
    pub name: String,
    pub node_slots: usize,
    pub port_slots: usize,
    pub nodes: Vec<Node>,
    pub ports: Vec<Port>,
}
pub struct Imported {
    pub model: WorkingModel,
    pub nodes: Vec<NodeId>,
    pub ports: Vec<PortId>,
}

const MESSAGES: [&str; 6] = [
    "expression too long",
    "unexpected trailing input",
    "expression nesting too deep",
    "expected closing parenthesis",
    "invalid or unrepresentable decimal",
    "expected decimal, name, sign, or parenthesis",
];
fn invalid() -> ModelError {
    ModelError::InvalidName("invalid graph transfer".into())
}

/// Internal already-resolved expression input. Not a parser, wire or storage ABI.
#[derive(Clone, Debug)]
pub enum ResolvedToken {
    Sum(super::selection::SelectionId),
    Decimal(PivotDecimal),
    Node(NodeId),
    Unary(UnaryOp),
    Binary(BinaryOp),
}
pub(super) fn checked_dependencies(tokens: &[BoundToken]) -> Result<Vec<NodeId>, ModelError> {
    if tokens.len() > 1_048_576 {
        return Err(invalid());
    }
    let mut depth = 0usize;
    let mut dependencies = BTreeSet::new();
    for token in tokens {
        match token {
            BoundToken::Decimal(_) | BoundToken::Sum(_) => depth += 1,
            BoundToken::Node(id) => {
                depth += 1;
                dependencies.insert(*id);
            }
            BoundToken::Unary(_) => {
                if depth < 1 {
                    return Err(invalid());
                }
            }
            BoundToken::Binary(_) => {
                if depth < 2 {
                    return Err(invalid());
                }
                depth -= 1;
            }
        }
    }
    if depth != 1 {
        return Err(invalid());
    }
    Ok(dependencies.into_iter().collect())
}
/// Validated trusted projection seam: preserve runtime identity and existing
/// evaluator semantics, without parsing synthetic names or historical source.
/// References to this model's allocated retired nodes remain diagnosable drafts.
pub fn set_resolved_expression(
    model: &mut WorkingModel,
    id: NodeId,
    source: &str,
    tokens: &[ResolvedToken],
) -> Result<(), ModelError> {
    if !model.contains_node(id) {
        return Err(ModelError::UnknownNode(id));
    }
    if source.len() > 4096 || tokens.len() > 1_048_576 {
        return Err(invalid());
    }
    let _copy = model.integrated_request_charge(
        4096 + tokens
            .len()
            .saturating_mul(std::mem::size_of::<ResolvedToken>()),
    )?;
    model.integrated_edit(super::structural::Edit::Resolved(
        id,
        source.into(),
        tokens.to_vec(),
    ))
}

/// Internal rejected-formula seam. The workbook retains its richer diagnostic;
/// the core stores a non-executable draft using an existing transfer error code.
/// No invalid formula is represented as a numeric input.
pub fn set_invalid_expression(
    model: &mut WorkingModel,
    id: NodeId,
    source: &str,
    offset: usize,
) -> Result<(), ModelError> {
    if !model.contains_node(id) {
        return Err(ModelError::UnknownNode(id));
    }
    if source.len() > 4096 || offset > source.len() || !source.is_char_boundary(offset) {
        return Err(invalid());
    }
    model.integrated_edit(super::structural::Edit::Invalid(
        id,
        source.into(),
        ParseError {
            offset,
            message: "unexpected trailing input",
        },
    ))
}

/// Caller supplies complete allocated identity ledgers, including retired IDs.
pub fn export(
    model: &WorkingModel,
    nodes: &[NodeId],
    ports: &[PortId],
) -> Result<Image, ModelError> {
    model.integrated_export(nodes, ports)
}
pub(super) fn export_image(
    model: &GraphImage,
    nodes: &[NodeId],
    ports: &[PortId],
) -> Result<Image, ModelError> {
    if nodes.len() > 8192 || ports.len() > 8192 || model.nodes.len() > 4096 {
        return Err(invalid());
    }
    let mut work = nodes.len().saturating_add(ports.len());
    for n in model.nodes.values() {
        if let Definition::Computed { source, binding } = &n.definition {
            work = work.saturating_add(source.len());
            if let Ok((tokens, _)) = binding {
                work = work.saturating_add(tokens.len());
            }
        }
        if work > 1_048_576 {
            return Err(invalid());
        }
    }
    let node_slots: BTreeMap<_, _> = nodes.iter().enumerate().map(|(i, &id)| (id, i)).collect();
    let port_slots: BTreeMap<_, _> = ports.iter().enumerate().map(|(i, &id)| (id, i)).collect();
    if node_slots.len() != nodes.len()
        || port_slots.len() != ports.len()
        || nodes.len() as u64 != model.next_node
        || ports.len() as u64 != model.next_port
        || nodes
            .iter()
            .any(|n| n.model != model.revision.model || n.node >= model.next_node)
        || ports.iter().any(|p| {
            p.model != model.revision.model || p.sequence == 0 || p.sequence > model.next_port
        })
    {
        return Err(invalid());
    }
    let mut image = Image {
        name: model.name.clone(),
        node_slots: nodes.len(),
        port_slots: ports.len(),
        nodes: vec![],
        ports: vec![],
    };
    for (&id, n) in &model.nodes {
        let definition = match &n.definition {
            Definition::NonNumeric => NodeDefinition::NonNumeric,
            Definition::Input(v) => NodeDefinition::Input(*v),
            Definition::Computed { source, binding } => NodeDefinition::Computed {
                source: source.clone(),
                binding: match binding {
                    Ok((tokens, _)) => Binding::Bound(
                        tokens
                            .iter()
                            .map(|t| {
                                Ok(match t {
                                    BoundToken::Sum(_) => {
                                        return Err(ModelError::Backend(
                                            "unsupported selection capability in scalar transfer",
                                        ));
                                    }
                                    BoundToken::Decimal(v) => Token::Decimal(*v),
                                    BoundToken::Node(id) => {
                                        Token::Node(*node_slots.get(id).ok_or_else(invalid)?)
                                    }
                                    BoundToken::Unary(op) => Token::Unary(*op),
                                    BoundToken::Binary(op) => Token::Binary(*op),
                                })
                            })
                            .collect::<Result<_, ModelError>>()?,
                    ),
                    Err(SnapshotError::Parse { error, .. }) => Binding::ParseFailed {
                        offset: error.offset,
                        code: MESSAGES
                            .iter()
                            .position(|m| *m == error.message)
                            .ok_or_else(invalid)? as u8,
                    },
                    Err(SnapshotError::UnknownName { name, .. }) => {
                        Binding::Unresolved(name.clone())
                    }
                    _ => return Err(invalid()),
                },
            },
        };
        image.nodes.push(Node {
            slot: *node_slots.get(&id).ok_or_else(invalid)?,
            name: n.name.clone(),
            definition,
        });
    }
    for p in model.interface.inputs() {
        image.ports.push(Port {
            slot: *port_slots.get(&p.id).ok_or_else(invalid)?,
            name: p.name.clone(),
            node: *node_slots.get(&p.node).ok_or_else(invalid)?,
            requirement: Some(p.requirement.clone()),
        });
    }
    for p in model.interface.outputs() {
        image.ports.push(Port {
            slot: *port_slots.get(&p.id).ok_or_else(invalid)?,
            name: p.name.clone(),
            node: *node_slots.get(&p.node).ok_or_else(invalid)?,
            requirement: None,
        });
    }
    Ok(image)
}

/// Exports captured definitions, never the possibly edited source WorkingModel.
pub fn export_snapshot(
    snapshot: &ModelSnapshot,
    nodes: &[NodeId],
    ports: &[PortId],
) -> Result<Image, ModelError> {
    let data = &snapshot.0;
    if data
        .nodes
        .values()
        .any(|n| matches!(n.definition, SnapshotDefinition::NonNumeric))
    {
        return Err(ModelError::Backend(
            "unsupported nonnumeric content in scalar export",
        ));
    }
    if data.nodes.len() > 4096 || nodes.len() > 8192 || ports.len() > 8192 {
        return Err(invalid());
    }
    let mut work = nodes.len().saturating_add(ports.len());
    for n in data.nodes.values() {
        if let SnapshotDefinition::Computed { source, tokens } = &n.definition {
            work = work
                .saturating_add(source.len())
                .saturating_add(tokens.len());
        }
        if work > 1_048_576 {
            return Err(invalid());
        }
    }
    let mut working = GraphImage {
        name: data.name.clone(),
        revision: data.revision,
        nodes: BTreeMap::new(),
        names: data.names.clone(),
        next_node: nodes.len() as u64,
        next_port: ports.len() as u64,
        interface: data.interface.clone(),
    };
    for (&id, n) in &data.nodes {
        let definition = match &n.definition {
            SnapshotDefinition::NonNumeric => Definition::NonNumeric,
            SnapshotDefinition::Input(v) => Definition::Input(*v),
            SnapshotDefinition::Computed { source, tokens } => Definition::Computed {
                source: source.clone(),
                binding: Ok((tokens.clone(), n.dependencies.clone())),
            },
        };
        working.nodes.insert(
            id,
            WorkingNode {
                name: n.name.clone(),
                definition,
            },
        );
    }
    export_image(&working, nodes, ports)
}

/// Validates before installing private core fields. Invalid *draft semantics*
/// (cycles, retired targets, computed input-port targets) deliberately survive.
pub(super) fn check_header(image: &Image) -> Result<(), ModelError> {
    if image.node_slots > 8192
        || image.port_slots > 8192
        || image.nodes.len() > 4096
        || image.ports.len() > 4096
        || image.name.is_empty()
        || image.name.len() > 128
    {
        return Err(invalid());
    }
    Ok(())
}
pub fn import(image: Image) -> Result<Imported, ModelError> {
    integration::IntegrationRuntime::with_convenience(|runtime| {
        runtime.import_portable(image, &integration::CancellationToken::default())
    })?
}
pub(super) struct Decoded {
    pub model: GraphImage,
    pub nodes: Vec<NodeId>,
    pub ports: Vec<PortId>,
}
pub(super) fn decode(image: Image) -> Result<Decoded, ModelError> {
    check_header(&image)?;
    let mut model = GraphImage::empty(image.name)?;
    let nodes: Vec<_> = (0..image.node_slots)
        .map(|i| NodeId {
            model: model.revision.model,
            node: i as u64,
        })
        .collect();
    let ports: Vec<_> = (0..image.port_slots)
        .map(|i| PortId {
            model: model.revision.model,
            sequence: i as u64 + 1,
        })
        .collect();
    let mut work = 0usize;
    for n in image.nodes {
        let id = *nodes.get(n.slot).ok_or_else(invalid)?;
        if model.nodes.contains_key(&id) || n.name.len() > 128 {
            return Err(invalid());
        }
        model.validate_name(&n.name)?;
        let definition = match n.definition {
            NodeDefinition::NonNumeric => Definition::NonNumeric,
            NodeDefinition::Input(v) => Definition::Input(v),
            NodeDefinition::Computed { source, binding } => {
                if source.len() > 4096 {
                    return Err(invalid());
                }
                work = work.checked_add(source.len()).ok_or_else(invalid)?;
                let binding = match binding {
                    Binding::Bound(tokens) => {
                        work = work.checked_add(tokens.len()).ok_or_else(invalid)?;
                        if work > 1_048_576 {
                            return Err(invalid());
                        }
                        let mut bound = Vec::new();
                        for token in tokens {
                            bound.push(match token {
                                Token::Decimal(v) => BoundToken::Decimal(v),
                                Token::Node(slot) => {
                                    BoundToken::Node(*nodes.get(slot).ok_or_else(invalid)?)
                                }
                                Token::Unary(op) => BoundToken::Unary(op),
                                Token::Binary(op) => BoundToken::Binary(op),
                            });
                        }
                        let dependencies = checked_dependencies(&bound)?;
                        Ok((bound, dependencies))
                    }
                    Binding::ParseFailed { offset, code } => {
                        if offset > source.len() || !source.is_char_boundary(offset) {
                            return Err(invalid());
                        }
                        Err(SnapshotError::Parse {
                            node: id,
                            error: ParseError {
                                offset,
                                message: MESSAGES.get(code as usize).ok_or_else(invalid)?,
                            },
                        })
                    }
                    Binding::Unresolved(name) => {
                        let mut bytes = name.bytes();
                        if name.len() > 128
                            || !bytes
                                .next()
                                .is_some_and(|b| b.is_ascii_alphabetic() || b == b'_')
                            || !bytes.all(|b| b.is_ascii_alphanumeric() || b == b'_')
                        {
                            return Err(invalid());
                        }
                        Err(SnapshotError::UnknownName { node: id, name })
                    }
                };
                if work > 1_048_576 {
                    return Err(invalid());
                }
                Definition::Computed { source, binding }
            }
        };
        model.names.insert(n.name.clone(), id);
        model.nodes.insert(
            id,
            WorkingNode {
                name: n.name,
                definition,
            },
        );
    }
    let mut used = BTreeSet::new();
    let mut inputs = BTreeSet::new();
    for p in image.ports {
        model.port_name(&p.name)?;
        if p.name.len() > 128 {
            return Err(invalid());
        }
        let id = *ports.get(p.slot).ok_or_else(invalid)?;
        let node = *nodes.get(p.node).ok_or_else(invalid)?;
        if !used.insert(id) {
            return Err(invalid());
        }
        if let Some(requirement) = p.requirement {
            if let InputRequirement::Optional { default } = &requirement
                && default.value_type() != pivot_types::ValueType::Decimal
            {
                return Err(invalid());
            }
            if !inputs.insert(node) {
                return Err(invalid());
            }
            model.interface.inputs.insert(
                p.name.clone(),
                InputPort {
                    id,
                    name: p.name,
                    node,
                    requirement,
                },
            );
        } else {
            model.interface.outputs.insert(
                p.name.clone(),
                OutputPort {
                    id,
                    name: p.name,
                    node,
                },
            );
        }
    }
    model.next_node = nodes.len() as u64;
    model.next_port = ports.len() as u64;
    Ok(Decoded {
        model,
        nodes,
        ports,
    })
}

/// Selection-aware logical transfer. The scalar skeleton deliberately contains
/// non-executable drafts for overlay definitions; it is not a range-free export.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct SelectionFormula {
    pub slot: usize,
    pub source: String,
    pub tokens: Vec<SelectionToken>,
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum SelectionToken {
    Decimal(PivotDecimal),
    Node(usize),
    Sum(selection::SelectionId),
    Unary(UnaryOp),
    Binary(BinaryOp),
}
#[derive(Clone, Debug)]
pub struct SelectionImage {
    pub version: u32,
    pub scalar_skeleton: Image,
    pub formulas: Vec<SelectionFormula>,
    pub identities: Vec<(selection::SelectionId, Option<selection::Rectangle>)>,
    pub points: Vec<(selection::SpaceId, u32, u32, usize)>,
}
impl WorkingModel {
    pub fn export_selections(
        &self,
        nodes: &[NodeId],
        ports: &[PortId],
    ) -> Result<SelectionImage, ModelError> {
        self.integrated_export_selections(nodes, ports)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn rejected_drafts_validate_before_mutation_and_retired_references_remain_bound() {
        let mut m = WorkingModel::new("Seam").unwrap();
        let a = m.add_input("A", PivotDecimal::ONE).unwrap();
        let b = m.add_input("B", PivotDecimal::ZERO).unwrap();
        let mut other = WorkingModel::new("Other").unwrap();
        let foreign = other.add_input("X", PivotDecimal::ZERO).unwrap();
        let revision = m.revision();
        for (target, source, offset) in [(foreign, "=1", 0), (b, "=1", 3), (b, "=é", 2)] {
            assert!(set_invalid_expression(&mut m, target, source, offset).is_err());
            assert_eq!(m.revision(), revision);
        }
        m.remove(a).unwrap();
        set_resolved_expression(&mut m, b, "=A1", &[ResolvedToken::Node(a)]).unwrap();
        assert!(
            matches!(m.snapshot(),Err(SnapshotError::MissingReference { target,.. }) if target==a)
        );
        let revision = m.revision();
        assert!(set_invalid_expression(&mut m, a, "=1", 0).is_err());
        assert_eq!(m.revision(), revision);
        set_invalid_expression(&mut m, b, "=)", 1).unwrap();
        assert!(matches!(m.snapshot(), Err(SnapshotError::Parse { .. })));
    }
    #[test]
    fn trusted_projection_setter_validates_before_mutating_without_parsing() {
        let mut model = WorkingModel::new("Proof").unwrap();
        let a = model.add_input("A", PivotDecimal::ONE).unwrap();
        let b = model.add_input("B", PivotDecimal::ZERO).unwrap();
        let mut other = WorkingModel::new("Other").unwrap();
        let foreign = other.add_input("A", PivotDecimal::ONE).unwrap();
        let initial = model.revision();
        for tokens in [
            vec![],
            vec![ResolvedToken::Binary(BinaryOp::Add)],
            vec![ResolvedToken::Node(foreign)],
        ] {
            assert!(set_resolved_expression(&mut model, b, "opaque typed proof", &tokens).is_err());
            assert_eq!(model.revision(), initial);
        }
        set_resolved_expression(
            &mut model,
            b,
            "not parseable spreadsheet source",
            &[
                ResolvedToken::Node(a),
                ResolvedToken::Decimal(PivotDecimal::ONE),
                ResolvedToken::Binary(BinaryOp::Add),
            ],
        )
        .unwrap();
        let snapshot = model.snapshot().unwrap();
        let result = crate::evaluation::EvaluationContext::new(snapshot)
            .evaluate(&[b])
            .unwrap();
        assert_eq!(
            *result.value(b).unwrap().as_ref().unwrap(),
            "2".parse().unwrap()
        );
        model.remove(a).unwrap();
        assert!(matches!(
            model.snapshot(),
            Err(SnapshotError::MissingReference { .. })
        ));
        let revision = model.revision();
        assert!(
            set_resolved_expression(
                &mut model,
                a,
                "retired target",
                &[ResolvedToken::Decimal(PivotDecimal::ZERO)]
            )
            .is_err()
        );
        assert_eq!(model.revision(), revision);
    }
    #[test]
    fn importer_validates_slots_names_tokens_and_failures_without_parsing() {
        let base = Image {
            name: "Restored".into(),
            node_slots: 1,
            port_slots: 1,
            nodes: vec![Node {
                slot: 0,
                name: "N".into(),
                definition: NodeDefinition::Computed {
                    source: "old name is not reparsed".into(),
                    binding: Binding::Bound(vec![Token::Decimal(PivotDecimal::ONE)]),
                },
            }],
            ports: vec![],
        };
        assert!(import(base.clone()).unwrap().model.snapshot().is_ok());
        for case in 0..7 {
            let mut image = base.clone();
            match case {
                0 => image.nodes[0].slot = 1,
                1 => image.nodes.push(image.nodes[0].clone()),
                2 => {
                    image.nodes[0].definition = NodeDefinition::Computed {
                        source: "x".into(),
                        binding: Binding::Bound(vec![Token::Node(1)]),
                    }
                }
                3 => {
                    image.nodes[0].definition = NodeDefinition::Computed {
                        source: "x".into(),
                        binding: Binding::Bound(vec![Token::Binary(BinaryOp::Add)]),
                    }
                }
                4 => {
                    image.nodes[0].definition = NodeDefinition::Computed {
                        source: "".into(),
                        binding: Binding::ParseFailed { offset: 1, code: 5 },
                    }
                }
                5 => image.ports.push(Port {
                    slot: 0,
                    name: "Bad".into(),
                    node: 99,
                    requirement: None,
                }),
                _ => image.node_slots = 8193,
            }
            assert!(import(image).is_err());
        }
    }
}
