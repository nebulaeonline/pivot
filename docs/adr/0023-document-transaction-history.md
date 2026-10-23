# ADR-0023: Bounded document transaction history

Status: **Architectural direction conditionally approved; bounded feasibility implemented and awaiting review; full implementation deferred (2026-10-23)**.
[Executable foundation and measured findings](../handoffs/2026-10-23-undo-redo-feasibility.md).
[Parent architecture review and feasibility boundary](../handoffs/2026-10-23-undo-redo-design-review.md).
Date: 2026-10-23.
[Design packet and acceptance matrix](../handoffs/2026-10-23-undo-redo-design.md).
Predecessors: [portable workbook lifecycle](0009-portable-documents-and-local-lifecycle.md),
[accepted Fill](../handoffs/2026-10-23-four-direction-fill-final-review.md),
[Fill transaction design](0022-four-direction-rectangular-fill.md).

## Purpose and first implementation boundary

Provide document-owned Undo/Redo for ordinary workbook mutations. One changed
committed request is one history transition: cell entry, rejected-draft commit,
clear, move/retire where exposed, rectangular paste, trusted duplication and Fill.
An entire changed Fill or paste is one Undo, irrespective of destination count.
No-op operations and rejected/cancelled-before-install requests create no entry.
Do not infer change from displayed values: typed content, bindings, identities and
the existing authoritative mutator result govern the transaction boundary.
Explicit formula resubmission retains its existing rebinding/provenance semantics.
No cross-commit typing coalescing in this first slice.

History is in-memory, per open document, and owned in Rust. Do not implement inverse
formula edits, persist an undo log, add a database/journal, replay historical commands,
or treat calculation captures as editable history. WB5 remains the document format;
Undo/Redo stacks and history tokens are not serialized. No formatting, extra functions,
structural row/column operations, collaboration or capacity expansion is included.

Initial history supports ordinary workbook mode only. Successful entry into either
non-savable projection/structured-binding experiment is an explicit history barrier:
release both stacks, disable document history and report that reason in the existing
experimental UI. Proof publications/layout/rename/retirement are not history actions.
There is no Undo across that mode transition or back into an old ordinary document.
Failed mode entry preserves the old history. Leaving the experiment by successful
New/Open establishes a new ordinary baseline. This bounded exclusion requires review;
it is not a permanent policy for future structured documents. Do not snapshot and
rewind external source contexts to fake proof-mode Undo.

## Three different identities

| Identity | Meaning | Behavior on Undo/Redo |
| --- | --- | --- |
| ContentStateId | Opaque identity of an authoritative committed document state | Revisit the original ID |
| Mutation generation / Expected / private revision | Authority for the next request | Always fresh; never rewind or reuse |
| Calculation CaptureId/context | Immutable invocation state, owned independently | Remains pinned to its captured state |

A ContentStateId is not a file checkpoint UUID, backend revision, pointer or content
hash. A genuinely changed commit creates a fresh ID; exact historical root activation
restores its ID. Re-entering equivalent text in a later changed transaction does not
make it the saved state by numeric or serialized-content equality.

Undo/Redo republishes historical definitions under a fresh canonical mutation
generation, increments the live workbook Expected.state and issues a fresh private
revision/computational-state token. Old Expected values and pending authoring handles
must reject even after returning to the same content. Never reinstall an old Workbook
wrapper containing its old public state counter. Stored formula diagnostic/binding
provenance remains historical data; it is not new request authority.

## What a history root owns

A private, version-pinned DocumentState owns the canonical definition/index/value
roots together with workbook persistent records, positions, portable/runtime bindings,
blank markers, literal/draft payloads, selection geometry/identities and sheet/axis/
graph identity metadata. It also carries ContentStateId. Both halves switch together.
Do not rebuild formulas from text, import a WB5 image, materialize the whole workbook,
resolve names, evaluate cells or regenerate identities during Undo/Redo.

Share existing persistent storage with admitted wrappers and existing deferred
reclamation. A history root is not CalculationCapture: it retains editable document
definitions rather than a prepared invocation/source context. History must not retain
unrelated calculated results, viewport frames, OS clipboard data or producer records.

Do not rewind document-lifetime allocation authority. Keep high-water/tombstone
reservations for runtime node/port identities, portable cell/node IDs and range
identities outside undoable content. A new branch may not allocate an identity that
belonged to an undone or evicted future state. Redo reactivates the original IDs;
branching uses fresh IDs. Preserve current allocated-identity ceilings over the
whole open-document lifetime, including identities absent from the active root.
History eviction is not permission to reuse IDs or recover an allocated-identity
quota. Failed candidates publish neither allocation floors nor reservation changes.

The canonical activation API must republish an owned historical root with current
allocation floors/fresh generation while preserving its definitions and bindings.
It must work with ordinary invalid/rejected/cyclic drafts already accepted by
historical authoring. Do not run Fill's candidate-cycle rejection over a trusted
historical root and thereby make an accepted old draft impossible to restore.
Existing captures can outlive branch deletion/eviction, continue evaluating their
original generation and never alias nodes allocated by a later branch.

