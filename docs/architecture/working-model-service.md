# Working-model service proposal

Status: Architectural baseline accepted in ADR-0006, with staged implementation.
The [management foundation](working-model-management.md) implements Slices A and B,
including [transactional editing](working-model-editing.md). Unpublished evaluation,
publication and authoring Protobuf operations remain deferred.

Expose graph authoring to arbitrary clients while preserving the existing immutable
invocation host. Keep model management independent of transport, REPL, workbook
coordinates, and GUI. API spellings and deferred implementation details below
remain provisional; acceptance does not authorize all slices at once.

## Boundaries

```text
REPL / C# / Rust / future Avalonia clients
                  |
     versioned authoring protocol adapter
                  |
     model-management service
     resources / edits / revisions / authorization / receipts
             |                       |
       editable Rust core      publication coordinator
             |                       |
      immutable calculation     existing invocation host
           snapshot            immutable external revisions
```

The management layer controls working documents and publication, not spreadsheet
structural semantics. A workbook projection can later bind sheet coordinates to
graph identities. No A1 address becomes a canonical model/node identifier.

Evaluation contexts and caches remain local to each calculation. Published model
invocations continue independently of editing, using their exact registered revision.
Do not route invocation through a mutable working model.

## Identities and lifecycle

| Identity | Proposed meaning |
| --- | --- |
| ServiceInstanceId | New opaque value for each in-memory service lifetime; detects restart |
| WorkingModelId | Service-assigned logical editable resource, independent of connections |
| AttachmentId | Revocable, authorized association for an opened working model; not a bearer credential |
| WorkingRevision | Opaque edit revision scoped to a working model and service instance |
| AuthoringNodeId / AuthoringPortId | Portable editing identities mapped privately to core handles |
| SnapshotId | Exact captured working state, distinct from working head and publication |
| Published ModelRef | Existing portable model_id/revision_id pair in the invocation registry |
| ClientOperationId | Client-generated mutation identifier, distinct from request_id |

Suggested wire IDs are opaque UUID-style strings within negotiated bounds. UUID
allocation, field tags, and durable identity encoding are not selected here.
Process-local NodeId, PortId, and core Revision values must not be serialized.
Neither an attachment nor possession of an ID grants authorization.

CreateWorkingModel allocates a resource and initial revision. OpenWorkingModel
attaches to an existing authorized resource and returns its current revision; it
does not create a private mutable copy. Multiple attachments may edit one resource.
Create may optionally fork an explicitly authorized published revision into a new
working resource, with origin lineage recorded. Opening a published model itself
does not make that revision mutable.

For the first implementation, retain resources in memory until explicit deletion
or service shutdown. Disconnect, idle timeout, CloseWorkingModel, and abandoned
attachments do not delete the working model. Allow listing authorized resources
after reconnect. Bound resource counts/bytes; when full, reject new allocations
instead of silently evicting edits. Attachment expiry may release attachment-only
resources, but never implies deletion of the underlying model.

Service shutdown or crash loses unpublished work in this initial in-memory design.
Reconnect is not restart recovery. A new ServiceInstanceId makes expired handles,
revisions, and mutation receipts unambiguously stale; never reuse their identity.
Future persistence can restore logical resources and receipts through a versioned
storage interface, but acknowledgment currently means in-memory commit, not fsync.

DeleteWorkingModel requires manage permission, expected revision, and an operation
ID. It removes the resource from new opens/edits. Published revisions remain.
Already captured calculations may finish against their immutable snapshots, subject
to result authorization. Retain a bounded tombstone/receipt sufficient to reconcile
the deletion for the service lifetime; deletion must not erase its own receipt.

## Atomic edits and stable references

ApplyEdits carries service instance, working identity/attachment, expected revision,
ClientOperationId, and an ordered list of typed edits. Candidate edit families:

- Add/remove/rename an input or computed node; update an input value/type.
- Replace a formula, or explicitly change a node definition kind.
- Declare/remove/rename/rebind a public input/output port.
- Set required versus optional input policy and explicit typed defaults.

