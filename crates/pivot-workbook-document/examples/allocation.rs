//! Bounded numeric work, with four retained captures; excludes setup and GUI latency.
use pivot_core::model::integration::IntegrationRuntime;
use pivot_types::{DecimalError, PivotDecimal, PivotValue};
use pivot_workbook::{csv, *};
use std::{error::Error, time::Instant};

fn main() -> std::result::Result<(), Box<dyn Error>> {
    let mode = std::env::args().nth(1).unwrap_or("cents".into());
    let (n, places) = match mode.as_str() {
        "cents" => (4096, 2),
        "tiny" => (4096, 28),
        "wide" => (3, 28),
        "large-count" => (u32::MAX, 2),
        "overflow" => (10, 1),
        _ => return Err("unknown allocation corpus".into()),
    };
    let rt = IntegrationRuntime::standalone()?;
    let memory = rt.memory();
    let text = "1\n".repeat(16384);
    let mut book = csv::parse(text.as_bytes(), csv::Policy::default(), Limits::capacity())?
        .workbook(
            csv::Options {
                typed: true,
                ..Default::default()
            },
            Limits::capacity(),
            &rt,
        )?;
    let edits = (1..=4096)
        .map(|row| Edit::SetFormulaText {
            position: Position::new(row, 2),
            source: format!(
                "=ALLOCATE($C$1,{n},{},{places})",
                if n < 4096 { (row - 1) % n + 1 } else { row }
            ),
        })
        .collect::<Vec<_>>();
    for batch in edits.chunks(256) {
        book.apply(book.expected(), batch)?;
    }
    book.apply(
        book.expected(),
        &[Edit::SetFormulaText {
            position: Position::new(1, 4),
            source: format!("=SUM(B1:B{})", if mode == "wide" { 3 } else { 4096 }),
        }],
    )?;
    let mut roots = Vec::new();
    let mut helper_ms = Vec::new();
    let mut evaluate_ms = Vec::new();
    for turn in 0..4 {
        let total = match mode.as_str() {
            "cents" => (100 + turn).to_string(),
            "tiny" => format!("0.000000000000000000000000000{}", turn + 1),
            "wide" => (79228162514264337593543950335u128 - 3 * turn).to_string(),
            "large-count" => (10 * (turn + 1)).to_string(),
            _ => (79228162514264337593543950335u128 - 10 * turn).to_string(),
        };
        book.apply(
            book.expected(),
            &[Edit::SetEntry {
                position: Position::new(1, 3),
                source: total.clone(),
            }],
        )?;
        let decimal: PivotDecimal = total.parse()?;
        let start = Instant::now();
        for row in 1..=4096 {
            let index = if n < 4096 { (row - 1) % n + 1 } else { row };
            let result = decimal.allocate_equal(n, index, places);
            assert_eq!(result.is_err(), mode == "overflow");
            let _ = std::hint::black_box(result);
        }
        helper_ms.push(start.elapsed().as_secs_f64() * 1000.0);
        let cap = book.capture_local(book.expected())?;
        let ids = (1..=4096)
            .map(|row| cap.cell_at(Position::new(row, 2)).unwrap())
            .collect::<Vec<_>>();
        let start = Instant::now();
        for batch in ids.chunks(256) {
            for (_, result) in cap.evaluate_values(batch)?.values {
                if mode == "overflow" {
                    assert_eq!(result, Err(CellError::Arithmetic(DecimalError::Overflow)));
                } else {
                    assert!(matches!(result, Ok(PivotValue::Decimal(_))));
                }
            }
        }
        if mode != "overflow" {
            let sum = cap.evaluate_values(&[cap.cell_at(Position::new(1, 4)).unwrap()])?;
            assert_eq!(
                sum.values[0].1,
                Ok(PivotValue::Decimal(decimal.normalize()))
            );
        }
        evaluate_ms.push(start.elapsed().as_secs_f64() * 1000.0);
        roots.push(cap);
    }
    let steady = memory.stats().charged;
    let peak = memory.stats().peak;
    println!(
        "{{\"mode\":\"{mode}\",\"base_cells\":16384,\"allocation_formulas\":4096,\"positioned\":{},\"retained_captures\":4,\"helper_ms\":{helper_ms:?},\"evaluate_ms\":{evaluate_ms:?},\"steady_bytes\":{steady},\"peak_bytes\":{peak}}}",
        book.stats().live_cells
    );
    assert!(helper_ms.iter().chain(&evaluate_ms).all(|&ms| ms <= 1000.0));
    assert!(steady <= 256 * 1024 * 1024 && peak <= 512 * 1024 * 1024);
    drop(roots);
    drop(book);
    assert!(rt.finish().is_ok());
    assert_eq!(memory.stats().charged, 0);
    println!("{{\"remaining_charge\":0}}");
    Ok(())
}
