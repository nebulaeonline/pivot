use super::*;
use wire::request::Operation as O;
fn request(s: &Session, op: O) -> wire::Request {
    wire::Request {
        version: 12,
        operation: op as i32,
        revision: s.revision.clone(),
        computational_state: s.computational_state.clone(),
        worksheet: s
            .book
            .as_ref()
            .map(|b| b.worksheet().uuid().to_string())
            .unwrap_or_default(),
        first: Some(location(Position::new(2, 1))),
        last: Some(location(Position::new(5, 2))),
        ..Default::default()
    }
}
fn call(s: &mut Session, q: wire::Request) -> Result<wire::Response, String> {
    let mut r = wire::Response::default();
    s.handle(&q, &mut r)?;
    Ok(r)
}
fn settle(s: &mut Session) {
    let start = Instant::now();
    loop {
        let mut q = request(s, O::ReadFiltered);
        q.filter_row_count = 10;
        let r = call(s, q).unwrap();
        if r.filter_status != 3 {
            return;
        }
        assert!(start.elapsed().as_secs() < 5);
        std::thread::sleep(std::time::Duration::from_millis(2));
    }
}
fn fixture() -> Session {
    let mut s = Session::new().unwrap();
    let q = request(&s, O::Create);
    call(&mut s, q).unwrap();
    let mut q = request(&s, O::Apply);
    for row in [2, 5, 10, 12] {
        q.edits.push(wire::Edit {
            at: Some(location(Position::new(row, 1))),
            action: Some(wire::edit::Action::Text("001234".into())),
        });
    }
    q.edits.push(wire::Edit {
        at: Some(location(Position::new(5, 2))),
        action: Some(wire::edit::Action::Formula("=A2".into())),
    });
    q.edits.push(wire::Edit {
        at: Some(location(Position::new(11, 4))),
        action: Some(wire::edit::Action::Text("hidden sentinel".into())),
    });
    call(&mut s, q).unwrap();
    let mut q = request(&s, O::SetFilter);
    q.filter_definition = Some(wire::FilterDefinition {
        first: Some(location(Position::new(1, 1))),
        last: Some(location(Position::new(30, 5))),
        conditions: vec![wire::FilterCondition {
            column: 1,
            predicates: vec![wire::FilterPredicate {
                literal: Some(wire::filter_predicate::Literal::Text("001234".into())),
                ..Default::default()
            }],
            ..Default::default()
        }],
        ..Default::default()
    });
    call(&mut s, q).unwrap();
    settle(&mut s);
    s
}
fn operation(s: &mut Session) -> wire::Request {
    let q = request(s, O::CopyVisible);
    let copied = call(s, q).unwrap();
    assert_eq!(copied.visible_rows, [2, 5]);
    let mut q = request(s, O::PasteVisible);
    q.first = Some(location(Position::new(9, 4)));
    q.last = Some(location(Position::new(13, 5)));
    q.clipboard = copied.clipboard;
    q.native_clipboard = true;
    q.clipboard_kind = 6;
    q
}
fn prepare(s: &mut Session, operation: &wire::Request) -> wire::Response {
    let mut q = request(s, O::PrepareVisiblePaste);
    q.filter_operation = operation.encode_to_vec();
    call(s, q).unwrap()
}
#[test]
fn exact_rows_hidden_integrity_blank_anchor_and_single_use_plan() {
    let mut s = fixture();
    let mut q = operation(&mut s);
    let plan = prepare(&mut s, &q);
    assert_eq!(plan.visible_rows, [10, 12]);
    assert_eq!(plan.visible_first_column, 4);
    assert_eq!(plan.visible_width, 2);
    assert_eq!(plan.visible_skipped_rows, 3);
    let hidden = s
        .book
        .as_ref()
        .unwrap()
        .cell_at(Position::new(11, 4))
        .unwrap();
    q.filter_confirmation = plan.filter_confirmation;
    call(&mut s, q.clone()).unwrap();
    let book = s.book.as_ref().unwrap();
    assert_eq!(book.cell_at(Position::new(11, 4)).unwrap(), hidden);
    assert_eq!(
        book.display_formula(
            book.cell_at(Position::new(12, 5))
                .unwrap()
                .unwrap()
                .reference
        )
        .unwrap()
        .unwrap(),
        "=D9"
    );
    assert!(matches!(
        book.cell_at(Position::new(9, 4)).unwrap().unwrap().content,
        Content::Blank
    ));
    assert!(call(&mut s, q).is_err());
    assert_eq!(s.close().unwrap(), 0);
}
#[test]
fn changed_payload_generation_and_shape_never_authorize_another_write() {
    let mut s = fixture();
    let mut q = operation(&mut s);
    let before = s.book.as_ref().unwrap().expected();
    let plan = prepare(&mut s, &q);
    q.filter_confirmation = plan.filter_confirmation;
    q.clipboard.push(0);
    assert!(call(&mut s, q).is_err());
    assert_eq!(s.book.as_ref().unwrap().expected(), before);
    let mut q = operation(&mut s);
    let plan = prepare(&mut s, &q);
    q.filter_confirmation = plan.filter_confirmation;
    let reapply = request(&s, O::ReapplyFilter);
    call(&mut s, reapply).unwrap();
    settle(&mut s);
    assert!(call(&mut s, q).is_err());
    assert_eq!(s.book.as_ref().unwrap().expected(), before);
    let mut q = operation(&mut s);
    q.last = Some(location(Position::new(10, 5)));
    let mut preview = request(&s, O::PrepareVisiblePaste);
    preview.filter_operation = q.encode_to_vec();
    assert!(call(&mut s, preview).is_err());
    assert_eq!(s.close().unwrap(), 0);
}
