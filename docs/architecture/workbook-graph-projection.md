# Graph projection and stable cell references

Status: Proposed ADR-0010 semantics, not current core capabilities.

## Identity and binding

A sparse position directory maps `(WorksheetId, RowId, ColumnId)` to a CellId when
content or an explicit scalar reference needs an anchor. A reverse directory maps
CellId to its current position/status. CellId is independent of the ordered axes:
moving content relocates its anchor rather than changing identity.

GraphId/portable NodeId remain computational identities. A binding directory maps a
cell anchor to its computational backing and ownership policy. Core process handles
remain private projections of those IDs. A decimal cell initially owns an input
node; a formula cell owns a computed node. Ordinary numeric/formula replacement can
keep that node identity while changing its definition through core staging.

Binding a scalar spreadsheet reference stores the stable target CellId, current
resolved graph target, copy flags and source-span provenance. Computational tokens
are derived from that owned binding record. They are never reconstructed by resolving
historical A1 text. The additional cell identity distinguishes a spreadsheet target
from a reusable model node that may outlive its cell projection.

The initial proof should use one canonical writable cell per backing node. Future
aliases/model-port views need an explicit projection proxy/binding guard so deleting
one cell invalidates references to that cell without deleting a shared model node.
Simply binding every alias directly to the same NodeId loses that distinction.
Generic alias nodes may be computationally reusable; WorksheetId does not enter
the core. Multiple read-only displays of a node are not multiple mutable owners.

## Proposed operation semantics

| Operation | Anchor/graph effect | Existing scalar references |
| --- | --- | --- |
| Set decimal or replace formula at an existing cell | Preserve CellId; preserve backing node when possible; install a new definition | Refer to the edited cell's new definition |
| Enter content at a never-materialized position | Allocate anchor and required backing | New explicit references can bind it |
| Enter content at an anchored blank | Preserve the existing blank anchor/backing | Observe the new value |
| Clear content | Preserve anchor; content becomes Blank; no tombstone | Remain bound to the blank target |
| Move/cut a cell | Carry CellId and its computational identity to the new position | Follow the moved target |
| Copy/fill | Create/reuse destination anchor as an ordinary edit; create copied definitions with transformed references | References to source remain unchanged; existing destination references observe edited destination |
| Delete cells with shift, or delete row/column | Retire removed anchors/backing owned solely by them; shift surviving positions | Removed targets become invalid; surviving targets retain identity |
| Later reuse of a deleted/vacated position | New CellId unless a distinct surviving blank anchor already owns it | Retired-target references remain invalid |
| Worksheet rename | Change label/directory and rendered address text | Bound WorksheetId and target meaning remain unchanged |

Clearing content and deleting a structural target are deliberately different APIs.
Deleting a formula node must not silently delete an independent named model it
displays. A move into occupied destinations is rejected by the first proof; a later
explicit overwrite move would need to retire displaced anchors and diagnose their
inbound references atomically. Cross-document moves/imports need coordinated ID
remapping and are deferred.

For a scalar reference to an empty A1, allocate only that sparse anchor on demand.
Entering a value at A1 later updates the same target. Ordinary arithmetic treats
Blank as zero under the proposed numeric policy; range aggregates can distinguish
blank from a real zero. A small decimal-only proof can use a marked zero input as
a derived backing for this arithmetic subset, retaining Blank in workbook metadata.
It must not claim complete typed-blank behavior or materialize zero inputs for all
positions in a range. Clearing may retain such a demanded input source; releasing
it requires proving no bindings/captures still rely on its identity.

## Structural edits and source text

Row insertion changes order/rank and displayed addresses, not surviving anchors.
Scalar references continue to target the same cells. `$` flags control copy/fill,
not whether a target survives structural movement. Deleted IDs remain tombstoned;
lookup by an old display name or reused coordinate cannot repair them automatically.

The original submitted formula source remains retained. Displayed formula text is
rendered from reference spans, current target positions and copy flags, so a sheet
rename can show its current label without changing bindings or destroying provenance.
Saving must retain source and authoritative bound descriptors. A user submitting
an edited formula performs explicit new resolution against the expected workbook
revision; a stale formula-bar buffer must not bind against an unnoticed newer layout.

## Responsibility split

Projection owns axis order, CellId lifetime, sheet/name scopes, A1 resolution,
reference copy flags, structural transforms and range geometry. Core owns typed
arithmetic, stable computational dependencies, snapshot validation, invocation and
calculation errors. Binding/resolution joins these through a reviewed Rust interface,
not text substitution or a second evaluator.

Hidden graph nodes and named model interfaces remain first-class. Workbook existence
never becomes a prerequisite for core use or independent invocation. Detached model
calculations and optional hosted publication retain their current boundaries.

Future garbage collection may reclaim unreferenced backing nodes or empty anchors,
but cannot reuse retired IDs, invalidate a live immutable capture or convert an
existing bound reference into a coordinate lookup. Reference/undo retention is an
explicit measured policy, not an automatic side effect of hiding or clearing cells.
