# ADR-0032: Worksheet sorting

Date: 2026-10-30.
Status: **Design approved with clarifications by parent review (2026-10-30).**

The owner authorized this design after committing ADR-0031. No feature implementation
or platform validation is authorized by this proposal. ADR-0031 remains accepted and
closed; ADR-0023 remains parked at G3. Follow the milestone cadence in
[STATUS](../STATUS.md).

The parent accepts the atomic identity-preserving permutation, fixed range geometry,
snapshot typed keys and stable ordering, explicit formatting policy, charged sparse
candidate and future six-target validation gates (macOS x64 under Rosetta). Retain
WB11/CB5, use private v10 for the new command, and retain the 33,554,432-unit work
ceiling and current capacity limits. These are approved implementation contracts,
not implemented behavior or measured sorting results.

Three mandatory implementation clarifications apply:

1. A completed calculation returning a legitimate cell error, including `#REF!` or
   division-by-zero, is a sortable error value. Incomplete capture, failed admission,
   cancellation or inconsistent revision aborts the sort as an operational failure.
   An erroneous cell value alone must not refuse an otherwise valid sort.
2. Determine no-ops from the final identity placements, reference/range semantics and
   explicit formatting assignments. Moving distinct CellIds is a real mutation even
   when displayed values are identical.
3. Show the final rectangle and partial-record warning visibly in the ordinary Sort
   dialog. Give an explicit opportunity to include adjacent populated columns; every
   expansion requires confirmation. Never silently expand or bury the warning in help.

Record this acceptance and stop for the owner's separate implementation authorization.
Filtering remains only a later candidate, provisionally ADR-0033.

## Scope and user contract

Sort row segments within one explicit worksheet rectangle by one or more selected
columns. Every column in the rectangle participates in the same row permutation;
cells outside it stay in place. This is record sorting within a selected rectangle,
not whole-worksheet row movement. Optional first-row exclusion is explicit, initially
off; no guessed header or automatic expansion to adjacent data. Excluded headers
retain their content, identities and formatting.

The Sort dialog visibly names the final rectangle and warns that cells outside it
do not move. The partial-record warning and expansion choice are ordinary visible
dialog controls, not information buried only in help.
Offer Expand selection and Sort selected range when adjacent populated columns
suggest a partial record; expansion requires confirmation of the new rectangle,
not an inferred table boundary. The dialog itself always shows the final rectangle.
Selecting a single cell requires an explicit range before submission. Whole-row or
column selections are valid sparse rectangles, subject to normal admission.

Controls: ordered key levels with Add/Remove/Move Up/Down; each chooses a column
within the rectangle and Ascending/Descending. Show column letters, with excluded
header text as an optional label, never as a unique identifier. Require at least one
key and reject duplicate columns. Maximum key count is the selected width, bounded
by the existing 1,024-column extent; work and message limits still apply.

One Apply performs one atomic command. Zero or one data row and already ordered
input are valid no-ops. Keep the selected rectangle/header setting visible after
success; retain active-cell coordinates within it rather than chasing a moved CellId.
Report changed/no change and row segments moved. Commit the active editor normally
before preparing the request; refusal/cancel leaves the workbook and selection intact.
No persistent sort definition, auto-resort, filtering, hidden rows, structured tables,
grouping, merges, column sorting, custom lists, locale options or Undo/Redo is included.

## Identity alternatives and proposed decision

Source constraints: `persistent.rs` ordinary `Edit::Move` rejects occupied targets
and snapshots resolved styles, losing inheritance intent. `structure.rs` already
stages collision-safe placement changes with shared immutable payloads and one
graph/workbook publication. Selection descriptors name geometry, not member lists.
ADR-0030 stores current per-property assignments, without their originating action.
Coordinate-derived AxisIds are not durable row identities.

| Alternative | References and formulas | Cost and disposition |
| --- | --- | --- |
| Exchange contents at fixed CellIds | Outside scalar references keep coordinates but silently observe a different record; formula definitions must be replaced and their bindings explicitly chosen | Conflicts with existing identity-following moves; reject for this command |
| Replay ordinary Move/Copy through temporary cells | Move collisions, style snapshots, possible temporary identities and intermediate states; Copy translates relative operands | Reject; neither operation defines record sorting |
| Explicit row permutation, relocating existing identities | Scalar references follow the same cells; formula definitions/bindings travel intact; rectangular ranges keep their geometry | **Recommend** a dedicated atomic sparse permutation primitive |

