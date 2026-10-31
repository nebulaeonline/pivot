# ADR-0033: Worksheet filtering

Date: 2026-10-31.
Status: **Design approved with clarifications by parent review (2026-10-31).
Implementation requires separate owner authorization.**

The parent accepts persisted typed definitions, derived whole-row visibility,
coherent coordinate mapping, automatic refresh, structural NeedsReview, full-rectangle
bulk actions, WB12/private v11 and the resource/validation gates. CB5 and hosted v1
remain unchanged. Three mandatory acceptance invariants apply:

1. **Operational versus persisted suspension:** structural NeedsReview is persisted
   within a successful structural transaction. A failed automatic refresh is transient
   presentation state: retain the definition and do not independently mutate/dirty
   the workbook. Reapply retries that definition without repair or rebinding.
2. **Exact-operation confirmation:** qualify the complete source/destination footprint,
   operation, hidden-row counts, addresses, revision, filter/view generations and
   clipboard payload as applicable. Any relevant change invalidates confirmation;
   there is no generic/reusable consent or implicit visible-only packing.
3. **Coherent visibility:** pending and failed refresh show explicitly unfiltered/
   suspended all-row presentation. Operations requiring trustworthy hidden-row counts
   remain blocked in both states. Publish map, viewport, actual addresses, values,
   styles and counts coherently against the same workbook/filter generations.

These are future implementation requirements, not tested filtering behavior.
Stop for separate owner implementation authorization under the standing cadence.

The owner authorized this design after committing ADR-0032 (`fbe50a1`). Sorting
remains accepted and closed. ADR-0023 remains parked at G3. The owner reports build
tools updated on the Windows x64 and ARM64 VMs; exact compiler/toolset versions must
be recorded by future validation, not inferred from the update notice. No platform
commands, toolchain probes, builds or tests are part of this design work.

## Decision and scope

Provide one configured filter rectangle per ordinary worksheet, with criteria on
one or more columns. Filtering controls **whole worksheet row visibility** within
the rectangle's data rows. Rows outside that interval and an explicitly excluded
header remain visible. Every cell retains its original coordinate, CellId, graph
identity, binding, value, source, style and fault provenance. Filtering does not
change calculation membership: `SUM(A1:A100)` still includes hidden rows.

Persist the filter definition as document-owned metadata; derive visibility as
revision-qualified presentation state. Filtering does not add graph predicates,
selection identities, filtered calculation ranges or synthetic worksheet records.
It does not implement manual row hiding, multiple tables, structured tables,
column hiding, sorting changes, deletion, Undo/Redo or a general query language.

The UI must explain that hiding a row also hides cells outside the chosen columns.
For example, filtering A1:C100 can hide D20 on the same row. Show the exact rectangle
and that consequence before initial Apply. Offer explicit adjacent-column expansion
using the accepted sorting bounds suggestion; never silently expand the rectangle.
This suggestion is guidance, not table detection or a claim that unrelated data is safe.

Header exclusion is an explicit checkbox, initially on and visibly selected; no
content-based guessing. With it off, every row in the rectangle is data. A single
cell requires entering an explicit rectangle. Empty ranges and zero/one data row
are valid. A configured filter with no criteria hides no rows.

## State ownership and alternatives

Pure client state would avoid a file change, but different clients and Save/reopen
could lose the intended filter. Persisting a visibility bitmap would save a result
whose formula inputs may have changed. Persist a bounded typed definition instead:
WorksheetId, rectangle, header flag, per-column predicates and a NeedsReview flag.
Selection/scroll and cached visibility remain session presentation state.

Applying or clearing a definition, changing its header/range/criteria, or setting
NeedsReview changes document metadata, revision and dirty state, atomically. These
changes leave graph revision/computational state untouched. Identical canonical
definitions are no-ops. Reapplying an unchanged definition changes only the derived
view generation if needed; it does not dirty the document or manufacture identities.
Old workbook captures and prepared exports retain the metadata they captured.

