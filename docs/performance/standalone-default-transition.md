# Standalone persistent-default transition — evidence

Date: 2026-10-19. Base: `532f60d7cd8283c92752ef0d5f082aef48c0cd05`.
This is the controlled default-facing transition, not another graph representation.
Native macOS ARM64 transition validation now passed; the native addendum below
validates the new default/lifecycle policy independently of accepted Stage 2 results.

## Method

```text
python scripts/standalone_transition.py --output-dir artifacts/standalone-transition
cargo test --locked --workspace --all-features
cargo test --locked -p pivot-core --no-default-features
cargo clippy --locked --workspace --all-targets --all-features -- -D warnings
cargo clippy --locked -p pivot-core --all-targets --no-default-features --features scaling-experiment -- -D warnings
cargo clippy --locked -p pivot-core --all-targets --no-default-features -- -D warnings
python scripts/ci.py
```

Ordinary model/evaluation/invocation/interface tests now exercise persistent new;
reference differential/research fixtures explicitly retain the oracle. Default CI
now includes core-integration rather than proving only the legacy default path.
Boundary assertions cover actual host/storage/workbook constructors and restore.

The new harness retains Stage 2's thirty isolated processes per platform, three
warmups and 31 measured samples per row, nearest-rank p95, identical arithmetic
oracles, fixtures, ordinary Release settings and counting allocator. Names
`reference` and `integrated` label explicit reference versus direct default-policy
persistent creation with WorkingModel::new_in and a shared runtime. No enable call
or populated legacy activation precedes persistent editing. Construction time now
includes all direct insertions and binding, reported separately as CONSTRUCTION_NS.
POST_CONSTRUCTION_DRAIN_NS is a flush, not activation latency. The initially-invalid
case saves its bad draft on directly persistent state; Stage 2 restore/import-invalid
regression tests remain separately retained.

Every process first constructs an independent small model with ordinary
WorkingModel::new, edits/calculates it against an independent integer expectation,
then drains and shuts down its convenience runtime and asserts zero domain charge.
SMALL_DEFAULT_NS includes cold convenience-runtime initialization and calculation
(the process recovery worker was already initialized by the owned runtime); SMALL_CLEANUP_NS
records drain/join. Both backends perform this identical prelude before timing bulk
fixture construction. Only the fixed process recovery worker survives that shutdown.

Local edits include staging, installation, capture, selected calculation where
applicable and disposal. Ports use up to 1,000 declared inputs. Bounded inspection
is distinct from explicit full snapshot/diagnosis; whole-graph costs retain their
known slower-than-reference behavior. Portable transfer stays at the frozen admitted
1K fixture; 10K/100K exercise refusal. Contention uses two complete-snapshot producer
threads on a second model sharing the persistent runtime. Process-wide allocation
counters in that row include background work. Retention measures 16/32 changed
formula captures. Final root enqueue, drain, join, cumulative destructor wall
intervals and final zero domain charge are recorded separately.

Budget/guards remain 2 GiB domain admission, 8 GiB process RSS and 600 seconds per
child; no higher product capacity is claimed. Source manifests and environment
records identify exact inputs. Runs are sequential, with no concurrent CI/build
workload. Requested heap, allocation reservations and RSS are different quantities.
The runner's native macOS RSS limitation remains unchanged.

Original targets remain input/capture <5 ms, short formula edit/capture/calculation
<10 ms, input plus 1,000-formula chain <25 ms and bounded inspection <5 ms at 100K.
Full snapshots/diagnosis are separately reported compatibility work, not interactive
capture. See [runtime and constructor policy](../architecture/standalone-default-transition.md).

Final measurements and CI results follow.

## Final optimized measurements

All thirty children passed on each platform, with matching final-source hashes, zero final session-domain charge and zero small convenience-domain charge.

- Windows: `artifacts/standalone-windows-verified/20261019T185214Z` (CSV, summary, source/environment manifests and build log).
- Linux: `artifacts/standalone-linux-verified/20261019T185420Z` (CSV, summary, source/environment manifests and build log).

