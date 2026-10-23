# ADR-0022: Four-direction rectangular Fill

Status: **Accepted design with three implementation clarifications; bounded implementation authorized; implementation not yet started (2026-10-23)**.
[Design approval and governing clarifications](../handoffs/2026-10-23-four-direction-fill-design-review.md).
Date: 2026-10-23.
[Review packet and acceptance matrix](../handoffs/2026-10-23-four-direction-fill-design.md).
Predecessors: [rectangular copy](0017-rectangular-clipboard-and-formula-copy.md),
[structured bindings](0020-worksheet-structured-output-bindings.md),
[accepted aggregate implementation](../handoffs/2026-10-23-spreadsheet-aggregate-final-review.md).

## Purpose and scope

Add command-driven Fill Up, Down, Left and Right over an existing ordinary worksheet
selection. Reuse Rust-owned formula duplication and one atomic workbook transaction.
Do not implement series generation, patterns, dates, mouse fill handles, Cut,
formatting, history, structural edits or increased capacities. Undo/Redo is the next
separate architecture-design milestone; this operation supplies a transaction boundary,
not retained history or a promise that existing captures alone implement Undo.

## Source edge and destination correspondence

Normalize inclusive selection R=(top,left,bottom,right). Selection anchor/active
orientation does not change correspondence. Source and destinations are disjoint.

| Direction | Source for destination (r,c) | Destinations |
| --- | --- | --- |
| Down | (top,c) | top+1 through bottom, all selected columns |
| Up | (bottom,c) | top through bottom-1, all selected columns |
| Right | (r,left) | left+1 through right, all selected rows |
| Left | (r,right) | left through right-1, all selected rows |

Capture the entire source edge and authoritative bindings from one pre-operation
state S. Compute every destination directly from its aligned source in S, never
from an intermediate destination. Plan destination edits in canonical row-major
order for deterministic validation/diagnostics; semantic results do not depend on
that order. Source-edge contents and bindings remain unchanged.

Validate request shape, direction, exact expected state and coordinate bounds first.
A one-row Up/Down or one-column Left/Right operation has no destinations and returns
unchanged, including a single cell. It does not enumerate/capture cells, allocate
identities, alter revision/checkpoint/dirty state or require the work-area quota.
It still rejects stale authority and invalid coordinates. Opposite-axis fills remain
real operations: one row may Fill Left/Right; one column may Fill Up/Down.

## Content and identity semantics

- Blank or unallocated source clears destination contents; preserve any existing
  destination CellId and normal blank-marker semantics. Clearing unallocated blank
  destinations must not materialize cells solely for Fill.
- Decimal source preserves coefficient, scale and sign, including signed zero.
- Literal text copies exact stored text, including numeric/formula-looking strings
  and leading apostrophes, through typed text edits; never reclassify it as entry text.
- Rejected formula drafts copy original bytes and non-executable status without
  translation, repair, resubmission or evaluation.
- Valid formulas duplicate definitions, not computed values. A formula evaluating
  to an error is still valid and copies normally if its bindings can be represented.

Existing destination CellIds remain. New destinations use normal allocation rules;
source identities are never imported as destination identities. Ordinary scalar
references translate from their bound target's current coordinate in S using existing
relative/absolute flags, not fresh lookup of historical formula spelling. Formula
origin is its actual position in S, including when it was moved. Retired/unplaced
scalar targets reject the whole fill; do not repair them by source-name lookup.

Translate range endpoints independently for SUM/MIN/MAX/AVERAGE/COUNT, preserving
absolute-axis flags, endpoint orientation, spelling outside reference spans and
current reviewed geometry. Create destination range bindings with the existing
selection identity/admission policy. Do not substitute one operation for another.
An out-of-bounds translated operand rejects everything; no clipping or partial fill.

Example: A3 contains `=B3+$C$1`. Fill Up over A1:A3 produces A1 `=B1+$C$1`,
A2 `=B2+$C$1`, with A3 unchanged. For a moved scalar target, the effective-coordinate
rules in ADR-0017 apply in every direction, including negative offsets.

In the existing opt-in structured-binding proof, reuse server-owned same-document
duplication: preserve DocumentId/ProducerId/OutputId and expected type; translate
only ordinary A1 operands. Do not resolve names again, accept client binding IDs or
use OS clipboard serialization. Renamed/retired/unavailable outputs retain the
existing binding/fault policies. Preserve experiment quotas and non-savability.
Projected records/header rows are not worksheet positions or sources/destinations.
A logical selection spanning an inserted display projection fills only ordinary
coordinates, without touching projection order, keys, producer data or presentation.

## Admission, transaction and cancellation

For operations with destinations, selected area including source edge must be at
most **256 positions**, checked with bounded/checked arithmetic before enumeration.
Do not count only occupied cells, chunk larger fills, or silently reduce the rectangle.
This reuses the existing bounded batch scale and does not claim large-sheet fill.
Retain the workbook's live/allocated, selection/edge, structured and memory limits.

