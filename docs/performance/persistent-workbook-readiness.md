# Persistent workbook construction profile and readiness evidence

Date: 2026-10-19. Base commit: d2e2a8b82504d4ac904c7e0609fe19b0d91a1653.
ADR-0012 foundation is accepted. This record compares a bounded private-index
improvement with its unchanged functional-insertion control; product constructor
selection and caps remain unchanged.

The bounded private-index change reduces 100K workbook construction from 2.44–3.13 s
to 1.16–1.81 s on Windows, and 1.97–2.47 s to 1.04–1.60 s on Linux. Numeric requested
allocation traffic falls from 1.492 GB to 651 MB. Peak/retained storage stays largely
unchanged; this fixes churn, not native memory capacity. All 234 ordinary/profile/core
children and Windows/Linux/macOS CI pass. Native 100K construction improves 1.44–1.68x.
Production Workbook selection stays reference; recommend explicit new_in for the
next bounded application/session slice after review.

## Reproduce and interpret

Builds/tests must finish before timings. Run the following five commands sequentially:

```sh
python scripts/bulk_workbook.py --example readiness --layers workbook --index-policy immutable --output-dir artifacts/readiness-control
python scripts/bulk_workbook.py --example readiness --layers workbook --output-dir artifacts/readiness-private
python scripts/bulk_workbook.py --example readiness --layers workbook --implementations persistent --index-policy immutable --profile --output-dir artifacts/readiness-profile-control
python scripts/bulk_workbook.py --example readiness --layers workbook --implementations persistent --profile --output-dir artifacts/readiness-profile-private
python scripts/bulk_workbook.py --example readiness --layers new --implementations batch --shapes independent,fanout --output-dir artifacts/readiness-core
```

78 isolated children per platform: 24 control, 24 private, 12 control profiles,
12 private profiles and six core-budget controls. Both workbook matrices compare
optimized reference and persistent across 1K/10K/100K numeric, mixed, 1,000-chain and
fan-out fixtures. Each child repeats construction three times, and retains the
accepted independent value/S/T/dispatch/cleanup assertions, bulk additions/updates/
formula replacements and 31-sample local edit/read measurements. Existing graph
bulk algorithms are unchanged; the accepted foundation's full-chain and 200K runs
remain historical diagnostics, not claims repeated by this focused matrix.

Both insertion policies use one binary, representation, binder, validator, quota
and publication path. The research-only immutable switch selects functional index
insertion instead of unique-node reuse. The reference coordinator checks final
quotas once for monotonic fixtures; this is not a comparison with production
reference transactions' per-edit checks or a user-facing regression claim.

Report non-profiled construction medians and min/max separately from 31-sample edit
p95. Profile clocks and allocator atomics perturb costs and must not be substituted
for optimized timings. Exclusive thread-wall phase time is not thread CPU time;
caller core-dispatch wait overlaps worker phase time and cleanup may overlap work.
Whole-child Unix CPU and wait4 RSS high-water include the entire workload, including
retained snapshots, rather than construction alone. Windows CPU is unavailable;
its live process-peak query is retained. macOS wait4 high-water is collected only
after exit and does not provide a live RSS guard. Preserve pressure monitoring.

Domain reservations bound known requested storage, not process RSS. Incoming caller
edits and returned caller-owned views are outside the ongoing ledger, while their
allocations appear in requested-heap/OS measurements. Source/environment manifests,
all raw CSVs, timeout/memory stops and failed exploratory logs remain under artifacts.
The initial readiness-before-profile-windows run was exploratory and overlapped
regression builds; it is excluded from final timing comparisons and attribution.

## Current evidence

Final Windows/Linux measurements and validation pass on frozen source; fresh
native macOS validation now passes on the same source. See the [architecture and compatibility matrix](../architecture/persistent-workbook-readiness.md)
and [handoff](../handoffs/2026-10-19-persistent-workbook-readiness.md).

## Windows matched evidence

