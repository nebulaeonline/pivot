# ADR-0032 — worksheet sorting implementation

2026-10-31. Accepted and closed by parent implementation review.

The parent accepts the implementation, sparse-index memory correction, confirmed
expanded-range submission fix, source-qualified six-target evidence and unchanged
resource objectives. Rosetta execution and observed timing maxima remain explicitly
qualified. Original failures and successful retries stay separate. End-of-batch
archive verification and cache cleanup are accepted. No further validation,
reconciliation packet or AOT rerun is required.

Pivot sorts row segments within an explicitly reviewed rectangle by ordered column
keys. The dialog displays the final range, warns about partial records and offers
explicit inclusion of adjacent populated columns. Header exclusion starts off;
Move formatting with records starts on. Key priorities/directions, cancellation,
both formatting policies, active-coordinate retention and save/reopen are exercised
through the actual desktop dialog.

CellIds, graph identities, scalar bindings, authored source and fault provenance
travel with records, including `$`-locked scalar references. Rectangular range
geometry and SelectionIds remain fixed; membership follows current occupants.
No Copy/Fill translation, silent rebinding, identity allocation or retirement occurs.
One immutable calculation supplies snapshot keys; recalculation does not re-sort.
Existing editable-cycle diagnostics apply if placement creates a range cycle.

Stable successful-value order is Decimal96, String, Boolean ascending, reversed
descending. Legitimate cell errors then Blank remain last in either direction.
Errors tie at a key; subsequent keys may distinguish records. Strings retain exact
content, including leading zeros, with case-sensitive ordinal Unicode comparison.
Operational calculation failures, stale requests, cancellation and exhausted work
or memory admission abort without a partial publication. Identity placements and
explicit property assignments determine no-ops, not equal displayed values.

Move formatting permutes explicit sparse property assignments and inheritance gaps,
preserving outside/header formatting and explicit defaults. Constant column styles
remain constant. Keep formatting retains worksheet-position assignments. Existing
fragment/byte limits refuse atomically without switching policies. Workbook capacity,
the 33,554,432-unit work ceiling, WB11 and CB5 remain unchanged. Private session v10
adds Sort and SortBounds; hosted protocol v1 remains unchanged.

## Implementation and resource correction

`pivot-workbook::sorting` builds one private candidate and publishes once. Sparse key
positions, row offsets and bounded stable merge sorting avoid a worksheet-sized key
matrix. Comparisons, examined String bytes and permutation copies consume the shared
work budget. Key values remain borrowed from the charged captured calculation result.
The formatting store transforms explicit assignments without resolving defaults.

The tall 65,536-by-1 checkpoint initially exceeded the unchanged 256/512 MiB memory
objectives: about 298 MiB steady and 536 MiB peak. Investigation found redundant
shared row wrappers and single-entry reverse-location trees in the canonical
selection index. Rows now embed their persistent column index; ordinary reverse
placements use a singleton that grows/collapses to/from the existing persistent
multi-location index for aliases. Two-dimensional indexing, old roots, range
membership, ordering and alias semantics remain intact. Focused regressions cover
alias growth/collapse, old captures, cancellation/quota failure and final charges.
No history work was resumed and no resource threshold was raised.

## Validation

All six targets passed headless CI **13/13** and managed plus Native AOT graphical
acceptance. macOS x64 is **Rosetta**, not native Intel hardware. The final selection
contains **48 successful application reports**, all with zero final runtime charges
and no surviving owned processes.

| Target | Headless | Managed GUI | Native AOT GUI | Execution |
| --- | --- | --- | --- | --- |
| Windows x64 | 13/13 | Passed | Passed | Native |
| Linux x64 | 13/13 | Passed | Passed | Native |
| Windows ARM64 | 13/13 | Passed | Passed | Native |
| Linux ARM64 | 13/13 | Passed | Passed | Native |
| macOS ARM64 | 13/13 | Passed | Passed | Native |
| macOS x64 | 13/13 | Passed | Passed | Rosetta |

Native headless runs preceded the C# dialog corrections; Rust implementation did
not change afterward. The final dialog source passed another local CI **13/13**,
an in-memory expanded-range regression and actual managed/AOT suites on all six
targets. All-feature Rust regression passed **471 tests**; all-target/all-feature
Clippy passed with warnings denied.

