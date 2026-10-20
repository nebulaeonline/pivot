# Workbook persistence Slice B: parent review decision

Review received: 2026-10-20. Disposition: **Accept Slice B and ADR-0015**.

The parent accepts coherent native workbook Save/Open, stable graph/worksheet
identity restoration, exact numeric input, drafts, saved-generation isolation,
staged replacement and cross-process/application validation. The storage boundary
and canonical engine are settled; no SQLite or alternate graph is authorized.

Retain the unresolved WSLg native startup abort, the mounted-Windows-filesystem
replacement failure and untested OS file-picker/dirty-close dialog interactions.
The child serializes requests: slow disk operations can delay workbook requests
while the UI continues rendering. Process-interruption tests do not establish
power-loss durability. Small-workbook timings do not establish 100K performance.

## Next assignment: first-class ranges and SUM — Spreadsheet Semantics Slice C

Begin with an explicit semantic decision, as the next ADR or a reviewed ADR-0010
extension, before committing to permanent representation. Scalar-reference policy
does not implicitly determine range affinity. Implementation has not begun.

Specify numeric inclusion, never-populated cells, demand-created blanks, cleared
cells, moves across boundaries, retired identities and coordinate reuse, endpoint
movement, self-reference/cycles, original source/spans, malformed/out-of-bounds
syntax, decimal overflow and deterministic aggregation order. Choose explicit
Pivot behavior rather than assuming Excel compatibility.

The implementation contract is:

- First-class same-sheet rectangular selections with sparse membership and explicit
  stable reference policy. Do not expand blank coordinates into graph edges or
  anchors. Track affected aggregates efficiently and coordinate worksheet/graph
  edits atomically, with bounded admission, cancellation and immutable S/T captures.
- Extend the canonical Rust parser/binder/executor only for range notation, SUM and
  scalar arithmetic combining SUM results, such as `=SUM(B2:B100)+1`. Preserve exact
  decimals, original source/provenance and editable drafts. No live worksheet
  callbacks in the engine, historical A1 reparsing for evaluation, second evaluator
  or C# computation.
- Test value edits, member insertion/clearing, boundary-crossing moves, retirement
  and coordinate reuse, computed members and aggregate cycles. Preserve selected
  local-closure calculation and explicitly available global validation. S must keep
  its original result after T advances.
- Extend native documents to preserve bound selections, semantics and provenance.
  Make a deliberate format-version decision; there are no deployed workbook files
  requiring speculative migration machinery. Retain bounded corruption rejection,
  safe installation and exact saved-generation/dirty behavior. Reopen must restore
  bound selections rather than bind original formula text again.
- Expose SUM through the existing session and Avalonia editor, with Rust results
  and meaningful errors. Preserve editing, scrolling, file actions, revision guards
  and dirty tracking.
- Compare deterministic results with independent arithmetic oracles. Test large
  mostly-empty rectangles and verify graph/dependency/memory growth is sparse.
  Measure binding, member edits/addition/removal, invalidation, selected evaluation,
  S/T retention, Save/Open and complete C# request latency. Guard research workloads;
  ordinary product limits stay unchanged.
- Before handoff, manually exercise native Open/Save As and Save/Discard/Cancel
  dirty-close paths on at least the primary desktop platform. This is a required
  follow-up, not coverage supplied by the existing automated window checks. If the
  WSLg abort recurs, capture an actionable native backtrace. Native NTFS/ext4/APFS
  passes do not resolve mounted-Windows-path replacement.

Deliver the semantic decision, canonical selection representation, parser/binder
and dependency/cycle tests, aggregate evaluation, document round trips, client/UI
integration, cross-platform validation, performance/memory evidence and Markdown
handoff. Stop for review.

Success: create a sparse worksheet, enter `=SUM(B2:B100)`, modify included data,
save, reopen and continue calculating correctly without hundreds of unnecessary
scalar dependencies.

No new engine, hosted database, broad function library, cross-sheet formulas,
copy/fill, row/column structural transforms, advanced formatting, installer/AOT
or production-million-cell claim is included.

[Accepted implementation handoff](2026-10-20-workbook-persistence-slice-b.md),
[ADR-0015](../adr/0015-native-workbook-documents.md),
[measurements](../performance/workbook-persistence-slice-b.md).
