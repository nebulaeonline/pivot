# Rectangular worksheet selection and navigation

Date: 2026-10-20. User authorized implementation after the parent accepted direct
single-cell paste. Three-platform automated/manual validation is complete and the
implementation is ready for parent review; parent acceptance remains pending.
[Reviewed scope](2026-10-20-direct-paste-review.md).

## Interaction specification

Selection consists of one fixed **anchor** and one **active cell**, both bounded to
rows/columns 1–1024. The rectangular extent is their inclusive row/column minima
and maxima. The active cell is the moving endpoint, not an extra cell outside the
rectangle. Two coordinates suffice; selection does not enumerate its contents.

| Input | Effect |
| --- | --- |
| Plain cell click | Set anchor and active cell to the clicked coordinate |
| Left-button drag | Keep press anchor; move active endpoint with pointer |
| Shift+click | Retain existing anchor; move active endpoint to clicked coordinate |
| Shift+arrow | Move active endpoint one bounded cell, retaining anchor |
| Plain arrow | Move active endpoint one cell and collapse rectangle there |
| Home / End | Collapse to column 1 / 1024 on the active row |
| Ctrl or Cmd + Home / End | Collapse to A1 / AMJ1024 |
| Page Up / Down | Move by current visible-row count, preserving column |
| Shift with Home/End/Page | Extend from anchor instead of collapsing |
| Tab / Shift+Tab | Existing one-cell right/left navigation; collapse region |
| Escape during drag/pending pointer selection | Restore pre-gesture anchor/active; stop capture/timer |
| Escape outside editing/drag | Collapse to the active cell; cancel pending clipboard read |
| Capture loss or window deactivation | Stop drag/auto-scroll; keep last installed selection |

Home/End are explicit worksheet-boundary operations, not data-dependent jumps.
Auto-scroll runs only during a captured left-button extension, in bounded one-row/
one-column steps every 75 ms beyond the data edges. Pointer release, Escape,
capture loss, detach, keyboard navigation or editing stops it. Release during an
unfinished cell commit still permits the pending click to select its target after
the commit; cancellation invalidates it. An accepted Rust edit is not undone by
cancelling its subsequent selection gesture.

The active cell has a stronger outline and white interior, while other selected
cells have pale blue fill and a region boundary. Inactive selection uses muted gray.
The selection label identifies the normalized rectangle and active coordinate.
Scrolling, resize, viewport refresh and focus changes preserve absolute coordinates.
Keyboard extension reveals its endpoint; dragging beyond the viewport auto-scrolls.
New/Open resets the UI selection to A1.

## Editing, clipboard and model boundaries

Typing, F2, double-click or starting a formula-bar draft collapses the region to its
active cell. Editing continues through the existing single synchronized draft and
exact displayed revision. Enter/Tab/Escape, focus/caret feedback and Save/close remain
as accepted. A pending pointer request checks that its gesture is still current
after asynchronous edit completion and viewport refresh, so cancelled gestures do
not install stale selections or steal focus.

Copy, Paste, Delete/Clear and Move still target **only the active cell**. The visible
selection label states this; no rectangular mutation is implied. Direct single-cell
paste keeps its immediate Rust commit behavior. Selecting/altering a rectangle
invalidates late clipboard context even if the active coordinate stays the same.

The selection is C# presentation state, distinct from SUM's computational ranges.
It allocates no Rust SelectionId, graph node or worksheet content, changes neither
workbook revision nor dirty state, and is not saved in WB2. The grid still paints
only the bounded viewport with one inline editor, not a control per worksheet cell.
Rust, IPC, native format, numeric policies and product capacity remain unchanged.

## Verification

Actual-window assertions route pointer presses/moves/releases and keyboard input
through the real Avalonia view, exercise actual pointer capture and timer-driven
scrolling, and use the real Rust child. They cover drag B2:D5, Shift+click/arrows,
anchor/active/extent, Escape restoration, capture loss, two-axis auto-scroll,
Home/End/Page bounds, resize/refresh/focus preservation, editor collapse, late
clipboard rejection and async commit followed by normal release versus cancellation.
The pure selection sequence asserts exact unchanged Rust revision/dirty state and
zero computational selections/placements. Existing clipboard, numeric/draft,
reference, SUM, Save/Open and zero-charge shutdown checks remain enabled.

Windows/Linux full CI and actual-window checks pass (290/291 default Rust tests,
.NET Release without warnings/errors, decimal/protocol/IPC/session checks and
zero-charge Rust close). The user confirmed manual checks on Windows and WSLg passed. Fresh native Mac/APFS
automation and user-reported manual acceptance now also pass. Routed events are not OS
mouse automation or manual/IME certification.

- [Windows CI](../../artifacts/rectangular-selection-ci-windows/20261020T180036217492Z/summary.json)
- [Windows window](../../artifacts/rectangular-selection-windows/results.json)
- [Linux CI](../../artifacts/rectangular-selection-ci-linux/20261020T180057515223Z/summary.json)
- [WSLg window](../../artifacts/rectangular-selection-linux/results.json)
- [WSLg selection image](../../artifacts/rectangular-selection-linux/results.selection.png)
- [Selection image](../../artifacts/rectangular-selection-windows/results.selection.png)
- [Build inputs](../../artifacts/rectangular-selection-build-inputs.json)
- [Native Mac packet](2026-10-20-macos-rectangular-selection-instructions.md)

