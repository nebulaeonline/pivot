# Section geometry, Text and document composition

Status: **Architecture accepted with clarifications under [ADR-0039](../adr/0039-named-windows-and-document-composition.md), 2026-11-01**.
This is a design contract, not implemented behavior.

## Section geometry

Use independent ordered **row bands selecting column-width profiles** and **column
bands selecting row-height profiles**. A horizontal boundary before row 21 can give
the line-items section different A/B/C widths from the header. A vertical boundary
can give cells to its right a different row-height profile. Profiles map logical
axis entries to positive bounded physical sizes; unset entries inherit the scope
default. There are no independently editable rectangles in a materialized band
cross-product. Creating a section copies effective geometry, then permits edits.

For cell (r,c), x is the prefix sum of widths preceding c in r's row-band profile;
y is the prefix sum of heights preceding r in c's column-band profile. This mapping
can yield overlaps or gaps where both regimes change: the initial design **refuses
any geometry candidate with overlapping cell interiors** rather than choosing a
paint-order winner. Gaps are background, not implied cells. This check uses bounded
partition/profile intervals and admission, not full-sheet cell enumeration. Extreme
profiles may therefore be refused even if no occupied cells overlap. This is a real
usability tradeoff of independent partitions; qualification must include the intended
invoice header/table examples before promotion.

The mandatory qualification set includes an invoice header with narrow metadata
columns, wide line-item descriptions, different totals/payment-information geometry,
interacting horizontal and vertical section boundaries, and selection/hit testing
across gaps and seams. Frequent refusal of ordinary invoice layouts requires revisiting
the geometry model; user workarounds do not satisfy this gate. These cases have not
been executed by the architecture milestone.

Viewport, selection and renderer share this mapping. Hit testing uses half-open
cell interiors (right/bottom edge belongs to the next interior); gaps return no cell.
A logical rectangle remains a logical rectangle, but its highlight may be a union
of physical spans, never a fabricated bounding-box selection. Keyboard navigation
uses logical axes. Scrolling retains a logical anchor plus within-cell offset.
Range calculation and formatting use logical coordinates, not painted boundaries.

Breaks are stable boundary metadata immediately before a surviving axis identity.
Insertion at that boundary belongs to the preceding band; the boundary remains before
the original following axis. Insertion inside a band inherits that band. Deleting the
following axis moves the break to the next surviving axis; coincident breaks coalesce
with the later band's profile winning. A trailing empty band disappears. Deleting
all axes retains the required default scope geometry. All profile/boundary transforms
stage atomically with structural edits. No changes to CellIds, graph bindings,
SelectionIds, region formatting or page-break metadata occur merely from a break.

## Native Text representation

One Text Window owns a bounded tree: paragraphs, literal Unicode runs, typed inline
reference runs and optional allowlisted hyperlink annotations. Character properties
include font manifest reference, size, weight, italic, underline and color. Paragraph
properties include alignment, indentation, before/after spacing and bounded bullet/
numbering metadata. Container properties include padding, vertical alignment, border
and background. No arbitrary XML/HTML, scripts, executable links or embedded layout
programs. Grapheme-aware editing preserves Unicode; shaping/bidi is a renderer concern.

An inline run stores bound source identity, type and format descriptor. A UI may
accept `{{OrderTotal}}` as an **authoring gesture**, but must resolve it to a typed
run with expected-revision validation; literal braces remain literal text. The
renderer formats captured values and faults, never evaluates expressions. Binding
deletion remains an explicit fault, not replacement by an identically named field.
Text has no independent formula evaluator and cannot become its own source.

Bound Text accepts an immutable schema-validated rich-text producer result under
these same constraints. Grid and Text producer schemas are distinct; rebinding a
table to Text cannot silently concatenate records. Typography overrides may be
Window-owned, but source content remains read-only.

## Rectangle conversion and reverse conversion

Conversion selects an unoccupied layout rectangle, such as A1:E20, creates a distinct
Text Window and a placement anchored to that rectangle. The Window is not the 100
cells: those cells become a reserved, non-editable backing footprint while the
placement exists. Direct edits and structural commands intersecting this footprint
refuse unless they atomically remove the reservation under the conversion contract.

