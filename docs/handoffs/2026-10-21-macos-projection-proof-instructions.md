# Native macOS/APFS — ADR-0019 bounded projection proof validation

Completed on 2026-10-21. [Reviewed results and three-platform handoff](2026-10-21-structured-results-projection-implementation.md#macosapfs-review--three-platform-validation-complete).
These instructions are retained as validation provenance; no repeat run is requested.

Validate the current dirty/untracked tree, not HEAD. Read the
[implementation handoff](2026-10-21-structured-results-projection-implementation.md)
and [approved boundary](2026-10-21-structured-results-projection-design-review.md).
This is validation only. Do not modify product code, dependencies, tests, assertions
or limits to obtain passes; do not commit, edit shared docs or start another feature.
Report exact blockers and retain failed logs. No routine manual replay is required.

## Native local source and output

Locate the SMB share (previously `/Volumes/RobMac/coding/pivot`). Copy source to a
fresh internal APFS directory. Source, Cargo target, .NET outputs, native publish,
temporary files, document fixtures and initial reports must all be on APFS, never
SMB. Verify the actual volume with `diskutil`/`df`. Include dirty and untracked source.

```sh
SHARE=/Volumes/RobMac/coding/pivot
WORK=$(mktemp -d /private/tmp/pivot-projections-XXXXXX)
mkdir -p "$WORK/source" "$WORK/reports" "$WORK/tmp" "$WORK/files"
rsync -a --exclude=.git/ --exclude=artifacts/ --exclude=target/ --exclude=bin/ --exclude=obj/ --exclude=__pycache__/ --exclude=.vscode/ "$SHARE/" "$WORK/source/"
export PYTHONUTF8=1 CARGO_TARGET_DIR="$WORK/target" TMPDIR="$WORK/tmp/"
export PIVOT_FILE_TEST_ROOT="$WORK/files" REPORTS="$WORK/reports"
cp "$SHARE/artifacts/projection-proof/build-inputs.json" "$REPORTS/"
cd "$WORK/source"
```

Record source HEAD/status/diff, copy command, OS/architecture, SDK/compiler versions,
APFS evidence and native ARM64 execution (no Rosetta). Compare copied source hashes
against the share before and after testing. Separately verify all **190** entries in
`build-inputs.json` against both local source and SMB. In particular include
projection.rs, projections.rs, ProjectionMap.cs, ProjectionProof.cs,
ProjectionAutomation.cs, ProjectionChecks.cs and the modified application proto.
The manifest also retains all previous worksheet-text build/test inputs, including
NativeAot.pubxml, app.manifest and fixtures. Preserve the local source/build tree.

## Gates

Run sequentially from the APFS source root, retaining actual exits and complete logs:

```sh
python3 scripts/ci.py --desktop --output-dir "$REPORTS/managed-ci"
cargo clippy --locked --workspace --all-targets --all-features -- -D warnings
cargo test --locked --workspace --all-features
python3 scripts/verify_native_aot.py --output-dir "$REPORTS/native-aot"
```

All .NET build/publish output remains local APFS. The native gate must publish
osx-arm64 and execute the actual published desktop with adjacent Rust child and
PIVOT_SESSION_PATH unset. Require no unsuppressed warnings, execution_mode=native-aot,
correct executable paths and zero remaining charge in main and all three peer
reports. Managed CI must report managed execution. Record `file` and `lipo -archs`
for the published desktop and `file` for its child; retain binaries/symbols on APFS.
No JIT/ReadyToRun substitution.

Require all **nine** desktop fields in both managed and native reports:
`structured_projection`, `worksheet_text`, `arrow_edit`, `clipboard_checklist`,
`rectangular_clipboard`, `rectangular_selection`, `direct_paste`, `clipboard_focus`
and `in_place_editing`. Each must start `passed:`. Preserve projections.png and all
prior screenshots, three peer reports, lock records and WB4 fixtures.

The new projection automation forces the delayed L2 interaction after removal,
checks key/header selection rather than ordinal retargeting, readonly entry/paste,
ordinary A9 identity and logical copy/navigation across bands, numeric invocation,
Save refusal before file creation, actual Cancel close and experimental Discard.
The session suite separately checks coherent old/new workbook/result captures and
zero cleanup. Rust adds four bounded ownership/key/overflow/reflow tests.
Expected Rust counts are 319 default / 335 all-feature on macOS (Windows318/334).
If counts differ, report them and identify the reason; do not alter tests.

Inspect projections screenshots: read-only full-width bands, separate # numbering,
ordinary row geometry, experimental non-savable banner/footer, and header-only
selection (footer must not share the header outline). No manual keyboard or picker
certification is implied. The private application protocol is **3**; hosted v1,
WB4 and PIVOTCB2/cell-block-v2 remain unchanged. No WB5, permanent `#` grammar,
connectors or persisted projections are authorized.

Both runners use `~/Library/Caches/Pivot/desktop-clipboard.lock`. Do not run another
clipboard suite outside that convention or bypass a lock. Preserve timeout owner
records; verify owner and children stopped before removing a stale lock. Normal
manual replay is optional, not an acceptance blocker.

## Return evidence to SMB

Write reports to `artifacts/macos-validation-<UTC timestamp>` on the SMB share so the
Windows Codex can review them. Retain full logs, summaries, screenshots, fixtures,
source/build-input manifests and hash verification. Large native `publish/`
directories may be excluded from transfer; state that exclusion and preserve them
on APFS. Native summary.json already inventories their hashes/sizes.

Write REPORT.md with copy/APFS/ARM64 provenance, actual commands/exits, Rust counts,
all nine fields, peer checks, zero-charge receipts, warnings/failures/reruns and
native image paths, hashes, size/startup observations. Verify SHA-256 of every
returned file against its local original and include transfer verification. Preserve
the local APFS tree. Report targeted blockers without changing product code.
Retain prior WSLg allocator abort, Windows-mounted-path save uncertainty and historical
Linux viewport p95 exception as existing limits. Stop for parent implementation review.
