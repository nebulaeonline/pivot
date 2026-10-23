# ADR-0023 cold identity clarification — restart remapping approved

Date: 2026-10-23. **Identity boundary resolved for the authorized disk-history
experiment. Exact cold restoration remains unproven; production integration deferred.**
[ADR-0023](../adr/0023-document-transaction-history.md),
[inventory and existing probes](../architecture/cold-history-state-inventory.md),
[experiment plan](../architecture/disk-history-feasibility-plan.md),
[earlier architecture authorization](2026-10-23-tiered-history-design-review.md).

## Parent decision

Separate durable logical identity, process-local runtime identity and fresh request
authority. Do not redefine core NodeId as a durable identifier.

| Operation | Durable logical IDs | Runtime NodeIds | Live request authority |
| --- | --- | --- | --- |
| Hot activation within a session | Preserve | Preserve exactly | Issue fresh |
| Cold activation after resident eviction within that session | Preserve | Preserve exactly | Issue fresh |
| Reopen after process restart | Preserve | Remap into a fresh runtime graph namespace | Issue fresh session/activation authority |
| Independent copies opened together | Preserve within each lineage | Isolated namespaces per opened graph instance | Independent authority |
| Changed branch after Undo | Preserve existing IDs; allocate new IDs for new objects | Fresh identities where required; no alias with prior branches | Issue fresh |

Preserve durable cell/node/range UUIDs, historical ContentStateIds, TransactionIds,
original bound-target relationships and durable identity reservation metadata.
References must remain logically equivalent after restart; old runtime bit patterns
are not the cross-process semantic oracle.

One reopened session must use a coherent mapping for every historical state it
loads. Do not assign conflicting runtime identities independently on each cold
retrieval. Preserve reservations for logical objects created on discarded or
unreachable branches; remapping does not recover allocation quota or permit logical
identity reuse. Keep stale-authority rejection and independent capture isolation.
Persisted provenance is data, never authority. No old request handles, pointers,
ownership tokens or runtime authority become valid after restart.

## Required separate evidence

1. Evict and cold-restore A in a live A/B/C sequence while B's capture survives:
   exact runtime NodeIds, durable IDs, historical bindings/drafts and fresh guards.
2. Restore after a real process restart: exact durable identities and bound-target
   equivalence under fresh runtime mapping; consistent mapping over repeated A/B/C
   loads, including after unrelated models have already been allocated.
3. Open independent copies simultaneously: overlapping historical logical IDs do
   not share runtime namespaces, authoring authority or captures.

Do not change NodeId representation or introduce persistent model-number ownership.
If same-session cold restoration cannot preserve runtime IDs without redesign,
stop and report before weakening that invariant. A durable-model redesign remains
separate future work only if a concrete requirement justifies it.

This clarification removes the identity-contract question, not the Stage 1 proof
gate. Existing probes and frozen manifests remain evidence for their original source;
none proves the new cold cases. Continue the already authorized bounded experiment;
the production/container/desktop/Optimize boundaries remain unchanged.
