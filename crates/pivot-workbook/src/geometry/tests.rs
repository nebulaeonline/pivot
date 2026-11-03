use super::*;

fn rect(r: u32, c: u32, end_r: u32, end_c: u32) -> Rectangle {
    Rectangle {
        first: Position::new(r, c),
        last: Position::new(end_r, end_c),
    }
}
fn sizes(values: &[i64]) -> Vec<SizeRun> {
    values
        .iter()
        .enumerate()
        .map(|(i, v)| SizeRun {
            first: i as u32 + 1,
            last: i as u32 + 1,
            size: v * 64,
        })
        .collect()
}
fn dense(
    widths: &[Vec<i64>],
    heights: &[Vec<i64>],
    hs: &[u32],
    vs: &[u32],
    visible: &[u32],
) -> Vec<(Position, CellRect)> {
    let mut out = Vec::new();
    for &r in visible {
        for c in 1..=widths[0].len() as u32 {
            let h = hs.iter().rposition(|n| *n <= r).unwrap();
            let v = vs.iter().rposition(|n| *n <= c).unwrap();
            let x = widths[h][..(c - 1) as usize].iter().sum::<i64>() * 64;
            let y = visible
                .iter()
                .filter(|n| **n < r)
                .map(|n| heights[v][(*n - 1) as usize])
                .sum::<i64>()
                * 64;
            out.push((
                Position::new(r, c),
                CellRect {
                    left: x,
                    top: y,
                    right: x + widths[h][(c - 1) as usize] * 64,
                    bottom: y + heights[v][(r - 1) as usize] * 64,
                },
            ));
        }
    }
    out
}
fn overlaps(a: CellRect, b: CellRect) -> bool {
    a.left < b.right && b.left < a.right && a.top < b.bottom && b.top < a.bottom
}

#[test]
fn invoice_map_and_both_single_boundary_closures() {
    let w = [
        sizes(&[160, 32, 32, 64, 64, 128, 60, 36]),
        sizes(&[240, 48, 48, 48, 48, 48, 60, 36]),
        sizes(&[96, 96, 96, 64, 64, 64, 60, 36]),
    ];
    let left = vec![18; 36];
    let mut right = left.clone();
    for r in [1, 29] {
        right[r - 1] = 36;
    }
    for r in [2, 3, 30, 31] {
        right[r - 1] = 9;
    }
    let h = [sizes(&left), sizes(&right)];
    let horizontal = [
        Band {
            start: 1,
            sizes: &w[0],
        },
        Band {
            start: 5,
            sizes: &w[1],
        },
        Band {
            start: 29,
            sizes: &w[2],
        },
    ];
    let vertical = [
        Band {
            start: 1,
            sizes: &h[0],
        },
        Band {
            start: 7,
            sizes: &h[1],
        },
    ];
    let memory = Memory::new(MAX_PLAN_BYTES * 2);
    let cancel = CancellationToken::default();
    let map = Map::build(36, 8, &horizontal, &vertical, None, &memory, &cancel).unwrap();
    for (request, expected, n) in [
        (rect(2, 1, 3, 3), rect(2, 1, 3, 3), 1),
        (rect(4, 1, 5, 3), rect(1, 1, 28, 6), 2),
        (rect(2, 2, 2, 7), rect(1, 1, 4, 8), 2),
        (rect(4, 6, 5, 7), rect(1, 1, 28, 8), 4),
    ] {
        let s = map.section_closure(request).unwrap();
        assert_eq!(s.effective, expected);
        assert_eq!(s.sections, n);
        assert_eq!(s.enlarged, request != expected);
        assert_eq!(map.section_closure(expected).unwrap().effective, expected);
    }
    let all: Vec<_> = (1..=36).collect();
    let widths: Vec<_> = w
        .iter()
        .map(|v| v.iter().map(|r| r.size / 64).collect())
        .collect();
    for (p, r) in dense(&widths, &[left, right], &[1, 5, 29], &[1, 7], &all) {
        assert_eq!(map.cell_rect(p), Some(r));
        assert_eq!(map.hit(r.left, r.top), Some(p));
        assert_eq!(map.hit(r.right - 1, r.bottom - 1), Some(p));
    }
    drop(map);
    assert_eq!(memory.stats().charged, 0);
    let map = Map::build(36, 8, &horizontal, &vertical[..1], None, &memory, &cancel).unwrap();
    assert_eq!(
        map.section_closure(rect(4, 1, 5, 3)).unwrap().effective,
        rect(1, 1, 28, 8)
    );
    drop(map);
    let map = Map::build(36, 8, &horizontal[..1], &vertical, None, &memory, &cancel).unwrap();
    assert_eq!(
        map.section_closure(rect(2, 6, 2, 7)).unwrap().effective,
        rect(1, 1, 36, 8)
    );
}

