# Sparse worksheet storage proposal

Status: Proposed ADR-0010 structure, not implemented capacity or a serialized schema.

## Content and allocation

| Content at a position | Authoritative representation | Computational backing |
| --- | --- | --- |
| Empty, never referenced | Implicit absence | None |
| Formatting-only region | Style overlay/interval, usually no individual cell record | None |
| Text | Sparse UTF-8 cell content and optional style | None unless a future typed computation consumes it |
| Decimal constant | Exact coefficient/sign/scale plus binding metadata | Input node/source in the initial numeric projection |
| Formula | Original source, reference descriptors, bound state and stable binding | Computed node, even while its draft is invalid |
| Referenced empty position | Sparse reference anchor, distinct from a populated value | Lazy blank source; initial arithmetic proof may use an explicitly marked zero input |

No executable formula node is required for every empty or presentation-only cell.
Referenced blank anchors are demand-driven metadata, not a preallocated grid. Range
selection over a million blanks must not create a million anchors or zero inputs.
Text and blank semantics need the reviewed typed boundaries described in the
[formula analysis](spreadsheet-formulas-and-ranges.md); Decimal remains the only
implemented computational value type today.

## Ordered axes without a fully allocated grid

Give sheets stable row/column identities independently of visible ordinal positions.
Represent each axis as a persistent order-statistic tree of runs. A run has an
allocation identity and offset interval; splitting its order representation does
not change existing identities. Rank/select translates a displayed row/column to
an axis identity and back. Unmaterialized default stretches remain compact runs.

Inserting rows allocates a new run and updates order-tree paths. Existing row IDs
and cell keys do not change. Deleting retires axis intervals and affected anchors;
it does not cause surviving rows to inherit deleted identities. Proposed initial
sheet dimensions and bounds are configurable and finite; do not encode Excel's
dimensions or a fixed preallocated array as a core invariant.

Row/column heights, widths, hidden flags and style defaults use interval/run metadata
with shared defaults. New axes initially use workbook defaults; copying neighbor
styles is a distinct explicit policy. A row-height edit must not materialize all
cells in that row. Sheet rename changes its directory label, not WorksheetId.

## Candidate chunk/index organization

Index sparse chunks by stable row-run/column-run allocation identities and offset
buckets. A candidate bucket spans up to 64 by 64 positions within those runs;
64 is a tuning candidate, not a format promise. Row insertion splits axis ordering
but does not shift every later tile or change its stored coordinate key.

Keep low-occupancy chunks as compact sorted entries. Above a measured occupancy
threshold, use a presence bitmap plus packed content/binding arrays rather than
4,096 heap objects. Intern repeated styles and keep large text/source in shared
immutable buffers where useful. Reference anchors and tombstones need their own
sparse indices; they must not force a content record for every blank.

Rectangular reads decompose visual axis spans into stable run intervals, query
intersecting buckets and emit values in requested visual order. Cost depends on
run fragmentation, intersected chunks and returned records, not only cell count.
Many alternating inserts can produce many intervals; avoid claiming constant-time
rectangles or hiding a Cartesian run expansion. Benchmark the candidate index and
consider a separate spatial accelerator or measured run compaction if necessary.
Compaction may rewrite indexes but must preserve portable axis/cell identities.

Localized updates path-copy a persistent directory plus changed bounded chunks.
Unchanged chunk/source/style buffers are shared by working state and snapshots.
Bulk paste groups writes by chunk; do not copy one chunk separately for every cell.
Large payload ingestion has explicit byte/work limits and cancellation before commit.

## Memory and retention

Target allocation proportional to populated content, materialized reference anchors,
style/axis intervals and dependencies. Track these categories separately; a million
formatted empty positions should cost a few style ranges rather than a million
cell/node records. A million numeric constants still require real input storage;
sparsity cannot make populated data free.

Retired IDs prevent silent reference retargeting. Shared snapshots and undo roots
can retain old content. Measure live versus retained bytes, anchor/tombstone growth,
allocation count and reclamation latency. No silent identity reuse or unbounded undo
history is proposed. Compaction/undo retention require explicit policies later.

The first small proof can use simpler ordered maps behind these logical interfaces,
provided it labels their copying/query costs and does not claim the scale target.
The architecture must allow bounded path-copy chunks without changing cell/reference
semantics. Existing WorkingModel staging and snapshot rebuilding remain separate
scale bottlenecks even if the worksheet map is sparse.
