# macOS — G1 ordinary Save promotion final gates

Validate the **promoted ordinary WB5 Save path**, not the earlier internal-only
G1 source. No manual UI checklist is needed. Do not edit source/tests/limits, add
history UI or weaken a failing assertion. Keep failed attempts and report exact
causes. Ordinary Save promotion is authorized; G1 closure and G2 are not accepted.

## Setup on native APFS

Set `SHARE` to the actual mounted SMB Pivot directory. All source, target, temporary
test data, journals, .NET build/publish files and report output must be on a local
APFS volume. Copy source off SMB; do not run .NET tests/builds against the share.
Use native ARM64 tools with `sysctl.proc_translated=0`. Preserve local originals.
Example (replace only the share path):

```sh
SHARE="/Volumes/your-share/coding/pivot"
STAMP=$(date -u +%Y%m%dT%H%M%SZ)
WORK=$(mktemp -d /private/tmp/pivot-g1-promotion.XXXXXXXX)
SRC="$WORK/source"
REPORT="$WORK/report"
mkdir -p "$SRC" "$REPORT" "$WORK/tmp"
export TMPDIR="$WORK/tmp"
export PIVOT_FILE_TEST_ROOT="$WORK/tmp"
export CARGO_TARGET_DIR="$WORK/target"
rsync -a --exclude='.git/' --exclude='.agents/' --exclude='.aws/' --exclude='.codex/' --exclude='.vscode/' --exclude='artifacts/' --exclude='target/' --exclude='bin/' --exclude='obj/' --exclude='__pycache__/' "$SHARE/" "$SRC/"
cp "$SHARE/artifacts/history-save-g1-promotion/build-input-manifest.json" "$REPORT/"
cp "$SHARE/artifacts/history-save-g1-promotion/source-manifest.json" "$REPORT/"
cp "$SHARE/artifacts/history-save-g1-promotion/run_rust_extra.py" "$REPORT/"
cd "$SRC"
```

Record uname, macOS version, native/Rosetta status, rustc/cargo/dotnet/Python
versions, source/share/local/target/data/report paths and APFS mount evidence.
Query `diskutil info /System/Volumes/Data` and match `df`/device IDs for `$WORK`.
Stop if a selected build/data directory is on SMB or tools run under Rosetta.
No SDK/package upgrades or implementation changes are needed for this assignment.

Verify every frozen build input (236 Rust/managed/test/script inputs at preparation)
and source-manifest entry against the copied tree and the share **before and after**
gates. Use SHA-256 of raw bytes. The ignored GPT baseline may be one extra copied
file; do not silently accept changed implementation inputs. Record inventory and
missing/changed/added files. Preserve the original Windows/Linux/G1 manifests.

## Required gates — run sequentially and retain exact exit codes

```sh
python3 scripts/ci.py --desktop --output-dir "$REPORT/ci" > "$REPORT/ci-progress.log" 2>&1
python3 "$REPORT/run_rust_extra.py" macos "$REPORT/extra-rust" > "$REPORT/extra-rust-progress.log" 2>&1
python3 scripts/research_history_association.py macos "$CARGO_TARGET_DIR/release/examples/history_association" "$WORK/native-data" "$REPORT/macos-results" > "$REPORT/matrix.log" 2>&1
python3 scripts/verify_native_aot.py --output-dir "$REPORT/native-aot" > "$REPORT/aot-progress.log" 2>&1
```

Inspect each exit status before advancing. `ci.py` covers strict default lint,
format, default Rust, .NET warning-as-error build, decimal/aggregate reference,
protocol parity, local IPC, managed application session and actual managed desktop
workflows. `run_rust_extra.py` adds strict all-target all-feature lint, all-feature
Rust and the release historical association worker. `verify_native_aot.py` must
publish **and execute** the actual native desktop and peers, with the adjacent Rust
session child. A successful publish alone is not acceptance.

Expected Rust counts are 348 default / 391 all-feature on Mac (Windows 347/390).
Investigate discrepancies; do not edit expectations merely to make a run pass.
Inspect managed and AOT reports for all twelve acceptance fields, native Save/Open
and Save As, ticket/cancellation/failed Save behavior, main/peer `remaining_charge=0`,
actual execution mode/executable and valid WB5 fixture checksums. Published AOT
warnings must be absent. Retain publication file hashes and executable identity.

The historical matrix must pass **69 cases** with **884 original file hashes**,
24 deliberately killed fault workers, separate held owner -9 and all completed
workers/probes reporting `FINISH 0`. Audit seed/decline logs too (150 completed logs;
93 result-bearing completed workers/probes). Check the historical-native-save marker
for actual WB5 Save/Open, preserved Redo and clean B → dirty D → clean B. Preserve
the separate strict generic-transfer rejection. Verify originals and copied native
files against the manifest while both are still on APFS.

Historical activation remains in the accepted internal research boundary; its
association and rich cold tests now call the **ordinary exporter**. The production
session/UI has no history activation yet. Desktop tests establish nonhistorical
Save regressions; do not claim a historical UI workflow or add Undo/Redo.

## Failure handling

Windows default CI and actual Native AOT have passed locally. Native Linux full CI
passed on unchanged source after a failed Fill selection/scroll assertion. Windows
AOT first failed `Unsupported OS Copy leaves clipboard unchanged`, then passed one
unchanged-source full rerun. Both causes remain unresolved; neither UI code nor
assertions were changed. The earlier null-destination clipboard issue is separate.

If an intermittent failure recurs, preserve its complete reports, exact assertion,
source hash and environment. Do not repeatedly run until green or silently remove
failed evidence. An explained, separately logged unchanged-source diagnostic rerun
may be useful; report both outcomes and unresolved causes. Separate implementation
failures from first-error wrappers such as “Worksheet text acceptance missing.”
A source fix requires new matching manifests and gates, not reuse of earlier passes.

## Report and return to SMB

Write `summary.md` with exact commands, exit codes, input/source match, test counts,
managed and native execution results, main/peer zero-charge evidence, matrix/log
counts, native filesystem/toolchain proof, timings as observations only, failures,
setup corrections and all retained APFS paths. Fingerprint the release research
worker and real Rust session child. Retain source before/after inventory and hashes.

Hash all report files on APFS before transfer, excluding the evidence manifest and
its separate checksum receipt from self-hashing. Copy the complete report and native
evidence to a **new** `$SHARE/artifacts/macos-validation-<UTC timestamp>/` directory.
Verify every copied report hash against the APFS-origin manifest and checksum its
transfer receipt. Leave source, target, native originals and report on APFS. Do not
run attachment/recovery against SMB copies of private history files.

The previous Mac G1 run validates only the old internal source. This run is required
for the promoted source. No power-loss, arbitrary network-filesystem or physical
reclamation guarantee is established. Notify the Windows session when results are
ready; return final evidence for parent G1 closure. G2–G7 remain deferred.
