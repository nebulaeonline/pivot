# ADR-0041: Worksheet-Range Producers and Bound Grid Windows

Date: 2026-11-02.
Status: **Design approved with four mandatory clarifications incorporated (2026-11-02). No implementation authorized or performed.**

## Parent design disposition

The parent approves the architecture subject to four implementation-contract
clarifications: combined positional/scalar authority, refresh state versus selected
values and freshness, complete native authority validation, and a single explicit
ownership/charge ledger. The normative examples, transition matrix, representation
gate and accounting table below incorporate all four. No further architectural
proposal or parent confirmation cycle is required. Implementation requires separate
owner authorization. Subsequent architectural directions are not started here.

## Purpose and boundary

Implement the second ordered scope of accepted [ADR-0039](0039-named-windows-and-document-composition.md),
following accepted [ADR-0040](0040-unbound-grid-windows.md). A named read-only Grid
Window can project a live worksheet rectangle or an explicitly refreshed immutable
fixture producer. Formula consumers retain source identity, membership and lifetime
authority. This is not a copied worksheet, a second evaluator or a database framework.

The accepted architecture remains authoritative. Detailed choices below specialize
[Window identity and binding](../architecture/named-windows-and-binding.md); they do
not reopen Text, geometry, layouts, invocation or PDF. The
[design handoff](../handoffs/2026-11-02-bound-grid-windows-design.md) summarizes review decisions.

## Existing implementation seams

- `pivot-workbook/src/windows.rs` supplies tagged scopes, client-qualified requests,
  ScopeCapture, transactional lifecycle and inbound-reference review. Bound ownership
  must be explicit in that directory; it must not impersonate an unbound cell index.
- The canonical model's CellId and SelectionId distinguish identity-following scalar
  references from spatial rectangles. Existing structural transformations and range
  faults supply the worksheet source geometry contract.
- `projection.rs` and `structured.rs` contain opt-in ADR-0019/0020 proofs of immutable
  keyed records and captured scalar contexts. They are fixed-schema experimental
  paths, not production producer directories, persistence or admission policies.
- `pivot-core/src/captured.rs` admits Decimal96 or a source fault. Preserve that seam;
  this milestone does not introduce typed captured values or a new expression operator.
- WB14/private v14 are implemented; hosted v1 and CB5/CB6 are unchanged. New authority
  cannot be serialized as WB14 or hidden in existing clipboard envelopes.

## Objects and ownership

Add a workbook producer directory, separate from Window and worksheet directories.
Each producer has non-reused ProducerId, immutable configuration revision, source
kind, schema authority, selected generation and bounded status. Multiple Windows
may share a producer; each carries its own non-reused WindowId and lifetime guard.
Deleting one Window never deletes the producer, another view or worksheet cells.

A Grid Window declares exactly one ownership variant:

| Variant | Content authority | Update mechanism |
| --- | --- | --- |
| Existing unbound | Window-owned sparse authored cells | Ordinary explicit edits |
| Worksheet-bound | WorksheetId and authoritative source SelectionId | Every relevant committed workbook change |
| Fixture-bound | ProducerId, configuration revision and selected immutable snapshot | Explicit Refresh only |

Creation chooses the variant. There is no in-place ownership toggle, source-range
replacement or producer reconfiguration. Changing source/schema/key policy creates
a new producer and Window, with a preview and explicit owner action. Existing
references never silently acquire that new authority. Bound content cannot be edited,
formatted, filtered, sorted, filled, cleared or structurally changed through its view.
Source operations remain available at the source worksheet. Rename/open/selection,
read-only copying, refresh where applicable and deletion remain supported.

## Worksheet source geometry and presentation

Create a worksheet producer from an explicit WorksheetId and nonempty finite
rectangle at expected state. Source is an ordinary worksheet, never another Window.
Register/reuse a source SelectionId using the existing spatial descriptor contract;
store its effective endpoints and fault authority, not reparsed A1 text. A descriptor
reference is charged even when its underlying immutable geometry is shared.

Local A1 maps to the source's current top-left; local rows/columns follow physical
source order. There is no inferred header row: every source position is data. Source
column axes provide stable FieldIds; row/column labels and current ordinal are
presentation, not cell identity. Sorting source contents changes the displayed
occupants, while previously authored scalar references follow their source CellIds.
Rows are not treated as durable business-record keys for worksheet scalar binding.

