# ADR-0030: Row, column and rectangular-range formatting

Date: 2026-10-29.
Status: **Accepted for implementation by parent design review (2026-10-29).**

The owner’s subsequent WB9 clarification supersedes the original compatibility
proposal: WB10 only, no WB9 reader/conversion/migration. Earlier versions are
explicitly rejected; current fixtures move to WB10. Preserve product semantics,
not obsolete serialization. The remaining design and complete implementation
milestone are authorized; no additional design review is required.

Owner authorized design work following acceptance of ADR-0029. This ADR extends
the accepted ADR-0026 presentation contract; it does not reopen that milestone.

## Context and governing interaction

Current `format_sheet` expands a rectangle into position-owned complete `Style`
records. `style_at` reads that position or the built-in default. Formatting blanks
is graph-free, but commands and non-default positions are independently capped at
4,096. This prevents ordinary whole-column and 65,536-cell table formatting.

Make rows, columns and rectangles first-class formatting targets without enumerating
their coordinates. Preserve exact rendering, position ownership, atomic publication
and the independent calculation model.

**The owner's clarification controls precedence: applying a property to a selection
changes that property everywhere selected, including individually formatted cells.**
There is no permanent cell-over-range, column-over-row or other specificity priority.
If 100 cells have another number format inside a 10,000-cell selection, applying
Currency to the selection replaces the numeric-format settings in all 10,000 cells.
No extra checkbox, cleanup operation or individual correction is required.

## Decision: last applicable edit wins, independently per property

Cells, rows, columns and rectangles describe command geometry, not precedence layers.
For each property at each position, the latest successful command that touches that
property determines its explicit value. Properties omitted from a patch are unchanged.
An explicit false, General, Auto or theme color is a value, not an absent override.

| Actions in order | Effective result at the intersection |
| --- | --- |
| A1 bold; then A1:P4096 Currency | A1 stays bold and becomes Currency; the entire rectangle becomes Currency |
| A1 not bold; then A1:P4096 bold | A1 and every other selected position become bold |
| Row 5 red background; then column B blue background | B5 blue, remainder of row 5 red |
| Column B blue background; then row 5 red background | B5 red, remainder of column B blue |
| 100 cells Number; then their 10,000-cell enclosing rectangle Currency | All 10,000 use Currency; unrelated visual properties remain |
| Whole column Currency; then one cell General | That cell is General; a later whole-column Number command changes it to Number |
| Overlapping range red; later overlapping range blue | Overlap blue; earlier range outside the overlap stays red |

Numeric presets are explicit multi-property patches, as in ADR-0026: selecting
Currency sets the numeric preset and its documented precision/grouping/symbol
defaults. Bold alone changes only bold. The pane shows which fields it will apply;
untouched Mixed fields are not implicitly reset. Selecting a property and applying
it to Mixed is a write throughout the selection, not a toggle per existing cell.

No timestamps or previous formatting operations are needed for resolution. Applying
an edit replaces the relevant current spatial assignments. Removing that assignment
does not resurrect an earlier formatting action.

### Inheritance, default and reset

Each sheet has one complete worksheet-default Style, initially the ADR-0026 built-in
Style. Each selected property supports these command intents:

| Intent | Result throughout the selected geometry |
| --- | --- |
| Keep | Leave that property's assignments unchanged |
| Set(value) | Replace all assignments there with that explicit value |
| Use built-in default | Set the property's ADR-0026 built-in value explicitly |
| Inherit worksheet default | Remove that property's assignments there; use the sheet default |

Reset Formatting explicitly sets all properties to built-in defaults throughout the
selection, replacing prior cell/row/column/range formatting. A separate **Use worksheet
defaults** action removes all selected assignments. Neither changes content.
Do not label both actions Reset or silently substitute one for the other.

Worksheet-default editing is a separately labeled action in the Format pane; it
changes only inherited properties. It is not what happens when the user selects the
whole sheet and applies formatting. Whole-sheet Apply uses the same overwrite rule
as every other selection. Editing defaults must explain that explicitly formatted
positions retain their settings; the UI must not conflate these two operations.

