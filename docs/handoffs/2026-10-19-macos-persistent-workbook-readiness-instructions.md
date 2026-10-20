# macOS Codex instructions — persistent workbook readiness validation

Validate Pivot's current persistent workbook readiness source on native macOS. You have no prior session context.
This is validation work: do not change product code, switch backends, increase limits,
update dependencies or modify the SMB source. Report failures with evidence. Read
AGENTS.md if present, plus these project records in the local copy:

- `docs/handoffs/2026-10-19-persistent-workbook-readiness.md`
- `docs/architecture/persistent-workbook-readiness.md`
- `docs/performance/persistent-workbook-readiness.md`
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

The foundation's prior native run passed 297 all-feature tests. This readiness
slice adds three canonical index tests and one workbook lifecycle test; expected
all-feature counts are 300 Windows / 301 Unix, default 251/252, fallback 21 each and
workbook-only 49. Derive actual counts from logs. Add these gates to the legacy
configurations above (which do not themselves enable persistent Workbook):

```sh
run_gate readiness-all-features-lint cargo clippy --locked --workspace --all-targets --all-features -- -D warnings
run_gate readiness-all-features-tests cargo test --locked --workspace --all-features
run_gate persistent-workbook-only cargo test --locked -p pivot-workbook --features persistent-core
```

## Matched Release experiments

Wait until every correctness/build process exits. Run sequentially without other
benchmarks, including the two profiling matrices. Both policies use the same binary
and validated graph; immutable is a research-only old insertion control, not another
backend. Preserve all source/environment manifests and failed logs too.

```sh
run_gate readiness-control python3 scripts/bulk_workbook.py --example readiness \
  --layers workbook --index-policy immutable --output-dir "$REPORT/readiness-control"
run_gate readiness-private python3 scripts/bulk_workbook.py --example readiness \
  --layers workbook --output-dir "$REPORT/readiness-private"
run_gate readiness-profile-control python3 scripts/bulk_workbook.py --example readiness \
  --layers workbook --implementations persistent --index-policy immutable --profile \
  --output-dir "$REPORT/readiness-profile-control"
run_gate readiness-profile-private python3 scripts/bulk_workbook.py --example readiness \
  --layers workbook --implementations persistent --profile \
  --output-dir "$REPORT/readiness-profile-private"
run_gate readiness-core python3 scripts/bulk_workbook.py --example readiness --layers new \
  --implementations batch --shapes independent,fanout --output-dir "$REPORT/readiness-core"
```

Expected children: 24 + 24 + 12 + 12 + 6 = 78, three construction samples each.
Profile rows are exclusive thread-wall and allocator traffic, not CPU time or normal
latency. Caller dispatch wait overlaps worker work; do not sum it with worker phases.
The non-profiled matrices establish medians/ranges and 31-sample interactive p95.
Report initial N separately from post-addition N+33,300 workbook populations. Preserve
ADD/UPDATE/FORMULAS, S/T/retention assertions, queue/cleanup and final zero charges.
Compare optimized reference end-of-batch quota coordinator honestly.

The runner now collects completed-child CPU and OS-reported RSS high-water with wait4.
On macOS ru_maxrss is bytes; confirm nonzero values against native tooling if results
look implausible and preserve raw evidence. These peaks include the complete child,
not just construction. The live RSS helper still lacks macOS support, so its zero
values mean unavailable and its live guard offers no protection. Retain separate
native memory-pressure monitoring and terminate the benchmark process group on
critical pressure. The driver creates a separate session for each Unix child; the
external monitor must terminate those owned benchmark child groups as well as the
driver, not merely signal the parent Python process. Report warnings and sampled evidence explicitly; the accepted
foundation run observed warning-level pressure. Also compare completed-child high-water with the configured RSS budget; record a
post-exit exceedance even if a periodic live sample missed it. Do not change fixtures/caps to pass.
Keep 600-second timeout and 8-GiB configuration; fail/incomplete if safe execution
cannot continue. Keep native process/CPU/thermal/AC/swap evidence before/after.

Check existing 100K p95 gates: input/capture <5 ms, short formula/capture/evaluation
<10 ms, update plus 1,000-chain <25 ms; proposed bulk budgets: core100K numeric1 s,
core100K fan-out2 s, workbook100K5 s, numeric10K addition500 ms, formula10K replacement
500 ms. Three construction medians are not p95 or product SLAs. Full-chain/200K
foundation diagnostics are historical; do not repeat broad viability sweeps.

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
and source evidence, nested full CI artifacts, all 78 readiness CSVs and five benchmark summaries with
their source/environment manifests. Compare copied report hashes or rsync dry-run output to
confirm transfer completed. Do not delete local evidence, commit anything, or alter
project status/docs on the share. The Windows session will review results and update
project documentation. Finish by reporting the exact SMB results path and outcome.
