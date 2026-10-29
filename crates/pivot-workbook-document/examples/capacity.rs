//! Production-path capacity checkpoint; no experiment features or quota bypasses.
use pivot_core::model::integration::IntegrationRuntime;
use pivot_storage::experimental_file::native;
use pivot_types::PivotValue;
use pivot_workbook::{Edit, Limits, Position, Rectangle, csv};
use pivot_workbook_document::Document;
use std::{error::Error, path::Path, time::Instant};

fn elapsed(start: Instant) -> f64 {
    start.elapsed().as_secs_f64() * 1000.0
}
fn main() -> Result<(), Box<dyn Error>> {
    let args: Vec<_> = std::env::args().collect();
    if args.get(1).is_some_and(|s| s == "identities") {
        return identities(Path::new(args.get(2).ok_or("native output path required")?));
    }
    if args.get(1).is_some_and(|s| s == "typed") {
        return typed_table(
            args.get(2).ok_or("typed count required")?.parse()?,
            args.get(3).ok_or("text or boolean required")?,
            Path::new(args.get(4).ok_or("native output path required")?),
        );
    }
    let count: usize = args.get(1).map_or(Ok(1024), |s| s.parse())?;
    let columns: usize = args.get(2).map_or(Ok(16), |s| s.parse())?;
    if count == 0 || count > 65_536 || columns == 0 || !count.is_multiple_of(columns) {
        return Err("count must be 1..=65536 and a multiple of columns".into());
    }
    let path = Path::new(args.get(3).ok_or("native output path required")?);
    let candidate = args.get(4).is_some_and(|s| s == "capacity");
    let limits = if candidate {
        Limits::capacity()
    } else {
        Limits::baseline()
    };
    let policy = if candidate {
        csv::Policy::capacity()
    } else {
        csv::Policy::default()
    };
    let rows = count / columns;
    let mut source = String::new();
    for i in 0..count {
        source.push_str(&(i + 1).to_string());
        source.push(if (i + 1) % columns == 0 { '\n' } else { ',' });
    }
    let runtime = IntegrationRuntime::standalone()?;
    let memory = runtime.memory();
    let report = |phase: &str| {
        let m = memory.stats();
        println!(
            "{{\"memory_phase\":\"{phase}\",\"charged\":{},\"allocated\":{},\"reserved\":{},\"peak\":{}}}",
            m.charged, m.allocated, m.reserved, m.peak
        );
    };
    let start = Instant::now();
    let table = csv::parse(source.as_bytes(), policy, limits)?;
    let parse_ms = elapsed(start);
    let start = Instant::now();
    let mut book = table.workbook(
        csv::Options {
            typed: true,
            ..Default::default()
        },
        limits,
        &runtime,
    )?;
    let build_ms = elapsed(start);
    report("after_import");
    let stats = book.stats();
    assert_eq!(stats.live_cells, count);
    assert_eq!(stats.allocated_nodes, count);
    let first = book.cell_at(Position::new(1, 1))?.unwrap().reference;
    let last = Position::new(rows as u32, columns as u32);
    let mut captures = Vec::new();
    for _ in 0..4 {
        captures.push(book.capture_sheet(book.worksheet(), book.expected())?);
    }
    let mut capture_times = Vec::new();
    let mut edit_times = Vec::new();
    for i in 0..31 {
        let start = Instant::now();
        let capture = book.capture_sheet(book.worksheet(), book.expected())?;
        capture_times.push(elapsed(start));
        drop(capture);
        let start = Instant::now();
        book.apply(
            book.expected(),
            &[Edit::SetEntry {
                position: Position::new(1, 1),
                source: (i + 2).to_string(),
            }],
        )?;
        edit_times.push(elapsed(start));
        if let Some(retained) = captures.get_mut(i) {
            *retained = book.capture_sheet(book.worksheet(), book.expected())?;
        }
        assert_eq!(book.cell_at(Position::new(1, 1))?.unwrap().reference, first);
        assert_eq!(book.stats().allocated_nodes, count);
    }
    book.apply(
        book.expected(),
        &[Edit::SetEntry {
            position: Position::new(1, 1),
            source: "1".into(),
        }],
    )?;
    let rect = Rectangle {
        first: Position::new(1, 1),
        last,
    };
    let capture = book.capture_sheet(book.worksheet(), book.expected())?;
    let start = Instant::now();
    for begin in (0..count).step_by(128) {
        let refs: Vec<_> = (begin..(begin + 128).min(count))
            .map(|i| {
                capture
                    .cell_at(Position::new(
                        (i / columns + 1) as u32,
                        (i % columns + 1) as u32,
                    ))
                    .unwrap()
            })
            .collect();
        let result = capture.evaluate_values(&refs)?;
        for (offset, (_, value)) in result.values.into_iter().enumerate() {
            assert_eq!(
                value,
                Ok(PivotValue::Decimal(((begin + offset + 1) as i64).into()))
            );
        }
    }
    let calculation_ms = elapsed(start);
    let start = Instant::now();
    let exported = csv::export(&capture, rect, policy)?;
    let csv_export_ms = elapsed(start);
    report("after_csv_export");
    println!(
        "{{\"checkpoint\":\"csv_verified\",\"cells\":{count},\"build_ms\":{build_ms},\"csv_export_ms\":{csv_export_ms}}}"
    );
    assert_eq!(String::from_utf8(exported)?.replace("\r\n", "\n"), source);
    let start = Instant::now();
    let native_export = Document::export_document(&book, book.expected())
        .map_err(|e| format!("native encode: {e}"))?;
    let encode_ms = elapsed(start);
    report("after_native_encode");
    let start = Instant::now();
    native::install(&native::destination(path)?, &native_export.bytes)
        .map_err(|e| format!("native install: {e}"))?;
    let install_ms = elapsed(start);
    let start = Instant::now();
    let (bytes, _) = native::read(path)?;
    let (reopened, _) = Document::open_document(&bytes, limits, &runtime)
        .map_err(|e| format!("native reopen: {e}"))?;
    let reopen_ms = elapsed(start);
    report("after_native_reopen");
    assert_eq!(
        reopened.cell_at(Position::new(1, 1))?.unwrap().reference,
        first
    );
    let restored = reopened.capture_sheet(reopened.worksheet(), reopened.expected())?;
    assert_eq!(
        csv::export(&restored, rect, policy)?,
        csv::export(&capture, rect, policy)?
    );
    capture_times.sort_by(f64::total_cmp);
    edit_times.sort_by(f64::total_cmp);
    let memory_stats = memory.stats();
    let traffic = memory.allocation_traffic();
    println!(
        "{{\"domain_allocation_count\":{},\"domain_allocation_bytes\":{}}}",
        traffic.0, traffic.1
    );
    println!(
        "{{\"cells\":{count},\"rows\":{rows},\"columns\":{columns},\"parse_ms\":{parse_ms},\"build_ms\":{build_ms},\"calculation_ms\":{calculation_ms},\"csv_export_ms\":{csv_export_ms},\"encode_ms\":{encode_ms},\"install_ms\":{install_ms},\"reopen_ms\":{reopen_ms},\"capture_p95_ms\":{},\"edit_p95_ms\":{},\"native_bytes\":{},\"charged\":{},\"peak\":{}}}",
        capture_times[29],
        edit_times[29],
        bytes.len(),
        memory_stats.charged,
        memory_stats.peak
    );
    drop((restored, reopened, capture, captures, native_export, book));
    runtime.finish().map_err(|_| "runtime still owned")?;
    assert_eq!(memory.stats().charged, 0);
    println!("{{\"remaining_charge\":0}}");
    if parse_ms + build_ms > 10_000.0
        || encode_ms + install_ms > 10_000.0
        || reopen_ms > 10_000.0
        || csv_export_ms > 10_000.0
        || memory_stats.peak > 512 * 1024 * 1024
    {
        return Err("capacity checkpoint exceeded approved latency or peak charged-memory objective; stop and diagnose".into());
    }
    Ok(())
}

