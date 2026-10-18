# Authoring protocol verification plan

Status: Foundation codec/preflight/compatibility cases implemented and executed;
22 serialized in-process adapter tests now cover paging, attachments, edits,
evaluation, publication and authorization. Five Stage B integration tests now
cover real daemon/IPC paths and controlled connection cancellation/revocation.
See [daemon evidence](daemon-authoring.md), [adapter evidence](serialized-authoring-adapter.md) and
[foundation evidence](authoring-protocol-foundation.md). The
production schema is generated in Rust/C#; the historical review draft is excluded.

Detailed receipt diagnostic paging is deferred pending byte-budgeted retention.
Tests below referring to receipt detail pages apply only to that future extension.

## Codecs and typed intent

Generate Rust/prost and C#/Google.Protobuf from the same accepted schema. Exchange
fixtures in both directions for every request/result/edit/definition/diagnostic/error
alternative. Verify optional offset/index presence, empty formula source, initial
revision zero, sequences above 2^53 and near u64 limits, canonical UUIDs, full UTF-8
source bytes, exact decimal layouts and unknown enum/value rejection.

Mutating fixture equivalence must test different field ordering and valid wire
encodings decoding to identical typed intent. Counterexamples change edit order,
expected revision, temporary mapping, source whitespace, port requirements and
decimal coefficient/sign/scale. Verify required empty-message oneofs survive both
generators. Do not compare entire Protobuf bytes as the operation intent.
Verify the proposed common-types extraction preserves fully qualified names and
wire bytes, creates no import cycle, and builds both languages. Check generated
source inclusion and changed descriptor/file metadata; old compiled clients must
still interoperate, and do not assume reflection/file layout stays identical.

Construct raw negative bytes outside generated writers: duplicate singular fields,
conflicting oneofs, unknown nested mutation fields/enums, missing values, bad UUIDs,
wrong instance/model revision, truncated strings, invalid UTF-8 and overflowed lengths.
Both server-side validation and documented C# expectations must agree; normal decoder
last-one-wins is not mutation admission. Decode additive unknown response fields
without changing known invocation behavior.

## Version/capability matrix

| Client | Server | Required result |
| --- | --- | --- |
| Existing v1.0 invocation | New v1.1-capable | Negotiate 1.0; unchanged invocation fixtures |
| New client with optional authoring | Existing v1.0 | Invocation-only fallback |
| New client requiring authoring | Existing v1.0 | Explicit capability/version rejection |
| New v1.1 authoring | New server with authoring disabled | Rejection, no accidental dispatch |
| New v1.1 authoring | Enabled authoring server | Negotiated typed authoring operations |
| Any pre-handshake authoring | Any server | HandshakeRequired |

Retain the existing 58 decimal-layout protocol suite and 10,099 numeric vectors.
Ensure an unchanged v1.0 client can discover and invoke authored published revisions
without obtaining draft source, authoring dependency details or new client logic.

## Management and replay integration

Drive actual serialized operations against the management adapter. Reuse in-process
invariant cases: create/discover/multiple attachments, atomic edits and rollback,
final-directory binding, invalid drafts, stable references through rename/reuse,
validation, financial evaluation with captured inputs/overrides, independent output
errors and immutable published revision selection. Verify public required/defaulted
semantics remain different from interactive submitted/captured semantics.

Drop responses after Create/Edit/Publish commits; reconnect with new request IDs
and the same operation IDs. Assert one mutation, one revision transition/registry
entry and original outcome before stale-head or full-quota checking. Changed intent
must reject. Query receipt diagnostic pages after later edits and verify original
diagnostics, identity and counts. Pre-admission failures have no false receipt.
Restart the service and ensure old-instance requests never mutate the new instance.

## Paging, limits and security

- Boundary-count and encoded-size cases for every request/response/page, including
  a maximal dependency list, large invalid-draft diagnostics and receipt diagnostics.
  No partial mutation, truncated source, dropped diagnostic or split calculation.
- Read all draft pages at one exact head; advance it mid-read and require ViewExpired.
  Exercise keyset discovery with concurrent creation and document best-effort behavior.
- Verify deterministic page ordering, terminal empty cursor and default/max page sizes.
  Reject expired, altered, oversized, wrong-source/section, cross-principal and old-
  instance cursors. Reauthorize each page and avoid denied-resource existence leaks.
- Fill receipt quota independently of graph/page/capture capacity. Existing replay
  remains available; rejected operations do not become half-committed resources.
- Bound attachment/closed-ID sets; repeated Close is idempotent for its own connection.
  Lost Open plus disconnect frees slots. Another connection/principal cannot close
  or use an attachment as authority; resource survives disconnect and idle timeout.
- Obtain principal from authenticated transport context, never payload identity.
  Independently deny Info/Inspect/Edit/Validate/Evaluate/Publish/Invoke. Revoke policy
  after capture/preparation and before commit/delivery; verify no privileged results.
- Prove daemon discovery includes both bootstrap/demo and authored publications in
  one registry view; there is no shadow invocation host.

## Execution plan

After review: implement schema/codecs/strict validation fixtures first; then serialized
adapter tests and real daemon IPC tests; then REPL and separate C# executable tests.
Run the existing full local CI plus new fixture gates. Validate Windows, Linux/WSL2
and macOS using isolated build directories. Add .NET Native AOT publish/run as a
separate concrete gate, not an inferred result from ordinary C# builds.

Persistence/crash recovery requires its own provider-independent design/test plan.
SQLite, PostgreSQL, MySQL and future adapters must pass common semantic conformance
tests plus provider-specific failure tests. Protocol fixtures must not assume a
provider from service mode, expose database configuration, or infer durability from
a provider name. Multi-daemon/store coordination is separately tested when designed.
No new durability or restart replay assertion becomes valid by changing a wire enum.
