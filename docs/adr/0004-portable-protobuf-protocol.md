# ADR-0004: Portable Protobuf model invocation protocol

Status: Proposed
Date: 2026-10-18

## Context

Pivot needs portable discovery, introspection, and invocation. Runtime NodeIds,
PortIds, and revisions are process-local and cannot serve as durable client IDs.

## Decision

Propose proto3 messages independent of transport, with prost-generated Rust and
Google.Protobuf-generated C#. Do not require gRPC or serialize runtime objects.
Use opaque external model, revision, public port, invocation, and request IDs.
Require exact revision selection and explicit version/capability negotiation.

Decimals use 96-bit integer limbs, scale, and sign without float conversion.
Separate request failures from per-output calculation failures. Bound message
sizes and validate decoded data before use. Decimal is the only enabled numeric
type; float64 remains schema-reserved. The full proposal is in
[the protocol design](../architecture/portable-protocol.md).

## Alternatives considered

- JSON: needs additional strict numeric conventions and schema rules.
- Direct runtime serialization: couples clients to implementation details.
- gRPC-first: adds service/transport decisions before message validation.
- Shared memory: premature for the first invocation contract.

## Consequences

Code generation adds build dependencies. Rust bundles protoc; C# build tooling is
private. Semantic validation remains separate from parsing. No external contract
is frozen until approval. No network, registry, authentication, result storage,
publishing, or persistence implementation is included.

## Verification / reconsideration trigger

Require codec/validation tests and generated Rust/C# interoperability. Review
identity lifecycle, bootstrap negotiation, limits, and result retention before
acceptance. Actual Native AOT publish/run tests precede any AOT support claim.
