# Minimal REPL authoring — review handoff

Date: October 18, 2026. This implements the narrowly scoped REPL follow-up to
accepted ADR-0007 Stage B. Persistence has not begun.

## Implemented workflow

The existing command REPL now creates/lists/selects working models, reads pinned
nodes/ports/diagnostics, adds or replaces decimal inputs and computed formulas,
adds required/optional public input ports and output ports, validates, evaluates
selected nodes with temporary overrides, and publishes an exact working revision.
Existing published-model discovery/description/invocation and one-shot CLI commands
retain their behavior. Commands use the actual typed daemon protocol; formula
parsing, binding and evaluation stay exclusively in Rust service/engine code.
The production CLI does not depend on pivot-host; that dependency is test-only.

Creating/opening a working model explicitly warns:

> Working models are currently memory-only and are lost when pivotd restarts.

Selection retains the exact WorkingRevision. Authorized discovery and draft reads
resolve portable identities; names and coordinates never manufacture IDs.
Revision conflicts never refresh or overwrite automatically. `working refresh`
explicitly adopts the current head. Incomplete formulas remain inspectable drafts
and can be repaired later; invalid models cannot evaluate or publish.
Working edits leave existing published revisions immutable.

## Mutation and connection behavior

Every mutation has a unique operation UUID and preserved typed request intent.
Lost/invalid responses retain that ID and intent and block new mutations.
`reconnect` clears selection/attachments but retains pending intent.
`working recover` queries its receipt without resending. Unknown retains intent;
`working retry` explicitly resends the same operation ID and request with a new
correlation ID. There is no automatic ambiguous replay or replacement operation.
Unauthorized/Internal mutation responses conservatively retain pending context
because delivery can be denied after commitment.

`working abandon` explicitly forgets pending client state and does not undo server
work. Endpoint changes are blocked while a mutation is pending. A changed service
instance cannot silently reinterpret an old operation. Known-dead connections
reject commands before creating new pending mutations. Pending intent is
session-local and should be resolved before exiting.

Invocation uses the existing v1.0 connection; authoring lazily opens a separate
v1.1 connection. Budget at least two daemon connection slots per authoring REPL
and one authoring session. An invocation-only daemon rejects authoring while
existing invocation commands remain usable. Negotiated message caps and bounded
page traversal apply; oversized submissions fail locally without truncation or
an admitted pending mutation.

See [command syntax and recovery details](../architecture/command-repl.md).
Public-port commands initially add ports; editing/removing ports, arbitrary
multiline input and a scripting language are outside this minimal surface.

## Validation

Full Windows CI passed: [summary](../../artifacts/ci/20261018T235955106802Z/summary.json).
It includes 135 Rust tests, .NET Release builds with warnings as errors, 10,099
decimal vectors, 58 protocol decimal layouts, 118 authoring fixtures, frozen
v1.0 compatibility and real daemon/CLI/REPL integration.

Linux WSL2: [135-test Rust log](../../artifacts/repl-authoring-linux-rust.log),
[real IPC results](../../artifacts/repl-authoring-linux-ipc/results.txt),
[authoring transcript](../../artifacts/repl-authoring-linux-ipc/authoring-repl.log).
Linux formatting and Clippy pass. After final review added the known-disconnected
state guard, all five focused executable REPL tests passed again on Windows and
Linux: [final Linux log](../../artifacts/repl-authoring-linux-final.log).

Five new executable tests run pivot-cli against actual local pipes/sockets with
the production connection router/service. They cover the complete workflow,
incomplete drafts/repair, overrides, port defaults, immutable publication,
two-client revision conflicts, daemon authorization denial, and refusal to select
published identities as editable resources. A controlled transport fault drops
an edit response after commitment; tests audit identical typed request/operation
replay, receipt-only recovery, and absence of new operations while uncertain.
Message/session limits and invocation-only capability rejection are exercised.

The existing Python IPC harness now also starts a separate `pivotd --demo
--authoring` process and executes the complete REPL workflow, then reopens the
resource from another CLI process. Existing noninteractive and invocation-REPL
regressions remain in that harness. Processes/endpoints use bounded waits and cleanup.

macOS ARM64 validation of Stage B and this REPL remains pending; use a local APFS
checkout. Terminal key handling, Native AOT and actual cross-user denial are not
newly verified here. Windows pipe/WSL checks required execution outside the sandbox.

## Review files and next boundary

- [Authoring client/state](../../apps/pivot-cli/src/authoring.rs)
- [Existing REPL integration](../../apps/pivot-cli/src/main.rs)
- [Executable recovery/authorization tests](../../apps/pivot-cli/tests/authoring.rs)
- [Separate daemon-process workflow](../../tests/ipc/verify.py)
- Production schemas, unchanged: [authoring.proto](../../crates/pivot-protocol/proto/pivot/authoring/v1/authoring.proto),
  [protocol.proto](../../crates/pivot-protocol/proto/pivot/v1/protocol.proto),
  [types.proto](../../crates/pivot-protocol/proto/pivot/v1/types.proto).

No persistence, storage provider, C# executable IPC/AOT, workbook addressing,
Avalonia UI, TCP/shared memory, incremental recalculation, macros, pipelines,
completion or persistent command history is implemented. The recommended next
milestone is provider-independent persistence design and SQLite, including atomic
state/receipt durability and bound-expression/published-revision restoration.
That requires a separate review and authorization.
