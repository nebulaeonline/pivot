# Native macOS/APFS — managed regression and Native AOT baseline

Read the [implementation/Windows-Linux evidence](2026-10-20-native-aot-baseline.md)
and [bounded assignment](2026-10-20-native-aot-next-scope.md).
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
all **178** entries of `artifacts/native-aot-build-inputs.json`; return the reference
manifest and verification. Preserve the APFS source/build/publish tree for diagnosis.
Confirm native ARM64 execution and available Xcode command-line tools/clang.

Set PYTHONUTF8=1 and local CARGO_TARGET_DIR, TMPDIR, PIVOT_FILE_TEST_ROOT and REPORTS.
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
the published executable path, successful seven-field acceptance and zero charge.
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
warnings, change tests or redesign the framework. No installer, text values, dynamic
projections, connector or graph work. Native publish dependencies may differ from
Windows/Linux; document missing prerequisites instead of treating a managed pass as AOT.

Return reports only to SMB `artifacts/macos-validation-<UTC timestamp>`. Keep the
published native binaries/symbols on APFS; report transfer may exclude `publish/`
directories because summary.json already inventories every file size and SHA-256.
State that exclusion explicitly, and retain all logs, summaries, screenshots, source
manifests, lock records and WB3 fixtures. Write REPORT.md with provenance, gate exits,
counts, Native AOT image/mode proof, size/startup observations, warnings, all failures/
reruns and cleanup. Verify hashes of every returned report and its links. Preserve
the local native executables for optional inspection; no manual launch is required.

Retain the historical WSLg native abort, Windows-mounted-path save uncertainty and
Linux viewport p95 exception. Stop for cross-platform parent review after reporting.
