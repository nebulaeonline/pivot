#![cfg(feature = "history-experiment")]
use pivot_core::model::integration::{CancellationToken, IntegrationRuntime};
use pivot_workbook::{
    history_experiment::{
        ColdLedger,
        changes::{Mutation, RecordImage},
    },
    *,
};
fn p(r: u32, c: u32) -> Position {
    Position::new(r, c)
}
fn e(r: u32, c: u32, s: &str) -> Edit {
    Edit::SetEntry {
        position: p(r, c),
        source: s.into(),
    }
}
fn step(
    book: &mut Workbook,
    ledger: &mut ColdLedger,
    mutation: Mutation<'_>,
    rt: &IntegrationRuntime,
) {
    let before = book.expected();
    let content = book.history_content_id();
    let floors = book.history_metrics().unwrap();
    let prior = book.history_cold_export(before, ledger).unwrap();
    let image = RecordImage::from_checkpoint(&prior, &rt.memory()).unwrap();
    let Some(prepared) = book
        .prepare_record_changes(before, ledger, mutation, &Default::default())
        .unwrap()
    else {
        assert_eq!(book.expected(), before);
        assert_eq!(book.history_content_id(), content);
        return;
    };
    assert_eq!(book.expected(), before);
    assert_eq!(book.history_content_id(), content);
    assert_eq!(book.history_metrics().unwrap(), floors);
    let mut next = ledger.clone_for_oracle(&rt.memory()).unwrap();
    next.observe(prepared.candidate()).unwrap();
    let full = prepared
        .candidate()
        .history_cold_export(prepared.candidate().expected(), &next)
        .unwrap();
    let folded = image.apply(&prepared, &rt.memory()).unwrap();
    let rebuilt = folded.checkpoint(&rt.memory()).unwrap();
    assert_eq!(rebuilt.bytes, full.bytes, "{:?}", prepared.metrics());
    let (cold, cold_ledger) =
        Workbook::history_cold_reopen(&full.bytes, &rebuilt.bytes, Limits::default(), rt).unwrap();
    for r in 1..=12 {
        for c in 1..=12 {
            assert_eq!(
                cold.cell_at(p(r, c)).unwrap(),
                prepared.candidate().cell_at(p(r, c)).unwrap()
            );
        }
    }
    assert_eq!(
        cold.validate(cold.expected()).unwrap(),
        prepared
            .candidate()
            .validate(prepared.candidate().expected())
            .unwrap()
    );
    if cold.validate(cold.expected()).unwrap().is_empty() {
        let refs: Vec<_> = (1..=12)
            .flat_map(|r| (1..=12).map(move |c| p(r, c)))
            .filter_map(|position| cold.cell_at(position).unwrap())
            .filter(|cell| !matches!(cell.content, Content::LiteralText(_)))
            .map(|cell| cell.reference)
            .collect();
        let a = cold.capture_local(cold.expected()).unwrap();
        let b = prepared
            .candidate()
            .capture_local(prepared.candidate().expected())
            .unwrap();
        assert_eq!(
            a.evaluate(&refs).unwrap().values,
            b.evaluate(&refs).unwrap().values
        );
    }
    drop((cold, cold_ledger));
    prepared.publish(book, ledger).unwrap();
    assert_eq!(
        book.history_cold_export(book.expected(), ledger)
            .unwrap()
            .bytes,
        full.bytes
    );
}
#[test]
fn exact_checkpoint_differential_for_basic_mutations() {
    let rt = IntegrationRuntime::standalone().unwrap();
    let mut b = Workbook::new_in("G2", Limits::default(), &rt).unwrap();
    let mut l = ColdLedger::new(&b).unwrap();
    for edits in [
        vec![e(1, 1, "10")],
        vec![e(1, 2, "=A1+1")],
        vec![e(1, 1, "20")],
        vec![Edit::SetFormula {
            position: p(2, 5),
            source: "=A1+3".into(),
            tokens: vec![
                Token::At(p(1, 1)),
                Token::Decimal(DecimalInput::from_decimal(3.into())),
                Token::Binary(pivot_core::expression::BinaryOp::Add),
            ],
        }],
        vec![e(1, 3, "=SUM(A1:A3)")],
        vec![e(2, 3, "=AVERAGE(A1:A3)")],
        vec![Edit::Move {
            from: p(1, 1),
            to: p(4, 1),
        }],
        vec![e(3, 4, "=("), e(3, 5, "=E3"), e(4, 4, "text")],
        vec![Edit::Retire(p(4, 1))],
        vec![Edit::Clear(p(1, 3))],
        vec![e(3, 4, "30"), e(3, 5, "40")],
        vec![Edit::SetDecimal {
            position: p(6, 6),
            value: DecimalInput::from_parts(0, 0, 0, true, 5).unwrap(),
        }],
        vec![e(6, 7, "=A7"), Edit::Retire(p(7, 1))],
        vec![e(8, 8, "5"), e(8, 8, "6"), e(8, 8, "7")],
    ] {
        step(&mut b, &mut l, Mutation::Edits(&edits), &rt);
    }
    drop((b, l));
    rt.flush();
    let memory = rt.memory();
    assert!(rt.finish().is_ok());
    assert_eq!(memory.stats().charged, 0);
}