Blank positions remain positions. Hidden/filter-excluded source rows and columns
remain members and are displayed in physical order; source visibility is reported
as provenance, never used to compact the projection. Bound Windows have no separate
filter mask in this slice. Effective source built-in formatting is read-only; Window
chrome and dimensions are not editable content styles. This avoids a second style
precedence system. Decimal metadata, signed zero, Boolean, String, empty String,
Blank and cell errors retain their exact existing typed presentation.

Structural affinity is exactly the existing source SelectionId transformation:
for an interval `[a,b]`, insertion before `p <= a` shifts both endpoints; insertion
at `a < p <= b` expands the interval; insertion after `b` is outside. Deletion removes
covered positions and shifts survivors under the existing fixed-extent rules. Use
the existing descriptor-transform implementation and endpoint fixtures from
[ADR-0031](0031-row-column-structure.md), including outgoing-identity admission.
Each effective rectangle maps contiguously from local A1 without a second independently
editable axis identity set. Whole-source descriptor deletion produces an unavailable
source fault, not a newly bound empty rectangle. Last known extent is diagnostic
only. No coordinate is valid for new formula binding while unavailable. Reinserting
rows, recreating a worksheet or using an old name never repairs retired authority.

Source edits, sorting, structural changes, visibility changes, formatting and
recalculated values become visible from the same committed workbook capture. A
worksheet producer has no independently published value cache or refresh command.
No per-position values, CellIds or graph nodes are allocated to display holes.
Viewport reads stay bounded by existing read-area and returned-record limits.

## Scalar and range authority

Author `View|A1` against one expected candidate. For worksheet binding, record
WindowId, ProducerId/configuration, source CellId and the membership guard; lower
the valid computational dependency to the existing source node. There is no alias
value node. If the scalar addresses an unallocated Blank position, demand only the
ordinary source blank anchor, transactionally within existing cell/identity budgets.
Viewport reads and source-range registration never demand all blank anchors.

Before every affected candidate publication, resolve that CellId's current source
position and verify it belongs to the producer's transformed rectangle. Membership
failure installs an explicit guarded reference error in the canonical candidate,
retaining binding metadata needed for later maintenance. Re-entry of the same live
CellId restores availability; address reuse with another CellId never does. Retirement
of source CellId, SelectionId, worksheet, producer or Window is permanent for that
authority. Display follows a surviving identity's current local address; unavailable
targets render an error with retained provenance. Rename changes presentation only.

A worksheet-bound range stores Window lifetime and source-producer guards plus a
Window-local subrectangle composed into a source SelectionId. On source structural
changes, transform the authoritative source selection first, then derive its local
coordinates. It must remain wholly inside the producer rectangle: partial overlap
faults the whole operand rather than silently clipping it. Temporary exclusion can
resume for the same still-live descriptor; descriptor retirement cannot. Dense hole
positions, reversed endpoint normalization and function-specific rules remain those
of the canonical range consumer, including strict ALLOCATEWEIGHTEDRANGE typing.
No per-member alias cells or external computational ranges are introduced.

### Normative combined scalar/range example

Baseline: `Items` projects `Data!B2:B4`, holding Decimal values `2,5,3` and source
CellIds `c2,c3,c4`. The scalar `=Items|A2` binds **c3**, not ordinal 2. The range in
`=SUM(Items|A1:A3)` binds spatial descriptor **R**, initially `Data!B2:B4`.
`=ALLOCATEWEIGHTEDRANGE(10,2,2,Items|A1:A3)` uses all three physical slots of its
authorized descriptor, initially returning `5`. Its descriptor may be shared with R
when admitted geometry is identical; scalar authority remains distinct.

Each independent case starts at that baseline, except explicitly paired transitions:

| Source operation/state | Scalar bound to c3 | Spatial range and formula behavior |
| --- | --- | --- |
| Insert row before 3 | c3 survives at B4/local A3; scalar remains 5 | Producer and R become B2:B5 with `2,Blank,5,3`. SUM remains 10. Allocation faults TypeMismatch on the physical Blank; occupancy must not compact it into three weights. |
| Sort B2:B4 descending | c3 moves to B2/local A1; scalar remains 5 | R stays B2:B4 with `5,3,2`. SUM remains 10; allocation at index 2 becomes 3. The range is not a retained ordered list of c2,c3,c4. |
| Move c3 from B3 to B8 within the worksheet, leaving producer geometry fixed | c3 is live but out of view: guarded reference fault | R stays B2:B4 with `2,Blank,3`; SUM is 5 and allocation faults on Blank. Occupancy or visibility cannot remove the middle slot. |
| Move that same live c3 back to B3 | Membership restores scalar value 5 | Live R still names three physical positions; SUM is 10 and allocation returns 5. A replacement CellId at B3 would not restore the scalar. |
| Delete source row 3 | c3 is retired: permanent scalar reference fault | R survives and shrinks to B2:B3 with `2,3`; SUM is 5, allocation at index 2 is 6. |
| Reinsert row 3 after that deletion | c3 stays retired even if a new value 5 is entered there | Surviving R follows insertion affinity and physical new occupants. It may calculate with the new value; that does not repair the retired scalar. |
| Delete all source rows 2..4 from baseline | c3 is permanently retired | Producer descriptor and R have no surviving positions: permanent reference faults, not empty/zero ranges. Recreated coordinates or names revive neither descriptor. |

Temporary range exclusion is separately normative: if a **still-live** R is not
wholly inside the live producer rectangle, the whole operand is out of view. Even
one excluded physical position faults SUM/allocation rather than clipping the range.
If an authorized transformation restores full containment of that same live R, it
may resume after candidate cycle validation. If R has been retired, containment of
another descriptor with identical endpoints cannot resume it. Guard maintenance
retains these states distinctly in native authority and fault provenance.
Ordinary monotone transformations of initially nested rectangles normally preserve
containment; this rule does not authorize source rebind merely to generate exclusion.
Test exclusion/restoration with independent membership-state oracles and valid
composed-descriptor candidates as well as reachable commands.

Required evidence combines all three formula forms across insertion, deletion,
sort, hidden positions, temporary exclusion/re-entry and Save/reopen of live,
temporarily excluded and permanently retired authority. Old captures keep their
baseline results. Restoration cannot consult old rendered names to repair a binding.

Copy/Fill into an editable destination preserves existing relative/$ semantics and
then explicitly binds translated targets in that destination or the retained explicit
scope. Unrepresentable or unsupported targets refuse atomically. Copy of a fault
retains its authority, never fresh name lookup. Bound Windows are never mutation
destinations, including trusted cross-scope copy. Cross-scope cut/move stays deferred.

## Fixture producer contract

Production-shaped producer objects are introduced with a deterministic, versioned
fixture provider only. A local installed fixture catalog supplies declared snapshots
and controlled failure/delay cases. The private request selects an admitted catalog
ID and fixture version; it cannot submit arbitrary paths, SQL, scripts or URLs.
The same provider interface can be called by headless tests. No connector discovery,
network access, credential store or production database adapter is part of this slice.

Each configuration declares:

- ProducerId, monotonically qualified configuration revision, provider/version and
  explicit refresh capability. Configuration changes use replacement, not retargeting.
- Immutable SchemaId, ordered FieldIds, labels, exact declared value kinds and
  nullability. Labels do not confer identity. Blank is explicit; missing required
  fields and incompatible types reject the complete generation. No inference.
- Key policy: durable exact UTF-8 keys whose contract guarantees the same key is
  the same record, or generation-qualified keys. Neither policy derives identity
  from row ordinal, display label or value equality.
- Ordered unique item keys and exact typed field values. Keys are opaque bytes:
  `0012` and `12` differ, and case or Unicode normalization is never silently applied.
  String values and leading zeros are preserved without numeric parsing. Decimal96
  sign/scale metadata, Blank and empty String remain distinct.
- Non-reused ProducerGenerationId, payload digest, provenance and persistence policy.
  Generation IDs identify successful immutable payloads, not freshness guarantees.

Schema/FieldIds and key policy remain fixed through refresh. Reordering records is
allowed and formula display follows keys to their new ordinals. Durable-key absence
is a missing-item fault and reappearance may resume only under that declared same-key
contract. A generation-qualified reference never retargets to a later generation,
even when its text key recurs. Native import verifies the stored policy and authority;
it cannot infer a durable contract from repeated values. Retired producers and fields
are never resurrected by catalog replacement.

Fixtures preserve Strings, Boolean and Blank for presentation. Only declared Decimal
fields can be bound by formulas; non-Decimal fields refuse authoring/copy with a
specific diagnostic before graph installation. A nullable Decimal currently Blank
can retain a Decimal-or-source-fault leaf, yielding a completed missing-value fault,
not numeric zero. Present Decimal values use their exact stored value. Source errors
use explicit captured-fault authority; they are never replaced with cached numbers.

