use pivot_host::{ModelHost, Session, management::*};
use pivot_protocol::v1 as wire;
use pivot_types::{DecimalError, PivotDecimal};
use prost::Message;
use std::sync::{
    Arc,
    atomic::{AtomicBool, AtomicUsize, Ordering},
};

#[derive(Default)]
struct Controls {
    deny_publish: AtomicBool,
    deny_evaluate: AtomicBool,
    deny_validate: AtomicBool,
    publish_calls: AtomicUsize,
    revoke_publish_at: AtomicUsize,
    evaluate_calls: AtomicUsize,
    revoke_evaluate_at: AtomicUsize,
}
struct Policy(Arc<Controls>);
impl Authorizer for Policy {
    fn allowed(&self, who: &str, action: Action, _: Option<WorkingModelId>) -> bool {
        if who != "alice" {
            return false;
        }
        match action {
            Action::Publish => {
                !self.0.deny_publish.load(Ordering::SeqCst)
                    && self.0.publish_calls.fetch_add(1, Ordering::SeqCst) + 1
                        != self.0.revoke_publish_at.load(Ordering::SeqCst)
            }
            Action::Evaluate => {
                !self.0.deny_evaluate.load(Ordering::SeqCst)
                    && self.0.evaluate_calls.fetch_add(1, Ordering::SeqCst) + 1
                        != self.0.revoke_evaluate_at.load(Ordering::SeqCst)
            }
            Action::Validate => !self.0.deny_validate.load(Ordering::SeqCst),
            _ => true,
        }
    }
}
fn d(s: &str) -> PivotDecimal {
    s.parse().unwrap()
}
fn add(t: &str, name: &str, def: NodeDefinition) -> Edit {
    Edit::AddNode {
        temporary: t.into(),
        name: name.into(),
        definition: def,
    }
}
fn n(t: &str) -> NodeRef {
    NodeRef::Temporary(t.into())
}
fn apply(
    s: &mut WorkingModelService,
    rev: WorkingRevision,
    op: &str,
    edits: Vec<Edit>,
) -> EditResult {
    s.apply_edits(
        "alice",
        ApplyEdits {
            instance: rev.instance,
            model: rev.model,
            expected: rev,
            operation_id: op.into(),
            edits,
        },
    )
    .unwrap()
}
fn fixture(
    limits: Limits,
    calc: CalculationLimits,
) -> (WorkingModelService, EditResult, Arc<Controls>) {
    let controls = Arc::new(Controls::default());
    let mut s = WorkingModelService::with_all_limits(
        limits,
        EditLimits::default(),
        calc,
        Box::new(Policy(controls.clone())),
    )
    .unwrap();
    let c = s
        .create(
            "alice",
            CreateRequest {
                instance: s.info().instance,
                operation_id: "create".into(),
                name: "Financial".into(),
            },
        )
        .unwrap();
    let edits = vec![
        add("revenue", "Revenue", NodeDefinition::Input(d("100"))),
        add("costs", "Costs", NodeDefinition::Input(d("25"))),
        add(
            "net",
            "Net",
            NodeDefinition::Formula("Revenue - Costs".into()),
        ),
        add(
            "margin",
            "Margin",
            NodeDefinition::Formula("Net / Revenue".into()),
        ),
        add(
            "independent",
            "Independent",
            NodeDefinition::Formula("Costs + 1".into()),
        ),
        add(
            "dependent",
            "Dependent",
            NodeDefinition::Formula("Margin + 1".into()),
        ),
        Edit::AddInputPort {
            temporary: "sales-port".into(),
            name: "Sales".into(),
            node: n("revenue"),
            requirement: Requirement::Required,
        },
        Edit::AddInputPort {
            temporary: "cost-port".into(),
            name: "CostInput".into(),
            node: n("costs"),
            requirement: Requirement::Optional(d("40")),
        },
        Edit::AddOutputPort {
            temporary: "profit-port".into(),
            name: "Profit".into(),
            node: n("net"),
        },
        Edit::AddOutputPort {
            temporary: "margin-port".into(),
            name: "Margin".into(),
            node: n("margin"),
        },
    ];
    let r = apply(&mut s, c.revision, "seed", edits);
    (s, r, controls)
}
fn ordinary() -> (WorkingModelService, EditResult, Arc<Controls>) {
    fixture(Limits::default(), CalculationLimits::default())
}
fn publish_request(rev: WorkingRevision, operation: &str) -> PublishRequest {
    PublishRequest {
        instance: rev.instance,
        model: rev.model,
        expected: rev,
        operation_id: operation.into(),
    }
}
fn evaluate(
    capture: &CalculationCapture,
    outputs: Vec<AuthoringNodeId>,
    inputs: Vec<(AuthoringNodeId, PivotDecimal)>,
) -> EvaluationResult {
    capture
        .evaluate(EvaluationRequest { inputs, outputs })
        .unwrap()
}
fn send(
    host: &ModelHost,
    session: &mut Session,
    op: wire::request::Operation,
) -> wire::response::Outcome {
    let req = wire::Request {
        request_id: "test".into(),
        version: Some(wire::Version { major: 1, minor: 0 }),
        operation: Some(op),
    };
    wire::Response::decode(host.dispatch(session, &req.encode_to_vec()).as_slice())
        .unwrap()
        .outcome
        .unwrap()
}
fn session(host: &ModelHost) -> Session {
    let mut s = Session::new(
        pivot_host::Limits::default(),
        Arc::new(pivot_host::AllowAll),
    )
    .unwrap();
    assert!(matches!(
        send(
            host,
            &mut s,
            wire::request::Operation::Handshake(wire::Handshake {
                supported_versions: vec![wire::Version { major: 1, minor: 0 }],
                required_capabilities: vec![],
                optional_capabilities: vec![]
            })
        ),
        wire::response::Outcome::Handshake(_)
    ));
    s
}
fn describe(host: &ModelHost, model: PublishedModelRef) -> wire::ModelDescription {
    let mut s = session(host);
    match send(
        host,
        &mut s,
        wire::request::Operation::DescribeModel(wire::DescribeModel {
            model: Some(model.wire()),
        }),
    ) {
        wire::response::Outcome::Description(d) => d,
        other => panic!("{other:?}"),
    }
}
fn invoke_named(
    host: &ModelHost,
    model: PublishedModelRef,
    output: &str,
) -> wire::InvocationResult {
    let description = describe(host, model);
    let sales = description
        .inputs
        .iter()
        .find(|p| p.name == "Sales")
        .unwrap();
    let output = description
        .outputs
        .iter()
        .find(|p| p.name == output)
        .unwrap();
    let mut s = session(host);
    match send(
        host,
        &mut s,
        wire::request::Operation::InvokeModel(wire::InvokeModel {
            model: Some(model.wire()),
            inputs: vec![wire::InputBinding {
                port_id: sales.port_id.clone(),
                value: Some(wire::Value {
                    kind: Some(wire::value::Kind::Decimal(wire::Decimal128 {
                        lo: 100,
                        mid: 0,
                        hi: 0,
                        negative: false,
                        scale: 0,
                    })),
                }),
                source: 0,
            }],
            output_port_ids: vec![output.port_id.clone()],
        }),
    ) {
        wire::response::Outcome::Invocation(r) => r,
        other => panic!("{other:?}"),
    }
}
fn result_decimal(r: &wire::InvocationResult) -> PivotDecimal {
    let Some(wire::output_result::Outcome::Value(wire::Value {
        kind: Some(wire::value::Kind::Decimal(v)),
    })) = &r.outputs[0].outcome
    else {
        panic!("{r:?}");
    };
    PivotDecimal::from_parts(v.lo, v.mid, v.hi, v.negative, v.scale).unwrap()
}
fn published_count(s: &WorkingModelService) -> usize {
    let host = s.published_host();
    let mut session = session(host);
    match send(
        host,
        &mut session,
        wire::request::Operation::ListModels(wire::ListModels::default()),
    ) {
        wire::response::Outcome::Models(list) => list.models.len(),
        other => panic!("{other:?}"),
    }
}

