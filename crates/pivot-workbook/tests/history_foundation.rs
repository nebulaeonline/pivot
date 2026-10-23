#![cfg(feature = "history-experiment")]
use pivot_core::model::integration::{CancellationToken, IntegrationRuntime};
use pivot_workbook::*;
fn p(r: u32, c: u32) -> Position {
    Position::new(r, c)
}
fn e(r: u32, c: u32, s: &str) -> Edit {
    Edit::SetEntry {
        position: p(r, c),
        source: s.into(),
    }
}
#[test]
fn root_activation_preserves_drafts_and_bindings_with_fresh_authority_and_no_aba() {
    let rt = IntegrationRuntime::standalone().unwrap();
    let mut b = Workbook::new_in("roots", Limits::default(), &rt).unwrap();
    b.apply(
        b.expected(),
        &[
            e(1, 1, "10"),
            e(1, 2, "=A1+1"),
            e(1, 4, "=("),
            e(1, 6, "=F1"),
        ],
    )
    .unwrap();
    let authority_a = b.expected();
    let content_a = b.history_content_id();
    let scalar = b.cell_at(p(1, 2)).unwrap().unwrap();
    let invalid = b.cell_at(p(1, 4)).unwrap().unwrap();
    let cycle = b.cell_at(p(1, 6)).unwrap().unwrap();
    let a = b.pin_history_root(authority_a).unwrap();
    b.apply(b.expected(), &[e(2, 1, "20"), e(2, 2, "=SUM(A1:A2)")])
        .unwrap();
    let node_b = b.history_runtime_node(p(2, 2)).unwrap();
    let cell_b = b.cell_at(p(2, 2)).unwrap().unwrap();
    let root_b = b.pin_history_root(b.expected()).unwrap();
    let content_b = b.history_content_id();
    let capture = b.capture_local(b.expected()).unwrap();
    let output = capture.cell_at(p(2, 2)).unwrap();
    assert_eq!(
        capture.evaluate(&[output]).unwrap().values[0].1,
        Ok(30.into())
    );
    b.activate_history_root(b.expected(), &a, &Default::default())
        .unwrap();
    assert_eq!(b.history_content_id(), content_a);
    assert_ne!(b.expected(), authority_a);
    assert_eq!(b.cell_at(p(1, 2)).unwrap().unwrap(), scalar);
    assert_eq!(b.cell_at(p(1, 4)).unwrap().unwrap(), invalid);
    assert_eq!(b.cell_at(p(1, 6)).unwrap().unwrap(), cycle);
    assert!(b.apply(authority_a, &[e(9, 9, "1")]).is_err());
    assert!(b.cell_at(p(2, 2)).unwrap().is_none());
    b.apply(b.expected(), &[e(2, 1, "7"), e(2, 2, "=MAX(A1:A2)")])
        .unwrap();
    let node_c = b.history_runtime_node(p(2, 2)).unwrap();
    let cell_c = b.cell_at(p(2, 2)).unwrap().unwrap();
    assert!(node_c > node_b);
    assert_ne!(cell_c.reference, cell_b.reference);
    assert_ne!(cell_c.node, cell_b.node);
    assert!(b.history_metrics().unwrap().lifetime_selections >= 2);
    let selection = |c: &CellView| match &c.content {
        Content::Formula { tokens, .. } => tokens.iter().find_map(|t| {
            if let Token::Sum { id, .. } = t {
                Some(*id)
            } else {
                None
            }
        }),
        Content::TextFormula(f) => f.tokens.iter().find_map(|t| {
            if let Token::Sum { id, .. } = t {
                Some(*id)
            } else {
                None
            }
        }),
        _ => None,
    };
    assert_ne!(selection(&cell_b), selection(&cell_c));
    assert_eq!(
        capture.evaluate(&[output]).unwrap().values[0].1,
        Ok(30.into())
    );
    b.activate_history_root(b.expected(), &root_b, &Default::default())
        .unwrap();
    assert_eq!(b.history_content_id(), content_b);
    assert_eq!(b.cell_at(p(2, 2)).unwrap().unwrap(), cell_b);
    assert_eq!(b.cell_at(p(1, 6)).unwrap().unwrap(), cycle);
    b.activate_history_root(b.expected(), &a, &Default::default())
        .unwrap();
    drop(root_b);
    rt.flush();
    b.apply(b.expected(), &[e(2, 1, "9"), e(2, 2, "=COUNT(A1:A2)")])
        .unwrap();
    assert!(b.history_runtime_node(p(2, 2)).unwrap() > node_c);
    assert_eq!(
        capture.evaluate(&[output]).unwrap().values[0].1,
        Ok(30.into())
    );
    drop((a, b, capture));
    rt.flush();
    let m = rt.memory();
    assert!(rt.finish().is_ok());
    assert_eq!(m.stats().charged, 0);
}
#[test]
fn cancelled_foreign_and_failed_candidates_preserve_state_and_lifetime_quota() {
    let rt = IntegrationRuntime::standalone().unwrap();
    let mut b = Workbook::new_in(
        "bounds",
        Limits {
            allocated_cells: 2,
            ..Limits::default()
        },
        &rt,
    )
    .unwrap();
    b.apply(b.expected(), &[e(1, 1, "1")]).unwrap();
    let a = b.pin_history_root(b.expected()).unwrap();
    b.apply(b.expected(), &[e(2, 1, "2")]).unwrap();
    b.activate_history_root(b.expected(), &a, &Default::default())
        .unwrap();
    let before = b.expected();
    let content = b.history_content_id();
    assert!(b.apply(before, &[e(3, 1, "3")]).is_err());
    assert_eq!(b.expected(), before);
    assert_eq!(b.history_content_id(), content);
    let cancel = CancellationToken::default();
    cancel.cancel();
    assert!(b.activate_history_root(before, &a, &cancel).is_err());
    assert_eq!(b.expected(), before);
    let other = Workbook::new_in("foreign", Limits::default(), &rt).unwrap();
    let root = other.pin_history_root(other.expected()).unwrap();
    assert!(
        b.activate_history_root(before, &root, &Default::default())
            .is_err()
    );
    assert_eq!(b.expected(), before);
    let mut candidate = b.stage_history_candidate(before).unwrap();
    assert!(candidate.apply(before, &[e(3, 1, "3")]).is_err());
    assert_eq!(b.expected(), before);
    drop((root, other, candidate, a, b));
    rt.flush();
    let m = rt.memory();
    assert!(rt.finish().is_ok());
    assert_eq!(m.stats().charged, 0);
}
#[test]
fn weight_queries_and_repeated_replacements_have_no_history_scan_or_gross_traffic_growth() {
    let rt = IntegrationRuntime::standalone().unwrap();
    let baseline = rt.memory().stats().allocated;
    let mut b = Workbook::new_in("weights", Limits::default(), &rt).unwrap();
    b.apply(b.expected(), &[e(1, 1, "3"), e(1, 2, "=A1+2")])
        .unwrap();
    rt.flush();
    let weight = b.history_weight().unwrap();
    let footprint = rt.memory().stats().allocated - baseline;
    assert!(
        weight >= footprint,
        "weight {weight}, allocated {footprint}"
    );
    for i in 0..40 {
        b.apply(b.expected(), &[e(1, 1, if i % 2 == 0 { "4" } else { "3" })])
            .unwrap();
        assert_eq!(b.history_weight().unwrap(), weight);
        assert_eq!(b.history_metrics().unwrap().weight_scan_visits, 0);
    }
    drop(b);
    rt.flush();
    let m = rt.memory();
    assert!(rt.finish().is_ok());
    assert_eq!(m.stats().charged, 0);
}