fn rect(r: u32, c: u32, rr: u32, cc: u32) -> Rectangle {
    Rectangle {
        first: p(r, c),
        last: p(rr, cc),
    }
}
fn finish(rt: IntegrationRuntime) {
    rt.flush();
    let memory = rt.memory();
    assert!(rt.finish().is_ok());
    assert_eq!(memory.stats().charged, 0);
}
#[test]
fn all_fill_directions_native_external_paste_and_copy_are_one_coalesced_batch() {
    let rt = IntegrationRuntime::standalone().unwrap();
    for direction in [
        FillDirection::Down,
        FillDirection::Up,
        FillDirection::Right,
        FillDirection::Left,
    ] {
        let mut b = Workbook::new_in("G2 fill", Limits::default(), &rt).unwrap();
        let row = if direction == FillDirection::Up { 4 } else { 2 };
        let col = if direction == FillDirection::Left {
            4
        } else {
            2
        };
        b.apply(
            b.expected(),
            &[
                e(row, col, &format!("=SUM($J$1:$J$2)+H{row}")),
                e(1, 10, "2"),
            ],
        )
        .unwrap();
        let mut l = ColdLedger::new(&b).unwrap();
        let mut foreign = Workbook::new_in("G2 cross document", Limits::default(), &rt).unwrap();
        foreign
            .apply(
                foreign.expected(),
                &[e(1, 1, "1.2300"), e(1, 2, "=A1+2"), e(1, 3, "text")],
            )
            .unwrap();
        let imported = foreign
            .copy_block(foreign.expected(), rect(1, 1, 1, 3))
            .unwrap();
        step(
            &mut b,
            &mut l,
            Mutation::Paste {
                rectangle: rect(5, 4, 5, 6),
                bytes: &imported.native,
                native: true,
            },
            &rt,
        );
        assert_ne!(
            b.cell_at(p(5, 4)).unwrap().unwrap().reference.cell,
            foreign.cell_at(p(1, 1)).unwrap().unwrap().reference.cell
        );
        drop(foreign);
        step(
            &mut b,
            &mut l,
            Mutation::Fill(rect(2, 2, 4, 4), direction),
            &rt,
        );
        let unchanged = b.expected();
        assert!(
            b.prepare_record_changes(
                unchanged,
                &l,
                Mutation::Fill(rect(2, 2, 4, 4), direction),
                &Default::default()
            )
            .unwrap()
            .is_none()
        );
        // Fill's duplication seam may stage the same logical target twice. Both
        // allocated range identities must survive as final live/retired records,
        // while the cell/definition each coalesce to one final operation.
        step(
            &mut b,
            &mut l,
            Mutation::Duplicate(&[(p(row, col), p(12, 10)), (p(row, col), p(12, 10))]),
            &rt,
        );
        let block = b.copy_block(b.expected(), rect(2, 2, 3, 3)).unwrap();
        step(
            &mut b,
            &mut l,
            Mutation::Paste {
                rectangle: rect(6, 3, 7, 4),
                bytes: &block.native,
                native: true,
            },
            &rt,
        );
        step(
            &mut b,
            &mut l,
            Mutation::Paste {
                rectangle: rect(9, 7, 10, 8),
                bytes: b"1\t2\n3\t4",
                native: false,
            },
            &rt,
        );
        step(
            &mut b,
            &mut l,
            Mutation::Duplicate(&[(p(9, 7), p(11, 7)), (p(9, 7), p(11, 7))]),
            &rt,
        );
        let edits = [Edit::Retire(p(9, 7)), e(9, 7, "new identity")];
        step(&mut b, &mut l, Mutation::Edits(&edits), &rt);
        drop((b, l));
        rt.flush();
    }
    finish(rt);
}
#[test]
fn original_state_coalescing_failed_noop_cancelled_admission_and_stale_candidates() {
    let rt = IntegrationRuntime::standalone().unwrap();
    let mut b = Workbook::new_in("G2 failures", Limits::default(), &rt).unwrap();
    b.apply(b.expected(), &[e(1, 1, "10"), e(1, 2, "=A1+1")])
        .unwrap();
    let mut l = ColdLedger::new(&b).unwrap();
    let expected = b.expected();
    let source = b.history_cold_export(expected, &l).unwrap();
    let content = b.history_content_id();
    let capture = b.capture_local(expected).unwrap();
    let cell = capture.cell_at(p(1, 2)).unwrap();
    let transient = [e(1, 1, "20"), e(1, 1, "10")];
    assert!(
        b.prepare_record_changes(
            expected,
            &l,
            Mutation::Edits(&transient),
            &Default::default()
        )
        .unwrap()
        .is_none()
    );
    let failed = [
        e(5, 5, "private only"),
        Edit::Move {
            from: p(12, 12),
            to: p(11, 11),
        },
    ];
    assert!(
        b.prepare_record_changes(expected, &l, Mutation::Edits(&failed), &Default::default())
            .is_err()
    );
    for (bytes, native) in [
        (b"not a native clipboard".as_slice(), true),
        (b"1\t2\n3".as_slice(), false),
    ] {
        assert!(
            b.prepare_record_changes(
                expected,
                &l,
                Mutation::Paste {
                    rectangle: rect(4, 4, 5, 5),
                    bytes,
                    native
                },
                &Default::default()
            )
            .is_err()
        );
    }
    let cancel = CancellationToken::default();
    cancel.cancel();
    assert!(
        b.prepare_record_changes(expected, &l, Mutation::Edits(&[e(1, 1, "30")]), &cancel)
            .is_err()
    );
    let mut stale = expected;
    stale.state += 1;
    assert!(
        b.prepare_record_changes(
            stale,
            &l,
            Mutation::Edits(&[e(1, 1, "30")]),
            &Default::default()
        )
        .is_err()
    );
    rt.flush();
    let memory = rt.memory();
    let stats = memory.stats();
    let pressure = memory.reserve(stats.limit - stats.charged).unwrap();
    assert!(
        b.prepare_record_changes(
            expected,
            &l,
            Mutation::Edits(&[e(1, 1, "30")]),
            &Default::default()
        )
        .is_err()
    );
    drop(pressure);
    rt.flush();
    assert_eq!(b.expected(), expected);
    assert_eq!(b.history_content_id(), content);
    assert_eq!(
        b.history_cold_export(expected, &l).unwrap().bytes,
        source.bytes
    );
    let repeated = [e(1, 1, "20"), e(1, 1, "30"), e(1, 1, "40")];
    let prepared = b
        .prepare_record_changes(
            expected,
            &l,
            Mutation::Edits(&repeated),
            &Default::default(),
        )
        .unwrap()
        .unwrap();
    use pivot_workbook::history_experiment::changes::RecordKind;
    assert_eq!(
        prepared
            .operations()
            .iter()
            .filter(|o| o.kind() == RecordKind::Cell)
            .count(),
        1
    );
    assert_eq!(
        prepared
            .operations()
            .iter()
            .filter(|o| o.kind() == RecordKind::Definition)
            .count(),
        1
    );
    assert!(
        prepared
            .operations()
            .windows(2)
            .all(|w| w[0].key() < w[1].key())
    );
    b.apply(expected, &[e(1, 1, "50")]).unwrap();
    l.observe(&b).unwrap();
    let fresh = b.expected();
    assert!(prepared.publish(&mut b, &mut l).is_err());
    assert_eq!(b.expected(), fresh);
    assert_eq!(
        capture.evaluate(&[cell]).unwrap().values[0].1,
        Ok(11.into())
    );
    step(&mut b, &mut l, Mutation::Edits(&[e(1, 1, "60")]), &rt);
    assert_eq!(
        capture.evaluate(&[cell]).unwrap().values[0].1,
        Ok(11.into())
    );
    drop((capture, source, b, l));
    finish(rt);
}
#[test]
fn undo_branch_lifetime_guards_and_cold_namespace_remapping() {
    let rt = IntegrationRuntime::standalone().unwrap();
    let mut b = Workbook::new_in("G2 branch", Limits::default(), &rt).unwrap();
    b.apply(b.expected(), &[e(1, 1, "10"), e(1, 2, "=A1+1")])
        .unwrap();
    let mut l = ColdLedger::new(&b).unwrap();
    let root_b = b.pin_history_root(b.expected()).unwrap();
    let cold_b = b.history_cold_export(b.expected(), &l).unwrap();
    let stale_image = RecordImage::from_checkpoint(&cold_b, &rt.memory()).unwrap();
    b.apply(b.expected(), &[e(9, 1, "D-only"), e(9, 2, "=SUM(A1:A2)")])
        .unwrap();
    l.observe(&b).unwrap();
    let root_d = b.pin_history_root(b.expected()).unwrap();
    let authority = b.history_cold_export(b.expected(), &l).unwrap();
    b.activate_history_root(b.expected(), &root_b, &Default::default())
        .unwrap();
    // Undo activation is a full-state baseline boundary; certify its identity
    // directory once before entering the local producer, not inside each edit.
    l.observe(&b).unwrap();
    let pending = b
        .prepare_record_changes(
            b.expected(),
            &l,
            Mutation::Edits(&[e(1, 1, "30")]),
            &Default::default(),
        )
        .unwrap()
        .unwrap();
    assert!(stale_image.apply(&pending, &rt.memory()).is_err());
    drop(pending);
    step(
        &mut b,
        &mut l,
        Mutation::Edits(&[e(8, 8, "new branch")]),
        &rt,
    );
    b.activate_history_root(b.expected(), &root_d, &Default::default())
        .unwrap();
    assert_eq!(
        b.cell_at(p(9, 1)).unwrap().unwrap().content,
        Content::LiteralText("D-only".into())
    );
    let (mut cold, mut cold_l) =
        Workbook::history_cold_reopen(&authority.bytes, &cold_b.bytes, Limits::default(), &rt)
            .unwrap();
    assert_ne!(
        cold.history_runtime_node(p(1, 1)).unwrap(),
        b.history_runtime_node(p(1, 1)).unwrap()
    );
    step(
        &mut cold,
        &mut cold_l,
        Mutation::Edits(&[
            e(7, 7, "=SUM(A1:B2)"),
            e(7, 8, "=("),
            Edit::PreserveRejectedFormula {
                position: p(7, 9),
                source: "=A1".into(),
            },
        ]),
        &rt,
    );
    drop((
        cold,
        cold_l,
        authority,
        cold_b,
        stale_image,
        root_b,
        root_d,
        b,
        l,
    ));
    finish(rt);
}

