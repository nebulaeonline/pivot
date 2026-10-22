# ADR-0020: Worksheet bindings to structured numeric outputs

Status: **Proposed — design only; parent review required before implementation**.
Date: 2026-10-22.
Predecessors: [stable references](0003-stable-references-and-public-interfaces.md),
[structured identities](0019-structured-results-and-projection-identities.md),
[accepted proof and next review](../handoffs/2026-10-22-structured-results-projection-implementation-review.md).
[Review packet and acceptance cases](../handoffs/2026-10-22-structured-output-binding-design.md).

## Context and proposed scope

The accepted proof consumes a captured TotalAmount through a named numeric graph
invocation. It does not let an ordinary worksheet formula bind to that output.
Design that bridge without coupling computation to a projection, introducing
computational records or choosing a permanent `#` language. This ADR proposes
semantics and a bounded possible implementation slice; it authorizes no code changes.

Start with one explicitly declared Decimal96 output, TotalAmount, of the fixed
InvoiceLines producer. Ordinary arithmetic and dependents can consume the scalar.
Keyed record fields, tables, strings, nulls, floats, coercions, parameterized producers,
connectors and persistence remain separate decisions. OutputId is the stable public
contract, not a field position, aggregate expression string or record identity.

## Decision (proposed)

### 1. Portable bindings and name resolution

A bound output operand contains `(DocumentId, ProducerId, OutputId, ExpectedType)`.
ExpectedType is Decimal96 in the proposed slice; the representation must leave room
for future explicit types without routing decimal through binary floating point.
The output descriptor records its immutable numeric contract, including canonical
checked reduction for TotalAmount. Reusing OutputId for a new meaning or type is
forbidden. Breaking replacement creates a new identity, even if the label is reused.

The document owns a bounded producer/output directory, independent of projections.
Resolve a producer label and its output label only when an authoring command is
submitted. Return a typed operand plus source span/provenance to the formula binder.
Persisted/bound execution never looks names up again. For the first slice, names
are exact case-sensitive UTF-8, without case folding or normalization; require
uniqueness within document producers and within each producer's outputs. Restrict
candidate labels to existing single-line text admission and aggregate byte limits.
A later UI name policy can add convenience without changing identity semantics.

Use a transient client reference token plus a typed source-span binding supplied
with the formula-authoring command for a possible experiment. The exact command
encoding and token spelling must be reviewed before implementation. The server
validates document scope, span/token agreement, uniqueness and type; clients cannot
inject arbitrary NodeIds. `=#InvoiceLines.TotalAmount * 1.1` is explanatory notation,
not approved parser grammar. No permanent escaping, namespace or completion rules
are fixed here. A typed binder must not be simulated by rewriting the output into
a literal number or a hidden worksheet input.

A successful binding survives producer/output rename, projection rename, movement,
hiding, deletion, sorting and reflow. Formula source may retain old labels as
provenance, consistent with ADR-0003. Reopening an editor without changing source
preserves bindings. Explicit formula resubmission resolves names anew. Keep identity
information in inspection so old source labels do not masquerade as current lookup.

Deleting a producer/output leaves a dangling stable operand and a typed error.
Recreating its old name allocates new IDs and never repairs existing formulas.
Only explicit rebinding changes the target. Initial unresolved authoring creates
an isolated invalid draft under existing worksheet policy; later name creation does
not silently activate it. No automatic forward-reference resolution is introduced.

### 2. One canonical scalar graph, with a captured-input boundary

Propose a document-owned adapter slot per distinct referenced numeric output,
shared by all formulas referencing that output. The directory maps the portable
operand to a stable scalar graph node/slot. Formulas retain normal graph edges to
that node; downstream invalidation and evaluation use the canonical graph. There
is no second evaluator or dependency graph. Projected records still create no
ordinary Cells, Positions or per-record graph nodes. An explicitly bound aggregate
output slot is not an implicit materialization of its records.

The proposed minimal core extension is a **captured scalar source leaf**: a private
opaque slot whose value is supplied from an immutable evaluation context. The core
knows the slot and numeric/fault contract, not ProducerId, records, names, refresh,
projection descriptors or acquisition. Existing decimal operations remain unchanged.
The current core does not implement this leaf or its fault propagation; design
approval must include that extension. Do not present existing named decimal inputs
as if they already support typed source errors.

A calculation context supplies each required slot exactly once with either a checked
Decimal96 or a typed scalar-source fault. An absent/duplicate/mismatched slot is an
internal capture-validation error, never zero/default. These slots are not public
user-overridable invocation ports. The document coordinator alone supplies them.
If future external invocation exposes an output, its authority rules need review.

