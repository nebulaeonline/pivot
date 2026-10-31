# ADR-0035: Visible-only Clear and Fill

Date: 2026-10-31.
Status: **Design approved with clarifications by parent review (2026-10-31).
Implementation requires separate owner authorization.**

## Context

ADR-0034 is accepted and closed. Its temporary actual-row plans and exact-operation
review provide a foundation for explicit visible-only editing. This ADR proposes
Clear and four-direction Fill without changing ordinary Clear, Fill, shortcuts,
selection geometry or calculation membership. The owner authorized design after the
ADR-0034 documentation closeout. No feature implementation or platform execution is
authorized by this record. ADR-0023 remains parked at G3.

Current Clear sets the active cell's content to Blank, retains its positioned identity
and does not change formatting. Ordinary Fill takes the selected rectangle's top,
bottom, left or right edge from one immutable capture and duplicates it directly to
the aligned destinations. It includes resolved styles, translates formulas by actual
coordinates, preserves destination identities and rejects candidate cycles atomically.
Ordinary Fill's selected area is bounded at 256 cells; ordinary session APPLY is
bounded at 256 edits. Neither boundary should silently become a visible Clear limit
or be raised for unrelated callers.

ADR-0030 uses latest-assignment property maps, not cell/row/column precedence layers.
Reset Formatting explicitly assigns built-in defaults; Use worksheet defaults removes
assignments and inherits the worksheet default. Those distinct meanings must survive
visible Clear. Selected destinations must not retain exceptional per-cell formatting
that forces the user to hunt down and reset individual cells afterward.

## Decision

The parent approves one coherent design with **two implementation slices: Clear first,
Fill second**. Both slices require subsequent owner implementation authorization.
There is one ADR-0035 implementation milestone,
one final parent implementation review and one commit/push boundary, with Clear
implemented and verified internally before Fill. Splitting the delivery milestones
requires an explicit parent/owner decision; neither slice is accepted independently
by this design. Fill adds trusted-source translation and cycle obligations that Clear
does not need, so keep their planners and bounded work contracts distinct.

### Explicit commands

Add an Edit > Clear Visible Cells submenu with these actions:

| Action | Content | Formatting |
| --- | --- | --- |
| Clear Visible Contents | Set existing content to Blank | Preserve every assignment and worksheet default |
| Clear Visible Formatting (Built-in Defaults) | Preserve typed content and bindings | Assign all ten built-in default properties to every addressed cell |
| Clear Visible Contents and Formatting | Set existing content to Blank | Same complete built-in reset |

The default Clear choice is Contents. No Delete/Backspace shortcut is reassigned.
Existing active-cell Clear/Delete behavior remains intact, including its established
editor behavior. Visible commands do not secretly commit, cancel or discard drafts.

Formatting Clear deliberately means ADR-0030 Reset Formatting, not Inherit. It replaces
selected cell/row/column/range assignments property by property, including exceptional
individual-cell formats. The review labels **Built-in defaults** and explains that
worksheet defaults are unchanged. Explicit default assignments remain explicit if
the worksheet default changes later. There is no second visible Inherit command in
this milestone; the ordinary Use worksheet defaults action retains its existing scope.
An explicit default-valued assignment can be a real change even if its current display
matches an inherited value. Do not use display equality as a no-op test.

Add a separate Edit > Fill Visible Cells submenu: Down, Up, Right, Left. It copies
both typed content and complete effective source styles. There is no series mode,
user-selected source block, values-only mode, tiling pattern or clipboard dependency.
Ordinary Fill and its existing shortcuts retain their ordinary full-rectangle semantics
and hidden-row confirmation. Visible commands have no new default shortcuts.

### Selection, qualification and membership

Keep one ordinary inclusive rectangle, anchor and active position. Normalize its
bounds without changing its orientation. From one pinned qualified view, derive
R = [r0, ..., rn-1], the ascending actual worksheet rows within that rectangle that
are shown by the current filter. C is its contiguous selected column interval.
Visibility is not viewport inclusion: offscreen shown rows participate. Visible
headers and shown rows outside the filter data interval participate if selected;
never silently omit headers, expand selection or stop at a filter boundary.

