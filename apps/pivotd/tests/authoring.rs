use pivot_protocol::{authoring::v1 as a, v1 as p, validation::MAX_MESSAGE_BYTES};
use pivot_transport::{self as t, tokio};
use prost::Message;
use std::{
    io::{BufRead, BufReader, Write},
    process::{Child, Command, Stdio},
    time::Duration,
};
type Op = a::authoring_request::Operation;
type Out = a::authoring_response::Outcome;
fn handshake(minor: u32, capabilities: Vec<String>) -> p::Request {
    request(
        minor,
        p::request::Operation::Handshake(p::Handshake {
            supported_versions: vec![p::Version { major: 1, minor }],
            required_capabilities: capabilities,
            optional_capabilities: vec![],
        }),
    )
}
fn envelope_error(r: p::Response) -> p::ErrorCode {
    let Some(p::response::Outcome::Error(e)) = r.outcome else {
        panic!("{r:?}")
    };
    p::ErrorCode::try_from(e.code).unwrap()
}
struct Daemon {
    child: Child,
    endpoint: String,
}
impl Daemon {
    fn start(enabled: bool) -> Self {
        Self::with_args(enabled, &[])
    }
    fn with_args(enabled: bool, extra: &[&str]) -> Self {
        let id = uuid::Uuid::new_v4();
        #[cfg(windows)]
        let endpoint = format!(r"\\.\pipe\pivot-stage-b-{id}");
        #[cfg(unix)]
        let endpoint = format!("/tmp/pivot-stage-b-{id}/service.sock");
        let mut command = Command::new(env!("CARGO_BIN_EXE_pivotd"));
        command.args([
            "--demo",
            "--shutdown-stdin",
            "--endpoint",
            &endpoint,
            "--idle-timeout-secs",
            "3",
            "--frame-read-timeout-secs",
            "1",
            "--frame-write-timeout-secs",
            "1",
        ]);
        if enabled {
            command.arg("--authoring");
        }
        command.args(extra);
        let child = command
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::inherit())
            .spawn()
            .unwrap();
        let mut daemon = Self { child, endpoint };
        let stdout = daemon.child.stdout.take().unwrap();
        let (tx, rx) = std::sync::mpsc::channel();
        std::thread::spawn(move || {
            for line in BufReader::new(stdout).lines().map_while(Result::ok) {
                if line.starts_with("READY ") {
                    let _ = tx.send(());
                }
            }
        });
        rx.recv_timeout(Duration::from_secs(10))
            .expect("daemon ready");
        daemon
    }
    fn stop(&mut self) {
        if let Some(mut stdin) = self.child.stdin.take() {
            let _ = stdin.write_all(b"x");
        }
        let deadline = std::time::Instant::now() + Duration::from_secs(5);
        loop {
            if self.child.try_wait().unwrap().is_some() {
                break;
            }
            if std::time::Instant::now() >= deadline {
                let _ = self.child.kill();
                self.child.wait().unwrap();
                panic!("daemon shutdown stalled");
            }
            std::thread::sleep(Duration::from_millis(10));
        }
    }
}
impl Drop for Daemon {
    fn drop(&mut self) {
        self.stop();
        #[cfg(unix)]
        {
            let _ = std::fs::remove_dir(std::path::Path::new(&self.endpoint).parent().unwrap());
        }
    }
}
fn request(minor: u32, op: p::request::Operation) -> p::Request {
    p::Request {
        request_id: uuid::Uuid::new_v4().to_string(),
        version: Some(p::Version { major: 1, minor }),
        operation: Some(op),
    }
}
async fn exchange(s: &mut t::Stream, r: p::Request) -> p::Response {
    tokio::time::timeout(Duration::from_secs(5), async {
        t::write_frame(s, &r.encode_to_vec(), MAX_MESSAGE_BYTES)
            .await
            .unwrap();
        let bytes = t::read_frame(s, MAX_MESSAGE_BYTES).await.unwrap().unwrap();
        let response = p::Response::decode(bytes.as_slice()).unwrap();
        assert_eq!(response.request_id, r.request_id);
        response
    })
    .await
    .expect("bounded IPC exchange")
}
async fn hello(endpoint: &str, minor: u32) -> t::Stream {
    let mut s = t::connect(endpoint).await.unwrap();
    let r = exchange(
        &mut s,
        request(
            minor,
            p::request::Operation::Handshake(p::Handshake {
                supported_versions: vec![p::Version { major: 1, minor }],
                required_capabilities: if minor == 1 {
                    vec!["working-models-v1".into()]
                } else {
                    vec![]
                },
                optional_capabilities: vec![],
            }),
        ),
    )
    .await;
    let Some(p::response::Outcome::Handshake(v)) = r.outcome else {
        panic!("{r:?}")
    };
    assert_eq!(v.selected_version, Some(p::Version { major: 1, minor }));
    s
}
fn authoring(op: Op) -> p::Request {
    request(
        1,
        p::request::Operation::Authoring(a::AuthoringRequest {
            operation: Some(op),
        }),
    )
}
async fn call(s: &mut t::Stream, op: Op) -> Out {
    let response = exchange(s, authoring(op)).await;
    let Some(p::response::Outcome::Authoring(v)) = response.outcome else {
        panic!("{response:?}")
    };
    v.outcome.unwrap()
}
async fn info(s: &mut t::Stream) -> a::ServiceInfo {
    let Out::ServiceInfo(v) = call(s, Op::GetServiceInfo(a::GetServiceInfo {})).await else {
        panic!()
    };
    v
}
fn context(r: &a::WorkingRevision, op: &str) -> a::MutationContext {
    a::MutationContext {
        service_instance_id: r.service_instance_id.clone(),
        working_model_id: r.working_model_id.clone(),
        expected_revision: Some(r.clone()),
        operation_id: op.into(),
    }
}
fn formula(temp: &str, name: &str, source: &str) -> a::Edit {
    a::Edit {
        kind: Some(a::edit::Kind::AddNode(a::AddNode {
            temporary_id: temp.into(),
            name: name.into(),
            definition: Some(a::NodeDefinition {
                kind: Some(a::node_definition::Kind::FormulaSource(source.into())),
            }),
        })),
    }
}
fn edits(r: &a::WorkingRevision, op: &str, edits: Vec<a::Edit>) -> Op {
    Op::ApplyEdits(a::ApplyEdits {
        mutation: Some(context(r, op)),
        edits,
    })
}
fn code(out: Out) -> a::AuthoringErrorCode {
    let Out::Error(v) = out else {
        panic!("{out:?}")
    };
    a::AuthoringErrorCode::try_from(v.code).unwrap()
}
async fn recover(s: &mut t::Stream, instance: &str, id: &str) -> a::operation_outcome::Outcome {
    let Out::OperationOutcome(v) = call(
        s,
        Op::GetOperationOutcome(a::GetOperationOutcome {
            service_instance_id: instance.into(),
            operation_id: id.into(),
        }),
    )
    .await
    else {
        panic!()
    };
    v.outcome.unwrap()
}
// Confirm commitment using a second connection while the mutation response
// remains unread, then disconnect the mutating pipe/socket.
async fn abandon(
    endpoint: &str,
    op: Op,
    observer: &mut t::Stream,
    instance: &str,
    id: &str,
) -> a::operation_outcome::Outcome {
    let mut s = hello(endpoint, 1).await;
    t::write_frame(&mut s, &authoring(op).encode_to_vec(), MAX_MESSAGE_BYTES)
        .await
        .unwrap();
    let outcome = wait_receipt(observer, instance, id).await;
    drop(s);
    outcome
}
async fn wait_receipt(
    s: &mut t::Stream,
    instance: &str,
    op: &str,
) -> a::operation_outcome::Outcome {
    for _ in 0..100 {
        let v = recover(s, instance, op).await;
        if !matches!(v, a::operation_outcome::Outcome::Unknown(_)) {
            return v;
        }
        tokio::time::sleep(Duration::from_millis(10)).await;
    }
    panic!("mutation not committed");
}

