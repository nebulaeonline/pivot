# Sparse Worksheet and Graph Projection — Slice A review handoff

Date: October 19, 2026.
Disposition: Bounded in-memory worksheet proof implemented; stop for review.

## Architecture disposition

ADR-0010 is Accepted for its structural principles, with explicit semantic
reservations. Workbook projection stays above the core, outside pivot-storage;
cell/axis/node identities are distinct; edits install graph and sheet state
atomically; future ranges and viewport access retain their proposed boundaries.

Compatibility is not frozen for cut/move versus copy/fill, structural deletion,
range edge/movement rules, blank/text coercion, aggregate reduction order or shared
writable projections. This proof's explicit typed policies are test behavior,
not an Excel or permanent workbook compatibility claim. No production format is selected.

## Implemented boundary

The new pivot-workbook crate depends only on pivot-core, pivot-types and the already
locked UUID package. It owns one worksheet and one graph, sparse ordered maps,
independent axis namespaces/ordered runs, stable CellIds and explicit portable/core
node mappings. It adds no workbook dependency to core, storage, host or clients.

The minimal core extension is `model::transfer::set_resolved_expression`, within the
existing doc-hidden transfer module. It checks target/reference ownership and token
stack shape, extracts dependencies through the same validator used on import, and
installs an already-bound definition. The existing parser and evaluator are unchanged.
No significant parser/binder redesign, invented formula identifiers or second
expression engine was necessary.

Worksheet formulas use typed postfix Decimal/At/Cell/Unary/Binary descriptors.
At is resolved once on explicit submission into a stable Cell reference, lazily
allocating an anchored blank when needed. Source is retained as opaque provenance:
this does not make textual `=A1+B1` supported. Diagnostic core labels are never
substituted into source and reparsed.

## Concrete independent proof

| Step | Expected observation |
| --- | --- |
| A1 = 100, B1 = 25; typed C1 adds At(A1), At(B1) | C1 = 125 |
| Replace A1 with 200 | C1 = 225; A1 identity retained |
| Move A1 to empty D1 | C1 = 225; original CellId/node move to D1; graph revision unchanged |
| Put 999 at the vacated A1 | C1 remains 225; new A1 has another identity |
| Clear D1 | Original anchor retained with authoritative Blank; C1 = 25 under limited zero backing |
| Explicitly retire D1 target | C1 has a missing-reference diagnostic; global structural snapshot validation rejects calculation |
| Put 500 at D1 | New identity; C1 remains broken |
| Explicitly resubmit C1 with At(D1), At(B1), Add | C1 = 525 |

Retirement is a single cell-target operation without shifting rows/columns. An
anchored blank occupies its destination for Move. Empty, unmaterialized positions
allocate nothing. Numeric/formula replacement retains identity; ordinary and net
no-ops retain public state/revision. Blank metadata remains distinct from Decimal(0),
and exact submitted decimal parts are retained independently of runtime values.

Atomic batches stage private sheet state and WorkingModel together. Invalid or
foreign targets, occupied/duplicate move destinations, stale revisions, malformed
typed expressions and resource-limit errors install neither side of the candidate.
New identities are not exposed before commit. Every real batch advances workbook
state once; only the final core revision is visible even if staging touched several
definitions. Moves need not invalidate unchanged computational definitions.

Calculation captures own original position/identity mappings and core snapshots.
They can run after later edits/retirement or dropping the workbook, with original
revision provenance. Basic rectangular reads are sparse, complete-coverage,
revision-tagged and bounded by area/record limits; oversized reads reject rather
than pretending omitted cells are empty.

## Implementation and review files

- [Accepted ADR-0010 and reservations](../adr/0010-workbook-projection-and-spreadsheet-semantics.md)
- [Actual implementation, limits and costs](../architecture/workbook-projection-foundation.md)
- [Workbook crate boundary](../../crates/pivot-workbook/Cargo.toml)
- [Workbook implementation](../../crates/pivot-workbook/src/lib.rs)
- [Trusted core transfer seam](../../crates/pivot-core/src/model/transfer.rs)
- [Independent semantic/rollback tests](../../crates/pivot-workbook/tests/projection.rs)
- [Small cost measurement harness](../../crates/pivot-workbook/tests/costs.rs)
- [Updated project status](../STATUS.md)

