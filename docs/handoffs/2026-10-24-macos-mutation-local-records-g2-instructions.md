# macOS/APFS — G2 mutation-local record validation

Validate the current **feature-only Rust G2 producer**, not the accepted G1 source.
No new manual UI, .NET, desktop or AOT run is required for this Rust-only experiment.
Do not promote default mutation, change tests/limits, replace the persisted journal
or choose a backend. G3–G7 remain deferred; return evidence for parent G2 review.

## Native setup

Set `SHARE` to the actual mounted Pivot SMB directory. Copy source to a local APFS
volume before any build/test. Source, target, temporary files and report output must
all be APFS-native; SMB is only the source/evidence exchange location. Preserve local
originals and all failed attempts. Use native ARM64 tools, not Rosetta.

```sh
SHARE="/Volumes/your-share/coding/pivot"
STAMP=$(date -u +%Y%m%dT%H%M%SZ)
WORK=$(mktemp -d /private/tmp/pivot-records-g2.XXXXXXXX)
SRC="$WORK/source"
REPORT="$WORK/report"
mkdir -p "$SRC" "$REPORT" "$WORK/tmp"
export TMPDIR="$WORK/tmp"
export PIVOT_FILE_TEST_ROOT="$WORK/tmp"
export CARGO_TARGET_DIR="$WORK/target"
rsync -a --exclude='.git/' --exclude='.agents/' --exclude='.aws/' --exclude='.codex/' --exclude='.vscode/' --exclude='.vs/' --exclude='artifacts/' --exclude='target/' --exclude='bin/' --exclude='obj/' --exclude='__pycache__/' "$SHARE/" "$SRC/"
cp "$SHARE/artifacts/history-records-g2/build-input-manifest.json" "$REPORT/"
cp "$SHARE/artifacts/history-records-g2/source-manifest.json" "$REPORT/"
cp "$SHARE/artifacts/history-records-g2/run_gates.py" "$REPORT/"
cd "$SRC"
```

Record source/share/target/test/report paths, uname/macOS/rustc/cargo/Python versions,
`sysctl.proc_translated=0`, ARM64 host/images and APFS mount evidence. Query
`diskutil info /System/Volumes/Data` and match the working directory's `df`/device.
No tool upgrades are part of this assignment. Stop/report if prerequisites are absent.

Verify raw-byte SHA-256 of every frozen **173 build/measurement input** against both
local source and the share before and after gates. These cover 172 Rust files/
manifests and `scripts/research_mutation_records.py`. Also verify every frozen source
entry and inventory additions/removals/changes. The ignored GPT baseline may be one
extra copied file; it is not an implementation difference. Preserve original G1/G2
source, input and evidence manifests rather than regenerating them.

## Execute the supplied final gates

```sh
python3 "$REPORT/run_gates.py" macos "$REPORT/macos-reviewed" > "$REPORT/progress.log" 2>&1
file "$CARGO_TARGET_DIR/release/examples/history_record_producer" > "$REPORT/worker.txt"
shasum -a 256 "$CARGO_TARGET_DIR/release/examples/history_record_producer" >> "$REPORT/worker.txt"
```

The supplied helper creates a new output root and runs sequentially: format,
strict default lint, strict all-target/all-feature lint, full default workspace
tests, full all-feature workspace tests, release G2 worker build and the measurement
runner. `gates.json` records commands/durations/exits and stops on the first failure.
Expected Mac Rust counts are **348 default / 402 all-feature**, with zero failed or
ignored tests; investigate any discrepancy rather than editing expectations.

The focused G2 workbook file has nine integration tests; two additional core/
workbook unit tests cover private cancellation and deliberate oracle omissions.
Coverage includes complete checkpoint equality, typed/text formulas and provenance,
invalid/cyclic/rejected drafts, exact Decimal96/Blank/text, Move/retirement, Fill in
all four directions, native/external and cross-document Paste/copy, repeated records,
160 deterministic mixed transitions, stale/foreign scopes, Undo-then-branch,
cold-restored namespace remapping, concurrent captures and publication exhaustion.

The new source validation marker is ephemeral, not serialized or a durable witness.
Full baseline observation/cold validation establishes it; local paired publication
updates reservations and the marker without a whole-state scan. The 48-step local
chain test does not call full observation between edits. The independent oracle
separately exports/observes/folds the complete state and checks all bytes, cold
CellViews, diagnostics and valid-graph evaluation. The persisted journal is unchanged.

## Measurement checks

`samples/` must contain **45 fresh-process zero-charge runs**: five cases
(input, scalar formula, range input, range formula, allocation) × sizes 32/256/768 ×
three samples. Every log must terminate in `FINISH 0`; retain raw logs, `samples.json`,
`summary.json`, worker fingerprint and exact failed return codes. The helper invokes:

```sh
python3 scripts/research_mutation_records.py macos "$CARGO_TARGET_DIR/release/examples/history_record_producer" "$REPORT/another-new-samples-root"
```

That command is shown for provenance, **not a request to duplicate a passing run**.
Do not reuse a preexisting root or overwrite failures.

Check emitted-record counts 3/3/4/10/9 for those five cases across sizes, selected
core reads 12/12/12/16/14, and no canonical full-record scans except the existing
range-formula validation path (35/259/771). Check source/candidate oracle bytes match
via successful child assertions, independent captures remain correct, and staging,
publication and full-oracle timings/peaks are separately reported.

Keep qualifications: these are three-sample diagnostics, not benchmarks; domain
requested/reserved charge is not RSS. Range edits retain the preexisting roughly
16.8 MB invalidation reservation. New-reservation publication retains the existing
old/new ledger-charge policy, so its requested peak grows with lifetime allocation
size even though it does not clone the ledger. No quota/default-policy change or
constant-time complete-edit guarantee is established.

## Failure handling and evidence return

The first Linux G2 all-feature run failed an existing Rust clipboard test's
`runtime.finish().is_ok()` assertion. Its unchanged-source targeted rerun passed;
the final hardened-source full Linux gates passed. Root cause remains unresolved.
Retain any recurrence independently of the earlier UI clipboard/Fill issues.
Do not weaken shutdown assertions or silently repeat gates until green. Keep exact
logs/source hashes and distinguish setup errors from test failures. A separately
reported unchanged-source diagnostic rerun may be appropriate; never erase the
initial outcome. Stop if a semantic/canonical mismatch, stale authority, lost ledger
reservation or capacity increase would be needed.

Write `summary.md` with commands, actual exits, test counts, focused coverage,
source/input match, ownership/zero-charge evidence, native ARM64/APFS proof, all
45 sample counts and measured qualifications. Include failures/reruns/setup changes
and preserved APFS paths. The prior G1 Mac runs are not evidence for this source.

Hash all report files while still on APFS, excluding the evidence manifest and its
separate checksum receipt from self-hashing. Preserve the local source, target,
report and any native regression-test data. Copy the complete report to a new:

`$SHARE/artifacts/macos-validation-<UTC timestamp>/`

Verify all copied report hashes against APFS-origin hashes and checksum the transfer
receipt. Do not probe private history attachment/recovery on copied SMB data. No
power-loss, network-filesystem or physical reclamation claim is implied. Notify the
Windows session when ready; parent G2 review is the stopping point, not G3.