Use a **WB12** experimental native format for the new metadata, retaining the current
64 MiB file ceiling and all existing admission checks. Reject obsolete versions;
no WB11 compatibility reader/migration is assigned. CB5 and hosted protocol v1 remain
unchanged: copying cells does not copy filter definitions. Propose private session
**v11** for definition, evaluation and mapped-viewport contracts. These are proposals,
not changes already made to codecs or protocol.

On open, validate the full definition before installing the document. Malformed,
duplicate/out-of-range criteria or excessive lengths refuse Open atomically. For a
valid enabled definition, derive visibility from the newly opened content. A derived
evaluation failure does not make valid workbook content inaccessible: open with all
rows visible and a persistent-on-screen filter error, retaining criteria for retry.
NeedsReview definitions open with all rows visible and require explicit confirmation.
Save stores definitions/NeedsReview, never transient maps, pending tasks or failures.

## Typed criteria

All active column conditions combine with AND. Each column permits up to two simple
predicates, combined by a visible AND/OR choice. No nested expressions or executable
formula criteria. Clearing a column removes its condition; Clear Filter removes the
whole definition and reveals all rows. The initial editor uses typed operators,
not an unbounded distinct-value checklist.

| Predicate | Meaning |
| --- | --- |
| Decimal equals/not-equals/less/less-or-equal/greater/greater-or-equal | Exact Decimal96 comparison; other kinds do not match |
| Text equals/not-equals/less/less-or-equal/greater/greater-or-equal | Exact case-sensitive ordinal Unicode comparison; other kinds do not match |
| Text contains/starts with/ends with | Case-sensitive literal text, no wildcards, regex, normalization or locale rules |
| Boolean equals/not-equals | Typed true/false comparison; other kinds do not match |
| Is Blank / Is not Blank | Blank alone / every non-Blank result, including empty String and completed errors |
| Is Error / Is not Error | Any completed cell calculation error / every successful result, including Blank |

The editor labels typed negatives explicitly, e.g. **Text is not equal to**, since
they do not implicitly include Blank, Decimal or Boolean. Use OR Is Blank when
that is intended. Error matching is by category (any completed error) for this
milestone; diagnostic messages are not compared. Other predicates never match errors.
No mixed-kind ordering is borrowed from sorting to coerce comparison operands.

Literal entry has an explicit type. Text is the default literal type, including
numeric-looking input; choosing Decimal invokes the existing invariant Decimal96
parser, with overflow and malformed input refused. Boolean uses a true/false choice.
Text `001234` never becomes Decimal `1234`, loses zeros, or matches that Decimal.
Empty String is a legitimate Text operand and differs from Blank. Empty contains/
starts/ends operands match every String, including empty String, and never Blank.
Text limits follow existing 4,096-byte stored String limits; no locale/date inference.

Compute relevant formula results from one immutable workbook capture. Completed
errors such as division by zero or `#REF!` are criterion values. Incomplete captures,
invariant violations, cancellation and exhausted evaluation admission are operational
failures. Reuse the accepted sorting distinction, without silently converting such
failures to nonmatching rows.

## Visibility, coordinates and viewport

Rust derives an immutable visible-row map qualified by document generation,
WorksheetId, workbook revision, computational state, definition generation and view
generation. Header/data counts are separate; status reads, for example,
**Filter A1:C100: 23 of 99 data rows shown**. Row labels keep actual worksheet row
numbers, with a gap indicator where rows are omitted. Header filter controls are
overlays on the original row, not new cells or a frozen-header implementation.
When the header scrolls away, a worksheet filter bar keeps criteria access and Clear
Filter available. A no-header filter uses that bar and column letters.

Scrolling, wheel/page navigation, hit-testing, drag autoscroll, reveal, inline editor
placement, selection painting and keyboard movement must all use the same mapping.
The first visible-row ordinal and worksheet row number are distinct fields/types.
Never pass a visual ordinal as a worksheet address. Home/End and Ctrl navigation
choose visible worksheet positions under existing axis conventions. Address/formula
displays and references always use original coordinates.

