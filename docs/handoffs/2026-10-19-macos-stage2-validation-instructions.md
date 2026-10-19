# macOS Codex instructions — complete Stage 2 validation

Validate Pivot's current source on native macOS. You have no prior session context.
This is validation work: do not change product code, switch backends, increase limits,
update dependencies or modify the SMB source. Report failures with evidence. Read
AGENTS.md if present, plus these project records in the local copy:

- `docs/handoffs/2026-10-19-controlled-core-integration-stage2.md`
- `docs/architecture/controlled-core-integration-stage2.md`
- `docs/performance/controlled-core-integration-stage2.md`
- `docs/local-ci.md`

## Storage requirements

The mounted SMB share contains the source repository `pivot`. Determine its actual
mount path; do not assume a particular `/Volumes/...` name. Copy the current working
tree, including uncommitted/untracked source, to a fresh directory on a **local APFS
volume**. Do not use git clone as a substitute: it can omit the changes under review.

All compilation, .NET restore/build/test, Rust targets, temporary test files and
initial reports must be on local APFS. Prior .NET builds on SMB failed with file
sharing errors. Do not point CARGO_TARGET_DIR, TMPDIR or PIVOT_FILE_TEST_ROOT at SMB.
Copy reports back to:

```text
<SMB pivot>/artifacts/macos-validation-<UTC timestamp>/
```

Use one timestamp for the whole run. Preserve failed logs and partial reports too.
Do not copy build outputs or dependency caches back to SMB. Keep the APFS worktree
until results have been reviewed.

## Prepare the copy

Adapt the first two paths before running this zsh/bash-compatible setup. Verify with
`diskutil info` that LOCAL_BASE is local APFS, not an SMB mount or another network
filesystem. Use an ordinary disk-backed location, not a RAM disk.

```sh
SHARE='/Volumes/REPLACE_WITH_SHARE_PATH/pivot'
LOCAL_BASE="$HOME/Developer/pivot-validation"
STAMP="$(date -u +%Y%m%dT%H%M%SZ)"
WORK="$LOCAL_BASE/$STAMP"
REPORT="$WORK/reports"
SHARED_REPORT="$SHARE/artifacts/macos-validation-$STAMP"

mkdir -p "$LOCAL_BASE"
diskutil info "$LOCAL_BASE"
# Stop here if this is not a local APFS volume.
mkdir -p "$WORK/source" "$REPORT" "$WORK/tmp" "$WORK/file-tests" "$SHARED_REPORT"

# Capture the share's revision and dirty state; never stage/commit/reset it.
git -C "$SHARE" rev-parse HEAD > "$REPORT/source-head.txt"
git -C "$SHARE" status --short > "$REPORT/source-status.txt"
git -C "$SHARE" diff --binary > "$REPORT/source-working-tree.patch"

rsync -a --exclude='.git/' --exclude='artifacts/' --exclude='target/' \
  --exclude='bin/' --exclude='obj/' --exclude='__pycache__/' \
  "$SHARE/" "$WORK/source/"

export CARGO_TARGET_DIR="$WORK/target"
export TMPDIR="$WORK/tmp/"
export PIVOT_FILE_TEST_ROOT="$WORK/file-tests"
export PYTHONUTF8=1
cd "$WORK/source"

sync_reports() {
  rsync -a "$REPORT/" "$SHARED_REPORT/"
}
trap sync_reports EXIT
```

Check every setup command succeeded before continuing. The source may be dirty; that
is expected. Record any copy failures. Capture a SHA-256 manifest of copied source
files (relative paths; exclude build outputs) so the exact tested source is known.
The Stage 2 benchmark also writes its own source manifest. Do not let other agents
edit the APFS copy during the run. If the SMB source changes, retain the original
manifest and report the difference rather than silently mixing versions.

## Prerequisites and environment

Use native architecture tools, not Rosetta unless explicitly documenting a separate
run. Required: Rust with cargo/rustfmt/clippy, Python 3.9 or newer, and .NET SDK 10
compatible with `global.json` (10.0.100, latestFeature roll-forward, no prerelease).
Use the installed Rust nightly used by this repository; record its exact version.
Do not change manifests/lockfiles to accommodate an incompatible toolchain.

Record `sw_vers`, `uname -a`, `uname -m`, CPU/model/RAM, available disk space,
`diskutil info` for the local volume, `rustc -Vv`, `cargo -V`, `rustup show active-toolchain`,
`python3 --version`, `dotnet --info`, and the environment paths above under reports.
If required tools are missing or permissions block local IPC, report that clearly.
Request the necessary permission to run actual socket tests; do not skip/weaken them.

## Run all correctness and CI gates

Run sequentially. The full CI script includes default Rust formatting/lint/tests,
the .NET Release solution build, all 10,099 decimal parity vectors, generated protocol
parity (including 58 decimal layouts), and real daemon/CLI Unix-socket integration.
Do not replace IPC tests with mocks. Feature gates below additionally exercise the
persistent integration and research configurations. Keep full stdout/stderr and
actual exit codes; continue independent gates after a failure to provide useful
coverage, but the overall result must then be failed/incomplete.

