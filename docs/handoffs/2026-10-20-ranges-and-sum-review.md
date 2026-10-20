# ADR-0016 review: ranges and SUM implementation authorized

Review received: 2026-10-20. Disposition: **Accept ADR-0016** and authorize
**First-Class Ranges and SUM — Spreadsheet Semantics Slice C implementation**.
Slice B/ADR-0015 acceptance is confirmed. No further design round is requested.
Implementation has not begun; this record does not claim feature completion.

## Accepted decisions

- Inclusive coordinate-area ranges in stable worksheet axis namespaces, using current
  occupants in the captured generation. Cell movement does not move bounds; formula
  movement preserves the bound rectangle. Scalar references still follow identities.
- Blank/never-populated coordinates contribute zero without materialization. Numeric
  and computed members contribute values; invalid members propagate errors.
- Existing checked Decimal96 addition from zero, in row-major order. Intermediate
  overflow remains an error even when later terms would mathematically cancel it.
- One rectangle per SUM, reversed-endpoint normalization, scalar arithmetic around
  SUM, original source/span/provenance retention and editable invalid drafts.
- First-class selection vertices, shared sparse placements, coverage and consumer
  indexes in the existing canonical graph. No per-blank nodes or durable expanded
  membership list for every selection, live workbook callback or second evaluator.
- Deliberate `PIVOTWB2` format cut. The new reader rejects WB1 with a clear
  unsupported-version error; preserve a WB1 rejection fixture, not a migration reader.

[ADR-0016](../adr/0016-first-class-ranges-and-sum.md) governs these same-sheet range
policies while ADR-0010's earlier history remains intact.

## Hard implementation requirements

Atomic installation/capture covers graph definitions/topology, sparse placements,
selection descriptors, reverse/coverage/consumer indexes, worksheet metadata and
logical revisions. Deliberately late preparation failure, admission failure and
cancellation tests must prove no partial installation or leaked retained roots.
Immutable S retains both membership and values after T changes.

Canonical dependency traversal, selected closure, global cycle validation,
invalidation and inspection must understand scalar and selection edges. Cycles
must be detected by canonical dependency analysis, not only recursive execution.
Test both `B2=SUM(B2:B100)` and the mixed cycle `B2=C2+1`,
`C2=SUM(B2:B100)`. Inspection must report actual edge types.

An outside input affecting a computed member must invalidate its aggregate: with
A1=2, B2=`=A1*3`, D1=`=SUM(B2:B100)`, editing A1 to 4 must produce D1=12.
Also cover overlap, move-in/out, endpoint/formula movement, retirement/reuse,
invalid drafts, deterministic overflow, sequential batches and immutable captures.

Document concrete admission ceilings for selection identities/descriptors,
consumers, index updates, temporary closure work, sparse member enumeration and
reduction state. Determine and validate these during implementation; leaving them
unspecified is not acceptable at implementation review. Ordinary workbook node/cell
limits stay unchanged. Admission failures must preserve the workbook and release roots.

Measure visited index nodes, candidate selections, emitted members and allocation
traffic alongside binding/edit/invalidation/evaluation latency, retained S/T and
cleanup. Include a sparse 1024×1024 geometry test without per-coordinate allocation;
a full-sheet aggregate residing in that sheet is necessarily a self-cycle, as noted
in the design packet. Do not hide that cycle or raise dimensions/populated-cell caps.
Do not claim constant-time invalidation or output-optimal traversal without evidence.

Selection-aware portable export must faithfully encode capabilities or explicitly
reject them. Frozen hosted storage/protocol v1 remains scalar-only and unchanged:
no omitted selection tokens, flattened dependencies or partial success. Add both
selection-aware round trips and scalar-only-boundary rejection tests. Native WB2
keeps bounded/corruption validation, exact saved-generation/dirty behavior and
staged installation. Restoring bindings must not re-resolve historical source.

## Application, reliability and final evidence

Use the existing Rust/C# session and Avalonia entry; C# does not calculate SUM.
Exercise actual entry/edit, file operations and independent-process restoration.
Before implementation handoff, manually exercise native Open/Save As and all
Save/Discard/Cancel dirty-close choices on the primary desktop. Automated document
method calls do not satisfy that manual requirement. Record the actual platform,
actions and outcomes; do not claim unperformed interaction.

Retain the unresolved WSLg native abort and mounted-Windows-filesystem replacement
failure. If the abort recurs, obtain an actionable native backtrace. Native
NTFS/ext4/APFS passes do not establish mounted-Windows-path replacement.

Provide full Windows, isolated Linux and native macOS verification: canonical
selection/cycle tests, independent decimal oracles, sparse large geometry,
admission/cancellation/rollback, S/T, WB2 corruption and cross-process round trips,
real C#–Rust/Avalonia workflows, performance/allocation/cleanup evidence and manual
dialog checks. Deliver implementation notes, exact quotas, complexity limitations,
performance results and the standard Markdown handoff. Stop for review after Slice C.

No broad functions, cross-sheet references, copy/fill, structural row/column edits,
new numeric engine, SQLite, alternate graph, advanced formatting, installer/AOT
or production-million-cell capacity increase is authorized.

[Submitted design packet](2026-10-20-ranges-and-sum-design.md),
[accepted persistence review](2026-10-20-workbook-persistence-review.md).
