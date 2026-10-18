use pivot_core::{interface::InputRequirement, model::WorkingModel};
use pivot_host::*;
use pivot_protocol::{v1::*, validation};
use pivot_types::PivotDecimal;
use prost::Message;
use std::{collections::BTreeMap, sync::Arc};

fn d(s: &str) -> PivotDecimal {
    s.parse().unwrap()
}
fn val(lo: u32, scale: u32) -> Value {
    Value {
        kind: Some(value::Kind::Decimal(Decimal128 {
            lo,
            scale,
            ..Default::default()
        })),
    }
}
fn reference(revision: &str) -> ModelRef {
    ModelRef {
        model_id: "profit".into(),
        revision_id: revision.into(),
    }
}
fn fixture() -> (ModelHost, WorkingModel, pivot_core::model::NodeId) {
    let mut m = WorkingModel::new("Profit").unwrap();
    let revenue = m.add_input("GrossSales", d("100")).unwrap();
    let costs = m.add_input("COGS", d("999")).unwrap();
    let net = m.add_computed("Net", "GrossSales - COGS").unwrap();
    let margin = m.add_computed("Margin", "Net / GrossSales").unwrap();
    let dependent = m.add_computed("Dependent", "Margin + 1").unwrap();
    m.add_input_port("Revenue", revenue, InputRequirement::Required)
        .unwrap();
    m.add_input_port(
        "Costs",
        costs,
        InputRequirement::Optional {
            default: d("25").into(),
        },
    )
    .unwrap();
    m.add_output_port("Profit", net).unwrap();
    m.add_output_port("ProfitMargin", margin).unwrap();
    m.add_output_port("DependentOutput", dependent).unwrap();
    let mut host = ModelHost::new("host_test").unwrap();
    register(&mut host, &m, "r1");
    (host, m, net)
}
fn register(host: &mut ModelHost, m: &WorkingModel, revision: &str) {
    let ports = BTreeMap::from([
        ("Revenue".into(), "p_revenue".into()),
        ("Costs".into(), "p_costs".into()),
        ("Profit".into(), "p_profit".into()),
        ("ProfitMargin".into(), "p_margin".into()),
        ("DependentOutput".into(), "p_dependent".into()),
    ]);
    host.register(reference(revision), m.snapshot().unwrap(), ports)
        .unwrap();
}
fn session() -> Session {
    Session::new(Limits::default(), Arc::new(AllowAll)).unwrap()
}
fn send(host: &ModelHost, session: &mut Session, operation: request::Operation) -> Response {
    let request = Request {
        request_id: "req_test".into(),
        version: Some(Version { major: 1, minor: 0 }),
        operation: Some(operation),
    };
    let bytes = host.dispatch(session, &request.encode_to_vec());
    validation::decode_response(&bytes, &request).unwrap()
}
fn hello(host: &ModelHost, session: &mut Session) {
    let response = send(
        host,
        session,
        request::Operation::Handshake(Handshake {
            supported_versions: vec![Version { major: 1, minor: 0 }],
            required_capabilities: vec!["decimal96".into()],
            optional_capabilities: vec![],
        }),
    );
    assert!(matches!(
        response.outcome,
        Some(response::Outcome::Handshake(_))
    ));
}
fn call(revision: &str, amount: u32) -> InvokeModel {
    InvokeModel {
        model: Some(reference(revision)),
        inputs: vec![InputBinding {
            port_id: "p_revenue".into(),
            value: Some(val(amount, 0)),
            source: 0,
        }],
        output_port_ids: vec!["p_profit".into(), "p_margin".into(), "p_dependent".into()],
    }
}
fn invoke(host: &ModelHost, s: &mut Session, call: InvokeModel) -> InvocationResult {
    let response = send(host, s, request::Operation::InvokeModel(call));
    let Some(response::Outcome::Invocation(result)) = response.outcome else {
        panic!("{response:?}")
    };
    result
}
fn code(response: Response) -> ErrorCode {
    let Some(response::Outcome::Error(error)) = response.outcome else {
        panic!("{response:?}")
    };
    ErrorCode::try_from(error.code).unwrap()
}
#[test]
fn management_resources_remain_separate_from_published_invocations() {
    struct ManagementPolicy;
    impl management::Authorizer for ManagementPolicy {
        fn allowed(
            &self,
            _: &str,
            _: management::Action,
            _: Option<management::WorkingModelId>,
        ) -> bool {
            true
        }
    }
    let (host, _, _) = fixture();
    let mut session = session();
    hello(&host, &mut session);
    let before = invoke(&host, &mut session, call("r1", 100));
    let mut manager = management::WorkingModelService::new(
        management::Limits::default(),
        Box::new(ManagementPolicy),
    )
    .unwrap();
    let instance = manager.info().instance;
    let created = manager
        .create(
            "editor",
            management::CreateRequest {
                instance,
                operation_id: "create".into(),
                name: "Profit".into(),
            },
        )
        .unwrap();
    let attachment = manager.open(instance, "editor", created.id).unwrap();
    let edit_request = management::ApplyEdits {
        instance,
        model: created.id,
        expected: created.revision,
        operation_id: "edit".into(),
        edits: vec![management::Edit::AddNode {
            temporary: "new".into(),
            name: "GrossSales".into(),
            definition: management::NodeDefinition::Input(d("9999")),
        }],
    };
    let edited = manager.apply_edits("editor", edit_request.clone()).unwrap();
    assert_eq!(manager.apply_edits("editor", edit_request), Ok(edited));
    manager.close(instance, "editor", attachment.id).unwrap();
    let after = invoke(&host, &mut session, call("r1", 100));
    assert_eq!(before.outputs, after.outputs);
    assert_eq!(before.effective_inputs, after.effective_inputs);
    let list = send(
        &host,
        &mut session,
        request::Operation::ListModels(ListModels::default()),
    );
    assert!(
        matches!(list.outcome, Some(response::Outcome::Models(models)) if models.models.len() == 1)
    );
    assert_eq!(manager.list(instance, "editor").unwrap().len(), 1);
}
#[test]
fn discovery_introspection_and_invocation_provenance() {
    let (host, _, _) = fixture();
    let mut s = session();
    hello(&host, &mut s);
    let list = send(
        &host,
        &mut s,
        request::Operation::ListModels(ListModels::default()),
    );
    assert!(
        matches!(list.outcome, Some(response::Outcome::Models(models)) if models.models.len() == 1)
    );
    let response = send(
        &host,
        &mut s,
        request::Operation::DescribeModel(DescribeModel {
            model: Some(reference("r1")),
        }),
    );
    let Some(response::Outcome::Description(description)) = response.outcome else {
        panic!()
    };
    let call = call("r1", 100);
    let result = invoke(&host, &mut s, call.clone());
    validation::validate_result(&result, &call, &description).unwrap();
    assert_eq!(
        result.outputs[0].outcome,
        Some(output_result::Outcome::Value(val(75, 0)))
    );
    assert_eq!(
        result
            .effective_inputs
            .iter()
            .find(|b| b.port_id == "p_costs")
            .unwrap()
            .source,
        InputSource::Defaulted as i32
    );
    assert_eq!(
        result
            .effective_inputs
            .iter()
            .find(|b| b.port_id == "p_revenue")
            .unwrap()
            .source,
        InputSource::Submitted as i32
    );
    let mut supplied = call.clone();
    supplied.inputs.push(InputBinding {
        port_id: "p_costs".into(),
        value: Some(val(5000, 2)),
        source: 0,
    });
    let explicit = invoke(&host, &mut s, supplied);
    assert_eq!(
        explicit.outputs[0].outcome,
        Some(output_result::Outcome::Value(val(50, 0)))
    );
    assert!(
        explicit
            .effective_inputs
            .iter()
            .all(|b| b.source == InputSource::Submitted as i32)
    );
    assert_ne!(explicit.invocation_id, result.invocation_id);
}
#[test]
fn revisions_remain_immutable_after_edits() {
    let (mut host, mut m, net) = fixture();
    m.set_expression(net, "GrossSales - COGS * 2").unwrap();
    register(&mut host, &m, "r2");
    drop(m);
    let mut s = session();
    hello(&host, &mut s);
    assert_eq!(
        invoke(&host, &mut s, call("r1", 100)).outputs[0].outcome,
        Some(output_result::Outcome::Value(val(75, 0)))
    );
    assert_eq!(
        invoke(&host, &mut s, call("r2", 100)).outputs[0].outcome,
        Some(output_result::Outcome::Value(val(50, 0)))
    );
    assert_eq!(
        code(send(
            &host,
            &mut s,
            request::Operation::InvokeModel(call("missing", 100))
        )),
        ErrorCode::RevisionUnavailable
    );
    let mut missing = call("r1", 100);
    missing.model.as_mut().unwrap().model_id = "other".into();
    assert_eq!(
        code(send(
            &host,
            &mut s,
            request::Operation::InvokeModel(missing)
        )),
        ErrorCode::NotFound
    );
}
#[test]
fn invalid_bindings_and_arithmetic_errors_do_not_leak_between_calls() {
    let (host, _, _) = fixture();
    let mut s = session();
    hello(&host, &mut s);
    for (change, expected) in [
        (0, ErrorCode::MissingInput),
        (1, ErrorCode::UnknownPort),
        (2, ErrorCode::DuplicateBinding),
        (3, ErrorCode::InvalidArgument),
    ] {
        let mut c = call("r1", 100);
        match change {
            0 => c.inputs.clear(),
            1 => c.inputs[0].port_id = "unknown".into(),
            2 => c.inputs.push(c.inputs[0].clone()),
            _ => c.inputs[0].value = Some(val(1, 29)),
        }
        assert_eq!(
            code(send(&host, &mut s, request::Operation::InvokeModel(c))),
            expected
        );
    }
    let bad = invoke(&host, &mut s, call("r1", 0));
    assert!(
        matches!(&bad.outputs[1].outcome, Some(output_result::Outcome::Error(e)) if e.code == CalculationErrorCode::DivisionByZero as i32)
    );
    assert!(
        matches!(&bad.outputs[2].outcome, Some(output_result::Outcome::Error(e)) if e.code == CalculationErrorCode::DependencyFailed as i32)
    );
    assert!(matches!(
        &invoke(&host, &mut s, call("r1", 100)).outputs[1].outcome,
        Some(output_result::Outcome::Value(_))
    ));
}
#[test]
fn handshake_bootstrap_and_limits_are_enforced() {
    let (host, _, _) = fixture();
    let mut s = session();
    assert_eq!(
        code(send(
            &host,
            &mut s,
            request::Operation::ListModels(ListModels::default())
        )),
        ErrorCode::HandshakeRequired
    );
    assert!(!s.negotiated());
    let failure = send(
        &host,
        &mut s,
        request::Operation::Handshake(Handshake {
            supported_versions: vec![Version {
                major: 99,
                minor: 0,
            }],
            ..Default::default()
        }),
    );
    assert_eq!(code(failure), ErrorCode::UnsupportedVersion);
    assert!(!s.negotiated());
    let malformed = Response::decode(host.dispatch(&mut s, &[0xff]).as_slice()).unwrap();
    assert!(malformed.request_id.is_empty());
    assert_eq!(code(malformed), ErrorCode::InvalidArgument);
    hello(&host, &mut s);
    assert_eq!(
        code(send(
            &host,
            &mut s,
            request::Operation::GetInvocationResult(GetInvocationResult {
                invocation_id: "diagnostic_only".into()
            })
        )),
        ErrorCode::UnsupportedCapability
    );
    let mut small = Session::new(
        Limits {
            message_bytes: 256,
            inputs: 1,
            outputs: 1,
        },
        Arc::new(AllowAll),
    )
    .unwrap();
    let accepted = send(
        &host,
        &mut small,
        request::Operation::Handshake(Handshake {
            supported_versions: vec![Version { major: 1, minor: 0 }],
            ..Default::default()
        }),
    );
    assert!(
        matches!(accepted.outcome, Some(response::Outcome::Handshake(h)) if h.max_message_bytes == 256 && h.max_inputs == 1 && h.max_outputs == 1)
    );
    assert_eq!(
        code(send(
            &host,
            &mut small,
            request::Operation::InvokeModel(call("r1", 100))
        )),
        ErrorCode::ResourceExhausted
    );
    let oversized = Response::decode(host.dispatch(&mut small, &[0; 257]).as_slice()).unwrap();
    assert_eq!(code(oversized), ErrorCode::ResourceExhausted);
}
#[test]
fn concurrent_sessions_share_only_immutable_models() {
    let (host, _, _) = fixture();
    std::thread::scope(|scope| {
        let handles: Vec<_> = [100, 200, 300, 400]
            .into_iter()
            .map(|amount| {
                let h = &host;
                scope.spawn(move || {
                    let mut s = session();
                    hello(h, &mut s);
                    let result = invoke(h, &mut s, call("r1", amount));
                    assert_eq!(
                        result.outputs[0].outcome,
                        Some(output_result::Outcome::Value(val(amount - 25, 0)))
                    );
                    result.invocation_id
                })
            })
            .collect();
        let ids: std::collections::BTreeSet<_> =
            handles.into_iter().map(|h| h.join().unwrap()).collect();
        assert_eq!(ids.len(), 4);
    });
}
struct Deny;
impl Authorizer for Deny {
    fn allowed(&self, _: Action, _: &ModelRef) -> bool {
        false
    }
}
#[test]
fn authorization_boundary_hides_models_and_denies_before_lookup() {
    let (host, _, _) = fixture();
    let mut s = Session::new(Limits::default(), Arc::new(Deny)).unwrap();
    hello(&host, &mut s);
    assert!(
        matches!(send(&host, &mut s, request::Operation::ListModels(ListModels::default())).outcome, Some(response::Outcome::Models(list)) if list.models.is_empty())
    );
    assert_eq!(
        code(send(
            &host,
            &mut s,
            request::Operation::InvokeModel(call("r1", 100))
        )),
        ErrorCode::Unauthorized
    );
    assert_eq!(
        code(send(
            &host,
            &mut s,
            request::Operation::DescribeModel(DescribeModel {
                model: Some(reference("unknown"))
            })
        )),
        ErrorCode::Unauthorized
    );
}

