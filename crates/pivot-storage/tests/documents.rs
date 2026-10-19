use pivot_storage::{
    Decimal96, Definition, DiagnosticKind, Error as GraphError, Limits, PortKind, Token, Uuid,
    document::{DocumentSession, Error},
};

fn id(n: u128) -> Uuid {
    Uuid::from_u128(n)
}
fn d(n: u32, scale: u32) -> Decimal96 {
    Decimal96::from_parts(n, 0, 0, false, scale).unwrap()
}
fn model() -> DocumentSession {
    let mut s = DocumentSession::new("ProfitModel", Limits::default()).unwrap();
    s.add_input(s.expected(), id(1), "Revenue", d(100, 0))
        .unwrap();
    s.add_input(s.expected(), id(2), "Costs", d(25, 0)).unwrap();
    s.add_formula(s.expected(), id(3), "Net", "Revenue - Costs")
        .unwrap();
    s.add_formula(s.expected(), id(4), "Margin", "Net / Revenue")
        .unwrap();
    s.add_port(s.expected(), id(10), "revenue", id(1), PortKind::Required)
        .unwrap();
    s.add_port(
        s.expected(),
        id(11),
        "costs",
        id(2),
        PortKind::Optional(d(2500, 2)),
    )
    .unwrap();
    s.add_port(s.expected(), id(12), "net", id(3), PortKind::Output)
        .unwrap();
    s.add_port(s.expected(), id(13), "margin", id(4), PortKind::Output)
        .unwrap();
    s
}
fn number(s: &DocumentSession, node: Uuid) -> pivot_types::PivotDecimal {
    s.evaluate(s.expected(), &[node]).unwrap().values[0]
        .1
        .as_ref()
        .unwrap()
        .runtime()
        .unwrap()
}
fn reopen(s: &mut DocumentSession) -> DocumentSession {
    let capture = s.export(s.expected()).unwrap();
    DocumentSession::load(capture.document().clone(), Limits::default()).unwrap()
}

#[test]
fn profit_model_exports_restores_and_edits_without_hosted_management() {
    let mut a = model();
    assert!(a.validate(a.expected()).unwrap().is_empty());
    assert_eq!(number(&a, id(3)), "75".parse().unwrap());
    assert_eq!(number(&a, id(4)), "0.75".parse().unwrap());
    let mut b = reopen(&mut a);
    assert_eq!(a.document_id(), b.document_id());
    assert_eq!(a.graph(), b.graph());
    assert_ne!(a.expected().session, b.expected().session);
    assert_eq!(number(&b, id(4)), number(&a, id(4)));
    b.set_input(b.expected(), id(1), d(200, 0)).unwrap();
    assert_eq!(number(&b, id(3)), "175".parse().unwrap());
    assert_eq!(number(&a, id(3)), "75".parse().unwrap());
}

#[test]
fn named_interface_required_optional_and_override_isolation() {
    let mut s = model();
    let before = s.export(s.expected()).unwrap();
    let outputs = vec!["net".into(), "margin".into()];
    assert!(s.invoke(s.expected(), &[], &outputs).is_err());
    let a = s
        .invoke(s.expected(), &[("revenue".into(), d(200, 0))], &outputs)
        .unwrap();
    assert_eq!(
        a.values[0].1.as_ref().unwrap().runtime().unwrap(),
        "175".parse().unwrap()
    );
    assert_eq!(
        a.values[1].1.as_ref().unwrap().runtime().unwrap(),
        "0.875".parse().unwrap()
    );
    let b = s
        .invoke(
            s.expected(),
            &[("revenue".into(), d(200, 0)), ("costs".into(), d(50, 0))],
            &outputs,
        )
        .unwrap();
    assert_eq!(
        b.values[0].1.as_ref().unwrap().runtime().unwrap(),
        "150".parse().unwrap()
    );
    assert_ne!(a.generation, b.generation);
    assert!(
        s.invoke(
            s.expected(),
            &[("revenue".into(), d(1, 0)), ("revenue".into(), d(2, 0))],
            &outputs
        )
        .is_err()
    );
    assert!(
        s.invoke(s.expected(), &[("unknown".into(), d(1, 0))], &outputs)
            .is_err()
    );
    assert_eq!(
        before.document(),
        s.export(s.expected()).unwrap().document()
    );
    let b = reopen(&mut s);
    assert_eq!(
        b.invoke(b.expected(), &[("revenue".into(), d(200, 0))], &outputs)
            .unwrap()
            .values,
        a.values
    );
}

