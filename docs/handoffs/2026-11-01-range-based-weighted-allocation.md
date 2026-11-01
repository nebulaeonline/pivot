# ADR-0038 — range-based weighted allocation implementation

Accepted and closed by implementation review, 2026-11-01, with the Windows ARM64
cold-build qualification retained. The four design clarifications, unchanged ADR-0037
arithmetic, separate adapter admission, WB13-only compatibility cut, source-qualified
six-target evidence, diagnostic code-size report and verified cleanup are accepted.

The three Windows ARM64 cold-build timing failures remain Failed and unexplained.
The retained-build-cache CI pass is separate successful evidence, not proof of cold-build
timing reliability. macOS x64 is Rosetta coverage; source hashes cover the documented
selected files. Timings are observed maxima and memory is charged-domain accounting,
not GUI latency, percentiles or process RSS. No additional validation is required for
this acceptance. ADR-0023 remains parked at G3; excluded capabilities remain outside scope.

`ALLOCATEWEIGHTEDRANGE(total,index,places,weight_range)` now binds an explicit spatial
range operand and delegates numeric settlement to the unchanged operation-20 helper.
Literal row/column rectangles have 1..64 physical positions; reversed endpoints use
ascending coordinates and singletons require colon syntax. Missing, cleared, calculated
Blank, String and Boolean weights are TypeMismatch. Zero remains explicit. Numeric
metadata and weights are validated in demand order, preserving lazy control flow.

## Implementation and representation

The dense accessor preserves holes and repeated node identities. Actual descriptor,
row and column AVL visits consume the invocation budget. It runs on the evaluator's
existing task stack and immutable capture with the same cancellation and allocation
context. An independent 8-KiB adapter reservation precedes collection; the existing
16-KiB numeric reservation and arithmetic/visit/limb limits remain unchanged.
Calculated Blank is checked before ordinary formula-result coercion to zero.

Invalid geometry retains descriptor subscriptions without member traversal or data
edges. Restoring admissible geometry reconstructs edges and independently checks the
whole candidate for cycles, including when unrelated invalid drafts exist. A restored
cycle rejects atomically. Identical live spatial rectangles may share a descriptor;
retired identities are never reused and numeric groups are never memoized. This keeps
the declared 256-call corpus within existing selection limits.

Operation 21 requires three scalar roots and a distinct range operand or valid range
fault; four scalars and leaked range operands are rejected. Graph token 8 and workbook
token 9 carry the new reference. WB13 is the only current native format; WB12 is refused
without migration. Private session v13, hosted v1 and CB5/CB6 retain their fields.
Source, effective endpoints and fault provenance preserve authority through copying,
Fill, full deletion and reopen. Experimental history remains outside this change.

## Validation evidence

ROBPC headless CI passed all 13 gates. All-feature workspace tests passed 525 and strict
Clippy passed. Local regressions cover dense aliases/holes, scalar/range authority,
strict types and calculated Blank, metadata error order, real SUM conservation,
geometry restoration with cycles, insertion/deletion, immutable captures, CB5/CB6 and
Fill, live/fault WB13 round trips and WB12 refusal. Real session checks distinguish
positional sorting from scalar identity references: sorting weights (3,1) changes range
index 1 from 7.5 to 2.5 while an old capture remains 7.5.

Independent Python Fraction and .NET BigInteger oracles match 1,993 worksheet-range
vectors; the existing 12,841 scalar weighted vectors also remain passing. Resource
corpora use 16,384 base cells, 64 weight positions, 256 calls, four changed totals and
four retained captures. Reported timings are observed engine/helper samples and memory
is charged-domain accounting, not GUI latency or process RSS. Original one-second,
256-MiB steady and 512-MiB peak objectives are unchanged.

All six targets have passing complete headless CI evidence, with Windows ARM64
explicitly qualified as a retained-build-cache run. Its three earlier cold-build
structural timing failures remain unchanged and unexplained. See the
[performance report](../performance/range-weighted-allocation.md) for the observed
maxima, original failures, isolated checks and warm-CI distinction. All six targets
passed managed and Native AOT GUI checks, including macOS x64 under Rosetta: 12 suites, 48 graphical application reports and zero final runtime charges.
Selected source hashes match for 23 headless and 28 GUI implementation/test files.
Final collected GUI matrices are `20261101T153730Z-d043488b` (managed) and
`20261101T153734Z-6c02991e` (AOT). Short target-local `rng.py` launchers retained normal
graphical-terminal consent.

The initial 256-call construction exposed redundant descriptor admission and was fixed
through live geometry reuse without raising limits. Earlier failing checks and retries
retain separate logs. Each completed native batch was archived and every evidence
member's SHA-256 and size verified before disposable caches were removed. Final verification preserves
original results, sources, reports, fixtures and current AOT publications. ROBPC
Cargo cleanup removed 28,182 files, 20.6 GiB on disk. Retention ledgers and verified
archives remain under `artifacts/archives`; host and guest free-space changes are
reported separately. No runtime-memory claim is derived from disk reclamation.

Diagnostic cloc 2.06 measurement: 92,351 code lines across 406 source files, +1,492 from
ADR-0037. Per-project breakdown and counting qualifications are retained in
[the measurement](../../artifacts/loc/ADR-0038/20261101T143646793437Z/report.md).

See [ADR-0038](../adr/0038-range-based-weighted-allocation.md) and
[range semantics](../architecture/range-weighted-allocation.md).