Source faults propagate through existing scalar/SUM dependency evaluation with the
origin slot and bounded document-side diagnostic metadata. Independent branches
remain calculable. Multiple faults follow the core's deterministic dependency/order
policy rather than whichever task completes first; the implementation must specify
and test that traversal policy before exposing stable error precedence. No error
string becomes a computational value, and no NonNumeric worksheet-text marker is
repurposed for structured faults.

A rejected alternative is updating a mutable ordinary input cell after publication:
it creates hidden worksheet state and can expose mixed generations. Another is
substituting literals into formulas: it loses identity/dependency information.
Installing aggregate values in the live graph at publication is possible but couples
publication to graph-root replacement; capture-local inputs keep the live formula
structure unchanged and reuse the proof's captured-adapter boundary. A future
performance-driven choice of that alternative requires a separate atomicity review.

### 3. Source heads, errors and publication

Distinguish immutable result payload generation from authoritative source-head state.
A head has a monotonically changing opaque version and is either Available with an
exact ResultGenerationId and descriptor revision, or Unavailable with an explicit
bounded reason. Producer/output directories and their retirement state are also
immutable roots. The first binding experiment may publish an unavailable state
explicitly; that is not an automatic acquisition/refresh policy.

| Condition in captured state | Formula outcome |
| --- | --- |
| Available valid empty result | Exact Decimal96 zero for TotalAmount |
| Available nonempty result | Canonical-order checked total |
| No result yet / explicitly published unavailable head | UnavailableResult |
| Producer removed | MissingProducer |
| Output removed | MissingOutput |
| Descriptor/schema incompatible | SchemaMismatch |
| Output type differs from bound ExpectedType | TypeMismatch |
| Decimal reduction or consumer arithmetic overflows | NumericOverflow at its origin |
| Invalid candidate or admission failure before publication | No new head; old capture/state remains authoritative |

MissingRecord remains reserved for future keyed binding; aggregate access must not
invent a record key. Descriptor admission normally prevents type/schema mismatch;
restoration or corrupted/stale typed operands must still fail explicitly.

A failed attempt to validate/admit a candidate is **not a published error generation**.
It leaves the prior successful head intact, with a separately visible attempt failure.
A formula evaluated from that retained head is deliberately reading the unchanged
publication, not silently recovering from a captured error. Diagnostics must show
that a replacement failed. If the owner intentionally publishes Unavailable instead,
new captures fail with UnavailableResult even when an older payload remains cached
for inspection. Cached/last-good data must never substitute for a captured unavailable
head, missing identity, type error or overflow. Old captures retain their old outcomes.
This extends the proof's rejected-publication behavior without redefining it.

Publish stages result/head roots and validates expected document/source state,
descriptor provenance and resource reservations. Installation is one coordinator
commit. Cancellation before commit installs nothing; cancellation after commit does
not undo it or justify automatic replay. The first source remains a fixed fixture;
parameter provenance, late network responses and multi-source transactions are not
implemented. Independent source heads can form a coherent capture without claiming
external acquisition occurred in one transaction.

### 4. Coherent calculation and invalidation

The document owner captures worksheet bindings and graph roots, producer/output
registry, source-head map and exact result payload roots in one serialized boundary.
It then computes required scalar adapters from those pins, builds an immutable slot
context, and validates completeness before scheduling detached graph evaluation.
Publication can proceed after pinning; adapter preparation never consults live heads.
Capture-time bounds failure aborts capture cleanly; an ordinary numeric source fault
is retained in its slot, allowing unrelated worksheet outputs to evaluate.

Example: S1 captures worksheet W1 and producer head H1/result G1. A cell edit then
creates W2 and publication creates H2/G2. S1 still evaluates W1/G1; S2 captures the
actual W2/H2 roots. A capture between those two independent commands can legitimately
see W2/H1. Coherence is one committed document instant, not a promise that separate
commands were one transaction. Any future combined edit/publish command must stage
and commit both roots atomically.

Keep WorksheetDefinitionRevision, SourceHeadVersion/result epoch and
PresentationRevision distinct. Publication does not rewrite formula definitions;
projection-only changes do not invalidate numeric computation. Use an exact composite
calculation identity including graph/binding roots, registry/head roots, slot context
and adapter contract revision. A cache keyed only by worksheet revision or CellId
is invalid. For the bounded slice, prefer fresh invocation-local evaluation per
capture over a new persistent incremental cache. A later cache optimization must
include source versions, including error/unavailable and deletion states.

The current private session's revision token is not sufficient authority for all
future binding operations by itself. Binding, registry mutation and publication
commands require an exact opaque composite expected-state token (or explicitly
validated worksheet plus registry/head components) checked by the owner. A name
resolved before a rename/publication must not be installed using a stale directory.
Do not choose a wire-version cut in this design; specify it in any implementation
plan after review. Unrelated layout changes must not alter numeric binding authority.

