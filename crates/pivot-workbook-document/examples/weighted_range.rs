//! Declared bounded range-weighted corpora; core timings and charged memory, not GUI/RSS.
use pivot_core::model::integration::IntegrationRuntime;
use pivot_types::{
    DecimalError, PivotDecimal, PivotValue,
    weighted::{self, Scratch},
};
use pivot_workbook::{csv, *};
use std::{error::Error, time::Instant};
fn main() -> std::result::Result<(), Box<dyn Error>> {
    let mode = std::env::args().nth(1).unwrap_or("mixed".into());
    let (places, weights): (u32, Vec<String>) = match mode.as_str() {
        "mixed" => (2, (1..=64).map(|i| i.to_string()).collect()),
        "mixed-scale" => (
            28,
            (0..64)
                .map(|i| {
                    if i % 2 == 0 {
                        "79228162514264337593543950335".into()
                    } else {
                        "0.0000000000000000000000000001".into()
                    }
                })
                .collect(),
        ),
        "zeros-ties" => (
            28,
            (0..64)
                .map(|i| if i < 3 { "1".into() } else { "0".into() })
                .collect(),
        ),
        "wide" => (
            28,
            (0..64)
                .map(|i| match i {
                    0 => "26409387504754779197847983445".into(),
                    1 => "52818775009509558395695966890".into(),
                    _ => "0.0000000000000000000000000000".into(),
                })
                .collect(),
        ),
        "overflow" => (1, vec!["1".into(); 64]),
        _ => return Err("unknown weighted corpus".into()),
    };
    let rt = IntegrationRuntime::standalone()?;
    let memory = rt.memory();
    let mut b = csv::parse(
        "1\n".repeat(16384).as_bytes(),
        csv::Policy::default(),
        Limits::capacity(),
    )?
    .workbook(
        csv::Options {
            typed: true,
            ..Default::default()
        },
        Limits::capacity(),
        &rt,
    )?;
    let mut edits = weights
        .iter()
        .enumerate()
        .map(|(i, s)| Edit::SetEntry {
            position: Position::new(i as u32 + 1, 3),
            source: s.clone(),
        })
        .collect::<Vec<_>>();
    edits.extend((1..=256).map(|r| Edit::SetFormulaText {
        position: Position::new(r, 2),
        source: format!(
            "=ALLOCATEWEIGHTEDRANGE($D$1,{},{places},$C$1:$C$64)",
            (r - 1) % 64 + 1
        ),
    }));
    edits.push(Edit::SetFormulaText {
        position: Position::new(1, 5),
        source: "=SUM(B1:B64)".into(),
    });
    for batch in edits.chunks(256) {
        b.apply(b.expected(), batch)?;
    }
    let decimal_weights = weights
        .iter()
        .map(|s| s.parse::<PivotDecimal>().unwrap())
        .collect::<Vec<_>>();
    let mut roots = Vec::new();
    let mut helpers = Vec::new();
    let mut evaluations = Vec::new();
    let mut max_limb_visits = 0;
    for turn in 0..4 {
        let total = match mode.as_str() {
            "mixed" => (100 + turn).to_string(),
            "mixed-scale" => (10 + turn).to_string(),
            "zeros-ties" => (8 + turn).to_string(),
            "wide" => (79228162514264337593543950335u128 - 3 * turn).to_string(),
            _ => (79228162514264337593543950335u128 - 10 * turn).to_string(),
        };
        b.apply(
            b.expected(),
            &[Edit::SetEntry {
                position: Position::new(1, 4),
                source: total.clone(),
            }],
        )?;
        let decimal: PivotDecimal = total.parse()?;
        {
            let _charge = memory
                .reserve(weighted::SCRATCH_BYTES)
                .map_err(|_| "scratch admission")?;
            let mut scratch = Box::<Scratch>::default();
            let start = Instant::now();
            for row in 1..=256 {
                let result = scratch.allocate(
                    decimal,
                    (row - 1) % 64 + 1,
                    places,
                    &decimal_weights,
                    &|| false,
                );
                assert_eq!(result.is_err(), mode == "overflow");
                if let Ok((_, work)) = result {
                    max_limb_visits = max_limb_visits.max(work);
                }
                let _ = std::hint::black_box(result);
            }
            helpers.push(start.elapsed().as_secs_f64() * 1000.0);
        }
        let cap = b.capture_local(b.expected())?;
        let outputs = (1..=256)
            .map(|r| cap.cell_at(Position::new(r, 2)).unwrap())
            .collect::<Vec<_>>();
        let start = Instant::now();
        for (_, result) in cap.evaluate_values(&outputs)?.values {
            if mode == "overflow" {
                assert_eq!(result, Err(CellError::Arithmetic(DecimalError::Overflow)));
            } else {
                assert!(matches!(result, Ok(PivotValue::Decimal(_))));
            }
        }
        if mode != "overflow" {
            let sum = cap.evaluate_values(&[cap.cell_at(Position::new(1, 5)).unwrap()])?;
            assert_eq!(
                sum.values[0].1,
                Ok(PivotValue::Decimal(decimal.normalize()))
            );
        }
        evaluations.push(start.elapsed().as_secs_f64() * 1000.0);
        roots.push(cap);
    }
    let steady = memory.stats().charged;
    let peak = memory.stats().peak;
    println!(
        "{{\"mode\":\"{mode}\",\"base_cells\":16384,\"weight_cells\":64,\"weighted_formulas\":256,\"positioned\":{},\"retained_captures\":4,\"helper_ms\":{helpers:?},\"evaluate_ms\":{evaluations:?},\"max_limb_visits\":{max_limb_visits},\"limb_ceiling\":{},\"visit_charge_per_call\":{},\"explicit_scratch_bytes\":{},\"steady_bytes\":{steady},\"peak_bytes\":{peak}}}",
        b.stats().live_cells,
        weighted::limb_limit(64),
        weighted::visit_charge(64),
        weighted::EXPLICIT_BYTES
    );
    assert!(helpers.iter().chain(&evaluations).all(|&ms| ms <= 1000.0));
    assert!(steady <= 256 * 1024 * 1024 && peak <= 512 * 1024 * 1024);
    drop(roots);
    drop(b);
    assert!(rt.finish().is_ok());
    assert_eq!(memory.stats().charged, 0);
    println!("{{\"remaining_charge\":0}}");
    Ok(())
}
