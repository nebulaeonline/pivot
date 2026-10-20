use pivot_core::{
    interface::{InputRequirement, PortId},
    model::{NodeId, WorkingModel},
};
use pivot_storage::*;
use std::collections::BTreeMap;
fn id(n: u128) -> Uuid {
    Uuid::from_u128(n)
}
fn d(n: u32, scale: u32) -> Decimal96 {
    Decimal96::from_parts(n, 0, 0, false, scale).unwrap()
}
fn base() -> Record {
    Record {
        codec: 1,
        numeric: 1,
        expression: 1,
        store: id(1),
        epoch: id(2),
        model: id(3),
        revision: id(4),
        sequence: 42,
        name: "Profit".into(),
        node_ids: vec![id(10), id(11), id(12)],
        port_ids: vec![id(100), id(101), id(102)],
        nodes: vec![
            Node {
                id: id(10),
                name: "Revenue".into(),
                definition: Definition::Input(d(10000, 2)),
            },
            Node {
                id: id(11),
                name: "Costs".into(),
                definition: Definition::Input(d(250, 1)),
            },
            Node {
                id: id(12),
                name: "Net".into(),
                definition: Definition::Bound {
                    source: "Revenue - Costs".into(),
                    tokens: vec![
                        Token::Node {
                            model: id(3),
                            node: id(10),
                        },
                        Token::Node {
                            model: id(3),
                            node: id(11),
                        },
                        Token::Binary(1),
                    ],
                },
            },
        ],
        ports: vec![
            Port {
                id: id(100),
                name: "Sales".into(),
                model: id(3),
                node: id(10),
                kind: PortKind::Required,
            },
            Port {
                id: id(101),
                name: "Expenses".into(),
                model: id(3),
                node: id(11),
                kind: PortKind::Optional(d(2500, 2)),
            },
            Port {
                id: id(102),
                name: "Profit".into(),
                model: id(3),
                node: id(12),
                kind: PortKind::Output,
            },
        ],
        publication: None,
    }
}
fn publication() -> Publication {
    Publication {
        model: id(200),
        revision: id(201),
        snapshot: id(202),
        public_ports: vec![
            (id(100), "sales".into()),
            (id(101), "expenses".into()),
            (id(102), "profit".into()),
        ],
    }
}
fn round(r: &Record) -> Record {
    let bytes = encode(r, Limits::default()).unwrap();
    let v = decode(&bytes, Limits::default()).unwrap();
    assert_eq!(r, &v);
    assert_eq!(encode(&v, Limits::default()).unwrap(), bytes);
    v
}
fn number(restored: &RestoredModel, node: Uuid) -> pivot_types::PivotDecimal {
    restored.evaluate(&[node]).unwrap()[0]
        .1
        .as_ref()
        .unwrap()
        .runtime()
        .unwrap()
}

#[test]
fn hosted_restore_retains_exact_encoded_budget_profile() {
    let mut record = base();
    for n in 103..200 {
        record.port_ids.push(id(n));
        record.ports.push(Port {
            id: id(n),
            name: format!("Output{n}"),
            model: record.model,
            node: id(12),
            kind: PortKind::Output,
        });
    }
    let bytes = encode(&record, Limits::default()).unwrap();
    let limits = Limits {
        bytes: bytes.len(),
        ..Limits::default()
    };
    // Document logical accounting is independent of hosted-v1 exact byte limits.
    assert!(
        record
            .graph_payload(limits)
            .unwrap()
            .validate(limits)
            .is_err()
    );
    let restored = restore(decode(&bytes, limits).unwrap(), limits).unwrap();
    assert_eq!(number(&restored, id(12)), "75".parse().unwrap());
}