All consumers of the same producer/configuration/item/FieldId share one admitted
captured scalar leaf, independent of Window. Window lifetime/membership guards remain
per binding. Leaf identity is stable under durable-key refresh; its immutable context
is pinned with the canonical calculation capture. New selected generation contexts
invalidate dependent calculation through the existing core mechanism, not a second
evaluation cache. External ranges and arbitrary record reductions remain unsupported.

## Refresh and publication

Refresh is an explicit command for fixture-bound sources only, never a side effect
of read, calculation, open, capture or Save. Creating a fixture binding starts with
no selected snapshot and an unavailable display until the owner explicitly refreshes.
Selecting a catalog version does not perform refresh. One producer publication updates
all of its bound Windows atomically.

| State | Selected data | Permitted transition |
| --- | --- | --- |
| Never refreshed | None; unavailable | Explicit begin |
| Refreshing | Previous successful snapshot, if any | Matching completion, failure, cancellation or supersession |
| Current | Last successful immutable snapshot | Explicit begin or disconnect |
| Failed/stale | Previous success retained; no success stays unavailable | Explicit retry |
| Disconnected | Saved snapshot readable; refresh disabled | Explicit local capability reconnection |

Begin checks expected state and allocates a bounded one-use ticket qualified by
session, ProducerId/configuration, connection epoch, attempt sequence and nonce.
A new attempt supersedes the previous ticket. Provider work occurs outside the
workbook publication lock against admitted immutable input. Completion may stage
against the latest workbook head, allowing unrelated intervening edits, but must
revalidate the ticket, producer lifetime/configuration, capability and selected base
generation. Deleted/replaced/disconnected/superseded or replayed completions cannot
publish. No unbounded automatic rebase loop or provider retry is permitted.

Validate full payload, keys, schema, counts, identity admission, downstream guard
maintenance and cycles before one atomic coordinator install. Successful publication
selects generation, contexts, affected bindings and canonical graph roots together;
never publish rows before formula authority. Increment workbook expected state once
per committed status/publication transition. Calculation revision changes only when
its canonical dependencies/contexts change. Status alone is not a new value generation.

Failure/cancellation retains the last successful selected data and values; display
explicit stale/failure status rather than replacing successful values with zero or
partially refreshed records. Bounded failure metadata may publish separately without
selecting the rejected payload. Obsolete completion cannot overwrite the current
attempt's status. Cancellation before install publishes no partial data; cancellation
after the atomic install does not undo a completed transaction.

### Selected generation, attempt status and freshness

These are three independent axes. **Selected generation** is successful immutable
value authority. **Attempt status** describes refresh activity. **Freshness/render
eligibility** is a future source-admission policy decision, not a synonym for selected
data, success or failure. This milestone records provenance/generation/attempt facts;
it implements no document renderer, freshness TTL or implicit freshness acceptance.

The matrix is normative. W+1 is one committed workbook expected-state advance.
C is the canonical calculation revision. Keep bounded latest-result diagnostics,
not unbounded attempt history.

| Transition | Selected data | Attempt status | Workbook/calculation effect | Freshness/render meaning |
| --- | --- | --- | --- | --- |
| Begin with G1 selected | G1 | In progress, ticket T | W+1; C unchanged | G1 remains calculable; no freshness assertion. |
| Matching T succeeds with changed context G2 | G2 | Idle; last outcome Succeeded(T,G2) | W+1; changed captured contexts invalidate and publish affected dependencies atomically. If no computational context/dependency changes, C need not advance. | Record success provenance; future policy still decides eligibility. |
| Matching T fails after G1 success | G1 | Failed(T), UI identifies retained G1 | W+1 for status; C unchanged; no G2 selected | G1 remains calculable; future policy may refuse it as stale. Displaying G1 never claims T succeeded. |
| T fails before any success | None | Failed(T), unavailable | W+1; C unchanged by failure | No successful data to admit. |
| Cancel matching T before install | G1, or None | Cancelled(T) | W+1 for status; C unchanged; staging charged until actually released | Selected provenance unchanged. |
| Begin T2 while T1 pending | G1 | T1 superseded; current In progress(T2) | W+1; C unchanged; T1 staging stays charged | No new value/freshness claim. |
| Obsolete/superseded/replayed completion | Unchanged | Newer status unchanged | No W/C advance or selected-generation change | Completion conveys no eligibility. |
| Unrelated edit between Begin(T) and Complete(T) | G1 until authorized completion | T stays in progress | Edit advances W/C as required by that edit; T rechecks authority against latest head | Edit asserts no external freshness. |
| Disconnect with T pending | G1, or None | Disconnected; T invalidated | W+1; C unchanged by disconnect; selected contexts retained | Snapshot readable; future policy may require connection/refresh. |
| Cancellation after successful atomic install of G2 | G2 | Success retained | No rollback or additional W/C transition solely for late cancellation | G2 retains its actual success provenance. |