Persistent p95 (milliseconds):

| Operation | Windows 1K | Windows 10K | Windows 100K | Linux 1K | Linux 10K | Linux 100K |
| --- | ---: | ---: | ---: | ---: | ---: | ---: |
| unchanged_capture | 0.0009 | 0.0012 | 0.0013 | 0.0011 | 0.0044 | 0.0021 |
| input_capture | 0.0446 | 0.0233 | 0.0416 | 0.1174 | 0.1333 | 0.1339 |
| input_chain | 0.6122 | 0.5854 | 0.5842 | 0.6406 | 0.6888 | 0.6846 |
| formula_front | 0.6479 | 0.5176 | 0.5980 | 0.6601 | 0.6373 | 0.6953 |
| formula_middle | 0.7370 | 0.6945 | 0.8015 | 0.6559 | 0.8471 | 0.8380 |
| formula_tail | 1.0386 | 0.8450 | 0.9736 | 0.8131 | 0.7965 | 1.0355 |
| rename | 0.7107 | 0.6570 | 0.6683 | 0.5853 | 0.5461 | 0.5579 |
| add_retire | 0.6360 | 0.5636 | 0.6992 | 0.7118 | 0.6817 | 0.8372 |
| invalid_draft_repair | 0.9359 | 1.0115 | 1.0283 | 0.9062 | 1.1660 | 1.0154 |
| named_add_formula | 0.0855 | 0.0679 | 0.1460 | 0.2513 | 0.2199 | 0.2683 |
| port_update_invoke | 0.7231 | 1.9521 | 1.6088 | 0.5744 | 2.1441 | 1.7138 |
| port_create_rename_remove | 0.6989 | 2.5513 | 2.2792 | 0.9803 | 2.8472 | 3.2256 |
| initially_invalid_repair | 1.0351 | 0.9782 | 0.8773 | 0.9745 | 0.9974 | 1.0460 |
| inspection_capture_page | 0.0059 | 0.0108 | 0.0104 | 0.0065 | 0.0085 | 0.0087 |
| reverse_edge_page | 0.0004 | 0.0005 | 0.0004 | 0.0003 | 0.0003 | 0.0003 |
| full_snapshot | 1.8279 | 12.9796 | 136.4949 | 1.0999 | 10.7379 | 131.8013 |
| complete_diagnosis | 1.8691 | 11.8103 | 129.4627 | 1.4309 | 11.5514 | 132.8076 |
| input_capture_contended | 0.0457 | 0.0468 | 0.0482 | 0.1238 | 0.1239 | 0.1225 |

100K reference versus persistent p95 (ms):

| Operation | Windows reference | Windows persistent | Linux reference | Linux persistent |
| --- | ---: | ---: | ---: | ---: |
| input_capture | 84.091 | 0.042 | 59.988 | 0.134 |
| input_chain | 82.126 | 0.584 | 62.837 | 0.685 |
| formula_tail | 66.986 | 0.974 | 63.263 | 1.035 |
| inspection_capture_page | 39.305 | 0.010 | 36.354 | 0.009 |
| full_snapshot | 75.013 | 136.495 | 73.214 | 131.801 |
| complete_diagnosis | 73.900 | 129.463 | 68.260 | 132.808 |
| input_capture_contended | 105.278 | 0.048 | 88.366 | 0.122 |

Resource/cleanup evidence for 100K edits (MiB or milliseconds as labeled):

| Platform/backend | Live requested MiB | Peak requested MiB | Domain peak MiB | 32 retained extra MiB | Drop/enqueue ms | Drain ms | Join ms | Total destructor wall ms |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| windows/reference | 32.7787 | 1009.9302 | 1.6914 | 947.5411 | 23.2396 | 0.0011 | 0.2938 | 0.0000 |
| windows/integrated | 43.5748 | 43.8066 | 42.9132 | 0.1273 | 0.0105 | 44.2074 | 0.2312 | 496.5054 |
| linux/reference | 32.7793 | 1009.9308 | 1.6914 | 947.5411 | 50.2149 | 0.0026 | 0.2407 | 0.0000 |
| linux/integrated | 43.5754 | 43.8072 | 42.9132 | 0.1273 | 0.0203 | 51.5106 | 5.6838 | 669.3645 |