#[test]
fn deterministic_mixed_sequences_match_the_independent_complete_state_oracle() {
    let rt = IntegrationRuntime::standalone().unwrap();
    let mut b = Workbook::new_in("G2 property", Limits::default(), &rt).unwrap();
    let mut l = ColdLedger::new(&b).unwrap();
    let mut seed = 0x7265736f6c766564u64;
    for turn in 0..160 {
        seed = seed
            .wrapping_mul(6364136223846793005)
            .wrapping_add(1442695040888963407);
        let r = ((seed >> 16) % 6 + 1) as u32;
        let c = ((seed >> 24) % 6 + 1) as u32;
        let edit = match (seed >> 32) % 8 {
            0 => Edit::SetDecimal {
                position: p(r, c),
                value: DecimalInput::from_parts(
                    turn,
                    0,
                    0,
                    seed % 2 == 0,
                    ((seed >> 40) % 6) as u32,
                )
                .unwrap(),
            },
            1 => Edit::SetText {
                position: p(r, c),
                text: format!("text \u{03bb} {turn}"),
            },
            2 => e(r, c, "=A1+$B$2"),
            3 => e(r, c, "=("),
            4 => Edit::Clear(p(r, c)),
            5 if b.cell_at(p(r, c)).unwrap().is_some() => Edit::Retire(p(r, c)),
            6 => e(r, c, "=COUNT($A$1:C3)"),
            _ => e(r, c, &turn.to_string()),
        };
        step(&mut b, &mut l, Mutation::Edits(&[edit]), &rt);
    }
    drop((b, l));
    finish(rt);
}

