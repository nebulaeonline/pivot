use pivot_types::PivotDecimal;
use pivot_workbook::{producers::*, *};
fn snapshot(b: &CalculationCapture, id: ProducerId) -> &Snapshot {
    let Source::Fixture {
        selected: Some(s), ..
    } = &b.producer(id).unwrap().source
    else {
        panic!("no snapshot")
    };
    s
}

#[test]
fn external_field_slot_ceiling_is_aggregate_and_failure_retains_selected_payload() {
    use pivot_core::model::integration::IntegrationRuntime;
    let rt = IntegrationRuntime::standalone().unwrap();
    let mut b = Workbook::new_in("slot admission", Limits::capacity(), &rt).unwrap();
    let id = b
        .create_fixture_producer(b.expected(), Fixture::Ledger, KeyPolicy::Durable)
        .unwrap();
    let ticket = b.begin_refresh(b.expected(), id).unwrap();
    let rows = (0..32768)
        .map(|i| Row {
            key: format!("{i:08}"),
            values: vec![
                Value::Decimal(DecimalInput::from_decimal(1.into())),
                Value::String("0012".into()),
            ],
        })
        .collect();
    b.finish_refresh(&ticket, Some(rows), false).unwrap();
    drop(ticket);
    let old = b.capture_local(b.expected()).unwrap();
    assert_eq!(snapshot(&old, id).rows.len() * 2, MAX_SLOTS);
    let other = b
        .create_fixture_producer(b.expected(), Fixture::Ledger, KeyPolicy::Durable)
        .unwrap();
    let ticket = b.begin_refresh(b.expected(), other).unwrap();
    let before = b.expected();
    assert!(b.refresh_fixture(&ticket, 0).is_err());
    assert_eq!(b.expected(), before);
    assert_eq!(
        snapshot(&b.capture_local(b.expected()).unwrap(), id).generation,
        snapshot(&old, id).generation
    );
    b.finish_refresh(&ticket, None, false).unwrap();
    drop((ticket, old, b));
    assert!(rt.finish().is_ok());
}

#[test]
fn durable_absence_can_resume_but_new_generation_never_inherits_old_reference() {
    for policy in [KeyPolicy::Durable, KeyPolicy::GenerationQualified] {
        let mut b = Workbook::new("key lifecycle", Limits::default()).unwrap();
        let producer = b
            .create_fixture_producer(b.expected(), Fixture::Typed, policy)
            .unwrap();
        let ticket = b.begin_refresh(b.expected(), producer).unwrap();
        b.refresh_fixture(&ticket, 0).unwrap();
        drop(ticket);
        let window = b
            .create_bound_window(b.expected(), "Items", producer)
            .unwrap();
        b.apply(
            b.expected(),
            &[Edit::SetEntry {
                position: Position::new(1, 1),
                source: "=Items|A1".into(),
            }],
        )
        .unwrap();
        let before = b.capture_local(b.expected()).unwrap();
        let cell = before.cell_at(Position::new(1, 1)).unwrap();
        let frame = before
            .project_window(
                window,
                Rectangle {
                    first: Position::new(1, 1),
                    last: Position::new(1, 4),
                },
            )
            .unwrap();
        assert_eq!(
            frame.cells[1].value,
            Ok(pivot_types::PivotValue::String("0012".into()))
        );
        assert_eq!(
            frame.cells[2].value,
            Ok(pivot_types::PivotValue::Boolean(true))
        );
        assert_eq!(frame.cells[3].value, Ok(pivot_types::PivotValue::Blank));
        drop(frame);
        let expected = b.expected();
        assert!(
            b.apply(
                expected,
                &[Edit::SetEntry {
                    position: Position::new(2, 1),
                    source: "=Items|C1".into()
                }]
            )
            .is_err()
        );
        assert_eq!(b.expected(), expected);
        let ticket = b.begin_refresh(b.expected(), producer).unwrap();
        b.refresh_fixture(&ticket, 2).unwrap();
        drop(ticket);
        assert!(
            b.capture_local(b.expected())
                .unwrap()
                .evaluate(&[cell])
                .unwrap()
                .values[0]
                .1
                .is_err()
        );
        let ticket = b.begin_refresh(b.expected(), producer).unwrap();
        b.refresh_fixture(&ticket, 0).unwrap();
        drop(ticket);
        let value = b
            .capture_local(b.expected())
            .unwrap()
            .evaluate(&[cell])
            .unwrap()
            .values[0]
            .1
            .clone();
        if policy == KeyPolicy::Durable {
            assert_eq!(value, Ok(1.into()));
        } else {
            assert!(value.is_err());
        }
        assert_eq!(before.evaluate(&[cell]).unwrap().values[0].1, Ok(1.into()));
    }
}

