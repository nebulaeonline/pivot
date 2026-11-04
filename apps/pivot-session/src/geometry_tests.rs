use super::*;
fn request(s: &Session, op: wire::request::Operation) -> wire::Request {
    wire::Request {
        version: 17,
        operation: op as i32,
        revision: s.revision.clone(),
        worksheet: s
            .book
            .as_ref()
            .map(|b| b.worksheet().uuid().to_string())
            .unwrap_or_default(),
        ..Default::default()
    }
}
fn call(s: &mut Session, q: wire::Request) -> Result<wire::Response, String> {
    let mut r = wire::Response::default();
    s.handle(&q, &mut r)?;
    Ok(r)
}
#[test]
fn geometry_uses_canonical_capture_scope_and_joint_four_lease_ceiling() {
    use wire::request::Operation as Op;
    let mut s = Session::new().unwrap();
    let q = request(&s, Op::Create);
    call(&mut s, q).unwrap();
    let q = request(&s, Op::Capture);
    let old = call(&mut s, q).unwrap().capture;
    let mut q = request(&s, Op::GeometryEdit);
    q.geometry_commands.push(wire::GeometryCommand {
        action: wire::geometry_command::Action::AddBreak as i32,
        horizontal: true,
        at: 5,
        ..Default::default()
    });
    call(&mut s, q).unwrap();
    let q = request(&s, Op::Capture);
    let new = call(&mut s, q).unwrap().capture;
    for (capture, enlarged) in [(&old, false), (&new, true)] {
        let mut q = request(&s, Op::GeometrySelection);
        q.capture = capture.clone();
        q.first = Some(location(Position::new(4, 1)));
        q.last = Some(location(Position::new(5, 2)));
        let response = call(&mut s, q).unwrap();
        assert!(
            response.capture.is_empty(),
            "A geometry read borrows an existing capture"
        );
        assert_eq!(response.capture_identity, *capture);
        let frame = response.geometry.unwrap();
        assert_eq!(frame.enlarged, enlarged);
        assert_eq!(frame.sections, if enlarged { 2 } else { 1 });
    }
    let mut q = request(&s, Op::GeometryMap);
    q.capture = new.clone();
    q.geometry_viewport = Some(wire::GeometryRect {
        left: 0,
        top: 0,
        right: 4512 * 10,
        bottom: 1248 * 20,
    });
    let frame = call(&mut s, q.clone()).unwrap().geometry.unwrap();
    assert_eq!(frame.total, 200);
    assert_eq!(frame.cells.len(), 200);
    let section_column = frame
        .local_headers
        .iter()
        .find(|h| {
            h.column
                && h.index == 3
                && h.section_first
                    .as_ref()
                    .is_some_and(|p| p.row == 5 && p.column == 1)
        })
        .unwrap();
    assert_eq!(section_column.edge, 1248 * 4);
    assert_eq!(section_column.section_last.as_ref().unwrap().row, 65536);
    assert_eq!(section_column.section_last.as_ref().unwrap().column, 1024);
    assert_eq!(
        frame
            .local_headers
            .iter()
            .filter(|h| h.column && h.index == 3)
            .count(),
        2
    );
    assert!(s.captures[&new].geometry.is_some());
    q.geometry_viewport.as_mut().unwrap().right += 1;
    assert!(call(&mut s, q).is_err());
    let mut wrong = request(&s, Op::GeometryMap);
    wrong.capture = new;
    wrong.worksheet = Uuid::new_v4().to_string();
    assert!(call(&mut s, wrong).is_err());
    for _ in 0..2 {
        let q = request(&s, Op::Capture);
        call(&mut s, q).unwrap();
    }
    let q = request(&s, Op::Capture);
    assert!(call(&mut s, q).is_err());
    assert_eq!(s.captures.len(), 4);
    assert_eq!(s.close().unwrap(), 0);
}

