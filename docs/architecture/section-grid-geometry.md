# Section Grid geometry and selection contract

Status: **Design accepted under [ADR-0042](../adr/0042-section-breaks-and-local-grid-geometry.md),
2026-11-02. Design only; no implemented map or platform qualification.**
This specializes the accepted [composition architecture](document-composition.md).
ADR-0042 is authoritative for this slice; it does not implement later Text/page work.

## Sections exist with either boundary type

The horizontal partition covers the scope's rows; each band selects a width profile.
The vertical partition covers its columns; each band selects a height profile.
Each partition has an implicit first band even when there are no explicit breaks.
A derived section is a pair of these bands intersected with the current scope extent.

Thus horizontal-only sections span all columns, vertical-only sections span all rows,
and combined partitions produce intersections. Neither section creation nor
whole-section selection requires crossing both orientations. No Cartesian-product
section objects or per-section value models are allocated.

Cell selection is one of:

- `Cells(anchor, active)`: its logical rectangle lies in one section.
- `Sections(anchor, active, touched band intervals)`: crossing either boundary
  expands to all complete touched sections, including the endpoint sections.

For rectangle D, compute the ordered row bands R and column bands C intersecting D.
If `len(R)==1 AND len(C)==1`, preserve D. Otherwise take the complete first-to-last
R span and complete first-to-last C span, clipped to the scope's actual extent.
This closure is idempotent, includes D and depends on current endpoints, not whether
the pointer briefly left and returned during a drag. There is no partial endpoint
section exception. The selected physical region is the union of its mapped cells,
not the bounding box of that union. Hidden rows remain logically selected but are
not painted. No CellIds or computational SelectionIds are allocated by selection.

The ADR's six examples are mandatory, particularly horizontal-only crossing and
vertical-only crossing. Large implicit tail sections are real; do not substitute
a guessed used range. The UI identifies kind, complete bounds, section count,
logical/hidden counts and admission failures. Section badges select a whole section.
Rulers name their active row/column section; ordinary header selection starts within
that active section. A dedicated whole-scope axis action uses the same closure.

No computational descriptor is enlarged by a geometry edit. Formula text and stored
SelectionIds continue to denote exactly their authored logical coordinates and
structural affinity. Interactive selection-derived operations use the closure;
explicit programmatic data requests remain explicit data requests. This distinction
is not an authorization bypass: geometry never grants value-write authority.

When closure enlarges the original request, each destructive/bulk-changing command
(Clear, Fill, Paste, Move, Format and visible-only variants) requires explicit
per-invocation confirmation. Show requested/effective bounds, added-position count,
operation and hidden-row policy. Preserve the original request separately from the
closure so this decision cannot disappear when selection is normalized. A drag
release is not consent; cancellation has no mutation. Bind confirmation to scope,
expected state, geometry/visibility/producer capture, range, operation and payload;
any change invalidates it. No blanket approval or size threshold. Copy/navigation
are exempt; exact, unexpanded requests retain existing safeguards. This is a client
interaction guard, not a new server-retained selection or programmatic authority.
Existing stale-state checks, limits and exact paste-shape rules remain mandatory.

## Stored roots, identities and sizes

A scope has defaults, ordered horizontal/vertical band records and immutable sparse
width/height profiles. Defaults have an implicit shared representation. Explicit
bands/profiles have non-reused scope-qualified metadata identities. Deleted records
remain charged while retained roots own them; allocation floors and actual retained
storage are both accounted. Equal profiles can share storage without merging bands.
Geometry with a break is semantically nontrivial even if every size equals default.

Authored anchors and size runs reference the scope's existing axis namespace and
stable sequences. Resolve ranks through compact axis runs; do not create identities
for every coordinate. New scope-owned metadata IDs count against aggregate identity
admission and cannot manufacture a separate per-profile allowance.

