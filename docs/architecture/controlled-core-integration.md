# Controlled production core integration — Stage 1

Date: 2026-10-19. Status: implemented behind `pivot-core/core-integration`;
default models remain unchanged. Stop after this stage for review.

The parent accepted the [compatibility foundation](persistent-core-compatibility-foundation.md)
and requested a real migration boundary. The selected operations now run through
`WorkingModel`, not a second public model constructor/edit API. The persistent
record/value pages and name/reverse-edge indexes are the same implementation
previously measured; this stage does not introduce another graph representation.

## Integration boundary

`WorkingModel::new` still selects the reference representation. A caller explicitly
creates `IntegrationRuntime`, then calls `enable_persistent(&runtime)` on a model.
Activation is a one-time, guarded O(N) import of stored definitions, including
invalid bindings and retired identities. It never reparses historical source.
A separately reserved scoped import worker borrows the intact source, constructs
the graph and disposes temporary legacy snapshots. Thread/admission/task failure
preserves the source model and its revision; no full source clone is required.
The original node/name/interface/name-string containers are relinquished to the
runtime's worker. Inspection delegates to the shared persistent metadata.
There is no maintained shadow reference model and no automatic reverse conversion.

The `core-integration` feature does **not** enable `scaling-experiment` or expose
its independent public Graph API. That API remains available only for the earlier
research harnesses when explicitly enabled separately. The production-facing seam
is the existing model, its normal edit methods, and a calculation-only capture.
Persistent edits delegate to the existing canonical graph transaction code on
an owned private root at the runtime worker. Binding, parsing and postfix
arithmetic execution are reused. Only selected execution gains a scratch-retirement
hook, so failed traversal/result containers are released on the worker too.

`capture_calculation` works on either backend and returns immutable
`CalculationCapture`; `evaluate` returns immutable `CapturedResult`. Both report
the exact captured public core `Revision`. Selected values include evaluated
dependencies. The reference backend delegates to ModelSnapshot/EvaluationContext;
the persistent backend pins shared graph roots and uses selected-closure execution.
No result cache, dirty scheduler or new expression language is introduced.

This additive capture seam is deliberate: `ModelSnapshot` exposes borrowed full
dependency-order and adjacency slices. This stage does not silently build those
containers while claiming cheap capture. Calling `snapshot` on an activated model
returns an explicit Stage 1 backend-not-ready error. Full-inspection snapshot
integration is a remaining API migration decision, not a completed replacement.

## API, revision and diagnostic compatibility matrix

| Accepted observable contract | Stage 1 persistent behavior |
| --- | --- |
| New standalone model starts at revision 0 in a fresh process-local identity domain | Construct normally, explicitly activate; activation preserves identity and revision |
| Named decimal input creation validates spelling and duplicate live names; advances once | Same method/errors/NodeId allocation; local index/page update |
| Formula creation includes its own name during binding | Same parser and lookup-based binder; same self-cycle draft |
| Input update requires a live input, including assignments of the same value | Same validation; every successful call advances once |
| Formula replacement can convert an input, saves invalid text, and advances even for identical source | Same; original text and stable bound IDs retained |
| Hard failures leave the public model/revision unchanged | Private persistent candidate installed only on success; resource rejection is explicit |
| stage preserves model identity/revision and creates an isolated private branch | Shared roots; later branch edits advance per accepted operation |
| Empty stage does not edit; installing a candidate is caller-serialized | Same; no new automatic commit/rebase operation |
| Core revision equality is model-scoped | Actual WorkingModel Revision retained independently of internal generation counters |
| Capture S and result S survive edits to T and writer destruction | Shared immutable roots; result pins its capture; captured core Revision remains S |
| Selected output normalization deduplicates and sorts, including unknown-output error precedence | Same normalization in canonical persistent request execution; differential test submits reversed/duplicate unknown IDs |
| Arithmetic/dependency failures remain per-node result data | Existing postfix executor and typed EvaluationError reused |
| Invalid source, missing names/references and cycles reject execution | Full accepted first-error/Kahn diagnostics, including blocked downstream nodes |
| Source text and historical binding can disagree after rename | Preserved exactly on activation; draft_nodes exposes original text/bound IDs |
| Name reuse cannot redirect retired references; explicit resubmission repairs | Imported history preserves this; new formula submission binds against current names |
| Input numeric representation survives capture | Exact Decimal parts retained, including signed scaled zero; no new normalization |
| contains_node/node_id/draft_nodes/name/interface inspection | Read current persistent state; interface is immutable imported metadata in this stage |
| Rename, retire, replace_input and all port mutations | Explicit not-ready error before mutation, including otherwise-no-op requests |
| Full ModelSnapshot construction/inspection, transfer export and resolved/invalid transfer edits | Explicit not-ready error; never read empty reference storage or implicitly materialize |
| Public named/override invocation | Existing default API unchanged; not exposed by the integrated calculation-only capture yet |

