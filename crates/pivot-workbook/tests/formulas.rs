use pivot_core::expression::{self, Expr};
use pivot_types::PivotDecimal;
use pivot_workbook::*;
const A: Position = Position::new(1, 1);
const B: Position = Position::new(1, 2);
const C: Position = Position::new(1, 3);
const D: Position = Position::new(1, 4);
fn book() -> Workbook {
    Workbook::new("Text", Limits::default()).unwrap()
}
fn input(position: Position, n: u32) -> Edit {
    Edit::SetDecimal {
        position,
        value: DecimalInput::from_parts(n, 0, 0, false, 0).unwrap(),
    }
}
fn formula(position: Position, source: &str) -> Edit {
    Edit::SetFormulaText {
        position,
        source: source.into(),
    }
}
fn apply(w: &mut Workbook, edits: &[Edit]) {
    w.apply(w.expected(), edits).unwrap();
}
fn text(w: &Workbook, p: Position) -> TextFormula {
    let Content::TextFormula(f) = w.cell_at(p).unwrap().unwrap().content else {
        panic!()
    };
    f
}
fn value(w: &Workbook, p: Position) -> PivotDecimal {
    let r = w.cell_at(p).unwrap().unwrap().reference;
    w.capture(w.expected())
        .unwrap()
        .evaluate(&[r])
        .unwrap()
        .values[0]
        .1
        .clone()
        .unwrap()
}
#[test]
fn raw_text_stable_targets_retirement_and_explicit_repair() {
    let mut w = book();
    let base = w.expected();
    apply(&mut w, &[input(A, 100), input(B, 25), formula(C, "=A1+B1")]);
    assert_eq!(value(&w, C), "125".parse().unwrap());
    let original = text(&w, C);
    assert_eq!(original.transaction_base, base);
    assert_eq!(original.edit_ordinal, 2);
    assert_eq!(original.references[0].span, 1..3);
    assert_eq!(original.references[1].span, 4..6);
    assert_eq!(
        original.references[0].node,
        w.node_for(original.references[0].target).unwrap()
    );
    let captured = w.capture(w.expected()).unwrap();
    let output = w.cell_at(C).unwrap().unwrap().reference;
    apply(&mut w, &[input(A, 200)]);
    assert_eq!(value(&w, C), "225".parse().unwrap());
    apply(&mut w, &[Edit::Move { from: A, to: D }, input(A, 999)]);
    assert_eq!(value(&w, C), "225".parse().unwrap());
    assert_eq!(text(&w, C), original);
    assert_eq!(
        w.cell_at(D).unwrap().unwrap().reference,
        original.references[0].target
    );
    apply(&mut w, &[Edit::Clear(D)]);
    assert_eq!(value(&w, C), "25".parse().unwrap());
    apply(&mut w, &[Edit::Retire(D), input(D, 500)]);
    assert!(w.capture(w.expected()).is_err());
    assert!(w.validate(w.expected()).unwrap().iter().any(|d|matches!(d,Diagnostic::Formula { error,.. } if error.offset==1 && error.message=="retired cell reference")));
    assert!(w.validate(w.expected()).unwrap().iter().any(|d|matches!(d,Diagnostic::MissingReference { target,.. } if *target==original.references[0].target)));
    apply(&mut w, &[formula(C, "=D1+B1")]);
    assert_eq!(value(&w, C), "525".parse().unwrap());
    assert_ne!(
        text(&w, C).references[0].target,
        original.references[0].target
    );
    assert_eq!(
        captured.evaluate(&[output]).unwrap().values[0]
            .1
            .clone()
            .unwrap(),
        "125".parse().unwrap()
    );
    apply(&mut w, &[formula(C, "=A1+B1")]);
    assert_eq!(value(&w, C), "1024".parse().unwrap());
}
#[test]
fn arithmetic_precedence_unary_and_exact_decimal_parity() {
    let mut w = book();
    apply(&mut w, &[input(A, 10), input(B, 4)]);
    for (source, expected) in [
        ("=A1+B1*2", "18"),
        ("(A1+B1)*2", "28"),
        (" = -(a1 - +b1) / 2 ", "-3"),
        ("=8/4/2", "1"),
        ("=-+1.2300 + (2.5*2)", "3.77"),
    ] {
        apply(&mut w, &[formula(C, source)]);
        assert_eq!(value(&w, C), expected.parse().unwrap(), "{source}");
    }
    for source in ["1.2300", "0.0000", "0.0000000000000000000000000001"] {
        let Expr::Decimal(standalone) = expression::parse(source).unwrap() else {
            panic!()
        };
        apply(&mut w, &[formula(C, source)]);
        assert_eq!(value(&w, C).parts(), standalone.parts());
        assert_eq!(text(&w, C).original_source, source);
    }
    apply(&mut w, &[formula(C, "=1/0")]);
    let output = w.cell_at(C).unwrap().unwrap().reference;
    assert!(matches!(
        w.capture(w.expected())
            .unwrap()
            .evaluate(&[output])
            .unwrap()
            .values[0]
            .1,
        Err(CellError::Arithmetic(_))
    ));
}
#[test]
fn malformed_and_out_of_bounds_references_are_editable_drafts() {
    let mut w = book();
    for (source, offset) in [
        ("=A0", 1),
        ("=A01", 1),
        ("=A", 1),
        ("=A1_", 1),
        ("=A1B2", 1),
        ("=A4294967296", 1),
        ("=ZZZZZZZZZZ1", 1),
        ("=A1025", 1),
        ("=ZZZ1", 1),
        ("=A1+)", 4),
        ("=A1+💰", 4),
        ("=$A$1", 1),
        ("=A1:B2", 3),
        ("=Sheet!A1", 1),
        ("=SUM(A1)", 1),
        ("==1", 1),
        ("=0.00000000000000000000000000001", 1),
        ("=1e3", 2),
    ] {
        apply(&mut w, &[formula(C, source)]);
        let f = text(&w, C);
        assert_eq!(f.diagnostic.as_ref().unwrap().offset, offset, "{source}");
        assert!(f.references.is_empty());
        assert!(w.capture(w.expected()).is_err());
        assert!(
            w.validate(w.expected())
                .unwrap()
                .iter()
                .any(|d| matches!(d,Diagnostic::Formula { error,.. } if error.offset==offset))
        );
        assert!(w.cell_at(A).unwrap().is_none());
    }
    apply(&mut w, &[formula(C, "=1+2")]);
    assert_eq!(value(&w, C), "3".parse().unwrap());
}
#[test]
fn blank_anchors_and_sequential_batch_binding() {
    let mut w = book();
    let base = w.expected();
    apply(&mut w, &[formula(C, "=A1+1")]);
    assert_eq!(text(&w, C).transaction_base, base);
    assert_ne!(text(&w, C).resolved_graph, base.graph);
    assert_ne!(text(&w, C).resolved_graph, w.expected().graph);
    assert_eq!(value(&w, C), "1".parse().unwrap());
    let first = text(&w, C).references[0].target;
    assert_eq!(w.cell_at(A).unwrap().unwrap().content, Content::Blank);
    apply(&mut w, &[input(A, 5)]);
    assert_eq!(value(&w, C), "6".parse().unwrap());
    apply(
        &mut w,
        &[Edit::Move { from: A, to: D }, formula(B, "=A1+1")],
    );
    assert_eq!(value(&w, C), "6".parse().unwrap());
    assert_eq!(value(&w, B), "1".parse().unwrap());
    assert_ne!(text(&w, B).references[0].target, first);
}
#[test]
fn scalar_cycles_and_repair() {
    let mut w = book();
    apply(&mut w, &[formula(A, "=B1+1"), formula(B, "=A1+1")]);
    assert!(
        w.validate(w.expected())
            .unwrap()
            .iter()
            .any(|d| matches!(d, Diagnostic::Unorderable { .. }))
    );
    assert!(w.capture(w.expected()).is_err());
    apply(&mut w, &[input(B, 2)]);
    assert_eq!(value(&w, A), "3".parse().unwrap());
}
#[test]
fn hard_failures_roll_back_and_stale_states_are_rejected() {
    let mut w = book();
    let initial = w.expected();
    let stats = w.stats();
    assert_eq!(
        w.apply(
            initial,
            &[formula(C, "=A1+1"), input(Position::new(0, 1), 5)]
        ),
        Err(Error::Position)
    );
    assert_eq!(w.expected(), initial);
    assert_eq!(w.stats(), stats);
    assert!(w.cell_at(A).unwrap().is_none());
    apply(&mut w, &[input(A, 7)]);
    let current = w.expected();
    assert_eq!(
        w.apply(initial, &[formula(C, "=A1+1")]),
        Err(Error::StaleRevision)
    );
    assert_eq!(w.expected(), current);
    assert_eq!(
        w.apply(book().expected(), &[formula(C, "=1")]),
        Err(Error::StaleSession)
    );
    for limits in [
        Limits {
            live_cells: 1,
            ..Limits::default()
        },
        Limits {
            formula_tokens: 2,
            ..Limits::default()
        },
        Limits {
            source_bytes: 3,
            ..Limits::default()
        },
    ] {
        let mut small = Workbook::new("small", limits).unwrap();
        let before = small.expected();
        assert_eq!(
            small.apply(before, &[formula(C, "=A1+1")]),
            Err(Error::Limit)
        );
        assert_eq!(small.expected(), before);
        assert_eq!(small.stats().live_cells, 0);
    }
}
#[test]
fn standalone_grammar_unchanged_and_typed_spans_use_original_source() {
    assert_eq!(expression::parse("A1").unwrap(), Expr::Name("A1".into()));
    assert!(expression::parse("=A1").is_err());
    let Expr::CellReference { row, column, span } =
        expression::parse_spreadsheet("  = aa22 ").unwrap()
    else {
        panic!()
    };
    assert_eq!((row, column, span), (22, 27, 4..8));
    for source in [
        "=1+", "=", "=()", "=A1 B1", "=A1<2", "=1,2", "=A+1", "=_name", "=A-1",
    ] {
        assert!(expression::parse_spreadsheet(source).is_err(), "{source}");
    }
    assert!(
        expression::parse_spreadsheet(&format!("={}1{}", "(".repeat(65), ")".repeat(65))).is_err()
    );
    assert!(expression::parse_spreadsheet(&"1".repeat(4097)).is_err());
}
