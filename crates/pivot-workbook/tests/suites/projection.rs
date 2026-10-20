use pivot_core::expression::{BinaryOp, UnaryOp};
use pivot_types::PivotDecimal;
use pivot_workbook::*;
use uuid::Uuid;
const A: Position = Position::new(1, 1);
const B: Position = Position::new(1, 2);
const C: Position = Position::new(1, 3);
const D: Position = Position::new(1, 4);
fn d(n: u32) -> DecimalInput {
    DecimalInput::from_parts(n, 0, 0, false, 0).unwrap()
}
fn input(position: Position, n: u32) -> Edit {
    Edit::SetDecimal {
        position,
        value: d(n),
    }
}
fn add(position: Position, left: Position, right: Position) -> Edit {
    Edit::SetFormula {
        position,
        source: "typed addition (no A1 parser)".into(),
        tokens: vec![
            Token::At(left),
            Token::At(right),
            Token::Binary(BinaryOp::Add),
        ],
    }
}
fn proof() -> Workbook {
    let mut w = build_book("Proof", Limits::default()).unwrap();
    w.apply(w.expected(), &[input(A, 100), input(B, 25), add(C, A, B)])
        .unwrap();
    w
}
fn reference(w: &Workbook, p: Position) -> CellRef {
    w.cell_at(p).unwrap().unwrap().reference
}
fn number(w: &Workbook, p: Position) -> PivotDecimal {
    w.capture(w.expected())
        .unwrap()
        .evaluate(&[reference(w, p)])
        .unwrap()
        .values[0]
        .1
        .clone()
        .unwrap()
}
fn region(w: &Workbook) -> RectangleRead {
    w.read(
        w.expected(),
        Rectangle {
            first: A,
            last: Position::new(4, 8),
        },
        1024,
    )
    .unwrap()
}

#[test]
fn move_clear_retire_reuse_and_explicit_rebinding_have_independent_expected_results() {
    let mut w = proof();
    let a = reference(&w, A);
    let c = reference(&w, C);
    let node = w.node_for(a).unwrap();
    assert_eq!(number(&w, C), "125".parse().unwrap());
    w.apply(w.expected(), &[input(A, 200)]).unwrap();
    assert_eq!(number(&w, C), "225".parse().unwrap());
    let before = w.expected();
    w.apply(before, &[Edit::Move { from: A, to: D }]).unwrap();
    // Sparse placement is canonical graph state, even without a SUM consumer.
    assert_ne!(w.expected().graph, before.graph);
    assert_ne!(w.expected().state, before.state);
    assert!(w.cell_at(A).unwrap().is_none());
    assert_eq!(reference(&w, D), a);
    assert_eq!(w.node_for(a).unwrap(), node);
    w.apply(w.expected(), &[input(A, 999)]).unwrap();
    assert_ne!(reference(&w, A), a);
    assert_eq!(number(&w, C), "225".parse().unwrap());
    w.apply(w.expected(), &[Edit::Clear(D)]).unwrap();
    assert_eq!(reference(&w, D), a);
    assert_eq!(w.node_for(a).unwrap(), node);
    assert_eq!(w.cell_at(D).unwrap().unwrap().content, Content::Blank);
    assert_eq!(number(&w, C), "25".parse().unwrap());
    w.apply(w.expected(), &[Edit::Retire(D)]).unwrap();
    assert_eq!(
        w.validate(w.expected()).unwrap(),
        vec![Diagnostic::MissingReference { cell: c, target: a }]
    );
    assert!(matches!(w.capture(w.expected()), Err(Error::InvalidDraft)));
    w.apply(w.expected(), &[input(D, 500)]).unwrap();
    let replacement = reference(&w, D);
    assert_ne!(replacement, a);
    assert_ne!(w.node_for(replacement).unwrap(), node);
    assert_eq!(
        w.validate(w.expected()).unwrap(),
        vec![Diagnostic::MissingReference { cell: c, target: a }]
    );
    w.apply(w.expected(), &[add(C, D, B)]).unwrap();
    assert!(w.validate(w.expected()).unwrap().is_empty());
    assert_eq!(number(&w, C), "525".parse().unwrap());
}

