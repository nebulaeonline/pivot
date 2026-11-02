# Unbound Grid Window qualification

2026-11-01. Accepted by ADR-0040 parent implementation review with the unresolved
Windows ARM64 cold-run qualification retained. No additional matrix is required.
All six targets passed managed and Native AOT GUI suites. Windows ARM64 headless
has the explicit cold-run qualification below.

The additional scope corpus runs at 1,024, 10,000 and 65,536 total authored cells,
with 64 live Grid Windows, one worksheet aggregate formula, structural row deletion,
whole-Window retirement/recreation, retained captures and WB14 save/reopen. Fixture
assembly uses admitted projection and portable restore; it does not implement a
cross-scope move API. Existing worksheet resource corpora also remain in CI.

The table records the 65,536-cell checkpoint. Values are observed helper timings,
not latency percentiles or GUI responsiveness measurements. Memory is charged-domain
accounting, not process RSS or disk usage.

| Target | Structural ms | Window deletion ms | Save ms | Reopen ms | Steady charged bytes | Peak charged bytes |
| --- | ---: | ---: | ---: | ---: | ---: | ---: |
| Windows x64 | 803.269 | 803.104 | 129.982 | 336.642 | 141,920,235 | 330,685,061 |
| Linux x64 | 612.586 | 670.369 | 109.077 | 329.133 | 141,900,366 | 340,110,279 |
| Windows ARM64, retained build caches | 430.684 | 442.815 | 89.490 | 213.629 | 141,913,065 | 330,667,782 |
| Linux ARM64 | 374.198 | 403.185 | 82.349 | 209.009 | 141,939,478 | 330,694,195 |
| macOS ARM64 | 280.848 | 309.217 | 66.334 | 171.866 | 141,935,257 | 330,689,974 |
| macOS x64, Rosetta | 729.288 | 677.086 | 88.066 | 239.619 | 141,921,797 | 330,676,514 |

The declared Grid Window checkpoints meet the unchanged one-second command,
ten-second persistence, 256-MiB steady and 512-MiB peak objectives. Every resource
child ends with zero runtime charges. No admission ceiling was increased.

## Cold-run qualification and development failures

The original Windows ARM64 headless run `20261101T221321Z-420606ca` remains Failed.
The existing 65,536-cell worksheet first-row insertion measured 1,089.4862 ms wall
and 1,089.136458 ms in Rust. A separate full CI run using retained build caches
passed all 13 gates with identical selected source. This does not establish the
cause of the cold failure or demonstrate cold-run timing reliability.

During development, local Grid Window deletion samples of about 1.189 and 1.083
seconds failed the same command objective. Majority retirement now rebuilds the
small surviving position/name indexes and shares immutable empty retired payloads.
Retired identity authority and older roots remain retained and charged. Original
failed logs remain evidence; limits were not relaxed.

## Evidence

Selected implementation/test/build hashes cover 92 files and match the canonical
checkout on all six targets. Documentation and private reporting helpers are outside
that selected-source claim. Native work directories and RID receipts qualify each
execution; macOS x64 is Rosetta execution, not an Intel Mac.

- ROBPC CI: `artifacts/grid-ci-final/20261101T224537253786Z/summary.json`, 13/13.
- ARM-host original matrix: `20261101T221321Z-420606ca`; Windows ARM64 retained-cache
  CI: `20261101T224655524594Z`, separately preserved under `warm-headless-01`.
- x64 headless matrix: `20261101T225439Z-23d2d2b2`, both targets Passed.
- Managed GUI execution: `20261101T225253Z-de3a43d3-attended`.
- AOT GUI execution: `20261101T225437Z-c13302aa-attended`.
- Local source-qualified rollups: `artifacts/grid-qualified-*.json` and
  `artifacts/grid-gui-qualified-*.json`.

GUI qualification totals 12 suites and 48 zero-charge application reports. Its
Window flow covers create/edit, target dimensions and navigation, range allocation,
OS clipboard refusal, rename, cross-scope references, WB14 reopen and exact deletion
review/faults. Headless Avalonia checks are additional evidence, not GUI substitutes.
