# Pivot Handoff — Linux .NET validation

Date: 2026-10-18
Branch / commit: uncommitted
Status: Complete

## Objective

Run the remaining Linux .NET checks after the owner installed .NET 10 in WSL2.

## Implemented

No product code changes were needed. Verified SDK 10.0.112 and ran the complete
existing CI pipeline natively in Ubuntu WSL2. Updated current coverage documentation.

## Interface / semantics

Used an isolated source copy under `artifacts/linux-dotnet-20261018T164648Z/source`,
excluding Git, build products, and artifacts. Linux Rust outputs used
`target/linux-wsl`; Linux .NET bin/obj files stayed inside the source copy.
Windows build products and reference reports were not overwritten.

## Tests actually run

In the isolated Linux source copy:

- `python3 scripts/ci.py --output-dir /mnt/j/coding/pivot/artifacts/linux-dotnet-20261018T164648Z/ci`
  — PASS, including every gate below.
- `cargo fmt --all -- --check` — PASS.
- `cargo clippy --locked --workspace --all-targets -- -D warnings` — PASS.
- `cargo test --locked --workspace` — PASS: 41 Linux Rust tests.
- `dotnet build Pivot.slnx -c Release --disable-build-servers -warnaserror` — PASS:
  all five .NET projects, zero warnings/errors.
- Decimal compatibility harness in Release — PASS: 10,099 vectors, zero value/error
  mismatches and zero scale differences.
- Generated Rust/C# Protobuf harness — PASS: binary request/result exchange and
  58 decimal representation layouts.
- Local IPC integration — PASS: native Linux daemon/client round trips, malformed
  traffic survival, concurrency, and cleanup.

Report:
`artifacts/linux-dotnet-20261018T164648Z/ci/20261018T164652188086Z/summary.json`.
Detailed build logs, decimal reports, protocol fixtures, and IPC logs are alongside.

## Decisions made or proposed

No architecture or API changes. ADR-0005 remains Proposed. Build isolation is a
validation implementation choice, not a new repository layout requirement.

## Open issues / risks

Native AOT publish/run, Avalonia GUI execution, macOS validation, and actual
cross-user endpoint denial remain untested. Passing Linux CI verifies the existing
build and compatibility gates, not these additional capabilities.

## Questions for ChatGPT / owner

No blocking questions. The Mac can now use the same full CI entry point once its
prerequisites are installed, preferably from a separate local checkout.

## Next recommended slice

Complete Mac validation or explicitly scope a Native AOT publish-and-run check.