Selection remains one logical worksheet rectangle, with its original coordinates;
it is not a new discontiguous computational range. Its visible highlight can contain
gaps. Show selected hidden-row count beside the range. If refresh hides the active
cell, collapse selection to the next visible row at/after it, otherwise the previous
visible row, retaining the column. If no worksheet rows are visible, keep a neutral
selection with editing disabled and filter controls enabled. An explicit Go To for
a hidden address offers Clear Filter and reveal or Cancel; it never edits a hidden
cell through a visible surrogate. Clearing the filter retains coordinates when valid.

Viewport reading needs a new mapped-row request: accept a visible ordinal/count and
return actual row addresses plus cells/styles for at most the existing displayed
row/column capacity. Do not request the bounding rectangle between distant visible
rows, or issue a separate calculation capture per displayed row. One snapshot and
one response must qualify the map and cells/styles together. A stale response is
discarded whole. Existing rectangular reads retain their unchanged address semantics.

The experimental projection proof's `ProjectionMap.cs` already distinguishes visual
rows from ordinary positions, but it interleaves synthetic read-only records. Its
producer/key semantics are not the ordinary filter model. Introduce a narrow ordinary
row-visibility mapping path; do not enable, generalize or merge the projection proof.

## Editing and other commands

Single-cell editing operates at a visible cell's real coordinate and commits normally.
Formula operands can still refer to hidden cells. Draft edits do not trigger filtering;
only successful commits do. Formula calculations and status never pretend hidden cells
are absent. Editing may hide the edited row after a successful visibility refresh;
the selection rule above then chooses a visible position.

Initial bulk operations retain their existing **logical rectangle** semantics:

| Command | Proposed initial behavior when its addressed rows include filtered-out rows |
| --- | --- |
| Native Copy / plain-text Copy | Confirm copying the full rectangle, including the displayed count of hidden rows; default Cancel |
| Paste, Clear, Move, Fill | Confirm the full addressed footprint, including hidden destination/source rows; preserve existing translation, limits and atomicity |
| Apply/reset formatting, worksheet-default changes | Preserve existing property semantics across all addressed positions; confirm affected hidden rows |
| CSV export of selection/sheet | Confirm exporting all addressed rows, including hidden ones; existing typed/quotation/error rules stay unchanged |
| Sort | Sort the explicit full rectangle, including hidden row segments, only after the Sort dialog confirms their count; refresh filtering afterward |
| Row/column insertion/deletion | Confirm the actual whole-axis band and any hidden rows; then suspend the affected worksheet's filter for review as below |

The confirmation names the exact action/range and includes an explicit
**Include N filtered-out rows** action. It is not consent to later commands. Counts,
revision, filter generation, clipboard read and selection must still match at dispatch;
stale confirmation refuses with no replay. Showing a warning only after the operation
is insufficient. Ordinary commands whose footprint contains no hidden rows keep
their current workflow. Programmatic coordinate APIs retain explicit rectangular
semantics; they do not acquire an implicit visible-only mode.

For Paste, derive the destination rectangle from the actual immutable clipboard
payload before asking, not merely the current source selection. Move covers both
source and destination; Fill covers its source and written footprint. Sort, formatting
and export qualify their full addressed extent, including whole-sheet defaults when
applicable. Count the union of affected hidden worksheet rows per worksheet without
double-counting overlapping footprints. Bind confirmation to those exact footprints,
the immutable payload/clipboard identity where relevant, operation and generations;
revalidate immediately before dispatch. Changed payload, address, hidden-row count or
qualifying state requires a fresh review, never an automatic replay.

The parent accepts this first-milestone limitation: no packed visible-row clipboard,
visible-only Fill/Paste, arbitrary multi-area edits or filtered-only aggregates/export.
Packing formulas into different row spacings requires a separate reference-translation
and payload contract. Do not silently pack a native copy, discard hidden rows or
change formulas into literal values. This interaction boundary is approved for
implementation after separate owner authorization. A future visible-only feature can be
scoped separately from basic row filtering.

## Refresh and structural invalidation

After a successful ordinary computational mutation, including edits on other sheets
and sorting, invalidate the current derived map and automatically recompute from one
new immutable capture. A conservative workbook-wide revision invalidation is adequate
initially; do not build a dependency-tracking subsystem just to prune refreshes.
Formatting-only edits do not require new key evaluation when computational state and
definition match; requalify the cached map to the new document revision.

