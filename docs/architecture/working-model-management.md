# Working-model management foundation

Implemented Slices A–C of [ADR-0006](../adr/0006-working-model-lifecycle-and-transactions.md)
in `pivot_host::management`. The [Stage A adapter](serialized-authoring-adapter.md)
exposes serialized Protobuf authoring in process. [Stage B](daemon-authoring.md)
connects it to the daemon behind explicit enablement. REPL and C# executable
clients do not expose authoring yet.

## Service and identity

`WorkingModelService::new(Limits, Box<dyn Authorizer>)` allocates a fresh random
UUIDv4 `ServiceInstanceId`. `info()` reports that ID, limits and explicit
`Durability::MemoryOnly`. A successful operation is an in-memory commit, never
a crash-durable save. Reconstructing the service changes its instance, even in
the same process. All resource requests supply the expected instance; stale
instances fail before resource lookup or operation replay.

Working-model and attachment IDs are independently allocated UUIDv4 values.
They support text formatting/parsing for portable API adapters, but are not
credentials. No ID derives from a core NodeId, PortId or Revision. The service
owns the private core `WorkingModel`; it never returns a graph reference.
`WorkingRevision` includes instance, working-model ID and sequence. New empty
models have sequence zero. Calculation and published revision identities are
separate. Authoring node/port IDs are independent UUIDv4 values mapped privately
to core handles; see [transactional editing](working-model-editing.md).

## In-process API

| Method | Behavior |
| --- | --- |
| `info` | Service identity, memory-only durability, configured quotas |
| `create` | Create a named empty working graph, return ID and exact revision |
| `operation_outcome` | Principal-scoped create/edit/publication outcome, or Unknown |
| `list` | Authorized bounded resource descriptions, filtered per resource |
| `inspect` | Name, exact revision and current graph/interface counts |
| `read_draft` | Owned definitions, stable references and diagnostics at the exact current revision |
| `apply_edits` | Private staging and atomic expected-revision batch commit |
| `open` | Allocate another authorized, principal-owned attachment |
| `close` | Release an attachment without deleting its resource |
| `validate_working` | Structured global validation at the exact current revision |
| `capture_evaluation` | Detached immutable capture, followed by local `evaluate` |
| `prepare_publication` / `commit_publication` | Capture, then atomically register an exact immutable revision with its receipt |
| `publish` | Synchronous convenience for prepare/commit |
| `published_host` | Read-only access to the coordinated invocation registry |

Create does not implicitly open an attachment. Multiple principals can attach
when policy permits. Attachment closure is the disconnect-equivalent lifecycle
action; the Stage A adapter releases its tracked attachments on close or Drop.
There is no idle deletion, resource eviction, or connection ownership. Resources
and receipts currently live for the entire service lifetime. Explicit resource
deletion is deferred; this slice has no delete API. Dropping/restarting the service
loses all resources, attachments and receipts.

Names are display metadata, not unique resource keys; two models may share a
name. Creation supports empty models only, not published-model forks. Description
counts reflect committed graph edits. Invalid formulas remain inspectable.

## Authorization

The embedder supplies a trusted, nonempty principal string (maximum 256 UTF-8
bytes). This API does not authenticate strings from an untrusted client.
`management::Authorizer` is independent of invocation `pivot_host::Authorizer`.
Actions are Info, Create, List, Inspect, Open, Close, Edit, Recover, Validate, Evaluate
and Publish. Global checks use no model ID;
resource checks supply its portable ID. Listing requires List and filters entries
with Inspect. Open does not grant Inspect implicitly. Closing checks both
attachment principal ownership and model-specific Close permission.

Policy decides resource sharing/ownership; the service does not automatically
grant a creator access to inspect or attach. Creation checks authorization again
after private preparation, before commit. Replays and outcome queries require
current authorization for the recorded operation (Create, model-specific Edit or Publish)
and are scoped to the authenticated principal. Unknown outcome queries require
the global Recover action. Draft reads require Inspect, independently of Edit.
Possessing an ID or another principal's operation ID grants no access. Policy
callbacks must return normally; panicking policy callbacks are not a recovery API.

## Creation receipts and logical commit

`CreateRequest` contains instance, client operation ID and name. Operation IDs are
1–128 ASCII alphanumeric, dot, underscore or hyphen bytes. Name is nonempty and
at most 256 UTF-8 bytes. Exact name bytes define intent; no normalization occurs.
Receipt keys are `(service instance, principal, operation ID)`, independent of
attachments or any future request correlation ID.

After context, permission and shape validation, lookup the receipt **before**
quotas or allocation. Same key and same name returns the original result; changed
name returns OperationIdConflict. A lost create response can be recovered by
`operation_outcome` or explicit same-ID/same-intent replay. It cannot create a
second resource. Edit receipt lookup also precedes expected-revision checks.

Reserve receipt-vector capacity before preparation. Reserve resource-vector
capacity before constructing the private candidate. Allocate identities and check
final authorization before state installation. Resource insertion and receipt
insertion then occur under exclusive `&mut self` access with reserved capacity,
no callbacks and no fallible operations between them. No observer can see half a
commit. A process crash loses both; no durable exactly-once guarantee is claimed.
Standard process-fatal allocation failure is outside recoverable API semantics.

Post-admission preparation, quota or final-permission failure records a rejected
receipt and leaves resources unchanged. That rejection is replayed too. Invalid
requests, initial authorization denial, exhausted receipt quota, or failure to
reserve receipt capacity occur before admission and do not create receipts.
`Unknown` therefore does not prove a previously submitted request was never sent;
reconcile only in the same instance with the same operation and intent.

## Limits and synchronization

Defaults: 128 resources, 256 active attachments across the service, 1024 receipts.
All configured limits must be positive. Resource/receipt limits include all
principals and mutation kinds, so a failed admitted create or edit can consume receipt capacity. New mutations
are refused when receipts fill; retained successes/rejections are never evicted.
Replay continues to work when limits are full. Closing frees an attachment slot,
not resource or receipt capacity. Strings and collection counts are bounded;
these are logical quotas rather than an exact allocator byte budget.

Mutating methods require exclusive service access, so concurrent callers must be
serialized by the embedder, for example with a mutex. Read methods share immutable
access. No pending-operation state is observable in this synchronous slice.
UUID collisions are treated with the usual UUIDv4 probabilistic uniqueness
assumption. Future transport, concurrency and storage choices remain replaceable.

## Deferred

Resource deletion/tombstones, persistence, authoring protocol/client operations,
workbook projection and Avalonia integration remain unimplemented. Existing
standalone immutable host APIs and invocation policy are unchanged. The management
service now owns a coordinated registry exposed through read-only `published_host`;
publication mutates it only through the management commit boundary. See
[calculation/publication details](working-model-calculation-publication.md).
