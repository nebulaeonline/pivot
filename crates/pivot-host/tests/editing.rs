use pivot_host::management::*;
use pivot_types::PivotDecimal;
use std::sync::{
    Arc,
    atomic::{AtomicUsize, Ordering},
};

#[derive(Default)]
struct Controls {
    edit_calls: AtomicUsize,
    deny_edit_call: AtomicUsize,
}
struct Policy(Arc<Controls>);
impl Authorizer for Policy {
    fn allowed(&self, principal: &str, action: Action, _: Option<WorkingModelId>) -> bool {
        if principal == "denied" {
            return false;
        }
        if action == Action::Edit {
            let n = self.0.edit_calls.fetch_add(1, Ordering::SeqCst) + 1;
            return n != self.0.deny_edit_call.load(Ordering::SeqCst);
        }
        true
    }
}
fn fixture(limits: Limits, edits: EditLimits) -> (WorkingModelService, Created, Arc<Controls>) {
    let controls = Arc::new(Controls::default());
    let mut s =
        WorkingModelService::with_edit_limits(limits, edits, Box::new(Policy(controls.clone())))
            .unwrap();
    let created = s
        .create(
            "alice",
            CreateRequest {
                instance: s.info().instance,
                operation_id: "create".into(),
                name: "Budget".into(),
            },
        )
        .unwrap();
    (s, created, controls)
}
fn ordinary() -> (WorkingModelService, Created, Arc<Controls>) {
    fixture(Limits::default(), EditLimits::default())
}
fn input(temp: &str, name: &str, value: &str) -> Edit {
    Edit::AddNode {
        temporary: temp.into(),
        name: name.into(),
        definition: NodeDefinition::Input(value.parse().unwrap()),
    }
}
fn formula(temp: &str, name: &str, source: &str) -> Edit {
    Edit::AddNode {
        temporary: temp.into(),
        name: name.into(),
        definition: NodeDefinition::Formula(source.into()),
    }
}
fn temp(t: &str) -> NodeRef {
    NodeRef::Temporary(t.into())
}
fn existing(id: AuthoringNodeId) -> NodeRef {
    NodeRef::Existing(id)
}
fn req(revision: WorkingRevision, op: &str, edits: Vec<Edit>) -> ApplyEdits {
    ApplyEdits {
        instance: revision.instance,
        model: revision.model,
        expected: revision,
        operation_id: op.into(),
        edits,
    }
}
fn seed(s: &mut WorkingModelService, c: Created) -> EditResult {
    s.apply_edits(
        "alice",
        req(
            c.revision,
            "seed",
            vec![
                formula("net", "Net", "Revenue - Costs"),
                input("revenue", "Revenue", "100"),
                input("costs", "Costs", "25"),
            ],
        ),
    )
    .unwrap()
}

#[test]
fn atomic_forward_reference_batch_advances_public_revision_once() {
    let (mut s, c, _) = ordinary();
    let result = s
        .apply_edits(
            "alice",
            req(
                c.revision,
                "batch",
                vec![
                    formula("net", "Net", " Revenue - Costs "),
                    input("revenue", "Revenue", "100"),
                    input("costs", "Costs", "25"),
                    Edit::AddInputPort {
                        temporary: "in".into(),
                        name: "Sales".into(),
                        node: temp("revenue"),
                        requirement: Requirement::Required,
                    },
                    Edit::AddOutputPort {
                        temporary: "out".into(),
                        name: "Profit".into(),
                        node: temp("net"),
                    },
                ],
            ),
        )
        .unwrap();
    assert_eq!(result.revision.sequence, 1);
    assert_eq!(result.revision.instance, c.revision.instance);
    assert!(result.diagnostics.is_empty());
    assert_eq!(result.nodes.len(), 3);
    assert_eq!(result.ports.len(), 2);
    let view = s.read_draft("alice", result.revision).unwrap();
    let net = view
        .nodes
        .iter()
        .find(|n| n.id == result.nodes["net"])
        .unwrap();
    assert_eq!(
        net.definition,
        NodeDefinition::Formula(" Revenue - Costs ".into())
    );
    assert!(net.dependencies.contains(&result.nodes["revenue"]));
    assert!(net.dependencies.contains(&result.nodes["costs"]));
    let description = s.inspect(c.revision.instance, "alice", c.id).unwrap();
    assert_eq!(
        (
            description.node_count,
            description.input_count,
            description.output_count
        ),
        (3, 1, 1)
    );
    assert_eq!(
        s.read_draft("alice", c.revision),
        Err(Error::RevisionConflict)
    );
}

