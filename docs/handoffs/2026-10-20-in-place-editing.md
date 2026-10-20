# Worksheet in-place editing

Date: 2026-10-20. User-authorized focused usability slice; ready for progress/implementation
review with clipboard key presses manually accepted by subsequent user confirmation.
[Parent review packet and deferred paste issue](2026-10-20-in-place-editing-review-packet.md).
Accepted by the parent review received 2026-10-20.
[Acceptance and bounded paste follow-up](2026-10-20-in-place-editing-review.md). The preceding range/SUM Slice C
is accepted and closed. No new engine, protocol or WB2 behavior is introduced.

## Workflow

Select a cell and type to replace its contents. Double-click or F2 edits its original
number/formula; a formula opens its source rather than its calculated value. A single
native Avalonia TextBox overlays the active painted cell. The optional formula bar
shares the same draft, and the separate Commit button is removed.

Enter commits and moves down; Shift+Enter moves up. Tab commits and moves right;
Shift+Tab moves left. Escape cancels without mutation. Arrow keys navigate while
not editing and belong to the native text editor while editing. Delete clears the
selected cell. Clicking another cell commits first; an invalid number blocks that
selection change and keeps the draft. Navigation reveals cells within existing
1024×1024 limits. The native editor supplies caret, text selection, clipboard and
composition behavior; platform-specific manual checks remain distinct evidence.

Save/Save As, New/Open, Close workbook and window-close finish the active cell edit
before proceeding. Unsupported numeric input keeps the window/workbook and draft
available; Escape cancels it. Invalid formulas are still accepted as Rust-owned,
editable formula drafts. Dirty-close choices run after a successful cell commit.
A failed/uncertain Save preserves the open dirty workbook. Actions already in flight
are serialized; no duplicate automatic edit or ambiguous conflict replay is added.

## Draft and revision handling

One draft records the coordinate, original displayed source and exact displayed
revision. Refreshes update calculated cells without replacing that draft. Inline and
formula-bar text synchronize through synchronous Text property notifications, avoiding
deferred-event feedback. The document title/footer identifies an uncommitted edit.

A hard failure or revision conflict preserves text and prevents automatic navigation;
the user can Escape to load the refreshed cell before editing again. Successful edits
use the existing Rust session, then refresh and release the draft. Editing an unchanged
existing formula and pressing Enter does not rebind it. Explicitly typing/replacing a
formula, including identical source, is an intentional submission and retains the
existing explicit-rebind behavior after moved/reused coordinates.

Viewport scrolling/resize repositions or hides the single editor without discarding
its text. Starting an inline edit waits for an installed viewport. The painted grid
still keeps only a bounded visible-cell dictionary; there is no per-cell control tree
or C# calculation model.

## Validation

The actual-window self-check now exercises routed text/key events and native control
state: typing, synchronized drafts, refresh retention, F2/no-op versus explicit
rebind, Enter/Tab/Escape/Delete, caret-arrow isolation, keyboard edge navigation,
selection-triggered commit, invalid/stale draft retention, exact numeric admission,
SUM and an active-draft Save/reopen. It retains the preceding scalar/reference,
viewport and document lifecycle assertions and requires zero-charge normal close.
This is programmatic interaction with a real Avalonia window/Rust child, not OS
keyboard/mouse automation, an IME certification or manual dialog acceptance.

Windows and isolated Linux full CI and actual-window checks pass (290/291 default
Rust tests, .NET Release without warnings/errors, 10,099 decimal vectors without
differences, protocol fixtures, IPC and real application session). Normal window
close reports zero remaining Rust domain charge. The subsequent native APFS/macOS automated run passed (291 default / 307
all-feature Rust tests, strict lint and actual-window checks), but **manual
acceptance failed** for clipboard shortcuts. The user reports manual work completed
on all three platforms and other functionality behaved as expected. The clipboard
fix/focus-feedback follow-up below now has a fresh Mac automated pass; the user subsequently
confirmed that copy/paste key presses work and accepted those manual checks. [Mac packet](2026-10-20-macos-in-place-editing-instructions.md).

