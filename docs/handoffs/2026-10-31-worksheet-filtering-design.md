# ADR-0033 — worksheet filtering design review

2026-10-31. Design approved with clarifications by parent review. No implementation.

The parent accepts the design, WB12/private v11, unchanged CB5/hosted v1, definition
admission and future six-target validation gates. Three mandatory clarifications:

1. Structural NeedsReview is persisted transactionally; automatic refresh failure
   is transient presentation status and does not independently mutate/dirty the
   workbook or rewrite its definition. Reapply retries without implicit rebinding.
2. Hidden-row confirmation protects the exact complete operation footprint: Paste
   destination from actual payload, Move source and destination, and all other
   addressed extents. Relevant payload/address/count/revision/generation changes
   invalidate confirmation. No reusable consent or automatic replay.
3. Pending and failed refresh are visibly unfiltered/suspended. Actions requiring
   qualified hidden-row counts remain blocked in both states. Map, frame, actual
   addresses and counts publish coherently against the same qualifying generations.

These are acceptance invariants for future implementation, not measured results.

The owner committed ADR-0032 (`fbe50a1`) and authorized filtering design. The
[full proposal](../adr/0033-worksheet-filtering.md) recommends one document-owned
filter definition per ordinary worksheet, with a revision-qualified derived visible
row map. Cells retain their coordinates, identities, bindings, values and formatting.
Calculations and rectangular range membership continue to include hidden cells.

The accepted design decisions:

- Filtering hides whole worksheet rows within an explicitly reviewed data interval.
  The dialog names the range and warns that cells in other columns on those rows
  also disappear. Header exclusion starts visibly on; no automatic table guessing.
- Persist range/header/typed criteria and a structural NeedsReview flag, not a cached
  visibility bitmap. Propose WB12 and private v11; CB5/hosted v1 remain unchanged.
  No obsolete-version reader or migration is assigned.
- Column conditions combine with AND; each permits two typed predicates with AND/OR.
  Decimal, ordinal case-sensitive String, Boolean, Blank and completed errors have
  explicit rules. Text literals default to String and preserve leading zeros.
  Typed negatives exclude other kinds; empty String differs from Blank. Operational
  calculation failures abort candidate evaluation, rather than becoming row values.
- Automatic refresh uses one immutable capture after committed computational edits,
  including cross-sheet edits and sorting. While pending or failed, show all rows
  with a visible status. Publish matching frame/map/counts atomically. A failed edit
  refresh does not undo committed content; failed replacement criteria preserve the
  previous definition/map. Reapply/Clear remain explicit recovery controls.
- Keep actual row labels and coordinate-based selection. All navigation, hit-testing,
  scrolling, painting and editor placement share one map. A mapped viewport response
  returns actual row addresses from one capture without reading distant bounding gaps.
  Active cells hidden by refresh move to the next/previous visible worksheet row.
- Initial clipboard, Fill, formatting, CSV export and Sort retain their existing
  full-rectangle semantics. When hidden rows are addressed, require an explicit
  action naming their count before dispatch; default Cancel, stale guards and no
  replay. No packed visible-only clipboard or multi-area edit is smuggled into this
  milestone. The parent accepts this explicit initial interaction boundary.
- Row/column structure changes on the filtered sheet reveal all rows and atomically
  persist NeedsReview. Retain prior criteria/range as a disabled draft; require
  explicit review before reactivation. This avoids inventing filter affinity or
  treating coordinate-derived axis IDs as durable column identities. Sorting keeps
  the rectangle and criteria and recomputes visibility over all resulting occupants.
- Bound definitions to one per sheet, two predicates per column and a proposed
  1 MiB aggregate metadata ceiling within existing workbook/file/memory admission.
  Use a row bitset plus bounded visible-row vector and sparse evaluated keys, not
  height-by-key or virtual-area materialization. Preserve the 33,554,432-unit work
  ceiling and existing capacity/style/evaluation limits.

The ADR compares transient filters and persisted masks with persisted definitions,
defines dirty-state/no-op/capture semantics, and proposes independent identity and
row-map oracles, resource/cancellation/adversarial cases, Save/reopen tests and first-
milestone six-target headless/managed/AOT validation. Timing and memory gates are
future requirements, not filtering measurements. Preserve the accepted 256/512 MiB
compact-corpus objectives and measure full evaluation plus visibility refresh.

The owner reports updated Windows build tools on both Windows x64 and ARM64 VMs.
No exact version is inferred; future validation must record the discovered toolsets.
No remote commands, builds, GUI runs, source-code changes or feature tests were
performed for this design. ADR-0032 stays closed; ADR-0023 remains parked at G3.

Parent design review is complete; its disposition and clarifications are recorded
in ADR-0033, STATUS and the ADR index. Stop for separate owner implementation
authorization under the standing milestone cadence. No filtering implementation,
format/protocol changes, builds or platform runs follow from design acceptance.