An explicit value equal to today's worksheet default remains explicit: a later
default change must not alter it. Such a write is meaningful even if nothing looks
different immediately. No-op detection compares assignment intent and values, not
only the currently rendered Style. Reapplying the same assignment is a no-op.
There is no hidden stack of row/column/cell overrides to expose on Inherit.

## Representation and bounded normalization

Use the existing immutable workbook root and charged persistent collections.
For each worksheet and each of the ten existing Style properties, store sparse,
non-overlapping rectangles associated with an explicit property value. A position
not covered for that property inherits its worksheet default. A cell is a 1×1
rectangle; a row or column spans the current admitted opposite axis. An empty sheet
needs only its default Style. No graph identity, calculation selection, formula,
style UUID registry or formatting-operation log is allocated.

Property maps deliberately avoid a cross-product partition of all properties:
intersecting bold and background regions do not split each other. A complete-style
rectangle may use ten property records; it is still bounded by style properties,
not by the rectangle's area. Consecutive equally formatted rows/columns can coalesce.
We do not promise a unique minimum rectangle cover.

For each changed property, stage this operation on the candidate root:

1. Find existing rectangles intersecting the selection using a row-interval index
   with column filtering. Remove each intersection, retaining up to four disjoint
   outside pieces with the old value.
2. For Set/default, insert the selection with the requested explicit value. For
   Inherit, insert nothing. Unchanged properties retain their old roots.
3. Merge same-value neighbors sharing a complete edge and identical orthogonal
   extent, using a deterministic sorted horizontal pass followed by a vertical pass.
   Additional bounded passes may merge newly exposed neighbors; correctness never
   depends on reaching a minimum cover. Admit a stable valid cover within work limits.
4. Validate disjointness, bounds, value types, fragment/byte/work limits and cancellation,
   then publish all changed property maps and the workbook revision together.

Use an augmented balanced interval index keyed by first row/column and bounds,
with subtree maximum ending row. Query once for the bounded viewport, filter column
intersections and rasterize only the requested positions. Per-property rectangles
are disjoint in two dimensions, although their row intervals may overlap. Therefore
do not claim a logarithmic two-dimensional query or ignore candidate filtering cost.
Count visited index nodes, candidate rectangles, splits, merges and output writes
against explicit work admission. No scan of historical commands and no enumeration
of a whole-sheet command's 67,108,864 coordinates is permitted.

Repeated Set/Inherit cycles remove superseded spatial records. Explicit assignments
equal to the default are retained for their inheritance meaning; they are not
historical tombstones. Old workbook captures retain their own immutable maps.

### Proposed initial admission policy

| Resource | Proposed bound |
| --- | --- |
| Style properties | Existing ten; no new visual features |
| Explicit property rectangles | 65,536 total across all sheets and properties |
| Worksheet defaults | One complete Style per existing sheet, at most 16 |
| Style-owned data and indexes | 8 MiB estimated owned bytes, also inside the existing total workbook budget |
| One command's normalization work | 2,097,152 counted visits/splits/merges/comparisons; candidate scratch separately charged |
| One style query's work | 1,048,576 counted visits/filter/output operations |
| Visible style positions | Existing bounded viewport policy; at most 4,096 positions per request and existing 1 MiB frame |
| Formatting command geometry | One bounded rectangle, whole row(s), column(s) or sheet; existing worksheet extent |

These are proposed configurable resource ceilings, not measurements or a guarantee
that all maxima fit simultaneously. Replace the 4,096 styled-position/visited-position
policies for formatting only; do not raise computational, clipboard, Fill, frame,
range-consumer, evaluation or runtime-memory limits. Keep the native 64 MiB envelope.
The 65,536-fragment ceiling is a current-format engineering target. Measure actual
style/index/scratch ownership; distinguish legitimate fragmentation from inefficient
normalization before changing work or byte limits. No legacy-conversion requirement
remains and the overall runtime guard is unchanged.

