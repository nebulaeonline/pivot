# ADR-0031: Insert and delete worksheet rows and columns

Date: 2026-10-29.
Status: **Accepted and closed by parent implementation review (2026-10-30).**

See the [implementation handoff](../handoffs/2026-10-30-row-column-structure.md)
for the completed behavior, resource measurements, six-target managed/AOT results,
clipboard correction and preserved failure/retry evidence.

The parent accepts the structural-editing contract, fault provenance and normal
operand repair, range affinity and stable SelectionIds, geometric formatting,
actual-clipping notices, complete-state no-ops and atomic publication. WB11-only,
private v9 and CB5 are accepted without obsolete readers or migrations. The ARM64-
discovered clipboard correction and its later local CI/six-target managed/AOT source
qualification are accepted. All 48 clean application reports, 458 all-feature Rust
tests, strict Clippy and independent correctness/resource evidence are accepted.

Preserve two qualifications: macOS x64 ran under Rosetta, not native Intel hardware;
structural command figures are maximum single observations, not percentiles. Original
failures and successful recoveries remain separate records. No further validation,
reconciliation packet or AOT run is required. ADR-0023 remains parked at G3.

Next assigned work is ADR-0032, Worksheet Sorting, design first. Record the assignment
without starting it: stop for the owner's commit/push and explicit go-ahead. Acceptance
does not itself authorize another milestone.

Parent approves the fixed-extent, stable-identity, atomic sparse structural model,
range affinity, formatting-band inheritance, initial measured resource targets and
verification scope. WB11-only/private v9/CB5 representation changes are confirmed
and accepted; no obsolete readers or
migrations are assigned. The three binding clarifications are fault provenance,
notice of actual discarded formatting and complete-semantic-state no-op detection.
The final owner/parent discussion retains the original no-automatic-rebinding rule
and explicitly withdraws the intermediate Repair References proposal. Normal formula
editing is the repair workflow. Details below incorporate that final disposition.

The owner has authorized implementation and confirmed that platform updates are
complete, lifting the earlier ARM64 execution hold. Managed and Native AOT GUI
validation must include Windows x64, Linux x64, Windows ARM64, Linux ARM64,
macOS ARM64 and macOS x64 under Rosetta, using normal attended consent and short
local launchers. Follow the
design → parent review → authorized implementation → parent comments → docs →
commit/push → explicit owner go-ahead cadence in [STATUS](../STATUS.md).

## Context and scope

ADR-0030 is accepted and closed. Formatting now consists of sparse property regions,
not row/column operation history. Content binds to stable CellId/GraphNodeId values;
range operands bind to SelectionId plus rectangular geometry. Worksheets have a
fixed admitted extent, normally 65,536 rows × 1,024 columns. Production Undo/Redo
does not exist and ADR-0023 remains parked at G3.

Implement four whole-axis operations in this milestone: insert rows
before a selected row, delete rows, insert columns before a selected column, and
delete columns. Support one contiguous band, on one explicit WorksheetId, per
atomic command. Keep worksheet count, coordinate/cell/identity/resource limits
unchanged. Do not add shifting individual cells, structural changes across grouped
sheets, row heights/column widths, sorting, filtering, tables, merges or history.

Source inspection identifies three important constraints:

- `Axis::at` currently derives AxisId from namespace and coordinate sequence. It is
  not a stable identity for a movable logical row. Preserve stable cell identities;
  do not invent row-lineage guarantees or an axis-identity allocation project.
- `Edit::Move` refuses an occupied destination and snapshots formatting. Thousands
  of those edits are not a structural operation; temporary collisions and repeated
  graph/style publication must not become visible.
- Core selection `define` refuses different geometry for an existing ID, while
  referenced retirement can invalidate an ordinary calculation capture. Structural
  edits need a narrow atomic geometry-update primitive and local reference-fault
  handling; increasing constants or reparsing formulas will not supply either.

## Fixed extent and position transformation

The worksheet keeps its admitted extent **L**. Insertion shifts surviving positions
toward the end; deletion shifts survivors toward the start and exposes blank tail
coordinates. New blank bands allocate no computational identities. Rows and columns
obey the same one-dimensional mapping; the orthogonal coordinate is unchanged.

For start **p** and positive count **n**, require `1 ≤ p ≤ L` and
`n ≤ L-p+1`, using checked arithmetic. Invalid counts/coordinates fail atomically.