#[test]
fn rename_and_name_reuse_never_retarget_bound_references() {
    let mut restored = restore(base(), Limits::default()).unwrap();
    restored.rename(id(10), "Sales").unwrap();
    restored.add_input(id(13), "Revenue", d(999, 0)).unwrap();
    let r = round(restored.record());
    assert_eq!(r.nodes[2].definition, base().nodes[2].definition);
    let mut a = restore(r.clone(), Limits::default()).unwrap();
    let b = restore(r.clone(), Limits::default()).unwrap();
    assert_eq!(number(&a, id(12)), "75".parse().unwrap());
    a.set_input(id(10), d(200, 0)).unwrap();
    assert_eq!(number(&a, id(12)), "175".parse().unwrap());
    assert_eq!(number(&b, id(12)), "75".parse().unwrap());
    assert_eq!(b.record(), &r);
}
#[test]
fn retired_targets_survive_and_cannot_be_reallocated() {
    let mut a = restore(base(), Limits::default()).unwrap();
    a.remove(id(10)).unwrap();
    a.add_input(id(13), "Revenue", d(999, 0)).unwrap();
    let mut a = restore(round(a.record()), Limits::default()).unwrap();
    assert!(a.record().node_ids.contains(&id(10)));
    assert!(
        a.diagnostics()
            .iter()
            .any(|d| d.kind == DiagnosticKind::Missing(id(10)))
    );
    assert!(
        a.diagnostics()
            .iter()
            .any(|d| d.kind == DiagnosticKind::InvalidPort)
    );
    assert_eq!(a.evaluate(&[id(12)]).unwrap_err(), Error::InvalidDraft);
    assert_eq!(
        a.add_input(id(10), "Again", d(1, 0)).unwrap_err(),
        Error::Identity
    );
    assert!(a.record().nodes.iter().any(|n| n.id == id(13)));
    a.set_formula(id(12), "Revenue - Costs").unwrap();
    a.set_port_target(id(100), id(13)).unwrap();
    assert!(a.diagnostics().is_empty());
    assert_eq!(number(&a, id(12)), "974".parse().unwrap());
    round(a.record());
}
#[test]
fn unresolved_and_incomplete_drafts_remain_invalid_until_explicit_edit() {
    let mut r = base();
    r.nodes[2].definition = Definition::BindFailed {
        source: "Future + 1".into(),
        name: "Future".into(),
    };
    let mut a = restore(round(&r), Limits::default()).unwrap();
    a.add_input(id(13), "Future", d(9, 0)).unwrap();
    let mut a = restore(round(a.record()), Limits::default()).unwrap();
    assert!(
        a.diagnostics()
            .iter()
            .any(|d| d.kind == DiagnosticKind::Unresolved("Future".into()))
    );
    a.set_formula(id(12), "Future + 1").unwrap();
    assert_eq!(number(&a, id(12)), "10".parse().unwrap());
    a.set_formula(id(12), "").unwrap();
    let mut a = restore(round(a.record()), Limits::default()).unwrap();
    assert!(
        matches!(a.record().nodes[2].definition,Definition::ParseFailed{ref source,..} if source.is_empty())
    );
    a.set_formula(id(12), "Revenue +").unwrap();
    let mut a = restore(round(a.record()), Limits::default()).unwrap();
    assert!(!a.diagnostics().is_empty());
    a.set_formula(id(12), "Revenue - Costs").unwrap();
    assert_eq!(number(&a, id(12)), "75".parse().unwrap());
}
#[test]
fn invalid_public_interfaces_remain_editable_but_cannot_be_published() {
    let mut a = restore(base(), Limits::default()).unwrap();
    a.set_formula(id(10), "1").unwrap();
    let r = round(a.record());
    let mut a = restore(r.clone(), Limits::default()).unwrap();
    assert!(
        a.diagnostics()
            .iter()
            .any(|d| d.kind == DiagnosticKind::InvalidPort)
    );
    assert!(a.evaluate(&[id(12)]).is_err());
    let mut p = r;
    p.publication = Some(publication());
    assert!(matches!(
        restore(p, Limits::default()),
        Err(Error::InvalidPublished)
    ));
    a.set_input(id(10), d(100, 0)).unwrap();
    assert!(a.diagnostics().is_empty());
    assert_eq!(number(&a, id(12)), "75".parse().unwrap());
}
#[test]
fn exact_decimal_corpus_survives_encoding_restore_and_unrelated_edits() {
    for scale in 0..=28 {
        for negative in [false, true] {
            for (lo, mid, hi) in [(0, 0, 0), (1, 0, 0), (u32::MAX, u32::MAX, u32::MAX)] {
                let value = Decimal96::from_parts(lo, mid, hi, negative, scale).unwrap();
                let mut r = base();
                r.nodes[0].definition = Definition::Input(value);
                r.ports[1].kind = PortKind::Optional(value);
                r.nodes[2].definition = Definition::Bound {
                    source: "literal from preserved record".into(),
                    tokens: vec![Token::Decimal(value)],
                };
                let mut a = restore(round(&r), Limits::default()).unwrap();
                assert_eq!(a.record(), &r);
                a.rename(id(11), "Renamed").unwrap();
                assert_eq!(a.record().nodes[0].definition, Definition::Input(value));
                assert_eq!(a.record().nodes[2].definition, r.nodes[2].definition);
                assert_eq!(a.record().ports[1].kind, PortKind::Optional(value));
                round(a.record());
            }
        }
    }
    let mut r = base();
    let mut bad = [0; 16];
    bad[12] = 1;
    r.nodes[0].definition = Definition::Input(Decimal96(bad));
    assert_eq!(encode(&r, Limits::default()).unwrap_err(), Error::Malformed);
}
#[test]
fn operators_literals_and_structural_dependencies_are_rebuilt() {
    let mut r = base();
    r.nodes[2].definition = Definition::Bound {
        source: "-(Revenue - Costs) / +2 * 3 + 1".into(),
        tokens: vec![
            Token::Node {
                model: id(3),
                node: id(10),
            },
            Token::Node {
                model: id(3),
                node: id(11),
            },
            Token::Binary(1),
            Token::Unary(1),
            Token::Decimal(d(2, 0)),
            Token::Unary(0),
            Token::Binary(3),
            Token::Decimal(d(3, 0)),
            Token::Binary(2),
            Token::Decimal(d(1, 0)),
            Token::Binary(0),
        ],
    };
    let a = restore(round(&r), Limits::default()).unwrap();
    assert_eq!(number(&a, id(12)), "-111.5".parse().unwrap());
    r.nodes[2].definition = Definition::Bound {
        source: "Net + 1".into(),
        tokens: vec![
            Token::Node {
                model: id(3),
                node: id(12),
            },
            Token::Decimal(d(1, 0)),
            Token::Binary(0),
        ],
    };
    let a = restore(round(&r), Limits::default()).unwrap();
    assert!(
        a.diagnostics()
            .iter()
            .any(|d| d.kind == DiagnosticKind::Unorderable)
    );
    assert!(a.evaluate(&[id(12)]).is_err());
}
#[test]
fn publications_keep_identity_defaults_and_immutable_results() {
    let mut r = base();
    r.publication = Some(publication());
    let mut a = restore(round(&r), Limits::default()).unwrap();
    assert_eq!(a.record(), &r);
    let values = a
        .invoke_public(&[("sales".into(), d(100, 0))], &["profit".into()])
        .unwrap();
    assert_eq!(values[0].0, "profit");
    assert_eq!(
        values[0].1.as_ref().unwrap().runtime().unwrap(),
        "75".parse().unwrap()
    );
    assert!(a.invoke_public(&[], &["profit".into()]).is_err());
    assert_eq!(
        a.set_input(id(10), d(1, 0)).unwrap_err(),
        Error::InvalidPublished
    );
    let mut working = restore(base(), Limits::default()).unwrap();
    working.set_input(id(10), d(200, 0)).unwrap();
    assert_eq!(number(&a, id(12)), "75".parse().unwrap());
    for bad in [0, 1, 2] {
        let mut r = r.clone();
        match bad {
            0 => {
                r.nodes[2].definition = Definition::ParseFailed {
                    source: "".into(),
                    offset: 0,
                    code: 5,
                }
            }
            1 => r.ports[0].node = id(12),
            _ => r.publication.as_mut().unwrap().public_ports[0].1 = "profit".into(),
        };
        assert!(restore(r, Limits::default()).is_err());
    }
}

