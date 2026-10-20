# Rectangular worksheet selection — acceptance and next direction

Review received: 2026-10-20. Disposition: **Accept the implementation and close the
rectangular-selection milestone**. [Accepted handoff](2026-10-20-rectangular-selection.md).
Three-platform automated and user-confirmed manual evidence is accepted. Retain the
original pointer-coordinate failure and corrected runs, async gesture/clipboard
regressions, and platform limitations.

The anchor/active/extent model stays C# presentation state. Selection must not
create content, graph nodes or computational SelectionIds, change Rust revisions
or dirty state. Keep the bounded painted viewport and single editing control.
Active-cell-only Copy/Paste/Clear/Delete/Move is accepted for this milestone, not
established as permanent rectangular-operation behavior.

## Next priority: rectangular clipboard and formula copy semantics

Prepare a concise semantic proposal and bounded implementation plan **for review
before implementing formula translation or committing permanent behavior**.
Define:

- Source/destination rectangle correspondence and overlap handling.
- Relative/absolute scalar references and copy versus move semantics.
- Translation of SUM's fixed coordinate-area selections when copied.
- Blank source cells and destination clearing.
- Invalid formulas and out-of-bounds translated references.
- Atomic batch submission, rollback and failure behavior.
- Exact decimal, original source and provenance preservation.
- Explicit worksheet/clipboard/transaction admission ceilings.

Rust workbook mutation/binding remains authoritative. Do not implement independent
formula translation in C#. Preserve stable CellIds for moves; copying deliberately
creates appropriate identities/transformed bindings under the reviewed policy.
Do not enumerate enormous unused rectangles without admission bounds, raise product
capacity, redesign the graph, add another engine, alter WB2 unnecessarily or broaden
into formatting, structural operations or a UI framework.

After proposal approval, the bounded end-to-end clipboard workflow requires actual
application tests, atomic rollback, source-matched Windows/WSLg/native APFS testing
and user-confirmed manual acceptance. Stop for review. Retain mounted-path uncertain
installation, historical WSLg SIGABRT and Linux viewport p95 exception separately.

This update records acceptance and the next design priority. No rectangular
clipboard or formula-translation implementation has begun.
