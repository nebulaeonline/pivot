//! Fresh-process G2 measurements. Full export/reconstruction is a separate oracle phase.
use pivot_core::model::integration::IntegrationRuntime;
use pivot_workbook::{
    history_experiment::{
        ColdLedger,
        changes::{Mutation, RecordImage},
    },
    *,
};
use std::time::Instant;
fn p(r: u32, c: u32) -> Position {
    Position::new(r, c)
}
fn main() {
    let args: Vec<_> = std::env::args().collect();
    let n: u32 = args[1].parse().unwrap();
    let case = args[2].as_str();
    assert!((2..=768).contains(&n));
    let rt = IntegrationRuntime::standalone().unwrap();
    let mut book = Workbook::new_in("G2 measurement", Limits::default(), &rt).unwrap();
    for first in (1..=n).step_by(256) {
        let edits: Vec<_> = (first..=(first + 255).min(n))
            .map(|r| Edit::SetDecimal {
                position: p(r, 1),
                value: DecimalInput::from_decimal(i64::from(r).into()),
            })
            .collect();
        book.apply(book.expected(), &edits).unwrap();
    }
    let formula = match case {
        "input" | "allocate" => None,
        "formula" => Some("=A1+2".into()),
        "range-input" | "range-formula" => Some(format!("=SUM(A1:A{n})")),
        _ => panic!("case"),
    };
    if let Some(source) = formula {
        book.apply(
            book.expected(),
            &[Edit::SetEntry {
                position: p(1, 2),
                source,
            }],
        )
        .unwrap();
    }
    let mut ledger = ColdLedger::new(&book).unwrap();
    let prior = book.history_cold_export(book.expected(), &ledger).unwrap();
    let prior_records = prior.records.len();
    let prior_bytes = prior.bytes.len();
    let image = RecordImage::from_checkpoint(&prior, &rt.memory()).unwrap();
    let original = book.expected();
    let content = book.history_content_id();
    let floor = book.history_metrics().unwrap();
    let capture = book.capture_local(original).unwrap();
    let position = if case == "input" || case == "allocate" {
        p(1, 1)
    } else {
        p(1, 2)
    };
    let output = capture.cell_at(position).unwrap();
    let before = capture.evaluate(&[output]).unwrap().values;
    let edits = match case {
        "allocate" => vec![Edit::SetDecimal {
            position: p(1, 2),
            value: DecimalInput::from_decimal(50.into()),
        }],
        "input" | "range-input" => vec![Edit::SetDecimal {
            position: p(1, 1),
            value: DecimalInput::from_decimal(50.into()),
        }],
        "formula" => vec![Edit::SetEntry {
            position: p(1, 2),
            source: "=A2+3".into(),
        }],
        "range-formula" => vec![Edit::SetEntry {
            position: p(1, 2),
            source: format!("=SUM(A2:A{n})"),
        }],
        _ => unreachable!(),
    };
    rt.flush();
    let memory = rt.memory();
    let baseline = memory.stats().charged;
    memory.history_reset_peak();
    let traffic = memory.allocation_traffic();
    let prepared = book
        .prepare_record_changes(
            original,
            &ledger,
            Mutation::Edits(&edits),
            &Default::default(),
        )
        .unwrap()
        .unwrap();
    rt.flush();
    let peak = memory.stats().peak;
    let local_traffic = memory.allocation_traffic();
    let metrics = prepared.metrics().clone();
    assert_eq!(book.expected(), original);
    assert_eq!(book.history_content_id(), content);
    assert_eq!(book.history_metrics().unwrap(), floor);
    assert_eq!(capture.evaluate(&[output]).unwrap().values, before);
    memory.history_reset_peak();
    let oracle_start = Instant::now();
    let mut next = ledger.clone_for_oracle(&memory).unwrap();
    next.observe(prepared.candidate()).unwrap();
    let full = prepared
        .candidate()
        .history_cold_export(prepared.candidate().expected(), &next)
        .unwrap();
    let rebuilt = image
        .apply(&prepared, &memory)
        .unwrap()
        .checkpoint(&memory)
        .unwrap();
    assert_eq!(full.bytes, rebuilt.bytes);
    let oracle_ns = oracle_start.elapsed().as_nanos();
    rt.flush();
    let oracle_peak = memory.stats().peak;
    let candidate_capture = prepared
        .candidate()
        .capture_local(prepared.candidate().expected())
        .unwrap();
    let target = candidate_capture
        .cell_at(if case == "allocate" {
            p(1, 2)
        } else {
            position
        })
        .unwrap();
    let expected = match case {
        "input" | "allocate" => 50,
        "formula" => 5,
        "range-input" => n * (n + 1) / 2 + 49,
        "range-formula" => n * (n + 1) / 2 - 1,
        _ => unreachable!(),
    };
    assert_eq!(
        candidate_capture.evaluate(&[target]).unwrap().values[0].1,
        Ok(i64::from(expected).into())
    );
    rt.flush();
    let publication_baseline = memory.stats().charged;
    memory.history_reset_peak();
    let publication_start = Instant::now();
    prepared.publish(&mut book, &mut ledger).unwrap();
    let publication_ns = publication_start.elapsed().as_nanos();
    rt.flush();
    let publication_peak = memory.stats().peak;
    let publication_increment = publication_peak - publication_baseline;
    assert_eq!(
        book.history_cold_export(book.expected(), &ledger)
            .unwrap()
            .bytes,
        full.bytes
    );
    assert_eq!(capture.evaluate(&[output]).unwrap().values, before);
    println!(
        "{{\"size\":{n},\"case\":\"{case}\",\"source_records\":{prior_records},\"source_bytes\":{prior_bytes},\"oracle_records\":{},\"oracle_bytes\":{},\"oracle_ns\":{oracle_ns},\"publication_ns\":{publication_ns},\"publication_peak_charge\":{publication_peak},\"publication_peak_increment\":{publication_increment},\"local_ns\":{},\"cell_record_reads\":{},\"core_record_reads\":{},\"mutation_marks\":{},\"canonical_record_reads\":{},\"canonical_record_scans\":{},\"membership_index_visits\":{},\"selection_candidates\":{},\"selection_members\":{},\"records_emitted\":{},\"bytes_compared\":{},\"bytes_emitted\":{},\"seal_scratch_admission\":{},\"baseline_charge\":{baseline},\"producer_peak_charge\":{peak},\"producer_peak_increment\":{},\"oracle_peak_charge\":{oracle_peak},\"producer_allocations\":{},\"producer_allocation_bytes\":{}}}",
        full.records.len(),
        full.bytes.len(),
        metrics.local_ns,
        metrics.workbook_keys_visited,
        metrics.core_keys_visited,
        metrics.mutation_writes,
        metrics.mutation_record_reads,
        metrics.mutation_record_scans,
        metrics.membership_index_visits,
        metrics.selection_candidates,
        metrics.selection_members,
        metrics.emitted_records,
        metrics.compared_bytes,
        metrics.emitted_bytes,
        metrics.scratch_admission,
        peak - baseline,
        local_traffic.0 - traffic.0,
        local_traffic.1 - traffic.1
    );
    drop((
        candidate_capture,
        full,
        rebuilt,
        capture,
        image,
        prior,
        next,
        ledger,
        book,
    ));
    rt.flush();
    assert!(rt.finish().is_ok());
    assert_eq!(memory.stats().charged, 0);
    println!("FINISH 0");
}
