# Final native macOS/APFS — WB5-only ADR-0021 candidate

Completed: `macos-validation-20261023T090419Z`, all four gates passed against the
WB5-only source. Retained as run provenance; no repeat requested.
See the [reviewed evidence](2026-10-23-wb5-only-aggregate-correction.md#completed-macos-validation).

Validate the current dirty/untracked tree, not HEAD. Read the
[implementation/evidence](2026-10-23-wb5-only-aggregate-correction.md) and
[approved boundary](2026-10-22-spreadsheet-aggregate-implementation-review.md).
This is final ADR-0021 validation after WB4 reader removal, not a new feature.
The earlier aggregate Mac run validates only the previous WB4-reading source. Validation only:
do not change product code, dependencies, assertions, tests or limits to obtain
passes; do not commit, update shared docs or start another feature. Preserve failed
logs and report blockers. No recurring manual keyboard/picker replay is required.

## Copy everything needed to native APFS

Locate the SMB share (previously `/Volumes/RobMac/coding/pivot`). Source, Cargo target,
.NET outputs, native publish, temporary files, fixtures and initial reports must all
reside on local native APFS. Never build/test .NET from SMB. Include dirty and
untracked files, including the new Rust/C# sources, rational corpus/generator and
current-format document tests. Copying HEAD alone is insufficient.

```sh
SHARE=/Volumes/RobMac/coding/pivot
WORK=$(mktemp -d /private/tmp/pivot-wb5-only-XXXXXX)
mkdir -p "$WORK/source" "$WORK/reports" "$WORK/tmp" "$WORK/files"
rsync -a --exclude=.git/ --exclude=.agents/ --exclude=.codex/ --exclude=.aws/ --exclude=artifacts/ --exclude=target/ --exclude=bin/ --exclude=obj/ --exclude=__pycache__/ --exclude=.vscode/ "$SHARE/" "$WORK/source/"
export PYTHONUTF8=1 CARGO_TARGET_DIR="$WORK/target" TMPDIR="$WORK/tmp/"
export PIVOT_FILE_TEST_ROOT="$WORK/files" REPORTS="$WORK/reports"
cp "$SHARE/artifacts/aggregate-wb5-only/build-inputs.json" "$REPORTS/"
cp "$SHARE/artifacts/aggregate-wb5-only/source-manifest.json" "$REPORTS/"
cd "$WORK/source"
```

Record actual APFS evidence (`diskutil`/`df`), native ARM64/no Rosetta, OS/SDK/compiler
versions, source HEAD/status/diff and copy commands. Verify both expected manifests
against copied local source and the share before and after testing. **206 build/test
inputs** retain prior coverage plus the new implementation/corpus/fixtures. Verify
all 407 source-manifest entries and record the actual count. Do not regenerate expected
manifests to hide a mismatch. Preserve the local source/build tree.

## Gates

Run sequentially, retaining complete logs and actual exits:

```sh
python3 scripts/ci.py --desktop --output-dir "$REPORTS/managed-ci"
cargo clippy --locked --workspace --all-targets --all-features -- -D warnings
cargo test --locked --workspace --all-features
python3 scripts/verify_native_aot.py --output-dir "$REPORTS/native-aot"
```

Expected Rust counts: **340 default / 356 all-feature** on Mac (Windows339/355).
Report and explain differences; do not edit tests. Managed CI now includes the
independent Fraction-corpus reproducibility gate: 544 cases. Actual Rust unit tests
check that corpus plus maximum points and extrema/cancellation; workbook/session
checks exercise Blank/zero, strict SUM, numeric reducers, WB1–WB4 rejection and WB5.

Native publication must produce and execute osx-arm64, with adjacent Rust child,
PIVOT_SESSION_PATH unset and correct executable discovery. Require no unsuppressed
publish warnings, execution_mode=native-aot, and zero remaining charge in the main
and all three peers. Managed reports must say managed. Retain `file` and `lipo -archs`
evidence for the native image and `file` for its child. Binaries/symbols remain on
APFS; no managed/ReadyToRun substitution.

Require all **eleven** desktop fields to begin `passed:` in managed and native reports:
`spreadsheet_aggregates`, `worksheet_binding`, `structured_projection`, `worksheet_text`,
`arrow_edit`, `clipboard_checklist`, `rectangular_clipboard`, `rectangular_selection`,
`direct_paste`, `clipboard_focus`, `in_place_editing`. Retain all screenshots, peers,
locks and generated fixtures. Both runners share
`~/Library/Caches/Pivot/desktop-clipboard.lock`; do not bypass/remove locks or run
another clipboard suite concurrently. Preserve owner records for any timeout.

Inspect `results.aggregates.png`: ordinary grid with all four functions, exact mean
visible within the existing bounded cell display, selected AVERAGE source, translated
copy mean 15 and explicit no-values/strict-SUM errors. Preserve binding/projection/
text/selection/clipboard screenshots too. These tests use routed UI events; no physical
keyboard or native-picker certification is implied.

Read/write supports **PIVOTWB5 only**; WB1–WB4 explicitly reject.
The Native AOT fixture check now requires WB5 and its valid header digest. Preserve
ordinary SUM, Blank/zero, stable identity, checkpoint and failed-Open atomicity tests.
No WB4 migration fixture or obsolete decoder remains in source. PIVOTCB2, private application v4 and hosted/model-database formats
remain unchanged. No permanent structured-data feature is authorized.

## Return results to SMB

Copy reports to `$SHARE/artifacts/macos-validation-<UTC timestamp>` so Windows Codex
can inspect them. Return full logs, summaries, screenshots, generated document
fixtures, both expected manifests, before/after verification, lock and peer evidence.
Large native `publish/` trees may be excluded; state that exclusion and retain them
on APFS. Native summary already inventories binary hashes/sizes.

Write REPORT.md with APFS/ARM64/copy provenance, actual commands/exits, Rust counts,
544-case oracle check, eleven fields, peer checks, zero-charge receipts, warnings,
failures/reruns, native image paths/sizes/startup observations and hash verification.
Verify SHA-256 for every returned evidence file against its local original and
retain transfer verification. Preserve the native source/build tree.

The Windows ClipboardChecklistAsync intermittent null destination remains unresolved;
do not reclassify it based on a successful Mac run. If it recurs, retain enough
clipboard availability/UI sequencing/commit/readback detail to investigate. Keep
WSLg startup abort, mounted-Windows-filesystem Save uncertainty and historical Linux
viewport p95 exception separate. Stop for parent implementation review.
