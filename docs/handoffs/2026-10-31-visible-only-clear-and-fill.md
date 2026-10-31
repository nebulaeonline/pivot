# ADR-0035 — visible-only Clear and Fill implementation

2026-10-31. Accepted and closed by parent implementation review. Clear-first and
Fill-second internal slices form one accepted milestone.

The parent accepts canonical no-ops, immutable Fill pairs/sources and combined atomic
content/style publication, together with graph-free formatting, hidden-row integrity,
exact review/cancellation and unchanged ordinary operations. The dedicated Clear seam,
property-map work accounting and absent-anchor batch-reservation correction are accepted
without increased limits. Private v13 is accepted; WB12, hosted v1 and CB5/CB6 remain
unchanged. Submitted six-target evidence, corrected Windows ARM64 result, selected
runtime-source qualification, supplementary test-only coverage and Rosetta designation
are accepted. Preserve observed-core-maxima and charged-memory qualifications, original
failures/recoveries, verified archives and diagnostic LoC. No additional validation,
reconciliation packet or Native AOT rerun is required.

Edit now offers Clear Visible Contents, Clear Visible Formatting (Built-in Defaults),
Clear Visible Contents and Formatting, and four-direction Fill Visible Cells. Ordinary
Clear/Delete, Fill, shortcuts and full-rectangle confirmations retain their behavior.
Clear addresses at most 4,096 visible logical slots including Blank/absent positions;
Fill at most 256 including source edges. Oversized operations refuse before staging,
with an actionable bound. No truncation, auto-chunking or budget increase is introduced.

## Three mandatory clarifications

**Canonical no-ops.** Clear Contents sets existing definitions to Blank, retaining
positioned identities and Blank anchors; absent cells stay absent. Formatting-only
Clear is graph-free and preserves the graph revision. Formatting Clear assigns all
ten explicit built-in defaults at every addressed visible position, overwriting
individual exceptions instead of inheriting worksheet defaults. Explicit-default
assignments are real canonical changes even if their appearance matches inheritance.
No-op detection compares definitions, identity state and explicit property assignments;
an exact no-op retains revision, dirty/checkpoint and graph/selection identities.

**Immutable Fill plan.** Down/Up uses the first/last visible selected row; Right/Left
uses each visible row's left/right selected edge. Hidden geometric boundaries cannot
be sources. All pairs are fixed before staging, and every source is read from one
immutable capture. Trusted duplication preserves exact types, leading-zero Strings,
styles, rejected drafts, fault provenance and structured bindings, translates by
actual coordinates and retains candidate-cycle validation. Filling B5 containing
=A2 down to B12 produces =A9. No dense-row translation, multi-row repetition,
contiguous-run restart or cross-record reference rewiring occurs.

**Combined publication.** Clear Both stages content and property-map resets on one
private candidate and publishes once. Fill uses canonical atomic content/style
installation with fixed pairs. There are no independently committed per-cell/per-row
or APPLY/FORMAT sequences. Failure, cancellation and admission refusal leave relevant
document and identity state unchanged. Filter refresh follows complete publication;
it cannot shorten a write set after an earlier destination becomes hidden.

No direct content/style writes target hidden rows. Canonical binding may still demand
hidden Blank reference anchors, and hidden dependents may recalculate. Qualification
requires a current no-filter/applied view; pending, failed and NeedsReview states,
projected records and active editors/drafts block the commands. Selection remains
an ordinary rectangle, with temporary actual-row metadata rather than computational
multi-area identities. See [implementation semantics](../architecture/visible-edit.md).

## Review, protocol and core integration

Every invocation reviews all actual source/destination row runs, columns, slot counts,
skipped hidden rows, Clear mode/Fill direction and applicable limit, including zero
skips and degenerate no-ops. Cancel is the default. One-use tickets bind exact command
bytes, selection, expected revision/computational state and qualified map; the client
also checks interaction state. Changed state refuses without replay. Matching-ticket
Cancel releases abandoned review state, including cancellation during preparation.
No clipboard access is required. Normal filter relocation applies when an active
cell becomes hidden after the committed operation.

Private v13 adds Prepare/Clear/Fill/Cancel operations 54–57 and typed ClearMode field55.
WB12, hosted v1, CB5 and CB6 are unchanged. No saved plan, obsolete reader or migration
is needed. Ordinary APPLY retains its 256-edit limit: a dedicated, Clear-only bounded
transaction seam admits the approved larger Clear plan.

Sparse resets coalesce adjacent selected rows, subtract only addressed spans and
normalize each property map once under one accumulated work meter. Scratch accounting
uses addressed slots rather than only row-run count, covering wide selections as well
as tall ones. Existing style region, byte/work, graph, identity, workbook, frame,
transient and runtime admission remain in force. A style-only canonical change does
not install an empty graph batch.

## Correction and original evidence