Cumulative cleanup includes direct fixture construction, warmups, measured operations and retention; it is not just the measured row work and sums potentially overlapping destructor wall intervals. Reference inline destruction appears in its drop/row times, not runtime counters. Reference domain figures contain runtime overhead only; reference graph allocations are not domain-managed.

Windows reference: maximum observed process RSS 1108.65 MiB; 100K edit-fixture construction 81.74 ms. Windows integrated: maximum observed process RSS 236.35 MiB; 100K edit-fixture construction 3012.89 ms. Small cold convenience initialization/edit/calculation observations ranged 0.387–1.004 ms across children (one observation per child, not a p95 estimate).

Linux reference: maximum observed process RSS 1191.42 MiB; 100K edit-fixture construction 79.98 ms. Linux integrated: maximum observed process RSS 248.89 MiB; 100K edit-fixture construction 11580.42 ms. Small cold convenience initialization/edit/calculation observations ranged 3.530–4.809 ms across children (one observation per child, not a p95 estimate).

All original 100K default-facing targets passed on both platforms. Complete snapshots and diagnosis remain slower than reference and explicitly chosen; no general million-cell capacity claim is made.

Shared-runtime 100K complete-work reservations (MiB; not RSS):

| Platform | Full snapshot/diagnosis case peak | Two-model contention case peak |
| --- | ---: | ---: |
| windows | 635.96 | 1270.41 |
| linux | 438.03 | 1072.42 |

Direct bulk construction is substantially slower than reference insertion because each public mutation is dispatched and reclaimed through the shared runtime. The measured setup costs above are disclosed separately; they do not invalidate the passed single-edit targets, nor establish efficient bulk loading at larger product capacities. No batching optimization is included in this bounded transition.

## Final correctness and platform validation

| Gate | Windows | Linux WSL2 |
| --- | --- | --- |
| Full default CI | Passed | Passed |
| Default workspace tests (persistent standalone new) | 241 | 242 |
| Combined workspace tests | 264 | 265 |
| Reference-only core tests | 21 | 21 |
| Strict normal, combined, reference-only and research-only lint | Passed | Passed |
| Verified default-facing Release children | 30/30 | 30/30 |

- Windows CI: `artifacts/standalone-ci-windows/20261019T190504784419Z/summary.json`. Feature/fallback logs: `artifacts/standalone-final-*-windows.log`.
- Linux CI: `artifacts/standalone-ci-linux/20261019T190705960678Z/summary.json`. Feature/fallback logs: `artifacts/standalone-final-*-linux.log`.

Default CI now enables persistent integration, so its counts increase from the accepted Stage 2 reference-default CI. Existing ordinary core model/evaluation/invocation/interface tests retain new and now run persistent. Differential/research fixtures explicitly use reference. .NET builds had zero warnings/errors; all 10,099 decimal vectors and protocol fixtures (58 decimal layouts, 118 authoring fixtures and frozen v1.0 compatibility) passed. Real daemon/CLI/REPL IPC passed. No dependency versions, frozen codecs, generated protocols, public capacity limits or .NET sources changed.

Native macOS transition CI/performance subsequently passed; see the native addendum below. The [native transition packet](../handoffs/2026-10-19-macos-standalone-transition-instructions.md) remains the reproduction procedure. Accepted Stage 2 APFS results remain historical.

## Native macOS transition addendum

