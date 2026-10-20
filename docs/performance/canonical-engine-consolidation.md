# Canonical engine consolidation: Release evidence

Current implementation: [ADR-0013](../adr/0013-canonical-engine-consolidation.md).
Historical source and reference measurements are preserved at commit
`3a6fd9719f491a9236dbf0197ed9d4fb7ade3e24`; no reference engine remains executable.
Compare matching readiness-private ordinary cases; profile probes are excluded.
The earlier optimized-reference coordinator remains a historical comparison, not a product target.

## Outcome

Windows, isolated WSL2 Linux and native macOS correctness/CI pass. All 99 matrix children satisfy
correctness, timeout/RSS guards, dispatch and zero-charge assertions. Of the original 224 Windows/Linux ordinary
budget comparisons, 223 pass in the first sweep; all 153 native report comparisons pass. Linux 1K-start fan-out workbook
viewport p95 is 5.343 ms against <5 ms (median 1.113 ms, maximum 6.416 ms).
A single [matched diagnostic repeat](../../artifacts/canonical-viewport-followup-linux/20261020T015403Z/summary.json)
passes at p95 3.381 ms, median 0.916 ms, maximum 3.560 ms, with identical 456,773
requested bytes / 3,264 calls per read. The historical same-case p95 was 3.420 ms.
This is evidence of variable tail timing, not proof of its cause or permission to
discard the original miss. The viewport margin remains a review limitation; do not
claim every first-sweep performance gate passed. No timing threshold was relaxed.
All 100 children including this diagnostic repeat have zero final domain charges.

100K workbook medians remain near readiness-private: numeric 1.156 s Windows /
1.041 s Linux, fan-out 1.796 / 1.591 s. Core numeric/fan-out construction and all
100K-start interactive/bulk comparisons pass their existing budgets. Numeric
ADD10K is 145.89 / 140.98 ms and FORMULAS10K 79.20 / 131.99 ms. Some bulk timings
are slower than the prior run; the full ranges below preserve that tradeoff.
Retained/domain representation is substantially unchanged. The numeric construction
traffic remains about 651 MB / 5.82M allocation calls, rather than the retired
immutable-insertion control's 1.492 GB / 14M calls.

## Method and interpretation

Sequential Release processes after CI/builds complete, ordinary counting allocator, three
construction samples and 31 interactive samples after three warmups. Construction
medians/ranges are observations, not p95/SLA estimates. The workbook starts at N;
later bulk additions reach N+33,300 plus a scratch cell. Research constructors permit
large fixtures; production caps remain unchanged. Main matrices have 24 children
(three sizes/four shapes/core+workbook), existing-graph matrices six (numeric/fan-out),
small standalone matrices three (1/8/32 nodes), 33 children per platform.

Core batch and individual mutation policies share the same graph implementation.
Small-model rows include construction, local capture, selected evaluation and drop.
The main workbook cases retain atomic ADD/UPDATE100/1K/10K, FORMULAS10K, selected
evaluation, viewport reads, old S/new T, retained generations and reclamation checks.
Existing core cases build N, then atomically add another N with old capture retained.

Commands:

```sh
python scripts/bulk_workbook.py --layers new,workbook --output-dir artifacts/canonical-main-PLATFORM
python scripts/bulk_workbook.py --layers existing --shapes independent,fanout --output-dir artifacts/canonical-existing-PLATFORM
python scripts/bulk_workbook.py --layers small --sizes 1,8,32 --shapes independent --output-dir artifacts/canonical-small-PLATFORM
```

Timeout 600 s/child, RSS budget 8 GiB. Windows uses OS process-peak queries; Unix
wait4 gives completed-child CPU and high-water (not live enforcement on macOS).
Requested allocation traffic, live heap, admitted-domain charges and RSS are distinct.
Caller input/returned buffers are outside continuing domain charges. Zero final
charge does not imply the allocator returns RSS to baseline. Fresh native macOS validation now passes; see its evidence below.

## Windows