#[test]
fn rename_name_reuse_and_tombstones_preserve_bindings() {
    let mut s = model();
    let tokens = s
        .graph()
        .nodes
        .iter()
        .find(|n| n.id == id(3))
        .unwrap()
        .definition
        .clone();
    s.rename(s.expected(), id(1), "Sales").unwrap();
    s.add_input(s.expected(), id(5), "Revenue", d(999, 0))
        .unwrap();
    let mut s = reopen(&mut s);
    assert_eq!(number(&s, id(3)), "75".parse().unwrap());
    assert_eq!(
        s.graph()
            .nodes
            .iter()
            .find(|n| n.id == id(3))
            .unwrap()
            .definition,
        tokens
    );
    s.remove(s.expected(), id(1)).unwrap();
    let mut s = reopen(&mut s);
    assert!(s.graph().node_ids.contains(&id(1)));
    assert!(
        s.validate(s.expected())
            .unwrap()
            .iter()
            .any(|d| d.kind == DiagnosticKind::Missing(id(1)))
    );
    assert!(s.add_input(s.expected(), id(1), "Again", d(1, 0)).is_err());
    s.set_formula(s.expected(), id(3), "Revenue - Costs")
        .unwrap();
    s.set_formula(s.expected(), id(4), "Net / Revenue").unwrap();
    s.set_port_target(s.expected(), id(10), id(5)).unwrap();
    assert_eq!(number(&s, id(3)), "974".parse().unwrap());
    s.remove_port(s.expected(), id(10)).unwrap();
    assert!(s.graph().port_ids.contains(&id(10)));
    assert!(
        s.add_port(s.expected(), id(10), "Again", id(5), PortKind::Output)
            .is_err()
    );
}

#[test]
fn invalid_drafts_survive_until_explicit_repair() {
    let mut s = DocumentSession::new("Draft", Limits::default()).unwrap();
    s.add_formula(s.expected(), id(1), "Result", "Future + 1")
        .unwrap();
    s.add_input(s.expected(), id(2), "Future", d(5, 0)).unwrap();
    let mut s = reopen(&mut s);
    assert!(matches!(
        s.graph().nodes[0].definition,
        Definition::BindFailed { .. }
    ));
    assert!(s.evaluate(s.expected(), &[id(1)]).is_err());
    s.set_formula(s.expected(), id(1), "Future + 1").unwrap();
    assert_eq!(number(&s, id(1)), "6".parse().unwrap());
    for source in ["", "Future +", "Result + 1"] {
        s.set_formula(s.expected(), id(1), source).unwrap();
        s = reopen(&mut s);
        assert!(!s.validate(s.expected()).unwrap().is_empty());
        assert!(s.evaluate(s.expected(), &[id(1)]).is_err());
    }
    s.set_formula(s.expected(), id(1), "Future * 2").unwrap();
    assert_eq!(number(&s, id(1)), "10".parse().unwrap());
}

#[test]
fn exact_decimal_edits_defaults_literals_and_unrelated_changes() {
    for scale in 0..=28 {
        for negative in [false, true] {
            for coefficient in [0, 1, u32::MAX] {
                let exact = Decimal96::from_parts(coefficient, 9, 7, negative, scale).unwrap();
                let mut s = DocumentSession::new("Exact", Limits::default()).unwrap();
                s.add_input(s.expected(), id(1), "N", exact).unwrap();
                s.add_port(s.expected(), id(2), "n", id(1), PortKind::Optional(exact))
                    .unwrap();
                s.add_formula(s.expected(), id(3), "Result", "N + 1.2300")
                    .unwrap();
                let literal = match &s.graph().nodes[1].definition {
                    Definition::Bound { tokens, .. } => tokens
                        .iter()
                        .find_map(|t| {
                            if let Token::Decimal(v) = t {
                                Some(*v)
                            } else {
                                None
                            }
                        })
                        .unwrap(),
                    _ => panic!(),
                };
                assert_eq!(literal.parts().unwrap(), (12300, 0, 0, false, 4));
                s.rename(s.expected(), id(1), "Renamed").unwrap();
                let mut s = reopen(&mut s);
                assert_eq!(s.graph().nodes[0].definition, Definition::Input(exact));
                assert_eq!(s.graph().ports[0].kind, PortKind::Optional(exact));
                let zero = Decimal96::from_parts(0, 0, 0, negative, scale).unwrap();
                s.set_input(s.expected(), id(1), zero).unwrap();
                assert_eq!(
                    reopen(&mut s).graph().nodes[0].definition,
                    Definition::Input(zero)
                );
            }
        }
    }
}

