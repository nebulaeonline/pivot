# Structural Mutation Feasibility — Research Handoff

Date: 2026-10-19
Disposition: structural sharing is promising for ordinary formula edits; **do not
replace production structures in this slice**. Parent review pending.

## What changed

The parent accepted assessment B and required a structural-edit experiment rather
than adopting the input treap. This slice compares representations and implements
a separate deterministic paged/radix candidate behind `scaling-experiment`.

- [Architecture, alternatives, limits and retention policy](../architecture/structural-mutation-feasibility.md)
- [Matched benchmark specification and reproduction](../performance/structural-experiment.md)
- [Core prototype](../../crates/pivot-core/src/model/structural.rs)
- [Fixed-projection workbook adapter](../../crates/pivot-workbook/src/structural.rs)
- [Core invariant tests](../../crates/pivot-core/tests/structural.rs)
- [Workbook atomicity and numeric-source tests](../../crates/pivot-workbook/tests/structural.rs)
- [Release harness](../../crates/pivot-workbook/examples/structural.rs)
- [Changing-input control](../../crates/pivot-workbook/examples/structural-inputs.rs)
- [Process-isolated runner](../../scripts/structural.py)
- [Analysis script](../../scripts/structural_analyze.py)

Core remains general purpose. No new dependency or formula evaluator: the accepted
Pratt parser/binder, transfer stack checker and postfix arithmetic executor are
reused. Two helper visibilities widen within core only; default execution stays
unchanged. Production caps, codecs and frozen fixtures are untouched.

Topology, input values and workbook metadata have separate immutable roots under
one aggregate version. Existing formula replacement updates affected definition
and edge records. Captures pin roots without rebuilding dependency order or cell
maps. Old S retains its exact graph, values and projection while T changes.

## Windows matched Release evidence

Reference hardware remains Ryzen 9 3950X, 16 cores/32 logical CPUs, about 64 GiB RAM.
Windows 11 build 26300; Rust nightly b6eeb93c9. Default Release opt-level 3, no custom
LTO/target CPU, affinity or frequency lock. No hermetic background-load control;
regression/build work can run during sweeps, but comparative benchmarks run
sequentially. Separate processes have fresh UUIDs and equal logical definitions.

[Windows sweep](../../artifacts/structural-windows/20261019T110425Z/summary.json),
[all distributions](../../artifacts/structural-windows/20261019T110425Z/analysis.md),
[normalized data](../../artifacts/structural-windows/20261019T110425Z/metrics.json).

100K cells, p95 milliseconds, 101 measured samples after three warmups:

| Shape | Baseline formula pipeline | Paged formula pipeline |
| --- | --- | --- |
| Independent, 128 existing tail formulas | 147.7303 | 0.0130 |
| Chain | 137.2368 | 0.0136 |
| Mixed | 166.5152 | 0.0160 |
| Fanout | 244.2019 | 0.6481 |

Both timers include source preparation, binding, staging, edits, all validation,
installation, capture and selected changed-output evaluation. The baseline's
deferred validation is included. This is the normalized semantic boundary missing
from the previous research. Different formula nodes change throughout the samples;
edges alternate between A1 dependencies and constants. It is not repeated hot-key
formula replacement. Long dependency closures are not representative of every row.

Independent formula pipeline p95 at 1K/10K/100K: baseline 0.7251 / 11.5845 /
147.7303 ms, paged 0.0088 / 0.0118 / 0.0130 ms. Fanout still exposes the cost of
copying a large reverse set. The narrower edit is fast here, but its degree-based
cost is not a constant-time guarantee or proof that the edge representation is final.

At 100K, independent input edit p95 is 0.0044 ms, unchanged capture 0.0001 ms
(timer-limited, individually measured), and input/capture/1,000-formula chain
0.2897 ms. The proposed short-formula pipeline gate is 10 ms p95; the earlier
5 ms input/capture and 25 ms chain gates are retained. All observed narrow rows
pass, without raising production limits.

Separate profile means for the 100K mixed formula pipeline, milliseconds:

| Instrumented scope | Baseline | Paged |
| --- | --- | --- |
| Core private staging | 26.1816 | <0.001 |
| Aggregate baseline staging / paged metadata staging | 45.5865 | <0.001 |
| Formula preparation/binding scope | 0.0192 | <0.001 |
| Apply scope (baseline aggregate / paged definition and indexes) | 74.1904 | 0.0043 |
| Snapshot scope | 67.8503 | <0.001 |
| Validation/order scope | 33.1115 | <0.001 |
| Selected evaluation scope | 0.2161 | <0.001 |
| Core commit/destruction scope | 19.2602 | <0.001 |

These scopes have different nesting: baseline core staging is inside aggregate
staging/apply, and order validation is inside snapshot. Do not sum them. The paged
commit scope covers inner core installation; final wrapper root installation and
some reclamation remain unattributed in the phase breakdown, but are included in
the complete pipeline and retention/cleanup wall timers. No work is subtracted from
those matching semantic timers to obtain the headline results.

