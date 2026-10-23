#![cfg(feature = "history-experiment")]
use pivot_core::model::integration::{CancellationToken, IntegrationRuntime};
use pivot_workbook::{
    history_experiment::{
        ColdLedger,
        representation::{Mode, Store},
    },
    *,
};
use std::{
    fs,
    io::{Seek, SeekFrom, Write},
    path::PathBuf,
};
fn p(r: u32, c: u32) -> Position {
    Position::new(r, c)
}
fn e(r: u32, c: u32, s: &str) -> Edit {
    Edit::SetEntry {
        position: p(r, c),
        source: s.into(),
    }
}
fn path() -> PathBuf {
    std::env::temp_dir().join(format!("pivot-record-comparison-{}", uuid::Uuid::new_v4()))
}
fn oracle(b: &Workbook) -> String {
    let read = b
        .read(
            b.expected(),
            Rectangle {
                first: p(1, 1),
                last: p(3, 8),
            },
            32,
        )
        .unwrap();
    format!(
        "{:?}|{:?}|{:?}|{:?}",
        read.cells,
        b.validate(b.expected()).unwrap(),
        b.invalidated_cells(),
        b.history_content_id()
    )
}
fn fixture(rt: &IntegrationRuntime) -> Workbook {
    let mut b = Workbook::new_in("record oracle", Limits::default(), rt).unwrap();
    b.apply(
        b.expected(),
        &[
            e(1, 1, "10"),
            e(1, 2, "10"),
            e(1, 3, "=A1"),
            e(1, 4, "=("),
            e(1, 5, "=E1"),
            e(1, 6, "=MAX($A$2:B1)"),
            e(1, 7, "λ text"),
            e(2, 1, "0"),
            e(2, 2, "0"),
            Edit::SetDecimal {
                position: p(1, 8),
                value: DecimalInput::from_parts(0, 0, 0, true, 4).unwrap(),
            },
        ],
    )
    .unwrap();
    b
}
#[test]
fn representations_preserve_exact_oracle_through_reopen_fill_paste_and_point_removal() {
    for (mode, work_limits) in [
        (Mode::Full, None),
        (Mode::Delta { interval: 1 }, None),
        (Mode::Delta { interval: 16 }, None),
        (Mode::Delta { interval: 64 }, None),
        (Mode::Objects, None),
        (Mode::Delta { interval: 64 }, Some((1, usize::MAX))),
        (Mode::Delta { interval: 64 }, Some((usize::MAX, 1))),
        (Mode::Delta { interval: 64 }, Some((64 * 1024, 256))),
    ] {
        let rt = IntegrationRuntime::standalone().unwrap();
        let mut b = fixture(&rt);
        let mut ledger = ColdLedger::new(&b).unwrap();
        let dir = path();
        let mut store = Store::create(&dir, mode, rt.memory()).unwrap();
        if let Some((bytes, records)) = work_limits {
            store.set_replay_work_limits(bytes, records).unwrap();
        }
        let mut bytes = Vec::new();
        let mut oracles = Vec::new();
        let first = b.history_cold_export(b.expected(), &ledger).unwrap();
        store.append(&first, false).unwrap();
        bytes.push(first.bytes.clone());
        oracles.push(oracle(&b));
        let capture = b.capture_local(b.expected()).unwrap();
        let c = capture.cell_at(p(1, 3)).unwrap();
        let before = b.cell_at(p(1, 3)).unwrap();
        b.apply(b.expected(), &[e(1, 3, "=B1")]).unwrap();
        assert_ne!(b.cell_at(p(1, 3)).unwrap(), before);
        assert_eq!(capture.evaluate(&[c]).unwrap().values[0].1, Ok(10.into()));
        for i in 0..70 {
            match i % 7 {
                0 => {
                    b.apply(b.expected(), &[e(1, 1, &(i + 20).to_string())])
                        .unwrap();
                }
                1 => {
                    b.apply(
                        b.expected(),
                        &[e(1, 3, if i % 2 == 0 { "=B1" } else { "=A1" })],
                    )
                    .unwrap();
                }
                2 => {
                    b.fill(
                        b.expected(),
                        Rectangle {
                            first: p(1, 1),
                            last: p(2, 2),
                        },
                        FillDirection::Down,
                        &CancellationToken::default(),
                    )
                    .unwrap();
                }
                3 => {
                    b.paste_block(
                        b.expected(),
                        Rectangle {
                            first: p(2, 1),
                            last: p(2, 2),
                        },
                        format!("{}\t{}", i + 30, i + 31).as_bytes(),
                        false,
                        &CancellationToken::default(),
                    )
                    .unwrap();
                }
                4 => {
                    b.apply(b.expected(), &[e(2, 3, "=SUM($A$1:B2)")]).unwrap();
                }
                5 => {
                    b.apply(b.expected(), &[Edit::Clear(p(2, 1))]).unwrap();
                }
                _ => {
                    let (from, to) = if i / 7 % 2 == 0 {
                        (p(1, 2), p(3, 2))
                    } else {
                        (p(3, 2), p(1, 2))
                    };
                    let mut edits = Vec::new();
                    // Earlier formula resubmission may have created a blank anchor here.
                    if b.cell_at(to).unwrap().is_some() {
                        edits.push(Edit::Retire(to));
                    }
                    edits.push(Edit::Move { from, to });
                    b.apply(b.expected(), &edits).unwrap();
                }
            }
            ledger.observe(&b).unwrap();
            let blob = b.history_cold_export(b.expected(), &ledger).unwrap();
            store.append(&blob, false).unwrap();
            bytes.push(blob.bytes.clone());
            oracles.push(oracle(&b));
            rt.flush();
        }
        assert_eq!(store.count(), 71);
        if work_limits.is_some() {
            assert!(store.metrics.forced_checkpoints > 0);
        }
        let current = b.history_cold_export(b.expected(), &ledger).unwrap();
        drop(store);
        let mut reader = Store::reopen(&dir, mode, rt.memory()).unwrap();
        if let Some((bytes, records)) = work_limits {
            reader.set_replay_work_limits(bytes, records).unwrap();
        }
        for tx in [0, 1, 15, 16, 31, 63, 64, 70] {
            let (blob, work) = reader.load(tx).unwrap();
            assert_eq!(blob.bytes, bytes[tx as usize]);
            if let Mode::Delta { interval } = mode {
                assert!(work.chain_steps < interval);
            }
            let (restored, l) =
                Workbook::history_cold_reopen(&current.bytes, &blob.bytes, Limits::default(), &rt)
                    .unwrap();
            assert_eq!(oracle(&restored), oracles[tx as usize]);
            drop((restored, l));
            b.history_cold_activate(
                b.expected(),
                &blob.bytes,
                &ledger,
                &CancellationToken::default(),
            )
            .unwrap();
            assert_eq!(oracle(&b), oracles[tx as usize]);
        }
        // Activating other historical roots must not alter the retained invocation.
        assert_eq!(capture.evaluate(&[c]).unwrap().values[0].1, Ok(10.into()));
        let authoritative = b.expected();
        let old = oracle(&b);
        let mut data = fs::OpenOptions::new()
            .write(true)
            .open(dir.join("data"))
            .unwrap();
        data.seek(SeekFrom::Start(0)).unwrap();
        data.write_all(&[255]).unwrap();
        drop(data);
        assert!(reader.load(0).is_err());
        assert_eq!(b.expected(), authoritative);
        assert_eq!(oracle(&b), old);
        drop((reader, current, capture, ledger, b, first));
        rt.flush();
        let memory = rt.memory();
        assert!(rt.finish().is_ok());
        assert_eq!(memory.stats().charged, 0);
        // Only this newly-created unique test directory is removed, with a resolved-parent check.
        let resolved = dir.canonicalize().unwrap();
        assert!(resolved.starts_with(std::env::temp_dir().canonicalize().unwrap()));
        assert!(
            resolved
                .file_name()
                .unwrap()
                .to_string_lossy()
                .starts_with("pivot-record-comparison-")
        );
        fs::remove_dir_all(resolved).unwrap();
    }
}
