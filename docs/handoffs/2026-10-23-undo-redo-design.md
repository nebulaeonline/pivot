# Undo/Redo architecture — parent review packet

Date: 2026-10-23. **Architectural direction conditionally approved; bounded feasibility authorized.
No code, feasibility results or passing history evidence claimed.**
[Review and strict phase boundary](2026-10-23-undo-redo-design-review.md).
[ADR-0023](../adr/0023-document-transaction-history.md),
[accepted Fill and next direction](2026-10-23-four-direction-fill-final-review.md).
ADR-0021/0022 remain accepted and closed.

## Architecture decisions and feasibility qualifications

Approve Rust-owned, document-level root history rather than inverse cell commands,
codec round trips or calculation captures. One changed committed request creates
one action, including a whole Fill/paste. Retain authoritative typed no-op semantics;
no implicit cross-commit grouping or per-keystroke document history.

Approve separate ContentStateId, live mutation authority and immutable calculation
capture identity. Undo revisits content identity but always republishes fresh guard
revisions. Keep session-lifetime allocation floors outside historical content so
branching cannot alias undone/evicted NodeIds/CellIds/ranges. Preserve existing
allocated-identity ceilings rather than restoring capacity by rewinding counters.

Approve these bounded first-slice policies explicitly:

- Ordinary workbook history only. Successful entry into either non-savable proof
  clears/disables history; failed entry does not. Do not rewind source publications,
  producer registries, captured source contexts or presentation state. Future permanent
  structured documents need a separately reviewed partitioned history contract.
- At most 32 transitions/33 states and 16 MiB conservative full-state retention weight.
  Count all states, including current, while transitions exist. Existing allocation
  domain/cleanup limits remain hard constraints. These are proposed bounds, not
  measured capacity guarantees or new numeric/workbook limits.
- On successful changed branch, drop Redo and prune oldest past transitions to fit.
  Keep the immediate before/after pair. If it cannot fit, reject the edit and retain
  the original document/history; never silently commit a non-undoable edit. A valid
  opened baseline may therefore encounter history-admission rejection on a later
  edit. Clear History is explicit, does not change content/dirty state, cannot make
  an oversized root pair smaller, and cannot release externally captured owners.
- Exact saved ContentStateId controls cleanliness, independent of fresh guard
  revisions. File path/fingerprint/checkpoint and installation uncertainty are outside
  history. Undo does not reverse Save As or certify an uncertain write.
- Keep history volatile; New/Open/close reset it only on successful lifecycle changes.
  Preserve current UI selection/viewport across Undo rather than historic selection.

Approve typed Undo/Redo/Clear History requests with expected history epoch plus
current document authority, coherent status/labels and no client-side root stack.
Grid-only Ctrl/Cmd+Z, Ctrl/Cmd+Shift+Z and Windows/Linux Ctrl+Y; editor-local undo
remains local. Document-history controls are disabled during drafts/pending work/
drag/proof mode. No hidden draft commit and no recurring manual checklist.

## Repository findings that motivate the contract

`Workbook` currently couples monotonic state and canonical graph revision in Expected.
Its persistent Data root owns positions, records, portable/runtime bindings and
structured annotations. WorkingModel.stage shares canonical persistent ownership;
there is no public document-history activation operation that safely republishes
old content with fresh mutation authority and lifetime allocation floors. A plain
Workbook clone/assignment would rewind guards/counters and is not sufficient.

`Session.saved` currently stores Expected, and dirty detection compares it with the
live Expected. That cannot make Undo-to-saved clean while also enforcing fresh request
revisions. Introduce saved ContentStateId separately from guard revision and the
existing on-disk checkpoint UUID. Formula diagnostic transaction provenance is data,
not request authority; do not rewrite it on history activation.

PendingSave owns exported bytes/checkpoint/Expected/location and accepts its captured
ticket authority after later edits. Add captured ContentStateId; preserve the existing
one-ticket lifecycle. Saving state A after an edit or Undo must mark A saved, not the
then-current root. It must still work if A's history entry has been evicted.

Current allocation ownership already supports shared roots, bounded domain admission,
cleanup lanes and explicit flush/finish. The history retention-weight quota is
conservative and separate from actual distinct-allocation domain accounting. Do not
claim shared roots are free, double-charge them as fake heap copies, or claim eviction
releases allocations still held by calculations. Implementation must account metadata,
pins and cleanup slots as well as bytes, without increasing capacities.

## Atomicity examples