The permutation is a transient plan, not a new persistent row identity or registry.
For data interval [a,b], construct a bijection old row → new row; columns are
unchanged. Apply it only inside the selected columns. Move all positioned identities,
including cleared cells and Blank reference anchors. Move no identities for virtual
blanks. Preserve CellId, portable GraphNodeId, runtime binding, source, exact literal
representation, fault provenance and payload. Update coordinate-derived axes and
placement indexes. Allocate/retire no computational or selection identities.

Sorting is movement, not Copy/Fill: do not translate relative references or reparse
authored formula text. `$` markers retain their later Copy/Fill meaning. Rejected
formula drafts stay rejected. An unchanged editor submission preserves bindings.
All surviving scalar targets, inside/outside the range and on other sheets, retain
their identity. Effective formula rendering follows their new coordinates.

Example: A1=2, A2=1, B1=`=A1*10`, B2=`=A2*10`, D1=`=A1`.
Sorting A1:B2 ascending on A moves original row 2 to row 1. New B1 renders
`=A1*10` and evaluates to 10 because its original target moved too. D1 renders
`=A2` and still evaluates to 2. Sorting A1:A2 alone leaves B1 in place but its
target moves to A2: B1 renders `=A2*10`. The dialog's range warning matters.

## Rectangular ranges and calculated keys

Keep every SelectionId, rectangular geometry, endpoint order, sheet qualifier and
absolute marker unchanged. A range means current occupants of its coordinates;
it does not become a discontiguous historical record set. Ranges entirely containing
the sorted rectangle retain the same members, though order-sensitive consumers may
change. Intersecting ranges may acquire different members. Recompute affected
membership/dependencies through the existing placement invalidation path, including
consumers on other sheets. Do not reframe ranges or allocate replacement SelectionIds.

For example, sorting A1:A3 changes the occupant of SUM(A1:A1); an ordinary scalar
`=A1` follows its original target instead. This established scalar/range distinction
must be documented in the dialog's help, with a concrete example. Sorting can change
formula results and even create a dependency cycle through changed range membership;
apply existing graph/evaluation semantics, never silently change bindings to avoid it.
Normal calculated errors are results; graph admission/invariant failures abort.

Read every key from one immutable pre-sort calculation capture paired with the
expected workbook/session/graph revision. Formula keys use their calculated typed
values, not their source, displayed strings or a mixture of revisions. Materialize
each required stored key result once before comparing. Missing coordinates read
Blank without allocating anchors. Never evaluate from a sorting comparator.

Key snapshot → permutation → publication occurs once. Recalculation after movement
may change formula keys, especially position-owned range expressions. Do not iterate
until sorted, auto-resort or promise that newly calculated keys remain in order.
State in help: “Formula keys use values before sorting; recalculation may change them.”
Cancellation, evaluation exhaustion and unavailable/incomplete key capture abort the
command; they are not sortable errors or permission to reuse stale displayed values.

## Deterministic ordering

Lexicographic comparison uses key levels in dialog order. Each level first compares
its two key values using this sort-only total order; equal values advance to the
next key. Completely equal tuples retain original row order. Original row ordinal
is the final ascending tie breaker regardless of key directions.

| Key kind | Ascending order within kind |
| --- | --- |
| Decimal96 | Exact numeric comparison; equal numeric values tie despite scale/source spelling |
| String | Ordinal Unicode scalar sequence, case-sensitive, without normalization or locale collation |
| Boolean | FALSE before TRUE |
| Calculation error | Equal for sorting; original error/provenance remains untouched |
| Blank | Equal for sorting; includes unpositioned and positioned Blank results |

For successful nonblank values, kind order is Decimal96, String, Boolean.
Descending reverses that kind order and comparisons within those kinds. Errors
always follow successful nonblank values; Blanks always follow errors, independent
of direction. Two errors tie without comparing messages, internal discriminants or
provenance. Error versus Blank is therefore fixed in both directions. Later keys can
order rows whose current key is equally Blank/error. This defines every pair and
keeps sort ordering separate from ADR-0025 expression comparison/coercion rules.

Empty String is String, distinct from Blank. Strings `001234`, `00.50`, `TRUE` and
`1234` stay exact Strings: no trimming, numeric conversion or leading-zero loss.
For Strings `001234` sorts before `01234` before `1234` in ascending order. Formatting
(Currency, percent, precision, grouping, colors) never changes a key's type/order.
Non-finite numeric values do not exist in Decimal96. Expose the kind order,
case-sensitive text rule and always-last Blank/error policy in concise dialog help.

