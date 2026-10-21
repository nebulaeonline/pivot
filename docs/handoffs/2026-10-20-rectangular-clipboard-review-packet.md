# Parent review packet — rectangular clipboard, WB3 and desktop acceptance

Date: 2026-10-20. **Implementation and three-platform automated validation complete;
request implementation review.** ADR-0017 design is already accepted with the clean
WB3 compatibility revision. No implementation acceptance is presumed.

## Delivered behavior

- Native rectangular copy/paste translates effective scalar bindings and SUM geometry,
  supports all dollar-axis modes and preserves exact decimal parts/source provenance.
  External TSV formulas bind literal addresses. Existing destination identities remain;
  new bindings are local. Blanks, overlap and later-created destination targets use one
  atomic Rust batch within the unchanged 256-slot/edit and underlying resource limits.
- Typed clipboard input is bounded and untrusted; malformed/unsupported native data
  rejects without TSV fallback. Retired targets, bounds, quota and revision failures
  install nothing. No alternate translator/evaluator, source graph ticket or per-cell IPC.
- PIVOTWB3 uses current grammar; WB1/WB2 explicitly reject, with retained fixtures and
  no legacy reader/parser mode. Rejected drafts remain non-executable until explicit
  resubmission; combined graph/selection validation and frozen hosted v1 remain intact.
- At the owner's request, `python scripts/ci.py --desktop` automates routine five-group
  acceptance, including actual cross-process OS clipboard and window-close Cancel/Save,
  Discard and clean-close peers. Screenshots support optional inspection. This replaces
  recurring manual replay, without claiming physical keyboard or native-picker testing.
- At the owner's request, plain inline-editor arrows commit and move. Rejected edits
  stay put; unchanged F2 exit preserves bindings. Modified arrows and formula-bar arrows
  retain text editing. Native bytes advertised but briefly unavailable receive two
  bounded read retries, with interaction guards and no text fallback or mutation replay.

[ADR-0017](../adr/0017-rectangular-clipboard-and-formula-copy.md),
[implementation/resource profile](2026-10-20-rectangular-clipboard.md),
[automation, arrow semantics and evidence](2026-10-20-desktop-acceptance-automation.md).

## Validation

| Platform | Default Rust | All-feature Rust | Desktop/boundary/cleanup |
| --- | ---: | ---: | --- |
| Windows | 309 | 325 | Passed, zero charge |
| Isolated Linux/WSLg | 310 | 326 | Passed, zero charge |
| Native macOS/APFS | 310 | 326 | Passed, zero charge |

Full CI, strict lint, .NET Release, decimal/protocol parity, IPC and application-session
checks pass. Final Windows/WSLg UI runs cover the later automation/arrow changes; the
fresh Mac integrated `--desktop` run covers the complete final implementation. Separate
peer PIDs, private UTI receipt, reopened Save/Discard generations, invalid-draft/WB3
roundtrips and historical version rejection are recorded. Main/peer/boundary children
close with zero Rust charge; the Mac report records no surviving owned processes.

The Mac run copied dirty/untracked source from SMB into fresh native APFS, with all
builds, .NET outputs, temporary files, test documents and initial reports on APFS.
All 339 copied source hashes independently matched this tree before this documentation
update; all 173 build/test inputs match. The transfer record verifies 391 returned
files and no missing links; 391 files are present here. No Mac setup failures, code
fixes, weakened assertions, capacity changes or gate reruns were needed.

- [Native Mac report](../../artifacts/macos-validation-20261020T235001Z/REPORT.md)
- [Mac integrated CI](../../artifacts/macos-validation-20261020T235001Z/ci/20261020T235011455134Z/summary.json)
- [Mac desktop and arrows](../../artifacts/macos-validation-20261020T235001Z/ci/20261020T235011455134Z/desktop-ui/results.json)
- [Mac cross-process clipboard and Cancel/Save](../../artifacts/macos-validation-20261020T235001Z/ci/20261020T235011455134Z/desktop-ui/results.clipboard-save.json)
- [Final Windows desktop](../../artifacts/desktop-acceptance-windows-final/results.json)
- [Final WSLg desktop](../../artifacts/desktop-acceptance-linux/results.json)

The Mac clipboard image was inspected here; selected E15:F16, active F16, transformed
SUM, values, controls and arrow help are readable without observed clipping. The Mac
inspection record also covers inactive selections, peers and synchronized editing.

## Limits and review boundary

Prior failed runs/logs remain intact. Windows and WSLg desktop suites must run
sequentially because their clipboards are connected. Transient native clipboard
availability was observed; the diagnostic probe did not establish its root cause.
Retries are bounded, and persistent failure remains an explicit error.

No optional Mac manual interaction occurred. Routed events do not certify physical
input, IMEs/accessibility, native pickers or clipboard managers. The old manual-failure
record carried in the report is historical, not a newly observed failure in this run.
Known WSLg native abort, mounted-Windows-path installation uncertainty and historical
Linux viewport p95 exception remain unresolved. No broader filesystem, scaling or
performance guarantee is claimed.

Cut, drag-fill/autofill, structural edits, formatting/text-value expansion, generalized
clipboard framework, alternate graph, SQLite and capacity increases remain excluded.
No further feature work has begun; stop for parent implementation review.
