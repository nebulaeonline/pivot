# Graph and Workbook Scaling Feasibility — Research Handoff

Date: 2026-10-19
Disposition: **B — viable with architectural restructuring.** Ready for review.

## Decision

Accept Scalar Formula Slice B as reviewed. Keep C# and feature expansion on hold.
The graph-first concept is not disproven: shared compiled topology/value roots can
preserve accepted semantics and meet the narrow 100K value-edit/capture/chain targets.
The existing whole-copy production implementation fails those targets. A bounded
fast path is not enough for production adoption: formula edits, structural snapshots,
coherent version/view APIs, dirty scheduling and physical memory policy need design.

A (small optimization sufficient) understates the remaining work. C (not currently
viable as a graph-first concept) is not supported by the alternative's correctness
and scaling evidence. B is conditional evidence for restructuring, not a guarantee
that every future spreadsheet requirement will scale. Stop for parent review.

## Review packet

- [Assessment, measurements and proposed memory budgets](../architecture/graph-workbook-scaling-feasibility.md)
- [Benchmark specification and reproduction](../performance/scaling-experiment.md)
- [Experimental workbook/root strategy and correctness tests](../../crates/pivot-workbook/src/scaling.rs)
- [Decimal representation and threaded capture tests](../../crates/pivot-workbook/tests/scaling.rs)
- [Main optimized harness](../../crates/pivot-workbook/examples/scaling.rs)
- [Changing-key control](../../crates/pivot-workbook/examples/scaling-random.rs)
- [Runner and resource guards](../../scripts/scaling.py)
- [Result normalization](../../scripts/scaling_analyze.py)
- [Offline sample resolution](../../scripts/scaling_symbolize.py)
- [Opt-in phase hooks](../../crates/pivot-core/src/experiment_profile.rs)

No new external dependency. Production constructors/capacities and default behavior
remain unchanged. Existing parser, stable binding and decimal postfix executor are
reused. New scheduling/overlay APIs are feature gated by scaling-experiment and not
exposed through RPC. The production dependency boundary test remains unchanged.

## Experiments and independent checks

Release workloads at 1K/10K/100K populated cells: independent inputs plus a formula,
up to 1,000 dependent formulas, 99,999-way fanout, and 80% inputs / 20% small mostly
independent formulas. Reference machine: Ryzen 9 3950X, 16 cores/32 logical CPUs,
about 64 GiB RAM. WSL2 exposes about 31 GiB. Environment files record exact OS,
compiler, CPU/memory and build settings. Windows Rust b6eeb93c9 and Linux 8006091eb
are different nightly builds. No cross-OS speed ranking is claimed.

101 samples after three warmups for ordinary latency rows; 31 for affected-output
capture/evaluation; one bulk-build and one 16-generation retention diagnostic.
Capture roots use 1,000-call batches for timer resolution. Five separate allocation/
phase-profile samples do not count as uninstrumented latency evidence. All p50,
p95, worst, sample counts, requested heap/copy counters and OS process peak RSS
remain in raw CSV/normalized JSON. Allocated bytes are not relabeled copied bytes.

Production caps prevent these workbook sizes. The opt-in builder controls capacity
and bulk setup explicitly; timed edits still use accepted apply. Bulk construction
skips repeated staging/quota scans and is not ordinary import throughput. Logical
quotas are not physical heap/RSS guarantees. Existing file/protocol limits are not
raised. Fresh UUID identities are shared within each comparison, not seeded for
bit-identical reproduction. No benchmark ran concurrently with another benchmark.

The prototype shares validated topology, positions and identities, and stages existing
input/Blank edits through a persistent input tree. Structural edits materialize and
use the accepted full-copy path, then compile/cache a new snapshot. This deliberately
moves structural validation cost into edits and can do extra copies. Thus structural
formula_replace rows are not matching validation-placement speed comparisons.
It does not implement automatic dirty-output selection or a persisted result cache.
Selected computations avoid unrelated nodes; asking for all outputs recomputes all.

