# Native macOS canonical-engine consolidation validation

Validate the current dirty working tree from the SMB Pivot share. This is a new
source milestone: earlier readiness passes do not validate it. Do not implement
features, modify product code/fixtures/dependencies, stage or commit. If a gate
fails, retain evidence and report it; do not weaken tests. Read ADR-0013 and the
canonical-engine architecture/handoff in the local copy first.

## Source and storage

Locate the SMB Pivot root (previously `/Volumes/RobMac/coding/pivot`). Record its
actual path, UTC timestamp, Git HEAD, status and binary diff. Copy the complete
working tree, including uncommitted/untracked files and reflecting deleted files,
into a fresh unique directory on internal APFS. Do not clone/archive HEAD. Exclude
`.git`, `artifacts`, `target`, all `bin`/`obj`, `__pycache__` and `.vscode` directories.
Use `rsync -a` with those exclusions. Verify the destination device/filesystem with
`diskutil info` and device IDs before running anything. Source, Cargo targets,
.NET outputs, TMPDIR, file-test root and initial reports must all be local APFS;
no testing/building on SMB. Keep the APFS tree for investigation.

Create a SHA-256 manifest of every copied source file, compare to SMB before tests,
and compare local/shared sources again at completion. Record intentional doc changes
separately; no product source difference is expected. Capture tool/OS/CPU/RAM,
free space, power/thermal/swap and pressure conditions. Set PYTHONUTF8=1,
CARGO_TARGET_DIR, TMPDIR and PIVOT_FILE_TEST_ROOT inside the APFS worktree.

## Correctness gates

Run sequentially, preserving exact commands, stdout/stderr and real exit codes:

```sh
python3 scripts/ci.py --output-dir "$REPORTS/ci"
cargo clippy --locked --workspace --all-targets --all-features -- -D warnings
cargo test --locked --workspace --all-features
cargo test --locked -p pivot-core --no-default-features
```

The former `core-integration` and `persistent-core` features and reference constructor
are removed. No-default-features now uses the same canonical graph, not a fallback.
Default CI covers .NET Release, 10,099 decimal vectors, frozen protocol/portable
fixtures, real daemon/CLI/REPL, document Save/Open, authoring receipts/replay and
runtime cleanup. The renamed `canonical_probe` still checks the frozen C# codec.
All-feature coverage adds low-level graph/profiling tests. Do not infer counts from
old milestone numbers; report actual passed/failed/ignored counts for each gate.

## Release workloads

Only after all builds/tests and unrelated benchmark jobs finish, run these three
commands sequentially from the APFS copy (33 children total):

```sh
python3 scripts/bulk_workbook.py --layers new,workbook --output-dir "$REPORTS/canonical-main"
python3 scripts/bulk_workbook.py --layers existing --shapes independent,fanout --output-dir "$REPORTS/canonical-existing"
python3 scripts/bulk_workbook.py --layers small --sizes 1,8,32 --shapes independent --output-dir "$REPORTS/canonical-small"
```

No reference/immutable-index policy flags. Default core mutation policy is batch;
workbook is canonical persistent. Retain three construction samples and 31 edit
samples after three warmups; raw medians/min/max/p95, allocation traffic, requested
live peak, admitted-domain peak, OS wait4 CPU/high-water, retained generations,
root release, cleanup/drain, one-dispatch assertions and final zero charges.
Main: 24 cases (three sizes/four shapes/core+workbook); existing: six; small: three.
The large workbook cases are explicit research fixtures, not new product caps.

Compare current ordinary results to the memorialized readiness-private reports in
`docs/performance/persistent-workbook-readiness.md`, using matching workloads only.
Budgets: 100K core numeric <1 s, mixed/fan-out <2 s; workbook <5 s; ADD10K and
FORMULAS10K <500 ms. Interactive p95 input/capture <5 ms, short formula <10 ms,
selected 1K-chain <25 ms and bounded inspection <5 ms. Record regressions/failures
honestly; three construction samples are not a construction p95/SLA.

Retain 600-second child timeout and configured 8 GiB RSS budget. macOS live RSS
helper remains unavailable; wait4 high-water is post-exit evidence and uses bytes
on Darwin. Check completed-child values against the budget. Independently sample
native pressure/swap/VM/process conditions throughout. Stop on unsafe/critical
pressure, terminating the actual owned child session groups AND driver; killing
only the driver can leave children running. Record warnings alongside timings and
preserve partial outputs. Do not add stress/200K/million/full-chain sweeps.

## Return evidence

Write REPORT.md, machine-readable summary, gate logs/counts, benchmark CSVs and
summaries/manifests, source/environment evidence, pressure samples, failures/reruns
and explicit limitations. Distinguish sampled RSS, child high-water, allocation
traffic, retained storage and admission charges. Zero final charges is not proof
that RSS returns to baseline. GUI/AOT, cross-user denial, durability and capacity
increases remain outside validation.

Copy reports only back to the SMB share under
`artifacts/macos-validation-<UTC timestamp>/`. Verify returned file hashes and report
links; include transfer verification. Do not update shared product docs. Provide the
artifact path and outcome so the Windows agent can integrate results for review.
