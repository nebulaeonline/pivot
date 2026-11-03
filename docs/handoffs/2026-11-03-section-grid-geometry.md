# ADR-0042 — Section Breaks and local Grid geometry implementation

Implemented, accepted and closed by parent implementation review, 2026-11-03.
The review accepts the supplied evidence without another validation cycle.
The prior Windows ARM64 cold-run
qualification remains unresolved, and ADR-0023 remains parked at G3.

Independent horizontal and vertical Section Breaks now support local column-width
and row-height profiles. Either boundary creates sections. Interactive cell
selection crossing either boundary closes over whole touched sections; formulas
and computational ranges retain their existing authority and meaning.

## Implementation

Rust owns the canonical bounded geometry map used by Desktop rendering, hit
testing, headers, scrolling, editor placement and selection. Geometry uses fixed
1/64-point coordinates. Complete interior-overlap validation includes unmaterialized
Blank cells; edge touching is allowed. Admission bounds profiles, runs, topology
tiles, temporary sweep work and retained roots without enumerating the declared
worksheet. Geometry-only publication advances workbook state without advancing
calculation revision. Authored-axis structural transforms retain stable identities.

Desktop supports local sizing, paired divider edits, explicit shared-seam changes,
preview, cancellation and reset. A preview does not publish workbook state.
Expanded destructive/bulk-changing operations require confirmation qualified to
the particular selection and state; cancellation and obsolete confirmation refuse
without an edit. Existing operation limits still apply after section closure.

Bound Window geometry is explicitly positional. Meaningful source schema/order
changes expose Review/Keep/Reset rather than silently carrying presentation with a
producer field or invalidating producer publication. Filter compaction that would
overlap cells suspends presentation and exposes explicit recovery. Current cells,
filter warnings and layout recovery controls publish in the same UI turn.

WB16 is the sole native workbook format and private session v16 carries geometry
capture/edit/selection requests. Earlier native formats are refused; no migration
reader is introduced. Hosted v1, CB5/CB6, canonical value types and calculation
semantics remain unchanged. Window native clipboard refusal remains explicit.
Geometry captures use the existing shared capture pool and bounded paging.

## Correctness and usability evidence

Rust and private-session regressions exercise overlap validation against independent
dense rectangle oracles, section closure, edge/gap hits, structural transforms,
retained roots/admission, hostile persisted geometry and failed atomic publication.
The shared virtual UI sequence constructs and revises the 36×8 invoice through
ordinary controls in two edit orders. It exercises resizing, adding breaks,
changing profiles, preview/reset, zoom, bound layout review, filter recovery,
one-boundary selection and expanded Clear confirmation acceptance/cancellation/staleness.
Leading-zero Strings remain exact. The same sequence runs in attended GUI suites.

Qualification exposed and corrected pointer selection races involving quick drag
release, edge scrolling and a gesture awaiting its own inline edit. Deferred
selection is cancelled by incompatible state changes. Clipboard test synchronization
uses a stable painted frame, one copy/paste action and bounded read-only verification;
product clipboard guards are preserved.

macOS ARM64 AOT exposed a production publication race: cells were current before
their warning/recovery controls. A deterministic regression holds deferred selection
resolution and observes both cells and controls. It fails on the old ordering and
passes with synchronous publication. All GUI targets were refreshed after that fix.
A later Windows ARM64 failure was in the stale-view clipboard fixture. The fixture
now establishes an older displayed frame explicitly and separates state assertions
from bounded clipboard readback. This latter amendment changes no product code.

## Qualification and source lineage

Full local CI passed 13/13, together with all-feature Rust tests and strict Clippy:
[local evidence](../../artifacts/adr0042-local/final-10/summary.json).
The six-target full headless baseline passed before the final UI publication fix:

| Target | Full headless passing matrix |
| --- | --- |
| Windows x64 | `20261103T100645Z-36a0e86a` |
| Linux x64 | `20261103T102113Z-8fbde0dd` |
| Windows ARM64, Linux ARM64 | `20261103T103502Z-7d4dd8db` |
| macOS ARM64, macOS x64 under Rosetta | `20261103T100725Z-538e907b` |

The publication fix then passed the complete virtual UI suite on all six native
targets, with three fresh iterations on each ARM64 target:
[updated native UI evidence](../../artifacts/adr0042-local/native-fixture-final-10/summary.json).
This is focused qualification, not a claim that all six full 13-gate pipelines were
rerun after the final UI change. The Rust/backend inputs were unchanged.

