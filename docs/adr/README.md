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

- [ADR-0023: Tiered recoverable document transaction history](0023-document-transaction-history.md) — feasibility accepted/closed; G0 architecture approved with clarifications; internal G1 historical native Save accepted; ordinary Save promoted and three-platform validated; ready for parent G1 closure review; G2–G7/production history gated (2026-10-24).
