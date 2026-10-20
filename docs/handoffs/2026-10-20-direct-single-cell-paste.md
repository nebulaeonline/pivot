# Direct single-cell paste commit

Date: 2026-10-20. User authorized implementation after the parent accepted the
[in-place editing milestone](2026-10-20-in-place-editing-review.md). This focused
correction is ready for parent review: three-platform automated validation and
user-confirmed manual checks pass. Parent implementation acceptance remains pending. No additional spreadsheet feature is started.

## Result

With a grid cell selected outside editing, Paste or Ctrl/Cmd+V now submits one
immediate Rust edit. A successful paste refreshes results/dirty state, retains the
selected coordinate and leaves the inline editor closed. No Enter is required.
Numbers use the existing exact-decimal validation; formulas use normal Rust parsing
and binding. An invalid formula becomes an ordinary editable Rust formula draft.
Empty text clears a cell. Cell copy retains original number/formula source; copying
a blank clears stale clipboard text. No relative formula translation is introduced.

Paste within the active inline editor or formula bar still inserts at the caret or
replaces selected text, synchronizes the shared draft and does not commit. Existing
Enter/Tab/Escape/F2, focus outlines and caret configuration remain intact.

## Failures and late responses

The immediate path prepares the same revision-bound edit as normal entry, but keeps
the editor hidden during a successful submission. Invalid numeric text or a rejected
submission exposes the retained draft for correction or Escape; no silent rounding,
retry or partial installation occurs. A stale displayed revision is rejected by Rust
rather than replaced with the session's newer revision.

Each asynchronous paste records a request generation, interaction generation,
selected coordinate, revision and any editor text/selection. Selection changes,
viewport changes, draft activity, document actions and cursor/selection changes
invalidate the old interaction, including an away-and-back selection. A newer paste
supersedes an older read. Escape cancels a pending read. Cancelled reads install
nothing; once an edit is dispatched, the existing session drains its response rather
than ambiguously retrying or pretending to undo it.

Plain text is limited to one cell and 4096 characters. A trailing line ending is
accepted; embedded newline/tab/rectangular data is rejected without mutation. Rust
remains authoritative for accepted content and calculation. No engine, graph,
application protocol, WB2, numeric-policy or product-capacity changes are included.

## Verification

The real-window self-check rebuilds Avalonia and uses the owned Rust child. Native
clipboard checks cover source/substrings and the primary keyboard command. Controlled
delayed reads use the same production asynchronous paste path to check away-and-back
selection, Escape cancellation, cancelled tasks, superseded reads, revision changes
and stale displayed-revision conflicts. Assertions cover immediate numeric/formula/
empty paste, dependents, dirty state, unchanged selection, closed editor on success,
invalid-number correction, editable invalid formulas and active-editor/formula-bar
insertion without commit. Earlier editing, Save/Open and zero-charge cleanup checks
remain enabled. This is not physical-key/IME/manual-dialog certification.

Windows/Linux full CI (290/291 default Rust tests), .NET Release/parity/IPC/session
checks and final actual-window clipboard checks pass with zero-charge normal close. The fresh native Mac APFS run passes full CI, strict all-feature lint, 291 default /
307 all-feature Rust tests, .NET Release/parity/IPC/session and actual-window paste
checks. The user confirms manual functionality across all three platforms works
as intended. These are new direct-paste results, not substituted editor evidence.

- [Windows CI](../../artifacts/direct-paste-ci-windows/20261020T172857717082Z/summary.json)
- [Windows window](../../artifacts/direct-paste-windows/results.json)
- [Linux CI](../../artifacts/direct-paste-ci-linux/20261020T172926318545Z/summary.json)
- [WSLg window](../../artifacts/direct-paste-linux/results.json)
- [Native Mac report](../../artifacts/macos-validation-20261020T173814Z/REPORT.md)
- [Mac CI](../../artifacts/macos-validation-20261020T173814Z/ci/20261020T173824917832Z/summary.json)
- [Mac window](../../artifacts/macos-validation-20261020T173814Z/ui/results.json)
- [Final build inputs](../../artifacts/direct-paste-build-inputs.json)
- [Native Mac packet](2026-10-20-macos-direct-paste-instructions.md)

