//! G3 research fixture: real G2 deltas with independent full-export oracles.
//! JSONL is an experiment interchange, never a workbook or protocol format.
use pivot_core::model::integration::{IntegrationRuntime, history_codec::Blob};
use pivot_workbook::{
    history_experiment::{
        ColdLedger,
        changes::{Mutation, RecordImage},
    },
    *,
};
use sha2::{Digest, Sha256};
use std::{
    fs::File,
    io::{BufWriter, Write},
    time::Instant,
};

fn hex(bytes: &[u8]) -> String {
    bytes.iter().map(|b| format!("{b:02x}")).collect()
}
fn records(blob: &Blob) -> String {
    blob.records
        .iter()
        .filter(|r| !matches!(r.key.0[0], 5 | 17))
        .map(|r| {
            format!(
                "[\"{}\",\"{}\"]",
                hex(&r.key.0),
                hex(&blob.bytes[r.range.clone()])
            )
        })
        .collect::<Vec<_>>()
        .join(",")
}
fn p(row: u32, column: u32) -> Position {
    Position::new(row, column)
}
fn rect(a: u32, b: u32, c: u32, d: u32) -> Rectangle {
    Rectangle {
        first: p(a, b),
        last: p(c, d),
    }
}
fn main() {
    let args: Vec<_> = std::env::args().collect();
    let count: usize = args[2].parse().unwrap();
    assert!(count > 0);
    let mut out = BufWriter::new(File::create_new(&args[1]).unwrap());
    let rt = IntegrationRuntime::standalone().unwrap();
    let memory = rt.memory();
    let mut book = Workbook::new_in("G3 fixed-identity stream", Limits::default(), &rt).unwrap();
    book.apply(
        book.expected(),
        &[
            Edit::SetEntry {
                position: p(1, 1),
                source: "0".into(),
            },
            Edit::SetEntry {
                position: p(1, 2),
                source: "=A1+1".into(),
            },
        ],
    )
    .unwrap();
    let mut ledger = ColdLedger::new(&book).unwrap();
    let mut prior = book.history_cold_export(book.expected(), &ledger).unwrap();
    writeln!(
        out,
        "{{\"seq\":0,\"content\":\"{}\",\"records\":[{}],\"oracle\":\"{}\"}}",
        book.history_content_id(),
        records(&prior),
        hex(&Sha256::digest(&prior.bytes))
    )
    .unwrap();
    let mut image = RecordImage::from_checkpoint(&prior, &memory).unwrap();
    let clock = Instant::now();
    for i in 1..=count {
        let edits = [Edit::SetEntry {
            position: if i % 5 == 2 {
                p(1, 2)
            } else if i % 5 == 4 {
                p(4, 6)
            } else {
                p(1, 1)
            },
            source: if i % 5 == 2 {
                format!("=A1+{i}")
            } else {
                format!("{i}")
            },
        }];
        let tsv = (0..8)
            .map(|_| (0..4).map(|_| i.to_string()).collect::<Vec<_>>().join("\t"))
            .collect::<Vec<_>>()
            .join("\n");
        let (case, mutation) = match i % 5 {
            3 => (
                "paste32",
                Mutation::Paste {
                    rectangle: rect(4, 1, 11, 4),
                    bytes: tsv.as_bytes(),
                    native: false,
                },
            ),
            0 => (
                "fill32",
                Mutation::Fill(rect(4, 6, 11, 9), FillDirection::Down),
            ),
            2 => ("formula", Mutation::Edits(&edits)),
            4 => ("fill-seed", Mutation::Edits(&edits)),
            _ => ("scalar", Mutation::Edits(&edits)),
        };
        let prepared = book
            .prepare_record_changes(book.expected(), &ledger, mutation, &Default::default())
            .unwrap()
            .unwrap();
        let folded = image.apply(&prepared, &memory).unwrap();
        let mut oracle_ledger = ledger.clone_for_oracle(&memory).unwrap();
        oracle_ledger.observe(prepared.candidate()).unwrap();
        let full = prepared
            .candidate()
            .history_cold_export(prepared.candidate().expected(), &oracle_ledger)
            .unwrap();
        assert_eq!(folded.checkpoint(&memory).unwrap().bytes, full.bytes);
        let ops = prepared
            .operations()
            .iter()
            .map(|op| {
                let field = |b: Option<&[u8]>| {
                    b.map(|v| format!("\"{}\"", hex(v)))
                        .unwrap_or("null".into())
                };
                format!(
                    "[\"{}\",{},{}]",
                    hex(&op.key().0),
                    field(op.before()),
                    field(op.after())
                )
            })
            .collect::<Vec<_>>()
            .join(",");
        writeln!(out,"{{\"seq\":{i},\"case\":\"{case}\",\"content\":\"{}\",\"ops\":[{ops}],\"oracle\":\"{}\",\"checkpoint\":{}}}",
            prepared.candidate().history_content_id(),hex(&Sha256::digest(&full.bytes)),
            if i % 64 == 0 {format!("[{}]",records(&full))} else {"null".into()}).unwrap();
        prepared.publish(&mut book, &mut ledger).unwrap();
        // Check real published authority as well as the independent reconstruction.
        assert_eq!(
            book.history_cold_export(book.expected(), &ledger)
                .unwrap()
                .bytes,
            full.bytes
        );
        image = folded;
        prior = full;
        if i % 64 == 0 {
            rt.flush();
        }
    }
    out.flush().unwrap();
    println!(
        "{{\"transactions\":{count},\"producer_oracle_ms\":{},\"domain_peak\":{}}}",
        clock.elapsed().as_millis(),
        memory.stats().peak
    );
    drop((out, image, prior, ledger, book));
    rt.flush();
    assert!(rt.finish().is_ok());
    assert_eq!(memory.stats().charged, 0);
    println!("FINISH 0");
}
