# Pivot Handoff — WSL2 Linux validation

Date: 2026-10-18
Branch / commit: uncommitted
Status: Complete

## Objective

Use the available WSL2 installation to verify Linux and provide reproducible checks
for the owner's Mac with access to the same project directory.

## Implemented

- Executed native Linux builds/tests in Ubuntu WSL2 from `/mnt/j/coding/pivot`.
- Isolated Linux build outputs with `CARGO_TARGET_DIR=target/linux-wsl`.
- Fixed a Unix-only Clippy collapsible-if finding in socket cleanup.
- Added a Unix permission/peer/cleanup test: insecure directory rejection, mode-0600
  sockets, same-user peer credentials, endpoint exclusivity, insecure socket rejection,
  symlink parent rejection, normal cleanup, and preserving replacement files.
- Added `docs/platform-validation.md` with Mac and WSL commands; updated current
  status/security documentation. Historical handoffs remain unchanged.

## Interface / semantics

No protocol, CLI, numeric, or engine behavior changed. The Unix cleanup refactor
preserves its existing device/inode guard. Linux endpoints remain in Linux `/tmp`,
outside the shared Windows-mounted checkout. Windows target outputs are unaffected.

## Tests actually run

Inside Ubuntu WSL2, with the separate Linux target directory:

- `cargo fmt --all -- --check` — PASS.
- `cargo clippy --locked --workspace --all-targets -- -D warnings` — PASS.
- `cargo test --locked --workspace --quiet` — PASS: 41 tests, including the new Unix test.
- `python3 tests/ipc/verify.py --output-dir artifacts/ipc-linux-wsl` — PASS: real Linux
  daemon/client round trips, malformed traffic survival, isolation, and cleanup.

The initial Linux Rust suite and IPC checks also passed before adding the Unix test.
Strict lint initially failed on the Unix-only nested condition, then passed after
the correction. Linux Rust/Cargo is 1.101.0-nightly; kernel reports
6.18.54.1-microsoft-standard-WSL2, x86_64. WSL runs as the normal user, not root.

Report: `artifacts/ipc-linux-wsl/results.txt`; daemon log alongside it.
Full Linux .NET CI — NOT RUN: dotnet exists but no SDKs are installed.
macOS execution — NOT RUN: no connected Mac execution session is available here.

## Decisions made or proposed

No new architecture decision. ADR-0005 remains Proposed; this adds Linux evidence.
Use the same Rust/IPC harness on macOS with isolated `target/macos` outputs.

## Open issues / risks

WSL2 tests are Linux tests, not macOS verification or a native Linux deployment test.
Actual other-user denial and Native AOT remain untested. macOS requires running the
documented commands locally. Shared .NET build directories should not be reused
concurrently by different OSes; use a separate checkout for full multi-OS .NET CI.

## Questions for ChatGPT / owner

Run the documented Mac commands and review its results/log. No blocking question
for Linux validation.

## Next recommended slice

Complete macOS validation, then review ADR-0005 using the platform evidence.