Count explicit default-valued records too. Expose **format regions / region limit**,
style bytes and actionable fragmentation/work errors instead of reporting covered
positions as stored-cell counts. Style coverage may exceed computational capacity.
Reject a command atomically if its final or temporary representation exceeds a bound;
do not partially format, silently flatten into cells, or discard explicit assignments.
Alternating/checkerboard edits can exhaust fragmentation admission legitimately.
Reserve candidate/index/scratch memory before mutation, retaining live and pinned roots.

## Existing editing contracts

Formatting remains position-owned and independent of content identity. Effective
styles are resolved from a coherent root when an operation captures its source.

| Operation | Required behavior |
| --- | --- |
| Edit or replace value/formula | Retain formatting assignments at that position |
| Clear content | Retain formatting; do not promise computational identity reclamation |
| Format / Reset / Inherit | Change only presentation; no calculation revision or graph invalidation |
| Native Copy/Paste | Capture complete effective source Style per bounded slot, including blanks/defaults; write all properties explicitly at destination, replacing all overlapping assignments there |
| Fill | Capture the original source edge's complete effective styles; overwrite destination properties alongside existing formula translation; inherited source values become explicit destination snapshots |
| Existing content Move | Capture source's effective Style, overwrite destination with it, reset source to built-in default explicitly; publish styles and identity/content movement together |
| External TSV paste | Preserve destination formatting, as today |
| CSV import/export | Preserve accepted typing/leading-zero rules; no formatting in CSV and no parsing from rendered strings |

Native paste/Fill/Move operate under their existing content/area bounds. Snapshot
semantics deliberately do not transfer row/column rules or links to source defaults.
Later formatting of any destination selection overwrites these snapshots normally;
they do not become privileged cell exceptions. Cutting holes in a larger region
during a small paste is normalized/admitted atomically. Self-overlap uses the original
capture, not incrementally modified source styles. Style-only blank paste stays graph-free.
Moving style-only blanks through the content Move command remains deferred as in
ADR-0026. Worksheet rename preserves its style maps by WorksheetId.

Row/column structural insertion/deletion remains a subsequent ADR. Current formatting
geometry covers the admitted axis extent, including absent computational positions.
This design stores spatial results rather than original row/column command provenance;
it does not promise Excel's future insertion inheritance or infer it from command
history. That ADR must explicitly transform/split regions, decide inserted-axis styles
and stable-reference behavior, and preserve atomic content/style movement.

## Exact values and desktop behavior

Retain the ADR-0026 renderer and typed-value contract. Styling String `001234`,
`"001234"` as decoded CSV text, or text `00.50` never parses or truncates it.
Numeric presets on Strings and Booleans do not change their type or textual content.
Formula source, Decimal96 values, scale, rounding semantics and CSV export remain
authoritative and independent of formatted display. Zero-padding/custom numeric
format strings remain outside this feature; padding is not a substitute for preserving
source identifiers.

Add row/column header selection and explicit target indication in the existing
Format pane; name-box rectangles and ordinary selections share one command path.
Represent large selections as geometry, not selected-cell lists. Keep single-rectangle
selection; discontiguous selection is deferred. Pane Mixed detection must inspect
the complete target's property coverage/default gaps using bounded region work,
not sample visible cells. Query failure must be shown rather than reported as uniform.
Do not send all region records through a viewport frame. Existing editor commit,
stale document/tab guards, cancellation, dirtiness and immutable capture semantics remain.

## Persistence and protocol

Use **WB10 as the sole supported native workbook format**. Do not implement a WB9
reader, conversion path or migration mechanism. Update current fixtures and tests
to WB10; retain historical artifacts as evidence, not required executable inputs.
Minimal obsolete headers may remain in unsupported-version rejection tests. Preserve
stable references, identity continuity, rejected-formula behavior and atomic failed
Open within WB10. Reject every obsolete native version explicitly.

Standing development policy: until Pivot has actual external users or an explicit
file-format compatibility commitment, justified native representation changes may
be breaking. Retain a format when it accommodates a change cleanly; do not add
backward readers or migrations automatically for nonexistent consumers. Establish
a deliberate compatibility commitment before external release.