## Formatting: explicit record or position policy

Offer **Move formatting with records** (default, checked). Unchecked means **Keep
formatting at worksheet positions**. This is independent of header/key/direction.
It is explicit because styles can express record meaning or positional presentation;
the store cannot reliably infer that intent from their current geometry.

With movement enabled, apply the identical row permutation to each property's
assignment function inside the data rectangle, including formatting-only blanks.
Transfer Set(value), explicit built-in/default-equal values and absence/inheritance
exactly. Do not resolve inherited values into explicit snapshots. Outside the data
rectangle and in an excluded header, retain assignments unchanged. Worksheet defaults
stay unchanged. With movement disabled, leave all style roots unchanged.

Whole-column formatting is invariant under a within-column row permutation, so
ordinary column Currency remains column Currency. A per-record exception travels
with its row segment; an explicit General exception stays explicit. A row background
travels within the rectangle only. There is no historical “column formatting” layer
to recover beneath exceptions; neither mode invents one. Position-owned striping
stays put by choosing Keep formatting. Subsequent selection-wide formatting retains
ADR-0030's latest-action-per-property overwrite behavior, including moved exceptions.

For movement, clip existing property rectangles against the data rectangle, retain
outside pieces and map inside row intervals through the permutation. Group adjacent
destination rows with identical assignments/column intervals; normalize disjoint
rectangles using bounded existing machinery. Process inherited gaps by removing old
inside assignments before adding mapped explicit ones; otherwise old overrides leak.
No ten-property dense grid or one Style per virtual coordinate. Permutation can
fragment regions beyond current limits: refuse atomically with a formatting-limit
explanation. Never drop assignments, clip formatting, switch policy silently or
raise limits. User can explicitly retry with Keep formatting.

## Sparse plan and atomic publication

1. Validate expected session/state/graph, WorksheetId, rectangle, header, keys and
   formatting policy. Capture content/calculation/style roots and reserve charged
   scratch. No asynchronous UI code constructs private graph edits.
2. Enumerate intersecting positioned records and stored key definitions; stream
   bounded key evaluation from the capture. A row without a positioned key gets an
   implicit all-Blank tuple. Preserve rows containing only non-key content/styles.
3. Build a stable permutation of data row ordinals. At most 65,536 ordinals are
   necessary, even for a 67,108,864-coordinate whole-sheet selection. Use compact
   row/key indexes and sparse stored values, not height × width or height × key-count
   materialization. Implicit Blank keys need no entries. Count comparator key probes
   and Unicode scalar/byte examination, with cancellation during long comparisons.
4. On private workbook/graph candidates, remove all moved source placement keys
   before inserting destinations. Share immutable content payloads as ADR-0031 does.
   Stage graph placement removal/addition together; update range membership once
   against the complete final placement, never against temporary holes/collisions.
   No selection reframing, formula replacement, source rewrite or identity allocation.
5. Transform styles if requested. Validate resulting limits, graph invariants,
   dependency/member expansion, complete-state change, expected revision and cancel.
   Publish workbook, placements, graph and styles once, incrementing revision once.
   Old captures/exports retain their original placements, styles, bindings and values.

The existing structural command is a pattern, not a reusable row-insertion algorithm:
sorting has arbitrary cycles and fixed range geometry. Extract narrowly shared staging
helpers only where needed during authorized implementation. No general transaction
framework or canonical query subsystem is assigned.

No-op detection compares resulting identity placements and assignment intent, not
only displayed values. Identical-value records with unequal keys/identities can
still move. A nonidentity plan moving only indistinguishable virtual blanks can be
a complete-state no-op. A style-only permutation is a real mutation if assignments
change. Stale/cancel/work/memory/style/dependency failures leave every root, counter,
revision and UI selection unchanged; candidate charges are released.

## Admission, performance and representation

Keep positioned-cell 65,536, lifetime-identity 262,144, worksheet extent, 16 sheets,
source/token/selection/consumer/edge limits, ten style properties, 65,536 style
rectangles and 8 MiB style budget unchanged. No selected-area cell allocation or
clipboard/Fill limit increase. Check all existing evaluation/frame/native budgets.
Charge plan, cached key values, permutation, index/path copies, style fragments,
graph candidates and simultaneous captures to existing domain memory admission.

