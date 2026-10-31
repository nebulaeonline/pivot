# Visible clipboard operations

ADR-0034 is accepted and closed by parent implementation review (2026-10-31),
including its three mandatory invariants and six-target validation.
See [the ADR](../adr/0034-visible-only-operations.md).

Copy Visible Cells and Paste into Visible Rows are independent explicit commands
in Edit. Ordinary Copy/Paste and their shortcuts retain full-rectangle behavior and
hidden-row confirmation. No sticky mode, visible-only Clear/Fill, arbitrary multi-
area selection, Cut/Move or new computational range type is introduced.

Source membership is the selected rectangle's visible worksheet rows in ascending
order, not just on-screen rows. All columns and Blank slots participate, including
a visible selected header. Pending, Failed and NeedsReview refuse these commands;
no-filter explicitly means all rows. Source reading creates no Blank anchors.

Paste Visible maps N image rows to N visible destination rows of matching width.
A single visible anchor finds the next N visible rows at or below it. An explicit
multi-cell rectangle must contain exactly N visible rows and the exact width;
there is no broadcasting, truncation, tiling or hidden destination resizing. The
dialog lists actual ordered row runs, columns, addressed slots and skipped rows,
including hidden edges of a multi-cell selection. It never substitutes a bounding
box for the write set, including when expansion exits a filter's data interval.

One-use session tickets bind exact immutable operation bytes, payload kind/content,
worksheet, selection geometry, actual destination vector, expected workbook state
and visibility generation. The desktop rechecks interaction and clipboard state
after review. Reapply can invalidate a plan even if visibility bits remain equal.
Refusal/cancel/stale state never retries the edit or switches representations.

The native CB6 image begins with `PIVOTCB6`, the first source row/column, row count
and width, followed by strictly increasing actual source rows and dense row-major
typed slots/resolved styles. The first row must match the first row-list entry;
all coordinates, lengths, types, references and trailing bytes are validated. CB5
remains current for ordinary Copy. Receivers prefer the v6 OS format, enforce its
body magic, and reject preferred native failure without falling through to v5/text.
Acquisition also rejects a native-kind change during bounded read-only retries.

Strings retain exact bytes and leading zeros, empty String differs from Blank, and
numbers retain exact submitted coefficient/scale/sign. CB6 omits TSV when stored
Strings or rendered formulas contain tab/CR/LF rather than changing cell boundaries
or flattening content. Native-only clipboard data remains fully pasteable in Pivot.
Representable TSV uses the existing String apostrophe prefix. Genuine external TSV
formulas keep literal addresses and destination formatting.

Native formulas translate from each actual source position to its actual destination
using existing relative/absolute-axis rules and captured effective bindings. Packing
does not rewire cross-record references: B5 `=A2` to D12 becomes `=C9`, regardless
of where another copied row lands. Ranges remain ordinary rectangles. Current
rejected-draft, fault-provenance and unsupported structured-output copy rules apply.

One cancel-aware canonical transaction stages every mapped content/style write.
Existing destinations retain identities. Hidden cells receive no direct writes;
ordinary new formulas can still demand hidden Blank reference anchors. Filter
refresh follows the committed transaction and cannot alter its write vector halfway.
The single active anchor remains when visible; normal filtering relocates it if
the edit hides it. Transient outlines show actual addressed cells without creating
a bounding-box selection or changing subsequent ordinary command behavior.

The addressed area is at most 256 slots, even when its bounding rows span the full
worksheet. Combined bytes remain 256 KiB, source/text work 128 KiB, token work
16,384, reference work 8,192 and operation transient admission 8 MiB. Row-map traversal,
response/ticket construction, existing graph/style quotas and cancellation remain
bounded; no worksheet/transport/runtime threshold is raised. WB12 and hosted v1 are
unchanged; private session v12 carries the new operations and row-plan metadata.

Sparse styles are staged by property as a batch rather than rebuilding every map
for every slot. Splitting/coalescing preserves exact assignments outside the point
set and checks existing region, byte and work admission before candidate publication.
Shared-map bookkeeping now subtracts the underlying old map's vector storage,
instead of counting a shared handle's zero heap footprint and accumulating historical
byte totals. Trusted duplication seams retain last-write-wins for repeated style
targets; visible row plans remain strictly unique. Both corrections are covered by
focused regressions alongside the existing clipboard, Fill and formatting tests.

Independent row-mask tests, IPC races/atomicity and in-memory dialog probes are
supplementary to successful six-target headless and attended managed/AOT GUI checks.
See the [implementation handoff](../handoffs/2026-10-31-visible-only-operations.md)
for selected receipts, source qualifications, resource observations and code size.
