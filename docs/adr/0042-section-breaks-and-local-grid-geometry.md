# ADR-0042: Section Breaks and Local Grid Geometry

Date: 2026-11-02.
Status: **Design accepted by final review, 2026-11-02. No feature implementation authorized or performed.**

## Purpose and decision

Specialize the third scope of accepted [ADR-0039](0039-named-windows-and-document-composition.md)
after accepted ADR-0040/0041. An invoice must support different header, line-item
and payment/totals geometry without merged cells, spacer-cell workarounds or altered
calculation authority. This milestone adds local Grid geometry and its editor map;
it does not add Text, Window placement, pagination or PDF.

Use independent row bands selecting column-width profiles and column bands
selecting row-height profiles. Either break type works by itself. Horizontal breaks
are the common document-design route; vertical breaks are optional. With only
horizontal breaks, sections are full-width row bands. With only vertical breaks,
sections are full-height column bands. With both, a section is their intersection.
These intersections are derived for navigation/selection, not independently stored
or editable two-dimensional geometry regions.

**Selection rule:** individual cells may be selected only within one section.
If a requested cell rectangle crosses **either** a horizontal **or** vertical
section boundary, select every touched section in full. Crossing both types is
not required. The same rule applies to drag, Shift-selection, keyboard extension,
address-box selection and selection-based operation previews.

A Section Break never changes a cell address, CellId, calculation node, stored
SelectionId membership, value, formatting or page-break policy. Pure geometry edits
advance workbook state W, not calculation revision C. Section-aware editor
selection does not reinterpret existing formula or producer range descriptors.

The detailed [geometry and selection contract](../architecture/section-grid-geometry.md)
is normative for mapping, transforms, filters, frames and admission. The
[design handoff](../handoffs/2026-11-02-section-grid-geometry-design.md) identifies
review decisions and qualification gates.

## Current implementation seams

`Pivot.Avalonia/WorksheetView.cs` currently assumes 94-DIP columns and 26-DIP rows
in drawing, hit testing, headers, scrolling, editor placement and selection borders.
`WorksheetFiltering.cs` maps one visible-row order for those uniform cells. Replacing
only `CellBounds` would leave conflicting maps; all these consumers must move together.

`pivot-workbook/src/axes.rs` supplies compact, non-reused authored axis identities.
`structure.rs` already stages structural edits; geometry transforms join that
candidate. Formatting remains its independent spatial property layer. Bound Grid
locations retain the ADR-0041 producer/item/field/capture qualification and own no
editable cell identities. No new evaluator or copied bound-value cache is introduced.

## Geometry representation and map

A Grid scope owns two ordered one-dimensional partitions and immutable profiles.
A horizontal break is immediately before a row and starts a new column-width
profile. A vertical break is immediately before a column and starts a new row-height
profile. The implicit first bands always exist. A new break shares a snapshot of
the preceding effective profile; editing either band uses copy-on-write. Removing
a break explicitly extends the preceding profile into the following section.
Equal-looking adjacent sections remain distinct until explicitly removed, because
their boundary affects selection.

Use signed checked 64-bit geometry coordinates in **1/64-point units** (72 points
per inch). Axis sizes are integer units, at least 1 point and at most 4,096 points;
zero, negative, overflow, NaN and infinity refuse. Scope defaults match today's
appearance: width 70.5 points/94 DIP and height 19.5 points/26 DIP at 100% zoom.
Zoom/DPI are view transforms, never saved cell dimensions. Profiles use sparse
constant-size runs and defaults; declared dimensions allocate no per-cell geometry.

For cell `(r,c)`, let `H(r)` be its row band and `V(c)` its column band:

- `x(r,c) = sum(width[H(r), k], k < c)`.
- `y(r,c) = sum(height[V(c), j], j < r)`.
- The half-open rectangle has that origin and the selected width/height.

No two distinct cells may have overlapping positive-area interiors, including
Blank/unmaterialized positions. Shared edges are allowed. Gaps are background and
never implied cells. A failure identifies conflicting logical cells and profiles;
paint order, silent clipping, zero sizes and data occupancy cannot make it valid.

Validation uses constant-size topology tiles, not a loop over all cells. Partition
each row-band/column-band intersection further at the applicable width/height run
boundaries. Each resulting tile exactly tiles its physical rectangle with cells.
A sweep over tile rectangles therefore detects all cell-interior overlaps. Ordered
half-open events remove ending rectangles before adding starting rectangles.
The same immutable plan supplies cell rectangles, inverse hits, section outlines
and future layout consumers. Single-orientation geometry cannot create overlaps
when sizes are positive; retain that inexpensive common-case path.