fn typed_table(count: usize, kind: &str, path: &Path) -> Result<(), Box<dyn Error>> {
    if count == 0
        || count > 65_536
        || !count.is_multiple_of(16)
        || !matches!(kind, "text" | "boolean")
    {
        return Err("invalid typed checkpoint".into());
    }
    let runtime = IntegrationRuntime::standalone()?;
    let memory = runtime.memory();
    let text = if kind == "text" { "001234" } else { "TRUE" };
    let mut source = String::new();
    let mut expected = String::new();
    for i in 0..count {
        source.push_str(text);
        expected.push_str(if kind == "text" { "\"001234\"" } else { "TRUE" });
        source.push(if (i + 1) % 16 == 0 { '\n' } else { ',' });
        expected.push_str(if (i + 1) % 16 == 0 { "\r\n" } else { "," });
    }
    let table = csv::parse(source.as_bytes(), csv::Policy::default(), Limits::default())?;
    let book = table.workbook(
        csv::Options {
            typed: kind == "boolean",
            ..Default::default()
        },
        Limits::default(),
        &runtime,
    )?;
    let capture = book.capture_sheet(book.worksheet(), book.expected())?;
    let rect = Rectangle {
        first: Position::new(1, 1),
        last: Position::new((count / 16) as u32, 16),
    };
    assert_eq!(
        String::from_utf8(csv::export(&capture, rect, csv::Policy::default())?)?,
        expected
    );
    let start = Instant::now();
    let export = Document::export_document(&book, book.expected())?;
    native::install(&native::destination(path)?, &export.bytes)?;
    let save_ms = elapsed(start);
    let start = Instant::now();
    let reopened = Document::open_document(&native::read(path)?.0, Limits::default(), &runtime)?.0;
    let reopen_ms = elapsed(start);
    let restored = reopened.capture_sheet(reopened.worksheet(), reopened.expected())?;
    assert_eq!(
        String::from_utf8(csv::export(&restored, rect, csv::Policy::default())?)?,
        expected
    );
    println!(
        "{{\"typed\":\"{kind}\",\"cells\":{count},\"save_ms\":{save_ms},\"reopen_ms\":{reopen_ms},\"peak\":{}}}",
        memory.stats().peak
    );
    if save_ms > 10_000.0 || reopen_ms > 10_000.0 || memory.stats().peak > 512 * 1024 * 1024 {
        return Err("typed checkpoint exceeded latency or peak charged-memory objective".into());
    }
    drop((restored, reopened, export, capture, book));
    runtime.finish().map_err(|_| "runtime still owned")?;
    assert_eq!(memory.stats().charged, 0);
    println!("{{\"remaining_charge\":0}}");
    Ok(())
}

