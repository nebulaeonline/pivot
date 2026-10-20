# First-class ranges and SUM — semantic/design review

Date: 2026-10-20. Historical design packet: subsequently accepted by the parent
review received 2026-10-20. [Implementation requirements](2026-10-20-ranges-and-sum-review.md).
The proposal/review questions below preserve the submitted design; implementation
is now authorized and has not begun.

Slice B and ADR-0015 are accepted. The next assignment explicitly requires range
semantics to be approved before the permanent representation is committed. The
owner selected parent review first after receiving the concrete ADR-0016 proposal.
No parser, evaluator, document bytes, product limits, session or UI code changed.

## Decision requested

Approve or revise [ADR-0016](../adr/0016-first-class-ranges-and-sum.md), particularly:

1. **Coordinate-area affinity:** a range binds to a worksheet's axis namespaces and
   inclusive normalized rectangle. Its members are current occupants in the captured
   generation. Moving an endpoint occupant does not move the rectangle. Moving the
   formula preserves the bound rectangle. Scalar references keep their existing
   identity affinity; mixed formulas deliberately combine these two policies.
2. **Exact deterministic SUM:** numeric inputs and calculated members contribute in
   row-major order, with existing checked decimal addition from zero. Empty/blank
   positions contribute zero without nodes/anchors. Member errors propagate;
   intermediate overflow is an error even if later cancellation would fit.
3. **First-class dependencies:** formula → selection → sparse member relations live
   in the canonical core. No durable edge/token/blank anchor per coordinate and no
   live workbook callback or second evaluator. Graph and membership generations
   commit/capture atomically.
4. **Explicit format cut:** selection-aware native `PIVOTWB2`, rejecting WB1 with a
   clear unsupported-version result. No deployed workbook files require migrations.
   Frozen hosted storage/protocol v1 stays unchanged and rejects unsupported selection
   capability rather than flattening or omitting it.

These are proposed Pivot policies, not inherited Excel behavior or already accepted
ADR-0010 semantics. Representation/API details and exact admitted selection quotas
still need implementation evidence; this packet does not claim benchmark results.

## Worked behavior

Each independent case starts with B2=1500, B3=2350, B4=1275, B5=900 and
D1=`=SUM(B2:B5)`, producing 6025.

| Operation | D1 result | Explanation |
|---|---:|---|
| Change B3 to 2400 | 6075 | Current included value changes |
| Clear B3 | 3675 | Existing scalar anchor can remain; selection treats blank as zero |
| Move B3 to C3 | 3675 | The cell leaves the fixed rectangle |
| Move B3 to B10 | 3675 | Moving an endpoint occupant does not move bounds |
| Move B3 to C3, then put 10 into B3 | 3685 | New occupant is included, without rebinding D1 |
| Retire B3, then put 10 into B3 | 3685 | Range sees new occupancy; old scalar references remain invalid |
| Move D1 to D2 | 6025 | Formula movement preserves its bound selection |
| Move D1 to a vacant B position inside its range | Cycle | The moved formula becomes a member of its own selection |

The last case uses a fresh otherwise-equivalent fixture with a vacant interior
position, since the existing Move command refuses an occupied destination.
For a populated endpoint example, moving B2 to C2 excludes 1500 while preserving
the same bounds and gives 4525. `SUM(B5:B2)` has the same normalized bounds/result.

For sparse insertion, start with B2=10 and B100=20 and D1=`=SUM(B2:B100)`.
Initially D1=30 and B3:B99 are not materialized. Entering 5 into B50 makes D1=35.
An existing blank scalar anchor at B50 contributes zero until populated; neither
binding the range nor evaluating it creates the other blank coordinates.

For transitive propagation, A1=2, B2=`=A1*3`, D1=`=SUM(B2:B100)` gives 6. Editing
A1 to 4 must invalidate D1 and give 12 even though A1 is outside the range.

For S/T, capture D1=6025 at S, then clear B3 in T. Selected evaluation of S still
gives 6025; T gives 3675. Saving S while T exists preserves the earlier values,
placement relation and bound selection together, and leaves T dirty.

For overflow, place Decimal96 maximum, 1 and -1 in ascending member positions.
SUM fails at the second addition. A different row-major ordering may succeed;
there is no hidden wider accumulator or associative reduction promise.

## Existing-code findings

The canonical core currently stores scalar BoundToken references and scalar
forward/reverse dependencies. Local closure, cycle validation, dependency inspection,
transfer and restoration all assume those edges. The workbook already has stable
axis namespaces, sparse positions and atomic persistent graph/projection updates.
Therefore adding only `SUM` to the parser and rebuilding scalar formulas after edits
would not satisfy the assignment. The shared core traversal must understand a
selection capability and immutable membership root.