[control (24 children)](../../artifacts/readiness-control-windows/20261020T002415Z/summary.json).
[private (24 children)](../../artifacts/readiness-private-windows/20261020T002813Z/summary.json).
[profile-control (12 children)](../../artifacts/readiness-profile-control-windows/20261020T003143Z/summary.json).
[profile-private (12 children)](../../artifacts/readiness-profile-private-windows/20261020T003253Z/summary.json).
[core (6 children)](../../artifacts/readiness-core-windows/20261020T003330Z/summary.json).

Ordinary construction median [min, max], milliseconds; three samples. Reference is the optimized end-of-batch quota coordinator.

| Initial N | Shape | Control reference | Control persistent | Private reference | Private persistent | Speedup |
|---:|---|---:|---:|---:|---:|---:|
| 1,000 | independent | 1.54 [1.51, 1.78] | 14.47 [14.18, 15.29] | 1.57 [1.53, 1.75] | 8.42 [8.15, 8.52] | 1.72x |
| 1,000 | mixed | 2.00 [1.98, 2.23] | 15.60 [15.43, 15.65] | 2.03 [1.96, 2.21] | 9.19 [9.12, 9.85] | 1.70x |
| 1,000 | chain | 3.63 [3.46, 4.00] | 18.36 [18.32, 18.89] | 3.68 [3.50, 4.18] | 12.30 [12.29, 12.67] | 1.49x |
| 1,000 | fanout | 3.38 [3.15, 3.85] | 19.64 [19.52, 19.83] | 3.30 [3.12, 3.47] | 13.32 [13.19, 13.91] | 1.47x |
| 10,000 | independent | 17.78 [17.54, 17.83] | 177.60 [175.79, 178.96] | 17.79 [17.34, 18.38] | 94.15 [94.15, 94.70] | 1.89x |
| 10,000 | mixed | 21.44 [21.40, 22.43] | 189.45 [188.95, 194.23] | 22.32 [21.55, 23.47] | 105.91 [105.74, 106.63] | 1.79x |
| 10,000 | chain | 20.57 [20.19, 20.86] | 182.73 [182.62, 183.35] | 20.38 [20.17, 20.83] | 99.30 [98.66, 99.55] | 1.84x |
| 10,000 | fanout | 34.97 [34.82, 36.43] | 237.02 [235.21, 237.34] | 34.50 [34.12, 35.89] | 153.39 [152.90, 156.92] | 1.55x |
| 100,000 | independent | 244.15 [231.84, 245.23] | 2438.57 [2436.88, 2446.69] | 236.42 [228.24, 244.94] | 1157.57 [1154.95, 1165.43] | 2.11x |
| 100,000 | mixed | 269.88 [269.31, 276.07] | 2575.23 [2574.48, 2585.80] | 271.79 [267.48, 279.36] | 1297.44 [1297.05, 1304.85] | 1.98x |
| 100,000 | chain | 233.34 [231.03, 245.30] | 2456.59 [2442.40, 2479.95] | 233.33 [232.32, 238.10] | 1165.47 [1154.03, 1167.34] | 2.11x |
| 100,000 | fanout | 415.88 [409.18, 416.43] | 3132.93 [3103.21, 3133.57] | 409.96 [403.61, 422.80] | 1808.44 [1797.32, 1822.66] | 1.73x |

Numeric construction requested allocations, median of three (traffic, not retained heap):

| N | Control MB | Private MB | Control calls | Private calls |
|---:|---:|---:|---:|---:|
| 1,000 | 10.41 | 5.46 | 90,814 | 42,494 |
| 10,000 | 128.68 | 61.89 | 1,156,341 | 504,229 |
| 100,000 | 1491.76 | 651.26 | 14,024,931 | 5,818,990 |

Exclusive instrumented phase median milliseconds, numeric construction. **These are diagnostic probe costs, not ordinary latencies or CPU time.** Dispatch wait overlaps worker phases; reclamation may overlap other phases.

