# Local CI

Run from the repository root:

```text
python scripts/ci.py
```

This is an on-demand local pipeline. It requires no CI server, container runtime,
cloud account, or Python packages. It can later be called unchanged by a hosted
runner or local build service.

## Prerequisites

- Python 3.8 or newer, available as `python`.
- Rust/Cargo with the `rustfmt` and `clippy` components.
- .NET 10 SDK. `global.json` selects an installed stable .NET 10 feature band.
- A C# desktop build environment appropriate for the host OS.

Package access is needed on the first run. Builds use normal Cargo and NuGet
caches. Rust tooling is not yet pinned; the actual versions are recorded in logs.
Cargo gates use the committed lockfile. NuGet dependencies are not yet lockfile
enforced, so the pipeline is not a hermetic build.

## Gates

1. Record Rust, Cargo, .NET, and Python versions.
2. Check Rust formatting without modifying source files.
3. Run Clippy across workspace targets with warnings treated as errors.
4. Run all Rust workspace tests, including documentation tests.
5. Build the full .NET solution in Release with warnings treated as errors.
6. Run all 10,099 Rust/C# decimal compatibility cases using the Release C# harness.
7. Generate and exchange Rust/C# Protobuf protocol fixtures.
8. Start a temporary demo daemon and test IPC/CLI/REPL round trips, extended idleness,
   malformed-client survival, and explicit reconnect without ambiguous invocation replay.

The runner stops at the first failed command and returns a nonzero exit status.
It streams output to the console and preserves each command's output in a log.
No commits, hooks, deployments, or background services are installed.

## Reports

Each invocation creates a unique timestamped directory under `artifacts/ci/`:

```text
artifacts/ci/<run>/
  summary.json
  rust-format.log
  rust-lint.log
  rust-tests.log
  dotnet-build.log
  decimal-parity.log
  decimal-parity/
    results.txt
    backend-baseline.txt
```

The summary records command arguments, exit codes, durations, and log paths for
executed steps. An early failure leaves later gates unexecuted. Python/platform
information is included. Missing required tools produce a failed summary.
Reports are ignored by Git and accumulate until manually removed.

To choose another report root:

```text
python scripts/ci.py --output-dir artifacts/my-ci
```

The compatibility checker now accepts `--output-dir` and `--configuration`.
CI writes its generated reports into the run directory, preserving the reference
reports in `tests/decimal-parity/`.

## Current coverage limits

The full pipeline is verified on Windows with .NET 10.0.402 and Ubuntu WSL2 Linux
with .NET 10.0.112, and macOS 27.0.1 ARM64 with .NET 10.0.402, using installed
Rust nightly toolchains. Linux and macOS used isolated source/build outputs.
On macOS, use local APFS storage: the SMB-mounted source copy failed with a
.NET file-sharing error; the local APFS copy passed all gates. See
[platform validation](platform-validation.md) for reproducible commands.
GUI execution and Native AOT publishing are not gates yet. There is no automatic
trigger: run the command before handoff or wire it into future CI hosting.