[main: 24 children](../../artifacts/canonical-main-windows/20261020T015028Z/summary.json).
[existing: 6 children](../../artifacts/canonical-existing-windows/20261020T015116Z/summary.json).
[small: 3 children](../../artifacts/canonical-small-windows/20261020T015127Z/summary.json).

Construction median [min, max], milliseconds, with prior readiness-private workbook median.

| N | Shape | Core batch | Workbook canonical | Prior workbook |
|---:|---|---:|---:|---:|
| 1,000 | independent | 4.41 [4.40, 4.74] | 8.52 [8.32, 8.75] | 8.42 |
| 1,000 | mixed | 5.51 [5.41, 5.81] | 9.20 [8.98, 9.67] | 9.19 |
| 1,000 | chain | 9.08 [8.96, 9.15] | 12.79 [12.56, 12.82] | 12.30 |
| 1,000 | fanout | 9.93 [9.82, 10.15] | 13.96 [13.93, 14.22] | 13.32 |
| 10,000 | independent | 50.11 [49.19, 50.44] | 94.81 [92.59, 96.08] | 94.15 |
| 10,000 | mixed | 62.84 [62.54, 62.88] | 105.56 [104.31, 105.60] | 105.91 |
| 10,000 | chain | 55.32 [54.88, 55.88] | 98.70 [98.62, 100.82] | 99.30 |
| 10,000 | fanout | 114.97 [114.75, 115.95] | 155.94 [154.28, 157.68] | 153.39 |
| 100,000 | independent | 583.55 [579.36, 585.52] | 1155.70 [1155.25, 1158.72] | 1157.57 |
| 100,000 | mixed | 717.16 [716.29, 722.08] | 1292.10 [1286.53, 1295.77] | 1297.44 |
| 100,000 | chain | 582.23 [580.94, 582.25] | 1171.28 [1160.63, 1184.34] | 1165.47 |
| 100,000 | fanout | 1254.87 [1250.64, 1258.89] | 1796.31 [1793.65, 1809.64] | 1808.44 |

100K-start workbook interactive p95 (31 samples), milliseconds:

| Shape | Input/capture | Selected | Short formula | Viewport |
|---|---:|---:|---:|---:|
| independent | 0.053 | 0.040 | 0.066 | 0.545 |
| mixed | 0.055 | 0.039 | 0.088 | 0.609 |
| chain | 0.065 | 1.127 | 0.043 | 0.703 |
| fanout | 0.029 | 0.037 | 0.039 | 0.820 |

Numeric 100K-start workbook bulk medians, milliseconds; same-fixture historical comparison:

| Work | Canonical median [min, max] | Prior private median |
|---|---:|---:|
| ADD 100 | 2.30 [2.09, 2.32] | 1.73 |
| ADD 1,000 | 16.83 [16.82, 17.16] | 15.39 |
| ADD 10,000 | 145.89 [144.80, 153.10] | 142.45 |
| UPDATE 100 | 0.97 [0.74, 1.20] | 0.51 |
| UPDATE 1,000 | 6.95 [6.17, 7.64] | 6.61 |
| UPDATE 10,000 | 51.58 [51.50, 52.16] | 48.54 |
| FORMULAS 10,000 | 79.20 [77.11, 114.61] | 73.64 |

Whole-child resource/lifecycle observations at 100K-start, not construction-only:

| Shape | Requested heap peak MB | Domain peak MB | CPU s | RSS MiB | Retain16 KB | Root release us | Max cleanup ms | Final drain ms |
|---|---:|---:|---:|---:|---:|---:|---:|---:|
| independent | 184.68 | 339.86 | Unavailable | 206.62 | 103.2 | 4.9 | 105.03 | 165.92 |
| mixed | 196.30 | 470.66 | Unavailable | 219.78 | 104.7 | 2.9 | 111.73 | 176.43 |
| chain | 185.24 | 352.18 | Unavailable | 207.47 | 103.2 | 4.7 | 118.35 | 217.60 |
| fanout | 242.68 | 840.25 | Unavailable | 270.96 | 100.4 | 2.8 | 147.61 | 226.45 |