| Operation | Old coordinate x | New coordinate |
| --- | --- | --- |
| Insert before p | x < p | x |
| Insert before p | p ≤ x ≤ L-n | x+n |
| Insert before p | x > L-n | Would leave the admitted extent; see admission below |
| Delete p through p+n-1 | x < p | x |
| Delete p through p+n-1 | p ≤ x < p+n | Removed |
| Delete p through p+n-1 | x ≥ p+n | x-n |

Insertion must refuse if **any positioned computational identity** would be shifted
past L, including Blank reference anchors and cleared positions. Never silently drop,
retire or recycle those identities to make room. Name the affected edge/positions in
the error; Clear alone does not free them. A selected coordinate without a graph
anchor is not occupied merely because it was selected or visible.

Formatting alone does not make the outgoing tail computationally occupied. Proposed
edge policy: transform and clip presentation regions at the fixed worksheet boundary,
including explicit-default assignments. Full-column/full-sheet formatting therefore
does not make every row insertion impossible. This can discard distinct formatting
that would move beyond the edge; it is a deliberate presentation-boundary policy,
not permission to discard values or bindings. Report presentation clipping in the
operation result/status. Parent approves this tradeoff with the requirement that
the result distinguish **actual discarded explicit assignments** from ordinary
remapping, splitting or coalescing. Show a clear post-operation notice when clipping
occurs; no elaborate preview or automatic repair dialog is assigned.

Deletion removes the targeted content, including positioned Blanks, and retains
their retired identity records. Allocated-identity counters do not decrease; positioned
counts can decrease. Survivors retain CellId, portable GraphNodeId, runtime identity,
literal representation and bindings. Coordinate-derived AxisIds update to match their
new positions; their namespaces remain unchanged. No row/column UUID allocation,
identity reset, compaction or rebinding is introduced.

Deleting the entire axis leaves the same extent, with blank unpositioned coordinates
and worksheet-default inheritance. It does not delete the sheet. An operation with
no change to content, assignments, references or range geometry is a semantic no-op:
no hidden structural-history entry or workbook revision is created. Compare the
complete resulting semantic state, including range descriptors, fault provenance
and explicit formatting/inheritance assignments, not just moved-cell counts.
Range-only and style-only changes are real changes even with zero moved cells.
The result/UI must distinguish a semantic no-op from a successful mutation.

## Scalar references and deleted-reference faults

Surviving formulas keep references to the same surviving cell identities, whether
the formula, its target, both, or neither move. This applies across worksheets.
`$` markers control subsequent Copy/Fill translation; they do **not** prevent a
structural edit from moving a target. Formula source remains authored provenance;
effective display follows current bound coordinates and retains relative/absolute
markers, qualifiers and quoted sheet names.

| Before | Action | Required result |
| --- | --- | --- |
| B1 contains `=A1`; A1 contains 7 | Insert row before 1 | Formula moves to B2, still targets the original cell now at A2 |
| C5 contains `=$A$2`; A2 contains 7 | Insert two rows before 2 | Target becomes A4; effective formula is `=$A$4` |
| Other!B1 targets Sheet1!A2 | Delete Sheet1 row 1 | Same target now at Sheet1!A1 |
| B5 targets A2 | Delete row 2 | Surviving formula refers to a permanently deleted identity |

A deleted target produces a **ReferenceDeleted / `#REF!`** operand fault. It does
not bind to the replacement occupant, become Blank/zero, or recover after inserting
a new row or writing the same coordinate. The original target remains retired in
the identity ledger. Deleting an already faulted formula retires it normally.

This is a broken binding, not a permanent defect in the cell containing the formula.
The formula bar/in-place editor displays the `#REF!` operand and lets the user
replace just that operand with a valid reference, without clearing the entire cell
or rewriting unrelated parts. For example, `=SUM(A1,#REF!,C1)` can be repaired by
replacing `#REF!` with `B1`. A corrected submission creates ordinary new bindings;
replacing the cell's entire formula or entering a literal also replaces the fault
normally. Merely entering content at the deleted target's old address does not.

No automatic reconnection, repair prompt, contextual suggestion or bulk Repair
References command is included. The parent proposed such a facility during the
discussion and withdrew it after the owner's clarification. Do not implement that
superseded suggestion or describe faults as unrepairable.

Distinguish fault origins internally:

- A structurally generated fault retains the retired cell/range identity, worksheet
  and last known coordinate/geometry where applicable. Those locations are provenance
  at deletion time, not live address bindings or instructions to find a replacement.
- An explicitly authored `#REF!` is a fault without an invented deleted target.
- A transferred fault remains a fault; cross-workbook provenance cannot claim a
  local identity relationship or cause foreign identities to be allocated.

Rendering, an unchanged edit/commit, Save/reopen and appropriate Copy/Fill/native
clipboard operations preserve this distinction. An unchanged formula-bar submission
must not round-trip through plain text in a way that loses structural provenance or
rebinds other references. Explicitly repairing an operand is a new authoring action.

This requires scoped canonical expression support: represent a reference fault as
a valid, immutable error-producing operand, with workbook provenance where available.
Replace the affected dependency/operand through the trusted structural candidate;
do not leave a dangling executable node edge or turn the whole formula into a
parse failure. Unrelated outputs must remain calculable. Preserve lazy branches:
`=IF(FALSE,#REF!,1)` returns 1, while an evaluated deleted-reference operand faults
and propagates according to existing error rules. COUNT does not silently skip a
reference fault as if it were nonnumeric text.

The worksheet expression front end must recognize `#REF!` as an error operand outside
quoted Strings, so rendered broken formulas remain editable and transferable.
String `"#REF!"` is ordinary text. No general error-value data type, additional error
literals or changes to hosted-v1 parsing are assigned. An unchanged editor commit
preserves existing bindings. Explicit resubmission is a new authoring action and
may bind repaired references; structural transformation itself never does so.
Previously rejected formulas stay rejected until explicitly resubmitted, even if
movement makes their old source appear valid. Never reparse every authored source
against the new coordinates as the transformation algorithm.

## Range references and endpoint affinity

Range references remain rectangular selections of **current occupants**, rather
than frozen member lists. Transform descriptors on the affected sheet even when
the consuming formulas live elsewhere. Preserve SelectionId for a surviving range,
update every shared consumer/cache consistently and retain old geometry in old roots.
Introduce a trusted structural reframe primitive; ordinary `define` must continue
rejecting conflicting redefinitions. Do not retire/reallocate a live selection on
every insert, which would exhaust the existing lifetime selection-ID budget.

For an inclusive interval `[a,b]` on the edited axis, insertion affinity is:

- `p ≤ a`: shift both endpoints by n; inserted positions are before the range.
- `a < p ≤ b`: extend the last endpoint by n; inserted positions are inside.
- `p > b`: leave the interval unchanged. Insertion immediately after b is outside.

Clip the transformed last endpoint at L. A shifted range with no coordinates left
inside the admitted extent becomes a reference fault. No surviving positioned
members may be lost through insertion because outgoing positioned identities
already block the operation. Empty boundary geometry can change without allocating
anchors merely to remember endpoints. Apply the same rules to reversed authored
endpoints while retaining their orientation and `$` markers in effective rendering.

Deletion computes the old range's surviving coordinates, maps them through the
deletion transform and takes their resulting contiguous interval. It shrinks or
shifts accordingly; replacement blank tail coordinates are not added as members.
If none survive, the operand becomes `#REF!`, not an empty rectangle or numeric zero.
Retire its selection descriptor only after all affected consumers are replaced;
retain the retired ID and fault provenance without recycling it.

Examples for `SUM(A2:A5)`:

| Edit | Effective range/result |
| --- | --- |
| Insert row before 2 | A3:A6; inserted row is outside |
| Insert row before 4 | A2:A6; inserted blank is inside |
| Insert row before 6 | A2:A5, unchanged |
| Delete row 3 | A2:A4, retaining surviving original members |
| Delete rows 2–5 | `SUM(#REF!)`, a reference fault |

Apply this contract to every range argument, nested aggregate, multi-argument call,
cross-sheet reference and mixed scalar/range expression. Scalar references to
individual deleted cells fault even if a containing range still has survivors.
Canonical cycle/error/edge/invalidation checks remain in force; no resource limit
or graph-validation bypass is implied. Existing authored errors remain diagnosable.

## Formatting transformation and new blank bands

Transform all explicit property assignments geometrically, retaining their values
and explicit-versus-inherited intent. This is not effective-style snapshot Copy or
content Move: do not flatten inherited formatting or run a formatting action once
per moved cell. Worksheet defaults remain unchanged.

