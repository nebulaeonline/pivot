#![cfg(feature = "persistent-core")]
use pivot_core::model::integration::{CancellationToken, IntegrationRuntime};
use pivot_workbook::*;
const A: Position = Position::new(1, 1);
const B: Position = Position::new(1, 2);
const C: Position = Position::new(1, 3);
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
#[test]
fn one_dispatch_atomic_batch_and_old_content_capture_survive_move_retire_and_reuse() {
    let runtime = IntegrationRuntime::standalone().unwrap();
    let mut w = Workbook::new_in("integrated", Limits::default(), &runtime).unwrap();
    let before = runtime.stats().mutations.accepted;
    w.apply(
        w.expected(),
        &[formula(C, "=A1+B1"), input(A, 100), input(B, 25)],
    )
    .unwrap();
    assert_eq!(runtime.stats().mutations.accepted, before + 1);
    let s = w.capture_local(w.expected()).unwrap();
    let original = s.pinned_content(C).unwrap().unwrap();
    let output = original.reference;
    assert_eq!(s.evaluate(&[output]).unwrap().values[0].1, Ok(125.into()));
    w.apply(
        w.expected(),
        &[
            Edit::Move {
                from: A,
                to: Position::new(2, 1),
            },
            input(A, 999),
            Edit::Retire(B),
        ],
    )
    .unwrap();
    assert_eq!(s.pinned_content(C).unwrap().unwrap(), original);
    assert_eq!(s.evaluate(&[output]).unwrap().values[0].1, Ok(125.into()));
    assert!(!w.validate(w.expected()).unwrap().is_empty());
    assert!(w.capture(w.expected()).is_err());
    w.apply(w.expected(), &[input(B, 50), formula(C, "=A1+B1")])
        .unwrap();
    assert_eq!(
        w.capture_local(w.expected())
            .unwrap()
            .evaluate(&[output])
            .unwrap()
            .values[0]
            .1,
        Ok(1049.into())
    );
    drop((w, s));
    let memory = runtime.memory();
    assert!(runtime.finish().is_ok());
    assert_eq!(memory.stats().charged, 0);
}
#[test]
fn cancelled_and_admission_failed_transactions_preserve_exact_state_and_can_retry() {
    let runtime = IntegrationRuntime::standalone().unwrap();
    let mut w = Workbook::new_in("rollback", Limits::default(), &runtime).unwrap();
    w.apply(w.expected(), &[input(A, 3)]).unwrap();
    let before = w.expected();
    let original = w.cell_at(A).unwrap();
    let cancel = CancellationToken::default();
    cancel.cancel();
    assert!(
        w.apply_cancelled(before, &[input(A, 7), input(B, 9)], &cancel)
            .is_err()
    );
    assert_eq!(w.expected(), before);
    assert_eq!(w.cell_at(A).unwrap(), original);
    runtime.flush();
    let memory = runtime.memory();
    let hold = memory
        .reserve(memory.stats().limit - memory.stats().charged)
        .unwrap();
    assert!(w.apply(before, &[input(A, 7), input(B, 9)]).is_err());
    assert_eq!(w.expected(), before);
    assert!(w.cell_at(B).unwrap().is_none());
    drop(hold);
    w.apply(before, &[input(A, 7), input(B, 9)]).unwrap();
    drop(w);
    assert!(runtime.finish().is_ok());
    assert_eq!(memory.stats().charged, 0);
}
#[test]
fn local_capture_allows_unrelated_invalid_drafts_and_rectangular_reads_are_bounded() {
    let runtime = IntegrationRuntime::standalone().unwrap();
    let mut w = Workbook::new_in("local", Limits::default(), &runtime).unwrap();
    w.apply(
        w.expected(),
        &[input(A, 3), formula(B, "=A1*2"), formula(C, "=(")],
    )
    .unwrap();
    assert!(w.capture(w.expected()).is_err());
    let output = w.cell_at(B).unwrap().unwrap().reference;
    assert_eq!(
        w.capture_local(w.expected())
            .unwrap()
            .evaluate(&[output])
            .unwrap()
            .values[0]
            .1,
        Ok(6.into())
    );
    let read = w
        .read(w.expected(), Rectangle { first: A, last: B }, 2)
        .unwrap();
    assert_eq!(read.cells.len(), 2);
    assert_eq!(
        w.read(w.expected(), Rectangle { first: A, last: B }, 1),
        Err(Error::Limit)
    );
    drop(w);
    let memory = runtime.memory();
    assert!(runtime.finish().is_ok());
    assert_eq!(memory.stats().charged, 0);
}

#[test]
fn owned_viewport_and_pinned_point_copy_out_require_transient_admission() {
    let runtime = IntegrationRuntime::standalone().unwrap();
    let mut w = Workbook::new_in("copy", Limits::default(), &runtime).unwrap();
    let source = format!("={}1", "1+".repeat(100));
    w.apply(w.expected(), &[formula(B, &source)]).unwrap();
    let capture = w.capture_local(w.expected()).unwrap();
    runtime.flush();
    let memory = runtime.memory();
    let spare = 4096 + std::mem::size_of::<CellView>() + 128;
    let hold = memory
        .reserve(memory.stats().limit - memory.stats().charged - spare)
        .unwrap();
    assert_eq!(w.cell_at(B), Err(Error::Limit));
    assert_eq!(capture.pinned_content(B), Err(Error::Limit));
    assert_eq!(
        w.read(w.expected(), Rectangle { first: B, last: B }, 1),
        Err(Error::Limit)
    );
    drop(hold);
    let Content::TextFormula(f) = capture.pinned_content(B).unwrap().unwrap().content else {
        panic!()
    };
    assert_eq!(f.original_source, source);
    drop((capture, w));
    assert!(runtime.finish().is_ok());
    assert_eq!(memory.stats().charged, 0);
}
