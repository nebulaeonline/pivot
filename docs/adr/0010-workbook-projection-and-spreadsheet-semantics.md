# ADR-0010: Workbook projection and spreadsheet semantics

Status: Accepted
Date: 2026-10-19

## Acceptance boundary

Accepted after the parent architecture review and owner authorization of the bounded
Sparse Worksheet and Graph Projection Slice A. Structural principles are accepted:
separate workbook/core/storage boundaries, independent identities, sparse content,
atomic graph/projection edits, immutable calculations, future bounded viewport access
and first-class selections as the range direction.

Spreadsheet compatibility semantics are **not frozen** for cut/move versus copy/fill,
structural deletion versus clear/replacement, range boundaries/movement, blank/text
coercion and aggregation, numeric reduction order, or multiple writable projections
of a shared node. The detailed policies below are candidates for separate review.
No Excel compatibility is claimed. Slice A exercises a narrow explicit typed
move/clear/retire policy as a proof, not a permanent user-facing contract.

The implemented proof lives in pivot-workbook; see
[implementation and costs](../architecture/workbook-projection-foundation.md).

## Context

ADR-0009 Slice B is accepted: a hosting-independent model survives experimental
file save and process restart. Stop expanding that file subsystem. Pivot now needs
the spreadsheet projection that makes its graph useful through familiar cells,
addresses and rectangular operations. At proposal time no worksheet runtime existed.

The core currently parses identifiers and decimal arithmetic, binds stable nodes,
and captures immutable snapshots. It has no cell/range grammar, range dependencies
or incremental recalculation. Current graph limits and full-copy staging are
correctness mechanisms, not spreadsheet-scale solutions.

The owner first authorized the design and then the bounded Rust-only proof after
review. General parser/range semantics, protocol, GUI, workbook serialization and
scale/incremental work remain outside that authorization.

## Decision

Propose a Rust workbook projection above the general computation core. Logical
document ownership includes a workbook and computational models; a workbook owns
sheets, sparse cell content, reference/projection metadata and presentation. A sheet
is not a graph. One graph may span sheets and contain nodes with no cells.

Keep worksheet/viewport code out of pivot-storage. The first approved workbook
implementation now establishes `pivot-workbook`, depending only on core/value
capabilities and UUID identity allocation. Existing document/storage code remains
in place; no speculative document split is required.

Use stable worksheet, cell and graph identities independent of displayed positions.
Separate content edits, clearing content, movement and structural deletion. An
ordinary edit preserves a cell binding; a moved cell retains it; a deleted target
remains invalid rather than rebinding to later content at the same address.

Propose sparse adaptive chunks indexed by stable axis runs, with independently
shared axis-order, formatting and geometry structures. Empty and presentation-only
positions need no executable node. Numeric constants and formulas participate in
the computational graph; referenced blank anchors may need a lazily created scalar
input, but a large blank range must not allocate a source per position.

Extend Rust's existing expression pipeline through reviewed reference AST/resolver
boundaries, rather than adding a C# evaluator or repeatedly parsing historical A1
strings. Bind scalar references to stable targets while retaining relative/absolute
copy flags, original source and editable invalid states. Scope and structural edit
rules belong to the projection; arithmetic and computational dependency semantics
belong to the core.

Treat ranges as first-class bound selections with sparse membership and indexed
invalidation. The core consumes immutable generic selection capabilities, not
worksheet objects or GUI callbacks. Range aggregation/cycles require core evolution;
they are not implemented by spelling a million scalar references in a formula.

Coordinate worksheet, reference and graph edits through one atomic aggregate-root
installation with expected-state checks. Calculations capture immutable graph and
selection generations while later edits continue. Formatting-only changes do not
advance computational revision. Avoid whole-workbook copies in the target design;
measure the existing core full-copy boundary explicitly in the initial small proof.

Use bounded rectangular viewport reads, separate geometry/presentation/value data,
and coherent revision/result tags. Selection and pending edits are client overlays;
authoritative computations remain in Rust. No wire fields or Avalonia rendering
are designed into the computation core.

## Alternatives considered

