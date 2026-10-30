use pivot_workbook::{Limits, Position as P, Rectangle, Workbook, formatting::*};
fn rect(r: u32, c: u32, h: u32, w: u32) -> Rectangle {
    Rectangle {
        first: P::new(r, c),
        last: P::new(r + h - 1, c + w - 1),
    }
}
fn paint(b: &mut Workbook, r: Rectangle, patch: StylePatch) {
    b.format_sheet(
        b.worksheet(),
        b.expected(),
        r,
        patch,
        false,
        &Default::default(),
    )
    .unwrap();
}
#[test]
fn selection_overwrites_all_one_hundred_exceptions_without_touching_other_properties() {
    let mut b = Workbook::new("overwrite", Limits::default()).unwrap();
    let graph = b.expected().graph;
    for i in 0..100 {
        paint(
            &mut b,
            rect(1 + i * 10, 3, 1, 1),
            StylePatch {
                numeric: Field::Set(NumericFormat::Number),
                bold: Field::Set(true),
                ..Default::default()
            },
        );
    }
    paint(
        &mut b,
        rect(1, 1, 1000, 10),
        StylePatch {
            numeric: Field::Set(NumericFormat::Currency),
            ..Default::default()
        },
    );
    for row in 1..=1000 {
        for col in 1..=10 {
            let s = b.style_at(b.worksheet(), P::new(row, col)).unwrap();
            assert_eq!(s.numeric, NumericFormat::Currency);
            assert_eq!(s.bold, col == 3 && row % 10 == 1);
        }
    }
    paint(
        &mut b,
        rect(1, 1, 1000, 10),
        StylePatch {
            bold: Field::Set(false),
            ..Default::default()
        },
    );
    assert_eq!(b.style_regions(), 2);
    assert_eq!(b.stats().allocated_nodes, 0);
    assert_eq!(b.expected().graph, graph);
    let expected = b.expected();
    paint(
        &mut b,
        rect(1, 1, 1000, 10),
        StylePatch {
            bold: Field::Set(false),
            ..Default::default()
        },
    );
    assert_eq!(b.expected(), expected);
    let (s, mixed) = b
        .style_summary(b.worksheet(), rect(1, 1, 1000, 10))
        .unwrap();
    assert_eq!(mixed, 0);
    assert!(!s.bold);
}
#[test]
fn deterministic_random_spatial_edits_match_independent_dense_boolean_color_oracle() {
    let mut b = Workbook::new("oracle", Limits::default()).unwrap();
    let mut seed = 19u64;
    let mut bold = [[None; 16]; 16];
    let mut colors = [[None; 16]; 16];
    let mut default_bold = false;
    let mut next = || {
        seed = seed.wrapping_mul(6364136223846793005).wrapping_add(1);
        (seed >> 32) as usize
    };
    for step in 0..400 {
        let r = next() % 16;
        let c = next() % 16;
        let h = 1 + next() % (16 - r);
        let w = 1 + next() % (16 - c);
        let action = next() % 6;
        let patch = match action {
            0 | 1 => StylePatch {
                bold: Field::Set(action == 1),
                ..Default::default()
            },
            2 => StylePatch {
                background: Field::Set(Some(7)),
                ..Default::default()
            },
            3 => StylePatch {
                bold: Field::Inherit,
                ..Default::default()
            },
            4 => StylePatch {
                background: Field::Reset,
                ..Default::default()
            },
            _ => StylePatch {
                background: Field::Inherit,
                ..Default::default()
            },
        };
        paint(
            &mut b,
            rect(r as u32 + 1, c as u32 + 1, h as u32, w as u32),
            patch,
        );
        for row in r..r + h {
            for col in c..c + w {
                match action {
                    0 | 1 => bold[row][col] = Some(action == 1),
                    2 => colors[row][col] = Some(Some(7)),
                    3 => bold[row][col] = None,
                    4 => colors[row][col] = Some(None),
                    _ => colors[row][col] = None,
                }
            }
        }
        if step % 17 == 0 {
            default_bold = !default_bold;
            b.set_worksheet_default(
                b.worksheet(),
                b.expected(),
                StylePatch {
                    bold: Field::Set(default_bold),
                    ..Default::default()
                },
                &Default::default(),
            )
            .unwrap();
        }
        let viewport = b.style_viewport(b.worksheet(), rect(1, 1, 16, 16)).unwrap();
        for (p, s) in viewport {
            let r = p.row as usize - 1;
            let c = p.column as usize - 1;
            assert_eq!(
                s.bold,
                bold[r][c].unwrap_or(default_bold),
                "step {step} at {p:?}"
            );
            assert_eq!(s.background, colors[r][c].unwrap_or(None));
        }
    }
    assert_eq!(b.stats().allocated_nodes, 0);
}
#[test]
fn whole_sheet_is_ten_regions_and_reset_is_distinct_from_inheritance() {
    let mut b = Workbook::new("whole", Limits::default()).unwrap();
    let sheet = b.worksheet();
    let all = rect(1, 1, 65536, 1024);
    b.format_sheet(
        sheet,
        b.expected(),
        all,
        Default::default(),
        true,
        &Default::default(),
    )
    .unwrap();
    assert_eq!(b.style_regions(), 10);
    b.set_worksheet_default(
        sheet,
        b.expected(),
        StylePatch {
            bold: Field::Set(true),
            ..Default::default()
        },
        &Default::default(),
    )
    .unwrap();
    assert!(!b.style_at(sheet, P::new(65536, 1024)).unwrap().bold);
    paint(&mut b, all, StylePatch::inherit());
    assert_eq!(b.style_regions(), 0);
    assert!(b.style_at(sheet, P::new(65536, 1024)).unwrap().bold);
    assert_eq!(b.stats().allocated_nodes, 0);
}
#[test]
fn fragmentation_and_work_exhaustion_are_atomic() {
    let mut limits = Limits::default();
    limits.styles.positions = 2;
    let mut b = Workbook::new("fragment", limits).unwrap();
    paint(
        &mut b,
        rect(1, 1, 5, 5),
        StylePatch {
            bold: Field::Set(true),
            ..Default::default()
        },
    );
    let before = b.expected();
    assert!(
        b.format_sheet(
            b.worksheet(),
            before,
            rect(3, 3, 1, 1),
            StylePatch {
                bold: Field::Set(false),
                ..Default::default()
            },
            false,
            &Default::default()
        )
        .is_err()
    );
    assert_eq!(b.expected(), before);
    assert!(b.style_at(b.worksheet(), P::new(3, 3)).unwrap().bold);
    let mut limits = Limits::default();
    limits.styles.command_work = 0;
    let mut b = Workbook::new("work", limits).unwrap();
    let before = b.expected();
    assert!(
        b.format_sheet(
            b.worksheet(),
            before,
            rect(1, 1, 65536, 1024),
            StylePatch {
                bold: Field::Set(true),
                ..Default::default()
            },
            false,
            &Default::default()
        )
        .is_err()
    );
    assert_eq!(b.expected(), before);
    assert_eq!(b.style_regions(), 0);
}
