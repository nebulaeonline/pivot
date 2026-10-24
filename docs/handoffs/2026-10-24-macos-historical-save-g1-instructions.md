# macOS validation — G1 historical native Save

Run this against the current shared Pivot tree. This validates the **internal Rust
G1 correction**, not ordinary Save promotion or desktop Undo/Redo. No new manual
UI testing is needed. Preserve failures and local native originals.

## Native local setup

Find the actual SMB share path for Pivot and set `SHARE` accordingly. Every source,
build target, test temporary directory and original native data file must be on a
**local APFS volume**, not SMB. Use native ARM64 tools; stop if Rosetta is active.
Example shell setup (replace only the share path):

```sh
SHARE="/Volumes/your-share/coding/pivot"
STAMP=$(date -u +%Y%m%dT%H%M%SZ)
WORK=$(mktemp -d /private/tmp/pivot-g1.XXXXXXXX)
SRC="$WORK/source"
REPORT="$WORK/report"
mkdir -p "$SRC" "$REPORT" "$WORK/tmp"
export TMPDIR="$WORK/tmp"
export PIVOT_FILE_TEST_ROOT="$WORK/tmp"
export CARGO_TARGET_DIR="$WORK/target"
uname -a > "$REPORT/platform.txt"
sysctl -n hw.optional.arm64 >> "$REPORT/platform.txt"
sysctl -in sysctl.proc_translated >> "$REPORT/platform.txt" 2>&1 || true
df "$WORK" >> "$REPORT/platform.txt"
diskutil info "$WORK" >> "$REPORT/platform.txt"
rustc -vV > "$REPORT/rustc.txt"
cargo -V > "$REPORT/cargo.txt"
rsync -a --exclude='.git/' --exclude='artifacts/' --exclude='target/' --exclude='bin/' --exclude='obj/' --exclude='.vs/' "$SHARE/" "$SRC/"
cp "$SHARE/artifacts/history-save-g1/build-input-manifest.json" "$REPORT/"
cp "$SHARE/artifacts/history-save-g1/run_gates.py" "$REPORT/"
cd "$SRC"
```

Confirm APFS and native ARM64 from the recorded platform/filesystem output. Do not
reuse old source copies, old workers or old test results. Do not overwrite the
Windows/Linux or original Stage 2–4 manifests.

## Input verification, gates and actual native matrix

Before running, verify the frozen build inputs (168 at preparation):

```sh
python3 - "$REPORT/build-input-manifest.json" <<'PY'
import hashlib,json,pathlib,sys
rows=json.load(open(sys.argv[1]))
for row in rows:
    assert hashlib.sha256(pathlib.Path(row['path']).read_bytes()).hexdigest()==row['sha256'],row['path']
print(len(rows),'matched Rust build inputs')
PY
python3 "$REPORT/run_gates.py" macos "$REPORT"
file "$CARGO_TARGET_DIR/release/examples/history_association" > "$REPORT/worker.txt"
shasum -a 256 "$CARGO_TARGET_DIR/release/examples/history_association" >> "$REPORT/worker.txt"
python3 scripts/research_history_association.py macos "$CARGO_TARGET_DIR/release/examples/history_association" "$WORK/native-data" "$REPORT/macos-results" > "$REPORT/macos-matrix.log" 2>&1
```

Record each actual exit status. `run_gates.py` runs format, strict all-target/
all-feature lint, default tests, all-feature tests and release worker build; it
records exact commands, durations, logs and exit codes. It stops at the first
failure. Run later independent gates separately if needed, retaining failed logs
and using new attempt filenames/roots. Do not weaken tests or silently retry.

The matrix should report **69 cases**, including `historical-native-save`, with
all normal workers exiting successfully after `FINISH 0`; deliberately killed
workers must have their expected nonzero termination status. Its fresh-process
historical Save checks real WB5 installation/reopen and clean B → dirty D → clean B.
The generic full-ledger rejection remains tested. Expected native-original file
hash count is 884; investigate differences instead of editing expectations.

The all-feature suite includes new sparse-root and rich cold restoration tests.
One test starts a real child test process, writes WB5 from that restored root,
installs it locally and opens the result in both processes. All test runtimes must
finish with zero domain charge. Invalid/cyclic/retired drafts retain their exact
content/provenance; mixed-invalid first-core diagnostic order is not a stable
cross-namespace enumeration contract. Isolated cycle diagnostics are compared
exactly. Existing native uncertainty safeguards must remain enforced.

Rerun the input verification above after the tests, saving its output. Record
source/input hashes before and after, test counts, native original-vs-copied hash
verification, worker hash and all failures. Write `summary.md` explaining whether
all gates passed and naming the exact local paths, APFS evidence and source match.
Do not claim physical power-loss durability or production integration.

## Preserve evidence and copy results back

While still on APFS, verify every `macos-results/native-original-hashes.json`
entry against both `$WORK/native-data/<path>` and its copied
`macos-results/native-evidence/<path>`. Hash every report file into an evidence
manifest before copying; exclude that manifest and its checksum receipt from
self-hashing. Copy the complete report to a **new** directory:

`$SHARE/artifacts/macos-validation-<UTC timestamp>/`

Use the actual `$STAMP` value. Check all copied report hashes against the APFS
manifest and write a separate copy-verification receipt. Preserve local source,
target, native originals and report; provide their paths in the summary. Do not
run attachment/recovery against SMB copies of private native history files.
Notify the Windows session when the report is ready.

## Explicit deferred promotion gate

This run is sufficient to review the internal correction; it does **not** promote
`export_historical_document` into ordinary application Save. Before any such
promotion, the corrected ordinary path must additionally pass source-matched
managed/session, desktop Save/Open and actual Native AOT execution on Windows,
native Linux and native macOS ARM64/APFS, including zero-charge cleanup. Keep that
requirement pending; passing the unchanged application path is not proof of the
new path's future promotion. G2–G7 remain deferred.
