# Native macOS/APFS — ADR-0023 Stage 4 validation

[Completed native APFS run and independent evidence review](2026-10-23-macos-history-association-validation.md).
The instructions below remain for reproduction; the 20261023T222012Z run passed.

Run this headless Rust research validation on the Mac. Do not change source, weaken
assertions or migrate any workbook/history format. No desktop/manual/.NET/AOT run
is needed for this feature-only Rust stage. Report every failure and stop dependent
gates; preserve failed logs/data. Do not install or upgrade tooling without reporting
that need. Context: [Stage 4 packet](2026-10-23-history-association-stage4.md).

## Native working copy

Set `share` to the mounted SMB Pivot repository. Use a new local APFS directory;
source, target, temporary test files and journals must all be native. Results are
copied back to the SMB artifacts directory after hashing on APFS.

```sh
share="/Volumes/REPLACE_WITH_SHARE/pivot"
stamp=$(date -u +%Y%m%dT%H%M%SZ)
run="$HOME/pivot-stage4-$stamp"
mkdir -p "$run/source" "$run/results" "$run/tmp"
diskutil info "$run" > "$run/results/filesystem.txt"
# Inspect filesystem.txt and confirm APFS; stop if this is SMB/NFS/another filesystem.
rsync -a --exclude=.git --exclude=target --exclude=artifacts --exclude=bin --exclude=obj --exclude=.vs --exclude=.vscode "$share/" "$run/source/"
cp "$share/artifacts/history-stage4/build-input-manifest.json" "$run/results/build-input-manifest.json"
cd "$run/source"
export CARGO_TARGET_DIR="$run/target"
export TMPDIR="$run/tmp/"
export PIVOT_FILE_TEST_ROOT="$run/tmp"
rustc -Vv > "$run/results/toolchain.txt"
cargo -V >> "$run/results/toolchain.txt"
sw_vers >> "$run/results/toolchain.txt"
```

Verify every input before building. This detects copying a changing SMB checkout:

```sh
python3 - "$run/source" "$run/results/build-input-manifest.json" <<'PY'
import hashlib,json,sys
from pathlib import Path
source=Path(sys.argv[1])
entries=json.loads(Path(sys.argv[2]).read_text())
for e in entries:
    assert hashlib.sha256((source/e['path']).read_bytes()).hexdigest()==e['sha256'],e['path']
print('matched build inputs:',len(entries))
PY
```

Record the output in results. If any input differs, report it; do not call results
validation of the frozen Windows/Linux source. Do not build/test directly on SMB.

## Gates

Execute sequentially and check each exit code before continuing. Redirect both
stdout and stderr to the listed APFS logs.

```sh
cargo fmt --all -- --check > "$run/results/format.log" 2>&1
cargo test --workspace > "$run/results/default.log" 2>&1
cargo test --workspace --all-features > "$run/results/all-features.log" 2>&1
cargo clippy --workspace --all-targets --all-features -- -D warnings > "$run/results/clippy.log" 2>&1
cargo build --release -p pivot-workbook-document --features history-experiment --example history_association > "$run/results/build.log" 2>&1
python3 scripts/research_history_association.py macos "$run/target/release/examples/history_association" "$run/native-journals" "$run/results/macos-results" > "$run/results/matrix.log" 2>&1
```

The matrix should finish with 69 cases and a native-original hash manifest. It
kills workers intentionally; those nonzero child exits are expected only where
asserted by the runner. Every completed worker/probe must report zero-charge finish.
The historical-export rejection is a known negative test, not permission to change
the transfer codec. Session-only retirement and Clear preserve archived research
bytes; no physical reclamation is claimed. Active directory rename can be refused
by the OS lock or can succeed and trigger the explicit namespace-unavailable branch;
report the actual Mac behavior.

Verify the runner's native-original hashes against BOTH `$run/native-journals` and
`$run/results/macos-results/native-evidence` before copying. Record test counts,
69-case outcome, zero-charge checks, source/binary hashes and failures in an APFS
`summary.md`. Do not infer power-loss durability from process-kill tests.

## Preserve and publish evidence

Hash every file under the APFS results directory (exclude the manifest itself),
then copy that directory to:

`$share/artifacts/history-stage4/macos-validation-$stamp/`

Verify every copied file against the APFS-origin manifest, and include a checksum
of that manifest. Preserve the APFS source and original journals for investigation.
Copied history on SMB is evidence only: its local physical identity/path bindings
are intentionally different, so do not run attachment probes against the SMB copy
or treat their rejection as a bug. All asserted native operations must run before
copying on APFS. Source/hash mismatch, unsupported filesystem behavior or any failed
gate must be prominent in the summary; do not claim three-platform acceptance.
