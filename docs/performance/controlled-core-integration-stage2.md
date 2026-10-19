# Controlled core integration — Stage 2 evidence

Date: 2026-10-19. Release experiments through WorkingModel, not the research Graph
API. The reference backend remains the default. Native macOS ARM64 Stage 2 also passed; see the native validation addendum below.

## Reproduction and method

```text
python scripts/integration_stage2.py --output-dir artifacts/stage2
cargo clippy --locked --workspace --all-targets --features pivot-core/core-integration -- -D warnings
cargo clippy --locked --workspace --all-targets --features pivot-core/core-integration,pivot-workbook/scaling-experiment -- -D warnings
cargo test --locked --workspace --features pivot-core/core-integration,pivot-workbook/scaling-experiment
python scripts/ci.py
```

Use separate Windows/Linux target directories as documented in local CI. WSL file
tests require disk-backed PIVOT_FILE_TEST_ROOT. Run benchmarks sequentially, without
regression builds overlapping latency measurements. Each platform executes thirty
isolated processes: 1K/10K/100K nodes, two backends and five cases. Each row has three
warmups then 31 measurements, nearest-rank p95; median/max and allocation bytes/calls
are in raw CSV. Release uses ordinary opt-level=3, no custom LTO or target CPU.
The counting allocator remains active for both implementations.

Fixtures have named input nodes and a 1,000-formula chain (997 at 1K to leave three
inputs). Formula edits cover front/middle/tail. Edits include private staging,
installation, local capture, selected calculation where applicable, and disposal.
Invalid repair includes submitting an invalid draft, observing a local failure,
repairing and calculating. The initially-invalid case activates a draft already
containing a parse error; it is an in-memory restore bridge, not an oversized
portable import. Independent integer expectations check results.

Wide ports use 1,000 declared inputs at 10K/100K, with named default invocation;
1K has three inputs. Rename/retire scenarios retain allocation ledgers. Retention
measures 16/32 structural captures after formula changes, settling cleanup before
reading extra live requested heap. Reference local capture stages the full source;
persistent local capture pins roots. Bounded inspection compares pinned draft
headers against a staged reference draft. Complete snapshot/diagnosis measures the
accepted complete contract independently. Portable transfer is measured only at
1K; 10K/100K verify frozen-limit refusal.

Contention runs two whole-snapshot producer threads on an independent model while
measuring mutation/capture. It measures real CPU/allocator contention, not a hard
real-time scheduling guarantee. Allocation counters are process-wide, so contention
row allocations include the background workload. Fault tests separately hold both
heavy workers and blocked cleanup while asserting mutation progress.

The runner guards each child at 8 GiB RSS and 600 seconds and records source hashes,
compiler/platform, command, exit code and OS resident high-water observations.
RSS sampling every 50 ms may miss very short processes. Requested allocator bytes,
conservative allocation-domain reservations and OS RSS are different measurements.
Both backends instantiate the same fixed runtime for comparable process overhead.
The persistent path additionally uses it for work. The process recovery worker is
fixed overhead outside the domain.

## Interpretation and budgets

The original valid-operation gates remain: 100K input edit/capture p95 below 5 ms,
formula edit/local capture and selected calculation below 10 ms, and input update
plus a 1,000-formula chain below 25 ms.

Proposed separate regression budgets on this hardware: invalid submission/repair
and initially-invalid repair below 10 ms; bounded 256-header inspection below 5 ms;
1,000-port edits/invocation below 10 ms; complete 100K snapshot or diagnosis below
500 ms; initial 100K activation below 2 seconds; admitted 1K portable export/import
below 100 ms. These are measured review proposals, not portable latency assertions
in ordinary CI. Whole-graph operations remain proportional to graph size.

Full snapshot materialization is slower than reference snapshot construction because
it reconstructs compatibility containers and validates them. This is a disclosed
tradeoff; callers needing interactive inspection should use the bounded view.
Conservative full-copy reservations can greatly exceed live requested heap. Runtime
admission is not an RSS cap. No higher public capacity is implied by these runs.

