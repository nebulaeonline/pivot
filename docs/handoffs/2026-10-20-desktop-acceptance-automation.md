# Desktop acceptance automation and directional edit commits

Date: 2026-10-20. Owner requested replacing repetitive five-group manual checks with
automation, retaining occasional inspection. Plain arrows should commit in-place edits.

## Routine acceptance

Run from a desktop session:

```sh
python scripts/ci.py --desktop
```

This runs ordinary CI plus `tests/application-session/ui_smoke.py`, records actual
exit codes and saves JSON/screenshots under the timestamped CI directory. For just
desktop checks, run `python tests/application-session/ui_smoke.py --output-dir artifacts/desktop-check`.
Headless CI still works without `--desktop`. Run Windows and WSLg desktop suites
sequentially: their OS clipboards are connected. The tests briefly open helper
windows and use the clipboard; previous text is restored, not arbitrary non-text formats.

The five groups are automated regression gates. Routine manual replay on every OS
is no longer required. This is the owner's current acceptance workflow, not a claim
that previously pending manual checks were performed or passed. Native Mac automation
has now passed from APFS; the slice is ready for parent implementation review.

| Former manual group | Automated coverage |
| --- | --- |
| Immediate native/TSV paste and translation | Real clipboard, Rust scalar/SUM results, destination highlighting, no editor/Enter |
| Shapes, overlap and blanks | Exact reversed-anchor destination, mismatch rollback, snapshot overlap, blank overwriting occupied content |
| Move and dollar-axis copy | Move button handler, effective A5→A6 target, unchanged original source, all four axis modes |
| Cross-window/workbook copy | Separate Avalonia process retrieves the private OS format after the source Rust engine closes; destination edits prove local bindings |
| Editing, persistence and close | Existing routed editor/shortcut/save/reopen checks plus actual window-close Cancel→Save, Discard and clean-close helper processes; saved generations reopened and verified |

The process coordinator fails on missing/failed peer reports, nonzero exits, timeouts,
wrong saved content or nonzero remaining Rust charge. Each peer records GUI/Rust PIDs,
native-format receipt, cancellation outcome and cleanup. Save/Discard are checked by
reopening their files in another Rust child; no test bypasses the real window Closing
handler. Dialog buttons receive programmatic routed Click events.

Screenshots include checklist, peer windows and the existing selection/editor/clipboard
views. Inspect them whenever useful; there is no recurring manual screenshot task.
Physical OS-generated keystrokes, IMEs, accessibility input and native Open/Save As
picker interaction are not claimed by routed-event tests. Those remain targeted spot
checks when their integration changes or a platform symptom appears, not five full
groups on every iteration. File tests select known native paths deterministically.

## Directional editing

Unmodified Up/Down/Left/Right in the inline editor commits and moves one cell in that
direction. A worksheet edge clamps movement but still commits. Hard rejected edits
remain in the editor at the original cell. Unchanged F2 edits exit without resubmitting
historical formula source or changing bindings. Modified arrows retain text selection/
word navigation; formula-bar arrows retain normal caret editing. Enter/Tab/Escape and
ordinary grid selection navigation retain their existing behavior.

Automation exercises all four directions for decimal and formula entry, boundary
clamping, numeric rejection, modified/formula-bar arrows and unchanged F2 after a
target move. The former caret-only test now explicitly uses Shift+Left.

## Evidence and source

The integrated `--desktop` CI command passed on both platforms before the arrow
follow-up; final desktop runs below cover the follow-up and bounded clipboard reads.
Rust is unchanged from the preceding 309/310 default and 325/326 all-feature results.

- [Windows integrated CI](../../artifacts/desktop-automation-ci-windows/20261020T233843363986Z/summary.json)
- [Linux integrated CI](../../artifacts/desktop-automation-ci-linux/20261020T233856076649Z/summary.json)
- [Final Windows desktop](../../artifacts/desktop-acceptance-windows-final/results.json)
- [Final Linux desktop](../../artifacts/desktop-acceptance-linux/results.json)
- [Windows cross-process clipboard and Cancel/Save](../../artifacts/desktop-acceptance-windows-final/results.clipboard-save.json)
- [Linux cross-process clipboard and Cancel/Save](../../artifacts/desktop-acceptance-linux/results.clipboard-save.json)
- [173 final build/test inputs](../../artifacts/desktop-automation-build-inputs.json)

Final Windows and WSLg desktop reports and all three peer outcomes pass with zero
remaining charge. All 173 final inputs match the Windows tree and isolated Linux copy.
[Windows source verification](../../artifacts/desktop-automation-windows-input-verification.json),
[Linux source verification](../../artifacts/desktop-automation-linux-input-verification.json).
Fresh native Mac/APFS execution now passes; see the integrated results below.

