# Portable protocol contract proposal

Status: Proposed; ADR-0004 is not accepted.

Schema: `crates/pivot-protocol/proto/pivot/v1/protocol.proto`. Messages and Rust
validation are transport-independent. The protocol crate has no engine dependency.

## Portable identities

| ID | Proposed scope and lifetime |
| --- | --- |
| model_id | Logical model, stable across restarts when durably registered |
| revision_id | Immutable definition/interface revision within a model; never reused for changed meaning |
| port_id | Public port within a model; retained across label changes, new identity for replacement |
| invocation_id | Host-generated calculation identity; later retrieval requires authorization |
| request_id | Client correlation token unique among outstanding requests on a connection |

IDs are opaque ASCII alphanumeric plus `.`, `_`, `-`, 1–128 bytes. Registry-generated
UUID text is a candidate allocator, not a required format. A future registry maps
portable IDs to runtime objects; no runtime handles are serialized. Allocators and
durable registration are not implemented. Published versions and runtime instances
remain distinct future concepts. Schema alone does not provide persistence.

ListModels supplies available model/revision pairs. DescribeModel and InvokeModel
require exact pairs, never implicit latest. Unknown/unavailable revisions fail
without substitution. Port labels resolve to portable IDs through DescribeModel;
invocation uses these IDs, not internal graph IDs. Publishing remains unimplemented.

## Negotiation

Handshake lists supported exact versions and required/optional capabilities. Only
v1.0 is implemented. No common version returns UnsupportedVersion. Unknown required
capabilities fail; unknown optional capabilities are ignored. Advertised capabilities
are `decimal96`, `model-discovery`, and `synchronous-invocation`.

Handshake requests may omit envelope version; their supported-version list is
authoritative. Accepted responses use v1.0 in envelope and selection. Subsequent
requests require v1.0. This implementation's errors also use v1.0 envelopes, even
for unsupported-version failures; review generic bootstrap before another major.

Responses echo request_id. Helpers validate messages and negotiation individually.
The future session layer must enforce handshake ordering, negotiated capabilities,
and unique outstanding IDs. Correlation is not idempotency, replay protection,
retry safety, authentication, or exactly-once execution.

## Operations

| Operation | Initial behavior |
| --- | --- |
| Handshake | Version/capabilities and maximum message size |
| ListModels | Bounded discovery with continuation tokens |
| DescribeModel | Public port IDs, labels, types, required/default declarations |
| InvokeModel | Synchronous isolated calculation returning InvocationResult |
| GetInvocationResult | Reserved; rejected until a future result-lookup capability |

Page size defaults to at most 100 when zero; maximum 100. Tokens are opaque and
limited to 512 bytes. Pagination consistency and result retention/expiry need
service decisions. No token/result store exists. Retrying invocation may create
another calculation. Cancellation and asynchronous completion remain future work.

## Values

Value is a oneof. Decimal128 contains fixed32 lo/mid/hi unsigned significand limbs,
scale 0–28, and sign. This is a logical .NET-compatible container, **not IEEE-754
decimal128**, a native memory layout, or a fixed 16-byte Protobuf message. Every
96-bit significand is valid. Scale above 28 is rejected. Zero/default limbs are
valid; a missing Value variant is not a decimal zero.

Encoded scale/sign are preserved for transport. Numeric arithmetic retains current
Pivot normalization and precision-reduction rules. No decimal uses float conversion.
Signed zero may round-trip but its sign is not a numeric semantic guarantee.
No per-invocation rounding-policy override is introduced.

Float64 is schema-reserved and rejected as UnsupportedValueType in v1.0. Unknown
type enums or future oneof-only values fail closed. New executable types require
negotiation and specified coercion rules. Finite-precision division and possible
underflow remain documented numeric behavior; matching Pivot helpers does not
promise universal native System.Decimal equivalence.

## Invocation, results, and errors

Validate against the exact revision's description. All required ports are supplied;
optional ports have explicit defaults. Validate every binding, even if unused.
Repeated input/output IDs fail instead of becoming last-wins maps. Empty selected
outputs still require complete valid inputs. Public declarations use unique IDs
and labels across inputs/outputs.

Request errors reject a request: malformed input, unsupported version/capability/type,
unknown ports, duplicates, missing inputs, unavailable revisions, resource limits,
authorization, or internal failure. Per-output calculation errors are overflow,
division by zero, dependency failure, or invalid expression. Independent outputs
can succeed. Public messages expose no internal graph identity or privileged trace.

InvocationResult includes invocation ID, model/revision, all effective public input
values including defaults, and exactly the selected outputs once each. Provenance
validation currently requires exact declared representations, including scale/sign.
Internal inputs remain hidden. Effective inputs can be sensitive; future transport
and retrieval need the same authorization boundary as invocation.

decode_response checks correlation, envelope version, operation shape, outputs,
values, and basic provenance structure. validate_result additionally verifies the
interface and effective input values. Use both for full invocation validation.
No engine adapter or host executes these messages yet.

## Limits and evolution

Maximum encoded message: 1 MiB; 1,024 input and 1,024 output entries. Handshakes
allow 32 versions and 64 capability entries. Model names are at most 256 bytes,
port labels 128, error messages 1,024. Byte bounds precede Protobuf decode. Future
transports must check frame length before allocating payloads. Framing, authorization,
compute budgets, and cancellation remain unselected service concerns.

After approval, evolve additively; reserve removed field tags/names and never reuse
them. Semantic changes require versions/capabilities. Unknown metadata may be skipped.
prost drops unknown fields on re-encode, so it is not a lossless proxy. Future value
variants fail closed. Normal Protobuf singular/oneof duplicate-field rules apply;
duplicate-binding rejection refers to repeated application-level port entries.

Sources: [proto3 evolution](https://protobuf.dev/programming-guides/proto3/),
[schema best practices](https://protobuf.dev/best-practices/dos-donts/),
[prost generated messages](https://docs.rs/prost/0.14.4/prost/).

## Verification and owner decisions

Rust tests cover decimal encoding, malformed data, limits, versions, capabilities,
missing/unknown/duplicate ports, required inputs/defaults, future values, correlation,
successful outputs, arithmetic errors, and provenance. Generated C# fixtures exchange
requests/results/errors and check 58 decimal layouts. Static generated serializers
are used; actual Native AOT publishing/execution is not tested or claimed.

Review portable ID lifecycle, revision selection, port mapping, bootstrap rules,
limits, representation-sensitive provenance, and synchronous result semantics before
accepting ADR-0004. GetInvocationResult remains disabled pending authorized lifetime,
retention, and expiry policies. No network implementation is authorized by this proposal.
