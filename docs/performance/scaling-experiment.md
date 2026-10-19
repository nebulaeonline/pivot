# Graph and workbook scaling experiment

Status: Opt-in research harness; not a production implementation or new capacity promise.

## Question and stop boundary

The scalar formula milestone is accepted. Before C# integration, test whether
stable graph/cell identities, atomic edits and immutable captures permit ordinary
spreadsheet input updates without work proportional to every populated cell.
Investigate performance honestly; a negative result is permitted. No grammar,
ranges, files, persistence, IPC or UI features belong in this experiment.

Provisional 100K-cell p95 targets: isolated input edit below 5 ms; unchanged capture
below 5 ms; updating/capturing/evaluating 1,000 dependent formulas below 25 ms;
sparse 100-by-50 viewport below 10 ms. These are feasibility targets, not release
promises. Memory must be measured before assigning a budget.

## Reproduction

```sh
python scripts/scaling.py --output-dir artifacts/scaling-local
```

Requires the existing Rust/Python toolchain, no new packages. The runner builds the
opt-in example once with Cargo's optimized Release profile, then executes its binary
directly. Build time is excluded. Raw logs, commands, exit statuses, environment,
source hashes and OS process peak RSS are retained under a UTC run directory.
Use --sizes, --shapes, --samples or --modes to select a diagnostic subset.

```sh
cargo test --locked -p pivot-workbook --features scaling-experiment
cargo clippy --locked --workspace --all-targets --features pivot-workbook/scaling-experiment -- -D warnings
```

On Linux set CARGO_TARGET_DIR to a separate target directory. On macOS copy the
current working tree to local APFS, especially before .NET; return only reports to
the share. Do not build on SMB. Native Mac corroboration is a separate follow-up;
Windows/WSL evidence is sufficient to submit the bounded research assessment.

## Workloads and admission

Sizes: 1,000 / 10,000 / 100,000 populated cells. One-cell spacing in both axes gives
sparse positional reads. Cells are initialized to decimal one. Then:

| Shape | Definitions |
| --- | --- |
| independent | Inputs with one formula at the end, referencing the first input plus one |
| chain | First input followed by up to 1,000 successive `previous + 1` formulas; remaining cells independent |
| fanout | Every cell after the first is `first input + 1` |
| mixed | Every fifth cell is `preceding input + 1`; remaining cells inputs |

At 1K cells the chain has 999 formulas; larger sizes have exactly 1,000. The
selected mixed formula is independent of edits to the first input. The affected
fanout evaluation requests every dependent output; other selected requests are
one terminal formula. Formula sources are parsed/bound by the accepted pipeline.

UUID identities are freshly generated, so shape/results are deterministic but
allocation identities/treap shape are not seeded for exact replay. Both alternatives
in a pair share the same initial workbook and identities. The raw source hash and
machine record identify the run; repeated measurements do not imply a controlled
hardware comparison with other machines.

Production workbook limits remain unchanged: default 1,024 live cells and 1 MiB
logical bytes, with constructor hard ceilings including 4,096 live cells. Core
WorkingModel itself has no analogous live-node admission cap. The private opt-in
bulk builder raises workbook caps to N+2,048 live / N+4,096 allocated cells, 1 GiB
logical estimate, 5,000 read area, and sufficient returned-record capacity.
Source, nesting, token and edit-batch limits remain intact. Bulk setup skips
per-cell transaction staging and quota scans, then validates its final budget.
Thus construction is labeled bulk_construct, not ordinary client import throughput.
Measured edits use the real apply path. Raising caps is experimental admission,
not evidence of a scalable production product. No file, RPC or transport cap changes.

## Sampling and accounting