#[test]
fn exhaustive_two_by_two_dense_overlap_and_hidden_row_oracle() {
    let memory = Memory::new(MAX_PLAN_BYTES);
    let cancel = CancellationToken::default();
    // Independent enumeration of all binary width/height choices. Test all four
    // visibility masks, including the empty one, with no production prefix code.
    for bits in 0u32..256 {
        let values: Vec<_> = (0..8).map(|i| 1 + i64::from((bits >> i) & 1)).collect();
        let widths = vec![values[..2].to_vec(), values[2..4].to_vec()];
        let heights = vec![values[4..6].to_vec(), values[6..].to_vec()];
        let wr = [sizes(&widths[0]), sizes(&widths[1])];
        let hr = [sizes(&heights[0]), sizes(&heights[1])];
        let h = [
            Band {
                start: 1,
                sizes: &wr[0],
            },
            Band {
                start: 2,
                sizes: &wr[1],
            },
        ];
        let v = [
            Band {
                start: 1,
                sizes: &hr[0],
            },
            Band {
                start: 2,
                sizes: &hr[1],
            },
        ];
        for mask in 0..4 {
            let visible: Vec<_> = (1..=2).filter(|r| mask & (1 << (r - 1)) != 0).collect();
            let cells = dense(&widths, &heights, &[1, 2], &[1, 2], &visible);
            let collision = cells
                .iter()
                .enumerate()
                .any(|(i, a)| cells[i + 1..].iter().any(|b| overlaps(a.1, b.1)));
            match Map::build(2, 2, &h, &v, Some(&visible), &memory, &cancel) {
                Ok(map) => {
                    assert!(!collision, "bits={bits}, mask={mask}");
                    for (p, r) in &cells {
                        assert_eq!(map.cell_rect(*p), Some(*r));
                    }
                    // Every half-point tests gaps, all edges and interiors.
                    for y in -1..=13 {
                        for x in -1..=13 {
                            let expected = cells
                                .iter()
                                .find(|(_, r)| r.contains(x * 32, y * 32))
                                .map(|(p, _)| *p);
                            assert_eq!(
                                map.hit(x * 32, y * 32),
                                expected,
                                "bits={bits}, mask={mask}, {x},{y}"
                            );
                        }
                    }
                }
                Err(MapError::Overlap { first, second }) => {
                    assert!(collision);
                    assert_ne!(first, second);
                    let a = cells.iter().find(|(p, _)| *p == first).unwrap().1;
                    let b = cells.iter().find(|(p, _)| *p == second).unwrap().1;
                    assert!(overlaps(a, b));
                }
                Err(e) => panic!("unexpected {e:?}"),
            }
            assert_eq!(memory.stats().charged, 0);
        }
    }
}

#[test]
fn sparse_maximum_extent_and_retained_admission() {
    let widths = [SizeRun {
        first: 1,
        last: crate::MAX_COLUMNS,
        size: DEFAULT_WIDTH,
    }];
    let heights = [SizeRun {
        first: 1,
        last: crate::MAX_ROWS,
        size: DEFAULT_HEIGHT,
    }];
    let h = [Band {
        start: 1,
        sizes: &widths,
    }];
    let v = [Band {
        start: 1,
        sizes: &heights,
    }];
    let cancel = CancellationToken::default();
    let memory = Memory::new(1024);
    let map = Map::build(
        crate::MAX_ROWS,
        crate::MAX_COLUMNS,
        &h,
        &v,
        None,
        &memory,
        &cancel,
    )
    .unwrap();
    assert_eq!(map.tile_count(), 1);
    let p = Position::new(crate::MAX_ROWS, crate::MAX_COLUMNS);
    let r = map.cell_rect(p).unwrap();
    assert_eq!(map.hit(r.right - 1, r.bottom - 1), Some(p));
    let charged = memory.stats().charged;
    let tiny = Memory::new(charged);
    let old = Map::build(
        crate::MAX_ROWS,
        crate::MAX_COLUMNS,
        &h,
        &v,
        None,
        &tiny,
        &cancel,
    )
    .unwrap();
    assert!(matches!(
        Map::build(
            crate::MAX_ROWS,
            crate::MAX_COLUMNS,
            &h,
            &v,
            None,
            &tiny,
            &cancel
        ),
        Err(MapError::Limit)
    ));
    assert!(tiny.stats().charged > 0);
    drop(old);
    assert_eq!(tiny.stats().charged, 0);
    assert!(
        Map::build(
            crate::MAX_ROWS,
            crate::MAX_COLUMNS,
            &h,
            &v,
            None,
            &tiny,
            &cancel
        )
        .is_ok()
    );
}

