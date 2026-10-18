# Pivot Handoff — Protocol refinements and in-process host

Date: 2026-10-18
Branch / commit: uncommitted
Status: Complete

## Objective

Apply the parent review's targeted ADR-0004 refinements and the owner-authorized
in-process host/dispatcher slice. Stop before networking or subsequent milestones.

## Implemented

- Added `pivot-host`: in-memory immutable revision registry, portable port mapping,
  explicit sessions, authorizer boundary, and serialized message dispatch.
- Handshake, paged discovery, description, and actual isolated engine invocation.
- Protocol InputSource distinguishes submitted versus service-defaulted effective
  inputs; negotiation advertises actual byte and input/output limits.
- Decimal numeric comparison for provenance; exact derived message equality remains
  available for representation-sensitive codec tests.
- Public checked decimal limb construction/extraction without float intermediates.
- ADR-0004 accepted after schema, validator, session, and test reconciliation.
- Host API and protocol documentation updated. Existing CI includes host tests via
  its workspace test gate, without a separate hosting service.

## Interface / semantics

Caller-assigned portable model/revision/port IDs map to immutable snapshots. Exact
revision entries cannot be replaced. Port identities remain stable within a live
registry. Invocation uses registered interface metadata, required/default semantics,
fresh local values, and public result/error mapping. No runtime handles enter wire
messages. Process-local execution revisions remain separate from external revisions.

Handshake is required before ordinary requests. Invalid handshakes may be retried;
successful negotiation is not repeatable. Bootstrap errors have documented stable
interpretation without implying successful negotiation. Actual session resource
limits are advertised and enforced independently of graph size.

Effective input values are authoritative execution provenance and carry Submitted
or Defaulted origin. Numeric equality ignores decimal scale and zero sign. Exact
serialization equality remains a distinct operation. Diagnostic invocation IDs do
not imply retention; result retrieval remains unsupported.

## Tests actually run

- `python scripts/ci.py` — PASS: 37 Rust tests, formatting, strict Clippy, complete
  .NET Release build, 10,099 decimal vectors, and generated Rust/C# protocol fixtures.
- `cargo test --workspace --quiet` — PASS before full CI.
- `cargo clippy --workspace --all-targets -- -D warnings` — PASS.

Seven new host tests cover serialized discovery/description/invocation; required,
optional, and explicit defaults; multiple immutable revisions; working-model edits
and destruction; unknown models/revisions/ports; duplicates and invalid decimals;
arithmetic and dependency errors; concurrent independent sessions; handshake and
malformed input; byte/entry limits; denied authorization; and invalid registry maps.
Two added protocol tests cover numeric versus representation equality and provenance
source validation. Existing result tests were updated for mandatory source metadata.

CI report: `artifacts/ci/20261018T161638395780Z/summary.json`.
Native AOT, Linux/macOS execution, and fuzzing — NOT RUN.

## Decisions made or proposed

- **Accepted:** review-refined ADR-0004 architecture after owner authorization.
- **Implementation choices:** dedicated host crate, caller-assigned identity mapping,
  in-memory append-only revisions, registry-index pagination, explicit policy trait,
  and diagnostic namespace plus checked invocation sequence.
- **Reserved:** durable registry allocation/publication policy, authentication,
  transport framing and permissions, retrieval/retention, and compute budgets.

## Open issues / risks

AllowAll is suitable only for explicitly trusted embedding, not authentication.
Future network integration must provide authenticated authorization context. Callers
must keep invocation namespaces unique across relevant host lifetimes. Registry
entries/results have no durable identity guarantee; results are not stored. Cursor
consistency across registry mutation is unspecified. Responses exceeding session
budgets are replaced with ResourceExhausted, possibly after a synchronous calculation.
No cancellation, registry eviction, audit store, fuzzing, or global compute quota exists.

## Questions for ChatGPT / owner

No blocking questions for this slice. Before local IPC, settle framing, connection
permissions, peer identity, and how host namespaces/model IDs are provisioned.

## Next recommended slice

Review the adapter, then design a minimal local IPC/session boundary for pivotd and
the CLI. No sockets, named pipes, shared memory, persistence, REPL, or GUI were added.
