# ADR-0024: Multiple worksheets in one workbook

Status: **Accepted — implementation accepted and milestone closed by parent review
(2026-10-28). No additional qualification round required.**

[Implementation handoff and three-platform evidence](../handoffs/2026-10-28-multiple-worksheets.md).

Deliver one usable workbook containing independently editable sheets, with formulas
connecting them. Keep the existing Rust calculation engine, session ownership and
document lifecycle. ADR-0023 remains parked; this feature does not depend on Undo/Redo
or a persistent history service.

## First implementation slice

- New workbooks start with `Sheet1`. Add a blank sheet, rename a sheet and switch tabs.
- Author local and qualified scalar references and single-sheet rectangles inside
  the existing arithmetic and aggregate functions.
- Copy/paste and Fill preserve the agreed local/qualified reference behavior.
- Save/reopen restores every sheet, its order, identities, content and bindings.

Examples:

```text
=Revenue!B2-Costs!B2
=SUM('January Sales'!B2:B20,February!B2:B20,5)
='O''Brien Costs'!$B$2
```

Sheet deletion, reordering, duplication, hiding, cross-sheet Move, external workbook
links, 3D ranges, named ranges and row/column structural edits are deferred. They
are not prerequisites for this slice. Sheet order is creation order. Existing
structured-result proof modes remain separate and single-sheet; unsupported
multi-sheet operations in those modes reject explicitly.

## Starting assumptions (before implementation)

Before this milestone, `Workbook` owned one `worksheet`, row/column axis pair, persistent state
and canonical `WorkingModel`. Its persistent positions index is keyed by `Position`.
`CellRef` already carries `WorksheetId`; selection rectangles already carry `SpaceId`.
Those identities provide useful foundations, but do not make the existing workbook
multi-sheet automatically.

Application requests and returned cells use unqualified row/column positions. The
desktop owns one worksheet view and editor. Native WB6 contains one sheet/axis pair.
Each of these boundaries must acquire explicit sheet context; changing the UI alone
would leave commands and saved bindings ambiguous.

## Ownership and identities

One workbook owns an ordered directory of sheets and **one canonical calculation
graph**, not one model/engine/process per tab. Each sheet has a stable `WorksheetId`,
display name, independent row/column axis namespaces and sheet-local position index.
Cell and graph bindings remain unique within the workbook and carry their owning
sheet. Selection `SpaceId` remains the corresponding worksheet identity.

Use persistent sheet roots/directory entries under one coherent workbook root.
Reuse existing persistent indexes and global allocation authority; do not clone
the complete workbook when switching or editing a tab. Choose the smallest internal
layout that enforces these invariants rather than introducing a provider framework.

Cell changes and metadata changes publish atomically through the workbook's exact
revision/session checks. Adding/renaming a sheet changes workbook state and dirtiness;
renaming need not change graph computation revision or rebind formulas. A rename
to the identical stored name is a no-op. Failed/stale/quota-limited operations leave
the prior root, directory and dirty state intact.

Switching tabs is client view state. It does not modify the document, graph or dirty
state. A calculation capture contains all sheet roots, directory metadata and the
common graph from one workbook generation. Later edits/renames do not alter it.
Cross-sheet dependencies, invalidation, errors and cycles use the existing core.
The calculation core receives stable nodes and selection spaces, never sheet names.

## Names and formula binding

Use nonempty NFC-normalized sheet labels, at most 64 UTF-8 bytes, with unique
Unicode default-casefolded keys after normalization. Rust owns normalization,
casefolding and validation so behavior is identical on all platforms. Preserve the
normalized display spelling. Reject control characters, leading/trailing whitespace
and `[]:*?/\\!`; apostrophes are allowed. Do not silently truncate names. Use pinned
Unicode normalization/casefold tables rather than platform-dependent comparisons.

Automatic names are the first available `SheetN`, using the same uniqueness rule.
Initial `Sheet1` is part of the clean New baseline; adding another blank sheet is a
real edit. A case-only display rename is a real metadata edit, not a duplicate name.

Unqualified `A1` binds in the formula's owning sheet. `Revenue!A1` and
`'January Sales'!A1` resolve the sheet label at authoring time, then bind stable
sheet/cell identities. Bare qualifiers use ASCII identifier syntax; other names
use single quotes, with doubled apostrophes. Dollar flags continue to apply only
to row and column axes. Retain exact UTF-8 occurrence spans and qualification
affinity for rendering/copy; names are not looked up during calculation.

`Sheet1!A1:B5` is one rectangle in Sheet1. Qualify only the first endpoint; repeated
endpoint qualifiers, `Sheet1!A1:Sheet2!B5`, 3D references and
bare ranges in arithmetic reject in this slice. Multi-argument aggregates can
combine several independently qualified rectangles. Existing range membership,
blank/text/error policies, argument order and one-final-rounding AVERAGE remain
unchanged. Unknown sheet names produce editable formula diagnostics, not silent
fallback to the active sheet. Creating that name later does not silently rebind a
stored rejected draft; explicit resubmission repairs it.