#[test]
fn imported_literal_representation_survives_runtime_normalization() {
    let mut s = model();
    let mut doc = s.export(s.expected()).unwrap().document().clone();
    let exact = Decimal96::from_parts(0, 0, 0, true, 28).unwrap();
    doc.graph
        .nodes
        .iter_mut()
        .find(|n| n.id == id(3))
        .unwrap()
        .definition = Definition::Bound {
        source: "0.0000000000000000000000000000".into(),
        tokens: vec![Token::Decimal(exact)],
    };
    let mut s = DocumentSession::load(doc, Limits::default()).unwrap();
    s.rename(s.expected(), id(2), "Expenses").unwrap();
    let capture = s.export(s.expected()).unwrap();
    let Definition::Bound { tokens, .. } = &capture.document().graph.nodes[2].definition else {
        panic!()
    };
    assert_eq!(tokens, &[Token::Decimal(exact)]);
}

#[test]
fn immutable_capture_saved_checkpoint_and_out_of_order_acknowledgment() {
    let mut s = model();
    assert!(s.is_dirty());
    let saved = s.export(s.expected()).unwrap();
    s.acknowledge_saved(&saved).unwrap();
    assert!(!s.is_dirty());
    assert_eq!(
        s.rename(s.expected(), id(1), "Revenue").unwrap(),
        s.expected()
    );
    assert!(!s.is_dirty());
    let old_expected = s.expected();
    s.set_input(s.expected(), id(1), d(200, 0)).unwrap();
    assert!(s.is_dirty());
    assert_eq!(saved.document().state_id, old_expected.state);
    s.acknowledge_saved(&saved).unwrap();
    assert!(s.is_dirty());
    assert_eq!(
        number(
            &DocumentSession::load(saved.document().clone(), Limits::default()).unwrap(),
            id(3)
        ),
        "75".parse().unwrap()
    );
    let newer = s.export(s.expected()).unwrap();
    s.acknowledge_saved(&newer).unwrap();
    s.acknowledge_saved(&newer.clone()).unwrap();
    assert!(!s.is_dirty());
    assert_eq!(s.acknowledge_saved(&saved), Err(Error::StaleCheckpoint));
    assert!(!s.is_dirty());
}

#[test]
fn capture_s_edit_t_then_ack_s_never_marks_t_saved() {
    let mut s = model();
    let capture = s.export(s.expected()).unwrap();
    s.set_input(s.expected(), id(1), d(200, 0)).unwrap();
    s.acknowledge_saved(&capture).unwrap();
    assert!(s.is_dirty());
    assert_eq!(
        capture.document().graph.nodes[0].definition,
        Definition::Input(d(100, 0))
    );
}

#[test]
fn copied_logical_ids_do_not_authorize_other_sessions_or_old_revisions() {
    let mut a = model();
    let capture = a.export(a.expected()).unwrap();
    let mut b = DocumentSession::load(capture.document().clone(), Limits::default()).unwrap();
    assert_eq!(a.graph().model, b.graph().model);
    assert_eq!(
        b.evaluate(a.expected(), &[id(3)]).unwrap_err(),
        Error::StaleSession
    );
    assert_eq!(b.acknowledge_saved(&capture), Err(Error::StaleSession));
    let old = b.expected();
    b.set_input(old, id(1), d(200, 0)).unwrap();
    assert_eq!(b.set_formula(old, id(3), "1"), Err(Error::StaleRevision));
    assert_eq!(b.export(old).unwrap_err(), Error::StaleRevision);
    let mut wrong = b.expected();
    wrong.graph_revision = Uuid::new_v4();
    assert_eq!(b.validate(wrong), Err(Error::StaleRevision));
}