#[test]
fn competing_writers_conflict_and_identical_replay_precedes_revision_check() {
    let (mut s, c, _) = ordinary();
    let a = req(c.revision, "writer-a", vec![input("a", "A", "1")]);
    let b = req(c.revision, "writer-b", vec![input("b", "B", "2")]);
    let committed = s.apply_edits("alice", a.clone()).unwrap();
    assert_eq!(
        s.apply_edits("bob", b.clone()),
        Err(Error::RevisionConflict)
    );
    for _ in 0..10 {
        assert_eq!(s.apply_edits("alice", a.clone()), Ok(committed.clone()));
    }
    assert_eq!(
        s.operation_outcome(c.revision.instance, "alice", "writer-a"),
        Ok(OperationOutcome::Edited(committed.clone()))
    );
    assert_eq!(
        s.operation_outcome(c.revision.instance, "bob", "writer-b"),
        Ok(OperationOutcome::Rejected(Error::RevisionConflict))
    );
    let view = s.read_draft("alice", committed.revision).unwrap();
    assert_eq!(view.nodes.len(), 1);
    assert_eq!(view.nodes[0].name, "A");
}

#[test]
fn midway_failure_rolls_back_names_nodes_formulas_ports_and_revision() {
    let (mut s, c, _) = ordinary();
    let result = seed(&mut s, c);
    let before = s.read_draft("alice", result.revision).unwrap();
    let edits = vec![
        Edit::RenameNode {
            node: existing(result.nodes["revenue"]),
            name: "Sales".into(),
        },
        Edit::SetNode {
            node: existing(result.nodes["net"]),
            definition: NodeDefinition::Formula("Sales + Costs".into()),
        },
        input("new", "Extra", "7"),
        Edit::AddInputPort {
            temporary: "p".into(),
            name: "NewPort".into(),
            node: temp("new"),
            requirement: Requirement::Optional("3".parse().unwrap()),
        },
        input("duplicate", "Costs", "9"),
    ];
    let failed = req(result.revision, "rollback", edits);
    assert_eq!(
        s.apply_edits("alice", failed.clone()),
        Err(Error::InvalidEdit)
    );
    assert_eq!(s.read_draft("alice", result.revision).unwrap(), before);
    let next = s
        .apply_edits(
            "alice",
            req(result.revision, "after", vec![input("new", "Extra", "8")]),
        )
        .unwrap();
    assert_eq!(next.revision.sequence, 2);
    assert!(
        s.read_draft("alice", next.revision)
            .unwrap()
            .ports
            .is_empty()
    );
    assert_eq!(s.apply_edits("alice", failed), Err(Error::InvalidEdit));
}