A new sparse short-formula regression exposed an existing shared batch reservation
underestimate when both a destination and referenced Blank target were absent. An
anchor requires add, Blank definition and placement work; counting a syntactic token
as one operation was insufficient. The trusted-formula estimate now conservatively
accounts for that work within the same internal batch limits. No capacity threshold
was raised. Initial local CI and the first Windows ARM64 native snapshot exposed the
failure; corrected local/all-feature tests and a fresh Windows ARM64 run pass.

Original failures remain separate: the initial all-feature run also hit sandbox
Access Denied on local daemon connections; its approved unrestricted retry passed.
Initial compile/test wiring failures and their corrections remain in the logs.

## Validation and source qualification

| Target | Headless | Managed GUI | Native AOT GUI |
| --- | --- | --- | --- |
| Windows x64 | Passed | Passed | Passed |
| Linux x64 | Passed | Passed | Passed |
| Windows ARM64 | Passed | Passed | Passed |
| Linux ARM64 | Passed | Passed | Passed |
| macOS ARM64 | Passed | Passed | Passed |
| macOS x64 (Rosetta) | Passed | Passed | Passed |

All six selected headless runs passed 13-step CI. Local 13-step CI passed; subsequent
focused checks and 497 all-feature Rust tests cover later refinements, with strict
all-target/all-feature Clippy passing. Independent row-mask/all-direction oracles,
Clear modes/default inheritance, stable identities/Blank demand, graph-free formatting,
wide/full-height boundary plans, cycles, cancellation and late style-quota rollback
pass. Real IPC checks cover exact review, one-use/cancel/stale state, 4,096/256 admission
and post-filter membership changes. In-memory routed-menu/dialog checks are supplementary.

The 12 attended GUI suites produced 48 successful application reports, 48 zero final
charges and no owned survivors. Actual routed Clear modes, four Fill directions,
exact row/source review, decline, complete formatting reset, hidden preservation and
degenerate review passed alongside existing clipboard/formatting/structure/filtering
regressions. The owner launched each suite with normal graphical-terminal consent.

[Final receipt selection and assertions](../../artifacts/visible-edit-final-matrix.json).
Windows x64 GUI collections: `20261031T213902Z-df27714f` managed and
`20261031T213913Z-165d4a59` AOT. Linux x64: `20261031T214254Z-60da48e2` managed and
`20261031T214253Z-fda6fcf6` AOT. Remaining final GUI collections:
`20261031T221848Z-9eafd00f` managed and `20261031T221846Z-ca6d7442` AOT.

Headless Windows/Linux x64 use `20261031T212513Z-122298df`; Linux ARM64/macOS use
`20261031T211908Z-ee456aa7`; successful Windows ARM64 replacement uses
`20261031T214255Z-a839c567`. The earlier failed Windows ARM64 result in the mixed
matrix is not relabeled. [Runtime source hashes](../../artifacts/visible-edit-source-qualification.json)
show the six selected runtime files match the current source across all six selected
headless snapshots, including corrected reservation, scratch accounting and SDK cancel
handling. This does not claim every file in every snapshot is byte-identical. Later
supplementary local IPC test-only oracles add membership-change coverage without
changing runtime code. macOS x64 is explicitly Rosetta coverage.

## Resource observations

Four changed-command and four exact-no-op samples per corpus per target: three Clear
modes at 4,096 slots and four Fill directions at 256 slots including sources. Base is
16,384 positioned cells; added formula positions make the declared totals 20,480 for
Clear and 16,640 for Fill. Sparse near-full-height gaps and four retained captures are
included. These are observed core-command maxima, not percentiles or OS/dialog latency.
Charged memory is neither RSS nor disk. Every new corpus satisfies existing 1-second,
256-MiB steady and 512-MiB peak objectives and finishes at zero charge. It is not a claim
about arbitrary whole-workbook or lifetime-identity-exhaustion memory behavior.

| Target | Clear max ms | Fill max ms | No-op max ms | Steady max MiB | Peak max MiB |
| --- | ---: | ---: | ---: | ---: | ---: |
| Windows x64 | 58.58 | 4.47 | 9.84 | 66.90 | 91.29 |
| Linux x64 | 61.89 | 6.82 | 9.58 | 66.89 | 91.28 |
| Windows ARM64 | 34.66 | 2.39 | 5.20 | 78.94 | 91.33 |
| Linux ARM64 | 58.21 | 6.01 | 7.16 | 66.92 | 91.32 |
| macOS ARM64 | 24.59 | 1.66 | 5.00 | 66.91 | 91.31 |
| macOS x64 (Rosetta) | 42.55 | 1.94 | 7.06 | 66.90 | 91.30 |

[Raw resource receipts](../../artifacts/visible-edit-native-headless-summary.json)
also retain separate single-observation SDK/session preparation and total round-trip
measurements with automatic test confirmation; those are not core-only or physical
GUI timings. Existing filter-refresh checks remain in the full CI suite.

## Code size

