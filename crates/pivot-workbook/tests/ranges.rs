use pivot_core::model::integration::{CancellationToken, IntegrationRuntime};
use pivot_workbook::*;
fn p(r: u32, c: u32) -> Position {
    Position::new(r, c)
}
fn n(r: u32, c: u32, v: &str) -> Edit {
    Edit::SetDecimal {
        position: p(r, c),
        value: DecimalInput::from_decimal(v.parse().unwrap()),
    }
}
fn f(r: u32, c: u32, s: &str) -> Edit {
    Edit::SetFormulaText {
        position: p(r, c),
        source: s.into(),
    }
}
fn value(book: &Workbook, r: u32, c: u32) -> String {
    let capture = book.capture_local(book.expected()).unwrap();
    let id = capture.cell_at(p(r, c)).unwrap();
    capture.evaluate(&[id]).unwrap().values[0]
        .1
        .as_ref()
        .unwrap()
        .to_string()
}
#[test]
fn coordinate_affinity_sparse_membership_and_captured_generations() {
    let runtime = IntegrationRuntime::standalone().unwrap();
    let mut book = Workbook::new_in("Ranges", Limits::default(), &runtime).unwrap();
    book.apply(
        book.expected(),
        &[
            n(1, 1, "10"),
            n(2, 1, "20"),
            n(3, 1, "30"),
            f(1, 4, " = a1 + sUm( A3 : A1 ) "),
        ],
    )
    .unwrap();
    assert_eq!(value(&book, 1, 4), "70");
    let s = book.capture_local(book.expected()).unwrap();
    let output = s.cell_at(p(1, 4)).unwrap();
    let allocated = book.stats().allocated_nodes;
    book.apply(
        book.expected(),
        &[Edit::Move {
            from: p(1, 1),
            to: p(5, 1),
        }],
    )
    .unwrap();
    assert_eq!(value(&book, 1, 4), "60");
    assert_eq!(
        s.evaluate(&[output]).unwrap().values[0]
            .1
            .as_ref()
            .unwrap()
            .to_string(),
        "70"
    );
    book.apply(book.expected(), &[n(1, 1, "5")]).unwrap();
    assert_eq!(value(&book, 1, 4), "65");
    assert_eq!(book.stats().allocated_nodes, allocated + 1);
    book.apply(book.expected(), &[Edit::Clear(p(2, 1))])
        .unwrap();
    assert_eq!(value(&book, 1, 4), "45");
    book.apply(book.expected(), &[Edit::Retire(p(3, 1)), n(3, 1, "7")])
        .unwrap();
    assert_eq!(value(&book, 1, 4), "22");
    book.apply(book.expected(), &[f(1, 5, "=SUM(A1:Z1024)")])
        .unwrap(); // self-including range is an editable cycle.
    assert!(!book.validate(book.expected()).unwrap().is_empty());
    book.apply(book.expected(), &[f(1, 5, "=SUM(A1:C1024)")])
        .unwrap();
    assert_eq!(value(&book, 1, 5), "22");
    let before = book.expected();
    let stats = book.stats();
    assert!(
        book.apply(
            before,
            &[
                n(10, 10, "9"),
                Edit::Move {
                    from: p(1, 4),
                    to: p(1, 5)
                }
            ]
        )
        .is_err()
    );
    assert_eq!(book.expected(), before);
    assert_eq!(book.stats().allocated_nodes, stats.allocated_nodes);
    let token = CancellationToken::default();
    token.cancel();
    assert!(
        book.apply_cancelled(before, &[n(10, 10, "9")], &token)
            .is_err()
    );
    assert_eq!(book.expected(), before);
    drop(s);
    drop(book);
    let memory = runtime.memory();
    assert!(runtime.finish().is_ok());
    assert_eq!(memory.stats().charged, 0);
}
#[test]
fn transitive_members_cycles_drafts_and_ordered_overflow() {
    let runtime = IntegrationRuntime::standalone().unwrap();
    let mut book = Workbook::new_in("Ranges", Limits::default(), &runtime).unwrap();
    book.apply(
        book.expected(),
        &[
            n(1, 1, "2"),
            f(2, 2, "=A1*3"),
            f(1, 4, "=SUM(B2:B100)"),
            f(1000, 10, "=("),
        ],
    )
    .unwrap();
    assert_eq!(value(&book, 1, 4), "6");
    book.apply(book.expected(), &[n(1, 1, "4")]).unwrap();
    assert_eq!(value(&book, 1, 4), "12");
    book.apply(book.expected(), &[f(2, 2, "=D1+1")]).unwrap();
    let capture = book.capture_local(book.expected()).unwrap();
    assert!(
        capture
            .evaluate(&[capture.cell_at(p(1, 4)).unwrap()])
            .is_err()
    );
    drop(capture);
    book.apply(
        book.expected(),
        &[
            f(2, 2, "=A1*3"),
            f(1, 6, "=SUM(A1)"),
            f(2, 6, "=SUM(A0:A1)"),
            f(3, 6, "=SUM(A1:B99999)"),
        ],
    )
    .unwrap();
    assert_eq!(value(&book, 1, 4), "12");
    for row in 1..=3 {
        let c = book.cell_at(p(row, 6)).unwrap().unwrap();
        assert!(matches!(
            c.content,
            Content::TextFormula(TextFormula {
                diagnostic: Some(_),
                ..
            })
        ));
    }
    book.apply(
        book.expected(),
        &[
            n(1, 8, "79228162514264337593543950335"),
            n(2, 8, "1"),
            n(3, 8, "-1"),
            f(1, 9, "=SUM(H1:H3)"),
        ],
    )
    .unwrap();
    let capture = book.capture_local(book.expected()).unwrap();
    assert!(
        capture
            .evaluate(&[capture.cell_at(p(1, 9)).unwrap()])
            .unwrap()
            .values[0]
            .1
            .is_err()
    );
    drop(capture);
    drop(book);
    assert!(runtime.finish().is_ok());
}

