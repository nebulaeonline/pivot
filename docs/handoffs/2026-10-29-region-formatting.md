# ADR-0030 — row, column and rectangular-range formatting

2026-10-29. Accepted and closed by parent implementation review.

Formatting now uses sparse, disjoint property regions under the immutable workbook
root. A cell, row, column, rectangle or entire sheet uses the same rule: the latest
successful action replaces each chosen property everywhere selected, including
earlier individually formatted exceptions. Unchanged properties remain intact.
The owner's 10,000-cell selection with 100 exceptions passes both independent
workbook tests and real session/Format-pane checks.

The pane resolves Mixed fields over the complete selection, including offscreen
positions. Row/column headers select geometric ranges; Shift extends header
selections. Worksheet-default editing affects inherited properties. Reset Formatting
sets explicit built-in values; Use worksheet defaults removes assignments. Explicit
false/default values remain explicit through later default changes and Save/reopen.
No scope has permanent precedence and no formatting-operation history is stored.

Copy/Paste, Fill and Move retain complete effective-style snapshot semantics,
including styled blanks. Move explicitly resets source formatting. A first default
snapshot can change workbook state even if its current appearance is unchanged;
repeating the same operation is a no-op. Styles remain position-owned, graph-free
and independent of calculation revisions. Strings such as `001234` remain exact
Strings; formatting never parses them. Decimal96 rendering and CSV policy are unchanged.

## Representation, admission and formats

Each worksheet/property has a charged immutable array with a balanced row-interval
index and column filtering. Candidate updates subtract intersections, coalesce changed
pieces with neighbors in deterministic horizontal/vertical passes and adaptively
sort by row to rebuild the index. Unchanged property maps remain shared. This is
bounded normalization, not a minimum-cover algorithm or a claim of logarithmic
two-dimensional lookup. Viewports query each relevant map once, then rasterize only
the bounded requested positions. Full-selection summaries count coverage/default gaps.

Admission is 65,536 property regions workbook-wide, 8 MiB estimated style/index data,
2,097,152 formatting-work units per command and 1,048,576 style-query work units.
Fill/Move/paste candidate style writes share one command-work budget. Splits, merges,
index traversal, sort-key comparisons, index construction and viewport output work
are counted. Persistent/candidate/scratch memory is charged; old captures retain
their own maps. The existing total workbook/runtime, content-command, clipboard,
Fill, 4,096-position style viewport and 1 MiB frame bounds remain independent.
Region/byte diagnostics replace the old styled-position count in the Format pane.
Fragmentation, byte/work exhaustion, cancellation and stale requests fail atomically.

**WB10 is the sole native format.** WB9 and earlier versions are explicitly rejected;
there is no backward reader, converter or migration. Current fixtures use WB10.
Historical artifacts remain evidence. Stable identities, rejected formula intent,
exact typed content and failed-Open/install semantics remain preserved in WB10.
The owned session uses **private v8**; **CB4** and frozen hosted v1 remain unchanged.
Until an explicit external compatibility commitment, justified experimental native
format cuts may break compatibility; automatic backward readers are not required.

The parked content-only history adapter's existing refusal guard now recognizes
inert built-in snapshots from ordinary Move/Fill and includes their retained bytes
in admission. It still rejects non-default presentation/worksheet defaults; no
formatting mutation records, presentation-history implementation or production
Undo/Redo were added. ADR-0023 remains parked at G3.

## Validation and measurements

All three native targets passed full headless CI **13/13**, attended managed desktop
automation and Native AOT publish/execution. Each graphical suite produced four
successful application reports per target: **24 successful graphical reports** total,
all with zero final runtime charges and no surviving owned processes. Backend reports
identify Win32, X11 and Avalonia.Native. Coverage includes the actual Format pane,
large-selection Mixed/overwrite behavior, row/column header events, defaults/reset/
inheritance, native clipboard/Fill/Move and WB10 persistence.

| 65,536-region workload, milliseconds | Windows x64 | Linux x64 | macOS ARM64 |
| --- | ---: | ---: | ---: |
| Fragmented local edit, maximum of four mutations | 3.526 | 2.788 | 2.260 |
| 16×16 viewport p95, 31 samples | 0.0555 | 0.0553 | 0.0297 |
| Replace fragmented map with whole-sheet assignment | 1.055 | 1.185 | 0.528 |
| Uniform whole-sheet Apply p95, 31 samples | 0.0040 | 0.0014 | 0.0033 |
| Whole-row Apply p95, 31 samples | 0.0031 | 0.0011 | 0.0030 |
| Whole-column Apply p95, 31 samples | 0.0039 | 0.0009 | 0.0008 |
| Native Save with all 65,536 regions present | 45.45 | 32.43 | 56.54 |
| Native reopen with all 65,536 regions present | 26.65 | 20.29 | 33.17 |
| Real .NET/Rust 65,536-cell numeric-table format p95 | 0.124 | 0.299 | 0.033 |