#[test]
fn validation_and_unpublished_financial_evaluation_use_captured_defaults() {
    let (s, r, _) = ordinary();
    let before = s.read_draft("alice", r.revision).unwrap();
    let valid = s.validate_working("alice", r.revision).unwrap();
    assert!(valid.valid && valid.diagnostics.is_empty());
    let capture = s.capture_evaluation("alice", r.revision).unwrap();
    let result = evaluate(&capture, vec![r.nodes["net"], r.nodes["margin"]], vec![]);
    assert_eq!(result.outputs[&r.nodes["net"]], Ok(d("75")));
    assert_eq!(result.outputs[&r.nodes["margin"]], Ok(d("0.75")));
    assert_eq!(result.provenance.working, r.revision);
    assert_eq!(
        result.effective_inputs[&r.nodes["costs"]],
        EffectiveInput {
            value: d("25"),
            source: EffectiveSource::Captured
        }
    );
    assert_eq!(s.read_draft("alice", r.revision).unwrap(), before);
    assert_eq!(published_count(&s), 0);
}

#[test]
fn overrides_are_temporary_and_arithmetic_errors_do_not_poison_other_outputs() {
    let (s, r, _) = ordinary();
    let capture = s.capture_evaluation("alice", r.revision).unwrap();
    let result = evaluate(
        &capture,
        vec![
            r.nodes["margin"],
            r.nodes["net"],
            r.nodes["independent"],
            r.nodes["dependent"],
        ],
        vec![(r.nodes["revenue"], d("0"))],
    );
    assert_eq!(
        result.outputs[&r.nodes["margin"]],
        Err(CalculationError::Arithmetic(DecimalError::DivisionByZero))
    );
    assert_eq!(result.outputs[&r.nodes["net"]], Ok(d("-25")));
    assert_eq!(result.outputs[&r.nodes["independent"]], Ok(d("26")));
    assert!(
        matches!(result.outputs[&r.nodes["dependent"]], Err(CalculationError::DependencyFailed { origin, .. }) if origin == r.nodes["margin"])
    );
    assert_eq!(
        result.effective_inputs[&r.nodes["revenue"]].source,
        EffectiveSource::Override
    );
    assert_eq!(
        evaluate(&capture, vec![r.nodes["net"]], vec![]).outputs[&r.nodes["net"]],
        Ok(d("75"))
    );
}