#[test]
fn mandatory_pair_and_actual_domain_admission_fail_without_installing_candidate() {
    let rt = IntegrationRuntime::standalone().unwrap();
    let mut b = Workbook::new_in("admission", Limits::default(), &rt).unwrap();
    b.apply(b.expected(), &[e(1, 1, "1")]).unwrap();
    let before = b.expected();
    let id = b.history_content_id();
    let a = b.pin_history_root(before).unwrap();
    let mut candidate = b.stage_history_candidate(before).unwrap();
    candidate.apply(before, &[e(1, 1, "2")]).unwrap();
    let after = candidate.pin_history_root(candidate.expected()).unwrap();
    let mandatory = a.weight() + after.weight();
    let injected_bound = mandatory - 1;
    assert!(mandatory > injected_bound);
    drop((candidate, after));
    rt.flush();
    assert_eq!(b.expected(), before);
    assert_eq!(b.history_content_id(), id);
    let memory = rt.memory();
    let charge = memory
        .reserve(memory.stats().limit - memory.stats().charged)
        .unwrap();
    assert!(b.pin_history_root(before).is_err());
    let mut candidate = b.stage_history_candidate(before).unwrap();
    assert!(candidate.apply(before, &[e(2, 1, "2")]).is_err());
    assert_eq!(candidate.expected(), before);
    assert_eq!(b.expected(), before);
    drop(charge);
    drop(candidate);
    b.activate_history_root(before, &a, &Default::default())
        .unwrap();
    drop((a, b));
    rt.flush();
    let m = rt.memory();
    assert!(rt.finish().is_ok());
    assert_eq!(m.stats().charged, 0);
}
