# Portable document representation and identities

Status: Proposed ADR-0009 logical representation. No byte layout, extension
registration, document codec or source-compatible API is established here.

## Logical envelope

Propose an envelope with document kind (model or workbook), DocumentId,
SavedFileVersion, format/feature manifest, metadata and a collection of graph
payloads. A model document initially has one graph and no worksheet. A future
workbook may contain multiple independent graphs, worksheets, projections and
resources. The manifest identifies mandatory and optional features and versions
for graph expressions, numeric semantics and presentation sections independently.

Graph payloads contain GraphId, graph revision, display name, complete allocated
node/port identity ledgers, live definitions and public interface mappings. Bound
references contain owner GraphId plus stable NodeId, never an address or runtime
handle. Names are lookup/display metadata; only explicitly submitted formula edits
invoke binding. An interface is usable directly without publication lineage,
published snapshot IDs or a service registry.

Preserve original source bytes and definition state: bound tokens, parse-failed
source/offset/code, or unresolved-name failure. Restore invalid but well-formed
drafts, including missing retired targets and cycles, as editable documents with
diagnostics. Reject malformed references to never-allocated identities, unknown
operators and unsupported numeric semantics. Structural calculation failure is
distinct from file corruption. Do not repair either by silently rebinding source.

Keep exact typed inputs, defaults and bound literals independent of runtime
normalization. Decimal96 retains coefficient/sign/scale, signed zero and trailing
scale; formatting is separate presentation metadata. New numeric types need explicit
tags, semantic versions and conversion policy, rather than coercion into Decimal.
IEEE-754 remains future work; no float support is claimed by this design.

Omit StoreId, RecoveryEpochId, operation receipts, service ownership, attachments,
authorization policy and mandatory publication metadata. Do not save dependency
indexes, process handles, cursors or runtime memory layouts. Saved results, if later
supported, are optional caches with complete calculation provenance; never silently
treat them as current authoritative values.

## Identity lifetimes

| Concept | Lifetime and comparison |
| --- | --- |
| DocumentId | Logical document identity across Save, Save As and copied files; not a path, lock or globally unique physical instance |
| SessionId | Fresh per loaded session/revert; scopes commands, runtime maps and result delivery |
| GraphId | Stable graph identity inside its owning document; document/session context disambiguates copied files |
| Portable node/port IDs | Stable within graph ownership, including retired allocations; never derived from names or reused |
| DocumentStateId | Current in-memory content checkpoint; changes on graph or document-only edit; not necessarily persisted as edit history |
| GraphRevisionId | Changes only with computational definitions/inputs/interface or semantic settings; multi-graph captures carry a revision vector |
| CalculationGenerationId | Fresh evaluation identity bound to session, graph revision vector, settings and temporary overrides |
| SavedFileVersion | Fresh opaque version for a successfully written content state; preserved when reopening that file; not a monotonic edit counter |
| File fingerprint | Broker-owned observation of physical target identity and bytes, including external changes; separate from embedded IDs |

Do not order UUIDs to infer recency. Two Save As copies may later diverge while
retaining logical identities; they get different saved versions and sessions.
Opening two copies must not alias their live runtime state. Importing a graph into
the same document requires explicit collision detection and coordinated ID remapping;
silently merging duplicate graph/node IDs is forbidden.

Presentation-only edits advance document state but not graph revision. Temporary
scenario inputs identify a calculation generation without modifying saved inputs.
Results from an older graph can be displayed as explicitly stale but cannot replace
the current generation untagged. Current core revision handles remain runtime-local;
the adapter maps them to portable graph revisions.

## Composition and compatibility

Propose multiple independently restorable graph sections, not a requirement that
each sheet is its own graph or that a workbook is one enormous graph. Initially
formulas remain graph-local. Embedded reusable model definitions can be instantiated
with separate identities. Cross-graph invocation nodes, dependency scheduling and
cycles require a later execution design; this proposal does not imply they work.

External model references should pin logical identity, version and content digest,
with an optional embedded exact copy for offline use. A path is a location hint,
not graph identity. Missing references are reported explicitly, never automatically
replaced by another model at the same path or fetched over the network. Initial
model-only files are self-contained; live external links are deferred.

Check manifest and budgets before runtime installation. Unsupported mandatory
features reject normal editable open. Optional opaque sections may be retained
only if the format explicitly guarantees they are independent of edits. Otherwise
offer a separately designed read-only inspection mode or reject; never silently
drop unsupported content on Save. No partial authoritative document is exposed.

Migrations run on private content with explicit old-version readers and fixtures.
Preserve the original file until an explicit successful Save; report upgrade needs
and require Save As when an older format cannot represent the new state. No silent
downgrade, loss of exact values or rebinding. Freeze document bytes only after a
physical format is accepted. Frozen hosted storage-v1 records remain unchanged.

Future chunked/lazy opening may validate sections on demand only with a separately
specified integrity and calculation-admission contract. This proposal initially
requires complete validation before editable use; the current small storage limits
must not be described as spreadsheet capacity.