Reader validation includes worksheet ownership, checked coordinates/lengths, known
property/value kinds, per-property non-overlap, no duplicate fragments, bounded
normalization work and all aggregate quotas before installation. Overlapping malformed
records are rejected, not interpreted as an ordered program. Equivalent valid
non-minimal covers may be normalized within admission. Never persist indexes,
formatted-value caches or historical formatting actions. Failed Open preserves the
complete prior document.

Use **private protocol v8** for changed formatting intents, geometric targets,
default editing and counters; confirm the exact incompatible schema fields and cut
once during implementation. Keep generated AOT-compatible messages and the 1 MiB
frame bound. Frozen hosted v1 is unchanged. **CB4 remains**: its existing per-slot
complete Style snapshot already expresses the clipboard contract; no region rules
or inheritance metadata cross the clipboard. Preserve unsupported/malformed native
clipboard refusal without silent TSV fallback.

## Verification and delivery

Use the existing harness and short target-local attended launchers. Parent has
authorized one complete implementation slice. Deliver one concise
handoff; do not create a formatting research project or restart ADR-0023.

Required independent correctness cases:

- The owner's 10,000-cell selection with 100 prior exceptions, for numeric preset,
  bold false/true and colors. Every selected changed property must match; unrelated
  properties and outside cells must remain intact. Reverse row/column action order.
- Overlap/nesting/edge/corner/full-sheet cases, repeated Set/Reset/Inherit, explicit
  default versus inheritance after changing worksheet defaults, and no-op revisions.
  Use a simple dense per-property reference model on small adversarial grids;
  expected values must not come from the production region resolver.
- Compare random bounded edit sequences against that oracle; check disjointness,
  bounded record growth, old captures and zero graph allocation on empty sheets.
- Styled blank native paste, all Fill directions, overlapping Move, cross-sheet/window
  paste, source-default changes after Copy, CSV leading-zero preservation and typed
  numeric/Boolean behavior. Verify complete rollback of both content and styles.
- Every quota boundary and first excess; pathological splits/checkerboards, memory
  pressure with four pinned roots, cancellation/stale state, malformed/versioned
  files/clipboard, coherent viewport frames and WB10 round trips and explicit obsolete-version rejection.

Measure sequential 10,000- and 65,536-cell table formatting, whole rows/columns and
whole blank sheet formatting without per-coordinate expansion. Full-sheet complete
formatting should require at most ten property rectangles on an initially inherited
sheet. Exercise 4,096 independently styled positions, 1,024/10,000/65,536-fragment
checkpoints, overlapping stripes and repeated edits. Stop at a material failure and
diagnose it rather than increasing limits or silently changing semantics.

Proposed performance gates on native Windows x64/Linux x64/macOS ARM64: ordinary
table/whole-row/whole-column/full-sheet Apply p95 ≤100 ms; worst admitted fragmented
edit ≤1 s; bounded viewport resolution p95 ≤100 ms; native Save/reopen ≤10 s for
the formatting corpus. Record cold and warm observations, normalization work,
fragment counts, domain steady/peak charge, cumulative allocations and available
process memory separately. Retain ADR-0029 compact-corpus ≤256 MiB steady / ≤512 MiB
peak objectives for compact table-formatting workloads; fragmentation stress must
remain within existing aggregate admission and be reported separately. Numeric
calculation/interaction gates must not regress.

Automate actual header/pane/grid behavior, Mixed selections and persistence on the
three existing native targets. Cover applicable Native AOT serialization/rendering
changes with the established runner; no new infrastructure or repeated qualification
of unchanged accepted paths. Original failures and source scopes remain retained.

Parent approves the governing interaction, sparse property regions, bounded indexing,
normalization and atomic publication, initial measured admission targets, explicit
default/inheritance distinction, snapshot editing contracts, UI/default editing and
native correctness/performance/AOT coverage. Ordinary uniform operations must scale
with stored regions/intersections rather than covered area; diagnose inefficient
normalization before increasing budgets. The superseding owner direction removes
all WB9 compatibility work. WB10/private v8/unchanged CB4 apply. Deliver one concise
handoff and stop for parent implementation review; no additional design review.