| Sequence | Required outcome |
| --- | --- |
| Fill then Undo then Redo | Entire rectangle reverses/reappears as one action; exact identities/bindings return |
| Save A; edit B; Undo A | A is clean under fresh guards; requests issued at the original A still reject |
| Undo B to A; no-op/failed edit | Redo B remains available |
| Undo B to A; changed edit C | Discard B only with successful C commit; C uses fresh identities |
| Capture B; Undo A; branch C | Capture B remains isolated; C never reuses B-only runtime IDs |
| PrepareSave B; Undo A; commit B | Disk saved marker becomes B; A stays dirty |
| PrepareSave B; edit C; Undo B; commit B | B is clean even though its live revision changed |
| Prune/Clear saved A; change content | Saved marker still identifies A without retaining its root |
| Memory/counter/cancellation failure during Undo | No cursor movement or root/guard/status changes |
| Save uncertain; Undo to saved content | Uncertainty and existing close safeguards remain |
| Failed Open or proof entry | Original roots, history, labels, dirty state and pending authority survive |

## Acceptance matrix for a later approved implementation

| Area | Required evidence |
| --- | --- |
| Transaction boundaries | Entry, drafts, clear, move/retire, rectangular paste/duplication and each Fill direction; one batch equals one action; no-op/failure adds none |
| Root activation | Worksheet and canonical roots switch together; no text reparse, value replay, codec restore or full graph materialization |
| Identity/ABA | Restore original CellIds/NodeIds/SelectionIds; fresh mutation/private/context tokens; stale original Expected rejects after returning to its content |
| Allocation authority | Allocate B-only IDs, Undo, discard/evict B, edit C with B capture retained; no aliasing; floors and quotas do not rewind; failed allocation changes no committed ledger |
| Formula/draft state | Moved scalar bindings, mixed-dollar/reversed aggregate ranges, blank/zero, exact decimal/text, retired references, invalid/cyclic drafts and original diagnostics restored |
| Branches/no-ops | Repeated Fill, clear of blank, empty Undo/Redo, failed branch after Undo; no identity/cursor churn or premature Redo destruction |
| Saved state | All table sequences; initial untouched document clean without a file; saved state eviction; recreation of equal values remains dirty; Save As association not undone |
| Pending saves | Save S while current T; Undo-to-S before completion; source root evicted; cancellation/failed write/uncertainty; stale ticket after successful document replacement |
| Bounds | Count and conservative weight boundaries; mandatory pair too large; actual domain and cleanup-ticket pressure; no hidden capacity increase or pre-commit eviction |
| Ownership | Shared leaves charged once by domain, conservative retention separately; capture/save ownership independent; prune/clear/close cleanup and zero-charge finish |
| Control epoch | Clear History changes only epoch/status; old control epoch rejects; normal no-ops preserve it; coherent counts/labels after every outcome |
| Modes/UI state | Proof barriers, failed entry, projected/presentation changes excluded; selection/viewport preserved; delayed UI work invalidated; draft controls do not mutate document |
| Cancellation | Before install preserves both stacks and root; committed result drained/acknowledged after caller cancellation; no automatic retry |
| Protocol | Typed operations/status, missing/unknown authority rejected, actual older-peer unsupported-opcode rejection; no codec adapters |
| Desktop | Routed Undo/Redo/Clear menu and scoped shortcuts, local editor isolation, full Fill undo/redo, save/dirty indicators; retain existing twelve fields plus history acceptance |
| Final gates | Windows/Linux/native Mac managed CI, strict all-feature and actual Native AOT, matched manifests, warning-free native publication, zero-charge main/peers and APFS-original per-file evidence hashes |

## Phases subject to the parent feasibility gate

1. Establish narrowly scoped root activation, fresh generation/allocation-floor and
   conservative weight/pin accounting foundation. Prove ABA/capture isolation and
   bounded reclamation with focused tests before session/UI integration. If this
   requires larger core redesign or cannot satisfy quotas, stop for review; do not
   replace history with serialization or raise limits to make tests pass.
2. **Deferred until separate authorization after feasibility review:** document history
   transaction admission, branch cursor, eviction and Clear History;
   distinguish current-root guards from saved content and in-flight save identity.
3. Private request/status and exact epoch/state checks; real C#–Rust boundary tests.
4. Desktop actions and integrated history acceptance automation alongside prior twelve
   fields; then final three-platform validation and parent implementation review.

The parent authorizes only phase 1 bounded root-activation/ownership feasibility.
A research harness may measure retained roots and pruning without introducing a
general history stack. Stop for parent feasibility review; phases 2–4 are not yet
authorized. Production saved-content/session/protocol/desktop integration is deferred.
No speculative remediation of the open Windows clipboard, WSLg startup, mounted-path
Save uncertainty or historical Linux viewport p95 issues is included.
