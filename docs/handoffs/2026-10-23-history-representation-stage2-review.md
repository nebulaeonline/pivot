# ADR-0023 Stage 2 accepted — durability research authorized

Date: 2026-10-23. **Stage 2 representation feasibility accepted. Proceed to Stage 3
research with checkpoint-plus-canonical-record-deltas; production format/container
and Undo/Redo integration remain undecided.**
[Accepted implementation and measurements](2026-10-23-history-representation-stage2.md),
[ADR-0023](../adr/0023-document-transaction-history.md),
[experiment plan](../architecture/disk-history-feasibility-plan.md).

## Accepted findings and qualifications

The parent accepts the Windows/Linux semantic comparisons, fixed/adaptive checkpoint
experiments, work-scaled admission and submitted evidence: 100 successful research
runs, 792 verified original sample hashes, passing Rust regressions and zero remaining
domain charge. Constant cache counts/capacities between 1,000 and 10,000 transactions
support bounded resident research; they do not certify complete production memory
bounds or exact heap usage. Encoded bytes, vector capacities, reserved scratch,
distinct domain allocations, reader/index caches and simultaneous reconstruction
ownership must remain separately reported from physical RSS and filesystem metadata.

Checkpoint-plus-canonical-record-deltas is the preferred Stage 3 candidate. Complete
checkpoints remain the independently loadable correctness baseline and fallback.
The directory-of-objects prototype is not preferred; this does not exclude a future
packed immutable-object design. Preserve effectively unlimited logical history as
the target, constrained by real storage and explicit user policy.

Interval 16 is approved only as a conservative Stage 3 experiment setting. Retain
interval 64 and selected adaptive comparisons where they illuminate recovery or
retrieval bounds. Neither interval, 64 KiB nor 256 records is a production default.
Broad-edit checkpoint amplification remains explicit. Future work bounds must cover
decoder admission, index reconstruction and all record operations, not just delta
payload bytes or counts.

Full authoritative export/traversal per changed transaction remains a performance
limitation. Stage 3 may reuse that encoder to isolate durability correctness, but
must retain a distinct transaction-record construction boundary for a later,
separately reviewed mutation-local encoder. Report full-export cost independently
of delta construction, writes, synchronization and acknowledgment. Reduced disk
bytes do not demonstrate incremental serialization performance.

## Authorized Stage 3 gate

Implement the feature-only durable transaction-decision and failure/restart experiment.
Prove a coherent persisted relationship between exact canonical state, TransactionId,
ContentStateId, committed head/cursor/Redo branch and the complete lifetime ledger,
including what the client was told had committed.

- A durable but unacknowledged operation recovers its exact state and ledger without
  replay or identity reuse.
- An uncertain write/flush blocks further mutations until the exact TransactionId
  is reconciled as committed or definitely uncommitted.
- A definitely uncommitted replacement after Undo preserves the prior timeline and
  Redo; only a durable replacement decision can remove that future.
- Interrupted cursor movement recovers one coherent cursor and corresponding state,
  never a mixture of generations.

Exercise partial payloads, missing referenced data, interrupted decision/metadata
writes, ambiguous flushes, interrupted branch/cursor changes, ledger advancement,
corruption and storage exhaustion. Check actual payload-before-decision ordering and
filesystem synchronization, including directory metadata where applicable. A valid
decision checksum alone cannot prove that its referenced payload is durable.
Use real persisted recovery and fresh child processes, not test-only commit booleans
or a surviving parent's catalog. Separate process-crash evidence from power-loss
claims and report filesystem limitations explicitly.

## Stop boundary

Stop for parent review after Stage 3 evidence, before Stage 4 or production integration.
Stage 4 separately covers Save/Save As, saved-content authority, file/history
association, recovery decisions and retained-history relocation. Declining immediate
recovery must not delete recoverable unsaved history.

No production container/database/packaging choice, session RPC, desktop Undo/Redo,
production Save integration or general history implementation is authorized. Keep
WB5, CB2, private protocol v4, hosted v1, lifetime identity limits and core NodeId
semantics unchanged. No identity recycling, compaction, Optimize Worksheet, JIT or
SIMD work. Preserve prior manifests, failures and unresolved reliability issues.
This receipt records Stage 3 authorization; it does not claim Stage 3 has started
or passed. Earlier evidence/manifests remain intact.