| Phase | 1K control | 1K private | 10K control | 10K private | 100K control | 100K private |
|---|---:|---:|---:|---:|---:|---:|
| request | 0.046 | 0.045 | 0.298 | 0.307 | 2.555 | 2.505 |
| identity | 0.396 | 0.384 | 4.015 | 3.966 | 42.364 | 39.946 |
| sheet_indexes | 9.492 | 2.305 | 128.967 | 27.790 | 1922.365 | 419.753 |
| parse | 0.000 | 0.000 | 0.000 | 0.000 | 0.000 | 0.000 |
| binding | 0.000 | 0.000 | 0.000 | 0.000 | 0.000 | 0.000 |
| core_install | 0.613 | 0.609 | 7.413 | 7.397 | 77.170 | 78.268 |
| graph_pages | 3.267 | 3.243 | 33.164 | 33.091 | 332.597 | 337.342 |
| graph_names | 2.677 | 2.652 | 34.984 | 34.889 | 433.912 | 436.065 |
| validation | 0.001 | 0.001 | 0.002 | 0.002 | 0.004 | 0.003 |
| admission | 4.594 | 1.709 | 57.938 | 19.403 | 701.759 | 218.512 |
| publication | 0.010 | 0.009 | 0.018 | 0.018 | 0.018 | 0.019 |
| worksheet | 0.518 | 0.466 | 6.112 | 6.051 | 76.553 | 72.160 |
| reclamation | 0.013 | 0.011 | 0.371 | 0.371 | 3.631 | 3.589 |
| core_dispatch_wait | 7.988 | 7.867 | 90.881 | 90.949 | 1019.974 | 1029.487 |

At 100K numeric, instrumentation inflates construction to 3587.48 / 1604.74 ms control/private, versus 2438.57 / 1157.57 ms ordinary. Admission clocks themselves perturb the measured admission bucket; do not infer that all of that time is intrinsic quota cost. Allocation counts and the same-binary non-profiled control establish the improvement independently.

| Numeric worksheet-index traffic | Control | Private |
|---|---:|---:|
| Requested MB | 929.63 | 89.47 |
| Allocation calls | 8,832,474 | 630,008 |

100K-start numeric bulk medians, milliseconds:

| Work | Control | Private |
|---|---:|---:|
| ADD 100 | 2.62 | 1.73 |
| ADD 1,000 | 25.00 | 15.39 |
| ADD 10,000 | 260.84 | 142.45 |
| UPDATE 100 | 0.72 | 0.51 |
| UPDATE 1,000 | 7.45 | 6.61 |
| UPDATE 10,000 | 71.46 | 48.54 |
| FORMULAS 10,000 | 101.43 | 73.64 |

Private p95, 31 samples after warmup. Population is 133,300 after additions plus the later scratch cell, not initial 100K.

| Shape | Input/capture | Input/selected | Short formula | Rectangle |
|---|---:|---:|---:|---:|
| independent | 0.041 | 0.081 | 0.047 | 0.506 |
| mixed | 0.028 | 0.034 | 0.040 | 0.663 |
| chain | 0.050 | 1.060 | 0.040 | 0.666 |
| fanout | 0.030 | 0.035 | 0.049 | 0.836 |

Whole-child CPU and peak RSS at 100K, including three constructions, bulk edits, retained generations and reads; not construction-only. Windows CPU is unavailable. Unix OS high-water comes from wait4; Windows peak is the maximum live OS peak query.

| Shape | Policy/backend | CPU seconds | RSS MiB |
|---|---|---:|---:|
| independent | control/reference | Unavailable | 1043.79 |
| independent | control/persistent | Unavailable | 209.82 |
| independent | private/persistent | Unavailable | 207.09 |
| fanout | control/reference | Unavailable | 1310.23 |
| fanout | control/persistent | Unavailable | 274.23 |
| fanout | private/persistent | Unavailable | 272.07 |

Control memory/lifecycle range across 100K-start shapes: peak requested heap 184.70–242.63 MB; domain peak 339.86–840.25 MB; 16 retained generations 104.3–107.1 KB; caller root release 3.2–6.6 microseconds; maximum cleanup task 99.41–144.48 ms; final drain 170.84–232.03 ms. All final domain charges are zero.

