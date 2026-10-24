# Native Mac/APFS — headless execution-safety validation

Validate the new infrastructure safety gate, separately from the frozen G2 run.
Do **not** run `--desktop`, `ui_smoke.py`, `verify_native_aot.py`, a GUI executable,
clipboard sentinel tests or any VM. No manual GUI checklist is requested.
Only the console safety probe may simulate consent, without Avalonia dependencies.

Copy current source to native APFS first; keep source, target, temp, reports and
all .NET bin/obj local. Use a native ARM64 shell/toolchain. Change SHARE to the
actual mount. Do not install or upgrade tools as part of validation.

```sh
SHARE="/Volumes/your-share/coding/pivot"
STAMP=$(date -u +%Y%m%dT%H%M%SZ)
WORK=$(mktemp -d /private/tmp/pivot-execution-safety.XXXXXXXX)
SRC="$WORK/source"
REPORT="$WORK/report"
mkdir -p "$SRC" "$REPORT" "$WORK/tmp"
export TMPDIR="$WORK/tmp"
export PIVOT_FILE_TEST_ROOT="$WORK/tmp"
export CARGO_TARGET_DIR="$WORK/target"
rsync -a --exclude='.git/' --exclude='.agents/' --exclude='.aws/' --exclude='.codex/' --exclude='.vscode/' --exclude='.vs/' --exclude='artifacts/' --exclude='target/' --exclude='bin/' --exclude='obj/' --exclude='__pycache__/' "$SHARE/" "$SRC/"
cp "$SHARE/artifacts/validation-safety/build-input-manifest.json" "$REPORT/"
cp "$SHARE/artifacts/validation-safety/review-source-manifest.json" "$REPORT/"
cd "$SRC"
python3 scripts/ci.py --output-dir "$REPORT/ci" > "$REPORT/headless-ci.log" 2>&1
```

The normal headless CI now includes `tests/execution-safety/verify.py`. Expect nine
safety tests and 348 default Rust tests; no failed/ignored tests. The solution has
seven projects, including the new standalone policy probe. Preserve all original
assertions and errors. Stop on failure and report; do not rerun until green, change
code/limits or weaken tests. No full graphical/AOT acceptance pass is implied.

Before and after gates verify SHA-256 of **all 247 code/build inputs** against both
the share and local copy. Also verify every review-source manifest entry and
inventory differences; an ignored GPT baseline may be an extra copied file.
Record APFS mount/device for WORK, OS/version, native ARM64/no-Rosetta evidence,
Python/rustc/cargo/dotnet versions, exact paths, commands, exits and durations.
Do not regenerate original manifests or compare changed source to old G2 hashes
as if this infrastructure code had been tested in the earlier run.

The synthetic consent broker uses local loopback sockets, not remote execution.
Its tests mock terminal answers solely to exercise the non-GUI probe and stubbed
CI routing. Never reuse/export a live permit to launch the desktop. Report any
socket/runtime blocker accurately. Unattended guest isolation remains NotConfigured.

Preserve native originals, before/after manifests, complete logs and CI subtree.
Record no owned helper/probe survivors. Write a concise summary with gate counts,
cleanup/remaining Rust charge evidence, source matches and any failures/skips.
Hash reports **on APFS before transfer**, excluding only self-hash/transfer receipt;
copy to `artifacts/macos-validation-<timestamp>` on SHARE, verify every returned hash
and add a transfer receipt/checksum. Keep source, native test fixtures and binaries
on APFS; disclose which are not transferred. No recovery against SMB fixtures.

Return evidence for safety-milestone review. G2 product review is separate;
no guest preflight, orchestration, bootstrap or expanded platform pass is authorized.