struct Source {
    core: WorkingModel,
    nodes: BTreeMap<Uuid, NodeId>,
    ports: BTreeMap<Uuid, PortId>,
}
fn source() -> Source {
    let mut core = WorkingModel::new("Profit").unwrap();
    let revenue = core
        .add_input("Revenue", "100.00".parse().unwrap())
        .unwrap();
    let costs = core.add_input("Costs", "25.0".parse().unwrap()).unwrap();
    let net = core.add_computed("Net", "Revenue - Costs").unwrap();
    let sales = core
        .add_input_port("Sales", revenue, InputRequirement::Required)
        .unwrap();
    let expenses = core
        .add_input_port(
            "Expenses",
            costs,
            InputRequirement::Optional {
                default: "25.00".parse::<pivot_types::PivotDecimal>().unwrap().into(),
            },
        )
        .unwrap();
    let profit = core.add_output_port("Profit", net).unwrap();
    Source {
        core,
        nodes: BTreeMap::from([(id(10), revenue), (id(11), costs), (id(12), net)]),
        ports: BTreeMap::from([(id(100), sales), (id(101), expenses), (id(102), profit)]),
    }
}
fn header() -> Header {
    let r = base();
    Header {
        store: r.store,
        epoch: r.epoch,
        model: r.model,
        revision: r.revision,
        sequence: r.sequence,
        publication: None,
    }
}
#[test]
fn capture_uses_actual_prebound_state_and_does_not_modify_source() {
    let mut source = source();
    source.core.rename(source.nodes[&id(10)], "Sales").unwrap();
    let next = source
        .core
        .add_input("Revenue", "999".parse().unwrap())
        .unwrap();
    source.nodes.insert(id(13), next);
    let before = source.core.revision();
    let r = capture(
        &source.core,
        &source.nodes,
        &source.ports,
        header(),
        Limits::default(),
    )
    .unwrap();
    let mut a = restore(round(&r), Limits::default()).unwrap();
    a.set_input(id(10), d(200, 0)).unwrap();
    assert_eq!(number(&a, id(12)), "175".parse().unwrap());
    assert_eq!(source.core.revision(), before);
    assert_eq!(source.core.node_id("Sales"), Some(source.nodes[&id(10)]));
    let result = pivot_core::evaluation::EvaluationContext::new(source.core.snapshot().unwrap())
        .evaluate(&[source.nodes[&id(12)]])
        .unwrap();
    assert_eq!(
        *result
            .value(source.nodes[&id(12)])
            .unwrap()
            .as_ref()
            .unwrap(),
        "75".parse().unwrap()
    );
}
#[test]
fn captured_published_snapshot_does_not_follow_later_working_edits() {
    let mut s = source();
    let snapshot = s.core.snapshot().unwrap();
    s.core
        .set_input(s.nodes[&id(10)], "999".parse().unwrap())
        .unwrap();
    let mut h = header();
    h.publication = Some(publication());
    let r = capture_snapshot(&snapshot, &s.nodes, &s.ports, h, Limits::default()).unwrap();
    assert_eq!(r.nodes[0].definition, base().nodes[0].definition);
    let a = restore(round(&r), Limits::default()).unwrap();
    assert_eq!(number(&a, id(12)), "75".parse().unwrap());
}
#[test]
fn malformed_and_cross_model_records_never_reach_runtime() {
    for case in 0..12 {
        let mut r = base();
        match case {
            0 => r.codec = 2,
            1 => r.numeric = 2,
            2 => r.expression = 2,
            3 => {
                r.nodes[2].definition = Definition::Bound {
                    source: "x".into(),
                    tokens: vec![Token::Binary(0)],
                }
            }
            4 => {
                r.nodes[2].definition = Definition::Bound {
                    source: "x".into(),
                    tokens: vec![Token::Decimal(d(1, 0)), Token::Decimal(d(2, 0))],
                }
            }
            5 => {
                r.nodes[2].definition = Definition::Bound {
                    source: "x".into(),
                    tokens: vec![Token::Node {
                        model: id(999),
                        node: id(10),
                    }],
                }
            }
            6 => {
                r.nodes[2].definition = Definition::Bound {
                    source: "x".into(),
                    tokens: vec![Token::Node {
                        model: id(3),
                        node: id(999),
                    }],
                }
            }
            7 => {
                r.nodes[2].definition = Definition::Bound {
                    source: "x".into(),
                    tokens: vec![Token::Decimal(d(1, 0)), Token::Unary(9)],
                }
            }
            8 => r.nodes[0].name = r.nodes[1].name.clone(),
            9 => r.node_ids.push(id(10)),
            10 => r.ports[0].model = id(999),
            _ => {
                r.nodes[2].definition = Definition::ParseFailed {
                    source: "".into(),
                    offset: 1,
                    code: 5,
                }
            }
        }
        assert!(encode(&r, Limits::default()).is_err());
        assert!(restore(r, Limits::default()).is_err());
    }
}
#[test]
fn entire_collection_fails_if_one_authoritative_record_is_invalid() {
    let valid = base();
    let mut invalid = base();
    invalid.revision = id(5);
    invalid.nodes[2].definition = Definition::Bound {
        source: "x".into(),
        tokens: vec![],
    };
    assert!(restore_all(vec![valid.clone(), invalid], Limits::default()).is_err());
    let mut foreign = base();
    foreign.model = id(999);
    foreign.revision = id(5);
    assert!(restore_all(vec![valid.clone(), foreign], Limits::default()).is_err());
    let mut next = valid.clone();
    next.revision = id(5);
    assert_eq!(
        restore_all(vec![valid, next], Limits::default())
            .unwrap()
            .len(),
        2
    );
}