These are operation proposals, not new core methods. Decimal is the only executable
type initially; reuse the extensible typed-value boundary and reject unsupported
types explicitly. No implicit float/decimal coercion is introduced.
Reject empty batches. A successful nonempty batch advances the service revision
once even if its final values happen to equal the preceding values.

Proposed processing:

1. Authenticate/authorize the request; check bounds and operation identity.
2. Look up an existing receipt before applying expected-revision checks on a replay.
3. Capture a private candidate at the expected revision, or return RevisionConflict.
4. Allocate tentative identities and apply structural edits privately in order.
5. Compile only newly submitted formulas against the candidate's final directory.
6. Check hard invariants and resource limits; collect draft diagnostics separately.
7. Recheck permission and expected head at commit. Atomically install the candidate,
   advance the service revision once, and record the operation receipt.

Readers observe either the old committed state or the complete new state. Other
writers can win while a candidate is being prepared; the final comparison rejects
the loser without changing visible state. No automatic rebase or conflict merge.
Core methods may advance internal revisions several times during staging; those
are not the public working revision. Staging IDs consumed by failed batches may
be burned, but must never alias later visible entities.

Temporary IDs within a batch let later edits refer to nodes created by that batch.
Return temporary-to-portable mappings only on commit. Structural command ordering
is significant: duplicate live names during structural edits reject the batch;
clients can use a temporary name to swap labels. New formulas bind after structural
edits, allowing forward references to nodes created later in the same atomic batch.
This final-directory rule is a proposed service extension to today's single-edit
binding boundary; it does not permit snapshot-time rebinding.

Existing valid formulas are never reparsed or rebound as a side effect of rename,
publication, or snapshot capture. Deleting and recreating a name cannot redirect an
existing reference. Portable authoring references resolve to stable core IDs once;
only private bound representations execute. Original text is provenance, not the
authority after a referenced label changes. A future formatter can render current
labels from bound identities without string replacement.

## Draft validity versus request validity

| Condition | Proposed outcome |
| --- | --- |
| Malformed command, unknown edit kind, unsupported type, invalid encoding | Reject whole batch |
| Wrong revision, unauthorized action, duplicate operation with different intent | Reject without edits |
| Target handle belongs to another resource, unknown temporary ID, duplicate live label | Reject whole batch |
| Newly submitted formula has invalid syntax or unresolved name | Save draft text and diagnostics atomically |
| Deleting a referenced node leaves a broken reference | Save deletion; preserve original target identity and report diagnostic |
| Cycle or previously valid port made invalid by a later edit | Save draft state; validation/capture reports the problem |
| Resource/work/receipt budget unavailable | Reject before commit |

Initially keep existing hard declaration checks, such as trying to declare an input
port against a computed node. Creating a reference to an entity that never existed
is different from retaining a reference broken by a deliberate deletion. Diagnostics
must distinguish invalid source, unresolved name, deleted identity, and cycle.
Incomplete formulas need explicit resubmission after missing names are created,
unless resolved within the same new batch. No speculative automatic name rebinding.

ApplyEdits success means the draft was saved, not that it is executable. Return the
new working revision and bounded diagnostics/validation status. Global validation
can be expensive; it need not run fully under the commit lock. Never label diagnostics
for revision R as results for a later head R+1.

## Snapshot capture and unpublished evaluation

ValidateWorkingModel and EvaluateWorkingModel require an exact expected working
revision. Capture its immutable definitions, input values, and current calculation
policy under a short consistent access boundary. Release editing access before
full validation or evaluation. A later edit does not cancel or change that capture.
Return the working identity, captured revision, and calculation/snapshot identity.
Pin numeric semantic/policy identity with the capture; initially this is the
existing fixed decimal behavior, not a new user-selectable calculation policy API.

Initially allow capture only at the current head; a requested older working revision
returns RevisionConflict or RevisionUnavailable rather than silently evaluating
the head. Do not promise working-revision history retention. Callers needing a
repeatable long-lived model should explicitly publish a revision.