Failed/cancelled attempts cannot silently invalidate successful calculated values.
Selected snapshot, status and C are captured coherently but have different change
criteria. Test exact W/C and ProducerGenerationIds for failure after/before success,
cancellation before/after install, reversed completions, replay/disconnection and
unrelated intervening edits. Old captures retain their contexts and status facts.

## Coherence, dependency validation and capture

One coordinator candidate contains source selections, producer membership, selected
external contexts, guarded formula bindings and canonical calculation roots. A
ScopeCapture pins these from one committed generation. An old capture reads its old
source positions, membership, values and fault state after source edits, refresh or
deletion; it never consults a live directory. No independent Window calculation
revision, graph, per-view value cache or mutable provider read is added.

Validate the complete candidate's graph plus producer-consumer relationships before
install, including currently excluded/faulted guarded bindings. Reject a consumer
that directly or transitively supplies its worksheet producer's source membership,
including range members and dependencies through other Windows. For example, source
`Sheet!A1 = View|A1` must refuse instead of cloning an alias to conceal self-feedback.
Do not merely evaluate producer values then run an unrelated model cycle check.
Retained potentially reactivatable bindings must not enable hidden feedback; restored
membership is cycle-checked in the same structural/sort/edit candidate. Unrelated
invalid drafts cannot mask a cycle. No iterative fixed-point calculation is allowed.

Dependency indexes must identify affected membership guards and producers; reads do
not repair stale bindings. Structural edits, sort, source retirement and refresh stage
all affected metadata or refuse atomically on cancellation/work/admission failure.
Existing worksheet-only draft behavior remains unchanged outside producer candidates.

## Lifecycle and editor

The binding creation preview shows source worksheet/range or fixture provenance,
physical extent, included hidden positions, key/schema policy, update mode and
read-only behavior. The Window list/properties show bound status, source, selected
generation and current/refreshing/stale/disconnected diagnostics. Worksheet binding
is labeled **Live worksheet**; fixture binding is **Explicit refresh**. Only the
latter has Refresh. Grid focus, selection, scrolling and nonuniform small extents use
the existing virtualized view; unavailable sources display an explanation.

Frame requests/replies carry explicit WindowId, expected state, producer/configuration
and captured generation qualification. Delayed frames cannot replace another scope
or a newer generation. Client selection stays local. Server checks read-only authority
on every mutation path, not just disabled buttons. Native Copy/Cut/Paste involving
Window authority still refuse before clipboard mutation. Explicit Copy Values may
export displayed typed text from one bounded captured generation; errors refuse and
String/leading-zero prefixes remain intact. Explicit Paste Text into bound content
also refuses. No implicit write-back or clipboard-triggered refresh exists.

Window deletion faults only references guarded by that Window; producer deletion
faults all its views and leaves source worksheet cells untouched. Use exact inbound
review, expected-state and one-use deletion authority. Worksheet deletion or complete
descriptor retirement leaves bound views unavailable with persistent retired-source
authority; another worksheet at the old name cannot repair them. Disconnect preserves
the selected external snapshot and value availability, clears local refresh capability
and invalidates in-flight tickets; it is not source deletion. Rename changes labels.

Detach is explicit snapshot materialization into a **new unbound Window** with fresh
owned IDs. Admit exact typed values and built-in formatting from one captured frame
set before creation; formulas are not copied as hidden dependencies, and errors or
unsupported values refuse the operation. Existing references remain attached to the
old bound Window, which is not silently deleted. A large detach may refuse existing
edit/creation limits; no limit increase or streaming partial document is authorized.
Rebind is explicitly create-replacement; it never retargets existing references.

## Persistence and representation

Propose **WB15 only** and **private session v15**, freezing final numbers with the
implementation codec audit. Do not emit new authority labeled WB14/v14. No historical
reader or migration; WB14/older explicitly refuse once this milestone is promoted.
Hosted v1 remains unchanged; CB5/CB6 remain unchanged and cannot transport producer
or Window authority. Experimental history/structured paths must refuse unsupported
new ownership instead of dropping guards. ADR-0023 remains parked at G3.