Latency mode disables allocator/phase timing counters. It retains the opt-in
mode-check branches and System allocator wrapper for both implementations; this
is the baseline algorithm in the research build, not a claim of an identical
uninstrumented production binary. Normal operations receive
three untimed warmups and 101 measured samples. p50 is the middle sorted sample;
p95 uses nearest rank; worst is the largest observed sample. Graph construction is
outside edit/capture timings. Returned captures/read results are dropped after the
timer; commits still include destruction of replaced mutable state. One construction
and one 16-generation retention sample are diagnostics, not percentile evidence.
Affected-output capture/evaluation has at most 31 samples because dense fanout is
substantial; report this smaller count without treating its tail as a stable SLA.

The shared capture loop measures 1,000 captures per sample to avoid timer-resolution
zeros; divide the three latency columns by 1,000 for per-capture amortized costs.
It includes capture return/drop and black-box overhead. Baseline captures are
individually timed; no false sub-clock-resolution single-capture percentile is claimed.

Separate profile mode takes five samples with a System-backed counting allocator
and nested phase timers enabled. It reports allocations/allocated bytes, live requested
heap bytes, peak extra requested heap, retained requested-byte deltas and attributed
phase allocation/time. These are instrumented profiles, not uninstrumented latency
numbers. The runner samples OS peak RSS too. Requested heap excludes allocator
metadata/reserved arenas, stacks and DLL/code mappings; RSS includes more and may
retain freed arenas. Whole-process RSS high-water is not a per-operation RSS delta.

Instrumented payload-copy bytes count core WorkingNode/SnapshotNode and worksheet
Record owned payloads and prototype Patch clones. They exclude BTree internal moves,
name/index-map duplication, Vec reallocation, cache-line traffic and allocator copies.
They are explicit logical payload-volume counters (including sized record extents),
not total physical memory-bus or memcpy bytes. Compiler treatment of enum padding
and inactive payload is not traced byte-for-byte. Allocated bytes are measured separately and never relabeled bytes copied.
Phase timings nest (sheet_stage includes core_stage; snapshot includes dependency_order;
apply includes bind/quota/commit). Do not sum nested phase totals.

## Bounded alternative

The feature-gated pivot-workbook::scaling module retains an immutable whole workbook
and its validated core ModelSnapshot as shared topology. Input/Blank updates to existing
input anchors use an immutable path-copy treap keyed by stable CellId. Each atomic
batch builds a private root; unchanged batches and failures do not publish it. Capture
pins topology/root/version through Arc. The tree gives expected logarithmic work in
the number of patched cells; random UUID-derived priorities are not adversarial
worst-case balancing guarantees. A full-overlay sweep outside single-edit timing
populates every eligible input before another edit/capture measurement.

Cell/node identities, source/reference metadata and dependency indexes stay in the
shared base. Input/Blank content overrides preserve exact DecimalInput metadata.
Fast-path edits cannot change logical live/allocated/source/token quotas; physical
overlay/snapshot memory amplification is measured separately. Source/quota/stale
preflight remains enforced. All structural edits, including input/formula conversion,
move/retire/new anchors and invalid formulas, materialize a complete accepted Workbook,
apply through its atomic API, rebuild validation/snapshot and publish a new topology.
Invalid topology remains an editable draft and capture still fails globally.

The prototype uses its own explicit Version tag, not a fabricated core Revision.
Its viewport returns that authoritative version alongside the base RectangleRead;
the latter's embedded Expected tag describes the base, not the overlay generation.
No protocol or production client exposes this research API.

Evaluation traverses only the requested dependency closure, reads values from the
captured overlay, and calls the production postfix expression executor. There is
no second decimal implementation, parser or formula evaluator. Scheduler/lookup
changes are feature gated; default evaluator behavior is unchanged. No mutable
cache is shared by calculations. Results for unaffected requested regions avoid
traversing every node, but there is no persisted result cache or automatic dirty
output scheduling. Asking for all outputs still recomputes them. Structural dependency
indexes are reused for value edits, not maintained incrementally for formula edits.