## Authored axes and bound presentation

Worksheet and unbound-Window breaks follow existing authored AxisIds, not rank-derived
identities. Size runs are qualified by that scope's row/column namespace. Insertions,
deletions, clipping and profile changes stage with the structural candidate; a
failure publishes neither data nor partial geometry.

Bound Window geometry is **Window-owned positional presentation**. Its breaks and
size runs use explicitly tagged local ordinals, never fake authored AxisIds, source
CellIds or producer keys. Reordering records does not move a positional break with
a record. Labels explain this. The view's data/formula authority still follows
ADR-0041 source identities. It has local geometry controls despite read-only values;
source values, formatting, filtering and structure remain source-owned.

Controls and rulers must label these settings “Positional presentation” and distinguish
them from source-field formatting. On a meaningful schema/order change (including
field addition, removal or reorder and declared record-order changes), show a
persistent “Review positional layout” notice with the changed source generation.
Offer explicit Review, Keep positional layout and Reset presentation actions. Reset
previews removal of local breaks/sizes to scope defaults and changes no source data
or formatting; review/keep acknowledges the current source generation. Another
meaningful change renews the notice. Ordinary value-only refresh does not. This
notice never invalidates or delays producer publication or silently remaps settings.

Validate a bound map against the existing maximum finite Grid envelope, separately
from the current producer extent. Out-of-extent metadata is dormant, not permission
to read future/missing cells. Growth/shrink/refresh changes the displayed intersection
without creating cell identities or making an otherwise valid refresh fail because
of view geometry. No source rows means no selectable data cells, even if the
existing empty-view placeholder is drawn. Rebinding still creates a new Window.

## Section selection and operations

Represent client selection as `Cells` within one section or `Sections` covering
complete touched sections. Given a logical anchor/active rectangle, first find its
intersected row bands and column bands. If exactly one of each is touched, retain
the requested rectangle. Otherwise expand to the first/last touched band edges in
both axes, clipped to the active scope extent. This is deterministic and idempotent.
Selection stays client-local and allocates no computational SelectionId.

The expanded result is a logical rectangle because both partitions are ordered;
its painted selection may be a union of physical spans with gaps. Never use a
physical bounding box to select intervening cells. Show full logical bounds,
section count and cell count before an operation; a small gesture can encompass a
large tail section. Do not enumerate or manufacture those cells merely to select.

Column/row rulers describe the active section's profiles and identify their scope.
A column-header selection spans its active row section; a row-header selection spans
its active column section. Extension across the other boundary invokes whole-section
closure. Explicit whole-scope axis selection also uses closure. With no breaks these
behaviors retain today's ordinary row/column selection.

Selection-derived Format, Clear, Fill, Move, Copy and operation previews use the
actual expanded bounds and existing admission/hidden-row policies. Formatting still
overwrites selected properties throughout the complete selected range, per ADR-0030.
Fill uses existing immutable logical edge semantics across the complete result;
it does not restart at each section. A large section selection does not raise edit,
clipboard, work or byte limits.

Whenever section closure enlarged the original requested rectangle, destructive or
bulk-changing operations require explicit confirmation for that invocation: Clear,
Fill, Paste, Move and Format, including visible-only variants. Retain the original
request and effective closure; show the operation, both bounds, added-position count
and hidden-row policy in the confirmation. Selecting or releasing the drag is not
confirmation. Cancel makes no change. Confirmation is specific to the scope,
expected state, geometry/visibility/producer capture, effective range, operation
and payload; changes invalidate it and require a fresh preview and confirmation.
No remembered blanket approval or cell-count threshold bypasses this requirement.
Copy and navigation do not mutate content and do not require it. Exact computational
ranges and explicit whole-section requests that were not enlarged retain existing
operation safeguards. Confirmation never overrides admission or paste-shape rules.

Paste must not silently enlarge its write footprint. A cell paste footprint wholly
inside one section follows existing rules. A cross-section paste requires an
explicit whole-section selection with an exactly matching supported payload shape;
otherwise refuse with the actual footprint and section bounds. Do not broadcast,
pad or discard values to make it fit. Window native clipboard refusals remain.

Typed formulas and stored source/filter/range definitions are data contracts, not
editor cell selections; their logical ranges remain exact, even across sections.
Programmatic data APIs keep explicit logical-coordinate semantics. Section closure
is not a new data-access permission boundary. Interactive selection-based commands
must not bypass it by submitting their pre-expansion pointer rectangle.

## Editor usability and visibility