Persist source producer/Window IDs, immutable configuration/schema/FieldIds and key
policy, source selections/effective geometry and retired authority, selected fixture
snapshot/digest/order, typed values, guarded scalar/range bindings, shared external
output identities and any fault metadata required by live references. Persist one
selected external snapshot per producer, not a value copy per Window or full refresh
history. Older captures retain old allocations only in memory; serialization retains
referenced identity metadata required for correct current faults.

Fixtures in this slice explicitly allow snapshot persistence, disclosed at creation.
Future sensitive/nonpersistable policies are not assumed: Save must refuse an included
source that forbids persistence, never omit its values. Saved local capability IDs,
credentials, consent, attempt tickets and executable provider instructions are forbidden.
Reopen validates schema, key uniqueness/policy, field types, bounds, selection and
fault authority, shared leaves, generation contexts, cycles and aggregate costs before
install. Worksheet producers rebuild from the same restored canonical roots; external
saved snapshots open disconnected and readable. Pending attempts become interrupted
diagnostics. Reconnect requires an explicit local authorized action and matching
provider contract; it does not refresh. Missing provider does not prevent reading a
valid saved snapshot. Corrupt authority refuses the entire native load.

### Representation acceptance gate

WB15/private v15 remain provisional until the codec audit establishes complete
authority closure. Visible-value round trips are insufficient.

| Encoded authority | Required pre-install validation |
| --- | --- |
| Producer/configuration/Window | Stable non-reused IDs, ownership variant, correct relationships and configuration revision; no substituted identities or duplicate producer ownership |
| Schema/fields | SchemaId/FieldId ownership, unique fields, fixed declared types/order/key contract, matching selected snapshot and output declarations |
| Items/generations | Exact durable versus generation-qualified identities and generation association; unique opaque keys; no ordinal/value-equality rebinding |
| Selected snapshot | GenerationId, canonical payload digest, schema/types/slot counts/order and provenance agree. Shared data is one producer generation, not copies posing as independent sources. |
| Worksheet geometry | WorksheetId, source SelectionId/effective endpoints, composed subdescriptor membership and retired authority form a legal relationship |
| Guards/faults | Window/producer/configuration/source targets match; active, temporarily excluded and retired states are consistent; reactivatable metadata cannot grant retired authority |
| Shared leaves | Unique producer/configuration/item/field output authority, graph bindings and captured contexts agree, including missing-value/source faults and producer-aware cycles |
| Local-only authority | No credentials, capabilities, tickets or executable provider instructions. External snapshots reopen disconnected, without provider execution. |

A matching digest only proves bytes match a claimed digest; it does not authorize
schema, ProducerId, key policy, membership or outputs. Validate the cross-record
relationships, graph membership, identity uniqueness and resource costs as one bounded
candidate before any install. Consistent saved provenance is not cryptographic proof
of an external business identity promise or current freshness. Reconnection separately
matches the provider contract. Hostile tests must recompute digests over forged
payloads and still prove refusal of mismatched schema/policy/producer/generation/
guard/leaf authority. Unsupported state cannot be omitted or imported as unbound cells.
Freeze format numbers only after this gate is audited.

## Admission and resource objectives

No global limit increases. Existing 64 live Windows and 16 worksheets remain aggregate.
Specialize ADR-0039's proposed producer admission to at most 32 live producers,
64 fields per external schema, and 100,000 projected logical rows across live bound
Windows; duplicate views consume that row admission even when storage is shared.
These are initial refusal ceilings, not measured capacity promises. Worksheet sparse
rectangles retain physical extents without materializing their Cartesian area.

Selected external generations are additionally bounded to 65,536 total record-field
slots across producers, including Blank slots, 256 UTF-8 bytes per opaque key, and
4,096 UTF-8 bytes per String value. Encoded snapshot/key/schema bytes share the
existing 16-MiB aggregate source/payload allowance; they are not an additional pool.
All producer metadata, rows, keys, indexes, contexts and outputs participate in the
existing 96-MiB workbook accounted-byte admission and enclosing allocation domain.
Format and private-message size caps still apply; the smallest applicable cap wins.

