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
  — Accepted and closed by parent implementation review (2026-10-31). Three Clear modes, complete
  built-in-format reset, four visible source-edge Fill directions, actual-coordinate
  formulas, exact-row one-use review and one canonical transaction. Approved Clear
  limit 4,096 slots; Fill 256 including sources. Clear-first/Fill-second slices within
  one combined milestone. Canonical no-ops, immutable Fill plans and combined
  content/style publication are mandatory. Implementation and six-target
  headless/managed/AOT validation accepted, including source/resource qualifications
  and the batch reservation correction. No additional validation or AOT rerun required.
  [Implementation handoff](../handoffs/2026-10-31-visible-only-clear-and-fill.md).
  [Design handoff](../handoffs/2026-10-31-visible-only-clear-and-fill-design.md).

ADR-0036: Exact, Conservation-Preserving Allocation is assigned as the next design
milestone, using the [ALLOCATE proposal](../proposals/allocate.md). Required precision,
exact representability and conservation for a complete consistent recipient set are
mandatory. The owner subsequently authorized design; the proposed ADR and handoff
follow below. ADR-0023 remains parked at G3.

- [ADR-0036: Exact, conservation-preserving allocation](0036-exact-conservation-preserving-allocation.md)
  — Accepted and closed by parent implementation review (2026-10-31).
  Four strict Decimal arguments, explicit precision, u32 indices, deterministic
  residuals, fixed 192-bit arithmetic and group-wide exact-SUM admission. Opcode 19;
  WB12/private v13/hosted v1/CB5/CB6 and budgets unchanged. Local CI 13/13, 504
  all-feature Rust tests, strict Clippy, and all six headless targets 13/13 with
  51,523 parity vectors each; 1,116 supplementary boundary vectors passed locally.
  macOS x64 is Rosetta. Approved scope needed no new GUI/AOT run. Verified archival,
  pruning and diagnostic cloc are complete. See the
  [implementation handoff](../handoffs/2026-10-31-exact-allocation.md).

- [ADR-0037: Weighted, conservation-preserving allocation](0037-weighted-conservation-preserving-allocation.md)
  — Accepted and closed by implementation review (2026-11-01).
  Four design clarifications implemented; strict operands, exact ranked settlement,
  explicit ordinal association, group-wide final-share / nonzero-subset SUM admission,
  fully metered/cancellable execution and consistent arity. Opcode 20; existing formats
  and global limits unchanged. Final local CI 13/13, 515 all-feature Rust tests, strict
  Clippy and all six selected headless targets 13/13 with 12,841 weighted vectors each.
  Linux ARM64 retry and the late test-only fixture correction are qualified explicitly;
  macOS x64 is Rosetta. Diagnostic LoC and verified cleanup are complete.
  [Implementation review](../handoffs/2026-11-01-weighted-allocation.md).

- [ADR-0038: Range-based weighted allocation](0038-range-based-weighted-allocation.md)
  — Accepted and closed with Windows ARM64 cold-build qualification (2026-11-01).
  Dense 1..64-position
  literal ranges, strict Blank/types, spatial membership and cycle-safe geometry
  restoration use operation 21 and tags 8/9 with WB13-only persistence. Private v13,
  hosted v1 and CB5/CB6 retain their fields. ROBPC CI 13/13, 525 all-feature tests,
  strict Clippy, six-target headless evidence and all 12 managed/AOT GUI suites pass.
  Windows ARM64 headless is a retained-cache pass after three preserved cold-build
  structural timing failures; their cause remains unresolved. Source qualification,
  observed resource maxima, diagnostic LoC and verified cleanup are documented.
  [Implementation review](../handoffs/2026-11-01-range-based-weighted-allocation.md).

- [ADR-0039: Named Windows, data binding and document composition](0039-named-windows-and-document-composition.md)
  — Architecture accepted with three clarifications (2026-11-01): concrete geometry
  usability qualification, explanatory lossless Text-conversion refusals, and declared
  InputId authority for every invocation route. Independent Window scopes,
  explicit ownership/producer generations, source-authoritative references, section
  geometry, bounded Text, immutable document capture and headless layout/PDF boundary.
  This ADR records the architecture; its unbound Grid and producer-binding scopes
  are implemented and accepted under ADR-0040 and ADR-0041. Later scopes remain
  unimplemented. See the [design handoff](../handoffs/2026-11-01-named-windows-design.md).

- [ADR-0040: Unbound Grid Windows, identity and references](0040-unbound-grid-windows.md)
  — Accepted and closed with unresolved Windows ARM64 cold-run qualification
  (2026-11-01). Client-only selection,
  single-generation scope capture and aggregate axis/identity admission. Tagged Grid
  ownership, sparse canonical graph integration, lifecycle/reference faults,
  WB14/private v14 cuts and clipboard refusals. All six targets passed managed/AOT
  GUI qualification. Windows ARM64 retains its original unresolved cold timing failure
  alongside a separate full retained-cache headless pass. See the
  [implementation handoff](../handoffs/2026-11-01-unbound-grid-windows.md).

- [ADR-0041: Worksheet-Range Producers and Bound Grid Windows](0041-worksheet-range-producers-and-bound-grid-windows.md)
  — Accepted and closed with unresolved Windows ARM64 cold-run qualification
  (2026-11-02). Live read-only worksheet projections, guarded scalar/spatial source
  authority, immutable fixture generations, explicit refresh, shared captured Decimal
  leaves and aggregate ownership/admission. WB15-only native persistence and private
  v15 are accepted; hosted v1 and CB5/CB6 are unchanged. All six targets passed
  managed/AOT GUI suites. Windows ARM64 retains the original cold timing failure
  alongside a separate full retained-cache headless pass. Recovered wrapper/DNS
  failures and their complete passing retries remain distinct. See the
  [implementation handoff](../handoffs/2026-11-02-bound-grid-windows.md).

- [ADR-0042: Section Breaks and Local Grid Geometry](0042-section-breaks-and-local-grid-geometry.md)
  — Design accepted by final review (2026-11-02), including all three required
  clarifications. Implemented, accepted and closed by parent implementation review
  (2026-11-03), with existing qualifications retained; no further validation required.
  Independent width/height partitions,
  either-boundary whole-section selection, one bounded mapping, complete overlap
  validation, authored-axis transforms, positional bound-view presentation and
  coherent filtered viewports. Concrete invoice/one-boundary selection examples;
  ordinary-layout usability is a promotion gate. WB16/private v16 are implemented.
  Six-target managed/AOT GUI evidence and full/focused headless source lineage are
  recorded in the [implementation handoff](../handoffs/2026-11-03-section-grid-geometry.md).
  Original cold timing qualifications remain unresolved. See the
  [design handoff](../handoffs/2026-11-02-section-grid-geometry-design.md).

- [ADR-0043: Section-Local Headings and Axis Selection](0043-section-local-headings-and-axis-selection.md)
  — Implemented, accepted and closed by parent implementation review (2026-11-03),
  with Windows x64 cold sparse-sort and historical Windows ARM64 cold structural
  timing qualifications retained.
  All six targets passed final managed/AOT GUI suites, including rendered light/dark
  contrast checks. See the [implementation handoff](../handoffs/2026-11-03-section-local-headings.md).
  Interior section row/column headings, exact local-axis selection, canonical
  either-boundary closure and one master Show interior headings toggle. View chrome
  does not change document geometry or selection authority. Private session v17;
  WB16, hosted v1 and CB5/CB6 unchanged. No further validation required. See the
  [design handoff](../handoffs/2026-11-03-section-local-headings-design.md) and
  [interaction contract](../architecture/section-local-headings.md).