Reference no-op rules outside the subset remain important: same-name node/port
rename and empty input-port patch do not advance; equal input/default assignments
and identical formula replacements do advance. The integrated backend deliberately
rejects unsupported port/rename calls, including no-ops, rather than claiming
equivalence for that API surface. An empty staged branch is supported on both.

## Version boundaries and hosted-service impact

Public core Revision advances once per successful supported edit, not once per
private batch. A branch with two edits has two increments; private staging itself
has none. `stage_expected` compares the actual core Revision and rejects stale or
foreign revisions; it does not provide a final publication compare-and-swap.
The owner must serialize installation and ensure discarded branch IDs never escape,
as required by the existing stage contract.

Internal aggregate/topology/input generations identify persistent root versions
and invalidation roles only. They are never converted into public revision IDs or
used as substitutes for core or service comparisons. The old materialization
bridge's synthetic revision is confined to compatibility diagnostics and research;
integrated captures/results carry WorkingModel's actual revision separately.

[ADR-0006](../adr/0006-working-model-lifecycle-and-transactions.md) and
[ADR-0007](../adr/0007-versioned-authoring-protocol.md) require a service edit
batch to advance its public working revision once, compile new formulas against
the final candidate directory, and atomically retain its operation receipt.
Replay lookup precedes stale-revision rejection; receipt matching includes target,
expected revision and canonical intent. Publication has its own immutable identity
and final working-head check. None of these contracts is replaced by core counters.
The standalone integration binds at each method call; it is not a hosted batch
compiler. No host resource, receipt adapter, wire ID or publication path is changed.

## Explicit slow paths

Valid ordinary input/formula edits, captures and selected evaluations do not
materialize a reference graph. Local validation still visits dependency closures;
long formulas/deep dependency chains can cost proportionally more than an input edit.

On invalid capture, the accepted full snapshot validator runs via guarded
materialization on the runtime worker. The caller waits for its exact diagnostic;
temporary graph/snapshot destruction also occurs there. This is O(N + E), and is
measured separately from the local valid-edit gate. Diagnostic vectors returned to
the caller are caller-owned data outside the graph allocation domain.

Initially invalid imported models retain the accepted full-validation repair
fallback until valid. Those edit candidates execute and dispose temporaries on
the same worker; a failed operation cannot install a candidate. That fallback is
explicitly O(N + E); a dedicated large invalid-import latency sweep is still pending.
Full snapshot inspection and transfer through the integrated facade are not ready;
their cost is not hidden inside an apparently local operation.

## Ownership and reclamation contract