Private memory/lifecycle range across 100K-start shapes: peak requested heap 184.73–242.78 MB; domain peak 339.86–840.25 MB; 16 retained generations 102.8–104.3 KB; caller root release 2.0–4.1 microseconds; maximum cleanup task 102.93–150.52 ms; final drain 161.95–231.53 ms. All final domain charges are zero.

Core budget controls (unchanged canonical graph), median ms:

| N | Numeric | Fan-out |
|---:|---:|---:|
| 1,000 | 4.52 | 10.15 |
| 10,000 | 50.63 | 113.04 |
| 100,000 | 592.33 | 1256.93 |

## Cost attribution and stopping point

The same-binary control identifies avoidable worksheet AVL path copying as the
primary numeric construction cost. At 100K, the private path removes about 8.2
million worksheet-index allocations and 840 MB of allocation traffic. UUID/anchor
planning is tens of instrumented milliseconds, not the dominant seconds-scale
cost. Numeric-only validation is tiny because no formula dependency closure needs
a cycle walk; this does not mean formulas skip validation.

After the change, Windows numeric profile time is concentrated in canonical graph
name indexing (436 ms), page installation (337 ms), worksheet indexes (420 ms) and
instrumented admission (219 ms), with about 40 ms identity/anchor planning. These
are perturbed probe timings, not unbiased CPU attribution. The non-profiled
construction result is 1.158 seconds. Canonical name insertion still copies charged
AVL key/value paths, and page updates still perform charged record/value work;
these are measured remaining implementation costs versus mutable reference maps.
No claim is made that this gap is an unavoidable lower bound of persistent graphs.

For 100K fan-out, the private instrumented parse/lowering phase is 56.8 ms,
binding/reverse links 354.5 ms and dependency validation 64.7 ms. For mixed formulas
these are 12.8 / 70.0 / 14.8 ms; the bounded-chain fixture is 0.7 / 1.7 / 0.6 ms.
All sizes/shapes retain complete raw phase rows. Publication itself is small; caller
core-dispatch wait overlaps worker work and is not a second cost to add to it.
The reclamation profile bucket measures deferred cleanup-worker jobs. Immediate
refcount destruction during an index/page operation remains charged to that
operation; final last-owner cleanup and drain are separately reported by the
ordinary workload.

Current Windows interactive and bounded bulk budgets pass, with roughly 1.7–2.1x
100K construction improvement. Requested peak heap and admitted domain peak barely
change: this reduces allocation churn, not retained representation size. That
supports one bounded optimization and a documented residual gap, not an indefinite
sequence of graph redesigns. Native warning-pressure findings remain unresolved as
an OS-capacity claim; fresh source validation/high-water evidence is required.

## Linux matched evidence

[control (24 children)](../../artifacts/readiness-control-linux/20261020T004255Z/summary.json).
[private (24 children)](../../artifacts/readiness-private-linux/20261020T004632Z/summary.json).
[profile-control (12 children)](../../artifacts/readiness-profile-control-linux/20261020T004950Z/summary.json).
[profile-private (12 children)](../../artifacts/readiness-profile-private-linux/20261020T005053Z/summary.json).
[core (6 children)](../../artifacts/readiness-core-linux/20261020T005132Z/summary.json).

Ordinary construction median [min, max], milliseconds; three samples. Reference is the optimized end-of-batch quota coordinator.

