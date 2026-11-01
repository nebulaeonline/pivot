# Range-weighted allocation qualification

2026-11-01. Accepted with the Windows ARM64 qualification below under ADR-0038
implementation review. All six targets passed managed and Native AOT GUI qualification.
The acceptance does not resolve or reclassify the original cold-build timing failures.

Each declared corpus has 16,384 base cells, 64 physical weight positions, 256 calls,
four changed totals and four retained immutable captures. The five modes are mixed
weights, mixed scales, zero/tie weights, wide coefficients and group overflow. These
are observed maxima over the declared samples, not percentile estimates or general
worksheet guarantees. Evaluation includes the real engine range adapter; helper timing
covers the unchanged arithmetic. Memory is the charged allocation domain, not RSS,
compiler/runtime stack measurement or GUI memory.

| Target | Helper max ms | Engine evaluation max ms | Steady charged bytes max | Peak charged bytes max |
| --- | ---: | ---: | ---: | ---: |
| Windows x64 | 67.588 | 89.061 | 28,223,572 | 58,686,762 |
| Linux x64 | 71.062 | 91.636 | 28,224,148 | 58,690,866 |
| Windows ARM64, retained build caches | 31.013 | 42.478 | 26,937,252 | 58,688,922 |
| Linux ARM64 | 32.210 | 43.393 | 28,224,148 | 58,690,218 |
| macOS ARM64 | 29.120 | 38.221 | 28,223,572 | 58,693,026 |
| macOS x64, Rosetta | 41.572 | 54.648 | 28,223,860 | 58,681,578 |

All declared range samples satisfy the unchanged one-second engine/helper objective,
256-MiB steady and 512-MiB peak charged-memory objectives. Each child terminates with
zero final runtime charges. No resource limit is increased. Every target matches 1,993
range-backed Python Fraction/.NET BigInteger oracle cases, alongside the existing scalar
weighted parity suite. Selected implementation and qualification source hashes match
the canonical checkout; later documentation edits are outside that source claim.

## Windows ARM64 cold-build qualification

Three initial full headless runs failed the existing 65,536-cell structural-command
objective before the range session checks. Largest observed row-deletion times were
about 1.45, 1.64 and 1.55 seconds. The failures remain genuine recorded failures; the
cause is not established and no threshold, test sequence or acceptance check was relaxed.

The unchanged capacity check passed separately in isolation (all structural actions
522–634 ms at 65,536 cells). The unchanged full session workload then passed from its
existing build (row deletion 462 ms, largest structural action 888 ms). Finally every
one of the 13 CI gates passed using those retained build caches, including the complete
session and resource corpora. This is a separate warm CI result, not a rewritten cold
worker result. It establishes a passing qualified workload, not a guarantee that the
cold-build timing excursions are resolved. Keep this distinction with future evidence.

## Evidence

- ROBPC: `artifacts/range-ci/20261101T141017343581Z/summary.json` (13/13);
  `artifacts/range-final-all-features-tests.log` (525 passed).
- x64 native headless matrix: `20261101T143412Z-3be8f7d0`.
- Linux ARM64 and both macOS headless matrix: `20261101T141933Z-2db186f1`.
- Windows ARM64 retained-cache full CI: original work
  `20261101T145536Z-bf3c50d5`, separate `reports/warm-headless-01` receipt.
- Original Windows ARM64 failures: `20261101T140357Z-007d1788`,
  `20261101T141933Z-2db186f1`, `20261101T145536Z-bf3c50d5`.
- Range source-qualified receipts: `artifacts/range-qualified-*.json`;
  Windows ARM64 uses `range-qualified-win-arm64-warm.json`.

Verified evidence archives precede disposable build-cache removal. Reports, fixtures,
sources and retained publications remain separate from caches. macOS host and guest
space changes must not be added together.

## Completed GUI evidence

Managed matrix `20261101T153730Z-d043488b` and AOT matrix
`20261101T153734Z-6c02991e` both passed every target. These are 12 suites and 48
graphical application reports, all with zero final runtime charges. macOS x64 is
explicitly Rosetta. Each suite passed the inline range calculation, strict missing
weight, actual OS native clipboard and WB13 save/reopen checks, together with the
existing desktop acceptance coverage. Selected hashes of 28 implementation/test
files match the canonical checkout for every suite. Normal attended graphical-terminal
consent was retained; no GUI was launched through SSH.

The final machine-readable rollup is `artifacts/range-validation-summary.json`.
Completed-run caches on all targets were removed only after evidence archive verification;
current AOT publications and original passing/failing reports remain preserved.
