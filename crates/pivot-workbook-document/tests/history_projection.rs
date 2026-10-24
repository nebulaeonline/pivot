#![cfg(feature = "history-experiment")]
use pivot_core::model::integration::{CancellationToken, IntegrationRuntime};
use pivot_storage::experimental_file::native;
use pivot_workbook::{history_experiment::ColdLedger, *};
use pivot_workbook_document::Document;
fn p(r: u32, c: u32) -> Position {
    Position::new(r, c)
}
fn apply(b: &mut Workbook, edits: &[Edit]) {
    b.apply(b.expected(), edits).unwrap();
}
fn entry(b: &mut Workbook, r: u32, c: u32, s: &str) {
    apply(
        b,
        &[Edit::SetEntry {
            position: p(r, c),
            source: s.into(),
        }],
    );
}
fn same(a: &Workbook, b: &Workbook) {
    for r in 1..=8 {
        for c in 1..=8 {
            assert_eq!(a.cell_at(p(r, c)).unwrap(), b.cell_at(p(r, c)).unwrap());
        }
    }
    // Validation reports the first core failure in runtime traversal order. A new
    // runtime namespace can select a cycle before a different rejected draft.
    // Compare all document-level diagnostics; isolated cycle coverage is below.
    let diagnostics = |book: &Workbook| {
        book.validate(book.expected())
            .unwrap()
            .into_iter()
            .filter(|d| !matches!(d, Diagnostic::Unorderable { .. }))
            .collect::<Vec<_>>()
    };
    assert_eq!(diagnostics(a), diagnostics(b));
}
#[test]
fn sparse_and_cold_historical_documents_preserve_exact_content_and_live_authority() {
    let rt = IntegrationRuntime::standalone().unwrap();
    let mut b = Workbook::new_in("G1", Limits::default(), &rt).unwrap();
    for (r, c, s) in [
        (1, 1, "10"),
        (1, 2, "=A1+1"),
        (2, 2, "=SUM(A1:A3)"),
        (2, 3, "=AVERAGE(A1:A3)"),
        (3, 2, "=B3"),
        (3, 3, "=("),
        (3, 4, "\u{03bb} \u{65e5}\u{672c}\u{8a9e}"),
        (3, 5, "=A4"),
    ] {
        entry(&mut b, r, c, s);
    }
    apply(
        &mut b,
        &[
            Edit::Retire(p(4, 1)),
            Edit::Move {
                from: p(1, 1),
                to: p(5, 1),
            },
            Edit::SetDecimal {
                position: p(5, 2),
                value: DecimalInput::from_parts(0, 0, 0, true, 5).unwrap(),
            },
            Edit::Clear(p(5, 3)),
            Edit::PreserveRejectedFormula {
                position: p(6, 1),
                source: "=A1".into(),
            },
        ],
    );
    let root = b.pin_history_root(b.expected()).unwrap();
    let mut l = ColdLedger::new(&b).unwrap();
    entry(&mut b, 7, 1, "future");
    entry(&mut b, 7, 2, "=SUM(A5:B5)");
    l.observe(&b).unwrap();
    let d = b.pin_history_root(b.expected()).unwrap();
    b.activate_history_root(b.expected(), &root, &Default::default())
        .unwrap();
    entry(&mut b, 8, 1, "90"); // sparse ordinal after a discarded branch
    l.observe(&b).unwrap();
    let root_b = b.pin_history_root(b.expected()).unwrap();
    let cold_b = b.history_cold_export(b.expected(), &l).unwrap();
    b.activate_history_root(b.expected(), &d, &Default::default())
        .unwrap();
    entry(&mut b, 8, 2, "newer");
    l.observe(&b).unwrap();
    let authority = b.history_cold_export(b.expected(), &l).unwrap();
    b.activate_history_root(b.expected(), &root_b, &Default::default())
        .unwrap();
    let expected = b.expected();
    let content = b.history_content_id();
    let node = b.history_runtime_node(p(5, 1)).unwrap();
    let metrics = format!("{:?}", b.history_metrics().unwrap());
    assert!(b.export_portable(expected).is_err());
    assert!(Document::export_document(&b, expected).is_err());
    let export = Document::export_historical_document(&b, expected, &Default::default()).unwrap();
    let (opened, checkpoint) =
        Document::open_document(&export.bytes, Limits::default(), &rt).unwrap();
    same(&b, &opened);
    assert_eq!(checkpoint, export.checkpoint);
    assert_ne!(opened.history_runtime_node(p(5, 1)).unwrap(), node);
    let (cold_book, cold_ledger) =
        Workbook::history_cold_reopen(&authority.bytes, &cold_b.bytes, Limits::default(), &rt)
            .unwrap();
    let e =
        Document::export_historical_document(&cold_book, cold_book.expected(), &Default::default())
            .unwrap();
    let (cold_opened, _) = Document::open_document(&e.bytes, Limits::default(), &rt).unwrap();
    same(&cold_book, &cold_opened);
    same(&b, &cold_book);
    let base = std::env::var_os("PIVOT_FILE_TEST_ROOT")
        .map(std::path::PathBuf::from)
        .unwrap_or_else(std::env::temp_dir);
    let scratch = base.join(format!("pivot-g1-{}", uuid::Uuid::new_v4()));
    std::fs::create_dir_all(&scratch).unwrap();
    let scratch = scratch.canonicalize().unwrap();
    std::fs::write(scratch.join("authority"), &authority.bytes).unwrap();
    std::fs::write(scratch.join("state"), &cold_b.bytes).unwrap();
    let child = std::process::Command::new(std::env::current_exe().unwrap())
        .args(["--exact", "cold_process_worker", "--nocapture"])
        .env("PIVOT_G1_CHILD", &scratch)
        .output()
        .unwrap();
    assert!(
        child.status.success(),
        "{}{}",
        String::from_utf8_lossy(&child.stdout),
        String::from_utf8_lossy(&child.stderr)
    );
    assert!(String::from_utf8_lossy(&child.stdout).contains("G1_CHILD_NATIVE_OPEN_FINISH_0"));
    let (disk, _) = native::read(&scratch.join("historical.pivot")).unwrap();
    let (disk_book, _) = Document::open_document(&disk, Limits::default(), &rt).unwrap();
    same(&b, &disk_book);
    drop(disk_book);
    assert!(scratch.starts_with(base.canonicalize().unwrap()));
    assert!(
        scratch
            .file_name()
            .unwrap()
            .to_string_lossy()
            .starts_with("pivot-g1-")
    );
    std::fs::remove_dir_all(&scratch).unwrap();
    let cancelled = CancellationToken::default();
    cancelled.cancel();
    assert!(Document::export_historical_document(&b, expected, &cancelled).is_err());
    let mut stale = expected;
    stale.state += 1;
    assert!(Document::export_historical_document(&b, stale, &Default::default()).is_err());
    rt.flush();
    let memory = rt.memory();
    let stats = memory.stats();
    let pressure = memory.reserve(stats.limit - stats.charged).unwrap();
    assert!(Document::export_historical_document(&b, expected, &Default::default()).is_err());
    drop(pressure);
    assert_eq!(b.expected(), expected);
    assert_eq!(b.history_content_id(), content);
    assert_eq!(b.history_runtime_node(p(5, 1)).unwrap(), node);
    assert_eq!(format!("{:?}", b.history_metrics().unwrap()), metrics);
    drop((
        cold_opened,
        cold_book,
        cold_ledger,
        opened,
        authority,
        cold_b,
        l,
        root_b,
        root,
        d,
        b,
    ));
    rt.flush();
    assert!(rt.finish().is_ok());
    assert_eq!(memory.stats().charged, 0);
}