#[test]
fn capture_survives_editing_and_service_destruction() {
    let (mut s, r, _) = ordinary();
    let capture = s.capture_evaluation("alice", r.revision).unwrap();
    let next = apply(
        &mut s,
        r.revision,
        "change",
        vec![Edit::SetNode {
            node: NodeRef::Existing(r.nodes["costs"]),
            definition: NodeDefinition::Input(d("50")),
        }],
    );
    assert!(matches!(
        s.capture_evaluation("alice", r.revision),
        Err(CaptureError::Service(Error::RevisionConflict))
    ));
    let fresh = s.capture_evaluation("alice", next.revision).unwrap();
    assert_ne!(capture.provenance().snapshot, fresh.provenance().snapshot);
    drop(s);
    let old = evaluate(&capture, vec![r.nodes["net"]], vec![]);
    let new = evaluate(&fresh, vec![r.nodes["net"]], vec![]);
    assert_eq!(old.outputs[&r.nodes["net"]], Ok(d("75")));
    assert_eq!(new.outputs[&r.nodes["net"]], Ok(d("50")));
    assert_eq!(old.provenance.working, r.revision);
    assert_eq!(new.provenance.working, next.revision);
}

#[test]
fn invalid_drafts_still_save_but_validation_and_capture_reject_globally() {
    for kind in [0, 1, 2, 3] {
        let (mut s, r, _) = ordinary();
        let edits = match kind {
            0 => vec![Edit::SetNode {
                node: NodeRef::Existing(r.nodes["net"]),
                definition: NodeDefinition::Formula("Revenue -".into()),
            }],
            1 => vec![Edit::RemoveNode(NodeRef::Existing(r.nodes["costs"]))],
            2 => vec![Edit::SetNode {
                node: NodeRef::Existing(r.nodes["net"]),
                definition: NodeDefinition::Formula("Margin".into()),
            }],
            _ => vec![Edit::SetNode {
                node: NodeRef::Existing(r.nodes["revenue"]),
                definition: NodeDefinition::Formula("5".into()),
            }],
        };
        let bad = apply(&mut s, r.revision, "invalid", edits);
        let before = s.read_draft("alice", bad.revision).unwrap();
        let report = s.validate_working("alice", bad.revision).unwrap();
        assert!(!report.valid && !report.diagnostics.is_empty());
        assert!(matches!(
            s.capture_evaluation("alice", bad.revision),
            Err(CaptureError::Validation(_))
        ));
        assert!(matches!(
            s.publish("alice", publish_request(bad.revision, "invalid-pub")),
            Err(PublicationError::Validation(_))
        ));
        assert_eq!(s.read_draft("alice", bad.revision).unwrap(), before);
        assert_eq!(published_count(&s), 0);
        assert!(matches!(
            s.operation_outcome(r.revision.instance, "alice", "invalid-pub"),
            Ok(OperationOutcome::PublicationRejected(
                PublicationError::Validation(_)
            ))
        ));
    }
}

