# Direct single-cell paste — acceptance and next scope

Review received: 2026-10-20. Disposition: **Accept the implementation and close the
focused paste correction**. [Accepted handoff](2026-10-20-direct-single-cell-paste.md).
The parent accepts the automated evidence and user-confirmed manual checks on
Windows, WSLg and macOS. Preserve earlier clipboard/focus and hidden-editor failure
evidence and the existing filesystem/platform limitations.

Grid paste immediately submits one authoritative Rust edit; active-editor paste
remains text insertion. Current formula paste binds the original source at its
destination, without relative-reference translation. The engine, protocol, WB2,
numeric policies and production caps stay unchanged.

## Parent-recommended next milestone: rectangular selection and navigation

Provide a concise interaction specification defining one anchor, active cell and
rectangular extent. Keep this in C# presentation state: selecting must not create
Rust graph SelectionIds, allocate worksheet content, change workbook revision or
mark the document dirty. Computational SUM selections remain a different concept.

Scope includes mouse drag, Shift+click, Shift+arrow extension, ordinary arrow
collapse/navigation, Home/End and bounded viewport navigation, extension auto-scroll,
active/selected/inactive visuals and consistent focus. Preserve selection across
scroll, resize, refresh and asynchronous requests. Editing targets one active cell;
retain F2/Enter/Tab/Escape/Delete, formula-bar, single-cell clipboard and file flows.
No per-worksheet-cell control tree is permitted.

Exercise actual-window mouse/keyboard behavior on Windows, WSLg and native macOS:
anchor/active preservation, drag/extension, auto-scroll, editor transitions, focus,
cancellation and stale async responses. Assert selection-only operations leave
Rust revisions/dirty state unchanged and normal close releases all domain charge.
Deliver the interaction specification, implementation, manual/platform evidence and
Markdown handoff, then stop for review.

Exclude multi-cell clipboard mutation, rectangular paste, formula translation,
copy/fill/drag-fill, structural edits, formatting, text values and new functions.
Do not introduce another computation-range system or backend/persistence milestone.
Keep WSL mounted-path uncertainty, historical native SIGABRT and viewport p95
exception separate from this UI work.

This documentation update records acceptance and the proposed bounded next scope;
rectangular-selection implementation has not begun.