A bound Window instead uses explicitly tagged positional ordinals within the finite
maximum Grid envelope. This is a proposed specialization of the general stable-axis
rule: a projection has no editable authored axes. Local width/height settings and
breaks follow display positions, **not records or fields**. Source reorder or column
insertion may therefore change which field occupies a locally sized position. Show
source provenance; never silently convert those settings into producer authority.
The implementation review must qualify this behavior. All source scalar/range
references retain ADR-0041 identity semantics.

Controls/rulers label bound geometry “Positional presentation”, separately from
source-field formatting. Meaningful schema/order changes (field addition/removal/
reorder or declared record-order changes) produce a persistent generation-qualified
“Review positional layout” notice. Review, Keep positional layout and Reset
presentation are explicit actions; review/keep acknowledges only that generation.
Reset previews dropping local breaks/sizes to scope defaults, with no source or
formatting mutation. A further meaningful change renews the notice; value-only
refresh does not. Publication remains valid and settings never silently follow fields.

Current producer extent clips display/selection only. Bound metadata beyond it is
dormant and remains charged; it can become visible after growth. Validate the map
against the full existing maximum envelope so growth, shrink or refresh cannot
introduce a geometry collision or invalidate a successful producer publication.
Unavailable sources produce no data-cell targets. Metadata is not a data-access grant.

Profiles contain positive sizes in integer 1/64-point units, with unset entries
inheriting scope defaults. Explicit full-axis settings can be compressed runs.
Coordinates/prefix arithmetic are checked signed 64-bit; reject invalid units,
foreign anchors, overflow and any result outside the declared coordinate domain.
Screen zoom, DPI, stroke thickness and font measurement never enter stored sizes.

## Complete mapping without cell enumeration

Let X_i(c) be width-prefix sums for row band i, and Y_j(r) height-prefix sums for
column band j. Cell `(r,c)` is
`[X_i(c), X_i(c+1)) × [Y_j(r), Y_j(r+1))`, with `i=H(r), j=V(c)`.

For each row-band/column-band pair, intersect its columns with that width profile's
constant-size runs and its rows with that height profile's constant-size runs.
A product of these intervals is a topology tile: its cells exactly fill the tile's
physical rectangular envelope, with no internal gaps. Tiles partition logical
cells, so positive-area intersection of two tile envelopes is equivalent to at least
one cell-interior collision. Blank cells are included. Checking only occupied cells
or a few sample points is insufficient.

First compute checked tile cardinality and reserve scratch/index storage. Sweep
tile x events with ending edges before starting edges at equal x. An ordered active
y-interval index detects positive overlap; deterministic event order yields a stable
witness. In a valid prefix, active y intervals are disjoint, so predecessor/successor
checks suffice. Convert a conflicting overlap back to logical cells using the tile
prefix and sizes, then verify the witness. Keep arithmetic and work metering bounded.
One-orientation maps have disjoint row or column slabs automatically and need no
quadratic cross-cell test.

A valid plan exposes `cell_rect`, `hit`, `section_closure`, tile visitation, extent
and seam/header geometry. It has no evaluator or value cache. Bound full-envelope
validation is separate from viewport clipping to actual source extent. No editor or
future renderer may invent a second width/height mapping algorithm.

## Filters and derived maps

Use one captured visibility mask for all column bands. In a filtered map,
`Y_j(r)` sums only preceding visible rows' heights in profile j. Hidden cells have
no painted rectangle; their positive stored sizes and logical membership remain.

A constant-height tile may contain a logical interval with hidden holes. Its visible
members still tile a rectangle; retain the shared mask's rank/select mapping to
recover logical row addresses. Weighted prefix sums count visible rows per size run.
An alternating mask must not force per-cell topology or millions of stored row runs.
Zero-visible tiles disappear. Validate the complete derived map under the same work
and byte admission before presenting it.