#[test]
fn publication_named_invocation_and_exact_immutable_revisions() {
    let (mut s, r, _) = ordinary();
    let first = s
        .publish("alice", publish_request(r.revision, "publish-1"))
        .unwrap();
    assert_eq!(
        result_decimal(&invoke_named(s.published_host(), first.published, "Profit")),
        d("60")
    );
    assert_eq!(
        s.inspect(r.revision.instance, "alice", r.revision.model)
            .unwrap()
            .revision,
        r.revision
    );
    let next = apply(
        &mut s,
        r.revision,
        "change-formula",
        vec![
            Edit::SetNode {
                node: NodeRef::Existing(r.nodes["net"]),
                definition: NodeDefinition::Formula("Revenue - Costs + 10".into()),
            },
            Edit::RenamePort {
                port: PortRef::Existing(r.ports["profit-port"]),
                name: "NetIncome".into(),
            },
        ],
    );
    let second = s
        .publish("alice", publish_request(next.revision, "publish-2"))
        .unwrap();
    assert_eq!(first.published.model, second.published.model);
    assert_ne!(first.published.revision, second.published.revision);
    assert_eq!(
        result_decimal(&invoke_named(s.published_host(), first.published, "Profit")),
        d("60")
    );
    assert_eq!(
        result_decimal(&invoke_named(
            s.published_host(),
            second.published,
            "NetIncome"
        )),
        d("70")
    );
    let old = describe(s.published_host(), first.published);
    let new = describe(s.published_host(), second.published);
    assert_eq!(
        old.outputs
            .iter()
            .find(|p| p.name == "Profit")
            .unwrap()
            .port_id,
        new.outputs
            .iter()
            .find(|p| p.name == "NetIncome")
            .unwrap()
            .port_id
    );
    assert_eq!(old.inputs[0].port_id, new.inputs[0].port_id);
    assert_eq!(published_count(&s), 2);
}

#[test]
fn prepared_publication_conflicts_after_head_advance_without_registering() {
    let (mut s, r, _) = ordinary();
    let request = publish_request(r.revision, "prepared");
    let prepared = s.prepare_publication("alice", request.clone()).unwrap();
    let next = apply(
        &mut s,
        r.revision,
        "edit",
        vec![Edit::SetNode {
            node: NodeRef::Existing(r.nodes["costs"]),
            definition: NodeDefinition::Input(d("50")),
        }],
    );
    assert_eq!(
        s.commit_publication("alice", prepared),
        Err(PublicationError::Service(Error::RevisionConflict))
    );
    assert_eq!(published_count(&s), 0);
    assert_eq!(
        s.publish("alice", request),
        Err(PublicationError::Service(Error::RevisionConflict))
    );
    assert_eq!(
        s.inspect(next.revision.instance, "alice", next.revision.model)
            .unwrap()
            .revision,
        next.revision
    );
}

