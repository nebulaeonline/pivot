# Workbook performance and verification plan

Status: Verification plan under accepted ADR-0010. Slice A reports small proof costs;
10K/100K/1M workloads, allocator profiling and scale gates remain unexecuted targets.

Review clarification: ADR-0010's structural architecture is Accepted. Reference
transforms, range boundaries/movement, blank/text coercion, reduction order and
shared writable aliases remain separately reviewable compatibility semantics.
[Slice A](workbook-projection-foundation.md) implements only the bounded typed
single-sheet proof; future mechanisms described here are not claimed implemented.

## Semantic and transactional oracle

Before optimizing, use a small independent positional/reference model to check
commands and stable identity behavior. Compare incremental results with clean full
evaluation using existing numeric semantics. Do not derive expected references by
calling the same production transform under test. Retain deterministic seeds and
minimized failing operation sequences.

| Scenario | Required assertion |
| --- | --- |
| Numeric input/formula cells | Same decimal results as a standalone model; exact input/literal/default representations retained |
| Empty/text/style-only positions | No node for an unreferenced empty/style-only position; referenced blanks and text follow the reviewed value policy |
| Rename and coordinate reuse | Labels/positions can change without target identity changing; a deleted target never rebinds to a new occupant |
| Clear versus delete | Clear retains a blank target; structural deletion retires it and diagnoses references |
| Move versus copy/fill | Move retains targets; copies apply the relative/absolute/mixed transformation table and allocate/update destination bindings explicitly |
| Sheet rename/cross-sheet references | Bind WorksheetId; current rendered labels can change without historical name lookup |
| Range insertion/deletion/movement | Exact boundary-affinity and spatial membership rules, including empty/deleted selections |
| Range cycles | Self-inclusion, indirect scalar/range cycles and hidden formula producers are detected; no silent exclusion of the consumer |
| Checked decimal aggregation | Defined traversal and error order; optimization agrees with the non-reassociated oracle |
| Atomic paste/row edits | Readers see old or new whole aggregate; faults/conflicts install no partial coordinate or graph state |
| Snapshot isolation | S calculates against S definitions/inputs/selections while T is edited; no stale S result installed as T |
| Formatting-only changes | Dirty document, unchanged computational revision; numeric cache reuse remains valid |
| Invalid drafts | Original source and bindings survive; valid arithmetic errors remain isolated as today; unsupported structural-error isolation is not claimed |
| Viewport paging | Coherent revision, bounded bytes/work, explicit empty coverage, forward progress and stale-generation rejection |
| Model independence | Core and model-only documents still work without workbook, publication, GUI, database or daemon |

Add property tests over randomized sparse edits, row/column operations and
copy/move/delete sequences. Exercise retired anchors, shared immutable roots and
malformed owned projection data. Tests should verify invariants and observable
semantics rather than mirror chunk implementation details.

## Workload matrix

For each populated-cell target (10K, 100K, 1M), record separate all-numeric,
formula-heavy and mixed text/formatting populations. Report how many graph nodes,
anchors, scalar edges, selections and logical rectangle positions each workload
actually creates. A million formatted empty positions is not a million-node graph.

Use clustered rectangles, widely scattered isolated entries, many sheets sharing a
graph, and heavily fragmented axes after alternating inserts/deletes. Include long
formula/text payloads, repeated styles, long dependency chains, high fan-out,
overlapping wide ranges, mostly-empty ranges and genuinely dense range dependencies.
Measure cold and warm/cache-reuse paths separately.

| Operation | Measurements and concern |
| --- | --- |
| Build/import | Time, peak/live bytes and allocation counts; separate worksheet construction from actual graph admission |
| Single-cell edit | Stage/commit latency and bytes copied; direct/range invalidation fan-out; unchanged data sharing |
| Multi-cell paste | Vary 100/10K/100K entries; batch chunk writes, graph edits, peak candidate memory and atomic commit time |
| Row/column insert/delete | Axis/tree work, affected anchors/selections and index fragmentation; detect accidental full-coordinate rewriting |
| Invalidation | Traversed scalar/range dependencies, dirty count and unchanged result reuse; distinguish structural from value-only edits |
| Range calculation | Populated versus geometric area, membership query cost, reduction cost, cycle-analysis work and allocations |
| Viewport read | Examples 50x20 and 100x50 plus bounded prefetch; payload size, p50/p95 latency, sparse coverage and paging progress |
| Snapshot capture | Root pin/capture latency, dependency rebuild time, incremental-index reuse and peak memory with old captures retained |
| Reclamation | Memory after edit churn, obsolete calculations, retired identities, canceled prefetch and released roots |

Record p50/p95 and worst cases with machine/OS/build/profile, seeds, budgets and
raw artifacts. Track allocator calls and estimated retained structure bytes alongside
RSS; RSS alone cannot explain sharing or delayed reclamation. Separate encode/file
cost from workbook operations. Do not select a production container from these tests.

## Complexity hypotheses to falsify

- Localized worksheet edits should copy changed tree paths/chunks, not every cell.
- Axis insertion should update order runs without rewriting all later coordinate
  keys, but affected ranges/references still have real work that must be counted.
- Rectangular access should visit intersecting runs/chunks/records, with fragmentation
  cost visible rather than hidden behind an asserted O(1) lookup.
- Range representation/transport should be proportional to descriptors and returned
  populated data rather than blank area. Evaluation/cycle checks can still visit many
  real producers and must respect admission budgets.
- Snapshot/result reuse must preserve immutable semantics; copying less is not evidence
  of correct invalidation or deterministic arithmetic.

Agree product latency/memory budgets before turning measurements into release gates.
Do not invent capacity or responsiveness guarantees from the current small tests.
Report any node/byte/work cap that prevented a workload instead of silently downsizing it.

## Existing implementation changes needed for representative scale

The current model/document path defaults to 1,024 live nodes and small graph/transfer
budgets. WorkingModel::stage clones node/name/interface structures; portable graph
editing/capture also copies owned records; snapshots rebuild dependency/order data;
evaluation is calculation-local and nonincremental. These must be measured and,
where necessary, replaced by reviewed persistent/chunked definitions, coherent
dependency indexes, bounded captures and result reuse. Coordinated limit profiles
are necessary; simply raising a CBOR or IPC limit does not fix them.

Current expression/typed-value boundaries do not support workbook references,
functions, ranges, general Blank/Text or partial structural-error evaluation.
Their implementation and oracle tests precede any benchmark claiming those features.
Container-only or sparse-map-only synthetic tests must be labeled as such, not
reported as end-to-end spreadsheet performance.

## Proposed implementation sequence after review

1. Implemented Slice A: small in-memory sparse worksheet/graph projection, typed
   decimal formulas, anchors, clear/move/retire, atomic batches and snapshot tests.
   Current graph semantics and small-model/full-copy limits remain explicit.
2. Review shared Rust reference parsing/resolver changes and implement the approved
   A1/copy/structural subset with explicit expected-state checks. No frontend evaluator.
3. Validate an executable C# client against real application/session operations,
   then bounded bulk rectangular reads and later Avalonia virtualization.
4. Add first-class selections, typed value/error support and range-aware dependency
   validation in bounded reviewed slices; then incremental calculation and scale gates.
   Parallel work is possible, but no range syntax should ship without matching semantics.

Slice A is implemented and stops for review; this sequence is not automatic
permission for additional work. Preserve existing core, document, file, protocol,
decimal and IPC tests throughout. Native macOS validation remains a separate backlog
item. Only the bounded typed proof has workbook evidence; future parser/range/scale
behavior still requires implementation and its own verification.