#[test]
fn data_driven_filter_collision_suspends_without_refusing_data_or_resuming_silently() {
    use pivot_workbook::{
        filtering::{Condition, Definition, Literal, Operator, Predicate},
        geometry::{Command, SizeRun},
    };
    use wire::request::Operation as Op;
    let mut s = Session::new().unwrap();
    let q = request(&s, Op::Create);
    call(&mut s, q).unwrap();
    let book = s.book.as_mut().unwrap();
    let scope = book
        .create_grid_window(book.expected(), "Collision", 3, 2)
        .unwrap()
        .into();
    let p = |row, column| Position::new(row, column);
    book.apply_scope(
        scope,
        book.expected(),
        &(1..=3)
            .map(|row| Edit::SetBoolean {
                position: p(row, 1),
                value: true,
            })
            .collect::<Vec<_>>(),
    )
    .unwrap();
    let sizes = |values: &[i64]| {
        values
            .iter()
            .enumerate()
            .map(|(i, n)| SizeRun {
                first: i as u32 + 1,
                last: i as u32 + 1,
                size: n * 64,
            })
            .collect()
    };
    book.edit_geometry(
        scope,
        book.expected(),
        &[
            Command::AddBreak {
                horizontal: true,
                before: 3,
            },
            Command::AddBreak {
                horizontal: false,
                before: 2,
            },
            Command::SetProfile {
                horizontal: true,
                at: 1,
                sizes: sizes(&[1, 1]),
            },
            Command::SetProfile {
                horizontal: true,
                at: 3,
                sizes: sizes(&[2, 1]),
            },
            Command::SetProfile {
                horizontal: false,
                at: 1,
                sizes: sizes(&[1, 2, 1]),
            },
            Command::SetProfile {
                horizontal: false,
                at: 2,
                sizes: sizes(&[2, 1, 1]),
            },
        ],
        &Default::default(),
    )
    .unwrap();
    let definition = Definition {
        rectangle: Rectangle {
            first: p(1, 1),
            last: p(3, 2),
        },
        exclude_header: false,
        needs_review: false,
        conditions: vec![Condition {
            column: 1,
            either: false,
            predicates: vec![Predicate {
                operator: Operator::Equal,
                literal: Some(Literal::Boolean(true)),
            }],
        }],
    };
    let scoped = |s: &Session, op| {
        let mut q = request(s, op);
        q.worksheet = scope.uuid().to_string();
        q.window_scope = true;
        q.first = Some(location(p(1, 1)));
        q.last = Some(location(p(3, 2)));
        q.filter_first_visible = 1;
        q.filter_row_count = 3;
        q
    };
    let mut q = scoped(&s, Op::SetFilter);
    q.filter_definition = Some(filtering::encode(&definition));
    call(&mut s, q).unwrap();
    let book = s.book.as_mut().unwrap();
    book.apply_scope(
        scope,
        book.expected(),
        &[Edit::SetBoolean {
            position: p(2, 1),
            value: false,
        }],
    )
    .unwrap();
    let start = Instant::now();
    let suspended = loop {
        let q = scoped(&s, Op::ReadFiltered);
        let frame = call(&mut s, q).unwrap();
        if frame.filter_status == 2 {
            break frame;
        }
        assert!(start.elapsed().as_secs() < 10);
        std::thread::sleep(std::time::Duration::from_millis(2));
    };
    assert!(suspended.filter_summary.as_ref().unwrap().needs_review);
    assert!(!suspended.filter_error.is_empty());
    assert_eq!(suspended.filter_rows, vec![1, 2, 3]);
    assert!(
        !s.book
            .as_ref()
            .unwrap()
            .filter(scope)
            .unwrap()
            .unwrap()
            .needs_review,
        "Criteria remain stored unchanged"
    );
    let q = scoped(&s, Op::Capture);
    let capture = call(&mut s, q).unwrap().capture;
    let mut q = scoped(&s, Op::GeometryMap);
    q.capture = capture;
    q.geometry_viewport = Some(wire::GeometryRect {
        left: 0,
        top: 0,
        right: 1000,
        bottom: 1000,
    });
    let map = call(&mut s, q).unwrap().geometry.unwrap();
    assert_eq!(map.filter_status, 2);
    assert_ne!(map.visible_mask[0] & 2, 0);
    let book = s.book.as_mut().unwrap();
    book.apply_scope(
        scope,
        book.expected(),
        &[Edit::SetBoolean {
            position: p(2, 1),
            value: true,
        }],
    )
    .unwrap();
    let q = scoped(&s, Op::ReadFiltered);
    assert_eq!(call(&mut s, q).unwrap().filter_status, 2);
    let q = scoped(&s, Op::ReapplyFilter);
    call(&mut s, q).unwrap();
    let start = Instant::now();
    loop {
        let q = scoped(&s, Op::ReadFiltered);
        if call(&mut s, q).unwrap().filter_status == 1 {
            break;
        }
        assert!(start.elapsed().as_secs() < 10);
        std::thread::sleep(std::time::Duration::from_millis(2));
    }
    assert_eq!(s.close().unwrap(), 0);
}