Only no-filter/all-shown and current successfully applied filter states qualify.
Pending, Failed, structural NeedsReview, stale revision/computational state or view
generation, projected records, active drafts/editors, drag selection and pending
mutations block these commands. A fallback that shows all rows after filter failure
is not a qualified map. Zero selected visible rows refuses with recovery guidance;
no operation may substitute the full rectangle. A single-cell selection stays a
single-cell selection; Clear and Fill never expand from it as Paste Visible does.

The temporary row plan is presentation/command metadata, not a computational
SelectionId, saved multi-area selection or new range kind. Calculations, references,
SUM and other aggregates retain ordinary worksheet membership, including hidden cells.

### Clear semantics and identities

Address exactly R x C, including currently Blank and absent positions. Clear Contents
reuses canonical Edit::Clear semantics: existing formulas, rejected drafts, typed
values and empty Strings become Blank; existing CellIds, GraphNodeIds and positioned
Blank anchors remain. Dependents observe the ordinary Blank result at that identity.
Absent positions remain absent and do not allocate identities. Clear does not promise
to recover positioned-cell or lifetime identity quota. A formula in a hidden row may
recalculate because its visible input was cleared; that is not a direct hidden edit.

Formatting-only Clear is graph-free: do not create a cell/anchor, rewrite definitions,
rebind references or invalidate calculation solely for presentation. It resets all
properties at addressed positions, including absent/Blank cells. Both stages of Clear
Contents and Formatting publish together or neither publishes. Styles outside the
actual addressed points, including hidden rows within the bounding rectangle, are
unchanged. Worksheet defaults and filter definitions are unchanged.

An exact no-op retains workbook revision, saved checkpoint, dirty state, graph and
selection identities, style representation and ownership. A content or style change
is one document transition; use existing content-versus-presentation computational
state rules. No garbage collection, identity recycling or row deletion is implied.

### Fill mapping

All sources and destinations are in the same selected ordinary worksheet. Sources
must be visible and are read from one immutable pre-operation capture. Compute pairs
directly from that capture; never feed newly written output into the next destination.

| Direction | Sources | Destinations and mapping |
| --- | --- | --- |
| Down | First visible selected row r0, all columns C | Each later visible row uses its aligned source column in r0 |
| Up | Last visible selected row rn-1, all columns C | Each earlier visible row uses its aligned source column in rn-1 |
| Right | Left selected column, independently in each row of R | Each later selected column uses that row's left-edge cell |
| Left | Right selected column, independently in each row of R | Each earlier selected column uses that row's right-edge cell |

A hidden geometric top/bottom boundary is not a source. All hidden rows are excluded
in every direction. Sources are not written. Vertical Fill with one visible row and
horizontal Fill with one column has no destinations: after state/shape/admission
validation and exact review, consume the operation as a no-op. Zero visible rows is
still refusal. The source strip is one row/column, not a repeating multi-row seed.

Example: selected B2:C12, shown rows [2,5,12]. Down uses B2:C2 for both B5:C5 and
B12:C12. Up uses B12:C12 for rows 2 and 5. Right uses B2, B5 and B12 independently
to fill C2, C5 and C12. No row between those actual rows is written.

Use ADR-0022 trusted same-document duplication, not CB6 encode/decode. Preserve exact
Decimal96 including signed zero, Boolean, String (including leading zeros and tabs),
Blank/empty-String distinction, structured output bindings, rejected non-executable
drafts and fault provenance under the same rules as ordinary Fill. Blank sources
clear destination content without materializing absent positions, while their complete
resolved styles still replace destination formatting. Equal displayed results do not
establish definition/binding/style equality.

Each formula uses delta = actual destination coordinate minus actual source coordinate.
Relative axes translate; absolute axes retain ordinary semantics. Range kinds, sheet
qualification, deleted-reference annotations and structured output identities follow
existing trusted Fill rules. In particular, filling B5 containing =A2 down to B12 produces
=A9, regardless of which intermediate rows are hidden. Sources/targets need not be
visible for referenced formulas to retain ordinary calculation meaning. Do not rewire
cross-record references or treat gaps as dense coordinates. Preserve ordinary Fill's
out-of-bounds refusal and cycle-only candidate dependency validation. Unrelated invalid
drafts remain leaves, not a reason to evaluate the workbook during Fill.

### Hidden-row integrity and atomicity

