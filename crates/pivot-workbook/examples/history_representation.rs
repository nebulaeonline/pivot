//! Stage 2 local-I/O comparison; no production commit, replay or durability claims.
use pivot_core::model::integration::{CancellationToken, IntegrationRuntime};
use pivot_workbook::{
    history_experiment::{
        ColdLedger,
        representation::{Mode, Store},
    },
    *,
};
use std::{
    collections::BTreeSet,
    fs,
    path::{Path, PathBuf},
    time::Instant,
};
struct Times {
    bins: [u64; 64],
    n: u64,
    total: u128,
    max: u64,
}
impl Default for Times {
    fn default() -> Self {
        Self {
            bins: [0; 64],
            n: 0,
            total: 0,
            max: 0,
        }
    }
}
impl Times {
    fn add(&mut self, n: u128) {
        let n = n.min(u64::MAX as u128) as u64;
        let bucket = n.max(1).ilog2() as usize;
        self.bins[bucket] += 1;
        self.n += 1;
        self.total += n as u128;
        self.max = self.max.max(n);
    }
    fn p95(&self) -> u128 {
        let target = self.n * 95 / 100;
        let mut count = 0;
        for (i, n) in self.bins.iter().enumerate() {
            count += n;
            if count > target {
                return (1u128 << (i + 1)) - 1;
            }
        }
        0
    }
}
fn p(r: u32, c: u32) -> Position {
    Position::new(r, c)
}
fn edit(r: u32, c: u32, s: String) -> Edit {
    Edit::SetEntry {
        position: p(r, c),
        source: s,
    }
}
fn oracle(book: &Workbook, rows: u32, cols: u32) -> String {
    let read = book
        .read(
            book.expected(),
            Rectangle {
                first: p(1, 1),
                last: p(rows, cols),
            },
            1024,
        )
        .unwrap();
    format!(
        "{:?}|{:?}|{:?}|{:?}",
        read.cells,
        book.validate(book.expected()).unwrap(),
        book.invalidated_cells(),
        book.history_content_id()
    )
}
fn read_bounded(
    path: &Path,
    memory: &pivot_core::model::integration::AllocationDomain,
) -> (Vec<u8>, pivot_core::model::integration::resources::Charge) {
    let len = fs::metadata(path).unwrap().len() as usize;
    assert!(len <= 16 * 1024 * 1024);
    let charge = memory.reserve(len + 1024).unwrap();
    (fs::read(path).unwrap(), charge)
}
fn main() {
    let args: Vec<_> = std::env::args().collect();
    assert!(
        args.len() >= 5,
        "case mode transactions output-directory [sync]"
    );
    let case = &args[1];
    let label = &args[2];
    let count: usize = args[3].parse().unwrap();
    assert!((1..=10000).contains(&count));
    let output = PathBuf::from(&args[4]);
    fs::create_dir_all(&output).unwrap();
    let sync = args.get(5).is_some_and(|x| x == "sync");
    let mixed = args.get(6).is_some_and(|x| x == "mixed");
    let workload = if mixed { "mixed" } else { "replace" };
    let mode = match label.as_str() {
        "full" => Mode::Full,
        "delta1" => Mode::Delta { interval: 1 },
        "delta16" => Mode::Delta { interval: 16 },
        "delta64" | "bounded64" => Mode::Delta { interval: 64 },
        "objects" => Mode::Objects,
        _ => panic!("mode"),
    };
    let (cells, cols, formulas) = match case.as_str() {
        "small" => (16, 4, false),
        "medium" => (256, 32, false),
        "formulas" => (256, 32, true),
        "near" => (1024, 32, false),
        _ => panic!("case"),
    };
    let rows = (cells / cols) as u32;
    let cols = cols as u32;
    let run = output.join(format!("working-{}", uuid::Uuid::new_v4()));
    let rt = IntegrationRuntime::standalone().unwrap();
    let memory = rt.memory();
    let mut book = Workbook::new_in("representation benchmark", Limits::default(), &rt).unwrap();
    for base in (0..cells).step_by(256) {
        let edits = (base..(base + 256).min(cells))
            .map(|i| {
                let r = i as u32 / cols + 1;
                let c = i as u32 % cols + 1;
                edit(
                    r,
                    c,
                    if formulas && c > 1 {
                        format!("=$A$1+{c}")
                    } else {
                        (i + 1).to_string()
                    },
                )
            })
            .collect::<Vec<_>>();
        book.apply(book.expected(), &edits).unwrap();
    }
    let mut ledger = ColdLedger::new(&book).unwrap();
    let mut store = Store::create(&run, mode, memory.clone()).unwrap();
    if label == "bounded64" {
        store.set_replay_work_limits(64 * 1024, 256).unwrap();
    }
    let _samples = memory.reserve(8192).unwrap();
    let sample_ids: BTreeSet<_> = [0, (count - 1) / 2, count - 1, 499, 999, 4999, 9999]
        .into_iter()
        .filter(|n| *n < count)
        .collect();
    let mut mutation = Times::default();
    let mut encoding = Times::default();
    let mut appending = Times::default();
    let mut loads = Times::default();
    let mut restore = Times::default();
    let mut full_bytes = 0u64;
    let mut encoded_records = 0u64;
    let mut buffer_capacity = 0;
    let mut mutation_peak = 0;
    let mut encode_peak = 0;
    let mut append_peak = 0;
    let mut load_peak = 0;
    let mut restore_peak = 0;
    let mut encoded_size = 0;
    let mut max_chain = 0;
    let mut max_read_bytes = 0;
    let mut max_read_records = 0;
    let mut max_file_reads = 0;
    for step in 0..count {
        rt.flush();
        memory.history_reset_peak();
        let start = Instant::now();
        let before = book.expected();
        if !mixed {
            book.apply(before, &[edit(1, 1, (step + 1000).to_string())])
                .unwrap();
        } else {
            match step % 8 {
                1 => {
                    book.fill(
                        before,
                        Rectangle {
                            first: p(1, 1),
                            last: p(rows.min(8), cols),
                        },
                        FillDirection::Down,
                        &CancellationToken::default(),
                    )
                    .unwrap();
                }
                3 => {
                    let tsv = (0..rows.min(8))
                        .map(|i| (step + 5000 + i as usize).to_string())
                        .collect::<Vec<_>>()
                        .join("\n");
                    book.paste_block(
                        before,
                        Rectangle {
                            first: p(1, 1),
                            last: p(rows.min(8), 1),
                        },
                        tsv.as_bytes(),
                        false,
                        &CancellationToken::default(),
                    )
                    .unwrap();
                }
                5 => {
                    book.apply(
                        before,
                        &[Edit::SetDecimal {
                            position: p(1, 1),
                            value: DecimalInput::from_parts((step + 1000) as u32, 0, 0, false, 2)
                                .unwrap(),
                        }],
                    )
                    .unwrap();
                }
                7 => {
                    let block = book
                        .copy_block(
                            before,
                            Rectangle {
                                first: p(1, 1),
                                last: p(1, cols),
                            },
                        )
                        .unwrap();
                    book.paste_block(
                        before,
                        Rectangle {
                            first: p(rows, 1),
                            last: p(rows, cols),
                        },
                        &block.native,
                        true,
                        &CancellationToken::default(),
                    )
                    .unwrap();
                }
                _ => {
                    let width = if step % 8 == 0 || formulas { 1 } else { cols };
                    let edits = (1..=width)
                        .map(|col| edit(1, col, (step + 1000 + col as usize).to_string()))
                        .collect::<Vec<_>>();
                    book.apply(before, &edits).unwrap();
                }
            }
        }
        assert_ne!(
            book.expected(),
            before,
            "workload must commit a changed transaction"
        );
        mutation.add(start.elapsed().as_nanos());
        mutation_peak = mutation_peak.max(memory.stats().peak);
        rt.flush();
        memory.history_reset_peak();
        let start = Instant::now();
        ledger.observe(&book).unwrap();
        let capsule = book.history_cold_export(book.expected(), &ledger).unwrap();
        encoding.add(start.elapsed().as_nanos());
        encode_peak = encode_peak.max(memory.stats().peak);
        full_bytes += capsule.bytes.len() as u64;
        encoded_records += capsule.records.len() as u64;
        encoded_size = capsule.bytes.len();
        buffer_capacity = buffer_capacity.max(capsule.buffer_capacity());
        memory.history_reset_peak();
        let start = Instant::now();
        assert_eq!(store.append(&capsule, sync).unwrap(), step as u64);
        appending.add(start.elapsed().as_nanos());
        append_peak = append_peak.max(memory.stats().peak);
        if sample_ids.contains(&step) {
            let _oracle_scratch = memory.reserve(cells * 2048 + 16384).unwrap();
            fs::write(
                output.join(format!("{case}-{label}-{count}-{sync}-{step}.capsule")),
                &capsule.bytes,
            )
            .unwrap();
            fs::write(
                output.join(format!("{case}-{label}-{count}-{sync}-{step}.oracle")),
                oracle(&book, rows, cols),
            )
            .unwrap();
        }
        if step + 1 == 1000 || step + 1 == count {
            for tx in [0, (step / 2) as u64, step as u64] {
                rt.flush();
                memory.history_reset_peak();
                let start = Instant::now();
                let (blob, work) = store.load(tx).unwrap();
                loads.add(start.elapsed().as_nanos());
                load_peak = load_peak.max(memory.stats().peak);
                max_chain = max_chain.max(work.chain_steps);
                max_read_bytes = max_read_bytes.max(work.bytes);
                max_read_records = max_read_records.max(work.records);
                max_file_reads = max_file_reads.max(work.file_reads);
                let reference = output.join(format!("{case}-{label}-{count}-{sync}-{tx}.capsule"));
                let (bytes, charge) = read_bounded(&reference, &memory);
                assert_eq!(blob.bytes, bytes);
                drop((bytes, charge));
                memory.history_reset_peak();
                let start = Instant::now();
                let (restored, l) = Workbook::history_cold_reopen(
                    &capsule.bytes,
                    &blob.bytes,
                    Limits::default(),
                    &rt,
                )
                .unwrap();
                restore.add(start.elapsed().as_nanos());
                restore_peak = restore_peak.max(memory.stats().peak);
                let _oracle_scratch = memory.reserve(cells * 2048 + 16384).unwrap();
                let (expected, _expected_charge) = read_bounded(
                    &output.join(format!("{case}-{label}-{count}-{sync}-{tx}.oracle")),
                    &memory,
                );
                assert_eq!(
                    oracle(&restored, rows, cols),
                    std::str::from_utf8(&expected).unwrap()
                );
                assert_ne!(restored.expected().session, book.expected().session);
                drop((restored, l, blob));
                rt.flush();
            }
        }
        drop(capsule);
        rt.flush();
        if step + 1 == 1000 || step + 1 == count {
            let m = memory.stats();
            let metrics = &store.metrics;
            let checkpoints = metrics.checkpoints;
            let forced = metrics.forced_checkpoints;
            println!(
                "{{\"case\":\"{case}\",\"mode\":\"{label}\",\"transactions\":{},\"workload\":\"{workload}\",\"sync_every_transaction\":{sync},\"encoded_bytes_total\":{full_bytes},\"encoded_bytes_last\":{encoded_size},\"encoded_records_total\":{encoded_records},\"encoded_buffer_capacity_max\":{buffer_capacity},\"checkpoints\":{checkpoints},\"forced_checkpoints\":{forced},\"stored_file_bytes\":{},\"object_files\":{},\"root_index_bytes\":{},\"compared_records\":{},\"changed_records\":{},\"resident_record_cache\":{},\"resident_record_buffer_capacity\":{},\"resident_page_cache\":{},\"domain_idle_allocated\":{},\"domain_idle_reserved\":{},\"domain_idle_charged\":{},\"mutation_peak_charged\":{mutation_peak},\"encode_peak_charged\":{encode_peak},\"append_peak_charged\":{append_peak},\"load_peak_charged\":{load_peak},\"restore_peak_charged\":{restore_peak},\"mutation_ns_total\":{},\"export_ns_total\":{},\"append_ns_total\":{},\"append_p95_upper_ns\":{},\"append_max_ns\":{},\"record_plan_ns_total\":{},\"object_io_ns_total\":{},\"payload_root_io_ns_total\":{},\"sync_ns_total\":{},\"load_ns_total\":{},\"load_max_ns\":{},\"reconstruct_ns_total\":{},\"reconstruct_max_ns\":{},\"max_chain_steps\":{max_chain},\"max_read_bytes\":{max_read_bytes},\"max_decoded_records\":{max_read_records},\"max_file_reads\":{max_file_reads},\"lifetime_node_floor\":{},\"semantic_samples_passed\":{}}}",
                step + 1,
                store.file_bytes().unwrap(),
                metrics.object_files,
                metrics.journal_bytes,
                metrics.compared_records,
                metrics.changed_records,
                store.resident_records(),
                metrics.records_capacity,
                metrics.pages_cached,
                m.allocated,
                m.reserved,
                m.charged,
                mutation.total,
                encoding.total,
                appending.total,
                appending.p95(),
                appending.max,
                metrics.record_plan_ns,
                metrics.object_io_ns,
                metrics.data_root_io_ns,
                metrics.sync_ns,
                loads.total,
                loads.max,
                restore.total,
                restore.max,
                ledger.allocated_floor(),
                loads.n
            );
        }
    }
    // Reopen the disk index without a surviving resident representation catalog.
    drop(store);
    let mut reopened = Store::reopen(&run, mode, memory.clone()).unwrap();
    if label == "bounded64" {
        reopened.set_replay_work_limits(64 * 1024, 256).unwrap();
    }
    assert_eq!(reopened.count(), count as u64);
    let (last, work) = reopened.load((count - 1) as u64).unwrap();
    assert_eq!(last.bytes.len(), encoded_size);
    assert!(work.chain_steps < 64);
    drop((last, reopened, ledger, book, _samples));
    rt.flush();
    assert!(rt.finish().is_ok());
    assert_eq!(memory.stats().charged, 0);
    println!(
        "{{\"case\":\"{case}\",\"mode\":\"{label}\",\"finished_transactions\":{count},\"remaining_charge\":0}}"
    );
    // Remove only the unique dataset created by this run, retaining sample bytes/oracles and receipts.
    let resolved = run.canonicalize().unwrap();
    assert!(resolved.starts_with(output.canonicalize().unwrap()));
    assert!(
        resolved
            .file_name()
            .unwrap()
            .to_string_lossy()
            .starts_with("working-")
    );
    fs::remove_dir_all(resolved).unwrap();
}
