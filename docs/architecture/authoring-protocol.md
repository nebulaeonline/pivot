# Authoring protocol proposal

Status: Accepted — ADR-0007. Schema/codecs, opt-in negotiation and bounded validation
are implemented. Stage A serialized dispatch and Stage B opt-in daemon authoring
are implemented. See [daemon integration](daemon-authoring.md), [adapter implementation](serialized-authoring-adapter.md)
and [foundation implementation](authoring-protocol-foundation.md).

## Versioning and envelope integration

The foundation supports v1.1 and `working-models-v1`, including `decimal96`.
Production daemon negotiation remains v1.0/invocation-only unless `--authoring` is enabled.
Existing v1.0 clients retain their original contract. Foundation negotiation is an
explicit opt-in API; codec installation does not advertise authoring automatically.

Shared Decimal128, Value, ValueType, ModelRef and CalculationErrorCode now live in
`pivot/v1/types.proto`, preserving fully qualified names and wire layouts. The
authoring schema imports types; the envelope imports both, without an import cycle.
Request.operation.authoring and Response.outcome.authoring use tag 20. Authoring
request operation tags 1–12 and response outcome tags 1–13 are production allocations.
See the production schema and compatibility evidence in the foundation document.
The earlier docs/proposals schema is historical and no longer authoritative.

An unnegotiated authoring request fails with existing envelope errors; a negotiated
authoring request returns either its typed result or AuthoringFailure. A frame that
cannot safely be decoded/correlated retains existing framing/error behavior. An
old codec need not understand an authoring operation; old clients never send it.
Future additions must preserve this capability/version discipline.

## Operation contract

| Operation | Inputs | Result / permission |
| --- | --- | --- |
| GetServiceInfo | None after handshake | Instance, durability, all ceilings, receipt/attachment/cursor policy; Info permission |
| CreateWorking | Instance, operation ID, display name | Working ID, exact revision; Create |
| ListWorking | Instance, page | Authorized current summaries; List plus per-entry Inspect |
| OpenWorking | Instance, working ID | Connection-bound attachment and observed revision; Open |
| CloseAttachment | Instance, attachment ID | Closed acknowledgement; owner/context plus Close |
| ReadDraft | Exact revision, section, page | Nodes or ports at that head; Inspect |
| ReadDiagnostics | Working revision, page | Portable diagnostics; Inspect/Validate/Evaluate; receipt detail paging deferred |
| ApplyEdits | Instance, working ID, expected revision, operation ID, ordered edits | New revision, surviving temporary mappings, diagnostic summary; Edit |
| ValidateWorking | Exact revision | Valid flag and diagnostic summary; Validate |
| EvaluateWorking | Exact revision, selected node IDs, overrides | Provenance, effective inputs, per-output values/errors; Evaluate |
| PublishWorking | Instance, working ID, expected revision, operation ID | Immutable ModelRef and snapshot provenance; Publish |
| GetOperationOutcome | Instance, operation ID | Unknown, recorded create/edit/publication result or rejection; recorded action / Recover |

All required message fields are semantically required even where proto3 has no
`required` keyword. Enforce presence for nested values and oneofs, nonempty strings,
explicit nonzero enum variants, duplicate rejection and range checks before service
admission. There is no delete, fork/import, destination-lineage, standalone capture,
remote prepare/commit or result-retrieval operation in the initial contract.

Service info must be separately authorized rather than borrowing Create or exposing
resource metadata. The service and adapter now implement the separate Info action.
The connection's authenticated identity supplies every principal; no principal,
user or owner claim is accepted from the request. Rights remain independent from
published invocation Discover/Describe/Invoke authorization.

## Portable identity and definitions

Use canonical lowercase hyphenated UUID strings for service, working, authoring
node/port and snapshot IDs. Reject malformed/noncanonical authoring IDs at admission.
Published ModelRef retains its existing string contract. WorkingRevision explicitly
contains service instance, working model ID and unsigned edit sequence; zero is a
valid initial sequence. Never pass a JSON/double representation of uint64 sequences;
generated C# ulong and Rust u64 preserve them.

Publications return separate ModelRef identities and never imply the working edit
revision advanced. One working resource maps to one service-assigned published
lineage initially. Stable published port IDs follow the current management mapping;
renaming preserves them, replacement creates new ones. No process-local identity
or Rust memory representation is a wire identifier.

Raw nodes contain source/decimal definition, portable bound dependencies and their
diagnostics via a separate stream. Dependencies may identify deleted tombstones.
Do not reconstruct binding by parsing source: source may use pre-rename labels or
be invalid. Clients may display/repair text, but the service alone binds and executes.
Public ports identify portable authoring targets, direction and explicit requirement.
An invalidated saved port remains readable. Decimal is the sole executable value
type; reject float64 and unknown value kinds until a future capability defines them.

## Edits and exact intent

Typed edit oneofs mirror Slice B: add/remove/rename/replace node; add/remove/rename
input/output port; update input target/requirement and output target. Definitions
and required/optional declarations are explicit oneofs. Temporary IDs use the
existing one-batch shared namespace and ordered reference rules. Formula forward
references bind against the final directory; existing bindings never rebind.

