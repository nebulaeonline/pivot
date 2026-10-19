# Reusing portable storage for documents

Status: Accepted ADR-0009 reuse direction. Slice A now factors shared graph validation,
restoration and editing behind hosted and document adapters inside pivot-storage.
See [actual implementation and limits](document-foundation.md). Remaining file and
workbook integration described below is future work.

## What exists

[The storage foundation](portable-storage-foundation.md) owns exact Decimal96 data,
portable node/port definitions, bound tokens and failure states, identity ledgers,
bounded deterministic CBOR, validated restoration and frozen compatibility fixtures.
The doc-hidden `pivot-core::model::transfer` bridge validates temporary slot images
and allocates fresh runtime handles. It restores bindings without reparsing source.

The hosted `Record`, codec and `restore_all` are **not** hosting-free document APIs:
they require store/epoch metadata and enforce collection/publication invariants.
`RestoredModel` edit helpers are a bounded foundation test surface, not an existing
document session manager. No native file workflow is implemented.

## Shared adapter boundary

Slice A factors a hosting-free owned GraphPayload and shared validation/mapping
logic behind two adapters:

```text
Document envelope -> graph adapter -> validated core transfer -> WorkingModel
Hosted v1 Record -> hosted adapter -> shared graph validation -> core transfer
```

The diagram is logical: no crate split or public API freeze is proposed. Keep the
existing hosted Record shape, codec behavior, discriminants, validation phases and
frozen fixture bytes unchanged. Its adapter retains store consistency, publication
lineage and collection rules. The owned model-document envelope handles logical
versions and interfaces without
calling a publication registry or inventing placeholder store IDs. It is not yet
a byte-format envelope.

Reuse graph field semantics and, where useful, codec primitives; do not assume the
entire 14-field CBOR record can serve as the document payload. New document bytes
need their own version and fixtures after container selection. A storage export/import
tool may explicitly convert records to documents, but database metadata must not
become a hidden requirement for opening the resulting file.

Share validation for exact values, ledger ownership, token arity, valid operators,
source bounds and failure states. Preserve separate validation policies: editable
drafts can contain structural errors; an immutable hosted publication must pass
global snapshot validation. Graph integrity and interface mapping are reusable;
store identity, receipt authority and complete hosted catalog rules are not.

## Exact values and edits

Keep the portable owned payload alongside its runtime projection as the source of
exact representation. Delegate edits to existing core staging/binding operations,
then update affected portable fields atomically. Unchanged inputs/defaults/literals
retain exact bytes. Explicit replacement values retain submitted representation
after validation; new arithmetic outputs use existing runtime numeric semantics.
Do not recapture all exact representations from normalized runtime decimals on Save.

Persist bound tokens and original source together. Renaming a node, deleting it or
reusing its old name must not retarget existing tokens. Repairing a formula is an
explicit edit that invokes the existing parser/binder. Restoring an unresolved
draft must not turn it into a binding merely because the name now exists.

## Limits and integration gates

Current defaults include 1,024 live nodes, 256 ports and a 1 MiB record budget; core
transfer has additional hard ceilings. Raising an outer decoder budget does not
remove those inner limits. A document implementation needs explicit coordinated
profiles and must fail predictably before allocation. No spreadsheet-sized capacity
is implied by changing a constant. Measure import, full staging/copying, export and
evaluation before choosing structural sharing or chunking.

Do not use `restore_all`'s one-store rule as a document composition rule. Validate
all authoritative document sections before exposing a normal editable session.
Across independently restored models, compare portable IDs rather than new core
PortIds/NodeIds. Keep document/session namespace around copied identities. Public
interfaces can use the existing snapshot invocation machinery without ModelHost
registration; Slice A now provides that direct in-process session façade.

Slice A verifies legacy frozen bytes, malformed-data rejection and existing daemon
behavior remain compatible. Hosted restore retains its exact CBOR resource profile,
separate from the document payload's logical accounting budgets. `pivot-core` still depends on
neither documents nor storage, filesystem, IPC, authorization or SQLite. The
Slice A introduced a graph/session Rust API with no new dependency or filesystem
operation. Slice B now reuses shared codec primitives for an experimental document
envelope and adds a Rust broker with SHA-256 and platform filesystem dependencies
inside pivot-storage. It adds no provider, core dependency, hosted record change
or protocol operation. See [broker details](experimental-file-broker.md).
