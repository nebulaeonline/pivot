# ADR-0010: Workbook projection and spreadsheet semantics

Status: Proposed
Date: 2026-10-19

## Context

ADR-0009 Slice B is accepted: a hosting-independent model survives experimental
file save and process restart. Stop expanding that file subsystem. Pivot now needs
the spreadsheet projection that makes its graph useful through familiar cells,
addresses and rectangular operations. No worksheet runtime exists today.

The core currently parses identifiers and decimal arithmetic, binds stable nodes,
and captures immutable snapshots. It has no cell/range grammar, range dependencies
or incremental recalculation. Current graph limits and full-copy staging are
correctness mechanisms, not spreadsheet-scale solutions.

The owner authorized this proposal only. No workbook, parser, evaluator, protocol,
GUI, storage format or runtime changes are authorized by this document.

## Decision

Propose a Rust workbook projection above the general computation core. Logical
document ownership includes a workbook and computational models; a workbook owns
sheets, sparse cell content, reference/projection metadata and presentation. A sheet
is not a graph. One graph may span sheets and contain nodes with no cells.

Keep worksheet/viewport code out of pivot-storage. The first approved workbook
implementation should establish its own module/crate boundary, provisionally
`pivot-workbook`, depending on core/value capabilities. Do not move existing
document/storage code speculatively or require this candidate crate now.

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

After architecture review, authorize a small sparse worksheet/numeric graph proof,
then the executable C# client and later bulk access/Avalonia integration. Do not
start implementation automatically. Keep ranges, parser extensions and incremental
evaluation behind their own reviewed steps within that direction.

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
