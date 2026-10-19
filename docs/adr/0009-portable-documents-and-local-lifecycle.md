# ADR-0009: Portable documents and local lifecycle

Status: Accepted
Date: 2026-10-18

## Context

Pivot is a reusable calculation engine powering spreadsheets and other clients.
Its ordinary workflow must be New/Open, Edit, Calculate, Save, Save As and Close,
without a hosted registry, publication or database server. The accepted ADR-0008
foundation preserves graph semantics, but its frozen record includes hosted store
and recovery identities. It is not the native document format.

The owner authorized this design proposal after the ADR-0008 implementation review.
The completed implementation and ADR-0006/0007/0008 history remain intact. Further
hosted persistence work is deferred. Following the document review, the owner
authorized Document Foundation Slice A: hosting-free graph payloads and one-graph
in-memory sessions. Filesystems, document containers, IPC, worksheets and UI remain
outside that initial implementation scope. After Slice A acceptance, the owner
authorized Slice B's experimental headless file workflow; production format
selection, IPC, worksheets and UI remain separate.

## Decision

Use four logical boundaries; these need not become four crates or processes:

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
Reuse validated core import, staging, binding and evaluation through a thin adapter;
do not alter frozen storage-v1 bytes or inject dummy StoreId/RecoveryEpochId fields
into user files. Document public interfaces require no publication operation.

Separate DocumentId, loaded SessionId, graph revision, calculation generation,
document state and saved-file version. Introduce each only when it enforces an
observable invariant; no general identity-management framework is required.
DocumentId records logical ancestry, not a uniquely opened file. Sessions and file
fingerprints distinguish copied files. Calculate against immutable captures while
edits continue. Save captures a consistent document state; it neither publishes
models nor persists every edit. Retain original source, stable bound references,
invalid drafts, retired identities and exact typed value representations.

For the proposed desktop mode, Rust owns document semantics and the C# application
owns user-authorized file I/O through a file broker. Exchange bounded document
content, not arbitrary daemon filesystem paths. Do not raise the existing 1 MiB
frame limit as a substitute for a separately reviewed bounded bulk-transfer design.
A Rust in-process application uses
the same document adapter with its own file broker; no daemon is required.

Define dirty state, conflict detection, safe replacement and recovery as application
contracts. Do not claim universal crash durability or exclusion of uncooperative
writers. Keep ZIP, SQLite and a versioned binary container under consideration;
select a production format only after requirements and measured prototypes have been
reviewed. First prove the in-memory document, then the smallest experimental
headless file workflow; broad million-cell container benchmarking is not a
prerequisite for that experimental proof.

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

The hosting-free payload and in-memory document adapter are implemented as Slice A
inside pivot-storage, without a new crate or dependency. They delegate graph
semantics to the core rather than reproducing hosted attachments, receipts,
publication or authorization. Slice B now implements a provisional envelope and
bounded Rust file broker, with verified installation and explicit uncertainty.
Production format/container selection and C# broker integration remain designs.
See the [Slice A implementation](../architecture/document-foundation.md). The current
record limits are
correctness budgets, not a large-workbook capacity promise. Future sparse storage,
range operations and incremental calculation must be designed and measured.

Optional hosted capabilities remain supported directions. ADR-0008 provider,
coordinator and cross-restart protocol work is deferred, not rejected or superseded.
The document lifecycle does not inherit hosted full-revision retention requirements.

## Verification / reconsideration trigger

Accepted after incorporating the parent review refinements and owner authorization
of Slice A. The first proof covers in-memory creation, editing, evaluation,
export/restoration, isolation and saved checkpoints without hosted management.
Slice B now proves experimental model-only files can be opened, evaluated, edited
and saved across processes with identical bindings and no registry/database
configuration. Its tested file profile and limitations are explicit; this is not a
production document-format acceptance. Verify in-process Rust and, separately, session-host use. Exercise
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

Latest implementation review packet: [Document Foundation Slice B](../handoffs/2026-10-19-document-foundation-slice-b.md).
