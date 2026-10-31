# ADR-0034 — visible-only operations implementation

2026-10-31. Accepted and closed by parent implementation review.

The parent accepts the three mandatory invariants, CB5/CB6 interoperability, private
v12 with unchanged WB12/hosted v1, atomic mapped writes and both sparse formatting
corrections without increased limits. Submitted six-target evidence, source
qualifications, Rosetta coverage, observed maxima, archival and per-project LoC are
accepted. No additional validation, reconciliation packet or AOT rerun is required.
Original failures and successful corrections remain separate.

Copy Visible Cells and Paste into Visible Rows are independent Edit commands. Ordinary
Copy/Paste and shortcuts retain full-rectangle behavior. Copy records actual ascending
source rows in bounded CB6; Paste maps slots to an exact qualified destination row
vector. Relative formula translation uses actual source/destination coordinates,
including cross-record references. Strings retain exact content and leading zeros.
Resolved formatting, including Blank cells, moves with the slots in one atomic write.

## Parent invariants

Preferred recognized native data is authoritative. Malformed/unsupported CB6 refuses
without falling back to CB5 or TSV. OS reads pin the selected native kind across
bounded reads; valid native-only Strings containing tabs/newlines are copied without
lossy text publication. Genuine TSV-only input remains external with literal formula
addresses. CB5 input can target visible rows; ordinary Paste can pack CB6 contiguously.

No content or style write directly targets hidden rows. Canonical formula resolution
may still demand a hidden Blank reference anchor. Every visible paste reviews actual
ordered worksheet row runs, columns, slot count and skipped rows, including zero skips.
The one-use confirmation binds exact immutable payload, destination plan, revision,
computational state and filter generation. Clipboard/interaction changes cancel;
there is no automatic replay. Feedback outlines actual cells. The active anchor stays
when visible; normal filter relocation applies if the edit hides it.

Single-anchor Paste expands to the next required visible rows; rectangular selection
requires exact visible count and width. Shape mismatch, stale maps, pending/failed
qualification, unavailable clipboard bytes and exhausted quotas refuse atomically.
Source and destination visibility are independent. No new multi-area selection,
visible Clear/Fill, Cut/Move, history or filter redesign is introduced.

## Representation and formatting corrections

Private session protocol advances to v12; hosted v1 and WB12 are unchanged. CB6 adds
bounded actual source-row origins; ordinary CB5 remains supported. Existing 256-slot,
256-KiB combined payload and 8-MiB operation admission remain in force; worksheet,
transport and runtime limits are unchanged. See [implementation semantics](../architecture/visible-clipboard.md).

The full-height sparse formatting corpus exposed two existing style-storage issues.
Shared-map accounting subtracted the shared handle's zero footprint rather than old
map storage, accumulating historical byte totals. It now subtracts actual vector
storage. Per-cell map rebuilding also exhausted the existing work budget for 256
gapped slots; property maps now stage changed points in a batch, split/coalesce once,
and check the same quotas before publication. Repeated trusted style targets retain
last-write-wins; visible destination rows remain strictly unique. All-feature
compatibility tests caught the initial duplicate-target rejection; the correction
passes existing Fill/formatting/clipboard compatibility checks. No limit was raised.

## Validation

All six targets passed 13-step headless CI and attended managed plus Native AOT GUI:
Windows x64/ARM64, Linux x64/ARM64, macOS ARM64 and macOS x64 under Rosetta.
The 12 graphical suites produced 48 successful application reports, 48 zero final
charges and no owned survivors. Each suite validates CB6 in the existing separate
clipboard-save peer after the source session closes; preferred-format and native-only
checks also pass. Owner launched graphical checks with local consent.

Selected immutable receipts and assertions: [final matrix](../../artifacts/visible-final-matrix.json).
Windows x64 managed/AOT collections are `20261031T201610Z-8b10d600` and
`20261031T201607Z-8d2958e9`; remaining final collections are
`20261031T204253Z-bce0f2f7` and `20261031T204252Z-67a1f02f`.
Headless selection is recorded per RID in the matrix. Final local CI passed 13/13,
all-feature Rust tests passed 488, and strict all-target/all-feature Clippy passed.
Core mask/origin/style/atomicity tests, real IPC tests and in-memory dialog probes
supplement actual platform execution.

Initial Windows x64 and Windows ARM64 headless snapshots precede the shared-style
last-write-wins compatibility correction. Final local CI, all-feature tests and all
final graphical suites include that correction. Original failed evidence remains
separate. The first Windows x64 GUI pilot refused its last paste because the test
issued it before a stable painted worksheet frame. The corrected test waits for
paint and stable interaction using read-only checks before its single action.
Product clipboard guards were unchanged; no copy/paste/edit is replayed.

## Resource observations

Four observed core-copy/atomic-commit samples for each values, formulas and styles
corpus per target; these are maxima, not percentiles, OS clipboard/dialog timings
or complete session round trips. Base population is 16,384 positioned cells (16,832
for formulas), with 256 slots across full-height gaps and four retained captures.
Memory is runtime charged, not RSS or disk. All samples satisfy existing 1-second,
256-MiB steady and 512-MiB peak objectives and finish with zero charge.

