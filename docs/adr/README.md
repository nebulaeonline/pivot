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

Numeric policy, protocol, persistence, and transaction semantics remain open.
