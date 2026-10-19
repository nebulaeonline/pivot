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