#[test]
fn rename_keeps_bindings_and_delete_name_reuse_does_not_retarget_them() {
    let (mut s, c, _) = ordinary();
    let r = seed(&mut s, c);
    let revenue = r.nodes["revenue"];
    let net = r.nodes["net"];
    let renamed = s
        .apply_edits(
            "alice",
            req(
                r.revision,
                "rename",
                vec![Edit::RenameNode {
                    node: existing(revenue),
                    name: "Sales".into(),
                }],
            ),
        )
        .unwrap();
    let view = s.read_draft("alice", renamed.revision).unwrap();
    let formula_node = view.nodes.iter().find(|n| n.id == net).unwrap();
    assert!(formula_node.dependencies.contains(&revenue));
    assert_eq!(
        formula_node.definition,
        NodeDefinition::Formula("Revenue - Costs".into())
    );
    let replaced = s
        .apply_edits(
            "alice",
            req(
                renamed.revision,
                "reuse",
                vec![
                    Edit::RemoveNode(existing(revenue)),
                    input("replacement", "Revenue", "999"),
                ],
            ),
        )
        .unwrap();
    assert_ne!(replaced.nodes["replacement"], revenue);
    assert!(
        replaced
            .diagnostics
            .iter()
            .any(|d| d.node == Some(net) && d.kind == DiagnosticKind::MissingReference(revenue))
    );
    let fixed = s
        .apply_edits(
            "alice",
            req(
                replaced.revision,
                "repair",
                vec![Edit::SetNode {
                    node: existing(net),
                    definition: NodeDefinition::Formula("Revenue - Costs".into()),
                }],
            ),
        )
        .unwrap();
    assert!(fixed.diagnostics.is_empty());
    let view = s.read_draft("alice", fixed.revision).unwrap();
    assert!(
        view.nodes
            .iter()
            .find(|n| n.id == net)
            .unwrap()
            .dependencies
            .contains(&replaced.nodes["replacement"])
    );
}

#[test]
fn incomplete_source_is_retained_and_later_repaired() {
    let (mut s, c, _) = ordinary();
    let original = "  Revenue -  ";
    let r = s
        .apply_edits(
            "alice",
            req(
                c.revision,
                "draft",
                vec![input("a", "Revenue", "10"), formula("f", "Net", original)],
            ),
        )
        .unwrap();
    assert!(
        r.diagnostics
            .iter()
            .any(|d| matches!(d.kind, DiagnosticKind::Parse { .. }))
    );
    let view = s.read_draft("alice", r.revision).unwrap();
    assert_eq!(
        view.nodes
            .iter()
            .find(|n| n.id == r.nodes["f"])
            .unwrap()
            .definition,
        NodeDefinition::Formula(original.into())
    );
    let repaired = s
        .apply_edits(
            "alice",
            req(
                r.revision,
                "fix",
                vec![Edit::SetNode {
                    node: existing(r.nodes["f"]),
                    definition: NodeDefinition::Formula("Revenue - 1".into()),
                }],
            ),
        )
        .unwrap();
    assert!(repaired.diagnostics.is_empty());
}

#[test]
fn unresolved_drafts_do_not_implicitly_rebind_in_later_batches() {
    let (mut s, c, _) = ordinary();
    let r = s
        .apply_edits(
            "alice",
            req(
                c.revision,
                "draft",
                vec![formula("f", "Result", "Future + 1")],
            ),
        )
        .unwrap();
    assert!(
        r.diagnostics
            .iter()
            .any(|d| d.kind == DiagnosticKind::UnknownName("Future".into()))
    );
    let added = s
        .apply_edits(
            "alice",
            req(r.revision, "add", vec![input("new", "Future", "2")]),
        )
        .unwrap();
    assert!(
        added
            .diagnostics
            .iter()
            .any(|d| d.kind == DiagnosticKind::UnknownName("Future".into()))
    );
    let repaired = s
        .apply_edits(
            "alice",
            req(
                added.revision,
                "resubmit",
                vec![Edit::SetNode {
                    node: existing(r.nodes["f"]),
                    definition: NodeDefinition::Formula("Future + 1".into()),
                }],
            ),
        )
        .unwrap();
    assert!(repaired.diagnostics.is_empty());
}

#[test]
fn only_submitted_formulas_bind_against_final_directory() {
    let (mut s, c, _) = ordinary();
    let r = seed(&mut s, c);
    let updated = s
        .apply_edits(
            "alice",
            req(
                r.revision,
                "final-directory",
                vec![
                    formula("new", "Copy", "Sales"),
                    Edit::RenameNode {
                        node: existing(r.nodes["revenue"]),
                        name: "Sales".into(),
                    },
                ],
            ),
        )
        .unwrap();
    assert!(updated.diagnostics.is_empty());
    let view = s.read_draft("alice", updated.revision).unwrap();
    for name in ["Net", "Copy"] {
        assert!(
            view.nodes
                .iter()
                .find(|n| n.name == name)
                .unwrap()
                .dependencies
                .contains(&r.nodes["revenue"])
        );
    }
}

