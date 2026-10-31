//! Staged full filtering costs, including capture, exact key evaluation and map production.
use pivot_core::model::integration::IntegrationRuntime;
use pivot_storage::experimental_file::native;
use pivot_workbook::{csv, filtering::*, *};
use pivot_workbook_document::Document;
use std::{error::Error, path::Path, time::Instant};
fn ms(t: Instant) -> f64 {
    t.elapsed().as_secs_f64() * 1000.0
}
fn main() -> std::result::Result<(), Box<dyn Error>> {
    let args = std::env::args().collect::<Vec<_>>();
    let count: usize = args.get(1).ok_or("count")?.parse()?;
    let path = Path::new(args.get(2).ok_or("destination")?);
    let mode = args.get(3).map(String::as_str).unwrap_or("numeric");
    if ![1024, 10000, 65536].contains(&count)
        || !["numeric", "text", "sparse", "ten", "blank"].contains(&mode)
    {
        return Err("invalid filtering checkpoint".into());
    }
    let width = match mode {
        "sparse" | "text" => 1,
        "ten" => 10,
        "blank" => 1024,
        _ => 16,
    };
    let height = if mode == "blank" {
        65536
    } else {
        count / width
    };
    if mode != "blank" && !count.is_multiple_of(width) {
        return Err("count/width mismatch".into());
    }
    let rt = IntegrationRuntime::standalone()?;
    let memory = rt.memory();
    let mut source = String::new();
    let mut b = if mode == "blank" {
        Workbook::new_in("blank filter", Limits::default(), &rt)?
    } else {
        for row in 1..=height {
            for column in 0..width {
                if mode == "text" && column == 0 {
                    source.push_str(&format!("\"{row:08}\""));
                } else {
                    source.push_str(&(if column == 0 { row } else { 1 }).to_string());
                }
                source.push(if column + 1 == width { '\n' } else { ',' });
            }
        }
        csv::parse(source.as_bytes(), csv::Policy::default(), Limits::default())?.workbook(
            csv::Options {
                typed: true,
                ..Default::default()
            },
            Limits::default(),
            &rt,
        )?
    };
    let sheet = b.worksheet();
    let identities = b.stats().allocated_nodes;
    let graph = b.expected().graph;
    let original = b.cell_at(Position::new(1, 1))?.map(|cell| cell.reference);
    let rectangle = Rectangle {
        first: Position::new(1, 1),
        last: Position::new(height as u32, width as u32),
    };
    let mut roots = Vec::new();
    let mut applies = Vec::new();
    let mut refreshes = Vec::new();
    let mut captures = Vec::new();
    let mut work = Vec::new();
    for turn in 0..4 {
        let t = Instant::now();
        roots.push(b.capture_sheet(sheet, b.expected())?);
        captures.push(ms(t));
        let threshold = if turn % 2 == 0 { 1 } else { height / 2 };
        let predicate = match mode {
            "blank" => Predicate {
                operator: Operator::Blank,
                literal: None,
            },
            "text" => Predicate {
                operator: if turn % 2 == 0 {
                    Operator::StartsWith
                } else {
                    Operator::EndsWith
                },
                literal: Some(Literal::Text(if turn % 2 == 0 { "00" } else { "0" }.into())),
            },
            _ => Predicate {
                operator: Operator::GreaterEqual,
                literal: Some(Literal::Decimal((threshold as i64).into())),
            },
        };
        let definition = Definition {
            rectangle,
            exclude_header: false,
            needs_review: false,
            conditions: vec![Condition {
                column: 1,
                either: false,
                predicates: vec![predicate],
            }],
        };
        let t = Instant::now();
        let (_, map) = b.set_filter_with_visibility(
            sheet,
            b.expected(),
            Some(&definition),
            &Default::default(),
        )?;
        applies.push(ms(t));
        let map = map.ok_or("missing applied map")?;
        let expected = if mode == "blank" || mode == "text" && turn % 2 == 0 {
            height
        } else if mode == "text" {
            height / 10
        } else {
            height - threshold + 1
        };
        assert_eq!(map.shown_data_rows as usize, expected);
        work.push(map.work);
        for row in 1..=height as u32 {
            let shown = if mode == "blank" || mode == "text" && turn % 2 == 0 {
                true
            } else if mode == "text" {
                row % 10 == 0
            } else {
                row >= threshold as u32
            };
            assert_eq!(map.ordinal(row).is_some(), shown);
        }
        drop(map);
        let t = Instant::now();
        let refreshed = b.filter_visibility(sheet, b.expected(), &Default::default())?;
        refreshes.push(ms(t));
        assert_eq!(refreshed.shown_data_rows as usize, expected);
        drop(refreshed);
        assert_eq!(b.expected().graph, graph);
        assert_eq!(b.stats().allocated_nodes, identities);
        assert_eq!(
            b.cell_at(Position::new(1, 1))?.map(|c| c.reference),
            original
        );
    }
    rt.try_flush()?;
    let steady = memory.stats().charged;
    let t = Instant::now();
    let export = Document::export_document(&b, b.expected())?;
    native::install(&native::destination(path)?, &export.bytes)?;
    let save = ms(t);
    let t = Instant::now();
    let reopened = Document::open_document(&native::read(path)?.0, Limits::default(), &rt)?.0;
    let reopen = ms(t);
    assert_eq!(reopened.filter(sheet)?, b.filter(sheet)?);
    assert_eq!(reopened.stats().allocated_nodes, identities);
    let peak = memory.stats().peak;
    println!(
        "{{\"count\":{count},\"positioned_cells\":{identities},\"rows\":{height},\"columns\":{width},\"mode\":\"{mode}\",\"apply_ms\":{applies:?},\"refresh_ms\":{refreshes:?},\"capture_ms\":{captures:?},\"work\":{work:?},\"steady_with_four_roots\":{steady},\"peak\":{peak},\"save_ms\":{save},\"reopen_ms\":{reopen}}}"
    );
    drop((roots, export, reopened, b));
    rt.finish().map_err(|_| "runtime retained")?;
    assert_eq!(memory.stats().charged, 0);
    println!("{{\"remaining_charge\":0}}");
    if applies.iter().chain(&refreshes).any(|v| *v > 1000.0)
        || save > 10000.0
        || reopen > 10000.0
        || steady > 256 * 1024 * 1024
        || peak > 512 * 1024 * 1024
    {
        return Err("filtering checkpoint exceeded unchanged objective; stop and diagnose".into());
    }
    Ok(())
}
