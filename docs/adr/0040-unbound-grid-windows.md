# ADR-0040: Unbound Grid Windows, identity and references

Date: 2026-11-01.
Status: **Accepted and closed by parent implementation review (2026-11-01), with the unresolved Windows ARM64 cold-run qualification retained.**

## Design review disposition

The review accepts tagged ownership, reference/deletion semantics, WB14-only/private
v14 compatibility cuts subject to the implementation codec audit, clipboard refusals
and six-target qualification. Three mandatory clarifications are incorporated below:
client-only scope selection, one-generation bounded scope captures, and explicit
aggregate axis/identity admission. Return this revised contract for confirmation
before implementation. That confirmation and separate implementation authorization
have now been received. No subsequent ADR is assigned or started by this milestone.

Implementation and compatibility details are recorded in
[unbound Grid Window semantics](../architecture/unbound-grid-windows.md). The
[implementation handoff](../handoffs/2026-11-01-unbound-grid-windows.md) and
[resource qualification](../performance/unbound-grid-windows.md) record six-target
managed/AOT results and the separate Windows ARM64 retained-cache headless pass.
The original cold-run timing failure remains Failed and unresolved.

## Scope and foundation

Implement the first ordered scope of accepted
[ADR-0039](0039-named-windows-and-document-composition.md): workbook-owned unbound
Grid Windows with independent coordinate spaces, authored cells, stable references
and explicit lifecycle. The broader architecture remains authoritative. This slice
does not implement producers, binding, Text, section geometry, placement, document
capture, pagination, invocation or PDF. It provides a standalone editor surface and
stable ownership that those scopes can later consume.

Source inspection establishes concrete integration points:

- `pivot-workbook/src/lib.rs` has `CellRef { worksheet, cell }`, worksheet-qualified
  formula tokens and source-span metadata. Window syntax needs a tagged owner, not
  an additional name convention on WorksheetId.
- `sheets.rs` provides a directory and short-lived request views over one staged
  core/domain. Its pattern can serve both scope kinds; its worksheet directory and
  sixteen-sheet cap must not acquire hidden Windows.
- `persistent.rs` position keys and `transfer.rs::CellRecord` carry WorksheetId.
  Formatting, filtering and structural metadata also require an owner audit.
- The shared spreadsheet parser in `pivot-core/src/expression.rs` recognizes `!`;
  it must distinguish `|` in spreadsheet mode without changing ordinary model grammar.
- Native `PIVOTWB13`, private session version 13 and PIVOTCB5/6 have no Window
  owner tag. Their omissions must not be hidden by serialization as worksheets.

This is a design packet, not a claim that these changes exist. See the
[review handoff](../handoffs/2026-11-01-unbound-grid-windows-design.md).

## Owned model and public API shape

Introduce distinct non-nil `WindowId` UUIDs and `GridScopeId` tagged as
`Worksheet(WorksheetId)` or `Window(WindowId)`. Use this tag in CellRef, position
indexes, formula qualifiers, formatting ownership, viewport requests, transfer records
and structural descriptors. CellId remains a globally unique workbook anchor;
GraphNodeId remains calculation authority. Axis namespaces remain unique across all
scopes. No cast between WindowId and WorksheetId is permitted.

Keep the public worksheet APIs as convenience wrappers selecting the Worksheet tag.
Add generic scope entry points, with equivalent validation for both tags:

```rust
// Proposed signatures, not existing interfaces.
fn grid_scopes(&self) -> Vec<GridScopeInfo>;
fn windows(&self) -> &[WindowInfo];
fn create_grid_window(&mut self, expected: Expected, name: &str,
                      rows: u32, columns: u32) -> Result<WindowId>;
fn rename_window(&mut self, expected: Expected, id: WindowId,
                 name: &str) -> Result<Expected>;
fn delete_window(&mut self, expected: Expected, id: WindowId,
                 cancel: &CancellationToken) -> Result<Expected>;
fn apply_scope(&mut self, scope: GridScopeId, expected: Expected,
               edits: &[Edit]) -> Result<Expected>;
fn read_scope(&self, scope: GridScopeId, expected: Expected,
              rectangle: Rectangle) -> Result<ReadFrame>;
fn capture_scope(&self, scope: GridScopeId,
                 expected: Expected) -> Result<ScopeCapture>;
```

WindowInfo owns id, canonical name, row/column axes, Grid/unbound discriminants and
normal local geometry/format defaults. Only Grid/unbound tags are admitted in this
slice; unknown future kinds or ownership modes refuse import, not downgrade. The
Window directory is a charged persistent root, separate from worksheets. Scope info
returned through IPC is bounded and paged if needed; API result ownership is charged.