#[test]
fn empty_grid_is_sparse_and_referenced_blanks_are_explicit_demanded_anchors() {
    let mut w = build_book("Sparse", Limits::default()).unwrap();
    assert_eq!(w.stats().live_nodes, 0);
    assert_eq!(w.stats().allocated_nodes, 0);
    assert!(w.cell_at(Position::new(1000, 1000)).unwrap().is_none());
    let before = w.expected();
    w.apply(before, &[Edit::Clear(A)]).unwrap();
    assert_eq!(w.expected(), before);
    w.apply(
        w.expected(),
        &[Edit::SetFormula {
            position: C,
            source: "opaque source, deliberately not parsable".into(),
            tokens: vec![
                Token::At(A),
                Token::Decimal(d(1)),
                Token::Binary(BinaryOp::Add),
            ],
        }],
    )
    .unwrap();
    let a = reference(&w, A);
    let node = w.node_for(a).unwrap();
    assert_eq!(w.stats().live_nodes, 2);
    assert_eq!(w.cell_at(A).unwrap().unwrap().content, Content::Blank);
    assert_eq!(number(&w, C), "1".parse().unwrap());
    w.apply(w.expected(), &[input(A, 35)]).unwrap();
    assert_eq!(reference(&w, A), a);
    assert_eq!(w.node_for(a).unwrap(), node);
    assert_eq!(number(&w, C), "36".parse().unwrap());
    let axes_a = w.axes_at(A).unwrap();
    let axes_d = w.axes_at(D).unwrap();
    assert_eq!(axes_a.0, axes_d.0);
    assert_ne!(axes_a.1, axes_d.1);
    let other = build_book("Other", Limits::default()).unwrap();
    assert_ne!(other.axes_at(A).unwrap().0, axes_a.0);
    assert_ne!(a.cell.uuid(), node.uuid());
}

#[test]
fn core_supported_operators_and_arithmetic_errors_are_reused() {
    let mut w = build_book("Operators", Limits::default()).unwrap();
    w.apply(
        w.expected(),
        &[
            Edit::SetFormula {
                position: A,
                source: "typed postfix".into(),
                tokens: vec![
                    Token::Decimal(d(10)),
                    Token::Decimal(d(2)),
                    Token::Decimal(d(3)),
                    Token::Binary(BinaryOp::Multiply),
                    Token::Binary(BinaryOp::Subtract),
                    Token::Decimal(d(4)),
                    Token::Binary(BinaryOp::Divide),
                    Token::Unary(UnaryOp::Minus),
                    Token::Unary(UnaryOp::Plus),
                ],
            },
            Edit::SetFormula {
                position: B,
                source: "division failure".into(),
                tokens: vec![
                    Token::Decimal(d(1)),
                    Token::Decimal(d(0)),
                    Token::Binary(BinaryOp::Divide),
                ],
            },
        ],
    )
    .unwrap();
    assert_eq!(number(&w, A), "-1".parse().unwrap());
    let before = region(&w);
    let values = w
        .capture(w.expected())
        .unwrap()
        .evaluate(&[reference(&w, A), reference(&w, B)])
        .unwrap()
        .values;
    assert!(values[0].1.is_ok());
    assert!(matches!(values[1].1, Err(CellError::Arithmetic(_))));
    assert_eq!(before, region(&w));
}

#[test]
fn content_replacement_preserves_ids_and_references() {
    let mut w = proof();
    let a = reference(&w, A);
    let node = w.node_for(a).unwrap();
    w.apply(
        w.expected(),
        &[Edit::SetFormula {
            position: A,
            source: "replacement".into(),
            tokens: vec![Token::Decimal(d(7))],
        }],
    )
    .unwrap();
    assert_eq!(reference(&w, A), a);
    assert_eq!(w.node_for(a).unwrap(), node);
    assert_eq!(number(&w, C), "32".parse().unwrap());
    w.apply(w.expected(), &[input(A, 2)]).unwrap();
    assert_eq!(reference(&w, A), a);
    assert_eq!(number(&w, C), "27".parse().unwrap());
}

#[test]
fn failed_batches_leave_both_sheet_and_graph_unchanged() {
    let mut w = proof();
    let before = region(&w);
    let stats = w.stats();
    let expected = w.expected();
    let mut other = proof();
    other.apply(other.expected(), &[input(D, 1)]).unwrap();
    let foreign = reference(&other, D);
    let unknown = CellRef {
        worksheet: w.worksheet(),
        cell: CellId::from_uuid(Uuid::new_v4()),
    };
    for bad in [
        Edit::Move { from: A, to: B },
        Edit::SetFormula {
            position: D,
            source: "bad foreign target".into(),
            tokens: vec![Token::Cell(foreign)],
        },
        Edit::SetFormula {
            position: D,
            source: "bad unknown target".into(),
            tokens: vec![Token::Cell(unknown)],
        },
        Edit::SetFormula {
            position: D,
            source: "bad stack".into(),
            tokens: vec![Token::Binary(BinaryOp::Add)],
        },
        Edit::SetFormula {
            position: D,
            source: "invalid coordinate".into(),
            tokens: vec![Token::At(Position::new(0, 1))],
        },
    ] {
        assert!(w.apply(expected, &[input(A, 900), bad]).is_err());
        assert_eq!(region(&w), before);
        assert_eq!(w.stats(), stats);
        assert_eq!(w.expected(), expected);
        assert_eq!(number(&w, C), "125".parse().unwrap());
    }
    assert_eq!(
        w.apply(
            expected,
            &[Edit::Move { from: A, to: D }, Edit::Move { from: B, to: D }]
        ),
        Err(Error::Occupied)
    );
    assert_eq!(region(&w), before);
    assert_eq!(w.expected(), expected);
}

