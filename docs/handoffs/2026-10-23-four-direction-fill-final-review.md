# ADR-0022 final implementation acceptance and scope closure

Date: 2026-10-23. **Parent disposition: accepted. Four-Direction Rectangular Fill
milestone closed.** This records implementation acceptance separately from the
[design approval](2026-10-23-four-direction-fill-design-review.md).
[Implementation and final evidence](2026-10-23-four-direction-fill-implementation.md).

## Accepted implementation

Up/Down/Left/Right derive aligned destination definitions directly from the original
captured source edge in one atomic changed document mutation. Shared trusted planning
retains typed Blank, Decimal96, literal text, rejected drafts, scalar/range formulas
and experimental structured output bindings. Preserve effective bound coordinates,
dollar flags, aggregate kinds and producer/output identities. Presentation records
and chrome never participate as logical worksheet cells.

The three governing clarifications are satisfied: compare authoritative intended
definitions before range/node identity allocation; preserve destination CellIds when
clearing without materializing absent blanks; distinguish pre-install cancellation
from post-dispatch committed-state acknowledgement, with no automatic replay.
Repeated identical Fill retains revision, checkpoint, dirty state and graph/range
identities. Explicit zero remains distinct from Blank for COUNT/AVERAGE.

The parent explicitly accepts staged canonical cycle-only validation as a focused
extension required for Fill. It uses canonical dependencies, evaluates no numeric
values/producer outputs and leaves ordinary authoring's editable-draft policy intact.
Late failures retain roots, identities/ledgers, revision, dirty state, path and saved
checkpoint. Preserve bounded planning/ownership/cancellation/deferred cleanup tests.

Accept FILL31/direction field28 under private v4, supported by actual older-peer
unsupported-opcode rejection. WB5-only read/write, CB2 and frozen hosted v1 remain
unchanged. Accept accessible submenu and grid-scoped Ctrl/Cmd+D, Shift+D, R, Shift+R,
editor exclusion, selection/active-cell/viewport/focus preservation and unchanged
clipboard. Routed tests do not certify physical keyboards, accessibility devices,
IME or native pickers. No recurring manual checklist is required.

Keep the real-fill area limit of 256 selected positions including source edge,
existing planning/graph/workbook/structured quotas and validated degenerate no-ops.
No chunking, partial installation, series generation or capacity increase.

## Accepted evidence

Windows: 346 default / 362 all-feature Rust tests. Linux/WSLg and native Mac/APFS:
347 / 363 each. All three platforms pass managed CI, strict all-feature checks and
actual Native AOT with twelve desktop fields, warning-free native publication and
zero-charge main/three-peer workflows. Mac run `macos-validation-20261023T121442Z`
matched 419 source hashes and 211 build inputs. Windows independently verified 465
APFS-original evidence hashes, the manifest hash and receipt checksum. This run
provides repeatable transfer evidence; ADR-0021's earlier receipt limitation remains
part of that milestone's historical record rather than being retroactively changed.

Preserve the initial Windows/Linux projection-automation failures and the correction
to read an offscreen logical cell through the session. The parent accepts this as
a test correction, not weakened Fill semantics or an unresolved product regression.
The final Mac run had no setup failures, product changes or gate reruns.

## Open issues and scope closure

The intermittent Windows ClipboardChecklistAsync null destination, prior WSLg startup
allocator abort, Windows-mounted Save installation uncertainty and historical Linux
viewport p95 exception remain unresolved. Passing runs do not establish root causes.
No speculative remediation or test weakening is authorized by this acceptance.

ADR-0022 is closed. No history, series, handles, formatting, extra functions,
persistent structured integration, structural edits or capacity expansion is included.
No further implementation begins automatically.

## Recommended next gate — Undo/Redo architecture design

The next proposed milestone is document-owned transaction-history design, not code.
Resolve historical workbook-root ownership, stable identity under reversal, bounded
history/quotas/reclamation, saved checkpoints and dirty state, branching after Undo,
and separation of user document mutations from structured-source publications and
presentation changes. Immutable calculation captures are not an undo log.
One successful changed Fill should eventually be one undoable document action.
This recommendation does not start or approve an Undo/Redo implementation.
