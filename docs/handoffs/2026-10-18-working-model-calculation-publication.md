# Working-model validation, evaluation and publication — Slice C

Date: 2026-10-18
Status: Implemented in process; ready for review

## Implemented

- Added exact-revision structural validation with portable structured diagnostics,
  leaving invalid drafts saveable and working revisions unchanged.
- Added detached immutable calculation captures with independent snapshot identity,
  portable authoring mappings, captured defaults and temporary decimal overrides.
- Reused EvaluationContext; returned selected output values/arithmetic/dependency
  errors and effective input provenance without leaking core handles.
- Kept captures independent of the service borrow and lifetime, so execution can
  continue after edits or service destruction. Retained policy checks execution
  and delayed result delivery without retaining the service itself.
- Added independent logical published model/revision identities. One working
  resource owns one publication lineage; public port IDs remain stable through
  compatible edits and change on remove/recreate.
- Added prepare/commit publication and a synchronous publish convenience. Commit
  rechecks the exact head and Publish authorization; editing between the two
  phases causes a conflict instead of publishing a newer revision.
- Added publication receipts under the existing shared mutation namespace/quota,
  with typed intent comparison and replay before expected-head validation.
- Split internal ModelHost registration into validation/preparation and installation.
  Coordinated publication owns both registry and receipt ledger under one exclusive
  boundary; no recoverable failure/callback lies between successful insertions.
- Added calculation capture, binding/output, work and retained-publication limits.
- Preserved existing standalone hosting, immutable invocations and all prior
  management/editing APIs. No Protobuf, transport, daemon or CLI authoring changes.

Detailed APIs, identity, ownership, authorization, limits and failure behavior:
[calculation/publication architecture](../architecture/working-model-calculation-publication.md).

Primary source: `crates/pivot-host/src/management/calculation.rs`.
Tests: `crates/pivot-host/tests/calculation.rs` plus the internal registry-failure test.

## Tests actually run

Full Windows local CI passed on the final runtime code:
`artifacts/ci/20261018T185429913235Z/summary.json`.

- Rust formatting.
- Locked workspace Clippy, all targets, warnings denied.
- Locked workspace Rust tests: **90 passed**, including **18 new Slice C tests**
  (17 integration tests plus one internal registry-failure test).
- .NET 10 Release solution build, warnings as errors.
- Decimal parity: 10,099 vectors, zero value/error failures or scale differences.
- Rust/C# protocol parity: 58 decimal layouts and existing request/value/error fixtures.
- Actual daemon/CLI/REPL IPC integration.
- Markdown relative links and `git diff --check`.

New coverage demonstrates:

- Valid financial models; structured rejection of incomplete syntax, missing
  references, cycles and invalid public bindings without changing saved drafts.
- Captured graph defaults versus distinct optional public defaults; temporary
  overrides and repeated-call isolation.
- Arithmetic and dependency failures alongside successful independent outputs.
- Old-revision results after editing and after destroying the management service.
- Unknown, duplicate and computed input overrides; invalid output selection.
- Named public invocation through described portable ports, exact old/new published
  revision selection, working revision unchanged by publication and stable port
  IDs through rename. Recreated ports receive new IDs within the same lineage.
- Real edit-between-publication-prepare-and-commit conflicts.
- Separate publication authorization, final-check revocation and capture/execution/
  result-delivery evaluation policy checks.
- Lost-response-equivalent replay with a stale working head and full quotas;
  mismatched intent and shared Create/Edit/Publish operation namespace conflicts.
- Capture slots, valid-snapshot work bounds, published revision limits and receipt
  exhaustion after preparation but before admission.
- Dropped proposals, duplicate prepared requests, stale-instance/principal-bound
  tokens and no duplicate registry entry.
- Actual registry validation failure injected through private test access:
  registry unchanged, one retained rejection, replay unchanged. No mocked
  successful insertion or allocator failure simulation.

No Linux/macOS rerun for this slice. Latest REPL macOS validation and Native AOT
remain unverified. There is no authoring transport, so ambiguous-response tests are
in-process reconciliation simulations, not authoring IPC fault tests.

## Accepted versus provisional

ADR-0006 remains Accepted without modification. Immutable capture, save-versus-
validation separation, final publication head/permission checks, immutable
published revisions and atomic successful publication/receipt visibility implement
the accepted architecture.

Provisional choices needing review:

- The management service owns a dedicated ModelHost, exposed read-only for dispatch.
  This prevents a receipt from referring to a different externally supplied registry
  and keeps both writes under one borrow. Existing standalone ModelHost remains
  available; the demo daemon has not switched to this managed registry.
- One service-assigned PublishedModelId per working resource, no arbitrary destination
  lineage or published-model fork/import operation yet.
- Published port strings prefix portable authoring UUIDs with `port-`; they never
  encode process-local IDs.
- Synchronous prepare/commit API and a noncloneable opaque proposal token.
  Preparation reserves no receipt and may be dropped; admission occurs at commit.
  Structurally invalid proposals can be committed only as recorded rejections.
- Memory-only quotas: 32 live captures/ready proposals, 1,024 input overrides,
  1,024 selected outputs, 100,000 conservative snapshot work and 256 published
  revisions, all configurable with `with_all_limits` and reported through info.
- Interactive evaluation uses captured graph defaults, not strict public-interface
  requirements/defaults. Existing published invocation retains its strict contract.

## Known limits and open issues

- Capture/structural snapshot construction still borrows the service. Release any
  embedding lock before evaluating. Mutations and publication registry preparation
  still serialize through exclusive service access; fine-grained concurrency is
  deferred. The split API nevertheless permits edits before publication commit.
- Pre-admission shape, permission, stale-head, work or receipt/capture capacity
  failure has no receipt. Post-admission publication validation, final-head,
  final-permission, registry and published-cap failures retain rejected outcomes.
- No service-owned results registry, snapshot retrieval, working revision history,
  cancellation/fairness scheduler or durable exactly-once guarantee.
- Capture permission is checked using its retained principal/policy. The embedder
  still authenticates principals; later IPC must derive them from connection context.
- Existing diagnostic completeness, coarse edit failures, cumulative identity
  tombstones and finite retained-receipt capacity remain as documented in Slice B.
- No resource deletion API. Evaluation after destruction is tested; deletion-specific
  lifecycle behavior awaits that future operation.
- Persistence remains absent: shutdown loses working resources, the coordinated
  published registry and receipts. Logical identity stability applies within this
  service instance, not across restarts.
- No authoring wire/client operations, new numeric types, workbook projection,
  Avalonia integration or incremental recalculation.

## Recommended next work

Review Slice C, then design/implement the versioned authoring protocol and external
client access in bounded stages. Follow with REPL authoring and actual C# IPC /
Native AOT validation. Prioritize persistence design before substantial real-world
authoring; keep workbook coordinates a frontend projection. Stop here pending review.
