# Native macOS validation — direct single-cell paste

Read the [focused handoff](2026-10-20-direct-single-cell-paste.md). Validate the current
dirty/untracked tree, not HEAD. Do not change product code, dependencies, fixtures,
assertions or caps to get passes. Preserve every failure/rerun; do not commit or
update shared project docs. The prior accepted editor run is historical evidence.

Locate SMB Pivot (previously `/Volumes/RobMac/coding/pivot`). Use `rsync -a` to copy
into a fresh internal APFS directory, excluding `.git/`, `artifacts/`, `target/`,
`bin/`, `obj/`, `__pycache__/` and `.vscode/`. Verify APFS with `diskutil info`/`df`.
Keep source, Cargo target, .NET bin/obj, TMPDIR, PIVOT_FILE_TEST_ROOT and initial
reports on APFS. Never build/test .NET on SMB. Record commands, HEAD/status/diff,
hardware/OS/SDKs and all source hashes before/after. Compare the 168 inputs from
`artifacts/direct-paste-build-inputs.json` on the share with the APFS copy and retain
the reference manifest/comparisons in reports.

Set PYTHONUTF8=1 and local CARGO_TARGET_DIR, TMPDIR and PIVOT_FILE_TEST_ROOT. REPORTS
must be local. Run sequentially, retaining full logs and actual exit codes:

```sh
python3 scripts/ci.py --output-dir "$REPORTS/ci"
cargo clippy --locked --workspace --all-targets --all-features -- -D warnings
cargo test --locked --workspace --all-features
python3 tests/application-session/ui_smoke.py --output-dir "$REPORTS/ui"
```

Require in_place_editing, clipboard_focus and direct_paste results to pass in the
actual native Avalonia window against Rust. Collect all CI/session/persistence/range
reports, PNGs and saved WB2 fixture; inspect images for readability. Clipboard tests
temporarily change native clipboard text, restoring prior text where available;
non-text formats are not preserved by that test. Controlled delayed-read tests are
not a substitute for physical Cmd-key or clipboard-manager testing.

From the tested APFS binaries with PIVOT_UI_SMOKE_REPORT unset, run the five manual
groups in the handoff. Specifically verify Cmd+V and Paste immediately commit a
selected cell, without opening editing or requiring Enter. Verify paste inside F2
and formula-bar editors remains text insertion. Preserve rejection/correction,
shortcut/focus, Save/Open and no-prompt untouched-close behavior. Record actual
user-confirmed outcomes separately. Missing manual/IME coverage remains explicit.

Copy reports only to SMB `artifacts/macos-validation-<UTC timestamp>` after the run.
Write REPORT.md with source/environment provenance, commands/exits, counts, all
failures/reruns, actual-window/manual outcomes and normal zero-charge cleanup.
Verify SHA-256 of every transferred report and local report links. Preserve APFS
source/build outputs for diagnosis. Keep WSL mounted-path uncertainty, historical
native abort and viewport p95 exception unresolved; no new scaling sweeps or broader
filesystem/deployment claims. Stop for parent review; do not start another feature.
