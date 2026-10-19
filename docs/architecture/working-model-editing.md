# Transactional working-model editing

Implemented Slice B of accepted ADR-0006 in
`pivot_host::management`. It now serves the versioned authoring adapter, opt-in
local daemon and minimal REPL; it is not limited to in-process clients. Slice C adds separate
[validation/evaluation/publication APIs](working-model-calculation-publication.md).

## API and revision boundary

`apply_edits(principal, ApplyEdits)` accepts service instance, working-model ID,
exact expected `WorkingRevision`, client operation ID and an ordered nonempty edit
list. A successful batch advances the public sequence exactly once, including a
batch whose final values equal the original state. Core operations may advance
their own internal revision many times; those revisions never leave the service.

`read_draft(principal, WorkingRevision)` returns an owned `DraftView` with source,
definitions, names, portable dependencies, port bindings and diagnostics. It
requires Inspect authorization and the exact current revision. Older revisions
return RevisionConflict, not a view of the new head. No history or read-view token
is retained. `inspect` remains a lightweight metadata/count operation.

## Supported edits

| Family | Operations |
| --- | --- |
| Nodes | Add, remove, rename, replace with decimal input or formula |
| Input ports | Add, rename, remove, update target and required/optional default |
| Output ports | Add, rename, remove, update target |

Node definition replacement preserves node identity, including input/computed
conversion. Port updates preserve both core and portable identity. Input and output
port kinds cannot be interchanged in place; explicitly remove/add to change kind.
Only Decimal is supported. Optional defaults are explicit decimals.

Node and port names use the existing ASCII identifier rules, at most 128 bytes.
Port names share one public namespace; graph names use a separate directory.
Duplicates reject the batch at the edit where they occur, even if a later edit
would remove the collision. Rename swaps can use an intermediate unused name.

## Portable and temporary references

`AuthoringNodeId` and `AuthoringPortId` are distinct UUIDv4 types. Private mappings
connect them to local NodeId/PortId handles. Existing references must belong to
the target working resource and refer to a currently live object.

Add operations declare a temporary identifier (1–64 ASCII alphanumeric, dot,
underscore or hyphen bytes). Later edits in the same batch may reference it using
`NodeRef::Temporary` or `PortRef::Temporary`. Node and port declarations share one
temporary namespace, with no reuse even after removal. Non-formula temporary
references require the object to have been added earlier in the ordered list;
formula forward references use the final name directory described below.

`EditResult.nodes` and `.ports` map temporary declarations to surviving committed
portable IDs. Objects added and removed in the same batch are omitted from these
result maps. Removed identities remain private tombstones for reference diagnostics
and consume the cumulative identity budget. They are never aliases for new objects.
Failed candidates expose no allocated portable IDs; later allocations use fresh
UUIDs. Staging may reuse discarded **core** allocation counters, which remain private.

## Staging, binding and diagnostics

The core's explicit `WorkingModel::stage()` copies raw state and existing compiled
bindings into a private branch, preserving stable handles. It is deliberately not
an implicit Clone or a new independent model. The service clones its identity
mappings with that branch, then applies structural edits in order.

After all structural edits, compile only the final definitions explicitly submitted
by AddNode/SetNode in this transaction, using the **final candidate directory**.
A formula may reference a node added later in the same batch. A final rename may
make an earlier submitted formula unresolved; that is saved as a draft. Definitions
removed or replaced by inputs later in the batch are not compiled as formulas.

Existing bound formulas are copied unchanged. Renaming or deleting/recreating a
name never retargets their dependencies. Existing unresolved formulas also stay
unresolved after later additions; repair requires explicit source resubmission.
Original formula source bytes, including whitespace and incomplete expressions,
are preserved. There is no runtime name rebinding or automatic source rewriting.

Malformed commands, unknown/cross-model targets, duplicate names or temporaries,
new invalid input-port targets and quota failures reject the complete batch.
Incomplete syntax, unresolved names, missing previously bound references, cycles
and ports invalidated by later graph edits are saved with diagnostics. An existing
input port can become invalid when its node becomes computed; the document remains
editable. Adding/rebinding an input port directly to a computed node is a hard error.