Existing graph atomic-add medians (N existing + N new), milliseconds:

| N | Numeric | Fan-out |
|---:|---:|---:|
| 1,000 | 4.82 | 10.66 |
| 10,000 | 55.48 | 117.34 |
| 100,000 | 629.08 | 1305.99 |

Small standalone create/capture/evaluate/drop, 31-sample median / p95 milliseconds:

| Nodes | Median | p95 |
|---:|---:|---:|
| 1 | 0.025 | 0.031 |
| 8 | 0.228 | 0.267 |
| 32 | 1.207 | 2.080 |

All 33 children pass, all final domain charges are zero, all checked dispatches
are one, no lane rejects occur, all 47 source hashes match the final tree.
Ordinary review budget checks: 112, failures: 0.

## Linux

[main: 24 children](../../artifacts/canonical-main-linux/20261020T015148Z/summary.json).
[existing: 6 children](../../artifacts/canonical-existing-linux/20261020T015237Z/summary.json).
[small: 3 children](../../artifacts/canonical-small-linux/20261020T015249Z/summary.json).

Construction median [min, max], milliseconds, with prior readiness-private workbook median.

| N | Shape | Core batch | Workbook canonical | Prior workbook |
|---:|---|---:|---:|---:|
| 1,000 | independent | 4.99 [3.64, 5.11] | 7.91 [7.45, 10.50] | 7.02 |
| 1,000 | mixed | 4.69 [4.31, 5.84] | 7.94 [7.45, 9.54] | 8.06 |
| 1,000 | chain | 7.52 [6.92, 8.85] | 10.05 [9.50, 13.26] | 10.21 |
| 1,000 | fanout | 9.54 [9.18, 9.86] | 11.31 [10.53, 14.70] | 10.94 |
| 10,000 | independent | 43.22 [39.86, 43.99] | 81.56 [77.88, 89.21] | 84.16 |
| 10,000 | mixed | 52.80 [49.71, 54.41] | 91.14 [87.07, 99.63] | 91.49 |
| 10,000 | chain | 47.71 [43.84, 48.65] | 84.05 [82.27, 91.90] | 85.50 |
| 10,000 | fanout | 97.04 [91.44, 97.65] | 126.71 [121.21, 139.35] | 129.98 |
| 100,000 | independent | 496.47 [462.73, 504.17] | 1041.25 [1010.11, 1066.97] | 1038.93 |
| 100,000 | mixed | 616.18 [579.35, 620.63] | 1144.01 [1104.58, 1193.98] | 1153.26 |
| 100,000 | chain | 505.99 [472.26, 520.02] | 1061.64 [1005.59, 1113.42] | 1065.87 |
| 100,000 | fanout | 1049.67 [1034.53, 1083.36] | 1590.96 [1516.65, 1621.70] | 1598.35 |

100K-start workbook interactive p95 (31 samples), milliseconds:

| Shape | Input/capture | Selected | Short formula | Viewport |
|---|---:|---:|---:|---:|
| independent | 0.164 | 0.167 | 0.241 | 2.247 |
| mixed | 0.140 | 0.155 | 0.250 | 2.915 |
| chain | 0.151 | 1.633 | 0.157 | 1.108 |
| fanout | 0.204 | 0.221 | 0.296 | 4.783 |

Numeric 100K-start workbook bulk medians, milliseconds; same-fixture historical comparison:

| Work | Canonical median [min, max] | Prior private median |
|---|---:|---:|
| ADD 100 | 3.35 [1.81, 3.93] | 2.44 |
| ADD 1,000 | 14.34 [13.47, 14.95] | 14.39 |
| ADD 10,000 | 140.98 [121.22, 149.30] | 137.37 |
| UPDATE 100 | 1.14 [1.11, 2.38] | 1.04 |
| UPDATE 1,000 | 8.95 [7.58, 9.64] | 8.10 |
| UPDATE 10,000 | 55.15 [53.01, 61.17] | 55.16 |
| FORMULAS 10,000 | 131.99 [82.43, 136.76] | 122.39 |