Creation allocates a WindowId and two axis namespaces, no cells/nodes for blank
positions. Dimensions are explicit, positive and no larger than the existing
65,536-row/1,024-column bounds or the active lower workbook limits. Maximum live
Windows is 64, separate from the sixteen worksheets. Existing aggregate workbook
cell, node, byte, source and work ceilings still apply **across all scopes**.
No multiplication of quotas by the number of Windows.

### Aggregate axis and identity admission

Logical dimensions are not allocated objects. Represent an empty scope's axes as
bounded compact runs/descriptors, not one object per declared row/column. Sixty-four
maximum-dimension empty Windows allocate only their directories, 128 namespace
descriptors and bounded axis-run metadata; they do not allocate millions of AxisIds,
CellIds or nodes. AxisId values resolved on demand are not new persistent identities
merely because their logical coordinates were inspected.

| Resource category | Transactional aggregate admission |
| --- | --- |
| Declared dimensions | Per-scope row/column bounds; no eager Cartesian allocation |
| Axis namespace/run storage | Charge actual allocated namespace descriptors, structural runs/indexes and scratch across worksheets and Windows to existing workbook/domain byte and work limits |
| Positioned/demanded cells | Existing aggregate allocated-cell/identity limits, including demanded Blank anchors, regardless of scope |
| Graph nodes/dependencies | Existing canonical graph identity, edge, work and memory limits across all scopes |
| Retired identities and retained roots | Count against applicable existing identity ledgers; charge retained descriptors, tombstones, axis runs and capture pins until legitimately reclaimed |

Namespace uniqueness validation covers live and referenced retired authority. A
delete/create loop cannot bypass allocation ledgers or free still-pinned metadata.
Import, structural splitting, creation and deletion must pre-admit new allocations
while old candidates/captures are still charged, then install atomically. Failure
publishes nothing. No new per-Window allowance or reset of existing identity counters
is permitted. Admission reports distinguish declared dimensions, allocated axis
metadata, positioned cells, graph nodes and retained retired authority so tests can
prove sparse creation rather than infer it from cell count alone.

Names are NFC, nonempty, at most 64 UTF-8 bytes after normalization, without leading/
trailing whitespace, controls or `[]:*?/\\!|`. Apostrophes are allowed and escaped in
quoted references. Use NFC(default-case-fold(NFC(name))) for lookup; pin the Unicode
tables in the format implementation. This matches the existing lookup approach while
making both normalization stages explicit. Worksheet and Window namespaces remain
separate: `Invoice!A1` and `Invoice|A1` may coexist. Same-name Window creation refuses.
Exact normalized rename is a no-op; case-only rename is an observable label change.

## Transactions, lifecycle and retirement

All mutations require the existing session/expected-state authority. Stage the
directory, sparse indexes, formula metadata and core candidate together, admit their
charges, validate, then publish once. Stale/cancelled/resource/invalid candidates
leave current roots and revision unchanged. Creation and rename advance workbook
state once; they do not alter the calculation graph merely because labels changed.
Scope selection is request metadata, never a retained server-side active scope.

### Scope capture contract

ScopeCapture is a bounded immutable view over the **same workbook calculation
generation**, carrying requested GridScopeId, axes, sparse position-index root,
relevant formatting/visibility roots, canonical CalculationCapture/revision and the
dependency authority needed for displayed values. It pins shared roots rather than
cloning a separate calculation model. Cross-scope references resolve entirely within
that same captured generation, including dependencies outside the visible scope.

There is no independently published Window calculation revision, second evaluation
cache or separate workbook evaluation. Any reuse is qualified by the canonical
capture identity. Bounded frames are read from this view with existing viewport
admission; capture creation does not enumerate all declared positions or eagerly
evaluate the entire Window. The scope root and canonical capture are obtained in one
coordinator boundary, never by joining reads from two mutable generations. Capture
pins and response buffers are separately charged; cancellation/admission failure
returns no partial capture. This is not ADR-0039's later document-generation capture.

Deleting a Window retires its directory entry and all owned cell anchors/backing
definitions in one bounded transaction. Before install, derive all inbound scalar and
range references, preserve their retired target authority and lower them to existing
deleted-reference fault semantics. Surviving formulas become completed reference
errors, not invalid parse drafts. Cross-scope dependents recalculate canonically.
No surviving reference may point at a newly reused address or name.

