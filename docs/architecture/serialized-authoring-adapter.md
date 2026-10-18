# Serialized in-process authoring adapter

Stage A of ADR-0007 is implemented in `pivot_host::management::adapter`.
It exercises real serialized v1.1 requests against the existing working-model
service. Subsequent [Stage B integration](daemon-authoring.md) now enables optional
daemon authoring IPC. REPL editing remains deferred and durability is MemoryOnly.

## Ownership and dispatch

`AuthoringAdapter::new` receives an `Arc<Mutex<WorkingModelService>>` and limits;
`open_session` receives a trusted principal supplied by the embedder. This API
does not authenticate client-supplied strings. Each session must negotiate v1.1
and `working-models-v1`. Bounded raw preflight runs before generated request
decoding, including the handshake. Malformed uncorrelatable frames use an empty
request ID rather than decoding rejected payloads again.

All twelve authoring operations delegate to the existing service: service info,
create, list, open, close, draft reads, diagnostic reads, edits, validation,
evaluation, publication and receipt recovery through operation-outcome requests.

The adapter owns no second model or receipt registry. Evaluation captures an
immutable exact revision under service access, then evaluates after releasing
the mutex. Publication uses the existing prepare/commit boundary and coordinated
published registry. Rights are rechecked before privileged response delivery;
denial after a commit does not erase its receipt or undo committed work.
Service info has its own `Info` permission. Diagnostic reads permit Inspect,
Validate or Evaluate independently.

Sessions track bounded live and previously closed attachment IDs. Duplicate
Close succeeds only for that session's own closed IDs and still checks current
permission. Explicit session close and Drop release owned attachments even when
Close permission has been revoked. Models and receipts survive disconnect.
Do not dispatch, close or drop sessions while holding the supplied service mutex.
Authorizers must not panic or recursively lock that service; policy execution
is trusted embedder code.

## Edits and replay

Wire SetInputPort becomes a native staged `PatchInputPort`: omitted fields remain
unchanged in the transaction candidate, including changes made earlier in the
same batch. Requirement-only patches preserve existing invalid targets so drafts
remain editable. Empty patches fail admission.

The existing service receipt stores decoded original wire intent alongside the
native edit request. This preserves presence and exact decimal limbs/sign/scale,
even when numeric execution normalizes a value. Equivalent protobuf field ordering
does not alter intent. No adapter receipt ledger or read/merge/resubmit is used.
A native-only edit and wire edit using the same operation ID conflict rather
than silently sharing receipts with different intent representations.

## Bounds and paging

Default ceilings are 1 MiB messages, a 512 KiB page target, 128 entries per page
(64 by default), 60-second cursors, 16 sessions per adapter and 256 tracked
attachment IDs per session. Service quotas remain separately enforced and
advertised. `ResourceLimits.authoring_sessions` is additive field 25.
The negotiated message ceiling is the smaller request/response ceiling; both
directions and page targets obey it. Configured message ceilings range from
512 bytes through 1 MiB.

Pages use portable-ID keysets for best-effort listing and exact-head indexed
views for draft/diagnostic reads. Head changes invalidate exact-view cursors.
Authorization applies on every page and at delivery. Encoded response bytes,
correlation IDs and continuation overhead count against limits. An individual
entry can exceed the soft target but never the hard ceiling; oversized entries
and calculation results fail with ResourceExhausted instead of truncation.
Mutation response capacity is conservatively checked before admission so a
commit cannot depend on an unrepresentable success response.

Cursors contain a version, original expiry, position, last UUID and query digest,
authenticated using HMAC-SHA256 with a random adapter-local key. The signature
binds the trusted principal, session, instance, revision, section and normalized
page options. Expiry is not renewed on continuation. There is no retained cursor
or read-view cache. Injected clocks must be monotonic. Restarting an adapter or
changing sessions invalidates its cursors.

## Validation and next boundary

`crates/pivot-host/tests/adapter.rs` exercises serialized operation paths,
transaction rollback/replay, partial patches, incomplete drafts, authorization
revocation, off-lock evaluation, immutable publication, attachment cleanup,
cursor integrity/expiry, competing writers and bounded responses.

Stage B adds daemon integration, trusted transport identity and a unified
demo/authored invocation registry. Persistence, receipt-detail paging, C# client
integration and REPL editing remain separate work.
