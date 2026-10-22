# Native macOS/APFS — ADR-0020 worksheet binding proof validation

Completed on 2026-10-22. [Reviewed results and three-platform handoff](2026-10-22-worksheet-structured-binding-implementation.md#macosapfs-review--three-platform-validation-complete).
These instructions are retained as validation provenance; no repeat run is requested.

Validate the current dirty/untracked tree, not HEAD. This is a new validation run,
not a repeat of accepted ADR-0019. Read the
[implementation handoff](2026-10-22-worksheet-structured-binding-implementation.md)
and [approved boundary](2026-10-22-structured-output-binding-contract-review.md).
Do not change product code, dependencies, tests, assertions or limits to obtain a
pass. Do not commit, update shared docs or start another feature. Preserve failed
logs and report blockers. Routine automation is the gate; no recurring manual
keyboard/picker replay is requested.

## Copy to native APFS

Locate the SMB share (previously `/Volumes/RobMac/coding/pivot`). Source, Cargo target,
.NET outputs, native publish, temporary files, document fixtures and initial reports
must all reside on local native APFS. Never build/test .NET from SMB. Include dirty
and untracked files; copying only HEAD would omit this milestone.

```sh
SHARE=/Volumes/RobMac/coding/pivot
WORK=$(mktemp -d /private/tmp/pivot-bindings-XXXXXX)
mkdir -p "$WORK/source" "$WORK/reports" "$WORK/tmp" "$WORK/files"
rsync -a --exclude=.git/ --exclude=.agents/ --exclude=.codex/ --exclude=.aws/ --exclude=artifacts/ --exclude=target/ --exclude=bin/ --exclude=obj/ --exclude=__pycache__/ --exclude=.vscode/ "$SHARE/" "$WORK/source/"
export PYTHONUTF8=1 CARGO_TARGET_DIR="$WORK/target" TMPDIR="$WORK/tmp/"
export PIVOT_FILE_TEST_ROOT="$WORK/files" REPORTS="$WORK/reports"
cp "$SHARE/artifacts/worksheet-bindings/build-inputs.json" "$REPORTS/"
cp "$SHARE/artifacts/worksheet-bindings/source-manifest.json" "$REPORTS/"
cd "$WORK/source"
```

Record actual APFS volume evidence with `diskutil`/`df`, native ARM64 (no Rosetta),
OS/SDK/compiler versions, source HEAD/status/diff and copy commands. Verify every
entry in both manifests against local source and the share before and after testing.
There are **199 build/test inputs**, retaining the previous 190 and adding the new
core/workbook/session/C# implementation and tests. Keep NativeAot.pubxml,
app.manifest, proto, fixtures and all new untracked files. Report differing counts
or hashes; do not regenerate the expected manifest to conceal differences. Preserve
the local source/build tree for investigation.

## Gates

Run sequentially from the APFS source root, recording complete logs and actual exits:

```sh
python3 scripts/ci.py --desktop --output-dir "$REPORTS/managed-ci"
cargo clippy --locked --workspace --all-targets --all-features -- -D warnings
cargo test --locked --workspace --all-features
python3 scripts/verify_native_aot.py --output-dir "$REPORTS/native-aot"
```

Require successful managed CI, warnings-as-errors build/lint, **329 default / 345
all-feature Rust tests** on macOS (Windows328/344). Identify any count difference;
do not alter tests. These include actual canonical source/arithmetic/SUM precedence,
context identity/lifetime, typed annotation validation, distinct retained allocations,
registry/slot retirement, all caps and the joint generation/formula budget.

The native gate must publish osx-arm64 and execute the actual published desktop
with the adjacent Rust child and PIVOT_SESSION_PATH unset. Require
execution_mode=native-aot, correct executable/child paths, no unsuppressed publish
warnings, and zero remaining charge in the main and all three peer reports. Managed
reports must say managed. Record `file` and `lipo -archs` for the native desktop and
`file` for the child. Preserve binaries/symbols locally on APFS; no JIT/ReadyToRun
substitution.

Require all **ten** desktop fields to begin `passed:` in managed and native reports:
`worksheet_binding`, `structured_projection`, `worksheet_text`, `arrow_edit`,
`clipboard_checklist`, `rectangular_clipboard`, `rectangular_selection`, `direct_paste`,
`clipboard_focus`, `in_place_editing`. Preserve all screenshots, peers, fixtures and
clipboard lock evidence. The new binding automation enters an actual inline formula,
checks publication without definition changes, rename/no-op editing, mixed A1
DuplicateBlock, unchanged clipboard on refused Copy, unavailable versus empty,
Save refusal before file creation and actual Discard/Cancel close.

Inspect `results.bindings.png`: experimental non-savable banner, private formula
entry, ordinary cell numeric result, current source labels/portable IDs and separate
read-only source bands. Also retain the existing projection/text/clipboard/selection
screenshots. This does not imply physical keyboard/native-picker certification.

The private child protocol is **4**; older versions reject. Hosted v1, WB4 and
PIVOTCB2 remain unchanged. There is no new persisted/native-clipboard binding format.
The only experimental source spelling is `@{Producer.Output}`; permanent `#` grammar,
connectors, refresh scheduling and permanent integration are excluded.

Both desktop runners use `~/Library/Caches/Pivot/desktop-clipboard.lock`. Do not bypass
that convention or run another clipboard suite concurrently. Preserve timeout owner
records; verify owners/children stopped before removing a stale lock.

## Return reports to SMB

Copy evidence back to `$SHARE/artifacts/macos-validation-<UTC timestamp>` so Windows
Codex can inspect it. Include full logs, summaries, screenshots, fixtures, both source
manifests and before/after hash verification. Large native `publish/` directories may
be excluded from transfer; state the exclusion and preserve them on APFS. Native
summary.json already inventories their sizes/hashes.

Write REPORT.md with copy/APFS/ARM64 provenance, exact commands/exits, actual Rust
counts, ten fields, peer results, zero-charge receipts, warnings/failures/reruns,
native image paths, sizes, startup observations and source/input verification. Hash
all returned files against their local originals and retain transfer verification.
Keep the local APFS tree. Retain the known WSLg allocator abort,
Windows-mounted-path save uncertainty and historical Linux viewport p95 exception
as separate existing limits. Stop for parent implementation review.