#[test]
fn port_updates_preserve_identity_and_repair_invalid_drafts() {
    let (mut s, c, _) = ordinary();
    let r = s
        .apply_edits(
            "alice",
            req(
                c.revision,
                "ports",
                vec![
                    input("a", "A", "1"),
                    input("b", "B", "2"),
                    Edit::AddInputPort {
                        temporary: "in".into(),
                        name: "Input".into(),
                        node: temp("a"),
                        requirement: Requirement::Required,
                    },
                    Edit::AddOutputPort {
                        temporary: "out".into(),
                        name: "Output".into(),
                        node: temp("a"),
                    },
                ],
            ),
        )
        .unwrap();
    let input_id = r.ports["in"];
    let output_id = r.ports["out"];
    let invalidated = s
        .apply_edits(
            "alice",
            req(
                r.revision,
                "invalidate",
                vec![Edit::SetNode {
                    node: existing(r.nodes["a"]),
                    definition: NodeDefinition::Formula("B + 1".into()),
                }],
            ),
        )
        .unwrap();
    assert!(
        invalidated
            .diagnostics
            .iter()
            .any(|d| d.port == Some(input_id) && d.kind == DiagnosticKind::InvalidPortBinding)
    );
    let updated = s
        .apply_edits(
            "alice",
            req(
                invalidated.revision,
                "update-ports",
                vec![
                    Edit::RenamePort {
                        port: PortRef::Existing(input_id),
                        name: "PublicInput".into(),
                    },
                    Edit::SetInputPort {
                        port: PortRef::Existing(input_id),
                        node: existing(r.nodes["b"]),
                        requirement: Requirement::Optional("7.50".parse().unwrap()),
                    },
                    Edit::SetOutputPort {
                        port: PortRef::Existing(output_id),
                        node: existing(r.nodes["b"]),
                    },
                ],
            ),
        )
        .unwrap();
    assert!(updated.diagnostics.is_empty());
    let view = s.read_draft("alice", updated.revision).unwrap();
    assert_eq!(
        view.ports.iter().find(|p| p.id == input_id).unwrap().name,
        "PublicInput"
    );
    assert_eq!(
        view.ports
            .iter()
            .find(|p| p.id == input_id)
            .unwrap()
            .definition,
        PortDefinition::Input {
            node: r.nodes["b"],
            requirement: Requirement::Optional("7.50".parse().unwrap())
        }
    );
    let removed = s
        .apply_edits(
            "alice",
            req(
                updated.revision,
                "remove",
                vec![Edit::RemoveNode(existing(r.nodes["b"]))],
            ),
        )
        .unwrap();
    assert_eq!(
        removed
            .diagnostics
            .iter()
            .filter(|d| d.kind == DiagnosticKind::InvalidPortBinding)
            .count(),
        2
    );
    let empty = s
        .apply_edits(
            "alice",
            req(
                removed.revision,
                "remove-ports",
                vec![
                    Edit::RemovePort(PortRef::Existing(input_id)),
                    Edit::RemovePort(PortRef::Existing(output_id)),
                ],
            ),
        )
        .unwrap();
    assert!(
        s.read_draft("alice", empty.revision)
            .unwrap()
            .ports
            .is_empty()
    );
}

