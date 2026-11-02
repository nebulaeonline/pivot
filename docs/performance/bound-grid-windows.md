# Bound Grid Window qualification

2026-11-02. Accepted by implementation review and ADR-0041 closed, with the
unresolved Windows ARM64 cold headless timing qualification retained. All six
targets passed managed/AOT GUI suites. The separate retained-cache CI pass does
not establish cold-run reliability. No further qualification round is required.

The Release corpus uses 1,024, 10,000 and 65,536 total computation nodes, including
worksheet cells, two formula consumers and one shared external Decimal leaf. A
live worksheet Window preserves physical positions; a fixture Window retains exact
String data. Timings cover source edits/guard maintenance, calculation, bounded
projection, explicit refresh publication, WB15 Save and reopen. Two old captures
and superseded refresh permits remain owned during the workload. Every completed
checkpoint releases its runtime charge to zero.

These are individual helper observations, not latency percentiles or graphical
responsiveness measurements. Memory columns are runtime-accounted bytes, not file
size or process RSS. The byte figures include retained generations and reopen work.

| Target | Edit ms | Calculate ms | Refresh ms | Save ms | Reopen ms | Steady charged bytes | Peak charged bytes |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| Windows x64 | 100.267 | 97.096 | 159.377 | 394.977 | 1817.036 | 163,016,237 | 380,891,066 |
| Linux x64 | 111.521 | 99.853 | 165.000 | 460.498 | 1947.650 | 163,016,621 | 380,891,450 |
| Windows ARM64 (retained cache) | 57.726 | 65.098 | 87.838 | 296.172 | 1957.560 | 163,017,101 | 380,891,930 |
| Linux ARM64 | 69.700 | 59.863 | 108.327 | 254.383 | 1071.970 | 163,016,333 | 380,891,162 |
| macOS ARM64 | 55.173 | 42.635 | 81.253 | 212.615 | 803.842 | 163,016,237 | 380,891,066 |
| macOS x64 / Rosetta | 61.450 | 54.099 | 97.803 | 317.008 | 1471.582 | 163,016,525 | 380,891,354 |

Observed bound-source operations fit the existing one-second mutation and
ten-second persistence objectives. Accounted steady/peak remain below 256/512 MiB.
No admission limits or timing thresholds were raised. Existing worksheet and
unbound-Window corpora remain in headless CI.

## Qualification records

The first six-target headless matrix is
`artifacts/validation-matrix/20261102T163837Z-0a068ba1`. Five targets passed all
13 CI gates. Windows ARM64 failed the existing 65,536-cell compact structural
command objective: measured Rust durations were 1338.990, 1613.571, 696.129 and
467.314 ms. This original cold failure remains unresolved and is preserved.

A separate full Windows ARM64 CI run using its existing build caches passed all
13 gates. Its receipts are in `artifacts/bound-win-arm64-warm-01`; the original
failed result was not replaced. The table's Windows ARM64 row belongs to this
separate retained-cache run. It does not establish a cold-run pass.

All six native working source snapshots match the frozen source hashes. macOS x64
is Rosetta evidence. Actual compiler identities record Rust nightly
1.101.0 dated 2026-11-01. Local all-features Rust tests and strict Clippy passed.

Windows x64 managed GUI passed. Its AOT application and all graphical peers passed,
but the final wrapper retained an obsolete WB14 envelope check and reported a
failure after execution. The original failed receipt remains unchanged. The
corrected WB15 validator passed its headless tests on all six targets; independent
read-only revalidation checked the retained WB15 digest, graphical reports,
published-file hashes and absence of owned processes. This supplemental receipt
is `artifacts/bound-win-x64-aot-revalidation-01.json`; GUI was not reexecuted.

Only `scripts/verify_native_aot.py`, its existing fixture test and the headless
execution-safety test entry changed in the second source freeze. Product code is
unchanged. Remaining AOT launchers use that corrected validator. A fresh complete Windows x64 AOT retry subsequently passed; receipts are in
`artifacts/validation-matrix/20261102T175029Z-a69c894e`. All four graphical
reports, zero charges, publication checks and process cleanup are verified. Windows ARM64 managed
GUI passed; its AOT publishing attempt failed on NuGet DNS resolution before any
GUI execution. Clearing the guest DNS cache restored resolution and HTTPS access.
A separate headless publish preflight passed with warnings treated as errors, then
a fresh complete Windows ARM64 AOT graphical retry passed. Receipts are in
`artifacts/validation-matrix/20261102T180002Z-84659b21`: all four native ARM64
graphical reports, zero charges, process cleanup and 921 source-input hashes verify.
Linux x64 managed/AOT subsequently passed, with four graphical reports per suite,
zero charges, no owned survivors and matching source snapshots. Linux ARM64
managed/AOT and macOS ARM64 managed also passed with the same verification.
All remaining macOS suites subsequently passed. Final GUI evidence is recorded in
`artifacts/bound-final-gui-qualification.json`: 12 passing suites, 48 graphical
reports, zero final charges and no owned survivors. Actual source snapshots match
their frozen manifests. The final Mac collections are
`20261102T190853Z-4da6d187` (managed) and `20261102T190856Z-7f40b5ee` (AOT).
The Windows ARM64 failure shown in that older AOT collection is its original DNS
failure; its accepted fresh retry is separately recorded above.

## Ownership and admission workloads

The coexistence regression admits 32 producers and 64 bound Windows: one selected
128-key/two-field fixture, 31 worksheet sources of 16 rows/two columns and two
views per producer. It populates 992 worksheet cells, 64 formula consumers and 32
shared Decimal leaves: 1,088 graph nodes across 1,248 projected row positions.
Bound display positions allocate no CellIds. Duplicate consumers through both
fixture Windows share leaves. Old captures retain their selected payload after
refresh; superseded tickets retain their permits until actual release.

An external-slot regression accepts 65,536 aggregate selected record-field slots,
then refuses another producer's refresh without changing the retained selection.
Schema, producer/Window/configuration relationships, key policy, guard authority,
captured outputs and forbidden independent bound formatting are checked during
native restore. Rejected candidates return to the drained ownership baseline.

## Retention and code size

All native batches were archived and every archive member was checked by size and
SHA-256 before cache removal. Source snapshots, workbooks/fixtures, original failures,
reports, installed toolchains, pins and latest canonical AOT publications remain
available. The final native audits show no remaining per-run compiler caches.

The local Cargo cache was cleared with `cargo clean`: 30,181 files and 23.0 GiB
(24.7 GB in the inventory). Inventory-verified duplicate publication copies were
also removed from collected/exported reports; those report files remain unchanged.
The retention summary and product-pruning ledger distinguish logical file-size
totals from physical free space. Guest and macOS host free-space changes must not
be summed as physical recovery.

See `artifacts/bound-final-retention-summary.json`,
`artifacts/bound-final-qualification.json`, `artifacts/archives/index.jsonl` and
`artifacts/pruned-build-products.json` for exact receipts and archive lookup.
The final source bundle retains each tested source freeze and the accepted candidate.

The per-project [cloc report](../../artifacts/loc/ADR-0041/latest.md) contains
100,580 source-code lines. Inputs, hashes, tool output and the project table are
retained with the source/evidence bundle. Generated/dependency/build output is
excluded; test/example lines are a subset of project totals.
