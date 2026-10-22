//! Private adapters reuse accepted transfer validation and port authoring rules.
use super::*;
use std::ops::Deref;

/// Admitted complete definition image for diagnostics and codecs.
pub struct MaterializedImage {
    model: GraphImage,
    _charge: Charge,
}
impl Deref for MaterializedImage {
    type Target = GraphImage;
    fn deref(&self) -> &GraphImage {
        &self.model
    }
}
pub struct Restored {
    pub graph: Graph,
    pub nodes: Vec<NodeId>,
    pub ports: Vec<PortId>,
}
impl Graph {
    pub fn interface(&self) -> &ModelInterface {
        &self.ports
    }
    pub fn name(&self) -> &str {
        &self.model_name
    }
    pub(super) fn check_name(&self, name: &str) -> Result<(), Error> {
        let mut chars = name.bytes();
        if !chars
            .next()
            .is_some_and(|c| c.is_ascii_alphabetic() || c == b'_')
            || !chars.all(|c| c.is_ascii_alphanumeric() || c == b'_')
        {
            return Err(ModelError::InvalidName(name.into()).into());
        }
        if self.names.contains_key(name) {
            return Err(ModelError::DuplicateName(name.into()).into());
        }
        Ok(())
    }
    pub(super) fn add_input(&mut self, name: String, value: PivotDecimal) -> Result<NodeId, Error> {
        self.check_name(&name)?;
        let id = NodeId {
            model: self.version.model,
            node: self.next,
        };
        self.next = self.next.checked_add(1).ok_or(Error::Exhausted)?;
        self.max_name = self.max_name.max(name.len());
        {
            let _profile = crate::experiment_profile::construction::scope(14);
            self.names.insert(name.clone(), id)?;
        }
        self.records.set(
            id.node,
            Record {
                name,
                live: true,
                definition: self.memory.alloc(Compiled::Input)?,
                reverse: Index::new(self.memory.clone()),
            },
        )?;
        self.values.set(id.node, value)?;
        self.live += 1;
        Ok(id)
    }
    fn working(&self, nodes: BTreeMap<NodeId, WorkingNode>) -> Result<GraphImage, Error> {
        Ok(GraphImage {
            name: self.model_name.to_string(),
            revision: Revision {
                model: self.version.model,
                version: self
                    .revision_origin
                    .checked_add(self.version.commit)
                    .ok_or(Error::Exhausted)?,
            },
            nodes,
            names: BTreeMap::new(),
            next_node: self.next,
            next_port: self.next_port,
            interface: (*self.ports).clone(),
        })
    }
    pub(crate) fn copy_reservation(&self) -> Result<Charge, Error> {
        let names = self.names.iter().map(|(name, _)| name.len()).sum::<usize>();
        Ok(self.memory.reserve(
            self.live
                .saturating_mul(2048)
                .saturating_add(self.bytes.saturating_mul(4))
                .saturating_add(names.saturating_mul(4))
                .saturating_add(self.ports.heap_bytes().saturating_mul(4))
                .saturating_add(65536),
        )?)
    }
    pub fn materialize(&self) -> Result<MaterializedImage, Error> {
        self.materialize_cancelled(|| false)
    }
    pub(in crate::model) fn materialize_cancelled(
        &self,
        cancelled: impl Fn() -> bool,
    ) -> Result<MaterializedImage, Error> {
        self.materialize_mode(cancelled, false)
    }
    fn materialize_mode(
        &self,
        cancelled: impl Fn() -> bool,
        allow_selections: bool,
    ) -> Result<MaterializedImage, Error> {
        if self.captured_scalar.is_some() {
            return Err(ModelError::Backend("captured scalar sources require a captured context; scalar materialization unsupported").into());
        }
        let name_bytes = self.names.iter().map(|(name, _)| name.len()).sum::<usize>();
        let charge = self.memory.reserve(
            self.live
                .saturating_mul(2048)
                .saturating_add(self.bytes.saturating_mul(4))
                .saturating_add(name_bytes.saturating_mul(4))
                .saturating_add(self.ports.heap_bytes().saturating_mul(4)),
        )?;
        if self.has_selection_formulas() && !allow_selections {
            return Err(ModelError::Backend(
                "unsupported selection capability in scalar materialization",
            )
            .into());
        }
        let mut nodes = BTreeMap::new();
        for (slot, record) in self.records.iter().filter(|(_, r)| r.live) {
            if cancelled() {
                return Err(Error::Cancelled);
            }
            let id = NodeId {
                model: self.version.model,
                node: slot,
            };
            let definition = match &*record.definition {
                Compiled::NonNumeric => Definition::NonNumeric,
                Compiled::CapturedScalar(id) => Definition::CapturedScalar(*id),
                Compiled::Input => Definition::Input(**self.values.get(slot).unwrap()),
                Compiled::Formula { source, binding } => Definition::Computed {
                    source: source.clone(),
                    binding: binding.clone(),
                },
            };
            nodes.insert(
                id,
                WorkingNode {
                    name: record.name.clone(),
                    definition,
                },
            );
        }
        let mut model = self.working(nodes)?;
        model.names = self.names.iter().map(|(n, id)| (n.clone(), *id)).collect();
        Ok(MaterializedImage {
            model,
            _charge: charge,
        })
    }
    pub(in crate::model) fn export_selections(
        &self,
        nodes: &[NodeId],
        ports: &[PortId],
    ) -> Result<transfer::SelectionImage, Error> {
        let mut image = self.materialize_mode(|| false, true)?;
        let slots: BTreeMap<_, _> = nodes
            .iter()
            .enumerate()
            .map(|(slot, id)| (*id, slot))
            .collect();
        let mut formulas = Vec::new();
        for (id, node) in image.model.nodes.iter_mut() {
            if let Definition::Computed {
                source,
                binding: Ok((tokens, _)),
            } = &node.definition
                && tokens.iter().any(|t| matches!(t, BoundToken::Sum(_)))
            {
                let translated = tokens
                    .iter()
                    .map(|token| {
                        Ok(match token {
                            BoundToken::Decimal(v) => transfer::SelectionToken::Decimal(*v),
                            BoundToken::Node(n) => transfer::SelectionToken::Node(
                                *slots.get(n).ok_or(Error::InvalidTransfer)?,
                            ),
                            BoundToken::Sum(id) => transfer::SelectionToken::Sum(*id),
                            BoundToken::Unary(op) => transfer::SelectionToken::Unary(*op),
                            BoundToken::Binary(op) => transfer::SelectionToken::Binary(*op),
                        })
                    })
                    .collect::<Result<Vec<_>, Error>>()?;
                formulas.push(transfer::SelectionFormula {
                    slot: *slots.get(id).ok_or(Error::InvalidTransfer)?,
                    source: source.clone(),
                    tokens: translated,
                });
                node.definition = Definition::Computed {
                    source: source.clone(),
                    binding: Err(SnapshotError::Parse {
                        node: *id,
                        error: ParseError {
                            offset: 0,
                            message: "unexpected trailing input",
                        },
                    }),
                };
            }
        }
        formulas.sort_by_key(|f| f.slot);
        let scalar_skeleton = transfer::export_image(&image.model, nodes, ports)?;
        let selections = self.selection_image();
        let points = selections
            .points
            .into_iter()
            .map(|(space, row, column, node)| {
                Ok((
                    space,
                    row,
                    column,
                    *slots.get(&node).ok_or(Error::InvalidTransfer)?,
                ))
            })
            .collect::<Result<Vec<_>, Error>>()?;
        Ok(transfer::SelectionImage {
            version: 2,
            scalar_skeleton,
            formulas,
            identities: selections.identities,
            points,
        })
    }
    pub fn export(&self, nodes: &[NodeId], ports: &[PortId]) -> Result<transfer::Image, Error> {
        if self.records.iter().any(|(_, r)| {
            r.live
                && matches!(
                    *r.definition,
                    Compiled::NonNumeric | Compiled::CapturedScalar(_)
                )
        }) {
            return Err(
                ModelError::Backend("unsupported nonnumeric content in scalar export").into(),
            );
        }
        let model = self.materialize()?;
        Ok(transfer::export_image(&model, nodes, ports)?)
    }
    pub fn restore(
        image: transfer::Image,
        limits: Limits,
        budget: usize,
    ) -> Result<Restored, Error> {
        // The accepted checker is the only transfer authoring/import validator.
        let memory = Memory::new(budget);
        let payload = image
            .nodes
            .iter()
            .map(|n| {
                n.name.len() * 4
                    + match &n.definition {
                        transfer::NodeDefinition::NonNumeric
                        | transfer::NodeDefinition::Input(_) => 0,
                        transfer::NodeDefinition::Computed { source, binding } => {
                            source.len() * 4
                                + match binding {
                                    transfer::Binding::Bound(t) => t.len() * 128,
                                    _ => 1024,
                                }
                        }
                    }
            })
            .sum::<usize>();
        let _scratch = memory.reserve(
            payload
                .saturating_add(image.nodes.len() * 2048)
                .saturating_add(image.ports.len() * 1024),
        )?;
        let imported = transfer::decode(image)?;
        let graph = Self::from_image_in(&imported.model, limits, memory.clone())?;
        Ok(Restored {
            graph,
            nodes: imported.nodes,
            ports: imported.ports,
        })
    }
    /// Expensive diagnostic path reproduces accepted complete affected-node lists.
    /// Calculation capture stays O(roots) for valid generations.
    pub fn diagnose(&self) -> Result<Result<(), SnapshotError>, Error> {
        if self.has_selection_formulas() {
            let roots = self
                .records
                .iter()
                .filter(|(_, r)| r.live)
                .map(|(slot, _)| NodeId {
                    model: self.version.model,
                    node: slot,
                })
                .collect();
            return Ok(self
                .validate_many(&roots, &|| false)?
                .map_or(Ok(()), |(_, error)| Err(error)));
        }
        Ok(self.materialize()?.snapshot().map(|_| ()))
    }
    pub(super) fn edit_port(&mut self, edit: PortEdit) -> Result<bool, Error> {
        let _scratch = self
            .memory
            .reserve(self.ports.heap_bytes().saturating_mul(4) + 4096)?;
        let mut targets: BTreeSet<_> = self
            .ports
            .inputs()
            .map(|p| p.node())
            .chain(self.ports.outputs().map(|p| p.node()))
            .collect();
        match &edit {
            PortEdit::AddInput(_, id, _) | PortEdit::AddOutput(_, id) | PortEdit::Output(_, id) => {
                targets.insert(*id);
            }
            PortEdit::Input(_, Some(id), _) => {
                targets.insert(*id);
            }
            _ => {}
        }
        let nodes = targets
            .into_iter()
            .filter_map(|id| {
                self.record(id).filter(|r| r.live).map(|r| {
                    let definition = match &*r.definition {
                        Compiled::NonNumeric => Definition::NonNumeric,
                        Compiled::CapturedScalar(id) => Definition::CapturedScalar(*id),
                        Compiled::Input => Definition::Input(**self.values.get(id.node).unwrap()),
                        _ => Definition::Computed {
                            source: String::new(),
                            binding: Ok((Vec::new(), Vec::new())),
                        },
                    };
                    (
                        id,
                        WorkingNode {
                            name: r.name.clone(),
                            definition,
                        },
                    )
                })
            })
            .collect();
        let mut proxy = self.working(nodes)?;
        let before = proxy.revision();
        match edit {
            PortEdit::AddInput(name, node, requirement) => {
                proxy.add_input_port(name, node, requirement)?;
            }
            PortEdit::AddOutput(name, node) => {
                proxy.add_output_port(name, node)?;
            }
            PortEdit::Rename(id, name) => proxy.rename_port(id, name)?,
            PortEdit::Remove(id) => proxy.remove_port(id)?,
            PortEdit::Input(id, node, requirement) => {
                proxy.patch_input_port(id, node, requirement)?
            }
            PortEdit::Output(id, node) => proxy.update_output_port(id, node)?,
        }
        if proxy.revision() == before {
            return Ok(false);
        }
        self.next_port = proxy.next_port;
        self.ports = self.memory.alloc(proxy.interface)?;
        Ok(true)
    }
    pub(crate) fn validate_ports(&self) -> Result<(), SnapshotError> {
        self.ports
            .validate_kinds(|id| {
                self.record(id)
                    .filter(|r| r.live)
                    .map(|r| matches!(*r.definition, Compiled::Input))
            })
            .map_err(SnapshotError::Interface)
    }
}

