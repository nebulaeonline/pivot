# Tiered Undo/Redo architecture — revised parent review packet

Date: 2026-10-23. **Revised architecture approved with safeguards; bounded experiment authorized.**
[Parent approval and next boundary](2026-10-23-tiered-history-design-review.md).
[Subsequent cold-identity clarification](2026-10-23-cold-history-identity-review.md).
[Revised ADR-0023](../adr/0023-document-transaction-history.md),
[bounded disk-history feasibility plan](../architecture/disk-history-feasibility-plan.md),
[parent acceptance and revised direction](2026-10-23-undo-redo-feasibility-review.md).
The [accepted hot-root evidence](2026-10-23-undo-redo-feasibility.md) and its frozen
manifests are preserved. No disk-history experiment or production integration was
implemented for this revision. ADR-0021/0022 remain accepted and closed.

## Proposed change

The resident cache accelerates history; it no longer determines how much recoverable
history a user has. Disk-backed transactions/checkpoints support effectively unlimited
logical Undo/Redo within available storage and explicit retention policy. A full
cache triggers recoverable eviction, not silent history deletion or a fixed-window
edit rejection. Existing actual memory/identity/cleanup quotas remain hard limits.

Keep one action per changed commit, exact historical content/bindings/drafts,
fresh activation authority, monotonic allocation floors, independent captures,
atomic branch replacement and the saved-content/in-flight-save/uncertainty contracts.
Keep one user-visible linear timeline; an intentional changed edit after Undo still
replaces Redo at commit. Unlimited Undo is not a permanent branch archive.

The identity clarification preserves durable logical UUIDs, ContentStateIds,
TransactionIds and bound-target relationships across restart, while permitting fresh
process-local runtime mappings. Hot or cold activation within one live session must
preserve exact runtime NodeIds. All states in a reopened session share one coherent
mapping, and independently opened copies have isolated runtime namespaces. Persist
discarded-branch logical reservations; never restore old request authority. The plan
now tests these three cases separately. No core NodeId migration is included.

## Review decisions

| Decision requested | Proposed contract |
| --- | --- |
| Tier boundary | Bounded root/index/buffer cache plus recoverable disk history; eviction preserves logical history |
| Cold state | Versioned canonical/workbook data, exact bindings/drafts/IDs, rebuilt private indexes; no authoring replay or WB5 shortcut |
| Commit boundary | Persist payload then verifiable transaction/head/cursor/ledger decision before infallible live publication; ambiguous outcomes suspend and reconcile |
| Lifetime authority | Persist committed identity floors across branches/eviction/restart; never recover quota by Undo or Clear |
| Retention | Support across-session retain and disclosed session-only close policy; Save alone does not discard; default/placement remain open |
| Disk failure | Preserve the old committed state when definitely uncommitted; explicit recovery choices; no unnoticed non-undoable edit |
| File/history association | Match proven saved checkpoint/content and instance/file observations; no auto-attachment by DocumentId/path; independent files remain usable |
| Explicit deletion | Clear establishes a durable current baseline/ledger before unlinking history; saved state/path/tickets remain independent |
| Proof modes | No silent history destruction on entry; explicit boundary consent or detached preserved ordinary history; external publications remain excluded |
| Optimize | Separate future compilation and optional destructive compaction; canonical definitions authoritative, no implementation now |

The durable-before-publish boundary introduces a cost to measure: edit acknowledgment
includes the history store's commit work, even when the hot-root change is cheap.
Do not advertise the nanosecond hot-activation measurements as disk-backed Undo or
edit latency. Physical synchronization guarantees remain an explicit experiment gate.

Current WB5 is not assumed to carry history-instance/content IDs. Attachment and
save-receipt reconciliation must be demonstrated without pretending a workbook write
and a history-store commit are one atomic operation. If a format change proves
necessary, return for separate representation review rather than slipping one into
the experiment. Store placement/container and exclusive-writer mechanics remain
specific design choices to resolve with evidence, not implementation claims.

## Bounded experiment requested

1. Define and validate the complete cold-state inventory and identity oracle. Prove
   cold A/B/C branch/capture isolation and restart before broadening storage work.
2. Compare full checkpoints, checkpoint plus canonical record deltas and shared
   immutable objects with commit journaling. Measure 1,000/10,000 replacement edits,
   existing workbook occupancy cases and batched Fill/Paste within current limits.
3. Demonstrate bounded retrieval/index/cache ownership, durable branch/cursor/ledger
   agreement, actual faulted reopen and resident eviction without history loss.
4. Test native file association/save receipts, retention/clear/relocation, copies,
   external replacement and storage failures. Report precise platform guarantees.

The plan defines measurements, failure injection and stop conditions. It is not a
production history coordinator and does not authorize session, protocol or desktop
work. No opcodes/fields, cache defaults, database or container are selected here.

The parent has authorized the narrow experiment with separate identity-exhaustion
measurements, non-destructive recovery decline and independent optimization/deletion.
Its findings must return for feasibility review before production container selection
or full implementation. Existing clipboard,
WSLg startup, mounted-filesystem Save uncertainty and Linux timing issues remain
open and outside this design round.