#[test]
fn collection_preserves_public_lineage_identity_across_historical_epochs() {
    let mut a = base();
    a.publication = Some(publication());
    let mut b = a.clone();
    b.revision = id(5);
    b.sequence = 43;
    b.epoch = id(99);
    b.publication.as_mut().unwrap().revision = id(203);
    b.publication.as_mut().unwrap().snapshot = id(204);
    assert_eq!(
        restore_all(vec![a.clone(), b.clone()], Limits::default())
            .unwrap()
            .len(),
        2
    );
    let mut changed = b.clone();
    changed.publication.as_mut().unwrap().public_ports[0].1 = "replacement".into();
    assert!(restore_all(vec![a.clone(), changed], Limits::default()).is_err());
    let mut changed = b.clone();
    changed.publication.as_mut().unwrap().snapshot = id(202);
    assert!(restore_all(vec![a.clone(), changed], Limits::default()).is_err());
    let mut changed = b;
    changed.ports[0].kind = PortKind::Output;
    assert!(restore_all(vec![a, changed], Limits::default()).is_err());
}
#[test]
fn codec_rejects_noncanonical_truncated_trailing_and_excess_budget_data() {
    let bytes = encode(&base(), Limits::default()).unwrap();
    for end in 0..bytes.len() {
        assert!(decode(&bytes[..end], Limits::default()).is_err());
    }
    let mut trailing = bytes.clone();
    trailing.push(0);
    assert_eq!(
        decode(&trailing, Limits::default()).unwrap_err(),
        Error::Malformed
    );
    let mut overlong = bytes.clone();
    overlong.splice(1..2, [0x18, 1]);
    assert_eq!(
        decode(&overlong, Limits::default()).unwrap_err(),
        Error::Malformed
    );
    for hostile in [
        vec![0x9f, 0xff],
        vec![0xbf, 0xff],
        vec![0xc0, 0x80],
        vec![0x9b, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff],
        vec![0xfa, 0, 0, 0, 0],
    ] {
        assert!(decode(&hostile, Limits::default()).is_err());
    }
    for limits in [
        Limits {
            bytes: bytes.len() - 1,
            ..Default::default()
        },
        Limits {
            depth: 2,
            ..Default::default()
        },
        Limits {
            items: 10,
            ..Default::default()
        },
        Limits {
            nodes: 2,
            ..Default::default()
        },
        Limits {
            identities: 2,
            ..Default::default()
        },
        Limits {
            work: 1,
            ..Default::default()
        },
    ] {
        assert!(decode(&bytes, limits).is_err());
        assert!(encode(&base(), limits).is_err());
    }
}

