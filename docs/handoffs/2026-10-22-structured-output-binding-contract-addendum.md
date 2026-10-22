# ADR-0020 focused implementation-contract addendum

Date: 2026-10-22. **Proposed; parent approval required. No implementation authorized.**
[ADR-0020](../adr/0020-worksheet-structured-output-bindings.md),
[review decision](2026-10-22-structured-output-binding-design-review.md).
These concrete proposals refine, rather than implement, the approved architecture.
The choices below, especially private spelling, copy restriction and quotas, need review.

## 1. Narrow core leaf and immutable context

Propose closed `CapturedScalar { slot: CapturedScalarId, expected: Decimal96 }`
variants in canonical working/snapshot definitions. CapturedScalarId is an opaque
non-reused UUID scoped to the owning graph. The graph node's existing stable NodeId
is distinct. The document keeps `(DocumentId, ProducerId, OutputId, Decimal96)` to
`(GraphId, NodeId, CapturedScalarId)` in its owned adapter table. No producer identity,
record, acquisition callback, trait/plugin hierarchy or error text enters the core.

Only a crate-private workbook integration path allocates/registers the leaf and
creates its bound Node operands. Ordinary input APIs, public named ports, frozen
hosted codecs and scalar exports cannot construct, override or export it. Attempts
reject explicitly; no fallback to Input(0) or NonNumeric. Rust visibility is an
integration authority boundary, not a security claim against hostile in-process code.
A client annotation contains portable output IDs only, never NodeId or slot ID.

The coordinator pins document roots and graph snapshot first, then prepares a
capture-owned context. Its logical type is an immutable map from CapturedScalarId to
`Decimal96(PivotDecimal)` or `Fault(CapturedFaultId)`. The builder accepts a bounded
sequence so duplicate entries are detected before constructing the map. Validate:

- Exact owning GraphId and graph-capture identity; reject a context for another graph
  or snapshot, even if the caller knows an otherwise valid slot ID.
- Exactly one entry for every captured scalar leaf in that snapshot (not only the
  requested output closure). Missing, duplicate and extra/unknown IDs reject capture.
- Entry type matches Decimal96; validate coefficient/sign/scale at boundary and keep
  exact representation without round-trip text or float conversion.
- Every FaultId has exactly one immutable document diagnostic in this capture, and
  its slot agrees with the context entry; no dangling diagnostics or fault substitution.

Expose only a validated context to detached evaluation. The evaluator borrows/owns
that immutable context beside its graph snapshot; no process-global map or mutable
last-value input exists. No ordinary evaluation overload silently supplies an empty
context to a graph containing these leaves: it rejects MissingCapturedContext. A graph
with no source leaves uses the existing path unchanged; a nonempty source context
for such a graph rejects. Named invocation may still override declared ordinary
inputs, but rejects attempts to target a captured leaf.

At leaf evaluation the numeric arm returns its exact decimal. The fault arm returns
`CapturedSourceFailed { node, fault_id }`. CapturedFaultId is an opaque capture-scoped
UUID, allocated once per failing slot per capture and retained as it propagates.
Extend dependency failure metadata to preserve this optional fault identity alongside
the existing origin NodeId, including through SUM; do not replace it at each consumer.
The document diagnostic table maps it to MissingProducer, MissingOutput,
UnavailableResult, SchemaMismatch, TypeMismatch or NumericOverflow and bounded
provenance. Core errors do not enumerate those source-specific categories.
A consumer's own arithmetic overflow remains the existing arithmetic error, distinct
from an overflow while preparing TotalAmount that arrived as a source fault.

Fix precedence to **first failure encountered by the existing postfix expression
executor**, which visits source operands in parsed expression order and stops on
first error. Preserve existing SUM ordinary row-major member order and first-failure
behavior; preserve operator sequencing and existing errors. Do not select errors by
worker completion, hash order or UUID order. For example `badSource + (1/0)` reports
the source fault, `(1/0) + badSource` the arithmetic fault. Repeated references to one
failing leaf retain the same FaultId. Independent output branches still evaluate.
Test this in canonical and feature-enabled evaluation paths. No general parallel
error aggregation or new source hierarchy is introduced.

