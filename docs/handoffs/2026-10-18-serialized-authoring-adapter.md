# Stage A serialized authoring adapter — review handoff

Date: October 18, 2026. Stage A is implemented and Windows CI passes.
This follows acceptance of the ADR-0007 protocol foundation. Stage B daemon
integration has not begun.

## Production schemas to include in review

Upload these actual production files together; the earlier `docs/proposals`
schema is historical and is not the generated or executed contract:

- [authoring.proto](../../crates/pivot-protocol/proto/pivot/authoring/v1/authoring.proto)
- [protocol.proto](../../crates/pivot-protocol/proto/pivot/v1/protocol.proto)
- [types.proto](../../crates/pivot-protocol/proto/pivot/v1/types.proto)

The only schema addition in this slice is
`ResourceLimits.authoring_sessions = 25`. Existing message names, operation tags,
shared types and v1.0 allocations remain unchanged. Detailed receipt diagnostic
paging remains deferred; its reserved field is not reused.

## Implemented boundary

`pivot_host::management::adapter` exposes bounded serialized sessions over one
existing `WorkingModelService`, using trusted embedder principals and explicit
v1.1 negotiation. All twelve request operations use existing service behavior,
including receipt recovery and the coordinated immutable publication registry.
There is no second model or operation ledger.

Input-port patches resolve omissions inside native transaction staging. Original
decoded wire intent is retained in service edit receipts, preserving decimal
representation and presence for replay comparisons without comparing raw bytes.
Requirement-only patches preserve invalid targets as editable draft state.

List/draft/diagnostic pages enforce entry and encoded byte bounds. HMAC-SHA256
cursors bind session, principal, instance, query options and exact revision where
applicable, with a fixed original expiry and no retained view cache. Session
close/Drop releases owned attachments despite revoked Close permission; resources
and receipts remain. Session and attachment bookkeeping are bounded.

Info is independently authorized. Privileged responses recheck authorization
before delivery, including pages and already-committed operations. Evaluation
releases the service lock after capturing the immutable requested revision.
Publication preserves prepare/commit head and authorization checks and atomic
registry/receipt visibility. Mutation reply capacity is checked before admission;
oversized read/evaluation results fail rather than truncate. Both directions obey
the smaller negotiated request/response message ceiling.

See [adapter architecture](../architecture/serialized-authoring-adapter.md),
[adapter implementation](../../crates/pivot-host/src/management/adapter/mod.rs),
[wire mapping](../../crates/pivot-host/src/management/adapter/wire.rs) and
[serialized tests](../../crates/pivot-host/tests/adapter.rs).

## Validation

Final complete Windows run: `python scripts/ci.py`;
[CI summary](../../artifacts/ci/20261018T210917145872Z/summary.json).

- Rust formatting, Clippy with warnings denied, and 124 workspace tests pass,
  including 22 serialized adapter tests.
- .NET 10 Release solution build passes with warnings treated as errors.
- All 10,099 decimal parity vectors and 58 protocol decimal layouts pass.
- Rust/C# authoring fixture exchange (118 fixtures), frozen v1.0 descriptor/codec
  compatibility and current-host invocation checks pass.
- Real Windows daemon/CLI IPC and command REPL integration pass.

Tests include atomic rollback, competing writers, exact-wire replay conflicts,
lost-response reconciliation, staged patches, incomplete drafts, permission
revocation, bounded session cleanup, cursor tampering/expiry/options, and immutable
published invocation. An evaluation test edits the service from an authorization
hook using `try_lock`, proving execution has released service access while its
results retain the captured revision. A regression covers a request ceiling below
the configured response ceiling.

One sandboxed rerun stopped at the .NET build without diagnostics; the full
approved unrestricted rerun above passed. This slice has not been validated again
on Linux/macOS. Latest REPL macOS validation and Native AOT remain backlog items.

## Scope and next review gate

The daemon still advertises only v1.0 invocation; no live authoring transport,
C# authoring client, REPL editing, new numeric type, GUI or persistence is added.
HMAC/SHA-256 dependencies support cursor authentication only. Durability is
MemoryOnly and service restart discards state/receipts.

Stage B should integrate the adapter with actual trusted transport identity and
one unified demo/authored invocation registry, preserving the existing publication
commit boundary. Resource UUIDs remain identifiers rather than credentials.
Persistence/provider recovery remains a separate design, with SQLite first and
PostgreSQL/MySQL planned behind a common transactional provider contract.
