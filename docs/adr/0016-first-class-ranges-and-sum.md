# ADR-0016: First-class rectangular selections and SUM

Status: Accepted — parent design review received 2026-10-20. Not yet implemented.
Governing policy for same-sheet rectangular ranges; ADR-0010 history is preserved.

## Review boundary

The parent accepts Slice B/ADR-0015 and assigns Spreadsheet Semantics Slice C.
Its first requirement is to define and approve range semantics before committing
to a permanent representation. ADR-0010 also leaves range affinity, blank coercion
and reduction order open. The parent review now explicitly approves this decision and authorizes bounded
Slice C implementation. These range rules are accepted independently of the earlier
scalar-reference policy. See the [review requirements](../handoffs/2026-10-20-ranges-and-sum-review.md).

## Accepted observable semantics

A range is an inclusive rectangle in one worksheet's existing row and column axis
namespaces. It selects the **current occupants of those coordinates in the captured
generation**, not a frozen collection of CellIds and not two moving endpoint cells.
Bound descriptors retain axis identities and canonical bounds. Cell movement does
not move those bounds. Row/column structural transforms remain outside Slice C.

| Case | Accepted behavior |
|---|---|
| Numeric input | Include its numeric value once |
| Computed member | Include its Rust-calculated scalar numeric value once |
| Never-populated coordinate | Contributes zero; allocate no cell, node or anchor |
| Demand-created blank anchor | Contributes zero; no range membership edge to the blank |
| Cleared cell | Contributes zero; its existing scalar identity/zero backing remains available |
| Move within rectangle | Include at its new position; reduction order may change |
| Move out of rectangle | Exclude it, even though scalar references still follow its identity |
| Move into rectangle | Include the incoming cell |
| Retire member | Remove its membership; the empty coordinate contributes zero |
| Reuse retired/vacated coordinate | Include its new occupant; scalar references to the retired cell remain invalid |
| Move an endpoint's occupant | Bounds stay fixed; endpoints are coordinates, not cell anchors |
| Move the SUM formula cell | Preserve its bound rectangle, without rebinding source; membership may now include itself |
| Formula cell lies inside its own rectangle | Dependency cycle, including an otherwise empty rectangle |
| Invalid/failed computed member | Propagate an error, not zero or partial success |
| Invalid draft outside selected closure | Does not prevent selected SUM evaluation |
| Future unsupported value type in a member | Explicit type error; no text-to-number coercion policy is adopted |

This deliberately differs from scalar identity affinity. For `=A1+SUM(A1:A3)`,
moving the original A1 to A5 keeps the scalar term bound to that cell but removes
it from SUM. The formula field continues showing the original source. No Excel
cut/copy/fill or structural insertion compatibility is claimed.

### Grammar and diagnostics

Add only `SUM(<same-sheet rectangle>)` and scalar arithmetic around its result.
For example, `=SUM(B2:B100)`, `=SUM(B2:B5)*2+A1` and
`=SUM(B2:B5)+SUM(D2:D5)`. Function spelling is ASCII case-insensitive. Existing
whitespace rules apply, including around the colon. Each SUM takes exactly one
rectangle; no scalar arguments, argument lists, named ranges or cross-sheet names.
A naked range such as `=B2:B5`, `SUM(B2)`, unknown functions and unsupported argument
forms remain editable invalid drafts. Existing general-model name parsing remains
separate from spreadsheet mode; this is not a broad function-language extension.

Accept reversed endpoints and normalize both dimensions, so `SUM(C5:B2)` selects
B2:C5. Retain submitted endpoint order, source text, UTF-8 spans and original
positions separately from normalized bound geometry. Validate both endpoints and
bounds before materializing anything. Out-of-bounds, malformed and incomplete
syntax remains an editable draft with an actionable diagnostic/span. Bounds do not
need populated endpoints and must not demand-create endpoint anchors.

### Exact reduction and error order

SUM starts at exact decimal zero and applies the existing checked decimal addition
in row-major order: ascending captured row, then column. Skip absent/blank positions
without iterating the area. Empty SUM returns canonical zero. Member input metadata
retains coefficient/sign/scale; the computed result uses normal canonical arithmetic.

