# Working-model validation, evaluation and publication

Implemented Slice C of ADR-0006 in `pivot_host::management`. The same APIs now
serve the ADR-0007 adapter, opt-in local daemon and minimal authoring REPL.
See [daemon integration](daemon-authoring.md) and [REPL](command-repl.md).
This service remains MemoryOnly. ADR-0008 is Accepted for optional hosted
persistence; its portable foundation is implemented but database work is deferred.
[Document sessions](document-foundation.md) calculate independently of publication.

## Validation and capture

`validate_working(principal, expected)` requires Validate permission and an exact
current `WorkingRevision`. It returns `ValidationReport { revision, valid,
diagnostics }`. Structural failures are report data; stale instance/revision,
authorization and work-budget failures are service errors. Validation does not
edit the model, advance its revision, publish it or retain a snapshot.

`capture_evaluation(principal, expected)` requires Evaluate permission and returns
an owned `CalculationCapture`. It globally validates and captures one consistent
immutable core snapshot with its portable ID mappings and input values. Structural
failure returns `CaptureError::Validation(ValidationFailure)` with portable
diagnostics. Service failures remain separately typed. Saving an invalid draft
through ApplyEdits continues to succeed under the existing draft rules.

Capture occurs under a shared service borrow; Rust prevents overlapping edits
during that capture. **Release the service lock before calling capture.evaluate.**
Evaluation borrows only the detached capture. Its snapshot and mappings survive
later edits, model/service destruction and dropped client attachments. The capture
retains the original authorization policy and principal, not the service itself.
No resource deletion API is added in this slice; destruction is tested directly.

`SnapshotProvenance` contains two distinct identities: exact working revision and
a fresh `CalculationSnapshotId`. Separate captures of one revision get different
snapshot IDs. These are not published revision IDs, retained lookup handles or
promises of future result retrieval. No working history or result registry exists.

## Unpublished calculation

`CalculationCapture::evaluate(EvaluationRequest)` accepts selected portable
authoring node IDs and temporary decimal input overrides. Output selection must
be nonempty. Unknown/duplicate outputs and unknown/duplicate/non-input bindings
reject the request, including bindings outside selected outputs' dependencies.

Omitted overrides use captured **graph input values**. Interactive calculation
does not enforce required public ports or use optional public-interface defaults.
Those are the separate published invocation contract. `effective_inputs` reports
all captured inputs with their effective value and Captured/Override source.
Overrides do not modify either the capture or working model.

Each call uses the existing EvaluationContext/InvocationRequest implementation
and fresh local execution state. No parser/evaluator duplication or mutable shared
calculation cache. Arithmetic failures are selected-output data, separate from
structural capture failures and service/request failures. Dependency errors carry
portable dependency/origin IDs; unrelated outputs can succeed. Core runtime
handles, internal revisions and raw core error values never escape this API.

Evaluate permission is checked at capture, after capture, at execution and before
result delivery. Revocation can deny a previously captured calculation. Policies
may continue allowing a retained capture after the service is destroyed. In-process
callers are responsible for releasing any external lock before calculation and
for scheduling concurrent calculations; no global fairness/cancellation policy.

## Publication lineage and host ownership

Every working resource receives an independent `PublishedModelId` at creation.
It is its sole logical publication lineage for this slice. A successful publication
allocates a fresh `PublishedRevisionId`; names and working edit sequences are not
published revision identities. Publication never advances the working edit revision.
Forking/importing/appending to an arbitrary destination lineage remains deferred.

Published port IDs are `port-<portable authoring port UUID>`. The public namespace
is distinct from raw authoring IDs, and no core handle contributes to the encoding.
Renames and compatible target/default updates preserve port identity. Remove/add
creates a new port ID even if the public name is reused. Core port identities also
remain stable through staged edits, satisfying the existing registry's lineage
checks. Future imported/forked graphs need explicit remapping, not formula reparsing.

The management service owns a `ModelHost` registry and exposes only a shared
`published_host()` reference for normal handshake/describe/invoke dispatch. This
ownership keeps registry writes and publication receipts under one exclusive
commit boundary and prevents receipts from being accidentally replayed against a
different registry. Standalone ModelHost usage remains supported and unchanged;
the current demo daemon still uses its standalone registry. No authoring daemon
integration or cross-registry migration is introduced.