fn identities(path: &Path) -> Result<(), Box<dyn Error>> {
    let runtime = IntegrationRuntime::standalone()?;
    let memory = runtime.memory();
    let limits = Limits::capacity();
    let mut book = pivot_workbook::Workbook::new_in("identity capacity", limits, &runtime)?;
    let stable = Position::new(1, 1);
    let churn = Position::new(1, 2);
    book.apply(
        book.expected(),
        &[
            Edit::SetEntry {
                position: stable,
                source: "11".into(),
            },
            Edit::SetEntry {
                position: churn,
                source: "7".into(),
            },
        ],
    )?;
    let stable_ref = book.cell_at(stable)?.unwrap().reference;
    let old_churn = book.cell_at(churn)?.unwrap().reference;
    let first = book.capture_sheet(book.worksheet(), book.expected())?;
    book.apply(book.expected(), &[Edit::Retire(churn)])?;
    let mut captured = Vec::new();
    let start = Instant::now();
    while book.stats().allocated_nodes < limits.allocated_cells {
        let adds = (limits.allocated_cells - book.stats().allocated_nodes).min(128);
        let mut edits = Vec::new();
        for _ in 0..adds {
            edits.push(Edit::SetEntry {
                position: churn,
                source: "7".into(),
            });
            edits.push(Edit::Retire(churn));
        }
        let before = book.stats().allocated_nodes;
        book.apply(book.expected(), &edits)?;
        assert_eq!(book.stats().allocated_nodes, before + adds);
        assert_eq!(book.stats().live_cells, 1);
        assert_eq!(book.cell_at(stable)?.unwrap().reference, stable_ref);
        if captured.len() < 3 && book.stats().allocated_nodes >= (captured.len() + 1) * 65_536 {
            captured.push(book.capture_sheet(book.worksheet(), book.expected())?);
        }
    }
    let churn_ms = elapsed(start);
    let before = book.expected();
    assert!(
        book.apply(
            before,
            &[Edit::SetEntry {
                position: churn,
                source: "8".into()
            }]
        )
        .is_err()
    );
    assert_eq!(book.expected(), before);
    assert_eq!(
        first.evaluate_values(&[old_churn])?.values[0].1,
        Ok(PivotValue::Decimal(7.into()))
    );
    for _ in 0..31 {
        book.apply(book.expected(), &[Edit::Clear(stable)])?;
        book.apply(
            book.expected(),
            &[Edit::SetEntry {
                position: stable,
                source: "11".into(),
            }],
        )?;
    }
    assert_eq!(book.stats().allocated_nodes, limits.allocated_cells);
    assert_eq!(book.cell_at(stable)?.unwrap().reference, stable_ref);
    let start = Instant::now();
    let export = Document::export_document(&book, book.expected())?;
    native::install(&native::destination(path)?, &export.bytes)?;
    let save_ms = elapsed(start);
    let start = Instant::now();
    let (reopened, _) = Document::open_document(&native::read(path)?.0, limits, &runtime)?;
    let reopen_ms = elapsed(start);
    assert_eq!(reopened.stats().allocated_nodes, limits.allocated_cells);
    assert_eq!(reopened.cell_at(stable)?.unwrap().reference, stable_ref);
    let m = memory.stats();
    let traffic = memory.allocation_traffic();
    println!(
        "{{\"domain_allocation_count\":{},\"domain_allocation_bytes\":{}}}",
        traffic.0, traffic.1
    );
    println!(
        "{{\"identities\":{},\"positioned\":1,\"churn_ms\":{churn_ms},\"save_ms\":{save_ms},\"reopen_ms\":{reopen_ms},\"peak\":{},\"native_bytes\":{}}}",
        limits.allocated_cells,
        m.peak,
        export.bytes.len()
    );
    drop((reopened, export, first, captured, book));
    runtime.finish().map_err(|_| "runtime still owned")?;
    assert_eq!(memory.stats().charged, 0);
    println!("{{\"remaining_charge\":0}}");
    Ok(())
}