#[test]
fn resource_failures_rollback_new_nodes_and_keep_retired_identity_quota() {
    let mut w = build_book(
        "Budget",
        Limits {
            live_cells: 1,
            ..Limits::default()
        },
    )
    .unwrap();
    let before = w.expected();
    assert!(matches!(
        w.apply(before, &[input(A, 1), input(B, 2)]),
        Err(Error::Limit)
    ));
    assert_eq!(w.expected(), before);
    assert_eq!(w.stats().live_nodes, 0);
    assert_eq!(w.stats().allocated_nodes, 0);
    let mut w = build_book(
        "Bytes",
        Limits {
            bytes: 512,
            ..Limits::default()
        },
    )
    .unwrap();
    assert!(matches!(
        w.apply(w.expected(), &[input(A, 1), input(B, 2)]),
        Err(Error::Limit)
    ));
    assert_eq!(w.stats().live_nodes, 0);
    let mut w = build_book(
        "Retired",
        Limits {
            allocated_cells: 1,
            ..Limits::default()
        },
    )
    .unwrap();
    w.apply(w.expected(), &[input(A, 1), Edit::Retire(A)])
        .unwrap();
    let before = w.expected();
    assert!(matches!(w.apply(before, &[input(A, 2)]), Err(Error::Limit)));
    assert_eq!(w.expected(), before);
    assert_eq!(w.stats().live_nodes, 0);
    assert_eq!(w.stats().allocated_nodes, 1);
    let mut w = proof();
    let before = region(&w);
    let huge = Edit::SetFormula {
        position: D,
        source: "x".repeat(4097),
        tokens: vec![Token::Decimal(d(1))],
    };
    assert!(matches!(w.apply(w.expected(), &[huge]), Err(Error::Limit)));
    assert_eq!(region(&w), before);
}

#[test]
fn stale_expected_revisions_and_sessions_reject_without_installation() {
    let mut w = proof();
    let old = w.expected();
    w.apply(old, &[input(A, 200)]).unwrap();
    let current = region(&w);
    assert_eq!(w.apply(old, &[input(A, 999)]), Err(Error::StaleRevision));
    let mut wrong = w.expected();
    wrong.graph = old.graph;
    assert_eq!(w.apply(wrong, &[input(A, 999)]), Err(Error::StaleRevision));
    let other = proof();
    assert_eq!(w.apply(other.expected(), &[]), Err(Error::StaleSession));
    assert_eq!(region(&w), current);
}

#[test]
fn immutable_calculation_capture_retains_original_positions_and_graph_revision() {
    let mut w = proof();
    let c = reference(&w, C);
    let a = reference(&w, A);
    let expected = w.expected();
    let capture = w.capture(expected).unwrap();
    assert_eq!(capture.cell_at(A), Some(a));
    w.apply(
        w.expected(),
        &[input(A, 200), Edit::Move { from: A, to: D }],
    )
    .unwrap();
    assert_eq!(capture.cell_at(A), Some(a));
    assert_eq!(capture.cell_at(D), None);
    assert_eq!(number(&w, C), "225".parse().unwrap());
    w.apply(w.expected(), &[Edit::Retire(D)]).unwrap();
    assert!(matches!(w.capture(w.expected()), Err(Error::InvalidDraft)));
    drop(w);
    let result = std::thread::spawn(move || capture.evaluate(&[c]).unwrap())
        .join()
        .unwrap();
    assert_eq!(result.expected, expected);
    assert_eq!(result.values[0].1.clone().unwrap(), "125".parse().unwrap());
}