#[test]
fn declared_producer_and_window_workload_coexists_with_retained_generations() {
    use pivot_core::model::integration::IntegrationRuntime;
    let rt = IntegrationRuntime::standalone().unwrap();
    let mut b = Workbook::new_in("coexistence", Limits::capacity(), &rt).unwrap();
    let GridScopeId::Worksheet(sheet) = b.worksheet() else {
        unreachable!()
    };
    let fixture = b
        .create_fixture_producer(b.expected(), Fixture::Ledger, KeyPolicy::Durable)
        .unwrap();
    let refresh = b.begin_refresh(b.expected(), fixture).unwrap();
    b.refresh_fixture(&refresh, 0).unwrap();
    drop(refresh);
    let mut names = Vec::new();
    for index in 0..32 {
        let producer = if index == 0 {
            fixture
        } else {
            let first = (index - 1) * 16 + 1;
            let edits = (first..first + 16)
                .flat_map(|row| {
                    (1..=2).map(move |column| Edit::SetEntry {
                        position: Position::new(row, column),
                        source: "1".into(),
                    })
                })
                .collect::<Vec<_>>();
            b.apply(b.expected(), &edits).unwrap();
            b.create_worksheet_producer(
                b.expected(),
                sheet,
                Rectangle {
                    first: Position::new(first, 1),
                    last: Position::new(first + 15, 2),
                },
            )
            .unwrap()
        };
        for view in 0..2 {
            let name = format!("Source{index}View{view}");
            b.create_bound_window(b.expected(), &name, producer)
                .unwrap();
            names.push(name);
        }
    }
    let edits = (0..64)
        .map(|i| Edit::SetEntry {
            position: Position::new(600 + i, 1),
            source: if i < 32 {
                format!("=Source0View0|A{}", i + 1)
            } else if i < 34 {
                "=Source0View1|A1".into()
            } else {
                format!("={}|A1", names[2 + (i as usize - 32) % 62])
            },
        })
        .collect::<Vec<_>>();
    b.apply(b.expected(), &edits).unwrap();
    assert_eq!(b.producers().len(), 32);
    assert_eq!(b.stats().live_nodes, 992 + 64 + 32);
    let old = b.capture_local(b.expected()).unwrap();
    let first = b.begin_refresh(b.expected(), fixture).unwrap();
    let second = b.begin_refresh(b.expected(), fixture).unwrap();
    assert!(b.begin_refresh(b.expected(), fixture).is_err());
    b.refresh_fixture(&second, 1).unwrap();
    assert_eq!(snapshot(&old, fixture).rows[0].key, "0012");
    assert!(b.refresh_fixture(&first, 2).is_err());
    drop((first, second, old, b));
    assert!(rt.finish().is_ok());
}
#[test]
fn explicit_refresh_status_and_old_snapshot_are_independent() {
    let mut b = Workbook::new("producer", Limits::default()).unwrap();
    let id = b
        .create_fixture_producer(b.expected(), Fixture::Ledger, KeyPolicy::Durable)
        .unwrap();
    let graph = b.expected().graph;
    let t = b.begin_refresh(b.expected(), id).unwrap();
    assert_eq!(b.expected().graph, graph);
    b.refresh_fixture(&t, 0).unwrap();
    let old = b.capture_local(b.expected()).unwrap();
    let generation = snapshot(&old, id).generation;
    assert_eq!(snapshot(&old, id).rows[0].key, "0012");
    assert_eq!(
        snapshot(&old, id).rows[0].values[1],
        Value::String("0012".into())
    );
    drop(t);
    let t = b.begin_refresh(b.expected(), id).unwrap();
    let pending = b.expected();
    b.apply(
        b.expected(),
        &[Edit::SetEntry {
            position: Position::new(1, 1),
            source: "7".into(),
        }],
    )
    .unwrap();
    assert_ne!(b.expected(), pending);
    b.finish_refresh(&t, None, false).unwrap();
    let failed = b.capture_local(b.expected()).unwrap();
    assert_eq!(snapshot(&failed, id).generation, generation);
    assert!(
        matches!(&failed.producer(id).unwrap().source, Source::Fixture { attempt, .. } if attempt.status == AttemptStatus::Failed)
    );
    let before_replay = b.expected();
    assert!(b.refresh_fixture(&t, 1).is_err());
    assert_eq!(b.expected(), before_replay);
    drop(t);
    let t = b.begin_refresh(b.expected(), id).unwrap();
    b.refresh_fixture(&t, 1).unwrap();
    let current = b.capture_local(b.expected()).unwrap();
    assert_ne!(snapshot(&current, id).generation, generation);
    assert_eq!(snapshot(&old, id).rows[0].key, "0012");
    assert_eq!(snapshot(&current, id).rows[127].key, "0012");
}
#[test]
fn superseded_tasks_remain_admitted_until_ticket_release() {
    let mut b = Workbook::new("attempts", Limits::default()).unwrap();
    let id = b
        .create_fixture_producer(b.expected(), Fixture::Typed, KeyPolicy::GenerationQualified)
        .unwrap();
    let t1 = b.begin_refresh(b.expected(), id).unwrap();
    let t2 = b.begin_refresh(b.expected(), id).unwrap();
    let before = b.expected();
    assert!(b.begin_refresh(before, id).is_err());
    assert_eq!(b.expected(), before);
    assert!(b.refresh_fixture(&t1, 0).is_err());
    assert_eq!(b.expected(), before);
    b.refresh_fixture(&t2, 1).unwrap();
    assert!(b.refresh_fixture(&t1, 0).is_err());
    drop((t1, t2));
    let t3 = b.begin_refresh(b.expected(), id).unwrap();
    b.finish_refresh(&t3, None, true).unwrap();
    assert!(
        matches!(&b.producers()[0].source, Source::Fixture { attempt, selected: Some(_), .. } if attempt.status == AttemptStatus::Cancelled)
    );
}
#[test]
fn malformed_payload_is_atomic_and_disconnect_retains_values() {
    let mut b = Workbook::new("validation", Limits::default()).unwrap();
    let id = b
        .create_fixture_producer(b.expected(), Fixture::Ledger, KeyPolicy::Durable)
        .unwrap();
    let t = b.begin_refresh(b.expected(), id).unwrap();
    let mut duplicate = Fixture::Ledger.rows(0).unwrap();
    duplicate[1].key = duplicate[0].key.clone();
    let before = b.expected();
    assert!(b.finish_refresh(&t, Some(duplicate), false).is_err());
    assert_eq!(b.expected(), before);
    b.refresh_fixture(&t, 0).unwrap();
    let old = b.capture_local(b.expected()).unwrap();
    drop(t);
    let t = b.begin_refresh(b.expected(), id).unwrap();
    b.disconnect_producer(b.expected(), id).unwrap();
    assert!(b.refresh_fixture(&t, 1).is_err());
    assert!(b.begin_refresh(b.expected(), id).is_err());
    assert_eq!(
        snapshot(&b.capture_local(b.expected()).unwrap(), id).generation,
        snapshot(&old, id).generation
    );
    assert!(
        b.reconnect_producer(b.expected(), id, Fixture::Typed)
            .is_err()
    );
    b.reconnect_producer(b.expected(), id, Fixture::Ledger)
        .unwrap();
}
#[test]
fn shared_bound_views_allocate_no_cells_and_refuse_mutation() {
    let mut b = Workbook::new("views", Limits::default()).unwrap();
    let id = b
        .create_fixture_producer(b.expected(), Fixture::Ledger, KeyPolicy::Durable)
        .unwrap();
    let a = b.create_bound_window(b.expected(), "A", id).unwrap();
    let _c = b.create_bound_window(b.expected(), "C", id).unwrap();
    assert_eq!(b.stats().live_cells, 0);
    assert_eq!(b.stats().live_nodes, 0);
    let before = b.expected();
    assert!(
        b.apply_scope(
            a.into(),
            before,
            &[Edit::SetEntry {
                position: Position::new(1, 1),
                source: "8".into()
            }]
        )
        .is_err()
    );
    assert!(
        b.restructure(
            a.into(),
            before,
            structure::Change {
                axis: structure::AxisKind::Rows,
                action: structure::Action::Insert,
                start: 1,
                count: 1
            },
            &Default::default()
        )
        .is_err()
    );
    assert_eq!(b.expected(), before);
}

