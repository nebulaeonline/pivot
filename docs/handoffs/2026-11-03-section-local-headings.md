# ADR-0043 — section-local headings implementation

Status: **Implemented, accepted and closed by parent implementation review,
2026-11-03, with recorded timing qualifications retained.**

Section-local column/row headings and whole-section corner controls now expose the
existing independent Grid geometry. Labels retain scope addresses and are centered
in their slots, including the outer rulers. A toolbar checkbox and checkable menu
action share one session-local **Show interior headings** setting, initially on.
Hiding removes interior controls and input/accessibility targets while preserving
selection, zoom, geometry, workbook dirtiness and revisions. Outer rulers and
established geometry editing remain available.

## Authority and presentation

Rust derives bounded local heading descriptors from the same captured geometry map
as the cells. Each descriptor includes its axis address, physical interval/edge and
complete section bounds. Hidden-row compaction, stepped profiles, actual scope
tails and positional bound data use that same authority. No additional stored
section objects, axis identities, cell values or calculation graph are introduced.

Row/column activation requests exactly that section's local axis; corner activation
requests its complete section. Drag and Shift+Enter/Space extension retain canonical
either-boundary closure and expanded-operation confirmation. Ordinary outer rulers
retain active-section semantics. Explicit whole-scope actions stay separate, and
local selection does not redefine structural insertion/deletion.

Bounded client overlays use captured edge coordinates with the existing pixel/zoom
transform. Sticky/clipped strips prioritize the active section. Corners reserve
distinct targets before labels; scope-edge ruler gutters can host a section corner.
Protected cell/editor bounds remain unobscured, with outer-ruler and explicit
section-selection fallback for crowded layouts. Labels keep canonical addresses;
bound presentation identifies positional, read-only context.

Overlay controls are laid out during publication with the cell frame and related
controls. Replacement captures cancel old gestures, including same-revision
viewport refreshes. Toggle-off releases pointer capture and obsolete controls
cannot replay actions. Keyboard extension is handled before normal Button
activation. Existing geometry dividers and editing commands remain independent of
the interior-heading toggle.

## Admission and codec audit

Enumeration uses the existing bounded viewport pages: at most 4,096 mapped cells
and 8,192 local heading descriptors, with temporary memory reserved and lookup work
charged against the existing geometry work ceiling. Wire/client accounting includes
the new metadata under the existing 8 MiB presentation allowance. Overlay targets
are capped at 256; density fallback remains explicit. The existing shared capture
pool, global admission and retained-root rules remain unchanged.

Private v16 described only active-section outer rulers. The required local edge and
full two-axis section bounds therefore use a coordinated private **v17** cut.
Earlier session versions refuse before state creation. **WB16**, hosted v1 and
CB5/CB6 are unchanged; editor chrome and its toggle have no workbook serialization.

## Qualification lineage

The full local baseline passed 13 CI gates, all-feature Rust tests and strict
Clippy. A final keyboard-routing amendment changed only the two Desktop heading
files and passed the complete virtual UI sequence. Its source-qualified production
snapshot is:

`5aa6e35ffd513f63f28c1254a9af5271624823c22184f2cc0bee3c8d5638aff0`.

All six full native headless targets pass on that snapshot, with 951 executed
source files verified per target. macOS x64 is Rosetta. The original matrix is
`20261104T021354Z-2a72a492`; Windows x64's passing full retained-cache retry is
`20261104T022702Z-45291dc3`.

Windows x64's cold run failed the existing 65,536-row sparse-sort objective: its
first sample was 1,044 ms against the unchanged 1,000 ms limit. An unchanged isolated
diagnostic subsequently measured 876–944 ms, then the full retained-cache retry
passed. No sort implementation or limit was changed. Original failed outcomes
remain retained; this does not establish a resolution of cold-run timing behavior.
Historical Windows ARM64 cold structural-timing failures remain unresolved despite
this milestone's new cold pass.

The first Windows x64 managed GUI pilot failed in automation because it queried
row 10 without revealing it in a smaller native viewport. A test-only amendment
explicitly reveals the required rows/columns and forces a compact viewport in
regression coverage. Its snapshot is:

`13749c2f49b93d88f78ceaf7124d4a111979f747c561b81af48619a0a161eb8d`.

Only `InteriorHeadingsAutomation.cs` differs from the production snapshot. The
complete amended virtual UI sequence passes locally and on all six targets, with
three fresh iterations on each ARM64 target. Windows x64 and Linux x64 managed/AOT
GUI receipts pass with verified amended source manifests. Windows ARM64 and Linux
ARM64 subsequently returned passing automation receipts as well. However, owner
inspection of Linux ARM64 and its saved dark-theme image exposed a real visual
defect: heading controls existed but their labels disappeared against the white
worksheet. These receipts retain their original outcomes and source provenance;
they do not qualify the defective appearance. The Mac has not started.

