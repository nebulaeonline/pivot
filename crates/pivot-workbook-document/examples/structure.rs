//! Production structural checkpoints, independent row-major/style/identity oracles.
use pivot_core::model::integration::IntegrationRuntime;
use pivot_storage::experimental_file::native;
use pivot_workbook::{csv, formatting::Region, structure::*, *};
use pivot_workbook_document::Document;
use std::{error::Error, path::Path, time::Instant};
fn ms(start: Instant) -> f64 {
    start.elapsed().as_secs_f64() * 1000.0
}
fn main() -> std::result::Result<(), Box<dyn Error>> {
    let args: Vec<_> = std::env::args().collect();
    let count: usize = args.get(1).ok_or("count")?.parse()?;
    let path = Path::new(args.get(2).ok_or("destination")?);
    let styles = args.get(3).is_some_and(|v| v == "styles");
    if ![1024, 10000, 65536].contains(&count) {
        return Err("invalid checkpoint".into());
    }
    let rt = IntegrationRuntime::standalone()?;
    let memory = rt.memory();
    let mut source = String::new();
    let mut b = if styles {
        let scaffold = Workbook::new_in("structural styles", Limits::default(), &rt)?;
        let mut image = scaffold.export_portable(scaffold.expected())?;
        let sheet = scaffold.worksheet();
        for i in 0..count {
            let p = Position::new((i / 512 * 2 + 1) as u32, (i % 512 * 2 + 1) as u32);
            image.styles.push(Region {
                sheet,
                property: 5,
                rectangle: Rectangle { first: p, last: p },
                value: 1,
            });
        }
        drop(scaffold);
        Workbook::restore_portable(image, Limits::default(), &rt)?
    } else {
        for i in 0..count {
            source.push_str(&(i + 1).to_string());
            source.push(if (i + 1) % 16 == 0 { '\n' } else { ',' });
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
    eprintln!("memory built: {:?}", memory.stats());
    let sheet = b.worksheet();
    let identities = b.stats().allocated_nodes;
    let first = if styles {
        None
    } else {
        Some(b.cell_at(Position::new(2, 2))?.unwrap().reference)
    };
    let mut roots = Vec::new();
    let mut times = Vec::new();
    let mut works = Vec::new();
    for (axis, action) in [
        (AxisKind::Rows, Action::Insert),
        (AxisKind::Rows, Action::Delete),
        (AxisKind::Columns, Action::Insert),
        (AxisKind::Columns, Action::Delete),
    ] {
        roots.push(b.capture_local(b.expected())?);
        let start = Instant::now();
        let outcome = b.restructure(
            sheet,
            b.expected(),
            Change {
                axis,
                action,
                start: 2,
                count: 1,
            },
            &Default::default(),
        )?;
        times.push(ms(start));
        works.push(outcome.work);
        assert!(outcome.changed);
        assert_eq!(b.stats().allocated_nodes, identities);
        eprintln!("memory after {axis:?} {action:?}: {:?}", memory.stats());
    }
    if let Some(first) = first {
        assert_eq!(b.cell_at(Position::new(2, 2))?.unwrap().reference, first);
        let c = b.capture_local(b.expected())?;
        for begin in (0..count).step_by(128) {
            let refs = (begin..(begin + 128).min(count))
                .map(|i| {
                    c.cell_at(Position::new((i / 16 + 1) as u32, (i % 16 + 1) as u32))
                        .unwrap()
                })
                .collect::<Vec<_>>();
            for (offset, (_, value)) in c.evaluate_values(&refs)?.values.into_iter().enumerate() {
                assert_eq!(
                    value,
                    Ok(pivot_types::PivotValue::Decimal(
                        ((begin + offset + 1) as i64).into()
                    ))
                );
            }
        }
        assert_eq!(roots[0].evaluate(&[first])?.values[0].1, Ok(18.into()));
    } else {
        assert_eq!(b.stats().allocated_nodes, 0);
        assert_eq!(b.style_regions(), count);
        for (index, r) in b.regions().enumerate() {
            assert_eq!(r.property, 5);
            assert_eq!(r.value, 1);
            assert_eq!(
                r.rectangle.first,
                Position::new((index / 512 * 2 + 1) as u32, (index % 512 * 2 + 1) as u32)
            );
            // Deleting the inserted column copies the last surviving profile
            // into the virtual tail. The final bold cell extends through 1024;
            // an insert/delete pair is not a presentation Undo operation.
            assert_eq!(
                r.rectangle.last,
                Position::new(
                    r.rectangle.first.row,
                    if index % 512 == 511 {
                        1024
                    } else {
                        r.rectangle.first.column
                    }
                )
            );
        }
        assert!(roots[0].style_at(sheet, Position::new(1, 1))?.bold);
        assert!(!roots[0].style_at(sheet, Position::new(1, 1024))?.bold);
        assert!(b.style_at(sheet, Position::new(1, 1024))?.bold);
    }
    eprintln!("memory evaluated: {:?}", memory.stats());
    let steady = memory.stats().charged;
    let start = Instant::now();
    let export = Document::export_document(&b, b.expected())?;
    native::install(&native::destination(path)?, &export.bytes)?;
    let save_ms = ms(start);
    eprintln!("memory saved: {:?}", memory.stats());
    let start = Instant::now();
    let reopened = Document::open_document(&native::read(path)?.0, Limits::default(), &rt)?.0;
    let open_ms = ms(start);
    eprintln!("memory reopened: {:?}", memory.stats());
    assert_eq!(reopened.stats().allocated_nodes, identities);
    assert_eq!(reopened.style_regions(), b.style_regions());
    let retained_after_reopen = memory.stats().charged;
    let peak = memory.stats().peak;
    println!(
        "{{\"count\":{count},\"styles\":{styles},\"command_ms\":{times:?},\"work\":{works:?},\"save_ms\":{save_ms},\"reopen_ms\":{open_ms},\"steady_with_four_roots\":{steady},\"retained_after_reopen\":{retained_after_reopen},\"peak\":{peak}}}"
    );
    drop((roots, reopened, export, b));
    rt.finish().map_err(|_| "runtime retained")?;
    assert_eq!(memory.stats().charged, 0);
    println!("{{\"remaining_charge\":0}}");
    if times
        .iter()
        .any(|t| *t > if styles { 2000.0 } else { 1000.0 })
        || save_ms > 10000.0
        || open_ms > 10000.0
        || (!styles && (peak > 512 * 1024 * 1024 || steady > 256 * 1024 * 1024))
    {
        return Err("structural checkpoint exceeded objective; stop and diagnose".into());
    }
    Ok(())
}
