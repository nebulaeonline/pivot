# In-place worksheet editing — acceptance and next scope

Review received: 2026-10-20. Disposition: **Accept the in-place editing,
clipboard and focus milestone**, including user-confirmed manual key checks.
[Reviewed packet](2026-10-20-in-place-editing-review-packet.md).
The earlier Mac Cmd+C/V failure remains historical evidence, not a current
unresolved shortcut defect. No universal IME/keyboard compatibility is claimed.
Engine, protocol, WB2, numeric behavior and product limits remain settled.

## Parent-recommended next assignment: direct single-cell paste commit

For a selected grid cell outside editing, Ctrl/Cmd+V or Paste should submit one
immediate authoritative Rust edit, update revision/dependents/viewport/dirty state,
and keep the coordinate selected without requiring Enter. Numeric text uses existing
exact-decimal validation; formulas use normal Rust binding and editable-draft rules.

Within an active inline editor or formula-bar edit, paste still inserts at the caret
or replaces selected text without committing. Retain one synchronized draft and
expected revision; do not add a generic UI state framework.

Unsupported contents must not partially install. Invalid numbers remain correctable;
conflicts must not overwrite or silently retry. Late clipboard reads must not target
a different cell/revision. Cancelled operations must not duplicate edits.

Verify numeric/formula paste, dependent recalculation, invalid numbers, editable
invalid formulas, empty-cell copy, rejected multiline/rectangular data, rapid
selection changes, active-editor paste and cancellation. Preserve Enter/Tab/Escape/F2,
focus feedback and zero-charge shutdown. Deliver actual-window and manual clipboard
checks on Windows, WSLg and native APFS/macOS, source provenance and a concise handoff;
then stop for review.

No multi-cell clipboard ranges, relative/absolute formula translation, copy/fill,
text-value support, graph/WB2/IPC changes or capacity increases. Preserve WSL
mounted-path uncertain installation, historical WSLg SIGABRT and viewport p95
exception separately. Future multi-cell/copy-fill versus text-value work requires
its own prioritization.

This records the received review and bounded next scope. No paste implementation
or test behavior is changed by this documentation update.