#[test]
fn aggregate_invalidation_follows_computed_members_and_admission_rolls_back() {
    let runtime = IntegrationRuntime::standalone().unwrap();
    let mut book = Workbook::new_in("Invalidation", Limits::default(), &runtime).unwrap();
    book.apply(
        book.expected(),
        &[
            n(1, 1, "2"),
            f(2, 2, "=A1*3"),
            f(1, 4, "=SUM(B2:B100)"),
            f(1, 5, "=SUM(C2:C100)"),
        ],
    )
    .unwrap();
    let output = book.cell_at(p(1, 4)).unwrap().unwrap().reference;
    let unrelated = book.cell_at(p(1, 5)).unwrap().unwrap().reference;
    let before = book.selection_metrics();
    book.apply(book.expected(), &[n(1, 1, "4")]).unwrap();
    let after = book.selection_metrics();
    assert!(after.selection_candidates > before.selection_candidates);
    assert!(book.invalidated_cells().contains(&output));
    assert!(!book.invalidated_cells().contains(&unrelated));
    let edits = (1..=126)
        .map(|r| f(r, 6, "=SUM(A1:A1)"))
        .collect::<Vec<_>>();
    book.apply(book.expected(), &edits).unwrap();
    assert_eq!(book.selection_metrics().selections, 128);
    let expected = book.expected();
    let stats = book.stats();
    assert!(
        book.apply(expected, &[n(500, 7, "9"), f(500, 8, "=SUM(A1:A1)")])
            .is_err()
    );
    assert_eq!(book.expected(), expected);
    assert_eq!(book.stats().allocated_nodes, stats.allocated_nodes);
    assert!(book.cell_at(p(500, 7)).unwrap().is_none());
    book.apply(book.expected(), &[f(1, 6, "=SUM(A2:A2)")])
        .unwrap();
    assert_eq!(book.selection_metrics().selections, 128);
    assert!(book.selection_metrics().identities > 128);
    drop(book);
    let memory = runtime.memory();
    assert!(runtime.finish().is_ok());
    assert_eq!(memory.stats().charged, 0);
}
