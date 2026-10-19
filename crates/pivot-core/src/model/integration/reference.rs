//! Compatibility adapter for the new local-capture contract on reference models.
//! Select stored bindings, then reuse accepted validation and arithmetic execution.
use super::*;
pub(super) fn evaluate(
    source: &WorkingModel,
    request: &crate::evaluation::InvocationRequest,
    cancel: &CancellationToken,
) -> Result<CalculationResult, CalculationFailure> {
    crate::evaluation::prepare_inputs(request, |id| {
        source
            .nodes
            .get(&id)
            .map(|node| matches!(node.definition, Definition::Input(_)))
    })
    .map_err(|e| match e {
        crate::evaluation::InvocationError::Evaluation(e) => CalculationFailure::Evaluation(e),
        other => CalculationFailure::Invocation(other),
    })?;
    let mut required = BTreeSet::new();
    let mut pending: Vec<_> = request
        .outputs
        .iter()
        .copied()
        .collect::<BTreeSet<_>>()
        .into_iter()
        .collect();
    while let Some(id) = pending.pop() {
        if cancel.is_cancelled() {
            return Err(CalculationFailure::Cancelled);
        }
        let node = source.nodes.get(&id).ok_or(CalculationFailure::Evaluation(
            EvaluationError::UnknownNode(id),
        ))?;
        if !required.insert(id) {
            continue;
        }
        if let Definition::Computed { binding, .. } = &node.definition {
            let (_, deps) = binding
                .as_ref()
                .map_err(|e| CalculationFailure::LocalInvalid(e.clone()))?;
            for &target in deps {
                if !source.nodes.contains_key(&target) {
                    return Err(CalculationFailure::LocalInvalid(
                        SnapshotError::MissingReference { node: id, target },
                    ));
                }
            }
            pending.extend(deps);
        }
    }
    required.extend(request.inputs.iter().map(|(id, _)| *id));
    let nodes: BTreeMap<_, _> = required
        .into_iter()
        .map(|id| (id, source.nodes[&id].clone()))
        .collect();
    let names = nodes
        .iter()
        .map(|(id, node)| (node.name.clone(), *id))
        .collect();
    let subset = WorkingModel {
        integrated: None,
        name: source.name.clone(),
        revision: source.revision,
        nodes,
        names,
        next_node: source.next_node,
        next_port: source.next_port,
        interface: ModelInterface::default(),
    };
    let snapshot = subset
        .snapshot_cancel_check(|| cancel.is_cancelled())
        .map_err(|e| match e {
            SnapshotError::Backend("cancelled") => CalculationFailure::Cancelled,
            other => CalculationFailure::LocalInvalid(other),
        })?;
    let value = EvaluationContext::new(snapshot)
        .invoke(request)
        .map_err(|e| match e {
            crate::evaluation::InvocationError::Evaluation(e) => CalculationFailure::Evaluation(e),
            other => CalculationFailure::Invocation(other),
        })?;
    if cancel.is_cancelled() {
        return Err(CalculationFailure::Cancelled);
    }
    Ok(value)
}