Use ADR-0031's 33,554,432 counted structural-work ceiling for the initial sort
proposal, including ordering and style normalization, with existing narrower
subsystem ceilings still enforced. Count scanning unrelated stored state if existing
indexes require it; do not claim purely intersecting-state complexity without a real
index. Typical planning is O(H log H × compared key cost + stored-state visits +
style output), H ≤ 65,536; adversarial long common text prefixes and fragmentation
can refuse. Work/byte preflights and mid-operation checks must cover actual loops.

Proposed measured gates, not current results: compact 65,536-cell numeric-table sort
≤1 s and admitted style stress ≤2 s; local edit/viewport p95 ≤100 ms, capture p95
≤5 ms, defined recalc ≤2 s and Save/reopen ≤10 s. Retain compact-corpus ≤256 MiB
steady and ≤512 MiB charged peak with four old roots and Save/reopen. Measure
1,024/10,000/65,536 cells, including 1,000×10, 4,096×16 and sparse 65,536×1
tables. Report actual corpus dimensions/counts, formula evaluation,
ordering, placement/invalidation, style normalization, capture/recalc/save timings,
charged scratch/peak/retained bytes, allocations and process RSS separately.

Persist final ordinary placements/styles/bindings only, no permutation log or sort
settings. **Propose retaining WB11 and CB5 unchanged:** this design introduces no
new persistent value, identity or style representation. Verify current reader/writer
round trips arbitrary final permutations; a representation change would require a
concrete reason and parent disposition, not automatic version inflation. Private
request/response adds a bounded Sort command with expected revision, WorksheetId,
rectangle, header exclusion, ordered keys/directions and formatting policy; propose
private v10 for that contract change. Result includes changed/revision and moved-row
count. Frozen hosted-v1 remains unchanged. No obsolete readers or migrations.

## Future implementation validation and review gate

No tests or platform execution are part of this design task. After parent review
and owner implementation authorization:

- Compare generated small grids against an independent stable row-tuple oracle and
  dense ten-property assignment oracle, including inheritance after default changes.
  Cover every kind/direction, multikey ties, empty/one-row/header cases, reversed input,
  exact Decimal equality, Unicode combining/supplementary text and exact leading zeros.
- Verify identity sets/counters, cycles of occupied destinations, cleared/Blank anchors,
  sparse tall/wide selections, formatting-only rows, defaults/explicit defaults, both
  style policies, partial-column warning and no movement outside the rectangle.
- Verify formulas inside/outside/on other sheets, all `$` combinations, authored versus
  effective rendering, fixed/intersecting/reversed ranges, selection/consumer continuity,
  calculated keys that change after movement, lazy faults, rejected drafts and existing
  reference-fault provenance. Verify post-sort Copy/Fill/Move and normal operand repair.
- Exercise old captures/prepared exports, duplicate/stale requests, cancellation during
  key evaluation/comparison/style/graph stages, exact and first-excess quotas, fragmented
  styles, oversized keys/messages and graph/member expansion. No partial publication,
  fresh identities, leaked charges or silent policy fallback on any refusal.
- Round-trip current WB11 with identical identities, styles, source/provenance and
  effective rendering; inspect CB5 and CSV after sorting without conversions. Existing
  obsolete/malformed inputs remain refused. Sorting twice in unchanged order is a no-op;
  style-only changes remain mutations. Save/reopen does not invent a sort definition.
- Measure the staged corpora and sparse full-extent cases before larger stress.
  Retain actual failures and successful retries separately; do not raise resource gates
  to hide a failure. Repeated timing distributions must be labeled separately from
  single observations and macOS x64 Rosetta from native Intel hardware.
- Exercise actual dialog/key reordering/header/range warning/format option/editor/focus
  and persistence in managed and Native AOT on all six configured targets for this
  first sorting milestone. Use short local launchers and normal graphical-terminal
  consent under [testing](../testing.md); never launch attended GUI suites through SSH.

Parent review accepts identity-following record movement, fixed rectangular ranges,
snapshot formula-key semantics, mixed-kind order, explicit formatting policy and the
bounded sparse architecture, with the three mandatory clarifications above. Feature
work awaits separate owner authorization. Filtering remains a later candidate,
provisionally ADR-0033; this acceptance does not authorize it or resume history research.
