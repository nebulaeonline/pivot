# Pivot Handoff — Working-model service design

Date: 2026-10-18
Branch / commit: uncommitted
Status: Proposal

## Objective

Design the owner-authorized working-model service slice recommended by the command
REPL review. Stop at the architecture proposal; do not implement editing operations,
expand Protobuf schemas, or choose persistence.

## Implemented

Documentation only:

- `docs/adr/0006-working-model-lifecycle-and-transactions.md` — Proposed.
- `docs/architecture/working-model-service.md` — boundaries, identity/lifecycle,
  atomic edits, draft validity, unpublished evaluation, publication, concurrency,
  failure/reconnect behavior, authorization, descriptive APIs, and end-to-end examples.
- Updated status to distinguish proposed authoring from implemented invocation.
- Corrected ADR index: ADR-0005 Accepted; ADR-0006 Proposed.

No runtime, dependency, test, schema, daemon, or CLI changes were made in this slice.
Existing historical ADRs/handoffs were preserved.

## Interface / semantics proposed

Working models belong to the service, not an IPC connection or terminal. Multiple
authorized attachments share one working head. Disconnect/close does not destroy
work; initial retention is memory-only until explicit deletion or service shutdown,
with quotas and no silent idle eviction. Restart is distinguished by instance ID.

Expected-revision edit batches commit atomically and advance the service revision
once. Stage privately and compare again at commit; no automatic merge. Existing
bound references remain stable. Newly submitted formulas bind against the final
candidate directory; unresolved/invalid drafts remain editable with diagnostics.
This transaction binding rule is explicitly a proposal, not a silent change to
the current single-edit API.

Evaluation captures a revision and releases edit access. Publication validates a
candidate then performs a final head/authorization check and coordinated immutable
registry insertion. Published invocations remain independent. Read-only invalid
draft views require a new boundary; today's ModelSnapshot is valid-only.

Mutation operation IDs and retained receipts handle ambiguous responses separately
from request IDs. Same-ID/same-intent reconciliation cannot reapply a committed edit
within the same instance. Restart does not promise durable exactly-once behavior.
Authorization for editing, definition inspection, publication, and invocation is
separate. Portable authoring identities never expose runtime handles.

## Tests actually run

- `git diff --check` — PASS (line-ending conversion notices only).
- Local Markdown link/required-deliverable review — PASS.
- Rust/.NET tests and CI — NOT RUN: documentation-only slice; no runtime change.
- macOS REPL rerun — NOT RUN: no connected Mac execution environment in this session.
  The previous REPL handoff's Windows/Linux verification boundary remains unchanged.

## Decisions made or proposed

- **Accepted directions applied:** graph-first core, edit-time stable references,
  immutable calculation snapshots, published exact revisions, client independence,
  explicit authorizer boundary, and no automatic ambiguous invocation retry.
- **Proposed:** service-owned retention and quota policy; whole-batch revisions;
  final-directory binding for newly submitted formulas; draft error classification;
  lifetime-scoped receipts; and final-head publication comparison.
- **Reserved:** durable identity encoding, persistence/recovery, concrete synchronization
  and staging storage, protocol version/field tags, receipt digest format, and quotas.

## Open issues / risks

The current core is not a transaction API and does not expose raw invalid draft
views, portable editing IDs, staging, rollback, or fork/remapping. The host's current
PortId-continuity check needs a deliberate logical-lineage adapter for reconstructed
working models. Atomic publication must coordinate registry insertion and its receipt.

Memory-only retention protects against disconnect, not daemon crash/restart. Receipt
and tombstone retention consumes bounded memory; resource exhaustion must reject
new mutations rather than weaken replay guarantees. A future persistence slice must
design crash-safe state/receipt commits together. Initial global snapshot validity
remains; partial evaluation of structurally invalid graphs is not added implicitly.

## Questions for ChatGPT / owner

1. Accept initial memory-only retention until explicit deletion/shutdown, with quotas
   and no disconnect-triggered data loss? Recommended; persistence remains separate.
2. Accept one revision per successful nonempty atomic batch, with incomplete formulas
   saved as diagnostics and only new formulas bound against the final batch directory?
3. Accept publication requiring the head to remain unchanged through its final commit,
   while ordinary captured evaluations may finish on an older revision?
4. Accept instance-scoped operation receipts retained for the resource lifetime,
   rejecting new mutations when receipt capacity is exhausted instead of silent eviction?

## Next recommended slice

Review ADR-0006 and the proposed operations before implementation. Preserve separate
backlog items for Mac REPL validation, C# IPC/Native AOT, batch mode, expression REPL,
incremental calculation, workbook projection, and Avalonia. No subsequent slice started.