| Initial N | Shape | Control reference | Control persistent | Private reference | Private persistent | Speedup |
|---:|---|---:|---:|---:|---:|---:|
| 1,000 | independent | 1.20 [1.19, 4.16] | 11.64 [11.21, 15.94] | 1.58 [1.38, 4.67] | 7.02 [6.83, 10.76] | 1.66x |
| 1,000 | mixed | 1.42 [1.42, 4.53] | 13.36 [12.95, 16.99] | 1.47 [1.44, 4.57] | 8.06 [7.37, 12.22] | 1.66x |
| 1,000 | chain | 2.66 [2.60, 6.49] | 14.59 [14.09, 18.97] | 2.51 [2.31, 5.73] | 10.21 [9.71, 14.72] | 1.43x |
| 1,000 | fanout | 2.07 [2.05, 5.62] | 15.30 [14.91, 21.05] | 2.08 [1.99, 5.36] | 10.94 [10.58, 15.33] | 1.40x |
| 10,000 | independent | 14.69 [14.29, 21.12] | 146.09 [139.66, 152.54] | 16.24 [15.27, 21.16] | 84.16 [77.37, 90.10] | 1.74x |
| 10,000 | mixed | 17.81 [17.63, 26.12] | 154.47 [148.07, 162.34] | 17.48 [17.11, 24.99] | 91.49 [88.85, 100.51] | 1.69x |
| 10,000 | chain | 16.86 [16.33, 22.46] | 151.86 [148.67, 160.81] | 16.69 [16.59, 23.94] | 85.50 [80.43, 94.20] | 1.78x |
| 10,000 | fanout | 25.41 [24.93, 34.53] | 195.38 [190.35, 205.46] | 26.17 [25.10, 34.13] | 129.98 [125.09, 149.66] | 1.50x |
| 100,000 | independent | 228.21 [208.42, 245.76] | 1970.67 [1904.76, 1992.55] | 211.81 [209.91, 254.71] | 1038.93 [999.75, 1075.11] | 1.90x |
| 100,000 | mixed | 231.71 [231.36, 298.88] | 2084.54 [2057.21, 2123.75] | 235.32 [232.81, 276.28] | 1153.26 [1129.89, 1216.40] | 1.81x |
| 100,000 | chain | 212.67 [212.00, 262.66] | 1984.99 [1914.97, 1990.40] | 220.84 [212.99, 247.38] | 1065.87 [1004.80, 1084.64] | 1.86x |
| 100,000 | fanout | 315.81 [306.13, 390.83] | 2472.83 [2428.44, 2530.30] | 309.19 [303.47, 373.05] | 1598.35 [1511.92, 1600.25] | 1.55x |

Numeric construction requested allocations, median of three (traffic, not retained heap):

| N | Control MB | Private MB | Control calls | Private calls |
|---:|---:|---:|---:|---:|
| 1,000 | 10.42 | 5.46 | 90,878 | 42,490 |
| 10,000 | 128.70 | 61.89 | 1,156,481 | 504,226 |
| 100,000 | 1492.21 | 651.25 | 14,029,264 | 5,818,945 |

Exclusive instrumented phase median milliseconds, numeric construction. **These are diagnostic probe costs, not ordinary latencies or CPU time.** Dispatch wait overlaps worker phases; reclamation may overlap other phases.

| Phase | 1K control | 1K private | 10K control | 10K private | 100K control | 100K private |
|---|---:|---:|---:|---:|---:|---:|
| request | 0.024 | 0.024 | 0.426 | 0.436 | 4.016 | 3.944 |
| identity | 0.337 | 0.327 | 3.467 | 3.284 | 36.572 | 34.600 |
| sheet_indexes | 7.354 | 1.812 | 98.251 | 23.452 | 1432.219 | 377.226 |
| parse | 0.000 | 0.000 | 0.000 | 0.000 | 0.000 | 0.000 |
| binding | 0.000 | 0.000 | 0.000 | 0.000 | 0.000 | 0.000 |
| core_install | 0.761 | 0.724 | 7.525 | 7.461 | 84.438 | 81.585 |
| graph_pages | 2.929 | 2.981 | 29.002 | 29.225 | 287.919 | 291.631 |
| graph_names | 1.997 | 2.085 | 25.854 | 26.063 | 350.848 | 360.258 |
| validation | 0.001 | 0.001 | 0.002 | 0.002 | 0.003 | 0.003 |
| admission | 3.877 | 1.433 | 49.179 | 16.435 | 594.134 | 183.704 |
| publication | 0.029 | 0.027 | 0.034 | 0.038 | 0.035 | 0.034 |
| worksheet | 0.547 | 0.456 | 6.207 | 5.287 | 77.909 | 68.116 |
| reclamation | 0.031 | 0.017 | 0.472 | 0.561 | 4.903 | 4.900 |
| core_dispatch_wait | 7.211 | 6.891 | 75.550 | 75.923 | 871.013 | 880.982 |

