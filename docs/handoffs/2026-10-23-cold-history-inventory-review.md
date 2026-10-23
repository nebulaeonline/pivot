# ADR-0023 cold inventory accepted — cold restoration remains the next gate

Date: 2026-10-23. **Inventory and boundary probes accepted. Continue previously
authorized Stage 1 cold-restoration feasibility. Production Undo/Redo not authorized.**
[Accepted inventory and measurements](../architecture/cold-history-state-inventory.md),
[identity clarification](2026-10-23-cold-history-identity-review.md),
[experiment plan](../architecture/disk-history-feasibility-plan.md),
[ADR-0023](../adr/0023-document-transaction-history.md).

## Accepted findings

The parent accepts the cold-state inventory, process-local NodeId findings,
portable-transfer boundary probes and identity-exhaustion measurements. Durable
logical UUIDs survive restart; runtime NodeIds remain process-local. Preserve exact
runtime IDs during same-session hot/cold activation and coherent remapping into a
fresh namespace after restart. Historical reservations survive branching, eviction
and restart. Old Expected values, handles and session authority never become live
authorization again.

Portable transfer correctly creates a new runtime namespace/ContentStateId and can
reject an old root whose mapping does not span later allocation authority. Preserve
that boundary. Do not modify portable import/export to act as a history codec or
redesign NodeId. Cold history needs a narrow validated reconstruction path with
explicit lifetime allocation authority.

## Separate product-scaling issue

Ten thousand fixed-identity replacements succeeded with one allocated identity.
Create/restore/branch committed 8,191 new allocations after the initial identity,
then reached the existing 8,192 lifetime ceiling with one active cell. The rejected
edit preserved authoritative state. This validates the current ABA rule but limits
useful long-lived authoring independently of disk-history capacity.

Identity-space scalability is a separate architectural prerequisite for removing
experimental workbook limits. Possible future approaches require their own retained
history/capture isolation proofs. No quota increase, identity recycling or document
rebasing is authorized here. Keep the measured limitation visible.

## Required next proofs and stop point

1. **Same session:** evict A, restore A from cold storage, preserve exact runtime
   NodeIds while B's calculation capture remains valid.
2. **Restart:** preserve durable identities and bound relationships with a fresh
   runtime namespace and one coherent mapping across historical A/B/C states.
3. **Independent copies:** overlapping historical logical UUIDs retain isolated
   runtime graph identities, request authority and captures.

Preserve exact typed content, historical invalid/cyclic drafts, bindings, diagnostic
provenance and ContentStateIds. Rebuild derived indexes under existing allocation,
admission and cleanup rules; publish fresh mutation authority. Stop if this requires
broad canonical-core redesign or cannot satisfy the invariants. Only successful
cold restoration permits checkpoint/delta/shared-object comparisons, durable
commit/restart faults and workbook/history-association work to follow.

Exact cold restoration and recoverable disk history remain unproven. No production
coordinator, WB5/protocol changes, desktop history, JIT or optimization is authorized.
This receipt changes documentation only and adds no runtime or validation claim.
Existing evidence/manifests and accepted earlier milestones remain intact.
