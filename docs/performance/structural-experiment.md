# Structural mutation experiment

An isolated successor to the [input-only experiment](scaling-experiment.md).
The [representation comparison](../architecture/structural-mutation-feasibility.md)
specifies the candidate and limitations. Defaults and frozen formats do not change.

```sh
python scripts/structural.py --output-dir artifacts/structural-local
python scripts/structural.py --sizes 1000000 --shapes independent --samples 1 --output-dir artifacts/structural-million
```

The standard-library runner builds Release once, then launches separate processes
for every implementation/size/shape/mode combination. Baseline retention cannot
consume a prototype process's memory budget. Guards are 8 GiB peak RSS and 600
seconds per process; a stopped row is incomplete evidence. Summary files retain
commands, statuses, peak process RSS, environment and source hashes. Build/setup
cost is separate from edits but construction/import peak remains included in RSS.
No native macOS execution is implied by WSL results; Mac builds must use local APFS.

The 1K/10K/100K shapes retain the original chain, mixed and fanout definitions.
Independent now has 128 tail formulas referencing A1 so the changing-formula test
can edit different existing formulas. These counts are not identical to the prior
one-formula independent fixture. Chain has 999 formulas at 1K, 1,000 above that.

Latency rows have 101 measured samples after three warmups. Profile rows have five
separate samples; construction, compilation, 16-generation retention and cleanup
are single diagnostics. A 1M --samples 1 run has no warmups or percentile evidence.
Percentiles use sorted middle p50 and nearest-rank p95; worst is the maximum.
Root capture is timed individually, so sub-microsecond results remain timer-limited.

The complete changing-formula timer includes source string preparation, accepted
parsing/binding or spreadsheet lowering, private staging, definition/index changes,
validation, installation, calculation-ready capture and selected changed-output
evaluation. Baseline capture includes its deferred validation and workbook maps;
prototype validation happens before installation. Neither side hides validation
outside the complete timer. Source alternates between A1 + constant and a literal,
exercising edge preservation/removal across different formula nodes. Core correctness
tests separately exercise edge addition, missing-reference repair and formula addition.
Closed-form selected outputs are asserted in both processes, as are old S values.

Phase-profile scopes report parsing/binding, core/metadata staging, index changes,
validation, installation, capture and evaluation. Production scopes nest; do not sum
them. Baseline apply includes definition changes and commit, but its destructor is
not separated from all work by the old instrumentation. Some wrapper/loop work is
unattributed; complete elapsed time remains authoritative. Retention is a separate
16-edit/16-capture operation, followed by timed last-owner cleanup of those captures.
The original S and current version remain alive during cleanup.

Allocation instrumentation uses the existing System-backed counting allocator in a
separate process mode. It reports mean requested allocated bytes/calls, extra peak
requested heap, net retained heap and defined logical copy extents. It is not a
complete CPU memory-traffic counter. Counter/mode branches remain in latency builds;
their overhead is not calibrated. Requested heap excludes allocator reserves and
is not RSS. OS peak RSS is per whole process and includes import and retention;
it must not be described as the steady compiled state's resident size.

No affinity, frequency lock or hermetic background-load control is used. Fresh
processes allocate fresh UUIDs, so separate baseline/prototype fixtures have the same
definitions and expected values, rather than equal physical UUID bytes. Correctness
tests compare the same initial identities within one process. Runs are sequential;
Windows and WSL use different nightly toolchain builds and are not a speed ranking.

The additional changing-input control is:

```sh
cargo run --locked --release -p pivot-workbook --features scaling-experiment --example structural-inputs
```

It edits 101 distinct existing inputs after three warmups at 1K/10K/100K, with
fully populated value pages, and compares every cell's metadata afterward. It uses
one staged fixture to preserve the same IDs in both implementations; its combined
memory is deliberately excluded from standalone RSS evidence. Main-sweep formula
controls edit different existing formula nodes throughout their measured samples.

The bulk fixture builder bypasses normal repeated user-edit staging and raises only
research admission allowances. Construction is not ordinary import throughput.
Default workbook caps, all previous tests, storage fixtures and codecs are preserved.
