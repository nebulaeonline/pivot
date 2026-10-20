# Canonical engine consolidation

Owner direction: retire reference implementations before application integration.
Historical baseline: `3a6fd9719f491a9236dbf0197ed9d4fb7ade3e24`.
Status: implementation complete; Windows/Linux/native macOS validated, ready for owner review.
One initial Linux viewport p95 miss remains explicit review evidence.

See [ADR-0013](../adr/0013-canonical-engine-consolidation.md),
[architecture and removed inventory](../architecture/canonical-engine-consolidation.md),
[Release evidence](../performance/canonical-engine-consolidation.md) and the
[self-contained native APFS packet](2026-10-19-macos-canonical-engine-instructions.md).

## Result

One mandatory paged graph handle in WorkingModel; one mandatory persistent worksheet
root in Workbook. `new` uses the shared lazy runtime; `new_in` retains explicit
application ownership. Reference constructors, activation/rehome paths, backend
storage/capture/result enums, reference local evaluation, full-copy workbook edits,
backend-selection features and immutable-index comparison switch are removed.
Port validation, parser/binder and decimal expression execution remain shared.
Owned complete diagnostic/portable images are not editable engines. Portable import
uses the checked decoder directly to build canonical pages without reparsing source.

Host, hosted transactions, storage/document creation/restoration and daemon sample
construct canonical models. Existing receipts/replay, expected-state publication,
Save/Open, invalid drafts, IDs, decimal parts, immutable S/T and IPC limits remain
under their existing regression suites. No new provider/container/C# IPC/UI or
capacity increase is included. Complete snapshot/diagnostic paths still have their
explicit broader cost; allocation-domain limits do not bound arbitrary caller copies.

## Tests and historical evidence

The prior scalar workbook differential is replaced by a 256-transaction independent
oracle checking stable identities, exact source/decimal metadata, cycles, failed-batch
rollback and original-generation content. Canonical branch tests retain immutable
snapshot/arithmetic checks; no second full engine is used as an oracle. A new
convenience-workbook lifecycle test verifies shared runtime shutdown refusal while
captures live and zero charge after final cleanup. Accepted admission/cancellation/
churn, ports, transactions and codec tests remain. Shared projection/formula suites
exercise both convenience and explicit-runtime constructors.

Historical timings, allocation/memory caveats, full-copy rationale and workload
specifications remain in the readiness/bulk evidence at the baseline revision.
Deleted prototype tests/examples/runners are recoverable from Git, not a parallel
archive tree. The frozen C# codec probe is retained as `canonical_probe`; only its
constructor and harness reference change. Frozen fixture bytes are unchanged.

Exploratory failures are preserved under artifacts: sandboxed Windows pipe tests
could not connect; unrestricted tests exercise real IPC. One research assertion
assumed a converted graph's commit zero; it now checks increments from actual
construction history. The replacement oracle initially expected a cycle as an inner
cell error; selected-closure rejection is an outer error, and the test now asserts
that existing contract. Strict lint caught a needless range loop. Initial CI exposed
the mistakenly removed frozen-codec probe; it was restored/adapted. These are not
weakened assertions or product-cap changes. Final verification must use corrected
source; earlier partial runs do not count as final validation.

## Validation results

| Gate | Windows | Isolated WSL2 Linux | Fresh native macOS |
|---|---:|---:|---|
| Full default CI | Passed | Passed | Passed |
| Default Rust tests | 275 | 276 | 276 |
| All-feature tests | 291 | 292 | 292 |
| Strict default/all-feature lint | Passed | Passed | Passed |
| .NET Release, decimal/protocol/portable fixtures, real IPC | Passed | Passed | Passed |
| Main/existing/small Release children | 33 | 33 | 33 |

[Windows CI](../../artifacts/canonical-ci-windows-final/20261020T014819958008Z/summary.json),
[isolated Linux CI](../../artifacts/canonical-ci-linux-isolated/20261020T014707639181Z/summary.json),
[Windows all-feature log](../../artifacts/canonical-tests-windows.log),
[Linux feature/CI log](../../artifacts/canonical-linux-isolated.log) and
[benchmark/source audit](../../artifacts/canonical-benchmark-audit.json).
Both platforms use the same current product sources; 113 product/test/harness files
match the Linux copy and each benchmark's 47-file manifest matches the final tree.
Default counts increase because persistent workbook coverage now runs normally;
all-feature counts decrease because obsolete prototype/backend-only tests were
removed and independent canonical tests added. No test count is represented as
an equivalent old reference configuration.

All 99 three-platform main/existing/small children pass correctness and resource assertions. All
final charges are zero; all checked bulk dispatches are one and no lane rejects
occur. All construction, mutation and 100K-start read budgets pass. One Linux
1K-start fan-out viewport p95 misses <5 ms at 5.343 ms (median 1.113, max 6.416).
One targeted repeat passes at p95 3.381 ms (median 0.916, max 3.560), with identical
allocation counts. Preserve the original miss: this suggests tail variability but
does not establish a cause or guarantee the margin. The review includes this
limitation, not an unconditional performance pass. Including the repeat, all 100
children finish with zero domain charge. Full tables/comparisons remain in the
performance record. No further optimization or threshold change was introduced.

## Native macOS corroboration

The [native report](../../artifacts/macos-validation-20261020T015824Z/REPORT.md)
passes all required gates and 33 children on Apple M1 ARM64, with source, builds,
temp files and initial reports on internal APFS. All 280 copied-file SHA-256 hashes
matched this workspace before documentation edits; all three 47-file benchmark
manifests match, completion comparisons show no differences, and 404 returned
report files were transfer-verified. No native source fixes or reruns occurred.
The native core no-default-feature suite passes 62 tests of the same canonical
graph; it is not reference fallback coverage.

100K native workbook numeric/mixed/selected-chain/fan-out construction medians are
0.737/0.787/0.743/1.000 s, near prior readiness-private results. Core numeric/fan-out
is 246.49/536.94 ms. Numeric ADD10K is 83.23 ms and FORMULAS10K 35.94 ms.
All 153 comparisons in the native report pass. Worst 100K-start workbook p95 is
0.0234 ms input/capture, 0.712 ms selected 1K-chain, 0.0313 ms short formula and
0.561 ms viewport. The native 1K-start fan-out viewport is 0.562 ms p95;
the original Linux miss remains a review limitation, unaffected by this pass.

All eight sparse pressure samples were normal. Completed-child wait4 high-water
peaks at 312.34 MiB; sparse RSS samples peak at 197.94 MiB. These are different
measurements; native live RSS guarding remains unavailable. Earlier pressure
warnings retain their historical meaning. No capacity increase or RSS-return-to-
baseline claim is made; caller buffers remain outside continuing domain charges.
Full distributions, lifecycle observations and historical comparisons are in the
performance record. The review now has three-platform evidence, including the
preserved Linux timing exception.

## Review boundary

Stop for review after fresh three-platform evidence is integrated. The owner has
accepted the single-engine direction, not yet this implementation. After acceptance,
prioritize the runnable C# application/session. No additional backend or migration
milestone is proposed. Fresh native validation is complete; implementation acceptance
remains with the owner, including the recorded Linux tail-latency limitation.
