use pivot_types::{PivotDecimal, RoundingMode};
use pivot_workbook::formatting::*;
use pivot_workbook::{DecimalInput, Edit, Limits, Position as P, Rectangle, Workbook};
fn rect(a: u32, b: u32) -> Rectangle {
    Rectangle {
        first: P::new(1, a),
        last: P::new(2, b),
    }
}
#[test]
fn sparse_atomic_style_identity_and_move() {
    let mut b = Workbook::new("styles", Limits::default()).unwrap();
    let sheet = b.sheets()[0].id;
    let initial = b.expected();
    let stats = b.stats();
    let bold = StylePatch {
        bold: Field::Set(true),
        ..Default::default()
    };
    b.format_sheet(sheet, initial, rect(1, 2), bold, false, &Default::default())
        .unwrap();
    assert_eq!(b.expected().graph, initial.graph);
    assert_eq!(b.stats().allocated_nodes, stats.allocated_nodes);
    assert_eq!(b.stats().live_nodes, 0);
    assert_eq!(b.styled_positions(), 4);
    let styled = b.expected();
    b.format_sheet(sheet, styled, rect(1, 2), bold, false, &Default::default())
        .unwrap();
    assert_eq!(b.expected(), styled);
    assert!(
        b.format_sheet(sheet, initial, rect(1, 2), bold, false, &Default::default())
            .is_err()
    );
    let bad = StylePatch {
        places: Field::Set(29),
        ..Default::default()
    };
    assert!(
        b.format_sheet(sheet, styled, rect(1, 2), bad, false, &Default::default())
            .is_err()
    );
    assert_eq!(b.expected(), styled);
    let cancelled = pivot_core::model::integration::CancellationToken::default();
    cancelled.cancel();
    assert!(
        b.format_sheet(
            sheet,
            styled,
            rect(1, 2),
            StylePatch::default(),
            true,
            &cancelled
        )
        .is_err()
    );
    assert_eq!(b.expected(), styled);
    b.apply_sheet(
        sheet,
        b.expected(),
        &[Edit::SetDecimal {
            position: P::new(1, 1),
            value: DecimalInput::from_decimal(12.into()),
        }],
    )
    .unwrap();
    assert!(b.style_at(sheet, P::new(1, 1)).unwrap().bold);
    b.apply_sheet(
        sheet,
        b.expected(),
        &[Edit::Move {
            from: P::new(1, 1),
            to: P::new(3, 3),
        }],
    )
    .unwrap();
    assert!(b.style_at(sheet, P::new(3, 3)).unwrap().bold);
    assert_eq!(b.style_at(sheet, P::new(1, 1)).unwrap(), Style::default());
    b.apply_sheet(sheet, b.expected(), &[Edit::Clear(P::new(3, 3))])
        .unwrap();
    assert!(b.style_at(sheet, P::new(3, 3)).unwrap().bold);
    let all = Rectangle {
        first: P::new(1, 1),
        last: P::new(3, 3),
    };
    b.format_sheet(
        sheet,
        b.expected(),
        all,
        Default::default(),
        true,
        &Default::default(),
    )
    .unwrap();
    assert_eq!(b.styled_positions(), 0);
}
#[test]
fn blank_clipboard_fill_and_policy_rollback() {
    let mut limits = Limits::default();
    limits.styles.positions = 2;
    let mut b = Workbook::new("bounds", limits).unwrap();
    let sheet = b.sheets()[0].id;
    let one = Rectangle {
        first: P::new(1, 1),
        last: P::new(1, 1),
    };
    let patch = StylePatch {
        background: Field::Set(Some(0xffeb9c)),
        ..Default::default()
    };
    b.format_sheet(sheet, b.expected(), one, patch, false, &Default::default())
        .unwrap();
    let block = b.copy_sheet(sheet, b.expected(), one).unwrap();
    assert!(block.native.starts_with(b"PIVOTCB4"));
    let two = Rectangle {
        first: P::new(1, 2),
        last: P::new(1, 2),
    };
    b.paste_sheet(
        sheet,
        b.expected(),
        two,
        &block.native,
        true,
        &Default::default(),
    )
    .unwrap();
    assert_eq!(b.styled_positions(), 2);
    assert_eq!(b.stats().allocated_nodes, 0);
    let before = b.expected();
    let three = Rectangle {
        first: P::new(1, 3),
        last: P::new(1, 3),
    };
    assert!(
        b.paste_sheet(
            sheet,
            b.expected(),
            three,
            &block.native,
            true,
            &Default::default()
        )
        .is_err()
    );
    assert_eq!(before, b.expected());
    assert_eq!(b.stats().allocated_nodes, 0);
    b.format_sheet(
        sheet,
        b.expected(),
        two,
        Default::default(),
        true,
        &Default::default(),
    )
    .unwrap();
    b.fill_sheet(
        sheet,
        b.expected(),
        Rectangle {
            first: P::new(1, 1),
            last: P::new(1, 2),
        },
        pivot_workbook::FillDirection::Right,
        &Default::default(),
    )
    .unwrap();
    assert_eq!(b.styled_positions(), 2);
    assert_eq!(b.stats().allocated_nodes, 0);
    b.paste_sheet(sheet, b.expected(), two, b"123", false, &Default::default())
        .unwrap();
    assert_eq!(
        b.style_at(sheet, P::new(1, 2)).unwrap().background,
        Some(0xffeb9c)
    );
}
#[test]
fn exact_renderer_independent_integer_reference() {
    use RoundingMode::*;
    let modes = [
        ToEven,
        MidpointAwayFromZero,
        MidpointTowardZero,
        ToZero,
        AwayFromZero,
        ToNegativeInfinity,
        ToPositiveInfinity,
    ];
    // Independent integer rational nearest/floor/ceiling reference, including both signs and carries.
    for mode in modes {
        for n in -1999i64..=1999 {
            let value = PivotDecimal::from_parts(n.unsigned_abs() as u32, 0, 0, n < 0, 2).unwrap();
            let sign = if n < 0 { -1 } else { 1 };
            let abs = n.abs();
            let q = abs / 100;
            let rem = abs % 100;
            let expected = match mode {
                ToEven => q + i64::from(rem > 50 || (rem == 50 && q % 2 != 0)),
                MidpointAwayFromZero => q + i64::from(rem >= 50),
                MidpointTowardZero => q + i64::from(rem > 50),
                ToZero => q,
                AwayFromZero => q + i64::from(rem > 0),
                ToNegativeInfinity => q + i64::from(n < 0 && rem > 0),
                ToPositiveInfinity => q + i64::from(n > 0 && rem > 0),
            } * sign;
            let style = Style {
                numeric: NumericFormat::Number,
                places: 0,
                rounding: mode,
                ..Default::default()
            };
            assert_eq!(render_decimal(value, style).unwrap(), expected.to_string());
        }
    }
    let percent = Style {
        numeric: NumericFormat::Percentage,
        ..Default::default()
    };
    assert_eq!(
        render_decimal(PivotDecimal::MAX, percent).unwrap(),
        "7922816251426433759354395033500.00%"
    );
    let d: PivotDecimal = "1234.5".parse().unwrap();
    let style = Style {
        numeric: NumericFormat::Currency,
        grouping: true,
        ..Default::default()
    };
    assert_eq!(render_decimal(d, style).unwrap(), "$1,234.50");
    assert_eq!(d.to_string(), "1234.5");
    let d: PivotDecimal = "-0.001".parse().unwrap();
    assert_eq!(render_decimal(d, style).unwrap(), "$0.00");
    let d: PivotDecimal = "9.999".parse().unwrap();
    assert_eq!(render_decimal(d, style).unwrap(), "$10.00");
    for style in [Style::default(), percent, style] {
        assert_eq!(Style::decode(&style.encode()).unwrap(), style);
    }
}

