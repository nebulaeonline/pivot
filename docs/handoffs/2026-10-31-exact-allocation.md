# ADR-0036 — exact, conservation-preserving allocation implementation

2026-10-31. Accepted and closed by parent implementation review.

The parent accepts the four-argument contract and all three mandatory clarifications,
the shared Blank-anchor invariant correction, unchanged representations/budgets,
six-target evidence, selected-file source qualification, Rosetta coverage, observed
core/charged-memory qualifications, diagnostic LoC and verified archival/cleanup.
No further validation, reconciliation packet or graphical/Native AOT rerun is required.
Git commit/push and subsequent work remain owner-controlled.

`ALLOCATE(total,recipient_count,recipient_index,decimal_places)` now provides the
approved scalar equal allocation. Four strict Decimal arguments are required;
count/index are one-based u32 integers and precision is an integer from 0 through
28. Blank, Boolean and String operands are TypeMismatch. Invalid specifications or
a total not integral in settlement units are InvalidArgument; group representation
failure is Overflow, exposed through the existing session NumericError category.
Local argument validation retains left-to-right dependency demand, with cross-argument
admission after local validation. Inactive lazy branches remain unevaluated.

The implementation uses six fixed u32 limbs, bounded work independent of recipient
count, and no runtime BigInteger, float division, recipient enumeration or group state.
First residual-many indices receive one additional unit, negative totals are symmetric,
and calculated zero is positive and normalized. Display formatting is independent.
See [semantics](../architecture/allocation.md) and the
[approved ADR](../adr/0036-exact-conservation-preserving-allocation.md).

## Three mandatory clarifications

**Production SUM.** Used shares are normalized, and the largest proper partial sum
must fit their common scale. The full total may normalize further. Actual Pivot SUM
passed for 576 complete groups in three recipient orders (1,728 checks), including
both signs, zero, tiny values, M at counts 1 and 3 with precision 28, and 10/8 split
three ways at precision 28. M denotes 2^96−1. These checks exercise the real evaluator,
not a replacement summation or oracle-only conservation proof. SUM itself is unchanged.

**Group-wide refusal.** Every valid index has the same admission outcome. Both shares
are validated when the remainder is nonzero; an unused Q+1 is never validated when
it is zero. M/count 2/precision 1 and M/count 10/precision 1 refuse. The latter also
exercises proper-partial admission. For (M−9)/count 11/precision 2, a fitting low share
cannot succeed when the actually used high share cannot fit. Wrong arity, strict
operands, fractional/out-of-range metadata, exact unit validation, dependency fault
precedence, trailing-zero integral metadata and lazy branches have dedicated coverage.

**Explicit completeness boundary.** Conservation requires each index exactly once
with consistent total/count/precision. Independent scalar calls do not certify that
condition. Ordinary/visible copy and Fill, sorting and structure preserve explicit
recipient indices without renumbering or repair. Cross-sheet references and retained
captures keep normal binding and snapshot semantics. No registration, spill, weighted
allocation, rotation, reporting, history or automatic repair is introduced.

## Shared Blank-anchor correction

Save/reopen after row insertion exposed an existing canonical inconsistency: moved
Blank anchors were inserted into calculation-range membership, although Blank records
must remain outside it. Export then correctly refused with InvalidTarget. The issue
also reproduced with the pre-existing `ROUND(E200,2)` function.

Structural placement now preserves Blank identities and positions while omitting them
from calculation membership. The document regression exercises row and column insertion
and deletion with ROUND, ALLOCATE's strict Blank precision, and an allocation SUM;
all twelve save/reopen cases pass. The real SDK session also passes structure followed
by Save/New/Open. This changes no UI, serialization metadata, wire shape or AOT dispatch.
Original failure logs and the corrected regressions are retained separately.

## Formats and integration

Typed opcode 19 fits existing codecs. WB12, private session v13, hosted v1 and CB5/CB6
remain unchanged. Existing unknown-function/opcode refusal remains the compatibility
boundary; this does not make older readers understand ALLOCATE. Ordinary division,
ROUND and decimal addition are unchanged, and no resource or request limit increased.

Real C#→Rust session coverage includes allocation values/errors, lazy evaluation,
production SUM, retained snapshots, ordinary and visible native clipboard paths,
ordinary and visible Fill, sorting, formatting independence, structural bindings,
WB12 persistence, error filtering and zero final runtime charge.

## Independent arithmetic evidence