#[test]
fn publication_replay_after_lost_response_precedes_revision_check_and_quotas() {
    let (mut s, r, _) = fixture(
        Limits {
            receipts: 4,
            ..Limits::default()
        },
        CalculationLimits {
            published_revisions: 1,
            ..CalculationLimits::default()
        },
    );
    let request = publish_request(r.revision, "lost");
    let committed = s.publish("alice", request.clone()).unwrap(); // discard network-equivalent response
    let next = apply(
        &mut s,
        r.revision,
        "later-edit",
        vec![Edit::RenameNode {
            node: NodeRef::Existing(r.nodes["costs"]),
            name: "Expenses".into(),
        }],
    );
    for _ in 0..5 {
        assert_eq!(s.publish("alice", request.clone()), Ok(committed.clone()));
    }
    assert_eq!(
        s.operation_outcome(r.revision.instance, "alice", "lost"),
        Ok(OperationOutcome::Published(committed.clone()))
    );
    assert_eq!(published_count(&s), 1);
    assert_eq!(
        s.publish("alice", publish_request(next.revision, "lost")),
        Err(PublicationError::Service(Error::OperationIdConflict))
    );
    assert_eq!(
        s.publish("alice", publish_request(next.revision, "new")),
        Err(PublicationError::Service(Error::ReceiptLimit))
    );
}

#[test]
fn publication_authorization_is_separate_and_rechecked_before_commit() {
    let (mut s, r, controls) = ordinary();
    controls.deny_publish.store(true, Ordering::SeqCst);
    assert!(s.capture_evaluation("alice", r.revision).is_ok());
    assert_eq!(
        s.publish("alice", publish_request(r.revision, "denied")),
        Err(PublicationError::Service(Error::Denied))
    );
    controls.deny_publish.store(false, Ordering::SeqCst);
    let prepared = s
        .prepare_publication("alice", publish_request(r.revision, "revoked"))
        .unwrap();
    let calls = controls.publish_calls.load(Ordering::SeqCst);
    controls
        .revoke_publish_at
        .store(calls + 2, Ordering::SeqCst); // allow commit entry, deny final check
    assert_eq!(
        s.commit_publication("alice", prepared),
        Err(PublicationError::Service(Error::Denied))
    );
    assert_eq!(published_count(&s), 0);
    controls.revoke_publish_at.store(0, Ordering::SeqCst);
    assert_eq!(
        s.publish("alice", publish_request(r.revision, "revoked")),
        Err(PublicationError::Service(Error::Denied))
    );
    assert_eq!(
        s.operation_outcome(r.revision.instance, "alice", "revoked"),
        Ok(OperationOutcome::PublicationRejected(
            PublicationError::Service(Error::Denied)
        ))
    );
}

#[test]
fn evaluation_checks_permission_at_capture_execution_and_result_delivery() {
    let (s, r, controls) = ordinary();
    controls.deny_validate.store(true, Ordering::SeqCst);
    assert_eq!(s.validate_working("alice", r.revision), Err(Error::Denied));
    controls.deny_evaluate.store(true, Ordering::SeqCst);
    assert!(matches!(
        s.capture_evaluation("alice", r.revision),
        Err(CaptureError::Service(Error::Denied))
    ));
    controls.deny_evaluate.store(false, Ordering::SeqCst);
    let capture = s.capture_evaluation("alice", r.revision).unwrap();
    controls.revoke_evaluate_at.store(
        controls.evaluate_calls.load(Ordering::SeqCst) + 2,
        Ordering::SeqCst,
    );
    assert_eq!(
        capture.evaluate(EvaluationRequest {
            outputs: vec![r.nodes["net"]],
            inputs: vec![]
        }),
        Err(Error::Denied)
    );
    controls.deny_evaluate.store(true, Ordering::SeqCst);
    assert_eq!(
        capture.evaluate(EvaluationRequest {
            outputs: vec![r.nodes["net"]],
            inputs: vec![]
        }),
        Err(Error::Denied)
    );
}

