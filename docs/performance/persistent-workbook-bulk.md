# Persistent workbook and bulk mutation evidence

Date: 2026-10-19. Ordinary optimized Release, counting System allocator enabled.
This is a bounded opt-in integration, not a production workbook backend switch.

## Reproduce

After correctness gates and all other builds/tests have finished:

```sh
python scripts/bulk_workbook.py --output-dir artifacts/bulk-workbook
python scripts/bulk_workbook.py --shapes long-chain --layers new,existing --implementations reference,batch --samples 1 --output-dir artifacts/bulk-long-chain
python scripts/bulk_workbook.py --sizes 200000 --shapes independent --layers new --implementations batch --samples 1 --output-dir artifacts/bulk-200k
```

Main matrix: 96 sequential children per platform, three construction samples each.
Core new and existing graphs compare explicit reference, persistent individual
edits, and one persistent batch. Existing fixtures add N nodes to N existing nodes.
Workbooks compare an optimized reference experiment coordinator with persistent
integration, using independent numeric, mixed, bounded 1,000-link chain and fan-out
fixtures. The reference experiment checks final quotas once for these monotonic
fixtures; production reference and correctness-oracle transactions retain per-edit
checks. This avoids an artificial repeated full-statistics scan in the benchmark.
It is not a claim that production reference loading already has this optimization.

Construction measures request creation, parsing/binding, validation and publication;
core constructor/runtime creation precedes that timer. Capture and evaluation have
separate columns. Complete core capture is intentionally used: reference materializes
arrays and persistent can reuse a certified root. Three construction samples support
median/range observations, not a robust tail-latency estimate. Edit rows have 31
samples following three warmups, with nearest-rank p95. Allocation columns are
requested bytes/calls, not retained memory or RSS.

Workbook ADD and UPDATE cover 100/1K/10K cells, three samples each; FORMULAS covers
10K replacements. Later edit/read rows run after adding 33,300 cells to initial N.
Old captures are independently checked across all these transactions. Retention
records 16 generations (core additionally 32), caller release/enqueue time, cleanup
work/drain, final join and zero domain charge. Root release is reported separately
from off-thread destruction. Queue peaks mean accepted pending-not-started tasks,
including up to two receiver-assigned tasks; they are not literal channel lengths.

Full 100K long-chain closure is a separate diagnostic, not the 1,000-chain interactive
target. The optional 200K numeric core run remains inside the batch cap; neither
raises production workbook limits nor establishes million-cell viability. The runner
retains 600-second/8-GiB child guards. Windows/Linux RSS is sampled, not exact HWM;
macOS RSS helper is unavailable, so native monitoring must be separate.

## Validation boundaries

Windows combined regression: 296 tests; Linux: 297. True core fallback: 21 each.
Strict combined-feature workspace lint and formatting pass. The shared accepted
workbook formula/projection suites run through both backends. Additional tests cover
exact differential revisions/metadata/results, atomic rollback, final-name binding,
stale same-counter roots, admission/cancellation, bounded copy-out and retired/reused
identity. Held heavy-worker tests prove bounded backpressure, queued cancellation
and independent short edits. Partial-candidate cancellation observes cleanup-thread
reclamation and zero final charge.

The initial held-worker test had a setup race: concurrent submissions could fill its
capacity-one queue before both workers started, and a panic left its gates closed.
The corrected test waits for each worker start and releases gates on unwind. Both
platforms pass the corrected test. Initial Windows sandbox IPC failures were rerun
with real named-pipe permission; failure logs are retained. No product IPC change.

Native macOS now validates this source from a fresh local APFS copy; see the native
results below. The [APFS packet](../handoffs/2026-10-19-macos-persistent-workbook-bulk-instructions.md) remains the reproduction procedure.

## Measurements

Matched results below retain raw source/environment manifests and all child logs. Native macOS results are included below.

### Windows matched matrix

[Raw 96-process summary](../../artifacts/bulk-final-windows/20261019T213547Z/summary.json).
[Isolated 1K numeric repeat](../../artifacts/bulk-isolated-1k-windows/20261019T214558Z/summary.json) replaces the first numeric group in timing comparisons: the original briefly overlapped the tail of Linux regression. All other timing runs were isolated.

Median core construction milliseconds (three samples; request/parse/bind/validate/install):