#[test]
fn invalid_input_and_cancellation_do_not_leak() {
    let memory = Memory::new(MAX_PLAN_BYTES);
    let cancel = CancellationToken::default();
    let invalid = [SizeRun {
        first: 1,
        last: 2,
        size: 0,
    }];
    let good = sizes(&[1, 1]);
    assert!(matches!(
        Map::build(
            2,
            2,
            &[Band {
                start: 1,
                sizes: &invalid
            }],
            &[Band {
                start: 1,
                sizes: &good
            }],
            None,
            &memory,
            &cancel
        ),
        Err(MapError::Invalid)
    ));
    for visible in [&[0][..], &[2, 1][..], &[1, 1][..], &[3][..]] {
        assert!(matches!(
            Map::build(
                2,
                2,
                &[Band {
                    start: 1,
                    sizes: &good
                }],
                &[Band {
                    start: 1,
                    sizes: &good
                }],
                Some(visible),
                &memory,
                &cancel
            ),
            Err(MapError::Invalid)
        ));
    }
    cancel.cancel();
    assert!(matches!(
        Map::build(
            2,
            2,
            &[Band {
                start: 1,
                sizes: &good
            }],
            &[Band {
                start: 1,
                sizes: &good
            }],
            None,
            &memory,
            &cancel
        ),
        Err(MapError::Cancelled)
    ));
    assert_eq!(memory.stats().charged, 0);
}

#[test]
fn viewport_pages_are_complete_bounded_and_clip_does_not_invent_cells() {
    let memory = Memory::new(MAX_PLAN_BYTES);
    let widths = [SizeRun {
        first: 1,
        last: 100,
        size: 64,
    }];
    let heights = [SizeRun {
        first: 1,
        last: 100,
        size: 64,
    }];
    let h = [Band {
        start: 1,
        sizes: &widths,
    }];
    let v = [Band {
        start: 1,
        sizes: &heights,
    }];
    let cancel = CancellationToken::default();
    let mut map = Map::build(100, 100, &h, &v, None, &memory, &cancel).unwrap();
    let dense = CellRect {
        left: 0,
        top: 0,
        right: 6400,
        bottom: 6400,
    };
    assert!(matches!(
        map.viewport_page(dense, 0, &memory, &cancel),
        Err(MapError::Limit)
    ));
    let view = CellRect {
        right: 20 * 64,
        bottom: 20 * 64,
        ..dense
    };
    let first = map.viewport_page(view, 0, &memory, &cancel).unwrap();
    assert_eq!(first.total, 400);
    assert_eq!(first.cells.len(), 256);
    let second = map
        .viewport_page(view, first.next.unwrap(), &memory, &cancel)
        .unwrap();
    assert_eq!(second.cells.len(), 144);
    assert_eq!(second.next, None);
    let mut seen = std::collections::BTreeSet::new();
    for cell in first.cells.iter().chain(&second.cells) {
        assert!(seen.insert(cell.position));
        assert_eq!(map.cell_rect(cell.position), Some(cell.rectangle));
    }
    map.clip_extent(3, 2).unwrap();
    assert!(map.cell_rect(Position::new(4, 1)).is_none());
    assert!(map.cell_rect(Position::new(1, 3)).is_none());
    assert_eq!(
        map.viewport_page(dense, 0, &memory, &cancel).unwrap().total,
        6
    );
    map.clip_extent(0, 2).unwrap();
    assert_eq!(map.hit(0, 0), None);
}

#[test]
fn rulers_keep_active_profiles_outside_view_and_through_hidden_sections() {
    let memory = Memory::new(MAX_PLAN_BYTES);
    let cancel = CancellationToken::default();
    let widths = [sizes(&[2, 1]), sizes(&[1, 2])];
    let heights = [SizeRun {
        first: 1,
        last: 4,
        size: 64,
    }];
    let h = [
        Band {
            start: 1,
            sizes: &widths[0],
        },
        Band {
            start: 3,
            sizes: &widths[1],
        },
    ];
    let v = [Band {
        start: 1,
        sizes: &heights,
    }];
    let map = Map::build(4, 2, &h, &v, None, &memory, &cancel).unwrap();
    let view = CellRect {
        left: 0,
        top: 128,
        right: 192,
        bottom: 256,
    };
    let headers = map
        .headers(Position::new(1, 1), view, &memory, &cancel)
        .unwrap();
    let a = headers
        .items
        .iter()
        .find(|h| h.column && h.index == 1)
        .unwrap();
    assert_eq!(
        (a.first, a.last, a.section_first, a.section_last),
        (0, 128, 1, 2)
    );
    assert_eq!(map.cell_rect(Position::new(3, 1)).unwrap().right, 64);
    let visible = [3, 4];
    let map = Map::build(4, 2, &h, &v, Some(&visible), &memory, &cancel).unwrap();
    let headers = map
        .headers(
            Position::new(1, 1),
            CellRect {
                top: 0,
                bottom: 128,
                ..view
            },
            &memory,
            &cancel,
        )
        .unwrap();
    assert_eq!(
        headers
            .items
            .iter()
            .find(|h| h.column && h.index == 1)
            .unwrap()
            .last,
        128
    );
    assert_eq!(
        headers
            .items
            .iter()
            .filter(|h| !h.column)
            .map(|h| h.index)
            .collect::<Vec<_>>(),
        vec![3, 4]
    );
    assert_eq!(
        map.scroll_anchor(Position::new(1, 2)),
        Some(Position::new(3, 2))
    );
}
