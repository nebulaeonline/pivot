# ADR-0037 — weighted allocation implementation review

2026-11-01. Accepted and closed by implementation review.

Review accepts all four mandatory requirements, exact arithmetic and production-SUM
evidence, context-bearing resource admission, unchanged representation/budgets,
six-target results, observed-core/charged-memory qualifications, Rosetta coverage,
diagnostic LoC and verified retention. The Linux ARM64 fixture correction and separate
retry are accepted, preserving the selected-file and late test-only source qualifications.
The numeric-only graphical/AOT scope decision is accepted; historical receipts retain
their original scope.

`ALLOCATEWEIGHTED(total,index,places,weight1,...,weightN)` now returns exact settled
proportional shares for 1..64 explicit scalar weights. Operands are strict Decimal96;
weights are nonnegative with a positive sum, index is one-based and precision 0..28
is required. Blank/String/Boolean are TypeMismatch; invalid domains or a nonintegral
settlement total are InvalidArgument. Group representation failure is arithmetic
Overflow, exposed through the existing NumericError session category.

For total 10, precision 2 and weights (1,2,3), results are 1.67, 3.33 and 5. Actual
values reconcile; formatting remains separate. Negative totals are symmetric, zeros
canonical and ties deterministic. Equal positive weights match ALLOCATE over the
supported domain. No floating-point allocation, runtime BigInteger or hidden state.
See [semantics](../architecture/weighted-allocation.md) and
[ADR-0037](../adr/0037-weighted-conservation-preserving-allocation.md).

## Four mandatory clarifications

**Explicit association.** Recipient index selects a weight-list ordinal. There is no
row, label or durable business-record lookup. Copy/Paste, ordinary/visible Fill, sorting
and structure use normal reference rules without renumbering or repair. The real session
regression sorts weight cells independently: index 1 retains its ordered bound weight,
returning 7.5 even though the current worksheet row now contains the other weight.
Completeness requires the same ordered specification and every index exactly once.
The (5,3,1) whole-unit example retains the documented nonmonotonic results when total
changes from 4 to 5. No cross-total fairness guarantee is introduced.

**Final awarded shares.** Exact quota/remainder ranking happens before narrowing.
Only actual final shares are validated; unused floors/ceilings never cause false
Overflow. Total M at precision 1 with weights (M,10^-28) returns (M,0), despite an
unrepresentable unused floor. M denotes 2^96−1; exponent notation here is descriptive.
The largest proper subset of strictly nonzero shares must fit at common normalized
scale. Full totals can normalize further. Adding zero shares does not force an
unnecessary full-scaled-total coefficient check. All valid indices share the same
representation decision, including zero-weight recipients.

Production SUM passed for 372 complete groups in three orders: **1,116 SUM checks**.
These include exhaustive small weight lists/totals, symmetric negatives, zero terms
first/middle/last, 28-place totals, M with a single positive share, M at weights (1,2),
and the unused-floor boundary. Every valid index is separately checked in group-wide
Overflow cases. Actual SUM is unchanged; mathematical oracles are supplementary.

**Metering and cancellation.** The public Rust typed-operation API now requires
remaining visits, an allocation domain and a cancellation callback. Its weighted
path reserves 4+16*N visits plus 16 KiB scratch before numeric execution. Normal
argument charges remain, without refunds or global limit increases. There is no
public context-free typed fallback. Captured, structured-token, scenario and worksheet
execution use the canonical path; pure bounded arithmetic accepts caller-owned scratch.
Direct and canonical tests cover work exhaustion, scratch refusal and cancellation
inside arithmetic. Ranking has its own cancellation test. Charges are released and
capture/revision state preserved; quiescence precedes exact ledger comparisons.

The helper uses six-limb units/weights, seven-limb denominator/remainders and twelve-limb
products, exact bounded long division and insertion ranking. Explicit storage is 9,472
bytes including input copies and fixed local/control allowance, below the admitted
16 KiB. The limb ceiling remains 4,096+16,384*N, at most 1,052,672. Static loop bounds,
a checked arithmetic-loop counter, independent extreme-value oracles and failure tests
support this accounting. Observed counters are not the proof or a CPU instruction count.
Compiler frames and existing evaluator argument storage remain separate.

**Consistent arity.** Scalar weight count 1..64 means total arity 4..67. Parser,
lowering, typed vocabulary, trusted-token installation, native decode/restoration and
execution share that rule. Malformed direct calls refuse before scratch allocation;
codec tests exercise all arities 0..100 and oversized/unknown-operation cases. Invalid
resolved tokens preserve the previous revision. Existing source/token/depth/dependency,
frame, request and runtime limits still apply independently.

## Integration and representations

Opcode 20 fits the generic typed opcode/arity representation. WB12, private session
v13, hosted v1 and CB5/CB6 are unchanged; unsupported engines refuse the operation.
No canonical value, generated DTO or Desktop interaction changed. The Rust typed
execution signature changed to enforce explicit resource context; this is not a new
wire field. ALLOCATE, division, ROUND, SUM, formatting and bindings retain semantics.