#[test]
fn frozen_v1_codec_compatibility_corpus() {
    let root = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/v1");
    for line in include_str!("fixtures/v1/manifest.tsv").lines() {
        let columns: Vec<_> = line.split('\t').collect();
        assert_eq!(columns.len(), 3);
        let hex = std::fs::read_to_string(root.join(columns[0])).unwrap();
        let text = hex.trim();
        assert_eq!(text.len() % 2, 0);
        let bytes = (0..text.len())
            .step_by(2)
            .map(|i| u8::from_str_radix(&text[i..i + 2], 16).unwrap())
            .collect::<Vec<_>>();
        match columns[1] {
            "ok" => {
                let record = decode(&bytes, Limits::default()).unwrap();
                assert_eq!(encode(&record, Limits::default()).unwrap(), bytes);
                let restored = restore(record.clone(), Limits::default()).unwrap();
                assert_eq!(restored.record(), &record);
                if record.publication.is_some() {
                    let mut expected = base();
                    expected.sequence = u64::MAX;
                    expected.publication = Some(publication());
                    assert_eq!(record, expected);
                    assert_eq!(number(&restored, id(12)), "75".parse().unwrap());
                } else {
                    assert_eq!(record.nodes.len(), 7);
                    assert_eq!(
                        record.nodes[6].definition,
                        Definition::Input(Decimal96::from_parts(0, 0, 0, true, 2).unwrap())
                    );
                    assert!(restored.diagnostics().len() >= 2);
                }
            }
            "decode" => assert_eq!(
                format!("{:?}", decode(&bytes, Limits::default()).unwrap_err()),
                columns[2]
            ),
            "restore" => {
                let record = decode(&bytes, Limits::default()).unwrap();
                match restore(record, Limits::default()) {
                    Ok(_) => panic!("invalid publication accepted"),
                    Err(e) => assert_eq!(format!("{e:?}"), columns[2]),
                }
            }
            _ => panic!("unknown fixture phase"),
        }
    }
}