#[test]
fn ordinary_and_net_noops_do_not_advance_public_state_or_graph_revision() {
    let mut w = proof();
    let expected = w.expected();
    w.apply(
        expected,
        &[
            input(A, 100),
            add(C, A, B),
            Edit::Move { from: A, to: A },
            Edit::Clear(D),
        ],
    )
    .unwrap();
    assert_eq!(w.expected(), expected);
    w.apply(expected, &[input(A, 900), input(A, 100)]).unwrap();
    assert_eq!(w.expected(), expected);
    w.apply(expected, &[Edit::Clear(A)]).unwrap();
    let blank = w.expected();
    w.apply(blank, &[Edit::Clear(A)]).unwrap();
    assert_eq!(w.expected(), blank);
}

#[test]
fn bounded_rectangle_reads_have_complete_sparse_coverage_and_revision_tags() {
    let mut w = build_book("Rect", Limits::default()).unwrap();
    w.apply(
        w.expected(),
        &[
            input(A, 1),
            input(D, 4),
            input(Position::new(2, 1), 2),
            input(Position::new(2, 4), 8),
        ],
    )
    .unwrap();
    let rectangle = Rectangle {
        first: A,
        last: Position::new(2, 2),
    };
    let result = w.read(w.expected(), rectangle, 2).unwrap();
    assert_eq!(result.expected, w.expected());
    assert_eq!(result.rectangle, rectangle);
    assert_eq!(
        result.cells.iter().map(|v| v.position).collect::<Vec<_>>(),
        vec![A, Position::new(2, 1)]
    );
    assert!(matches!(
        w.read(w.expected(), rectangle, 1),
        Err(Error::Limit)
    ));
    assert!(matches!(
        w.read(w.expected(), Rectangle { first: D, last: A }, 10),
        Err(Error::Position)
    ));
    assert!(matches!(
        w.read(
            w.expected(),
            Rectangle {
                first: A,
                last: Position::new(100, 100)
            },
            10
        ),
        Err(Error::Limit)
    ));
    assert!(
        w.read(w.expected(), Rectangle { first: B, last: C }, 0)
            .unwrap()
            .cells
            .is_empty()
    );
    let expected = w.expected();
    w.apply(expected, &[input(A, 9)]).unwrap();
    assert!(matches!(
        w.read(expected, rectangle, 2),
        Err(Error::StaleRevision)
    ));
    assert_eq!(result.cells[0].content, Content::Decimal(d(1)));
}

#[test]
fn typed_cycle_drafts_are_diagnosable_and_global_snapshot_limit_is_explicit() {
    let mut w = proof();
    let a = reference(&w, A);
    w.apply(
        w.expected(),
        &[Edit::SetFormula {
            position: A,
            source: "self reference".into(),
            tokens: vec![Token::Cell(a)],
        }],
    )
    .unwrap();
    assert!(
        w.validate(w.expected())
            .unwrap()
            .iter()
            .any(|d| matches!(d,Diagnostic::Unorderable {cells} if cells.contains(&a)))
    );
    assert!(matches!(w.capture(w.expected()), Err(Error::InvalidDraft)));
    w.apply(w.expected(), &[input(A, 100)]).unwrap();
    assert_eq!(number(&w, C), "125".parse().unwrap());
}

#[test]
fn exact_submitted_decimals_are_metadata_separate_from_blank_zero_backing() {
    let mut w = build_book("Exact", Limits::default()).unwrap();
    for scale in [0, 4, 28] {
        for negative in [false, true] {
            let exact = DecimalInput::from_parts(0, 0, 0, negative, scale).unwrap();
            w.apply(
                w.expected(),
                &[Edit::SetDecimal {
                    position: A,
                    value: exact,
                }],
            )
            .unwrap();
            assert_eq!(
                w.cell_at(A).unwrap().unwrap().content,
                Content::Decimal(exact)
            );
            assert_eq!(exact.parts(), (0, 0, 0, negative, scale));
        }
    }
    assert!(DecimalInput::from_parts(1, 0, 0, false, 29).is_err());
    w.apply(w.expected(), &[Edit::Clear(A)]).unwrap();
    assert_eq!(w.cell_at(A).unwrap().unwrap().content, Content::Blank);
}

#[test]
fn workbook_boundary_has_no_storage_gui_or_transport_dependency() {
    let manifest = include_str!("../../Cargo.toml");
    for dependency in [
        "pivot-storage",
        "pivot-protocol",
        "pivot-host",
        "avalonia",
        "tokio",
    ] {
        assert!(!manifest.contains(dependency));
    }
    let core_manifest = include_str!("../../../pivot-core/Cargo.toml");
    assert!(!core_manifest.contains("pivot-workbook"));
}