For insertion, map existing regions around the gap. Populate the new band's
formatting from the **pre-edit immediately preceding row/column's assignment profile**;
at coordinate 1, use the pre-edit first row/column. All n new coordinates receive
that profile, but no values or formulas. Copy property intervals, including explicit
default masks; inherited gaps remain inherited. Read the profile from the original
root once, not from incrementally inserted rows. Insertion inside a uniform region
therefore keeps that formatting continuous without enumerating columns/rows.

For deletion, remove assignments in the band and map surviving regions upward/left.
The exposed tail receives the last surviving row/column's assignment profile from
the original root. If there is no survivor, the tail inherits worksheet defaults.
This preserves a uniform full-column format through row deletion, while respecting
the same per-property intent. Uniform formatting is not inferred from historical
row/column commands, which ADR-0030 does not retain.

Normalize/coalesce within existing region/byte/work admission. A boundary split may
increase fragmentation and legitimately cause an atomic refusal. Retain the latest-
action-per-property rule for subsequent user formatting: transformed assignments do
not acquire special precedence. Styling virtual new bands allocates no CellIds,
GraphNodeIds or calculation selections. Explain inserted/tail formatting inheritance
in the operation help; no automatic content copying is performed.

## One candidate, one publication

Add a structural request containing exact Expected, WorksheetId, axis, action, start
and count. Do not expand it into an unbounded wire list of ordinary edits. Build a
charged sparse plan from positioned records, reference occurrences, selection
descriptors and property regions. Validate checked coordinates and outgoing-tail
content before publication; do not scan all 67 million virtual sheet positions.

Stage one canonical graph/workbook candidate, removing old placements and installing
new placements as a set. Occupied intermediate coordinates must not cause failure
or overwrite another survivor. Retire deleted identities, install fault operands,
reframe selections, update membership/consumer indexes and transform presentation
under the same transaction authority. All old captures retain their original cells,
coordinates, range geometry, styles and values.

The existing 256-edit interactive bound remains. Reuse or narrowly extend the
trusted bulk-candidate path, with cancellation and canonical validation; no partially
published batches, save/reopen reconstruction, second graph authority or history
coordinator. Publish one workbook revision and the necessary graph revision only
after all content, reference and style admission succeeds. Failure preserves dirty
state, identity counters/floors, graph, styles, selection and the prior document.

Propose a separate initial structural work ceiling of **33,554,432 counted units**
for sparse traversal, comparisons, remaps, operand/descriptor updates and validation.
This is a measured policy target, not a way to raise ordinary edit/format budgets.
Keep 65,536 positioned cells, 262,144 allocated identities, selection/edge limits,
65,536 style regions, 8 MiB style data, total workbook/runtime admission, 1 MiB frames
and 64 MiB native files. Charge scratch, candidate and four retained roots. Diagnose
inefficient sorting/repeated validation before proposing any limit change.

## Desktop, clipboard and persistence

Expose Insert Rows Above, Delete Entire Rows, Insert Columns Left and Delete Entire
Columns through the existing Edit/header menus. Labels state the selected band count;
ordinary rectangular selections use their row span or column span for the explicitly
chosen command. Delete is distinct from Clear contents. Do not imply Undo is available.
Commit an active editor first; an invalid/uncommitted draft prevents the operation.
Invalidate stale frames, pending paste targets and Format-pane summaries by revision.

After insertion select the inserted band and focus its first coordinate; after
deletion focus/select the band starting at p within the unchanged extent. Preserve
the orthogonal focus coordinate and a clamped viewport; whole-column selection must
not jump to the bottom row. Other worksheets retain their view positions while
their affected formula displays/results refresh. No operation edits the OS clipboard.

Copy/Fill after a structural edit uses current effective coordinates and retained
`$` flags. Native transfer must preserve `#REF!` operands rather than rebinding them
or rejecting an otherwise useful whole block. A cross-workbook paste carries the
fault without allocating foreign identities. A previously captured clipboard image
remains a snapshot: a later explicit paste uses its captured coordinates under the
existing rebasing contract, not a live link to the structurally edited source.