- A graph per sheet: prevents natural shared computations and equates presentation
  with graph ownership.
- One node per possible coordinate: allocates unused grid space and formatting.
- A1 text as identity, reparsed after every edit: silently retargets historical bindings.
- Only relative offsets as bindings: cannot distinguish deletion from coordinate reuse.
- Expand ranges into scalar tokens: makes dependency storage and transport scale
  with rectangle area, including empty space.
- Put workbook/viewport behavior in pivot-storage or C# view models: obscures ownership
  or duplicates calculation authority.
- Promise incremental performance by raising the 1,024-node limit: ignores copying,
  indexing, invalidation and result-generation costs.

## Consequences

Stable reference anchors, ordered sparse axes and coordinated edits add projection
state beyond a simple coordinate dictionary. Range membership introduces dependency
and cycle-analysis work that must be represented honestly. Dense genuine dependency
graphs can still require substantial work; indexing does not make every update cheap.

This proposal chooses observable semantics for review, not production bytes, protocol
messages or permanent data-structure constants. In particular, clear/delete/move,
spatial range movement, blank/text coercion and structural boundary rules require
review before syntax or compatibility fixtures freeze them. Excel compatibility is
not claimed.

## Verification / reconsideration trigger

Slice A implements the approved small sparse worksheet/numeric graph proof.
Slice B now adds the reviewed scalar same-sheet Rust formula parser/binder; see
[its implementation note](../architecture/spreadsheet-scalar-formulas.md). Stop after
its handoff for review. The parent review accepts Slice B and makes performance
feasibility the critical path before C# integration. See the
[bounded research assessment](../architecture/graph-workbook-scaling-feasibility.md):
shared input/topology roots are an experiment, not an adopted production storage
policy. The parent accepts assessment B; the bounded
[structural mutation experiment](../architecture/structural-mutation-feasibility.md)
now tests local formula edits and retained topology generations. Review its
[handoff](../handoffs/2026-10-19-structural-mutation-feasibility.md) before production
replacement or further feature implementation.
The parent accepts the structural research. The subsequent
[compatibility foundation](../architecture/persistent-core-compatibility-foundation.md)
adds persistent indexes, general-model adapters and initial memory admission while
leaving the default backend unchanged. Review its
[handoff](../handoffs/2026-10-19-compatibility-foundation.md) before a production switch.
Keep ranges and extended reference syntax behind their own reviewed steps.

Require independent-reference semantic oracles, transaction/snapshot isolation tests,
and measured 10K/100K/1M populated-cell workloads before claiming capacity. Revisit
chunk organization under axis fragmentation, range cycle costs, reference tombstone
growth and value-type/aggregation semantics as workloads expose constraints.

Supporting proposals:

- [Ownership](../architecture/workbook-ownership.md)
- [Sparse storage](../architecture/sparse-worksheet-storage.md)
- [Graph projection](../architecture/workbook-graph-projection.md)
- [Formula and range semantics](../architecture/spreadsheet-formulas-and-ranges.md)
- [Atomic edits](../architecture/workbook-edit-transactions.md)
- [Calculation and invalidation](../architecture/workbook-calculation.md)
- [Viewport access](../architecture/workbook-viewport-access.md)
- [Performance and verification](../architecture/workbook-performance-verification.md)
- [Review handoff](../handoffs/2026-10-19-workbook-projection-design.md)

Latest implementation review packet: [Worksheet Slice A](../handoffs/2026-10-19-workbook-projection-slice-a.md).

Latest formula implementation packet: [Slice B](../handoffs/2026-10-19-spreadsheet-formulas-slice-b.md).

Latest core checkpoint: [Controlled Production Core Integration — Stage 1](../architecture/controlled-core-integration.md).
This bounded standalone integration does not authorize a workbook backend switch.


Stage 2 standalone integration now covers ports, retirement, bounded inspection,
complete snapshots and portable transfer through WorkingModel; see the
[contract](../architecture/controlled-core-integration-stage2.md). This does not
migrate workbook projection transactions or hosted service publication. Those
remain separate compatibility reviews before backend selection in either layer.