Mutation keys are `(instance, authenticated principal, operation ID)`, shared across
Create/Edit/Publish. Request ID, connection, attachments and cursor are excluded
from mutation intent. All typed operation content, including expected revision,
edit order, names, temporary IDs, source bytes and exact decimal limbs/sign/scale,
is included. A reconnect may use a fresh request ID with identical operation intent.
Compare decoded typed intent, not encoded bytes or a raw Protobuf hash. Field order
and equivalent valid wire encodings must not change intent.

Use schema-aware preflight validation for mutations: reject unknown fields,
duplicate singulars, conflicting/missing oneofs, unknown enums and invalid values
before receipt admission. Preserve legitimate ordered repetition and permitted
wire representations with identical typed intent. The foundation implements this
boundary; the in-process adapter routes raw frames through it before generated
decoding. Read-only response additions retain normal unknown-field compatibility.
Existing invocation validation is unchanged. Codecs alone are not preflight.

After authorization and shape checks, receipt reconciliation precedes expected-head
checks. Identical replay returns its original typed success/rejection; changed intent
returns OperationIdConflict. Quota full does not prevent existing receipt replay.
State/receipt visibility remains atomic. The wire never acknowledges an in-memory
receipt as a durable save. Unknown means no retained outcome was found, not proof
the original request was never queued; an older request may still arrive.

After a lost response: reconnect, handshake, compare service instance, then query
the original operation ID or explicitly resubmit exactly that ID and intent. Never
automatically choose a new ID. A changed instance returns StaleInstance and ends
the current guarantee; do not silently replay lost edits into a different service.
No durable exactly-once or cross-restart replay promise until persistence design.

## Attachments and connection lifecycle

Open returns an attachment bound to the authenticated principal AND connection;
it is not required to authorize direct resource operations. Multiple attachments
can observe one resource. Reads/edits still check rights independently.

Open/Close are session bookkeeping, not retained resource mutations, so they do
not consume mutation operation IDs. Lost Open may consume an attachment slot until
disconnect; cleanup releases every attachment tracked by that connection. Duplicate
Close for an attachment already closed by that same connection returns success
using a bounded connection-local closed-ID set; unknown/foreign IDs fail without
revealing another principal's attachment. If that set fills, refuse further Opens
on the connection until reconnect. Disconnect drops this set and closes live
attachments; working models and receipts remain. The Stage A adapter implements
and tests this lifecycle over the principal-owned service attachment API.

## Validation, calculation and publication

Validate returns structured report data for draft structural failures and never
changes saving rules. Global snapshot validity remains required; no partial
evaluation of invalid models. Evaluate captures the exact head under bounded
service access, releases the edit lock, executes the existing engine, then checks
permission before sending results. Arithmetic failures belong to individual
outputs; unknown outputs/bindings are request failures, not arithmetic results.

Evaluation inputs report Submitted or Captured source. Captured means the graph's
captured input value, not a public optional default. Published invocation keeps
its existing Submitted/Defaulted contract. Do not reuse one enum with ambiguous
meaning. Effective values cover all captured graph input nodes as in Slice C.

Remote Publish is one operation; the adapter privately runs prepare/commit and
releases service access between phases where appropriate. No remotely retained
prepared token or invisible pending-capture registry. A concurrent edit can cause
RevisionConflict at commit. Publication validates structure/interface, not arithmetic
success for every input scenario. It never implicitly selects a newer head.

Implemented pivotd integration uses one coherent published registry. Built-in/demo
and authored models must appear in the same authorized discovery/describe/invoke
view. Add a controlled bootstrap/import boundary if needed; do not leave a standalone
demo registry alongside an unrelated management registry. Preserve the same logical
publication/receipt commit boundary. See [daemon integration](daemon-authoring.md).

## Errors and diagnostics

| Condition | Proposed category / outcome |
| --- | --- |
| Bad shape, ambiguous mutation encoding, bad IDs/targets | InvalidArgument / InvalidReference, optional edit index |
| Missing capability/version/type | Existing envelope negotiation error or UnsupportedValueType |
| Permission failure | Unauthorized, without graph detail |
| Wrong instance | StaleInstance; client must rediscover service |
| Wrong exact head | RevisionConflict; current revision only if Inspect is authorized |
| Unknown authorized resource | NotFound |
| Expired/changed draft page | ViewExpired; restart the read at a newly chosen head |
| Changed operation intent | OperationIdConflict |
| Graph/capture/work/message/receipt ceiling | ResourceExhausted with safe limit category |
| Invalid draft validation/publication | Structured ValidationFailure with diagnostic summary |
| Arithmetic/dependency failure | Per-output CalculationFailure, portable origin/dependency |
| Internal invariant/registry rejection | Internal or RegistryRejected; no Rust debug strings |