The proposed sparse relation is indexed by populated rows and columns, shared once
per space. A separate rectangle/consumer index identifies aggregates covering a
changed point. Propagation must also consult placements of affected computed members,
plus old/new positions on moves and removals. Candidate/retained roots, not live UI
lookups, supply membership. Do not duplicate each overlapping range's member list.
Measure visited rows, interval candidates and emitted members; a narrow range can
still visit populated rows or rectangle candidates outside its columns. Overlap and
actual member traversal are genuine costs, not constant-time operations.

## Implementation sequence after approval

1. Extend canonical resolved expression/selection, persistent membership and consumer
   indexes, topology/local-closure/inspection and atomic admission contracts together.
   Add dependency/cycle and S/T tests before exposing syntax in the application.
2. Extend spreadsheet mode for one rectangular argument to SUM and scalar arithmetic.
   Keep original source, endpoint/spans and stable selection provenance; invalid input
   remains editable. General named-model grammar is not silently expanded.
3. Add selection-aware portable graph/workbook representation and the explicit format
   version cut. Restore stable descriptors and sparse placements without resolving
   original addresses again. Retain corruption/resource/failure and saved-generation
   guarantees through the existing broker.
4. Exercise the existing C# session/Avalonia editor against real Rust calculations
   and document round trips. Add only boundary fields actually needed for versioning
   or meaningful errors; C# does not calculate SUM.
5. Collect guarded correctness/resource/performance evidence and cross-platform
   results. Manually exercise native pickers and dirty-close outcomes on the primary
   desktop, then submit the implementation handoff for review.

## Required acceptance cases and evidence

| Area | Required cases |
|---|---|
| Syntax/provenance | SUM case/whitespace, reversed endpoints, scalar combinations, exact original UTF-8 spans, unsupported functions/argument forms, naked ranges, malformed/out-of-bounds drafts and repair |
| Membership | Never-populated space, blank anchors, clear, numeric/computed insertion, move within/across boundary, endpoint movement, retirement/reuse, formula movement and repeated/overlapping SUM terms |
| Dependencies | Direct member changes, computed members depending outside the rectangle, scalar/selection mixed cycles, self-inclusion, multiple aggregate cycles, unrelated invalid draft isolation, complete global validation |
| Arithmetic | Independent decimal oracle, deterministic row-major error/overflow behavior, empty zero, errors within members, supported type checks |
| Transactions | Sequential batches, rollback on late hard failure, exact expected tokens, cancellation during index/topology/evaluation work, allocation failure, immutable S/T membership and values |
| Documents | Independent-process reopen preserving selection IDs/bounds/policy and scalar identities, drafts/provenance, retired ledgers, malformed records/versions, no historical A1 rebind, Save S/edit T and file failure/conflict behavior |
| Sparsity/resources | Large mostly-empty rectangles within current dimensions, no blank-node/endpoint-anchor growth, shared point-index growth, selection consumer/index growth, cancellation and zero-charge orderly cleanup |
| Measurements | Binding, member value edit, add/remove/move, invalidation, selected evaluation, retained S/T, memory/index counters, Save/Open components and complete C# request latency |
| User workflow | Actual Rust-backed SUM entry/edit/save/reopen; manual native Open/Save As and Save/Discard/Cancel close outcomes; stale-response guards and existing scrolling/editing behavior |

A full-sheet selection includes its own formula if that formula lives on that sheet,
so it is a cycle test. For a valid large sparse sum, put the aggregate outside the
rectangle (for example, select almost the full sheet while excluding its final
column). Do not use a full-sheet self-reference as a zero-allocation success fixture.
Viewport read-area bounds and range geometry are different concerns; do not enumerate
large ranges with one request per coordinate or raise product caps to make tests pass.

Retain all known Slice B limitations: initial WSLg native abort, mounted-Windows-path
replacement, manually untested dialogs and serial disk occupancy of the child request
slot. If the native abort recurs, capture an actionable backtrace; native disk passes
do not establish mounted-share behavior. No SQL, alternate engine, broad functions,
copy/fill, structural transforms, formatting expansion, AOT/installer or capacity
milestone is included. Stop at this design review before implementation.

[Accepted Slice B review](2026-10-20-workbook-persistence-review.md),
[ADR-0010](../adr/0010-workbook-projection-and-spreadsheet-semantics.md),
[accepted native document ADR](../adr/0015-native-workbook-documents.md).
