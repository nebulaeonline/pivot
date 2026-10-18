# ADR-0006: Working-model lifecycle and transaction boundaries

Status: Accepted
Date: 2026-10-18

## Context

Clients can invoke immutable registered models but cannot author models through the
service. REPL, C#, Rust, and future Avalonia clients need the same authoring API.
Editing cannot mutate published revisions, depend on a terminal's lifetime, or
expose mutable graph objects and process-local IDs through IPC.

## Decision

Introduce a transport-independent model-management layer alongside the invocation
host. It owns editable working models, portable authoring identities, authorization,
revision checks, operation receipts, and publication coordination. The daemon hosts
it; the REPL and GUI call it. This is a logical boundary, not a requirement for a
new crate or a particular synchronization/storage framework.

Working models are service resources independent of IPC connections. Open returns
an authorized attachment to an existing resource, not exclusive ownership. Closing
or disconnecting releases an attachment, not the model. Initially retain working
models in memory until explicit authorized deletion or host shutdown; enforce quotas
instead of silently expiring a disconnected user's work. Clearly report that this
does not survive a crash or restart without future persistence.

Apply ordered edit batches privately and commit all changes under one expected
working-revision comparison. Success advances the public working revision once;
failure exposes none of the batch. Invalid formula drafts may be saved with
diagnostics, while malformed commands and structural identity violations reject the
batch. Existing bound references retain stable identity through edits and renames.
Preserve original submitted formula text, including incomplete drafts. Newly
submitted formulas bind against the final candidate directory for the batch;
existing bound expressions are never silently rebound.

Capture exact working revisions for validation and unpublished evaluation without
holding an edit lock during calculation. Publish only a validated immutable snapshot
and only if a final revision check still succeeds. Published revisions and active
calculations remain unchanged by later edits or deletion of a working model.
Working revision, calculation snapshot and published revision are distinct
identities. A snapshot implies neither publication nor retained result retrieval.

Use client operation IDs and bounded, retained receipts for mutation ambiguity,
separate from request correlation. Replays of the same operation and payload cannot
apply an edit twice within the same service instance. Restart invalidates the initial
in-memory receipt guarantee; do not imply durable exactly-once operations.
Receipt replay lookup precedes expected-revision conflict checking. Reusing an
operation ID with different intent is an error. Reserve receipt capacity before
commit; installing state and its successful receipt is one indivisible logical
commit. Refuse new mutations rather than evicting guaranteed receipts. Retain
resource receipts for the resource lifetime, and deletion receipts through the
service instance lifetime.

The detailed operations, failure behavior, examples, and review questions
are in [working-model service design](../architecture/working-model-service.md).
Acceptance establishes the architectural baseline. Implement in bounded stages:
management foundation with replay-safe creation, transactional editing, then
unpublished evaluation/publication. Authoring protocol and clients follow those
in-process foundations. Persistence and detailed storage choices remain deferred.

## Alternatives considered

- Connection-owned models: an idle timeout or terminal crash destroys work.
- REPL-owned editing: other clients need a competing management implementation.
- Direct mutation of host registry entries: violates immutable revision semantics.
- Independently committing each edit: exposes partial multi-node changes.
- One global edit lock during calculation: prevents unrelated edits and calculations.
- Automatic retry using request IDs: correlation does not resolve mutation ambiguity.
- Immediate persistence implementation: prematurely chooses durability and file format.

## Consequences

Authoring introduces a distinct authorization and lifecycle boundary. A client must
track its observed working revision and handle conflicts; no automatic merge is
promised. Service identity must distinguish a restart from an ordinary reconnect.
Node/port identity mappings and receipts need bounded resource accounting.

Current core single-edit methods, snapshot-only host registration, and process-local
revision types do not yet implement these transaction and recovery guarantees.
Private staging/capture APIs and an atomic registry insertion boundary will be needed.
Their storage strategy must remain replaceable. Decimal-only execution remains;
new numeric types, workbook addresses, and scripting are separate proposals.

## Verification / reconsideration trigger

Accepted on 2026-10-18 following parent review and project-owner authorization.
Before shipping,
require conflict, rollback, disconnect/replay, restart, authorization, snapshot,
publication, and bounded-resource tests described in the design document.
Revisit lifecycle when persistence or multiple daemon processes are introduced.