Diagnostics have stable categories, optional node/port identity and expression UTF-8
byte offset. Hard batch failures may carry an edit index when known; do not fabricate
an index for today's coarse core errors. Saved-draft diagnostics are not batch
rejection. Invalid publication's retained failure includes diagnostic summary.
Public invocation results never gain privileged authoring dependencies or definitions.
Do not reveal names, current heads, limit usage or targets on denied access.

## Bounds, paging and consistency

Preserve the existing 1 MiB hard frame/message limit. Propose page_size default 64,
maximum 128 entries and a soft encoded payload target of 512 KiB, with actual encoded
response size checked before sending. Every single definition/diagnostic must fit
the hard limit or fail ResourceExhausted explicitly; never truncate source, dependency
lists or diagnostics. Mutations are atomic and never split automatically across frames.

Report service limits plus adapter ceilings: batch commands/accounting, names/source,
live/cumulative identities, receipts, captures, calculation inputs/outputs/work,
published revisions, pages, attachments and cursor policy. Enforce counts and raw
bytes before allocating where possible. Evaluation results fit the negotiated
input/output and byte ceilings; oversized requests/results fail, not silently page
a calculation into unrelated snapshots. New settings stay configurable.

Use authenticated, opaque, stateless cursor tokens, proposed maximum 512 bytes and
60-second total query lifetime. Bind instance, principal and connection, operation,
source, revision or receipt key, section, position, page options and expiry. Future
implementation needs per-instance signing material; clients never interpret tokens.
Reauthorize each page. Tokens do not confer access and are invalid after disconnect
or restart. Reconnecting clients restart paging, including receipt diagnostic reads.

Draft node/port/diagnostic pages are sorted deterministically by portable identity
and stable diagnostic keys. Require the exact current working head on every page.
If an edit advances it, return ViewExpired rather than concatenate revisions. No
server-side immutable read-view cache is needed initially; pages may recompute bounded
views. This intentionally trades read stability during active editing for simplicity.

ListWorking is keyset discovery ordered by portable working ID. Each page reflects
then-authorized resources; concurrent creation can appear or fall behind the cursor.
It is not a frozen catalog. Return exact per-entry revisions; clients must still use
revision checks. Do not claim a cross-page atomic catalog view.

Mutation outcomes carry diagnostic summaries and exact original provenance.
Detailed receipt diagnostic paging is deferred: the production ReadDiagnostics
supports only a current working revision and reserves field 3. Never recreate an
old outcome's details from a newer model head. Existing in-process receipts retain
full diagnostics, but lack a dedicated byte-accounted retention/admission policy.
A future extension must retain revision, complete ordered diagnostics (IDs, offsets,
categories and text), original typed intent and outcome, with reserved storage and
retained-byte limits before commit. Exhaustion must reject admission, not truncate
or evict guaranteed details. No receipt-detail capability is currently advertised.

Working diagnostic reads require Inspect, Validate or Evaluate, grant no raw-source
right, and can expire if the head advances. Ordinary published Invoke never qualifies.

## Durability and storage providers

Initial service info reports MemoryOnly, receipt lifetime through resource/service
lifetime as implemented, and no crash/restart recovery. Retained publication does
not mean disk persistence. Do not advertise Durable until recovery behavior is
implemented and tested.

The owner's direction is a common transactional persistence contract with
configurable providers. SQLite is first and the default local provider, with an
explicit filesystem path. PostgreSQL and MySQL adapters are planned, with others
possible. Local/network service mode does not dictate provider choice: a network
daemon may own a local SQLite database, and a local service may use a server database.

Provider configuration belongs to the service, not authoring requests. The wire
reports durability/recovery guarantees independently of database choice; it does
not expose a database path, credentials or SQL schema. Implement SQLite first,
then validate the abstraction using a second provider and common conformance tests.
Initially one active service owns each store. Database-provider support does not
promise safe multi-daemon sharing or coordinated in-memory registries.

Follow [persistence providers and recovery direction](persistence-providers.md)
for the separate design agenda: provider migrations, stable binding restoration,
atomic durable state/receipt commits, exact decimals, ambiguous commit recovery
and service/store ownership. None of those providers is implemented in this slice.

## Review gates and implementation gaps

Production schema allocation and codec/validation foundation are complete.
Stage A implements error mapping, byte-aware paging/signed cursors, attachment
cleanup, Info authorization and staged partial-port patches. Original decoded
wire intent is retained in the existing edit receipt. Omitted fields resolve
against the staged transaction candidate. Stage B adds unified daemon registry
bootstrap and live authoring IPC behind explicit enablement. REPL authoring remains deferred.

Port update semantics: omitted means unchanged, supplied means replacement, and at
least one update field is required. Present-but-empty formula source is valid draft
content; missing definition oneof is invalid. Working edit sequence zero is valid.
Model-size ceilings are configurable advertised policy, not permanent wire maxima.
The initial transport ceiling remains 1 MiB; future bulk mechanisms need independent
negotiation rather than raising model limits by changing protocol major version.

See [verification plan](authoring-protocol-tests.md) and
[foundation evidence](authoring-protocol-foundation.md). Persistence/provider recovery
design remains separate and near-term.
