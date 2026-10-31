//! Production sorting checkpoints with retained roots and independent value/style checks.
use pivot_core::model::integration::IntegrationRuntime;
use pivot_storage::experimental_file::native;
use pivot_workbook::{csv, formatting::Region, sorting::*, *};
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
    if ![1024, 10000, 65536].contains(&count) {
        return Err("invalid checkpoint".into());
    }
    let styles = mode == "styles";
    let width = if styles {
        1024
    } else if mode == "sparse" {
        1
    } else if mode == "ten" {
        10
    } else {
        16
    };
    if !styles && !count.is_multiple_of(width) {
        return Err("count/width mismatch".into());
    }
    let height = if styles {
        count.div_ceil(512) * 2
    } else {
        count / width
    };
    let rt = IntegrationRuntime::standalone()?;
    let memory = rt.memory();
    let mut source = String::new();
    let mut b = if styles {
        let scaffold = Workbook::new_in("sort style stress", Limits::default(), &rt)?;
        let mut image = scaffold.export_portable(scaffold.expected())?;
        let sheet = scaffold.worksheet();
        for i in 0..count {
            let p = Position::new((i / 512 * 2 + 2) as u32, (i % 512 * 2 + 1) as u32);
            image.styles.push(Region {
                sheet,
                property: 5,
                rectangle: Rectangle { first: p, last: p },
                value: 1,
            });
        }
        drop(scaffold);
        let mut b = Workbook::restore_portable(image, Limits::default(), &rt)?;
        let edits = (1..=height)
            .map(|r| Edit::SetDecimal {
                position: Position::new(r as u32, width as u32),
                value: DecimalInput::from_decimal(((height - r + 1) as i64).into()),
            })
            .collect::<Vec<_>>();
        b.apply(b.expected(), &edits)?;
        b
    } else {
        for row in 0..height {
            for col in 0..width {
                source.push_str(&((height - row - 1) * width + col + 1).to_string());
                source.push(if col + 1 == width { '\n' } else { ',' });
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
    eprintln!("built: {:?}; workbook: {:?}", memory.stats(), b.stats());
    let identities = b.stats().allocated_nodes;
    let original = b
        .cell_at(Position::new(1, if styles { width as u32 } else { 1 }))?
        .unwrap()
        .reference;
    let initial_styles = b.regions().collect::<Vec<_>>();
    let mut roots = Vec::new();
    let mut times = Vec::new();
    let mut work = Vec::new();
    let mut capture_times = Vec::new();
    for descending in [false, true, false, true] {
        let t = Instant::now();
        roots.push(b.capture_local(b.expected())?);
        capture_times.push(ms(t));
        let t = Instant::now();
        let outcome = b.sort_rows(
            sheet,
            b.expected(),
            &Options {
                rectangle: Rectangle {
                    first: Position::new(1, 1),
                    last: Position::new(height as u32, width as u32),
                },
                exclude_header: false,
                keys: vec![Key {
                    column: if styles { width as u32 } else { 1 },
                    descending,
                }],
                move_formatting: true,
            },
            &Default::default(),
        )?;
        times.push(ms(t));
        work.push(outcome.work);
        assert!(outcome.changed);
        assert_eq!(b.stats().allocated_nodes, identities);
        eprintln!("after sort: {:?}", memory.stats());
    }
    assert_eq!(
        b.cell_at(Position::new(1, if styles { width as u32 } else { 1 }))?
            .unwrap()
            .reference,
        original
    );
    assert_eq!(b.regions().collect::<Vec<_>>(), initial_styles);
    if !styles {
        let c = b.capture_local(b.expected())?;
        for begin in (0..count).step_by(128) {
            let refs = (begin..(begin + 128).min(count))
                .map(|i| {
                    c.cell_at(Position::new(
                        (i / width + 1) as u32,
                        (i % width + 1) as u32,
                    ))
                    .unwrap()
                })
                .collect::<Vec<_>>();
            for (offset, (_, value)) in c.evaluate_values(&refs)?.values.into_iter().enumerate() {
                let i = begin + offset;
                let expected = (height - i / width - 1) * width + i % width + 1;
                assert_eq!(
                    value,
                    Ok(pivot_types::PivotValue::Decimal((expected as i64).into()))
                );
            }
        }
    }
    let old_value = if styles {
        height
    } else {
        (height - 1) * width + 1
    };
    assert_eq!(
        roots[0].evaluate(&[original])?.values[0].1,
        Ok((old_value as i64).into())
    );
    eprintln!("before steady drain: {:?}", memory.stats());
    rt.try_flush()?;
    let steady = memory.stats().charged;
    eprintln!("after steady drain: {:?}", memory.stats());
    let t = Instant::now();
    let export = Document::export_document(&b, b.expected())?;
    native::install(&native::destination(path)?, &export.bytes)?;
    let save_ms = ms(t);
    eprintln!("after save: {:?}", memory.stats());
    let t = Instant::now();
    let reopened = Document::open_document(&native::read(path)?.0, Limits::default(), &rt)?.0;
    let reopen_ms = ms(t);
    eprintln!("after reopen: {:?}", memory.stats());
    assert_eq!(reopened.stats().allocated_nodes, identities);
    assert_eq!(reopened.regions().collect::<Vec<_>>(), initial_styles);
    let peak = memory.stats().peak;
    println!(
        "{{\"count\":{count},\"rows\":{height},\"columns\":{width},\"styles\":{styles},\"command_ms\":{times:?},\"capture_ms\":{capture_times:?},\"work\":{work:?},\"save_ms\":{save_ms},\"reopen_ms\":{reopen_ms},\"steady_with_four_roots\":{steady},\"peak\":{peak}}}"
    );
    drop((roots, reopened, export, b));
    rt.finish().map_err(|_| "runtime retained")?;
    assert_eq!(memory.stats().charged, 0);
    println!("{{\"remaining_charge\":0}}");
    if times
        .iter()
        .any(|t| *t > if styles { 2000.0 } else { 1000.0 })
        || save_ms > 10000.0
        || reopen_ms > 10000.0
        || (!styles && (steady > 256 * 1024 * 1024 || peak > 512 * 1024 * 1024))
    {
        return Err("sorting checkpoint exceeded objective; stop and diagnose".into());
    }
    Ok(())
}