Each result frame carries capture identity plus head/generation provenance. A live
UI installs it only if it still matches the requested/current calculation state;
late S1 must not overwrite a newer S2 frame. An explicitly selected old capture may
still be inspected with its provenance visible. Ordinary repaint is never acquisition.

### 5. Copy, move, editing and document boundaries

Same-document formula copy retains ProducerId/OutputId/ExpectedType exactly. It has
no row/column-relative component and ignores `$`/placement. Ordinary A1 operands in
the same formula keep ADR-0017's existing translation rules. Move preserves binding;
copying a formula never clones a producer or publishes another generation.

Cross-document native typed copy must reject a foreign structured operand atomically
in the proposed slice, without partially applying the rectangle or falling back to
text. Do not resolve a same-name producer in the destination. Explicit plain-text
paste is new authoring and may resolve destination names using the destination
directory; the first experiment need not support text encoding of these operands.
Clipboard format/version and user-facing warnings are later implementation decisions,
not silent changes to PIVOTCB2. Reject unsupported representations explicitly.

Binding consumes one scalar output. SUM over ordinary cells includes bound numeric
cells by ordinary geometry; it does not absorb projected records. Removing all
projections does not affect a formula. Producer deletion does, with a stable missing
reference. No per-record computations, keyed syntax or dynamic worksheet insertion
are introduced.

### 6. Possible first slice and review stop

If approved, propose an opt-in non-savable experiment extending the accepted proof:
one producer, one referenced Decimal96 output, two views, bounded ordinary formulas
and downstream arithmetic/SUM. The normal parser/UI remains unchanged until a reviewed
typed authoring fixture/command can exercise the binder without approving permanent
syntax. Tests must use actual ordinary cell formulas, not another standalone named
invocation presented as worksheet integration.

Retain existing workbook/formula/node/admission ceilings and ADR-0019's one-producer,
two-view, 64-record, text/key/result/ownership/two-capture caps. Count directory entries,
shared output slots, typed operands, dependency edges, faults and capture contexts
against explicit owners and existing aggregate limits before allocation. At most one
adapter slot per referenced output in this slice, never per record or per consumer.
Tombstoned slots retained by live bindings/captures count toward admission; no ID reuse,
eviction of captures or silent unbounded tombstone growth. Set exact adapter/context
byte reservations in the implementation proposal; this design expands no capacity.

All documents containing these experimental bindings remain visibly non-savable;
reject Save/Save As and native clipboard formats that cannot faithfully represent
them. Keep ordinary WB4/PIVOTCB2 behavior unchanged. No WB5 or partial serialization.

Review must approve identity/error/head semantics, the proposed scalar-source leaf,
composite authority, admission details and the authoring/clipboard test boundary
before code. A later slice decides permanent authoring syntax and persistence.
SQL/connectors, scheduling, collection runtime, keyed field access, IEEE754 policies,
row-axis edits, anchor affinity, formatting, printing and capacity expansion stay out.

## Alternatives considered

- Look up producer/output names on every calculation: rejected because renames and
  reused labels can redirect an existing formula without an authoring action.
- Bind via projection coordinates or visible record ordinal: rejected because view
  reflow would become a computational change and multiple views would be ambiguous.
- Rewrite formulas to numeric literals or ordinary hidden input cells: rejected
  because stable dependencies and source provenance are lost or worksheet state is
  invented. The accepted proof's standalone invocation alone does not establish
  ordinary worksheet integration.
- Stage permanent scalar adapter values in the live graph at every publication:
  deferred in favor of captured inputs; this requires a stronger atomic root-install
  contract and is unnecessary before measuring the bounded experiment.
- Add general record-valued graph nodes: deferred; one explicitly typed output
  answers this milestone without introducing collection evaluation.

## Consequences

The core needs a narrow scalar source/fault boundary; the document owns all producer
knowledge and prepares its immutable input context. Existing capture/cache identity,
formula-authoring and native clipboard representations need deliberate integration;
this is not a parser-only feature. Names and layout stay editable while captured
numeric dependencies remain stable. The first experiment remains non-savable and
cannot imply a portable structured-document format or cross-document rebinding policy.

## Verification / reconsideration trigger

The linked review packet supplies the adversarial matrix; it is a proposed test plan,
not passing implementation evidence. Review descriptor evolution, keyed access,
parameter cycles, generalized source dependencies, source-error precedence, persistent
incremental caches and structural row editing before extending the scope. Future
producer parameters that depend on worksheet consumers require an explicit cycle and
scheduling model; this fixed-source design establishes neither. Reconsider captured
input preparation only with measured costs and an equally coherent atomic alternative.
