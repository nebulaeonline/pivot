use pivot_host::management::{self as m, adapter::*};
use pivot_protocol::{authoring::v1 as a, foundation, v1 as p};
use prost::Message;
use std::{
    collections::BTreeMap,
    sync::{
        Arc, Mutex, Weak,
        atomic::{AtomicBool, AtomicU64, AtomicUsize, Ordering},
    },
};
type Op = a::authoring_request::Operation;
type Out = a::authoring_response::Outcome;
static REQUEST: AtomicUsize = AtomicUsize::new(1);
#[derive(Default)]
struct Controls {
    deny_all: AtomicBool,
    deny_info: AtomicBool,
    deny_inspect: AtomicBool,
    deny_validate: AtomicBool,
    deny_evaluate: AtomicBool,
    deny_publish: AtomicBool,
    publish_calls: AtomicUsize,
    deny_publish_at: AtomicUsize,
    deny_close: AtomicBool,
    edit_calls: AtomicUsize,
    deny_edit_at: AtomicUsize,
    info_calls: AtomicUsize,
    deny_info_at: AtomicUsize,
    eval_calls: AtomicUsize,
    off_lock: AtomicBool,
    service: Mutex<Option<Weak<Mutex<m::WorkingModelService>>>>,
    edit_on_eval: Mutex<Option<m::ApplyEdits>>,
}
struct Policy(Arc<Controls>);
impl m::Authorizer for Policy {
    fn allowed(&self, who: &str, action: m::Action, _: Option<m::WorkingModelId>) -> bool {
        if who == "intruder" || self.0.deny_all.load(Ordering::SeqCst) {
            return false;
        }
        let c = &self.0;
        match action {
            m::Action::Info => {
                !c.deny_info.load(Ordering::SeqCst)
                    && c.info_calls.fetch_add(1, Ordering::SeqCst) + 1
                        != c.deny_info_at.load(Ordering::SeqCst)
            }
            m::Action::Inspect => !c.deny_inspect.load(Ordering::SeqCst),
            m::Action::Validate => !c.deny_validate.load(Ordering::SeqCst),
            m::Action::Publish => {
                !c.deny_publish.load(Ordering::SeqCst)
                    && c.publish_calls.fetch_add(1, Ordering::SeqCst) + 1
                        != c.deny_publish_at.load(Ordering::SeqCst)
            }
            m::Action::Close => !c.deny_close.load(Ordering::SeqCst),
            m::Action::Edit => {
                c.edit_calls.fetch_add(1, Ordering::SeqCst) + 1
                    != c.deny_edit_at.load(Ordering::SeqCst)
            }
            m::Action::Evaluate => {
                if c.deny_evaluate.load(Ordering::SeqCst) {
                    return false;
                }
                if c.eval_calls.fetch_add(1, Ordering::SeqCst) == 2
                    && let Some(edit) = c.edit_on_eval.lock().unwrap().take()
                {
                    let service = c
                        .service
                        .lock()
                        .unwrap()
                        .as_ref()
                        .unwrap()
                        .upgrade()
                        .unwrap();
                    if let Ok(mut guard) = service.try_lock() {
                        c.off_lock.store(true, Ordering::SeqCst);
                        guard.apply_edits("writer", edit).unwrap();
                    }
                }
                true
            }
            _ => true,
        }
    }
}
#[derive(Default)]
struct TestClock(AtomicU64);
impl Clock for TestClock {
    fn seconds(&self) -> u64 {
        self.0.load(Ordering::SeqCst)
    }
}
struct Fixture {
    service: Arc<Mutex<m::WorkingModelService>>,
    adapter: Arc<AuthoringAdapter>,
    session: AuthoringSession,
    controls: Arc<Controls>,
    clock: Arc<TestClock>,
    instance: String,
}
fn fixture(limits: m::Limits, options: AdapterLimits) -> Fixture {
    let controls = Arc::new(Controls::default());
    let service = Arc::new(Mutex::new(
        m::WorkingModelService::new(limits, Box::new(Policy(controls.clone()))).unwrap(),
    ));
    *controls.service.lock().unwrap() = Some(Arc::downgrade(&service));
    let instance = service.lock().unwrap().info().instance.to_string();
    let clock = Arc::new(TestClock::default());
    let adapter = AuthoringAdapter::with_clock(service.clone(), options, clock.clone()).unwrap();
    let mut session = adapter.open_session("alice").unwrap();
    hello(&mut session);
    Fixture {
        service,
        adapter,
        session,
        controls,
        clock,
        instance,
    }
}
fn ordinary() -> Fixture {
    fixture(m::Limits::default(), AdapterLimits::default())
}
#[test]
fn negotiated_request_ceiling_also_bounds_responses() {
    let options = AdapterLimits {
        preflight: foundation::Limits {
            bytes: 512,
            ..foundation::Limits::default()
        },
        response_bytes: 4096,
        page_target_bytes: 4096,
        ..AdapterLimits::default()
    };
    let mut f = fixture(m::Limits::default(), options);
    let request = envelope(Op::GetServiceInfo(a::GetServiceInfo {}));
    let bytes = f.session.dispatch(&request.encode_to_vec());
    assert!(bytes.len() <= 512);
    let response = p::Response::decode(bytes.as_slice()).unwrap();
    assert!(response.outcome.is_some());
}
fn envelope(op: Op) -> p::Request {
    p::Request {
        request_id: format!("r{}", REQUEST.fetch_add(1, Ordering::SeqCst)),
        version: Some(p::Version { major: 1, minor: 1 }),
        operation: Some(p::request::Operation::Authoring(a::AuthoringRequest {
            operation: Some(op),
        })),
    }
}
fn hello(s: &mut AuthoringSession) {
    let r = p::Request {
        request_id: "hello".into(),
        version: Some(p::Version { major: 1, minor: 1 }),
        operation: Some(p::request::Operation::Handshake(p::Handshake {
            supported_versions: vec![p::Version { major: 1, minor: 1 }],
            required_capabilities: vec!["working-models-v1".into()],
            optional_capabilities: vec![],
        })),
    };
    let response = p::Response::decode(s.dispatch(&r.encode_to_vec()).as_slice()).unwrap();
    assert!(matches!(
        response.outcome,
        Some(p::response::Outcome::Handshake(_))
    ));
}
fn call(s: &mut AuthoringSession, op: Op) -> Out {
    let request = envelope(op);
    let bytes = s.dispatch(&request.encode_to_vec());
    let response =
        foundation::decode_response(&bytes, &request, foundation::Limits::default()).unwrap();
    let Some(p::response::Outcome::Authoring(response)) = response.outcome else {
        panic!()
    };
    response.outcome.unwrap()
}
fn error(out: Out) -> a::AuthoringErrorCode {
    match out {
        Out::Error(e) => a::AuthoringErrorCode::try_from(e.code).unwrap(),
        other => panic!("expected failure: {other:?}"),
    }
}
fn number(s: &str) -> p::Value {
    let (lo, mid, hi, negative, scale) = s.parse::<pivot_types::PivotDecimal>().unwrap().parts();
    p::Value {
        kind: Some(p::value::Kind::Decimal(p::Decimal128 {
            lo,
            mid,
            hi,
            negative,
            scale,
        })),
    }
}
fn nr(temp: &str) -> a::NodeReference {
    a::NodeReference {
        kind: Some(a::node_reference::Kind::TemporaryId(temp.into())),
    }
}
fn node(id: &str) -> a::NodeReference {
    a::NodeReference {
        kind: Some(a::node_reference::Kind::NodeId(id.into())),
    }
}
fn port(id: &str) -> a::PortReference {
    a::PortReference {
        kind: Some(a::port_reference::Kind::PortId(id.into())),
    }
}
fn add(temp: &str, name: &str, def: a::node_definition::Kind) -> a::Edit {
    a::Edit {
        kind: Some(a::edit::Kind::AddNode(a::AddNode {
            temporary_id: temp.into(),
            name: name.into(),
            definition: Some(a::NodeDefinition { kind: Some(def) }),
        })),
    }
}
fn input(temp: &str, name: &str, value: &str) -> a::Edit {
    add(temp, name, a::node_definition::Kind::Input(number(value)))
}
fn formula(temp: &str, name: &str, source: &str) -> a::Edit {
    add(
        temp,
        name,
        a::node_definition::Kind::FormulaSource(source.into()),
    )
}
fn context(revision: &a::WorkingRevision, operation: &str) -> a::MutationContext {
    a::MutationContext {
        service_instance_id: revision.service_instance_id.clone(),
        working_model_id: revision.working_model_id.clone(),
        expected_revision: Some(revision.clone()),
        operation_id: operation.into(),
    }
}
fn edit(revision: &a::WorkingRevision, operation: &str, edits: Vec<a::Edit>) -> Op {
    Op::ApplyEdits(a::ApplyEdits {
        mutation: Some(context(revision, operation)),
        edits,
    })
}
fn create(s: &mut AuthoringSession, instance: &str, op: &str) -> a::Created {
    match call(
        s,
        Op::CreateWorking(a::CreateWorking {
            service_instance_id: instance.into(),
            operation_id: op.into(),
            name: "Financial".into(),
        }),
    ) {
        Out::Created(v) => v,
        other => panic!("{other:?}"),
    }
}
struct Model {
    revision: a::WorkingRevision,
    nodes: BTreeMap<String, String>,
    ports: BTreeMap<String, String>,
}
fn seed(f: &mut Fixture) -> Model {
    let created = create(&mut f.session, &f.instance, "create");
    let edits = vec![
        formula("net", "Net", "Revenue - Costs"),
        input("revenue", "Revenue", "100"),
        input("costs", "Costs", "25"),
        formula("margin", "Margin", "Net / Revenue"),
        formula("independent", "Independent", "Costs + 1"),
        formula("dependent", "Dependent", "Margin + 1"),
        a::Edit {
            kind: Some(a::edit::Kind::AddInputPort(a::AddInputPort {
                temporary_id: "sales".into(),
                name: "Sales".into(),
                node: Some(nr("revenue")),
                requirement: Some(a::Requirement {
                    kind: Some(a::requirement::Kind::Required(a::Required {})),
                }),
            })),
        },
        a::Edit {
            kind: Some(a::edit::Kind::AddInputPort(a::AddInputPort {
                temporary_id: "costs-port".into(),
                name: "CostInput".into(),
                node: Some(nr("costs")),
                requirement: Some(a::Requirement {
                    kind: Some(a::requirement::Kind::OptionalDefault(number("40"))),
                }),
            })),
        },
        a::Edit {
            kind: Some(a::edit::Kind::AddOutputPort(a::AddOutputPort {
                temporary_id: "profit".into(),
                name: "Profit".into(),
                node: Some(nr("net")),
            })),
        },
    ];
    match call(
        &mut f.session,
        edit(created.revision.as_ref().unwrap(), "seed", edits),
    ) {
        Out::Edited(v) => Model {
            revision: v.revision.unwrap(),
            nodes: v
                .nodes
                .into_iter()
                .map(|v| (v.temporary_id, v.node_id))
                .collect(),
            ports: v
                .ports
                .into_iter()
                .map(|v| (v.temporary_id, v.port_id))
                .collect(),
        },
        other => panic!("{other:?}"),
    }
}
fn read(revision: &a::WorkingRevision, section: a::DraftSection, token: &str) -> Op {
    Op::ReadDraft(a::ReadDraft {
        revision: Some(revision.clone()),
        section: section as i32,
        page: Some(a::Page {
            page_size: 1,
            continuation_token: token.into(),
        }),
    })
}
fn all_nodes(s: &mut AuthoringSession, r: &a::WorkingRevision) -> Vec<a::RawNode> {
    let mut nodes = Vec::new();
    let mut token = String::new();
    loop {
        let Out::Draft(page) = call(s, read(r, a::DraftSection::Nodes, &token)) else {
            panic!()
        };
        nodes.extend(page.nodes);
        token = page.continuation_token;
        if token.is_empty() {
            return nodes;
        }
    }
}
fn publish(s: &mut AuthoringSession, r: &a::WorkingRevision, op: &str) -> Out {
    call(
        s,
        Op::PublishWorking(a::PublishWorking {
            mutation: Some(context(r, op)),
        }),
    )
}
fn native_revision(r: &a::WorkingRevision) -> m::WorkingRevision {
    m::WorkingRevision {
        instance: r.service_instance_id.parse().unwrap(),
        model: r.working_model_id.parse().unwrap(),
        sequence: r.sequence,
    }
}