#[test]
fn cold_process_worker() {
    let Some(root) = std::env::var_os("PIVOT_G1_CHILD") else {
        return;
    };
    let root = std::path::PathBuf::from(root);
    let rt = IntegrationRuntime::standalone().unwrap();
    let (b, l) = Workbook::history_cold_reopen(
        &std::fs::read(root.join("authority")).unwrap(),
        &std::fs::read(root.join("state")).unwrap(),
        Limits::default(),
        &rt,
    )
    .unwrap();
    let expected = b.expected();
    let content = b.history_content_id();
    let metrics = format!("{:?}", b.history_metrics().unwrap());
    assert!(b.export_portable(expected).is_err());
    let export = Document::export_historical_document(&b, expected, &Default::default()).unwrap();
    let path = root.join("historical.pivot");
    native::install(&native::destination(&path).unwrap(), &export.bytes).unwrap();
    let (bytes, _) = native::read(&path).unwrap();
    let (opened, checkpoint) = Document::open_document(&bytes, Limits::default(), &rt).unwrap();
    same(&b, &opened);
    assert_eq!(checkpoint, export.checkpoint);
    assert_eq!(b.expected(), expected);
    assert_eq!(b.history_content_id(), content);
    assert_eq!(format!("{:?}", b.history_metrics().unwrap()), metrics);
    drop((opened, b, l));
    rt.flush();
    let memory = rt.memory();
    assert!(rt.finish().is_ok());
    assert_eq!(memory.stats().charged, 0);
    println!("G1_CHILD_NATIVE_OPEN_FINISH_0");
}