Deletion must traverse and charge affected indexes/formulas, not only visible cells;
it may refuse on cancellation or budget. Retained captures keep the old immutable
Window/graph roots and evaluate normally. Newly created scopes receive fresh IDs.
Retired scope/cell descriptors needed by current faults remain charged and are
serialized; reclaim descriptors only after no live references or retained owners
need them. No historical restoration, UUID reuse or implicit repair is introduced.
The codec retains referenced tombstone authority even after the live Window is gone.

Deleting a Window does not delete independently owned nodes outside that scope.
There are no producer aliases in this slice. No placement is created by selecting
or opening a Window; closing its editor does not delete it.

## Formula grammar and binding

Spreadsheet mode recognizes `Scope!A1` for worksheets and `Scope|A1` for Windows,
with `'Bob''s grid'|$A$1` quoting and ordinary whitespace handling. Extend qualifier
syntax with a namespace discriminator and preserve byte spans. `|` is never an
arithmetic operator. Ordinary non-spreadsheet expression parsing is unchanged.

Resolve a qualifier to GridScopeId during authoring against the expected candidate.
Unqualified A1 resolves in the formula's Grid scope. Store CellId, owner tag, flags,
submitted source and spans. Display current scope names and positions from those
bindings; Save stores authority, not a reparsed display string. Invalid syntax/name
follows the existing editable formula-draft behavior; invalid native authority is
refused before installation. A deleted reference remains an error until the user
explicitly authors a new valid reference.

Ranges must belong to one Grid scope. `SUM(Costs|A1:A9)` and
`ALLOCATEWEIGHTEDRANGE(10,1,2,Weights|A1:A2)` use existing operand contracts with a
Window-owned SelectionId and axis namespaces. Same-scope explicitly qualified second
endpoints may be accepted only when both resolve to the identical owner; mixed
worksheet/Window or distinct Window endpoints refuse. Do not add cross-scope union
ranges, change Blank behavior or relax strict allocation typing.

One canonical graph spans every scope. Cross-scope references participate in ordinary
complete candidate cycle validation, including cycles whose path leaves and reenters
a Window. Empty scalar targets demand sparse anchors exactly as worksheets do.
Range descriptors do not eagerly materialize blank cells. Renaming changes display
qualifiers without changing graph bindings or original source provenance.

## Editing and copy/fill

Unbound Grid supports the existing typed entry, clear, in-scope copy/fill, formatting,
row/column structural edits, sorting and filtering through owner-tagged adapters.
Existing spatial selection, row visibility and formatting overwrite policies apply;
no new visible-only semantics are introduced. Every command's owner must match its
selection/confirmation token. Unknown/deleted scopes refuse before reads or edits.

Within a scope, moves and sorting preserve scalar identity; range membership remains
spatial. Row/column deletion faults removed anchors and transforms surviving range
geometry by existing rules. Formatting is local to the tagged scope; editing one
Window never alters a same-address worksheet cell.

Cross-scope formula copy is a server-owned command over current roots, not OS native
clipboard transport. Relative coordinates translate by source/destination positions.
Unqualified references resolve in the destination scope; explicit `!`/`|` qualifiers
remain their explicit scope. `$` suppresses coordinate translation, not identity
retirement or scope qualification. Out-of-bounds translation refuses atomically.
Copying a fault preserves its fault descriptor, never resolves its old source name.
Copy creates destination-owned definitions; existing references to the source remain
unchanged. Cross-scope cut/move is deferred and refuses before mutation; in-scope
cut/move retains the established identity contract.

## Native, private protocol and clipboard cuts

Propose **WB14 as the sole native format for this slice**, replacing WB13 without
a historical reader or automatic migration. This is an explicit proposed compatibility
cut, to be frozen on review. Persist the tagged scope directory, live Windows,
referenced retired scopes, per-scope axes/geometry/formats/filter metadata, tagged
cells, formula qualifiers/faults and range authority. Retain exact decimal input
parts and String bytes. No producer/Text/layout placeholders masquerade as content.

Native validation requires unique/non-nil IDs and axis namespaces, valid names and
counts, exact owner matching, bounded index/graph membership, valid source spans,
complete reference/fault authority and a cycle-valid derived graph. Reject forged
Window-to-worksheet substitutions, references to absent unretired scopes, duplicate
CellIds and resurrection through live/tombstone collisions. Derive dependencies from
validated bindings, not serialized cached membership. The existing 64 MiB file cap
remains; all decoded lengths admit before allocation. WB13 and older signatures
report unsupported version and do not partially install a document.