#[test]
fn failed_edits_are_atomic_including_budgets() {
    let mut s = model();
    let capture = s.export(s.expected()).unwrap();
    assert!(s.rename(s.expected(), id(1), "Costs").is_err());
    assert!(
        s.set_input(s.expected(), id(1), Decimal96([255; 16]))
            .is_err()
    );
    assert!(
        s.add_formula(s.expected(), id(20), "TooLong", &"1".repeat(4097))
            .is_err()
    );
    assert!(
        s.add_port(s.expected(), id(20), "revenue", id(2), PortKind::Required)
            .is_err()
    );
    assert_eq!(
        capture.document(),
        s.export(s.expected()).unwrap().document()
    );
    assert!(
        s.set_formula(s.expected(), id(3), &"1".repeat(4097))
            .is_err()
    );
    assert!(s.rename(s.expected(), id(1), &"X".repeat(129)).is_err());
    assert_eq!(
        capture.document(),
        s.export(s.expected()).unwrap().document()
    );
    let limits = Limits {
        nodes: 1,
        ..Limits::default()
    };
    let mut s = DocumentSession::new("Bounded", limits).unwrap();
    s.add_input(s.expected(), id(1), "N", d(1, 0)).unwrap();
    let before = s.export(s.expected()).unwrap();
    assert_eq!(
        s.add_input(s.expected(), id(2), "Extra", d(1, 0)),
        Err(Error::Graph(GraphError::Limit))
    );
    assert_eq!(
        before.document(),
        s.export(s.expected()).unwrap().document()
    );
}

#[test]
fn malformed_payloads_fail_before_exposing_a_session() {
    let mut s = model();
    let original = s.export(s.expected()).unwrap().document().clone();
    let mutations: Vec<fn(&mut pivot_storage::document::ModelDocument)> = vec![
        |d| d.format = 99,
        |d| d.document_id = Uuid::nil(),
        |d| d.state_id = Uuid::nil(),
        |d| d.graph.version = 99,
        |d| d.graph.numeric = 99,
        |d| d.graph.expression = 99,
        |d| d.graph.node_ids.push(id(1)),
        |d| d.graph.nodes[0].id = id(999),
        |d| d.graph.ports[0].model = Uuid::new_v4(),
        |d| d.graph.nodes[0].definition = Definition::Input(Decimal96([255; 16])),
        |d| {
            d.graph.nodes[0].definition = Definition::Bound {
                source: "1".into(),
                tokens: vec![Token::Node {
                    model: d.graph.model,
                    node: id(999),
                }],
            }
        },
        |d| {
            d.graph.nodes[0].definition = Definition::Bound {
                source: "1".into(),
                tokens: vec![Token::Binary(0)],
            }
        },
    ];
    for mutate in mutations {
        let mut bad = original.clone();
        mutate(&mut bad);
        assert!(DocumentSession::load(bad, Limits::default()).is_err());
    }
    for limits in [
        Limits {
            bytes: 1,
            ..Limits::default()
        },
        Limits {
            items: 1,
            ..Limits::default()
        },
        Limits {
            work: 1,
            ..Limits::default()
        },
    ] {
        assert!(DocumentSession::load(original.clone(), limits).is_err());
    }
    assert_eq!(number(&s, id(3)), "75".parse().unwrap());
}

#[test]
fn runtime_restore_uses_tokens_without_reparsing_original_source() {
    let mut s = model();
    let mut doc = s.export(s.expected()).unwrap().document().clone();
    if let Definition::Bound { source, .. } = &mut doc.graph.nodes[2].definition {
        *source = "original source deliberately not parseable".into();
    }
    let s = DocumentSession::load(doc, Limits::default()).unwrap();
    assert_eq!(number(&s, id(3)), "75".parse().unwrap());
}

#[test]
fn core_has_no_document_storage_or_filesystem_dependencies() {
    let manifest = include_str!("../../pivot-core/Cargo.toml");
    let dependencies = manifest.split("[dependencies]").nth(1).unwrap().trim();
    assert_eq!(dependencies, "pivot-types = { path = \"../pivot-types\" }");
}