During recomputation show **Refreshing filter — all rows shown** and use a qualified
unfiltered mapping. Never combine a stale hidden-row mask with newly edited values.
Successful refresh atomically installs map/frame and counts. Pending input, viewport
and clipboard actions carry view/interaction generations and cannot land on rows
replaced by a newer map. Keep one running evaluation plus at most one coalesced latest
request; cancel/discard obsolete work. Do not hide a cell while its uncommitted editor
draft is active; defer view publication or commit/cancel through normal editor rules.
While refresh is pending or has failed, disable bulk clipboard/edit/export/sort
actions requiring a qualified hidden-row count. Explain that the user can wait for
pending refresh, explicitly Reapply after failure, or Clear Filter. Single-cell edits
and navigation remain available. An all-row fallback is not proof that the configured
filter excludes no rows. These blocks and coherent publication are acceptance
invariants, not optional interface guidance.

Only the active ordinary worksheet needs a retained display map. Inactive sheet
definitions are still persisted and structurally invalidated, but their derived
maps can be evicted. On activation, evaluate against current state before enabling
qualified bulk actions. A headless rectangular read need not build a display map.
Do not launch sixteen parallel filter calculations after every workbook edit.

Automatic refresh failure leaves the committed content intact, all rows visible and
an explicit **Filter suspended: reason — Reapply / Clear Filter** state. It does not
roll back an accepted edit, pretend a failed calculation succeeded or retain a stale
mask as current. Reapply retries only by explicit user action after an operational
failure. Creating/replacing criteria is different: compute its candidate map first,
then publish definition/map together; failure preserves the previous definition and
visibility. The transient failure itself neither changes the saved definition nor
increments document revision/dirty state; only the preceding successful content
mutation has its normal document effect. Reapply does not repair/rebind criteria.
Clear Filter is available without recalculation and cancels pending work.

Any successful row/column insertion or deletion on the filtered worksheet atomically
sets its persisted NeedsReview flag and reveals all rows. Preserve the former range
and predicates as a disabled editable draft, visibly labeled with their former column
letters; require explicit range/key review and Apply before enabling again. Do not
silently retarget criteria to a different column occupant, reuse coordinate-derived
AxisIds as stable column identities, or invent filter-boundary affinity in this ADR.
Even a band outside the rectangle uses this conservative rule. Changes on another
worksheet trigger ordinary value refresh, not a range review. No-op/refused structure
commands do not suspend anything. Sorting keeps the same rectangle/criteria geometry
and triggers ordinary refresh; it never sorts only the visible subset implicitly.

## Bounded implementation proposal

The persistent workbook owns filter metadata; a session owns its charged derived
map/cache. Store at most one definition per worksheet, at most one condition per
selected column (1,024), and two predicates per condition. Propose a 1 MiB **aggregate
filter-definition byte ceiling per workbook**, within existing domain/file admission,
not extra uncharged storage. Count literals, predicates and indexing overhead. Maximum
String length and all existing cell/style/identity/evaluation/frame limits remain.

A 65,536-row bitset costs 8 KiB; a u32 visible ordinal-to-row vector costs at most
256 KiB. Keep only bounded current/candidate maps, with in-flight references charged;
no per-sheet dense cell grid or height-by-key matrix. Empty/style-only/non-key cells
do not need evaluation identities. A missing/cleared key is Blank. Evaluate the
all-Blank row's combined predicate once, then visit actual positioned key results
and correct only affected rows. Group sparse key positions by row and use the Blank
baseline for absent columns. Account for all scans of unrelated stored state where
the existing indexes require them; do not claim an index the code lacks.

Evaluation cost is bounded by stored-state visits, actual captured key computation,
predicate/text work and at most 65,536 row-map entries, not 67,108,864 virtual cells.
Reuse charged captured typed results and existing evaluation admission. Use the
33,554,432-unit ceiling for filter planning/comparisons/map production; count byte
examination, scans, row writes and predicate probes with cancellation checks inside
long operations. Existing narrower subsystem gates still apply. Definition and map
publication must check full expected state and release all failed candidate charges.

