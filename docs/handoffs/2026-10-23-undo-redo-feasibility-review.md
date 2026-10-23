# ADR-0023 feasibility accepted — history architecture revision required

Date: 2026-10-23. **Root-activation feasibility accepted. Fixed retention policy
declined as product policy. Production Undo/Redo remains deferred.**
[Accepted experiment and evidence](2026-10-23-undo-redo-feasibility.md),
[ADR requiring revision](../adr/0023-document-transaction-history.md).
[Subsequent revised architecture packet, pending review](2026-10-23-undo-redo-design.md),
[proposed disk-history experiment](../architecture/disk-history-feasibility-plan.md).

## Accepted foundation

The parent accepts the feature-only Windows/Linux evidence for canonical/workbook
root activation under fresh mutation authority and monotonic lifetime allocation
floors. Historical invalid/cyclic drafts, stable identities, branch isolation,
retained calculation captures and bounded ownership establish the intended hot-root
mechanism. This accepts a research foundation, not a production history coordinator.
The existing evidence and frozen artifact manifests remain unchanged.

## Superseding product direction

Effectively unlimited recoverable transaction history is the target, subject to real
storage availability and explicit user retention policy. The earlier 32 transitions,
33 states and 16 MiB conservative full-state weight are experimental measurements,
not accepted product limits. A full memory cache must not itself cause an otherwise
valid edit to fail or silently destroy recoverable history.

Design a bounded resident cache of recent immutable roots backed by extensible
disk history for older transactions/checkpoints. Evicting resident roots must not
delete their recoverable history. Preserve individual transaction boundaries,
ContentStateId, stable transaction identities, fresh activation/request authority
and monotonic allocation authority across branches, eviction and restart.

Keep the existing transaction, cancellation, saved-content, in-flight-save,
dirty-state and uncertain-installation contracts. Current workbook content,
history persistence and filesystem save authority remain separate. Across-session
history is an intended capability with explicit retain/discard choices; Save does
not automatically discard history. Define safe behavior when a workbook is copied,
moved or externally replaced independently of its history.

Storage exhaustion, unavailability, corruption and interrupted writes require
explicit recovery/failure policies. Never silently truncate promised history or
commit an unnoticed non-undoable edit. The parent suggests offering explicit choices
such as freeing storage, clearing history, relocating it or deliberately continuing
without retaining an operation. The revised design must specify the actual contract.
Unlimited logical history does not promise infinite physical resources.

## Next design gate

Prepare a revised ADR-0023 and bounded disk-history feasibility plan. Compare
checkpoint-plus-delta, persistent-state serialization and journal approaches by
correctness, bounded retrieval/restoration work, disk usage, reliability and
operational complexity. Do not select an embedded container, sidecar or database
prematurely. Do not introduce WB5 compatibility readers, replay ordinary authoring
commands or use calculation captures as mutation history.

The plan must measure checkpoint/delta size, retrieval bounds, disk-backed Undo
latency, crash/restart consistency, resident eviction without history loss and
resource ownership. In particular, the accepted in-memory activation proof does
not establish a cold-state representation or recovery protocol. Returning from disk
must preserve canonical identities/bindings and accepted draft semantics while
minting fresh request authority and retaining allocation floors.

Submit the revised architecture for review before full session/desktop history
integration. This receipt records direction; it does not implement that revision
or start disk-history runtime work.

## Future Optimize Worksheet direction

Record optimization separately from Undo/Redo implementation. Compiling eligible
graphs into fused execution units and possible SIMD acceleration are future work,
subject to exact numeric/dependency semantics and evidence. Keep canonical computation
definitions authoritative unless a separate representation contract is approved.
Compilation and history deletion are separate operations; discarding history requires
explicit user authorization. No JIT, SIMD, format changes or irreversible compaction
are part of the present phase.

ADR-0021/0022 remain accepted and closed. Existing reliability issues remain open.