## 2. Slot lifecycle and bounded retirement

Allocate a slot only on the first successfully committed reference to that output;
multiple occurrences/consumers share it. Rejected authoring allocates no published
slot. The slot is stable while any live bound formula or retained graph capture owns
it. Rename/reflow/publication never replaces it. Removing a producer/output retains
its binding identity and leaf; new contexts supply a missing-identity fault. Old
captures keep their pinned successful context and descriptor roots.

For this experiment propose **one resident source slot total**, including any retired
slot pinned by live bindings or captures. There remains only one active producer and
one active declared output. Deleting/recreating a producer may allocate new portable
IDs, but a new output slot cannot coexist with a pinned retired slot. Rebinding that
would require a second slot rejects with a specific admission diagnostic; it never
retargets the old slot. Once the old slot has no consumers or captures, a staged edit
may reclaim it and allocate a fresh identity atomically. Never reuse the retired ID.
This deliberately bounds lifecycle tests; a later capacity proposal can allow more
slots. Tests must cover both successful single-consumer rebind after release and
refusal while another old consumer/capture retains the slot.

Maintain at most 64 retained registry/tombstone entries combined. A retired identity
needed by a live operand or capture stays charged. Unreferenced tombstones can be
reclaimed; portable UUIDs remain non-reused after reclamation. Registry churn rejects
before exceeding the cap rather than retaining an unbounded historical directory.
Old textual labels never serve as tombstones or repair bindings by name.

## 3. Exact private authoring occurrences

For the proof propose the private token `@{Producer.Output}` as an expression primary.
This is **not permanent syntax**, not `#` syntax, and is recognized only in an opt-in
experimental document. Producer/output labels in this token each match
`[A-Za-z_][A-Za-z0-9_]{0,62}`. No whitespace, escaping, nested braces, quoting, ordinal,
field/key access or Unicode label spelling occurs inside it. The broader directory
can retain existing UTF-8 policy, but an experimental rename to an unspellable label
must reject rather than make explicit reauthoring impossible. Ordinary syntax outside
the token retains current behavior. Maximum source stays 4,096 UTF-8 bytes.

An experimental ApplyFormula command carries exact UTF-8 source and ordered
occurrences `{ordinal, start_byte, end_byte, DocumentId, ProducerId, OutputId,
ExpectedType}`. Offsets are zero-based half-open ranges in the **entire transmitted
source**, including a leading `=` if present. Ordinals start at zero in increasing
source-start order. Repeated tokens require separate occurrences even when their
portable targets are identical. Bound at 16 occurrences per formula.

Rust lexes/parses the original bytes using the opt-in primary, producing its own
occurrence sequence and mixed AST. Require the submitted sequence to match this
sequence one-for-one: exact spans/ordinal/count, valid UTF-8 boundaries, no overlap,
no omitted/extra annotations and no annotation on a substring/A1/literal/operator.
The source substring must be exactly the parsed whole token. Resolve those token
labels in the pinned directory and compare the result with every submitted portable
ID/type and the owning document. Unknown/mismatched IDs reject; the client cannot
claim an arbitrary slot/graph node. Compile A1 operands by the normal stable CellId
binder and structured operands through the validated adapter table. Do not preprocess
by substituting numbers or invented A1 addresses, which would corrupt source spans.

Use one command state token for directory resolution and install. A stale token or
forged span/identity annotation rejects the whole command before worksheet mutation;
these are command validation failures, not stored draft content. A genuine syntax
or unresolved-name authoring error can be retained as an InvalidDraft under existing
worksheet behavior, with no partially bound dependencies or allocated source slot.
No later directory change repairs that draft. Precision errors retain current policy.

