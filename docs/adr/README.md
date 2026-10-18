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

Authoring protocol review, final numeric policy, persistence and broader resource
scheduling remain open. Persistence will use a common transactional provider
contract: SQLite first, PostgreSQL/MySQL adapters planned. Service mode and provider
choice are independent; schemas, recovery and multi-instance coordination require
separate design.