Interactive evaluation uses captured input values with optional temporary overrides
of input identities. It does not enforce public required-port invocation rules, and
does not edit inputs or advance the working revision. A public preview can instead
explicitly choose strict named-interface invocation semantics. These must be distinct
operations/modes, never an implicit choice based on which fields were omitted.

Authorized authoring evaluation may select internal portable node IDs and return
authoring diagnostics; ordinary published invocation still exposes only public ports.
First implementation retains global structural snapshot validation. Selected outputs
cannot yet bypass an unrelated malformed formula or cycle. Per-node arithmetic
errors remain result data and do not poison independent valid output branches.

A SnapshotId here identifies the capture in the returned result; it does not promise
retained snapshots, result retrieval, or durable storage. Existing source/default
provenance and explicit rounding semantics remain unchanged. Evaluation cancellation
and asynchronous completion are separate future capabilities.

## Publication boundary

PublishWorkingModel requires separate publish permission, expected working revision,
destination logical model identity, and ClientOperationId. No edit auto-publishes.

1. Capture the expected revision and its portable identity/port mappings.
2. Validate the entire candidate and public interface outside long-held edit locks.
3. Prepare an immutable snapshot, new external revision ID, and registry insertion.
4. At commit, recheck authorization and that working head is still the expected
   revision; coordinate one atomic registry insertion and publication receipt.

If edits advanced the head while validation ran, return RevisionConflict and expose
no publication. The owner can deliberately retry against the desired current state;
publication never silently picks a newer revision. Once published, later edits are
allowed and cannot alter that revision. Publication records source lineage but does
not advance the working edit revision because definitions did not change.
Deletion or revocation before that commit also prevents publication; a prepared
candidate is not an independently authorized way to bypass the deleted resource.

The working draft need not have executable captured default input values: structural
and interface validation is required, but division by zero for a particular input
is a calculation error, not automatically a publication defect.

Portable public port identities persist through renames within a logical model
lineage; removing/recreating a port allocates a new identity. Forking to a different
logical published model uses a new model-scoped public port identity mapping.
Never copy internal handles into the protocol or infer portable identity from names.

Implementation must adapt the existing live registry's PortId-continuity checks
when a working model is reconstructed or forked with different runtime handles.
Carry validated logical lineage and remap bound references explicitly; do not
recreate expressions by rebinding historical names. This is not supported today.

The candidate may be prepared privately, but no other client may observe a registry
entry without its complete snapshot/interface/mapping or a contradictory operation
receipt. An initial single-process coordinator can serialize that small commit
boundary. Storage/lock choice and future cross-process publication are not selected.
Invalid drafts, destination authorization failures, allocation failure, and conflicts
must leave the registry untouched. Crash-durable atomic publication awaits persistence.

## Reconnect, idempotency, and failure semantics

Mutation ClientOperationIds are scoped to the authenticated principal and service
instance, including creation before a WorkingModelId exists. A receipt binds operation
kind, target, expected revision, and canonical typed intent. Include formula source
and exact supplied value representation in intent comparison; ordinary numeric
equality does not make two edit payloads identical. Canonical intent encoding and
digest algorithm require protocol review; do not hash noncanonical Protobuf bytes.

Concurrent duplicate submissions reserve one receipt entry before executing. Same
ID/same intent returns the recorded outcome or OperationPending, never reapplies it.
Same ID/different intent returns OperationIdConflict. A replay lookup precedes the
expected-revision check so a successful edit can be reconciled after its old expected
revision has become stale. Recheck current authorization before revealing receipts.

Retain compact terminal outcomes and committed identity mappings throughout this
in-memory instance's advertised receipt lifetime. Proposed initial guarantee is the
resource lifetime, plus deletion receipts through instance lifetime. Bound ledger
space and reserve it before commit; reject new mutation admission rather than
silently evicting receipts and weakening an advertised at-most-once guarantee.
Initial quotas and any later explicit expiry negotiation remain review decisions.