#[test]
fn service_info_create_list_open_close_and_disconnect_are_serialized() {
    let mut f = ordinary();
    let Out::ServiceInfo(info) = call(&mut f.session, Op::GetServiceInfo(a::GetServiceInfo {}))
    else {
        panic!()
    };
    assert_eq!(info.durability, a::Durability::MemoryOnly as i32);
    assert_eq!(info.service_instance_id, f.instance);
    assert!(info.attachments_bound_to_connection);
    assert!(info.limits.unwrap().authoring_sessions > 0);
    let c = create(&mut f.session, &f.instance, "created");
    let Out::WorkingModels(list) = call(
        &mut f.session,
        Op::ListWorking(a::ListWorking {
            service_instance_id: f.instance.clone(),
            page: None,
        }),
    ) else {
        panic!()
    };
    assert_eq!(list.models.len(), 1);
    assert_eq!(list.models[0].revision, c.revision);
    let open = Op::OpenWorking(a::OpenWorking {
        service_instance_id: f.instance.clone(),
        working_model_id: c.working_model_id.clone(),
    });
    let Out::Opened(a) = call(&mut f.session, open.clone()) else {
        panic!()
    };
    let mut second = f.adapter.open_session("alice").unwrap();
    hello(&mut second);
    let Out::Opened(b) = call(&mut second, open.clone()) else {
        panic!()
    };
    assert_ne!(a.attachment_id, b.attachment_id);
    let close = Op::CloseAttachment(a::CloseAttachment {
        service_instance_id: f.instance.clone(),
        attachment_id: a.attachment_id,
    });
    assert_eq!(
        error(call(&mut second, close.clone())),
        a::AuthoringErrorCode::NotFound
    );
    assert!(matches!(
        call(&mut f.session, close.clone()),
        Out::Closed(_)
    ));
    assert!(matches!(call(&mut f.session, close), Out::Closed(_)));
    f.controls.deny_close.store(true, Ordering::SeqCst);
    drop(second);
    f.controls.deny_close.store(false, Ordering::SeqCst);
    assert!(matches!(call(&mut f.session, open), Out::Opened(_)));
    assert_eq!(
        f.service
            .lock()
            .unwrap()
            .list(f.instance.parse().unwrap(), "alice")
            .unwrap()
            .len(),
        1
    );
}

