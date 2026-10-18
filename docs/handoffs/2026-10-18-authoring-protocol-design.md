# Versioned authoring protocol — design handoff

Date: 2026-10-18
Status: Proposed; ready for review

## Deliverables

- [ADR-0007](../adr/0007-versioned-authoring-protocol.md), Proposed.
- [Protocol architecture](../architecture/authoring-protocol.md): operations,
  negotiation, identity, replay, diagnostics, authorization, limits and pagination.
- [Review-only Protobuf](../proposals/authoring-v1/authoring.proto): illustrative
  portable request/response and typed edit contracts. All new tags remain unfrozen.
- [Rust/C# test plan](../architecture/authoring-protocol-tests.md).
- [Persistence providers and recovery direction](../architecture/persistence-providers.md),
  recording the owner's configurable-provider direction and separate recovery/schema
  design questions.
- Updated status and ADR index. ADR-0006 and its accepted implementation are unchanged.

## Proposed contract

Negotiate v1.1 plus `working-models-v1`, requiring `decimal96`. Preserve v1.0
invocation clients and payloads. Add one authoring request/result alternative to
the existing envelope at reviewed, unused tags; 20 is an illustration, not frozen.
Extract shared types first to avoid circular imports, preserving their wire names,
tags and enum values while testing changed generated-file/descriptor metadata.

Expose service/durability information, create/list/open/close, exact-revision raw
draft reads, typed edits, validation, unpublished evaluation, publication and outcome
lookup. Keep snapshot capture and publication preparation private to the adapter.
Maintain one coherent daemon publication/discovery view when implementation begins.

Use typed portable working/node/port/revision/snapshot identities, never runtime
handles. Do not reparse source to recreate binding. Keep working defaults and
Submitted/Captured input provenance distinct from public Submitted/Defaulted.

Shared Create/Edit/Publish receipt identity remains instance/principal/operation ID.
Compare exact typed intent before expected-head checks, independent of request ID.
Strict schema-aware mutation preflight rejects unknown/ambiguous content before
decoders can discard it. No automatic retry or durable replay guarantee is added.

Page definitions and diagnostics under frame/count/byte ceilings. Draft pages pin
the current head and explicitly expire if editing advances it. Discovery is keyset
best effort, not an atomic catalog snapshot. Receipt diagnostics page the original
retained outcome, independent of current head. Signed opaque cursors bind principal,
connection, instance, query source and expiry; reconnect restarts paging.

Connection-scoped attachments are not credentials or owners. Disconnect cleanup
preserves models. Close replay is idempotent within bounded connection bookkeeping.
Authenticate principals from connection context; never trust payload identity.

## Persistence provider direction

Use one transactional storage contract with configurable providers. SQLite is the
first implementation/default local provider and accepts a filesystem database path.
Plan PostgreSQL/MySQL adapters and allow other conforming providers. Local/network
service mode and provider choice are independent; network serving does not require
a server database, and local serving does not prohibit one.

Keep configuration, provider SQL and migrations behind the Rust storage boundary.
Initially use compiled adapters selected by configuration; dynamic plugin loading
is deferred. Implement SQLite first, then prove the contract with a second provider.
All adapters must pass common persistence conformance tests.

Initially one active service owns a store. Supporting a server database does not
make multiple daemon instances safe without further coordination of heads,
in-memory projections, publication registries and policy state.

No bespoke native storage format or database dependency is introduced here. A
separate design must resolve provider migrations, portable bound-expression storage,
exact decimals, restart identity and receipt retention. Database transactions alone
do not solve recovery after durable commit but failed memory installation. The
protocol reports durability/recovery independently of provider; current runtime
continues reporting MemoryOnly.

## Verification actually performed

- Compiled the standalone draft with the existing vendored protoc compiler and
  current shared protocol import. Descriptor generated successfully at
  `artifacts/authoring-protocol-design/authoring.pb` (not tracked).
- Checked Markdown relative links and ADR structure.
- `git diff --check` passed.
- Confirmed changes are documentation and the isolated draft schema only; no
  production proto, code-generation input, generated codec or runtime change.

No Rust/.NET CI or new interop tests were run for this design-only slice. Descriptor
compilation validates draft syntax/imports only; it does not prove negotiation,
envelope integration, codec interoperability, paging or authoring dispatch.

## Review questions and implementation gaps

1. Accept additive v1.1 capability negotiation and the common-types extraction,
   preserving existing v1.0 wire behavior?
2. Accept synchronous remote Evaluate/Publish with private captures/preparations,
   rather than remotely retained handles in the first authoring contract?
3. Accept strict mutation preflight, including unknown/duplicate-field rejection,
   as a server requirement for truthful exact-intent comparison?
4. Accept current-head draft paging with explicit expiry, best-effort discovery,
   and original receipt diagnostic paging? Default 64/max 128 entries, 512 KiB
   target, 1 MiB hard frame, 512-byte cursors and 60-second total query lifetime
   remain provisional configurable adapter policy.
5. Accept bounded connection attachment bookkeeping and idempotent Close, with
   cleanup on disconnect and no resource deletion?

Info authorization, mutation preflight, cursor signing, paged views/receipt
diagnostics, attachment adapter lifecycle and unified daemon registry bootstrap
are future implementation work. Working diagnostic pages require one authoring
Inspect/Validate/Evaluate right; receipt pages require the recorded mutation right.
Ordinary published invocation must not expose privileged graph details.

## Deferred and recommended sequence

Review contract and schema before assigning production tags or advertising the
capability. Then implement codecs/validation and cross-language fixtures, serialized
service adapter/daemon integration, and external REPL/C# authoring in bounded stages.
Keep transactional persistence/provider recovery design near the front of the roadmap, with Native
AOT and latest REPL macOS validation tracked separately.

No live dispatch, REPL authoring, storage-provider implementation, persistence, spreadsheet
projection or Avalonia changes. Stop after this design slice pending review.