At 100K numeric, instrumentation inflates construction to 2877.17 / 1416.26 ms control/private, versus 1970.67 / 1038.93 ms ordinary. Admission clocks themselves perturb the measured admission bucket; do not infer that all of that time is intrinsic quota cost. Allocation counts and the same-binary non-profiled control establish the improvement independently.

| Numeric worksheet-index traffic | Control | Private |
|---|---:|---:|
| Requested MB | 930.11 | 89.48 |
| Allocation calls | 8,837,314 | 630,034 |

100K-start numeric bulk medians, milliseconds:

| Work | Control | Private |
|---|---:|---:|
| ADD 100 | 4.29 | 2.44 |
| ADD 1,000 | 21.73 | 14.39 |
| ADD 10,000 | 245.00 | 137.37 |
| UPDATE 100 | 2.16 | 1.04 |
| UPDATE 1,000 | 15.04 | 8.10 |
| UPDATE 10,000 | 99.89 | 55.16 |
| FORMULAS 10,000 | 149.97 | 122.39 |

Private p95, 31 samples after warmup. Population is 133,300 after additions plus the later scratch cell, not initial 100K.

| Shape | Input/capture | Input/selected | Short formula | Rectangle |
|---|---:|---:|---:|---:|
| independent | 0.153 | 0.176 | 0.190 | 1.020 |
| mixed | 0.144 | 0.257 | 0.225 | 1.499 |
| chain | 0.153 | 1.160 | 0.162 | 1.152 |
| fanout | 0.150 | 0.152 | 0.222 | 4.406 |

Whole-child CPU and peak RSS at 100K, including three constructions, bulk edits, retained generations and reads; not construction-only. Windows CPU is unavailable. Unix OS high-water comes from wait4; Windows peak is the maximum live OS peak query.

| Shape | Policy/backend | CPU seconds | RSS MiB |
|---|---|---:|---:|
| independent | control/reference | 23.001 | 1086.70 |
| independent | control/persistent | 8.428 | 272.17 |
| independent | private/persistent | 4.976 | 272.46 |
| fanout | control/reference | 33.947 | 1365.44 |
| fanout | control/persistent | 10.379 | 342.50 |
| fanout | private/persistent | 6.896 | 342.43 |

Control memory/lifecycle range across 100K-start shapes: peak requested heap 184.74–242.62 MB; domain peak 339.86–840.25 MB; 16 retained generations 102.8–105.7 KB; caller root release 19.1–21.8 microseconds; maximum cleanup task 161.95–207.31 ms; final drain 256.22–345.04 ms. All final domain charges are zero.

Private memory/lifecycle range across 100K-start shapes: peak requested heap 184.72–242.71 MB; domain peak 339.86–840.25 MB; 16 retained generations 102.8–105.7 KB; caller root release 19.7–24.7 microseconds; maximum cleanup task 137.82–159.03 ms; final drain 227.56–289.48 ms. All final domain charges are zero.

Core budget controls (unchanged canonical graph), median ms:

| N | Numeric | Fan-out |
|---:|---:|---:|
| 1,000 | 3.84 | 8.60 |
| 10,000 | 43.09 | 95.69 |
| 100,000 | 495.38 | 1073.85 |

## Native macOS readiness corroboration

The [native report](../../artifacts/macos-validation-20261020T010244Z/REPORT.md)
passes on Apple M1, 16 GiB, macOS 27.0.1 ARM64, Rust nightly 1.101.0 and .NET
10.0.402. A fresh dirty/untracked working-tree copy, targets, .NET outputs and temp
files were on internal APFS; no build/test ran on SMB. Before these doc edits, all
296 copied source SHA-256 hashes matched this tree. All five 68-file benchmark
manifests match; final local/share verification found no changes and transfer
verification covers 464 returned report files. No source fixes or reruns occurred.
The superseded bulk packet was stopped before tests; no older broad sweep is counted.

