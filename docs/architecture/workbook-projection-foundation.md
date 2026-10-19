# Sparse Worksheet and Graph Projection — Slice A

Historical Slice A boundary: [Slice B](spreadsheet-scalar-formulas.md) now adds
text parsing, stable binding metadata and editable formula diagnostics. Descriptions
of missing A1 parsing below apply to the original Slice A proof.

Implemented bounded proof under accepted ADR-0010, 2026-10-19. Architecture is
accepted; general spreadsheet compatibility semantics remain reserved for review.

## Boundary and trusted binding

[pivot-workbook](../../crates/pivot-workbook/Cargo.toml) depends only on pivot-core,
pivot-types and the existing UUID package. It does not depend on storage, the host,
transport, Protobuf, C# or a GUI. Existing model-only documents remain independent.

The sole core extension is a doc-hidden
[resolved-expression setter](../../crates/pivot-core/src/model/transfer.rs) in the
existing trusted transfer boundary. It accepts typed decimal/node/unary/binary
tokens, validates node ownership/allocation and stack arity, then installs a bound
definition with retained opaque source. Its dependency/stack validator is shared
with the existing importer. It never parses names or A1 strings, and it uses the
unchanged core snapshot/evaluator. No parser or general binder redesign was needed.

Generated core labels such as N0 are diagnostic labels, not fabricated variables
inserted into formula text. Worksheet formulas submit typed postfix descriptors;
the source string is provenance only and is not claimed to compile as spreadsheet
syntax. The later real parser must produce reviewed reference descriptors rather
than treating this trusted proof API as a textual parser.

## State and identities

Workbook owns one WorksheetId and graph identity, an immutable single run per axis,
a sparse position directory, cell/tombstone records, explicit CellId-to-portable-node
and private runtime-handle mappings, and a core WorkingModel. Axes have independent
namespaces and lazily described ordered identity sequences; no row/column arrays or
full grid are allocated. Row/column reordering/insertion is not implemented.

Positions are one-based integer pairs, not cell identities. New anchors get distinct
CellId and GraphNodeId UUIDs; actual core NodeIds stay private. A move preserves both
bindings while changing position. Tombstones preserve retired IDs and prevent
coordinate reuse from repairing old references. There is one writable cell per
backing node; shared writable aliases/model-port projections are not implemented.

DecimalInput retains submitted coefficient/sign/scale separately from runtime
projection, including signed zero. Content is Blank, Decimal or typed Formula.
Only explicitly materialized/referenced positions get anchors and input/computed
nodes. Never-referenced empty positions use no per-cell storage or executable nodes.

## Tested proof policies

| Operation | Bounded Slice A behavior |
| --- | --- |
| Set decimal/formula | Preserve an existing cell/node identity; replace its definition |
| Typed `At(position)` reference | Resolve once, creating a blank anchor if necessary; store stable `Cell` reference thereafter |
| Typed `Cell(reference)` | Validate worksheet and allocated identity; a known retired target remains a broken draft |
| Clear | Keep the anchor; authoritative Content::Blank with zero-valued decimal input backing |
| Move to unoccupied destination | Carry the anchor and node; preserve computational bindings and graph revision |
| Move into occupied/anchored blank destination | Reject the whole batch |
| Retire | Remove the live core definition, retain identity ledger and diagnose inbound references |
| Reuse retired/vacated coordinate | Allocate a new anchor/node; old references remain broken |
| Resubmit typed formula | Explicitly resolve new positional descriptors, permitting deliberate repair |

This is not a finalized cut/copy/fill or row/column structural deletion contract.
Retire is an explicit cell-target operation without shifting cells. Blank-as-zero
supports only this decimal arithmetic proof: Blank remains distinct from Decimal(0)
in worksheet metadata. No generic typed blank/text value, range aggregation or
permanent coercion policy is added to the engine.

## Atomic edits, captures and reads

Every edit batch requires matching session, workbook state and core graph revision.
Stage private sheet/maps and WorkingModel together, apply bounded commands, validate
logical quotas and install once. Failures expose neither intermediate state nor
provisional identities. One real batch advances workbook state once; internal core
revision may advance for multiple definitions, but only its final value is visible.
Ordinary and net no-ops discard the candidate without advancing public state.

Moves can advance projection/workbook state without changing core revision. No
formatting subsystem is included. A calculation capture owns the immutable core
snapshot plus captured position/identity maps and revision tags. It can calculate
after later edits, retirement or even dropping the workbook. Results carry a fresh
generation and original provenance. They do not mutate the workbook.

Invalid reference/cycle drafts remain editable and diagnosable. Current core
structural validation still blocks the whole snapshot; this proof does not make
unaffected cells calculate beside structural errors. Arithmetic error propagation
continues to use the existing core's isolated result behavior.

`read` returns sparse cell content/identities with complete rectangle coverage and
expected-state tags. Area and record caps reject oversized results rather than
returning ambiguous partial coverage. This is a bounded in-process read, not the
future paged viewport cache, value/geometry transport or protocol contract.

## Limits and measured costs

Defaults: 1,024 live anchors/nodes, 8,192 allocated identities, 256 edits per batch,
4,096 tokens/source bytes per formula, a 1 MiB logical estimated-content budget,
4,096-position read area and 1,024 returned records. Logical accounting is not an
allocator or RSS upper-bound measurement. Retired identities consume quota. Bounds
are proof policy; they are not a permanent spreadsheet capacity model.

One unoptimized Windows test run on the development machine produced these
microsecond timings (inputs plus one formula, one sample each):

| Live cells | Build | Single edit | Snapshot capture | Selected evaluation | Rectangular read | Estimated sheet bytes |
| --- | --- | --- | --- | --- | --- | --- |
| 17 | 329 | 29 | 77 | 21 | 13 | 5,570 |
| 129 | 1,819 | 160 | 381 | 24 | 97 | 41,410 |
| 513 | 15,699 | 600 | 1,553 | 59 | 462 | 164,290 |

These are diagnostic observations, not latency gates, representative distributions
or scale claims. Estimates exclude the core graph, snapshot allocations and exact
map/allocator overhead. Linux measurements are retained in the handoff artifacts;
startup/scheduling noise is visible even at the smallest size. Allocator profiling
and 10K/100K/1M workloads have not been run.

Known costs: staging clones both sheet maps and core definitions; core snapshots
rebuild dependencies/order; captures clone position mappings; quota scans visit
current records after each staged operation; rectangular map reads can scan unrelated
columns in the selected row band. These deliberately simple bounded implementations
need measured path-copy/chunk/index work before large workloads. No sophisticated
axis tree, range index or incremental calculation was introduced to hide those costs.

See the [review handoff](../handoffs/2026-10-19-workbook-projection-slice-a.md) for tests,
CI and exclusions. Stop here before general parser/range or application integration.


## Persistent integration follow-up (2026-10-19)

[ADR-0012](../adr/0012-bulk-mutation-and-persistent-workbook.md) and the
[bounded integration](../architecture/persistent-workbook-bulk.md) add an opt-in
persistent Workbook path while retaining this accepted identity, draft and
transaction behavior. Production Workbook remains reference pending separate
review. Exact shared behavior suites and a differential transaction oracle cover
the new path; this historical foundation record is not a backend-switch decision.
