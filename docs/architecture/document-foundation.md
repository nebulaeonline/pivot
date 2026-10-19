# Document Foundation — Slice A

Implemented bounded in-memory subset of accepted
[ADR-0009](../adr/0009-portable-documents-and-local-lifecycle.md). A standalone model
can be created, edited, validated, evaluated, exported and restored without becoming
a hosted resource. No filesystem, byte-format or IPC document workflow exists yet.

## Code boundaries

All new functionality lives in `pivot-storage`; no new crate or dependency was
needed. `pivot-core` is unchanged and still depends only on `pivot-types`.

| Implementation | Responsibility |
| --- | --- |
| [graph.rs](../../crates/pivot-storage/src/graph.rs) | Hosting-free GraphPayload, EditableGraph, core transfer mapping, shared staging/evaluation |
| [document.rs](../../crates/pivot-storage/src/document.rs) | One-graph ModelDocument, private DocumentSession, revision guards, export and saved checkpoint |
| [runtime.rs](../../crates/pivot-storage/src/runtime.rs) | Existing hosted Record adapter, publication mappings and whole-store collection checks |
| [lib.rs](../../crates/pivot-storage/src/lib.rs) | Shared graph field types and borrowed semantic validation, unchanged hosted Record shape |
| [codec.rs](../../crates/pivot-storage/src/codec.rs) | Unchanged frozen hosted-v1 CBOR encoding/decoding |

The document path never constructs a hosted Record, placeholder store/epoch,
publication entry or operation receipt. Both document and hosted adapters share
the graph runtime and core transfer boundary. Hosted collection validation and
publication policy remain outside the document session. There is no global mutable
registry or new parser/binder/evaluator.

## Owned representation and identity

GraphPayload contains graph-model UUID, portable revision UUID, display name,
numeric/expression/payload versions, allocated node/port ledgers and live definitions.
The existing Token/Node/Port types are shared; the `model` field means graph owner,
not hosted registration. Ports expose names directly without publication lineage.
Retired identities remain allocated and cannot be reused.

ModelDocument contains logical format version 1, DocumentId, DocumentStateId and
exactly one GraphPayload. Its graph revision is stored once in that payload. These
are owned Rust values with validation, **not a serialized document format** or a
promise of stable source-compatible public APIs. No SavedFileVersion or fingerprint
is allocated before there is a file lifecycle that needs one.

Every load gets a fresh SessionId and new process-local node/port handles. Copied
documents retain their logical IDs and state but have independent runtime maps and
edits. DocumentId expresses logical ancestry; it cannot authorize cross-session
commands or checkpoint acknowledgment. `load_graph` also supports a hosting-free
payload alone, assigning new document/state identity.

## Editing and calculation

DocumentSession exposes New/load, add or replace decimal inputs, add/repair formulas,
rename/remove nodes, add/remove ports, repair port targets, validate, selected-node
evaluation and named public invocation. Each operation takes an Expected token with
SessionId, DocumentStateId and GraphRevisionId. Foreign sessions and stale revisions
fail explicitly. Mutation staging uses the existing core; failed edits install
nothing. Actual no-ops retain state/revision, while representation changes such as
a different decimal scale count as content edits.

Structural invalidity remains editable and exportable. Validation returns portable
diagnostics; calculation rejects structurally invalid snapshots. A bound reference
to an allocated retired target is a draft error, while a never-allocated or foreign
reference is malformed input and rejects load. Formula text is parsed/bound only on
explicit formula submission, never on restoration or unrelated edits.

Named invocation uses existing required/default and temporary-override semantics.
It needs no publication. Results carry the exact Expected tokens and a fresh
CalculationGenerationId; arithmetic results/errors come from the core. Definitions,
saved checkpoints and dirty state are unaffected by calculation. This is a synchronous
in-process API; detached asynchronous document calculation is not implemented here.
The current foundation reports some edit/invocation failures through coarse graph
error categories; it does not add a complete client diagnostic protocol.

## Exact values and immutable exports

Portable inputs, defaults and literals retain exact coefficient/sign/scale including
signed zero. Explicit input/default edits retain the submitted Decimal96, not a
reconstructed normalized runtime value. Unrelated edits retain existing portable
definitions. Newly submitted formulas use the existing parser's literal semantics;
original source is retained. No value representation already lost by a caller before
submission can be reconstructed by this API.

`export(expected)` returns an ExportCapture owning an immutable Arc<ModelDocument>.
Later edits cannot change its content. Loading a cloned document creates a fresh
independent session. Callers may construct or modify owned payloads, but loading
always validates them before installing runtime state; they cannot forge private
capture provenance to mark a different session saved.

New and loaded sessions start dirty: loading owned data is not evidence of a
verified file save. `acknowledge_saved(capture)` checks the originating session and
records that exact captured state as the in-memory saved checkpoint. It is a trusted
caller's attestation, not file I/O or a disk-durability promise. A later file broker
must reconcile the write before using it.

Capture S, edit to T, acknowledge S: T remains dirty. Duplicate acknowledgment is
idempotent. Acknowledging an older export after a newer one was acknowledged fails
StaleCheckpoint, preventing saved-state regression. The export sequence is private
and session-local, not a retained mutation receipt or durable revision history.
Undo/checkpoint equality beyond direct no-op detection is not implemented.

## Bounds and hosted compatibility

Default graph counts and work limits remain small: 1,024 live nodes, 256 ports,
8,192 allocated identities and approximately 1 MiB logical payload budget, with
independent core transfer ceilings. Source is bounded to 4,096 bytes. Logical byte
and item accounting bounds owned payload work before import; it is not an encoded
file size or a native document codec. Full capture/staging copies are correctness
mechanisms, not a large-workbook performance claim.

The hosted Record codec and frozen 14-field format are unchanged. Its adapter keeps
exact encoded byte/item/depth budgets; document logical accounting must not reject
an otherwise valid hosted record under a tight encoded-byte limit. A dedicated
regression test exercises that distinction. All 11 frozen fixture bytes and rejection
phases remain unchanged. Hosted public-port external-name translation, immutable
publication validation and one-store collection rules remain in that adapter.

## Evidence and next boundary

The [Slice A handoff](../handoffs/2026-10-18-document-foundation-slice-a.md) records
full CI evidence and the new document tests. Filesystems, experimental containers,
safe replacement, daemon RPC, C# transfer, worksheets/ranges and incremental
calculation are not implemented. The next proposed milestone after review is the
smallest headless model-file workflow using an explicitly experimental container;
production format selection and broad container benchmarks follow separately.