#[test]
fn mixed_patch_reset_cycles_and_capture_do_not_change_dependencies() {
    let mut b = Workbook::new("positions", Limits::default()).unwrap();
    let sheet = b.sheets()[0].id;
    let a = P::new(1, 1);
    let target = P::new(1, 2);
    b.apply_sheet(
        sheet,
        b.expected(),
        &[
            Edit::SetEntry {
                position: a,
                source: "1.2345".into(),
            },
            Edit::SetEntry {
                position: target,
                source: "=A1*2".into(),
            },
        ],
    )
    .unwrap();
    let capture = b.capture_sheet(sheet, b.expected()).unwrap();
    let output = capture.cell_at(target).unwrap();
    let graph = b.expected().graph;
    let allocated = b.stats().allocated_nodes;
    let one = Rectangle { first: a, last: a };
    for _ in 0..32 {
        b.format_sheet(
            sheet,
            b.expected(),
            one,
            StylePatch {
                bold: Field::Set(true),
                background: Field::Set(Some(7)),
                ..Default::default()
            },
            false,
            &Default::default(),
        )
        .unwrap();
        b.format_sheet(
            sheet,
            b.expected(),
            one,
            StylePatch {
                italic: Field::Set(true),
                ..Default::default()
            },
            false,
            &Default::default(),
        )
        .unwrap();
        let style = b.style_at(sheet, a).unwrap();
        assert!(style.bold && style.italic);
        assert_eq!(style.background, Some(7));
        b.format_sheet(
            sheet,
            b.expected(),
            one,
            StylePatch::default(),
            true,
            &Default::default(),
        )
        .unwrap();
        assert_eq!(b.styled_positions(), 0);
    }
    assert_eq!(b.expected().graph, graph);
    assert_eq!(b.stats().allocated_nodes, allocated);
    assert_eq!(
        capture.evaluate(&[output]).unwrap().values[0].1,
        Ok("2.469".parse().unwrap())
    );
    // Definition replacement does not follow the old stable target's identity.
    b.format_sheet(
        sheet,
        b.expected(),
        one,
        StylePatch {
            bold: Field::Set(true),
            ..Default::default()
        },
        false,
        &Default::default(),
    )
    .unwrap();
    b.apply_sheet(
        sheet,
        b.expected(),
        &[Edit::SetEntry {
            position: a,
            source: "=3+4".into(),
        }],
    )
    .unwrap();
    assert!(b.style_at(sheet, a).unwrap().bold);
    let dest = P::new(2, 2);
    b.format_sheet(
        sheet,
        b.expected(),
        Rectangle {
            first: dest,
            last: dest,
        },
        StylePatch {
            italic: Field::Set(true),
            ..Default::default()
        },
        false,
        &Default::default(),
    )
    .unwrap();
    b.apply_sheet(
        sheet,
        b.expected(),
        &[Edit::Move {
            from: target,
            to: dest,
        }],
    )
    .unwrap();
    assert_eq!(b.style_at(sheet, dest).unwrap(), Style::default());
}
#[test]
fn all_display_precisions_and_malformed_style_bytes() {
    let mut value = Style {
        numeric: NumericFormat::Percentage,
        ..Default::default()
    };
    for places in 0..=28 {
        value.places = places;
        let display = render_decimal(PivotDecimal::MAX, value).unwrap();
        assert!(display.len() <= 128);
        assert!(display.starts_with("7922816251426433759354395033500"));
    }
    value.numeric = NumericFormat::Number;
    value.places = 28;
    assert_eq!(
        render_decimal("0.0000000000000000000000000001".parse().unwrap(), value).unwrap(),
        "0.0000000000000000000000000001"
    );
    let mut bytes = value.encode();
    bytes[2] = 7;
    assert!(Style::decode(&bytes).is_err());
    bytes = value.encode();
    bytes[8] = 1;
    assert!(Style::decode(&bytes).is_err());
    bytes = value.encode();
    bytes[9..13].copy_from_slice(&0x1000000u32.to_le_bytes());
    assert!(Style::decode(&bytes).is_err());
}