#[test]
fn structural_errors_reject_whole_batches() {
    let (mut s, c, _) = ordinary();
    let r = seed(&mut s, c);
    let before = s.read_draft("alice", r.revision).unwrap();
    let bad_batches = vec![
        vec![Edit::RenameNode {
            node: existing(r.nodes["revenue"]),
            name: "Costs".into(),
        }],
        vec![Edit::RenameNode {
            node: existing(r.nodes["revenue"]),
            name: "bad name".into(),
        }],
        vec![Edit::RemoveNode(temp("missing"))],
        vec![input("same", "A", "1"), input("same", "B", "2")],
        vec![Edit::AddInputPort {
            temporary: "p".into(),
            name: "In".into(),
            node: existing(r.nodes["net"]),
            requirement: Requirement::Required,
        }],
        vec![Edit::RemovePort(PortRef::Temporary("missing".into()))],
        vec![
            Edit::AddOutputPort {
                temporary: "p".into(),
                name: "Out".into(),
                node: existing(r.nodes["net"]),
            },
            Edit::AddOutputPort {
                temporary: "q".into(),
                name: "Out".into(),
                node: existing(r.nodes["net"]),
            },
        ],
        vec![
            Edit::AddInputPort {
                temporary: "p".into(),
                name: "One".into(),
                node: existing(r.nodes["revenue"]),
                requirement: Requirement::Required,
            },
            Edit::AddInputPort {
                temporary: "q".into(),
                name: "Two".into(),
                node: existing(r.nodes["revenue"]),
                requirement: Requirement::Required,
            },
        ],
    ];
    for (i, edits) in bad_batches.into_iter().enumerate() {
        assert_eq!(
            s.apply_edits("alice", req(r.revision, &format!("bad-{i}"), edits)),
            Err(Error::InvalidEdit)
        );
        assert_eq!(s.read_draft("alice", r.revision).unwrap(), before);
    }
}

#[test]
fn complete_typed_receipt_intent_is_compared_including_decimal_layout() {
    let (mut s, c, _) = ordinary();
    let first = req(c.revision, "exact", vec![input("a", "A", "1.0")]);
    let r = s.apply_edits("alice", first.clone()).unwrap();
    let changes = vec![
        req(c.revision, "exact", vec![input("a", "A", "1.00")]),
        req(
            c.revision,
            "exact",
            vec![input("different-temp", "A", "1.0")],
        ),
        req(c.revision, "exact", vec![input("a", "Other", "1.0")]),
        req(r.revision, "exact", first.edits.clone()),
        req(c.revision, "exact", vec![formula("a", "A", "1.0")]),
    ];
    for changed in changes {
        assert_eq!(
            s.apply_edits("alice", changed),
            Err(Error::OperationIdConflict)
        );
    }
    assert_eq!(s.apply_edits("alice", first), Ok(r.clone()));
    assert_eq!(s.read_draft("alice", r.revision).unwrap().nodes.len(), 1);
    // Create and edit operation IDs share the principal/service namespace.
    assert_eq!(
        s.apply_edits(
            "alice",
            req(r.revision, "create", vec![input("b", "B", "2")])
        ),
        Err(Error::OperationIdConflict)
    );
    assert_eq!(
        s.create(
            "alice",
            CreateRequest {
                instance: c.revision.instance,
                operation_id: "exact".into(),
                name: "Other".into()
            }
        ),
        Err(Error::OperationIdConflict)
    );
}

#[test]
fn receipt_exhaustion_is_independent_of_model_capacity_and_replay_still_works() {
    let (mut s, c, _) = fixture(
        Limits {
            receipts: 2,
            ..Limits::default()
        },
        EditLimits::default(),
    );
    let first = req(c.revision, "last-slot", vec![input("a", "A", "1")]);
    let r = s.apply_edits("alice", first.clone()).unwrap();
    assert_eq!(
        s.apply_edits(
            "alice",
            req(r.revision, "no-slot", vec![input("b", "B", "2")])
        ),
        Err(Error::ReceiptLimit)
    );
    assert_eq!(s.read_draft("alice", r.revision).unwrap().nodes.len(), 1);
    assert_eq!(s.apply_edits("alice", first), Ok(r));
}