The experimental client obtains typed annotations through a bounded ResolveFormula
request returning occurrences plus the opaque state token; ApplyFormula sends that
same token. Directory/publication/worksheet change in between rejects, preserving
the user's editor draft for an explicit retry. There is no name lookup at evaluation.
Re-resolving and resubmitting after a conflict is a new authoring action, not replay.

Unchanged editor exit compares exact source with the committed source and performs
no reauthoring, preserving bound IDs even after rename. Explicit resubmission/rebind
is separate and resolves all names against current state. Edited source always uses
new occurrence resolution. Display committed source exactly as stored; read-only
inspection shows authoritative IDs, current labels and captured generation/head.
Do not rewrite old labels in place or imply the visible name is execution authority.
Binding annotations are not accepted through ordinary formula endpoints/normal mode.

## 4. Copy and clipboard — no format change proposed

Same-document **server-owned DuplicateBlock** is the proposed experimental copy
path, not OS clipboard serialization. Supply exact state token, ordinary source
rectangle and destination rectangle to the session owner. Use existing rectangle
bounds/exact-shape admission. Capture all source contents before staging destination
edits so overlap is deterministic; reserve before constructing candidates.

For bound mixed formulas, copy authoritative structured operands from the captured
server source, preserving portable target IDs/type. Translate only A1 spans using
ADR-0017. Since A1 rewriting can change source length, recompute structured byte
spans/ordinals in the translated source; validate each original structured token is
unchanged and corresponds one-for-one. Compile via a crate-private trusted-copy path,
not the external authoring resolver: a copied old label after rename must keep its
original ID, not resolve the old spelling. Do not expose a client `trustBindings`
flag. Directory removal does not prevent copying a legitimate dangling binding;
its evaluation remains MissingProducer/Output. Copy creates no source publication.
Stage the entire block and its new edges/annotations; one failed entry rejects all.

**OS native Copy/Cut of any rectangle containing a structured operand is unsupported
in this slice.** Refuse before setting either native or text clipboard content; leave
clipboard unchanged and show the explicit limitation. Ordinary Copy/Paste keeps
PIVOTCB2/cell-block-v2. Do not put structured content into a payload that loses its
operands. Copying text inside an active text editor remains a text operation; such
text is new authoring if later submitted, not an identity-preserving transfer.

There is no foreign structured native payload emitted by this experiment. Reject any
native payload claiming unsupported structured entries/version/foreign document IDs
atomically, and never fall back to TSV when native data is present but unsupported.
Existing legitimate PIVOTCB2 ordinary-cell transfer remains unchanged. A malicious
TSV string is just untrusted source, never trusted annotations. Structured text paste
into cells is unsupported until resolved through the experimental ApplyFormula path;
reject whole rectangles containing private tokens, rather than partially applying.

A future OS-clipboard representation must carry portable identities and source
occurrences, validate document scope and bounded byte accounting, and obtain explicit
format compatibility review. No new clipboard version, private native format or
cross-window identity broker is authorized by this addendum. This restriction is a
proposed tradeoff: genuine identity-preserving same-document duplication is tested,
while identity-preserving OS Copy remains unavailable for these formulas.

## 5. Composite authority and capture publication

Use one opaque session-issued document computational-state token. Server-side it
identifies immutable worksheet/graph roots, binding table, producer/output registry
revision, authoritative source-head map and adapter-contract revision. Rotate it on
any successful mutation of those components, even if numeric bytes are unchanged.
Do not rotate it for layout-only operations or failed-attempt diagnostics. Existing
worksheet revision remains separately observable; it is not this token.

ResolveFormula returns the token for the roots actually used. ApplyFormula,
DuplicateBlock, producer/output rename/delete/create, valid result publication and
explicit unavailable publication require equality with the current token plus exact
owning session/document identity. Validate under the serialized owner before staging
and again before installation if work ever leaves it. Validate expected producer
head/definition provenance from the same pinned roots; one common token is deliberately
conservative and may reject a concurrent but unrelated computational edit.

