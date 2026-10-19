//! Deterministic, single-threaded evaluation with no shared mutable cache.
use crate::{
    expression::{BinaryOp, UnaryOp},
    model::{BoundToken, ModelSnapshot, NodeId, Revision, SnapshotDefinition},
};
use pivot_types::{DecimalError, PivotDecimal};
use std::{
    collections::{BTreeMap, BTreeSet},
    fmt,
};

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum EvaluationError {
    UnknownNode(NodeId),
    Arithmetic {
        node: NodeId,
        error: DecimalError,
    },
    DependencyFailed {
        node: NodeId,
        dependency: NodeId,
        origin: NodeId,
    },
    /// A broken bound-expression invariant, not an arithmetic failure.
    InvalidExpression {
        node: NodeId,
    },
}
impl EvaluationError {
    pub fn origin(&self) -> NodeId {
        match *self {
            Self::UnknownNode(node)
            | Self::Arithmetic { node, .. }
            | Self::InvalidExpression { node } => node,
            Self::DependencyFailed { origin, .. } => origin,
        }
    }
}
impl fmt::Display for EvaluationError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{self:?}")
    }
}
impl std::error::Error for EvaluationError {}

type NodeResult = Result<PivotDecimal, EvaluationError>;

/// Temporary bindings are a sequence so duplicate IDs can be rejected.
/// Values are decimals for now; this is not a wire schema.
#[derive(Debug, Clone, Default)]
pub struct InvocationRequest {
    pub inputs: Vec<(NodeId, PivotDecimal)>,
    pub outputs: Vec<NodeId>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum InvocationError {
    UnknownInput(NodeId),
    NotInput(NodeId),
    DuplicateInput(NodeId),
    Evaluation(EvaluationError),
}
impl fmt::Display for InvocationError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{self:?}")
    }
}
impl std::error::Error for InvocationError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::Evaluation(error) => Some(error),
            _ => None,
        }
    }
}

/// Owns one snapshot. Every evaluate call creates fresh local execution state.
/// Invocations reuse bound snapshot expressions without editing captured inputs.
pub struct EvaluationContext {
    snapshot: ModelSnapshot,
}
impl EvaluationContext {
    pub fn new(snapshot: ModelSnapshot) -> Self {
        Self { snapshot }
    }
    pub fn snapshot(&self) -> &ModelSnapshot {
        &self.snapshot
    }

    /// Evaluate selected outputs and their transitive dependencies only.
    /// Unknown outputs reject the request. Node failures remain result data and
    /// do not prevent independent outputs from being evaluated.
    pub fn evaluate(&self, outputs: &[NodeId]) -> Result<CalculationResult, EvaluationError> {
        self.evaluate_with_inputs(outputs, BTreeMap::new())
    }

    /// Validate every binding, even if it is outside the requested dependency
    /// closure. Unspecified inputs use captured defaults. No state is retained
    /// in the context between invocations.
    pub fn invoke(
        &self,
        request: &InvocationRequest,
    ) -> Result<CalculationResult, InvocationError> {
        let mut inputs = BTreeMap::new();
        for &(id, value) in &request.inputs {
            let node = self
                .snapshot
                .node(id)
                .ok_or(InvocationError::UnknownInput(id))?;
            if !matches!(node.definition(), SnapshotDefinition::Input(_)) {
                return Err(InvocationError::NotInput(id));
            }
            if inputs.insert(id, value).is_some() {
                return Err(InvocationError::DuplicateInput(id));
            }
        }
        self.evaluate_with_inputs(&request.outputs, inputs)
            .map_err(InvocationError::Evaluation)
    }

    fn evaluate_with_inputs(
        &self,
        outputs: &[NodeId],
        inputs: BTreeMap<NodeId, PivotDecimal>,
    ) -> Result<CalculationResult, EvaluationError> {
        #[cfg(feature = "scaling-experiment")]
        let _phase = crate::experiment_profile::scope(8);
        let outputs: BTreeSet<_> = outputs.iter().copied().collect();
        let mut required = BTreeSet::new();
        let mut pending: Vec<_> = outputs.iter().copied().collect();
        while let Some(id) = pending.pop() {
            let node = self
                .snapshot
                .node(id)
                .ok_or(EvaluationError::UnknownNode(id))?;
            if required.insert(id) {
                pending.extend(node.dependencies());
            }
        }
        let mut values = BTreeMap::new();
        for &id in self.snapshot.dependency_order() {
            if !required.contains(&id) {
                continue;
            }
            let node = self
                .snapshot
                .node(id)
                .ok_or(EvaluationError::UnknownNode(id))?;
            let value = match node.definition() {
                SnapshotDefinition::Input(value) => Ok(inputs.get(&id).copied().unwrap_or(*value)),
                SnapshotDefinition::Computed { tokens, .. } => {
                    evaluate_expression(id, tokens, &values)
                }
            };
            values.insert(id, value);
        }
        Ok(CalculationResult {
            snapshot: self.snapshot.clone(),
            outputs: outputs.into_iter().collect(),
            values,
            input_overrides: inputs,
        })
    }