Use labels such as “New column-width section above row…” and “New row-height
section left of column…” so the effect is clear. Creation/removal and numeric sizing
have a preview naming affected bands and bounds. Default interior-divider dragging
adjusts the two adjacent sizes together, visibly preserving the local span. Moving
an aligned section seam is an explicit batch across its perpendicular profiles;
no hidden compensating size edits. Arbitrary numeric sizes remain available and
are validated before publication.

The invoice workflows below must work through these normal tools. Requiring users
to discover compensating column numbers, add spacer cells or split an ordinary
invoice into unrelated workbooks does not satisfy the usability gate.

Hit testing distinguishes cell, section chrome, header, gap and outside. A gap
click preserves selection; a drag over a gap retains its last valid endpoint and
shown closure. Releasing there commits that visible selection. Keyboard navigation
uses logical visible rows/columns. An active gesture is cancelled on map change.
If new geometry/extent would widen the current selection, require explicit reselection
before enabling content commands; no background expansion may authorize a wider edit.

Filtering keeps its existing logical inclusion rules. Compact visible rows using
the same height profiles and visibility capture. Validate that derived map too.
The usual aligned-seam invoice profiles remain valid under arbitrary row hiding.
For a nonaligned map whose compaction collides, an attempted geometry/filter command
refuses with a witness. If later data changes make an existing filter's view invalid,
suspend that filter presentation with an explicit reason and show the validated
unfiltered map; retain its criteria, block visible-only operations, and require
explicit revalidation. Never block a source-data commit, use stale values, or display
an overlapping map. See the contract for coherent frame qualification.

## Normative invoice and selection examples

Use an unbound Grid Window of 36 rows/eight columns. Row breaks are before 5 and 29;
an optional column break before G exercises the less common vertical case.
Widths below are points and each A:F span is 480 points; A:H is 576 points.

| Row section | A | B | C | D | E | F | G | H |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| Header 1–4 | 160 | 32 | 32 | 64 | 64 | 128 | 60 | 36 |
| Line items 5–28 | 240 | 48 | 48 | 48 | 48 | 48 | 60 | 36 |
| Totals/payment 29–36 | 96 | 96 | 96 | 64 | 64 | 64 | 60 | 36 |

Left-side row heights are 18 points. On the right, rows 1/29 are 36 points,
rows 2/3/30/31 are 9 points, and other rows are 18 points. Both sides meet at
row-section starts y=72 and y=504 points. Header metadata fits narrow columns,
customer/address content has a broader area, descriptions have 240 points and
totals/payment use their own widths. No merged cells or spacer cells are required.
Values such as invoice number `000042` and postal code `001234` stay exact Strings.
This is a geometry fixture, not a page, font-wrapping or PDF implementation claim.

| Requested cell selection | Breaks present | Effective selection |
| --- | --- | --- |
| A2:C3 | Either/both orientations | Cells A2:C3: one section |
| A4:C5 | Horizontal only | Whole header + line-items sections: A1:H28 |
| F2:G2 | Vertical only | Whole left + right sections: A1:H36 |
| A4:C5 | Both; crosses horizontal only | Whole left header + left line-items: A1:F28 |
| B2:G2 | Both; crosses vertical only | Whole left + right header: A1:H4 |
| F4:G5 | Both; crosses both | Four complete sections: A1:H28 |

A worksheet version needs explicit row/column fences after the finite invoice if
its sections should stop there; otherwise the remaining scope is genuinely part
of a tail section. It must not invent a “used area” selection boundary.

Independent design arithmetic enumerates the 288 invoice cells without overlaps.
The stronger reason it works is that each column band's outer x edges agree across
all width profiles, while positive heights preserve order inside a column band.
This also covers any filter mask and arbitrary table length with these profiles.
It does not establish editor usability, implementation performance or platform passes.

A required safe-gap example uses two rows/two columns, widths `(2,1)` then `(1,1)`
and height profiles `(1,1)` for A and `(2,1)` for B. Point `(1.5,1.5)` is background.
Changing the first width profile to `(1,1)` and the second to `(2,1)` instead makes
B1 overlap A2; refuse even if both cells are Blank. These point units are illustrative.

## Capture, protocol and persistence

Geometry roots, canonical calculation capture, selected producer generations,
formatting and visibility are pinned together. Geometry-only W changes preserve C.
One geometry root/visibility combination produces one charged immutable map; no
value cache or independent Window calculation revision is added. Existing four
ordinary session captures also bound viewport leases, not four additional leases.

