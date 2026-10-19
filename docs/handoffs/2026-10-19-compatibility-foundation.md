# Production Graph Representation - Compatibility Foundation

Date: 2026-10-19
Recommendation: **Further focused restructuring required.**
The persistent/paged direction remains supported; do not switch production users.

## Review packet

The parent accepted Structural Mutation Feasibility and requested persistent
indexes, general-model compatibility, restoration, memory admission and reclamation.
This slice extends the opt-in core, not the production WorkingModel backend.

- [Architecture and limitations](../architecture/persistent-core-compatibility-foundation.md)
- [Benchmark specification and reproduction](../performance/compatibility-foundation.md)
- [Paged core](../../crates/pivot-core/src/model/structural.rs)
- [Persistent name/edge index](../../crates/pivot-core/src/model/structural/index.rs)
- [Charged allocation domain](../../crates/pivot-core/src/model/structural/memory.rs)
- [Transfer/port adapters](../../crates/pivot-core/src/model/structural/compatibility.rs)
- [Bounded reclamation worker](../../crates/pivot-core/src/model/structural/reclaim.rs)
- [Differential and independent-oracle tests](../../crates/pivot-core/tests/compatibility_foundation.rs)
- [General-core Release harness](../../crates/pivot-core/examples/foundation.rs)
- [Process-isolated runner](../../scripts/foundation.py)
- [Final validation and provenance summary](../../artifacts/compatibility-foundation-validation.json)

No dependency package, persisted ABI, default capacity or production backend switch
is introduced. Core remains free of workbook, storage, transport and UI dependencies.
The intended destination is one reviewed core implementation behind the accepted
facade, not two user-facing computation engines.

## Compatibility implemented

Names and reverse edges use persistent AVL paths. Definitions/value pages remain
shared; renames preserve bound IDs; retired identities cannot be reused. Named
inputs/formulas, additions/replacement/removal, explicit repair, invalid drafts,
public ports/defaults, named invocation, temporary overrides and captured S/T
semantics are exercised. Results are immutable, version tagged and retain captures.
Same-name renames and empty port patches preserve the experimental version.

Accepted and experimental paths share binding lookup, invocation validation, named
request preparation, public error conversion and interface target validation.
Accepted port-authoring methods run through a small private classification proxy.
The existing postfix arithmetic executor is unchanged.

Transfer restoration uses the accepted checker and stored tokens, never historical
source rebinding. A round-trip test compares full transfer images, invalid bindings,
allocated ledgers and ports, with separate exact numeric-parts assertions. Tests
also check name reuse, explicit repair and independent expected values. Existing
storage/document/host/codecs are untouched; their normal frozen-fixture and save/
restore tests still run. The adapter is not yet wired into their production owners.

Full Graph::diagnose uses the accepted snapshot checker and complete affected-node
lists. Local capture rejection can still return a representative cycle path, so
diagnostic-detail integration requires review. Invalid imports use guarded full
validation during repair until valid; ordinary valid changes use bounded local
dependency traversal. No incremental result cache is implemented.

## Optimized performance evidence

Reference hardware: Ryzen 9 3950X, 16 cores/32 logical CPUs, about 64 GiB RAM.
Windows 11 build 26300, Rust nightly b6eeb93c9; WSL2 Ubuntu, Rust nightly 8006091eb.
Default Release opt-level 3, no custom LTO/target CPU, affinity or frequency lock.
Benchmarks are sequential; ordinary build/regression work may overlap them. No
hermetic load control or cross-OS speed ranking is claimed.

Full sweeps cover 1K/10K/100K and three degrees, with 31 measured latency samples
after three warmups and five separate profile samples. Final policy rechecks cover
all prototype sizes/degrees after admission/no-op audit corrections. Use baseline
rows from the full sweeps and final prototype rows from the policy sweeps:

| Platform | Full sweep / baseline evidence | Final prototype evidence |
| --- | --- | --- |
| Windows | [36 runs](../../artifacts/foundation-windows/20261019T115140Z/summary.json), [distributions](../../artifacts/foundation-windows/20261019T115140Z/analysis.md) | [18 runs](../../artifacts/compatibility-final-windows/core/20261019T123209Z/summary.json), [distributions](../../artifacts/compatibility-final-windows/core/20261019T123209Z/analysis.md) |
| Linux | [36 runs](../../artifacts/foundation-linux/20261019T121313Z/summary.json), [distributions](../../artifacts/foundation-linux/20261019T121313Z/analysis.md) | [18 runs](../../artifacts/compatibility-final-linux/core/20261019T123429Z/summary.json), [distributions](../../artifacts/compatibility-final-linux/core/20261019T123429Z/analysis.md) |

The harness/accepted baseline code is unchanged between those runs. Early prototype
measurements remain preserved, not rewritten: audits added conservative fixed-port-
root accounting, no-op checks, trusted-token/port-name headroom and wide-structure work reserves. Source manifests
identify each build. The final policy manifests match the submitted runtime sources.