No direct content or style write may target a hidden row. Source capture is immutable;
only explicit destination pairs may change. Existing canonical reference binding may
resolve hidden cells or demand-create ordinary hidden Blank anchors; do not suppress
those anchors or claim all hidden graph records are bit-for-bit unchanged. Tests must
separate permissible reference demand and derived recalculation from forbidden direct
writes. Ordinary cell references and computational identities are not visibility-aware.

Use one cancellation-aware canonical candidate transaction per command, including
Clear Both and content-plus-style Fill. Validate state, bounds, counts and admission
before staging; poll row-map traversal, content planning, style splitting/coalescing,
dependency work and installation. Stage all identity-ledger, graph, style and document
changes privately. A failure or pre-install cancellation publishes no partial state,
revision, checkpoint, dirty change, identity reservation or filter change. Do not loop
through independent committed APPLY/FORMAT calls or split an oversized command into
smaller hidden commits. Unchanged destinations do not demand fresh identities.

Post-install filter recomputation uses existing coherent publication. Never recompute
membership halfway through a command: edits that hide a destination/source cannot
change the already reviewed write vector. Preserve selection orientation, scroll and
active cell when visible; use normal filter relocation/recovery when it becomes hidden.
Formatting-only changes retain the existing computation/filter behavior. Cancellation
after dispatch drains/acknowledges an already committed result and never replays.

### Review and stale-state binding

Every visible Clear/Fill invocation requires an explicit exact-plan review, even with
zero skipped rows or a degenerate no-op. Cancel is the default. Show worksheet, action
and Clear mode or Fill direction, original selection, actual addressed/source/destination
row runs and columns, source/destination slot counts, skipped hidden rows and applicable
limit. For Clear show complete formatting-reset meaning; for Fill show the actual
source edge and actual-coordinate formula rule. Selected source rows and written
rows must be distinguishable. Hidden boundary rows count among skipped rows.

Review counts describe logical addressed slots, not nonempty cells or changed values.
Clear has N*W addressed slots. Vertical Fill has W source slots and (N-1)*W destination
slots; horizontal Fill has N source slots and N*(W-1) destinations. Skipped rows are
selection height minus N, including hidden edges. Show an explicit No destinations
message for degenerate Fill. Bound the review and make every actual row run accessible
in a scrollable accessible control; never abbreviate away undisclosed affected rows.

Prepare stores one bounded, charged one-use plan tied to exact immutable command
bytes, worksheet, rectangle, mode/direction, expected revision/computational state,
qualified filter generation, source capture identity for Fill, and ordered actual
rows/pairs.
The client additionally binds selection/anchor/editor/interaction state. Any change
requires a new review; no silent replan. Consume the ticket on dispatch, including
refusal/no-op; a new prepared mutation invalidates other outstanding confirmation
tickets. Closing, changing document/sheet, cancellation and replacement release plans.
No OS clipboard read, write or revalidation is involved. Direct callers without a
valid ticket refuse. No automatic action retry or consent bypass is permitted.

### Admission and initial product limits

| Boundary | Approved initial policy |
| --- | --- |
| Clear addressed geometry | At most 4,096 visible logical slots N*W, including absent/Blank cells, for every Clear mode |
| Fill addressed geometry | At most 256 visible logical slots N*W including sources, for every direction, including degenerate shapes |
| Actual row bounds | Existing 65,536 rows and 1,024 columns; bounding area may exceed addressed-slot limit |
| Plan/review storage | At most 4,096 actual rows for Clear, 256 for Fill; coalesce row runs; charge within existing 4-MiB response/plan admission and 1-MiB frame |
| Command transient planning | At most 8 MiB, including owned row/pair/edit buffers; charge candidate graph/style ownership separately under existing limits |
| Fill source/text work | Existing 128 KiB source/text, 16,384 tokens, 8,192 references; count repeated-source work |
| Clear source/text work | No formula parsing/translation; bounded position lookup and existing graph invalidation work, not borrowed clipboard byte limits |
| Styles | Existing 65,536 property regions, 8 MiB style data, 2,097,152 normalization work; no per-run budget reset |
| Canonical/runtime admission | Existing positioned/lifetime cell, graph/selection, 96-MiB workbook and runtime budgets; no increase |

