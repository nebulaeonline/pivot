# ADR-0043: Section-Local Headings and Axis Selection

Date: 2026-11-03.
Status: **Design accepted by parent review, 2026-11-03. Implementation awaits separate owner authorization; no feature implementation or platform qualification.**

The review accepts the design without mandatory architectural corrections or
another design review. Implementation must keep heading-divider targets distinct
from established geometry-resize controls and publish overlays coherently with
the captured cell frame. These are obligations within the accepted architecture.

## Purpose

Accepted and closed [ADR-0042](0042-section-breaks-and-local-grid-geometry.md)
supports independent section geometry. Its outer rulers follow the active section,
but this alone does not make the differing interior grids readily discoverable.
Users need headings at section boundaries and a direct way to select an entire
row or column within a particular section. They also need an unobstructed document
view through one master toggle for the interior headings.

This is a separate editor milestone. It does not reopen ADR-0042's acceptance or
change its canonical geometry, identity, resource or selection-closure contracts.
The [section-heading interaction contract](../architecture/section-local-headings.md)
defines the detailed requirements. See the
[design handoff](../handoffs/2026-11-03-section-local-headings-design.md).

## Decisions

### 1. Headings belong to derived sections

A section is the current intersection of a row band and column band, clipped to
scope extent. Either break orientation works alone. A horizontal break introduces
column headings for the following row band; a vertical break introduces row
headings for the following column band. With both orientations, the headings and
corner target identify their particular derived section. Do not introduce stored
rectangular section objects, independent cell grids or a new axis namespace.

Each visible section can expose column headings along its leading top edge and
row headings along its leading left edge. At scope edges, the existing outer ruler
can serve that orientation without duplicate labels. Internal edges supply the
missing local heading strip. Combined partitions require correct local segments,
not a fictitious straight seam where independent profiles produce stepped edges.

Labels use scope addresses: column C remains C and row 21 remains 21. Do not restart
numbering at each section or relabel a positional bound column with a producer
field name. A section corner/menu exposes its full logical bounds and selects the
whole section. Bound views retain a visible positional-presentation/provenance cue.

### 2. A local heading selects within its section

For section S = rows r0..r1 × columns c0..c1:

- Clicking row heading r requests r..r × c0..c1.
- Clicking column heading c requests r0..r1 × c..c.
- Clicking the section corner requests exactly S.
- Drag or Shift-extension within the same section selects the corresponding
  inclusive local row/column interval.
- Extension into another section uses ADR-0042's existing whole-section closure
  when either boundary is crossed. No partial endpoint-section exception.

These are value-cell selections, not instructions to delete or insert authored
axes. Ordinary outer headings continue to address the active section, as already
specified by ADR-0042. Do not silently change them to whole-scope selection.
An explicitly labelled whole-scope row/column action remains distinct and uses
the same closure and confirmation policy.

Use the original requested rectangle and canonically resolved effective bounds.
Do not normalize away whether section closure enlarged a request. Expanded Clear,
Fill, Format, Paste and other bulk-changing operations retain per-invocation
confirmation and state qualification. Selecting exactly one local row, column or
section is not itself an enlargement; normal command safeguards and limits remain.
Formula text, stored SelectionIds and explicit computational ranges are unchanged.

### 3. One master toggle controls interior headings

Provide a discoverable checkable toolbar/View control, **Show interior headings**,
enabled by default, with an accessible equivalent. One action hides or shows both
interior row and column headings, section corner controls and associated heading
dividers. No separate orientation switch is required for this milestone.

Hiding removes their painting, pointer targets, tooltips, tab stops and accessibility
nodes. It does not leave invisible clickable strips. Outer rulers, normal cell
selection and the existing geometry tools remain available. It does not hide cell
content or change section breaks, width/height profiles or selection semantics.
Existing non-heading review/error notices remain visible. A selection already
expanded by section closure stays expanded, with its confirmation requirement.

The preference is client view state shared by Grid views within that Desktop
instance, not a workbook field or producer property. Apply it consistently when
switching scopes. No new durable preference storage is required; a fresh Desktop
instance starts enabled. Changing it does not dirty a workbook, advance W or C,
publish geometry, change save output or trigger recalculation. Preserve selection,
zoom and the current content anchor. Cancel an active heading drag/resize preview
without committing if the toggle changes; never synthesize a final pointer release.

This is an editor viewing aid, not a print mode or document-rendering feature.
No requirement to add an export, pagination or PDF backend follows from the toggle.

### 4. Headings are viewport chrome, not authored geometry

No heading is a worksheet row/column, cell, profile-size addition or document
coordinate. Canonical content rectangles stay exactly those of ADR-0042. Drawing,
hit testing, editor placement and scrolling continue to consume the same capture.
C# must not reconstruct cell positions from uniform-size assumptions or maintain
an independent section geometry model.

Use bounded editor overlays anchored to the captured section edges, clipped to
the viewport. A strip can become sticky at the corresponding viewport edge while
its section is active/visible; it must identify the section and cannot imitate a
global ruler for another section. When layouts or zoom make all headings unreadable,
prioritize the active section, retain bounded boundary badges for other visible
sections, and allow explicit activation of their local headings. Never merge
distinct section identities or silently reinterpret a heading target.

Overlays may temporarily cover content as ordinary sticky editor chrome, but no
cell may become permanently unreachable or unreadable. Reveal/navigation must
bring a focused cell, full inline editor and selection endpoint into the unobscured
content area. Permit bounded view padding at scope/section extremes where needed.
The toggle provides a full content view without interior overlays. Neither toggling
nor revealing content alters stored geometry. If safe overlay placement is not
possible, expose the active section's headings through the outer rulers and a
clearly labelled section control rather than hiding an inaccessible cell.

