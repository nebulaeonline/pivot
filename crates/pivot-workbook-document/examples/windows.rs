//! Scope-specific production checkpoints. Fixture assembly is not a move API.
use pivot_core::model::{integration::IntegrationRuntime, selection::SpaceId};
use pivot_workbook::{csv, structure::*, *};
use pivot_workbook_document::Document;
use std::{error::Error, time::Instant};
use uuid::Uuid;
fn ms(start: Instant) -> f64 {
    start.elapsed().as_secs_f64() * 1000.0
}
fn p(row: u32) -> Position {
    Position::new(row, 1)
}
fn main() -> std::result::Result<(), Box<dyn Error>> {
    let count: u32 = std::env::args()
        .nth(1)
        .ok_or("checkpoint count required")?
        .parse()?;
    if ![1024, 10000, 65536].contains(&count) {
        return Err("unsupported checkpoint".into());
    }
    let rt = IntegrationRuntime::standalone()?;
    let memory = rt.memory();
    let limits = Limits::capacity();
    let source: String = (1..count).map(|n| format!("{n}\n")).collect();
    let mut fixture = csv::parse(source.as_bytes(), csv::Policy::default(), limits)?.workbook(
        csv::Options {
            typed: true,
            ..Default::default()
        },
        limits,
        &rt,
    )?;
    let window = fixture.create_grid_window(fixture.expected(), "Orders", MAX_ROWS, MAX_COLUMNS)?;
    let scope: GridScopeId = window.into();
    let (mut image, projection_lease) =
        fixture.project_document(fixture.expected(), &Default::default())?;
    for (_, _, record) in &mut image.cells {
        record.worksheet = scope;
    }
    for point in &mut image.graph.points {
        point.0 = SpaceId(*window.uuid().as_bytes());
    }
    for ordinal in 1..64 {
        image.sheets.push(pivot_workbook::transfer::SheetRecord {
            retired: false,
            id: WindowId::from_uuid(Uuid::new_v4()).into(),
            name: format!("Grid{ordinal}"),
            rows: Uuid::new_v4(),
            columns: Uuid::new_v4(),
            row_count: MAX_ROWS,
            column_count: MAX_COLUMNS,
            row_axis: AxisImage {
                next: MAX_ROWS + 1,
                runs: vec![AxisRun {
                    first: 1,
                    count: MAX_ROWS,
                }],
            },
            column_axis: AxisImage {
                next: MAX_COLUMNS + 1,
                runs: vec![AxisRun {
                    first: 1,
                    count: MAX_COLUMNS,
                }],
            },
        });
    }
    drop((fixture, source));
    let mut book = Workbook::restore_portable(image, limits, &rt)?;
    drop(projection_lease);
    book.apply(
        book.expected(),
        &[Edit::SetEntry {
            position: p(1),
            source: format!("=SUM(Orders|A1:A{})", count - 1),
        }],
    )?;
    assert_eq!(book.stats().live_cells, count as usize);
    assert_eq!(book.windows().count(), 64);
    let old = book.capture_scope(scope, book.expected())?;
    let middle = count / 2;
    let old_cell = old
        .calculation
        .cell_at(p(middle))
        .ok_or("missing fixture cell")?;
    let old_axis = old.axes_at(p(middle + 1))?;
    let start = Instant::now();
    book.restructure(
        scope,
        book.expected(),
        Change {
            axis: AxisKind::Rows,
            action: Action::Delete,
            start: middle,
            count: 1,
        },
        &Default::default(),
    )?;
    let structural_ms = ms(start);
    assert_eq!(book.axes_at_sheet(scope, p(middle))?, old_axis);
    let sum_capture = book.capture_local(book.expected())?;
    let sum_cell = sum_capture.cell_at(p(1)).ok_or("missing summary")?;
    let expected = u64::from(count - 1) * u64::from(count) / 2 - u64::from(middle);
    assert_eq!(
        sum_capture.evaluate(&[sum_cell])?.values[0].1,
        Ok(i64::try_from(expected)?.into())
    );
    let start = Instant::now();
    book.delete_window(book.expected(), window, &Default::default())?;
    let deletion_ms = ms(start);
    let replacement = book.create_grid_window(book.expected(), "Orders", MAX_ROWS, MAX_COLUMNS)?;
    assert_ne!(replacement, window);
    assert_eq!(
        old.calculation.evaluate(&[old_cell])?.values[0].1,
        Ok(i64::from(middle).into())
    );
    let steady = memory.stats().charged;
    let start = Instant::now();
    let saved = Document::export_document(&book, book.expected())?;
    let save_ms = ms(start);
    let start = Instant::now();
    let (reopened, _) = Document::open_document(&saved.bytes, limits, &rt)?;
    let reopen_ms = ms(start);
    assert_eq!(reopened.windows().count(), 64);
    let reopened_capture = reopened.capture_local(reopened.expected())?;
    let out = reopened_capture
        .cell_at(p(1))
        .ok_or("missing reopened summary")?;
    assert!(reopened_capture.evaluate(&[out])?.values[0].1.is_err());
    let peak = memory.stats().peak;
    println!(
        "{{\"count\":{count},\"live_windows\":64,\"structural_ms\":{structural_ms},\"deletion_ms\":{deletion_ms},\"save_ms\":{save_ms},\"reopen_ms\":{reopen_ms},\"steady_charge\":{steady},\"peak_charge\":{peak}}}"
    );
    drop((old, sum_capture, reopened_capture, reopened, saved, book));
    rt.finish().map_err(|_| "retained runtime")?;
    assert_eq!(memory.stats().charged, 0);
    println!("{{\"remaining_charge\":0}}");
    if structural_ms > 1000.0
        || deletion_ms > 1000.0
        || save_ms > 10000.0
        || reopen_ms > 10000.0
        || steady > 256 * 1024 * 1024
        || peak > 512 * 1024 * 1024
    {
        return Err("Window checkpoint exceeded existing objectives; stop and diagnose".into());
    }
    Ok(())
}