#[test]
fn atomic_edits_rollback_and_stable_bound_names_survive_adapter() {
    let mut f = ordinary();
    let m = seed(&mut f);
    let before = all_nodes(&mut f.session, &m.revision);
    assert_eq!(m.revision.sequence, 1);
    let edits = vec![
        a::Edit {
            kind: Some(a::edit::Kind::RenameNode(a::RenameNode {
                node: Some(node(&m.nodes["revenue"])),
                name: "Sales".into(),
            })),
        },
        input("x", "Extra", "1"),
        input("duplicate", "Costs", "9"),
    ];
    assert_eq!(
        error(call(&mut f.session, edit(&m.revision, "rollback", edits))),
        a::AuthoringErrorCode::InvalidReference
    );
    assert_eq!(all_nodes(&mut f.session, &m.revision), before);
    let Out::Edited(renamed) = call(
        &mut f.session,
        edit(
            &m.revision,
            "rename",
            vec![a::Edit {
                kind: Some(a::edit::Kind::RenameNode(a::RenameNode {
                    node: Some(node(&m.nodes["revenue"])),
                    name: "Sales".into(),
                })),
            }],
        ),
    ) else {
        panic!()
    };
    let nodes = all_nodes(&mut f.session, renamed.revision.as_ref().unwrap());
    let net = nodes.iter().find(|n| n.node_id == m.nodes["net"]).unwrap();
    assert!(net.bound_dependency_ids.contains(&m.nodes["revenue"]));
    assert_eq!(
        net.definition.as_ref().unwrap().kind,
        Some(a::node_definition::Kind::FormulaSource(
            "Revenue - Costs".into()
        ))
    );
}

#[test]
fn staged_partial_port_updates_preserve_omissions_and_original_intent_on_replay() {
    let mut f = ordinary();
    let m = seed(&mut f);
    let patch = edit(
        &m.revision,
        "patch",
        vec![
            input("other", "Other", "30"),
            a::Edit {
                kind: Some(a::edit::Kind::SetInputPort(a::SetInputPort {
                    port: Some(port(&m.ports["costs-port"])),
                    node: Some(nr("other")),
                    requirement: None,
                })),
            },
            a::Edit {
                kind: Some(a::edit::Kind::SetInputPort(a::SetInputPort {
                    port: Some(port(&m.ports["costs-port"])),
                    node: None,
                    requirement: Some(a::Requirement {
                        kind: Some(a::requirement::Kind::OptionalDefault(number("41.0"))),
                    }),
                })),
            },
        ],
    );
    let Out::Edited(committed) = call(&mut f.session, patch.clone()) else {
        panic!()
    };
    let r = committed.revision.clone().unwrap();
    let other = committed.nodes[0].node_id.clone();
    let Out::Draft(page) = call(
        &mut f.session,
        Op::ReadDraft(a::ReadDraft {
            revision: Some(r.clone()),
            section: a::DraftSection::Ports as i32,
            page: None,
        }),
    ) else {
        panic!()
    };
    let a::raw_port::Kind::Input(p) = page
        .ports
        .iter()
        .find(|p| p.port_id == m.ports["costs-port"])
        .unwrap()
        .kind
        .as_ref()
        .unwrap()
    else {
        panic!()
    };
    assert_eq!(p.node_id, other);
    assert_eq!(
        p.requirement.as_ref().unwrap().kind,
        Some(a::requirement::Kind::OptionalDefault(number("41.0")))
    );
    let Out::Edited(next) = call(
        &mut f.session,
        edit(
            &r,
            "later",
            vec![a::Edit {
                kind: Some(a::edit::Kind::SetInputPort(a::SetInputPort {
                    port: Some(port(&m.ports["costs-port"])),
                    node: Some(node(&m.nodes["costs"])),
                    requirement: None,
                })),
            }],
        ),
    ) else {
        panic!()
    };
    assert_eq!(
        call(&mut f.session, patch.clone()),
        Out::Edited(committed.clone())
    );
    let mut mismatch = patch;
    if let Op::ApplyEdits(v) = &mut mismatch
        && let Some(a::edit::Kind::SetInputPort(p)) = &mut v.edits[2].kind
    {
        p.requirement = Some(a::Requirement {
            kind: Some(a::requirement::Kind::OptionalDefault(number("41.00"))),
        });
    }
    assert_eq!(
        error(call(&mut f.session, mismatch)),
        a::AuthoringErrorCode::OperationIdConflict
    );
    assert_eq!(
        f.service
            .lock()
            .unwrap()
            .inspect(
                f.instance.parse().unwrap(),
                "alice",
                m.revision.working_model_id.parse().unwrap()
            )
            .unwrap()
            .revision
            .sequence,
        next.revision.unwrap().sequence
    );
}