Propose **WB11 only** for reference-fault operands/provenance and structural range
invariants, **private v9** for commands/results/reference-error presentation, and
**CB5** for portable fault-bearing formula snapshots. Geometry shifts alone do not
justify a format cut; the new bound fault representation does. Confirm exact fields
and cut once during authorized implementation. Reject obsolete native/clipboard
versions explicitly; no WB10/CB4 reader or migration is assigned. Preserve the frozen
hosted-v1 contract and map internal faults to its existing error boundary if necessary.

Readers validate surviving and retired identities, fault provenance, descriptor/
consumer consistency, current coordinates, style ownership and all quotas before
installation. A newly written occupant must never heal a persisted fault on reload.
Save/Open and failed installation retain their accepted atomic/uncertain-outcome
contracts. No structural command log or axis-lineage registry is persisted.

## Verification and review gates — future authorized work

Use existing harnesses and short attended launchers for the authorized implementation.
The earlier ARM64 execution hold has been lifted by the owner.

Use independent coordinate-map, survivor-ID, range-membership and dense per-property
formatting oracles on small grids. Verify both axes, start/interior/end bands,
multiple-count and whole-axis operations, reversed endpoints, `$` combinations,
cross-sheet dependents, Blank anchors, style-only bands and insertion/deletion pairs.
An insert/delete pair is not Undo: it must not resurrect deleted identities or faults.

Required cases include:

- Exact scalar target/graph identity continuity; deletion faults; new occupants at
  old coordinates; unrelated and lazy-branch evaluation; nested/multi-range aggregates;
  entirely/partially deleted ranges and fixed-edge clipping.
- All ten style properties; explicit default versus inheritance after changing sheet
  defaults; predecessor/first-coordinate inheritance, surviving-tail inheritance,
  full-column/full-sheet formatting and distinct outgoing-edge presentation clipping.
- No fresh computational identity for insertion or virtual formatting; deletion
  retains allocated counters; range reframing does not consume new SelectionIds;
  old captures and prepared exports remain coherent through later edits.
- Colliding placements, protected tail identities, exact and first-excess quotas,
  fragmentation/scratch limits, stale requests and mid-plan/graph/style cancellation.
  Every late failure must leave the entire original state and UI selection intact.
- Formula editing/repair, post-structure native Copy/Paste/Fill/Move, fault-bearing
  cross-window/workbook clipboard, exact Strings such as `001234`, CSV boundaries,
  complete WB11 round trips and explicit obsolete/malformed input refusal.
- Generated versus authored fault provenance through unchanged formula-bar commits,
  rendering and persistence; repair of just one `#REF!` operand; replacement of the
  whole formula/literal; no repair caused by writing at the old target coordinate.
  Cross-workbook transfers retain faults without fabricating local retired targets.
- Zero-moved-cell changes to ranges/styles/inheritance still create revisions;
  true complete-state no-ops do not claim a mutation. Clipping status is raised for
  actually discarded explicit assignments, not merely remapped/coalesced regions.

Measure sparse 1,024/10,000/65,536-position checkpoints, both prescribed numeric
tables, cross-sheet/range-heavy cases and 65,536-region formatting stress. Include
shifts across the entire admitted coordinate extent with very few actual records;
cost must follow stored state/intersections, not virtual area. Stop at a material
failure and diagnose it before a larger case or budget change.

Proposed gates: compact-table structural commands ≤1 s, admitted fragmentation stress
≤2 s, post-command local edit/viewport p95 ≤100 ms, capture p95 ≤5 ms, defined recalc
≤2 s and native Save/reopen ≤10 s. Retain compact-corpus ≤256 MiB steady / ≤512 MiB
charged peak targets; separately report identity/fragmentation stress within existing
runtime admission. Record actual remapped/retired records, descriptor/operand/style
changes, normalization work, memory/allocation traffic and available process memory.
Cover actual menu/header/editor/selection in managed and Native AOT runs on all six
configured targets named above; distinguish macOS x64 emulation from native evidence.

Parent design review approves fixed extent/protected computational tails, presentation
edge clipping and band inheritance, stable cell versus coordinate-axis identity,
scalar/range deletion faults and affinity, one atomic sparse structural primitive,
initial resource targets/format direction and the bounded future verification scope.
Fault provenance, actual-clipping notice and complete-state no-op detection are
required. Normal editing of `#REF!` provides repair; the intermediate automatic/bulk
repair proposal is withdrawn. Implementation, validation and ARM64 execution are now
authorized. After implementation and validation, deliver
one concise handoff and follow the standing milestone cadence.