    pub fn evaluate_all(&self) -> CalculationResult {
        // All identifiers come from the validated snapshot itself.
        self.evaluate(self.snapshot.dependency_order())
            .expect("snapshot node identities are valid")
    }
}

/// Immutable results retain the exact snapshot they describe, including names
/// and bound definitions. No lifetime dependency on the editable working model.
#[derive(Debug, Clone)]
pub struct CalculationResult {
    snapshot: ModelSnapshot,
    outputs: Vec<NodeId>,
    values: BTreeMap<NodeId, NodeResult>,
    input_overrides: BTreeMap<NodeId, PivotDecimal>,
}
impl CalculationResult {
    pub fn revision(&self) -> Revision {
        self.snapshot.revision()
    }
    pub fn snapshot(&self) -> &ModelSnapshot {
        &self.snapshot
    }
    pub fn outputs(&self) -> &[NodeId] {
        &self.outputs
    }
    /// Includes evaluated dependencies. None means the node was not evaluated.
    pub fn value(&self, id: NodeId) -> Option<&Result<PivotDecimal, EvaluationError>> {
        self.values.get(&id)
    }
    pub fn evaluated_count(&self) -> usize {
        self.values.len()
    }
    /// Accepted invocation overrides, including those outside selected outputs.
    /// Captured defaults remain accessible through snapshot().
    pub fn input_overrides(&self) -> &BTreeMap<NodeId, PivotDecimal> {
        &self.input_overrides
    }
}

pub(crate) fn evaluate_expression(
    id: NodeId,
    tokens: &[BoundToken],
    values: &BTreeMap<NodeId, NodeResult>,
) -> NodeResult {
    let invalid = || EvaluationError::InvalidExpression { node: id };
    let arithmetic = |error| EvaluationError::Arithmetic { node: id, error };
    let mut stack = Vec::new();
    for token in tokens {
        match *token {
            BoundToken::Decimal(value) => stack.push(value),
            BoundToken::Node(dependency) => {
                let value = values.get(&dependency).ok_or_else(invalid)?;
                stack.push(
                    *value
                        .as_ref()
                        .map_err(|error| EvaluationError::DependencyFailed {
                            node: id,
                            dependency,
                            origin: error.origin(),
                        })?,
                );
            }
            BoundToken::Unary(op) => {
                let value = stack.pop().ok_or_else(invalid)?;
                stack.push(match op {
                    UnaryOp::Plus => value,
                    UnaryOp::Minus => value.checked_neg().map_err(arithmetic)?,
                });
            }
            BoundToken::Binary(op) => {
                let right = stack.pop().ok_or_else(invalid)?;
                let left = stack.pop().ok_or_else(invalid)?;
                let value = match op {
                    BinaryOp::Add => left.checked_add(right),
                    BinaryOp::Subtract => left.checked_sub(right),
                    BinaryOp::Multiply => left.checked_mul(right),
                    BinaryOp::Divide => left.checked_div(right),
                }
                .map_err(arithmetic)?;
                stack.push(value);
            }
        }
    }
    if stack.len() != 1 {
        return Err(invalid());
    }
    stack.pop().ok_or_else(invalid)
}

/// Experiment-only scheduling adapter. Reuses the production postfix executor;
/// traverses only the selected closure and reads immutable input overlays.
#[cfg(feature = "scaling-experiment")]
pub fn evaluate_shared_inputs(
    snapshot: &ModelSnapshot,
    outputs: &[NodeId],
    input: impl Fn(NodeId) -> Option<PivotDecimal>,
) -> Result<BTreeMap<NodeId, Result<PivotDecimal, EvaluationError>>, EvaluationError> {
    let _phase = crate::experiment_profile::scope(8);
    let mut required = BTreeSet::new();
    let mut order = Vec::new();
    let mut pending: Vec<_> = outputs.iter().map(|&n| (n, false)).collect();
    while let Some((id, visited)) = pending.pop() {
        let node = snapshot.node(id).ok_or(EvaluationError::UnknownNode(id))?;
        if visited {
            order.push(id);
        } else if required.insert(id) {
            pending.push((id, true));
            pending.extend(node.dependencies().iter().map(|&d| (d, false)));
        }
    }
    let mut values = BTreeMap::new();
    for id in order {
        let node = snapshot.node(id).expect("validated snapshot closure");
        let value = match node.definition() {
            SnapshotDefinition::Input(value) => Ok(input(id).unwrap_or(*value)),
            SnapshotDefinition::Computed { tokens, .. } => evaluate_expression(id, tokens, &values),
        };
        values.insert(id, value);
    }
    Ok(values)
}
