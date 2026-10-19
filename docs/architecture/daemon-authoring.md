# Daemon authoring integration

Stage B connects the accepted ADR-0007 adapter to actual local IPC. Enable it
explicitly:

```text
cargo run -p pivotd -- --authoring
cargo run -p pivotd -- --demo --authoring --authoring-sessions 16 --work-budget 100000
```

`--demo` alone retains invocation-only behavior. `--authoring` alone starts an
empty publication registry. With both flags, demo and authored publications share
the management service's single registry. Existing CLI/REPL discovery, description
and invocation work unchanged. The subsequent [minimal authoring REPL](command-repl.md#minimal-authoring)
now exposes this contract through typed IPC commands.

**Durability is MemoryOnly. Restart loses all authored resources and receipts.**
The daemon reports this at startup and through GetServiceInfo. Persistence remains
a separate milestone.

## Configuration and authorization

`--authoring` grants explicit authoring rights to the verified current OS user.
The policy is independent of the public invocation AllowAll policy. There is no
client-supplied username, per-application trust or user administration interface.
Any process running as that user and admitted by the endpoint can exercise these
rights. The daemon has no live policy administration; the embedding API supports
independent and revocable policies.

`--authoring-sessions` defaults to 16, range 1–1024. It counts negotiated v1.1
sessions; legacy sessions do not consume these slots. `--max-connections` defaults
to 16 and bounds all accepted sessions and concurrent dispatch jobs.
`--work-budget` defaults to 100000 and applies to both unpublished captures and
published invocations. Existing idle/read/write timeout flags remain effective.
Management quotas and adapter byte/page/attachment limits retain their defaults
and are advertised through service info; they are not all daemon command flags.

The transport verifies the OS peer and the daemon derives a principal from that
connected stream: `sid:<SID>` on Windows, `uid:<UID>` on Unix. Failure rejects the
connection. Existing current-user DACL, remote-pipe rejection, private socket
directory and peer UID checks are preserved. No TCP/network listener is added.
See [endpoint security](local-ipc.md#endpoint-security).

## Negotiation and execution

`management::connection::Connection` routes connections to v1.0 invocation or
the explicitly enabled v1.1 adapter. Enabled-endpoint handshakes undergo bounded
structural preflight. A v1.0 session cannot switch to authoring without reconnecting.
A v1.1 session can also discover, describe and invoke publications: the router
reuses v1.0 invocation validation/execution through an internal envelope conversion,
retaining v1.1 response versions. Authoring mutations retain raw preflight and
decoded-intent receipt handling. Opaque invocation port IDs remain distinct from
authoring UUIDs.

The trusted `register_bootstrap` boundary uses the same bounded registry and
structural validation as publication without creating editable demo resources.
Bootstrap runs before listening. `ModelHost` clones are immutable registry views
sharing hosted snapshots and one invocation-ID sequence. Registry updates use
copy-on-write entry maps; existing views retain their revisions. Each invocation
request obtains the latest view under brief service access, then validates,
calculates and serializes after releasing the management mutex.

`pivotd::serve_connection` executes each request in a blocking job, keeping
synchronous calculations off asynchronous I/O workers. Connections bound the
number of concurrent jobs. Unpublished evaluation still uses detached capture;
publication still uses prepare/commit and atomic registry/receipt visibility.
No service lock is held across pipe/socket writes. Write deadlines disconnect
slow readers.

The blocking job owns its session during dispatch. Cancelling its asynchronous
waiter cannot trigger attachment cleanup while that dispatch holds a service
guard. The session drops after bounded dispatch completes, releasing attachments
even after permission revocation. Models and receipts survive disconnect.
No mutation is automatically replayed. A completed mutation may outlive its
response; clients recover its receipt using the same OS principal and operation
ID, with a fresh request ID if needed.

Shutdown stops accepting and aborts connection waiters. Started blocking jobs
finish their bounded work and release sessions; Tokio runtime shutdown waits for
them. Results are not guaranteed delivery. Work budgets bound graph operations,
not wall-clock latency. Trusted authorizers must not panic, block indefinitely or
recursively lock the service.

## Verification

`apps/pivotd/tests/authoring.rs` runs real daemon processes and local pipe/socket
clients for all twelve authoring operations, legacy/v1.1 publication invocation,
negotiation, disabled authoring, concurrent edits, rollback, receipt recovery after
confirmed commits with unread responses, session quotas and hostile frames.

The suite also runs the production connection runner over actual IPC with an
injectable policy. It verifies denied reads hide existence/revisions, revocation
before delivery returns Unauthorized, and cancellation during evaluation or
publication releases a one-slot attachment quota without deadlock. These controlled
policy tests add no test-only daemon control protocol.

Validation evidence is recorded in the [Stage B handoff](../handoffs/2026-10-18-daemon-authoring.md).
macOS verification of this slice remains pending. Persistence,
C# executable IPC, Native AOT, spreadsheet projection, benchmarks and GUI work
remain separate follow-ups.
