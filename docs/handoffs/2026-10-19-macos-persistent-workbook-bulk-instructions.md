# macOS Codex instructions — persistent workbook and bulk validation

Validate Pivot's current persistent workbook and bulk foundation source on native macOS. You have no prior session context.
This is validation work: do not change product code, switch backends, increase limits,
update dependencies or modify the SMB source. Report failures with evidence. Read
AGENTS.md if present, plus these project records in the local copy:

- `docs/handoffs/2026-10-19-persistent-workbook-bulk.md`
- `docs/architecture/persistent-workbook-bulk.md`
- `docs/performance/persistent-workbook-bulk.md`
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
The bulk benchmark also writes its own core/workbook/harness source manifest. Do not let other agents
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
run_gate fallback-tests cargo test --locked -p pivot-core --no-default-features
run_gate fallback-lint cargo clippy --locked -p pivot-core --all-targets --no-default-features -- -D warnings
run_gate research-only-lint cargo clippy --locked -p pivot-core --all-targets \
  --no-default-features --features scaling-experiment -- -D warnings
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

The current combined feature regression has 296 Windows / 297 Linux tests; fallback
has 21 on each. Derive native counts from actual logs. Also run these new gates:

```sh
run_gate bulk-all-features-lint cargo clippy --locked --workspace --all-targets --all-features -- -D warnings
run_gate bulk-all-features-tests cargo test --locked --workspace --all-features
run_gate persistent-workbook-only cargo test --locked -p pivot-workbook --features persistent-core
```

Default CI still exercises the reference Workbook; the additional feature gates
exercise persistent Workbook and the exact shared formula/projection suites. Keep
both. The standalone default remains persistent. No downstream default changes.

## Matched Release experiments

Wait until all correctness builds/tests finish. Run sequentially, without other
builds or benchmarks. Preserve every source manifest, environment record, CSV,
summary, build log, child exit status and guard stop.

```sh
run_gate bulk-release python3 scripts/bulk_workbook.py --output-dir "$REPORT/bulk-release"
run_gate long-chain-release python3 scripts/bulk_workbook.py --shapes long-chain \
  --layers new,existing --implementations reference,batch --samples 1 \
  --output-dir "$REPORT/long-chain-release"
```

The main matrix has 96 isolated children: 1K/10K/100K, independent/mixed/1,000-chain/
fan-out, core new/existing with reference/individual/batch, and workbook reference/
persistent. Construction has three samples; edit rows have 31 samples after warmup.
The long-chain diagnostic has 12 children, including full 100K selected closures;
it is not subject to the 1,000-chain interactive target. Check every final domain
charge is zero. Large batches should dispatch once; small edit rows use mutation
workers. The workbook reference experiment checks final quotas once for monotonic
fixtures; production reference transactions still check each edit.

If the 100K matrix completes safely, run the bounded core-only diagnostic:

```sh
run_gate numeric-200k python3 scripts/bulk_workbook.py --sizes 200000 \
  --shapes independent --layers new --implementations batch --samples 1 \
  --output-dir "$REPORT/numeric-200k"
```

This does not change product workbook limits or claim million-cell support. Report
initial population separately from workbook post-addition population (N+33,300).
Keep ADD/UPDATE/FORMULAS rows, retained generations, S/T checks, capture/evaluation,
allocation peaks, lane statistics, root enqueue and cleanup/drain timings.

The RSS helper supports Windows/Linux, **not macOS**: zero means unavailable. Its
RSS guard offers no native protection. Monitor native memory separately and label
sampled observations versus process high-water measurements. Keep the 600-second
child timeout; stop and report incomplete if native memory pressure is unsafe.
Do not alter code/fixtures/caps to make a run pass. Ordinary Release counting
allocator settings are intentional. Existing provisional 100K p95 targets remain
input/capture <5 ms, short formula/capture/evaluation <10 ms and update plus
1,000-formula chain <25 ms. Full long-chain results are separate diagnostics.

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
and source evidence, nested full CI artifacts and all 96 main-matrix CSVs and 12 long-chain CSVs plus
benchmark summary/manifests. Compare copied report hashes or rsync dry-run output to
confirm transfer completed. Do not delete local evidence, commit anything, or alter
project status/docs on the share. The Windows session will review results and update
project documentation. Finish by reporting the exact SMB results path and outcome.