## Linux corroboration

A separate changing-input control edits 101 measured distinct input positions
(plus three warmups) in fully populated value pages, then compares every cell's
metadata with the accepted baseline. At 1K/10K/100K Windows paged p95 is
5.9 / 9.1 / 11.3 microseconds; Linux 4.429 / 7.514 / 9.889 microseconds. Baseline
100K p95 is 65.0303 ms Windows and 46.0736 ms Linux. Both pass:
[Windows control](../../artifacts/structural-windows-changing-inputs.log),
[Linux control](../../artifacts/structural-linux-changing-inputs.log).
These correctness/latency controls share a fixture in one process; their memory
is not used in the standalone RSS comparison.

WSL2 Ubuntu corroboration uses the same hardware, Linux Rust nightly 8006091eb:
[Linux sweep](../../artifacts/structural-linux/20261019T111026Z/summary.json),
[distributions](../../artifacts/structural-linux/20261019T111026Z/analysis.md),
[normalized data](../../artifacts/structural-linux/20261019T111026Z/metrics.json).
At 100K, baseline/paged formula pipeline p95 milliseconds are independent
162.464 / 0.0144, chain 120.616 / 0.0144, mixed 163.839 / 0.0229, and fanout
184.632 / 1.9203. The chain input pipeline is 0.2591 ms. This corroborates locality,
not a cross-OS speed ranking. Fanout variance reinforces the remaining edge-set risk.

## Retention and memory

Separate profile invocations count requested allocator bytes/calls and defined
logical copy extents. These are not resident memory. At 100K mixed:

| Profile operation | Baseline | Paged |
| --- | --- | --- |
| Formula pipeline mean requested bytes | 148,455,886 | 11,032 |
| Allocation calls per formula pipeline | 771,276 | 75 |
| Instrumented logical copied payload | 48,797,811 | 5,422 |
| Net heap retained by 16 structural edits/captures | 759,632,915 | 119,144 |

The 16-generation row is a single diagnostic with old S also retained, not a
percentile. Single Windows latency cleanup observations: mixed baseline 335.86 ms,
paged 0.0257 ms; fanout baseline 497.80 ms, paged 5.93 ms. Releasing captures does
not necessarily free all edited data: current T and original S still own roots.
Fanout retention adds about 14.66 MB requested heap in the profile process because
changed reverse sets are copied. That remaining cost must not be hidden.

Whole-process peak RSS, including construction/import and retained generations:

| 100K shape | Baseline latency/profile MiB | Paged latency/profile MiB |
| --- | --- | --- |
| Independent | 894.7 / 879.6 | 150.4 / 150.9 |
| Chain | 883.0 / 883.1 | 151.8 / 151.8 |
| Mixed | 964.4 / 964.9 | 171.8 / 171.5 |
| Fanout | 1,185.5 / 1,182.6 | 228.5 / 228.5 |

These are standalone process measurements, not combined baseline/prototype memory.
Linux paged peaks across latency/profile processes are about 153 MiB independent,
154 MiB chain, 174 MiB mixed and 236 MiB fanout; baseline is about 914 / 918 / 997 /
1,239 MiB respectively. Linux RSS is sampled high-water from /proc, so very short
unobserved peaks can be missed; Windows uses the OS peak working-set counter.
As provisional reference-workload investigation ceilings, use 256 MiB whole-process
peak for 100K independent/chain/mixed and 384 MiB for 100K fanout, including the
specified 16 short-edit captures. These give measured headroom, not a promise for
arbitrary formula lengths, edge density, histories or a GUI/service process. Physical
admission and reclamation must still be implemented and exercised before adoption.

## Million-cell diagnostic, isolated fairly

[Windows million-cell summary](../../artifacts/structural-windows-million/20261019T111507Z/summary.json)
and [raw/normalized observations](../../artifacts/structural-windows-million/20261019T111507Z/analysis.md).
The fixture has 1M populated cells with 128 short tail formulas. Each implementation
and profile mode gets a fresh process, 8 GiB RSS guard and 600-second time guard.
The overall summary is intentionally `incomplete` because baseline runs stop.

Baseline latency/profile runs exceeded the RSS guard during the 16-structural-
generation retention operation, after the full individual formula pipeline had
completed. Observed peaks: 8,607,997,952 / 8,604,184,576 bytes. Last completed latency
rows: input edit 900.66 ms, capture 1,104.52 ms, complete formula pipeline 1,982.27 ms.
There is no baseline retention/cleanup result and no tail-latency claim.