All 78 sequential children pass (24 ordinary control, 24 ordinary private,
12 profile control, 12 profile private, six core). The readiness total is now 234
across three platforms, separate from the accepted foundation's 335 children.
All final domain charges are zero, bulk dispatch assertions pass and no unexpected
lane rejection, timeout or post-exit 8 GiB exceedance occurred.

| Initial N | Shape | Control reference | Control persistent | Private reference | Private persistent | Control/private persistent speedup |
| ---: | --- | ---: | ---: | ---: | ---: | ---: |
| 1000 | independent | 3.941 [3.623, 4.386] | 7.355 [7.302, 7.719] | 3.592 [3.382, 3.969] | 6.390 [6.115, 7.258] | 1.151x |
| 1000 | mixed | 3.508 [3.471, 3.928] | 8.343 [8.173, 9.241] | 3.498 [3.465, 4.039] | 6.449 [6.115, 6.987] | 1.294x |
| 1000 | chain | 3.283 [3.261, 3.541] | 10.941 [10.255, 13.139] | 3.249 [3.240, 3.543] | 8.049 [7.611, 8.242] | 1.359x |
| 1000 | fanout | 3.503 [3.305, 3.693] | 10.631 [10.026, 11.195] | 3.742 [3.533, 4.008] | 8.905 [8.342, 9.514] | 1.194x |
| 10000 | independent | 26.731 [26.426, 30.108] | 93.007 [92.677, 100.694] | 26.798 [26.617, 30.715] | 60.830 [60.649, 68.502] | 1.529x |
| 10000 | mixed | 28.189 [28.022, 30.674] | 98.494 [98.074, 103.379] | 28.467 [28.396, 32.249] | 65.750 [65.411, 66.543] | 1.498x |
| 10000 | chain | 28.298 [27.784, 33.497] | 96.025 [94.978, 101.423] | 27.912 [27.713, 29.727] | 62.863 [62.854, 69.161] | 1.528x |
| 10000 | fanout | 33.611 [33.079, 37.789] | 120.820 [119.492, 123.696] | 33.754 [33.041, 36.101] | 85.660 [84.086, 91.066] | 1.410x |
| 100000 | independent | 319.865 [312.449, 376.555] | 1252.953 [1205.629, 1293.425] | 318.692 [318.039, 321.106] | 745.033 [740.432, 776.627] | 1.682x |
| 100000 | mixed | 330.990 [330.291, 339.011] | 1230.707 [1215.581, 1232.025] | 333.645 [333.097, 337.039] | 795.899 [782.915, 803.739] | 1.546x |
| 100000 | chain | 315.258 [315.218, 317.130] | 1175.225 [1159.969, 1175.358] | 318.591 [316.181, 326.332] | 741.708 [741.593, 754.895] | 1.584x |
| 100000 | fanout | 386.676 [386.562, 388.222] | 1443.408 [1438.991, 1450.508] | 388.767 [385.056, 389.968] | 999.499 [997.527, 1012.209] | 1.444x |


Ordinary 100K private construction is 0.742–0.999 s, improving 1.44–1.68x;
it remains about 2.3–2.6x slower than the matched optimized reference coordinator.
Three construction samples give medians/ranges, not an SLA or construction p95.
Numeric requested traffic falls from 1.491 GB / 14.02M calls to 651 MB / 5.82M calls.
Instrumented numeric sheet-index thread-wall falls 845.27 → 237.73 ms, with traffic
929.94 → 89.47 MB and calls 8.84M → 0.63M. This independently corroborates the
worksheet path-copying diagnosis; profile probes perturb timings, and dispatch
wait overlaps worker phases. These phase times are not CPU totals.

At the later 133,300-cell population, worst private workbook p95 across shapes is
0.040 ms input/capture, 0.682 ms selected 1K-chain, 0.027 ms short formula and
0.533 ms bounded inspection. Numeric ADD10K is 87.52 ms, UPDATE10K 24.32 ms and
FORMULAS10K 36.41 ms. All ordinary interactive, inspection and bulk targets pass.
Core numeric/fan-out 100K controls are 255.63 / 536.69 ms, inside 1 s / 2 s.
Full bulk samples, phase rows, CPU/high-water and budget comparisons remain in the
native report and its CSVs; these are matched readiness cases, not a repeat of
full-chain or 200K foundation diagnostics.

