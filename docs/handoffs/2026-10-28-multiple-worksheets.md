# Multiple worksheets — ADR-0024 implementation

Date: 2026-10-28. **Accepted and closed by parent implementation review.**
The supplied validation is sufficient; no additional qualification round is needed.
ADR-0023 remains parked at G3.

## Functionality

New workbooks start clean with Sheet1. Add/Rename controls and worksheet tabs support
up to 16 sheets, each with independent axes, positions, selection and scroll state.
Switching commits the old sheet's editor first and does not itself dirty the workbook.
New/Open resets view state, including reopening the same durable document identity.

One canonical Rust graph serves every sheet. Local references bind in their owning
sheet; qualified references and aggregate rectangles can cross sheets. Scalar
bindings follow stable identities, ranges follow current occupants, and captures
retain all sheets coherently. Cross-sheet cycles and invalidation use the existing
calculation engine. Cell/token/allocation budgets remain workbook-wide.

Rename preserves authored source and stable bindings. Display/editor formulas and
clipboard exports use current effective names, including `'O''Brien Sales'!A1`.
Copy/Paste/Fill retain qualification and dollar axes; missing destination qualifiers
reject atomically. Across workbooks, qualified names resolve in the destination,
without an external link. Creating a previously unknown sheet does not execute an
already rejected formula draft; resubmission is required.

Save/Open handles the complete workbook. WB7 adds the sheet directory, cell ownership
and bound qualifier metadata. Private session version 5 requires explicit worksheet
context and rejects incompatible peers; queued .NET requests retain their original
sheet. Desktop replies are checked against document generation, worksheet and view
epoch before painting. Hosted v1 and native clipboard CB2 are unchanged.

## Validation

| Gate | Result / evidence |
| --- | --- |
| Windows x64 native headless CI | Passed, 13/13 |
| Linux x64 native headless CI | Passed, 13/13 |
| macOS ARM64 native headless CI | Passed, 13/13, local APFS |
| Frozen implementation inputs | All 349 non-Markdown source inputs match on all three targets |
| Changed Rust crates, all features | Core/workbook suites and document suite passed; 247 tests combined |
| Rust formatting / strict all-feature Clippy | Passed |
| Session / complete Save/reopen | Passed on all three native targets, including projection/binding regressions |
| Managed actual-window automation | Passed on Windows x64, Linux x64 and macOS ARM64; four real application reports each, zero remaining charges and no owned survivors |

Final native reports and source hashes:
[`artifacts/worksheets-local/final-headless`](../../artifacts/worksheets-local/final-headless/).
Local all-feature logs: `artifacts/worksheets-rust-regression.log` (core/workbook),
`artifacts/worksheets-document-allfeatures.log` (document rerun),
`artifacts/worksheets-clippy.log`.
Desktop activation packet:
[`20261028T151654Z-a90f4206`](../../artifacts/validation-matrix/20261028T151654Z-a90f4206/).
Its initial NeedsGraphicalSession results are preparation outcomes, not GUI passes.
All three attended runs subsequently failed at the same existing binding-editor
check: after proof-mode layout changed, typing could occur before the new frame was
ready, and no cell was entered. The new worksheet checks preceded this failure.
The binding test now waits for a current frame and asserts selection/editor entry
before typing is committed; the corrected desktop build passed with no warnings.
The retry packet is
[`20261028T162256Z-2741e23c`](../../artifacts/validation-matrix/20261028T162256Z-2741e23c/).
The collected retry results are
[`20261028T164150Z-bf7c565c`](../../artifacts/validation-matrix/20261028T164150Z-bf7c565c/):
all three targets passed, including the explicit `multiple_worksheets` assertion.
The observed backends were Win32, X11 and native macOS respectively. Each target
produced four successful managed application reports, zero remaining runtime charges
and no surviving owned processes. Only the GUI automation changed after the headless
source snapshot; the corrected automation was built and executed in these runs.
Unselected ARM64 guests' old "already passed" messages do not establish coverage of
this feature; macOS x64 was not attempted or required for this scoped milestone.

The first native attempt is retained in
`artifacts/validation-matrix/20261028T142251Z-a0502999`: Windows exhausted disk space;
Linux/Mac caught an incorrect new cycle-test assertion (a local capture is allowed;
evaluation of a cyclic output must fail). Corrected source passed using the same
native workspaces/caches, with fresh reports. Obsolete Windows build/download caches
were removed; source and evidence were preserved. The owner subsequently expanded
the Windows x64 VM disk to 200 GB and reported approximately 90 GB free on C:,
providing headroom for later builds; this is not an additional qualification claim.
Two optional-history local tests
initially encountered sandbox filesystem permissions; the document all-feature rerun
with native filesystem access passed. No history research or integration was resumed.

Desktop automation adds actual Add/Rename/tab/editor, effective formula, stale-frame,
per-sheet viewport, clean-switch and Save/reopen checks to the existing suite. No new
VM qualification or unrelated six-RID/AOT requalification is claimed.

## Material limits

Rectangle qualifiers appear on the first endpoint only: `Revenue!A1:B5`. Repeated
endpoint qualifiers, 3D ranges, external links, sheet deletion/duplication/reordering,
cross-sheet Move and sheet history remain out of scope. View preferences are not
saved. A rename that would exceed the existing editable formula byte limit rejects
atomically. Existing structured-result proof modes remain separate and single-sheet;
parked history adapters explicitly refuse unsupported new workbook inputs.

**WB7 rejects WB1–WB6.** Earlier experimental documents cannot be opened; no migration
reader is included. Before real users save valuable files, establish a deliberate
compatibility commitment. Perpetual incompatibility is not a product policy.

Parent accepted the implementation, validation and documented corrections and closed
the milestone. Retain this single handoff and the original failure evidence; no
additional validation or reconciliation packet is required. The recommended next
feature is bounded document-owned cell formatting. ADR-0023 remains parked.