Cleanup reporting includes last-owner enqueue, explicit drain, worker joining and
zero final domain charge. TOTAL_CLEANUP sums timed destructor wall intervals across
workers, not CPU time; they can overlap. Reference inline destructor time appears in
the enqueue/drop measurements rather than runtime cleanup counters. Long destruction
is moved off the submitter, not eliminated. Final root reclamation remains O(N).

Measured tables and final validation records follow.

## Measured results

All 30 processes passed on each platform; every source-manifest hash matches the final source.

- Windows: `artifacts/stage2-windows-final/20261019T172219Z` (environment, source hashes, CSVs, summary).
- Linux: `artifacts/stage2-linux-final/20261019T172830Z` (environment, source hashes, CSVs, summary).

Persistent p95 in milliseconds (31 samples):

| Operation | Windows 1K | Windows 10K | Windows 100K | Linux 1K | Linux 10K | Linux 100K |
| --- | ---: | ---: | ---: | ---: | ---: | ---: |
| unchanged_capture | 0.0009 | 0.0031 | 0.0011 | 0.0005 | 0.0011 | 0.0008 |
| input_capture | 0.0268 | 0.0586 | 0.0260 | 0.1044 | 0.1184 | 0.1427 |
| input_chain | 0.7139 | 0.9581 | 0.6703 | 0.7477 | 0.5562 | 0.6411 |
| formula_front | 0.6512 | 0.8157 | 0.5848 | 0.5924 | 0.5996 | 0.6235 |
| formula_middle | 0.8817 | 1.1024 | 0.7075 | 0.7798 | 0.7079 | 0.6632 |
| formula_tail | 1.0639 | 1.3409 | 0.9549 | 0.8941 | 0.7723 | 0.8441 |
| rename | 0.7869 | 0.6691 | 0.5887 | 0.5293 | 0.5140 | 0.5483 |
| add_retire | 0.7231 | 0.7208 | 0.6144 | 0.7593 | 0.7594 | 0.8091 |
| invalid_draft_repair | 0.9991 | 0.9678 | 0.8617 | 0.9311 | 1.0116 | 1.0638 |
| named_add_formula | 0.0663 | 0.0722 | 0.1424 | 0.2409 | 0.2344 | 0.2524 |
| port_update_invoke | 0.7361 | 1.7776 | 1.3256 | 0.6086 | 2.1129 | 1.1764 |
| port_create_rename_remove | 0.7086 | 2.6540 | 2.9285 | 0.8665 | 2.7978 | 3.4255 |
| initially_invalid_repair | 1.1554 | 1.0067 | 0.9332 | 0.9510 | 0.9174 | 1.1094 |
| inspection_capture_page | 0.0086 | 0.0096 | 0.0106 | 0.0051 | 0.0086 | 0.0093 |
| reverse_edge_page | 0.0005 | 0.0005 | 0.0006 | 0.0004 | 0.0004 | 0.0003 |
| full_snapshot | 2.0816 | 12.4873 | 152.7114 | 1.5771 | 9.0118 | 129.1656 |
| complete_diagnosis | 2.0941 | 13.3642 | 157.4114 | 1.2900 | 9.4225 | 115.1039 |
| input_capture_contended | 0.0527 | 0.0463 | 0.0811 | 0.1094 | 0.1529 | 0.1477 |

100K reference versus persistent p95 (milliseconds):

| Operation | Windows reference | Windows persistent | Linux reference | Linux persistent |
| --- | ---: | ---: | ---: | ---: |
| input_capture | 80.673 | 0.026 | 68.448 | 0.143 |
| input_chain | 80.180 | 0.670 | 58.892 | 0.641 |
| formula_tail | 77.205 | 0.955 | 59.410 | 0.844 |
| initially_invalid_repair | 111.135 | 0.933 | 87.226 | 1.109 |
| port_update_invoke | 82.222 | 1.326 | 62.483 | 1.176 |
| inspection_capture_page | 37.847 | 0.011 | 40.325 | 0.009 |
| full_snapshot | 101.913 | 152.711 | 76.952 | 129.166 |
| complete_diagnosis | 85.854 | 157.411 | 70.300 | 115.104 |
| input_capture_contended | 103.258 | 0.081 | 80.489 | 0.148 |

Admitted 1K transfer p95 (ms):