Clear's 4,096-slot bound is an approved command work contract, not a reinstated styled-
position storage limit or a limit on ordinary range formatting. It is sixteen times
clipboard capacity, accommodates common table cleanup and permits a 4,096-row sparse
selection spanning full worksheet height. It deliberately does not promise a 65,536-row
unfiltered column clear or a 10,000-cell selected Clear in one operation. Refuse such
commands before staging and explain the bound; do not truncate or auto-chunk. This is
an explicit accepted product limit. Larger Clear needs separate measurements/design,
not an inferred clipboard limit or a silent fallback. All three Clear modes share the
same limit to keep the contract predictable. Ordinary formatting keeps full-sheet
region geometry under ADR-0030.

Count rows and checked products before per-cell allocation; traverse at most the
existing full-height map. Prefer coalesced spans/property-batched normalization for
resetting styles, with one accumulated work meter for all ten properties and all runs.
Do not repeat whole-map normalization per cell/run or broaden over hidden gaps. The
parent accepts the initial 4,096-slot ceiling; a passing benchmark does not authorize
raising it.
If the worst fragmentation/candidate workload cannot fit existing admission, refuse
atomically with an actionable work/fragmentation error; do not discard styles, apply
values only, clear broader regions or raise limits to make it pass.

### Private protocol and persistence

The parent approves private session v13 with typed PREPARE_VISIBLE_EDIT, CLEAR_VISIBLE and
FILL_VISIBLE operations, plus a required ClearMode enum (Contents, Built-in Formatting,
Both; unspecified rejected). Reuse typed FillDirection and bounded row-plan metadata
where meaningful, with explicit source/destination counts and run descriptions. Reserve
actual field/opcode numbers during implementation; do not overload Paste payloads,
change APPLY's 256-edit cap or let old binaries interpret these as ordinary commands.

Core Rust owns row plans and canonical mutation; C# routes commands and exact review,
without formula parsing, style inference or per-cell edit fan-out. The transaction seam
must accept bounded Clear edits plus staged property-map resets in one installation;
merely composing existing independently publishing helpers is insufficient. Trusted
Fill can reuse duplicate planning once supplied exact source/destination pairs.

No WB12, hosted v1, CB5 or CB6 change is proposed. Temporary plans/tickets are not saved.
Current readers already represent Blank, exact values, styles and formulas. No migration,
obsolete reader or history schema is required. Existing command behavior and rejected
unknown-operation behavior remain explicit compatibility checks.

## Alternatives considered

| Alternative | Disposition |
| --- | --- |
| Change Delete and ordinary Fill to skip hidden rows | Reject: changes established operations and shortcut meaning |
| Clear through repeated 256-edit requests | Reject: partial publication, stale membership and multiple document transitions |
| Use clipboard's 256-slot cap for Clear | Reject: Clear has no encoded source image and no translation work |
| Initially permit full-sheet Clear | Defer: content/graph invalidation costs differ from region-only formatting; qualify bounded Clear first |
| Clear formatting by Inherit | Reject for these labels: exposes worksheet defaults instead of the specified built-in reset; keep existing separately named Inherit |
| Repeat a multi-row pattern or fill each contiguous visible run independently | Reject: ambiguous seed/restart semantics; use one visible edge and immutable direct pairs |
| Translate vertical formulas by visible ordinal or rewire references | Reject: violates actual-coordinate reference semantics |
| Implement Clear and Fill as unrelated simultaneous paths | Reject: shared review/qualification should be coherent; distinct implementation slices reduce coupling |

## Consequences

Explicit commands retain predictable ordinary behavior and protect hidden rows while
allowing exact typed and formatted editing. Content/format distinctions and source
edges are reviewable. A bounded Clear command is useful beyond clipboard scale but
requires an honest product limit. Batched style resets and a combined transaction
seam require core work; Fill adds candidate-cycle and trusted binding coverage.
No new capacity claim, history feature or computational multi-range abstraction follows.

## Verification / reconsideration trigger

This is source-based design, not new benchmark or execution evidence. After design
approval and owner implementation authorization, require:

- Independent row-mask oracles, not the production planner: all small masks, hidden
  boundary/source rows, contiguous/gapped/full-height rows, headers/outside filter
  interval, zero shown rows and all four directions. Check exact counts and pair sets.
- Clear modes over formulas, rejected drafts, exact signed zero, Boolean, leading-zero
  Strings, empty Strings, positioned Blank anchors and absent positions. Observe stable
  identities/dependents and no identity materialization from content/format Clear.
- Worksheet defaults unlike built-in defaults; overlapping row/column/range and isolated
  cell assignments. Every selected visible property resets; hidden/unselected assignments
  stay exact. Explicit default versus inherited no-op semantics and later default changes.