#[test]
fn incomplete_drafts_diagnostics_and_validation_are_distinct_from_saving() {
    let mut f = ordinary();
    let m = seed(&mut f);
    let source = "  Revenue -  ";
    let Out::Edited(saved) = call(
        &mut f.session,
        edit(
            &m.revision,
            "invalid",
            vec![a::Edit {
                kind: Some(a::edit::Kind::SetNode(a::SetNode {
                    node: Some(node(&m.nodes["net"])),
                    definition: Some(a::NodeDefinition {
                        kind: Some(a::node_definition::Kind::FormulaSource(source.into())),
                    }),
                })),
            }],
        ),
    ) else {
        panic!()
    };
    let r = saved.revision.unwrap();
    assert!(saved.diagnostics.unwrap().total > 0);
    let nodes = all_nodes(&mut f.session, &r);
    assert_eq!(
        nodes
            .iter()
            .find(|n| n.node_id == m.nodes["net"])
            .unwrap()
            .definition
            .as_ref()
            .unwrap()
            .kind,
        Some(a::node_definition::Kind::FormulaSource(source.into()))
    );
    let Out::Validation(v) = call(
        &mut f.session,
        Op::ValidateWorking(a::ValidateWorking {
            revision: Some(r.clone()),
        }),
    ) else {
        panic!()
    };
    assert!(!v.valid);
    let Out::Diagnostics(d) = call(
        &mut f.session,
        Op::ReadDiagnostics(a::ReadDiagnostics {
            service_instance_id: f.instance.clone(),
            working_revision: Some(r.clone()),
            page: None,
        }),
    ) else {
        panic!()
    };
    assert_eq!(d.diagnostics[0].code, a::DiagnosticCode::Parse as i32);
    assert_eq!(
        d.diagnostics[0].location.as_ref().unwrap().node_id,
        m.nodes["net"]
    );
    assert_eq!(
        error(call(
            &mut f.session,
            Op::EvaluateWorking(a::EvaluateWorking {
                revision: Some(r.clone()),
                inputs: vec![],
                output_node_ids: vec![m.nodes["independent"].clone()]
            })
        )),
        a::AuthoringErrorCode::ValidationFailed
    );
    assert_eq!(
        error(publish(&mut f.session, &r, "bad-publish")),
        a::AuthoringErrorCode::ValidationFailed
    );
}

#[test]
fn evaluation_uses_captured_inputs_and_isolates_arithmetic_errors() {
    let mut f = ordinary();
    let m = seed(&mut f);
    let operation = |inputs| {
        Op::EvaluateWorking(a::EvaluateWorking {
            revision: Some(m.revision.clone()),
            inputs,
            output_node_ids: vec![
                m.nodes["net"].clone(),
                m.nodes["margin"].clone(),
                m.nodes["independent"].clone(),
                m.nodes["dependent"].clone(),
            ],
        })
    };
    let Out::Evaluation(v) = call(&mut f.session, operation(vec![])) else {
        panic!()
    };
    assert_eq!(
        v.outputs
            .iter()
            .find(|o| o.node_id == m.nodes["net"])
            .unwrap()
            .outcome,
        Some(a::node_result::Outcome::Value(number("75")))
    );
    assert_eq!(
        v.effective_inputs
            .iter()
            .find(|i| i.node_id == m.nodes["costs"])
            .unwrap()
            .value,
        Some(number("25"))
    );
    let Out::Evaluation(v) = call(
        &mut f.session,
        operation(vec![a::Override {
            node_id: m.nodes["revenue"].clone(),
            value: Some(number("0")),
        }]),
    ) else {
        panic!()
    };
    assert!(
        matches!(&v.outputs.iter().find(|o|o.node_id==m.nodes["margin"]).unwrap().outcome,Some(a::node_result::Outcome::Error(e))if e.code==p::CalculationErrorCode::DivisionByZero as i32)
    );
    assert_eq!(
        v.outputs
            .iter()
            .find(|o| o.node_id == m.nodes["independent"])
            .unwrap()
            .outcome,
        Some(a::node_result::Outcome::Value(number("26")))
    );
    assert!(
        matches!(&v.outputs.iter().find(|o|o.node_id==m.nodes["dependent"]).unwrap().outcome,Some(a::node_result::Outcome::Error(e))if e.code==p::CalculationErrorCode::DependencyFailed as i32)
    );
    assert_eq!(
        v.effective_inputs
            .iter()
            .find(|i| i.node_id == m.nodes["revenue"])
            .unwrap()
            .source,
        a::WorkingInputSource::Submitted as i32
    );
}

#[test]
fn serialized_evaluation_releases_edit_lock_and_keeps_old_revision() {
    let mut f = ordinary();
    let m = seed(&mut f);
    f.controls.eval_calls.store(0, Ordering::SeqCst);
    *f.controls.edit_on_eval.lock().unwrap() = Some(m::ApplyEdits {
        instance: f.instance.parse().unwrap(),
        model: m.revision.working_model_id.parse().unwrap(),
        expected: native_revision(&m.revision),
        operation_id: "edit-during-eval".into(),
        edits: vec![m::Edit::SetNode {
            node: m::NodeRef::Existing(m.nodes["costs"].parse().unwrap()),
            definition: m::NodeDefinition::Input("50".parse().unwrap()),
        }],
    });
    let Out::Evaluation(v) = call(
        &mut f.session,
        Op::EvaluateWorking(a::EvaluateWorking {
            revision: Some(m.revision.clone()),
            inputs: vec![],
            output_node_ids: vec![m.nodes["net"].clone()],
        }),
    ) else {
        panic!()
    };
    assert!(f.controls.off_lock.load(Ordering::SeqCst));
    assert_eq!(v.provenance.unwrap().working, Some(m.revision.clone()));
    assert_eq!(
        v.outputs[0].outcome,
        Some(a::node_result::Outcome::Value(number("75")))
    );
    assert_eq!(
        f.service
            .lock()
            .unwrap()
            .inspect(
                f.instance.parse().unwrap(),
                "alice",
                m.revision.working_model_id.parse().unwrap()
            )
            .unwrap()
            .revision
            .sequence,
        m.revision.sequence + 1
    );
}