| Operation | Windows reference | Windows persistent | Linux reference | Linux persistent |
| --- | ---: | ---: | ---: | ---: |
| portable_export | 0.389 | 1.449 | 0.284 | 0.782 |
| portable_import_calculate | 3.930 | 8.594 | 2.530 | 7.097 |

100K requested heap and allocation evidence (MiB, except calls):

| Platform/backend | Settled live edits | Peak edits | Domain peak edits | 32 captures extra | Input/capture allocated bytes / calls |
| --- | ---: | ---: | ---: | ---: | ---: |
| windows/reference | 32.78 | 1009.93 | 1.69 | 947.5407 | 62,050,362 / 470,275 |
| windows/integrated | 43.28 | 71.52 | 142.16 | 0.1273 | 5,816 / 30 |
| linux/reference | 32.78 | 1009.93 | 1.69 | 947.5407 | 62,050,362 / 470,275 |
| linux/integrated | 43.29 | 71.52 | 142.16 | 0.1273 | 5,816 / 30 |

100K final root disposal (milliseconds); final domain charge was zero in every process:

| Platform/backend | Last-owner drop/enqueue | Drain | Finish/join | Edits cumulative runtime destructor wall | Inspection cumulative destructor wall |
| --- | ---: | ---: | ---: | ---: | ---: |
| windows/reference | 23.648 | 0.001 | 0.601 | 0.000 | 0.000 |
| windows/integrated | 0.004 | 43.545 | 0.295 | 69.527 | 1257.445 |
| linux/reference | 44.162 | 0.002 | 0.867 | 0.000 | 0.000 |
| linux/integrated | 0.019 | 58.055 | 4.859 | 82.492 | 699.717 |

Windows: maximum observed process RSS 1108.5 MiB across both backends; maximum 100K activation 641.5 ms. Linux: maximum observed process RSS 1191.1 MiB across both backends; maximum 100K activation 568.6 ms. All original valid-operation gates and proposed broader-operation budgets passed on both platforms.

RSS high-water and worst conservative domain reservation across all cases (MiB):

| Platform/backend | Observed RSS | Domain reservation peak |
| --- | ---: | ---: |
| windows/reference | 1108.54 | 1.69 |
| windows/integrated | 236.09 | 1269.83 |
| linux/reference | 1191.11 | 1.69 |
| linux/integrated | 284.56 | 1071.84 |

The largest reservation occurs under concurrent complete snapshots, not local edits. Reference allocations are not domain-managed; its domain column contains the common runtime overhead only. At 100K, sixteen retained formula captures add about 0.062 MiB persistent requested heap versus 473.77 MiB for staged reference captures; the thirty-two-capture figures are above.

## Final validation

| Gate | Windows | Linux WSL2 |
| --- | --- | --- |
| Combined feature tests | 259 passed | 260 passed |
| Default Rust workspace tests | 209 passed | 210 passed |
| Strict Clippy: integration-only / research-only / combined | Passed | Passed |
| Full default CI: format, lint, Rust, .NET, parity and real IPC | Passed | Passed |
| Release experiments | 30/30 passed | 30/30 passed |

- Windows default CI: `artifacts/stage2-ci-windows/20261019T174134408808Z/summary.json`.
- Windows feature evidence: `artifacts/stage2-final-windows-tests.log`, plus `stage2-final-windows-core-lint.log`, `research-lint.log` and `combined-lint.log` in artifacts.
- Linux default CI: `artifacts/stage2-ci-linux/20261019T174245802849Z/summary.json`.
- Linux feature evidence: `artifacts/stage2-final-linux-tests.log`, plus `stage2-final-linux-core-lint.log`, `research-lint.log` and `combined-lint.log` in artifacts.

.NET Release builds had zero warnings/errors. All 10,099 decimal vectors had zero value/error failures and scale differences; protocol fixtures included all 58 decimal layouts. Real named-pipe/Unix-socket daemon and CLI integration passed. No frozen storage/protocol fixtures, dependencies, limits, workbook code or service publication code changed. Format, UTF-8/LF, Markdown links/fences, Python syntax and diff whitespace checks passed. Native macOS Stage 2 results are recorded below; prior Slice B native results remain historical.

## Native macOS validation addendum