#[derive(Debug, PartialEq, Eq)]
pub enum NamedError {
    Interface(InterfaceError),
    Compute(ComputeError),
}
pub struct NamedCalculation {
    version: Version,
    pub outputs:
        BTreeMap<String, Result<pivot_types::PivotValue, crate::interface::PublicEvaluationError>>,
    bindings: BTreeMap<String, pivot_types::PivotValue>,
    _calculation: Calculation,
    _charge: Charge,
}
impl NamedCalculation {
    pub fn version(&self) -> Version {
        self.version
    }
    pub fn outputs(
        &self,
    ) -> &BTreeMap<String, Result<pivot_types::PivotValue, crate::interface::PublicEvaluationError>>
    {
        &self.outputs
    }
    pub fn bindings(&self) -> &BTreeMap<String, pivot_types::PivotValue> {
        &self.bindings
    }
}
impl Capture {
    pub fn invoke_named(
        &self,
        request: &crate::interface::NamedInvocationRequest,
    ) -> Result<NamedCalculation, NamedError> {
        self.invoke_named_retiring(request, drop, || false)
    }
    pub(in crate::model) fn invoke_named_retiring(
        &self,
        request: &crate::interface::NamedInvocationRequest,
        retire: impl FnOnce(CalculationScratch),
        cancelled: impl Fn() -> bool,
    ) -> Result<NamedCalculation, NamedError> {
        if cancelled() {
            return Err(NamedError::Compute(ComputeError::Cancelled));
        }
        self.0.validate_ports().map_err(|e| match e {
            SnapshotError::Interface(e) => NamedError::Interface(e),
            _ => unreachable!(),
        })?;
        let bytes = request
            .inputs
            .iter()
            .map(|(name, _)| name.len() + 512)
            .sum::<usize>()
            + request
                .outputs
                .iter()
                .map(|name| name.len() + 512)
                .sum::<usize>()
            + self.0.ports.heap_bytes() * 2
            + 4096;
        let charge = self
            .0
            .memory
            .reserve(bytes)
            .map_err(|_| NamedError::Compute(ComputeError::Admission))?;
        let prepared = crate::interface::prepare_named(&self.0.ports, request)
            .map_err(NamedError::Interface)?;
        let calculation = self
            .invoke_retiring_cancelled(&prepared.request, retire, cancelled)
            .map_err(|e| match e {
                ComputeError::Invocation(e) => NamedError::Interface(InterfaceError::Invocation(e)),
                other => NamedError::Compute(other),
            })?;
        let outputs = request
            .outputs
            .iter()
            .map(|name| {
                (
                    name.clone(),
                    crate::interface::public_value(&calculation[&self.0.ports.outputs[name].node]),
                )
            })
            .collect();
        Ok(NamedCalculation {
            version: self.version(),
            outputs,
            bindings: prepared.bindings,
            _calculation: calculation,
            _charge: charge,
        })
    }
}