#[test]
fn worksheet_projection_keeps_physical_holes_and_old_geometry() {
    let mut b = Workbook::new("source", Limits::default()).unwrap();
    let GridScopeId::Worksheet(sheet) = b.worksheet() else {
        unreachable!()
    };
    b.apply(
        b.expected(),
        &[
            Edit::SetEntry {
                position: Position::new(2, 2),
                source: "2".into(),
            },
            Edit::SetEntry {
                position: Position::new(4, 2),
                source: "'0012".into(),
            },
        ],
    )
    .unwrap();
    let id = b
        .create_worksheet_producer(
            b.expected(),
            sheet,
            Rectangle {
                first: Position::new(2, 2),
                last: Position::new(4, 2),
            },
        )
        .unwrap();
    let window = b.create_bound_window(b.expected(), "Items", id).unwrap();
    let old = b.capture_local(b.expected()).unwrap();
    let area = Rectangle {
        first: Position::new(1, 1),
        last: Position::new(3, 1),
    };
    let frame = old.project_window(window, area).unwrap();
    assert_eq!(frame.cells.len(), 3);
    assert_eq!(frame.cells[1].value, Ok(pivot_types::PivotValue::Blank));
    assert!(frame.cells[1].source.is_none());
    assert_eq!(
        frame.cells[2].value,
        Ok(pivot_types::PivotValue::String("0012".into()))
    );
    assert_eq!(b.stats().live_cells, 2);
    b.restructure(
        sheet.into(),
        b.expected(),
        structure::Change {
            axis: structure::AxisKind::Rows,
            action: structure::Action::Insert,
            start: 3,
            count: 1,
        },
        &Default::default(),
    )
    .unwrap();
    let current = b.capture_local(b.expected()).unwrap();
    let frame = current
        .project_window(
            window,
            Rectangle {
                first: Position::new(1, 1),
                last: Position::new(4, 1),
            },
        )
        .unwrap();
    assert_eq!(frame.rows, 4);
    assert_eq!(
        frame.cells[3].value,
        Ok(pivot_types::PivotValue::String("0012".into()))
    );
    assert_eq!(old.project_window(window, area).unwrap().rows, 3);
    b.restructure(
        sheet.into(),
        b.expected(),
        structure::Change {
            axis: structure::AxisKind::Rows,
            action: structure::Action::Delete,
            start: 2,
            count: 4,
        },
        &Default::default(),
    )
    .unwrap();
    assert_eq!(
        b.capture_local(b.expected())
            .unwrap()
            .project_window(window, area)
            .unwrap()
            .rows,
        0
    );
    assert_eq!(old.project_window(window, area).unwrap().rows, 3);
}