Four unit and two integration tests compare accepted values/bindings/diagnostics,
rollback, stale states, quotas, exact representations, invalid drafts and cycles.
Tests retain S while T changes values and topology, execute on another thread, keep
multiple populated overlay generations, and check closed-form chain/fanout/mixed
results. New branch-allocated UUIDs are compared by binding meaning; existing shared
identities must remain equal. A changing-key control edits 101 distinct positions
with a populated overlay, then compares every cell view against the baseline.

## Primary evidence

Windows complete Release sweeps:

- [Run summary](../../artifacts/scaling-windows/20261019T095920Z/summary.json)
- [Environment](../../artifacts/scaling-windows/20261019T095920Z/environment.json)
- [Source manifest](../../artifacts/scaling-windows/20261019T095920Z/source-manifest.json)
- [p50/p95/worst comparison](../../artifacts/scaling-windows/20261019T095920Z/analysis.md)
- [Raw allocation/phase profile](../../artifacts/scaling-windows/20261019T095920Z/profile.csv)
- [Normalized metrics](../../artifacts/scaling-windows/20261019T095920Z/metrics.json)

Linux complete Release sweeps:

- [Run summary](../../artifacts/scaling-linux/20261019T101055Z/summary.json)
- [Environment](../../artifacts/scaling-linux/20261019T101055Z/environment.json)
- [Source manifest](../../artifacts/scaling-linux/20261019T101055Z/source-manifest.json)
- [p50/p95/worst comparison](../../artifacts/scaling-linux/20261019T101055Z/analysis.md)
- [Raw allocation/phase profile](../../artifacts/scaling-linux/20261019T101055Z/profile.csv)
- [Normalized metrics](../../artifacts/scaling-linux/20261019T101055Z/metrics.json)

Core/workbook runtime and main benchmark source have not changed since those sweeps.
Later changes are manifest section ordering, registering a supplemental example,
resource-guard/report scripts, additional test source and documentation; the retained
per-run source manifests identify the exact inputs to each build.

At 100K on Windows: independent input edit p95 56.3 ms baseline / 0.0004 ms shared;
full populated overlay edit 0.005 ms; capture 83.7 ms / <0.001 ms amortized;
update/capture/1,000-formula chain 183.0 / 0.397 ms; mixed viewport 0.529 / 0.537 ms.
Linux corroborates these targets. True full fanout remains expensive (Windows
241.8 / 132.6 ms capture/evaluation) and is not disguised as a constant-time edit.

The stronger changing-key control gives 100K p95 71.6 ms baseline / 0.0154 ms shared
on Windows and 49.1 / 0.0130 ms on Linux. These satisfy the provisional input-edit
investigation target without depending on one repeatedly hot key.
[Windows raw control](../../artifacts/scaling-windows-random.log),
[Linux raw control](../../artifacts/scaling-linux-random.log).

Memory profile: a 100K mixed baseline input edit allocated 77.5 MB versus 5.8 KB for
a shared full-overlay edit. A returned baseline capture retained 47.5 MB versus no
new allocation for a shared root capture. Sixteen single-key value generations
retained 759.7 MB versus 3.9 KB. These are requested heap bytes, not RSS or arbitrary
structural-generation guarantees. Whole-run OS peak RSS was about 1.1 GiB.
See the assessment for qualified 256/512 MiB research budget proposals and limits.

## Sampling profile and bottlenecks

Native optimized-with-debuginfo perf run: mixed 100K shape, 199 Hz user-CPU samples,
11 harness samples. Captured 2,468 CPU samples, zero lost; total data about 20.5 MB.

- [Record log](../../artifacts/scaling-linux-perf-native-record.log)
- [Whole-workload rows](../../artifacts/scaling-linux-perf-native-workload.csv)
- [Raw offsets](../../artifacts/scaling-linux-perf-offsets.txt)
- [Offline inline-frame resolution](../../artifacts/scaling-linux-perf-symbolized.md)
- [All resolved program offsets](../../artifacts/scaling-linux-perf-symbolized.json)
- [Native perf data copied back](../../artifacts/scaling-linux-perf-native.data)