#[test]
fn old_calculation_runs_independently_while_private_records_are_prepared() {
    let rt = IntegrationRuntime::standalone().unwrap();
    let mut b = Workbook::new_in("G2 concurrent", Limits::default(), &rt).unwrap();
    b.apply(b.expected(), &[e(1, 1, "10"), e(1, 2, "=A1+1")])
        .unwrap();
    let mut l = ColdLedger::new(&b).unwrap();
    let capture = b.capture_local(b.expected()).unwrap();
    let cell = capture.cell_at(p(1, 2)).unwrap();
    std::thread::scope(|scope| {
        let reader = scope.spawn(|| {
            for _ in 0..32 {
                assert_eq!(
                    capture.evaluate(&[cell]).unwrap().values[0].1,
                    Ok(11.into())
                );
            }
        });
        let pending = b
            .prepare_record_changes(
                b.expected(),
                &l,
                Mutation::Edits(&[e(1, 1, "20")]),
                &Default::default(),
            )
            .unwrap()
            .unwrap();
        assert_eq!(
            capture.evaluate(&[cell]).unwrap().values[0].1,
            Ok(11.into())
        );
        pending.publish(&mut b, &mut l).unwrap();
        reader.join().unwrap();
    });
    assert_eq!(
        capture.evaluate(&[cell]).unwrap().values[0].1,
        Ok(11.into())
    );
    drop((b, l, capture));
    finish(rt);
}

