# Worksheet record sorting

ADR-0032 implementation has passed six-target headless and managed/Native AOT GUI
validation and is accepted and closed by parent implementation review (2026-10-31); see the
[handoff](../handoffs/2026-10-31-worksheet-sorting.md) and
[decision record](../adr/0032-worksheet-sorting.md).

The desktop Edit menu opens a Sort dialog for an explicit rectangular range. The
dialog shows the final range and warns that columns outside it do not move. Adjacent
populated columns can be included only through an explicit expansion action. Blank
anchors and formatting alone do not suggest expansion. A single-cell selection
requires an explicit range. Review the range after editing its address.

Choose one or more distinct column keys, their priority and ascending/descending
direction. Header exclusion starts off; Move formatting with records starts on.
Successful choices remain in the current window. Sorting moves every selected
column through the same stable row permutation. It keeps the active coordinate
inside the confirmed rectangle rather than following a moved cell.

Ascending successful values are ordered Decimal96, String, Boolean. Descending
reverses that successful-value order. Strings compare ordinally by Unicode scalar
order, case-sensitively and without normalization or type conversion. Stored
leading-zero Strings remain exact. Legitimate cell errors sort after successful
values, followed by Blank, in either direction. Errors tie at that key; subsequent
keys can distinguish their records. Equal key tuples retain their original order.
Quoted empty String and Blank remain distinct values under these rules.

Keys are evaluated once from a pre-sort immutable capture. Formula keys may
recalculate to different values after publication; sorting does not repeat to chase
those changes. Operational capture/evaluation failures, stale revisions, cancellation
and exhausted admission abort the command without publishing a partial result.

CellIds, graph identities, scalar bindings, authored source and fault provenance
travel with records. Scalar references, including `$`-locked references, follow
their target identities. Rectangular range geometry and SelectionIds remain fixed;
range membership follows the new occupants. Sorting performs no Copy/Fill reference
translation or reparsing. Existing editable-cycle semantics still apply if the new
placement creates a range dependency cycle; unrelated outputs remain calculable.

Move formatting permutes explicit property assignments within the sorted data
rectangle while preserving outside assignments and header formatting. It preserves
inheritance gaps and explicit defaults, without materializing resolved styles.
Assignments constant across the sorted rows, such as column formatting, remain
constant. Unchecking the option keeps formatting at its worksheet coordinates.
Fragmentation exceeding the existing style limits refuses atomically; it does not
silently switch policies. A no-op requires identical final identity placements and
explicit formatting assignments, not merely equal displayed values.

The Rust workbook owns a private sparse candidate and publishes one revision.
Typed results remain borrowed from their charged calculation result; key positions
and row offsets avoid a height-by-width key matrix. Stable merge comparisons,
examined String bytes and permutation work share the existing 33,554,432-unit
ceiling. Workbook, style, message and identity limits are unchanged. The canonical
two-dimensional selection index retains its semantics while avoiding redundant
wrappers and single-entry reverse-index trees for ordinary unaliased placements.

Private session protocol v10 adds Sort and SortBounds requests. Hosted protocol v1,
native WB11 and clipboard CB5 remain unchanged. Save/open preserves the resulting
placements and existing bindings; no persistent sort definition is stored.
