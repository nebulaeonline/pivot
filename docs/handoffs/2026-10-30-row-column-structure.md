# ADR-0031 — row and column insertion/deletion

2026-10-30. Accepted and closed by parent implementation review.

The parent accepts the implementation, clipboard correction, source-qualified
six-target evidence and reported resource objectives. Rosetta execution and single-
observation timing qualifications remain explicit. Original failures and successful
recoveries remain separately recorded. No further validation cycle, reconciliation
packet or AOT run is required.

The desktop provides four whole-axis commands for the selected contiguous band:
insert rows above, delete rows, insert columns left and delete columns. Extents stay
fixed. Insertion refuses to discard any positioned computational identity, including
cleared cells and Blank reference anchors. Deletion permanently retires identities;
new occupants at the old coordinates do not repair references. Surviving CellIds,
graph identities and scalar bindings remain stable, including `$` references.

Deleted operands calculate a local `ReferenceDeleted` fault and display `#REF!`.
Unrelated outputs and unexecuted lazy branches remain calculable. Authored source
remains exact provenance while the formula bar displays effective coordinates.
An unchanged editor commit preserves provenance and revision; replacing a broken
operand is ordinary formula editing. There is no automatic or bulk repair workflow.

Range operands retain SelectionIds while trusted structural reframing implements
the approved affinity, reversed endpoints and fixed-edge clipping. An entirely
deleted range faults instead of becoming an empty sum. Sparse property regions
transform geometrically, copying the predecessor profile into inserted bands and
the last surviving profile into deletion tails. Explicit defaults remain distinct
from inherited gaps. Notices identify actually discarded assignments; coalescing
alone does not imply clipping. Complete semantic no-ops leave state unchanged.
An insert/delete pair is not Undo, including at the formatting tail.

Planning, placements, operand/range changes, retirements and formatting publish as
one private graph/workbook candidate. Existing captures remain coherent. Selected
bands retain a useful focused cell and viewport, including whole-column selections.
The approved structural work limit is 33,554,432; existing content, identity, region,
file, frame and runtime limits remain unchanged.

## Representation and interoperability

**WB11 is the sole native format; private session v9 and CB5 apply.** Fault provenance
distinguishes local retired cell/range identities, explicitly authored `#REF!`, and
transferred faults that cannot claim local retired targets. Save/reopen, native
Copy/Paste and Fill preserve that distinction. Obsolete formats are rejected; no
backward reader or migration is included. Frozen hosted v1 and canonical value types
remain unchanged. Production Undo/Redo and parked history research remain outside
this milestone. Exact Strings such as `001234` retain their type and content.

Immutable cell payloads are shared across moved records instead of copied into every
retained root. Native restore constructs its transient graph before persistent cell
copies; native projection charges transfer/codec representations separately from
persistent index estimates. Copy-out and history admission still count the shared
payload's actual heap footprint.

