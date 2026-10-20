# Native macOS: runnable application/session and worksheet Slice A

Validate the current dirty Pivot working tree, including new files and deletions.
Earlier canonical-engine reports do not validate this application slice. No product
changes, test weakening, staging or commits. Read ADR-0014 and the application Slice A
handoff in the copy. Record failures and actual statuses rather than assuming a pass.

## Native filesystem and source

Locate the SMB share root (previously `/Volumes/RobMac/coding/pivot`). Record actual
path, UTC stamp, Git HEAD/status/binary diff. Copy the complete working tree into a
fresh unique internal APFS directory, excluding .git, artifacts, target, all bin/obj,
__pycache__, .vscode. Use rsync -a, not a clone/archive of HEAD; include dirty/untracked
files and reflect deletions. Verify APFS device IDs/diskutil before building.

Keep source, Cargo target, .NET outputs, TMPDIR, PIVOT_FILE_TEST_ROOT and initial
reports on APFS. Do not build or test on SMB. Set PYTHONUTF8=1. SHA-256 manifest the
copied source, compare with SMB before tests and compare local/share again afterward.
Record tools, hardware, power/thermal/swap/pressure conditions and free space.
Preserve the APFS worktree for investigation. No project capacity increase is allowed.

## Correctness and actual application execution

Run sequentially from the local source with local output paths, retaining exact
commands, stdout/stderr and exit codes:

```sh
python3 scripts/ci.py --output-dir "$REPORTS/ci"
cargo clippy --locked --workspace --all-targets --all-features -- -D warnings
cargo test --locked --workspace --all-features
python3 tests/application-session/ui_smoke.py --output-dir "$REPORTS/ui"
```

Full CI now includes `application-session`: it builds the real Rust child and runs
the C# integration executable, producing an application-session/results.json with
semantic assertions and boundary latency observations. It also retains all existing
.NET Release, 10,099 decimal vectors, frozen hosted descriptors/portable fixtures,
canonical codec probe, real daemon/CLI/REPL and document checks. Report actual Rust
counts, not counts inferred from older configurations. The new application schema
is compiled separately from frozen hosted descriptors. No backend selectors remain.

The last command starts the real Avalonia window, enters exact decimals and scalar
formulas, edits dependencies, repairs an invalid draft, moves/clears cells, verifies
stable coordinate reuse, scrolls/supersedes viewport loads, measures rendered-frame
latency, writes results.json and results.png, and closes with zero final Rust charge.
It requires a working graphical login/display. Obtain any environment-required GUI
permissions and record the actual execution status; a build or headless skip is not
a GUI pass. Inspect results.png with the image viewer for clipping/overlap/readability.
If GUI access is unavailable, record that as an explicit execution limitation.

For manual interaction, run `python3 scripts/run_desktop.py`. Click a cell, use the
formula field, commit with Enter/button, clear/move and scroll. Do not accidentally
leave a manual window/Rust child running. This creates an unsaved, volatile workbook;
closing the session discards it. The test report writes only to local APFS.

## Evidence and interpretation

Retain session startup/shutdown; 31 boundary samples after three warmups for edit,
viewport and selected 256-formula chain; Rust execution; C#/Rust serialization probes;
framing/transport/scheduling residual; complete request/response time; and UI apply,
paint delay and edit-to-render observations. Separate these components: medians do
not sum into an exact decomposition. Rust encode is a diagnostic first-pass probe;
final encoding also occurs. Residual includes framing, Rust decode/final encode,
OS scheduling and pipe overhead, not pure kernel transport. Eight UI action samples
are observations, not p95/SLA or physical screen presentation. UI refresh includes
35 ms coalescing. Do not rerun old reference/100K research sweeps for this milestone.

The actual boundary checks cover exact Decimal96/scale/signed zero, arithmetic errors,
formulas/dependencies, stable moves/reuse, drafts/repair, rollback/conflicts, immutable
S/T, bounded rectangles, queued and late cancellation, pre-queue byte admission, abandoned-capture release,
capture limits, transport failure, stale displayed-frame conflicts, viewport-edge
scrolling, and explicit zero-charge cleanup. Cancellation
after dispatch drains the response and does not roll back accepted edits. Broken
transport is terminal with no replay. Caller buffers are outside the retained ledger.

Preserve the historical Linux 5.343 ms viewport p95 and its 3.381 ms repeat as existing
backend evidence; do not erase them or claim this UI work resolves their cause.
No new functions/ranges, providers/containers/persistence, formatting or product
capacity/million-cell claim is included. Native GUI/AOT and durability are separate;
this run tests GUI execution, not AOT or power-loss durability.

## Return reports

Write REPORT.md and machine-readable summary, CI/feature logs and counts, boundary
results, UI results/image/execution status, environment/source evidence, warnings,
failures/reruns and limitations. Return reports only to the SMB root under
`artifacts/macos-validation-<UTC timestamp>/`, verifying hashes of returned files
and report links. Do not update shared project docs; the Windows agent will integrate
results for parent review. Report the artifact directory and actual outcome.

Known application issue to retain: the first WSLg GUI attempt aborted natively with
free(): invalid pointer/SIGABRT. A debugger run and subsequent normal runs succeeded;
no cause or renderer workaround was established. This is not a known macOS failure,
but preserve any native startup issue and original logs rather than rerunning until
passing or marking headless execution as a GUI pass.
