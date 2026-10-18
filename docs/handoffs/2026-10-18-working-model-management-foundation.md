# Working-model management foundation — Slice A

Date: 2026-10-18
Status: Implemented in process; ready for review

## Implemented

- Accepted ADR-0006 and its index with the review clarifications: draft source
  preservation, final-directory binding, separate revision/snapshot/publication
  identities, receipt-before-revision lookup, capacity reservation, and atomic
  state/receipt commit. Editing-specific invariants are accepted but deferred.
- Added `pivot_host::management::WorkingModelService` alongside the unchanged
  immutable invocation host. No new crate or authoring wire operations.
- Added independent UUIDv4 service, working-model and attachment identities;
  exact portable working revisions; explicit memory-only durability reporting.
- Added replay-safe empty-model creation, authorized list/inspect, multiple
  principal-owned attachments, and close without model deletion.
- Added a separate authoring authorization trait, service-instance checks,
  resource/attachment/receipt quotas, bounded strings and typed API failures.
- Added principal-scoped successful and rejected create receipts. Identical
  replay returns the original outcome; mismatched name intent is rejected.
- Added private preparation failure and final-authorization revocation tests.
  Reserved vector capacity and exclusive access make resource/receipt insertion
  one logical commit, with no fallible work between insertions.

API, ownership, identity, limits, authorization and recovery details:
[management foundation](../architecture/working-model-management.md).
Source: `crates/pivot-host/src/management.rs`.

## Tests actually run

Full Windows local CI passed:
`artifacts/ci/20261018T182011320836Z/summary.json`.

- `cargo fmt --all -- --check`.
- `cargo clippy --locked --workspace --all-targets -- -D warnings`.
- `cargo test --locked --workspace` (including seven new management unit tests
  and one published-invocation coexistence regression test).
- .NET 10 Release solution build, warnings as errors.
- Decimal parity: 10,099 vectors, no value/error failures or scale differences.
- Rust/C# protocol parity: 58 decimal layouts and existing protocol fixtures.
- Real local daemon/CLI/REPL integration.

The initial sandboxed CI run passed Rust gates but the .NET build failed with no
diagnostic; rerunning outside the sandbox passed the entire pipeline. Afterwards,
the receipt-quota test was strengthened to leave resource capacity available;
format checking and all 16 pivot-host tests passed again. Runtime implementation
was unchanged after full CI. `git diff --check` also passed.

Coverage includes creation/discovery, exact revisions, multiple attachments,
disconnect-equivalent close, invalid/wrong-principal attachment use, stale
instances on every contextual method, authorization denial and discovery
filtering, quotas without eviction, repeated create reconciliation after a
discarded response, mismatched intent, principal-isolated outcomes, failed
preparation, commit-time denial and existing published invocation behavior.

No Linux/macOS rerun for this slice. Latest REPL macOS validation and Native AOT
remain on the verification backlog. No claim of durability, actual network
lost-response testing for authoring, or allocator fault injection.

## Accepted versus provisional

Accepted: connection-independent resources, explicit memory-only retention,
portable identities, authorization separate from invocation, retained receipts,
capacity-before-commit and atomic creation/receipt visibility. ADR-0006 also
accepts future transaction and publication invariants; this does not implement them.

Provisional implementation choices: module within pivot-host, UUIDv4 allocation,
typed Rust API spellings, synchronous exclusive-access serialization, vectors for
bounded registries, default quotas (128 resources / 256 attachments / 1024
receipts), display-name bounds, and empty-model creation without an implicit
attachment. These do not establish a persisted or wire format.

## Open issues and boundaries

- Principals must be authenticated by the embedder; IDs are not credentials.
  Policy decides sharing and creator permissions. No default allow-all policy.
- Resources/receipts currently remain until service shutdown; no delete API in
  this slice. Future deletion needs instance-lifetime tombstone receipts.
- An adapter must track and close attachments on disconnect; no IPC authoring
  adapter exists yet. Abandoned attachments remain bounded by the active quota.
- Rejected admitted operations consume receipt quota. Admission failures do not
  record an outcome. Once receipt capacity is full, refuse new creates; replay
  remains available. There is no silent eviction or durable exactly-once claim.
- Inspection exposes only empty graph metadata. No core handles escape; portable
  authoring node/port mappings are deferred until graph editing is implemented.
- No general mutations, forks, unpublished evaluation, publication coordination,
  persistence, workbook projection, GUI integration or protocol/CLI extensions.

## Recommended next slice

Review Slice A, then implement Slice B: private graph staging, atomic edit batches,
stable authoring node/port references, expected revisions, retained original draft
source and diagnostics, edit receipts and rollback/replay tests. Keep evaluation,
publication and transport out of Slice B. Stop here pending that review.
