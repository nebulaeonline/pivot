# Local IPC daemon and CLI

`pivot-transport` supplies local streams, OS security, and framing without depending
on the engine or host. `pivotd` reuses pivot-host sessions/dispatch. `pivot-cli` only
discovers, describes, and invokes through the actual protocol; it has no evaluator.
ADR-0005 remains Proposed.

## Run the demo

From the repository root, in one terminal:

```text
cargo run -p pivotd -- --demo
```

In another terminal:

```text
cargo run -p pivot-cli -- models
cargo run -p pivot-cli -- describe RestaurantProfit
cargo run -p pivot-cli -- invoke RestaurantProfit Revenue=250000 Costs=85000
cargo run -p pivot-cli -- invoke RestaurantProfit Revenue=250000
cargo run -p pivot-cli -- invoke RestaurantProfit Revenue=250000 --revision demo-v1 --outputs Margin
```

The deterministic demo is `RestaurantProfit`, model ID `restaurant-profit`, revision
`demo-v1`. Revenue is required; Costs defaults to 85000. Outputs are Net and Margin,
with `Net = Revenue - Costs`, `Margin = Net / Revenue`. Supplying 250000 and 85000
returns 165000 and 0.66. Discovery by name must resolve exactly one revision;
multiple matches require `--revision`. No implicit latest revision is selected.

Both executables accept `--endpoint LOCAL_ENDPOINT` and `--help`. Windows defaults
to `\\.\pipe\pivot-<current-user-SID>`; Unix defaults to
`/tmp/pivot-<effective-UID>/pivot.sock`. Custom Windows names must be local pipe
names. Unix endpoints must be absolute and have a private parent directory.

CLI input syntax is strict Name=Decimal; no floats or full formula language.
No `--outputs` selects all public outputs. CLI exits 0 on success, 1 for connection,
argument, or request failures, and 2 if an output contains a calculation error.
Results include diagnostic invocation ID and exact external revision. The full
interactive REPL is deferred.

## Frames, sessions, and lifecycle

A frame is `u32 little-endian length` followed by exactly that many Protobuf bytes.
One frame is one request or response. Length must be 1..=1 MiB. Length checks happen
before allocating payload buffers. The bootstrap and advertised daemon limit is
currently 1 MiB. EOF before any header byte is a clean disconnect; partial header
or payload is truncation. Zero/oversized lengths close that connection. Malformed
bounded Protobuf receives a public error without crashing the daemon.

Every connection gets a fresh host session. Handshake precedes ordinary operations.
Multiple sequential operations reuse the same negotiated connection. The CLI
handshakes, discovers the exact revision, describes it, resolves public labels,
and invokes by portable IDs. Registry snapshots survive client disconnects.

Frame reads and writes have five-second total deadlines, including idle reads.
This initial daemon therefore closes idle clients after five seconds. Up to sixteen
connection tasks are retained; excess connections are closed. A host budget defaults
to 100,000 units, conservatively counting all snapshot nodes, edges, and bound tokens
before calculation. This bounds work including evaluator graph scans, but may reject
a large model even when selected outputs are small. It is not a wall-clock deadline,
memory quota, or a complete future resource scheduler.

Ctrl+C shuts down on all platforms; Windows also handles Ctrl+Break and Unix handles
SIGTERM. Shutdown stops accepting, aborts idle connection tasks, and joins them;
in-flight results are not guaranteed delivery. Bounded synchronous calculations
finish their local work before task cancellation can take effect. `--shutdown-stdin`
is an opt-in supervised/test mode: one byte or EOF requests the same clean shutdown.
There is no public remote shutdown operation.

## Endpoint security

Windows uses a protected DACL with a single allow ACE for the current user SID,
rejects remote clients, and requires first-instance exclusivity at startup. A pipe
instance stays alive between accepts to prevent endpoint replacement. Server and
client query OS-reported peer process IDs and process-token user SIDs; failures
reject the connection. Clients request identification-only impersonation QoS.
This is user identity, not signed binary/application identity or administrator-proof
isolation. A same-user process can still masquerade as a same-user service.

Unix requires a nonsymlink private parent directory owned by the effective UID,
rejects unsafe writable ancestors except sticky directories such as /tmp, and sets
the socket to mode 0600. Both peers verify kernel-reported peer UID. The socket is
removed at normal shutdown only if its device/inode still matches the bound socket.
Existing/stale endpoints are never automatically removed. Empty private directories
may remain; after a forced crash, inspect and explicitly remove a stale socket.

Only OS-authenticated same-user peers reach the demo's explicit AllowAll model
policy. Same-user access does not make request data trustworthy. Message validation,
host limits, per-connection isolation, and work budgets still apply. No secure
configuration failure triggers an insecure fallback. No TCP listener exists.

## Verification and limits

Run `python tests/ipc/verify.py` or full `python scripts/ci.py`. Integration starts
a unique demo daemon, invokes the real CLI, probes malformed traffic, and requests
graceful shutdown. Process waits and frame operations have finite timeouts. Logs
and reports are saved under artifacts; cleanup force-kills a stuck test daemon.

Windows named-pipe execution and actual ACL inspection/exclusivity were verified.
Actual denial under another OS user was not tested; ACL inspection is not a substitute
for that integration test. Linux compiled and passed strict lint, all Rust tests, and real IPC tests under
Ubuntu WSL2. macOS 27.0.1 ARM64 passed the full local CI pipeline, including Unix
permissions/peer credential checks and real daemon/CLI integration. See
[platform validation](../platform-validation.md) for reports and build constraints.
Native AOT, fuzzing, and global fairness are unverified.
No persistence, shared memory, result retention, general scripting, publication,
editable document-management service, or GUI integration was added.

API references: [Tokio named-pipe security options](https://docs.rs/tokio/latest/tokio/net/windows/named_pipe/struct.ServerOptions.html),
[Unix peer credentials](https://docs.rs/tokio/latest/tokio/net/struct.UnixStream.html#method.peer_cred).
