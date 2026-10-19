# Linux and macOS validation

## Portable storage foundation — 2026-10-18

Windows full CI and WSL Linux Rust/IPC checks cover the portable record codec and
validated restoration foundation, including its frozen compatibility corpus.
See the [storage foundation handoff](handoffs/2026-10-18-portable-storage-foundation.md)
for exact reports. This validates portable in-memory restoration, **not SQLite or
database durability**. macOS verification of this slice remains pending.

## Minimal REPL authoring — 2026-10-18

Full Windows CI and available WSL Linux Rust/IPC checks cover the new authoring
commands, mutation recovery, conflicts and executable client tests. The Rust suite
has 135 tests. The real-process harness exercises both ordinary invocation and
authoring REPL workflows. See the [REPL authoring handoff](handoffs/2026-10-18-minimal-repl-authoring.md)
for exact evidence. macOS ARM64/APFS validation of Stage B and this REPL remains pending.

## Stage B authoring — 2026-10-18

Full local CI passed on Windows and Ubuntu WSL2, including the new daemon
authoring tests over named pipes and Unix sockets. The final Rust suite has
130 tests. Linux formatting, Clippy and all Rust tests were rerun after the final
embedded message-limit correction. See the [Stage B handoff](handoffs/2026-10-18-daemon-authoring.md)
for exact reports and coverage. .NET SDK versions were 10.0.402 on Windows and
10.0.112 in WSL. macOS validation below predates authoring and must be refreshed.

## Verified coverage

Windows named pipes and Unix-domain sockets on Ubuntu WSL2 Linux and macOS ARM64 have passed
the full local CI pipeline: Rust tests/lint/formatting, .NET Release solution build,
10,099 decimal compatibility vectors, generated Rust/C# protocol fixtures, and real
daemon/CLI integration. Linux uses .NET SDK 10.0.112; macOS uses 10.0.402.

This verifies Linux code running on the WSL2 Linux kernel, not merely a Windows
binary invoked from a Linux shell. Socket endpoints are created in Linux `/tmp`.
Cross-user access denial, Native AOT, and native Linux deployment remain separate
coverage items. GUI execution is not part of the pipeline.

The Linux .NET run used an isolated source copy under
`artifacts/linux-dotnet-20261018T164648Z/source` so Windows bin/obj outputs were not
overwritten. Its CI report is
`artifacts/linux-dotnet-20261018T164648Z/ci/20261018T164652188086Z/summary.json`.

## macOS

In the Mac's VS Code terminal, change into the Pivot repository root. Prerequisites
are Rust/Cargo with rustfmt and Clippy, a working native linker/C toolchain, and
Python 3.8+. No .NET SDK is needed for these Rust/IPC checks.

```sh
export CARGO_TARGET_DIR="$PWD/target/macos"
cargo fmt --all -- --check
cargo clippy --locked --workspace --all-targets -- -D warnings
cargo test --locked --workspace
python3 tests/ipc/verify.py --output-dir artifacts/ipc-macos
```

The separate target directory prevents shared-folder build collisions with Windows
or Linux. The IPC script discovers Cargo's actual output directory automatically.
It starts a unique local daemon, runs CLI/protocol checks, and cleans up the process
and endpoint. Inspect `artifacts/ipc-macos/results.txt` and `daemon.log` afterward.
The Unix permission test also runs as part of `cargo test` on macOS.

With a native .NET 10 SDK, full CI can also run as `python3 scripts/ci.py`. Use a
separate local checkout for full multi-OS CI, since .NET bin/obj outputs are currently
shared by path and should not be built concurrently across operating systems.
Use local APFS storage for source and build outputs on macOS. The SMB-mounted
workspace failed in `GenerateDepsFile` with a file-sharing error on
`Pivot.Client.dll`; a fresh local APFS copy passed the unchanged pipeline.
A sandbox that forbids Unix socket binding must allow that capability for the
Rust socket test and IPC harness; do not weaken endpoint checks to bypass it.

### Actual macOS run — 2026-10-18

macOS 27.0.1 (26A434), native ARM64; Apple Clang 21.0.0; Python 3.9.6;
Rust 1.101.0-nightly (8006091eb, 2026-10-17); Cargo 1.101.0-nightly
(3960fa69b, 2026-10-12); .NET SDK 10.0.402.

Full `python3 scripts/ci.py` passed from the isolated APFS source copy
`/private/tmp/pivot-macos-validation-20261018T171109Z/source`, with
`CARGO_TARGET_DIR=/private/tmp/pivot-macos-validation-20261018T171109Z/target/macos`.
The copy excluded `.git`, `target`, `artifacts`, `bin`, and `obj`.
All 41 Rust tests passed, including Unix endpoint permissions, same-user peer
credentials, endpoint exclusivity, and replacement-safe cleanup. The five-project
.NET Release solution build had zero warnings/errors; all 10,099 decimal vectors
and 58 protocol decimal layouts passed. Real daemon/CLI integration passed,
including malformed traffic survival, concurrent clients, and graceful cleanup.
No product code changes were required. ADR-0005 remains Proposed.

Report: `artifacts/macos-validation-20261018T171109Z/ci/20261018T171247760372Z/summary.json`.
The run directory contains detailed gate logs, parity reports, and
`ipc/results.txt` plus `ipc/daemon.log`; the latter records READY, expected
malformed-client rejections, and STOPPED. Earlier failed reports are retained:
`20261018T171120582806Z` (sandbox denied socket binding) and
`20261018T171208467020Z` (SMB .NET file-sharing failure).
Native AOT, GUI execution, and actual cross-user denial were not run.

## WSL2 rerun

From a PowerShell terminal on this development machine:

```powershell
wsl.exe -d Ubuntu -- bash -lc 'cd /mnt/j/coding/pivot && export CARGO_TARGET_DIR=/mnt/j/coding/pivot/target/linux-wsl && cargo fmt --all -- --check && cargo clippy --locked --workspace --all-targets -- -D warnings && cargo test --locked --workspace && python3 tests/ipc/verify.py --output-dir artifacts/ipc-linux-wsl'
```

Paths and distro name are specific to this machine. Dependencies and Linux compiler
tools live inside WSL, while build products remain in the ignored Linux target
subdirectory. Windows builds continue using their existing target directory.