Reuse existing copy-planning ceilings: 128 KiB aggregate source/text work,
16,384 token work and 8,192 reference occurrences; 4,096 UTF-8 bytes per stored
text/formula remain. Count each source analysis and every destination transform,
including repeated sources, against aggregate work bounds. Rejected-draft bytes and
text escape overhead use the same conservative accounting as typed clipboard edits.
No native/TSV buffers are needed and no OS clipboard data is read or changed.

Factor trusted duplication into shared internal planning helpers; do not implement
Fill by repeatedly calling mutating duplicate/paste operations or constructing a
second evaluator. Helpers return typed candidate edits and account transient source,
translation and edit storage through the existing allocation domain. Reserve bounded
scratch before allocating; retain it for the lifetime of owned scratch/captures.
Release all transient ownership on success, rejection and cancellation. Poll existing
cancellation/work budgets during capture and per-destination planning, then use one
existing cancel-aware atomic apply. Check cancellation even for no-op completion.
Do not weaken quotas or reuse the structured helper's current non-cancel-aware apply
without adapting the shared planner/commit boundary.

Plan, validate and install once. A failure in the last destination, stale state,
cycle, quota failure, unavailable target or cancellation leaves contents, roots,
identities/ledgers, revision, path, saved checkpoint and dirty state unchanged.
Successful definition changes create one workbook transition and one private-session
revision update. Source publication is a separate state transition and cannot mix
contexts into Fill. Existing captures remain immutable.

No-op detection compares the intended typed definitions, including source spelling,
blank kind, bound scalar/output targets and range kind/geometry, before introducing
fresh destination identities. Equal displayed numeric values alone do not qualify.
A fully unchanged Fill leaves revision/dirty state/checkpoint unchanged. Repeating
an identical Fill should be a no-op; blank-versus-explicit-zero is a real change.
Do not accidentally allocate fresh SelectionIds merely to discover an unchanged fill.
A changed fill obeys existing saved-state rules; do not make content equality with an
older saved file imply checkpoint restoration. Undo's dirty-state policy is deferred.

## Private session and desktop

Propose one additive private request operation `FILL = 31`, using existing normalized
`first`/`last`, `revision` and `computational_state`, plus new typed field
`FillDirection fill_direction = 28` with UNSPECIFIED=0, UP=1, DOWN=2, LEFT=3, RIGHT=4.
These are proposed next available numbers, not source changes. Reject missing/unknown
direction, malformed rectangle, stale revision/context and invalid request versions.
The client supplies positions/direction, never formula text, trusted bindings or a
loop of cell edits. Response uses existing revision/dirty/checkpoint/error fields;
no clipboard or producer output payload is returned.

Retain application v4 if this additive opcode leaves existing semantics intact and
old peers reject the unsupported operation before mutation. Verify that rejection;
if inspection/testing finds silent interpretation or a genuinely incompatible wire
change, stop for version-cut review. No adapter, fallback, auto-replay or hosted v1
change. WB5 and CB2 representations are unchanged. In-flight cancellation follows
the existing session rule: a dispatched result is drained and committed state
acknowledged; do not claim a cancelled caller implies a rolled-back dispatched Fill.

Add an accessible Fill menu with Up/Down/Left/Right commands, selection-size limit
and actionable failure text. Proposed shortcuts when the ordinary grid owns focus:
primary+D Down, primary+Shift+D Up, primary+R Right, primary+Shift+R Left; primary
is Ctrl on Windows/Linux and Command on macOS. No shortcuts are intercepted inside
inline/formula-bar editors, projected rows or unrelated controls. Scope them to the
grid and suppress handled text input as established for clipboard commands.

Disable Fill while any cell editor is active, during pending mutation/drag or when
selection is projected/read-only; do not commit or discard drafts as a hidden step.
The user finishes/cancels the edit before Fill. Disabled menu actions explain the
reason. Keep selection geometry, anchor/active orientation, scroll and logical active
cell on success/no-op/failure; refresh values without selecting another region.
Single-axis no-ops remain harmless; over-limit real fills report the bound and retain
selection. Old formula-bar/selection timing protections still apply: use the intended
selection/state captured at dispatch and never retarget delayed commands.

## Validation and stop point

The linked packet specifies adversarial Rust, actual session and desktop coverage.
Run normal managed CI, strict all-feature checks and real Native AOT on Windows,
Linux/WSLg and fresh native Mac/APFS, with matching manifests, real Rust child,
all existing eleven fields plus a distinct Fill acceptance field, peers and zero
charge. No new recurring manual checklist. Preserve the Windows clipboard issue
and earlier platform limitations without speculative fixes or weakened tests.

The parent approves this design with three governing clarifications: no-op comparison
before range/graph identity allocation; blank clears preserve destination CellIds
without materializing absent blanks (including COUNT/AVERAGE regressions); and
pre-install cancellation rolls back while post-install caller cancellation requires
committed-state acknowledgement. Bounded implementation is authorized without
another design cycle; no passing implementation evidence is claimed.
After approved implementation/validation, stop for acceptance; do not start Undo/Redo
code. Follow with a separate transaction-history architecture design that treats
one Fill as one undoable action.
