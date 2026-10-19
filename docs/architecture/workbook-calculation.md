# Range dependencies and incremental calculation implications

Status: Proposed ADR-0010 direction. Current evaluation is synchronous/full-snapshot;
no incremental engine, typed ranges or worksheet calculations are implemented.

## Dependencies beyond scalar edges

Keep ordinary scalar forward/reverse edges by stable node identity. Add a separate
selection dependency index: formula consumers reference RangeId, while the workbook
spatial index identifies regions containing changed positions. Membership generations
track structural changes. A constant edit inside a selected region can invalidate
consumers even if that position was empty when the formula was bound.

The proposed logical graph is `consumer -> selection -> producers`. Selection is
not an executable node for every coordinate. Store compact geometry/membership
descriptors and indexed producer queries; do not eagerly materialize the Cartesian
grid. The core sees generic selection/producers from immutable captured roots,
while worksheet geometry remains projection-owned.

An edit queries overlapping range consumers plus direct scalar dependents. A move
invalidates membership at both old and new positions. A row insertion can change
selection membership without editing any populated cell. Deleting a range endpoint
must update the membership/index generation and error state coherently with the axes.

## Cycle analysis

Selections participate in cycles. A formula in A5 aggregating A1:A10 includes its
own producer and is a cycle; it must not silently exclude itself. A formula reached
through a selection can depend back on the consumer indirectly. Dependency ordering
and SCC/cycle diagnostics must query selected formula producers, not only explicit
scalar tokens. Constant/blank members cannot create computation cycles themselves.

Sparse producer queries avoid enumerating unused coordinates, but true range edges
may still be dense. Worst-case overlapping selections can expose quadratic real
dependency relationships. Measure generated-edge work, index size and validation
cost. Apply explicit work budgets rather than claiming spatial indexing gives
constant-time cycle checks. Compressed dependency structures can be explored later
without changing the logical meaning of membership or cycles.

## Incremental results with immutable provenance

Maintain dirty candidates from edited inputs/definitions, removed targets, changed
membership and semantic settings, then propagate through scalar and selection
dependents. Structural invalidation and current value equality are separate: a
membership change may affect later edits even if the immediate sum is equal.

A future result cache is keyed by immutable computational state, semantics/settings
and scenario inputs. Reuse is permitted only when dependencies and membership
provenance establish validity. Formatting-only changes should reuse numeric results.
Temporary invocation overrides remain isolated and must not overwrite base-workbook
cache entries or stored inputs. No global mutable calculation state is proposed.

Run calculation away from rendering/edit installation. Cancellation may stop obsolete
work, but even an uncancelled S result cannot replace current T results untagged.
UI data may expose explicit stale or pending status alongside the generation.
Deterministic arithmetic/error behavior must match a clean full-evaluation oracle.

Range reductions may scan populated entries in deterministic row-major order, using
implicit blank counts where appropriate. That is O(populated members), not O(1).
Aggregate summaries require a separate proof for checked-decimal order/overflow
semantics before reuse or parallel reassociation. Prefix indexes or cached sums
must not silently change numeric behavior.

## Current gaps and staged proof

Today Snapshot construction rebuilds dependencies/order, WorkingModel staging copies
definitions/name/interface structures, and evaluation builds calculation-local state.
Structural parse/bind/cycle failures can reject the whole model snapshot, although
arithmetic failures already have local result propagation. A spreadsheet eventually
needs unaffected regions to calculate beside structural cell errors; do not claim
that capability from the current evaluator. It needs reviewed typed diagnostic/error
sources and partial-admission rules, not a second workbook evaluator.

The first sparse worksheet proof should reuse current decimal evaluation and its
global structural validation honestly. Later steps introduce generic selection
binding, typed blank/text/error values, dependency-index evolution and incremental
calculation, each checked against existing standalone-model and numeric tests.
Graph budgets and transfer hard ceilings must be reviewed together; increasing an
outer document cap does not remove them or avoid full-copy costs.
