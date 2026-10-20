# Native macOS validation — rectangular UI selection

Read the [interaction specification/handoff](2026-10-20-rectangular-selection.md).
Validate the current working tree, including dirty/untracked files, not HEAD.
Do not change product code, dependencies, assertions, fixtures or caps to obtain
passes. Retain all failures/reruns; do not commit or edit shared project docs.

Locate the SMB Pivot root (previously `/Volumes/RobMac/coding/pivot`). Use `rsync -a`
to copy into a fresh internal APFS directory, excluding `.git/`, `artifacts/`,
`target/`, `bin/`, `obj/`, `__pycache__/` and `.vscode/`. Verify native APFS with
`diskutil info` and `df`. Source, Cargo target, .NET bin/obj, temporary files, test
files and initial reports must all be on APFS. Never build/test .NET from SMB.
Record HEAD/status/diff, copy command, OS/hardware/SDKs and all source hashes; compare
local/share before/after testing. Also verify the 168 entries in the share's
`artifacts/rectangular-selection-build-inputs.json` and return that reference manifest.

Set PYTHONUTF8=1 and local CARGO_TARGET_DIR, TMPDIR, PIVOT_FILE_TEST_ROOT and REPORTS.
Run sequentially from the local copied root, retaining logs and actual exits:

```sh
python3 scripts/ci.py --output-dir "$REPORTS/ci"
cargo clippy --locked --workspace --all-targets --all-features -- -D warnings
cargo test --locked --workspace --all-features
python3 tests/application-session/ui_smoke.py --output-dir "$REPORTS/ui"
```

Require rectangular_selection, direct_paste, clipboard_focus and in_place_editing
results to pass in the actual Avalonia window/Rust child. The pointer tests use
routed events in window coordinates, real capture and auto-scroll timers; they do
not substitute for physical mouse/trackpad/key checks. Collect the selection PNG,
active-editor/final PNGs, WB2 fixture and all CI/session/persistence/range reports.
Inspect active corner, region fill/boundary, gray inactive state, label and viewport
clipping. Clipboard checks temporarily replace text; prior text is restored where
available, but non-text clipboard formats are not preserved.

Run all five manual groups in the handoff from tested APFS binaries, with
PIVOT_UI_SMOKE_REPORT unset and native APFS document files. Emphasize drag outside
the window, Shift extension across the anchor, timer stop on release/capture/focus
loss, Escape cancellation, Fn/Home/End/Page mappings, editor transitions and quick
click-release during a pending edit commit. Pure selection must not dirty a fresh
workbook or prompt on close. Clipboard/Delete still affect only the active cell.
Record exact user-reported outcomes, keyboard/trackpad mappings and untested IMEs
separately. Missing manual coverage remains pending, not an automated pass.

Copy reports back only after validation to SMB `artifacts/macos-validation-<UTC timestamp>`.
Write REPORT.md with provenance, commands/exits, counts, all failures/reruns, visual/
manual outcomes and zero-charge orderly close. Verify SHA-256 of every returned file
and local report links. Preserve the APFS source/build tree for diagnosis. Do not
begin copy/fill, rectangular clipboard or another milestone. Retain historical WSLg
native abort, mounted-path uncertain install and viewport p95 exception separately.