fn invoke_public(
    service: &Arc<Mutex<m::WorkingModelService>>,
    model: &p::ModelRef,
) -> p::InvocationResult {
    let service = service.lock().unwrap();
    let host = service.published_host();
    let mut s = pivot_host::Session::new(
        pivot_host::Limits::default(),
        Arc::new(pivot_host::AllowAll),
    )
    .unwrap();
    let send = |session: &mut pivot_host::Session, op| {
        let r = p::Request {
            request_id: "legacy".into(),
            version: Some(p::Version { major: 1, minor: 0 }),
            operation: Some(op),
        };
        p::Response::decode(host.dispatch(session, &r.encode_to_vec()).as_slice())
            .unwrap()
            .outcome
            .unwrap()
    };
    send(
        &mut s,
        p::request::Operation::Handshake(p::Handshake {
            supported_versions: vec![p::Version { major: 1, minor: 0 }],
            ..Default::default()
        }),
    );
    let p::response::Outcome::Description(d) = send(
        &mut s,
        p::request::Operation::DescribeModel(p::DescribeModel {
            model: Some(model.clone()),
        }),
    ) else {
        panic!()
    };
    let sales = d.inputs.iter().find(|p| p.name == "Sales").unwrap();
    let profit = d.outputs.iter().find(|p| p.name == "Profit").unwrap();
    let p::response::Outcome::Invocation(v) = send(
        &mut s,
        p::request::Operation::InvokeModel(p::InvokeModel {
            model: Some(model.clone()),
            inputs: vec![p::InputBinding {
                port_id: sales.port_id.clone(),
                value: Some(number("100")),
                source: 0,
            }],
            output_port_ids: vec![profit.port_id.clone()],
        }),
    ) else {
        panic!()
    };
    v
}
#[test]
fn publication_replay_and_v1_invocation_keep_immutable_public_contract() {
    let mut f = ordinary();
    let m = seed(&mut f);
    let Out::Published(first) = publish(&mut f.session, &m.revision, "publish") else {
        panic!()
    };
    let first_model = first.published.as_ref().unwrap();
    assert_eq!(
        invoke_public(&f.service, first_model).outputs[0].outcome,
        Some(p::output_result::Outcome::Value(number("60")))
    );
    let Out::Edited(next) = call(
        &mut f.session,
        edit(
            &m.revision,
            "new-formula",
            vec![a::Edit {
                kind: Some(a::edit::Kind::SetNode(a::SetNode {
                    node: Some(node(&m.nodes["net"])),
                    definition: Some(a::NodeDefinition {
                        kind: Some(a::node_definition::Kind::FormulaSource(
                            "Revenue - Costs + 10".into(),
                        )),
                    }),
                })),
            }],
        ),
    ) else {
        panic!()
    };
    let r = next.revision.unwrap();
    assert_eq!(
        publish(&mut f.session, &m.revision, "publish"),
        Out::Published(first.clone())
    );
    let Out::Published(second) = publish(&mut f.session, &r, "publish2") else {
        panic!()
    };
    assert_ne!(first.published, second.published);
    assert_eq!(
        invoke_public(&f.service, first_model).outputs[0].outcome,
        Some(p::output_result::Outcome::Value(number("60")))
    );
    assert_eq!(
        invoke_public(&f.service, second.published.as_ref().unwrap()).outputs[0].outcome,
        Some(p::output_result::Outcome::Value(number("70")))
    );
    assert_eq!(
        error(publish(&mut f.session, &r, "publish")),
        a::AuthoringErrorCode::OperationIdConflict
    );
}

#[test]
fn lost_responses_reconcile_creation_edits_and_publication_after_reconnect() {
    let mut f = ordinary();
    let created = create(&mut f.session, &f.instance, "lost-create");
    let r = created.revision.as_ref().unwrap();
    let operation = edit(r, "lost-edit", vec![input("n", "N", "1")]);
    let Out::Edited(edited) = call(&mut f.session, operation.clone()) else {
        panic!()
    };
    let new_r = edited.revision.as_ref().unwrap();
    let Out::Published(published) = publish(&mut f.session, new_r, "lost-publish") else {
        panic!()
    };
    f.session.close();
    let mut session = f.adapter.open_session("alice").unwrap();
    hello(&mut session);
    assert_eq!(create(&mut session, &f.instance, "lost-create"), created);
    assert_eq!(call(&mut session, operation), Out::Edited(edited.clone()));
    assert_eq!(
        publish(&mut session, new_r, "lost-publish"),
        Out::Published(published.clone())
    );
    let Out::OperationOutcome(v) = call(
        &mut session,
        Op::GetOperationOutcome(a::GetOperationOutcome {
            service_instance_id: f.instance.clone(),
            operation_id: "lost-edit".into(),
        }),
    ) else {
        panic!()
    };
    assert_eq!(
        v.outcome,
        Some(a::operation_outcome::Outcome::Edited(edited))
    );
    let Out::OperationOutcome(v) = call(
        &mut session,
        Op::GetOperationOutcome(a::GetOperationOutcome {
            service_instance_id: f.instance.clone(),
            operation_id: "lost-publish".into(),
        }),
    ) else {
        panic!()
    };
    assert_eq!(
        v.outcome,
        Some(a::operation_outcome::Outcome::Published(published))
    );
}

#[test]
fn cursor_binding_tampering_expiry_and_revision_changes_are_enforced() {
    let mut f = ordinary();
    let m = seed(&mut f);
    let Out::Draft(first) = call(
        &mut f.session,
        read(&m.revision, a::DraftSection::Nodes, ""),
    ) else {
        panic!()
    };
    let token = first.continuation_token;
    assert!(!token.is_empty());
    let mut tampered = token.clone();
    tampered.replace_range(0..1, if token.starts_with('0') { "1" } else { "0" });
    assert_eq!(
        error(call(
            &mut f.session,
            read(&m.revision, a::DraftSection::Nodes, &tampered)
        )),
        a::AuthoringErrorCode::InvalidArgument
    );
    let mut other = f.adapter.open_session("bob").unwrap();
    hello(&mut other);
    assert_eq!(
        error(call(
            &mut other,
            read(&m.revision, a::DraftSection::Nodes, &token)
        )),
        a::AuthoringErrorCode::InvalidArgument
    );
    assert_eq!(
        error(call(
            &mut f.session,
            read(&m.revision, a::DraftSection::Ports, &token)
        )),
        a::AuthoringErrorCode::InvalidArgument
    );
    f.clock.0.store(50, Ordering::SeqCst);
    let Out::Draft(second) = call(
        &mut f.session,
        read(&m.revision, a::DraftSection::Nodes, &token),
    ) else {
        panic!()
    };
    f.clock.0.store(61, Ordering::SeqCst);
    assert_eq!(
        error(call(
            &mut f.session,
            read(
                &m.revision,
                a::DraftSection::Nodes,
                &second.continuation_token
            )
        )),
        a::AuthoringErrorCode::ViewExpired
    );
    f.clock.0.store(0, Ordering::SeqCst);
    call(
        &mut f.session,
        edit(&m.revision, "advance", vec![input("x", "X", "1")]),
    );
    assert_eq!(
        error(call(
            &mut f.session,
            read(&m.revision, a::DraftSection::Nodes, &token)
        )),
        a::AuthoringErrorCode::ViewExpired
    );
}

