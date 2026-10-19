# Graph and workbook scaling feasibility

Disposition: **B â€” viable with architectural restructuring**, subject to review.
The evidence supports retaining stable graph/cell identities and immutable captures,
but not retaining whole-workbook staging and snapshot rebuilding for ordinary edits.
This is a bounded research result, not production adoption or permission to resume C#.

The accepted Scalar Formula Slice B remains intact. The feature-gated alternative
and its limits are specified in the [benchmark specification](../performance/scaling-experiment.md).
Reproduction, platform evidence and follow-up decision appear in the
[research handoff](../handoffs/2026-10-19-scaling-feasibility.md).

## Optimized comparison

Reference: Ryzen 9 3950X, 16 cores/32 logical CPUs, approximately 64 GiB RAM,
Windows 11 build 26300, Rust 1.101.0-nightly b6eeb93c9, default optimized Release.
No custom LTO/target-cpu, CPU affinity, frequency lock or hermetic background-load
control. Each baseline/prototype pair uses the same workbook identities and shape.
The sweeps run sequentially; allocation profiling is a separate invocation.
WSL2 on the same machine corroborates behavior, not an independent hardware claim.

Windows p95 milliseconds at 100K populated cells:

| Operation | Baseline | Shared prototype | Provisional target |
| --- | --- | --- | --- |
| Input edit, independent shape | 56.316 | 0.0004 | <5 |
| Input edit after every input has an overlay entry | 56.316 | 0.0050 | <5 |
| Unchanged capture, independent shape | 83.730 | <0.001 amortized | <5 |
| Input update + capture + selected 1,000-formula chain | 183.034 | 0.397 | <25 |
| Sparse 100x50 read, mixed shape | 0.529 | 0.537 | <10 |
| Capture + all 99,999 fan-out formulas | 241.838 | 132.551 | No constant-time target |

Normal rows have 101 samples after three warmups; fanout capture/evaluation has 31.
The full-overlay baseline comparator is the ordinary existing-cell edit: the baseline
has no overlay population concept. Shared captures are timed in 1,000-call batches
and normalized; this is an amortized observation, not a single-call tail guarantee.
See raw p50/p95/worst values and sample counts in the linked data.

Independent-shape growth, observed p95:

| Cells | Baseline edit ms | Baseline capture ms | Shared full-overlay edit us |
| --- | --- | --- | --- |
| 1K | 0.249 | 0.461 | 1.4 |
| 10K | 4.181 | 6.853 | 1.5 |
| 100K | 56.316 | 83.730 | 5.0 |

This is evidence of unnecessary total-model work in the baseline. Copied/allocated
payload grows approximately in proportion to model size; latency grows somewhat
worse here because allocation, cache locality and destruction matter too. Shared
captures pin roots; input edits visit lookup/path-copy structures rather than every
cell. This supports the expected logarithmic direction, not a worst-case bound for
randomized treap balancing or every graph mutation.

Formula replacement is still a counterexample: Windows p95 is 77.8 ms baseline /
184.2 ms prototype on the independent shape, and 91.3 / 261.5 ms on mixed. These are
not equal validation-placement timings: the prototype eagerly validates/compiles
its new topology; baseline validation is deferred to capture. The prototype also
materializes then stages through accepted apply, intentionally doing extra copies.
Both remain O(total graph). No formula replacement improvement is claimed.

## Corroboration and negative controls

WSL2 on the same hardware gives the same direction. At 100K cells, independent
input edit p95 is 48.308 ms baseline / 0.000201 ms shared; capture 68.155 ms /
<0.001 ms amortized; update/capture/1,000-formula chain 139.904 / 0.293 ms;
mixed viewport 0.488 / 0.410 ms. Genuine 99,999-formula fanout capture/evaluation
still costs 167.543 / 111.615 ms. Compiler/allocator/OS differences mean these
are corroboration, not a platform speed ranking.

A separate changing-key control populates the overlay, then edits 101 distinct
positions and compares every cell view with the baseline. Windows p95 shared
cost grows 4.0 / 9.5 / 15.4 us at 1K/10K/100K; baseline 0.203 / 5.532 / 71.591 ms.
Linux shared costs are 3.0 / 6.0 / 13.0 us, baseline 0.187 / 4.705 / 49.115 ms.
This is a stronger locality control than repeatedly editing a single hot key.

The optional 1M independent-cell diagnostic is **incomplete**. Single baseline
observations were 896 ms input edit and 1,518 ms capture; these are not percentile
evidence. The runner stopped at its 8 GiB OS RSS budget (observed 8,608,436,224 bytes)
before the 16-generation retention operation completed. No prototype 1M comparison
was reached. The failure is retained; no 1M capacity or viability claim follows.

## Measured causes

