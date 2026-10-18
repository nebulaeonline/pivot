# ADR-0007: Versioned working-model authoring protocol

Status: Proposed
Date: 2026-10-18

## Context

ADR-0006 authoring works in process, including transactional drafts, detached
calculation and immutable publication. External clients currently invoke only
published models. They need one portable contract without exposing Rust handles,
moving evaluation into clients, or weakening revision/replay guarantees.

## Decision proposal

Extend the existing envelope additively in a negotiated v1.1 minor version, gated
by `working-models-v1`. Preserve v1.0 published invocation messages and behavior.
Do not enable capabilities or freeze new tags before this proposal is reviewed.
Use an isolated review-only schema, not the production code-generation inputs.

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
same exact current head; fail explicitly if it advances. Operation diagnostic
pages derive from retained receipts. Discovery is explicitly best effort, not an
atomic catalog snapshot. No unbounded hidden read-view retention is introduced.

The future daemon must expose one coherent publication registry for built-in and
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

Adapters need strict mutation validation, portable error mapping, byte-aware
pagination, principal-bound cursor verification and budgeted off-lock calculation.
These are future implementation tasks, not capabilities of this documentation.
Wire receipt diagnostics must remain available without depending on a newer head.
Existing Rust/C# generated invocation codecs remain unchanged in this slice.

See [protocol architecture](../architecture/authoring-protocol.md),
[review-only schema](../proposals/authoring-v1/authoring.proto), and
[interop test plan](../architecture/authoring-protocol-tests.md).

## Verification / reconsideration trigger

Review schema placement/tags, v1.1 negotiation, strict mutation validation,
attachment lifecycle, cursor consistency, receipt diagnostics and shared daemon
registry ownership before implementing dispatch. Require the Rust/C# positive and
negative compatibility matrix before enabling the capability. Revisit restart and
receipt guarantees when provider-independent persistence is specified; do not infer durable
replay semantics from the current in-memory service.