Propose WB16-only persistence and private session v16, finalized by codec audit
before implementation qualification. Persist scope-owned bands/profiles, sizes,
anchor kinds, defaults, allocation floors and dormant bound ordinal metadata.
Validate axis ownership, order, unique IDs, dimensions, profile ownership, positivity,
overflow, topology, disjoint interiors and budgets before installing the candidate.
Never trust a supplied map or digest as authority. WB15/older refuse under the
existing pre-release policy; no migration reader is added. Hosted v1 and CB5/CB6
remain unchanged. Unsupported experimental history/structured ownership refuses;
ADR-0023 remains parked at G3.

Private commands carry GridScopeId and expected state for add/remove break,
profile/range sizing, reset and batch preview/commit. Selection resolution is a
stateless response, not a retained server selection/scope. Viewport requests use a
capture-qualified document rectangle and cursor, returning explicit fixed-unit cell
rectangles, section/header/chrome descriptors and tagged data locations. Painting,
hit testing, editor placement and selection consume these same rectangles; C# must
not retain a second width/height arithmetic implementation.

Viewport pages retain the existing 256-cell/one-MiB frame limits and one capture
across all chunks. Proposed total viewport caps are 4,096 cells and eight MiB of
charged buffered payload, subordinate to global admission. A complete frame set
publishes atomically to the view; over-dense views give a zoom-in diagnostic, not
interactive partial/Blank substitute cells. Future document layout consumes the
same Rust map interface; no print/PDF backend is selected here.

## Admission and qualification gates

No global limit increase. Geometry-only edits must allocate no cell/graph identities
or calculation selections. Proposed geometry sublimits, within the existing book/
domain budgets: 64 bands per orientation per scope, 512 explicit breaks aggregate,
512 explicit profiles aggregate, 16,384 stored size runs, eight MiB stored geometry,
65,536 derived tiles per scope, 16 MiB transient plan/validation charge and ten million
metered work units per command. Counts, old roots, shared profiles, transient plans,
frame buffers and new metadata IDs all participate in aggregate admission. A count
ceiling is not a guarantee all maxima combine. Refuse before allocation/publication.

Required ordinary-layout qualification has **zero unexpected refusals** across
reviewed header/table/totals workflows: horizontal-only first, vertical-only,
combined aligned seams, balanced divider changes, shared-seam movement, source
refresh/growth and filtering. Every named ordinary workflow must fit without merge,
spacer or manual compensation workarounds. If it fails, revisit the geometry model
or editing contract before promotion; do not simply weaken overlap detection.
Qualification must start from the default grid and execute complete user editing
sequences, not install a finished profile fixture. Record adding boundaries, resizing,
changing profiles, shared-seam movement, and revising an existing invoice (for example,
widening descriptions and changing header/totals proportions). Cover horizontal-only,
vertical-only and combined cases, plus positional-layout review/reset after source
schema/order changes. Every published intermediate map must be valid; the normal
preview/batch controls must permit each intended transition without unexpected
refusal, manual balancing of unrelated sizes or a specially discovered construction
order. Record actions and intermediate outcomes as usability evidence. A valid final
map alone cannot pass this gate. Independent overlap/selection oracles and actual
GUI sequences are both required. Test confirmation acceptance, cancellation and
invalidation for every expanded destructive/bulk-changing operation.
Intentional overlap/overflow/over-budget cases must still refuse atomically.

Require independent dense rectangle and section-closure oracles, exact gap/edge
hits, both one-boundary selection cases, all header/keyboard paths, large tail
selection without enumeration, changed-root gesture refusal, ordinary/visible-only
bulk commands, stale paste, source identities, detach and unchanged calculations.
Exercise insertion at/inside breaks, deletion/coalescence, clipped tails, source
refresh/reorder/absence, native hostile input, disconnected reopen, retained captures,
strict typed values and zero final charges. Resource checkpoints reserve all source/
consumer/leaf nodes within 1,024/10,000/65,536 totals and include maximum empty scopes,
64 Windows, multiple profiles, retained maps and dense viewport refusals.

Implementation qualification will include local CI/strict all-feature checks,
six-target headless and managed/AOT GUI suites, with macOS x64 identified as Rosetta.
Normal graphical-terminal consent and short launchers remain required. Record
observed timings, accounted memory versus RSS, source hashes, diagnostic per-project
cloc and verified retention. Original Windows ARM64 cold structural timing failures
remain unresolved; later passes or toolchain updates do not reclassify them.

## Exclusions

No feature or platform execution accompanies this design. No Text conversion,
Window placements, merge cells, page breaks, text autofit/wrapping engine, renderer
or PDF backend, SQL/network provider, invocation API, licensing change or history
integration. No implementation or geometry-performance result is claimed. Design
approval and separate owner authorization are required before feature work.