Whole-child resource/lifecycle observations at 100K-start, not construction-only:

| Shape | Requested heap peak MB | Domain peak MB | CPU s | RSS MiB | Retain16 KB | Root release us | Max cleanup ms | Final drain ms |
|---|---:|---:|---:|---:|---:|---:|---:|---:|
| independent | 184.70 | 339.86 | 5.011 | 272.02 | 103.2 | 19.9 | 141.68 | 232.48 |
| mixed | 196.30 | 470.66 | 5.443 | 288.89 | 103.2 | 21.2 | 144.67 | 250.23 |
| chain | 185.25 | 352.18 | 5.142 | 264.72 | 103.2 | 19.4 | 138.68 | 237.96 |
| fanout | 242.71 | 840.25 | 6.958 | 342.46 | 103.2 | 23.5 | 158.99 | 288.17 |

Existing graph atomic-add medians (N existing + N new), milliseconds:

| N | Numeric | Fan-out |
|---:|---:|---:|
| 1,000 | 3.87 | 8.59 |
| 10,000 | 43.50 | 95.86 |
| 100,000 | 510.38 | 1094.63 |

Small standalone create/capture/evaluate/drop, 31-sample median / p95 milliseconds:

| Nodes | Median | p95 |
|---:|---:|---:|
| 1 | 0.114 | 0.146 |
| 8 | 1.033 | 1.146 |
| 32 | 3.933 | 4.572 |

All 33 children pass, all final domain charges are zero, all checked dispatches
are one, no lane rejects occur, all 47 source hashes match the final tree.
Ordinary review budget checks: 112, failures: 1.

## Native macOS corroboration

The [native report](../../artifacts/macos-validation-20261020T015824Z/REPORT.md)
validates the dirty working tree at HEAD `3a6fd9719f491a9236dbf0197ed9d4fb7ade3e24`
on Apple M1, 16 GiB RAM, macOS 27.0.1 ARM64, Rust 1.101.0-nightly and .NET 10.0.402.
Source, Cargo/.NET outputs, temporary files and initial reports were on verified
internal APFS; no builds/tests ran on SMB. All 280 copied-file hashes matched this
workspace before these documentation edits. All three 47-file benchmark manifests
match the reviewed source; final local/share comparisons have no differences, and
404 returned report files were transfer-verified. No source fixes or reruns occurred.

Full CI passes 276 default tests, strict default/all-feature lint, 292 all-feature
tests and 62 core no-default-feature tests. No-default means the same canonical
graph, not a fallback. .NET Release has zero warnings/errors; all 10,099 decimal
vectors, frozen protocol/portable fixtures, canonical_probe/C# interoperability,
real Unix daemon/CLI/REPL, document Save/Open and receipt/replay checks pass.

All 33 native children pass correctness/resources: 24 main, six existing, three
small. All final domain charges are zero, checked bulk dispatches are one, no lane
rejections/timeouts/RSS overruns occur, and all 153 comparisons in the native report
pass. That report also checks workloads beyond the 112 comparisons per platform
in the Windows/Linux audit; these counts are not interchangeable configurations.
The three-platform matrix total is 99 children; including the retained Linux
diagnostic repeat, 100 children pass correctness and final zero-charge assertions.
The original Linux viewport timing miss remains recorded above.

Construction median [min, max], milliseconds; prior column is the matching native
readiness-private workbook fixture, not the retired reference implementation:

| N | Shape | Canonical core batch | Canonical workbook | Prior private workbook |
|---:|---|---:|---:|---:|
| 1,000 | independent | 3.43 [3.38, 4.16] | 8.17 [7.39, 9.47] | 6.39 |
| 1,000 | mixed | 3.09 [3.07, 3.40] | 7.42 [6.77, 8.44] | 6.45 |
| 1,000 | chain | 5.15 [4.77, 5.62] | 8.20 [7.68, 9.22] | 8.05 |
| 1,000 | fanout | 5.28 [4.92, 5.86] | 8.81 [8.30, 10.00] | 8.91 |
| 10,000 | independent | 22.50 [21.24, 27.55] | 60.47 [60.29, 65.95] | 60.83 |
| 10,000 | mixed | 26.88 [26.32, 30.92] | 65.34 [64.93, 69.81] | 65.75 |
| 10,000 | chain | 23.49 [23.42, 24.10] | 63.27 [62.84, 66.01] | 62.86 |
| 10,000 | fanout | 48.38 [47.73, 49.06] | 85.39 [84.23, 85.76] | 85.66 |
| 100,000 | independent | 246.49 [245.88, 247.55] | 737.09 [733.36, 738.14] | 745.03 |
| 100,000 | mixed | 307.94 [306.92, 308.81] | 786.53 [777.26, 792.69] | 795.90 |
| 100,000 | chain | 249.60 [248.37, 256.58] | 742.72 [731.13, 745.65] | 741.71 |
| 100,000 | fanout | 536.94 [536.42, 539.79] | 999.72 [985.13, 1007.80] | 999.50 |

Native construction remains near the accepted representation's prior measurements;
no universal speedup is claimed. 100K numeric construction requests 651.25 MB /
5.82M allocations, consistent with the prior private-index implementation.
Numeric ADD10K is 83.23 ms, UPDATE10K 24.23 ms and FORMULAS10K 35.94 ms.
Full distributions and historical comparisons are retained in the native report.

100K-start workbook p95, milliseconds (population later 133,300 plus scratch):

| Shape | Input/capture | Selected | Short formula | Viewport |
|---|---:|---:|---:|---:|
| independent | 0.021 | 0.024 | 0.028 | 0.340 |
| mixed | 0.020 | 0.027 | 0.027 | 0.384 |
| chain | 0.023 | 0.712 | 0.024 | 0.314 |
| fanout | 0.020 | 0.031 | 0.031 | 0.561 |

The 1K-start fan-out viewport case corresponding to the earlier Linux miss passes
natively at 0.562 ms p95; this does not establish the cause of Linux tail variability.
Small 1/8/32-node create/capture/evaluate/drop p95 is 0.022/0.336/0.749 ms;
there is no matched historical readiness-small baseline.
Existing 100K + 100K core additions are 279.10 ms numeric / 554.43 ms fan-out.

Private workbook whole-child numeric CPU/high-water is 2.912 s / 238.91 MiB;
fan-out is 3.779 s / 312.34 MiB. These include all constructions, edits and retained
captures, not construction-only CPU/RSS. Largest completed-child wait4 high-water
across all native cases is 312.34 MiB, below 8 GiB. This matrix excludes reference
cases; comparing its maximum to a prior reference-inclusive maximum would not
establish a memory improvement. Requested live peak at 100K remains 184.73–242.75 MB,
domain peak 339.86–840.25 MB, and 16 retained generations add about 99–105 KB.
Caller root release is 1.42–1.71 microseconds; maximum cleanup task 32.57–51.68 ms,
final drain 53.06–69.54 ms. Cleanup is off the caller, not eliminated.

All eight sparse pressure samples were normal; sampled RSS maximum 197.94 MiB
is distinct from per-child wait4 high-water. Native live RSS guarding remains
unavailable and external critical-pressure monitoring is still required. Earlier
warning-pressure findings remain valid for their workloads. No RSS-baseline,
OS-capacity increase, GUI/AOT, cross-user denial or durability claim is made.

## Review limits

Consolidation is not claimed to improve every timing. Current/prior samples expose
variation and any slower observations; allocation/retained representation and
investigation budgets are the relevant gates, not an assertion of identical timing.
No product caps, runtime worker/queue/domain budgets, protocol or fixture bytes change.
Historical macOS pressure observations remain relevant. Fresh native APFS validation
passes on matching source; the original Linux viewport miss remains a review limitation.

