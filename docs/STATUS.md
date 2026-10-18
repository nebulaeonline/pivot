# Pivot implementation status

Date: 2026-10-18

The Rust workspace and .NET 10/Avalonia solution are scaffolded. Architecture
records live in `docs/adr/` using `ADR-NNNN` identifiers. ADR-0001 records the
previously agreed graph-first design.

`pivot-types` provides `PivotDecimal`, a private `rust_decimal` wrapper with
decimal text parsing, integer construction, formatting, comparisons, and checked
addition, subtraction, multiplication, division, and negation. Arithmetic errors
distinguish overflow and division by zero. Unrepresentable literals are rejected.
No float conversion API is exposed. Arithmetic outputs are canonicalized; explicit
rounding supports seven modes with nearest/ties-away-from-zero as default. .NET
helpers expose the same behavior. A shared 10,099-vector suite passes for values,
errors, and scales, including seeded arithmetic cases and independent rounding
expectations. See `tests/decimal-parity/verify.py` and the numeric architecture note.

`pivot-core::expression` parses decimal literals, ASCII identifiers, unary signs,
binary arithmetic, and parentheses into an AST. Multiplication/division precede
addition/subtraction; binary operators associate left. Names are case-preserving
and unresolved. The AST has no evaluation or I/O behavior.

Editable models now support stable node IDs, named decimal inputs, computed nodes,
and immutable snapshots with captured inputs and model-scoped revisions. Snapshot
validation checks bound IDs, extracts forward/reverse dependencies, orders nodes, and
rejects cycles. See ADR-0002 and the model/snapshot architecture note.

Snapshot evaluation now supports selected outputs or all nodes, bound decimal
expressions, calculation-local state, typed arithmetic/dependency errors, and
immutable results retaining their snapshot revision. Isolated invocation accepts
temporary input overrides keyed by NodeId, with captured defaults and typed binding
validation. Independent invocations share only immutable snapshot definitions.
Independent arithmetic failures do not prevent other outputs from being computed.
Forty-four Rust tests pass on Windows and Ubuntu WSL2 Linux for the command-REPL
slice. macOS ARM64 last passed 41 tests before this slice; the REPL changes require
a fresh Mac run. Public model ports now support named invocation,
required inputs, explicit optional defaults, and stable port identities. Formula
names bind only at edit submission; snapshots reuse stable-ID references.
Public values have extensible tagged value/type boundaries (Decimal-only today).

Invalidation, IEEE-754 and other non-decimal
values remain deferred. A demo daemon and minimal CLI now use named pipes on
Windows and Unix-domain sockets on Unix (Linux WSL2 and macOS ARM64 verified). ADR-0004 is accepted after the review
refinements. The portable contract has generated Rust/C# codecs, numeric provenance
validation, explicit input sources, negotiated limits, and an in-process pivot-host
registry/dispatcher with mandatory handshake sessions. ADR-0005 is Accepted;
local IPC includes current-user peer checks, bounded framing, connection limits,
and a conservative calculation work budget. Idle/partial-read/write deadlines,
connection caps, and work budgets are configurable operational policy. A persistent
command REPL shares the actual IPC client and supports session-local history,
line editing, explicit reconnect, and no automatic replay after disconnect.
Expression evaluation, model editing, and scripting remain deferred.
Local CI is available through `python scripts/ci.py`, covering Rust formatting,
lint/tests, .NET Release builds, and cross-language decimal compatibility. See
`docs/local-ci.md` for prerequisites and current platform/reproducibility limits.
The complete CI pipeline now passes in Ubuntu WSL2 with .NET SDK 10.0.112,
including both cross-language suites and the .NET Release solution build.
The complete CI pipeline also passes on macOS 27.0.1 ARM64 with .NET SDK
10.0.402 using an isolated local APFS source/build copy. See
`docs/platform-validation.md` for reports and the SMB build limitation.
Native AOT, GUI execution, and actual cross-user endpoint denial remain unverified.

The editable working-model service is now designed in accepted ADR-0006 and
`docs/architecture/working-model-service.md`: connection-independent resources,
atomic expected-revision edit batches, draft validation, unpublished evaluation,
publication boundaries, and operation-receipt reconciliation. Slice A is implemented
in `pivot_host::management`: memory-only resources, portable identities, authorized
discovery/inspection and attachments, explicit quotas, and replay-safe creation.
Slice B adds private transactional staging, atomic edit batches, one public revision
per batch, portable node/port IDs, revision-pinned raw draft reads and diagnostics,
final-directory binding for submitted formulas, and retained edit receipts.
Slice C adds exact-revision structural validation, detached immutable evaluation
captures with temporary overrides, and replay-safe publication into a coordinated
immutable invocation registry. Published revisions and working revisions remain
distinct. Serialized authoring is available in process and through opt-in local
daemon IPC; REPL authoring remains deferred. See
`docs/architecture/working-model-management.md` for the in-process API and limits.
Transactional editing details are in `docs/architecture/working-model-editing.md`.
Calculation/publication details are in
`docs/architecture/working-model-calculation-publication.md`. Persistence, identity
compaction and richer edit diagnostics remain follow-ups; memory-only commits do
not survive service shutdown. Latest REPL macOS validation and Native AOT remain
on the verification backlog.

ADR-0007 is Accepted. Production authoring schemas and shared types, Rust/C# codecs,
bounded recursive mutation preflight, typed intent and opt-in v1.1 negotiation are
implemented and compatibility-tested. The daemon defaults to invocation-only;
`--authoring` explicitly enables v1.1 authoring. Stage A implements all twelve
serialized authoring operations in process, signed bounded paging, session-owned
attachment cleanup, staged partial-port patches and exact wire-intent replay.
Evaluation runs outside the service lock and delivery rechecks authorization.
Detailed receipt diagnostic retention remains deferred. See
`docs/architecture/serialized-authoring-adapter.md` and
`docs/architecture/authoring-protocol-foundation.md`. Stage B integrates actual
local IPC, OS-derived principals, one demo/authored publication registry and
blocking dispatch with no service lock across transport writes. Five new IPC
tests cover daemon authoring, legacy compatibility, lost responses, concurrent
edits, denial/revocation, cancellation cleanup, quotas and malicious traffic.
See `docs/architecture/daemon-authoring.md` and the Stage B handoff for current
Windows/WSL validation; macOS verification of this slice remains pending.
The final Rust suite has 130 tests, including 23 adapter/router tests and five
daemon IPC integration tests. Full Windows and WSL CI reports are linked from
`docs/handoffs/2026-10-18-daemon-authoring.md`.
Persistence direction is a common transactional contract with configurable
providers: SQLite first/default locally with a filesystem path, and planned
PostgreSQL/MySQL adapters. Storage choice is independent of local/network service
mode. Initially one active service owns a store; multi-instance coordination remains
separate work. See `docs/architecture/persistence-providers.md`. Provider support,
schemas and recovery are not implemented; runtime durability remains MemoryOnly.