- [Windows CI](../../artifacts/inplace-ci-windows/20261020T140919702103Z/summary.json)
- [Windows window](../../artifacts/inplace-ui-windows-final/results.json)
- [Windows image](../../artifacts/inplace-ui-windows-final/results.png)
- [Windows active editor](../../artifacts/inplace-ui-windows-final/results.editing.png)
- [Linux CI](../../artifacts/inplace-ci-linux/20261020T141001662577Z/summary.json)
- [WSLg window](../../artifacts/inplace-ui-linux/results.json)
- [WSLg image](../../artifacts/inplace-ui-linux/results.png)

Earlier actual-window attempts found deferred text synchronization and unchanged
versus explicit-rebind distinctions; both were fixed and the checks passed. The final UI-only test addition also checks real hit-testing outside the overlay
and captures the active editor; final Windows/WSLg window checks run after that
addition. No new performance SLA is claimed. Rust/hosted protocol/WB2 numeric and selection semantics
and product caps remain settled.

## Manual checklist

Launch `python scripts/run_desktop.py` from the working tree. Use native storage
(NTFS on Windows, local WSL2 storage on Linux, APFS on Mac).

1. Click A1 and type 12.50. Enter should commit and select A2. Tab/Shift+Tab should
   commit and move horizontally; arrows navigate outside editing.
2. Double-click A1 or press F2. Move the caret, select a substring, copy/paste within
   the editor, change the text and Escape. The old value must remain unchanged.
   Confirm ordinary keyboard/IME composition if available; report untested IMEs.
3. Enter `=A1+1` in B1. F2 must show the formula, not the result. Edit through the
   formula bar and verify it shares the inline draft without duplicate submission.
4. Type an unrepresentable number, click another cell and press Save: retain the
   draft and selected cell, with an error. Escape should recover normal navigation.
5. Start a valid edit, scroll/resize and return; text must survive. Click another
   cell to commit. Use Delete to clear a selected cell and check dependent results.
6. Leave a valid edit unfinished, Save As, then reopen: that edit must be present.
   Repeat unfinished-edit window-close and Save/Discard/Cancel, including native
   pickers. A fresh untouched workbook still closes without prompting.

Preserve the unresolved WSL mounted-Windows-path installation uncertainty,
historical WSLg SIGABRT and Linux viewport p95 exception. Native-path success does
not resolve them. If the abort recurs, retain a native backtrace.

Multi-cell selection, rectangular paste, copy/fill, formula reference insertion by
clicking, text values and additional functions remain separate follow-up work.

## Source provenance

The final [168 build/test/resource inputs](../../artifacts/inplace-build-inputs.json)
match the isolated Linux copy. Relative to the accepted Mac Slice C build-input
manifest, only WorksheetView.cs, MainWindow.axaml and MainWindow.axaml.cs changed.
The last file also contains the expanded window self-check. Full CI preceded the
final hit-test/image-only addition; both final window runs rebuild the desktop and
execute it. Rust, private/frozen schemas and native workbook codecs are unchanged.

## Clipboard and focus follow-up — 2026-10-20

The [Mac report](../../artifacts/macos-validation-20261020T142038Z/REPORT.md)
records automated success and a manual failure: Cmd+C/Cmd+V could insert c/v into a
cell. The user also reported missing clipboard functionality across the application.
Do not turn this report into an overall manual pass. All 320 source hashes matched
at that run, all 168 build inputs matched, and the transfer record verifies 365
returned files. One setup-only launch failure was retained; no validation gate
reruns or product fixes were made on Mac. These new changes differ in the same
three UI files and are not covered by that earlier Mac run.

