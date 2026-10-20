# Spreadsheet formula, reference and range analysis

Historical scalar Slice B design context. The accepted range policies and current
implementation are now governed by [ADR-0016](../adr/0016-first-class-ranges-and-sum.md)
and the [Slice C handoff](../handoffs/2026-10-20-ranges-and-sum-slice-c.md).
References below to future/unimplemented SUM describe the earlier design state;
SUM and WB2 are now implemented and validated on Windows, Linux and native macOS.


Current clipboard semantics are governed by [ADR-0017](../adr/0017-rectangular-clipboard-and-formula-copy.md)
and the [implementation handoff](../handoffs/2026-10-20-rectangular-clipboard.md).
Dollar-axis parsing, effective-binding native scalar/SUM copy, literal external TSV,
one bounded atomic paste and the WB3 format cut are now implemented; fresh Mac/manual
validation and parent implementation acceptance remain pending. WB1/WB2 readers are
not retained. Future/unimplemented statements below are historical design context.


Status: Semantic candidates under accepted ADR-0010 architecture, not compatibility
contracts. Same-sheet scalar A1 parsing/binding is implemented in
[Slice B](spreadsheet-scalar-formulas.md). Copy/fill, extended reference syntax,
ranges and aggregate functions remain unimplemented.

Review clarification: ADR-0010's structural architecture is Accepted. Reference
transforms, range boundaries/movement, blank/text coercion, reduction order and
shared writable aliases remain separately reviewable compatibility semantics.
[Slice A](workbook-projection-foundation.md) provides the bounded typed
single-sheet proof. Slice B adds scalar formula text; the broader mechanisms below
remain proposals unless explicitly covered by its implementation note.

## Shared Rust expression pipeline

Slice B introduces spreadsheet reference AST descriptors through an extension of the existing
Rust parsing/binding pipeline. Reuse operator precedence, decimal literal handling
and the evaluator. A resolver translates reference descriptors to stable graph/cell
targets. It must not rewrite A1 strings into artificial variable names and then
reparse them, or evaluate formulas in C#.

Standalone model parsing keeps its current identifier semantics. Spreadsheet mode
recognizes scalar A1 patterns. `$` markers, sheet qualifiers and range operators
remain future proposals.
Names resembling valid A1 addresses are reserved in workbook expression scope to
avoid ambiguity; they remain valid identifiers where the standalone model grammar
already permits them. `parse_spreadsheet` is the current scalar-only API. Address column
letters normalize to ASCII uppercase. Sheet/name labels retain case-preserving,
ordinal case-sensitive lookup for the initial proof, consistent with current named
models; a broader case-insensitive compatibility policy needs explicit review.

| Form | Proposed meaning |
| --- | --- |
| `A1` | Target resolved on the origin sheet, row/column relative for copy |
| `$A$1` | Same stable target, both axes absolute for copy |
| `$A1`, `A$1` | One absolute and one relative copy axis |
| `Sheet2!A1` | Resolve stable WorksheetId by label on submission; keep explicit sheet qualification |
| `'Quarter 1'!B2` | Proposed quoted sheet label; doubled quote escaping requires lexical tests |
| `A1:B100` | Bound rectangular selection with structural boundary policy |
| Named expression | Resolve a stable named binding, sheet-local before workbook-local; never rebind on label reuse |

A named expression initially denotes a scalar node or a named range, with an
explicit kind. A named formula can be represented by a hidden computed node.
Deleting/recreating a name is not renaming its stable binding. Cross-model calls,
dynamic references, volatile functions, locale-dependent syntax and external
automatic fetching remain separate proposals.

## Copy versus movement

Each bound scalar reference retains its target identity plus absolute/relative flags
for each axis and whether the sheet was explicit. The target is authoritative during
ordinary evaluation. For copy/fill, compute the target's current positional offset
from the source formula cell, transform relative axes and resolve a new destination
target. Do not let stored stale offsets override a moved target's identity.

Example: copying a formula from C3 to D5 (one column right, two rows down):

| Source reference | Copied reference |
| --- | --- |
| `A1` | `B3` |
| `$A1` | `$A3` |
| `A$1` | `B$1` |
| `$A$1` | `$A$1` |

An unqualified reference resolves relative to the destination sheet when copied
across sheets. An explicitly qualified reference keeps its WorksheetId while its
relative axes transform. Out-of-bounds copies produce an explicit invalid reference,
not clipping or wraparound. Copying an already invalid draft preserves a diagnosable
draft; it does not resolve formerly failed names simply because they now exist.

Moving the formula cell itself keeps its existing bound targets. Copy/fill is a
different operation. Moving the referenced target makes inbound references follow
its stable identity, regardless of `$` flags. Render source addresses from these
bindings; do not recalculate meaning by reparsing old strings after a move.

## Range geometry and structural policy

Propose a stable RangeId with sheet identity, ordered axis boundary descriptors and
versioned membership. It denotes a spatial region rather than a permanently expanded
list of cells. Endpoints use stable axis boundaries with explicit edge affinity:

- Insert strictly inside the selected interval: expand membership to include new axes.
- Insert immediately before the first or after the last selected axis: exclude the
  new axes; existing selected axes retain their identities and positions may shift.
- Delete selected axes: contract to surviving selected axes. If none survive on an
  axis, preserve an invalid/deleted selection rather than silently selecting neighbors.
- Insert outside: retain membership; rendered coordinates may change.

Relative/absolute flags still govern copying each endpoint. They do not disable
structural updates. Whole-row/whole-column selection can later use bounded axis
domains rather than allocating every member; initial syntax need not support it.

For cell content moves inside/outside a range, the proposed default is **spatial
membership**: the region remains anchored to its axes, so values moved out leave
and values moved in join. Scalar references instead follow the moved CellId. This
is a deliberate semantic difference, not a claim of Excel parity. Review it before
implementing cut/paste ranges; a future explicit selection-relocation command could
transform the region. Partial moves must not silently create undocumented unions.

## First-class computational selections

A formula binds a generic selection identity, not one scalar token per coordinate.
Workbook projection owns geometric resolution and an immutable sparse membership
view. The core consumes a reviewed engine-neutral selection abstraction: typed values,
producer identities, membership generation and deterministic traversal. No callback
into mutable GUI/workbook state or filesystem is allowed during a calculation.

An aggregate such as proposed `SUM(A1:A1000000)` has one bound selection descriptor.
Its evaluation may still inspect many populated values. The million-cell syntax
does not imply a million transmitted tokens, dependencies to blank positions or
preallocated zero nodes. Creating function-call/range AST and typed selection
support is future core work; neither SUM nor ranges work in the current parser.

## Proposed value/error policy

| Input kind | Decimal arithmetic reference | Proposed decimal SUM over a range |
| --- | --- | --- |
| Decimal | Existing checked numeric semantics | Include in deterministic visual row-major order |
| Blank | Coerce to zero for arithmetic | Ignore |
| Text, including empty text | Typed error, no locale-based numeric parsing | Ignore text entries |
| Invalid reference or failed computation | Preserve typed error/diagnostic | Propagate first encountered error in traversal order |

Blank is distinct from zero and empty text in owned workbook state. General typed
blank/text/error sources require future core value-boundary changes; a temporary
zero backing supports only the explicit arithmetic subset. Never silently claim
text computation support by converting text to zero.

Decimal addition can have order-sensitive intermediate overflow/rounding behavior.
Do not use cached chunk totals, parallel reassociation or a wider accumulator to
change the defined reduction implicitly. Any optimized aggregate must match the
selected semantics or obtain a separate numeric policy decision. Traversal and
error ordering are part of compatibility, not incidental hash-map iteration order.