Evidence: [full native report](../../artifacts/macos-validation-20261019T182116Z/REPORT.md),
[summary](../../artifacts/macos-validation-20261019T182116Z/summary.json), and
[raw benchmark summary](../../artifacts/macos-validation-20261019T182116Z/stage2-release/20261019T182409Z/summary.json).
Apple M1 / Macmini9,1, eight CPUs, 16 GiB RAM, macOS 27.0.1 ARM64,
Rust 1.101.0-nightly (b6eeb93c9), .NET SDK 10.0.402, Python 3.9.6.
All work ran from a copied dirty working tree on internal APFS under
`/private/tmp/pivot-stage2-20261019T182116Z.8nejvyzy`; only reports were returned
to SMB. Source, target, temporary file-test root and initial evidence were local.
All 265 copied source hashes matched the reviewed source before this documentation
update, including benchmark inputs. No product/configuration changes or reruns.

Full default CI passed: 210 default Rust tests, warning-free .NET Release build,
10,099 decimal vectors with zero value/error/scale differences, protocol parity
including 58 decimal layouts and frozen-v1.0 compatibility, and actual daemon/CLI/
REPL Unix-socket integration. Integration-only tests passed 237, research-only 234,
combined 260; all three strict feature lint configurations passed.
Thirty benchmark children passed with 31 samples per row, no timeout/budget stop,
and zero final domain charges. All original and broader proposed targets passed.

Native persistent p95 in milliseconds:

| Operation | 1K | 10K | 100K |
| --- | ---: | ---: | ---: |
| unchanged_capture | 0.0047 | 0.0042 | 0.0032 |
| input_capture | 0.0107 | 0.0138 | 0.0143 |
| input_chain | 0.3903 | 0.3409 | 0.3176 |
| formula_front | 0.3578 | 0.3223 | 0.3337 |
| formula_middle | 0.4414 | 0.3856 | 0.4431 |
| formula_tail | 0.4970 | 0.4672 | 0.4849 |
| rename | 0.3337 | 0.3197 | 0.3180 |
| add_retire | 0.3248 | 0.3343 | 0.3295 |
| invalid_draft_repair | 0.4908 | 0.4923 | 0.4890 |
| named_add_formula | 0.0335 | 0.0341 | 0.0377 |
| port_update_invoke | 0.3532 | 0.7376 | 0.7146 |
| port_create_rename_remove | 0.3513 | 1.0203 | 1.0171 |
| initially_invalid_repair | 0.5968 | 0.5310 | 0.4965 |
| inspection_capture_page | 0.0057 | 0.0067 | 0.0083 |
| reverse_edge_page | 0.0002 | 0.0002 | 0.0002 |
| full_snapshot | 0.9426 | 4.4497 | 50.4763 |
| complete_diagnosis | 0.7904 | 4.3700 | 80.4300 |
| input_capture_contended | 0.0495 | 0.0343 | 0.0389 |

At 1K, portable export/import-calculation p95 was 0.379/3.445 ms. Maximum
100K activation was 278.904 ms. Complete native reference snapshot p95 was
30.402 ms versus 50.476 ms persistent; complete diagnosis was 31.805 versus
80.430 ms. Native results reinforce the existing bounded-inspection contract,
without making full materialization constant-time or changing frozen limits.

At 100K, sixteen/thirty-two retained formula captures added 64,770/133,474
requested bytes. The edits case settled at 45,400,302 requested bytes, peaked
at 75,003,332, and reserved a domain peak of 149,066,751. Final root enqueue,
drain and join took 0.0012/15.375/0.105 ms; cumulative edits-case destructor wall
intervals totaled 25.579 ms. Cleanup work is moved off the caller, not eliminated.
Full parsed allocation/retention/cleanup evidence is in the native report.

The unchanged runner reports zero RSS on macOS because its helper is unsupported;
those values are unavailable and its RSS guard was ineffective. Separate native
monitoring recorded seven samples with normal pressure and a sampled benchmark
process maximum of 323.36 MiB. This is neither a per-child high-water mark nor
directly comparable to Windows/Linux RSS. No equivalent native high-water or RSS-cap
enforcement claim is made. AC power and no reported thermal/performance warning
were recorded. Ninety Stage 2 benchmark processes now passed across three platforms.