Globally aligned column-band x seams give a useful sufficient condition: column
bands stay physically disjoint under every common row mask, while each band's rows
remain ordered. The ordinary invoice uses that condition. It is not imposed as an
extra universal rule: valid nonaligned/gapped maps are also supported.

A proposed geometry/filter command whose current derived map overlaps or exceeds
presentation admission refuses atomically, with a reason. A later data change must
not be refused to preserve an editor map. If its new mask cannot produce a valid
map, retain the filter definition as presentation-suspended/NeedsReview, show the
validated unfiltered geometry with current values, disclose the reason and block
visible-only commands. Never show an old filtered generation against new values.
Explicit revalidation is required to resume. Source-bound views continue including
all source positions under ADR-0041; this does not add source compaction.

## Structural and lifecycle table

| Event | Geometry result |
| --- | --- |
| Add break | Before a current interior axis; snapshot/share predecessor profile; no size or data change |
| Existing break at same axis | Report existing boundary/no-op; never create duplicates |
| Remove break | Preceding profile extends into following band, shown in preview; implicit first band cannot be removed |
| Insert exactly at authored break | New axes join preceding band; break stays before the original following AxisId |
| Insert at rank 1 | Inherit first band; no nonexistent preceding band |
| Insert inside band | Inherit band; each profile assigns new-axis size from its preceding old axis, or following old first axis at scope start |
| Delete an anchor | Move boundary to next surviving old axis; never repair it using a newly appended tail identity |
| Several boundaries meet | Later band in pre-edit order wins; promote its profile if it becomes the implicit first band |
| No following surviving old axis | Drop that trailing empty band |
| All original axes deleted | Keep one implicit band using the pre-command first band's opposite-axis profile; remove retired-axis size entries; retain scope defaults |
| New tail axes after deletion | Inherit last surviving band; size is the profile's scope default unless explicitly set by the structural rule above |
| Sort or content move | Axis geometry stays at axes; values/reference identities follow existing data rules |
| Bound refresh/reorder/grow/shrink | Ordinal geometry remains positional; clip display, not stored metadata; no source mutation |
| Retire scope | Retire its geometry with the scope; old captures retain old roots and charges |
| Detach bound copy | Under the existing 256-position limit, copy captured geometry clipped to extent into fresh authored-axis/band identities atomically with values/styles; no reference rebinding |

Size entries for retired axes are removed from the candidate, not from old captures.
Insertion/deletion may split/coalesce sparse runs; namespaces and allocation cursors
never reuse retired identities. Native restore validates the resulting relationships.
Unchanged data no-op structural commands must still publish meaningful geometry/axis
changes when section metadata requires them. Cancellation, clipping/admission or
collision failure preserves all original data and geometry heads.

## Editing and coherent frames

Interior-divider dragging changes exactly the two displayed adjacent sizes and
shows both values. It preserves their total and refuses at positive-size limits.
Aligned section-seam movement is explicitly labelled as a batch across the affected
perpendicular profiles; a nonaligned segment is edited in its own profile rather
than presented as one global line. Numeric sizing previews independent changes.
No edit silently adjusts unrelated sizes to hide a collision. Reset-sizes keeps
breaks; reset-all-geometry explicitly removes breaks and returns scope defaults.

The ordinary invoice gate executes complete editing sequences from default geometry:
add boundaries, resize, change profiles, move shared seams and revise an existing
invoice. Exercise each orientation alone and both together, including bound layout
review/reset. Record user actions and intermediate maps. Normal preview/batch tools
must support valid transitions without unexpected refusals, manual balancing of
unrelated dimensions or discovering a special construction order. A preinstalled
valid final fixture is arithmetic evidence only and cannot satisfy this gate.
Independent map oracles and actual GUI editing sequences are both required.

A frame pins scope, W/C, geometry root, formatting, visibility and producer context
from the canonical capture. Every page/cursor belongs to that same lease; scope
selection remains client-only. The existing four ordinary captures are the joint
ceiling, not four calculation captures plus four geometry leases. Keep old frames
readable while their owners remain, but reject their targets for current writes.