#[tokio::test]
async fn real_ipc_authoring_publication_and_lost_response_recovery() {
    let daemon = Daemon::start(true);
    let mut s = hello(&daemon.endpoint, 1).await;
    let service = info(&mut s).await;
    assert_eq!(service.durability, a::Durability::MemoryOnly as i32);
    let instance = &service.service_instance_id;
    let create = Op::CreateWorking(a::CreateWorking {
        service_instance_id: instance.clone(),
        operation_id: "create".into(),
        name: "Authored".into(),
    });
    let a::operation_outcome::Outcome::Created(created) =
        abandon(&daemon.endpoint, create.clone(), &mut s, instance, "create").await
    else {
        panic!()
    };
    drop(s);
    let mut s = hello(&daemon.endpoint, 1).await;
    assert_eq!(
        recover(&mut s, instance, "create").await,
        a::operation_outcome::Outcome::Created(created.clone())
    );
    assert_eq!(call(&mut s, create).await, Out::Created(created.clone()));
    let r = created.revision.unwrap();
    let Out::WorkingModels(list) = call(
        &mut s,
        Op::ListWorking(a::ListWorking {
            service_instance_id: instance.clone(),
            page: None,
        }),
    )
    .await
    else {
        panic!()
    };
    assert!(
        list.models
            .iter()
            .any(|m| m.working_model_id == r.working_model_id)
    );
    let edit = edits(
        &r,
        "seed",
        vec![
            formula("n", "Answer", "40 + 2"),
            a::Edit {
                kind: Some(a::edit::Kind::AddOutputPort(a::AddOutputPort {
                    temporary_id: "p".into(),
                    name: "Answer".into(),
                    node: Some(a::NodeReference {
                        kind: Some(a::node_reference::Kind::TemporaryId("n".into())),
                    }),
                })),
            },
        ],
    );
    let a::operation_outcome::Outcome::Edited(edited) =
        abandon(&daemon.endpoint, edit.clone(), &mut s, instance, "seed").await
    else {
        panic!()
    };
    drop(s);
    let mut s = hello(&daemon.endpoint, 1).await;
    assert_eq!(
        recover(&mut s, instance, "seed").await,
        a::operation_outcome::Outcome::Edited(edited.clone())
    );
    assert_eq!(call(&mut s, edit).await, Out::Edited(edited.clone()));
    let r = edited.revision.unwrap();
    assert_eq!(r.sequence, 1);
    let Out::Validation(v) = call(
        &mut s,
        Op::ValidateWorking(a::ValidateWorking {
            revision: Some(r.clone()),
        }),
    )
    .await
    else {
        panic!()
    };
    assert!(v.valid);
    let Out::Diagnostics(diagnostics) = call(
        &mut s,
        Op::ReadDiagnostics(a::ReadDiagnostics {
            service_instance_id: instance.clone(),
            working_revision: Some(r.clone()),
            page: None,
        }),
    )
    .await
    else {
        panic!()
    };
    assert_eq!(diagnostics.summary.unwrap().total, 0);
    let Out::Evaluation(v) = call(
        &mut s,
        Op::EvaluateWorking(a::EvaluateWorking {
            revision: Some(r.clone()),
            inputs: vec![],
            output_node_ids: vec![edited.nodes[0].node_id.clone()],
        }),
    )
    .await
    else {
        panic!()
    };
    assert_eq!(v.outputs.len(), 1);
    let publish = Op::PublishWorking(a::PublishWorking {
        mutation: Some(context(&r, "publish")),
    });
    let a::operation_outcome::Outcome::Published(published) = abandon(
        &daemon.endpoint,
        publish.clone(),
        &mut s,
        instance,
        "publish",
    )
    .await
    else {
        panic!()
    };
    drop(s);
    let mut s = hello(&daemon.endpoint, 1).await;
    assert_eq!(
        recover(&mut s, instance, "publish").await,
        a::operation_outcome::Outcome::Published(published.clone())
    );
    assert_eq!(
        call(&mut s, publish).await,
        Out::Published(published.clone())
    );
    let model = published.published.unwrap();
    for minor in [0, 1] {
        let mut legacy = hello(&daemon.endpoint, minor).await;
        let listed = exchange(
            &mut legacy,
            request(
                minor,
                p::request::Operation::ListModels(p::ListModels::default()),
            ),
        )
        .await;
        let Some(p::response::Outcome::Models(list)) = listed.outcome else {
            panic!()
        };
        assert!(list.models.iter().any(|m| m.model.as_ref() == Some(&model)));
        assert!(list.models.iter().any(|m| m.name == "RestaurantProfit"));
        let described = exchange(
            &mut legacy,
            request(
                minor,
                p::request::Operation::DescribeModel(p::DescribeModel {
                    model: Some(model.clone()),
                }),
            ),
        )
        .await;
        let Some(p::response::Outcome::Description(description)) = described.outcome else {
            panic!()
        };
        let invoked = exchange(
            &mut legacy,
            request(
                minor,
                p::request::Operation::InvokeModel(p::InvokeModel {
                    model: Some(model.clone()),
                    inputs: vec![],
                    output_port_ids: vec![description.outputs[0].port_id.clone()],
                }),
            ),
        )
        .await;
        let Some(p::response::Outcome::Invocation(v)) = invoked.outcome else {
            panic!()
        };
        let Some(p::output_result::Outcome::Value(value)) = &v.outputs[0].outcome else {
            panic!()
        };
        let Some(p::value::Kind::Decimal(value)) = &value.kind else {
            panic!()
        };
        assert_eq!(value.lo, 42);
        assert_eq!(invoked.version, Some(p::Version { major: 1, minor }));
    }
}