## Transaction and branch contract

Keep a linear cursor over at most 32 retained transitions/33 document states. Each
transition has trusted bounded action metadata (for example Edit cells, Paste,
Fill Down), not arbitrary client-provided text or stored cell contents in menu labels.
Before/after roots are adjacent states, not independently duplicated per action.

For a changed ordinary edit, stage mutation, before/after root ownership, fresh
identity tokens, history admission, proposed pruning and the new cursor together.
Install document roots, allocation authority and history metadata atomically once.
If any preparation fails, preserve live contents/identities, both stacks/cursor,
revision, dirty state, path and saved marker. In particular, do not destroy Redo
or evict old Undo entries before a new branch has successfully committed.

Undo activates the preceding state and moves the cursor back; Redo activates the
following state. Neither executes the old command. A changed edit after Undo drops
the future branch only at successful commit. A no-op/failed edit after Undo leaves
Redo intact. Empty-stack Undo/Redo returns an unchanged validated no-op, preserving
all tokens. Stale state, invalid mode and pre-install cancellation still reject.

Cancellation before installation changes nothing. After installation the session
must return or drain/reconcile the committed transition; caller cancellation cannot
undo it and must not trigger automatic retry. A dispatched ambiguous transport
failure follows existing session-failure rules, never command replay.

## Bounded ownership and eviction — proposed concrete policy

Initial design targets pending accounting feasibility: **32 transitions**, plus a
**16 MiB conservative history-state weight**.
When transitions exist, the weight counts all retained states including the current
state, so moving the cursor cannot change admission. Empty-history baselines own
only their normal current workbook root; their weight is checked with the first
candidate transition. These bounds are additional admission constraints, not an
increase to current workbook, graph, runtime, capture or cleanup capacities.

A backend-supplied conservative weight must cover full reachable canonical/workbook
state, including allocated/retired records, selection/index structure, strings,
tokens, reference metadata and wrapper overhead. Maintain trusted checked counters
at mutation boundaries; do not scan/serialize every root on each edit. Shared leaves
may be charged conservatively in more than one state weight. This is a retention
quota, not a claim of exact RSS or incremental unique-node measurement.

Actual allocation-domain accounting continues to charge each distinct live allocation
once. Metadata/wrappers/plans and cleanup tickets require actual domain admission;
do not reserve the full logical weight a second time as fake heap allocation. The
existing domain remains the hard limit, even if the logical history quota has room.

For changed-edit admission, propose dropping the Redo branch, then evict oldest past
transitions until count/weight fit. Retain the immediate before/after pair as an
indivisible mandatory unit. If even that pair cannot fit, reject the edit explicitly
as history-capacity admission, leaving its content and existing history unchanged.
Do not silently perform a non-undoable edit. Thus some otherwise admissible workbook
edits can reject under the new bounded history contract; approving this policy is
an explicit review decision, not an undocumented fallback. Valid Open can establish
an empty baseline even when the first subsequent edit would exceed this quota.

Admission must accommodate candidate/transient owners and existing history together;
do not assume planned evictions have freed memory before commit. An operation may
reject on actual domain/cleanup pressure even if post-pruning logical weight fits.
After commit, detach pruned owners promptly and reclaim them through the established
bounded cleanup lanes. Captures or other owners may keep storage resident; do not
claim eviction releases their memory. No synchronous deep destruction or unbounded
waiting on the UI/request thread. Final flush/finish must show zero remaining charge.

Provide explicit Clear History as a recovery/control action: release Undo/Redo
ownership, leave the current document, ContentStateId, saved marker, file association,
dirty state and mutation revision unchanged, and refresh only history epoch/status.
It cannot release capture-owned or current-root storage, recover lifetime identity
quotas or guarantee that an oversized before/after pair will fit. It creates no undo
entry and is not itself undoable. Clearing an already empty history is a no-op.

If safe constant/bounded root activation, generation/allocation-floor handling or
conservative retention-weight maintenance cannot be established without larger core
changes, stop and report the evidence for design review. Do not substitute a codec
round trip, increase quotas, weaken ID isolation or silently turn history off.

## Saved state and file lifecycle

Replace revision-equality dirty detection with ordinary document content-state
comparison: clean when current ContentStateId equals the last successfully saved
ContentStateId and file installation is not uncertain. A new untouched workbook
has a clean initial baseline without inventing a path or saved-file checkpoint.
Guard revisions remain fresh regardless of cleanliness.

The saved marker is metadata, not an extra root pin or a protected history entry.
It may refer to a state no longer reachable through history. It persists through
pruning, branching and Clear History until another successful Save/New/Open/close.
Do not silently mark a branch clean because it renders the same values as the file.

| Sequence | Required result |
| --- | --- |
| Save A; edit B; Undo to A | Clean, with a new request revision |
| Save A; Undo to predecessor | Dirty |
| Save A; Undo; Redo to A | Clean |
| Save A; edit B; Undo A; changed edit C | Dirty; discarded B cannot Redo |
| Evict saved A; edit/recreate similar values | Dirty until a successful Save or exact reachable saved-state return |

