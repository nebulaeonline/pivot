# ADR-0023 Stage 3 accepted — file/history association research authorized

Date: 2026-10-23. **Stage 3 accepted as Windows/Linux process-crash consistency
research. Stage 4 association and retention feasibility authorized; production
Undo/Redo remains deferred.**
[Accepted implementation and evidence](2026-10-23-history-durability-stage3.md),
[ADR-0023](../adr/0023-document-transaction-history.md),
[experiment plan](../architecture/disk-history-feasibility-plan.md).

## Accepted findings

The parent accepts the 366 scenarios, persisted prepare/payload/decision ordering,
synchronization before acknowledgment, coherent branch/cursor decisions and exact
lifetime identity-ledger recovery under the tested process interruptions. Definitely
uncommitted edits preserve the prior timeline and Redo. Complete decisions recover
without authoring replay even when no acknowledgment was received. An uncertain
TransactionId blocks mutation until reconciliation; failed reconciliation retains
the barrier. Independently reopened acknowledged operations recover their exact state.

This satisfies the bounded Stage 3 process-crash gate, not physical power-loss
durability. SIGKILL/TerminateProcess leave the OS running; injected sync failures
are not hardware flush failures. The chain cannot independently detect removal of
an entire valid suffix without a trusted acknowledgment anchor. Windows namespace
durability, physical storage ordering, native macOS/APFS and network filesystems
remain unverified. No production durability promise is accepted.

Full-state export, repeated candidate reconstruction and linear disk-history lookup
remain research costs. Stage 4 may reuse them to isolate association correctness.
Production requires mutation-local record construction, bounded-memory indexed
transaction lookup and efficient verified startup. Keep export, validation, record
construction, writes, synchronization, reconciliation and acknowledgment separately
measurable. No interval, threshold, memory policy or single-file layout is approved
as a production default.

## Authorized Stage 4 gate

Use feature-only persistence and the actual native workbook adapter, with these
independent authorities and failure cases:

- Match saved ContentStateId, checkpoint, actual workbook file observation and
  history-instance authority. Path or DocumentId alone cannot authorize attachment.
- Preserve ordinary valid WB5 Open when history is missing, incompatible or corrupt.
  Newer unsaved journal work requires explicit recovery; ordinary Open must not
  silently replace saved A with unsaved D.
- Decline recovery without deleting D, then restart and deliberately recover the
  original timeline and complete ledger.
- Exercise Save As, independent copies, moves, renames, external replacement and
  two-instance writer exclusion without inventing attachment authority.
- Complete a prepared Save of historical B while live content is A; preserve both
  identities and the outstanding valid ticket. Journal receipts cannot make a file
  clean without verified installation/checkpoint/content agreement.
- Interrupt verified workbook installation before the history save receipt persists;
  reconcile fail-closed and retain existing installation-uncertainty safeguards.
- Exercise explicit retained/session-only policies, Clear History, relocation,
  unavailable storage and failed/cancelled lifecycle operations. Clear preserves
  live content, saved-file authority, valid save tickets and committed identity floors.

No silent abandonment of recoverable work or test-only success flags. Report actual
platform/filesystem coverage; native macOS/APFS remains required before cross-platform
durability or association claims. The research journal is a reusable substrate for
this experiment, not a production container decision.

## Stop boundary

Stop after Stage 4 for parent review of the complete feasibility findings before
proposing production integration. Do not select a database, container, sidecar or
packaging policy merely because the prototype works. No production coordinator,
RPC/desktop Undo/Redo, private protocol changes, WB5 changes, compatibility adapters,
capacity increases, identity recycling, compaction, Optimize Worksheet, JIT or SIMD.
Preserve all Stage 1–3 manifests, failed attempts and evidence. Existing reliability
issues remain open. This receipt records authorization; Stage 4 has not started.