#[tokio::test]
async fn real_ipc_concurrent_edits_rollback_and_connection_ownership() {
    let mut daemon = Daemon::start(true);
    let mut one = hello(&daemon.endpoint, 1).await;
    let instance = info(&mut one).await.service_instance_id;
    let Out::Created(c) = call(
        &mut one,
        Op::CreateWorking(a::CreateWorking {
            service_instance_id: instance.clone(),
            operation_id: "create".into(),
            name: "Concurrent".into(),
        }),
    )
    .await
    else {
        panic!()
    };
    let r = c.revision.unwrap();
    let mut two = hello(&daemon.endpoint, 1).await;
    let (a, b) = tokio::join!(
        call(&mut one, edits(&r, "one", vec![formula("a", "A", "1")])),
        call(&mut two, edits(&r, "two", vec![formula("b", "B", "2")]))
    );
    let r = match (a, b) {
        (Out::Edited(v), e) | (e, Out::Edited(v)) => {
            assert_eq!(code(e), a::AuthoringErrorCode::RevisionConflict);
            v.revision.unwrap()
        }
        v => panic!("{v:?}"),
    };
    assert_eq!(
        code(
            call(
                &mut one,
                edits(
                    &r,
                    "rollback",
                    vec![formula("x", "New", "3"), formula("y", "New", "4")]
                )
            )
            .await
        ),
        a::AuthoringErrorCode::InvalidReference
    );
    let Out::Draft(page) = call(
        &mut one,
        Op::ReadDraft(a::ReadDraft {
            revision: Some(r.clone()),
            section: a::DraftSection::Nodes as i32,
            page: None,
        }),
    )
    .await
    else {
        panic!()
    };
    assert_eq!(page.nodes.len(), 1);
    let Out::Opened(opened) = call(
        &mut one,
        Op::OpenWorking(a::OpenWorking {
            service_instance_id: instance.clone(),
            working_model_id: c.working_model_id.clone(),
        }),
    )
    .await
    else {
        panic!()
    };
    assert_eq!(
        code(
            call(
                &mut two,
                Op::CloseAttachment(a::CloseAttachment {
                    service_instance_id: instance.clone(),
                    attachment_id: opened.attachment_id.clone()
                })
            )
            .await
        ),
        a::AuthoringErrorCode::NotFound
    );
    for _ in 0..2 {
        assert!(matches!(
            call(
                &mut one,
                Op::CloseAttachment(a::CloseAttachment {
                    service_instance_id: instance.clone(),
                    attachment_id: opened.attachment_id.clone(),
                })
            )
            .await,
            Out::Closed(_)
        ));
    }
    assert!(matches!(
        call(
            &mut one,
            Op::OpenWorking(a::OpenWorking {
                service_instance_id: instance.clone(),
                working_model_id: c.working_model_id,
            })
        )
        .await,
        Out::Opened(_)
    ));
    drop(one);
    // Disconnect cannot remove the resource or its receipt.
    assert!(matches!(
        recover(&mut two, &instance, "create").await,
        a::operation_outcome::Outcome::Created(_)
    ));
    // Active connections must not prevent bounded daemon shutdown.
    daemon.stop();
}