Filtering cannot reduce lifetime identity counts, discard stored rows or shrink
calculation admission to the visible subset. Definition/mask resource exhaustion
must not encourage silently showing only the rows evaluated so far.

## Proposed validation and review gates

These are future implementation gates, not current measurements. Preserve numeric
workbook objectives: compact corpus with four retained roots at most 256 MiB steady,
512 MiB charged peak including candidate maps and Save/reopen; capture p95 at most
5 ms, defined recalc at most 2 s, Save/reopen at most 10 s and ordinary edit/viewport
p95 at most 100 ms. Measure first Apply and automatic refresh separately; propose
at most 1 s on admitted 65,536-cell numeric/string-key corpora. Include evaluation
and map production; do not report cached-map lookup as full filtering latency.

- Stage 1,024/10,000/65,536 positioned cells, 1,000-by-10, 4,096-by-16 and tall
  65,536-by-1 shapes. Include whole-sheet virtual ranges, sparse keys, Blank-matching
  predicates, no matches, all matches, all ten style properties and zero graph cells.
- Compare row visibility and rank/select round trips to an independent dense small
  oracle over generated criteria. Assert every CellId, coordinate, graph/selection
  identity, formula binding/source, value and explicit style assignment is unchanged
  by Apply/Clear. Check aggregates still include hidden members.
- Test exact Decimal96 edges, mixed types, `001234`, empty String versus Blank,
  Unicode/case, text limits, OR/AND, legitimate error results, cross-sheet keys and
  operational failures. Exercise work/byte ceilings, first excess, cancellation,
  stale requests, old captures, candidate cleanup and zero final runtime charges.
- Test all-matching sparse virtual rows without an area-sized allocation; long text
  and many conditions may refuse within the existing work budget. Measure charged
  steady/scratch/peak, retained captures and process RSS separately from file size.
- Exercise rapid edits, coalesced refresh, stale frames/clipboard reads, selection
  hiding, zero visible rows, scroll/hit-test/editor mapping, hidden-address navigation,
  cross-sheet invalidation and enabled/suspended/NeedsReview Save/reopen.
- Assert failed automatic refresh does not independently change definition, revision,
  dirty state or persisted metadata; Reapply does not rebind. Structural NeedsReview
  must publish only with the successful structure transaction. In both pending and
  failed states, qualified-count bulk commands stay blocked and no stale map/value
  combination is published.
- Change clipboard payload/dimensions, source/destination, revision, filter/view
  generation and hidden-row count after confirmation. Each invalidates the exact
  operation with no replay. Include overlapping Move footprints and Paste whose
  destination extends beyond the visible source selection.
- Check bulk-operation confirmations and explicit full-rectangle effects including
  hidden rows. Refusal/cancel preserves state. Verify Sort then refresh, structure
  then NeedsReview, no-op structure and independent filters on multiple worksheets.
- Test malformed/over-limit WB12 definitions, canonical metadata no-ops, saved dirty
  state, unchanged CB5 and current obsolete-version refusal. No migration suite.
- Run full headless and actual managed/Native AOT GUI acceptance on all six configured
  targets for this first visibility-mapping milestone. macOS x64 is Rosetta. Use short
  attended launchers with normal graphical-terminal consent. Record actual Windows
  compiler/toolset versions following the reported VM updates. Collect, hash-verify
  archives and prune completed caches after every batch under the standing policy.

The parent accepts document-owned definitions plus derived visibility, whole-row
scope, exact typed predicates, automatic refresh/fail-open presentation, conservative
structure suspension, explicit full-rectangle bulk actions, WB12/private v11 and
bounded definition admission with the three mandatory clarifications above. Future
six-target headless/managed/AOT gates are accepted; macOS x64 is Rosetta. Discover
actual Windows toolsets during implementation validation. Stop for separate owner
implementation authorization. No feature, format/protocol change or test run follows
from design acceptance alone; ADR-0023 remains parked at G3.