Overflow is reported at the first overflowing addition, even if a later negative
member could bring an unbounded mathematical sum back into range. No wider hidden
accumulator, reassociation, rounding mode change or parallel unordered reduction is
introduced. This is reproducible with the existing executor rather than promising
unbounded-integer cancellation. Moving members can change overflow behavior because
it can change row-major order; that is part of the explicit policy.

Each rectangle includes each occupied position once. Overlap between separate SUM
calls is intentional: `SUM(A1:A3)+SUM(A2:A4)` counts A2/A3 in both calls. Source
arithmetic continues using existing operator precedence. Invalid closure/cycle
validation precedes arithmetic. For a valid acyclic closure, a SUM reports the first
failing member encountered in row-major reduction order; it does not silently skip
errors. Resource admission/cancellation failures remain operation failures and
must not be cached as formula values.

## Accepted canonical representation direction

Extend the existing core, not a second evaluator. Introduce a generic selection
space, stable SelectionId and a bound rectangle descriptor. The workbook translates
axis identities/bounds and sparse computational placements into that space. The
core receives owned persistent data, never a live worksheet callback or A1 resolver.
Only rectangular numeric SUM is implemented; this is not permission to create a
general collection/query framework.

The aggregate formula stores a selection reference token. It does not store a list
of scalar tokens or durable edges for every member. Selection vertices are distinct
from user computational NodeIds; no hidden executable input is created per blank.
Selection lifetime/reference accounting and retired IDs are explicit. Descriptor
sharing is allowed when space, bounds and policy are identical; it must not merge
source spans or a formula's own provenance. Repeated terms still have their stated
arithmetic multiplicity. Public type/byte layouts remain subject to implementation
review after this semantic decision.

Maintain one persistent sparse placement relation per space, shared by all its
selections, containing numeric/formula occupants mapped to existing stable graph
identities. Blank anchors remain in the scalar graph but need no point membership.
A two-level ordered index over populated rows and their populated columns supports
rectangle enumeration without stepping through absent rows/columns. Cost depends
on visited populated rows/index paths and emitted members, not rectangle area.
Do not claim pure output-only complexity when rows contain points outside the
requested columns. A node-to-placement reverse relation supports propagation through
computed members; the current workbook has one placement per node.

Maintain an index of bound rectangles for point invalidation and an index from
SelectionId to formulas using it. A row-interval index with column filtering is a
bounded initial choice; report row candidates and visited index nodes, including
worst-case overlap. It does not promise constant-time edits or output-optimal 2D
lookup. Store neither a full sheet copy nor a separate member list for each range.
Persistent storage is proportional to occupied points, selection descriptors,
selection consumers and scalar edges, plus retained generations. Genuine overlapping
selection traversal can still require substantial transient work.

### Dependency closure and cycles

The canonical dependency walker traverses formula → selection → current member
nodes alongside existing scalar edges. Enumerate selection members lazily from the
same immutable root. Do not populate durable scalar dependency/reverse lists as an
alternative implementation of ranges. Temporary closure/topology work may visit
all actual members and is admitted/bounded; it is not a dense coordinate expansion.
Global validation uses this same dependency relation. Selected calculation traverses
only its requested closure, preserving unrelated-invalid-draft isolation.

Value changes propagate through existing scalar dependents and the selections
covering each affected computed member's placement, then onward through selection
consumers and their dependents. Looking up rectangles only at the directly edited
input is insufficient: an input outside a rectangle may change a computed member
inside it. Point removal/movement must consult old and new placements so departed
members invalidate their former aggregates. Cycles can cross scalar and selection
edges or involve multiple aggregates. Dependency inspection must distinguish these
edge kinds and expose them honestly rather than claim a complete scalar-only view.

### Atomic state, generations and admission

Candidate transactions update graph definitions, sparse placements, selection
bindings, rectangle/consumer indexes and worksheet projection together. Sequential
batch binding uses the staged candidate; final validation sees the complete candidate.
Cycles remain editable invalid computational states under the existing policy;
hard failure/cancellation installs no transaction prefix. No successful edit may
expose graph S with placement/index T. No-op behavior and exact expected-state
checks remain intact.