#[test]
fn scalar_guards_follow_identity_and_reject_producer_feedback() {
    let mut b = Workbook::new("guards", Limits::default()).unwrap();
    let GridScopeId::Worksheet(sheet) = b.worksheet() else {
        unreachable!()
    };
    b.apply(
        b.expected(),
        &[
            Edit::SetEntry {
                position: Position::new(2, 2),
                source: "2".into(),
            },
            Edit::SetEntry {
                position: Position::new(3, 2),
                source: "5".into(),
            },
            Edit::SetEntry {
                position: Position::new(4, 2),
                source: "3".into(),
            },
            Edit::SetEntry {
                position: Position::new(1, 3),
                source: "1".into(),
            },
        ],
    )
    .unwrap();
    let id = b
        .create_worksheet_producer(
            b.expected(),
            sheet,
            Rectangle {
                first: Position::new(2, 2),
                last: Position::new(4, 2),
            },
        )
        .unwrap();
    b.create_bound_window(b.expected(), "Items", id).unwrap();
    b.apply(
        b.expected(),
        &[Edit::SetEntry {
            position: Position::new(10, 1),
            source: "=Items|A2+C1".into(),
        }],
    )
    .unwrap();
    let old = b.capture_local(b.expected()).unwrap();
    let output = old.cell_at(Position::new(10, 1)).unwrap();
    assert_eq!(
        old.evaluate(&[output]).unwrap().values[0].1,
        Ok(PivotDecimal::from(6))
    );
    let before = b.expected();
    assert!(
        b.apply(
            before,
            &[Edit::SetEntry {
                position: Position::new(2, 2),
                source: "=A10".into()
            }]
        )
        .is_err()
    );
    assert_eq!(b.expected(), before);
    b.apply(
        b.expected(),
        &[Edit::Move {
            from: Position::new(3, 2),
            to: Position::new(8, 2),
        }],
    )
    .unwrap();
    assert!(
        b.capture_local(b.expected())
            .unwrap()
            .evaluate(&[output])
            .unwrap()
            .values[0]
            .1
            .is_err()
    );
    assert!(
        b.display_formula(output)
            .unwrap()
            .unwrap()
            .contains("#REF!")
    );
    b.apply(
        b.expected(),
        &[Edit::Move {
            from: Position::new(8, 2),
            to: Position::new(3, 2),
        }],
    )
    .unwrap();
    assert_eq!(
        b.capture_local(b.expected())
            .unwrap()
            .evaluate(&[output])
            .unwrap()
            .values[0]
            .1,
        Ok(PivotDecimal::from(6))
    );
    assert!(
        b.display_formula(output)
            .unwrap()
            .unwrap()
            .contains("Items|A2")
    );
    b.apply(
        b.expected(),
        &[
            Edit::Retire(Position::new(3, 2)),
            Edit::SetEntry {
                position: Position::new(3, 2),
                source: "5".into(),
            },
        ],
    )
    .unwrap();
    assert!(
        b.capture_local(b.expected())
            .unwrap()
            .evaluate(&[output])
            .unwrap()
            .values[0]
            .1
            .is_err()
    );
    assert_eq!(
        old.evaluate(&[output]).unwrap().values[0].1,
        Ok(PivotDecimal::from(6))
    );
}

