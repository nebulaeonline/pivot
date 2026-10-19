# ADR-0007: Versioned working-model authoring protocol

Status: Accepted
Date: 2026-10-18

## Context

ADR-0006 authoring works in process, including transactional drafts, detached
calculation and immutable publication. External clients currently invoke only
published models. They need one portable contract without exposing Rust handles,
moving evaluation into clients, or weakening revision/replay guarantees.

## Decision

Extend the existing envelope additively in a negotiated v1.1 minor version, gated
by `working-models-v1`. Preserve v1.0 published invocation messages and behavior.
Reviewed production envelopes assign tag 20 to authoring requests/responses.
Shared types are extracted without changing wire names or layouts. Foundation
negotiation/codecs, Stage A adapter and Stage B local daemon integration are
implemented. Live authoring advertisement and dispatch require explicit
`--authoring` enablement; invocation-only remains the default profile.

Expose service info, create/list/open/close, revision-pinned draft reads, typed
edit batches, validation, synchronous unpublished evaluation, publication and
operation-outcome lookup. Keep capture and publication preparation tokens private
to the adapter initially; no remote long-lived calculation handles are required.

Authenticate principals from connection context; never accept identity claims in
payloads. Authorize every resource access, page, mutation and delayed result.
An attachment is revocable connection bookkeeping, not a bearer credential or
resource owner. Disconnect releases attachments, never working resources.

Mutations carry operation IDs distinct from request correlation. Compare complete
typed intent, including exact numeric representation and source bytes. Reconcile
receipts before expected-revision checks. Reject unrecognized mutation content
rather than silently dropping intent. Never automatically retry ambiguous calls.

Keep working revisions, calculation snapshot IDs and published model references
distinct. Surface invalid drafts and diagnostics independently of request failures
and arithmetic output errors. Bindings are portable node identities, not formula
source reparsed at read or calculation time. Numeric execution stays Decimal-only.

Use bounded pages for raw definitions and diagnostics. Draft pages require the
same exact current head; fail explicitly if it advances. Detailed receipt diagnostic
paging is deferred until complete retained data and byte-budget admission are
specified; never reconstruct historical diagnostics from a newer head. Receipt
outcomes and diagnostic summaries remain available. Discovery is best effort, not an
atomic catalog snapshot. No unbounded hidden read-view retention is introduced.

The daemon exposes one coherent publication registry for built-in and
authored models. Publication remains a management-owned atomic registry/receipt
commit, never a second independent daemon publication path.

Report MemoryOnly durability and service-instance-scoped recovery truthfully.
Design persistence behind one transactional contract with configurable providers,
independently of local/network service mode. SQLite is the first/default local
provider with a configurable filesystem path; PostgreSQL, MySQL and potentially
others are planned adapters. Provider SQL/migrations stay private; the protocol
reports durability and recovery semantics rather than database configuration.
Initially retain one active service owner per store; multiple daemon instances
sharing a database require separate coordination design. Implement SQLite first
and prove the contract with a second provider. Detailed storage/recovery design
remains separate; no provider implementation or bespoke native format is introduced.

## Alternatives considered

- Separate authoring socket/protocol: splits negotiation, identity and discovery.
- A v2 replacement for invocation: unnecessary break for additive authoring.
- Expose Rust objects or generic JSON edits: loses portable typed intent checks.
- Client-side expression execution: creates a competing semantic implementation.
- Remote capture/prepare handles now: introduces retention, expiry and cancellation
  complexity before ordinary authoring is externally usable.
- Implicit retry or name-based rebinding: violates accepted recovery/identity rules.
- Persist before protocol design: couples two review slices; storage design remains
  near-term but is not implemented here.

## Consequences

The foundation implements bounded recursive structural preflight before generated
decoding, identity/value validation and typed mutation intent. Port patches use
presence: omitted means unchanged, supplied means replacement; at least one update
is required. Stage A implements patches inside staged state with native
PatchInputPort, retaining omitted fields and original wire intent. Empty formula source is
a present invalid draft, and edit sequence zero is valid. Model limits are
configurable metadata, not permanent protocol maxima. Initial transport remains
1 MiB; future bulk mechanisms require separate negotiation. Existing invocation
wire layouts are preserved, while generated source/descriptor organization changes.

Implementation status (2026-10-18): Stage A supplies authorization, receipt
accounting, portable errors, byte-aware paging, signed cursors, attachment cleanup
and off-lock calculation. Stage B and the minimal REPL expose these through local
IPC. The accepted instance-scoped recovery decision is unchanged. ADR-0008 is a
separate proposed durable design, not an implemented extension of these guarantees.

See [protocol architecture](../architecture/authoring-protocol.md),
[production schema](../../crates/pivot-protocol/proto/pivot/authoring/v1/authoring.proto), and
[interop test plan](../architecture/authoring-protocol-tests.md).

## Verification / reconsideration trigger

Accepted after the 2026-10-18 review refinements and project-owner authorization.
Verify schema placement/tags, v1.1 negotiation, strict mutation validation,
attachment lifecycle, cursor consistency, receipt diagnostics and shared daemon
registry ownership before implementing dispatch. Require the Rust/C# positive and
negative compatibility matrix before enabling the capability. Revisit restart and
receipt guarantees when provider-independent persistence is specified; do not infer durable
replay semantics from the current in-memory service.
