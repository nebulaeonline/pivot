# Native macOS/APFS — worksheet-text managed and Native AOT validation

Read the [worksheet-text implementation](2026-10-21-worksheet-text-implementation.md)
and [accepted ADR-0018](../adr/0018-document-owned-worksheet-text.md).
Validate the current dirty/untracked tree, not just HEAD. Do not modify product
code, dependencies, assertions, fixtures or caps to obtain passes, suppress warnings,
commit, edit shared project docs or start another feature. Report blockers precisely.

Locate the SMB root (previously `/Volumes/RobMac/coding/pivot`). Rsync all source into
a fresh internal APFS directory, excluding `.git/`, `artifacts/`, `target/`, `bin/`,
`obj/`, `.vscode/` and `__pycache__/`. Include AutomationReports.cs, NativeAot.pubxml,
verify_native_aot.py, desktop_test_lock.py and test_desktop_lock.py. Verify APFS with
diskutil/df. Source, Cargo target, .NET outputs, native publish directory, temporary
files, document fixtures and initial reports must all be on APFS, never SMB.

Record source HEAD/status/diff, copy command, OS/architecture and SDKs. Compare all
copied source hashes with the share before/after testing. Separately copy and verify
all **183** entries of `artifacts/worksheet-text-build-inputs.json`; return the reference
manifest and verification. Preserve the APFS source/build/publish tree for diagnosis.
Confirm native ARM64 execution and available Xcode command-line tools/clang.

For example (adjust SHARE if the mount differs), establish local paths before any
build. Record and verify the selected volume is APFS:

```sh
SHARE=/Volumes/RobMac/coding/pivot
WORK=$(mktemp -d /private/tmp/pivot-text-XXXXXX)
mkdir -p "$WORK/source" "$WORK/reports" "$WORK/tmp" "$WORK/files"
rsync -a --exclude=.git/ --exclude=artifacts/ --exclude=target/ --exclude=bin/ --exclude=obj/ --exclude=__pycache__/ --exclude=.vscode/ "$SHARE/" "$WORK/source/"
export PYTHONUTF8=1 CARGO_TARGET_DIR="$WORK/target" TMPDIR="$WORK/tmp/"
export PIVOT_FILE_TEST_ROOT="$WORK/files" REPORTS="$WORK/reports"
cp "$SHARE/artifacts/worksheet-text-build-inputs.json" "$REPORTS/"
cd "$WORK/source"
```

From the APFS source root, run sequentially, retaining actual exits and complete logs:

```sh
python3 scripts/ci.py --desktop --output-dir "$REPORTS/managed-ci"
cargo clippy --locked --workspace --all-targets --all-features -- -D warnings
cargo test --locked --workspace --all-features
python3 scripts/verify_native_aot.py --output-dir "$REPORTS/native-aot"
```

The separate gate must publish osx-arm64 via NativeAot, with no unsuppressed warnings,
then execute the actual published image. It builds/copies the adjacent Rust child
and unsets PIVOT_SESSION_PATH. Main and all peers must report execution_mode=native-aot,
the published executable path, successful eight-field acceptance (including
`worksheet_text`) and zero charge.
The source Rust engine must close before cross-process private UTI clipboard receipt;
Cancel/Save, Discard and clean peers must pass, with reopened generations verified.
Managed CI must continue reporting managed execution. No JIT/R2R substitution.

The gate records compiler/SDK versions, warnings, hashes, sizes including/excluding
symbols, and Main-to-first-ready observations. These are diagnostic observations,
not benchmarks. Inspect screenshots for rendered controls, selection/editor feedback
and clipboard results. Routine manual replay is unnecessary. Physical keys, IME,
accessibility and native pickers remain targeted spot checks, not automated claims.

Both runners now enforce a shared clipboard-suite lock. Native Mac uses
`~/Library/Caches/Pivot/desktop-clipboard.lock`. Do not run another clipboard suite
outside this locking convention during validation. If a lock times out, preserve its
owner record and verify the owner/children have stopped before removing a stale lock.
Do not weaken or disable the lock to pass.

On failure, retain the exact command/warning/error and affected path; do not suppress
warnings, change tests or redesign the framework. Do not modify product code, dependencies, tests, assertions or capacity limits to
repair a failure. No installer, computational strings, dynamic projections or connectors. Native publish dependencies may differ from
Windows/Linux; document missing prerequisites instead of treating a managed pass as AOT.

Return reports only to SMB `artifacts/macos-validation-<UTC timestamp>`. Keep the
published native binaries/symbols on APFS; report transfer may exclude `publish/`
directories because summary.json already inventories every file size and SHA-256.
State that exclusion explicitly, and retain all logs, summaries, screenshots, source
manifests, lock records and WB4 fixtures. Write REPORT.md with provenance, gate exits,
counts, Native AOT image/mode proof, size/startup observations, warnings, all failures/
reruns and cleanup. Verify hashes of every returned report and its links. Preserve
the local native executables for optional inspection; no manual launch is required.

Retain the historical WSLg native abort, Windows-mounted-path save uncertainty and
Linux viewport p95 exception. Stop for cross-platform parent review after reporting.

## Worksheet-text-specific checks

Read [implementation/scope](2026-10-21-worksheet-text-implementation.md) and ADR-0018.
Copy the dirty/untracked entry.rs, text.rs, TextChecks.cs and TextAutomation.cs as
well as all other current files. Build .NET from local APFS; never execute tests
from SMB. Confirm the private application version is 2, native clipboard is
PIVOTCB2/cell-block-v2 and saved documents have PIVOTWB4 headers.

Require application-session/text.json passed with zero charge and the desktop
worksheet_text field passed in both managed and native reports. Inspect the native
text screenshot. The mixed clipboard peer must receive literal text and the
coordinator must independently reopen Save/Discard generations with their expected
text values. Preserve all three peer reports, lock records and text.png alongside
the existing screenshots and fixture checks. Old WB1/WB2/WB3 Open tests must preserve
the dirty document. Native publish binaries stay on APFS; transfer all reports and
fixture files back to SMB artifacts/macos-validation-<UTC timestamp>.

The escaped-source boundary is explicit in the handoff: 4,096-byte stored text
payload, with one possible extra syntax apostrophe; all transport bytes are counted
in aggregate budgets. Do not alter that policy during validation. Record failures
or mismatches and stop for diagnosis. No new recurring manual test gate.

Retain `file` output for the native published desktop and adjacent Rust child,
`lipo -archs` for the desktop, and Rosetta/native execution checks. Both images must
be ARM64. Record exact published image paths and independent hash verification of
the returned files. The existing Native AOT summary inventories the binaries; do
not copy large publish/symbol directories onto SMB merely to rerun there.
