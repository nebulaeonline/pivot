# Native macOS validation: ranges and SUM Slice C

Validate the current working tree, including untracked files and deletions. This is
new canonical selection/SUM and WB2 work; earlier Slice B results cannot be reused. Read
[ADR-0016](../adr/0016-first-class-ranges-and-sum.md) and the
[handoff](2026-10-20-ranges-and-sum-slice-c.md) first. Do not change product
source, dependencies, fixtures, assertions or caps to obtain passes. Retain all
original failures and reruns. Do not update shared project docs or commit.

Locate the SMB Pivot root (previously `/Volumes/RobMac/coding/pivot`). Create a fresh
local directory on internal APFS and verify its filesystem with `diskutil info`
and `df`. Copy the working tree with `rsync -a`, excluding `.git/`, `artifacts/`,
`target/`, all `bin/`, `obj/`, `__pycache__/` and `.vscode/` directories. Do not use
clone/archive HEAD: dirty and untracked source must be included. Never build .NET
from SMB. Keep source, Cargo target, .NET outputs, temporary files and original
reports on APFS. Record copy command, HEAD, working-tree status/diff, OS/hardware,
SDK/tool versions and source SHA-256 hashes; verify local/share hashes before and
after validation and report every difference. Also verify the 168 build inputs in
`artifacts/ranges-build-inputs.json` on the share against the APFS copy and include
that reference manifest in the returned reports. No source edits are expected.

Set PYTHONUTF8=1, CARGO_TARGET_DIR and TMPDIR to local paths. Set
PIVOT_FILE_TEST_ROOT to a local APFS test directory. With a working graphical login,
run sequentially from the copied root, retaining stdout/stderr and exit codes:

```sh
python3 scripts/ci.py --output-dir "$REPORTS/ci"
cargo clippy --locked --workspace --all-targets --all-features -- -D warnings
cargo test --locked --workspace --all-features
python3 tests/application-session/ui_smoke.py --output-dir "$REPORTS/ui"
```

REPORTS must be a local APFS directory. Full CI now includes real cross-process
workbook persistence tests and `persistence.json` / `persistence-production.json`
and `persistence-larger-diagnostic.json` beside application-session results. Also collect
`ranges.json`: selection index visits/candidates/members, allocation traffic, retained
charges, capture/release and single-observation range-file timings. Collect all of these, not only the CI summary.
Native file tests inject staging failures, terminate a writer and preserve prior
file content; verify they run on APFS. Record default/all-feature counts, .NET build
warnings/errors, decimal/protocol/IPC gates and Rust final allocation charges.

SUM semantic tests, sparse geometry, independent integer/decimal oracles, cycles,
admission/cancellation and WB1 rejection must all pass. Native files use WB2; no WB1
migration reader is expected. Confirm an untouched New closes without prompting.

The UI self-check must open the actual Avalonia window and real Rust child, exercise
Save As → New → Open → edit → Save, and produce results.json, results.png and a
saved `.pivot` fixture. Inspect the rendered PNG for controls/footer/grid clipping,
overlap and readability. A headless build or absent graphical session is an explicit
limitation, not a GUI pass. OS file-picker/confirmation interaction is not automated;
manually check Open/Save dialogs and Save/Discard/Cancel on dirty close,
recording exactly what was tried. Do not claim an unperformed manual check.

Summarize the 11 production-fixture Save/Open observations after one warmup and each
capture/encode/write/read/decode/restore component. Keep single-observation small
fixture timings separate; do not present tiny samples as SLAs. Source caps stay
unchanged. Normal close must report zero charge; abrupt termination cannot provide
a graceful receipt. Preserve known WSLg startup and historical backend viewport
issues; native macOS passes do not resolve their causes.

Copy reports only back to SMB under `artifacts/macos-validation-<UTC timestamp>`.
Write REPORT.md with source/environment provenance, exact commands/exits, all failures,
semantic/file/GUI findings, timings and limitations. Verify every returned report’s
SHA-256 and relative report links, and record transfer verification. Preserve the
APFS tree for diagnosis. Do not begin a new milestone.