The initial routed-pointer test used the wrong coordinate root and timed out. Its
[original failure](../../artifacts/rectangular-selection-pointer-initial-failure.log)
is retained. Events now use actual window-root coordinates, and the complete check
passes. The selection image was inspected for active/region distinction and clipping.
No new performance SLA or broad UI/IME compatibility claim is made.

## Manual checklist

Launch `python scripts/run_desktop.py`. Selection-only operations on a fresh workbook
should leave it clean; closing should not ask to save.

1. Drag B2:D5 forward and backward; try Shift+click and Shift+arrows across the anchor.
   Check the active corner and normalized rectangle label. Plain arrows collapse
   and move, and Shift reverses/contracts/extends without losing the anchor.
2. Drag beyond every viewport edge and corners; confirm bounded auto-scroll. Release
   outside the grid and confirm it stops. Escape during drag restores the old
   selection. Switch windows while dragging; scrolling must stop.
3. Try Home/End, Ctrl/Cmd+Home/End, Page Up/Down and Shift variants. On Mac record
   any Fn/key mappings used. Scroll/resize/refocus; coordinates must remain stable
   and the active/selected/inactive visuals must remain distinguishable.
4. Select a region, then type, F2 or double-click the active cell. Edit only that
   cell, verify Enter/Tab/Escape and formula-bar synchronization. Click-release on
   another cell while committing a draft; it must select after the edit completes.
   Escape during that pending gesture must not install a stale selection.
5. Copy/Paste/Clear/Delete/Move target only the active cell. Verify direct paste,
   original formula source, no spurious shortcut letters, focus/caret controls and
   Save/Open. Do not infer multi-cell clipboard/fill behavior from the rectangle.

Keep the mounted-Windows-path WSL installation uncertainty, historical WSLg SIGABRT
and Linux viewport p95 exception as separate limitations. Native filesystem success
does not resolve them. No rectangular paste, relative/absolute formula translation,
copy/fill, drag-fill, structural edits, formatting, text values or new functions are
included. Stop for parent review before any such feature.

## Source and final evidence

All 168 final build/test/resource inputs match the isolated Linux copy. Relative to
the accepted direct-paste Mac manifest, only WorksheetView.cs, MainWindow.axaml and
MainWindow.axaml.cs changed. The desktop code also contains the expanded self-check.
Windows/WSLg selection PNGs were inspected: normalized B2:D5 region, active D5,
selection label and surrounding controls are readable without observed clipping.
Fresh native Mac automated and user-confirmed manual results now pass. No files were
staged or committed by this work.

## Manual follow-up — 2026-10-20

The user confirmed the rectangular-selection manual tests succeeded on Windows
and WSLg. Record these as user-reported manual acceptance, separate from routed
window assertions. Native macOS subsequently passed automated validation and user-confirmed manual
checks, as recorded below. No additional IME or clipboard-manager coverage is inferred.

## Native Mac results and review readiness — 2026-10-20

The fresh [APFS report](../../artifacts/macos-validation-20261020T222305Z/REPORT.md)
passes full CI, strict all-feature lint, 291 default / 307 all-feature Rust tests,
.NET Release, numeric/protocol/IPC/session and actual-window selection checks.
The window reports rectangular_selection, direct_paste, clipboard_focus and
in_place_editing passed, with zero remaining Rust charge at normal close.
[Mac CI](../../artifacts/macos-validation-20261020T222305Z/ci/20261020T222321501915Z/summary.json),
[window result](../../artifacts/macos-validation-20261020T222305Z/ui/results.json).

Source, Cargo/.NET outputs, temporary files, test files and initial reports stayed
on internal APFS before reports returned to SMB. All 327 copied source hashes and
168 reference build inputs matched; this review independently compared all recorded
source hashes with the current tree before doc updates, without differences.
No product fixes, setup failures or validation-gate reruns were needed. The transfer
record verifies 368 returned files with no missing report links.
[Source verification](../../artifacts/macos-validation-20261020T222305Z/source-final-verification.json),
[build inputs](../../artifacts/macos-validation-20261020T222305Z/reference-input-final-verification.json),
[transfer record](../../artifacts/macos-validation-20261020T222305Z/transfer-verification.json).

The Mac selection PNG was inspected here: pale B2:D5 extent, outlined active D5 and
active-cell-only action label are clear, with no observed clipping. The supplied PNG
shows active selection; it does not independently certify gray inactive appearance.
Focus/inactive behavior has automated assertions and user-reported manual acceptance.

The report's final manual acceptance supersedes its earlier pending paragraphs. The
user reports manual tests work as expected on Mac, as on Windows/Linux, while noting
rectangular cut/paste is absent. That absence is expected: this milestone delivers
UI selection/navigation, not rectangular cut/copy/paste. Clipboard operations remain
single-active-cell operations; multi-cell mutation, formula translation and copy/fill
are separate future scope, not missing acceptance requirements here.
[Mac manual evidence](../../artifacts/macos-validation-20261020T222305Z/manual-acceptance.json).
Specific Fn/key/trackpad, IME and clipboard-manager combinations were not enumerated;
no additional coverage is inferred.

Windows/Linux/native Mac automated evidence and user-confirmed manual acceptance are
now complete for the bounded scope. Preserve original failures and platform limits;
request parent implementation review and stop before rectangular clipboard or other
feature work. No product code changed during this result integration.