- Fill trusted values, Blank/style transfer, rejected drafts, structured output identities,
  absolute/mixed/range references, deleted-reference provenance and actual gap offsets.
  Use Down/Up and Left/Right independent expected formulas; sources never change.
- Hidden-row sentinels for content, styles and identity; separately permit demanded Blank
  reference anchors and normal dependent recomputation. Never assert false graph isolation.
- Exact no-ops, cycle/out-of-bounds refusal, late graph/style/identity quota failure,
  style-fragment exhaustion, cancellation before installation and after dispatch, stale
  revision/state/map/interaction, changed mode/direction/source, repeated ticket and
  document-close release. Compare roots, bindings, ledgers, dirty/checkpoint and charges.
- Clear at 4,095/4,096/4,097 addressed slots; Fill at 255/256/257 including source edges,
  wide/tall degenerate cases and sparse full-height bounding boxes. No truncation or
  partial auto-chunking; no budget reset between runs or properties.
- Ordinary Clear/Delete, ordinary Fill shortcuts, existing hidden-row confirmation,
  CB5/CB6 interchange and WB12 reopen remain unchanged. Save/reopen preserves resulting
  styles/definitions without transient plan metadata.
- Native resource checkpoints on bounded 4,096-slot Clear and 256-slot Fill, values/
  formulas/styles, dense and sparse gaps, four retained captures, at least 16,384 base
  positions (declare any extra reference anchors), and near-limit fragmented styles.
  Measure plan/review construction separately from atomic commit and filter refresh.
  Record observed maxima with sample counts, not percentile claims. Retain existing
  1-second engine-operation and 256-MiB steady/512-MiB peak objectives on stated corpora;
  do not infer arbitrary whole-workbook memory behavior. Diagnose first failing gate
  before reducing targets or changing limits; return material scope changes for review.
- Final local CI, strict Clippy, compatibility/core/IPC tests, attended managed and AOT
  graphical checks on all six RIDs for this combined milestone, with macOS x64 explicitly
  Rosetta. Use short owner-launched local-consent scripts, x64 pilot first, host-serial
  timing, immutable failure evidence and verified archival/pruning per completed batch.
  Refresh canonical per-project cloc once at milestone completion, as diagnostic evidence.

## Parent disposition and next step

The parent approves this design with three mandatory implementation clarifications:

1. **Canonical no-ops:** compare final content definitions, identity state and explicit
   per-property assignments, including Clear Both. Rendered equality does not erase
   the distinction between explicit built-in defaults and inherited worksheet defaults.
2. **Immutable Fill plan:** determine all source/destination pairs from one qualified
   pre-operation state and source capture before staging writes. Recalculation or
   post-publication filter changes cannot alter this operation's write set or sources.
3. **Combined publication:** Clear Both and Fill content/style transfer stage privately
   and publish once. No independently committed per-cell, per-row or APPLY/FORMAT
   sequence; failure, cancellation and admission refusal preserve relevant document
   and identity state.

The parent accepts all three Clear modes, four visible-edge Fill directions, complete
built-in reset, actual-coordinate translation, exact review including degenerate
no-ops, single-use state/interaction binding and hidden-row integrity with canonical
Blank-anchor demand permitted. One combined milestone has Clear-first/Fill-second
internal slices and one final implementation review, validation acceptance and owner
commit/push boundary; internal Clear completion is not separate acceptance.

Initial Clear 4,096 / Fill 256 addressed-slot limits, accumulated style work and all
existing resource budgets are accepted without increases, truncation or auto-chunking.
Private v13 is approved; WB12, hosted v1 and CB5/CB6 remain unchanged. The independent
correctness, native measurements and all-six-RID headless/managed/AOT plan are accepted;
macOS x64 is Rosetta and observed maxima are not percentiles. Retain original failures,
source qualifications, verified batch archival and per-project diagnostic cloc.

See the [parent design handoff](../handoffs/2026-10-31-visible-only-clear-and-fill-design.md).
No implementation or new platform validation has run. Await separate owner implementation
authorization. Parent implementation comments, acceptance-doc updates, owner commit/push
and explicit next-work authorization retain the standing cadence. ADR-0023 remains
parked at G3. The ALLOCATE proposal remains separate and unscheduled.