Canonical-checkout cloc 2.06 reports **88,600 source code lines across 383 files**,
up **1,420** from ADR-0034's 87,180. Counts include tests/examples and exclude generated,
dependency and cache output. [Raw report and hashed measurement inputs](../../artifacts/loc/ADR-0035/20261031T221917287937Z/report.json)
record language counts and separate documentation/configuration totals at measurement
time, before this closing handoff. Physical line count is diagnostic, not a reduction
objective. The table's test/example column is a subset of code, not an added total.

| Project | Code | Comments | Blank | Test/example code* | Languages (code) | Change |
| --- | ---: | ---: | ---: | ---: | --- | ---: |
| ApplicationSession | 1,299 | 15 | 32 | 1,299 | C# 1,144, Python 155 | +47 |
| DecimalParity | 182 | 7 | 17 | 182 | C# 35, Python 147 | +0 |
| LegacyProtocol | 161 | 5 | 2 | 161 | C# 43, Protocol Buffers 118 | +0 |
| Pivot.Avalonia | 419 | 3 | 2 | 0 | C# 419 | +0 |
| Pivot.Client | 521 | 25 | 17 | 0 | C# 521 | +25 |
| Pivot.Desktop | 2,889 | 43 | 41 | 1,090 | C# 2,761, XML 128 | +95 |
| PolicyProbe | 1,619 | 54 | 127 | 1,619 | C# 1, PowerShell 38, Python 1,580 | +0 |
| ProtocolParity | 81 | 2 | 8 | 81 | C# 55, Python 26 | +0 |
| pivot-cli | 1,413 | 12 | 3 | 280 | Rust 1,413 | +0 |
| pivot-core | 16,291 | 213 | 167 | 3,014 | Rust 16,291 | +0 |
| pivot-host | 8,431 | 69 | 95 | 4,165 | Rust 8,431 | +0 |
| pivot-protocol | 2,636 | 47 | 29 | 1,023 | Protocol Buffers 565, Rust 2,071 | +2 |
| pivot-session | 3,549 | 17 | 11 | 382 | Rust 3,549 | +265 |
| pivot-storage | 5,134 | 52 | 72 | 1,952 | Rust 5,134 | +0 |
| pivot-transport | 897 | 26 | 13 | 369 | Rust 897 | +0 |
| pivot-types | 503 | 14 | 11 | 172 | Rust 503 | +0 |
| pivot-workbook | 25,354 | 179 | 121 | 11,658 | Rust 25,354 | +773 |
| pivot-workbook-document | 6,901 | 49 | 27 | 4,260 | Rust 6,901 | +213 |
| pivotd | 1,134 | 20 | 7 | 899 | Rust 1,134 | +0 |
| Other source | 214 | 11 | 4 | 0 | Protocol Buffers 214 | +0 |
| Scripts/tooling | 7,673 | 373 | 671 | 0 | Bourne Shell 126, C 166, DOS Batch 12, PowerShell 438, Python 6,931 | +0 |
| Standalone tests | 1,299 | 87 | 191 | 1,299 | Python 1,299 | +0 |


## Retention and next step

Each completed native batch is archived and SHA-256 verified before compiler caches
and superseded products are pruned. Sources, original success/failure reports, fixtures,
configuration and current AOT publications remain. Linux x64's older unverified result
directory remains untouched. Receipt archives and per-target free-space/cleanup events
are recorded in `artifacts/archives/index.jsonl` and `artifacts/pruned-build-products.json`.
Logical deleted bytes and free-space change differ; host/guest recovery is not additive.
Final local evidence and source snapshots are archived separately before disposable
ROBPC compiler-cache removal.

ADR-0035 is accepted and closed. Stop for owner git commit/push and explicit next-work
authorization. No commit/push or next-ADR design is performed here. ADR-0023 remains
parked at G3.

ADR-0036: Exact, Conservation-Preserving Allocation is assigned next for design and
parent review. Its starting document is [docs/proposals/allocate.md](../proposals/allocate.md),
which is the existing proposal referenced as an architecture document in the review;
retain its current location. Begin with equal-weight scalar allocation and evaluate
the illustrative `ALLOCATE(total, count, index, decimal_places)` interface against
alternatives. Precision is required, with no implicit two-decimal default. For example,
10.005 at precision 2 must error instead of changing the obligation. A complete,
consistently specified recipient set must conserve the exact total; independent calls
cannot prove completeness or uniqueness of recipient indices.

The future design must specify negatives/signed zero, deterministic residual ordering,
invalid arguments, checked Decimal96/integral-unit overflow, admission, cross-platform
behavior, scalar versus multi-output tradeoffs, compatibility without assumed version
changes and independent conservation/representability/parity tests. Preserve ordinary
division, ROUND, graph/reference and Copy/Fill/sort/persistence behavior. Weighted
allocations, dynamic arrays, rotation, persistent group state, financial reporting and
automatic formula repair remain outside the initial scope. This is a recorded next
assignment, not authorization to begin its design before the owner's normal cadence.