| Nodes added | Shape | New ref | New individual | New batch | Existing ref | Existing individual | Existing batch |
|---:|---|---:|---:|---:|---:|---:|---:|
| 1,000 | independent | 0.64 | 32.60 | 4.32 | 0.64 | 35.18 | 4.78 |
| 1,000 | mixed | 1.01 | 40.70 | 5.63 | 0.83 | 40.50 | 6.07 |
| 1,000 | chain | 1.53 | 181.44 | 8.81 | 1.58 | 183.94 | 9.42 |
| 1,000 | fanout | 1.33 | 60.63 | 10.20 | 1.44 | 59.38 | 10.37 |
| 10,000 | independent | 7.60 | 345.57 | 50.65 | 7.60 | 378.13 | 55.39 |
| 10,000 | mixed | 9.39 | 384.19 | 62.41 | 9.17 | 384.48 | 68.62 |
| 10,000 | chain | 8.80 | 464.74 | 55.59 | 8.81 | 481.78 | 60.24 |
| 10,000 | fanout | 15.11 | 644.57 | 115.67 | 17.85 | 628.36 | 119.76 |
| 100,000 | independent | 84.24 | 3279.74 | 579.42 | 90.89 | 3284.46 | 625.34 |
| 100,000 | mixed | 102.67 | 6059.46 | 717.10 | 109.35 | 6045.94 | 783.74 |
| 100,000 | chain | 87.44 | 5236.11 | 586.68 | 90.87 | 5150.52 | 621.33 |
| 100,000 | fanout | 158.94 | 9212.52 | 1236.35 | 163.07 | 9596.78 | 1328.50 |

Existing cases start with N inputs, then add N nodes/formulas (final 2N). Bounded chain means at most 1,000 links. At 100K, core batches improve construction roughly 5–9x over individual persistent edits, while remaining about 6–8x slower than reference mutation alone. Complete capture/evaluation increases reference ready time; raw CSVs retain these separate phases.

Median workbook construction milliseconds; optimized reference research coordinator:

| Initial cells | Shape | Reference | Persistent |
|---:|---|---:|---:|
| 1,000 | independent | 1.58 | 14.14 |
| 1,000 | mixed | 2.38 | 15.31 |
| 1,000 | chain | 3.44 | 18.37 |
| 1,000 | fanout | 3.38 | 19.32 |
| 10,000 | independent | 18.95 | 176.93 |
| 10,000 | mixed | 24.54 | 192.76 |
| 10,000 | chain | 21.36 | 187.58 |
| 10,000 | fanout | 37.89 | 238.85 |
| 100,000 | independent | 249.96 | 2516.50 |
| 100,000 | mixed | 291.56 | 2738.98 |
| 100,000 | chain | 246.00 | 2543.34 |
| 100,000 | fanout | 408.73 | 3174.86 |

100K workbook construction is 2.52–3.17 s, roughly 8–10x slower than the optimized reference coordinator. This is a measured remaining cost, not hidden by interactive timings.

100K-start workbook transactions (numeric fixture), median of three samples:

| Operation | 100 cells | 1K cells | 10K cells |
|---|---:|---:|---:|
| ADD | 3.03 ms | 25.31 ms | 270.39 ms |
| UPDATE | 0.64 ms | 7.46 ms | 74.20 ms |

10K formula replacements: 111.02 ms median. All reported persistent bulk calls dispatch once. Post-addition edit/read population is 133,300 cells (plus the later scratch cell).

100K-start persistent workbook p95 milliseconds, 31 samples:

| Shape | Input/capture | Input/selected calculation | Short formula | Rectangle |
|---|---:|---:|---:|---:|
| independent | 0.054 | 0.104 | 0.068 | 0.623 |
| mixed | 0.143 | 0.087 | 0.055 | 0.854 |
| chain | 0.045 | 1.184 | 0.084 | 0.662 |
| fanout | 0.039 | 0.037 | 0.046 | 1.163 |

All existing interactive targets pass on Windows. Numerical and selected-output assertions pass independently; no complete graph is copied by these persistent local rows.

The [12 full-chain diagnostics](../../artifacts/bulk-long-chain-windows/20261019T214608Z/summary.json) and [guarded 200K numeric run](../../artifacts/bulk-200k-windows/20261019T214732Z/summary.json) also pass. Full 100K chain construction is 1.076 s new / 1.147 s existing, with selected evaluation about 91 ms new (separate from the 1,000-chain gate). The 200K numeric core batch constructs in 1.209 s, with one dispatch and zero final domain charge. These single-sample diagnostics are not tail estimates.