PrepareSave captures immutable exported bytes, captured ContentStateId and the
existing ticket/document/location authority. It need not pin an additional history
root once encoding is complete. Allow edits and Undo/Redo while a save is prepared.
Committing Save of S marks S as the saved content, not whichever content is currently
active. If Undo returned to S under fresh guards before Save completed, S is clean;
if current content is T, it remains dirty. Pruning S from history does not invalidate
its prepared bytes or ticket. Preserve current one-ticket bounds and no-replay rules.

Save/Save As and file fingerprints/path/checkpoint are not undoable content. Undo
never restores an old filesystem observation or reverses a successful write/Save As.
A failed Save preserves prior successful saved marker/location as currently specified;
installation uncertainty remains a separate sticky condition. Undo/Redo/Clear cannot
clear uncertainty or certify the disk file; even equal saved content must retain the
existing uncertain/dirty close safeguards until a successful permitted save/reopen.

Successful New/Open/Close Document clears history and invalidates pending tickets/
old document handles under existing lifecycle rules. Failed/cancelled Open/New/close
preserves everything. Open creates a clean new in-memory content baseline with empty
history, not persisted history. Do not support Undo Open, Undo New or Undo Save As.

## Source, presentation and editor boundaries

Read/evaluate/capture/release, viewport/selection/focus changes, refresh and clipboard
Copy do not create document-history entries. Computed cache values are derived, not
restored document content. Editor keystrokes use TextBox-local undo where supported;
a successful cell commit creates one document action. Document Undo/Redo does not
implicitly commit/discard a live draft and is disabled until that draft is finished.

The initial proof-mode barrier above prevents restoring stale source contexts or
producer registries. Future structured-document history must partition user-owned
bindings/layout from externally published source state in a separately reviewed
contract; source refresh must never be undone as if it were a user cell edit.

History activation preserves the current logical selection/anchor/active coordinate,
scroll and focus rather than restoring old UI selection. A cell that disappears on
Undo is displayed as blank at the same valid coordinate. Invalidate pending viewport,
clipboard/selection callbacks and prepared authoring handles with fresh guards;
never retarget late work onto a restored state. Captured calculations remain valid
against their own IDs/contexts, not the refreshed live document.

## Private session and desktop proposal

Propose additive operations UNDO=32, REDO=33, CLEAR_HISTORY=34. Request field 29 is
`expected_history_epoch`, mandatory for these control operations alongside exact
revision/computational state. Return response field 42 `HistoryStatus` containing
opaque epoch, enabled, undo_count, redo_count, fixed bounded undo/redo labels and an
unavailable reason. Use a fresh epoch on changed edits/cursor moves/pruning/clear/
lifecycle or mode barriers; no-op operations preserve it. Counts and labels describe
one coherent installed result, never a C#-maintained imitation stack.

Clear History changes only history epoch/status, so stale history-control requests
must reject even if the document revision has not changed. Normal cell requests
continue using exact document authority; they need not carry a history cursor or
client-supplied root. Old native clipboard payloads are data, not history authority.

Retain private v4 only if actual older peers reject new opcodes without side effects
and additive fields leave existing operations intact; stop for version-cut review
if a real incompatibility is found. WB5, CB2 and frozen hosted v1 remain unchanged.
No compatibility adapters or persisted history representation are proposed.

Add accessible Undo/Redo menu commands showing the current fixed action label and
availability, plus explicit Clear Undo/Redo History. Grid-only shortcuts: Ctrl/Cmd+Z
Undo; Ctrl/Cmd+Shift+Z Redo; Ctrl+Y additionally on Windows/Linux. Do not intercept
inside TextBoxes or unrelated controls. Disable document-history controls during
a draft, drag, pending mutation/lifecycle action, or proof mode; expose why unavailable.
Apply one typed request, refresh from its response and preserve logical viewport/
selection. No native dialogs or recurring physical-keyboard checklist are required.

## Validation and stop point

The linked matrix requires branch/ID ABA isolation, historical invalid drafts,
atomic admission and cancellation, saved-marker/in-flight-save behavior, proof
barriers and captures surviving history eviction. Validate runtime reclamation and
all existing regressions; only then add actual desktop automation and normal final
managed/all-feature/Native AOT gates on Windows/Linux/native Mac/APFS with matched
manifests and APFS-original per-file evidence hashes. No test counts are predicted.

The parent conditionally approves the architecture and authorizes only bounded
root-activation/ownership feasibility. Prove A → B → restore A → branch C with a B
capture retained, including invalid/cyclic drafts, monotonic identity floors, bounded
weight maintenance and realistic charge measurements. The 16 MiB target and fail-closed
admission require practical evidence. Saved-content/file-authority separation remains
mandatory. Stop for parent feasibility review before a general history stack or any
production save/session/protocol/desktop integration. No runtime work is claimed
by this approval record.
ADR-0022 remains accepted/closed; existing reliability issues remain unresolved.