Renaming a sheet must preserve existing references. Keep original authored source
as provenance, and derive editable/display source from bound occurrences using
current sheet names, escaping as necessary. Rust renders those qualifiers; C# does
not rewrite formulas. Do not reparse/rebind the document during rename. Retain the
existing scalar-coordinate provenance behavior; this slice does not redesign how
moved cell addresses are presented. Editing/resubmitting presented text is an
explicit new bind against the current directory.

## Commands and desktop behavior

Sheet-scoped commands carry an explicit `WorksheetId`: edits, reads, moves,
copy/paste, Fill, formula resolution and calculation outputs. Metadata commands
list/add/rename sheets. Unknown or missing IDs reject; there is no mutable Rust
"active sheet" that can redirect a queued request. Document New/Open/Save/close
and immutable captures remain workbook-wide. Responses echo sheet context; cells
and diagnostics identify their owning sheet.

The initial cell-edit batch remains scoped to one sheet; formulas in it may bind
other sheets. A multi-sheet atomic edit-batch API is not required now. One exact
workbook revision guards both sheet metadata and content. Keep the current owned
Rust child and bounded serialized client request handling.

Avalonia gets a tab strip, Add control and Rename action. Each tab is keyed by ID,
not label or ordinal. Preserve selection/scroll position per sheet in client view
state. Before switching, use the existing editor commit path on the old sheet;
if commit fails, remain there. An accepted invalid formula draft can be committed
and left just as it can today. Escape retains its normal cancel behavior.

Late reads/calculations must not paint another tab: match document/session identity,
sheet ID and a client view epoch before applying a reply. A response still updates
known committed workbook state according to the existing drain/cancellation contract.
New/Open resets old view state. Reopen initially selects the first sheet; active-tab,
selection and scroll preferences are not persisted in the document in this slice.
Save captures all sheets even while the user is viewing just one.

## Clipboard and Fill

Copy creates a new formula at the destination; it does not move its source binding.
Unqualified references translate into the destination sheet. Explicitly qualified
references keep their effective qualifier, while row/column shifts obey existing
dollar flags. Fill stays within one sheet and follows the same translation rules.
Cross-sheet Move is not introduced under the guise of paste.

Retain the portable clipboard boundary: do not send runtime graph identities or
create external links. Export current effective sheet names and coordinates for
qualified occurrences. On paste, explicitly qualified labels resolve in the
destination workbook. Across workbooks this deliberately means the destination's
same-named sheet, not a live connection to the source workbook. A missing qualifier
rejects the whole formula paste atomically with a sheet-specific error. Preserve
the existing rejected-draft clipboard policy separately; do not newly execute drafts.

CB2 remains current: its existing source/effective-coordinate representation carries
this contract without additional fields. Endpoint annotations retain their existing
ordering; the destination resolves exported effective qualifiers before binding. Malformed/unsupported
native clipboard data must not silently fall back to text.

## Documents, protocol and bounds

WB7 implements the incompatible change from one sheet to an ordered sheet
directory with scoped records, axes and formula provenance. Validate sheet/name/axis
uniqueness, cell ownership, graph overlays and every referenced selection space
before installation. Preserve all sheets in one captured Save; failed Open preserves
the complete existing workbook and lifecycle state. No cached results or history
database enters the native document.

Under the current experimental policy, WB7 rejects WB1-WB6 explicitly. Before real
users depend on files,
define the compatibility commitment separately; do not turn repeated incompatibility
into a permanent product policy or add a migration subsystem to this feature.

The owned application protocol requires explicit sheet-scoped commands/responses:
version 5 replaces version 4, with incompatible peers rejected up front.
Frozen hosted v1 and daemon model semantics remain unchanged.

Enforce **16 sheets maximum** initially. Keep current row/column bounds per sheet;
live/allocated cell, token, selection, work and memory limits remain **workbook-wide**.
Do not multiply the existing budgets by the number of tabs. Charge directory/name/
axis metadata to the existing resource domain. Keep file/frame/clipboard limits.
At a limit, refuse atomically; do not drop another sheet or content to make room.

Parked experimental history adapters may explicitly reject multi-sheet inputs until
needed; they must not silently serialize only the first sheet. Keep optional builds
compiling without starting another history feasibility or integration milestone.

## Acceptance and stopping point

Automate coverage of overlapping A1 coordinates on different sheets; add/rename
uniqueness, quotas and rollback; tab-switch/editor/view-epoch behavior; qualified
scalar/range/nested aggregates; rename and moved-target identity; cross-sheet cycles
and invalidation; old captures; local/qualified clipboard translation and missing
destination qualifiers; and complete Save/reopen plus atomic failed Open.

Run scoped Rust/session regressions and appropriate desktop tab automation. This
adds GUI behavior, so targeted desktop checks are warranted; do not repeat unrelated
platform/VM qualification. Use existing automation and report actual failures.

Deliver one concise feature handoff with regression evidence. No scaling research,
new CI infrastructure, history work, sheet deletion or external links is implied.
**Implement as one milestone, then stop for parent implementation review.**

Parent clarifications: renaming must render effective qualified references (including
quoted names and escaped apostrophes) without changing bindings; copy/paste uses
those effective qualifiers. Record a future compatibility obligation before users
save valuable workbooks; WB7 does not establish perpetual incompatibility policy.
