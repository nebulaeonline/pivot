# ADR-0009: Portable documents and local lifecycle

Status: Proposed
Date: 2026-10-18

## Context

Pivot is a reusable calculation engine powering spreadsheets and other clients.
Its ordinary workflow must be New/Open, Edit, Calculate, Save, Save As and Close,
without a hosted registry, publication or database server. The accepted ADR-0008
foundation preserves graph semantics, but its frozen record includes hosted store
and recovery identities. It is not the native document format.

The owner authorized this design proposal after the ADR-0008 implementation review.
The completed implementation and ADR-0006/0007/0008 history remain intact. Further
hosted persistence work is deferred. This proposal authorizes no runtime changes.

## Decision

Propose four logical boundaries; these need not become four crates or processes:

| Boundary | Owns | Depends on |
| --- | --- | --- |
| Computation core | Expressions, graph definitions, immutable captures, evaluation | Value types only |
| Document model | Portable models, workbook projections, presentation and document versions | Core and a portable graph adapter |
| Application/session host | Loaded sessions, edits, calculations, file lifecycle and client IPC | Document model and core |
| Optional hosted-model service | Registry, publication, authorization, durable receipts and provider recovery | Core and optional storage providers |

A model document works with no spreadsheet. The proposed logical envelope can
contain multiple independent computational graphs and, later, a workbook projecting
onto them. Start with one graph and no workbook for the first implementation proof.
Cell addresses and display names never replace stable graph identities. Cross-graph
execution and automatically refreshed external links remain separate designs.

Separate hosting-free graph payloads from document metadata and hosted authority.
Reuse validated core import and existing graph semantics through a future adapter;
do not alter frozen storage-v1 bytes or inject dummy StoreId/RecoveryEpochId fields
into user files. Document public interfaces require no publication operation.

Separate DocumentId, loaded SessionId, graph revision, calculation generation,
document state and saved-file version. Calculate against immutable captures while
edits continue. Save captures a consistent document state; it neither publishes
models nor persists every edit. Retain original source, stable bound references,
invalid drafts, retired identities and exact typed value representations.

For the proposed desktop mode, Rust owns document semantics and the C# application
owns user-authorized file I/O through a file broker. Exchange bounded document
content, not arbitrary daemon filesystem paths. A Rust in-process application uses
the same document adapter with its own file broker; no daemon is required.

Define dirty state, conflict detection, safe replacement and recovery as application
contracts. Do not claim universal crash durability or exclusion of uncooperative
writers. Keep ZIP, SQLite and a versioned binary container under consideration;
select only after requirements and measured prototypes have been reviewed.

## Alternatives considered

- Native documents equal frozen storage-v1 records: unnecessarily requires hosted
  identity and publication concepts.
- Source-only files: cannot preserve bindings after rename or name reuse.
- Mandatory durable daemon registry: prevents ordinary independent file use.
- Workbook-only documents or cell-based core identity: prevents general model reuse.
- Commit every edit to the user file: confuses live edits, explicit Save and recovery.
- Select SQLite or ZIP now: settles a physical choice before workload requirements.
- Reimplement evaluation in C#: creates a second calculation authority.

## Consequences

The existing foundation remains useful, but a hosting-free payload API and document
adapter still need implementation after review. The current record limits are
correctness budgets, not a large-workbook capacity promise. Future sparse storage,
range operations and incremental calculation must be designed and measured.

Optional hosted capabilities remain supported directions. ADR-0008 provider,
coordinator and cross-restart protocol work is deferred, not rejected or superseded.
The document lifecycle does not inherit hosted full-revision retention requirements.

## Verification / reconsideration trigger

Acceptance requires review of the supporting packet and explicit authorization of
a bounded implementation slice. First prove a model-only file can be opened,
evaluated, edited and saved with identical bindings and no registry or database
configuration. Verify in-process Rust and, separately, session-host use. Exercise
cross-platform replacement failures before advertising supported save guarantees.

Reconsider graph composition for cross-model formulas, identity remapping for true
fork/merge, file concurrency guarantees, container selection and large-workbook
resource profiles when those features become concrete.

Supporting proposals:

- [Ownership and lifecycle](../architecture/document-lifecycle.md)
- [Portable representation and identities](../architecture/portable-document-representation.md)
- [Local file failure matrix](../architecture/document-file-failure-matrix.md)
- [Container alternatives](../architecture/document-container-analysis.md)
- [Storage reuse plan](../architecture/document-storage-reuse.md)
- [Workbook projection](../architecture/workbook-projection-considerations.md)
- [Verification plan](../architecture/document-verification-plan.md)
- [Review handoff](../handoffs/2026-10-18-portable-document-design.md)
