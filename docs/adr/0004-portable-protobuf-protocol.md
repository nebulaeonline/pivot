# ADR-0004: Portable Protobuf model invocation protocol

Status: Accepted
Date: 2026-10-18

## Context

Pivot needs portable discovery, introspection, and invocation. Runtime NodeIds,
PortIds, and revisions are process-local and cannot serve as durable client IDs.

## Decision

Use proto3 messages independent of transport, with prost-generated Rust and
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
private. Semantic validation remains separate from parsing. The initial contract incorporates the authorized review refinements: numeric
provenance equality, explicit Submitted/Defaulted input source, mandatory handshake
and bootstrap errors, advertised host limits separate from model capacity, and
invocation identities that do not imply stored results. An in-process registry and
adapter now implement this boundary. Authentication, networking, result retention,
publishing, and persistence remain outside this decision.

## Verification / reconsideration trigger

Require codec/validation tests and generated Rust/C# interoperability. The review refinements were reconciled in schemas, validation, host sessions,
and tests before accepting this initial architecture. Revisit durable identity
allocation and result retention before implementing those capabilities. Actual Native AOT publish/run tests precede any AOT support claim.