```sh
run_gate() {
  gate="$1"
  shift
  "$@" > "$REPORT/$gate.log" 2>&1
  gate_code=$?
  printf '%s\t%s\n' "$gate" "$gate_code" >> "$REPORT/gate-exit-codes.tsv"
  sync_reports
  return 0
}

run_gate default-ci python3 scripts/ci.py --output-dir "$REPORT/ci"
run_gate integration-lint cargo clippy --locked --workspace --all-targets \
  --features pivot-core/core-integration -- -D warnings
run_gate research-lint cargo clippy --locked --workspace --all-targets \
  --features pivot-workbook/scaling-experiment -- -D warnings
run_gate combined-lint cargo clippy --locked --workspace --all-targets \
  --features pivot-core/core-integration,pivot-workbook/scaling-experiment -- -D warnings
run_gate integration-tests cargo test --locked --workspace \
  --features pivot-core/core-integration
run_gate research-tests cargo test --locked --workspace \
  --features pivot-workbook/scaling-experiment
run_gate combined-tests cargo test --locked --workspace \
  --features pivot-core/core-integration,pivot-workbook/scaling-experiment
```

`run_gate` deliberately returns success after recording the real status so subsequent
gates run. Inspect the TSV and nested CI summary; never interpret the wrapper's
return value as proof that tests passed. Run without shell `set -e`, which would
abort before recording a failed command. Preserve initial failures even if a later
rerun succeeds; explain the reason for any rerun or environmental adjustment.

For context, final Windows/Linux combined feature counts were 259/260, and default
counts were 209/210. Native Unix platform tests may affect counts; derive actual
counts from logs rather than forcing these numbers. Three feature lint configurations
must pass independently. No new native GUI/AOT validation is required by this slice.

## Run the current Release benchmark suite

After correctness gates complete and all build/test processes have exited, run:

```sh
run_gate stage2-release python3 scripts/integration_stage2.py \
  --sizes 1000,10000,100000 --samples 31 \
  --implementations reference,integrated \
  --cases edits,ports-wide,invalid-import,inspection,contention \
  --output-dir "$REPORT/stage2-release"
```

This executes thirty isolated processes, sequentially, through WorkingModel. Keep
ordinary Release settings and allocation instrumentation. Do not run old research
benchmark sweeps unless needed to investigate a failure. Do not run CI/builds or
other benchmarks concurrently. Avoid deliberate background workload and record
power/thermal conditions. The contention case supplies its own background work.

Check all thirty child exit codes, assertions, timeout/budget stops and final zero
domain charges. Preserve environment/source manifests, build logs, every CSV and
summary JSON. Review 1K/10K/100K local edits, ports, invalid and initially-invalid
repair, local capture, bounded inspection, complete snapshot/diagnosis, contention,
retained generations and reclamation. At 10K/100K portable transfer is intentionally
rejected by frozen limits; do not raise limits. Transfer throughput is measured at 1K.

The existing RSS helper in `scripts/scaling.py` supports Windows/Linux, **not macOS**.
The runner's macOS zero RSS values mean unavailable, not zero usage; its RSS guard
therefore provides no macOS protection. Retain allocation-domain/requested-heap
measurements and monitor native memory separately. If collecting `/usr/bin/time -l`
for an individual benchmark child, preserve its command and stderr separately and
label it as an additional run. Whole-run time/RSS includes compilation and is not
comparable to individual-child RSS. Do not edit shared source or claim unavailable
RSS evidence. Retain the 600-second per-child timeout and stop if native memory
pressure becomes unsafe; report an incomplete run instead of changing fixtures.

The existing 100K provisional p95 gates are input edit/capture <5 ms, local formula
edit/capture/selected evaluation <10 ms, and input update plus 1,000-formula chain
<25 ms. Broader proposed budgets: invalid repair <10 ms; bounded inspection <5 ms;
1,000-port work <10 ms; full snapshot/diagnosis <500 ms; activation <2 seconds;
admitted 1K transfer <100 ms. Report native results honestly if they differ. These
are review targets, not reasons to change tests or code during validation.

## Return evidence to the SMB share

Write `REPORT.md` and `summary.json` at the report root. Include:

- Tested share revision/dirty state, copy timestamp, source hashes, exact APFS paths
  and evidence that compilation/test outputs were local.
- Native hardware, OS, Rust/.NET/Python versions and relevant environment.
- Every command, exit status, actual test count, CI gate status, warnings and failures.
- Decimal/protocol/real IPC outcomes and any permission/environment issues.
- Benchmark process count, p95 comparisons at all sizes, gates, retained heap,
  allocations, cleanup work/final charges, and RSS availability limitations.
- Clear overall passed/failed/incomplete status, remaining gaps, and any source changes
  (normally none). Do not declare overall pass if a required gate did not execute.

```sh
sync_reports
printf 'Returned reports: %s\n' "$SHARED_REPORT"
```

Verify the SMB directory contains REPORT.md, summary.json, gate logs, environment
and source evidence, nested full CI artifacts and all thirty benchmark CSVs plus
benchmark summary/manifests. Compare copied report hashes or rsync dry-run output to
confirm transfer completed. Do not delete local evidence, commit anything, or alter
project status/docs on the share. The Windows session will review results and update
project documentation. Finish by reporting the exact SMB results path and outcome.