Evidence: [native report](../../artifacts/macos-validation-20261019T191136Z/REPORT.md),
[summary](../../artifacts/macos-validation-20261019T191136Z/summary.json), and
[benchmark summary](../../artifacts/macos-validation-20261019T191136Z/transition-release/20261019T191415Z/summary.json).
Apple M1 / Macmini9,1, eight CPUs, 16 GiB RAM, macOS 27.0.1 ARM64.
The fresh dirty working-tree copy ran from internal APFS at
`/private/tmp/pivot-transition-20261019T191136Z.5kj5_6xl`, including targets,
temporary file tests and initial reports. All 274 copied source hashes matched
before this results-only documentation update. No source changes or reruns.

Full CI passed with 242 default tests, warning-free .NET Release, all 10,099 decimal
vectors, protocol compatibility and real Unix-socket daemon/CLI/REPL integration.
Combined tests passed 265, integration-only 242, fallback 21. All requested strict
lint configurations passed. The workspace research-feature test run also passed
265 but includes default integration; the separate core no-default-features
research-only lint passed as well. The fallback test is the true reference-only run.

All thirty Release children passed with 31 samples per measured row, no timeout/
budget stops, and zero final charges for both small convenience and shared session
domains. All original default-facing targets passed. Transition benchmark coverage
now totals ninety successful processes on three platforms.

Native persistent p95 (milliseconds):

| Operation | 1K | 10K | 100K |
| --- | ---: | ---: | ---: |
| unchanged_capture | 0.0004 | 0.0023 | 0.0004 |
| input_capture | 0.0119 | 0.0123 | 0.0127 |
| input_chain | 0.3330 | 0.3337 | 0.3309 |
| formula_front | 0.3372 | 0.3505 | 0.3317 |
| formula_middle | 0.3995 | 0.4196 | 0.4141 |
| formula_tail | 0.5573 | 0.5026 | 0.4905 |
| rename | 0.3650 | 0.3360 | 0.3346 |
| add_retire | 0.3399 | 0.3430 | 0.3553 |
| invalid_draft_repair | 0.4969 | 0.5138 | 0.5125 |
| named_add_formula | 0.0322 | 0.0339 | 0.0351 |
| port_update_invoke | 0.3412 | 0.7249 | 0.7652 |
| port_create_rename_remove | 0.4188 | 1.0928 | 1.0542 |
| initially_invalid_repair | 0.5272 | 0.5288 | 0.5155 |
| inspection_capture_page | 0.0034 | 0.0071 | 0.0073 |
| reverse_edge_page | 0.0002 | 0.0002 | 0.0002 |
| full_snapshot | 0.7135 | 4.4280 | 48.7476 |
| complete_diagnosis | 0.7020 | 5.2278 | 48.5097 |
| input_capture_contended | 0.0379 | 0.0425 | 0.0340 |

At 1K, portable export/import-calculation p95 was 0.387/3.414 ms. Frozen refusal
paths remained unchanged at 10K/100K. At 100K, native reference snapshot/diagnosis
p95 was 30.313/31.224 ms versus persistent 48.748/48.510 ms; complete operations
retain their disclosed cost. The 100K edit fixture required 1,300.037 ms persistent
versus 46.676 ms reference construction. This confirms the per-edit dispatch/setup
tradeoff rather than removing it. Small convenience initialization/edit/calculation
observations ranged 0.113–0.684 ms; these are individual child observations, not p95.

For the native 100K persistent edits case: live requested heap 45,704,230 bytes,
peak requested 45,947,312, domain peak 44,997,717. Sixteen/thirty-two captures added
64,770/133,474 bytes. Final root enqueue/drain/join took 0.0014/12.053/0.094 ms;
cumulative destructor wall intervals totaled 230.959 ms, including construction.
This is moved cleanup work, not eliminated work or CPU-time measurement.

The runner still reports zero native RSS as unavailable, and its RSS guard was
ineffective. Independent monitoring recorded eight samples, normal memory pressure
and a sampled benchmark-process maximum of 253,116,416 bytes (241.39 MiB). This
is not per-child high-water evidence and cannot be directly compared as such to
Windows/Linux measurements. AC power/no thermal warning and exact native tools
are recorded in the environment/post-run artifacts. Full native allocation,
retention, contention and cleanup measurements remain in the raw CSVs/report.
