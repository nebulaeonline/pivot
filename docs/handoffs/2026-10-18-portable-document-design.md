# Portable documents and local lifecycle — design review handoff

Date: October 18, 2026.
Disposition: Proposed ADR-0009; design only; stop for review.

## Problem and proposed direction

Pivot needs an ordinary model/document lifecycle before further durable hosting.
A computational model should be saved in a user-controlled portable file, reopened,
edited and calculated with the same stable bindings, without a spreadsheet,
database-backed registry, publication, receipts or recovery epoch.

The ADR-0008 implementation review accepts the completed serialization/restoration
foundation. Preserve its code, tests, frozen bytes and acceptance history. ADR-0008
is now explicitly scoped to optional durable hosted storage. SQLite feasibility,
schemas/migrations, provider commits, durable daemon integration, cross-restart
receipts and PostgreSQL/MySQL adapters are deferred. Historical handoffs remain
historical; their former next-step suggestions do not override this direction.

## Review packet

| Deliverable | File |
| --- | --- |
| Proposed decision | [ADR-0009](../adr/0009-portable-documents-and-local-lifecycle.md) |
| Ownership, sessions and lifecycle | [Document lifecycle](../architecture/document-lifecycle.md) |
| Logical payload and identity boundaries | [Portable representation](../architecture/portable-document-representation.md) |
| File operations and recovery outcomes | [Failure matrix](../architecture/document-file-failure-matrix.md) |
| ZIP, SQLite and binary comparison | [Container analysis](../architecture/document-container-analysis.md) |
| Existing foundation reuse | [Storage reuse plan](../architecture/document-storage-reuse.md) |
| Spreadsheet implications | [Workbook projection](../architecture/workbook-projection-considerations.md) |
| Verification and compatibility | [Test plan](../architecture/document-verification-plan.md) |

## Principal proposals to review

- Four logical boundaries: core, document model, application/session host, optional
  hosted-model service. No mandated new crates or processes.
- First prove a one-graph model document without worksheets. Allow a later envelope
  containing independent graphs and workbook projections; no implicit cross-graph
  execution or external fetching.
- Keep the portable graph payload free of hosted authority; reuse the core import
  bridge and factor shared validation behind document/hosted adapters. Frozen
  hosted-v1 records remain unchanged and are not native document bytes.
- Preserve exact typed representations alongside runtime state. Restore existing
  bound tokens and invalid drafts without rebinding names.
- Distinguish DocumentId, SessionId, document state, graph revision, calculation
  generation, saved-file version and physical-file fingerprint.
- Save As preserves logical IDs; independent copied files have isolated sessions.
  True Duplicate/Fork needs explicit remapping later.
- In desktop mode, C# owns user-authorized file I/O while Rust owns document
  semantics. Exchange bounded content under a client-owned session, not arbitrary
  daemon paths. Direct in-process Rust use remains supported by the design.
- Save captures S while edits may continue to T. Acknowledging S cannot mark T
  clean. Publication and autosave are separate from explicit Save.
- Cooperative writer reservations and strong baseline fingerprints detect ordinary
  conflicts. They cannot promise exclusion of arbitrary uncooperative writers.
- Select no physical format yet. Compare ZIP, SQLite and binary containers against
  common workloads and platform file-safety evidence after separate authorization.

## Required scope coverage

| Review requirement | Where addressed |
| --- | --- |
| Model file without spreadsheet | ADR and representation; first acceptance scenario |
| Embedded/referenced models without cell identity | Representation and workbook projection |
| Document/session/graph/calculation/saved identities | Representation identity table and lifecycle |
| Reuse without store/epoch/receipt requirement | Storage reuse plan and ADR-0008 scope clarification |
| New/Open/Save/Save As/Close/Revert | Lifecycle operation table |
| Dirty state and document-only changes | Lifecycle dirty-state section |
| Safe write, replace and failure recovery | File sequence, matrix and platform proof obligations |
| External changes and multiple applications | Lifecycle conflict policy and file matrix |
| Source, bindings, exact values, invalid drafts | Representation and reuse plan |
| Versions, features and migrations | Representation compatibility policy and test plan |
| Container alternatives without selection | Container analysis |
| Explicit saves versus recovery | Lifecycle recovery policy |
| C#/Rust filesystem security boundary | Lifecycle ownership and security sections |
| Calculation without registry | ADR, reuse plan and acceptance tests |
| In-process Rust use | ADR, lifecycle and verification |
| Sparse projection, ranges and incremental calculation | Workbook considerations and container workload plan |

## Decisions intentionally left for review or later slices

The document byte layout, container, schema, compression, optional-feature registry,
large-workbook budgets and bulk IPC contract are not selected. No new wire messages
or public API signatures are allocated. Cross-graph execution, external-link refresh,
range grammar, identity remapping for forks, incremental storage/calculation and
collaborative editing remain separate designs.

File adapters must establish their actual OS/filesystem/locking/flush guarantees;
this packet does not prove crash or power-loss durability. The container comparison
includes measurement criteria, not completed benchmarks. Current transfer budgets
remain much smaller than spreadsheet targets; changing an outer limit alone would
not establish capacity.

## Validation and limits

This slice changes documentation only. Checks cover local Markdown links in the
review packet and updated architecture files, UTF-8, balanced code fences and diff
whitespace. No Rust/.NET suite was rerun for prose changes. The previous implementation
handoff reports 153 passing Rust tests plus Windows CI and Linux WSL checks; those
results are historical foundation evidence, not document-workflow verification.
Native macOS verification of recent implementation remains pending.

## Separate backlog and stop boundary

After review, separately authorize an implementation slice. Proposed sequencing is
shared graph adapter/session proof, container comparison and selection, then model-only
file lifecycle. This is not authorization to start any of them automatically.

Keep separate backlog items for optional hosted persistence, workbook/range APIs,
bulk viewport transport, incremental calculation, executable C# IPC and Native AOT,
native macOS validation, and the Avalonia virtualized spreadsheet control.

No source, dependencies, storage fixtures, protocol, daemon, REPL, workbook or UI
changes were made in this design slice. Stop here for architecture review.