#[test]
fn capture_work_and_publication_limits_release_slots_without_mutations() {
    let (s, r, _) = fixture(
        Limits::default(),
        CalculationLimits {
            captures: 1,
            ..CalculationLimits::default()
        },
    );
    let capture = s.capture_evaluation("alice", r.revision).unwrap();
    assert!(matches!(
        s.capture_evaluation("alice", r.revision),
        Err(CaptureError::Service(Error::CaptureLimit))
    ));
    assert!(matches!(
        s.prepare_publication("alice", publish_request(r.revision, "no-capture")),
        Err(PublicationError::Service(Error::CaptureLimit))
    ));
    drop(capture);
    assert!(s.capture_evaluation("alice", r.revision).is_ok());
    let (s, r, _) = fixture(
        Limits::default(),
        CalculationLimits {
            work: 1,
            ..CalculationLimits::default()
        },
    );
    assert_eq!(
        s.validate_working("alice", r.revision),
        Err(Error::WorkLimit)
    );
    assert!(matches!(
        s.capture_evaluation("alice", r.revision),
        Err(CaptureError::Service(Error::WorkLimit))
    ));
    let (mut s, r, _) = fixture(
        Limits::default(),
        CalculationLimits {
            published_revisions: 1,
            ..CalculationLimits::default()
        },
    );
    s.publish("alice", publish_request(r.revision, "one"))
        .unwrap();
    assert_eq!(
        s.publish("alice", publish_request(r.revision, "two")),
        Err(PublicationError::Service(Error::PublishedLimit))
    );
    assert_eq!(published_count(&s), 1);
}

#[test]
fn evaluation_rejects_unknown_duplicate_or_computed_overrides_and_invalid_outputs() {
    let (s, r, _) = fixture(
        Limits::default(),
        CalculationLimits {
            inputs: 2,
            outputs: 2,
            ..CalculationLimits::default()
        },
    );
    let capture = s.capture_evaluation("alice", r.revision).unwrap();
    let unknown: AuthoringNodeId = "00000000-0000-0000-0000-000000000000".parse().unwrap();
    for inputs in [
        vec![(unknown, d("1"))],
        vec![(r.nodes["net"], d("1"))],
        vec![(r.nodes["costs"], d("1")), (r.nodes["costs"], d("2"))],
    ] {
        assert_eq!(
            capture.evaluate(EvaluationRequest {
                inputs,
                outputs: vec![r.nodes["net"]]
            }),
            Err(Error::InvalidInput)
        );
    }
    for outputs in [vec![], vec![unknown], vec![r.nodes["net"], r.nodes["net"]]] {
        assert_eq!(
            capture.evaluate(EvaluationRequest {
                inputs: vec![],
                outputs
            }),
            Err(Error::InvalidOutput)
        );
    }
    assert_eq!(
        capture.evaluate(EvaluationRequest {
            inputs: vec![],
            outputs: vec![r.nodes["net"]; 3]
        }),
        Err(Error::WorkLimit)
    );
}

#[test]
fn publication_operation_namespace_is_shared_with_create_and_edits() {
    let (mut s, r, _) = ordinary();
    for op in ["create", "seed"] {
        assert_eq!(
            s.publish("alice", publish_request(r.revision, op)),
            Err(PublicationError::Service(Error::OperationIdConflict))
        );
    }
    s.publish("alice", publish_request(r.revision, "publish"))
        .unwrap();
    assert_eq!(
        s.create(
            "alice",
            CreateRequest {
                instance: r.revision.instance,
                operation_id: "publish".into(),
                name: "Other".into()
            }
        ),
        Err(Error::OperationIdConflict)
    );
    assert_eq!(
        s.apply_edits(
            "alice",
            ApplyEdits {
                instance: r.revision.instance,
                model: r.revision.model,
                expected: r.revision,
                operation_id: "publish".into(),
                edits: vec![Edit::RemoveNode(NodeRef::Existing(r.nodes["net"]))]
            }
        ),
        Err(Error::OperationIdConflict)
    );
}

#[test]
fn arithmetic_failure_does_not_prevent_structural_publication() {
    let (mut s, r, _) = ordinary();
    let next = apply(
        &mut s,
        r.revision,
        "zero",
        vec![Edit::SetNode {
            node: NodeRef::Existing(r.nodes["revenue"]),
            definition: NodeDefinition::Input(d("0")),
        }],
    );
    assert!(s.validate_working("alice", next.revision).unwrap().valid);
    let capture = s.capture_evaluation("alice", next.revision).unwrap();
    assert!(matches!(
        evaluate(&capture, vec![r.nodes["margin"]], vec![]).outputs[&r.nodes["margin"]],
        Err(CalculationError::Arithmetic(_))
    ));
    assert!(
        s.publish("alice", publish_request(next.revision, "publish-zero"))
            .is_ok()
    );
}

