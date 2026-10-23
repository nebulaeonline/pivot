# ADR-0023 architecture review — feasibility foundation authorized

Historical first review. The [subsequent acceptance and retention direction](2026-10-23-undo-redo-feasibility-review.md)
supersedes the fixed product retention policy below. See the
[revised tiered architecture packet](2026-10-23-undo-redo-design.md) for current proposals.

Date: 2026-10-23. **Architectural direction approved with three clarifications.
Immediate authorization: bounded root-activation and ownership feasibility.
Full Undo/Redo implementation is not authorized.**
[Subsequent executable feasibility findings, pending review](2026-10-23-undo-redo-feasibility.md).
[ADR-0023](../adr/0023-document-transaction-history.md),
[design packet](2026-10-23-undo-redo-design.md).
This is architecture approval, not feasibility acceptance or implementation acceptance.
ADR-0022 remains accepted and closed.

## Approved direction

Rust-owned immutable document-root history, one action per changed committed user
transaction, separate ContentStateId/mutation authority/calculation capture identity,
monotonic document-lifetime allocation floors and immutable capture isolation.
No inverse commands, replay, text reconstruction or codec restore. No-op/failed/
pre-install-cancelled actions add no entries. One Fill/paste is one future Undo.

Approve ordinary-workbook-only scope and successful proof-entry history barriers;
failed entry preserves history. External source publications and presentation state
are excluded. Preserve current viewport/selection/focus rather than historical UI.
A linear cursor, atomic branch replacement/pruning, Redo preservation on failed/no-op
edits and explicit non-undoable Clear History are accepted design directions.

## Clarification 1 — root activation must be proven first

Demonstrate narrow canonical historical-root activation with fresh mutation/Expected
authority and current lifetime allocation floors. Execute A → B → restore A → branch C
while retaining a calculation capture of B. Prove exact historical identities/bindings,
no reuse of B-only IDs in C, stale-authority rejection, capture isolation and bounded
reclamation. Include accepted historical invalid and cyclic drafts, without applying
Fill's stricter candidate-cycle rule to historical root activation.

Stop and submit findings if this requires broad core redesign. Do not work around it
with WB5 serialization, larger quotas, weakened identity guarantees or altered document
semantics. No general history stack, session/protocol surface or desktop integration
is permitted in this phase.

## Clarification 2 — ownership/accounting is an independent feasibility gate

32 transitions/33 states and 16 MiB conservative history weight remain design targets,
not proven practical limits. Demonstrate bounded trustworthy weight maintenance
without full-root scanning/serialization per edit. Separately report logical weights,
actual distinct domain charge, wrappers, staging and deferred cleanup ownership.
Measure small, medium and near-capacity cases, repeated Fill, captures retained
across pruning and branch replacement. Pruning must not assume memory owned by
captures has been reclaimed. Show atomic admission/failure behavior.

Fail-closed oversized before/after-pair rejection is accepted for experimental
consideration, subject to measured impact. If normal workloads unexpectedly reject
their first undoable edit, return for review. Never silently commit a non-undoable edit.
Use a bounded research harness for measurements, not a production history stack.

## Clarification 3 — saved content is distinct from file authority

Exact saved ContentStateId determines ordinary content cleanliness independently of
fresh guard revisions. Paths, fingerprints, checkpoint authority and uncertainty are
not restored by Undo. Prepared bytes, captured content identity and valid save ticket
remain authoritative if their state is edited away, pruned or cleared from history.
Require tests for completion after the state no longer belongs to history; the ticket
must not be redirected to current content. Undo does not resolve uncertain installation.
This contract is approved; production save/session integration remains deferred.

## Deliverable and stop point

Provide focused executable tests, root/identity/capture evidence, measured logical
weights versus domain charge, ownership/reclamation receipts and explicit failure/
stop conditions. No full Undo/Redo claim or three-platform desktop certification is
required for this bounded feasibility experiment. Report what was actually executed.
Stop for parent feasibility review. Only later separate authorization can start full
history/session/save/desktop integration and its normal three-platform managed/AOT gates.

Typed UNDO/REDO/CLEAR_HISTORY, history epoch/status and shortcuts are directionally
approved but deferred. Retaining private v4 will require actual older-peer rejection
and semantic compatibility evidence at that stage. WB5, CB2 and hosted v1 stay unchanged.
No formatting, structural edits, persisted history, structured-source Undo or capacity
expansion. Existing reliability issues remain open. This receipt starts no runtime work.