#[test]
fn registry_rejects_replacement_and_bad_port_mappings() {
    let (mut host, m, _) = fixture();
    let ports: BTreeMap<_, _> = m
        .interface()
        .inputs()
        .map(|p| (p.name().to_owned(), format!("new_{}", p.name())))
        .chain(
            m.interface()
                .outputs()
                .map(|p| (p.name().to_owned(), format!("new_{}", p.name()))),
        )
        .collect();
    assert!(
        host.register(reference("r1"), m.snapshot().unwrap(), ports.clone())
            .is_err()
    );
    assert!(
        host.register(reference("r2"), m.snapshot().unwrap(), ports)
            .is_err()
    );
    assert!(
        host.register(reference("r3"), m.snapshot().unwrap(), BTreeMap::new())
            .is_err()
    );
    let mut s = session();
    hello(&host, &mut s);
    assert_eq!(
        invoke(&host, &mut s, call("r1", 100)).outputs[0].outcome,
        Some(output_result::Outcome::Value(val(75, 0)))
    );
}

#[test]
fn calculation_work_budget_rejects_before_execution() {
    let (mut host, _, _) = fixture();
    host.set_work_budget(1).unwrap();
    let mut s = session();
    hello(&host, &mut s);
    assert_eq!(
        code(send(
            &host,
            &mut s,
            request::Operation::InvokeModel(call("r1", 100))
        )),
        ErrorCode::ResourceExhausted
    );
    host.set_work_budget(100_000).unwrap();
    assert_eq!(
        invoke(&host, &mut s, call("r1", 100)).outputs[0].outcome,
        Some(output_result::Outcome::Value(val(75, 0)))
    );
}