#[test]
fn node_port_source_and_identity_quotas_roll_back() {
    let limits = EditLimits {
        nodes: 1,
        ports: 1,
        source_bytes: 3,
        node_identities: 2,
        port_identities: 2,
        ..EditLimits::default()
    };
    let (mut s, c, _) = fixture(Limits::default(), limits);
    let first = s
        .apply_edits(
            "alice",
            req(c.revision, "first", vec![input("a", "A", "1")]),
        )
        .unwrap();
    let before = s.read_draft("alice", first.revision).unwrap();
    assert_eq!(
        s.apply_edits(
            "alice",
            req(first.revision, "nodes", vec![input("b", "B", "2")])
        ),
        Err(Error::ModelLimit)
    );
    assert_eq!(
        s.apply_edits(
            "alice",
            req(
                first.revision,
                "source",
                vec![Edit::SetNode {
                    node: existing(first.nodes["a"]),
                    definition: NodeDefinition::Formula("1 + 2".into())
                }]
            )
        ),
        Err(Error::ModelLimit)
    );
    assert_eq!(
        s.apply_edits(
            "alice",
            req(
                first.revision,
                "ports",
                vec![
                    Edit::AddOutputPort {
                        temporary: "p".into(),
                        name: "P".into(),
                        node: existing(first.nodes["a"])
                    },
                    Edit::AddOutputPort {
                        temporary: "q".into(),
                        name: "Q".into(),
                        node: existing(first.nodes["a"])
                    },
                ]
            )
        ),
        Err(Error::ModelLimit)
    );
    assert_eq!(s.read_draft("alice", first.revision).unwrap(), before);
    let next = s
        .apply_edits(
            "alice",
            req(
                first.revision,
                "replace",
                vec![
                    Edit::RemoveNode(existing(first.nodes["a"])),
                    input("a", "A", "2"),
                ],
            ),
        )
        .unwrap();
    assert_ne!(first.nodes["a"], next.nodes["a"]);
    assert_eq!(
        s.apply_edits(
            "alice",
            req(
                next.revision,
                "identity-cap",
                vec![
                    Edit::RemoveNode(existing(next.nodes["a"])),
                    input("a", "A", "3")
                ]
            )
        ),
        Err(Error::ModelLimit)
    );
    assert_eq!(s.read_draft("alice", next.revision).unwrap().nodes.len(), 1);
}

#[test]
fn revoked_authorization_at_commit_rejects_candidate_and_retains_rejection() {
    let (mut s, c, controls) = ordinary();
    controls.deny_edit_call.store(2, Ordering::SeqCst);
    let request = req(c.revision, "revoked", vec![input("a", "A", "1")]);
    assert_eq!(s.apply_edits("alice", request.clone()), Err(Error::Denied));
    assert!(s.read_draft("alice", c.revision).unwrap().nodes.is_empty());
    controls.deny_edit_call.store(0, Ordering::SeqCst);
    assert_eq!(s.apply_edits("alice", request), Err(Error::Denied));
    assert_eq!(
        s.operation_outcome(c.revision.instance, "alice", "revoked"),
        Ok(OperationOutcome::Rejected(Error::Denied))
    );
    assert_eq!(
        s.apply_edits(
            "denied",
            req(c.revision, "denied", vec![input("a", "A", "1")])
        ),
        Err(Error::Denied)
    );
}

#[test]
fn cross_model_and_stale_references_are_rejected_without_mutation() {
    let (mut s, c, _) = ordinary();
    let r = seed(&mut s, c);
    let other = s
        .create(
            "alice",
            CreateRequest {
                instance: c.revision.instance,
                operation_id: "other".into(),
                name: "Other".into(),
            },
        )
        .unwrap();
    assert_eq!(
        s.apply_edits(
            "alice",
            req(
                other.revision,
                "foreign",
                vec![Edit::RemoveNode(existing(r.nodes["net"]))]
            )
        ),
        Err(Error::InvalidEdit)
    );
    let (mut restarted, _, _) = ordinary();
    assert_eq!(
        restarted.apply_edits(
            "alice",
            req(r.revision, "stale", vec![input("a", "A", "1")])
        ),
        Err(Error::StaleInstance)
    );
    assert_eq!(
        restarted.read_draft("alice", r.revision),
        Err(Error::StaleInstance)
    );
    assert!(
        s.read_draft("alice", other.revision)
            .unwrap()
            .nodes
            .is_empty()
    );
}