Both paged runs completed. Windows latency single observations: trusted bulk
construction 2.933 s, conversion 2.446 s, input edit 0.0429 ms, capture 0.0001 ms,
input/capture/selected evaluation 0.0134 ms, formula pipeline 0.0239 ms, 16 structural
edits/captures 0.1905 ms and cleanup 0.0364 ms. Peak process RSS is 1,519,161,344 /
1,519,247,360 bytes (about 1.42 GiB); the profile's final requested live heap is
743,628,195 bytes while current T and original S remain retained. Bulk conversion
and allocator residency are included in process RSS; requested-live is measured
later. These are different measurements, not conflicting memory totals.

[Linux million-cell summary](../../artifacts/structural-linux-million/20261019T111703Z/summary.json)
and [observations](../../artifacts/structural-linux-million/20261019T111703Z/analysis.md)
corroborate the outcome. Baseline latency/profile stop during structural retention
at 8,613,257,216 / 8,606,486,528 bytes RSS. Both paged processes complete at
1,575,243,776 / 1,574,993,920 bytes (about 1.47 GiB). Single latency observations:
input edit 0.0326 ms, capture 0.000441 ms, selected input pipeline 0.0396 ms, formula
pipeline 0.0383 ms, 16 structural generations 0.4923 ms and cleanup 0.1902 ms.
The final requested live heap in profile mode is 743,341,643 bytes. The result is
one guarded independent-shape diagnostic per mode, not a percentile or capacity test.

A 2 GiB whole-process research ceiling is defensible for repeating this specific
independent 1M diagnostic on the reference machine. It is not a default limit or
product guarantee. No million-formula, dense-edge, range, port, import, churn or UI
capacity follows from one sample; production admission remains unchanged.

## Correctness and regression evidence

Six new core tests and one workbook integration test cover forward/reverse edge
changes, unchanged-definition sharing, baseline agreement, independent expected
values, 256 changing formula nodes, retained topology generations, malformed drafts,
cycle rejection/repair, arithmetic failures, retirement/name reuse, explicit missing
reference repair, stable IDs, transfer rejection, stale versions, validation budgets,
quotas, atomic rollback and exact signed-zero/scale-28 worksheet metadata.

Windows default full CI passes: 209 Rust tests, format, Clippy, .NET Release,
10,099 decimal vectors, protocol/frozen fixtures and actual IPC/REPL:
[summary](../../artifacts/ci/20261019T110930363527Z/summary.json).
Feature workspace: 222 tests pass in
[test log](../../artifacts/structural-windows-feature-tests-unrestricted.log);
[feature Clippy](../../artifacts/structural-windows-feature-lint.log) passes.
The initial sandbox feature run failed named-pipe access; the unchanged suite passed
outside the sandbox. The failed evidence remains in
[sandbox log](../../artifacts/structural-windows-feature-tests.log).

Linux passes 210 default tests, 223 feature tests, feature all-target Clippy and
actual IPC/REPL. File tests use disk-backed PIVOT_FILE_TEST_ROOT under /var/tmp,
not tmpfs: [default tests](../../artifacts/structural-linux-default-tests.log),
[feature tests](../../artifacts/structural-linux-feature-tests.log),
[lint](../../artifacts/structural-linux-feature-lint.log),
[IPC](../../artifacts/structural-linux-ipc/results.txt).

Windows source manifest predates formatting-only changes in the main harness and
registration of the additional input-control example. The core/adapter runtime
sources are unchanged. Linux uses the subsequently formatted harness; exact source
hashes and commands remain in each run rather than being retroactively rewritten.

## Production recommendation

Proceed toward a separately reviewed persistent/paged production design, rather
than dismissing graph-first calculation or adopting the old treap. The measured
local formula path no longer reconstructs every definition. This is evidence for
the ownership strategy, not a finished replacement core or spreadsheet capacity.

Before adoption:

1. Make the name directory and large reverse-edge indexes persistent/local too;
   quantify broad dependency edits, long chains, invalid-draft repair and bulk paste.
2. Integrate public ports, invalid-model import, exact quota/no-op semantics and
   generation-tagged public result/view contracts. Research helpers are not those
   contracts; core result maps are associated with their capture rather than a new
   accepted application result envelope.
3. Extend atomic paged projection to demanded anchors, moves/retirement and axis
   operations; the present workbook adapter deliberately keeps positions fixed.
4. Add physical page accounting, capture/generation admission and scheduled large
   reclamation, then validate churn rather than only 16 retained versions.
5. For incremental calculation, traverse reverse edges to select dirty outputs and
   retain generation-aware derived results with correct error/dependency invalidation.
   This experiment always recomputes requested closures and does not make fanout free.
6. Adapt WorkingModel/ModelSnapshot ownership, transfer/import/export, hosted draft
   staging and immutable publication to coherent roots while preserving existing
   identity ledgers, port validation and frozen external formats. File infrastructure
   and protocol expansion are unnecessary for this change.

Stop for review. C#, Avalonia, new syntax/ranges, files and hosted persistence remain
held. Native macOS corroboration of this prototype is not yet performed; historical
Scalar Formula Slice B macOS CI remains separate evidence.
