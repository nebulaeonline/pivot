# Persistent workbook integration and bulk foundation — review handoff

Date: 2026-10-19. Base commit: daff0f9bad55afb4cdccccb34db564e7b7f969c1.
Uncommitted working-tree implementation. ADR-0011 is accepted; ADR-0012 is Accepted following parent review.
Production Workbook, host, storage, daemon and document paths remain reference.

## Delivered behavior

Canonical bounded core batches predeclare stable identities/names, prepare one
candidate and publish once. Hard failure, admission rejection, stale base or
cancellation publishes nothing. Logical core revisions count requested operations;
workbook/host state remains one public revision per changed transaction. Formula
names resolve against the final new-name directory; spreadsheet references retain
the accepted sequential candidate-coordinate semantics and exact source provenance.
The same kernel handles formulas, reverse dependencies and invalid drafts.

Opt-in `persistent-core` Workbook uses canonical persistent indexes and charged
metadata, preserving sparse demanded blanks, moves, clear, retirement/reuse and
atomic multi-cell edits. Local captures pin graph plus workbook metadata at S while
T stays editable. Bounded rectangle reads seek rows and reserve copies. Explicit
complete diagnostics/captures retain global contracts; ordinary local paths do not
materialize the entire graph. A 256-step identity-preserving differential test
compares exact public revisions, contents, provenance, diagnostics and values.

Large batches use existing heavy workers; short edits retain mutation workers.
Request metadata, partial private roots, published metadata and old generations
have pre-admitted cleanup ownership. Runtime worker counts/policies are unchanged.
Worksheet preparation remains bounded O(request) CPU on the caller; applications
must schedule large authoring off the UI thread. No asynchronous UI API is added.

## Review material

- [ADR-0012](../adr/0012-bulk-mutation-and-persistent-workbook.md)
- [Architecture and contracts](../architecture/persistent-workbook-bulk.md)
- [Matched measurements and caveats](../performance/persistent-workbook-bulk.md)
- [Native macOS validation instructions](2026-10-19-macos-persistent-workbook-bulk-instructions.md)

No range/SUM, fill/copy, new numeric types, persistence, wire changes, IPC redesign,
UI work or default workbook switch is included. Frozen formats, dependencies and
production resource caps remain unchanged. The 200K core diagnostic is not a product
workbook-cap increase. Caller-owned read copies leave admission after return.

## Validation

Combined feature regression passed 296 Windows / 297 Linux tests; reference-only
core passed 21 on each. Native macOS validation of this source passes as recorded below; the preceding
transition reports remain historical.
Final CI and Release evidence follows in the performance record.

Stop for review before any production Workbook backend selection change.

## Windows Release result

All 117 isolated child runs passed: 96 main matrix, eight isolated first-group
repeats, 12 full-chain diagnostics and one guarded 200K core diagnostic. Main matrix
100K numeric core construction is 579 ms versus 3.28 s individual / 84 ms reference;
formula-heavy core batches improve about 5–9x over individual persistent edits.
100K workbook population is 2.52–3.17 s versus 0.25–0.41 s optimized reference.
Bulk dispatch overhead is resolved as a per-cell bottleneck, while persistent
construction still has a substantial measured reference gap. No parity claim.

Post-addition 133,300-cell workbook p95 edit/capture, short formula and 1,000-chain
calculation satisfy existing targets. Copy/retention/admission/cleanup measurements
and proposed bulk budgets are in the performance note. Final-root release stays on
the bounded enqueue path; cleanup tasks and final drain remain measurable work.
The benchmark's three construction samples are not tail-latency guarantees.

## Linux Release result

All 109 child runs passed: 96 matched matrix, 12 full-chain and one 200K numeric.
100K core batches improve about 21–25x over individual persistent edits; numeric
construction is 499 ms versus 12.2 s individual / 64 ms reference. Workbook population
is 1.91–2.44 s. Existing interactive and proposed bulk budgets pass. The full 100K
chain is a separate closure diagnostic, not a failed 1,000-chain gate. Across both
platforms 226 child runs pass, all final charges are zero and all 66 core/workbook/
harness source hashes match the current tree. See the linked performance record
for complete tables, allocation/RSS distinctions and retained-generation cleanup.

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

## Native macOS validation and review disposition

Native Apple M1 ARM64 validation is complete:
[report](../../artifacts/macos-validation-20261019T220845Z/REPORT.md),
[summary](../../artifacts/macos-validation-20261019T220845Z/summary.json),
[source verification](../../artifacts/macos-validation-20261019T220845Z/source-final-verification.json).
A fresh rsync copy of the dirty SMB tree was tested entirely on internal APFS,
including .NET outputs, targets and temporary file tests. All 290 source hashes
match our tree before this documentation update. No native source changes, skipped
gates or reruns; report return was SHA-256 verified (482 report files).

Full CI, all required strict lints, 249 default / 297 all-feature / 21 fallback /
48 workbook-only tests pass. All 96 main, 12 full-chain and one 200K native children
pass; all final charges are zero, bulk dispatches are one and there are no normal
lane rejections. Three-platform benchmark total is 335 successful children.
The legacy combined-feature command reports 272 tests; the 297 count specifically
uses --all-features and includes opt-in persistent Workbook coverage.

Native 100K numeric construction is 239.8 ms batch versus 1.205 s individual /
43.2 ms reference; core batching improves about 4.5–5.3x across shapes. Workbook
population is 1.14–1.40 s, still roughly 3.5–3.7x slower than optimized reference.
At the later 133,300-cell population, worst native workbook p95 input/capture is
0.023 ms, 1,000-chain calculation 0.662 ms, short formula 0.026 ms and rectangle
0.634 ms. The 10K numeric addition median is 123.6 ms. Existing interactive and
proposed bulk budgets pass. Full 100K chain and guarded 200K numeric diagnostics
also pass, without increasing production caps.

Memory caveat: large cases caused warning-level native pressure, with no critical
stop. Forty-eight sparse samples reached 2,235.25 MiB for a benchmark process;
these are not per-child high-water readings. macOS runner RSS/guard remains
unavailable. Retain this caveat alongside the passing timings and distinguish
requested heap, domain reservations and observed process RSS.

This foundation is ready for parent review with current three-platform evidence.
ADR-0012 is now Accepted following parent review; no production Workbook default switch or
scope expansion is included. Construction remains slower than reference, worksheet
preparation remains synchronous caller work, and native GUI/AOT/cross-user denial,
persistence/durability and million-cell support are not established by this run.


Parent accepted this foundation and authorized the targeted
[production-readiness follow-up](2026-10-19-persistent-workbook-readiness.md).
This report remains the accepted foundation evidence; new implementation changes
require their own validation before any production backend switch.