Real C#–Rust session checks cover values/errors, lazy branches, production SUM,
retained captures, CB5/CB6, ordinary/visible Fill, formatting independence, sorting
and changed weight association, structural bindings, native Save/New/Open, error
filtering and zero final session charge. Row/column insertion and deletion each
round-trip weighted Blank-precision and SUM formulas alongside the existing ROUND
control. Structured-token/scenario adapters are separately tested.

## Arithmetic evidence

Each of the six targets passed **12,841 weighted vectors**, with zero mismatches against
independent Python Fraction/subset and .NET BigInteger/ranking references. Comparisons
include exact coefficient, sign, scale and error category. Small groups enumerate
proper subsets; larger groups use algebraic complements. Vectors include signs, strict
metadata bounds, zero/all-zero/negative weights, extreme and mixed-scale values, positive
common rescaling, equal-weight cases, nonmonotonicity, ties and large products.

Weighted input SHA-256:
`545e18fe505a50f915989015bd9705c024956f13f892768badda30878555e08d`.

The existing 51,523 equal-allocation vectors, 10,099 decimal vectors and other ordinary
CI gates also pass. No ordinary arithmetic or reconciliation guarantee was weakened.

## Local and native validation

Final local CI passed **13/13** at
`artifacts/weighted-final-local-ci/20261101T113639711481Z/summary.json`.
All-feature Rust regression passed **515 tests**, zero failures/ignored; strict
workspace/all-target/all-feature Clippy passed. Earlier local receipts remain separate.

Windows/Linux x64 use matrix `20261101T103630Z-04c04d2a`. Windows ARM64 and both Mac
architectures use `20261101T103054Z-009bb816`. The selected Linux ARM64 retry is
`20261101T113545Z-5e9dca07`; its original failed run is retained, not relabeled.
`artifacts/weighted-selected-matrix.json` selects final results, with qualified per-target
receipts in `artifacts/weighted-qualified-<rid>.json`. macOS x64 is Rosetta evidence.

All five resource modes have 16,384 base cells, 64 weight cells, 256 weighted formulas,
one total input and one SUM output: **16,706 positioned cells**. The first 64 formulas
form the complete group. Four changed-input samples retain four captures. Modes cover
ordinary ratios, mixed scales, zero/tied weights, wide products/final normalization and
proper-partial Overflow. Helper timing covers 256 calls; core timing includes evaluation
of 256 scalar outputs and a separate production SUM of the first 64. No request limit
was raised. Successful-return limb counters are distinct from error-only modes: a zero
reported maximum in the Overflow mode does not mean zero work was performed.

| Target | Headless CI | Maximum core ms | Maximum steady MiB | Maximum peak MiB | Final charge |
| --- | --- | ---: | ---: | ---: | ---: |
| win-x64 | 13/13 | 82.82 | 36.78 | 135.22 | 0 |
| linux-x64 | 13/13 | 84.02 | 36.78 | 135.22 | 0 |
| win-arm64 | 13/13 | 42.20 | 36.78 | 135.22 | 0 |
| linux-arm64 | 13/13 | 39.02 | 36.78 | 135.21 | 0 |
| osx-arm64 | 13/13 | 36.61 | 36.78 | 135.22 | 0 |
| osx-x64 (Rosetta) | 13/13 | 52.68 | 36.78 | 135.23 | 0 |

These are observed maxima across four samples per mode, not percentiles, GUI latency
or RSS. Memory is charged integration-runtime memory. Every mode passed the original
one-second bounded-operation and 256 MiB steady/512 MiB peak objectives. No quota,
weight cap, work/scratch bound or target was enlarged to obtain a pass.

No new attended graphical/Native AOT round was required under the approved numeric-only
scope: no production Desktop interaction, generated DTO metadata or AOT dispatch changed.
Historical graphical receipts remain historical rather than ADR-0037 executions.

## Failure retention and source qualification

Linux ARM64 initially failed during setup of an existing scratch-admission regression:
1,024 rapid input edits exhausted its eight-slot reclamation queue before its calculation
assertion ran. The fixture now drains retired roots between construction edits. The
queue, actual scratch-admission failure assertion and product behavior are unchanged;
no mutation is retried. Final local CI/all-feature checks and the fresh Linux ARM64
headless run pass with this test-only correction.

Eleven selected arithmetic, evaluator, codec/resource and oracle/session source files
match the final checkout on all six selected targets. Five earlier passing native
snapshots precede the final setup-only edit to
`crates/pivot-core/tests/core_integration.rs`; the Linux ARM64 retry and final local CI
include it. No production code changed after those earlier passes. Documentation was
completed afterward. This qualification does not assert full byte identity for every
historical snapshot or document. Frozen sources, hashes and original outcomes remain.

A local headless session fixture initially encountered a sandbox-denied workbook save;
its permitted rerun passed. The test executable now reports top-level exceptions on
stderr with exit code 1 rather than leaving them unhandled. A controlled missing-worker
fixture verifies failure reporting in every headless CI run. This is a test-runner
correction; production Desktop behavior and machine-wide error-reporting settings were
not changed. Original failures and recoveries remain separate from passing results.

## Diagnostic code size

