//! Production bound-source checkpoints; node counts include consumers and leaves.
use pivot_core::model::integration::IntegrationRuntime;
use pivot_workbook::{
    csv,
    geometry::{CellRect, Command, DEFAULT_WIDTH, SizeRun},
    producers::*,
    *,
};
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
    let scope = window.into();
    let geometry_before = memory.stats().charged;
    for index in 0..62 {
        b.create_grid_window(
            b.expected(),
            &format!("Empty {index}"),
            MAX_ROWS,
            MAX_COLUMNS,
        )?;
    }
    assert_eq!(b.windows().filter(|w| !w.retired).count(), 64);
    assert_eq!(b.stats().live_nodes, count as usize);
    b.edit_geometry(
        scope,
        b.expected(),
        &[
            Command::AddBreak {
                horizontal: true,
                before: 5,
            },
            Command::AddBreak {
                horizontal: false,
                before: 2,
            },
        ],
        &Default::default(),
    )?;
    let geometry_old = b.capture_scope(scope, b.expected())?;
    let old_map = geometry_old.geometry_map(None, &Default::default())?;
    let map_start = Instant::now();
    b.edit_geometry(
        scope,
        b.expected(),
        &[Command::SetProfile {
            horizontal: true,
            at: 1,
            sizes: vec![
                SizeRun {
                    first: 1,
                    last: 1,
                    size: 160 * 64,
                },
                SizeRun {
                    first: 2,
                    last: MAX_COLUMNS,
                    size: DEFAULT_WIDTH,
                },
            ],
        }],
        &Default::default(),
    )?;
    let geometry_edit_ms = ms(map_start);
    let geometry_new = b.capture_scope(scope, b.expected())?;
    let new_map = geometry_new.geometry_map(None, &Default::default())?;
    let page = new_map
        .viewport_page(
            CellRect {
                left: 0,
                top: 0,
                right: 640 * 64,
                bottom: 390 * 64,
            },
            0,
            &memory,
            &Default::default(),
        )
        .map_err(|e| format!("{e:?}"))?;
    assert_eq!(page.cells.len(), 20);
    assert_eq!(
        old_map.cell_rect(p(1)).ok_or("old geometry cell")?.right,
        DEFAULT_WIDTH
    );
    assert_eq!(
        new_map.cell_rect(p(1)).ok_or("new geometry cell")?.right,
        160 * 64
    );
    let _buffer = memory
        .reserve(8 * 1024 * 1024)
        .map_err(|_| "viewport buffer admission")?;
    let geometry_retained_charge = memory.stats().charged;
    drop((_buffer, page, new_map, old_map, geometry_new, geometry_old));
    let geometry_steady_charge = memory.stats().charged;
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
        "{{\"geometry_before_charge\":{geometry_before},\"geometry_retained_charge\":{geometry_retained_charge},\"geometry_steady_charge\":{geometry_steady_charge},\"geometry_edit_ms\":{geometry_edit_ms},\"empty_windows\":62,\"count\":{count},\"edit_ms\":{edit_ms},\"calculate_ms\":{calculate_ms},\"viewport_ms\":{viewport_ms},\"refresh_ms\":{refresh_ms},\"save_ms\":{save_ms},\"reopen_ms\":{reopen_ms},\"steady_charge\":{steady},\"peak_charge\":{peak}}}"
    );
    drop((old, edited, opened, saved, b));
    rt.finish().map_err(|_| "retained runtime")?;
    assert_eq!(memory.stats().charged, 0);
    Ok(())
}
