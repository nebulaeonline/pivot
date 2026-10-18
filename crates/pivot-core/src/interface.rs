//! Public ports are explicit labels, separate from the graph's name directory.
use crate::{
    evaluation::{CalculationResult, EvaluationContext, InvocationError, InvocationRequest},
    model::{ModelSnapshot, NodeId, Revision, SnapshotDefinition},
};
use pivot_types::{PivotValue, ValueType};
use std::{
    collections::{BTreeMap, BTreeSet},
    fmt,
};

/// Stable for a port's lifetime, independent of its public label. Process-local.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct PortId {
    pub(crate) model: u64,
    pub(crate) sequence: u64,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum InputRequirement {
    Required,
    Optional { default: PivotValue },
}
#[derive(Debug, Clone)]
pub struct InputPort {
    pub(crate) id: PortId,
    pub(crate) name: String,
    pub(crate) node: NodeId,
    pub(crate) requirement: InputRequirement,
}
impl InputPort {
    pub fn id(&self) -> PortId {
        self.id
    }
    pub fn name(&self) -> &str {
        &self.name
    }
    pub fn value_type(&self) -> ValueType {
        ValueType::Decimal
    }
    pub fn requirement(&self) -> &InputRequirement {
        &self.requirement
    }
}
#[derive(Debug, Clone)]
pub struct OutputPort {
    pub(crate) id: PortId,
    pub(crate) name: String,
    pub(crate) node: NodeId,
}
impl OutputPort {
    pub fn id(&self) -> PortId {
        self.id
    }
    pub fn name(&self) -> &str {
        &self.name
    }
    pub fn value_type(&self) -> ValueType {
        ValueType::Decimal
    }
}
#[derive(Debug, Clone, Default)]
pub struct ModelInterface {
    pub(crate) inputs: BTreeMap<String, InputPort>,
    pub(crate) outputs: BTreeMap<String, OutputPort>,
}
impl ModelInterface {
    pub fn inputs(&self) -> impl Iterator<Item = &InputPort> {
        self.inputs.values()
    }
    pub fn outputs(&self) -> impl Iterator<Item = &OutputPort> {
        self.outputs.values()
    }
    pub(crate) fn validate(&self, snapshot: &ModelSnapshot) -> Result<(), InterfaceError> {
        for port in self.inputs.values() {
            let node = snapshot
                .node(port.node)
                .ok_or(InterfaceError::InvalidBinding(port.id))?;
            if !matches!(node.definition(), SnapshotDefinition::Input(_)) {
                return Err(InterfaceError::InvalidBinding(port.id));
            }
        }
        for port in self.outputs.values() {
            if snapshot.node(port.node).is_none() {
                return Err(InterfaceError::InvalidBinding(port.id));
            }
        }
        Ok(())
    }
}
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum InterfaceError {
    InvalidName(String),
    DuplicateName(String),
    DuplicateInputTarget(NodeId),
    InvalidBinding(PortId),
    UnknownPort(PortId),
    UnknownInput(String),
    DuplicateInput(String),
    MissingRequired(String),
    UnknownOutput(String),
    DuplicateOutput(String),
    TypeMismatch {
        name: String,
        expected: ValueType,
        actual: ValueType,
    },
    Invocation(InvocationError),
}
impl fmt::Display for InterfaceError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{self:?}")
    }
}
impl std::error::Error for InterfaceError {}

#[derive(Debug, Clone, Default)]
pub struct NamedInvocationRequest {
    pub inputs: Vec<(String, PivotValue)>,
    pub outputs: Vec<String>,
}

/// Named results expose only requested public outputs, not the internal graph.
#[derive(Debug, Clone)]
pub struct NamedCalculationResult {
    revision: Revision,
    outputs: BTreeMap<String, Result<PivotValue, PublicEvaluationError>>,
    bindings: BTreeMap<String, PivotValue>,
}
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PublicEvaluationError {
    Arithmetic(pivot_types::DecimalError),
    DependencyFailed,
    InvalidExpression,
}
impl NamedCalculationResult {
    pub fn revision(&self) -> Revision {
        self.revision
    }
    pub fn outputs(&self) -> &BTreeMap<String, Result<PivotValue, PublicEvaluationError>> {
        &self.outputs
    }
    pub fn inputs(&self) -> &BTreeMap<String, PivotValue> {
        &self.bindings
    }
}

impl EvaluationContext {
    pub fn invoke_named(
        &self,
        request: &NamedInvocationRequest,
    ) -> Result<NamedCalculationResult, InterfaceError> {
        let interface = self.snapshot().interface();
        let mut supplied = BTreeMap::new();
        for (name, value) in &request.inputs {
            let port = interface
                .inputs
                .get(name)
                .ok_or_else(|| InterfaceError::UnknownInput(name.clone()))?;
            if value.value_type() != port.value_type() {
                return Err(InterfaceError::TypeMismatch {
                    name: name.clone(),
                    expected: port.value_type(),
                    actual: value.value_type(),
                });
            }
            if supplied.insert(name.clone(), value.clone()).is_some() {
                return Err(InterfaceError::DuplicateInput(name.clone()));
            }
        }
        let mut bindings = BTreeMap::new();
        let mut inputs = Vec::new();
        for port in interface.inputs.values() {
            let value = match supplied.remove(&port.name) {
                Some(value) => value,
                None => match &port.requirement {
                    InputRequirement::Required => {
                        return Err(InterfaceError::MissingRequired(port.name.clone()));
                    }
                    InputRequirement::Optional { default } => default.clone(),
                },
            };
            let decimal = match &value {
                PivotValue::Decimal(value) => *value,
                _ => {
                    return Err(InterfaceError::TypeMismatch {
                        name: port.name.clone(),
                        expected: ValueType::Decimal,
                        actual: value.value_type(),
                    });
                }
            };
            inputs.push((port.node, decimal));
            bindings.insert(port.name.clone(), value);
        }
        let mut seen = BTreeSet::new();
        let mut outputs = Vec::new();
        for name in &request.outputs {
            let port = interface
                .outputs
                .get(name)
                .ok_or_else(|| InterfaceError::UnknownOutput(name.clone()))?;
            if !seen.insert(name.clone()) {
                return Err(InterfaceError::DuplicateOutput(name.clone()));
            }
            outputs.push(port.node);
        }
        let result = self
            .invoke(&InvocationRequest { inputs, outputs })
            .map_err(InterfaceError::Invocation)?;
        Ok(named_result(result, request, interface, bindings))
    }
}
fn named_result(
    result: CalculationResult,
    request: &NamedInvocationRequest,
    interface: &ModelInterface,
    bindings: BTreeMap<String, PivotValue>,
) -> NamedCalculationResult {
    use crate::evaluation::EvaluationError;
    let outputs = request
        .outputs
        .iter()
        .map(|name| {
            let value = match result
                .value(interface.outputs[name].node)
                .expect("selected public output was evaluated")
            {
                Ok(value) => Ok(PivotValue::Decimal(*value)),
                Err(EvaluationError::Arithmetic { error, .. }) => {
                    Err(PublicEvaluationError::Arithmetic(*error))
                }
                Err(EvaluationError::DependencyFailed { .. }) => {
                    Err(PublicEvaluationError::DependencyFailed)
                }
                Err(_) => Err(PublicEvaluationError::InvalidExpression),
            };
            (name.clone(), value)
        })
        .collect();
    NamedCalculationResult {
        revision: result.revision(),
        outputs,
        bindings,
    }
}