Demanded source Blank anchors and external scalar leaves consume existing aggregate
65,536 live computation-node/cell and 262,144 allocated-identity ceilings, with no
per-producer allowance. Producer/field/item/output/tombstone ledgers are explicitly
admitted within that identity allowance; projected display coordinates alone do not
allocate identities. Immutable snapshots are charged once, while pins, membership
indexes, request buffers, decoding scratch and refresh staging are charged separately.
Old captures plus selected/staged generations coexist under the same limits. Bound
active attempts to one per producer and two across the session; a superseded task
continues consuming its staging reservation until actually released. Never reclaim
an allocation while an older capture/task owns it. Meter membership/graph validation
against existing transaction work budgets, not an unbounded scan at read time.

### Single ownership and charge ledger

Maintain one workbook producer-admission ledger keyed by object kind and authoritative
identity/allocation. It records owner, identity counts, byte/work categories and
retained ownership. Directory/Window/graph references link to these entries rather
than independently inventing producer budgets. Existing core/selection ledgers stay
enforced; cross-links count a shared object once in each applicable limit, never once
per Window. Identity counts and byte counts are separate, not interchangeable units.

| Resource | Authoritative owner / identity responsibility | Byte and lifetime responsibility |
| --- | --- | --- |
| Producer/configuration/schema/FieldIds | Producer directory enters each allocated stable ID once. Numeric revision counters are not new identity objects. | Own immutable configuration/schema bytes; additional references/lookup entries pay for actual independent allocations. |
| Generation/item keys | Producer owns GenerationIds and key tables. Durable item authority is producer/configuration/key; generation-qualified adds generation. Demanded/retained item IDs enter allocated-identity accounting; ordinals do not create CellIds. | Generation owns key/value bytes and order/key indexes once. Duplicate view rows still consume logical-row admission. |
| Window/membership guards/SelectionIds | Directory owns WindowId; bindings own guard metadata; canonical selection registry owns shared source SelectionIds. | Charge distinct guards/index entries and descriptor references. No per-position anchor or per-Window payload copy. |
| Worksheet anchors | Canonical workbook/graph owns source CellIds/nodes and demanded Blank anchors under existing ledgers. | Source remains charged at its owner; producer pays additional links/indexes only. |
| Captured Decimal outputs | Output registry owns one shared leaf per authorized producer/configuration/item/field; applicable graph/captured identities link to that object. | Charge actual leaf/context/index allocations once; consumers pay their own edges/guards, not another leaf per Window. |
| Shared payload/indexes | Producer generation owns immutable storage; each additional distinct index has an allocation entry. | One physical allocation has one domain charge. Workbook footprint accounting is not a second domain reservation of the same bytes. |
| Old generations/ScopeCaptures | Retain original owners and capture/root links until release. Retirement follows existing non-reuse ledgers; replacement never resets them. | Payload stays charged once while pinned; actual pin/context/root wrappers pay separately. Removing a selected head does not free retained data. |
| Failed/superseded staging | Attempt/task owns staged allocations and producer/ticket links; rejected IDs cannot repair existing bindings. | Charged until buffers/reservations actually release, even after cancellation/deletion/supersession. No phantom reclaimed slot. |
| Decoder/validation scratch | Import/mutation transaction owns temporary lookup tables, relationship/cycle work and output buffers. | Reserve before decode/allocation. Old roots, candidate, scratch and captures coexist; rollback releases only actually unowned allocations. |

Ledger totals must reconcile directory, Window and graph statistics. A projected row
can consume no CellId yet require indexing/serialization bytes; a shared output can
consume one identity while multiple guards/edges allocate separately. Independent
tests must detect missed charges and duplicate charges.

Required accepted coexistence workload: 32 producers comprising one fixture producer
with one selected generation of 128 keys/two fields (Decimal and exact String), plus
31 worksheet producers. Two Windows per producer give 64 views. Each worksheet view
spans 16 physical rows/two columns: 992 worksheet view rows plus 256 fixture view rows,
or 1,248 logical rows. Populate 992 worksheet source cells, add 64 formula consumers
and demand 32 distinct shared Decimal leaves (1,088 total nodes, including duplicate
consumers through both fixture Windows). This separate ownership workload supplements
the 1,024/10,000/65,536 checkpoints; it does not claim all maxima combine. Assert one
selected fixture payload, shared outputs and reconciled exact IDs/bytes. Repeat with
an old capture and admitted staged replacement. If this representative workload does
not fit, return measured evidence for review, without increasing limits or silently
reducing the contract.