GetOperationOutcome returns Committed, Rejected, Pending, or UnknownOperation.
An unknown result does not prove a previously issued request cannot still arrive
or be queued. Clients may reconcile/reissue only with the same operation ID and
intent in the same service instance; never automatically issue a fresh mutation ID
after ambiguous disconnect. A client may explicitly abandon reconciliation, but
must not assume an earlier mutation was rolled back. Server-side at-most-once
protection is not an instruction to auto-retry invocations.

Admission/authorization/encoding failures that cannot reserve a receipt do not
claim a retained terminal outcome. Deterministic rejection after reservation is
recorded. If a disconnect happens before commit, policy may stop preparation; if
commit already happened, connection loss cannot undo it. At commit, install state
and receipt together. Acknowledgment follows that boundary, not vice versa.

| Failure | Observable guarantee |
| --- | --- |
| Two writers use the same expected revision | At most one commits; other sees conflict |
| Edit N in a batch fails a hard invariant | No edits from that batch are visible |
| Client disconnects after commit, before acknowledgment | State remains; same-ID receipt reconciliation |
| Client idles or exits its REPL | Working model remains; attachment may close |
| Model deleted during an existing calculation | Captured state stays immutable; no new opens/edits |
| Authorization revoked before commit/result delivery | Deny new commit or result disclosure; cancellation remains best effort |
| Service restarts without persistence | New instance identity; old resources/receipts unavailable, no replay guarantee |
| Publication fails its final revision comparison | No new published entry |

## Authorization and resource controls

Separate permissions: list/read working metadata, inspect definitions, edit,
validate/evaluate, publish into a model namespace, manage/delete, and invoke public
published revisions. Forking needs permission to read/copy the source definition;
invocation permission alone does not reveal private graph internals. Open handles
are revocable authorizations, not permanent grants. Check access on every request
and again at commit or delayed result delivery. Filter discovery and deny before
existence-revealing lookup where appropriate.

OS current-user IPC restrictions remain the initial peer identity boundary. They
do not automatically authorize all same-user processes to edit every model. The
management policy must be explicit; do not silently reuse the demo's AllowAll.
Multi-user ownership/sharing and authentication systems remain deferred.

Bound messages, edit counts, source lengths, graph size/work, concurrent captures,
retained drafts, attachment metadata, diagnostics, and receipt bytes. Wire limits
are not ultimate engine capacity limits. Page large reads/diagnostics and associate
every page with an exact revision; if that captured view expires, fail explicitly
rather than mixing revisions. Quota rejection must leave edits/publication unchanged.
No globally mutable calculation cache or unlimited disconnected-session retention.
Paged authoring reads need a short-lived, bounded immutable raw-draft view, including
invalid definitions; they cannot simply reuse today's globally validated ModelSnapshot.
Read-view tokens and their expiry are distinct from diagnostic calculation SnapshotIds.

## Proposed service and protocol operations

These are descriptive signatures, not generated schema or implemented APIs. A future
adapter negotiates a `working-models` capability and explicit compatible version;
older clients continue published invocation unchanged. No field tags are allocated.

| Operation | Main request | Main response |
| --- | --- | --- |
| GetServiceInfo | Existing negotiated/authenticated context | Instance identity, authoring capability, limits, durability/receipt policy |
| CreateWorkingModel | Instance, operation ID, metadata, optional exact authorized source | Working ID, initial revision, attachment, receipt |
| ListWorkingModels | Authorized scope, cursor | Bounded resource metadata; no private definitions |
| OpenWorkingModel | Instance, working ID | Attachment, current revision, permissions, lifecycle policy |
| ReadWorkingModel | Attachment, expected revision, selection/cursor | Paged nodes/ports/directory with authoring IDs and exact revision |
| ApplyEdits | Instance, attachment, expected revision, operation ID, edit batch | New revision, committed identity mappings, bounded diagnostics/receipt |
| ValidateWorkingModel | Attachment, expected revision | Captured revision, structured diagnostics and truncation/paging metadata |
| EvaluateWorkingModel | Attachment, expected revision, selected node IDs, temporary typed overrides | Snapshot provenance and authoring node values/errors |
| PreviewPublicInterface | Attachment, expected revision, public typed inputs/outputs | Strict named-interface calculation and source working revision |
| PublishWorkingModel | Instance, attachment, expected revision, destination, operation ID | Exact new ModelRef, source revision, port mapping, receipt |
| GetOperationOutcome | Instance, operation ID, authorized scope | Pending/Committed/Rejected/UnknownOperation |
| CloseWorkingModel | Attachment | Attachment released; resource retained |
| DeleteWorkingModel | Instance, working ID, expected revision, operation ID | Tombstone/receipt; published revisions unchanged |

