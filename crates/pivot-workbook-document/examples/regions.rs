//! Native current-format spatial-formatting correctness and resource checkpoints.
use pivot_core::model::integration::IntegrationRuntime;
use pivot_storage::experimental_file::native;
use pivot_workbook::{Limits, Position as P, Rectangle, Workbook, formatting::*};
use pivot_workbook_document::Document;
use std::{error::Error, path::Path, time::Instant};
fn main() -> Result<(), Box<dyn Error>> {
    let args: Vec<_> = std::env::args().collect();
    let count: usize = args.get(1).ok_or("count")?.parse()?;
    if ![1024, 10000, 65536].contains(&count) {
        return Err("checkpoint count".into());
    }
    let path = Path::new(args.get(2).ok_or("destination")?);
    let runtime = IntegrationRuntime::standalone()?;
    let memory = runtime.memory();
    let mut b = Workbook::new_in("format regions", Limits::default(), &runtime)?;
    let sheet = b.worksheet();
    let mut image = b.export_portable(b.expected())?;
    for i in 0..count {
        let p = P::new((i / 512 * 2 + 1) as u32, (i % 512 * 2 + 1) as u32);
        image.styles.push(Region {
            sheet,
            property: 5,
            rectangle: Rectangle { first: p, last: p },
            value: 1,
        });
    }
    drop(b);
    b = Workbook::restore_portable(image, Limits::default(), &runtime)?;
    assert_eq!(b.style_regions(), count);
    if count == 65536 {
        let before = b.expected();
        assert!(
            b.format_sheet(
                sheet,
                before,
                Rectangle {
                    first: P::new(2, 2),
                    last: P::new(2, 2)
                },
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
    }
    let mut retained = Vec::new();
    let mut worst = 0f64;
    for i in 0..4 {
        retained.push(b.capture_local(b.expected())?);
        let start = Instant::now();
        b.format_sheet(
            sheet,
            b.expected(),
            Rectangle {
                first: P::new(1, 1),
                last: P::new(1, 1),
            },
            StylePatch {
                bold: Field::Set(i % 2 != 0),
                ..Default::default()
            },
            false,
            &Default::default(),
        )?;
        worst = worst.max(start.elapsed().as_secs_f64() * 1000.0);
    }
    assert!(retained[0].style_at(sheet, P::new(1, 1))?.bold);
    assert!(!retained[1].style_at(sheet, P::new(1, 1))?.bold);
    let fragmented_work = b.style_command_work();
    let mut viewport = Vec::new();
    for _ in 0..31 {
        let start = Instant::now();
        let values = b.style_viewport(
            sheet,
            Rectangle {
                first: P::new(1, 1),
                last: P::new(16, 16),
            },
        )?;
        for (p, s) in values {
            assert_eq!(
                s.bold,
                p.row % 2 == 1
                    && p.column % 2 == 1
                    && ((p.row as usize - 1) / 2 * 512 + (p.column as usize - 1) / 2) < count
            );
        }
        viewport.push(start.elapsed().as_secs_f64() * 1000.0);
    }
    viewport.sort_by(f64::total_cmp);
    let viewport_p95 = viewport[29];
    let start = Instant::now();
    let export = Document::export_document(&b, b.expected())?;
    native::install(&native::destination(path)?, &export.bytes)?;
    let save_ms = start.elapsed().as_secs_f64() * 1000.0;
    let start = Instant::now();
    let reopened = Document::open_document(&native::read(path)?.0, Limits::default(), &runtime)?.0;
    let open_ms = start.elapsed().as_secs_f64() * 1000.0;
    assert_eq!(reopened.style_regions(), count);
    assert_eq!(reopened.stats().allocated_nodes, 0);
    for r in reopened.regions() {
        let p = r.rectangle.first;
        assert_eq!(r.rectangle.last, p);
        assert_eq!(r.property, 5);
        assert_eq!(r.value, 1);
        assert!(
            p.row % 2 == 1
                && p.column % 2 == 1
                && ((p.row as usize - 1) / 2 * 512 + (p.column as usize - 1) / 2) < count
        );
    }
    let native_bytes = export.bytes.len();
    let all = Rectangle {
        first: P::new(1, 1),
        last: P::new(65536, 1024),
    };
    let start = Instant::now();
    b.format_sheet(
        sheet,
        b.expected(),
        all,
        StylePatch {
            bold: Field::Set(true),
            ..Default::default()
        },
        false,
        &Default::default(),
    )?;
    let uniform_ms = start.elapsed().as_secs_f64() * 1000.0;
    let uniform_work = b.style_command_work();
    assert_eq!(b.style_regions(), 1);
    assert!(b.style_at(sheet, P::new(65536, 1024))?.bold);
    assert_eq!(b.stats().allocated_nodes, 0);
    let mut geometry_p95 = Vec::new();
    for rectangle in [
        all,
        Rectangle {
            first: P::new(5, 1),
            last: P::new(5, 1024),
        },
        Rectangle {
            first: P::new(1, 2),
            last: P::new(65536, 2),
        },
    ] {
        let mut samples = Vec::new();
        for i in 0..31 {
            let start = Instant::now();
            b.format_sheet(
                sheet,
                b.expected(),
                rectangle,
                StylePatch {
                    bold: Field::Set(i % 2 != 0),
                    ..Default::default()
                },
                false,
                &Default::default(),
            )?;
            samples.push(start.elapsed().as_secs_f64() * 1000.0);
        }
        samples.sort_by(f64::total_cmp);
        geometry_p95.push(samples[29]);
    }
    let whole_p95 = geometry_p95[0];
    let row_p95 = geometry_p95[1];
    let column_p95 = geometry_p95[2];
    println!(
        "{{\"fragments\":{count},\"fragmented_edit_max_ms\":{worst},\"viewport_p95_ms\":{viewport_p95},\"whole_sheet_ms\":{uniform_ms},\"whole_sheet_p95_ms\":{whole_p95},\"row_p95_ms\":{row_p95},\"column_p95_ms\":{column_p95},\"fragmented_work\":{fragmented_work},\"uniform_work\":{uniform_work},\"native_bytes\":{native_bytes},\"save_ms\":{save_ms},\"reopen_ms\":{open_ms},\"steady\":{},\"peak\":{}}}",
        memory.stats().charged,
        memory.stats().peak
    );
    if worst > 1000.0
        || viewport_p95 > 100.0
        || geometry_p95.iter().any(|t| *t > 100.0)
        || uniform_ms > 100.0
        || save_ms > 10000.0
        || open_ms > 10000.0
    {
        return Err("format checkpoint exceeded objective".into());
    }
    drop((retained, reopened, export, b));
    runtime.finish().map_err(|_| "runtime retained")?;
    assert_eq!(memory.stats().charged, 0);
    println!("{{\"remaining_charge\":0}}");
    Ok(())
}
