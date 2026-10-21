# Native macOS validation — rectangular clipboard and WB3

Read the [implementation/optional inspection checklist](2026-10-20-rectangular-clipboard.md)
and [ADR-0017](../adr/0017-rectangular-clipboard-and-formula-copy.md).
Validate the current working tree, including dirty/untracked files, not HEAD.
Do not change product code, dependencies, assertions, fixtures or caps to obtain
passes. Retain every failure/rerun. Do not commit or edit shared project docs.

Locate the SMB Pivot root (previously `/Volumes/RobMac/coding/pivot`). Copy with
`rsync -a` into a fresh internal APFS directory, excluding `.git/`, `artifacts/`,
`target/`, `bin/`, `obj/`, `__pycache__/` and `.vscode/`. Include new/untracked Rust
clipboard module/tests, C# ClipboardChecks, ClipboardAutomation.cs and the frozen WB2 fixture. Verify native
APFS with `diskutil info` and `df`. Source, Cargo target, .NET bin/obj, temporary
files, document fixtures and initial reports must stay on APFS. Never build or test
.NET on SMB. Preserve the APFS source/build directory for diagnosis.

Record share HEAD/status/diff, copy command, OS/hardware/SDKs and all copied source
hashes; compare local/share before and after testing. Separately copy and verify
all **173** entries from `artifacts/desktop-automation-build-inputs.json`, returning
that reference manifest and verification results. Do not silently use stale source.

Set PYTHONUTF8=1, local CARGO_TARGET_DIR, TMPDIR, PIVOT_FILE_TEST_ROOT and REPORTS.
Run sequentially from the APFS source root; retain complete logs and actual exits:

```sh
python3 scripts/ci.py --desktop --output-dir "$REPORTS/ci"
cargo clippy --locked --workspace --all-targets --all-features -- -D warnings
cargo test --locked --workspace --all-features
```

Collect the full CI, application session `clipboard.json`, persistence/range reports,
window JSON, selection/editor/clipboard/final PNGs and WB3 document fixtures. Window
JSON must report rectangular_clipboard, rectangular_selection, direct_paste,
clipboard_focus and in_place_editing passed, with zero remaining Rust charge.
ClipboardChecks must show zero charge on both source/destination children. Verify
WB3 reopen of dollar scalar/SUM formulas and preserved rejected drafts, and explicit
WB1/WB2 rejection. Frozen hosted v1 remains unchanged.

Desktop checks now run inside CI under `ci/<timestamp>/desktop-ui`. Require the
clipboard_checklist and arrow_edit fields plus all three peer reports
(`results.clipboard-save.json`, `results.discard.json`, `results.clean.json`) to pass.
The clipboard peer must confirm private UTI receipt in a separate GUI process and
Cancel/Save; all peers must show zero Rust charge. Parent tests reopen Save/Discard
files to verify the installed generation. Collect all screenshots and reports.

The [automation handoff](2026-10-20-desktop-acceptance-automation.md) replaces the
routine five-group manual gate at the owner's request. No repeated manual run is
required. Optional visual inspection can use saved screenshots or an interactive
APFS launch. Distinguish routed key/button events from OS-generated physical input.
Native picker/IME/accessibility combinations are targeted spot checks when relevant,
not an inferred automated pass. Keep any actual user feedback separately.
Native clipboard uses UTI `org.pivot.cell-block-v1` plus text. Do not infer clipboard
retention after provider exit or clipboard-manager compatibility. Do not run this
suite concurrently with another suite using the same system clipboard.

Copy reports back only after validation to SMB
`artifacts/macos-validation-<UTC timestamp>`. Write REPORT.md with provenance,
commands/exits, test counts, every failure/rerun, visual/manual results and cleanup.
Verify SHA-256 of every returned file and report links. Do not start another feature.
Retain prior WSLg native abort, Windows-mounted-path installation uncertainty and
historical Linux viewport p95 exception as separate unresolved limitations.