Windows 100K-start workbook resource evidence (all four shapes, including later
133,300-cell population): peak requested heap 184.7–242.7 MB, peak allocation-domain
charge 339.9–840.2 MB, sampled child RSS 207.4–273.7 MiB. These are different
measurements; domain reservations deliberately exceed retained payload and are not
an exact RSS ceiling. Sixteen retained changed-formula generations add about
101–104 KB requested live heap after cleanup, rather than full worksheet copies.
Caller final-root enqueue costs 4.3–8.2 microseconds; individual cleanup tasks can
cost 109.6–153.5 ms and final drain 187.6–242.9 ms. That work is explicitly off the
caller, not absent. Runtime pending cleanup peaks at 548–556 of 1,088 reserved slots.
Ordinary benchmark mutation/heavy pending peaks are one with no rejections;
controlled queue tests separately force saturation/cancellation. All final domain
charges are zero. Reference captures retain complete copies and reach higher sampled
RSS (up to 2.54 GiB for 200K existing-graph fan-out); this matrix includes retained
generations, so RSS must not be described as construction-only usage.

## Proposed bulk acceptance budgets

For this in-memory foundation, propose the following review budgets, measured with
this harness on an otherwise idle supported desktop (not permanent CI assertions):

| Work | Proposed budget |
|---|---:|
| 100K numeric canonical core batch | 1 second |
| 100K mixed/fan-out core population | 2 seconds |
| Workbook population 1K / 10K / 100K | 100 ms / 500 ms / 5 seconds |
| Numeric addition 100 / 1K / 10K cells to populated workbook | 10 ms / 50 ms / 500 ms |
| 10K formula replacement/binding | 500 ms |

These retain substantial headroom over Windows observations, while requiring the
bulk path to remain materially faster than repeated persistent edits. Native macOS
now also passes these budgets; they remain proposals for review. They include in-memory
request preparation/binding/publication, not file I/O, decompression or a future
workbook container. Three-sample construction medians are a first bound, not a
p95 service guarantee. Existing single-edit p95 targets remain unchanged. The
remaining 8–10x Windows workbook/reference construction gap merits future profiling
if population becomes product-critical; it does not justify a second graph engine
or switching the production workbook default in this milestone.

Cancellation/backpressure evidence is controlled and bounded, not a cancellation
p95 benchmark: two heavy workers are held, one 1,024-node request is queued then
canceled, another is rejected, and a short mutation still completes. Assertions
require one rejection, pending peak no greater than three, unchanged rejected
writers and zero final charges. The corrected isolated scenario completes in 0.00 s
Windows / 0.03 s Linux at the test runner's coarse resolution; this includes the
whole setup/release scenario and is not a per-cancel latency measurement. A separate
10K partial-construction cancellation observes cleanup off the caller. Cancellation
polls between operations/validation visits; parsing one bounded formula and caller
worksheet preparation are not preempted instruction by instruction.

### Linux WSL2 matched matrix

[Raw 96-process summary](../../artifacts/bulk-final-linux/20261019T214819Z/summary.json). Same source/harness and ordinary Release settings; no concurrent regression builds.

Median core construction milliseconds:

| Nodes added | Shape | New ref | New individual | New batch | Existing ref | Existing individual | Existing batch |
|---:|---|---:|---:|---:|---:|---:|---:|
| 1,000 | independent | 0.59 | 118.98 | 3.82 | 0.50 | 111.40 | 3.94 |
| 1,000 | mixed | 0.59 | 139.12 | 4.67 | 0.73 | 152.24 | 4.89 |
| 1,000 | chain | 1.32 | 357.95 | 7.85 | 1.39 | 355.11 | 7.73 |
| 1,000 | fanout | 0.99 | 249.69 | 8.46 | 0.98 | 241.58 | 9.46 |
| 10,000 | independent | 5.75 | 1162.71 | 43.52 | 5.71 | 1165.53 | 43.01 |
| 10,000 | mixed | 6.76 | 1391.83 | 53.29 | 6.84 | 1447.03 | 52.33 |
| 10,000 | chain | 6.53 | 1398.07 | 46.44 | 6.44 | 1384.70 | 46.90 |
| 10,000 | fanout | 11.29 | 2284.51 | 95.10 | 11.66 | 2320.76 | 93.39 |
| 100,000 | independent | 64.40 | 12217.20 | 499.41 | 66.12 | 12123.91 | 503.28 |
| 100,000 | mixed | 79.53 | 14188.14 | 603.11 | 79.48 | 14369.86 | 630.72 |
| 100,000 | chain | 65.12 | 12425.04 | 488.80 | 64.01 | 11887.95 | 515.27 |
| 100,000 | fanout | 117.98 | 23193.49 | 1031.72 | 118.28 | 22886.86 | 1080.90 |

Median workbook construction milliseconds:

| Initial cells | Shape | Reference | Persistent |
|---:|---|---:|---:|
| 1,000 | independent | 1.25 | 11.44 |
| 1,000 | mixed | 1.61 | 12.24 |
| 1,000 | chain | 2.32 | 15.14 |
| 1,000 | fanout | 2.18 | 15.29 |
| 10,000 | independent | 15.92 | 145.24 |
| 10,000 | mixed | 17.90 | 157.90 |
| 10,000 | chain | 16.60 | 146.70 |
| 10,000 | fanout | 25.76 | 195.82 |
| 100,000 | independent | 222.83 | 1961.10 |
| 100,000 | mixed | 250.67 | 2126.40 |
| 100,000 | chain | 215.37 | 1911.32 |
| 100,000 | fanout | 318.99 | 2441.90 |

100K-start numeric workbook batch medians:

| Operation | 100 cells | 1K cells | 10K cells |
|---|---:|---:|---:|
| ADD | 4.94 ms | 30.43 ms | 250.61 ms |
| UPDATE | 1.71 ms | 29.29 ms | 123.39 ms |

10K formula replacements: 187.97 ms median. Bulk calls dispatch once.

100K-start persistent workbook p95 milliseconds (later population 133,300):

| Shape | Input/capture | Input/selected calculation | Short formula | Rectangle |
|---|---:|---:|---:|---:|
| independent | 0.162 | 0.149 | 0.256 | 1.918 |
| mixed | 0.169 | 0.174 | 0.329 | 3.161 |
| chain | 0.160 | 1.808 | 0.248 | 1.261 |
| fanout | 0.146 | 0.182 | 0.207 | 3.944 |

All interactive and proposed bulk budgets pass on Linux. Core 100K batches improve about 21–25x over individual edits, while remaining about 8–9x slower than reference mutation. Workbook population is 1.91–2.44 s versus 0.22–0.32 s reference.

Peak requested heap is 184.7–242.7 MB; peak domain reservation is 339.9–840.2 MB; sampled workbook child RSS is 266.1–343.8 MiB. Sixteen generations retain 102.8–105.7 KB additional requested live heap. Caller root release is 19.9–23.7 microseconds; maximum cleanup task spans 164.8–212.7 ms and final drains 264.5–353.2 ms. All final charges are zero.


Linux [12 full-chain diagnostics](../../artifacts/bulk-long-chain-linux/20261019T220159Z/summary.json)
and [200K numeric diagnostic](../../artifacts/bulk-200k-linux/20261019T220304Z/summary.json)
pass. Full 100K chain construction is 0.922 s new / 0.934 s existing, with selected
evaluation about 79 ms new. The 200K numeric core batch constructs in 1.029 s and
finishes with zero domain charge. Main matrices plus diagnostics total 226 successful
children across Windows/Linux (including eight isolated Windows repeats).

All benchmark core/workbook/harness manifests contain the same 66 source files and
match the final working tree after normalizing Windows/Linux path separators.
Documentation is intentionally outside those manifests. Full CI covers the unchanged
remaining workspace, .NET projects and frozen compatibility fixtures separately.

## Final regression and CI

| Gate | Windows | Linux WSL2 | macOS ARM64 |
|---|---:|---:|---:|
| Full default CI | Passed | Passed | Passed |
| Default Rust tests | 248 | 249 | 249 |
| Combined all-feature Rust tests | 296 | 297 | 297 |
| True core reference fallback tests | 21 | 21 | 21 |
| Persistent-workbook feature-only tests | 48 | 48 | 48 |
| Formatting; default/combined strict lint | Passed | Passed | Passed |
| Core fallback/research-only strict lint | Passed | Passed | Passed |
| .NET Release build, 10,099 decimal vectors | Passed | Passed | Passed |
| Frozen protocol, C# codec and real IPC | Passed | Passed | Passed |

[Windows full CI](../../artifacts/bulk-ci-windows/20261019T220317680064Z/summary.json),
[Linux full CI](../../artifacts/bulk-ci-linux/20261019T220443034511Z/summary.json).
Detailed combined/fallback/feature-only logs are under artifacts/bulk-final-*.
Initial Windows sandbox IPC failure and Linux test-setup hang logs are retained;
corrected source and appropriately permitted real IPC reruns pass. No test is skipped
to obtain these results. Native macOS also passes, without source fixes, skipped
gates or reruns; [full native report](../../artifacts/macos-validation-20261019T220845Z/REPORT.md)
and [CI summary](../../artifacts/macos-validation-20261019T220845Z/ci/20261019T220904027470Z/summary.json).
Windows/Linux additionally ran the workbook-only strict lint; native coverage uses
all-feature strict lint plus the 48-test workbook-only suite.