The correction gives interior controls an explicit worksheet-compatible background,
foreground and border. A rendered-pixel regression checks actual centered
glyph/background contrast in both light and dark themes. An isolated copy of the
original production styling fails that regression in Dark with zero glyph pixels;
the corrected styling passes both themes. This is a production fix, requiring
fresh native UI and attended qualification. The corrected snapshot is
`5fa5148b6f71734328da30bb5fd963623dc1065f2484583b6d8829cec50351e4`.
All six targets pass the corrected complete virtual UI sequence, including the
light/dark pixel regression, with three fresh iterations on each ARM64 target.
Rust/protocol/backend inputs are unchanged from the full headless baseline.
All twelve final managed/AOT suites pass on the correction, with all 952 executed
source files verified per suite and readable saved images in both themes. Each
suite contains four actual graphical application reports, zero final charges and
no owned process survivors: **48 graphical reports** in total. macOS x64 remains
Rosetta evidence, not native Intel hardware.

| Targets | Final managed collection | Final AOT collection |
| --- | --- | --- |
| Windows x64/ARM64, Linux x64, macOS ARM64/x64 | `20261104T041727Z-c5b00b2c` | `20261104T041726Z-e4305a7a` |
| Linux ARM64 contrast pilot | `20261104T033746Z-c353d0a2` | `20261104T033745Z-621ed44a` |

The [reconciled qualification summary](../../artifacts/adr0043-local/qualification-summary.json)
records these receipts, executed manifests, per-theme pixel checks and cleanup.
The working runtime and test code match the final executed snapshot; documentation
was updated afterward for closeout. No full six-target 13-gate rerun is claimed
after the UI-only palette correction: the earlier full headless baseline is retained,
and the final runtime has focused complete native UI checks on every target plus
fresh managed/AOT graphical qualification on every target. Rust, protocol and
non-heading runtime inputs are unchanged. This distinction preserves the actual
scope of qualification rather than rewriting earlier receipts.

The checks exercise combined and single-orientation sections, exact local axis and
corner bounds, actual Button and keyboard activation, cross-boundary drag, master
toggle, released capture, stale same-revision overlays, accessibility names,
arranged visual targets and protected active-cell bounds. They supplement the
existing complete invoice construction/revision, confirmation, editing, clipboard
and workbook lifecycle sequences. Virtual checks do not replace attended GUI
evidence or claim OS-generated physical input.

## Environment and retention

The local NuGet vulnerability service was unavailable; local qualification used a
process-local `NuGetAudit=false` override. Package locks were not changed. Native
target receipts retain their actual tool and restore results.

Isolated .NET first-use setup had accumulated obsolete validation tools paths in
the two Windows VMs' user PATHs. Backed-up cleanup removed only absent per-run
entries; live tools, machine PATH and SSH settings were preserved. Ordinary Windows
x64 SSH/SFTP then worked again. Validation environments now set
`DOTNET_ADD_GLOBAL_TOOLS_TO_PATH=0` to prevent recurrence.

Native failures and completed evidence are archived with verified hashes before
cache pruning. Authored source, reports, workbooks and published AOT products are
preserved. The final contrast-runtime cleanup batch removed **72.8 GiB** of logical
disposable cache contents across all six targets, in addition to the earlier
35.3 GiB x64 batch. Native filesystem free-space observations are retained separately;
they need not equal logical file sizes, especially across guest volumes and APFS.
No WSL execution was used this milestone. cloc 2.06 reports
**107,836 source code lines across 443 files**. Tests/examples are subsets of those
totals. The [per-project report](../../artifacts/loc/ADR-0043/20261104T034153373838Z/report.md)
excludes generated output, dependencies and caches. Current project totals include
36,480 lines in pivot-workbook, 5,255 in pivot-session, 4,134 in Pivot.Desktop,
755 in Pivot.Avalonia and 720 in Pivot.Client. Test/example counts are subsets.
Parent implementation review accepts the corrected runtime and source-qualified
evidence, including the earlier full headless baseline followed by focused native
UI and fresh attended qualification of the UI-only palette correction. No further
implementation corrections, benchmarks or platform reruns are required.
Commit/push remains the owner's responsibility. ADR-0042 is
closed; ADR-0023 remains parked at G3. No next milestone is authorized here.

## Usability and future direction

Acceptance establishes the specified interaction contract, not that traditional
spreadsheet users will find Section Breaks intuitive. The owner favors a
spreadsheet-first experience. Actual document construction and small task-based
human usability studies can inform future interaction design; no such study is
claimed or required for this milestone's acceptance. Rendered-image checks remain
important for presentation features, as the dark-theme defect demonstrated.

Visually distinct section surfaces with independent alignment and spacing are a
possible future direction. These would remain logical regions of their parent
Grid scope, preserving addresses, cell ownership and calculation semantics rather
than becoming new named Grid Windows. Independent placement would require its own
presentation transformation and architectural evaluation, including intersecting
row/column breaks, overlap rules and view behavior. This discussion authorizes no
new ADR, runtime changes or placement implementation.