100K general-core cells, complete single-edge edit through selected evaluation,
p95 milliseconds:

| Dependents | Windows baseline | Windows paged | Linux baseline | Linux paged |
| --- | --- | --- | --- | --- |
| 1 | 85.7488 | 0.0092 | 70.2944 | 0.0084 |
| 1,000 | 85.6239 | 0.0108 | 70.9458 | 0.0081 |
| 99,998 | 146.2558 | 0.0128 | 100.3307 | 0.0089 |

The degree-1 control alternates its single source; higher degrees change different
formula positions. Every edge row changes its dependency. Both pipelines include
preparation, private staging, all validation, installation, capture and evaluation.

Additional 100K, degree-1 pipelines, Windows p95 milliseconds:

| Operation | Baseline | Paged |
| --- | --- | --- |
| Add named input | 88.3642 | 0.0180 |
| Rename stable node | 81.2733 | 0.0120 |
| Add then retire name/node | 170.4359 | 0.0164 |
| Add formula | 89.9544 | 0.0180 |
| Save missing-name draft and repair | 109.6698 | 0.0140 |
| Update port and invoke required/default inputs | 81.6299 | 0.0044 |

These use three public ports, not 100K ports. All observed narrow valid-edit rows
meet the provisional 10 ms formula gate; this does not bound arbitrary dependency
closures, broad retirement or invalid-model diagnosis.

The retained workbook harness was also rerun with the final core, 31 samples:
[Windows final chain](../../artifacts/compatibility-final-windows/chain/20261019T123224Z/summary.json) and
[Linux final chain](../../artifacts/compatibility-final-linux/chain/20261019T123444Z/summary.json).
Baseline comparisons remain in the earlier full chain sweeps:
[Windows baseline](../../artifacts/foundation-windows-chain/20261019T122050Z/summary.json),
[Linux baseline](../../artifacts/foundation-linux-chain/20261019T122146Z/summary.json).
At 100K, the 1,000-formula input/capture/evaluation p95 is 144.8921 /
0.3744 ms baseline/paged on Windows and 116.0282 /
0.3785 ms on Linux. Windows paged input edit/capture
p95 is 0.0058 / 0.0003 ms. The 5 ms input/capture
and 25 ms chain investigation gates remain satisfied. Do not compare general-core
and workbook pipeline timings as if they were identical workloads.

## Allocations, retention and cleanup

At 100K/99,998 dependents, separate Windows profile means:

| Operation/metric | Baseline | Paged |
| --- | --- | --- |
| Requested allocation per edge pipeline | 104,028,366 bytes | 10,472 bytes |
| Allocation calls | 1,190,066 | 77 |
| Instrumented logical clone extent | 41,977,412 bytes | 4,652 bytes |
| Net heap retained by 32 structural captures | 1,512,371,706 bytes | 128,218 bytes |

High-degree paged edge-pipeline allocations at 1K/10K/100K are 9,856 / 10,120 /
10,472 bytes and 70 / 73 / 77 allocation calls. The degree index no longer copies
all unrelated edges. Counters are defined logical extents/requested heap, not
physical memory traffic or allocator-resident memory.

Whole-process peak RSS includes import and all retained generations:

| 100K degree | Windows baseline/paged MiB | Linux baseline/paged MiB |
| --- | --- | --- |
| 1 | about 1,285 / 86 | about 1,367 / 89 |
| 1,000 | about 1,290 / 86 | about 1,386 / 90 |
| 99,998 | about 1,768 / 115 | about 1,898 / 123 |

The per-process runs are independent. Native Linux uses sampled /proc high-water;
Windows uses OS peak working set. They are not steady-state RSS or GUI budgets.
For repeating this exact standalone-core/32-capture workload, 192 MiB OS peak is a
provisional investigation ceiling for 100K, with a separate whole-workbook budget.
The configured core allocation-domain limit is not that RSS ceiling.

At 100K/high-degree, 32-capture cleanup is a single 0.0390 ms Windows observation;
last-owner worker enqueue is 0.0037 ms and join is
54.75 ms. Across Windows degrees the worker join
takes roughly 41-55 ms; Linux high-degree join is
73.18 ms. This is real destruction work, not eliminated
work. The worker keeps it off the submitter, but shutdown/join must also stay off
the interactive path. Domain charges return to zero after join in every prototype
benchmark.

The raw churn diagnostic performs 1,024 additions/removals and records retirement
growth; baseline uses in-place authoring and paged uses atomic transactions, so it
is not a matched end-to-end speed ratio. Candidate rejection includes cleanup and
verifies unchanged versions/charges. Source manifests and raw phase rows retain
all other distributions, allocation figures and single-diagnostic timings.

## Guarded million-cell repeat