This fallback is for clipping/density/irregular boundaries. It is not permission to
omit the interior headings in ordinary invoice layouts. Their construction,
selection and unobstructed-view workflows are explicit usability gates.

### 5. Capture qualification and bounded ownership remain mandatory

Heading identity includes scope, captured state, row/column bands, axis address,
orientation and the exact local selection bounds. Bound axes are positional and
qualified by their producer/view generation. A heading does not grant editable
authority to a bound Window. Filtered rows remain logically selected under existing
ordinary-selection rules; visible-only commands retain their explicit policies.

Rust supplies authoritative local heading descriptors from the same geometry
capture as the cells. Reuse compatible existing descriptors; audit whether bounded
enumeration of multiple visible sections needs a private protocol extension.
Publish overlay descriptors, painted targets and associated controls coherently
with the captured cell frame, preserving ADR-0042's publication-ordering correction.
Stale responses must not reinstall obsolete chrome or revive cancelled gestures.
Heading-divider targets remain distinct from existing geometry-resize controls;
hiding interior headings must not disable otherwise available geometry editing.
Pixel placement, clipping and accessibility are client view concerns. Selection
closure still resolves canonically before commands. Do not retain a server-side
active selection or create a second capture pool.

Enumerate only bounded visible heading segments and active-section metadata, not
all declared cells or the full product of partitions. Use existing aggregate
admission and capture limits. If the current response cannot carry the result,
return bounded continuation or an explicit density/refusal state; do not truncate
targets silently or increase global ceilings. Dispose superseded captures/overlays.

The native WB16 format remains unchanged because this adds no authored document
state. Hosted v1 and CB5/CB6 are unchanged. Reuse private v16 only if its existing
fields and semantics suffice. If the codec audit requires a changed private wire
contract, use a coordinated private v17 cut and reject mismatches; do not call an
incompatible protocol v16 or bump WB16 merely to match a session number. Record
the exact audit result before implementation qualification.

## Concrete selection examples

Use a 36×8 invoice with horizontal breaks before rows 5 and 29; optionally add a
vertical break before G. Addresses below are scope-relative canonical addresses.

| Layout and action | Requested/effective selection |
| --- | --- |
| Horizontal-only; click C at row-5 section heading | C5:C28 |
| Horizontal-only; click row 10 heading | A10:H10 |
| Horizontal-only; click line-items corner | A5:H28 |
| Both orientations; click C in A5:F28 section | C5:C28 |
| Both orientations; click row 10 in A5:F28 section | A10:F10 |
| Both orientations; click row 10 in G5:H28 section | G10:H10 |
| Column-heading extension C to G within row band 5..28 | Request C5:G28; closure A5:H28; expanded bulk confirmation required |
| Local row extension 27 to 30 in A..F section column band | Request A27:F30; closure A5:F36; expanded bulk confirmation required |
| Vertical-only break before G; click row 10 in right section | G10:H10 |

The last row-band example includes the full tail through row 36, not a guessed
used region. Selecting a local row/column never shrinks its extent because cells
are Blank, filtered or offscreen. Oversized operations still refuse under existing
limits. Turning headings off preserves every selection above exactly.

## Implementation qualification required after separate authorization

1. Independent descriptor/hit-target tests verify both single-boundary layouts,
   combined partitions, nonuniform and stepped profiles, clipping, actual scope
   tails, Blank cells, hidden rows and exact section-local bounds.
2. Stale pointer/keyboard operations after structural edits, scope switches,
   filter changes, bound refresh and superseded captures refuse without replay or
   wrong-section selection. Toggle-off during a gesture cancels without mutation.
3. Build and revise the invoice through ordinary controls. Select rows/columns in
   header, items and totals sections using interior headings; extend within and
   across sections; verify actual cells, highlights and operation previews.
4. Exercise accepted/cancelled/stale expanded-operation confirmation, exact local
   operations, read-only bound selection and visible-only hidden-row policies.
5. Toggle headings off/on at zoom extremes and while scrolled, editing and showing
   a layout-review notice. Verify no invisible targets, no dirty/revision change,
   unchanged selections and no unrecoverably covered cell/editor. Check pointer,
   keyboard, accessibility names/focus and high-DPI behavior.
6. Dense many-section views remain bounded and expose explicit section activation;
   near-origin, tail and sticky strips identify the correct section without
   duplicate/conflicting targets. Retained old frames release their charges.
7. Run appropriate local/headless checks and source-qualified six-target managed
   and Native AOT attended suites, identifying macOS x64 as Rosetta. Use graphical
   terminal consent and short launchers. Archive/verify before cache pruning and
   refresh the per-project cloc report at milestone close.

No implementation, resource measurement, accessibility execution or platform test
is claimed by this design. Existing timing qualifications remain unresolved and
ADR-0023 stays parked at G3. Parent design acceptance requires no further design
review. Separate owner implementation authorization is required before feature work.

## Exclusions

No change to canonical values, formulas, axis identity, section closure, structural
insertion/deletion meaning, calculation revisions or producer authority. No authored
heading rows/cells, per-section renumbering, persisted UI toggle, second geometry
engine, global limit increase, Text Window, placement, page/PDF feature, database
provider, licensing or history implementation. ADR-0042 stays accepted and closed.