Source includes the new `ClipboardAutomation.cs` partial class, existing window test
entry points and Python CI/window runners. Tests run only under explicit test environment
variables. Final source hashes and final peer outcomes are recorded with validation.

Retain initial build errors (missing extension namespace and a test helper name
collision), concurrent Windows/WSLg clipboard interference, and subsequent native
format-advertised/bytes-unavailable failures in their logs. The diagnostic probe showed
native/local bytes both available in a passing run; it does not establish a root cause.
Production reads now retry an unavailable native payload twice at 20 ms, releasing each
snapshot and checking interaction cancellation. They never downgrade to TSV or replay a
mutation. Malformed bytes still reject through Rust once, without text fallback.
New tests initially exposed state carried between automation groups; fresh-workbook/
focus reset now isolates them. Original failures remain, with later passes recorded
separately. Known WSLg native abort/mounted-path uncertainty and prior viewport timing
exceptions remain unresolved. Fresh native Mac/APFS automation now passes; parent implementation review is pending.

## Native Mac validation integrated — 2026-10-20

The fresh [native APFS report](../../artifacts/macos-validation-20261020T235001Z/REPORT.md)
passes integrated `python3 scripts/ci.py --desktop`, strict all-feature lint and
310 default / 326 all-feature Rust tests. No setup failures, product fixes or gate
reruns were needed. Source, Cargo/.NET outputs, temporary files, document fixtures
and original reports stayed under internal APFS before reports returned to SMB.
[CI summary](../../artifacts/macos-validation-20261020T235001Z/ci/20261020T235011455134Z/summary.json),
[environment](../../artifacts/macos-validation-20261020T235001Z/environment.json).

All seven window result fields pass, including clipboard_checklist and arrow_edit.
Separate-process private UTI receipt and actual Cancel→Save, Discard and clean close
pass; the coordinator reopens Save/Discard files to verify their installed generations.
The main window, all three peers and source/destination boundary children report zero
remaining Rust charge; the report records no surviving owned processes.
[Window](../../artifacts/macos-validation-20261020T235001Z/ci/20261020T235011455134Z/desktop-ui/results.json),
[clipboard peer](../../artifacts/macos-validation-20261020T235001Z/ci/20261020T235011455134Z/desktop-ui/results.clipboard-save.json),
[boundary](../../artifacts/macos-validation-20261020T235001Z/ci/20261020T235011455134Z/application-session/clipboard.json).
WB3 header/length/digest checks pass; suites cover dollar scalar/SUM reopen,
preserved rejected drafts/resubmission and explicit WB1/WB2 rejection. Sixteen
returned document fixtures include deliberately malformed/rejected cases, not an
assertion that every fixture is a valid workbook.

Before this documentation integration, all 339 copied source hashes and 173 reference
build/test hashes independently matched this tree. The final reference inputs still
match the implementation. The transfer record reports SHA-256 agreement for all
391 returned files and no missing report links; 391 files are present here.
[Source manifest](../../artifacts/macos-validation-20261020T235001Z/source-manifest.json),
[source checks](../../artifacts/macos-validation-20261020T235001Z/source-final-verification.json),
[reference checks](../../artifacts/macos-validation-20261020T235001Z/reference-input-final-verification.json),
[transfer](../../artifacts/macos-validation-20261020T235001Z/transfer-verification.json).

The Mac clipboard PNG was inspected here: E15:F16, active F16, values 2,3 / 3,18,
translated SUM, arrow-commit help and action-scope label are clear without observed
clipping. The Mac inspector also records distinguishable inactive/active selections
and synchronized editor/formula-bar contents.
[Clipboard image](../../artifacts/macos-validation-20261020T235001Z/ci/20261020T235011455134Z/desktop-ui/results.clipboard.png),
[inspection record](../../artifacts/macos-validation-20261020T235001Z/image-inspection.json).

No optional manual interaction was performed. Routed keys/buttons do not certify
physical input, IMEs/accessibility, native pickers or clipboard-manager behavior.
The carried `prior-manual-failure.json` describes the earlier editor milestone, not
a new manual failure in this run; retain it as historical evidence. Routine five-group
manual replay is not required under the owner's updated acceptance workflow.
Known WSLg native abort, mounted-path save uncertainty and historical Linux viewport
p95 exception remain unresolved. No new capacity, filesystem or performance guarantee.

Windows/Linux/Mac automated validation is complete for the bounded scope. Ready for
parent implementation review; implementation acceptance has not yet been granted.
