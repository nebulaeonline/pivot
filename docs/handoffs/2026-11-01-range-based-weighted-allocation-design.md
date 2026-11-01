# ADR-0038 — range-based weighted allocation design review

2026-11-01. Design approved with four clarifications. No implementation, benchmark
or platform execution.

See [ADR-0038](../adr/0038-range-based-weighted-allocation.md) for the full contract,
source findings, bounds and alternatives. ADR-0037 remains accepted and unchanged;
ADR-0023 remains parked at G3. This document records the technical contract and required qualifications.

## Approved user contract

`=ALLOCATEWEIGHTEDRANGE($F$1,C2,2,$B$2:$B$4)`

Use a separate function, with required total, explicit one-based index, precision
0..28 and one literal row/column range of 1..64 physical positions. Singleton syntax
is B2:B2, not a scalar B2. Reverse endpoints normalize to ascending coordinates.
General rectangles, computed/multiple ranges and a larger cap are deferred. The distinct
name leaves scalar ALLOCATEWEIGHTED unchanged and makes deleted-range #REF! restoration
unambiguous; the previously illustrated overload was not a frozen API.

**Blank is a type error, never a skipped recipient or implicit zero.** Weights 1,Blank,3
mean three positions and fail for every valid index. Entering explicit zero yields
2.5,0,7.5 at total 10/precision 2. Empty String, Boolean and numeric-looking String are
also type errors; negative/all-zero weights are invalid. Errors retain provenance.
All weights are validated in ordinal order after the three metadata operands and
range admission; index<=N and arithmetic cross-checks follow local weight validation.

## Association and structural policy

Recipient j selects position j in the captured spatial range. Existing SelectionId,
worksheet identity, endpoint-copy flags and structural affinity apply. Interior inserts
add slots; before/after inserts remain outside; deletion contracts or produces a retained
range fault. Inserted Blank causes TypeMismatch; a 64->65 expansion causes completed
InvalidArgument without rejecting the structural command solely for this cap. Explicit
indices are never repaired. Filtering does not remove hidden positions.

Sorting weights 3,1 in B2:B3 changes range index 1 from 7.5 to 2.5 at total 10. This is
spatial membership, deliberately different from scalar-list references following moved
cells. Neither policy certifies business-record association or a complete group. Values,
geometry and references are read from one immutable capture, not a viewport or later state.

## Required representation and bounded collection

The current occupied-member selection API omits Blank/absent positions. Add a separate
bounded dense slot accessor with metered index lookups, preserving exactly N slots and
repeated-node positions. Do not alter SUM membership or create Blank anchors for reading.
Descriptor subscriptions must invalidate on absent/Blank/occupied transitions and reframing.

The approved dependency decision is: an unsupported shape/over-limit live range retains
its descriptor subscription but has no range-member data edges, since it returns a
bounded domain error before reading values. Scalar dependencies remain. Restoring valid
geometry restores member dependencies and must cycle-check before atomic publication.
Valid ranges retain normal conservative cycle validation. This avoids traversing a huge
invalid rectangle merely to reject it; import must reconstruct rather than trust saved edges.

Recommend a new range operand and operation 21, fixed arity four, validated as three
scalar roots plus a bound range or deleted-range fault. No general range-valued PivotValue.
The canonical range-aware task collects weights then delegates once to metered operation
20. Direct scalar typed calls cannot reinterpret the new operation as one scalar weight.

Approve **WB13-only**, because a new bound operand/token shape is needed; no WB12
reader/migration is proposed under the existing format-cut policy. This compatibility
impact is explicitly accepted under the pre-release policy. Graph tag 8 and worksheet tag 9 appear free, subject to
a full codec audit. Propose keeping private v13/hosted v1/CB5/CB6 because existing source,
endpoint and fault-origin fields appear sufficient; actual field changes require review.

## Resource, arithmetic and validation decisions

Keep the same exact ADR-0037 arithmetic and actual SUM guard. Reserve at most 8 KiB
additional adapter scratch before collection, plus unchanged 16-KiB numeric reservation.
Charge descriptor/slot steps and actual index nodes through existing visits; retain
numeric 4+16*N charges and cancellation within collection, division and ranking. No
hidden allocation cache, uncharged path or global quota increase. Prove layout and work
bounds before implementation acceptance.

Independent oracles must cross-check dense range results against scalar lists and real
SUM. Cover Blank variants, zero/error/hidden slots, all indices, source reversal, 1/64/65
and huge rejection, strict errors, laziness, old captures, absent->zero->Blank invalidation,
geometry transitions/cycles, structural affinity, sort association, Copy/Fill, cross-sheet
behavior, native/clipboard fault preservation and malformed range/selection authority.

Declare 16,384-base-cell corpora with 64-position vectors, complete-group SUM and 256
calls, four changed-input samples/four retained captures. Compare scalar and range work;
include lookup/dependency/adapter costs and failure/cancellation paths. Keep the original
one-second and 256/512-MiB objectives, reporting observed core maxima and charged memory.
Require all-six-target headless/parity plus managed/AOT GUI qualification for the new
reference, clipboard/rendering and WB13 paths; macOS x64 remains Rosetta. No tests or VMs
have run. Preserve failure/source qualifications, verified pruning and diagnostic cloc.

## Four mandatory clarifications

- Preserve one slot per physical position, even when NodeIds repeat. Never compact
  missing slots or deduplicate the weight vector. Independent tests must distinguish
  this from the existing occupied-member selection API.
- Keep descriptor subscriptions on rejected geometry without member traversal. Every
  geometry transition must invalidate reliably; restored admissibility reinstates
  member edges and cycle validation before publication, without stale values/errors.
- Enforce operand authority, not only four-argument arity. Only three scalar roots and
  an authorized range/deleted-range operand may execute operation 21. No scalar fallback
  or range leakage. Tags 8/9 remain subject to complete codec audit.
- Keep WB13-only explicit, without WB12 readers or migration. Older artifacts remain
  historical/refusal fixtures. Retained private/hosted/clipboard versions depend on
  proving that existing fields preserve all required authority and fault origins.

The separate name, strict Blank policy, vector/position cap, spatial association,
independent 8-KiB adapter admission and unchanged allocator bounds are accepted.
The full six-target headless/managed GUI/AOT scope is required for the new reference,
clipboard and native-representation behavior. No execution or implementation is included
in design approval. Range-only allocation remains the scope; keyed groups, persistent
recipients, repair, arrays/spills, visible-only weights, fairness, licensing implementation
and history are excluded.