The final prototype completes both latency and profile diagnostics with allocation
admission enabled and the unchanged 8 GiB OS guard:
[Windows](../../artifacts/compatibility-final-windows/million/20261019T123230Z/summary.json),
[Windows observations](../../artifacts/compatibility-final-windows/million/20261019T123230Z/analysis.md),
[Linux](../../artifacts/compatibility-final-linux/million/20261019T123451Z/summary.json),
[Linux observations](../../artifacts/compatibility-final-linux/million/20261019T123451Z/analysis.md).

| Single observation | Windows | Linux |
| --- | --- | --- |
| Trusted bulk construction | 3.983 s | 3.658 s |
| Conversion/admission | 8.179 s | 5.532 s |
| Local input edit | 0.0525 ms | 0.1136 ms |
| Capture | 0.0006 ms | 0.0077 ms |
| Input/capture/selected evaluation | 0.0215 ms | 0.0622 ms |
| Formula pipeline | 0.0400 ms | 0.0450 ms |
| 16 structural edits/captures | 0.3194 ms | 0.4282 ms |
| Releasing those 16 captures | 0.0922 ms | 0.1067 ms |

Windows latency/profile peak RSS is 1,261,748,224 / 1,261,084,672 bytes, about
1.18 GiB. Linux is 1,330,696,192 / 1,330,454,528 bytes, about
1.24 GiB. Final System-profile live requested heap is
855,269,165 / 855,452,829 bytes while current T and original S remain
alive; those figures include the workbook harness, not just the core admission
domain. Final whole-graph destruction is outside individual operation timers;
process duration/RSS includes it. The 100K general-core harness separately measures
off-thread final-owner reclamation.

This repeat has 1M populated cells, only 128 short formulas, one measured operation
per row and trusted bulk construction. It is not percentile evidence, ordinary file
import, broad dependency coverage or a general million-cell capacity claim. Baseline
1M retention already exceeded 8 GiB in the accepted prior milestone and was not
repeated here. A 2 GiB whole-process investigation ceiling remains reasonable for
this exact prototype diagnostic, distinct from the 2 GiB core allocation-domain
limit. Neither raises a production capacity or transport limit.

## Memory policy and its limit

Known unique allocations are charged once across shared roots. Private candidate
headroom, parsing, trusted tokens, port names, validation, calculations and imports
reserve space before construction. Captures and returned results keep their charges.
The default experimental domain is 2 GiB and configurable. MemoryStats separates
allocated charges from reservations; its admission peak intentionally approaches
2 GiB in the forced-rejection control without allocating that much heap.

The initial policy covers known engine storage and conservative work allowances,
not all process memory. Allocator reserves, application-wide aggregation, external
source/request/copy-out data and frozen projection containers need additional policy.
Regular Drop still performs synchronous last-owner destruction unless the explicit
bounded reclaimer is used. Queue/admission rejection returns ownership safely.

## Regression evidence

Eleven new tests (ten integration, one AVL invariant test) add differential/independent
coverage for ports/defaults, error validation, immutable S/T, renamed/retired targets,
invalid imports and multiple cycles, complete diagnostics, transfer restoration,
numeric parts, no-op behavior, admission rollback, trusted payload headroom,
shared-owner accounting, wide existing dependencies/retirement, bounded queue
rejection and off-thread reclamation.

Windows default full CI passes 209 Rust tests, format/Clippy, .NET Release,
10,099 decimal vectors, protocol/frozen fixtures and actual IPC/REPL:
[CI summary](../../artifacts/ci/20261019T120613565363Z/summary.json).
Final feature suite passes 233 tests:
[tests](../../artifacts/foundation-windows-feature-tests-submitted.log),
[all-target feature Clippy](../../artifacts/foundation-windows-feature-lint-final.log).
Feature-only admission/no-op audits followed default CI; default code was unchanged.

Linux passes 210 default tests, 234 feature tests, feature all-target Clippy and
actual IPC/REPL, using disk-backed file tests under /var/tmp:
[default](../../artifacts/foundation-linux-default-tests.log),
[feature](../../artifacts/foundation-linux-feature-tests-submitted.log),
[lint](../../artifacts/foundation-linux-feature-lint-submitted.log),
[IPC](../../artifacts/foundation-linux-ipc/results.txt).
No new Linux .NET or native macOS run is claimed. Prior macOS validation remains
historical; a new Mac run must copy source/builds to local APFS.

## Decision

Do not reconsider graph-first representation on this evidence. Persistent structural
indexes address the previous broad-copy bottlenecks, and compatibility tests support
continuing toward one accepted core backend.

Do not call this production ready. A focused integration-contract/policy slice is
still needed: settle full diagnostic and revision/view adapters, preserve hosted
final-directory batch binding/publication, bound many-port operations, aggregate
memory/copy-out budgets, wire cancellation and reclaimer ownership, and handle
retirement compaction and initially invalid repair work. The fixed workbook adapter
also still lacks demanded-anchor and broad projection-mutation coverage.

No default backend switch, limit increase, new formula grammar, result cache,
production workbook format, C# IPC, Avalonia or hosted persistence. Stop for review.
