# First-class ranges and SUM — Slice C implementation

Date: 2026-10-20. Implementation ready for parent review; three-platform validation is complete.
Parent implementation acceptance remains pending. [ADR-0016](../adr/0016-first-class-ranges-and-sum.md)
is accepted. No new milestone is started.

## Behavior

`=SUM(A1:B20)` accepts one same-sheet rectangle, case-insensitive spelling and
reversed endpoints. Scalar arithmetic can surround SUM. Bounds stay in the stable
worksheet coordinate namespace; moving occupants changes membership, moving the
formula preserves its bounds, and scalar references continue following CellIds.
Blanks contribute zero without anchors. Original source and spans survive WB2.
Invalid syntax and out-of-bounds references remain editable drafts. Decimal96
addition starts at zero, visits current populated coordinates in row-major order,
and reports intermediate overflow. Captured S retains values and membership after T.

## Canonical representation and admission

The existing canonical core owns SelectionId descriptors, sparse row/column
placements, reverse locations, selection consumers and a persistent augmented AVL
row-interval coverage index. Columns filter coverage candidates. No per-blank node,
durable expanded member list, workbook callback or C# evaluator is introduced.
Canonical validation temporarily enumerates/deduplicates populated member nodes;
cycle/selected-closure analysis uses that same dependency walker. Reduction retains
coordinate order, including duplicate-position values if the generic core places a
node more than once. Workbook identities remain one-to-one.

| Resource | Ceiling |
| --- | ---: |
| Active selection descriptors after commit | 128 |
| Temporary descriptors during replacement | 256 |
| Allocated selection identities including retired IDs | 8192 |
| Generic core sparse points | 4096 |
| Selection consumer links | 4096 |
| Per-formula sparse member traversal budget | 65,536 |
| Canonical validation visit budget | 2,000,000 |
| Invalidation seen/pending nodes | 65,536 each |
| Invalidation scratch admission | 16 MiB |
| WB2 envelope | 4 MiB |

Existing workbook limits remain 1024 rows/columns, 1024 live cells, 8192 allocated
identities, 256 edits and 4096 formula tokens/source bytes. Persistent index path
allocations use the existing shared domain ledger. Closure scratch reserves at most
64 bytes per bounded dependency plus 4096 bytes; reduction reserves 64 bytes per
sparse point plus 4096 and 64 bytes per expression token. Cancellation and admission
failure discard all candidate roots. Batch index updates are bounded by admitted
operations and point/descriptor/consumer ceilings, never rectangle area.

Typed inspection reports Scalar(NodeId) or Selection(SelectionId); selection images
provide descriptor geometry and placements. Scalar-only edge/materialization APIs
explicitly reject selection graphs. The older NodeView binding slice remains the
scalar portion; clients requiring complete dependency types use typed_dependencies.

## Complexity and measurements

Membership uses populated-row and column range traversal. Coverage queries prune by
row interval, then filter columns; overlapping candidates can still approach all
active descriptors. Topology edits with selections currently validate all live roots.
This is not a constant-time invalidation or output-optimal traversal claim.

Private response diagnostics count actual visited row/column range-tree and coverage
AVL nodes, selection candidates and emitted members. They do not count every keyed
lookup or scalar graph node visit. Counters are cumulative across shared captures;
newly opened graphs start new selection counters. Allocation traffic counts successful
domain-accounted allocations/bytes, not every process malloc or RSS. Diagnostic
output buffers and continuing retained ledger charge are distinct.

`tests/application-session/RangeChecks.cs` records 11 observations after one warmup
for binding, member edit, add/remove, boundary move, selected SUM and capture/release.
Two-request operations include both requests; capture includes release. File timing
is a separate single observation, not a latency guarantee. Ordinary caps stay intact.
Full 1024×1024 sparse geometry and 40 overlapping rectangles with 80 points have
independent integer-oracle tests; a full-sheet formula residing in that sheet is
correctly a self-cycle. No production-million-cell claim is made.

## Portable boundary

PIVOTWB2 deliberately rejects WB1, with an accepted WB1 fixture retained solely for
rejection. WB2 stores the scalar graph skeleton, authoritative selection overlays,
identity ledger, policy version and sparse cell records. Overlay slots in the
transient scalar skeleton are explicit non-executable invalid markers; they are never
numeric substitutes. The selection-aware importer validates the complete combined
image and reinstalls bound IDs, without resolving historical formula text. Sparse
points are reconstructed once from nonblank records, not repeated for every range.
Frozen hosted v1 descriptors/fixtures remain unchanged; storage capture returns
UnsupportedVersion for selections. No hosted range publication is added.

Tests cover selection-aware round trips, hosted rejection, checksum-valid WB2
policy/count/tag corruption, source/binding consistency, retired identities,
canonical cycles and repair, outside-input/computed-member invalidation, overlap,
move/clear/retire/reuse, ordered overflow, late cancellation, late failure, admission
rollback and zero-charge cleanup. Real C#–Rust tests cover S-save/T-edit and a separate
reader process. The actual Avalonia window checks SUM, edit/move and Save/Open.

## Manual checks and unresolved issues

The user confirmed Windows native Open/Save As and Save/Discard/Cancel dirty-close
checks passed. An untouched new document initially prompted unnecessarily; the
session now establishes a clean initial baseline without claiming a saved file,
with real-boundary regressions for initial/New/Close and blank-clear no-op behavior.

The user reports WSLg Save As and standalone Save succeeded to Windows storage,
but Save-on-close left the window open with the footer **“Unsaved Changes -
Installation uncertain: reopen to verify”**. The session did not acknowledge a clean
Save; preserving the window protects work even if new bytes reached the file. This
is consistent with the existing mounted-path limitation; the exact failing operation
was not isolated. The user subsequently confirmed that the manual WSLg Save-on-close
repeat **passed** with the `.pivot` file on local WSL2 storage: saving closed the
application window correctly. This validates that native-path close scenario, not
Windows-mount installation certainty. The user also confirmed that an untouched
workbook closes under WSLg without a Save/Discard/Cancel prompt.
Preserve the historical WSLg native abort
(`free(): invalid pointer`, SIGABRT); native filesystem passes resolve neither issue.
If the abort recurs, retain an actionable native backtrace.

Fresh native macOS APFS validation passed, with 291 default / 307 all-feature tests,
strict lint, .NET/parity/IPC, real session and actual-window checks. All 318 source
hashes and 168 build inputs match; no product fixes or validation reruns were needed.
The user confirmed all Mac manual checklist groups, including native pickers,
Save/Discard/Cancel and untouched close. Evidence sources remain distinct.
[Native report](../../artifacts/macos-validation-20261020T130207Z/REPORT.md).

The [validation note](2026-10-20-ranges-validation.md) contains the three-platform
measurements, provenance and retained limitations. Submit this note with ADR-0016
and the validation note for parent implementation review. The
[APFS packet](2026-10-20-macos-ranges-instructions.md) is retained as the executed
validation procedure; no new milestone is started.
