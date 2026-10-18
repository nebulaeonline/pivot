# Pivot Handoff — Portable protocol proposal

Date: 2026-10-18
Branch / commit: uncommitted
Status: Complete

## Objective

Implement the owner-authorized protocol design slice recommended by the parent
review: proposed transport-independent contract, ADR-0004, schemas, and tests.
The design remains proposed; no external contract was silently finalized.

## Implemented

- `docs/architecture/portable-protocol.md`: identity/version/capability/error/value
  proposal and outstanding owner decisions.
- `docs/adr/0004-portable-protobuf-protocol.md`: Proposed.
- `crates/pivot-protocol/proto/pivot/v1/protocol.proto`: negotiation, discovery,
  descriptions, invocation, values, results/provenance, errors, and correlation.
- Rust generated types and bounded validation, including exact interface validation.
- Six contract tests plus Rust/C# generated codec interoperability fixtures.
- Local CI now runs the protocol fixture gate and builds the C# harness.

## Interface / semantics

External IDs are opaque and distinct from process-local graph/port/revision handles.
An eventual host registry must allocate and resolve them. Describe/invoke reference
an exact revision. Invocation uses public portable port IDs discovered from metadata.
Decimals carry integer limbs, scale, and sign without float conversion.

Only exact v1.0 and decimal execution capabilities are enabled. Float64 is reserved
and rejected. Unknown future value variants fail closed. Responses echo request IDs
and distinguish request-level errors from per-output calculation failures. Invocation
results return effective public inputs and requested outputs. GetInvocationResult is
reserved but unsupported until result retention and authorization are designed.

## Tests actually run

- `cargo test -p pivot-protocol` — PASS: six new tests, including 174 decimal
  representation round trips across scales, signs, zero, and coefficient boundaries.
- `cargo clippy --workspace --all-targets -- -D warnings` — PASS.
- `python tests/protocol-parity/verify.py` — PASS: generated binary request exchange,
  value/error results, unknown-value handling, and 58 .NET decimal limb layouts.
- `python scripts/ci.py` — PASS: 28 Rust tests, formatting, strict Clippy, .NET Release
  solution build, 10,099 decimal vectors, and the new protocol compatibility gate.
- Native AOT publish/run — NOT RUN; static generated C# codecs are used, but actual
  AOT compatibility is not claimed.

Initial lint findings were corrected before full CI passed. Reports are under
`artifacts/ci/20261018T160522325807Z/`.

## Decisions made or proposed

- **Proposed, not accepted:** Protocol Buffers; opaque external IDs with host registry
  mapping; exact revision selection; limits; negotiation; synchronous invocation;
  provenance representation; disabled result lookup.
- **Implementation choices:** prost 0.14.4, prost-build 0.14.4, bundled protoc via
  protoc-bin-vendored 3.3.0; Google.Protobuf 3.36.2 and private Grpc.Tools 2.84.0.
  No gRPC runtime, daemon, or GUI dependency was added to the Rust protocol crate.

## Open issues / risks

No host registry, allocators, engine adapter, handshake session state, transport
framing, authorization, cancellation, result retention, publishing, persistence,
REPL, or Avalonia integration is implemented. C# tests cover codecs, not a duplicate
semantic validator. Snapshot lifetime and execution budgets remain future hosting
work. Protobuf unknown metadata is skipped by prost, not losslessly relayed.
Numeric policy stays provisional; this transport does not broaden parity claims.

## Questions for ChatGPT / owner

Before accepting ADR-0004, review opaque identity lifetimes and allocation, exact
revision selection, portable port mapping, bootstrap-version behavior, 1 MiB / 1,024
port limits, exact-representation provenance, and synchronous result responses.
Keep result retrieval disabled until authorization and retention are specified.

## Next recommended slice

Review ADR-0004. Once approved, implement the host adapter and local IPC/session
design without exposing runtime identities. Do not begin networking before review.