Python Fraction/subset enumeration and .NET BigInteger/multiplicity construction are
independent of production fixed-limb arithmetic. All six targets passed 51,523 vectors,
comparing exact coefficient, sign, scale and error classification with zero mismatches.
The shared input SHA-256 is
`f3ca6580942b34636718fb6514f62a8e949be574ea00afe982367e700e9f5950`.
Small groups enumerate bounded proper subsets; large counts use algebraic complements.
A separate 192-bit bitwise division test checks full-width quotient/remainder arithmetic.

Supplementary ROBPC coverage checks 1,116 vectors at indices 1, R, R+1 and N where
valid, for counts 65,536 and 4,294,967,295. Rust and both independent oracles agree.
Its input SHA-256 is `05b04d4b9b140945d3b821c58b493932bee9e139556b1b9a2227d74ba36047e4`;
raw inputs/outputs are retained in `artifacts/allocation-residual-boundaries`.
This supplementary coverage is local; the six-target corpus remains 51,523 vectors.

## Validation and resource evidence

Final local CI passed 13/13, all-feature Rust tests passed 504 with zero failures or
ignored tests, and strict workspace/all-target/all-feature Clippy passed. The local
CI receipt is `artifacts/allocation-local-ci/20261031T233555162106Z/summary.json`.

All six headless targets passed 13/13. Windows/Linux x64 use matrix
`20261031T234017Z-3873cc70`; Windows/Linux ARM64 and both Mac architectures use
`20261031T233641Z-8e8ce1e3`. macOS x64 is Rosetta, not native x64 hardware evidence.
Selected receipts are collected in `artifacts/allocation-selected-matrix.json` and
`artifacts/allocation-qualified-<rid>.json`.

The resource corpus has 16,384 base cells, 4,096 allocation formulas, one input anchor
and one SUM output: 20,482 positioned cells. Each of five modes takes four samples
with four captures retained, using changed inputs and setup outside timed work.
Modes cover cents, tiny precision-28 totals, wide settlement units, maximum count
and group-wide Overflow. Helper work is measured separately. Values are evaluated
in bounded 256-output batches; accepted sample SUMs use the production evaluator.
The maximum-count fixture samples 4,096 indices, including all its nonzero shares;
it does not materialize or certify a maximum-sized worksheet group.

| Target | Headless CI | Maximum evaluation ms | Maximum steady MiB | Maximum peak MiB | Final charge |
| --- | --- | ---: | ---: | ---: | ---: |
| win-x64 | Passed 13/13 | 33.67 | 43.69 | 59.65 | 0 |
| linux-x64 | Passed 13/13 | 34.61 | 43.69 | 59.51 | 0 |
| win-arm64 | Passed 13/13 | 28.17 | 43.69 | 59.65 | 0 |
| linux-arm64 | Passed 13/13 | 22.79 | 43.69 | 59.65 | 0 |
| osx-arm64 | Passed 13/13 | 14.40 | 43.69 | 59.58 | 0 |
| osx-x64 (Rosetta) | Passed 13/13 | 24.50 | 43.69 | 59.65 | 0 |

These are observed maxima across four samples per mode, not percentiles, GUI latency
or process RSS. Memory is charged integration-runtime memory. The original 1 s bounded
engine objective and 256 MiB steady/512 MiB peak objectives are unchanged and passed.
Explicit arithmetic scratch/control accounting stays below 256 bytes and 1,024 limb
visits per call; compiler frames and existing evaluator argument storage are separate.

The approved numeric-only scope requires no new attended GUI/Native AOT round. No UI,
DTO, wire metadata or AOT-sensitive path changed. The shared structural correction was
qualified through core/document/real-session tests, including both axes/actions.
Earlier GUI/AOT receipts remain historical evidence, not new ADR-0036 executions.

Native runs use frozen dirty-source snapshots based on commit `6606ba0`. All eight
selected numeric, structural, resource and oracle/session source files match the final
checkout on every target; hashes are retained with each qualified receipt. Documentation
was completed after deployment. This is a selected-file qualification, not an assertion
that every historical source snapshot or report is byte-identical to current docs.

## Diagnostic code size

Measured once on the canonical checkout with cloc 2.06: **89,459 code lines across
391 source files**, +859 lines and +8 files versus ADR-0035. Test/example code is a
subset, not an additional total. Generated output, dependencies and caches are excluded.
Raw inputs, digest, tool output and per-language counts are in
`artifacts/loc/ADR-0036/20261031T233804215663Z`; documentation counts describe measurement
time, before this final handoff. This is diagnostic evidence, not a reduction target.