#[test]
fn isolated_cycles_and_numeric_captures_survive_native_projection() {
    let rt = IntegrationRuntime::standalone().unwrap();
    for source in ["=A1", "=A1+1", "=(", "=SUM(A1:A2)"] {
        let mut b = Workbook::new_in("draft", Limits::default(), &rt).unwrap();
        entry(&mut b, 1, 1, source);
        let export =
            Document::export_historical_document(&b, b.expected(), &Default::default()).unwrap();
        let (opened, _) = Document::open_document(&export.bytes, Limits::default(), &rt).unwrap();
        same(&b, &opened);
        assert_eq!(
            b.validate(b.expected()).unwrap(),
            opened.validate(opened.expected()).unwrap()
        );
        drop((b, opened));
    }
    let mut b = Workbook::new_in("numeric", Limits::default(), &rt).unwrap();
    entry(&mut b, 1, 1, "1.2300");
    entry(&mut b, 1, 2, "=A1*2");
    entry(&mut b, 1, 3, "=SUM(A1:A2)");
    entry(&mut b, 1, 4, "=AVERAGE(A1:A2)");
    let capture = b.capture_local(b.expected()).unwrap();
    let cells: Vec<_> = (2..=4).map(|c| capture.cell_at(p(1, c)).unwrap()).collect();
    let before = capture.evaluate(&cells).unwrap().values;
    let root = b.pin_history_root(b.expected()).unwrap();
    entry(&mut b, 3, 1, "future");
    b.activate_history_root(b.expected(), &root, &Default::default())
        .unwrap();
    let export =
        Document::export_historical_document(&b, b.expected(), &Default::default()).unwrap();
    let (opened, _) = Document::open_document(&export.bytes, Limits::default(), &rt).unwrap();
    same(&b, &opened);
    assert_eq!(capture.evaluate(&cells).unwrap().values, before);
    let reopened = opened.capture_local(opened.expected()).unwrap();
    let outputs: Vec<_> = (2..=4)
        .map(|c| reopened.cell_at(p(1, c)).unwrap())
        .collect();
    assert_eq!(reopened.evaluate(&outputs).unwrap().values, before);
    drop((b, opened, root, capture, reopened));
    rt.flush();
    let memory = rt.memory();
    assert!(rt.finish().is_ok());
    assert_eq!(memory.stats().charged, 0);
}