The implementation must report these categories separately and reserve before
allocation, including decode and validation scratch. Exhaustion/cancellation refuses
atomically. Keep the existing one-second command/engine work, ten-second persistence,
256-MiB steady and 512-MiB peak objectives for declared qualification corpora; provider
delay is reported separately, never used to hide workbook publication time. If a
required corpus cannot fit, stop and return the measured conflict for review rather
than silently raise limits or weaken identity/consistency semantics.

## Implementation and validation plan

Implementation requires separate owner authorization after design review. Order:
producer/ownership authority and admission; worksheet geometry/guards and cycle
maintenance; immutable fixture contexts/refresh; native/private audit; read-only editor
and lifecycle; then qualification and retention. No feature work accompanies this ADR.

Required regressions:

| Area | Required evidence |
| --- | --- |
| Mapping and values | Local offsets, no header inference, Blank/empty String/leading zeros, exact Decimal metadata, errors, hidden positions and source style changes |
| Source transformations | Edits/recalc, sort identity following, every insertion/deletion boundary, partial/full descriptor deletion, range overlap refusal, same-identity re-entry versus occupant reuse |
| Reference authority | Worksheet/Window scalar and range consumers, demanded Blank budget, shared external leaves, relative/$ copy, fault copy, unsupported external types/ranges and read-only mutations |
| Generations | Durable/generation-qualified keys, reorder, absent/reappearing keys, duplicate keys, schema/type mismatch, missing fields, exact key bytes, zero partial publication |
| Refresh races | Two attempts completing backwards, replay, config/connection replacement, deletion, cancellation, provider failure, no first success, last-good retention and unrelated workbook edits |
| Coherence | Old captures across edits/refresh/retirement, selected contexts and guards from one generation, delayed frame refusal, no read-triggered I/O |
| Cycles | Direct feedback, indirect cross-Window/range feedback, unrelated invalid drafts, membership restoration and complete rollback |
| Lifecycle/native | Window versus producer/source deletion, disconnected reopen without provider, explicit reconnect/detach/replacement, retired names/keys, malicious schema/guard/context authority and unsupported versions |
| Resources | Sparse empty geometry, shared view storage versus logical-row cap, retained snapshots/captures/tasks, decode scratch, identity exhaustion, bounded cancellation and zero final charges |

Use independent mapping/membership and generation-state oracles, not tests that mirror
the implementation's lookup helper. Preserve all original failures. Resource checkpoints
are 1,024, 10,000 and 65,536 total live source/consumer cells and external leaves,
reserving consumer nodes within each total, with 32 producer descriptors/64 Windows within
100,000 view rows, sparse large rectangles, multiple old captures and up to two
refresh staging tasks. Separately exercise external field-slot/payload boundaries and
one accepted near-capacity workload; rejection at a byte/work cap is distinct from a
completed value error. Report edit/guard maintenance, publication, evaluation, Save,
reopen, charged steady/peak and final charges separately. Do not claim every maximum
can be combined simultaneously or multiply memory allowances per producer.

Plan six-target headless CI and managed/AOT GUI coverage: Windows/Linux x64,
Windows/Linux ARM64, macOS ARM64 and macOS x64 under Rosetta. Use a Windows x64 pilot,
then owner-run short launchers in each graphical terminal under `docs/testing.md`.
The GUI flow must demonstrate live worksheet update without Refresh, explicit fixture
refresh/failure retention, read-only refusal, exact String export, status, stale frame
protection, source deletion and disconnected Save/reopen. No GUI is launched through
SSH. Record selected source hashes, archive verified evidence before cache cleanup,
and report cloc per project from the canonical checkout at implementation completion.

Owner reports all six systems updated to Rust nightly 1.101.0 dated 2026-11-01.
Future receipts must capture actual compiler identity; no toolchain commands or tests
were run for this design. ADR-0040's Windows ARM64 cold-run timing failure remains
Failed and unexplained; future passing runs do not resolve it implicitly.

## Exclusions and alternatives

Exclude production SQL/providers, arbitrary query languages, write-back, nested
Windows, Text, section geometry, pagination, JSON invocation, PDF, licensing and
history integration. No computational external range or general typed captured
source expansion is authorized. No new independent producer calculation engine.

Copying worksheet values would lose live source identity and coherent updates;
creating alias nodes for every display position would multiply graph and identity
budgets. Both are rejected. Automatically refreshing on read makes capture dependent
on side effects and races, so refresh stays explicit. Rebinding by label/ordinal on
missing keys would silently change a formula's subject, so missing authority faults.
Adding a full database provider now would obscure the immutable producer contract;
deterministic fixtures qualify that contract first.
