# ADR-0033 — worksheet filtering implementation

2026-10-31. Accepted and closed by parent implementation review.

The parent accepts all three design clarifications, the mapped-viewport corrections,
exact typed payload parity, unchanged resource objectives, WB12/private v11 and
unchanged CB5/hosted v1. The supplied source qualification, Rosetta coverage,
observed timing maxima and compiler-discovery qualifications remain in force.
Original failures and successful recoveries stay separate. Verified archives,
cleanup and the initial diagnostic LoC baseline are accepted. No further validation,
reconciliation packet or AOT rerun is required.

Pivot persists one typed filter definition per ordinary worksheet and derives
whole-row visibility without moving cells, changing identities or rebinding formulas.
Hidden cells remain calculation members. Exact String criteria retain leading zeros,
case and Unicode content; Decimal96, Boolean, Blank, empty String and completed
errors keep distinct meanings. Columns combine with AND; each column supports one
or two predicates with AND/OR. Text matching is ordinal, without coercion or patterns.

The desktop offers an explicit rectangle, header exclusion on by default, reviewed
adjacent-range expansion, typed criteria, header controls, active counts and Clear.
Scrolling, navigation, hit testing and editor geometry use actual worksheet rows.
Maps, cells, styles, addresses and counts publish together. Draft-hiding publication
waits for commit/cancel; zero visible rows leaves recovery controls available.

## Mandatory invariants

Structural mutation atomically marks the existing definition NeedsReview and reveals
all rows. A no-op/refused mutation does not suspend it. The editor labels the former
range/keys and requires explicit review; Reapply never repairs or rebinds them.
Operational refresh failure is transient presentation state: retain the definition,
revision and dirty state, show all rows, and require explicit Reapply to retry a
failed refresh. Pending and failed views block commands requiring qualified hidden
row counts. Calculation runs on pinned captures with cancellation, bounded pending
work, stale-result refusal and joined worker cleanup.

Bulk operations retain full logical rectangles, including hidden cells. Confirmation
binds exact immutable operation bytes, complete source/destination footprints,
revision, view generation and hidden-row union count. Paste bounds come from the
actual payload; Move includes both endpoints; structure includes the shifted tail.
Changed clipboard content, selection, generation or state cancels dispatch. Tickets
are one-use, with no automatic replay or visible-only packing. Single-cell editing
remains available in the explicitly unfiltered fallback.

## Persistence and resources

WB12 stores definitions and structural NeedsReview, not derived maps or transient
failures. Obsolete native workbook versions are refused; no migration reader was
added. Private session v11 supplies filtering and exact-operation preview/tickets.
CB5, hosted v1, canonical value types and calculation semantics remain unchanged.

Aggregate definition admission remains 1 MiB. Canonical storage accounting uses
fixed structural units plus literal UTF-8 bytes, independent of allocator capacity;
actual heap capacity is charged separately. Sparse key scans and charged captured
values avoid a height-by-width key matrix or virtual-cell enumeration. Work remains
bounded at 33,554,432 units, including String comparisons and map construction.
Definitions, jobs, maps/status, tickets and response construction use existing
resource admission. No resource threshold was raised.

Routine responses contain a compact filter summary. The editor retrieves the full
definition separately so near-quota criteria cannot crowd viewport cells out of
the existing frame. Oversized reads refuse without terminating the workbook session.

## Validation

All six configured targets passed headless CI **13/13**, managed GUI and Native AOT
GUI acceptance. macOS x64 is **Rosetta**, not native Intel hardware. The selected
results contain **48 successful graphical application reports**, **48 zero final
runtime charges** and **no surviving owned processes**.

| Target | Headless | Managed GUI | Native AOT GUI |
| --- | --- | --- | --- |
| Windows x64 | 13/13 | Passed | Passed |
| Linux x64 | 13/13 | Passed | Passed |
| Windows ARM64 | 13/13 | Passed | Passed |
| Linux ARM64 | 13/13 | Passed | Passed |
| macOS ARM64 | 13/13 | Passed | Passed |
| macOS x64, Rosetta | 13/13 | Passed | Passed |

Final local CI passed **13/13**, all-feature Rust tests passed **480**, and strict
all-target/all-feature Clippy passed. Focused IPC tests cover exact criteria,
stale tickets, complete footprints, structural review, near-quota WB12 reopen,
operational suspension, joined jobs and read-path parity. In-memory UI probes
reproduce expanded criteria, mapping, confirmation, zero-row recovery and the
viewport regressions; they are supplementary, not graphical acceptance.

Preserve source qualification: native headless snapshots other than Linux x64
precede the final session viewport corrections. Linux x64 headless, final local
CI and all final managed/AOT GUI suites include those corrections. Core filtering
and performance-checkpoint code did not change during the viewport corrections.

The original Windows x64 failures remain separate:

1. The new test omitted TSV's existing apostrophe prefix on three Strings. Correcting
   the expected text preserved exact leading zeros; product clipboard behavior did
   not change.
2. Mapped reads omitted ordinary error fields for completed errors, causing client
   validation to reject a deleted-reference cell. Restoring error diagnostics fixed
   the transport closure and subsequent structural refresh timeout.