| Target | Copy max ms | Commit max ms | Steady max MiB | Peak max MiB |
| --- | ---: | ---: | ---: | ---: |
| Windows x64 | 1.77 | 12.02 | 29.09 | 40.76 |
| Linux x64 | 1.68 | 10.37 | 29.09 | 40.76 |
| Windows ARM64 | 0.88 | 6.88 | 29.07 | 40.74 |
| Linux ARM64 | 1.64 | 7.29 | 29.07 | 40.73 |
| macOS ARM64 | 0.64 | 5.55 | 29.07 | 40.74 |
| macOS x64 (Rosetta) | 1.26 | 10.12 | 29.09 | 40.76 |

[Raw selected resource receipts](../../artifacts/visible-native-headless-summary.json).

## Code size

Canonical-checkout cloc 2.06 reports **87,180 source code lines across 375 files**,
up **1,528 lines** from ADR-0033's 85,652. This includes tests/examples and is a
diagnostic size measure. Generated output, dependencies and build caches are excluded.
The [raw report and hashed input manifest](../../artifacts/loc/ADR-0034/20261031T204338521809Z/report.json)
retain language counts and documentation/configuration totals.

| Project | Code | Comments | Blank | Test/example code* | Languages (code) | Change |
| --- | ---: | ---: | ---: | ---: | --- | ---: |
| ApplicationSession | 1,252 | 14 | 32 | 1,252 | C# 1,099, Python 153 | +35 |
| DecimalParity | 182 | 7 | 17 | 182 | C# 35, Python 147 | +0 |
| LegacyProtocol | 161 | 5 | 2 | 161 | C# 43, Protocol Buffers 118 | +0 |
| Pivot.Avalonia | 419 | 3 | 2 | 0 | C# 419 | +16 |
| Pivot.Client | 496 | 22 | 17 | 0 | C# 496 | +24 |
| Pivot.Desktop | 2,794 | 42 | 41 | 1,053 | C# 2,677, XML 117 | +95 |
| PolicyProbe | 1,619 | 54 | 127 | 1,619 | C# 1, PowerShell 38, Python 1,580 | +0 |
| ProtocolParity | 81 | 2 | 8 | 81 | C# 55, Python 26 | +0 |
| pivot-cli | 1,413 | 12 | 3 | 280 | Rust 1,413 | +0 |
| pivot-core | 16,291 | 213 | 167 | 3,014 | Rust 16,291 | +0 |
| pivot-host | 8,431 | 69 | 95 | 4,165 | Rust 8,431 | +0 |
| pivot-protocol | 2,634 | 47 | 29 | 1,023 | Protocol Buffers 563, Rust 2,071 | +7 |
| pivot-session | 3,284 | 16 | 9 | 310 | Rust 3,284 | +439 |
| pivot-storage | 5,134 | 52 | 72 | 1,952 | Rust 5,134 | +0 |
| pivot-transport | 897 | 26 | 13 | 369 | Rust 897 | +0 |
| pivot-types | 503 | 14 | 11 | 172 | Rust 503 | +0 |
| pivot-workbook | 24,581 | 173 | 116 | 11,252 | Rust 24,581 | +750 |
| pivot-workbook-document | 6,688 | 45 | 27 | 4,047 | Rust 6,688 | +162 |
| pivotd | 1,134 | 20 | 7 | 899 | Rust 1,134 | +0 |
| Other source | 214 | 11 | 4 | 0 | Protocol Buffers 214 | +0 |
| Scripts/tooling | 7,673 | 373 | 671 | 0 | Bourne Shell 126, C 166, DOS Batch 12, PowerShell 438, Python 6,931 | +0 |
| Standalone tests | 1,299 | 87 | 191 | 1,299 | Python 1,299 | +0 |


## Retention and next step

Completed native batches are archived with per-file SHA-256 verification before
cache pruning; receipt archives and events are recorded in `artifacts/archives/index.jsonl`
and `artifacts/pruned-build-products.json`. Sources, original success/failure reports,
fixtures, configuration and current AOT publications remain. Linux x64's unverified
older result directory is preserved. Host/guest disk changes are not additive.
Final local evidence and source snapshots are retained separately from disposable
compiler caches. Cleanup receipts qualify logical bytes separately from free space.

ADR-0034 is accepted and closed. ADR-0035: Visible-Only Clear and Fill is assigned
next, beginning with design and parent review. Its design must resolve Clear content/
format/both semantics, identity and Blank-anchor treatment, Fill directions/source
edges/repetition and actual-coordinate formula translation, temporary qualified row
plans, hidden-row integrity, atomicity, exact-plan one-use review, bounded admission
and independent/native validation. Compare a combined milestone with separate Clear
and Fill implementation slices. Arbitrary multi-area selection, visible sorting,
filtered aggregates, tables and history remain outside scope.
Stop for owner git commit/push and explicit authorization before starting that design.
ADR-0023 remains parked at G3. No commit/push or next-ADR design is performed here.