#[test]
fn list_and_diagnostic_pages_are_complete_and_byte_bounded() {
    let mut f = ordinary();
    for n in 0..3 {
        create(&mut f.session, &f.instance, &format!("create-{n}"));
    }
    let mut token = String::new();
    let mut ids = Vec::new();
    loop {
        let Out::WorkingModels(v) = call(
            &mut f.session,
            Op::ListWorking(a::ListWorking {
                service_instance_id: f.instance.clone(),
                page: Some(a::Page {
                    page_size: 1,
                    continuation_token: token,
                }),
            }),
        ) else {
            panic!()
        };
        ids.extend(v.models.iter().map(|m| m.working_model_id.clone()));
        token = v.continuation_token;
        if token.is_empty() {
            break;
        }
    }
    assert_eq!(ids.len(), 3);
    let mut sorted = ids.clone();
    sorted.sort();
    sorted.dedup();
    assert_eq!(ids, sorted);
    let m = seed(&mut f);
    let Out::Edited(v) = call(
        &mut f.session,
        edit(
            &m.revision,
            "bad",
            vec![formula("a", "BadA", "-"), formula("b", "BadB", "")],
        ),
    ) else {
        panic!()
    };
    let r = v.revision.unwrap();
    let mut token = String::new();
    let mut count = 0;
    loop {
        let Out::Diagnostics(v) = call(
            &mut f.session,
            Op::ReadDiagnostics(a::ReadDiagnostics {
                service_instance_id: f.instance.clone(),
                working_revision: Some(r.clone()),
                page: Some(a::Page {
                    page_size: 1,
                    continuation_token: token,
                }),
            }),
        ) else {
            panic!()
        };
        assert_eq!(v.summary.as_ref().unwrap().total, 2);
        count += v.diagnostics.len();
        token = v.continuation_token;
        if token.is_empty() {
            break;
        }
    }
    assert_eq!(count, 2);
}

#[test]
fn authorization_is_independent_and_rechecked_on_page_and_mutation_delivery() {
    let mut f = ordinary();
    f.controls.deny_info.store(true, Ordering::SeqCst);
    assert_eq!(
        error(call(
            &mut f.session,
            Op::GetServiceInfo(a::GetServiceInfo {})
        )),
        a::AuthoringErrorCode::Unauthorized
    );
    f.controls.deny_info.store(false, Ordering::SeqCst);
    f.controls.info_calls.store(0, Ordering::SeqCst);
    f.controls.deny_info_at.store(2, Ordering::SeqCst);
    assert_eq!(
        error(call(
            &mut f.session,
            Op::GetServiceInfo(a::GetServiceInfo {})
        )),
        a::AuthoringErrorCode::Unauthorized
    );
    f.controls.deny_info_at.store(0, Ordering::SeqCst);
    let m = seed(&mut f);
    let Out::Draft(page) = call(
        &mut f.session,
        read(&m.revision, a::DraftSection::Nodes, ""),
    ) else {
        panic!()
    };
    f.controls.deny_inspect.store(true, Ordering::SeqCst);
    assert_eq!(
        error(call(
            &mut f.session,
            read(
                &m.revision,
                a::DraftSection::Nodes,
                &page.continuation_token
            )
        )),
        a::AuthoringErrorCode::Unauthorized
    );
    assert!(matches!(
        call(
            &mut f.session,
            Op::ReadDiagnostics(a::ReadDiagnostics {
                service_instance_id: f.instance.clone(),
                working_revision: Some(m.revision.clone()),
                page: None
            })
        ),
        Out::Diagnostics(_)
    ));
    f.controls.deny_inspect.store(false, Ordering::SeqCst);
    f.controls.edit_calls.store(0, Ordering::SeqCst);
    f.controls.deny_edit_at.store(2, Ordering::SeqCst);
    let operation = edit(&m.revision, "revoked", vec![input("x", "X", "1")]);
    assert_eq!(
        error(call(&mut f.session, operation.clone())),
        a::AuthoringErrorCode::Unauthorized
    );
    assert_eq!(all_nodes(&mut f.session, &m.revision).len(), 6);
    f.controls.deny_edit_at.store(0, Ordering::SeqCst);
    assert_eq!(
        error(call(&mut f.session, operation)),
        a::AuthoringErrorCode::Unauthorized
    );
    f.controls.edit_calls.store(0, Ordering::SeqCst);
    f.controls.deny_edit_at.store(3, Ordering::SeqCst);
    assert_eq!(
        error(call(
            &mut f.session,
            edit(
                &m.revision,
                "committed-then-revoked",
                vec![input("y", "Y", "1")]
            )
        )),
        a::AuthoringErrorCode::Unauthorized
    );
    f.controls.deny_edit_at.store(0, Ordering::SeqCst);
    let Out::OperationOutcome(v) = call(
        &mut f.session,
        Op::GetOperationOutcome(a::GetOperationOutcome {
            service_instance_id: f.instance.clone(),
            operation_id: "committed-then-revoked".into(),
        }),
    ) else {
        panic!()
    };
    assert!(matches!(
        v.outcome,
        Some(a::operation_outcome::Outcome::Edited(_))
    ));
}

#[test]
fn attachment_and_session_quotas_recover_on_close_or_drop() {
    let mut f = fixture(
        m::Limits {
            attachments: 1,
            ..Default::default()
        },
        AdapterLimits {
            attachment_ids_per_session: 1,
            sessions: 2,
            ..Default::default()
        },
    );
    let c = create(&mut f.session, &f.instance, "create");
    let open = Op::OpenWorking(a::OpenWorking {
        service_instance_id: f.instance.clone(),
        working_model_id: c.working_model_id.clone(),
    });
    assert!(matches!(call(&mut f.session, open.clone()), Out::Opened(_)));
    let mut second = f.adapter.open_session("alice").unwrap();
    hello(&mut second);
    assert!(f.adapter.open_session("alice").is_err());
    assert_eq!(
        error(call(&mut second, open.clone())),
        a::AuthoringErrorCode::ResourceExhausted
    );
    f.controls.deny_close.store(true, Ordering::SeqCst);
    f.session.close();
    assert!(matches!(call(&mut second, open.clone()), Out::Opened(_)));
    drop(second);
    f.controls.deny_close.store(false, Ordering::SeqCst);
    let mut third = f.adapter.open_session("alice").unwrap();
    hello(&mut third);
    let Out::Opened(v) = call(&mut third, open.clone()) else {
        panic!()
    };
    call(
        &mut third,
        Op::CloseAttachment(a::CloseAttachment {
            service_instance_id: f.instance.clone(),
            attachment_id: v.attachment_id,
        }),
    );
    assert_eq!(
        error(call(&mut third, open)),
        a::AuthoringErrorCode::ResourceExhausted
    );
}

