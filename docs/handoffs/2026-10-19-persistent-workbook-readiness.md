# Persistent Workbook Production Readiness and Construction Profiling

Historical milestone: the subsequent owner decision in [ADR-0013](../adr/0013-canonical-engine-consolidation.md)
retires the reference runtime. Constructor-selection recommendations and source
links below describe the pre-consolidation revision; benchmarks remain evidence.

Date: 2026-10-19. Base: d2e2a8b82504d4ac904c7e0609fe19b0d91a1653.
The parent accepted ADR-0012 and the opt-in bulk foundation. This follow-up profiles
the remaining construction gap and tests one compatible optimization. No production
Workbook constructor or host/storage/document path is migrated.

Windows/Linux 100K construction improves about 1.5–2.1x through private-node reuse;
all 234 measurement children and three-platform CI pass (300/301/301 all-feature
tests). Native macOS corroborates a 1.44–1.68x 100K improvement. Allocation
traffic drops substantially; peak memory remains similar. Recommend explicit new_in for the next bounded
application/session implementation after review, with no
default constructor switch in this slice.

## Review material

- [Construction evidence](../performance/persistent-workbook-readiness.md)
- [Implementation contract and full compatibility matrix](../architecture/persistent-workbook-readiness.md)
- [Native APFS validation packet](2026-10-19-macos-persistent-workbook-readiness-instructions.md)

## Change and limits

Existing worksheet AVL indexes reuse uniquely owned private nodes; shared published
nodes are copied before mutation. This reduces avoidable per-insertion path copying
without changing the canonical representation. Internal insert_private requires its
caller to discard a failed candidate; the workbook already does so atomically and
retires the partial root through its pre-admitted cleanup owner. Ordinary insert
and all core mutation/validation/binding paths remain unchanged. Heap-owning payloads
retain functional insertion, keeping charge sizes correct. Deletion is unchanged.

The same binary supplies an immutable-insertion research control. Profile runs
attribute exclusive thread-wall time and allocator traffic; ordinary runs establish
performance. No timer-instrumented result is presented as normal latency. Unix
wait4 now records whole-child CPU and OS high-water, including on macOS after exit;
this improves evidence without pretending to provide a native live RSS guard.

Shared accepted formula/projection suites and exact differential oracle remain the
compatibility tests. New canonical tests cover rotations/shared roots, partial
admission failure and heap-payload charge fallback. New workbook lifecycle coverage
uses four bounded 1K-cell configurations, rejected late edits/admission/cancellation,
8,192 edits, 128 retained generations, per-round baseline and final zero charge.
Input arrays and returned caller-owned buffers remain outside the retained ledger.
Large worksheet preparation still belongs on an application/session worker.

No functions/ranges/copy-fill, persistence/SQLite/files, C# IPC, Avalonia, new value
types, cap increases or speculative graph architectures are included. GUI/AOT,
durability and million-cell support remain outside validation. The accepted native
foundation warning-pressure finding remains relevant, even if the improvement
reduces allocation traffic.

## Selection recommendation

Three-platform measurements support recommending persistent new_in as the preferred
implementation for the next bounded application/session slice, conditional on
owner review, with an explicitly owned shared runtime. Keep Workbook.new reference and explicit fallback/oracles in
this slice. No host/document/storage migration through feature unification. Stop
for review before constructor policy changes; the next delivery priority should be
a runnable C# application/session interface, not indefinite backend optimization.


## Matched Windows/Linux findings

All 156 isolated children pass (78 per platform); all 68 core/workbook/harness source
hashes match each other and the final tree, all final charges are zero and no normal
lane rejection occurs. Profiled diagnostics are separated from ordinary medians.

| 100K workbook shape | Windows control → private | Linux control → private |
|---|---:|---:|
| Numeric | 2.439 → 1.158 s | 1.971 → 1.039 s |
| Mixed | 2.575 → 1.297 s | 2.085 → 1.153 s |
| 1,000-chain | 2.457 → 1.165 s | 1.985 → 1.066 s |
| Fan-out | 3.133 → 1.808 s | 2.473 → 1.598 s |