Requests carry a finite document viewport; frames return explicit fixed-unit cell
rectangles, logical coordinates, section/header/chrome fragments and ordinary or
projected value provenance. Client drawing, editor rectangles, focus outlines and
hit tests consume that map. Conversion to DIP uses the pinned rational zoom and
scope origin; subtract large document origins before floating-point painting.
At the protocol boundary a document point is quantized by floor toward negative
infinity to 1/256 of a geometry unit (1/16,384 point), checked for finite range.
Rectangles remain integer geometry units and compare against points at that common
subunit scale. All pointer paths use the same pinned transform and rule; parity
fixtures cover exact boundaries and fractional zoom. Native map decisions and
client rectangle containment must agree, including gaps at right/bottom edges.

Pages respect 256 cells and one MiB each; payload size may stop a page earlier.
A viewport has at most 4,096 cells/eight MiB buffered payload and publishes only
when complete. Over-density/admission refusal produces a clear zoom-in/reduce-view
state with no fabricated Blank cells or active partial hit targets. Rust reserves
server buffers plus a conservative client-payload allowance in the lease before encoding. The C# client separately bounds live frame/page ownership
using the same cell/byte caps and drops obsolete buffers before releasing that
allowance. This is accounted interprocess ownership, not a claim to measure C# GC
heap residency or process RSS. Plans, old roots and mirrored payload charges join
the enclosing budgets; cancellation/close releases only actually unowned leases.

Scroll state is a logical scope-qualified anchor plus within-cell document offset,
not a guessed uniform first row/column. Arrow navigation follows logical visible
positions; the map reveals the resulting rectangle. Headers describe the active
profile even when other sections' physical row/column edges differ. Gaps return
no cell. Seams/chrome have distinct hit targets; they are not value cells or page
breaks. Selection shape uses tile spans clipped to the viewport without iterating
its potentially enormous logical area.

Any map/extent change cancels a drag. If canonical selection would gain positions,
show the proposed extent but disable content operations until an explicit user
selection gesture. Never silently widen a pending Clear, Paste, Fill or Format.
Geometry commands that intentionally change section selection disclose that result
in their preview. Existing hidden-row tickets and exact paste/operation shapes remain.

## Persistence and aggregate ownership

Proposed WB16/private v16 are audited cuts, not implemented versions. Save stores
source-independent presentation roots, scope-qualified profile/band IDs, anchor kind,
ordered starts, positive size runs/defaults and allocation floors. Bound ordinals
cannot masquerade as authored axes or producer references. Imported caches/tiles,
map leases and selection state are not saved. Restore privately rebuilds and validates
base topology and authority before install, retaining the existing producer checks.
A mask-dependent view can suspend on open; invalid base geometry rejects the file.
No historical reader, clipboard geometry envelope, hosted change or history promotion.

| Owner | Charged storage |
| --- | --- |
| Scope geometry root | Bands, profiles, defaults, sparse size runs and indexes, metadata identity floors |
| Shared immutable profile | One actual allocation; referencing bands pay handles, not duplicate payload |
| Derived map/capture | Topology/sweep indexes, visibility references, old roots and complete capture ownership |
| Mutation/import candidate | Old + staged roots, transform/validation scratch, rollback ownership |
| Viewport lease/client frame | Server buffers, conservative client-payload allowance, actual client ownership limits, rectangles/strings/styles/provenance |

All charges join existing book/domain limits. Proposed caps are those in ADR-0042;
new metadata IDs share aggregate allocated-identity admission. Declared cell extents
are not physical allocations. Reserve before expanding runs/tiles/buffers; meter
visits, comparisons, arithmetic and emitted records, including rejection work.
Refuse when retained old/new roots or simultaneous leases exhaust admission; never
claim storage freed merely because a boundary is no longer current.