#[tokio::test]
async fn disabled_authoring_negotiation_and_malicious_frames_leave_daemon_alive() {
    use tokio::io::AsyncWriteExt;
    for enabled in [false, true] {
        let daemon = Daemon::start(enabled);
        let mut s = t::connect(&daemon.endpoint).await.unwrap();
        let rejected = exchange(&mut s, handshake(1, vec!["unknown-capability".into()])).await;
        assert!(matches!(
            rejected.outcome,
            Some(p::response::Outcome::Error(_))
        ));
        if !enabled {
            let rejected = exchange(&mut s, handshake(1, vec!["working-models-v1".into()])).await;
            assert!(matches!(
                rejected.outcome,
                Some(p::response::Outcome::Error(_))
            ));
            let mut legacy = hello(&daemon.endpoint, 0).await;
            let mut r = authoring(Op::GetServiceInfo(a::GetServiceInfo {}));
            r.version = Some(p::Version { major: 1, minor: 0 });
            assert_eq!(
                envelope_error(exchange(&mut legacy, r).await),
                p::ErrorCode::UnsupportedCapability
            );
        }
        drop(s);
        for bytes in [vec![0xff, 0xff], vec![0], vec![0x0a, 0x80]] {
            let mut s = t::connect(&daemon.endpoint).await.unwrap();
            t::write_frame(&mut s, &bytes, MAX_MESSAGE_BYTES)
                .await
                .unwrap();
            let _ = tokio::time::timeout(
                Duration::from_secs(3),
                t::read_frame(&mut s, MAX_MESSAGE_BYTES),
            )
            .await
            .expect("malformed response timeout");
        }
        for size in [0, MAX_MESSAGE_BYTES as u32 + 1] {
            let mut s = t::connect(&daemon.endpoint).await.unwrap();
            s.write_all(&size.to_le_bytes()).await.unwrap();
            assert!(!matches!(
                tokio::time::timeout(
                    Duration::from_secs(3),
                    t::read_frame(&mut s, MAX_MESSAGE_BYTES)
                )
                .await
                .unwrap(),
                Ok(Some(_))
            ));
        }
        let mut legacy = hello(&daemon.endpoint, 0).await;
        assert!(matches!(
            exchange(
                &mut legacy,
                request(
                    0,
                    p::request::Operation::ListModels(p::ListModels::default())
                )
            )
            .await
            .outcome,
            Some(p::response::Outcome::Models(_))
        ));
        if enabled {
            let mut s = hello(&daemon.endpoint, 1).await;
            // A mutation with unknown fields is rejected before generated decode
            // or receipt admission, and the session remains usable.
            let mut raw = authoring(Op::GetServiceInfo(a::GetServiceInfo {})).encode_to_vec();
            raw.extend_from_slice(&[0xf8, 0x07, 1]);
            t::write_frame(&mut s, &raw, MAX_MESSAGE_BYTES)
                .await
                .unwrap();
            let response = t::read_frame(&mut s, MAX_MESSAGE_BYTES)
                .await
                .unwrap()
                .unwrap();
            assert!(
                p::Response::decode(response.as_slice())
                    .unwrap()
                    .request_id
                    .is_empty()
            );
            assert_eq!(
                info(&mut s).await.durability,
                a::Durability::MemoryOnly as i32
            );
        }
    }
}