## Verification evidence

Added 13 workbook semantic tests, one measured-cost test and one core trusted-binding
test: 15 additional tests over Slice B. The core test verifies malformed/foreign
bindings reject before mutation, opaque source is not parsed, retirement stays
diagnosable and the existing evaluator computes the result.

Workbook tests assert independent numeric/reference outcomes, sparsity, stable
identities, input/formula replacement, every supported arithmetic operator, scoped
blank behavior, malformed/stale/quota rollback, duplicate move rollback, typed cycles,
no-ops, complete bounded reads, exact decimal metadata, dependency separation and
immutable captures evaluated after later mutations/drop. They do not merely round-trip
production transformations to manufacture expected values.

Full Windows CI passes **200 Rust tests**, formatting, workspace Clippy with warnings
denied, .NET 10.0.402 Release, 10,099 decimal vectors, Rust/C# protocol/legacy
compatibility and actual daemon/CLI/REPL named-pipe integration:

- [Final Windows CI summary](../../artifacts/ci/20261019T090206790543Z/summary.json)
- [Windows Rust tests](../../artifacts/ci/20261019T090206790543Z/rust-tests.log)
- [Windows IPC results](../../artifacts/ci/20261019T090206790543Z/ipc/results.txt)

Available Ubuntu WSL2 formatting, workspace Clippy, **201 Rust tests** and real
Unix-socket IPC/REPL checks pass. The extra test is the existing Unix-only file
permission/alias test. The previous experimental file regressions keep their native
disk-backed /var/tmp test directory, independent of the new in-memory proof.

- [Linux Rust tests](../../artifacts/workbook-slice-a-linux-tests.log)
- [Linux IPC results](../../artifacts/workbook-slice-a-linux-ipc/results.txt)

Frozen hosted storage-v1 fixtures and the model-only document/file tests remain
unchanged and pass through workspace CI. No protocol, host, daemon, storage-codec,
file-broker or C# source changes were needed. Cargo.lock adds only the new workspace
package; no external package version changed. Core still depends only on pivot-types.
Native macOS validation of recent work remains pending. No new Linux .NET, Native
AOT, GUI or spreadsheet-scale result is claimed.

## Measured small workloads and remaining costs

The unoptimized test harness builds 16/128/512 numeric inputs plus one formula,
changes one input, captures, evaluates one output and reads the populated row.
It validates results and emits timings without performance thresholds:

- [Windows costs](../../artifacts/workbook-slice-a-windows-costs.log)
- [Linux costs](../../artifacts/workbook-slice-a-linux-costs.log)

Windows observed single-edit times were 29/160/600 microseconds and capture times
77/381/1,553 microseconds at 17/129/513 total live cells. Linux observations are
recorded separately and include visible startup/scheduling noise. These single
samples are neither distributions nor representative throughput promises.
Estimated sheet bytes were 5,570/41,410/164,290, excluding the core graph, snapshot
and exact allocator/map overhead; no allocator/RSS profiling was performed.

Known bottlenecks remain: full sheet/core staging copies, full dependency/order
rebuild during snapshot capture, copied capture position maps, per-edit quota scans,
and row-band rectangle scans that may visit unrelated columns. Defaults remain small
(1,024 live cells, 8,192 allocated anchors, bounded batches/source/tokens and a 1 MiB
logical sheet estimate). No sophisticated order-statistic tree, range index,
incremental evaluator or high-capacity storage claim is hidden in this proof.

## Exclusions and stop boundary

No textual A1 grammar, relative/absolute copy/fill, cross-sheet references, row/column
structural edits, names/ranges/SUM, selection evaluation, generic blank/text engine
types, shared writable aliases, incremental recalculation, workbook serialization,
production container, C# IPC, Avalonia UI or hosted persistence was added.

The following milestone should review the shared Rust reference parser/binder
extension and executable C# application boundary using these demonstrated behaviors.
General reference/range semantics remain separately reviewable. Stop after this
bounded proof; do not expand the experimental file subsystem or hosted storage.