Existing invocation authorization is supplied independently through Session. A
Publish grant neither grants nor bypasses Discover/Describe/Invoke permission.
Public named invocation continues to discover descriptions and use their portable
port IDs; the existing strict public interface and required/default rules apply.

## Preparation, commit and receipts

`PublishRequest` contains instance, working model, exact expected revision and
client operation ID. `publish` is a prepare/commit convenience. The split API lets
an embedder release service access between phases:

1. `prepare_publication` checks Publish permission, shape and existing operation
   receipt before expected-head validation. Capture the exact candidate without
   publishing. Return an opaque principal/instance-bound, noncloneable token.
2. The caller may release its lock; editing may advance the model.
3. `commit_publication` rechecks current permission and principal ownership,
   reconciles any existing receipt before revision/quota checks, and reserves
   receipt capacity before admission. Check the expected head and candidate.
4. Prepare the registry entry with the existing host's description/identity/lineage
   validation, without insertion. Recheck Publish permission and exact working
   head after that preparation. Prepare owned receipt/result values.
5. Install the registry entry and successful receipt under exclusive service
   access, without callbacks or recoverable failures between them. No observer
   can see half a commit. Standard allocator aborts are process-fatal and outside
   recoverable API semantics; no crash-durable save is claimed.

Invalid draft preparation returns an opaque invalid candidate, so commit/publish
can record its structured validation rejection. No snapshot is installed. A
structurally valid model with arithmetic failures in some input scenarios may
still publish; publication does not run test calculations implicitly.

Preparation is observational: it reserves no receipt, changes no working state,
and may be dropped without leaving a pending operation. Stale-head, permission,
shape, work or capacity failure before admission has no receipt. A ready token
holds a bounded capture lease; dropping/consuming it releases that slot. Capacity
must still be available at commit. Post-admission validation, final-head,
final-permission, registry or publication-quota failure records a replayable
rejection and leaves the registry unchanged.

Create, Edit and Publish share the service/principal operation namespace and
receipt count. Complete typed publication intent includes instance, model and
expected revision. Identical replay returns the original publication/rejection
before validating a now-stale head or exhausted capacity. Changed intent or
reusing an ID from another mutation type returns OperationIdConflict. Multiple
prepared tokens for one request still commit at most one registry entry.

`operation_outcome` reports Published or PublicationRejected for publication,
requiring current model-specific Publish permission. It retains existing Create,
Edit and Unknown behavior. Reconcile lost responses against the same instance;
there is no automatic retry or durable exactly-once guarantee. All resources,
published revisions and receipts remain memory-only.

## Limits and concurrency

`with_all_limits` accepts existing Limits/EditLimits plus CalculationLimits,
reported by `info()`. Existing constructors use default calculation limits.

| Calculation/publication limit | Default |
| --- | ---: |
| Live detached captures / ready publication tokens | 32 shared slots |
| Input overrides per calculation | 1,024 |
| Selected outputs per calculation | 1,024 |
| Conservative valid-snapshot work cost | 100,000 |
| Retained published revisions, all lineages | 256 |

Work cost uses the existing host's full-snapshot node + edge + bound-token count.
Structural snapshot construction precedes that cost check and is bounded by the
existing graph/source limits. Invalid draft diagnostics remain bounded by those
same authoring limits. Published invocations use the same configured work budget.
Capture slots are atomically counted and released with owned state; validation
does not retain one. No result retention or snapshot retrieval service is added.
Returned values/views are caller-owned; limits are logical work/retention bounds,
not a complete process allocator budget.

Mutations and publication commit still serialize through exclusive service access.
Registry preparation currently also runs inside commit, bounded by retained
revision/port limits. Immutable calculation runs outside it. The split publication
API exposes a real edit-between-prepare-and-commit conflict window; the final
comparison rejects it instead of choosing a newer revision. Fine-grained locks,
parallel staging, cancellation and scheduling remain deferred.

## Deferred

No authoring Protobuf, IPC/REPL/C# operations, persistence, resource deletion,
identity compaction, richer edit-index diagnostics, workbook projection, GUI,
new numeric types or incremental recalculation. Next review should prioritize
the versioned authoring protocol and external clients, with persistence design
as a near-term follow-up before substantial real-world authoring.