The follow-up adds explicit Copy/Paste buttons and platform-primary keyboard
commands: Cmd+C/V on macOS, Ctrl+C/V elsewhere. Ctrl/Cmd+A selects all text in the
active editor. Shortcut-modified text input cannot start a cell draft; AltGr text
input remains permitted. Grid copy uses original formula/number source (not the
calculated result); pasting into a cell opens an editable replacement draft. There
is no relative formula translation. Within an editor, copy uses the selected
substring and paste replaces its selection/caret range. Buttons retain the most
recent worksheet editor target when moving focus to the toolbar. An asynchronous
paste checks its original cell, revision, text and selection before installation.

Only one cell of plain text, at most 4096 characters, is admitted. A terminal line
ending is tolerated; embedded newlines/tabs are rejected with a visible message.
No rectangular paste, copy/fill or implicit numeric rounding is introduced. Native
clipboard failures are surfaced without committing any edit.

The active grid cell is blue only when the grid or inline editor actually has focus
in the active window. An inactive selection has a slate-gray outline without blue
fill. The inline editor border likewise becomes gray when it loses focus. A native
text caret appears while editing; the visible Blink caret checkbox switches between
blinking and steady carets in both editors. This is a session option, not a saved
workbook property. A selected cell outside editing has no fake insertion caret.

Real-window native-clipboard tests pass on Windows and WSLg: formula source round
trip through Rust, selected substring round trip, primary-shortcut interception,
suppression of c text input, explicit rectangular-paste rejection, inactive focus
state and the caret option. Existing editor/Save/Open assertions still pass, with
zero-charge close and no build warnings/errors. This does not independently certify
physical Cmd-key delivery on Mac or the user's clipboard manager/input methods.

- [Windows follow-up](../../artifacts/clipboard-focus-windows/results.json)
- [WSLg follow-up](../../artifacts/clipboard-focus-linux/results.json)
- [Final input manifest](../../artifacts/clipboard-focus-build-inputs.json)

Manual retest: Copy a selected cell, paste into another and press Enter; verify the
original number/formula text. In F2 and formula-bar editors, select a substring and
try Copy/Paste buttons and Ctrl/Cmd+C/V/A. Confirm no c/v/a is inserted by shortcuts.
Try empty-cell copy and rejected multi-cell paste. Move focus to another control or
window: the active-cell/editor border must become gray, returning blue on focus.
Toggle Blink caret and check blinking versus steady. Retest native Mac shortcuts
for physical-key/manual acceptance; the updated APFS automated run below is complete.

## Updated Mac results and review stop — 2026-10-20

The [fresh APFS follow-up](../../artifacts/macos-validation-20261020T170901Z/REPORT.md)
passes all automated gates, including in_place_editing and clipboard_focus, with
291 default / 307 all-feature Rust tests and zero-charge normal close. All 320
source hashes and 168 build inputs match; 366 report files are transfer-verified.
No setup failures, source fixes or validation-gate reruns were required. This review
also compared the recorded source hashes with the current tree before doc edits.
The report originally marked physical-key/manual acceptance pending. The user
subsequently confirmed the copy/paste keys work and explicitly accepted the manual
key checks on 2026-10-20. Preserve the prior failed result and do not infer
unreported IME coverage.

The user explicitly deferred the remaining paste UX issue: grid-cell paste should
commit immediately rather than open a draft requiring Enter. Current code and tests
are unchanged this round. Paste inside an active text editor remains a distinct
operation. See the review packet for the bounded next-round concern. Stop for parent
review before implementation of this issue or further scope.

## Manual key-check acceptance — 2026-10-20

The user explicitly confirmed that copy/paste key combinations work and authorized
marking manual key acceptance complete. This does not mean clipboard functionality
is fully rounded out: immediate grid-cell paste commit remains deferred to the next
round, as previously requested. No implementation or test behavior changed for this
confirmation, and no additional IME/dialog coverage is inferred.

## Parent acceptance — 2026-10-20

The parent accepts the current in-place editor, clipboard and focus milestone,
including manual key checks. Direct grid-cell paste commit is the bounded next
assignment; no implementation is included in this review-record update.
