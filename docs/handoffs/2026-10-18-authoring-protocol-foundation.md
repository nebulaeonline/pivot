# Authoring protocol foundation — implementation handoff

Date: 2026-10-18
Status: Implemented; ready for review. Live authoring remains disabled.

## Implemented

- Incorporated required refinements and marked ADR-0007/index Accepted.
- Added production authoring schema; extracted shared values/identities with
  original fully qualified names and wire layouts intact. Generated Rust/C# codecs.
- Allocated Request.operation.authoring / Response.outcome.authoring tag **20**.
  Production authoring request tags **1–12**, response tags **1–13**, edit tags
  **1–10** follow schema declaration order. Production schema is authoritative.
- Reserved ReadDiagnostics field **3** and deferred detailed receipt diagnostic
  paging. Current wire diagnostic reads pin the exact working revision; mutation
  outcomes expose original summaries. No reconstruction from a newer head.
- Added explicit opt-in v1.1/working-models-v1 negotiation API. Default daemon/host
  negotiation remains v1.0/invocation-only and rejects authoring requests.
- Added descriptor-driven bounded raw preflight before generated decoding,
  structural/identity/value validation and semantic scope/duplicate-binding checks.
- Added typed Create/Edit/Publish intent comparison, preserving exact decimal
  representation, field presence, ordered edits and source text.
- Added response correlation/alternative and exact snapshot revision checks;
  read-only generated response decoding tolerates additive unknown metadata.
- Added Rust/C# authoring fixtures, frozen-v1.0 schema/codec compatibility tests
  and unchanged-codec/current-host serialized invocation checks to local CI.

See [foundation architecture](../architecture/authoring-protocol-foundation.md),
[accepted ADR](../adr/0007-versioned-authoring-protocol.md) and
[production schema](../../crates/pivot-protocol/proto/pivot/authoring/v1/authoring.proto).

## Review refinements resolved

1. **Port presence:** SetInputPort omits unchanged node/requirement fields and
   requires at least one replacement. SetOutputPort has one replacement field, so
   a target is required. Empty formula is distinct from missing definition; revision
   zero is valid; optional location zero preserves presence.
2. **Preflight boundary:** raw bounded bytes -> recursive descriptor validation ->
   generated decode and semantic checks -> future authorization/receipts/admission.
   Unknown fields, singular duplicates, conflicting/missing oneofs, invalid enums,
   wire types, UTF-8, lengths and overflow fail before mutation decoding can hide
   intent. Ordered repetition, reordered fields and permitted nonminimal varints
   remain valid. Limits cover depth, fields, text, edits, input/output and frame size.
3. **Receipt detail:** explicitly deferred instead of making an unsupported retention
   guarantee. Existing in-process receipts retain diagnostics but lack a dedicated
   retained-byte reservation policy. Future detail paging requires immutable full
   detail and reserved storage before commit, with no truncation/eviction of promises.
4. **Model ceilings:** configurable metadata uses uint64, not fixed wire maxima;
   validation accepts a ten-million-node advertised limit. Initial transport stays
   1 MiB. Future bulk transport requires separate negotiation.
5. **Compatibility:** shared type extraction tested against old descriptors and
   the separately generated old codec, plus a binary built before extraction.
6. **Attachments:** connection-bound ownership, independent rights, bounded
   idempotent Close bookkeeping and disconnect cleanup remain accepted requirements
   for the next adapter slice; no fake lifecycle implementation is claimed here.

## Tests actually executed

Final full Windows CI passed:
`artifacts/ci/20261018T203803594562Z/summary.json`.

- Formatting and locked workspace Clippy/all-targets with warnings denied.
- Locked workspace tests: **102 passed**, including **12 foundation tests**.
- .NET 10 Release solution build with warnings as errors.
- Numeric parity: **10,099 vectors**, zero failures/scale differences.
- Existing protocol fixtures: **58 decimal layouts**.
- **118 authoring fixtures**, covering every message and defined oneof alternative,
  including all requests, responses and edits; C# decode/re-encode -> Rust verification.
- A C#-constructed partial input-port update validates in Rust with target absent.
- Frozen-v1.0 C# request -> current ModelHost -> frozen-v1.0 response decoding,
  checking invocation version and exact computed decimal value.
- Old descriptor comparison for every existing field name/tag/type/label/reference/
  oneof/presence marker, and existing enum values.
- Actual daemon/CLI/REPL IPC regression integration.

An additional pre-extraction compiled v1.0 C# binary was built and retained at
`artifacts/legacy-v1-binary/LegacyProtocol.dll`, then successfully rerun after
extraction against current fixture files. This complements the reproducible frozen
schema/current-host test; it is not a C# IPC or Native AOT claim. Artifacts are local,
not source-controlled. `tests/protocol-parity/legacy/protocol.proto` is the tracked
frozen source for reproducible compatibility checks.

Markdown links/UTF-8 checks and `git diff --check` passed. No Linux/macOS rerun,
fuzzing, authoring IPC fault injection or Native AOT validation in this slice.

## Scope and remaining ambiguities

- The foundation validates wire structure and portable semantic shape, not resource
  existence, graph transaction validity, policy grants or persistence. Future adapters
  must route raw requests through it before receipt reconciliation and service calls.
- Current Rust SetInputPort is a full replacement API. Add native staged patch
  support during adapter integration; preserve original patch presence in receipt
  intent and resolve omissions inside the candidate, never by client-side resubmit
  or before original-intent replay lookup. This slice changes no graph editing logic.
- Response checks establish correlation/kind, snapshot revision and selected output
  set. Full service-state, authorization and application-policy validation belongs
  to adapter integration; fixture validation does not prove live operation acceptance.
- All authoring requests currently use strict structure validation. Additive unknown
  read-only response fields remain compatible; unknown mutation fields never become
  ignored semantics. Future request extensions need explicit version/capability policy.
- Raw preflight implementation uses embedded schema descriptors and immutable cached
  metadata, not a second expression parser or calculation cache.
- Signed paging/cursor lifetimes, attachment security/cleanup, Info authorization,
  diagnostic summaries/error mapping and unified registry bootstrap still need the
  separately authorized adapter implementation. No remotely retained prepare tokens.
- Default validation budgets are adjustable policy, not guarantees of large-model
  throughput. Existing service resource limits remain provisional.
- Persistence provider direction remains separately accepted project guidance:
  SQLite first, PostgreSQL/MySQL planned, mode independent of provider, one service
  owner per store. No provider, migrations or durable recovery is implemented.

## Recommended next slice

Review this foundation, then implement the serialized management adapter and daemon
integration in a bounded slice, including patch semantics, authorization, paging,
connection cleanup and one coherent published registry. Keep detailed receipt paging
deferred until its retention budget is designed. REPL authoring, persistence design/
SQLite implementation, C# IPC/Native AOT and GUI remain separate work. Stop here.