Whole-child private numeric CPU is 2.980 s / high-water 239.11 MiB; fan-out is
3.804 s / 312.45 MiB. These include three constructions and later edits/reads/
retention. The private reference cases use 10.439 s / 1,216.55 MiB numeric and
14.654 s / 1,418.64 MiB fan-out. Across all 78 children, maximum wait4 high-water
is 1,418.80 MiB; Darwin wait4 reports bytes. This post-exit high-water is separate
from 52 sparse native samples (maximum sampled RSS 500.55 MiB).
All sampled pressure levels were normal; earlier foundation warning pressure
remains valid evidence for that earlier workload, not disproved by this run.
Native live RSS guarding is still unavailable; the external critical-pressure
monitor must terminate owned child groups as well as the driver.

At 100K, private requested live peak is 184.72–242.75 MB, domain peak
339.86–840.25 MB and 16 retained generations add 104–106 KB. Caller final-root
release is 1.46–2.13 microseconds, maximum cleanup task 54–73 ms, final drain
32–55 ms; zero final domain charge is not proof of RSS returning to baseline.
Caller input/result copies remain outside continuing domain accounting. Allocation
traffic reduction does not establish reduced retained size or increased capacity.

## Final validation and selection gate

| Gate | Windows | Linux WSL2 | Fresh native macOS |
|---|---:|---:|---|
| Full default CI | Passed | Passed | Passed |
| Default Rust tests | 251 | 252 | 252 |
| All-feature Rust tests | 300 | 301 | 301 |
| True core fallback tests | 21 | 21 | 21 |
| Persistent-workbook-only tests | 49 | 49 | 49 |
| Default/all-feature strict lint and formatting | Passed | Passed | Passed |
| Core fallback/research-only and workbook-only strict lint | Passed | Passed | Passed |
| .NET Release, decimal/protocol parity, real IPC | Passed | Passed | Passed |
| Matched ordinary/profile/core children | 78 | 78 | 78 |

[Windows CI](../../artifacts/readiness-ci-frozen-windows/20261020T004023199943Z/summary.json),
[Linux CI](../../artifacts/readiness-ci-frozen-linux/20261020T005140262292Z/summary.json)
and [156-child/source audit](../../artifacts/readiness-evidence-audit.txt).
All 68 core/workbook/harness hashes match the final tree after path-separator
normalization. No dependency, .NET or frozen fixture changes. No default constructor
migration. The [fresh native APFS report](../../artifacts/macos-validation-20261020T010244Z/REPORT.md) validates this readiness source;
the accepted foundation run remains separate historical evidence.

Early exploratory failures are retained: the initial strict lint found an unused
import in the test module, removed before frozen validation; the first churn setup
exceeded its default 256-edit limit, then used an allowed bounded product
configuration (2K live-cell limit, 1K-edit limit, unchanged 1 MiB metadata cap).
No product cap or correctness assertion was weakened. The exploratory first profile
is excluded because it overlapped builds. Final matrices and CI use frozen source.

Recommend accepting the private-index optimization and preferring persistent new_in
for the next application/session implementation after owner review. Keep Workbook.new reference, retain explicit fallback/oracles and
require an application-owned runtime, worker scheduling and bounded caller buffers.
The residual construction gap is measured rather than dismissed: roughly 4.4–5.2x on Windows/Linux and 2.3–2.6x on macOS
versus the optimized reference coordinator at 100K, with canonical graph name/page
and accounting work still present. Current investigation budgets pass with the
remaining costs explained well enough to stop this optimization slice. This is not
a claim that the representation has reached its theoretical lower bound.

Do not switch production constructors in this slice. Stop for review; after this
gate, prioritize a runnable C# application/session interface rather than another
speculative backend milestone. Native OS capacity, GUI/AOT, cross-user denial,
durability and million-cell claims remain outside this evidence.
