# Architecture decision records

Use `NNNN-short-title.md` and refer to each record as `ADR-NNNN`.
Allocate the next unused four-digit number; never reuse IDs. Each record
contains Status, Date, Context, Decision, Alternatives considered,
Consequences, and Verification / reconsideration trigger.

Statuses are Proposed, Accepted, and Superseded. Preserve old records when
superseding them and link to the replacement. Acceptance records owner
decisions, not implementation suggestions.

- [ADR-0001: Graph-first computation core](0001-graph-first-core.md) — Accepted
- [ADR-0002: Editable models with immutable calculation snapshots](0002-editable-model-calculation-snapshots.md) — Accepted
- [ADR-0003: Stable references and public interfaces](0003-stable-references-and-public-interfaces.md) — Accepted
- [ADR-0004: Portable Protobuf protocol](0004-portable-protobuf-protocol.md) — Accepted
- [ADR-0005: Local IPC and endpoint security](0005-local-ipc-and-endpoint-security.md) — Accepted
- [ADR-0006: Working-model lifecycle and transactions](0006-working-model-lifecycle-and-transactions.md) — Accepted
- [ADR-0007: Versioned working-model authoring protocol](0007-versioned-authoring-protocol.md) — Accepted
- [ADR-0008: Transactional persistence and recovery](0008-transactional-persistence-and-recovery.md) — Accepted

- [ADR-0009: Portable documents and local lifecycle](0009-portable-documents-and-local-lifecycle.md) — Accepted
- [ADR-0010: Workbook projection and spreadsheet semantics](0010-workbook-projection-and-spreadsheet-semantics.md) — Accepted
- [ADR-0011: Standalone persistent default and shared runtime lifecycle](0011-standalone-persistent-default.md) — Accepted.
- [ADR-0012: Atomic bulk mutation and opt-in persistent workbook integration](0012-bulk-mutation-and-persistent-workbook.md) — Accepted.

Authoring protocol, daemon integration and the minimal REPL are implemented.
ADR-0008 remains Accepted for optional durable hosted storage; further provider,
SQLite feasibility, coordinator and cross-restart protocol work is deferred. The
portable codec/import foundation is accepted and implemented; no database provider
or durable service is present. Accepted ADR-0009 defines the document-focused
direction. Slice A implements hosting-free payloads/in-memory sessions; Slice B
adds an experimental headless file envelope and broker. Production container
selection, document RPC and workbook/UI integration remain separate. Ordinary local
documents must not require hosted storage. Final numeric policy, broader scheduling
and multi-instance coordination remain separate work.

Slice B's experimental implementation is accepted. ADR-0010 accepts the structural
workbook architecture while reserving spreadsheet compatibility semantics. Its
bounded Slice A now implements one sparse worksheet/graph proof with typed references.
Scalar A1 parsing and stable binding are implemented. Persistent workbook integration
is opt-in under accepted ADR-0012; production workbook selection stays reference.
Ranges, GUI, new protocol and production workbook files remain future work.

- [ADR-0013: One canonical persistent graph and workbook](0013-canonical-engine-consolidation.md) — Accepted.

- [ADR-0014: Owned local application session and bounded worksheet](0014-application-session-and-worksheet.md) — Accepted (Slice A; unresolved WSLg startup defect retained).

- [ADR-0015: Native workbook documents and local application lifecycle](0015-native-workbook-documents.md) — Accepted (Slice B; validation limitations retained).

- [ADR-0016: First-class rectangular selections and SUM](0016-first-class-ranges-and-sum.md) — Accepted; Slice C implementation accepted (2026-10-20).

- [ADR-0017: Bounded rectangular clipboard and formula copy](0017-rectangular-clipboard-and-formula-copy.md) — Accepted with compatibility revision; WB3/clipboard implemented; implementation accepted; three-platform automated validation complete; workstream closed (2026-10-20).

- [ADR-0018: Document-owned worksheet text](0018-document-owned-worksheet-text.md) — Accepted with clarifications; implementation and escape-overhead clarification accepted; three-platform validation complete; workstream closed (2026-10-21).

- [ADR-0019: Structured results and projection identities](0019-structured-results-and-projection-identities.md) — Design approved with clarifications; bounded experimental implementation accepted and milestone closed (2026-10-22).

- [ADR-0020: Worksheet bindings to structured numeric outputs](0020-worksheet-structured-output-bindings.md) — Architecture/contracts and bounded experimental implementation accepted; three-platform validation accepted; milestone closed (2026-10-22).

- [ADR-0021: Bounded spreadsheet aggregate functions](0021-spreadsheet-aggregate-functions.md) — Design and WB5-only corrected implementation accepted; three-platform validation complete; workstream closed (2026-10-23).

- [ADR-0022: Four-direction rectangular Fill](0022-four-direction-rectangular-fill.md) — Design and three-platform-validated implementation accepted; workstream closed (2026-10-23).

- [ADR-0023: Tiered recoverable document transaction history](0023-document-transaction-history.md) — G1/G2 accepted and closed; G3 parked by owner/parent direction (2026-10-28), with original 32K failure preserved. G4–G7 remain gated.


- [ADR-0024: Multiple worksheets in one workbook](0024-multiple-worksheets.md) — Accepted; implementation and three-platform validation accepted by parent review, milestone closed (2026-10-28).

