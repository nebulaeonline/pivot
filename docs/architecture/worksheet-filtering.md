# Worksheet filtering

ADR-0033 is accepted and closed by parent implementation review (2026-10-31).
The disposition and source-qualified validation are recorded in
[the ADR](../adr/0033-worksheet-filtering.md). No further platform rerun is required.

See the [implementation handoff](../handoffs/2026-10-31-worksheet-filtering.md).
Mapped reads preserve the ordinary read path's exact submitted decimal parts,
formula binding metadata, authored text and independent cell diagnostics. Mapped
reads normally evaluate in one batch; if invalid authored formulas refuse batch planning,
independent reads from the same pinned capture retain editable errors without
losing the other viewport cells.

One filter definition per ordinary worksheet stores an explicit rectangle, header
choice and typed column conditions. Filtering hides whole worksheet rows within
the data interval, including cells outside the selected columns. It leaves cells,
coordinates, identities, scalar bindings, fixed ranges, source and formatting intact.
Calculations still include hidden cells. No implicit table inference is performed.

The Filter dialog shows the final rectangle and whole-row effect, with explicit
adjacent-column expansion. Header exclusion starts on. Column conditions combine
with AND; each has one or two predicates joined with AND/OR. Decimal96, Boolean and
case-sensitive ordinal String comparisons retain their types. Text starts/ends/
contains predicates are literal. Text input defaults to String, preserving leading
zeros; Decimal input is validated by Rust's exact entry parser. Blank, empty String
and completed errors remain distinct. Typed negatives exclude other kinds.

Definitions and structural NeedsReview persist in WB12; derived maps and operational
failures do not. Configuring/clearing a definition changes document metadata without
changing graph revision/computational state. Identical canonical definitions are
no-ops. Reapply changes presentation only. CB5 and hosted v1 remain unchanged; the
owned application session uses private v11. Obsolete native versions are rejected.

The session runs at most one cancellable filter evaluation, coalescing newer work
without retaining another queued workbook capture. The job reads one immutable
capture and its pinned definition. Missing keys use a precomputed Blank baseline;
sparse positioned key results adjust the failed-condition count for each affected
row. Evaluation does not materialize a height-by-width/key matrix or create Blank
anchors. Its visible-row vector supports rank/select and hidden-row counts. The
viewport transmits an 8 KiB row mask plus actual page addresses, values and styles.

Automatic refresh follows committed computational changes, including cross-sheet
edits and sorting. Formatting-only changes requalify a cached map without evaluating
keys again. Pending/failed refresh shows explicitly unfiltered rows and blocks
operations needing unknown hidden-row counts. Operational failure retains criteria
and does not independently dirty the workbook. Reapply explicitly retries; Clear
Filter cancels work without recalculation. Old job results are discarded by capture
qualification. Closing joins the owned job before disposing the runtime.

The UI publishes matching map/frame/counts together, retains actual row labels and
routes scrolling, navigation, hit-testing and editor placement through the map.
View-generation changes cancel pending gestures and clipboard interactions. If a
new map would hide an uncommitted draft, publication waits for commit/cancel. A
hidden active coordinate moves to the next/previous visible row. With no visible
rows the grid is neutral and editing is disabled; filter recovery remains available.

Bulk commands keep their logical rectangular footprints. A preview derives Paste
bounds from the actual immutable payload, includes both Move endpoints, and counts
the union of affected hidden rows. Structural footprints include the shifted tail,
not merely the inserted/deleted band. One-use confirmation binds operation bytes,
source/destination, expected state, view generation and count. The desktop rechecks
interaction state and clipboard payload before dispatch. There is no reusable
consent, automatic replay or visible-only packing. Single-cell editing remains
available in the unfiltered fallback; footprints outside filter data rows have a
provable zero hidden-row count.

Actual structural mutations on the filtered sheet atomically persist NeedsReview
and reveal all rows. The dialog labels the former range/column letters and requires
an explicit range review before Apply; Reapply cannot rebind them. Sorting retains
filter geometry and triggers ordinary refresh. Refused/no-op structure leaves the
definition unchanged.

Definitions share the existing runtime/workbook budgets and have a 1 MiB aggregate
canonical storage ceiling: fixed conservative structural units plus literal UTF-8
bytes. Actual allocation capacities are charged separately. This avoids allocator
spare capacity changing admission on Save/reopen. Planning/predicates/maps use the
existing 33,554,432-unit ceiling, with cancellation during long text comparisons.
Other capacity, style, transport and evaluation limits remain unchanged.

Routine responses carry a small geometry/column summary. The full criteria are
retrieved separately when opening the editor, keeping near-quota definitions from
crowding out viewport cells within the existing frame limit. Candidate definitions,
retained maps/status, operation tickets and response construction have explicit
admission. Oversized read responses refuse without terminating the workbook session.
The experimental structured-output projection path remains separate.