Only MainWindow.axaml.cs changes relative to the accepted clipboard/focus source;
that file also contains the expanded actual-window assertions. Retain all original
logs and failures; no performance SLA is claimed.

## Manual checklist

Run `python scripts/run_desktop.py`. Use native local storage for file checks.

1. Enter 12.50 in A1 and `=B2+1` in C2. Copy A1, select B2 and Paste via button or
   Ctrl+V (Cmd+V on Mac). B2 must immediately show 12.50 and C2 13.50, with B2 still
   selected and no edit box/Enter required. Confirm dirty state.
2. Copy a formula to another selected cell and paste. It should calculate immediately
   using its original text through normal Rust binding, without relative translation.
3. In F2 editing, select part of a number/formula and paste text; do the same in the
   formula bar. Text changes, but the model must wait for an ordinary commit. Escape
   must cancel, and Enter/Tab must keep the accepted navigation behavior.
4. Paste an unrepresentable decimal: retain the prior workbook value and a correctable
   edit. Paste `=(`: accept an editable formula error. Copy/paste an empty cell: clear
   the target. Multi-cell/tab/newline content must be rejected without partial edits.
5. Confirm no c/v text appears from shortcuts, and retain focus/caret behavior. Save
   and reopen to verify committed results. Record any clipboard-manager/IME coverage
   explicitly; do not infer untested combinations.

The WSL mounted-Windows-path uncertain install, historical WSLg SIGABRT and Linux
viewport p95 exception remain separate limitations. Native-path passes do not
resolve them. No rectangular selection/paste, copy/fill, text-value support or
formula-translation framework is authorized. Stop for review after this correction.

## Final verification notes

An additional hidden-editor probe initially assumed hiding a native TextBox preserves
its selected substring. Native focus changes had collapsed that selection before
paste began. The test now compares insertion against the actual paste-time caret/
selection, while still asserting no model revision change and a retained draft.
The failure is retained in
[log](../../artifacts/direct-paste-hidden-editor-failure.log) and
[result](../../artifacts/direct-paste-hidden-editor-failure.json). The handler also
retains active-draft context if native focus moves while the editor is offscreen.
Both final window checks rebuild and execute that code. Full CI preceded this last
context/test tightening; no Rust or boundary changes occurred. All 168 final build
inputs match the isolated Linux copy. Fresh Mac automation and user-confirmed
three-platform manual checks now pass, as recorded below.

## Native Mac review and manual completion — 2026-10-20

The fresh run used internal APFS for source, Cargo/.NET outputs, temporary files,
file tests and original reports, then returned reports to SMB. All 324 copied source
hashes and 168 reference build inputs matched; this review independently compared
all recorded source hashes with the current tree before documentation edits, without
differences. No product fixes, setup failures or validation-gate reruns were needed.
The transfer record verifies 367 returned reports with no missing report links.

[Source verification](../../artifacts/macos-validation-20261020T173814Z/source-final-verification.json),
[build inputs](../../artifacts/macos-validation-20261020T173814Z/reference-input-final-verification.json),
[transfer record](../../artifacts/macos-validation-20261020T173814Z/transfer-verification.json).

The Mac window reports in_place_editing, clipboard_focus and direct_paste passed,
including immediate numeric/formula/empty paste, dependent recalculation, numeric
correction, editable invalid formulas, editor insertion and delayed-read/conflict
checks. Normal close has zero remaining Rust charge. The rendered final image was
inspected here: controls, grid and footer are readable, with no observed clipping.

The report's final user-reported manual acceptance supersedes its earlier pending
paragraphs. The user additionally confirmed in this session that manual functionality
on Windows, WSLg and macOS works as intended. Treat those as manual evidence,
separate from routed-input automation. Specific IME/clipboard-manager combinations
were not enumerated; no additional coverage is inferred.
[Mac manual evidence](../../artifacts/macos-validation-20261020T173814Z/manual-acceptance.json).

The delivered correction is direct **single-cell** paste, not a completed clipboard
feature set. Rectangular paste, relative/absolute reference translation, copy/fill,
multiple selection and text-value support remain future scope. Do not broaden this
acceptance request or infer production capacity from sparse geometry. Historical
clipboard failures, the hidden-editor probe and filesystem/platform limitations
remain retained. Submit for parent review and stop before subsequent feature work.