#[test]
fn combined_scalar_and_spatial_ranges_preserve_blank_slots() {
    let mut b = Workbook::new("combined", Limits::default()).unwrap();
    let GridScopeId::Worksheet(sheet) = b.worksheet() else {
        unreachable!()
    };
    for (row, value) in [(2, "2"), (3, "5"), (4, "3")] {
        b.apply(
            b.expected(),
            &[Edit::SetEntry {
                position: Position::new(row, 2),
                source: value.into(),
            }],
        )
        .unwrap();
    }
    let producer = b
        .create_worksheet_producer(
            b.expected(),
            sheet,
            Rectangle {
                first: Position::new(2, 2),
                last: Position::new(4, 2),
            },
        )
        .unwrap();
    b.create_bound_window(b.expected(), "Items", producer)
        .unwrap();
    b.apply(
        b.expected(),
        &[
            Edit::SetEntry {
                position: Position::new(10, 1),
                source: "=Items|A2".into(),
            },
            Edit::SetEntry {
                position: Position::new(11, 1),
                source: "=SUM(Items|A1:A3)".into(),
            },
            Edit::SetEntry {
                position: Position::new(12, 1),
                source: "=ALLOCATEWEIGHTEDRANGE(10,2,2,Items|A1:A3)".into(),
            },
        ],
    )
    .unwrap();
    let old = b.capture_local(b.expected()).unwrap();
    let outputs = [10, 11, 12].map(|row| old.cell_at(Position::new(row, 1)).unwrap());
    let values = old.evaluate(&outputs).unwrap().values;
    assert_eq!(
        values
            .iter()
            .map(|v| v.1.clone().unwrap().to_string())
            .collect::<Vec<_>>(),
        ["5", "10", "5"]
    );
    b.restructure(
        sheet.into(),
        b.expected(),
        structure::Change {
            axis: structure::AxisKind::Rows,
            action: structure::Action::Insert,
            start: 3,
            count: 1,
        },
        &Default::default(),
    )
    .unwrap();
    let current = b.capture_local(b.expected()).unwrap();
    let values = current.evaluate(&outputs).unwrap().values;
    assert_eq!(values[0].1, Ok(PivotDecimal::from(5)));
    assert_eq!(values[1].1, Ok(PivotDecimal::from(10)));
    assert!(values[2].1.is_err());
    assert!(
        b.display_formula(outputs[1])
            .unwrap()
            .unwrap()
            .contains("Items|A1:A4")
    );
    b.restructure(
        sheet.into(),
        b.expected(),
        structure::Change {
            axis: structure::AxisKind::Rows,
            action: structure::Action::Delete,
            start: 2,
            count: 4,
        },
        &Default::default(),
    )
    .unwrap();
    let values = b
        .capture_local(b.expected())
        .unwrap()
        .evaluate(&outputs)
        .unwrap()
        .values;
    assert!(values.iter().all(|v| v.1.is_err()));
    assert!(
        old.evaluate(&outputs)
            .unwrap()
            .values
            .iter()
            .all(|v| v.1.is_ok())
    );
}