Diagnostics carry portable node/port IDs only: parse offset/message, unresolved
name, missing stable reference, invalid port binding, or unorderable graph node.
All local parse/binding/missing-reference and port diagnostics are collected.
Cycle/blocked-node diagnostics use the existing core structural snapshot check;
if an earlier formula error prevents that check, cycle reporting may be incomplete
until that error is repaired. This internal check performs no calculation and no
snapshot is returned or retained. Raw drafts are the storage representation.

`EditResult` includes revision-pinned diagnostics. A successful save does not imply
an executable model. Current hard failures use coarse typed service errors without
an edit index; richer authoring error detail can be added before a wire API.

## Receipts, authorization and commit

Create, Edit and Publish share `(instance, principal, operation ID)` identity and receipt
quota. Reusing a Create ID for an Edit, or vice versa, returns OperationIdConflict.
After authentication-context, current Edit permission and request-shape checks,
lookup the receipt **before** comparing the expected revision or checking quotas.

Compare the full typed request: instance/model/expected revision, ordered edits,
temporary/reference IDs, names, exact source and decimal coefficient/sign/scale.
Decimal numeric equality alone is insufficient: submitting `1.0` versus `1.00`
is different mutation intent. Same ID and intent returns the original success or
rejection without staging or advancing the revision again.

Reserve receipt storage before preparing a candidate. Validate the initial expected
revision, stage edits, prepare diagnostics and result mappings, recheck Edit
authorization, then compare the expected revision again at the commit boundary.
Prepare owned caller/receipt results before installation. Replace the resource and
append its receipt under exclusive service access with reserved capacity and no
callbacks/fallible work between them. Failure leaves graph, name directory, port
mappings and public revision unchanged; an admitted rejection receipt remains.

`operation_outcome` returns Edited, Committed (Create), Rejected, Published,
PublicationRejected or Unknown.
Known outcomes require current authorization for their recorded mutation; Unknown
requires the separate global Recover action. Publication outcomes require Publish.
The authenticated principal scopes
all outcomes. No attachment grants edit or inspect permission by itself.

All mutations are synchronous exclusive `&mut self` operations. Competing writers
observing the same head serialize, and only the first can commit. The final revision
comparison is explicit even though safe exclusive access prevents a public race
window today. There is no automatic merge, rebase, reconnect or retry. Reconcile a
lost response by outcome lookup or explicit **same-ID/same-intent** replay against
the same service instance. No crash durability or durable exactly-once claim.

## Bounds

`with_edit_limits` configures positive `EditLimits`; `new` uses defaults, reported
through `info().edit_limits`. Existing resource/attachment/receipt quotas also apply.

| Edit limit | Default |
| --- | ---: |
| Commands per batch | 128 |
| Batch accounting bytes | 65,536 |
| Live nodes per model | 1,024 |
| Live public ports per model | 256 |
| Stored formula source bytes per model | 262,144 |
| Cumulative committed node identities per model | 4,096 |
| Cumulative committed port identities per model | 1,024 |

Each command accounts for 512 bytes of bounded fixed fields plus its submitted
formula source bytes. This is conservative request/work accounting, not an exact
allocator measurement or wire encoding. Each formula is at most 4,096 UTF-8 bytes;
the existing parser's depth limit also applies. Names and temporaries have the
field bounds above. Oversized/empty/malformed requests fail before admission and
retain no receipt. A within-admission model-limit or structural failure records
its rejection. All receipts count against the shared configurable service quota;
capacity exhaustion refuses new mutations and never evicts retained outcomes.

Live-node/port caps apply during staging as well as at the end. Stored source cap
applies to final definitions. Identity caps include tombstones, even identities
created then deleted within one successful batch. Receipt and identity garbage
collection, resource deletion and persistence remain future lifecycle work.

## Deferred

The subsequent [Stage A adapter](serialized-authoring-adapter.md) adds public
authoring Protobuf in process, native staged partial input-port patches and
original wire intent in service receipts. [Stage B](daemon-authoring.md) adds local
daemon IPC authoring, now exposed through the minimal REPL. Still deferred:
C# editing, new numeric types, persistence,
workbook coordinates or GUI integration. Slice C separately implements validation,
capture, unpublished evaluation and publication; it does not strengthen the
validation required to save a draft. Immutable model hosting remains independent
of later working edits.