#[test]
fn same_floor_same_session_foreign_private_branch_cannot_borrow_ledger_authority() {
    let rt = IntegrationRuntime::standalone().unwrap();
    let mut base = Workbook::new_in("G2 paired authority", Limits::default(), &rt).unwrap();
    base.apply(base.expected(), &[e(1, 1, "10")]).unwrap();
    let mut d = base.stage_history_candidate(base.expected()).unwrap();
    d.apply(d.expected(), &[e(2, 1, "D")]).unwrap();
    let mut c = base.stage_history_candidate(base.expected()).unwrap();
    c.apply(c.expected(), &[e(2, 1, "C")]).unwrap();
    assert_eq!(d.expected(), c.expected());
    assert_eq!(
        d.history_metrics().unwrap().next_node,
        c.history_metrics().unwrap().next_node
    );
    assert_ne!(d.history_content_id(), c.history_content_id());
    let mut ledger = ColdLedger::new(&d).unwrap();
    let edit = [e(1, 1, "20")];
    assert!(c.history_cold_export(c.expected(), &ledger).is_err());
    assert!(
        c.prepare_record_changes(
            c.expected(),
            &ledger,
            Mutation::Edits(&edit),
            &Default::default()
        )
        .is_err()
    );
    assert!(ledger.observe(&c).is_err());
    assert!(
        d.prepare_record_changes(
            d.expected(),
            &ledger,
            Mutation::Edits(&edit),
            &Default::default()
        )
        .unwrap()
        .is_some()
    );
    drop((base, d, c, ledger));
    finish(rt);
}