Numeric requested construction traffic falls from about 1.492 GB / 14.0 million
allocations to 651 MB / 5.82 million. Windows numeric worksheet-index traffic itself
falls from 930 MB / 8.83 million calls to 89 MB / 630K calls. Independent same-binary
control and unchanged expected outcomes substantiate the optimization; timer probes
inflate the wall times and are not used as latency claims. Identity planning and
validation were not the dominant numeric construction cost. Canonical name-index
path copying, graph page/record work and their admission/Arc costs explain much of
the remaining gap; no claim is made that further improvement is impossible.

All established interactive and investigation bulk budgets pass. After later
population reaches 133,300 cells, worst private workbook p95 input/capture is
0.050 ms Windows / 0.153 ms Linux, 1,000-chain calculation 1.060 / 1.160 ms, short
formula 0.049 / 0.225 ms and rectangle 0.836 / 4.406 ms. Ten-thousand numeric additions
are 142 / 137 ms and formula replacements 74 / 122 ms in the numeric fixture.
Core100K numeric/fan-out controls remain inside 1 s / 2 s. Full three-sample ranges,
phase rows, original sources and all gate comparisons are in the performance record.

## Memory and lifecycle assessment

Allocation churn improves; retained size does not substantially change. At 100K,
requested live peak remains about 185–243 MB and admitted domain peak 340–840 MB.
Sixteen retained formula generations add roughly 103–107 KB. Linux wait4 reports
whole-child high-water about 272 MiB numeric / 342 MiB fan-out persistent versus
1.06 / 1.33 GiB for the retained-copy reference cases. Whole-child CPU also includes
all edit/read/retention work; neither CPU nor OS peak is construction-only.

Caller final-root release is 2–4 microseconds Windows and about 20–25 microseconds
Linux. Actual cleanup tasks can take 103–151 / 138–159 ms, with final drain
162–232 / 228–289 ms. This remains off the caller rather than disappearing. Each
churn round returns to the runtime ledger baseline and final shutdown reaches zero;
benchmark requested-live heap after reclamation is around 116 KB (runtime/tool
headers still present). This is not proof that the system allocator returns RSS to
its initial level. Caller-held result/input copies need application-level bounds.

The prior macOS warning-pressure result is retained, not declared fixed by reduced
traffic. The fresh readiness run observed normal pressure in all 52 samples. wait4 now supplies
completed-child native high-water, while live RSS guarding still needs the external
critical-pressure monitor. Native OS-capacity, GUI/AOT/durability and million-cell
claims remain excluded.

## Native macOS findings

The [fresh APFS report](../../artifacts/macos-validation-20261020T010244Z/REPORT.md)
passes all requested gates, 252 default / 301 all-feature / 21 fallback / 49 workbook-only
tests and 78 benchmark children. Full CI includes .NET Release, 10,099 decimal vectors,
frozen protocol and real Unix IPC. All 296 copied source hashes matched this tree
before documentation edits; all five benchmark manifests match, and 464 returned
report files were transfer-verified. No native source fixes or reruns were needed.
The readiness total is 234 children across three platforms.

Native 100K numeric/mixed/1K-chain/fan-out construction is 0.745/0.796/0.742/0.999 s,
1.44–1.68x faster than immutable insertion, still 2.3–2.6x the optimized reference.
All ordinary review targets pass: worst private workbook p95 input/capture 0.040 ms,
selected chain 0.682 ms, formula 0.027 ms, inspection 0.533 ms. Numeric ADD10K is
87.52 ms and FORMULAS10K 36.41 ms. Source-matched profile evidence confirms the same
worksheet path-copying cost; full distributions and raw links are in the performance record.

All 52 pressure samples were normal; maximum completed-child wait4 high-water was
1,418.80 MiB across all cases (private persistent numeric/fan-out 239.11/312.45 MiB).
Sparse sampled RSS differs from child high-water. Earlier foundation warning pressure
remains relevant, native live RSS guarding remains unavailable, and no OS-capacity
increase is claimed. Retained size remains similar and every final domain charge is zero.

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