Corrected runtime snapshot A:
`a983aa351bd9646ab7f45d9c082a2d4b92f3ad5a45c1fa2a121207d53921e40b`.
Snapshot B:
`ae3591ebe1f850e45eec7a8ada6ae1b9476225b32931465a120812eeec920469`
changes only `InputAutomation.cs` and the then-current status document relative to A.
[Local amendment comparison](../../artifacts/adr0042-local/final-11/summary.json),
[Windows ARM64 virtual checks](../../artifacts/adr0042-local/native-fixture-final-11/summary.json)
and [Linux virtual checks](../../artifacts/adr0042-local/native-fixture-final-11-linux/summary.json)
pass. Windows ARM64 and Linux ARM64 each passed three fresh virtual iterations.

All twelve attended GUI suites pass, including full wrappers, four actual graphical
application reports per suite, expected execution architecture, zero final charges
and owned-process cleanup:

| Target | Snapshot | Managed collection | Native AOT collection |
| --- | --- | --- | --- |
| Windows x64 | A | `20261103T133803Z-direct-managed` | `20261103T133804Z-direct-aot` |
| Windows ARM64 | B | `20261103T134841Z-1bbe6407` | `20261103T135113Z-1ceea058` |
| Linux x64, Linux ARM64 | B | `20261103T142540Z-8f431c6b` | `20261103T142539Z-8861931b` |
| macOS ARM64, macOS x64 under Rosetta | A | `20261103T130442Z-3e4753eb` | `20261103T130441Z-8c04c834` |

Executed source manifests are verified against their qualified snapshot. Windows
x64 normal transfer failed after SSH authentication; an absolute-PowerShell
read-only export collected the original native reports, which passed the normal
graphical-report validators. No GUI rerun or receipt rewriting was used for transport.
The oversized user PATH remains a separate environment maintenance issue.

Original failed attempts remain failures. These include sparse-sort timing on
Windows x64, a Linux ARM64 divider timeout, earlier test-readiness failures and the
macOS publication failure. Isolated diagnostics and fresh passing retries are kept
separately. Previous Windows ARM64 cold structural timing failures are not resolved
by these later passes or by toolchain parity.

## Resource observations

Native headless checkpoints cover 1,024, 10,000 and 65,536 total source/consumer/leaf
nodes, 64 Windows including 62 empty Windows, multiple profiles and retained maps.
The source-qualified Rust checkpoint binaries are unchanged by the final UI fixes.
Process RSS is measured independently of Pivot's accounted resource charge:

| Target | Observed RSS at 65,536 nodes (bytes) | Measurement |
| --- | ---: | --- |
| Windows x64 | 308,875,264 | PeakWorkingSetSize |
| Linux x64 | 396,271,616 | VmHWM |
| Windows ARM64 | 347,844,608 | PeakWorkingSetSize |
| Linux ARM64 | 403,083,264 | VmHWM |
| macOS ARM64 | 388,939,776 | sampled ps; lower bound on peak |
| macOS x64 under Rosetta | 413,163,520 | sampled ps; lower bound on peak |

Per-target JSON under `artifacts/adr0042-local/resource-*-01.json` records timing,
accounted steady/retained/peak charges, binary hashes, identical backend input
proof and observer method. These are in-memory observations, not disk sizes or
guarantees of future performance. Local ROBPC's separate 65,536-node observation
used 311,246,848 bytes RSS and 380,924,318 bytes peak accounted charge.

## Size and retention

cloc 2.06 reports **107,251 code lines across 441 source files** on the canonical
checkout. Tests/examples are subsets, not additions to the total. Generated code,
dependencies and caches are excluded. See the
[per-project report](../../artifacts/loc/ADR-0042/20261103T142631818231Z/report.md).

Completed evidence and failures are retained under `artifacts/validation-matrix`;
verified ZIP archives and per-file manifests are under
`artifacts/archives/validation-matrix`. Archives are verified before native
disposable build caches are removed. Reports, authored source and published AOT
products are preserved. Per-target cleanup receipts and native free-space readings
are under `artifacts/adr0042-local/closed-*-cache-cleanup.json`; the pruning ledger
is `artifacts/pruned-build-products.json`. The final native cleanup batch removed
64.1 GiB of logical disposable cache contents across the six targets; filesystem
free-space changes are recorded separately and need not equal logical sizes.
No WSL execution was used this milestone.

## Review boundary

The review accepts geometry authority, overlap/admission behavior, interactive
invoice workflows, expanded-operation confirmation, positional bound presentation
and source-qualified platform evidence. It accepts the two final GUI snapshots,
the full-headless/focused-UI distinction and the disclosed platform/resource
qualifications. Automated editing sequences qualify the specified workflows;
they do not establish convenience for every possible human-authored layout.
No further validation, benchmark or evidence reconciliation is required. No next
ADR, text conversion, placement/page/PDF engine, SQL provider, implicit write-back,
history integration or licensing change is included. Commit and push remain with
the owner. No further milestone work is authorized by this acceptance.