- [ADR-0025: Typed spreadsheet values](0025-typed-spreadsheet-values.md) — Accepted; implementation and source-qualified evidence accepted by parent review, milestone closed (2026-10-28).

- [ADR-0026: Basic cell formatting](0026-basic-cell-formatting.md) — Accepted; implementation and explicitly qualified macOS AOT evidence accepted by parent review, milestone closed (2026-10-28).

- [ADR-0027: Calculation rounding functions](0027-calculation-rounding-functions.md) — Accepted; implementation and native three-platform evidence accepted by parent review, milestone closed (2026-10-29).

- [ADR-0028: CSV import and export](0028-csv-import-export.md) — Accepted; implementation, file-installation clarification and native three-platform headless/managed desktop evidence accepted by parent review; milestone closed (2026-10-29).

- [ADR-0029: Production workbook capacity and identity scalability](0029-production-workbook-capacity.md) — Accepted and closed by parent implementation review (2026-10-29), including source-qualified native evidence and separately reported identity-memory cost.

- [ADR-0030: Row, column and rectangular-range formatting](0030-region-formatting.md) — Accepted and closed by parent implementation review (2026-10-29), including the normalization correction, native validation and separately qualified benchmark-only extension. WB10 only, private v8 and unchanged CB4.

- [ADR-0031: Insert and delete worksheet rows and columns](0031-row-column-structure.md) — Accepted and closed by parent implementation review (2026-10-30), including the clipboard correction and source-qualified six-target headless/managed/AOT evidence. macOS x64 is Rosetta; structural timings are single-observation maxima. WB11 only, private v9, CB5. No further validation required. See the [implementation handoff](../handoffs/2026-10-30-row-column-structure.md).

- [ADR-0032: Worksheet sorting](0032-worksheet-sorting.md) — Design approved with
  clarifications by parent review (2026-10-30). Atomic record
  permutation, stable identities, fixed rectangular ranges, snapshot typed keys and
  explicit formatting movement. Mandatory sortable-error/operational-failure distinction,
  identity-aware no-ops and visible confirmed range expansion. WB11/CB5 unchanged,
  private v10 implemented. All six targets passed headless CI and attended managed/AOT
  acceptance; macOS x64 uses Rosetta. Final local CI, 471 all-feature Rust tests and
  strict Clippy passed. Original failures are retained and batch cleanup is complete.
  Accepted and closed by parent implementation review (2026-10-31), including the
  sparse-index and range-submission corrections and source qualification. No further
  validation required. See the
  [implementation handoff](../handoffs/2026-10-31-worksheet-sorting.md) and
  [semantics](../architecture/worksheet-sorting.md).
  [Design handoff](../handoffs/2026-10-30-worksheet-sorting-design.md).

- [ADR-0033: Worksheet filtering](0033-worksheet-filtering.md) — Accepted and closed
  by parent implementation review (2026-10-31). Persist
  typed definitions, derive row visibility without moving identities, explicitly
  confirm hidden-row bulk actions, and review filters after structural edits.
  Require transient operational suspension without document mutation, exact complete-
  footprint confirmations and coherent visibility with bulk actions blocked while
  pending/failed. WB12/private v11 and bounded metadata admission are approved,
  with implementation and six-target headless/managed/AOT validation complete.
  Viewport corrections, source-qualified validation, archives, cleanup and the LoC
  baseline accepted. No further validation or AOT rerun required.
  [Implementation handoff](../handoffs/2026-10-31-worksheet-filtering.md).
  [Design handoff](../handoffs/2026-10-31-worksheet-filtering-design.md).

- [ADR-0034: Visible-only operations on filtered data](0034-visible-only-operations.md)
  — Accepted and closed by parent implementation review (2026-10-31). Independent visible-source Copy
  and visible-destination Paste; exact row-count matching, per-cell geometric formula
  translation, bounded CB6 source-row origins and one-use destination review.
  Ordinary CB5/full-rectangle operations stay intact; approve private v12 with WB12
  and hosted v1 unchanged. Visible-only Clear/Fill are deferred. Implementation and
  six-target headless/managed/AOT validation accepted.
  [Design handoff](../handoffs/2026-10-31-visible-only-operations-design.md).

ADR-0034 implementation is accepted. Native precedence/no fallback, hidden-row direct-write
integrity and review of the precise destination plan are mandatory invariants.
The owner authorized implementation, now complete. See the
[implementation handoff](../handoffs/2026-10-31-visible-only-operations.md). No further
validation is required. ADR-0035: Visible-Only Clear and Fill is assigned as the next
design milestone; the owner subsequently authorized its design (see below).
ADR-0023 remains parked at G3.

- [ADR-0035: Visible-only Clear and Fill](0035-visible-only-clear-and-fill.md)
  — Design approved with clarifications by parent review (2026-10-31). Three Clear modes, complete
  built-in-format reset, four visible source-edge Fill directions, actual-coordinate
  formulas, exact-row one-use review and one canonical transaction. Approved Clear
  limit 4,096 slots; Fill 256 including sources. Clear-first/Fill-second slices within
  one combined milestone. Canonical no-ops, immutable Fill plans and combined
  content/style publication are mandatory. Implementation awaits separate owner
  authorization; no feature work or new native validation.
  [Design handoff](../handoffs/2026-10-31-visible-only-clear-and-fill-design.md).