At 100K independent cells, instrumented input-edit means attributed approximately
42.8 ms to aggregate staging (including 27.2 ms core staging), 3.75 ms to quota
scanning and 19.5 ms to commit/destruction, within 69.3 ms total apply. Timings nest
and are profile-mode observations, not latency-run percentiles. Additional total-state
no-op equality checks also remain outside the explicitly separated subphases.

Snapshot profiling attributes owned definition/name copying and validation to snapshot
construction, and reverse-index/count/ready-set rebuilding to dependency_order.
Selected evaluation already avoids computing unrelated formulas in the baseline,
but scans every captured node's order entry. The experiment schedules only the
selected dependency closure using the same production postfix arithmetic executor.
Parser-only optimization would not address the measured dominant costs.

The exact phase/allocator profiles and native sampling report are linked from the
handoff. Native perf captured 2,468 user-CPU samples, zero lost. The installed perf
build lacks symbol/unwind libraries, so addr2line resolved 548 program offsets
separately. Sampled inline locations include BTree subtree cloning/traversal and
sorting captured cell maps, corroborating the instrumented profiles. This is flat
IP/inline evidence, not a reconstructed non-inline call tree. Copy counters count defined owned payload extents and cloned heap lengths; they
are logical payload-volume evidence, not physical memory-bus traffic or all BTree
moves. Enum padding/active-variant code generation is not measured byte-for-byte.

## Memory evidence and provisional budget

Windows instrumented 100K mixed workbook observations:

| Operation/state | Observation |
| --- | --- |
| Baseline existing input edit | 77,481,107 bytes allocated; 395,487 allocation calls; 34,282,690 instrumented copied-payload bytes |
| Baseline capture | 72,097,767 bytes allocated; 47,478,447 bytes retained by the returned capture |
| Shared full-overlay edit | 5,808 bytes allocated, 27 calls, 5,200 copied-payload bytes |
| Shared capture | No allocation or copied payload |
| Retain 16 value-edit captures, baseline | 759,657,072 additional requested heap bytes retained |
| Retain 16 value-edit captures, shared | 3,880 additional requested heap bytes retained (one repeatedly edited key) |

The mixed shared compiled state occupied about 114.5 MiB requested process heap in
this harness. The later populated overlay added 16.5 MiB for 80K eligible inputs.
With original S's topology still alive after structural edits, the process requested
about 245.5 MiB. Retaining structural generations duplicates full topologies in this
prototype; it does not have the small overhead of value-only generations.
The combined baseline/prototype sweep reached roughly 1.1 GiB OS peak RSS, dominated
by diagnostic multi-generation workloads. That is not the normal shared state's RSS.
Requested heap excludes allocator overhead/reserved arenas and is not RSS.

Suggested next research budgets for this specified 100K mixed short-formula workload:
256 MiB requested graph/workbook heap for a steady compiled/overlaid state, 512 MiB
with one retained structural generation, and <1 MiB incremental retention for 16
single-cell value-edit generations. These are proposed engineering gates, not tested
worst-case guarantees or general limits for arbitrary formula sizes/densities.
A separate standalone RSS measurement and explicit generation/byte admission policy
are still needed; these component numbers cannot establish a whole-application budget.

## What the prototype proves and leaves open

The input overlay preserves stable existing cell/node identities, exact decimal
metadata, bound targets, atomic rollback and immutable old/new calculation roots.
Structural operations deliberately return to the accepted workbook implementation,
including editable invalid drafts and whole-graph structural diagnostics. There is
no production switch, alternate parser, alternate decimal evaluator, shared mutable
calculation cache, new IPC, file behavior or dependency package.

Its own Version tag and base-tagged viewport wrapper are research APIs, not accepted
application contracts. Structural topology rebuilding, automatic dirty-output
scheduling, persistent result reuse, source/binding-version integration, coherent
production view tags, physical memory quotas and generation reclamation remain
unsolved. Asking for all outputs still recomputes all of them. Dense affected work
is inherently significant; cached immutable topology does not make fan-out free.

## Decision and next review

A (small optimization sufficient) is not supported: ordinary formula edits and
structural generations still copy/rebuild, and production admission cannot simply
be raised. C (graph-first concept not currently viable) is also not supported: the
same bound computations and stable identities can meet the narrow value-edit,
capture and chain targets with shared state. B best matches the evidence.

Review a core/worksheet storage design separating immutable compiled topology from
value roots, with coherent transaction/version tags and incremental structural index
maintenance or explicit bounded rebuild policy. Consider compact indexed chunks or
pages instead of adopting this treap literally. Preserve existing evaluator/numeric
semantics and all frozen codecs; retain the baseline and workload oracles for each
step. Require changing-key and retained-generation controls, formula-edit scaling,
physical memory accounting and production limit policy before adopting it.

The feasibility investigation ends here. The next decision is restructuring scope,
not automatic C# implementation or additional spreadsheet features.