The initial **lossless** command admits only semantically Blank cells with no formulas,
inbound scalar references, names, input targets or intersecting range dependencies.
Refuse if proving those conditions exceeds admission; do not assume absence. All
100 positions, including demanded blank anchors, remain logically present; preserve
existing CellIds/backing identities and exact region/cell formatting as dormant
metadata. Do not retire or rebind them. New references into the reserved footprint
are refused, so later formulas cannot accidentally observe hidden cells. Existing
content, formula references or numerical zeros make conversion refuse atomically.
The diagnostic identifies why; it never flattens a formula or drops 99 cells.
The conversion UI must identify whether values, formulas, inbound references or
other dependencies caused refusal and explain the blank/unreferenced requirement.

Reverse conversion restores the exact grid footprint and formatting only when Text
is empty, binding-free and still has its original reservation. Otherwise it refuses;
the user may first explicitly export/move the text to another Text Window, leaving
this one empty. Reverse retires the Text WindowId/placement, with inbound Text
presentation references faulting. Grid anchors never need resurrection. Save stores
the reservation and dormant metadata, so the same check works after reopen. This
is a reversible presentation conversion, not Undo/Redo or a promise to map edited
paragraphs back into 100 cells. Nonempty-grid import to rich text is a separate future
transformation requiring a reviewed mapping and explicit data-loss policy.

## Layout and pagination

A DocumentLayout has physical page size, margins, printable area, bounded page bands
and an explicitly ordered body flow. Placement coordinates use fixed-point document
units, not screen pixels. Anchors identify a logical cell boundary, a page-band
origin or the preceding flow placement; they never use a transient painted pixel.
Deleting an anchor yields NeedsReview and blocks rendering, not automatic relocation.
Ordinary worksheet/page-associated content can be placed directly: headings, logos,
notes and totals need no independent Window. Assets are immutable captured bytes.

| Content policy | Meaning |
| --- | --- |
| Every page | Default for static page-associated content; reserve a fixed band |
| First page | Reserve its band only on page 1 |
| Last page | Reserve on the final page; final totals normally use this policy |
| Once | Emit once at its ordered flow position, then consume it |
| Continue | Default for flowing Windows; emit successive content fragments |

Fixed dimension clips only with explicit Clip policy. Expand grows in the body up
to remaining printable space, then follows its declared break rule. Flow fragments
across pages while retaining one Window identity. Shrink uses a deterministic bounded
scale search down to an explicit minimum font size; failure at minimum refuses output
unless explicit Clip was selected. Text expands vertically at fixed width initially;
simultaneous automatic width/height solving is excluded. Grid rows are indivisible
by default; Text breaks at line boundaries with bounded widow/orphan rules. Repeated
Grid headings consume space on each fragment but are not extra logical records.

Sibling flowing placements consume body space in declared order. Side-by-side content
requires a fixed-size placement group whose height is the maximum of its children;
children cannot independently flow across pages in the initial model. This is a
layout group, not a nested Window. An oversized indivisible object produces a
layout error; no infinite page creation or automatic scale-to-invisible. Empty flow
content consumes zero body height unless a minimum height is explicitly configured.
A wholly empty document emits one page, including any applicable static content.

Pagination first lays out without final-only content, then reserves the final band
and repaginates the affected tail. If it does not fit, append a final page, retaining
the final-only band there and moving the minimum tail content required. Final bands
have fixed measured size independent of page count; page-number runs use bounded
reserved width. This prevents last-page content from toggling page count. Every
iteration consumes a layout-work budget; refusal is atomic. First/last can both apply
on a single-page document. Conflicting fixed placements refuse unless explicit
decorative overlay order is declared; flow cannot silently overwrite them.

For 23 line items with capacity eight, pages project 1–8, 9–16 and 17–23 before any
final-band adjustment. `LineItems|A9` is still the ninth logical row, wherever its
page fragment is drawn. Page projections carry source item and capture provenance,
never new cells, revisions, formulas or identities. Editor chrome is excluded by
default. Interactive viewport scrolling and printable page placement share geometry
and text measurement primitives, but are separate views of the immutable content.

## Design qualification cases for later implementation

Require scope-isolated A1 references, rename/delete/reuse faults, positional copy vs
stable existing item references, worksheet-sort identity and unsupported external
types. Exercise refresh/capture races, failed refresh retaining the selected snapshot,
source cycles and cancellation with retained owners. Geometry cases must include
both break types, insertion exactly at a break, deletion/coalescence, gaps, overlap
refusal and matching renderer/hit-test maps. Text conversion must preserve every
blank anchor/format and reject nonempty or referenced selections before mutation.
Layout cases include empty sources, an oversize row, repeated headings, two siblings,
first/last on one page, final totals requiring another page, complex scripts, missing
fonts, asset limits and exact leading-zero Strings. These are proposed acceptance
cases; this design milestone does not execute them or claim they pass.
