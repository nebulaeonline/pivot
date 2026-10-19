# Native macOS validation review — Spreadsheet Formula Slice B

Date: 2026-10-19
Disposition: Validation passed; no product changes required.

Reviewed the Mac Codex report, CI summary and detailed Rust/.NET/parity/IPC logs,
separate cost log, copy verification and source manifest under
[shared artifacts](../../artifacts/macos-validation-20261019T093243Z/macos-validation.md).
All CI gates and the separate cost command exited zero. Independently summing Rust
result lines confirms 210 passed, zero failures; the separate cost test is already
included in that workspace count. Before this documentation update, all 215 current
source files matched the Mac SHA-256 manifest. No implementation changed afterward.

## Environment and reproducibility

Native arm64 macOS 27.0.1 (26A434); Rust 1.101.0-nightly (b6eeb93c9, 2026-10-18),
Cargo 1.101.0-nightly (3960fa69b), Python 3.9.6, .NET SDK 10.0.402.

The SMB working tree at `/Volumes/RobMac/coding/pivot` was copied with rsync into
`/private/tmp/pivot-macos-validation.cronqr_4/source`, including uncommitted and
untracked sources. Exclusions: .git, target, artifacts, .vs and all bin/obj directories.
The local directory was confirmed on internal APFS using the Data mount point and
matching device IDs. Source, Cargo targets and .NET outputs remained local.
Only reports returned to `artifacts/macos-validation-20261019T093243Z` on SMB.
This preserves the required APFS build workflow, especially for .NET.

From the local source with CARGO_TARGET_DIR pointing to the local target directory:

```sh
python3 scripts/ci.py --output-dir /private/tmp/pivot-macos-validation.cronqr_4/reports/ci
cargo test --locked -p pivot-workbook --lib report_text_formula_costs -- --nocapture
```

The Mac report retains full commands, versions, preflight notes and original local
absolute paths. Two filesystem preflight attempts failed before any build; checking
the Data mount point with necessary permission resolved them. Actual CI and costs
ran once, with no test weakening, source fixes, dependency changes or commits.

## Verification evidence

| Gate | Result |
| --- | --- |
| Formatting and workspace Clippy, warnings denied | Passed |
| Rust workspace | 210 tests passed, zero failed/ignored |
| .NET Release, warnings as errors | Five projects, zero warnings/errors |
| Decimal parity | 10,099 vectors, zero value/error failures or scale differences |
| Protocol parity | 58 decimal layouts, authoring, frozen legacy codec/descriptor checks passed |
| Actual Unix-socket daemon/REPL | Invocation, concurrency, malformed traffic, reconnect, authoring and cleanup passed |
| Separate scalar formula cost harness | Passed; independent expected output 4 verified |

- [CI summary](../../artifacts/macos-validation-20261019T093243Z/ci/20261019T093324130572Z/summary.json)
- [Rust tests](../../artifacts/macos-validation-20261019T093243Z/ci/20261019T093324130572Z/rust-tests.log)
- [IPC results](../../artifacts/macos-validation-20261019T093243Z/ci/20261019T093324130572Z/ipc/results.txt)
- [Source manifest](../../artifacts/macos-validation-20261019T093243Z/source-manifest.json)
- [Copy verification](../../artifacts/macos-validation-20261019T093243Z/copy-verification.json)
- [Mac final source verification](../../artifacts/macos-validation-20261019T093243Z/working-tree-verification.json)

All retained file/process tests ran on local APFS; no Linux-specific temporary-root
override was used. Current three-platform evidence is Windows full CI (209 Rust
tests), WSL2 Rust/IPC (210) and native macOS full CI (210). Linux .NET's earlier full
CI evidence remains historical; this review does not claim a new Linux .NET run.

## Cost observations

[Raw cost log](../../artifacts/macos-validation-20261019T093243Z/formula-costs.log).
Debug profile, parser/resolver means over 500 repetitions; other columns single samples.

| Live cells | Parse mean ns | Resolve mean ns | Formula edit us | Input edit us | Capture us | Evaluation us |
| --- | --- | --- | --- | --- | --- | --- |
| 17 | 2,354 | 3,275 | 63 | 45 | 83 | 22 |
| 129 | 2,270 | 3,334 | 215 | 152 | 522 | 32 |
| 513 | 1,572 | 2,572 | 386 | 431 | 1,770 | 52 |

Resolution excludes parsing, staging, first-use blank allocation, installation and
retained presentation metadata. Full formula/input edits include copies and quotas;
capture rebuilds dependency order and copies position maps. These observations
support the documented full-copy scaling concern. Different machines, compiler
nightlies and tiny debug samples do not support platform rankings or capacity claims.

GUI execution, Native AOT, actual cross-user denial, production workbook persistence
and power-loss durability remain untested. The following implementation priority
remains the C# application/session IPC client, subject to parent review. This update
changes documentation only and does not advance that milestone.