#[test]
fn retired_port_identity_and_source_failure_states_are_captured() {
    let mut s = source();
    s.core.remove_port(s.ports[&id(102)]).unwrap();
    let bad = s.core.add_computed("Broken", "").unwrap();
    s.nodes.insert(id(13), bad);
    let unresolved = s.core.add_computed("Unresolved", "Future + 1").unwrap();
    s.nodes.insert(id(14), unresolved);
    let future = s.core.add_input("Future", "9".parse().unwrap()).unwrap();
    s.nodes.insert(id(15), future);
    let r = capture(&s.core, &s.nodes, &s.ports, header(), Limits::default()).unwrap();
    assert!(r.port_ids.contains(&id(102)));
    assert!(!r.ports.iter().any(|p| p.id == id(102)));
    assert!(matches!(
        r.nodes.iter().find(|n| n.id == id(13)).unwrap().definition,
        Definition::ParseFailed { .. }
    ));
    assert!(matches!(
        r.nodes.iter().find(|n| n.id == id(14)).unwrap().definition,
        Definition::BindFailed { .. }
    ));
    let a = restore(round(&r), Limits::default()).unwrap();
    assert!(
        a.diagnostics()
            .iter()
            .any(|d| d.kind == DiagnosticKind::Unresolved("Future".into()))
    );
}

#[test]
fn arbitrary_bounded_bytes_cannot_panic_and_accepted_bytes_are_canonical() {
    let mut state = 0x123456789abcdefu64;
    for length in 0..512 {
        let mut bytes = vec![];
        for _ in 0..length {
            state ^= state << 13;
            state ^= state >> 7;
            state ^= state << 17;
            bytes.push(state as u8);
        }
        if let Ok(record) = decode(&bytes, Limits::default()) {
            assert_eq!(encode(&record, Limits::default()).unwrap(), bytes);
        }
    }
}

#[test]
fn frozen_hosted_capture_rejects_selection_capability() {
    use pivot_core::model::{
        integration::{CancellationToken, IntegrationRuntime},
        selection::{Rectangle, SelectionId, SpaceId},
        transfer::ResolvedToken,
    };
    let runtime = IntegrationRuntime::standalone().unwrap();
    let mut core = WorkingModel::new_in("Range", &runtime).unwrap();
    let mut batch = core.begin_batch(4, 4096).unwrap();
    let node = batch.add_input("Sum", "0".parse().unwrap()).unwrap();
    let selection = SelectionId([1; 16]);
    batch
        .define_selection(
            selection,
            Rectangle {
                space: SpaceId([2; 16]),
                first_row: 1,
                last_row: 2,
                first_column: 1,
                last_column: 1,
            },
        )
        .unwrap();
    batch
        .set_resolved(node, "SUM", vec![ResolvedToken::Sum(selection)])
        .unwrap();
    core.apply_batch(batch, &CancellationToken::default())
        .unwrap();
    let result = capture(
        &core,
        &BTreeMap::from([(id(10), node)]),
        &BTreeMap::new(),
        Header {
            store: id(1),
            epoch: id(2),
            model: id(3),
            revision: id(4),
            sequence: 1,
            publication: None,
        },
        Limits::default(),
    );
    assert!(matches!(result, Err(Error::UnsupportedVersion)));
    drop(core);
    let memory = runtime.memory();
    assert!(runtime.finish().is_ok());
    assert_eq!(memory.stats().charged, 0);
}