| Owner/resource | Storage and charges | Disposal path |
| --- | --- | --- |
| Runtime | Shared allocation domain, bounded queue, reserved worker stack/queue metadata | finish refuses live owners, then drains and joins; explicit noninteractive operation |
| WorkingModel graph root | Charged shared record/value/index objects; root handles reference those objects | Automatic queued retirement on writer drop |
| Private stage/candidate | Shared roots plus admitted changed pages, indexes, parser/validation scratch | Preparation, validation and failure cleanup execute on worker; caller installs only successful candidate and queues obsolete writer root |
| Calculation capture and clones | One charged captured graph state; shared payload charged once | Automatic queued retirement, including last clone |
| Calculation result | Charged work/result reservations; owns the exact capture | Automatic queued retirement; values exposed only immutably |
| Selected execution scratch, including partial failure | Admitted traversal containers and partial results; reservations retained until actual disposal | Scratch-retirement hook queues containers before returning result/error |
| Invalid-draft diagnostic bridge | Guarded temporary materialization and legacy validation | Whole slow task runs on worker, including error cleanup |
| Imported original reference containers | Pre-existing caller/model allocation, not retroactively graph charged; separately admitted temporary import stack | Original containers handed to worker only after successful activation; temporary import snapshots are destroyed by scoped import worker |
| Interface/model-name metadata | Charged shared graph objects; facade delegates access and keeps no shadow copy | Released with graph roots on worker; stage does not clone the interface directory |
| Document/workbook adapters | Not integrated; current accepted ownership remains | Existing paths; no assertion of asynchronous disposal for them |
| Hosted resource/receipt/publication | Not integrated; current accepted owner/revision contracts remain | Existing paths |

Stage 1 schedules private edit preparation and exact diagnostic fallbacks on the
runtime worker; independent calculations execute on the calling thread and queue
only their disposal scratch/results. Edits within one runtime are serialized with
reclamation. A long diagnostic or backlog can delay another model's edit. Multiple
runtimes may share the allocation domain while using separate workers. This is an
explicit scheduling limitation of this integration policy, not a global engine
lock or an assertion of production multi-model throughput.

The queue is bounded. A full queue applies blocking backpressure while preserving
the owner; it never drops a large root merely because enqueue failed. Ordinary
model/capture/result Drop can therefore wait, and is not promised nonblocking.
The worker catches destructor/task unwinds and remains available. If disconnected,
the returned owner is safely released synchronously: an explicit emergency cleanup
exception, not a leak or an asynchronous-latency guarantee. Tests cover full and
disconnected channels. Shutdown returns the runtime while any owner still uses
its queue; callers dispose resources, then retry finish off the interactive path.
Dropping the runtime handle without finish detaches the join; remaining owners
keep the worker alive until the channel closes. flush waits for prior disposal.

Models in one runtime share one allocation domain. `with_domain` permits several
runtimes to share the same domain for aggregate known-allocation admission, tested
with multiple workers. Graph object charges release only at final ownership drop;
candidate/result reservations coexist with retained generations. Retaining objects
and queue backpressure can correctly cause later admission rejection.

These counters are **not an RSS limit**. Caller request/output/error containers,
legacy import source, facade handles, bounded iterator staging buffers, boxed
channel handoffs, allocator
overhead/reserves and non-core resources are not all charged. The queue metadata
reservation is an estimate, not allocator interception. Worker stack reservation
also differs from committed resident pages. Future application policy must budget
these separately and share the core domain with its document/host ownership plan.
Exact allocation/OS limits and cancellation-aware backpressure remain later work.

## Review gate and downstream impact

- pivot-storage: no codec, transfer layout, file broker or admission-cap changes.
  It continues using the reference backend; its full snapshot/export needs are
  explicit blockers to selecting the integrated backend there.
- pivot-workbook: no projection or coordinate transaction changes. Demanded anchors,
  moves, retirement and paste remain outside Stage 1. It retains the reference path.
- pivot-host: no lifecycle, receipts, public revisions or protocol changes. It
  retains the reference path; final-directory binding/ports/inspection must migrate
  before any backend activation is considered there.

No production default or capacity increase is authorized by this feature. The next
review should assess this facade/capture boundary, ownership backpressure and the
full-inspection/ports gaps before authorizing another bounded migration stage.
See the [handoff](../handoffs/2026-10-19-controlled-core-integration.md) and
[benchmark specification](../performance/controlled-core-integration.md).