#[test]
fn local_publication_advances_reservations_without_full_observation_between_edits() {
    let rt = IntegrationRuntime::standalone().unwrap();
    let mut b = Workbook::new_in("G2 local chain", Limits::default(), &rt).unwrap();
    let mut l = ColdLedger::new(&b).unwrap();
    let mut oracle = l.clone_for_oracle(&rt.memory()).unwrap();
    for r in 1..=48 {
        let edits = [
            e(r, 1, &r.to_string()),
            e(r, 2, &format!("=SUM(A{r}:A{r})")),
        ];
        let pending = b
            .prepare_record_changes(
                b.expected(),
                &l,
                Mutation::Edits(&edits),
                &Default::default(),
            )
            .unwrap()
            .unwrap();
        pending.publish(&mut b, &mut l).unwrap();
    }
    // Only the independent end-of-sequence oracle scans the complete directory.
    oracle.observe(&b).unwrap();
    assert_eq!(
        b.history_cold_export(b.expected(), &l).unwrap().bytes,
        b.history_cold_export(b.expected(), &oracle).unwrap().bytes
    );
    let before = b.history_cold_export(b.expected(), &l).unwrap();
    let expected = b.expected();
    let floor = l.allocated_floor();
    let pending = b
        .prepare_record_changes(
            expected,
            &l,
            Mutation::Edits(&[e(64, 1, "new reservation")]),
            &Default::default(),
        )
        .unwrap()
        .unwrap();
    rt.flush();
    let memory = rt.memory();
    let stats = memory.stats();
    let pressure = memory.reserve(stats.limit - stats.charged).unwrap();
    assert!(pending.publish(&mut b, &mut l).is_err());
    drop(pressure);
    rt.flush();
    assert_eq!(b.expected(), expected);
    assert_eq!(l.allocated_floor(), floor);
    assert_eq!(
        b.history_cold_export(expected, &l).unwrap().bytes,
        before.bytes
    );
    drop((before, b, l, oracle));
    finish(rt);
}

#[test]
fn prepared_publication_cannot_cross_independent_ledger_instances() {
    let rt = IntegrationRuntime::standalone().unwrap();
    let mut b = Workbook::new_in("G2 instance", Limits::default(), &rt).unwrap();
    b.apply(b.expected(), &[e(1, 1, "10")]).unwrap();
    let mut a = ColdLedger::new(&b).unwrap();
    let mut foreign = ColdLedger::new(&b).unwrap();
    let before = b.expected();
    let content = b.history_content_id();
    let floor = foreign.allocated_floor();
    let edits = [e(2, 2, "20")];
    let pending = b
        .prepare_record_changes(before, &a, Mutation::Edits(&edits), &Default::default())
        .unwrap()
        .unwrap();
    assert!(pending.publish(&mut b, &mut foreign).is_err());
    assert_eq!(b.expected(), before);
    assert_eq!(b.history_content_id(), content);
    assert_eq!(foreign.allocated_floor(), floor);
    b.prepare_record_changes(before, &a, Mutation::Edits(&edits), &Default::default())
        .unwrap()
        .unwrap()
        .publish(&mut b, &mut a)
        .unwrap();
    drop((b, a, foreign));
    finish(rt);
}