#[test]
fn external_shared_leaves_follow_keys_and_pin_contexts() {
    let mut b = Workbook::new("external", Limits::default()).unwrap();
    let producer = b
        .create_fixture_producer(b.expected(), Fixture::Ledger, KeyPolicy::Durable)
        .unwrap();
    let ticket = b.begin_refresh(b.expected(), producer).unwrap();
    b.refresh_fixture(&ticket, 0).unwrap();
    drop(ticket);
    b.create_bound_window(b.expected(), "First", producer)
        .unwrap();
    b.create_bound_window(b.expected(), "Second", producer)
        .unwrap();
    b.apply(
        b.expected(),
        &[
            Edit::SetEntry {
                position: Position::new(1, 1),
                source: "=First|A1+Second|A1".into(),
            },
            Edit::SetEntry {
                position: Position::new(2, 1),
                source: "=First|A2".into(),
            },
        ],
    )
    .unwrap();
    assert_eq!(b.stats().live_nodes, 4); // two authored cells and two shared leaves
    let old = b.capture_local(b.expected()).unwrap();
    let outputs = [
        old.cell_at(Position::new(1, 1)).unwrap(),
        old.cell_at(Position::new(2, 1)).unwrap(),
    ];
    assert_eq!(
        old.evaluate(&outputs).unwrap().values[0].1,
        Ok(PivotDecimal::from(2))
    );
    let ticket = b.begin_refresh(b.expected(), producer).unwrap();
    let before = b.expected();
    b.refresh_fixture(&ticket, 1).unwrap();
    assert_ne!(b.expected().graph, before.graph);
    assert_eq!(
        b.capture_local(b.expected())
            .unwrap()
            .evaluate(&outputs)
            .unwrap()
            .values[0]
            .1,
        Ok(PivotDecimal::from(4))
    );
    assert!(
        b.display_formula(outputs[0])
            .unwrap()
            .unwrap()
            .contains("A128")
    );
    assert_eq!(
        old.evaluate(&outputs).unwrap().values[0].1,
        Ok(PivotDecimal::from(2))
    );
    drop(ticket);
    let ticket = b.begin_refresh(b.expected(), producer).unwrap();
    let before = b.expected();
    b.finish_refresh(&ticket, None, false).unwrap();
    assert_eq!(b.expected().graph, before.graph);
    assert_eq!(
        b.capture_local(b.expected())
            .unwrap()
            .evaluate(&outputs)
            .unwrap()
            .values[0]
            .1,
        Ok(PivotDecimal::from(4))
    );
    let before = b.expected();
    assert!(
        b.apply(
            before,
            &[Edit::SetEntry {
                position: Position::new(3, 1),
                source: "=First|B1".into()
            }]
        )
        .is_err()
    );
    assert_eq!(b.expected(), before);
}