Topology compilation is a separately measured cost, not free initialization.
Structural edits eagerly rebuild validation here while the baseline defers that
work to capture, so formula_replace timings have different validation placement;
report that distinction instead of claiming direct formula speedup. Existing input
edits and update_capture_evaluate have matching end-to-end semantics.

## Correctness and interpretation

Tests compare accepted workbook values, stable existing identities, bound target
runtime identities, structural diagnostics and source spans across edits. New UUIDs
allocated independently by two branches are compared by binding meaning, not forced
equal. Closed-form workload results provide an independent numeric oracle. Tests
retain S while T changes inputs/topology, cover blank/retire/reuse/repair, editable
invalid drafts, cycles, decimal errors and exact representations, stale/quota failures,
net no-ops, many patched keys and calculations on another thread.

The prototype is deliberately not a universal storage redesign. A successful input
fast path demonstrates a possible scaling direction; it does not certify formula
editing, reverse invalidation, caches, structural snapshot memory, production limits
or future ranges. Report A (bounded optimization sufficient), B (substantial storage/
mutation/snapshot restructuring needed), or C (not currently viable) from evidence.
Stop for review before C# work or production adoption.

## Changing-key control

```sh
cargo run --release --locked -p pivot-workbook --features scaling-experiment --example scaling-random
```

This additional control uses 101 distinct deterministic input indices after three
warmups, selected by `(65537*i + 13) % (N-1)`. It excludes the final formula cell.
The shared overlay is populated at every input and restored to the baseline's
values before either timer. Baseline/prototype receive identical edits, and all
cell views are compared afterward at every size. This distinguishes hot repeated
key costs from pointer/cache behavior across a populated input set. Setup, overlay
population and final comparisons remain outside timed edits.

## Optional million-cell diagnostic and native sampling

```sh
python scripts/scaling.py --sizes 1000000 --shapes independent --samples 1 --modes latency --timeout-seconds 600 --max-rss-mib 8192 --output-dir artifacts/scaling-million
```

This is a resource-bounded diagnostic, not a statistical run. The submitted Windows
attempt stopped at the RSS budget while retaining baseline generations; no shared
1M result exists. The main runner defaults to an 8 GiB RSS / 1,800-second budget per
mode and records a stop as failed rather than silently skipping work.

Native Linux sampling used another optimized build with debug information:

```sh
CARGO_TARGET_DIR=target/linux-scaling-perf CARGO_PROFILE_RELEASE_DEBUG=1 cargo build --release --locked -p pivot-workbook --features scaling-experiment --example scaling
perf record -F 199 -e cpu-clock:u --call-graph dwarf -o /var/tmp/pivot-scaling-profile-20261019/perf.data -- target/linux-scaling-perf/release/examples/scaling --sizes 100000 --shapes mixed --samples 11
perf report --stdio --no-children --percent-limit 0 --sort dso,symbol -i /var/tmp/pivot-scaling-profile-20261019/perf.data > artifacts/scaling-linux-perf-offsets.txt
python3 scripts/scaling_symbolize.py artifacts/scaling-linux-perf-offsets.txt target/linux-scaling-perf/release/examples/scaling artifacts/scaling-linux-perf-symbolized
```

Record data on native Linux storage: direct writes to the mounted J: drive failed
with Bad address in this environment. Copy the resulting data/reports back to shared
artifacts. The installed perf lacks libunwind/libdw support; offline addr2line gives
individual sampled-IP inline locations, not non-inline caller stacks. The workload
includes setup and both implementations; per-operation attribution comes from the
separate phase/allocator profiles. Do not interpret whole-run sampling percentages
as operation-specific latency shares. Build/sample logs retain failed attempts too.

Run `python scripts/scaling_analyze.py RUN_DIRECTORY` to normalize 1,000-call
capture batches, convert unmeasured latency-mode memory fields to null, and emit
metrics.json/analysis.md. Raw CSVs and source/environment manifests remain unchanged.
