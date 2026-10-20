# In-place worksheet editing — parent review packet

Date: 2026-10-20. Submitted packet; the parent subsequently accepted this milestone.
[Acceptance record](2026-10-20-in-place-editing-review.md).
Original request: review progress and the bounded implementation. No
subsequent feature or paste-semantics change is being started before this review.
[Implementation detail](2026-10-20-in-place-editing.md).

## Delivered behavior

One native text editor overlays the active painted grid cell. Typing replaces;
F2/double-click opens original number/formula text. Enter/Tab commits and moves,
Escape cancels, arrows navigate outside editing, and Delete clears. The optional
formula bar shares the same revision-bound draft; the separate Commit button is
gone. Refreshes preserve drafts, rejected/conflicting submissions retain text, and
Save/close account for unfinished edits through the existing Rust session.

Explicit Copy/Paste buttons and Ctrl+C/V (Cmd+C/V on Mac) support cell-source and
selected-editor-text clipboard operations. Modified text is suppressed so shortcut
letters cannot start cell entry. Focused grid/editor outlines are blue; inactive
ones are gray. A session checkbox selects a blinking or steady native editing caret.
The core, protocol, workbook format, numeric policies and capacity limits are unchanged.

## Evidence

Windows/Linux full CI passed for the editor (290/291 default Rust tests), followed
by final actual-window native-clipboard/focus checks after the follow-up. Those
checks rebuild the desktop and exercise the real Rust child, not a mock evaluator.
Normal close reports zero remaining Rust charge.

The fresh [native Mac APFS report](../../artifacts/macos-validation-20261020T170901Z/REPORT.md)
passes full CI, strict all-feature lint, 291 default / 307 all-feature Rust tests,
.NET Release without warnings/errors, decimal/protocol/IPC/application checks and
actual-window editor plus clipboard/focus assertions. Source, builds, file tests,
temporary files and original reports were local on APFS; reports returned to SMB.
All 320 copied source hashes and 168 reference build inputs matched. This review
independently compared all 320 source hashes to the current tree before documentation
updates and found no differences. No product fixes, setup failures or validation-gate
reruns were needed. The transfer record verifies 366 returned files with no missing
report links. Active-editor imagery was inspected here; controls/grid/footer remain
readable with Copy/Paste and Blink caret visible and Commit absent.

- [Mac CI](../../artifacts/macos-validation-20261020T170901Z/ci/20261020T170920968799Z/summary.json)
- [Mac window](../../artifacts/macos-validation-20261020T170901Z/ui/results.json)
- [Windows clipboard/focus](../../artifacts/clipboard-focus-windows/results.json)
- [WSLg clipboard/focus](../../artifacts/clipboard-focus-linux/results.json)

The user previously reported manual work completed on all three platforms, with
other editor functionality behaving as expected, but clipboard behavior failed.
The [earlier Mac manual failure](../../artifacts/macos-validation-20261020T142038Z/manual-gui-results.json)
(Cmd+C/V typed c/v) is preserved. The fresh Mac report originally labels physical-key/manual retesting pending. In
a subsequent direct confirmation on 2026-10-20, the user authorized marking the key
checks **manually accepted**: copy/paste shortcuts work. This is new user-reported
evidence, separate from automation, and supersedes the pending key-check status.
Clipboard functionality remains incomplete as described in the deferred issue below.
No specific IME/clipboard-manager coverage is inferred.

## Deferred next-round issue: cell paste should commit immediately

User-reported usability issue, explicitly deferred on 2026-10-20: pasting onto a
selected grid cell currently opens a replacement draft and requires Enter (or
another commit action). Ordinary spreadsheet cell paste should install the copied
number/formula directly without entering editing mode. This request does not change
paste inside an already active text editor, where inserting at the caret/replacing
selected text is appropriate.

Retain the current behavior for this review. No fix or assertion change has been
made. In the next round, define direct single-cell paste as one checked Rust edit,
retaining exact numeric validation, original formula/source policy, atomic failure,
revision conflict handling, recalculation and dirty tracking. Do not silently grow
this into rectangular paste, relative-reference copy/fill or a clipboard framework.
The user requested deferral, not acceptance of draft-on-paste as the final UX.

## Retained boundaries and limitations

Uncertain installation on WSL Windows-mounted paths, the historical WSLg native
SIGABRT and Linux viewport p95 exception remain open. Native NTFS/ext4/APFS passes
do not resolve mounted-path certainty or establish power-loss durability. Earlier
failures and successful follow-ups remain separate evidence. Broader functions,
text values, multi-cell selection, copy/fill and structural edits remain outside
this slice. Slice C/ADR-0016 stays accepted and settled.

Review should assess the current editor/clipboard/focus work with the user-confirmed
manual key-check acceptance and deferred paste UX. Stop for parent direction before
implementing that issue or selecting further scope.
