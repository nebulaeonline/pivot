# Working-model transactional graph editing — Slice B

Date: 2026-10-18
Status: Implemented in process; ready for review

## Implemented

- Added `apply_edits` against an exact public working revision, with ordered
  nonempty batches, private staging and one public revision advancement per commit.
- Added raw `read_draft` views pinned to the exact current revision, including
  original formula source, portable dependencies, public ports and diagnostics.
- Added portable UUIDv4 authoring node/port identities and temporary batch
  references. Core graph handles and internal revisions remain private.
- Added node add/remove/rename/definition replacement, input-port add/remove/
  rename/target/default updates, and output-port add/remove/rename/target updates.
- Added explicit core staging, raw definition inspection, definition-kind input
  replacement and port updates preserving stable core identity.
- Preserved existing compiled bindings. Newly submitted formulas bind against
  the final candidate directory; unresolved old drafts require explicit resubmission.
- Added edit receipts sharing the creation receipt namespace and capacity quota.
  Full typed intent comparison includes decimal layout and exact source bytes.
  Replay lookup precedes expected-revision validation; identical retry returns
  the original outcome without advancing the model again.
- Reserved receipt capacity before staging; prepared caller/receipt results
  before atomically replacing state and recording its outcome under exclusive
  service access. Added final authorization and expected-revision checks.
- Added configurable command, batch, graph, source and cumulative identity bounds.
  Hard failures leave graph, name directory, port mappings and revision unchanged.
- Extended the published invocation regression test to perform real working-model
  edits/replay between invocations and confirm unchanged published results.

Documentation: [transactional editing](../architecture/working-model-editing.md),
[management API](../architecture/working-model-management.md).
ADR-0006 remains Accepted without changes for this slice.

## Tests actually run

Full Windows CI passed:
`artifacts/ci/20261018T183458656559Z/summary.json`.

- Formatting and locked workspace Clippy, all targets, warnings denied.
- Locked Rust workspace tests, including 20 new transactional editing tests.
- .NET 10 Release solution build, warnings as errors.
- Decimal parity: 10,099 vectors; zero value/error failures and scale differences.
- Rust/C# protocol parity: 58 decimal layouts plus existing protocol fixtures.
- Real daemon/CLI/REPL IPC integration.

After that full run, batch accounting was conservatively increased from 256 to
512 fixed bytes per command (plus formula bytes). The final code passed another
format check, locked workspace Clippy and locked workspace test run. This final
change affects only in-process authoring admission; no wire or numeric changes.
Markdown relative-link checks and `git diff --check` passed.

The 20 editing tests cover:

- Atomic multi-edit success and exactly one public revision transition.
- Forward formula references and final-directory binding across renames.
- Competing writers with the same observed head; stale draft read rejection.
- Mid-batch rollback of nodes, source, names, existing port bindings and directory.
- Identity preservation across rename; broken references after deletion/name reuse.
- Incomplete source preservation, repair and no implicit unresolved-draft rebinding.
- Input/output port creation, updates, invalidation, repair and removal.
- Duplicate names/temporaries, invalid and cross-model node/port targets.
- Temporary-to-portable mappings, transient removals and deleted ID non-aliasing.
- Identical replay/lost-response reconciliation and typed-intent mismatch,
  including source whitespace, edit order and optional-default decimal scale.
- Cross-kind Create/Edit operation ID conflicts.
- Authorization denial/revocation at the final commit check.
- Receipt exhaustion independent of model capacity, model/source/identity limits,
  batch admission failure without a receipt, cycles and later repair.

No Linux/macOS rerun for this slice. Latest REPL macOS validation and Native AOT
remain outstanding. Lost-response simulation is in process by discarding/replaying
an outcome; there is deliberately no authoring transport yet. No allocator failure
injection or crash-durability claim.

## Accepted and provisional decisions

Accepted ADR-0006 guarantees remain intact: connection-independent resources,
explicit memory-only retention, stable identities, private staging, draft saves,
atomic revision-checked commit, retained receipts and immutable publication
separation. Invalid drafts use raw definitions, never valid snapshots as storage.

Provisional implementation/API choices:

- Exclusive synchronous service access and full raw-state staging copies.
- Rust API/type spellings and coarse hard-error categories without edit indices.
- One shared temporary declaration namespace; temporary object references must
  follow declaration, while formula name references may point forward.
- Port kinds remain fixed; changing input/output kind requires remove/add.
- Current default limits and conservative 512-byte command accounting.
- Revision-pinned whole draft reads, no pagination or retained history.
- Separate Create/Edit receipt vectors under one shared quota/operation namespace.
- Inspect authorizes draft reads, Edit authorizes edits and known edit outcomes,
  and Recover authorizes Unknown outcome queries.

## Open issues and boundaries

- Cumulative identity budgets retain deleted-reference tombstones. Models with
  extensive churn can hit them even when the live graph is small. No garbage
  collection, resource deletion or receipt eviction is introduced here.
- Default 1,024 shared receipts may be restrictive for editing sessions; it remains
  configurable. Admitted conflicts/structural rejections also consume capacity.
- Local formula and port diagnostics are collected; cycle/blocked-node reporting
  may wait until earlier formula errors are repaired. Core structural snapshot
  checking is used internally for these diagnostics, without exposing or retaining
  a calculation snapshot and without evaluating anything.
- Final expected-revision comparison is explicit. Public mutation calls serialize
  through exclusive access, so there is currently no public mid-staging writer race.
- Authenticated principals still come from the embedder. Future IPC must derive
  them from authenticated connection context, not trust client-supplied strings.
- Bounds are logical counts/work accounting, not exact allocator byte accounting.
- No unpublished evaluation, publication coordination, Protobuf/daemon/REPL/C#
  editing, persistence, new numeric types, workbook projection or GUI integration.

## Next recommended slice

Review Slice B before implementing Slice C: exact working-revision captures,
unpublished validation/evaluation and coordinated publication into the immutable
host. Keep calculation snapshots distinct from published revisions and ensure
publication final-head/authorization checks and publication receipts share an
atomic logical commit. Stop here pending review.
