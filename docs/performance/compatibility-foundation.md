# Compatibility foundation experiment

The [architecture note](../architecture/persistent-core-compatibility-foundation.md)
specifies semantics, admission and remaining integration limits.

```sh
python scripts/foundation.py --output-dir artifacts/foundation-local
python scripts/structural_analyze.py <run-directory>
```

The runner builds the feature-gated core foundation example in optimized Release,
then starts separate processes for each implementation, size, degree and mode.
Default sizes are 1K/10K/100K. Degree is 1, min(1K,(N-2)/2), or N-2. Each fixture
has two input sources, the specified number of n0+1 formulas, remaining inputs,
two public input ports and one output port. These are general-core workloads,
not workbook viewport or multi-port-width benchmarks.
The standalone conversion timer covers constructing/validating the paged graph;
destruction of its original source fixture follows outside that timer and before
ordinary edit timings. Process duration and peak RSS include that destruction.

Ordinary latency rows use 31 measured samples after three warmups; a separate
allocation/phase process uses five samples. Construction, 32-generation retention,
cleanup, 1,024-operation churn and last-owner reclamation are single diagnostics.
Report nearest-rank p95, middle p50 and maximum; 31 samples do not establish a
production tail-latency SLA. Both forms of instrumentation remain in the binary;
latency disables collection, without calibrating the mode-check overhead.

For ordinary valid edits, baseline staging, all deferred structural validation,
installation, snapshot and selected evaluation are inside the timer. Paged edits
include private roots/index updates, incremental validation, installation, charged
capture and selected evaluation. Source/name preparation is inside each timer.
Formula edits change different target positions where degree permits; the degree-1
control alternates the source edge on its single formula. Every edge row changes
the dependency, rather than just changing a numeric token on a hot target.

Name addition and rename include calculation-ready capture and selected evaluation.
The add/retire row includes two private commits, an intermediate calculation-ready
capture and final selected evaluation. Formula addition and missing-name draft/repair
are separate rows. Repair includes both saving the invalid draft and explicitly
resubmitting its repaired expression. Port update invokes the actual named API with
one required override and one optional default. Expected values and old S are checked.

Candidate rejection first changes an input privately, then encounters a foreign ID;
the timer includes disposal and verifies no installation. The paged admission row
deliberately exhausts reserved headroom and verifies rejection. Baseline has no
matching allocation-domain policy. Stale rejection is a small raw precondition
control, not a relative performance claim.

The raw churn row performs 1,024 input additions/removals. Baseline uses its raw
in-place authoring API; paged uses atomic root transactions. It does not include
equivalent staging/validation/evaluation and must not be reported as a matched
pipeline speed comparison. It exercises retirement growth and admission instead.

The final paged owner is handed to a bounded worker; enqueue and worker join have
separate timers, and all domain charges must return to zero. Join can be expensive.
The baseline final-owner row destroys its remaining model/snapshot inline. Compare
the work involved, not enqueue time against a complete destructor as a speed ratio.

System-backed allocation counters report requested bytes/calls, net retained heap,
extra peak heap and instrumented logical clone extents. Domain MEMORY rows separately
report allocated charges, reservations and admission peak. These are not interchangeable
with allocator counters or RSS. Per-process peak RSS includes construction/import and
all diagnostics. Linux samples /proc high-water at 50 ms; short peaks can be missed.
Windows reads peak working set. The RSS guard is 8 GiB and timeout 600 seconds.

The original structural harness is retained for workbook/chain regression and a
guarded 1M independent-shape repeat:

```sh
python scripts/structural.py --sizes 100000 --shapes chain --output-dir artifacts/structural-chain-foundation
python scripts/structural.py --sizes 1000000 --shapes independent --samples 1 --implementations paged --output-dir artifacts/structural-million-foundation
```

The million-cell fixture still has only 128 short tail formulas and trusted bulk
construction. It cannot establish arbitrary million-cell capacity or file-loading
throughput. Default workbook/transfer limits stay unchanged.

Reference hardware remains Ryzen 9 3950X, about 64 GiB RAM; exact compilers and source
hashes are recorded per run. No affinity/frequency lock or hermetic background-load
control is used. Benchmarks run sequentially; builds/regressions may overlap them.
Native macOS is a distinct follow-up using local APFS for source/build/.NET work.