Rejected candidate attempts install neither source head nor slot context; record
attempt diagnostics separately with their attempted token. Available/Unavailable
head changes rotate authority and result epoch even if worksheet definitions did not
change. Capture pins all computational roots at one owner instant and records the
token, then builds its immutable context from those pins only. A scalar fault is a
valid context entry; resource/completeness corruption aborts the whole capture.
No persisted adapter root or second publication occurs while preparing the context.
Cancellation before root installation installs nothing; after installation, drain and
report known committed state without automatic replay. Layout maps retain their own
presentation token; changing one neither invalidates a numeric binding nor refreshes
a source. Late UI result installation requires the requested computational token and
capture identity, with explicit old-capture inspection remaining separate.

## 6. Concrete reservations and acceptance stop

Keep ADR-0019's one producer, two views, 64 records, text/key/result caps, two captures
and 1 MiB structured-ownership budget. These new reservations share that same budget,
including staged and pinned owners; do not create an additional unbounded allowance.
Ordinary graph nodes, edges and worksheet source still consume existing workbook/core
limits and runtime-domain charges. Conservative proof reservations are additional
admission charges for integration-owned overhead, not process heap measurements.

| Owner | Proposed reservation / ceiling |
| --- | --- |
| Registry entry, including tombstone | 1,024 bytes each; at most 64; labels/diagnostics bounded to fit, no embedded result payload |
| Source adapter slot/table entry | 2,048 bytes; at most one resident including retired/pinned slot |
| Structured formula annotations/source integration | 8,192 bytes per owned formula record, including at most 16 occurrences and at most 4,096 source bytes |
| Retained annotated formula versions | At most 64 across current/staged/capture-pinned versions; at most 32 in current worksheet |
| Captured scalar context plus fault diagnostics | 4,096 bytes per capture; at most two, including one slot and at most one fault |
| Fault diagnostic detail | At most 1,024 UTF-8 bytes, counted within its capture reservation |

Reserve before copying source/annotations, staging registry changes, creating a slot
or context, and before accepting DuplicateBlock destination ownership. Counts cover
retained versions, not just current rows. A new staged version may be refused while
captures pin old ones. Charge immutable shared records once per allocation, not once
per Arc owner. Distinct indexes/contexts/diagnostics own separate reservations. Caller
requests still obey existing frame/work limits; validate count/byte declarations
before allocating proportional buffers. Actual core graph/snapshot allocations keep
their existing accounting in addition to these conservative admission reservations.

Release the charge only after final retained owner destruction; no runtime finish
with borrowed contexts. Failed staging drops partial reservations; candidate bytes,
slot count, head and exact state token remain unchanged. Test shutdown with captures
released, queued/cancelled work drained and zero runtime-domain charge. Tests requiring
more than one resident slot must expect admission rejection, not quietly raise caps.

Before implementation, parent review must approve these exact leaf/context/fault,
private token/annotation, DuplicateBlock-versus-OS-copy and quota choices. The future
acceptance suite must extend the original matrix with forged/extra/missing context
entries; arbitrary NodeId injection refusal; invalid spans and duplicate occurrences;
repeated/mixed references; unchanged versus explicit reauthoring; deterministic
source/arithmetic/SUM error order; overlap copy after rename; clipboard unchanged on
unsupported copy; pinned-retired-slot refusal and reclamation; staged quota rollback;
actual ordinary formula dependents and SUM; and ordinary graphs unchanged.

Keep opt-in non-savability and all three-platform managed/Native AOT gates. This is
an addendum for review, not evidence that any source leaf, authoring command, protocol
change, copy path or test is implemented. No core/workbook/protocol/desktop code,
PIVOTCB2, WB4 or capacity increase accompanies it.