#[tokio::test]
async fn configured_authoring_session_limit_releases_on_disconnect() {
    let daemon = Daemon::with_args(true, &["--authoring-sessions", "1"]);
    let one = hello(&daemon.endpoint, 1).await;
    let mut two = t::connect(&daemon.endpoint).await.unwrap();
    assert_eq!(
        envelope_error(exchange(&mut two, handshake(1, vec!["working-models-v1".into()])).await),
        p::ErrorCode::ResourceExhausted
    );
    // Invocation-only sessions do not consume authoring slots.
    let _legacy = hello(&daemon.endpoint, 0).await;
    drop(one);
    for _ in 0..100 {
        let r = exchange(&mut two, handshake(1, vec!["working-models-v1".into()])).await;
        if matches!(r.outcome, Some(p::response::Outcome::Handshake(_))) {
            return;
        }
        assert_eq!(envelope_error(r), p::ErrorCode::ResourceExhausted);
        tokio::time::sleep(Duration::from_millis(10)).await;
    }
    panic!("session quota leaked after disconnect");
}

// Exercise the production connection runner with an injectable revocable policy.
// No test-only control protocol or principal field is added to the daemon.
#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn disconnect_during_evaluation_and_publication_cleans_attachments_without_deadlock() {
    use pivot_host::management::{
        self as m,
        adapter::{AdapterLimits, AuthoringAdapter},
        connection::Connection,
    };
    use std::sync::{
        Arc, Condvar, Mutex,
        atomic::{AtomicBool, AtomicUsize, Ordering},
    };
    struct Policy {
        action: m::Action,
        calls: AtomicUsize,
        enabled: AtomicBool,
        revoked: AtomicBool,
        deny_inspect: AtomicBool,
        reached: tokio::sync::Notify,
        released: (Mutex<bool>, Condvar),
    }
    struct Authorization(Arc<Policy>);
    impl std::ops::Deref for Authorization {
        type Target = Policy;
        fn deref(&self) -> &Policy {
            &self.0
        }
    }
    impl m::Authorizer for Authorization {
        fn allowed(&self, _: &str, action: m::Action, _: Option<m::WorkingModelId>) -> bool {
            if action == m::Action::Inspect && self.deny_inspect.load(Ordering::SeqCst) {
                return false;
            }
            if action != self.action {
                return true;
            }
            if self.enabled.load(Ordering::SeqCst) {
                let target = if action == m::Action::Evaluate { 3 } else { 1 };
                if self.calls.fetch_add(1, Ordering::SeqCst) + 1 == target {
                    self.reached.notify_one();
                    let (lock, cv) = &self.released;
                    let result = cv
                        .wait_timeout_while(
                            lock.lock().unwrap(),
                            Duration::from_secs(5),
                            |released| !*released,
                        )
                        .unwrap();
                    assert!(*result.0, "test gate timed out");
                }
            }
            !self.revoked.load(Ordering::SeqCst)
        }
    }
    for (action, cancel) in [
        (m::Action::Evaluate, true),
        (m::Action::Publish, true),
        (m::Action::Evaluate, false),
        (m::Action::Publish, false),
    ] {
        let id = uuid::Uuid::new_v4();
        #[cfg(windows)]
        let endpoint = format!(r"\\.\pipe\pivot-cancel-{id}");
        #[cfg(unix)]
        let endpoint = format!("/tmp/pivot-cancel-{id}/service.sock");
        let mut listener = t::Listener::bind(&endpoint).unwrap();
        let policy = Arc::new(Policy {
            action,
            calls: AtomicUsize::new(0),
            enabled: AtomicBool::new(false),
            revoked: AtomicBool::new(false),
            deny_inspect: AtomicBool::new(false),
            reached: tokio::sync::Notify::new(),
            released: (Mutex::new(false), Condvar::new()),
        });
        let service = Arc::new(Mutex::new(
            m::WorkingModelService::new(
                m::Limits {
                    attachments: 1,
                    ..Default::default()
                },
                Box::new(Authorization(policy.clone())),
            )
            .unwrap(),
        ));
        let adapter = AuthoringAdapter::new(service.clone(), AdapterLimits::default()).unwrap();
        let server_service = service.clone();
        let (principal_tx, principal_rx) = tokio::sync::oneshot::channel();
        let server = tokio::spawn(async move {
            let stream = listener.accept().await.unwrap();
            let principal = t::verified_peer_principal(&stream).unwrap();
            principal_tx.send(principal.clone()).unwrap();
            let session = Connection::new(
                server_service,
                Some(adapter),
                principal,
                Arc::new(pivot_host::AllowAll),
            )
            .unwrap();
            pivotd::serve_connection(stream, session, t::FrameTimeouts::default()).await
        });
        let mut s = hello(&endpoint, 1).await;
        let principal = principal_rx.await.unwrap();
        let instance = info(&mut s).await.service_instance_id;
        let Out::Created(created) = call(
            &mut s,
            Op::CreateWorking(a::CreateWorking {
                service_instance_id: instance.clone(),
                operation_id: "create".into(),
                name: "Cancel".into(),
            }),
        )
        .await
        else {
            panic!()
        };
        let r = created.revision.unwrap();
        let Out::Edited(edited) = call(
            &mut s,
            edits(&r, "seed", vec![formula("n", "Answer", "42")]),
        )
        .await
        else {
            panic!()
        };
        let r = edited.revision.unwrap();
        policy.deny_inspect.store(true, Ordering::SeqCst);
        for model in [
            created.working_model_id.clone(),
            uuid::Uuid::new_v4().to_string(),
        ] {
            let mut hidden = r.clone();
            hidden.working_model_id = model;
            let Out::Error(e) = call(
                &mut s,
                Op::ReadDraft(a::ReadDraft {
                    revision: Some(hidden),
                    section: a::DraftSection::Nodes as i32,
                    page: None,
                }),
            )
            .await
            else {
                panic!()
            };
            assert_eq!(e.code, a::AuthoringErrorCode::Unauthorized as i32);
            assert!(e.current_revision.is_none());
            assert!(e.validation_diagnostics.is_none());
        }
        policy.deny_inspect.store(false, Ordering::SeqCst);
        assert!(matches!(
            call(
                &mut s,
                Op::OpenWorking(a::OpenWorking {
                    service_instance_id: instance.clone(),
                    working_model_id: created.working_model_id.clone()
                })
            )
            .await,
            Out::Opened(_)
        ));
        policy.enabled.store(true, Ordering::SeqCst);
        let operation = if action == m::Action::Evaluate {
            Op::EvaluateWorking(a::EvaluateWorking {
                revision: Some(r.clone()),
                inputs: vec![],
                output_node_ids: vec![edited.nodes[0].node_id.clone()],
            })
        } else {
            Op::PublishWorking(a::PublishWorking {
                mutation: Some(context(&r, "publish")),
            })
        };
        t::write_frame(
            &mut s,
            &authoring(operation).encode_to_vec(),
            MAX_MESSAGE_BYTES,
        )
        .await
        .unwrap();
        tokio::time::timeout(Duration::from_secs(3), policy.reached.notified())
            .await
            .expect("operation reached gate");
        // Cancel while the blocking job owns the authoring session. The job must
        // finish before Drop tries to acquire the management mutex for cleanup.
        if cancel {
            server.abort();
        }
        policy.revoked.store(true, Ordering::SeqCst);
        *policy.released.0.lock().unwrap() = true;
        policy.released.1.notify_all();
        if !cancel {
            let bytes = tokio::time::timeout(
                Duration::from_secs(3),
                t::read_frame(&mut s, MAX_MESSAGE_BYTES),
            )
            .await
            .unwrap()
            .unwrap()
            .unwrap();
            let Some(p::response::Outcome::Authoring(v)) =
                p::Response::decode(bytes.as_slice()).unwrap().outcome
            else {
                panic!()
            };
            assert_eq!(
                code(v.outcome.unwrap()),
                a::AuthoringErrorCode::Unauthorized
            );
        }
        drop(s);
        let _ = tokio::time::timeout(Duration::from_secs(3), server)
            .await
            .expect("connection cleanup timeout");
        let mut cleaned = false;
        for _ in 0..200 {
            if let Ok(mut guard) = service.try_lock() {
                match guard.open(
                    instance.parse().unwrap(),
                    &principal,
                    created.working_model_id.parse().unwrap(),
                ) {
                    Ok(_) => {
                        cleaned = true;
                        break;
                    }
                    Err(m::Error::AttachmentLimit) => (),
                    other => panic!("{other:?}"),
                }
            }
            tokio::time::sleep(Duration::from_millis(10)).await;
        }
        assert!(
            cleaned,
            "cancelled operation leaked attachment or deadlocked"
        );
        if action == m::Action::Publish {
            // Revocation before commit prevents publication; lookup itself has
            // independent Recover rights and reports only this principal's intent.
            assert!(!matches!(
                service
                    .lock()
                    .unwrap()
                    .operation_outcome(instance.parse().unwrap(), &principal, "publish")
                    .unwrap(),
                m::OperationOutcome::Published(_)
            ));
        }
        #[cfg(unix)]
        {
            let _ = std::fs::remove_dir(std::path::Path::new(&endpoint).parent().unwrap());
        }
    }
}