#[test]
fn temporary_port_references_and_nonempty_noop_revision_semantics() {
    let (mut s, c, _) = ordinary();
    let r = s
        .apply_edits(
            "alice",
            req(
                c.revision,
                "temps",
                vec![
                    input("a", "A", "1"),
                    Edit::AddOutputPort {
                        temporary: "p".into(),
                        name: "Out".into(),
                        node: temp("a"),
                    },
                    Edit::RenamePort {
                        port: PortRef::Temporary("p".into()),
                        name: "Result".into(),
                    },
                    Edit::SetOutputPort {
                        port: PortRef::Temporary("p".into()),
                        node: temp("a"),
                    },
                ],
            ),
        )
        .unwrap();
    let same = s
        .apply_edits(
            "alice",
            req(
                r.revision,
                "same",
                vec![Edit::RenameNode {
                    node: existing(r.nodes["a"]),
                    name: "A".into(),
                }],
            ),
        )
        .unwrap();
    assert_eq!(same.revision.sequence, r.revision.sequence + 1);
    let gone = s
        .apply_edits(
            "alice",
            req(
                same.revision,
                "transient",
                vec![input("x", "X", "3"), Edit::RemoveNode(temp("x"))],
            ),
        )
        .unwrap();
    assert!(gone.nodes.is_empty());
    let view = s.read_draft("alice", gone.revision).unwrap();
    assert_eq!(view.ports[0].id, r.ports["p"]);
    assert_eq!(view.ports[0].name, "Result");
    assert_eq!(
        s.apply_edits("alice", req(gone.revision, "empty", vec![])),
        Err(Error::InvalidArgument)
    );
}

#[test]
fn cycles_are_saved_as_diagnostics_and_can_be_repaired() {
    let (mut s, c, _) = ordinary();
    let r = s
        .apply_edits(
            "alice",
            req(
                c.revision,
                "cycle",
                vec![formula("a", "A", "B + 1"), formula("b", "B", "A + 1")],
            ),
        )
        .unwrap();
    assert_eq!(
        r.diagnostics
            .iter()
            .filter(|d| d.kind == DiagnosticKind::Unorderable)
            .count(),
        2
    );
    let repaired = s
        .apply_edits(
            "alice",
            req(
                r.revision,
                "fix",
                vec![Edit::SetNode {
                    node: existing(r.nodes["a"]),
                    definition: NodeDefinition::Input(
                        PivotDecimal::from_parts(1, 0, 0, false, 0).unwrap(),
                    ),
                }],
            ),
        )
        .unwrap();
    assert!(repaired.diagnostics.is_empty());
}

#[test]
fn bounded_batch_admission_allocates_no_receipt() {
    let (mut s, c, _) = fixture(
        Limits::default(),
        EditLimits {
            operations: 1,
            batch_bytes: 512,
            ..EditLimits::default()
        },
    );
    assert_eq!(
        s.apply_edits(
            "alice",
            req(
                c.revision,
                "large",
                vec![input("a", "A", "1"), input("b", "B", "2")]
            )
        ),
        Err(Error::BatchLimit)
    );
    assert_eq!(
        s.operation_outcome(c.revision.instance, "alice", "large"),
        Ok(OperationOutcome::Unknown)
    );
    assert_eq!(
        s.apply_edits(
            "alice",
            req(
                c.revision,
                "source",
                vec![formula("f", "F", &"1".repeat(300))]
            )
        ),
        Err(Error::BatchLimit)
    );
    assert_eq!(
        s.apply_edits(
            "alice",
            req(
                c.revision,
                "huge-temp",
                vec![input(&"t".repeat(65), "A", "1")]
            )
        ),
        Err(Error::InvalidArgument)
    );
    assert!(s.read_draft("alice", c.revision).unwrap().nodes.is_empty());
}