Final canonical cloc 2.06 count: **90,859 code lines across 400 source files**, +1,400
lines and +9 files versus ADR-0036. The final recount includes the test-fixture correction;
the earlier 90,855-line receipt is retained separately. This is diagnostic evidence,
not a reduction goal. Test/example code is a subset of the project total. Generated
output, dependencies and caches are excluded. Documentation counts describe measurement
time before this final review document. Raw inputs, source digest, tool output and
per-language/project data are under
`artifacts/loc/ADR-0037/20261101T113816456043Z`.

| Project | Code | Comments | Blank | Test/example code* | Languages (code) | Change |
| --- | ---: | ---: | ---: | ---: | --- | ---: |
| ApplicationSession | 1,392 | 17 | 34 | 1,392 | C# 1,228, Python 164 | +57 |
| DecimalParity | 394 | 11 | 22 | 394 | C# 113, Python 281 | +117 |
| LegacyProtocol | 161 | 5 | 2 | 161 | C# 43, Protocol Buffers 118 | +0 |
| Pivot.Avalonia | 419 | 3 | 2 | 0 | C# 419 | +0 |
| Pivot.Client | 521 | 25 | 17 | 0 | C# 521 | +0 |
| Pivot.Desktop | 2,889 | 43 | 41 | 1,090 | C# 2,761, XML 128 | +0 |
| PolicyProbe | 1,619 | 54 | 127 | 1,619 | C# 1, PowerShell 38, Python 1,580 | +0 |
| ProtocolParity | 81 | 2 | 8 | 81 | C# 55, Python 26 | +0 |
| pivot-cli | 1,413 | 12 | 3 | 280 | Rust 1,413 | +0 |
| pivot-core | 16,669 | 218 | 171 | 3,192 | Rust 16,669 | +327 |
| pivot-host | 8,431 | 69 | 95 | 4,165 | Rust 8,431 | +0 |
| pivot-protocol | 2,636 | 47 | 29 | 1,023 | Protocol Buffers 565, Rust 2,071 | +0 |
| pivot-session | 3,549 | 17 | 11 | 382 | Rust 3,549 | +0 |
| pivot-storage | 5,134 | 52 | 72 | 1,952 | Rust 5,134 | +0 |
| pivot-transport | 897 | 26 | 13 | 369 | Rust 897 | +0 |
| pivot-types | 1,222 | 26 | 13 | 194 | Rust 1,222 | +449 |
| pivot-workbook | 25,734 | 179 | 121 | 12,030 | Rust 25,734 | +165 |
| pivot-workbook-document | 7,378 | 51 | 29 | 4,699 | Rust 7,378 | +285 |
| pivotd | 1,134 | 20 | 7 | 899 | Rust 1,134 | +0 |
| Other source | 214 | 11 | 4 | 0 | Protocol Buffers 214 | +0 |
| Scripts/tooling | 7,673 | 373 | 671 | 0 | Bourne Shell 126, C 166, DOS Batch 12, PowerShell 438, Python 6,931 | +0 |
| Standalone tests | 1,299 | 87 | 191 | 1,299 | Python 1,299 | +0 |


## Retention and disposition

Member sizes and SHA-256 were verified before pruning. Four-VM cleanup removed 45.68
GiB of logical build output, including both Linux ARM64 attempts; native Mac ARM64/x64
cleanup removed 10.58 GiB. ROBPC's disposable Cargo cache contained 15.85 GiB. Exact
path, expected children, absent reparse links and inactive compilers were checked;
the standard cache marker was restored where absent, then supported cargo clean ran.
These logical totals are distinct from observed free-space changes; host/guest free
space is not added together. Source, reports, original failures, fixtures, configuration,
current/pinned publications and the unrelated unverified Linux x64 legacy run remain.

Local evidence: `artifacts/archives/weighted-final-local-evidence-20261101.zip`.
Final source/receipts: `artifacts/archives/weighted-final-source-and-receipts-20261101.zip`.
Native archives, verification receipts and pruning events are indexed by
`artifacts/archives/index.jsonl` and `artifacts/pruned-build-products.json`.

ADR-0037 is accepted and closed. Range-valued weights, arrays, keyed groups, automatic
repair, fairness state, Desktop licensing implementation and history were excluded
from this implementation. ADR-0023 remains parked at G3.

## Next design direction

ADR-0038 concerns range-based weighted allocation. The intended improvement is an
ordered worksheet range in place of a long scalar list; formula syntax is not yet
fixed. Its design must compare row/column/rectangular ordering, strict Blank and error
policies, explicit recipient indices, ordinary binding and structural affinity, Copy/Fill,
sorting, filtering and cross-sheet behavior. Blank positions must not be skipped or
renumbered silently. Compare the 64-position bound and range admission, reuse the same
exact arithmetic and SUM guarantee, and identify any necessary typed/workbook/protocol
representation changes before implementation.

Keep this extension limited to range-based weights: no keyed groups, persistent
recipient identities, automatic repair, arrays/spills, fairness/history state, visible-only
allocation or new financial subsystem. No ADR-0038 design, code or platform run is
included in this closeout.