Regression coverage includes stable multi-key typed ordering, error keys, exact
Strings, Unicode/long-prefix work exhaustion, midflight cancellation, identity-aware
no-ops, old roots, scalar versus fixed ranges, cross-sheet/absolute references,
canonical cycle diagnostics, graph-free/style-only blanks, fragmentation refusal,
both formatting policies and WB11/CB5 persistence. Twenty-four generated small-grid
cases compare all ten style properties and stable key tuples against independent
dense oracles. Every native headless run completed nine staged sorting corpora,
including 1,024/10,000/65,536 numeric/style checkpoints, a ten-column case and tall
sparse selections.

The figures below are maximum observations over four permutations at 65,536, **not
percentiles**. Memory is charged runtime memory, **not disk size or process RSS**.
Steady includes four retained old roots; peak includes save/open of a second book.
The style corpus has 65,536 property regions and a sparse key population.

| Target | Numeric max, ms | Tall sparse max, ms | Tall steady, MiB | Tall peak, MiB | Style max, ms |
| --- | ---: | ---: | ---: | ---: | ---: |
| Windows x64 | 791.05 | 963.70 | 242.88 | 481.04 | 31.58 |
| Linux x64 | 721.61 | 849.18 | 242.88 | 481.04 | 32.12 |
| Windows ARM64 | 396.47 | 491.48 | 242.88 | 481.04 | 33.16 |
| Linux ARM64 | 413.62 | 483.37 | 242.88 | 481.04 | 23.60 |
| macOS ARM64 | 431.88 | 615.76 | 242.88 | 481.04 | 29.54 |
| macOS x64, Rosetta | 681.07 | 783.92 | 242.88 | 481.04 | 33.28 |

Original failures remain unchanged. The initial concurrent headless batch failed
existing structural timing gates on Windows x64/ARM64 and Linux ARM64 before their
sorting cases. Isolated complete retries passed without changing thresholds;
Windows ARM64's retry also added diagnostic timing output to the existing assertion.
The original local lint failure was corrected by moving a test module, and the
original over-budget tall checkpoint remains recorded separately from its recovery.

The first Windows x64 GUI pilot exposed a delayed TextChanged event invalidating
the review started by the same range edit. Synchronous TextChanging invalidation
fixed that race. The next pilot exposed constructor-parameter captures causing
Sort to submit the original rectangle despite showing the expanded range. Sort and
Add key now use the reviewed fields; column handlers retain their own range origin.
An in-memory reproduction confirmed the defect and its correction; the graphical
regression checks the delayed notification, expanded column key, actual moved CellId,
exact leading-zero text and focus separately. The acceptance helper also stopped
refreshing away the product's completion status before asserting it.

## Evidence and next step

- `artifacts/sorting-final-matrix.json` selects final successful runs without
  rewriting the original failures; `sorting-native-headless-summary.json` contains
  all nine checkpoints per target with their source report paths.
- Native headless matrices: initial `20261031T104854Z-6d671abf`, isolated Windows x64
  `20261031T105700Z-0a169d2a`, and serial ARM retries `20261031T111013Z-16811e03`.
- Final GUI collections: Windows x64 managed `20261031T114403Z-c14eba54`, AOT
  `20261031T114404Z-29841919`; remaining five targets managed
  `20261031T120631Z-fc268090`, AOT `20261031T120634Z-37be926b`.
- Final local CI: `artifacts/sorting-final-reviewed-ci/20261031T120652956878Z/summary.json`.
  Rust/lint: `sorting-all-features-final.log`, `sorting-all-features-clippy-final.log`.
  In-memory diagnostic: `sorting-main-window-headless-probe-verified.log`.
- End-of-batch cleanup records are in `artifacts/pruned-build-products.json` and
  `artifacts/archives/index.jsonl`, with SHA-256-verified evidence/receipt archives.
  Completed native caches are removed; source snapshots, reports, fixtures and current
  publications remain. The local Cargo cache is also cleared after final CI.

Parent review is complete and its disposition is recorded in ADR-0032, STATUS and
the ADR index. Stop for the owner's commit/push and explicit next go-ahead.
ADR-0033, Worksheet Filtering, is assigned as the next design milestone, on hold.
Filtering must change row visibility without deleting, rearranging or rebinding
cells; the design must resolve persistence, typed criteria, coordinate/viewport
mapping, interactions, invalidation, bounded resources and visible filter controls.
Return that design for parent review before implementation. No commit/push, filtering
design or feature work, or history work is started by this acceptance.
