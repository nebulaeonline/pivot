# ADR-0007 Stage B — daemon authoring review handoff

Date: October 18, 2026. Stage B connects the accepted adapter to actual local IPC.
Work stops at that boundary.

## Implemented behavior

`pivotd --authoring` explicitly enables v1.1 `working-models-v1` sessions.
`--demo` alone remains invocation-only. Combining the flags hosts demo and authored
publications in one management-owned registry. Existing v1.0 clients discover,
describe and invoke authored published revisions. v1.1 clients can author and
invoke on the same connection.

Principals derive from verified OS peers (SID/UID), never request fields. Explicit
authoring policy grants rights to the admitted current user independently of
invocation policy. Failed verification rejects admission. Existing pipe ACLs,
remote-client rejection and Unix socket protections remain; no TCP is added.

All twelve operations reuse Stage A: staged patches, original wire intent,
signed paging, attachments and operation receipts. Publication preserves atomic
registry/receipt visibility. Trusted bootstrap registers demo snapshots in the
same bounded registry before listening. Immutable registry views share snapshots
and one invocation-ID counter, with copy-on-write entry maps.

Invocation calculations/serialization run after releasing service access;
unpublished evaluation retains detached capture. No management lock crosses
transport writes. Dispatch runs in a blocking job owning its session, so cancellation
cannot trigger attachment cleanup while that dispatch holds a service guard.
Started jobs finish bounded work; shutdown aborts connection waiters and runtime
shutdown waits for those jobs. Attachments close while models/receipts survive
disconnect. Delivery is not guaranteed; clients recover receipts without automatic replay.

`--authoring-sessions` defaults to 16 (1–1024). Existing connection caps,
idle/read/write deadlines and work budgets remain. `--work-budget` applies to
unpublished and published calculations. The router honors smaller embedded adapter
message limits for invocation too. Defaults/quotas are advertised in service info.
The daemon has no live policy administration; revocation tests inject a policy
into the production connection runner.

**Durability remains MemoryOnly. Restart destroys authored resources and receipts.**
Startup and service info report this. See [configuration and security](../architecture/daemon-authoring.md).

## Validation

Full Windows CI: [summary](../../artifacts/ci/20261018T214311019250Z/summary.json).
Full Ubuntu WSL2 CI: [summary](../../artifacts/ci-linux-wsl-stage-b/20261018T213825367000Z/summary.json).
Following the final embedded-limit correction, Linux formatting, Clippy and all
Rust tests were rerun: [final Rust log](../../artifacts/ci-linux-wsl-stage-b/20261018T213825367000Z/final-rust-tests.log).

Final coverage: 130 Rust tests, including 23 adapter/router tests and five new IPC
tests. Full CI includes .NET Release builds with warnings as errors, 10,099 decimal
vectors, 58 protocol decimal layouts, 118 authoring fixtures, frozen v1.0 compatibility
and existing daemon/CLI/REPL integration.

New tests cover all twelve operations over actual IPC; legacy/v1.1 invocation;
disabled authoring and capability rejection; concurrent edits, conflicts and rollback;
attachment ownership and session quotas; malformed/oversized traffic survival.
For Create/Edit/Publish, an observer confirms commitment while the mutation response
remains unread, then the mutating socket/pipe disconnects. Reconnect recovers the
receipt and identical replay with a fresh request ID returns the original outcome.

Controlled-policy tests use the actual production connection runner and local IPC,
without a daemon test-control protocol. They check denial hides existence/revisions,
revocation yields Unauthorized, and cancellation during evaluation/publication
releases a one-slot attachment quota without deadlocks. Daemon process shutdown
with active connections also has a bounded wait. Tests use unique endpoints and
bounded readiness/exchange/cleanup deadlines.

Windows named-pipe tests require execution outside this sandbox (the initial
sandboxed attempt received Access Denied). macOS Stage B validation is pending;
Native AOT and actual cross-user endpoint denial remain unverified.

## Review files and scope

- [Daemon configuration/bootstrap](../../apps/pivotd/src/main.rs)
- [Connection runner](../../apps/pivotd/src/lib.rs)
- [Protocol router](../../crates/pivot-host/src/management/connection.rs)
- [IPC tests](../../apps/pivotd/tests/authoring.rs)
- [Registry views](../../crates/pivot-host/src/lib.rs)
- Actual production schemas: [authoring.proto](../../crates/pivot-protocol/proto/pivot/authoring/v1/authoring.proto),
  [protocol.proto](../../crates/pivot-protocol/proto/pivot/v1/protocol.proto),
  [types.proto](../../crates/pivot-protocol/proto/pivot/v1/types.proto).
  No schema allocations changed; the historical proposal is not authoritative.

Status, ADR-0007 and architecture docs are updated. The local IPC doc also had an
existing invalid UTF-8 dash repaired. No REPL authoring, persistence, executable
C# IPC/AOT, coordinates, GUI, shared memory, new numeric types, historical receipt
paging or benchmark project is added. Follow-ups remain REPL authoring,
provider-independent persistence/SQLite, C# IPC/AOT, then workbook projection and
performance measurement. None has begun here.
