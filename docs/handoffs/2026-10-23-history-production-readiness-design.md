# ADR-0023 production-readiness design — parent review packet

Date: 2026-10-23. **Design prepared; parent approval pending. No runtime correction,
container, coordinator, protocol or UI implementation performed.**

The single substantive deliverable is the
[production-readiness architecture packet](../architecture/history-production-readiness.md).
It answers the [final feasibility assignment](2026-10-23-disk-history-final-feasibility-review.md)
without reopening or reinterpreting the accepted research evidence.

The first recommendation is a validated, exhaustive current-root projection into
existing WB5 slots. Generic full-ledger transfer stays strict; original runtime IDs,
monotonic allocation authority and Redo remain unchanged. The packet compares
content/lifetime persistence alternatives and gives a concrete adversarial Save plan.
No correction is implemented by this handoff.

The remaining proposals cover atomic durable transaction indexing and bounded verified
startup, mutation-local record production, separate resource/checkpoint budgets,
conditional suffix-loss detection with an independently placed receipt-eligibility
witness, and explicit retention/reclamation/filesystem decisions. SQLite is the first
backend candidate recommended for evaluation, not a selected production dependency,
format or packaging policy. Indexed lookup and incremental construction precede UI
history integration.

Approval decisions P1–P9 and staged gates G0–G7 are in the packet. Parent review should
resolve the proposed contracts and G1's narrow scope before any implementation begins.
Numerical defaults, production container, witness trust/placement, product retention
policy and power-loss guarantees remain unapproved. No identity recycling, quota
increase, WB migration, Optimize Worksheet, JIT/SIMD or unrelated fixes.

Validation is documentation/source review only: source boundaries and transfer
invariants were inspected, links and whitespace checked, and unchanged Rust inputs
compared against the accepted Stage 4 manifest. Tests in this packet are proposed;
implementation tests were not rerun for documentation changes. All accepted artifacts
and their manifests remain untouched. The packet cites official SQLite documentation
for its candidate's WAL/synchronization constraints; their feasibility in Pivot still
requires the approved evaluation gates.
