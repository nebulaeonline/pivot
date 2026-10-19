//! Private adapters reuse accepted transfer validation and port authoring rules.
use super::*;
use std::ops::Deref;

/// Trusted bulk fixture construction, not an import-throughput claim.
pub fn benchmark_model(size: usize, degree: usize) -> (WorkingModel, Vec<NodeId>, PortId) {
    assert!((1000..=1_000_000).contains(&size) && degree <= size - 2);
    let mut m = WorkingModel::new("Compatibility benchmark").unwrap();
    let nodes: Vec<_> = (0..size)
        .map(|i| m.add_input(format!("n{i}"), PivotDecimal::ONE).unwrap())
        .collect();
    for &id in nodes.iter().skip(2).take(degree) {
        m.set_expression(id, "n0+1").unwrap();
    }
    m.add_input_port("required", nodes[0], InputRequirement::Required)
        .unwrap();
    m.add_input_port(
        "optional",
        nodes[1],
        InputRequirement::Optional {
            default: pivot_types::PivotValue::Decimal(7.into()),
        },
    )
    .unwrap();
    let output = m.add_output_port("out", nodes[2]).unwrap();
    (m, nodes, output)
}
pub struct Materialized {
    model: WorkingModel,
    _charge: Charge,
}
impl Deref for Materialized {
    type Target = WorkingModel;
    fn deref(&self) -> &WorkingModel {
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
        self.names.insert(name.clone(), id)?;
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
    fn working(&self, nodes: BTreeMap<NodeId, WorkingNode>) -> Result<WorkingModel, Error> {
        Ok(WorkingModel {
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
    pub fn to_model(&self) -> Result<Materialized, Error> {
        let name_bytes = self.names.iter().map(|(name, _)| name.len()).sum::<usize>();
        let charge = self.memory.reserve(
            self.live
                .saturating_mul(2048)
                .saturating_add(self.bytes.saturating_mul(4))
                .saturating_add(name_bytes.saturating_mul(4))
                .saturating_add(self.ports.heap_bytes().saturating_mul(4)),
        )?;
        let mut nodes = BTreeMap::new();
        for (slot, record) in self.records.iter().filter(|(_, r)| r.live) {
            let id = NodeId {
                model: self.version.model,
                node: slot,
            };
            let definition = match &*record.definition {
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
        Ok(Materialized {
            model,
            _charge: charge,
        })
    }
    pub fn export(&self, nodes: &[NodeId], ports: &[PortId]) -> Result<transfer::Image, Error> {
        let model = self.to_model()?;
        Ok(transfer::export(&model, nodes, ports)?)
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
                        transfer::NodeDefinition::Input(_) => 0,
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
        let imported = transfer::import(image)?;
        let graph = Self::from_model_in(&imported.model, limits, memory.clone())?;
        Ok(Restored {
            graph,
            nodes: imported.nodes,
            ports: imported.ports,
        })
    }
    /// Expensive diagnostic path reproduces accepted complete affected-node lists.
    /// Calculation capture stays O(roots) for valid generations.
    pub fn diagnose(&self) -> Result<Result<(), SnapshotError>, Error> {
        Ok(self.to_model()?.snapshot().map(|_| ()))
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
    pub(super) fn validate_ports(&self) -> Result<(), SnapshotError> {
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
        let calculation = self.invoke(&prepared.request).map_err(|e| match e {
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