#[test]
fn competing_serialized_writers_cannot_both_commit_same_head() {
    let mut f = ordinary();
    let m = seed(&mut f);
    let mut a = f.adapter.open_session("alice").unwrap();
    let mut b = f.adapter.open_session("alice").unwrap();
    hello(&mut a);
    hello(&mut b);
    let first = edit(&m.revision, "writer1", vec![input("a", "A", "1")]);
    let second = edit(&m.revision, "writer2", vec![input("b", "B", "2")]);
    let a = std::thread::spawn(move || call(&mut a, first));
    let b = std::thread::spawn(move || call(&mut b, second));
    let results = [a.join().unwrap(), b.join().unwrap()];
    assert_eq!(
        results
            .iter()
            .filter(|r| matches!(r, Out::Edited(_)))
            .count(),
        1
    );
    assert_eq!(results.iter().filter(|r|matches!(r,Out::Error(e)if e.code==a::AuthoringErrorCode::RevisionConflict as i32)).count(),1);
}

#[test]
fn unknown_duplicate_ids_and_incompatible_values_fail_without_mutation() {
    let mut f = ordinary();
    let m = seed(&mut f);
    let nil = "00000000-0000-0000-0000-000000000000";
    assert_eq!(
        error(call(
            &mut f.session,
            Op::EvaluateWorking(a::EvaluateWorking {
                revision: Some(m.revision.clone()),
                inputs: vec![],
                output_node_ids: vec![nil.into()]
            })
        )),
        a::AuthoringErrorCode::InvalidReference
    );
    let request = envelope(Op::EvaluateWorking(a::EvaluateWorking {
        revision: Some(m.revision.clone()),
        inputs: vec![],
        output_node_ids: vec![m.nodes["net"].clone(); 2],
    }));
    let p::response::Outcome::Authoring(v) =
        p::Response::decode(f.session.dispatch(&request.encode_to_vec()).as_slice())
            .unwrap()
            .outcome
            .unwrap()
    else {
        panic!()
    };
    assert_eq!(
        error(v.outcome.unwrap()),
        a::AuthoringErrorCode::InvalidArgument
    );
    let request = envelope(edit(
        &m.revision,
        "float",
        vec![add(
            "bad",
            "Bad",
            a::node_definition::Kind::Input(p::Value {
                kind: Some(p::value::Kind::Float64(1.0)),
            }),
        )],
    ));
    let p::response::Outcome::Authoring(v) =
        p::Response::decode(f.session.dispatch(&request.encode_to_vec()).as_slice())
            .unwrap()
            .outcome
            .unwrap()
    else {
        panic!()
    };
    assert_eq!(
        error(v.outcome.unwrap()),
        a::AuthoringErrorCode::UnsupportedValueType
    );
    assert_eq!(all_nodes(&mut f.session, &m.revision).len(), 6);
}

#[test]
fn oversized_page_entry_and_calculation_result_are_not_truncated() {
    let mut f = ordinary();
    let c = create(&mut f.session, &f.instance, "create");
    let source = format!("{}1", " ".repeat(600));
    let Out::Edited(v) = call(
        &mut f.session,
        edit(
            c.revision.as_ref().unwrap(),
            "large",
            vec![formula("f", "Formula", &source)],
        ),
    ) else {
        panic!()
    };
    let r = v.revision.unwrap();
    let low = AuthoringAdapter::new(
        f.service.clone(),
        AdapterLimits {
            preflight: foundation::Limits {
                bytes: 512,
                ..foundation::Limits::default()
            },
            response_bytes: 4096,
            page_target_bytes: 4096,
            ..Default::default()
        },
    )
    .unwrap();
    let mut s = low.open_session("alice").unwrap();
    hello(&mut s);
    assert_eq!(
        error(call(&mut s, read(&r, a::DraftSection::Nodes, ""))),
        a::AuthoringErrorCode::ResourceExhausted
    );
    assert_eq!(
        all_nodes(&mut f.session, &r)[0]
            .definition
            .as_ref()
            .unwrap()
            .kind,
        Some(a::node_definition::Kind::FormulaSource(source))
    );
    let many = (0..12)
        .map(|n| input(&format!("i{n}"), &format!("Input{n}"), "1"))
        .collect();
    let Out::Edited(edited) = call(&mut f.session, edit(&r, "inputs", many)) else {
        panic!()
    };
    assert_eq!(
        error(call(
            &mut s,
            Op::EvaluateWorking(a::EvaluateWorking {
                revision: edited.revision,
                inputs: vec![],
                output_node_ids: vec![v.nodes[0].node_id.clone()]
            })
        )),
        a::AuthoringErrorCode::ResourceExhausted
    );
}

#[test]
fn mutation_reply_capacity_is_checked_before_commit_and_raw_frames_are_bounded() {
    let mut f = fixture(
        m::Limits::default(),
        AdapterLimits {
            response_bytes: 512,
            page_target_bytes: 512,
            ..Default::default()
        },
    );
    let c = create(&mut f.session, &f.instance, "create");
    let r = c.revision.unwrap();
    let adds = (0..3)
        .map(|n| input(&format!("{}{n}", "t".repeat(63)), &format!("N{n}"), "0"))
        .collect();
    let request = envelope(edit(&r, "too-large-reply", adds));
    assert!(request.encoded_len() <= 512);
    let response =
        p::Response::decode(f.session.dispatch(&request.encode_to_vec()).as_slice()).unwrap();
    let Some(p::response::Outcome::Authoring(v)) = response.outcome else {
        panic!()
    };
    assert_eq!(
        error(v.outcome.unwrap()),
        a::AuthoringErrorCode::ResourceExhausted
    );
    assert!(
        f.service
            .lock()
            .unwrap()
            .read_draft("alice", native_revision(&r))
            .unwrap()
            .nodes
            .is_empty()
    );
    let bytes = f.session.dispatch(&vec![0; 513]);
    assert!(bytes.len() <= 512);
}

