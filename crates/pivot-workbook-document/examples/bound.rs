//! Production bound-source checkpoints; node counts include consumers and leaves.
use pivot_core::model::integration::IntegrationRuntime;
use pivot_workbook::{csv, producers::*, *};
use pivot_workbook_document::Document;
use std::{error::Error, time::Instant};
fn ms(t: Instant) -> f64 {
    t.elapsed().as_secs_f64() * 1000.0
}
fn p(row: u32) -> Position {
    Position::new(row, 1)
}
fn main() -> std::result::Result<(), Box<dyn Error>> {
    let count: u32 = std::env::args().nth(1).ok_or("count required")?.parse()?;
    if ![1024, 10000, 65536].contains(&count) {
        return Err("unsupported checkpoint".into());
    }
    let rt = IntegrationRuntime::standalone()?;
    let memory = rt.memory();
    let limits = Limits::capacity();
    let source = (0..count - 3).map(|_| "1\n").collect::<String>();
    let mut b = csv::parse(source.as_bytes(), csv::Policy::default(), limits)?.workbook(
        csv::Options {
            typed: true,
            ..Default::default()
        },
        limits,
        &rt,
    )?;
    drop(source);
    let GridScopeId::Worksheet(sheet) = b.worksheet() else {
        unreachable!()
    };
    let live = b.create_worksheet_producer(
        b.expected(),
        sheet,
        Rectangle {
            first: p(1),
            last: p(count - 3),
        },
    )?;
    let window = b.create_bound_window(b.expected(), "Live", live)?;
    let external = b.create_fixture_producer(b.expected(), Fixture::Ledger, KeyPolicy::Durable)?;
    let ticket = b.begin_refresh(b.expected(), external)?;
    b.refresh_fixture(&ticket, 0)?;
    drop(ticket);
    b.create_bound_window(b.expected(), "External", external)?;
    b.apply(
        b.expected(),
        &[
            Edit::SetEntry {
                position: p(count - 2),
                source: format!("=SUM(Live|A1:A{})", count - 3),
            },
            Edit::SetEntry {
                position: p(count - 1),
                source: "=External|A1".into(),
            },
        ],
    )?;
    assert_eq!(b.stats().live_nodes, count as usize);
    let old = b.capture_local(b.expected())?;
    let outputs = [
        old.cell_at(p(count - 2)).ok_or("sum")?,
        old.cell_at(p(count - 1)).ok_or("external")?,
    ];
    let start = Instant::now();
    let values = old.evaluate(&outputs)?;
    let calculate_ms = ms(start);
    assert_eq!(values.values[0].1, Ok(i64::from(count - 3).into()));
    let start = Instant::now();
    let frame = old.project_window(
        window,
        Rectangle {
            first: p(1),
            last: p(20),
        },
    )?;
    let viewport_ms = ms(start);
    assert_eq!(frame.cells.len(), 20);
    drop(frame);
    let start = Instant::now();
    b.apply(
        b.expected(),
        &[Edit::SetEntry {
            position: p(1),
            source: "2".into(),
        }],
    )?;
    let edit_ms = ms(start);
    let edited = b.capture_local(b.expected())?;
    assert_eq!(
        edited.evaluate(&outputs)?.values[0].1,
        Ok(i64::from(count - 2).into())
    );
    assert_eq!(
        old.evaluate(&outputs)?.values[0].1,
        Ok(i64::from(count - 3).into())
    );
    let first = b.begin_refresh(b.expected(), external)?;
    let second = b.begin_refresh(b.expected(), external)?;
    let start = Instant::now();
    b.refresh_fixture(&second, 1)?;
    let refresh_ms = ms(start);
    assert!(b.refresh_fixture(&first, 2).is_err());
    drop((first, second));
    assert_eq!(old.evaluate(&outputs)?.values[1].1, Ok(1.into()));
    let steady = memory.stats().charged;
    let start = Instant::now();
    let saved = Document::export_document(&b, b.expected())?;
    let save_ms = ms(start);
    let start = Instant::now();
    let (opened, _) = Document::open_document(&saved.bytes, limits, &rt)?;
    let reopen_ms = ms(start);
    let peak = memory.stats().peak;
    println!(
        "{{\"count\":{count},\"edit_ms\":{edit_ms},\"calculate_ms\":{calculate_ms},\"viewport_ms\":{viewport_ms},\"refresh_ms\":{refresh_ms},\"save_ms\":{save_ms},\"reopen_ms\":{reopen_ms},\"steady_charge\":{steady},\"peak_charge\":{peak}}}"
    );
    drop((old, edited, opened, saved, b));
    rt.finish().map_err(|_| "retained runtime")?;
    assert_eq!(memory.stats().charged, 0);
    Ok(())
}