## Native macOS ARM64 follow-up (20261019T220845Z)

[Native report](../../artifacts/macos-validation-20261019T220845Z/REPORT.md), [109-process summary](../../artifacts/macos-validation-20261019T220845Z/summary.json) and [source verification](../../artifacts/macos-validation-20261019T220845Z/source-final-verification.json). Apple M1 Mac mini, 16 GiB RAM, macOS 27.0.1, native Rust nightly 1.101.0 and .NET 10.0.402. Fresh rsync of the dirty SMB working tree to internal APFS; all source, targets, temporary files, .NET outputs and initial reports remained local. All 290 copied source hashes match our tree before this documentation update. No product/source changes or validation reruns.

All required CI/lint/test gates pass, including 249 default / 297 all-feature / 21 fallback / 48 workbook-only tests, 10,099 decimal vectors, frozen C# protocol and real Unix-socket IPC. The separately named legacy combined-feature configuration runs 272 tests; the 297 total is explicitly `--all-features`, which enables persistent Workbook.

All 96 main children, 12 full-chain children and one guarded 200K child pass. All final domain charges are zero, all bulk dispatches are one and no normal-run lane rejects work. Existing interactive and proposed bulk targets pass. Three-platform total: **335 successful children**, including eight isolated Windows repeats.

100K construction medians, milliseconds (three samples):

| Shape | New ref | New individual | New batch | Existing ref | Existing individual | Existing batch | Workbook ref | Workbook persistent |
|---|---:|---:|---:|---:|---:|---:|---:|---:|
| independent | 43.18 | 1205.04 | 239.79 | 46.28 | 1223.91 | 255.51 | 320.10 | 1136.26 |
| mixed | 51.52 | 1472.70 | 297.59 | 55.93 | 1480.22 | 312.82 | 334.69 | 1234.36 |
| chain | 43.66 | 1289.41 | 242.85 | 46.91 | 1338.37 | 258.97 | 313.92 | 1165.46 |
| fanout | 79.77 | 2452.17 | 525.54 | 82.01 | 2457.07 | 542.23 | 383.87 | 1400.88 |

Native core construction improves about 4.5–5.3x over individual persistent edits. Workbook population takes 1.14–1.40 s versus 0.31–0.38 s optimized reference, still roughly 3.5–3.7x slower. The remaining construction gap is real; no default backend change follows from these results. Full per-size capture/evaluation/ready and allocation data are in the native report/CSVs.

100K-start numeric workbook transaction medians (three samples):

| Operation | 100 cells | 1K cells | 10K cells |
|---|---:|---:|---:|
| ADD | 1.24 ms | 12.06 ms | 123.60 ms |
| UPDATE | 0.52 ms | 4.08 ms | 35.04 ms |

10K formula replacements: 48.58 ms median. Subsequent rows run at 133,300 cells plus the later scratch cell, not the initial 100K.

Native persistent workbook p95 milliseconds at that later population:

| Shape | Input/capture | Input/selected calculation | Short formula | Rectangle |
|---|---:|---:|---:|---:|
| independent | 0.022 | 0.024 | 0.025 | 0.410 |
| mixed | 0.018 | 0.027 | 0.023 | 0.443 |
| chain | 0.023 | 0.662 | 0.024 | 0.341 |
| fanout | 0.020 | 0.023 | 0.025 | 0.633 |

The full 100K-link chain diagnostic constructs in 467.7 ms new / 489.6 ms existing, with selected evaluation 43.4 / 42.6 ms; it is separate from the 1,000-link target. The guarded 200K numeric core constructs in 522.1 ms. Single construction samples are diagnostic observations, not tail guarantees or production-cap increases.

Native 100K-start workbook peak requested heap is 184.8–242.7 MB and domain charge 339.9–840.2 MB. Sixteen generations add about 101–106 KB requested live heap. Caller final-root release is 1.4–1.7 microseconds; maximum cleanup tasks are 35.2–54.4 ms and final drains 55.9–85.7 ms. Reclamation work remains off the caller, not absent.

**Memory limitation:** 48 separate native samples recorded normal and warning-level pressure during large cases; no critical-pressure stop occurred. The largest sampled benchmark-process RSS was 2,235.25 MiB, across the run, not a per-child high-water measurement. The runner's macOS RSS helper/guard remains unavailable; zero runner RSS is not zero usage. Preserve warning pressure when interpreting timings. The external monitor would terminate the benchmark group on critical pressure. GUI, AOT, actual cross-user denial, durability and million-cell capacity remain outside this validation.

