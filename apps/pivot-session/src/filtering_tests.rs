use super::*;
use pivot_workbook::filtering::{Condition, Definition, Literal, Operator, Predicate};
fn request(s: &Session, op: wire::request::Operation) -> wire::Request {
    wire::Request {
        version: 12,
        operation: op as i32,
        revision: s.revision.clone(),
        worksheet: s
            .book
            .as_ref()
            .map(|b| b.worksheet().uuid().to_string())
            .unwrap_or_default(),
        first: Some(location(Position::new(1, 1))),
        last: Some(location(Position::new(9, 1))),
        filter_first_visible: 1,
        filter_row_count: 10,
        ..Default::default()
    }
}
fn call(s: &mut Session, q: wire::Request) -> Result<wire::Response, String> {
    let mut r = wire::Response::default();
    s.handle(&q, &mut r)?;
    Ok(r)
}
fn frame(s: &mut Session) -> wire::Response {
    let q = request(s, wire::request::Operation::ReadFiltered);
    call(s, q).unwrap()
}
fn settle(s: &mut Session) -> wire::Response {
    let start = Instant::now();
    loop {
        let r = frame(s);
        if r.filter_status != 3 {
            return r;
        }
        assert!(start.elapsed().as_secs() < 20, "filter worker timeout");
        std::thread::sleep(std::time::Duration::from_millis(2));
    }
}
#[test]
fn operational_failure_is_transient_blocks_bulk_allows_single_edits_and_reapply_does_not_dirty() {
    let mut s = Session::new().unwrap();
    let create = request(&s, wire::request::Operation::Create);
    call(&mut s, create).unwrap();
    let book = s.book.as_mut().unwrap();
    let edits = (1..=9)
        .map(|row| Edit::SetDecimal {
            position: Position::new(row, 1),
            value: DecimalInput::from_decimal(1.into()),
        })
        .collect::<Vec<_>>();
    book.apply(book.expected(), &edits).unwrap();
    let definition = Definition {
        rectangle: Rectangle {
            first: Position::new(1, 1),
            last: Position::new(9, 1),
        },
        exclude_header: false,
        needs_review: false,
        conditions: vec![Condition {
            column: 1,
            either: false,
            predicates: vec![Predicate {
                operator: Operator::Contains,
                literal: Some(Literal::Text("a".repeat(2048) + "b")),
            }],
        }],
    };
    let mut set = request(&s, wire::request::Operation::SetFilter);
    set.filter_definition = Some(filtering::encode(&definition));
    call(&mut s, set).unwrap();
    assert_eq!(settle(&mut s).filter_status, 1);
    let book = s.book.as_mut().unwrap();
    let edits = (1..=9)
        .map(|row| Edit::SetText {
            position: Position::new(row, 1),
            text: "a".repeat(4096),
        })
        .collect::<Vec<_>>();
    book.apply(book.expected(), &edits).unwrap();
    let expected = book.expected();
    let graph = expected.graph;
    s.saved = Some(expected);
    s.revision = Uuid::new_v4().to_string();
    let failed = settle(&mut s);
    assert_eq!(failed.filter_status, 4);
    assert!(failed.filter_error.contains("filter work"));
    assert!(failed.filter_visible_mask.iter().all(|b| *b == 255));
    assert_eq!(s.book.as_ref().unwrap().expected(), expected);
    assert_eq!(s.saved, Some(expected));
    assert_eq!(
        s.book
            .as_ref()
            .unwrap()
            .filter(s.book.as_ref().unwrap().worksheet())
            .unwrap(),
        Some(&definition)
    );
    let copy = request(&s, wire::request::Operation::CopyBlock);
    assert!(call(&mut s, copy).unwrap_err().contains("suspended"));
    for row in 1..=9 {
        let mut edit = request(&s, wire::request::Operation::Apply);
        edit.edits.push(wire::Edit {
            at: Some(location(Position::new(row, 1))),
            action: Some(wire::edit::Action::Number(decimal(1.into()))),
        });
        call(&mut s, edit).unwrap();
    }
    assert_ne!(s.book.as_ref().unwrap().expected().graph, graph);
    assert_eq!(
        frame(&mut s).filter_status,
        4,
        "automatic requests must not retry operational failure"
    );
    let before = s.book.as_ref().unwrap().expected();
    let revision = s.revision.clone();
    let retry = request(&s, wire::request::Operation::ReapplyFilter);
    call(&mut s, retry).unwrap();
    assert_eq!(settle(&mut s).filter_status, 1);
    assert_eq!(s.book.as_ref().unwrap().expected(), before);
    assert_eq!(s.revision, revision);
    assert_eq!(s.close().unwrap(), 0);
}
#[test]
fn pending_refresh_is_unfiltered_and_closing_joins_the_owned_job() {
    let mut s = Session::new().unwrap();
    let create = request(&s, wire::request::Operation::Create);
    call(&mut s, create).unwrap();
    let mut set = request(&s, wire::request::Operation::SetFilter);
    set.filter_definition = Some(wire::FilterDefinition {
        first: Some(location(Position::new(1, 1))),
        last: Some(location(Position::new(65536, 1))),
        conditions: vec![wire::FilterCondition {
            column: 1,
            predicates: vec![wire::FilterPredicate {
                operator: Operator::Blank as u32,
                literal: None,
            }],
            ..Default::default()
        }],
        ..Default::default()
    });
    call(&mut s, set).unwrap();
    let mut edit = request(&s, wire::request::Operation::Apply);
    edit.edits.push(wire::Edit {
        at: Some(location(Position::new(1, 1))),
        action: Some(wire::edit::Action::Number(decimal(1.into()))),
    });
    call(&mut s, edit).unwrap();
    let first = frame(&mut s);
    assert_eq!(first.filter_status, 3);
    assert!(first.filter_visible_mask.iter().all(|b| *b == 255));
    assert_eq!(first.filter_rows[0], 1);
    let copy = request(&s, wire::request::Operation::CopyBlock);
    assert!(call(&mut s, copy).unwrap_err().contains("pending"));
    assert_eq!(s.close().unwrap(), 0);
}