3. An invalid formula refused batch planning and lost the whole viewport. Mapped
   reads retain the batch fast path; failed planning now uses independent cell reads
   from the same pinned capture, preserving editable errors and unaffected cells.
   Comparison also corrected normalization of submitted signed zero and preserved
   the existing formula binding metadata/content support.

Complete ordinary and mapped cell payloads now agree for the regression fixture:
exact decimals, String/empty String, Boolean, authored text and error diagnostics.
In-memory UI reproduction passes invalid-formula input followed by empty clearing.
No clipboard safety guard was weakened and no operation was replayed to gain a pass.

## Observed performance

All nine corpora per target passed the unchanged apply/refresh, Save/reopen and
256 MiB steady/512 MiB peak objectives. These are maxima of four observed apply/
refresh samples, **not percentiles**. Memory is runtime-charged allocation, not RSS
or disk. Each corpus retains four old captures and verifies identities, graph state,
typed membership, WB12 reopen and zero final charge.

| Target | 65,536-row sparse apply max, ms | Sparse refresh max, ms | Text apply max, ms | Sparse steady, MiB | Sparse peak, MiB |
| --- | ---: | ---: | ---: | ---: | ---: |
| Windows x64 | 119.64 | 122.40 | 218.49 | 92.88 | 331.04 |
| Linux x64 | 122.01 | 111.74 | 205.98 | 92.88 | 331.04 |
| Windows ARM64 | 88.69 | 71.75 | 146.44 | 92.88 | 331.04 |
| Linux ARM64 | 74.81 | 77.03 | 126.25 | 92.88 | 331.04 |
| macOS ARM64 | 47.81 | 45.33 | 75.24 | 92.88 | 331.04 |
| macOS x64, Rosetta | 67.02 | 59.03 | 129.19 | 92.88 | 331.04 |

The blank virtual-sheet case covers 65,536 by 1,024 geometry with zero positioned
cells; it does not allocate a virtual worksheet of values. Other corpora cover
numeric/text keys, ten conditions, sparse tall data and 1K/10K/64K positioned cells.

## Evidence, tooling and next step

- `artifacts/filtering-final-matrix.json` selects successful receipts without
  rewriting failures. `filtering-native-headless-summary.json` records all nine
  checkpoints per target, observed timings and original report paths.
- Headless matrices: Windows x64 `20261031T150034Z-4bb1ee4f`, four Mac-hosted RIDs
  `20261031T150901Z-af14147d`, Linux x64 `20261031T154943Z-e27e358e`.
- GUI collections: Windows x64 managed `20261031T154810Z-34609275`, AOT
  `20261031T154938Z-812dcb8b`; remaining five targets managed
  `20261031T161454Z-0e6bdc3d`, AOT `20261031T161453Z-e7a7689e`.
- Final local CI: `artifacts/filtering-read-parity-ci/20261031T152811084361Z/summary.json`.
  Rust/lint: `filtering-read-parity-allfeatures.log`, `filtering-read-parity-clippy.log`.
  Parity/UI: `filtering-read-parity-comparison-final.log`,
  `filtering-invalid-input-ui-fixed.log`. Pilot diagnosis JSON files retain causes
  and corrected run qualifications.
- Windows VMs report VS 17.14.44, toolset folder 14.44.35207 and compiler
  19.44.35229. ROBPC has that matching VS 2022 profile installed; selecting its
  developer prompt pins it. The default latest-installation probe selected VS 18.11.
  Discovery is not a retrospective assertion about a build's chosen environment.
  `/Bv` exits with the expected missing-source diagnostic because the probe compiles
  no source. Raw probes and native prerequisites are retained.
- The [per-project LoC report](../../artifacts/loc/ADR-0033/latest.md) starts the
  recorded milestone baseline: **85,652 code lines across 366 files**. It counts
  canonical source once with cloc 2.06,
  excludes generated/dependency/cache output, and separates comments, blank lines,
  identifiable tests/examples and language totals. Future milestones include deltas.
- Cleanup receipts and SHA-256-verified archives are indexed in
  `artifacts/pruned-build-products.json` and `artifacts/archives/index.jsonl`.
  Completed native/local compiler caches are discarded; source snapshots, evidence,
  fixtures, configuration, pins and current AOT publications remain. An older Linux
  x64 directory lacking a verified result remains untouched. Complete source and
  final local evidence are retained in the filtering final archives.
  Cargo refused local cleanup because its cache marker was absent; after checking
  the exact workspace path, build-only contents, absence of links and inactive
  compilers, native PowerShell removed the cache. Both receipts are retained.

Parent review is complete and its disposition is recorded in the ADR, STATUS and
ADR index. Stop for the owner's commit/push and explicit next-ADR authorization.
ADR-0034, Visible-Only Operations on Filtered Data, is assigned for design first and
remains on hold. Compare transfer alternatives before choosing a contract; do not
assume skipping hidden rows in rectangular Copy/Paste is correct. Implementation
requires separate parent review and owner authorization. No commit/push, next
feature or parked history work is part of this acceptance.
