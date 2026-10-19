# Workbook projection and spreadsheet semantics — design review handoff

Date: October 19, 2026.
Disposition: Proposed ADR-0010; design only; stop for review.

## Direction and scope

The owner accepted the parent review direction after Document Foundation Slice B.
Preserve the experimental file workflow and tests, and stop expanding document
infrastructure. The next proof should be a spreadsheet projection over the general
graph, then application/C# integration, rather than more hosted storage.

This packet defines proposed ownership, sparse representation, reference semantics,
coordinated edits, range/incremental implications and viewport access. It makes no
runtime, grammar, protocol, GUI, dependency or format changes. ADR-0010 remains
Proposed; existing ADRs retain their status and historical decisions.

## Review packet

| Deliverable | Document |
| --- | --- |
| Proposed decision | [ADR-0010](../adr/0010-workbook-projection-and-spreadsheet-semantics.md) |
| Document/workbook/sheet/model ownership | [Ownership](../architecture/workbook-ownership.md) |
| Sparse/chunked cell and axis storage | [Sparse worksheet storage](../architecture/sparse-worksheet-storage.md) |
| Stable graph projection and target lifecycle | [Graph projection](../architecture/workbook-graph-projection.md) |
| Relative/absolute/mixed/sheet/name/range semantics | [Formula analysis](../architecture/spreadsheet-formulas-and-ranges.md) |
| Atomic structure/content/graph edits | [Workbook transactions](../architecture/workbook-edit-transactions.md) |
| Range dependencies, cycles and incremental implications | [Calculation](../architecture/workbook-calculation.md) |
| Rectangular reads, caching and geometry | [Viewport proposal](../architecture/workbook-viewport-access.md) |
| 10K/100K/1M workloads and semantic oracle | [Verification plan](../architecture/workbook-performance-verification.md) |

## Concrete proposals for review

One graph can span several worksheets and contain nodes without cells. Standalone
model documents and named-interface invocation remain independent of spreadsheets
and optional publication. New workbook logic belongs in a dedicated Rust boundary,
provisionally pivot-workbook when implementation is approved, not in pivot-storage.
No speculative relocation of existing document modules is required now.

Stable cell anchors are distinct from axis positions and graph nodes. Ordinary
content replacement preserves the anchor; Clear keeps a blank target; Move carries
identity; structural deletion retires it. Reusing a deleted/vacated coordinate cannot
retarget old formulas. Referenced blanks are sparse anchors; unused/formatted empty
positions require no executable nodes. Initial numeric cells use core input/compute
nodes, with explicit room for future typed sources and proxy bindings.

Sparse chunks are indexed by stable axis-run identities, with compact default axis
runs, adaptive occupancy and independent styles/geometry. Proposed path-copy updates
avoid a complete worksheet copy, but current core staging/snapshots still copy or
rebuild whole graph structures. These are separate bottlenecks, not solved by a
sparse cell map or increased byte limit.

The shared Rust parser/binder must evolve for spreadsheet reference descriptors;
current identifiers/arithmetic do not already support A1, ranges or SUM. Preserve
original source plus stable bindings and copy flags. Render effective addresses from
bound targets after structural edits; never reparse historical source to recover
meaning. There is no C# formula evaluator.

Ranges bind compact spatial selections, not scalar lists of every position. Their
captured membership participates in dependency ordering, invalidation and cycles.
Sparse membership avoids blank-area allocation but cannot make a genuinely dense
dependency graph free. Checked decimal aggregation order must remain deterministic;
chunk summaries or parallel reassociation need semantic proof.

Atomic edit installation coordinates sheet, anchor/reference, graph and presentation
roots. Calculations capture one coherent graph/selection generation; later edits
continue. Viewports receive bounded rectangular batches with coverage and provenance,
separate content/value/style/geometry fields and stale-result rejection.

## Observable policies needing acceptance

These are explicit initial choices, not compatibility claims:

- Clear means Blank, distinct from structural deletion and from numeric zero.
- Scalar references follow moved cell identities; `$` controls copying, not immunity
  from structural target movement.
- Copy/fill transforms relative axes from current target/source positions; explicit
  sheet qualifiers remain bound, unqualified copies resolve on the destination sheet.
- Range membership is spatial. Insertions strictly inside expand it; immediately
  outside boundaries do not. Deletions contract surviving axes; a fully removed
  axis leaves an invalid selection. Moving cell contents does not move the region.
- Proposed numeric arithmetic coerces Blank to zero; decimal SUM ignores blanks/text
  and propagates errors in visual row-major order. Typed values/functions are future
  core work; the present engine does not implement these semantics.
- First proof rejects occupied-destination moves. Alias/model-port projections need
  explicit ownership/proxy rules before addressable shared views are introduced.

The packet does not promise Excel compatibility, dynamic references, external-link
refresh, cross-graph call scheduling, collaborative editing or a production format.
Those choices must not be smuggled into an implementation under an A1-shaped syntax.

## Acceptance review coverage

All nine assignment areas are addressed in the linked documents: ownership, sparse
storage, graph projection, references, first-class ranges, atomic edits, viewport/GUI
contract, performance verification and architectural compatibility. Range-cycle and
partial structural-error evaluation gaps are explicit. Every scale workload is a
measurement target rather than supported capacity.

The proposed first implementation remains a small in-memory sparse worksheet/graph
proof. After review, sequence the executable C# client and later bulk viewport/grid
integration alongside bounded core reference/range evolution. Do not return
automatically to hosted persistence or grow the experimental file broker.

## Slice B acceptance and production backlog

Slice B is accepted as an experimental milestone, not as production file-format
approval. Preserve its implementation and all existing tests. Required production
issues remain separate, nonblocking for this workbook design:

1. Windows replacement adopts the candidate's private ACL and does not preserve
   arbitrary original ACL/alternate-stream metadata.
2. Uncooperative writers can race the final fingerprint check and replacement.
3. Native macOS file behavior remains unverified.
4. Process termination tests do not prove power-loss durability.
5. Full-copy graph staging and current small graph budgets are not spreadsheet-scale
   solutions.

Address these through separately reviewed production requirements; no broker changes
are authorized by this design slice. Production container selection remains open.

## Validation and stop boundary

This pass changes Markdown only. Verify local links, UTF-8, code fences and diff
whitespace. No new workbook tests or performance measurements are claimed. The
previous 185 Windows/186 Linux Rust tests and Slice B CI reports are historical
evidence for existing behavior, not evidence for this unimplemented design.

No Cargo/dependency, source, fixture, protocol, file broker, GUI or core behavior
changes are included. Stop after Proposed ADR-0010 for architecture review.