ARM64 GUI coverage exposed a clipboard read defect: editor state and verified text
were intact, but Paste's native snapshot returned no text. Pivot now uses Avalonia's
ownership-checked [in-process API](https://raw.githubusercontent.com/AvaloniaUI/Avalonia/12.1.4/src/Avalonia.Base/Input/Platform/Clipboard.cs)
for its own data, without disposing clipboard-owned data. External snapshots are
disposed normally and allow three bounded read attempts with the existing 20 ms
delay. Native data never downgrades to text; obsolete CB4 is explicitly refused.
Stale interaction, selection and revision checks remain in place. No copy, paste or
workbook edit is replayed. Test readback uses bounded reads with exact content checks.

## Validation

All six targets passed full headless CI **13/13** and managed/Native AOT graphical
acceptance. macOS x64 is **Rosetta emulation**, separately labeled. Final selection:
**48 successful application reports**, all with zero final runtime charges and no
surviving owned processes. Structural menus, focused bands, unchanged editor commits,
operand repair, local/lazy faults, CB5/Fill, clipping, exact Strings and WB11 passed.

| Target | Headless | Managed GUI | Native AOT GUI | Execution |
| --- | --- | --- | --- | --- |
| Windows x64 | 13/13 | Passed | Passed | Native |
| Linux x64 | 13/13 | Passed | Passed | Native |
| Windows ARM64 | 13/13 | Passed | Passed | Native |
| Linux ARM64 | 13/13 | Passed | Passed | Native |
| macOS ARM64 | 13/13 | Passed | Passed | Native |
| macOS x64 | 13/13 | Passed | Passed | Rosetta |

The full native headless matrix preceded the GUI-discovered clipboard correction;
Rust/workbook code did not change afterward. That C# correction passed another full
local CI **13/13**, headless clipboard regressions, and final managed/AOT execution
on all six targets. Clipboard cases cover delayed reads, empty/exact text, obsolete
formats, mode changes, bounded exhaustion, stale interactions, ownership changes
and disposal. All-feature Rust regression passed **458 tests**; all-target/all-feature
Clippy passed with warnings denied.

Independent oracles cover identity continuity, lazy faults, cross-sheet/reversed/
absolute ranges on both axes, all ten style properties over 48 dense small-grid
changes, defaults versus inheritance, protected Blank tails, persistence and repair.
The prescribed numeric tables and 1,024/10,000/65,536 structural/style checkpoints passed.

Figures below are single-observation maxima over four structural actions at 65,536,
not latency percentiles. Memory is charged runtime memory, not file size or process
RSS. Steady includes four old roots; peak includes Save and opening a second book.

| Target | Numeric max, ms | Steady, MiB | Peak, MiB | 65,536-region max, ms |
| --- | ---: | ---: | ---: | ---: |
| Windows x64 | 827.52 | 251.19 | 489.35 | 14.50 |
| Linux x64 | 749.23 | 251.20 | 489.36 | 13.89 |
| Windows ARM64 | 417.35 | 251.20 | 489.36 | 17.67 |
| Linux ARM64 | 545.02 | 251.20 | 489.36 | 22.57 |
| macOS ARM64 | 270.66 | 251.19 | 489.35 | 11.31 |
| macOS x64, Rosetta | 684.47 | 251.19 | 489.35 | 16.28 |

Original failures remain recorded. The first 65,536-cell checkpoint exceeded the
unchanged 512 MiB objective; sharing/accounting/restore scheduling corrected it
without raising the gate. GUI retries corrected stale format/address assertions,
isolated the clipboard defect, and standardized readback verification. The initial
Rosetta tool probe timed out; a fresh full run passed. Windows x64's later AOT restore
exhausted its disk before launching the app. Collected compiler caches were removed
under the retention policy, preserving reports, sources and published images; headless
publishing and final AOT then passed. Other volumes were checked for adequate space.
No original result was rewritten.

## Evidence and next step

- `artifacts/structure-final-matrix.json` selects successful logical runs across
  retries, retaining paths to original receipts without erasing failed rows.
- `artifacts/structure-native-headless-summary.json` contains checkpoint measurements.
- Full headless matrices: `20261030T050753Z-6972fa6b` and
  `20261030T054837Z-e58e1c21` under `artifacts/validation-matrix/`.
- Final GUI collections: managed `20261030T235815Z-2989dfe8` plus Windows ARM64
  `20261030T123452Z-776d7f1a`; AOT `20261030T235821Z-b6159b0d` with successful
  Windows x64 retry `20261030T235830Z-6310faff` and Windows ARM64
  `20261030T131412Z-36880b38`.
- Local post-fix CI: `artifacts/structure-clipboard-ci/20261030T122341231132Z/summary.json`.
  Rust/lint: `artifacts/structure-allfeatures-reviewed.log` and `artifacts/structure-clippy-final.log`.
- Housekeeping/recovery: `artifacts/pruned-build-products.json`,
  `artifacts/structure-native-space.json`, and `artifacts/structure-win-x64-publish-check.json`.

Parent review is complete and its disposition is recorded in ADR-0031, STATUS and
the ADR index. Stop for the owner's commit/push and explicit next go-ahead.
ADR-0032, Worksheet Sorting, is assigned as design-first work on hold; record/identity/
formatting semantics must be examined before implementation. No sorting design or
feature work, commit/push, production history work or additional validation is started
by this acceptance. Filtering is only a later candidate, provisionally ADR-0033.