#[test]
fn portable_producer_authority_round_trips_without_refresh_capability() {
    use pivot_core::model::integration::IntegrationRuntime;
    let rt = IntegrationRuntime::standalone().unwrap();
    let mut b = Workbook::new_in("portable", Limits::default(), &rt).unwrap();
    let id = b
        .create_fixture_producer(b.expected(), Fixture::Ledger, KeyPolicy::Durable)
        .unwrap();
    let t = b.begin_refresh(b.expected(), id).unwrap();
    b.refresh_fixture(&t, 0).unwrap();
    drop(t);
    let window = b.create_bound_window(b.expected(), "External", id).unwrap();
    b.apply(
        b.expected(),
        &[Edit::SetEntry {
            position: Position::new(1, 1),
            source: "=External|A1".into(),
        }],
    )
    .unwrap();
    let (mut image, lease) = b
        .project_document(b.expected(), &Default::default())
        .unwrap();
    let encoded = codec::encode(&image.producers).unwrap();
    image.producers = codec::decode(&encoded).unwrap();
    let restored = Workbook::restore_portable(image, Limits::default(), &rt).unwrap();
    assert!(
        matches!(&restored.producers()[0].source,Source::Fixture{attempt,..} if attempt.status==AttemptStatus::Disconnected)
    );
    let capture = restored.capture_local(restored.expected()).unwrap();
    let output = capture.cell_at(Position::new(1, 1)).unwrap();
    assert_eq!(
        capture.evaluate(&[output]).unwrap().values[0].1,
        Ok(PivotDecimal::from(1))
    );
    assert_eq!(
        capture
            .project_window(
                window,
                Rectangle {
                    first: Position::new(1, 1),
                    last: Position::new(1, 2)
                }
            )
            .unwrap()
            .cells[1]
            .value,
        Ok(pivot_types::PivotValue::String("0012".into()))
    );
    drop((capture, restored, lease, b));
    assert!(rt.finish().is_ok());
}

#[test]
fn copying_temporary_guard_fault_preserves_identity_and_absolute_flags() {
    let mut b = Workbook::new("fault copies", Limits::default()).unwrap();
    let GridScopeId::Worksheet(sheet) = b.worksheet() else {
        unreachable!()
    };
    b.apply(
        b.expected(),
        &[Edit::SetEntry {
            position: Position::new(3, 2),
            source: "5".into(),
        }],
    )
    .unwrap();
    let p = b
        .create_worksheet_producer(
            b.expected(),
            sheet,
            Rectangle {
                first: Position::new(2, 2),
                last: Position::new(4, 2),
            },
        )
        .unwrap();
    b.create_bound_window(b.expected(), "Items", p).unwrap();
    b.apply(
        b.expected(),
        &[Edit::SetEntry {
            position: Position::new(10, 1),
            source: "=Items|$A$2".into(),
        }],
    )
    .unwrap();
    b.apply(
        b.expected(),
        &[Edit::Move {
            from: Position::new(3, 2),
            to: Position::new(8, 2),
        }],
    )
    .unwrap();
    b.copy_between_scopes(
        b.expected(),
        sheet.into(),
        Rectangle {
            first: Position::new(10, 1),
            last: Position::new(10, 1),
        },
        sheet.into(),
        Position::new(20, 1),
        &Default::default(),
    )
    .unwrap();
    let capture = b.capture_local(b.expected()).unwrap();
    let copy = capture.cell_at(Position::new(20, 1)).unwrap();
    assert!(capture.evaluate(&[copy]).unwrap().values[0].1.is_err());
    b.apply(
        b.expected(),
        &[Edit::Move {
            from: Position::new(8, 2),
            to: Position::new(3, 2),
        }],
    )
    .unwrap();
    assert_eq!(
        b.capture_local(b.expected())
            .unwrap()
            .evaluate(&[copy])
            .unwrap()
            .values[0]
            .1,
        Ok(PivotDecimal::from(5))
    );
    assert!(
        b.display_formula(copy)
            .unwrap()
            .unwrap()
            .contains("Items|$A$2")
    );
    let rt = pivot_core::model::integration::IntegrationRuntime::standalone().unwrap();
    let image = b.export_portable(b.expected()).unwrap();
    let restored = Workbook::restore_portable(image, Limits::default(), &rt).unwrap();
    assert_eq!(
        restored
            .capture_local(restored.expected())
            .unwrap()
            .evaluate(&[copy])
            .unwrap()
            .values[0]
            .1,
        Ok(PivotDecimal::from(5))
    );
    drop(restored);
    assert!(rt.finish().is_ok());
}
