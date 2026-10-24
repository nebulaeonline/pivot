# ADR-0023 production-readiness design — parent review packet

Date: 2026-10-23. **G0 direction approved with clarifications on 2026-10-24; only G1 authorized.
No runtime correction, container, coordinator, protocol or UI implementation performed.**
[Approval and exact G1 scope](2026-10-24-history-readiness-g1-authorization.md).

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

Decisions P1–P9 and gates G0–G7 remain in the packet. The subsequent review approves
the direction and authorizes G1 only, under the linked clarifications. G2–G7 still
require separate authorization; backend, witness placement and numeric defaults remain
unapproved implementation choices.
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

Implementation follow-through: [G1 internal historical native Save correction](2026-10-24-historical-native-save-g1.md).
Ordinary Save promotion and G2–G7 remain gated; the original design packet is not an implementation acceptance.

[G1 three-platform validation and parent review packet](2026-10-24-macos-historical-save-g1-validation.md)
are now ready. This completes internal-correction validation, not ordinary Save promotion or G2 authorization.

[Parent G1 implementation review](2026-10-24-historical-save-g1-internal-review.md) accepts
the internal correction and authorizes bounded ordinary Save promotion with final
three-platform managed/session/desktop/AOT gates. G1 closure and G2 authorization remain separate.

[Ordinary Save final three-platform validation](2026-10-24-macos-g1-promotion-validation.md)
now supports parent G1 closure review. G1 closure has not been inferred and G2 remains deferred.

[Parent G1 closure and G2 authorization](2026-10-24-g1-closure-g2-authorization.md)
now close ordinary Save promotion and authorize only the bounded feature-only
mutation-local producer. G3–G7 remain gated; earlier authorization notes above are
historical decisions rather than the current gate status.
