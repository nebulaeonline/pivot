# ADR-0023 tiered history architecture approved — experiment authorized

Date: 2026-10-23. **Architectural direction approved with three safeguards.
Bounded disk-history feasibility authorized; production Undo/Redo not authorized.**
[Approved ADR](../adr/0023-document-transaction-history.md),
[authorized experiment](../architecture/disk-history-feasibility-plan.md),
[design packet](2026-10-23-undo-redo-design.md).
[Subsequent logical/runtime identity clarification](2026-10-23-cold-history-identity-review.md)
resolves the Stage 1 identity question without changing this experiment boundary.
The earlier [hot-root feasibility acceptance](2026-10-23-undo-redo-feasibility-review.md)
and its frozen evidence remain intact.

## Accepted direction

Effectively unlimited logical recoverable history, constrained by real storage and
explicit retention/failure policy. Bounded resident roots are an acceleration cache,
not the authority deciding which transactions survive. The former 32-transition/
16 MiB window is historical research, not a product limit.

Preserve exact content, bindings, IDs, drafts and diagnostics; one transition per
changed request including whole Fill/Paste; fresh request authority; monotonic
allocation floors; independent captures; and existing saved-content/file authority.
Cold restoration is not WB5 Open, name resolution or authoring-command replay.
Durable-before-live-publication is approved for experimentation, not certified for
performance or power-loss durability. Measure payload construction, write, sync,
publication and acknowledgment costs separately.

## Three mandatory safeguards

1. **Identity scalability:** long transaction retention does not expand finite
   lifetime identity budgets. Measure repeated existing-cell edits separately from
   create/Undo/branch cycles. Report where exhaustion becomes restrictive; never
   increase quotas or weaken ABA safety to make the experiment pass.
2. **Non-destructive recovery:** saved A remains normal saved-open A even if the
   journal contains unsaved B/C/D. Offer recovery explicitly. Declining immediate
   recovery preserves recoverable transactions; it is not deletion consent. Prove
   later recovery remains possible and require more than filename/DocumentId for
   association. Reconcile ambiguous commits before mutation or destructive action.
3. **Independent optimization:** Clear History and compiling computation are separate
   decisions. Optimize/JIT/SIMD/compiled formats require their own representation and
   correctness review. None is authorized in this experiment.

## Authorized work and stop point

Execute the bounded four-stage research plan: complete cold inventory and restart
identity/capture proof; checkpoint/delta/shared-object comparison including 1,000/
10,000 transactions; real faulted commit/cursor/branch/reopen paths; and verified
native file/save association, recovery/retention/relocation and writer restrictions.
Stage gates still apply: stop on semantic failures, broad core redesign, required
quota changes, unbounded resource use or unreconcilable commit outcomes.

Keep ordinary workbook edits, memory/cleanup limits and stable IDs intact. No WB5
changes, compatibility adapters, session/protocol operations, desktop controls or
production history integration. Do not select the production container during
research. A preliminary preference for deltas is a hypothesis, not a selection.

Preserve source/build manifests, original evidence hashes and failed attempts.
Report actual platform/filesystem coverage; native macOS/APFS storage validation
precedes cross-platform durability claims. Focused Rust research gates suffice;
no full desktop/Native AOT matrix is required for this phase.

Return measured representation, checkpoint/cache and durability recommendations,
ownership/recovery results and unresolved risks. Stop for parent feasibility review
before production container selection or document/session/desktop history work.
This receipt starts no experiment and claims no disk-history results. ADR-0021/0022
remain closed; existing reliability issues remain open.