Staged 1,024/10,000/65,536-region checkpoints pass. Fragmented edits used 524,801
work units at the maximum checkpoint; whole-sheet replacement used 131,073, driven
by stored records rather than the 67,108,864 covered coordinates. A pristine
whole-sheet complete reset stores ten regions and allocates no graph identities.
The exact-boundary and first-excess cases verify atomic region admission.

The fragmented native file is 4,194,668 bytes; every reopened region is checked
against the independent checkerboard geometry/property/value oracle. Four retained
captures preserve their prior styles. The final stress checkpoint's charged peak
was 72,699,460 bytes (69.33 MiB). This is domain accounting, not process RSS.
Both required numeric table shapes (2,000×10 and 4,096×16) additionally pass full
formatted import/calculation/CSV/native persistence with independent complete-value
oracles. Full-size compact-table peak was 483,977,916 bytes (461.56 MiB), with
approximately 93 MiB steady charge: both original compact-corpus memory gates pass.
Raw results retain available process working-set observations; macOS peak-working-set
zero means unavailable evidence. Independent 262,144-identity checks still pass.

Evidence:

- [Native headless matrix](../../artifacts/validation-matrix/20261029T192021Z-954c3b65/matrix.md),
  [managed collection](../../artifacts/validation-matrix/20261029T192655Z-df8f264c/matrix.md),
  [Native AOT collection](../../artifacts/validation-matrix/20261029T192716Z-07b70dce/matrix.md).
- All **387 implementation inputs** match in all nine original source copies:
  [headless audit](../../artifacts/region-source-audit.json),
  [managed audit](../../artifacts/region-desktop-source-audit.json),
  [AOT audit](../../artifacts/region-aot-source-audit.json).
- Final review extended only `examples/regions.rs` to save/reopen the fully fragmented
  state and explicitly measure row/column/whole-sheet p95. Application/library sources
  are unchanged. Separate isolated native source-qualified executions passed on all
  three targets: [summary](../../artifacts/region-checkpoint-final-summary.json),
  [measurements](../../artifacts/region-checkpoint-measurements.json),
  [final hashes](../../artifacts/region-qualified-source-inputs.json).
  These benchmark-only results do not relabel the earlier source snapshots.
- [Native numeric-table and original region measurements](../../artifacts/region-native-measurements.json).
  [Local all-feature regression](../../artifacts/region-allfeatures-complete.log):
  **451 test executions passed**, including randomized independent dense-grid oracles,
  exact strings, old captures, malformed regions, defaults, resource boundaries,
  stale/cancelled rollback and current/obsolete native formats.
  [Strict workspace Clippy](../../artifacts/region-clippy-frozen.log) and
  [final benchmark Clippy](../../artifacts/region-final-checkpoint-clippy.log) passed.

## Retained failures and limits

The initial 65,536-region normalization attempt exhausted the unchanged work limit.
Global multi-axis sorting was replaced with changed-piece coalescing and adaptive
row sorting; no admission increase was needed. Original
[failure](../../artifacts/region-65536.log),
[intermediate correction](../../artifacts/region-65536-corrected.log) and
[passing correction](../../artifacts/region-65536-final.log) remain retained.
Earlier compile/fixture failures and corrected logs remain separate, including the
old envelope offsets/version-mutating tests and explicit-snapshot no-op expectations.

Fragmentation can legitimately exhaust a quota; unlimited checkerboard formatting,
minimum rectangle covers and arbitrary retained roots are not promised. Clipboard/
Fill remain bounded content operations, not whole-sheet formatting shortcuts.
Structural insertion/deletion, conditional formatting, custom formats and production
history remain outside this milestone. No format migration support was retained.

Short target-local `fmt.py` launchers ran managed then AOT with the existing live
consent prompts. [Short commands](../../artifacts/formatting-commands.txt) and
[installation receipt](../../artifacts/region-launcher-install.json) remain retained.
No additional validation infrastructure or consent bypass was introduced.

Parent accepts the implementation, normalization correction, performance/memory
results and supplied validation, including the separately qualified benchmark-only
extension and documented source distinction. No additional validation, acceptance
packet, reconciliation or AOT rerun is required. Preserve original failures and the
standing short-launcher convention. ADR-0023 remains parked at G3.

The owner will commit/push this completed milestone after the documentation update.
macOS updating is still pending. No further design, implementation or validation
work starts until the owner explicitly indicates it is OK to proceed. The standing
milestone cadence is recorded in [STATUS](../STATUS.md).