#[test]
fn requires_explicit_v11_handshake_and_never_enables_daemon_profile() {
    let f = ordinary();
    let mut s = f.adapter.open_session("alice").unwrap();
    let request = envelope(Op::GetServiceInfo(a::GetServiceInfo {}));
    assert!(
        matches!(p::Response::decode(s.dispatch(&request.encode_to_vec()).as_slice()).unwrap().outcome,Some(p::response::Outcome::Error(e))if e.code==p::ErrorCode::HandshakeRequired as i32)
    );
    let hello = p::Handshake {
        supported_versions: vec![p::Version { major: 1, minor: 1 }],
        required_capabilities: vec!["working-models-v1".into()],
        optional_capabilities: vec![],
    };
    assert!(pivot_protocol::validation::negotiate(&hello).is_err());
}

#[test]
fn policy_only_patch_preserves_an_existing_invalid_target() {
    let mut f = ordinary();
    let m = seed(&mut f);
    let Out::Edited(converted) = call(
        &mut f.session,
        edit(
            &m.revision,
            "convert",
            vec![a::Edit {
                kind: Some(a::edit::Kind::SetNode(a::SetNode {
                    node: Some(node(&m.nodes["costs"])),
                    definition: Some(a::NodeDefinition {
                        kind: Some(a::node_definition::Kind::FormulaSource("1".into())),
                    }),
                })),
            }],
        ),
    ) else {
        panic!()
    };
    let Out::Edited(patched) = call(
        &mut f.session,
        edit(
            converted.revision.as_ref().unwrap(),
            "policy-only",
            vec![a::Edit {
                kind: Some(a::edit::Kind::SetInputPort(a::SetInputPort {
                    port: Some(port(&m.ports["costs-port"])),
                    node: None,
                    requirement: Some(a::Requirement {
                        kind: Some(a::requirement::Kind::OptionalDefault(number("5"))),
                    }),
                })),
            }],
        ),
    ) else {
        panic!()
    };
    assert!(
        patched
            .diagnostics
            .as_ref()
            .unwrap()
            .counts
            .iter()
            .any(|c| c.code == a::DiagnosticCode::InvalidPortBinding as i32)
    );
    let Out::Draft(ports) = call(
        &mut f.session,
        Op::ReadDraft(a::ReadDraft {
            revision: patched.revision.clone(),
            section: a::DraftSection::Ports as i32,
            page: None,
        }),
    ) else {
        panic!()
    };
    assert!(
        matches!(&ports.ports.iter().find(|p|p.port_id==m.ports["costs-port"]).unwrap().kind,Some(a::raw_port::Kind::Input(p))if p.node_id==m.nodes["costs"])
    );
}

#[test]
fn original_wire_intent_preserves_signed_zero_even_if_execution_normalizes_values() {
    let mut f = ordinary();
    let c = create(&mut f.session, &f.instance, "create");
    let request = edit(
        c.revision.as_ref().unwrap(),
        "zero",
        vec![add(
            "n",
            "N",
            a::node_definition::Kind::Input(p::Value {
                kind: Some(p::value::Kind::Decimal(p::Decimal128 {
                    negative: true,
                    scale: 2,
                    ..Default::default()
                })),
            }),
        )],
    );
    let committed = call(&mut f.session, request.clone());
    assert!(matches!(committed, Out::Edited(_)));
    assert_eq!(call(&mut f.session, request.clone()), committed);
    let mut changed = request;
    if let Op::ApplyEdits(v) = &mut changed
        && let Some(a::edit::Kind::AddNode(v)) = &mut v.edits[0].kind
        && let Some(a::node_definition::Kind::Input(v)) = &mut v.definition.as_mut().unwrap().kind
        && let Some(p::value::Kind::Decimal(v)) = &mut v.kind
    {
        v.negative = false;
    }
    assert_eq!(
        error(call(&mut f.session, changed)),
        a::AuthoringErrorCode::OperationIdConflict
    );
}

#[test]
fn publication_and_evaluation_authorization_are_independent_with_commit_revocation() {
    let mut f = ordinary();
    let m = seed(&mut f);
    f.controls.deny_publish.store(true, Ordering::SeqCst);
    assert_eq!(
        error(publish(&mut f.session, &m.revision, "denied")),
        a::AuthoringErrorCode::Unauthorized
    );
    f.controls.deny_publish.store(false, Ordering::SeqCst);
    f.controls.publish_calls.store(0, Ordering::SeqCst);
    f.controls.deny_publish_at.store(3, Ordering::SeqCst);
    assert_eq!(
        error(publish(&mut f.session, &m.revision, "revoked-publish")),
        a::AuthoringErrorCode::Unauthorized
    );
    f.controls.deny_publish_at.store(0, Ordering::SeqCst);
    assert_eq!(
        error(publish(&mut f.session, &m.revision, "revoked-publish")),
        a::AuthoringErrorCode::Unauthorized
    );
    let Out::OperationOutcome(v) = call(
        &mut f.session,
        Op::GetOperationOutcome(a::GetOperationOutcome {
            service_instance_id: f.instance.clone(),
            operation_id: "revoked-publish".into(),
        }),
    ) else {
        panic!()
    };
    assert!(
        matches!(v.outcome,Some(a::operation_outcome::Outcome::Rejected(e))if e.code==a::AuthoringErrorCode::Unauthorized as i32)
    );
    f.controls.deny_evaluate.store(true, Ordering::SeqCst);
    assert_eq!(
        error(call(
            &mut f.session,
            Op::EvaluateWorking(a::EvaluateWorking {
                revision: Some(m.revision.clone()),
                inputs: vec![],
                output_node_ids: vec![m.nodes["net"].clone()]
            })
        )),
        a::AuthoringErrorCode::Unauthorized
    );
    assert!(matches!(
        call(
            &mut f.session,
            Op::ValidateWorking(a::ValidateWorking {
                revision: Some(m.revision)
            })
        ),
        Out::Validation(_)
    ));
}

#[test]
fn cursors_cannot_move_between_connections_or_page_options() {
    let mut f = ordinary();
    let m = seed(&mut f);
    let Out::Draft(v) = call(
        &mut f.session,
        read(&m.revision, a::DraftSection::Nodes, ""),
    ) else {
        panic!()
    };
    let mut other = f.adapter.open_session("alice").unwrap();
    hello(&mut other);
    assert_eq!(
        error(call(
            &mut other,
            read(&m.revision, a::DraftSection::Nodes, &v.continuation_token)
        )),
        a::AuthoringErrorCode::InvalidArgument
    );
    let mut changed = read(&m.revision, a::DraftSection::Nodes, &v.continuation_token);
    if let Op::ReadDraft(r) = &mut changed {
        r.page.as_mut().unwrap().page_size = 2;
    }
    assert_eq!(
        error(call(&mut f.session, changed)),
        a::AuthoringErrorCode::InvalidArgument
    );
}
