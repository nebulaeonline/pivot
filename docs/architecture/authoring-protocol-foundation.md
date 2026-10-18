# Authoring protocol foundation

Implemented portion of Accepted ADR-0007: production schema, generated codecs,
bounded structural/semantic validation, opt-in negotiation, typed intent and
cross-language/legacy compatibility tests. This document records the foundation
slice; the subsequent [Stage A adapter](serialized-authoring-adapter.md) now adds
serialized in-process dispatch. [Stage B](daemon-authoring.md) adds explicitly
enabled local daemon authoring. Foundation scope and test results below are historical.

## Schema allocations and generation

- `pivot/v1/types.proto`: Decimal128, Value, ValueType, ModelRef and
  CalculationErrorCode retain their original fully qualified names, enum values,
  fields and wire encodings.
- `pivot/v1/protocol.proto`: existing invocation envelope/operations preserved;
  Request.operation.authoring and Response.outcome.authoring allocate tag **20**.
- `pivot/authoring/v1/authoring.proto`: request operations **1–12** and result
  alternatives **1–13** in declared order. Edit alternatives **1–10** retain the
  reviewed order. The production schema is the authoritative allocation list.
- ReadDiagnostics field **3** is reserved: operation-receipt detail paging is
  deliberately deferred. Current diagnostic reads identify an exact working head.

Rust prost generation exports `v1` plus `authoring::v1`; C# uses existing
`Pivot.Protocol.V1` plus `Pivot.Protocol.Authoring.V1`. The parity project compiles
all three protos with an explicit common root. Generated output is build-owned.
The production descriptor set is embedded for schema-aware validation. No import
cycle; historical docs/proposals schema is not a generation input.

## Presence and patch semantics

NodeDefinition must contain an input or formula oneof. Present empty formula text
is valid editable draft content; missing alternative is invalid. Revision message
and scope IDs are required, but omitted sequence means valid zero. Optional
location offsets/indexes preserve presence, including present zero.

SetInputPort requires a port reference and at least one of node/requirement. Omitted
update fields mean unchanged; supplied values replace. SetOutputPort requires a
target because it has only one updateable field. Rename is a separate operation.
Typed mutation intent preserves this presence distinction.

The current Rust edit service accepts full target/requirement replacements. A future
adapter/service patch primitive must resolve omitted fields from its staged graph
inside the same atomic batch; reading a current value and resubmitting it externally
would violate partial-update semantics. This foundation does not implement that adapter.

## Validation boundary

Future untrusted entry point: `foundation::decode_envelope(raw, Limits)`.

1. Reject oversized raw frame/payload before generated decoding.
2. Recursively walk embedded descriptors, bounding depth, total field count,
   text length, ordered edit count, input/output count, names and cursor length.
3. Reject unknown fields, wrong wire types, truncated/overflowed data, duplicate
   singular fields, conflicting/missing oneofs, unknown/unspecified enums, invalid
   UTF-8, noncanonical UUID strings and unsupported values.
4. Generated decode; semantic revision-scope and duplicate evaluation-binding checks.
5. Future adapter: authentication-context authorization, receipt reconciliation and
   transactional admission. No state lookup or authorization is done by this library.

All authoring request alternatives currently use this strict structural boundary.
Existing invocation decoding remains unchanged. Typed authoring responses can be
validated after ordinary generated decoding; additive unknown response fields are
not interpreted as mutations. Fragment validation is useful for codecs/tests, but
is not a substitute for raw envelope validation on incoming operations.

Legitimate repeated edits remain ordered, not mistaken for singular duplicates.
Field order is immaterial. Valid nonminimal varints are accepted if they represent
the same permitted value; malformed/overflowing varints are rejected. Fixed-width
decimal limbs, scale <=28 and sign are preserved exactly; float64 remains unsupported.

Default validation policy: 1 MiB payload, depth 32, 65,536 traversed fields, 128
edits, 4,096-byte text fields, 1,024 input overrides/output IDs, 128-byte identifier
names and 512-byte cursor. Limits are configurable policy. The existing transport
still enforces its 1 MiB ceiling. ResourceLimits metadata uses uint64 and imposes
no permanent small node/port/model ceiling; a ten-million-node advertised limit is
covered by tests. Future bulk transport needs separate negotiation.

## Negotiation and intent

`foundation::negotiate` explicitly opts into v1.1/working-models-v1 with decimal96;
it selects v1.0 when that is the offered supported version. Unknown required
capabilities fail. Default `validation::negotiate`, daemon and host sessions retain
their invocation-only behavior and do not advertise or dispatch authoring.

After validated decode, `MutationIntent` captures Create/Edit/Publish typed content,
including instance, operation ID, revision, ordered edits and exact value layout.
Request correlation and raw encoding order are excluded. The generated messages'
presence and decimal limb/sign/scale equality implement exact representation, not
numeric-equivalence comparison. This foundation does not store/reconcile receipts;
management integration remains separate work.

## Receipt diagnostics and remaining adapters

The existing in-process ledger retains complete edit/publication diagnostic vectors,
typed requests, revision and outcomes, but has no dedicated retained-byte admission
budget. Do not promise detailed historical paging from a summary or newer head.
Production wire outcomes expose original diagnostic summaries and provenance only.
Detailed receipt paging requires a later extension with complete immutable ordered
diagnostics, revision, IDs, source offsets/text/categories, lifetime guarantees and
storage reservations before any successful mutation commit. Exhaustion rejects
admission; never silently truncate or evict promised details.

Working-head paging, signed cursors, connection-bound attachment cleanup and bounded
idempotent Close are accepted adapter requirements, not implemented by codecs.
Service Info authorization, unified daemon registry/bootstrap and portable failure
mapping likewise remain deferred. Database-provider direction remains in
[persistence providers](persistence-providers.md), outside ADR-0007 implementation.

## Compatibility evidence

`tests/protocol-parity/legacy/protocol.proto` is frozen from production before type
extraction. Its C# codec is built in an isolated project. A pre-extraction compiled
binary was also retained locally and successfully rerun after extraction. The
current host processes a frozen-codec request; that codec decodes and checks the
response using v1.0 types. This is serialized in-process host interoperability,
not a C# IPC client or Native AOT validation claim.

The fixture harness compares every existing field name, tag, scalar/message type,
label, type reference, oneof membership and proto3 presence marker against the old
descriptor, and compares existing enum values. Generated file/descriptor ownership
changes are intentional; wire-level identities remain stable.

118 authoring fixtures cover every message and every defined oneof alternative,
including every request/result/edit alternative. C# decodes/re-encodes Rust fixtures;
Rust verifies returned bytes and structure. A C#-constructed partial port update
also validates in Rust with target absent. Twelve focused tests exercise raw negative
wire data, presence, exact intent, budgets, scope, response correlation and versioning. Adapter behavior,
signature verification, transactional acceptance, fuzzing and network-authoring
failure tests are not implied by these codec tests.