#[test]
fn rollback_restores_existing_public_port_directory_and_bindings() {
    let (mut s, c, _) = ordinary();
    let r = s
        .apply_edits(
            "alice",
            req(
                c.revision,
                "seed-ports",
                vec![
                    input("a", "A", "1"),
                    input("b", "B", "2"),
                    Edit::AddInputPort {
                        temporary: "in".into(),
                        name: "In".into(),
                        node: temp("a"),
                        requirement: Requirement::Required,
                    },
                    Edit::AddOutputPort {
                        temporary: "out".into(),
                        name: "Out".into(),
                        node: temp("b"),
                    },
                ],
            ),
        )
        .unwrap();
    let before = s.read_draft("alice", r.revision).unwrap();
    let failed = s.apply_edits(
        "alice",
        req(
            r.revision,
            "rollback-ports",
            vec![
                Edit::RenamePort {
                    port: PortRef::Existing(r.ports["in"]),
                    name: "Renamed".into(),
                },
                Edit::SetInputPort {
                    port: PortRef::Existing(r.ports["in"]),
                    node: existing(r.nodes["b"]),
                    requirement: Requirement::Optional("99".parse().unwrap()),
                },
                Edit::RemovePort(PortRef::Existing(r.ports["out"])),
                Edit::AddOutputPort {
                    temporary: "new".into(),
                    name: "New".into(),
                    node: existing(r.nodes["a"]),
                },
                Edit::RemoveNode(temp("never-added")),
            ],
        ),
    );
    assert_eq!(failed, Err(Error::InvalidEdit));
    assert_eq!(s.read_draft("alice", r.revision).unwrap(), before);
}

#[test]
fn removed_port_ids_cannot_alias_replacements_or_cross_models() {
    let (mut s, c, _) = ordinary();
    let r = s
        .apply_edits(
            "alice",
            req(
                c.revision,
                "seed",
                vec![
                    input("a", "A", "1"),
                    Edit::AddOutputPort {
                        temporary: "p".into(),
                        name: "Result".into(),
                        node: temp("a"),
                    },
                ],
            ),
        )
        .unwrap();
    let next = s
        .apply_edits(
            "alice",
            req(
                r.revision,
                "replace",
                vec![
                    Edit::RemovePort(PortRef::Existing(r.ports["p"])),
                    Edit::AddOutputPort {
                        temporary: "p".into(),
                        name: "Result".into(),
                        node: existing(r.nodes["a"]),
                    },
                ],
            ),
        )
        .unwrap();
    assert_ne!(r.ports["p"], next.ports["p"]);
    assert_eq!(
        s.apply_edits(
            "alice",
            req(
                next.revision,
                "old",
                vec![Edit::RemovePort(PortRef::Existing(r.ports["p"]))]
            )
        ),
        Err(Error::InvalidEdit)
    );
    let other = s
        .create(
            "alice",
            CreateRequest {
                instance: c.revision.instance,
                operation_id: "other".into(),
                name: "Other".into(),
            },
        )
        .unwrap();
    assert_eq!(
        s.apply_edits(
            "alice",
            req(
                other.revision,
                "foreign-port",
                vec![Edit::RemovePort(PortRef::Existing(next.ports["p"]))]
            )
        ),
        Err(Error::InvalidEdit)
    );
    let before = s.read_draft("alice", next.revision).unwrap();
    assert_eq!(before.ports.len(), 1);
}

#[test]
fn source_bytes_command_order_and_optional_default_layout_are_receipt_intent() {
    let (mut s, c, _) = ordinary();
    let original = req(
        c.revision,
        "typed",
        vec![
            input("a", "A", "1"),
            formula("f", "F", "A + 1"),
            Edit::AddInputPort {
                temporary: "p".into(),
                name: "Input".into(),
                node: temp("a"),
                requirement: Requirement::Optional("2.0".parse().unwrap()),
            },
        ],
    );
    let result = s.apply_edits("alice", original.clone()).unwrap();
    let mut source = original.clone();
    source.edits[1] = formula("f", "F", "A+1");
    assert_eq!(
        s.apply_edits("alice", source),
        Err(Error::OperationIdConflict)
    );
    let mut order = original.clone();
    order.edits.swap(0, 1);
    assert_eq!(
        s.apply_edits("alice", order),
        Err(Error::OperationIdConflict)
    );
    let mut layout = original.clone();
    if let Edit::AddInputPort { requirement, .. } = &mut layout.edits[2] {
        *requirement = Requirement::Optional("2.00".parse().unwrap());
    }
    assert_eq!(
        s.apply_edits("alice", layout),
        Err(Error::OperationIdConflict)
    );
    assert_eq!(s.apply_edits("alice", original), Ok(result));
}
