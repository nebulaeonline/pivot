# Native macOS validation — in-place worksheet editing

Validate the current working tree, including dirty/untracked files, against the
[in-place editing handoff](2026-10-20-in-place-editing.md). Do not alter product
code, dependencies, assertions, fixtures or resource caps to obtain passes. Retain
all failures and reruns; do not commit or update shared project documentation.

Locate the SMB Pivot root (previously `/Volumes/RobMac/coding/pivot`). Copy it with
`rsync -a` to a fresh internal APFS directory, excluding `.git/`, `artifacts/`,
`target/`, `bin/`, `obj/`, `__pycache__/` and `.vscode/`. Do not clone/archive HEAD.
Verify APFS using `diskutil info` and `df`. Source, Cargo target, .NET bin/obj,
temporary files, file-test root and initial reports must all be on APFS. Never
build or test .NET on SMB. Record copy command, HEAD/status/diff, hardware/OS/SDKs
and all copied source hashes, verifying against the share before/after testing.
Also compare the 168 entries of `artifacts/clipboard-focus-build-inputs.json` on the share
to the local copy; include the reference manifest and comparisons in reports.

Set PYTHONUTF8=1 and local CARGO_TARGET_DIR, TMPDIR and PIVOT_FILE_TEST_ROOT. Set
REPORTS to a local APFS report directory. From the copied root, run sequentially,
retaining full logs and actual exits:

```sh
python3 scripts/ci.py --output-dir "$REPORTS/ci"
cargo clippy --locked --workspace --all-targets --all-features -- -D warnings
cargo test --locked --workspace --all-features
python3 tests/application-session/ui_smoke.py --output-dir "$REPORTS/ui"
```

The window check must launch the actual Avalonia app/Rust child. Collect results,
rendered PNG, saved WB2 fixture, all application/persistence/range reports and CI
logs. Inspect the image for clipping/overlap/readability and the removed Commit
button. The `in_place_editing` and `clipboard_focus` results must pass. The native clipboard
self-check temporarily changes clipboard text and restores previous text where
available; non-text clipboard formats are not preserved by that test. Programmatic routed-input
checks do not certify actual pointer, clipboard, IME or native-dialog behavior.

Run the six manual checklist groups in the handoff from the tested APFS binaries,
with PIVOT_UI_SMOKE_REPORT unset. Record actual user-reported outcomes separately;
if interactive testing is unavailable, mark it pending. Pay particular attention to
native double-click/F2, Cmd+C/V/A and Copy/Paste buttons both on the grid and inside
each text editor, absence of spurious c/v/a insertion, blue focused versus gray
inactive outlines, the Blink caret option, text selection and paste, Enter/Tab/Escape, draft retention
on scroll/resize, unfinished-edit Save/close and Mac keyboard/input-method behavior.
Do not claim untested IMEs or automatically generalize previous manual passes.

Preserve known WSLg mounted-path installation uncertainty, startup SIGABRT and
historical viewport p95 exception. Native APFS success does not resolve them. No
performance/scaling sweeps or broader capacity claims are requested.

Copy reports back to SMB under `artifacts/macos-validation-<UTC timestamp>` only
after validation, with REPORT.md covering provenance, exact commands/exits, test
counts, all failures/reruns, UI/manual evidence, cleanup and limitations. Verify
SHA-256 of all transferred report files and local report links. Preserve the APFS
source/build tree for diagnosis. Do not begin a subsequent milestone.

The prior `macos-validation-20261020T142038Z` automated pass is not a pass for this
follow-up: its manual acceptance failed for clipboard shortcuts. Preserve that
report. Run from a fresh APFS copy and record a new manual outcome, rather than
editing the old report into a pass.