The installed perf lacks symbol/unwind libraries. addr2line resolves 548 individual
program offsets, including BTree subtree cloning/traversal and sorting capture maps.
This is native sampling evidence plus inline scopes, not complete non-inline call
stacks or per-operation CPU percentages. Direct perf.data writes to mounted J: failed
with Bad address; recording on native /var/tmp succeeded, then data returned to the
share. Failed DWARF/frame-pointer attempts remain in artifacts.

Separate phase/allocator profiles attribute baseline edit costs to whole worksheet/core
staging, quota scans and replaced-state destruction, and capture costs to definition/
name copying, dependency/index/order rebuilding and copied cell/position maps. No-op
whole-state equality work remains too. Parser-only optimization does not fix them.
Logical copy counters exclude unobserved BTree/internal/hardware traffic and compiler
padding decisions; they do not claim byte-exact physical copy tracing.

## Million-cell diagnostic: explicitly incomplete

[Budget-stop summary](../../artifacts/scaling-windows-million/20261019T101714Z/summary.json),
[partial rows](../../artifacts/scaling-windows-million/20261019T101714Z/latency.csv).

One independent-cell Release sample, with a 600-second / 8 GiB RSS budget. Baseline
input edit observed 896 ms and capture 1,518 ms. The runner stopped after 46.5 seconds
at 8,608,436,224 observed OS RSS bytes before 16-generation retention completed.
The prototype 1M phase was never reached. This is a preserved failed diagnostic,
not percentile evidence, complete correctness validation or a 1M capacity claim.

## Regression verification

Windows full default CI passes **209 Rust tests**, formatting, Clippy, .NET Release,
10,099-vector decimal parity, protocol/frozen legacy parity and real named-pipe IPC.
The opt-in workspace suite passes **215 tests** and matching all-targets Clippy,
including the added changing-key example. Linux passes default **210** and opt-in
**216** Rust tests, formatting/feature Clippy and actual Unix-socket daemon/REPL.

- [Windows full CI summary](../../artifacts/ci/20261019T100925284480Z/summary.json)
- [Windows opt-in tests](../../artifacts/scaling-windows-feature-tests.log)
- [Windows final opt-in lint](../../artifacts/scaling-windows-feature-lint-final.log)
- [Linux default tests](../../artifacts/scaling-linux-default-tests.log)
- [Linux opt-in tests](../../artifacts/scaling-linux-feature-tests.log)
- [Linux opt-in lint](../../artifacts/scaling-linux-feature-lint.log)
- [Linux IPC results](../../artifacts/scaling-linux-ipc/results.txt)

The existing Windows debug target hit generated-DLL LNK1104, including outside the
sandbox; a fresh target/scaling-windows-checks resolved it. The
[initial failed CI summary](../../artifacts/ci/20261019T100732006678Z/summary.json)
is preserved. A development feature-manifest section initially tripped the existing
literal dependency test; reordering sections fixed it without changing dependencies
or weakening the test. Storage code/fixtures, lockfile, protocols, host, apps and .NET
sources remain unchanged. Native macOS has not rerun this new experiment; its earlier
accepted scalar-formula validation remains historical. No new Linux .NET run is claimed.

## Next decision, not automatic work

Approve or revise a bounded core/worksheet state restructuring design before more
product features. Separate compiled topology and value roots while preserving one
coherent revision/transaction/view contract. Evaluate persistent chunks/pages and
incremental structural indexes; retain this baseline, independent oracles and memory
controls. Formula edits and structural snapshot retention still require credible gates.
Do not adopt this tree literally or raise production caps as a substitute for design.
No ranges, aggregation, serialization, hosted persistence, RPC, C# or UI was added.