Propose private session **v14**, rejecting other versions. Replace worksheet-only
targets with `{ kind: worksheet|window, id }` and return tagged scope locations.
New commands list/create/rename/delete Windows. There is **no server-side scope
selection command or active-scope state**. The desktop owns its local current scope;
every scope-dependent request explicitly carries GridScopeId and expected-state
qualification. Every frame carries scope, session/state and computation qualification.
Update both Rust and C# together. Delayed responses for another scope or revision
must not overwrite the displayed grid. Hosted v1 and pivotd/REPL public calculation
contracts remain unchanged; this is the existing private Desktop session boundary.

Do not introduce CB7 here. CB5/CB6 continue only for ordinary worksheet content that
can be represented completely. Native Copy/Cut from a Window, or worksheet formulas
containing Window references/faults, **refuse before any OS clipboard mutation**.
Native paste into a Window also refuses; UI offers an explicit text/values-only path.
Do not silently retry native as text. Explicit values-only export preserves String
identity markers/leading zeros and separates Blank from empty String according to
the established text contract; it does not claim native round-trip semantics.
Plain-text external paste into a Window uses the existing explicit text policy,
transactional admission and user-selected interpretation. Never infer formulas from
a values-only export. Same-document copy/fill remains available without OS clipboard.

Experimental structured/projection documents and active history-experiment state
refuse Window creation and Window document import. Their non-savable proof/parked
status is unchanged; do not extend experimental codecs to make the new feature work.

## Desktop surface and safety

Provide a separate named Windows list and open/select action alongside worksheet
navigation. The grid is the existing virtualized editor driven by a tagged scope,
not a new calculation session or a page placement. Show the Window name and unbound
Grid status. Create asks for name/dimensions; rename preserves selection identity.
Delete explains retirement and affected inbound reference counts, then requires an
explicit user action against an expected-state token. Counts must be exact/qualified;
if unavailable, block deletion rather than claim zero. This product confirmation is
not an implementation approval request.

Closing the selected Window after deletion chooses a surviving worksheet locally;
the server does not change other clients' scope. New/Open clears obsolete selections
only after existing document lifecycle acceptance. A Window with no open editor is
still saved. No document-layout controls, refresh button or bound ownership toggle
appear in this slice.

## Qualification plan and promotion gates

After design approval and implementation authorization, test real invariants:

- Same A1 in worksheet/two Windows stays isolated; blank scopes allocate no per-cell
  nodes. Name normalization, quoting, collisions and rename retain authority.
- Cross-scope calculations and complete-cycle refusals; scalar identity vs spatial
  range effects through insert/delete/sort; strict range allocation and production SUM.
- Window deletion faults scalar/range dependents, leaves unrelated cells unchanged,
  retains old captures and never repairs via name/address reuse; Save/reopen preserves
  faults and live identities. Failed/cancelled deletion installs nothing.
- Relative/absolute, qualified/unqualified copy/fill and explicit cross-scope copy;
  unsupported cuts and native clipboard paths leave both document and OS clipboard
  unchanged. Values-only leading-zero Strings remain exact.
- WB14 round trips, WB13 refusal, hostile lengths/tags/IDs/spans/tombstones, private
  version mismatch and stale/delayed cross-scope UI frames.
- Aggregate resource admission across many sparse scopes, retained generations and
  refusal rollback; closing/deleting views releases only their own charges.

Use focused Rust/native/IPC and in-memory UI coverage, then local CI/strict Clippy and
six-target headless validation. Plan managed and AOT GUI acceptance for all six targets
because this changes editing/navigation and protocol authority. macOS x64 remains
Rosetta-qualified. Install a short owner-run launcher, retain graphical consent,
archive original failures and verify evidence before cache cleanup. Record per-project
cloc after the implementation milestone, not a code-growth target for design docs.
Do not reclassify ADR-0038's unresolved Windows ARM64 cold-build failures.

No benchmark objective or global resource ceiling is relaxed. Measure additional
directory/reference overhead and structural fanout against existing workload objectives;
separate operational refusal from completed cell errors. Any required limit or
semantic expansion returns to review.

## Alternatives and consequences

Hidden worksheets would reduce initial refactoring but lose separate namespaces and
Window lifetime authority, so are rejected. A separate graph per Window would make
cross-scope references a second invocation/coordinator problem, so reuse one graph.
Extending native clipboard now would enlarge this slice and its cross-document
authority surface; explicit refusal plus server-owned copy keeps the boundary clear.
Deferring structural/formatting owner tags would leave an editor with inconsistent
commands, so the tagged-scope audit is required rather than optional follow-up.

The main implementation risk is the breadth of worksheet-qualified assumptions.
Promotion requires auditing every owner-bearing path, not merely parser/UI success.
The intended result is a reusable Grid ownership foundation; downstream scopes remain
separately authorized work, in the order accepted by ADR-0039.
