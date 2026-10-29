//! Defined fan-out/aggregate workload, independent closed-form numeric oracles.
use pivot_core::model::integration::IntegrationRuntime;
use pivot_types::PivotValue;
use pivot_workbook::{Edit, Limits, Position, Rectangle, Workbook};
use std::{error::Error, time::Instant};
fn p(i: usize, width: usize) -> Position {
    Position::new((i / width + 1) as u32, (i % width + 1) as u32)
}
fn ms(t: Instant) -> f64 {
    t.elapsed().as_secs_f64() * 1000.0
}
fn main() -> Result<(), Box<dyn Error>> {
    let count: usize = std::env::args().nth(1).ok_or("count required")?.parse()?;
    let width = if count == 65_536 { 16 } else { 8 };
    if !(1024..=65_536).contains(&count) || !count.is_multiple_of(width) {
        return Err("invalid bounded graph count".into());
    }
    let runtime = IntegrationRuntime::standalone()?;
    let memory = runtime.memory();
    let mut book = Workbook::new_in("capacity graph", Limits::default(), &runtime)?;
    let start = Instant::now();
    book.apply(
        book.expected(),
        &[Edit::SetEntry {
            position: p(0, width),
            source: "1".into(),
        }],
    )?;
    for begin in (1..count).step_by(256) {
        let edits: Vec<_> = (begin..(begin + 256).min(count))
            .map(|i| Edit::SetFormulaText {
                position: p(i, width),
                source: "=A1+1".into(),
            })
            .collect();
        book.apply(book.expected(), &edits)?;
    }
    let build_ms = ms(start);
    let old = book.capture_sheet(book.worksheet(), book.expected())?;
    let start = Instant::now();
    book.apply(
        book.expected(),
        &[Edit::SetEntry {
            position: p(0, width),
            source: "3".into(),
        }],
    )?;
    let capture = book.capture_sheet(book.worksheet(), book.expected())?;
    for begin in (0..count).step_by(128) {
        let refs: Vec<_> = (begin..(begin + 128).min(count))
            .map(|i| capture.cell_at(p(i, width)).unwrap())
            .collect();
        for (offset, (_, result)) in capture
            .evaluate_values(&refs)?
            .values
            .into_iter()
            .enumerate()
        {
            assert_eq!(
                result,
                Ok(PivotValue::Decimal(
                    if begin + offset == 0 { 3 } else { 4 }.into()
                ))
            );
        }
    }
    let recalculate_ms = ms(start);
    let old_ref = old.cell_at(p(count - 1, width)).unwrap();
    assert_eq!(
        old.evaluate_values(&[old_ref])?.values[0].1,
        Ok(PivotValue::Decimal(2.into()))
    );
    let range = format!(
        "=SUM(A1:{}{})",
        (b'A' + width as u8 - 2) as char,
        count / width
    );
    let start = Instant::now();
    book.apply(
        book.expected(),
        &[Edit::SetFormulaText {
            position: p(count - 1, width),
            source: range,
        }],
    )?;
    let aggregate = book.capture_sheet(book.worksheet(), book.expected())?;
    let expected = 3 + ((count / width * (width - 1) - 1) * 4) as i64;
    assert_eq!(
        aggregate
            .evaluate_values(&[aggregate.cell_at(p(count - 1, width)).unwrap()])?
            .values[0]
            .1,
        Ok(PivotValue::Decimal(expected.into()))
    );
    let aggregate_ms = ms(start);
    let independent = p(count - width - 1, width);
    // Last column is outside the SUM rectangle, so this closure stays local.
    let independent = Position::new(independent.row, width as u32);
    book.apply(
        book.expected(),
        &[Edit::SetEntry {
            position: independent,
            source: "9".into(),
        }],
    )?;
    let mut local = Vec::new();
    let mut viewport = Vec::new();
    for i in 0..31 {
        let start = Instant::now();
        book.apply(
            book.expected(),
            &[Edit::SetEntry {
                position: independent,
                source: (i + 10).to_string(),
            }],
        )?;
        local.push(ms(start));
        let start = Instant::now();
        let frame = book.read(
            book.expected(),
            Rectangle {
                first: Position::new(1, 1),
                last: Position::new(20, 10),
            },
            200,
        )?;
        let cap = book.capture_sheet(book.worksheet(), book.expected())?;
        let refs: Vec<_> = frame.cells.iter().map(|c| c.reference).collect();
        cap.evaluate_values(&refs)?;
        viewport.push(ms(start));
    }
    local.sort_by(f64::total_cmp);
    viewport.sort_by(f64::total_cmp);
    let current = book.capture_sheet(book.worksheet(), book.expected())?;
    assert_eq!(
        current
            .evaluate_values(&[current.cell_at(p(count - 1, width)).unwrap()])?
            .values[0]
            .1,
        Ok(PivotValue::Decimal(expected.into()))
    );
    println!(
        "{{\"cells\":{count},\"build_ms\":{build_ms},\"recalculate_ms\":{recalculate_ms},\"aggregate_ms\":{aggregate_ms},\"local_edit_p95_ms\":{},\"viewport_p95_ms\":{},\"charged\":{},\"peak\":{}}}",
        local[29],
        viewport[29],
        memory.stats().charged,
        memory.stats().peak
    );
    drop((current, aggregate, capture, old, book));
    runtime.finish().map_err(|_| "runtime still owned")?;
    assert_eq!(memory.stats().charged, 0);
    println!("{{\"remaining_charge\":0}}");
    if recalculate_ms > 2000.0 || local[29] > 100.0 || viewport[29] > 100.0 {
        return Err("defined graph performance objective failed; stop and diagnose".into());
    }
    Ok(())
}
