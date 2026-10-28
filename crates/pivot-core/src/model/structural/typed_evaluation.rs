//! Demand-driven execution over the canonical captured graph. Static validation
//! always precedes value demand; evaluation uses explicit stacks, not Rust recursion.
use super::*;
use crate::{
    expression::{AggregateControl as C, AggregateKind as K},
    typed::{self, Op},
};
use pivot_types::PivotValue as V;
type Answer = Result<V, EvaluationError>;
enum Expr {
    Members(usize),
    Token(BoundToken, Vec<usize>),
    Aggregate(K, Vec<usize>),
    Range(K, selection::SelectionId),
}
struct Program {
    arena: Vec<Expr>,
    root: usize,
}
fn program(tokens: &[BoundToken]) -> Result<Program, ComputeError> {
    transfer::checked_dependencies(tokens).map_err(|_| ComputeError::WorkBudget)?;
    let mut arena = Vec::new();
    let mut stack = Vec::new();
    let mut calls: Vec<(K, Vec<usize>)> = Vec::new();
    for token in tokens {
        let expr = match token {
            BoundToken::Control(C::Start(k)) => {
                calls.push((*k, Vec::new()));
                continue;
            }
            BoundToken::Control(C::Range) => {
                let root = stack.pop().unwrap();
                let index = arena.len();
                arena.push(Expr::Members(root));
                calls.last_mut().unwrap().1.push(index);
                continue;
            }
            BoundToken::Control(C::Value) => {
                calls.last_mut().unwrap().1.push(stack.pop().unwrap());
                continue;
            }
            BoundToken::Control(C::End) => {
                let (k, args) = calls.pop().unwrap();
                Expr::Aggregate(k, args)
            }
            BoundToken::Sum(id) => Expr::Range(K::Sum, *id),
            BoundToken::Aggregate(k, id) => Expr::Range(*k, *id),
            _ => {
                let n = match token {
                    BoundToken::Typed(_, n) => *n,
                    BoundToken::Unary(_) => 1,
                    BoundToken::Binary(_) => 2,
                    _ => 0,
                };
                let args = stack.split_off(stack.len() - n);
                Expr::Token(token.clone(), args)
            }
        };
        stack.push(arena.len());
        arena.push(expr);
    }
    Ok(Program {
        arena,
        root: stack.pop().unwrap(),
    })
}
enum Task {
    CheckArgument {
        node: NodeId,
        token: BoundToken,
        index: usize,
        tasks: usize,
        stack: usize,
    },
    Node(NodeId),
    FinishNode(NodeId),
    Expr(NodeId, usize),
    Apply(NodeId, BoundToken, usize, Option<Charge>),
    Choose(NodeId, Vec<usize>),
    Logic(NodeId, Vec<usize>, usize, bool),
    Reduce(NodeId, Vec<Operand>, usize, crate::aggregate::Reduction),
    Feed(NodeId, Vec<Operand>, usize, crate::aggregate::Reduction),
}
#[derive(Clone, Copy)]
enum Operand {
    Expr(usize),
    Node(NodeId),
}
pub struct TypedCalculation {
    pub values: BTreeMap<NodeId, Answer>,
    pub evaluated_nodes: usize,
    _capture: Capture,
    _work: Work,
}
impl Capture {
    pub fn evaluate_values(&self, outputs: &[NodeId]) -> Result<TypedCalculation, ComputeError> {
        self.evaluate_values_cancelled(outputs, None, || false)
    }
    pub(in crate::model) fn evaluate_values_cancelled(
        &self,
        outputs: &[NodeId],
        sources: Option<&BTreeMap<crate::captured::CapturedScalarId, crate::captured::ScalarValue>>,
        cancelled: impl Fn() -> bool,
    ) -> Result<TypedCalculation, ComputeError> {
        self.evaluate_values_with_inputs(outputs, sources, &BTreeMap::new(), cancelled)
    }
    pub(super) fn evaluate_values_with_inputs(
        &self,
        outputs: &[NodeId],
        sources: Option<&BTreeMap<crate::captured::CapturedScalarId, crate::captured::ScalarValue>>,
        inputs: &BTreeMap<NodeId, PivotDecimal>,
        cancelled: impl Fn() -> bool,
    ) -> Result<TypedCalculation, ComputeError> {
        self.validate_acyclic(outputs, &cancelled)?;
        if sources.is_none() && !self.captured_slots().is_empty() {
            return Err(crate::evaluation::InvocationError::CapturedContext(
                crate::captured::ContextError::Missing,
            )
            .into());
        }
        let mut work = Work::new(self.memory());
        work.add(4096 + outputs.len() * 256)?;
        let mut programs = BTreeMap::new();
        let mut seen = BTreeSet::new();
        let mut pending = outputs.to_vec();
        let mut visits = self.0.limits.validation_visits;
        while let Some(id) = pending.pop() {
            if cancelled() {
                return Err(ComputeError::Cancelled);
            }
            if !seen.insert(id) {
                continue;
            }
            visits = visits.checked_sub(1).ok_or(ComputeError::WorkBudget)?;
            let record = self.0.live_record(id).map_err(|_| {
                crate::evaluation::InvocationError::Evaluation(EvaluationError::UnknownNode(id))
            })?;
            work.add(512)?;
            if let Compiled::Formula { binding, .. } = &*record.definition {
                let (tokens, _) = binding
                    .as_ref()
                    .map_err(|e| ComputeError::Invalid(e.clone()))?;
                work.add(
                    tokens.len().saturating_mul(256)
                        + tokens
                            .iter()
                            .map(|t| {
                                if let BoundToken::Typed(op, _) = t {
                                    op.heap_bytes()
                                } else {
                                    0
                                }
                            })
                            .sum::<usize>(),
                )?;
                programs.insert(id, program(tokens)?);
            }
            let deps = self
                .0
                .selection_dependencies(id, &mut visits, &cancelled)
                .map_err(|_| {
                    if cancelled() {
                        ComputeError::Cancelled
                    } else {
                        ComputeError::WorkBudget
                    }
                })?;
            work.add(deps.items.len() * 64)?;
            pending.extend(deps.items.iter().copied());
        }
        let mut values = BTreeMap::<NodeId, Answer>::new();
        let mut tasks: Vec<_> = outputs.iter().rev().map(|n| Task::Node(*n)).collect();
        let mut stack = Vec::<Answer>::new();
        let typed = |node, error| EvaluationError::Typed { node, error };
        let mut evaluated_nodes = 0;
        while let Some(task) = tasks.pop() {
            if cancelled() {
                return Err(ComputeError::Cancelled);
            }
            visits = visits.checked_sub(1).ok_or(ComputeError::WorkBudget)?;
            match task {
                Task::Node(id) => {
                    if let Some(v) = values.get(&id) {
                        if let Ok(V::String(s)) = v {
                            work.add(s.len() + 64)?;
                        }
                        stack.push(v.clone());
                        continue;
                    }
                    evaluated_nodes += 1;
                    work.add(256)?;
                    let record = self.0.live_record(id).unwrap();
                    let value = match &*record.definition {
                        Compiled::Blank => Ok(V::Blank),
                        Compiled::NonNumeric => Err(EvaluationError::NonNumericInput { node: id }),
                        Compiled::Input => Ok(V::Decimal(
                            inputs
                                .get(&id)
                                .copied()
                                .unwrap_or(**self.0.values.get(id.node).unwrap()),
                        )),
                        Compiled::CapturedScalar(slot) => match sources.and_then(|s| s.get(slot)) {
                            Some(crate::captured::ScalarValue::Decimal(v)) => Ok(V::Decimal(*v)),
                            Some(crate::captured::ScalarValue::Fault(fault_id)) => {
                                Err(EvaluationError::CapturedSourceFailed {
                                    node: id,
                                    fault_id: *fault_id,
                                })
                            }
                            None => Err(EvaluationError::MissingCapturedContext { node: id }),
                        },
                        Compiled::Formula { .. } => {
                            tasks.push(Task::FinishNode(id));
                            tasks.push(Task::Expr(id, programs[&id].root));
                            continue;
                        }
                    };
                    values.insert(id, value.clone());
                    stack.push(value);
                }
                Task::FinishNode(id) => {
                    let mut value = stack.pop().unwrap();
                    if value == Ok(V::Blank) {
                        value = Ok(V::Decimal(PivotDecimal::ZERO));
                    }
                    if let Ok(V::String(s)) = &value {
                        work.add(s.len() + 64)?;
                    }
                    values.insert(id, value.clone());
                    stack.push(value);
                }
                Task::Expr(id, index) => match &programs[&id].arena[index] {
                    Expr::Token(BoundToken::Node(target), _) => {
                        tasks.push(Task::Apply(id, BoundToken::Node(*target), 1, None));
                        tasks.push(Task::Node(*target));
                    }
                    Expr::Token(BoundToken::Typed(Op::If, _), args) => {
                        tasks.push(Task::Choose(id, args.clone()));
                        tasks.push(Task::Expr(id, args[0]));
                    }
                    Expr::Token(BoundToken::Typed(op @ (Op::And | Op::Or), _), args) => {
                        tasks.push(Task::Logic(id, args.clone(), 0, matches!(op, Op::Or)));
                        tasks.push(Task::Expr(id, args[0]));
                    }
                    Expr::Token(token, args) => {
                        let task_base = tasks.len();
                        let stack_base = stack.len();
                        let opcode_copy = if let BoundToken::Typed(op, _) = token {
                            Some(self.memory().reserve(op.heap_bytes() + 64)?)
                        } else {
                            None
                        };
                        tasks.push(Task::Apply(id, token.clone(), args.len(), opcode_copy));
                        for (index, a) in args.iter().enumerate().rev() {
                            tasks.push(Task::CheckArgument {
                                node: id,
                                token: token.clone(),
                                index,
                                tasks: task_base,
                                stack: stack_base,
                            });
                            tasks.push(Task::Expr(id, *a));
                        }
                    }
                    Expr::Aggregate(kind, args) => {
                        let mut operands = Vec::new();
                        let mut reduction = crate::aggregate::Reduction::new(*kind);
                        for arg in args {
                            if let Expr::Members(root) = programs[&id].arena[*arg] {
                                let Expr::Range(_, selection) = programs[&id].arena[root] else {
                                    return Err(ComputeError::WorkBudget);
                                };
                                reduction.selection = Some(selection);
                                let state =
                                    self.0.selections.as_ref().ok_or(ComputeError::WorkBudget)?;
                                let _member_scratch = self.memory().reserve(
                                    state
                                        .metrics()
                                        .points
                                        .saturating_mul(64)
                                        .saturating_add(4096),
                                )?;
                                let members = state
                                    .members(selection, &mut visits, &cancelled)
                                    .map_err(|_| {
                                        if cancelled() {
                                            ComputeError::Cancelled
                                        } else {
                                            ComputeError::WorkBudget
                                        }
                                    })?;
                                work.add(members.len() * std::mem::size_of::<Operand>() + 64)?;
                                operands.extend(members.into_iter().map(Operand::Node));
                            } else {
                                operands.push(Operand::Expr(*arg));
                            }
                        }
                        tasks.push(Task::Reduce(id, operands, 0, reduction));
                    }
                    Expr::Members(_) => return Err(ComputeError::WorkBudget),
                    Expr::Range(kind, selection) => {
                        let _member_scratch = self.memory().reserve(
                            self.0
                                .selections
                                .as_ref()
                                .ok_or(ComputeError::WorkBudget)?
                                .metrics()
                                .points
                                .saturating_mul(64)
                                .saturating_add(4096),
                        )?;
                        let members = self
                            .0
                            .selections
                            .as_ref()
                            .ok_or(ComputeError::WorkBudget)?
                            .members(*selection, &mut visits, &cancelled)
                            .map_err(|_| {
                                if cancelled() {
                                    ComputeError::Cancelled
                                } else {
                                    ComputeError::WorkBudget
                                }
                            })?;
                        work.add(members.len() * std::mem::size_of::<Operand>() + 64)?;
                        let mut reduction = crate::aggregate::Reduction::new(*kind);
                        reduction.selection = Some(*selection);
                        tasks.push(Task::Reduce(
                            id,
                            members.into_iter().map(Operand::Node).collect(),
                            0,
                            reduction,
                        ));
                    }
                },
                Task::Choose(id, args) => match stack
                    .pop()
                    .unwrap()
                    .and_then(|v| typed::condition(&v).map_err(|e| typed(id, e)))
                {
                    Ok(b) => tasks.push(Task::Expr(id, args[if b { 1 } else { 2 }])),
                    Err(e) => stack.push(Err(e)),
                },
                Task::Logic(id, args, index, is_or) => match stack
                    .pop()
                    .unwrap()
                    .and_then(|v| typed::condition(&v).map_err(|e| typed(id, e)))
                {
                    Err(e) => stack.push(Err(e)),
                    Ok(b) if b == is_or || index + 1 == args.len() => stack.push(Ok(V::Boolean(b))),
                    Ok(_) => {
                        let next = args[index + 1];
                        tasks.push(Task::Logic(id, args, index + 1, is_or));
                        tasks.push(Task::Expr(id, next));
                    }
                },
                Task::Reduce(id, args, index, reduction) => {
                    if index == args.len() {
                        stack.push(reduction.finish(id).map(V::Decimal));
                    } else {
                        let arg = args[index];
                        tasks.push(Task::Feed(id, args, index, reduction));
                        tasks.push(match arg {
                            Operand::Node(n) => Task::Node(n),
                            Operand::Expr(e) => Task::Expr(id, e),
                        });
                    }
                }
                Task::Feed(id, args, index, mut reduction) => {
                    let result = stack.pop().unwrap();
                    let result = match result {
                        Ok(V::Decimal(n)) => reduction
                            .push(n)
                            .map_err(|error| EvaluationError::Arithmetic { node: id, error }),
                        Ok(V::Blank) => Ok(()),
                        Ok(_) if reduction.kind != K::Sum => Ok(()),
                        Ok(_) => Err(typed(id, typed::Error::TypeMismatch)),
                        Err(e) => Err(match args[index] {
                            Operand::Node(n) => e.dependency(id, n),
                            _ => e,
                        }),
                    };
                    match result {
                        Err(e) => stack.push(Err(e)),
                        Ok(()) => tasks.push(Task::Reduce(id, args, index + 1, reduction)),
                    }
                }
                Task::CheckArgument {
                    node,
                    token,
                    index,
                    tasks: task_base,
                    stack: stack_base,
                } => {
                    let answer = stack
                        .last()
                        .unwrap()
                        .as_ref()
                        .map_err(Clone::clone)
                        .and_then(|v| match &token {
                            BoundToken::Binary(_) | BoundToken::Unary(_) => {
                                typed::number(v).map(|_| ()).map_err(|e| typed(node, e))
                            }
                            BoundToken::Typed(op, _) => {
                                typed::validate_argument(op, index, v).map_err(|e| typed(node, e))
                            }
                            _ => Ok(()),
                        });
                    if let Err(error) = answer {
                        tasks.truncate(task_base);
                        stack.truncate(stack_base);
                        stack.push(Err(error));
                    }
                }
                Task::Apply(id, token, n, _opcode_copy) => {
                    let mut args = stack.split_off(stack.len() - n);
                    if let BoundToken::Node(target) = token {
                        for value in &mut args {
                            if let Err(e) = value {
                                *e = e.dependency(id, target);
                            }
                        }
                    }
                    // Admit result construction before allocating, independently of
                    // evaluation faults. Admission failures remain operation failures.
                    let bytes = match &token {
                        BoundToken::Typed(Op::String(s), _) => s.len(),
                        BoundToken::Typed(Op::Concat, _) => args
                            .iter()
                            .map(|v| if let Ok(V::String(s)) = v { s.len() } else { 0 })
                            .sum(),
                        BoundToken::Typed(Op::Mid, _) => {
                            if let Some(Ok(V::String(s))) = args.first() {
                                s.len()
                            } else {
                                0
                            }
                        }
                        _ => 0,
                    };
                    let _result_copy = if bytes > 0 {
                        Some(
                            self.memory()
                                .reserve(bytes.min(typed::MAX_TEXT_BYTES).saturating_add(64))?,
                        )
                    } else {
                        None
                    };
                    let answer = (|| {
                        let args = args.into_iter().collect::<Result<Vec<_>, _>>()?;
                        Ok(match token {
                            BoundToken::Node(target) => args
                                .into_iter()
                                .next()
                                .ok_or(EvaluationError::InvalidExpression { node: target })?,
                            BoundToken::Decimal(d) => V::Decimal(d),
                            BoundToken::Typed(op, _) => {
                                typed::apply(&op, &args).map_err(|e| typed(id, e))?
                            }
                            BoundToken::Unary(op) => {
                                let d = typed::number(&args[0]).map_err(|e| typed(id, e))?;
                                V::Decimal(match op {
                                    UnaryOp::Plus => d,
                                    UnaryOp::Minus => d.checked_neg().map_err(|error| {
                                        EvaluationError::Arithmetic { node: id, error }
                                    })?,
                                })
                            }
                            BoundToken::Binary(op) => {
                                let a = typed::number(&args[0]).map_err(|e| typed(id, e))?;
                                let b = typed::number(&args[1]).map_err(|e| typed(id, e))?;
                                V::Decimal(
                                    match op {
                                        BinaryOp::Add => a.checked_add(b),
                                        BinaryOp::Subtract => a.checked_sub(b),
                                        BinaryOp::Multiply => a.checked_mul(b),
                                        BinaryOp::Divide => a.checked_div(b),
                                    }
                                    .map_err(|error| {
                                        EvaluationError::Arithmetic { node: id, error }
                                    })?,
                                )
                            }
                            _ => return Err(EvaluationError::InvalidExpression { node: id }),
                        })
                    })();
                    if let Ok(V::String(s)) = &answer {
                        work.add(s.len() + 64)?;
                    }
                    stack.push(answer);
                }
            }
        }
        Ok(TypedCalculation {
            values,
            evaluated_nodes,
            _capture: self.clone(),
            _work: work,
        })
    }
}