Proposed typed failures include RevisionConflict with authorized current revision,
StaleServiceInstance, UnknownWorkingModel, InvalidAttachment, InvalidEdit, unsupported
type, identity/reference failure, OperationIdConflict, OperationPending,
ResourceExhausted, and authorization failure. Draft diagnostics and arithmetic errors
are separate from request rejection. Published invocation errors remain unchanged.

## Minimal end-to-end use cases

### Create, edit, evaluate, and publish

1. Client negotiates authoring and learns instance S; creates W with operation C.
2. At revision R0, apply batch E: create Revenue/Costs input nodes, Net and Margin
   computed nodes, and independently named required/defaulted public ports.
3. Commit returns R1 and stable authoring identities. Evaluate W at R1 with captured
   values; a parallel temporary scenario cannot alter W or another calculation.
4. Publish operation P against R1 creates ModelRef M/V1. Existing invoke clients
   can now use M/V1 without an attachment or knowledge of W's graph.
5. Edit W to R2. Invoking M/V1 still uses R1's published definitions and interface.

### Two writers and incomplete drafts

1. A and B open W at R1. A submits E1; it commits R2.
2. B submits E2 expecting R1; no edits commit. B reads R2 and deliberately revises
   its batch with a new operation ID. No automatic last-writer-wins merge.
3. A saves an incomplete formula at R3. Read/repair remain available; global capture
   validation rejects evaluation/publication until a later explicit edit repairs it.

### Lost response and reconnect

1. E commits R4 but its connection disappears before the response.
2. A reconnects and authenticates, verifies the same S, reopens W, and queries E.
3. The receipt returns R4 and mappings; repeating E does not create R5.
4. If S changed after restart, report unavailable in-memory work. Do not pretend
   that an old attachment or operation ID refers to new state.

### Concurrent editing during evaluation/publication

Evaluation captures R4, releases edit access, and returns an R4 result even if W
reaches R5 meanwhile. Publication prepared from R4 performs its final compare at
commit; if W is R5, publication fails without adding a registry revision. Concurrent
published invocations remain independent of both activities.

## Implementation gaps and review gates

Current WorkingModel mutation methods are individually visible and not a transaction
API; it is not automatically cloneable. ModelSnapshot only represents globally valid
graphs. Portable authoring IDs, raw draft reads, staging/rollback, fork/remapping,
receipts, and coordinated publication do not exist. Initial implementation must add
private core/service boundaries without exposing a clone/locking/storage mandate.
Do not implement a second parser/evaluator in the client or management service.

Before approval, resolve the proposed in-memory retention/quota policy, whole-batch
revision semantics, final-directory binding of newly submitted formulas, draft error
classification, receipt lifetime/admission, and final-head publication check. Treat
operation names and identity encoding as proposals until protocol review.

Required implementation tests: atomic read visibility and rollback; writer conflicts;
rename/deletion identity preservation; new-batch forward references; retained invalid
drafts; evaluation while editing; publication CAS and immutable old invocations;
lost Create/Edit/Publish/Delete responses and same-ID reconciliation; payload mismatch;
disconnect retention; restart rejection; authorization revocation; bounded resources;
stable portable port lineage; and old-client invocation compatibility.

Deferred backlog: C# IPC and Native AOT, failure-sensitive batch mode, expression REPL,
incremental recalculation, persistence/recovery, workbook projection, and Avalonia UI.
