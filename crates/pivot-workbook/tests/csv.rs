use pivot_core::model::integration::IntegrationRuntime;
use pivot_types::PivotValue as V;
use pivot_workbook::{
    Content, Edit, Limits, Position, Rectangle, Workbook,
    csv::{self, Options, Policy},
};

fn p(row: u32, column: u32) -> Position {
    Position::new(row, column)
}
fn typed() -> Options {
    Options {
        typed: true,
        ..Options::default()
    }
}
fn values(source: &str, options: Options) -> Vec<V> {
    let runtime = IntegrationRuntime::standalone().unwrap();
    let table = csv::parse(source.as_bytes(), Policy::default(), Limits::default()).unwrap();
    let book = table
        .workbook(options, Limits::default(), &runtime)
        .unwrap();
    let capture = book
        .capture_sheet(book.worksheet(), book.expected())
        .unwrap();
    (1..=table.rows)
        .flat_map(|row| (1..=table.columns).map(move |column| p(row, column)))
        .map(|pos| {
            capture
                .cell_at(pos)
                .map(|r| {
                    capture
                        .evaluate_values(&[r])
                        .unwrap()
                        .values
                        .remove(0)
                        .1
                        .unwrap()
                })
                .unwrap_or(V::Blank)
        })
        .collect()
}
#[test]
fn leading_zeros_quotes_and_empty_fields_are_independent() {
    let source = "001234,+001234,-001234,00.50,0000,0,TRUE,\"001234\",\"+001234\",\"00.50\",\"TRUE\",\"\",\n";
    let a = values(source, typed());
    assert_eq!(a[0], V::String("001234".into()));
    assert_eq!(a[1], V::Decimal("1234".parse().unwrap()));
    assert_eq!(a[2], V::Decimal("-1234".parse().unwrap()));
    assert_eq!(a[3], V::Decimal("0.50".parse().unwrap()));
    assert_eq!(a[4], V::String("0000".into()));
    assert_eq!(a[5], V::Decimal("0".parse().unwrap()));
    assert_eq!(a[6], V::Boolean(true));
    assert_eq!(
        &a[7..],
        &[
            V::String("001234".into()),
            V::String("+001234".into()),
            V::String("00.50".into()),
            V::String("TRUE".into()),
            V::String("".into()),
            V::Blank
        ]
    );
    for preserve in [false, true] {
        let b = values(
            source,
            Options {
                recognize_quoted: true,
                preserve_leading_zeros: preserve,
                ..typed()
            },
        );
        assert_eq!(b[0], b[7]);
        assert_eq!(b[1], b[8]);
        assert_eq!(b[3], b[9]);
        assert_eq!(b[6], b[10]);
        assert_eq!(b[11], V::String("".into()));
        assert_eq!(b[12], V::Blank);
        assert!(
            matches!(&b[0], V::String(_) if preserve)
                || matches!(&b[0], V::Decimal(_) if !preserve)
        );
    }
    let c = values(
        source,
        Options {
            recognize_quoted: true,
            preserve_leading_zeros: false,
            ..Options::default()
        },
    );
    assert!(c[..12].iter().all(|v| matches!(v, V::String(_))));
    assert_eq!(c[12], V::Blank);
}
#[test]
fn strict_dialect_unicode_escaping_and_terminal_records() {
    let a = values(
        "\u{feff}\"hé😀\r\nλ\",\"a\"\"b\",\r\n1,2,3",
        Options::default(),
    );
    assert_eq!(
        a,
        vec![
            V::String("hé😀\r\nλ".into()),
            V::String("a\"b".into()),
            V::Blank,
            V::String("1".into()),
            V::String("2".into()),
            V::String("3".into())
        ]
    );
    assert_eq!(values("\n", typed()), vec![V::Blank]);
    assert_eq!(values(",", typed()), vec![V::Blank, V::Blank]);
    for input in [
        "", "\u{feff}", "a\rb", "\"x", "a\"b", "\"x\" y", "a,b\nc", "\"x\"\r",
    ] {
        assert!(
            csv::parse(input.as_bytes(), Policy::default(), Limits::default()).is_err(),
            "{input:?}"
        );
    }
    assert!(
        csv::parse(&[0xff], Policy::default(), Limits::default())
            .unwrap_err()
            .contains("UTF-8")
    );
    assert!(csv::parse(&[0xff, 0xfe, b'a', 0], Policy::default(), Limits::default()).is_err());
}
#[test]
fn no_formulas_no_locale_and_exact_decimals() {
    let vals = values(
        "=1+1, 123,1e3,.5,1.,FALSE,79228162514264337593543950335,0.0000000000000000000000000001",
        typed(),
    );
    assert!(vals[..5].iter().all(|v| matches!(v, V::String(_))));
    assert_eq!(vals[5], V::Boolean(false));
    assert_eq!(vals[6], V::Decimal(pivot_types::PivotDecimal::MAX));
    let runtime = IntegrationRuntime::standalone().unwrap();
    for s in [
        "79228162514264337593543950336",
        "0.00000000000000000000000000001",
    ] {
        let table = csv::parse(s.as_bytes(), Policy::default(), Limits::default()).unwrap();
        assert!(
            table
                .workbook(typed(), Limits::default(), &runtime)
                .err()
                .unwrap()
                .contains("row 1, column 1: numeric conversion")
        );
        assert!(
            table
                .workbook(Options::default(), Limits::default(), &runtime)
                .is_ok()
        );
    }
}
#[test]
fn distinct_limits_and_blank_budgets() {
    let runtime = IntegrationRuntime::standalone().unwrap();
    let policy = Policy {
        bytes: 3,
        fields: 2,
    };
    assert!(
        csv::parse(b"1234", policy, Limits::default())
            .unwrap_err()
            .contains("file-size")
    );
    assert!(
        csv::parse(b",,", policy, Limits::default())
            .unwrap_err()
            .contains("logical-field")
    );
    let limits = Limits {
        rows: 2,
        columns: 2,
        live_cells: 1,
        ..Limits::default()
    };
    assert!(
        csv::parse(b"a,b,c", Policy::default(), limits)
            .unwrap_err()
            .contains("coordinate")
    );
    assert!(
        csv::parse(b"a\nb\nc", Policy::default(), limits)
            .unwrap_err()
            .contains("coordinate")
    );
    let table = csv::parse(b"a,b", Policy::default(), limits).unwrap();
    assert!(
        table
            .workbook(typed(), limits, &runtime)
            .err()
            .unwrap()
            .contains("live-cell limit: 1")
    );
    let table = csv::parse(b"a,\n,", Policy::default(), limits).unwrap();
    assert_eq!(
        table
            .workbook(typed(), limits, &runtime)
            .unwrap()
            .stats()
            .live_cells,
        1
    );
    assert!(
        csv::parse(&vec![b'x'; 4097], Policy::default(), Limits::default())
            .unwrap_err()
            .contains("String/source")
    );
}
#[test]
fn calculated_export_is_typed_exact_and_uses_old_capture() {
    let mut book = Workbook::new("export", Limits::default()).unwrap();
    book.apply(
        book.expected(),
        &[
            Edit::SetText {
                position: p(1, 1),
                text: "001234".into(),
            },
            Edit::SetText {
                position: p(1, 2),
                text: "".into(),
            },
            Edit::SetBoolean {
                position: p(1, 4),
                value: true,
            },
            Edit::SetEntry {
                position: p(2, 1),
                source: "12.345".into(),
            },
            Edit::SetFormulaText {
                position: p(2, 2),
                source: "=ROUND(A2,2)".into(),
            },
            Edit::SetText {
                position: p(2, 4),
                text: "a,\"b\"\r\nλ".into(),
            },
        ],
    )
    .unwrap();
    let captured = book
        .capture_sheet(book.worksheet(), book.expected())
        .unwrap();
    book.apply(
        book.expected(),
        &[Edit::SetEntry {
            position: p(2, 1),
            source: "99".into(),
        }],
    )
    .unwrap();
    let rect = Rectangle {
        first: p(1, 1),
        last: p(2, 4),
    };
    let bytes = csv::export(&captured, rect, Policy::default()).unwrap();
    assert_eq!(
        String::from_utf8(bytes.clone()).unwrap(),
        "\"001234\",\"\",,TRUE\r\n12.345,12.35,,\"a,\"\"b\"\"\r\nλ\"\r\n"
    );
    let table = csv::parse(&bytes, Policy::default(), Limits::default()).unwrap();
    assert!(!table.all_quoted);
    let runtime = IntegrationRuntime::standalone().unwrap();
    let imported = table
        .workbook(typed(), Limits::default(), &runtime)
        .unwrap();
    assert_eq!(
        imported.cell_at(p(1, 1)).unwrap().unwrap().content,
        Content::LiteralText("001234".into())
    );
    assert_eq!(
        imported.cell_at(p(1, 2)).unwrap().unwrap().content,
        Content::LiteralText("".into())
    );
    assert!(imported.cell_at(p(1, 3)).unwrap().is_none());
    assert!(
        csv::export(
            &captured,
            rect,
            Policy {
                bytes: 5,
                ..Policy::default()
            }
        )
        .unwrap_err()
        .contains("output-size")
    );
}
#[test]
fn batched_export_reports_first_row_major_error_and_content_bounds_ignore_anchors() {
    let mut book = Workbook::new("bounds", Limits::default()).unwrap();
    book.apply(
        book.expected(),
        &[
            Edit::SetFormulaText {
                position: p(2, 2),
                source: "=Z90+1".into(),
            },
            Edit::SetFormulaText {
                position: p(3, 1),
                source: "=1/0".into(),
            },
            Edit::SetFormulaText {
                position: p(3, 2),
                source: "=2/0".into(),
            },
        ],
    )
    .unwrap();
    assert_eq!(
        book.csv_content_bounds(book.worksheet()).unwrap(),
        Some(Rectangle {
            first: p(2, 1),
            last: p(3, 2)
        })
    );
    let cap = book
        .capture_sheet(book.worksheet(), book.expected())
        .unwrap();
    assert!(
        csv::export(
            &cap,
            Rectangle {
                first: p(1, 1),
                last: p(4, 40)
            },
            Policy::default()
        )
        .unwrap_err()
        .contains("A3")
    );
    assert!(
        csv::export(
            &cap,
            Rectangle {
                first: p(0, 1),
                last: p(1, 1)
            },
            Policy::default()
        )
        .is_err()
    );
}
