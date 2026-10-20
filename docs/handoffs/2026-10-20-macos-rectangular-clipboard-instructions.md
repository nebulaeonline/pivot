# Native macOS validation — rectangular clipboard and WB3

Read the [implementation/manual checklist](2026-10-20-rectangular-clipboard.md)
and [ADR-0017](../adr/0017-rectangular-clipboard-and-formula-copy.md).
Validate the current working tree, including dirty/untracked files, not HEAD.
Do not change product code, dependencies, assertions, fixtures or caps to obtain
passes. Retain every failure/rerun. Do not commit or edit shared project docs.

Locate the SMB Pivot root (previously `/Volumes/RobMac/coding/pivot`). Copy with
`rsync -a` into a fresh internal APFS directory, excluding `.git/`, `artifacts/`,
`target/`, `bin/`, `obj/`, `__pycache__/` and `.vscode/`. Include new/untracked Rust
clipboard module/tests, C# ClipboardChecks and the frozen WB2 fixture. Verify native
APFS with `diskutil info` and `df`. Source, Cargo target, .NET bin/obj, temporary
files, document fixtures and initial reports must stay on APFS. Never build or test
.NET on SMB. Preserve the APFS source/build directory for diagnosis.

Record share HEAD/status/diff, copy command, OS/hardware/SDKs and all copied source
hashes; compare local/share before and after testing. Separately copy and verify
all **172** entries from `artifacts/rectangular-clipboard-build-inputs.json`, returning
that reference manifest and verification results. Do not silently use stale source.

Set PYTHONUTF8=1, local CARGO_TARGET_DIR, TMPDIR, PIVOT_FILE_TEST_ROOT and REPORTS.
Run sequentially from the APFS source root; retain complete logs and actual exits:

```sh
python3 scripts/ci.py --output-dir "$REPORTS/ci"
cargo clippy --locked --workspace --all-targets --all-features -- -D warnings
cargo test --locked --workspace --all-features
python3 tests/application-session/ui_smoke.py --output-dir "$REPORTS/ui"
```

Collect the full CI, application session `clipboard.json`, persistence/range reports,
window JSON, selection/editor/clipboard/final PNGs and WB3 document fixtures. Window
JSON must report rectangular_clipboard, rectangular_selection, direct_paste,
clipboard_focus and in_place_editing passed, with zero remaining Rust charge.
ClipboardChecks must show zero charge on both source/destination children. Verify
WB3 reopen of dollar scalar/SUM formulas and preserved rejected drafts, and explicit
WB1/WB2 rejection. Frozen hosted v1 remains unchanged.

Inspect destination E15:F16 highlighting, active F16, formula/values, action-scope
label, inactive/focused appearance and clipping. Routed-event window assertions are
not physical keyboard/trackpad acceptance. Run all five manual groups in the handoff
using the APFS binaries, with PIVOT_UI_SMOKE_REPORT unset and APFS document files.
Emphasize Cmd+C/V, rectangle expansion/exact-shape rejection, native relative versus
external literal formulas, mixed dollar SUM, overlapping paste, blanks, native copy
between two application processes and editor text insertion. Native clipboard uses
UTI `org.pivot.cell-block-v1` on macOS plus ordinary text. Verify actual cross-process
transport, not only application-local data. Record exact key mappings and outcomes;
unperformed manual checks remain pending. Do not infer clipboard retention after
provider process exit, IME or third-party clipboard-manager compatibility.

Copy reports back only after validation to SMB
`artifacts/macos-validation-<UTC timestamp>`. Write REPORT.md with provenance,
commands/exits, test counts, every failure/rerun, visual/manual results and cleanup.
Verify SHA-256 of every returned file and report links. Do not start another feature.
Retain prior WSLg native abort, Windows-mounted-path installation uncertainty and
historical Linux viewport p95 exception as separate unresolved limitations.