A calculation capture pins graph values/topology, selection descriptors and the
placement root together. Membership changes count as dependency-topology changes;
value-only changes do not require rebuilding rectangle geometry. Captured S keeps
its original membership and values after edits/moves/retirement in T. Any memoized
selection result is invocation/generation-local and never written into a different
generation or serialized as a cache.

Keep existing rows/columns, populated/allocated cells, source/tokens, user edit batch,
viewport/frame, runtime queue and allocation-domain limits. A rectangle may cover
more area than a viewport read: the read-area cap is not an area-expansion license
or a reason to reject a sparse range within the worksheet's configured bounds.
A 1024×1024 rectangle is a required sparse test under current dimensions, not a
million-populated-cell capacity claim. Add explicit admitted budgets for selection
identities/descriptors, point/index deltas, visited membership/closure work and
transient reduction state before exposing the feature. Exact quota constants must
be documented with measured evidence; do not borrow unlimited capacity from blank
area or increase ordinary node limits. Cancellation checks are required during
candidate/index work, enumeration, topology and evaluation.

## Native documents and client boundary

Use a deliberate native format cut to `PIVOTWB2` with selection-aware payload
versioning. Retain bounded length/integrity framing, exact decimals, graph/cell and
retired identity ledgers, dirty/checkpoint semantics and staged file installation.
There are no deployed workbook documents requiring migration: explicitly reject
WB1 in the new workbook reader rather than introduce an unreviewed compatibility
path. Keep old-format evidence historical; mark the version cut in the release
handoff. The parent explicitly approves this format break. WB1 must return a clear
unsupported-version error; retain a WB1 rejection fixture, not a migration reader.

Persist stable selection IDs, space/axis identity, normalized bounds, policy version,
source/provenance and bound selection tokens. Encode sparse placements once, or
reconstruct them from validated sparse cell records and bindings. Never serialize
per-selection expanded member edges or internal indexes/caches. Opening cross-checks
the restored relation and canonical graph atomically. Syntax/span consistency may
be validated; historical A1 text must not be used to resolve saved selections again.

The new selection-aware portable graph encoding belongs in the existing storage/
document boundary. Frozen hosted storage/protocol v1 remains unchanged. Selection
capabilities must be explicitly versioned and rejected at scalar-only export
boundaries; never omit selections or flatten them into scalar dependencies to fit
an old encoding. No hosted range publication or public protocol capability is added
by this desktop slice. The private application boundary can continue transporting
original formula text and Rust-calculated values/errors, with no C# evaluator.

## Alternatives and consequences

Frozen CellId membership misses later entries and retains cells moved out of the
rectangle. Endpoint-cell affinity would let moving two occupants reshape a range
and requires additional movement/deletion policy. Both are valid possible products,
but neither is the accepted coordinate-area contract. Scalar expansion, full-sheet
materialization and workbook-side arithmetic fail the accepted sparse/core boundary.

Coordinate-area affinity is easy to explain for data entered later and sparse
regions, but intentionally differs from scalar move semantics. Row/column structural
operations, text values, coercion, cross-sheet references and copy/fill require later
decisions. The selection capability and policy version leave that extension explicit.
The core dependency/cycle/evaluation and portable transfer paths must all understand
selection edges; adding only a parser branch would be incomplete.

## Acceptance evidence required for implementation

The historical [design handoff](../handoffs/2026-10-20-ranges-and-sum-design.md) provides worked
cases, implementation order and evidence gates. Full Windows/isolated Linux/native
macOS regression, real cross-process/native-document and actual-window tests remain
required. Before the implementation handoff, manually exercise Open/Save As and each
Save/Discard/Cancel dirty-close outcome on at least the primary desktop. Do not call
automated method invocation a manual native-dialog acceptance pass.

Preserve the unresolved WSLg startup abort and mounted-Windows-path replacement
failure; if the native abort recurs, capture an actionable backtrace. No SQLite,
new engine, broad function library, structural transforms, copy/fill, formatting
expansion, installer/AOT or production-million-cell claim is authorized.

## Implementation validation

The [Slice C implementation note](../handoffs/2026-10-20-ranges-and-sum-slice-c.md)
records concrete quotas, typed inspection, WB2 layout, measurement scope and
remaining platform acceptance. Design acceptance does not imply implementation acceptance.