| Project | Code | Comments | Blank | Test/example code* | Languages (code) | Change |
| --- | ---: | ---: | ---: | ---: | --- | ---: |
| ApplicationSession | 1,335 | 15 | 32 | 1,335 | C# 1,179, Python 156 | +36 |
| DecimalParity | 277 | 10 | 22 | 277 | C# 59, Python 218 | +95 |
| LegacyProtocol | 161 | 5 | 2 | 161 | C# 43, Protocol Buffers 118 | +0 |
| Pivot.Avalonia | 419 | 3 | 2 | 0 | C# 419 | +0 |
| Pivot.Client | 521 | 25 | 17 | 0 | C# 521 | +0 |
| Pivot.Desktop | 2,889 | 43 | 41 | 1,090 | C# 2,761, XML 128 | +0 |
| PolicyProbe | 1,619 | 54 | 127 | 1,619 | C# 1, PowerShell 38, Python 1,580 | +0 |
| ProtocolParity | 81 | 2 | 8 | 81 | C# 55, Python 26 | +0 |
| pivot-cli | 1,413 | 12 | 3 | 280 | Rust 1,413 | +0 |
| pivot-core | 16,342 | 214 | 167 | 3,014 | Rust 16,342 | +51 |
| pivot-host | 8,431 | 69 | 95 | 4,165 | Rust 8,431 | +0 |
| pivot-protocol | 2,636 | 47 | 29 | 1,023 | Protocol Buffers 565, Rust 2,071 | +0 |
| pivot-session | 3,549 | 17 | 11 | 382 | Rust 3,549 | +0 |
| pivot-storage | 5,134 | 52 | 72 | 1,952 | Rust 5,134 | +0 |
| pivot-transport | 897 | 26 | 13 | 369 | Rust 897 | +0 |
| pivot-types | 773 | 20 | 11 | 194 | Rust 773 | +270 |
| pivot-workbook | 25,569 | 179 | 121 | 11,865 | Rust 25,569 | +215 |
| pivot-workbook-document | 7,093 | 50 | 28 | 4,452 | Rust 7,093 | +192 |
| pivotd | 1,134 | 20 | 7 | 899 | Rust 1,134 | +0 |
| Other source | 214 | 11 | 4 | 0 | Protocol Buffers 214 | +0 |
| Scripts/tooling | 7,673 | 373 | 671 | 0 | Bourne Shell 126, C 166, DOS Batch 12, PowerShell 438, Python 6,931 | +0 |
| Standalone tests | 1,299 | 87 | 191 | 1,299 | Python 1,299 | +0 |


## Retention and closeout

Reports, original development failures, source snapshots, fixtures, oracle vectors
and cleanup receipts are retained. Archives verify member sizes and SHA-256 before
pruning. Four-VM cleanup removed 37.18 GiB of logical build output; both macOS
architectures removed 10.42 GiB. These logical-byte totals are distinct from observed
free-space changes; host and guest free space are not added together. Current/pinned
publications and the unrelated unverified Linux x64 legacy run remain preserved.

The local Cargo cache contained 15.51 GiB. Cargo initially refused its missing cache
marker; after exact path/child/reparse/inactive-compiler checks, the standard marker
was restored and supported `cargo clean` succeeded. The original refusal is retained.
Source, reports, .NET developer output and native publications remain available.

Archive indexes and pruning receipts are in `artifacts/archives/index.jsonl` and
`artifacts/pruned-build-products.json`. Local evidence is
`artifacts/archives/allocation-final-local-evidence-20261031.zip`; final source/receipts
are bundled as `artifacts/archives/allocation-final-source-and-receipts-20261031.zip`.
The incomplete sandbox-denied archive is retained separately, not indexed as verified.

ADR-0036 is accepted and closed. Stop for owner commit/push and explicit next-work
authorization. The parent recommends ADR-0037, Weighted, Conservation-Preserving
Allocation, as the next design candidate only; neither design nor implementation is
authorized yet. A future design must resolve explicit weight sources and recipient
association, exact proportional remainder ranking and deterministic ties, zero/negative
weights, group-wide representability, actual SUM, reference stability and bounded work.
Sorting or range membership changes cannot be assumed to preserve intended weight
associations. ADR-0023 stays parked at G3.

## Accompanying discussion

The owner emphasizes that the function is usable and understandable to a lay spreadsheet
user. The parent agrees: each argument answers a practical question—how much, how many
ways, which share and how many decimal places. The numerical sophistication belongs
inside the engine; the formula should express the user's settlement intent clearly.
The result is an actual reconciled value for calculations, copy and export, independent
of display formatting. Keep explicit precision and the complete/consistent index-set
requirement clear; do not imply that ALLOCATE rounds an otherwise invalid total or
certifies independently authored group membership.
