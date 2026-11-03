# Section-local heading interaction contract

Status: **Design accepted by parent review under [ADR-0043](../adr/0043-section-local-headings-and-axis-selection.md), 2026-11-03. Implementation awaits separate owner authorization.**

This supplements the accepted [canonical geometry contract](section-grid-geometry.md).
ADR-0042 remains closed. The accepted design adds discoverable section-local editor
headings and a master visibility toggle without changing authored geometry.

## Authority and selection

An interior heading names a derived row-band/column-band intersection, orientation,
scope axis address and exact local row/column bounds. It is qualified to the same
scope, geometry, visibility and producer generation as the displayed cells.
Authored axes retain stable identity; bound axes remain positional presentation.

Row click selects the row across that section's entire column interval. Column
click selects the column across its entire row interval. Corner click selects the
section. Labels retain full scope addresses. Within-section extension selects the
inclusive local interval; crossing either boundary applies canonical whole-section
closure. Original and effective rectangles remain distinct for expanded-operation
confirmation. Read-only authority, hidden-row policies, operation limits and stale
state checks remain in force. No formula or stored range is expanded by a heading.

Outer headings continue to follow the active section; a whole-scope action must
be explicit. Heading selection does not redefine structural insertion/deletion
as a section-local axis edit. Such product behavior is outside this milestone.

## Presentation and input

Interior column strips follow section top edges and row strips follow left edges.
Scope-edge outer rulers can serve an orientation without duplication. Stepped
boundaries are segmented from captured canonical geometry. No independent
uniform-cell or section-map arithmetic is permitted in the client.

Overlays and sticky strips are bounded editor chrome. They do not reserve document
space or alter canonical cell rectangles. Active-section labels have priority
under crowding; other visible sections remain explicitly activatable. No cell or
inline editor may remain unreachable under chrome: reveal it in the unobscured
viewport, use bounded scroll padding when necessary, or expose the active section
through the outer rulers if a safe interior strip cannot be placed. Ordinary
invoice layouts must show useful interior strips without routine fallback.

Paint, pointer targets and accessibility nodes use the same published overlay
layout and capture identity. Resolve overlaps deterministically: heading corner,
explicit divider target and heading label have distinct non-conflicting targets;
chrome takes input precedence only in its painted interactive footprint. Hidden
chrome has no input authority. A superseded capture invalidates an active gesture
rather than retargeting its final event onto a newer section. Reuse ADR-0042's
paired/shared sizing commands if interior divider resizing is exposed; do not
invent a new resize rule or silently modify unrelated profiles.

Publish overlays coherently with the captured cell frame and associated controls,
preserving ADR-0042's corrected publication order. Do not introduce a separately
current overlay layer. Stale responses cannot reinstall obsolete chrome or revive
cancelled gestures. Keep heading-divider targets distinct from existing geometry
resize controls; hiding headings must preserve otherwise available geometry editing.

Keyboard focus and accessibility expose section bounds, axis orientation/address,
selected state and read-only/positional context. Equivalent commands must work
without a pointer. Drag and Shift anchor to the explicitly chosen section/axis;
current selection normalization must not silently replace the requested anchor.

## Master toggle

**Show interior headings** is one checkable toolbar/View action, enabled initially.
It controls both interior orientations, their corners and heading dividers. Hidden
means no paint, hit targets, tooltips, keyboard stops or accessibility nodes.
Outer headings, cells, geometry commands and error/layout-review notices remain.

The setting is session-local client view state shared by Grid views. A new Desktop
instance starts enabled. Scope switching honors the current setting. Toggle changes
neither workbook dirtiness nor W/C, save bytes, geometry, zoom or selection and
does not clear expanded-operation confirmation requirements. Preserve the content
anchor. Cancel an unfinished heading gesture/resize preview; do not publish it.
If focus was in hidden chrome, return it to the worksheet without synthesizing an
edit, selection extension or command. The control itself remains available while
headings are hidden. No persisted preference, printing or PDF policy is introduced.

## Admission and protocol boundary

Visible heading descriptors and overlay ownership are bounded by existing capture,
payload and aggregate resource admission. Do not enumerate the declared grid,
allocate cell identities for headings or create a second retained capture pool.
Density reduction must remain explicit and preserve section identity. Deferred
responses cannot republish hidden chrome or restore a cancelled gesture.

The implementation codec audit must identify whether existing private v16 heading
descriptors cover all visible local segments. Reuse them if sufficient; otherwise
make a coordinated private v17 cut, without changing WB16 or hosted v1/CB5/CB6.
No format/protocol change or platform qualification accompanies this proposal.
