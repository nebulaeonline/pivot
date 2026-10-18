# Pivot Handoff — macOS validation

Date: 2026-10-18
Branch / commit: uncommitted
Status: Complete

## Objective

Complete the documented macOS validation without beginning another milestone.

## Implemented

No product code changes were needed. Updated current coverage documentation and
the verification evidence in ADR-0005; its status remains Proposed.

## Environment and isolation

Native ARM64 macOS 27.0.1 (26A434), Apple Clang 21.0.0, Python 3.9.6,
rustc 1.101.0-nightly (8006091eb 2026-10-17), Cargo 1.101.0-nightly
(3960fa69b 2026-10-12), and .NET SDK 10.0.402.

Initial Rust formatting and strict Clippy checks passed in the shared workspace
with `CARGO_TARGET_DIR="$PWD/target/macos"`. Full CI then used isolated source
copies excluding `.git`, `target`, `artifacts`, `bin`, and `obj`, preserving
existing Windows/Linux .NET outputs and reference reports.

The first CI attempt failed because the execution sandbox denied Unix socket
binding (`Operation not permitted`). With sandbox permission, all Rust tests
passed, but the .NET solution build failed in GenerateDepsFile with an IOException
opening Pivot.Client.dll on the SMB-mounted source copy. A fresh local APFS source
copy passed the unchanged full pipeline. This establishes a working local macOS
build path; it does not establish reliable .NET builds over SMB.

## Tests actually run

Successful full run, outside the socket-restricting sandbox:

```sh
cd /private/tmp/pivot-macos-validation-20261018T171109Z/source
export CARGO_TARGET_DIR=/private/tmp/pivot-macos-validation-20261018T171109Z/target/macos
python3 scripts/ci.py --output-dir /Volumes/RobMac/coding/pivot/artifacts/macos-validation-20261018T171109Z/ci
```

- Rust formatting — PASS.
- Locked workspace Clippy, all targets, warnings denied — PASS.
- Locked workspace tests — PASS: 41 tests, including Unix private permissions,
  same-user peer credentials, endpoint exclusivity, and replacement-safe cleanup.
- .NET Release solution build with warnings as errors — PASS: five projects,
  zero warnings/errors.
- Release decimal parity — PASS: 10,099 vectors, zero value/error failures and
  zero scale differences.
- Generated Rust/C# Protobuf parity — PASS: request/result exchange and 58
  decimal representation layouts.
- Real local daemon/CLI integration — PASS: discovery, descriptions, input
  validation/defaults, decimal outputs, revisions, arithmetic errors, concurrent
  clients, malformed/session traffic survival, graceful shutdown, endpoint cleanup.

Report:
`artifacts/macos-validation-20261018T171109Z/ci/20261018T171247760372Z/summary.json`.
Detailed logs and parity outputs are alongside. `ipc/results.txt` reports PASS;
`ipc/daemon.log` contains READY, expected invalid-client rejection logs, and STOPPED.
Earlier failed reports remain in the same CI root: `20261018T171120582806Z`
(sandbox) and `20261018T171208467020Z` (SMB).

## Decisions and remaining coverage

ADR-0005 remains Proposed. No architecture, API, or milestone changes.
Native AOT publish/run, Avalonia GUI execution, and actual cross-user endpoint
denial remain untested. Only native ARM64 macOS execution was verified.
Use a separate local APFS checkout/copy for future macOS full CI runs.