#[test]
fn publication_preparations_can_be_dropped_or_replayed_without_duplicate_entries() {
    let (mut s, r, _) = ordinary();
    let request = publish_request(r.revision, "prepared");
    let dropped = s.prepare_publication("alice", request.clone()).unwrap();
    drop(dropped);
    assert_eq!(
        s.operation_outcome(r.revision.instance, "alice", "prepared"),
        Ok(OperationOutcome::Unknown)
    );
    let a = s.prepare_publication("alice", request.clone()).unwrap();
    let b = s.prepare_publication("alice", request.clone()).unwrap();
    let committed = s.commit_publication("alice", a).unwrap();
    assert_eq!(s.commit_publication("alice", b), Ok(committed));
    assert_eq!(published_count(&s), 1);
}

#[test]
fn publication_cannot_commit_when_receipt_capacity_fills_after_preparation() {
    let (mut s, r, _) = fixture(
        Limits {
            receipts: 3,
            ..Limits::default()
        },
        CalculationLimits::default(),
    );
    let prepared = s
        .prepare_publication("alice", publish_request(r.revision, "prepared"))
        .unwrap();
    s.publish("alice", publish_request(r.revision, "fill-slot"))
        .unwrap();
    assert_eq!(
        s.commit_publication("alice", prepared),
        Err(PublicationError::Service(Error::ReceiptLimit))
    );
    assert_eq!(published_count(&s), 1);
    assert_eq!(
        s.operation_outcome(r.revision.instance, "alice", "prepared"),
        Ok(OperationOutcome::Unknown)
    );
}

#[test]
fn deleted_recreated_public_port_gets_a_new_identity_within_the_same_lineage() {
    let (mut s, r, _) = ordinary();
    let first = s
        .publish("alice", publish_request(r.revision, "first"))
        .unwrap();
    let next = apply(
        &mut s,
        r.revision,
        "recreate-port",
        vec![
            Edit::RemovePort(PortRef::Existing(r.ports["profit-port"])),
            Edit::AddOutputPort {
                temporary: "replacement".into(),
                name: "Profit".into(),
                node: NodeRef::Existing(r.nodes["net"]),
            },
        ],
    );
    let second = s
        .publish("alice", publish_request(next.revision, "second"))
        .unwrap();
    let old = describe(s.published_host(), first.published);
    let new = describe(s.published_host(), second.published);
    assert_eq!(first.published.model, second.published.model);
    assert_ne!(
        old.outputs
            .iter()
            .find(|p| p.name == "Profit")
            .unwrap()
            .port_id,
        new.outputs
            .iter()
            .find(|p| p.name == "Profit")
            .unwrap()
            .port_id
    );
    assert_eq!(
        result_decimal(&invoke_named(s.published_host(), first.published, "Profit")),
        d("60")
    );
    assert_eq!(
        result_decimal(&invoke_named(
            s.published_host(),
            second.published,
            "Profit"
        )),
        d("60")
    );
}

#[test]
fn prepared_publications_are_bound_to_the_service_instance_and_principal() {
    let (s, r, _) = ordinary();
    let prepared = s
        .prepare_publication("alice", publish_request(r.revision, "prepared"))
        .unwrap();
    let (mut next, _, _) = ordinary();
    assert_eq!(
        next.commit_publication("alice", prepared),
        Err(PublicationError::Service(Error::StaleInstance))
    );
    assert_eq!(published_count(&next), 0);
    let (mut s, r, _) = ordinary();
    let prepared = s
        .prepare_publication("alice", publish_request(r.revision, "principal"))
        .unwrap();
    assert_eq!(
        s.commit_publication("other", prepared),
        Err(PublicationError::Service(Error::Denied))
    );
    assert_eq!(published_count(&s), 0);
}
