# G3 first result accepted; bounded verified-open research authorized

Parent disposition, 2026-10-27: accept the first bounded SQLite-first research
result on NTFS, ext4 and APFS. G3 remains active and incomplete. This acceptance
does not select a production backend or authorize G4–G7 or production Undo/Redo.

## Accepted baseline

Preserve `artifacts/g3-index/g3-index-20261027-c/` and the earlier development
failures. Each of the three targets processed 1,000 actual G2 transactions,
passed nine G2 regressions and fifteen store tests, and retained independent
checkpoint-oracle validation. Atomic indexed decisions, exact-intent receipt
replay, conflicting-intent refusal, terminal negatives, lifetime/cursor references
and selected fault/restart mechanics are accepted as research evidence.

Measurements retain their single-run limitations: no cross-platform performance
ranking, power-loss guarantee or production memory bound. The complete reopen
audit is explicitly unsuitable for normal bounded startup; fast lookup after that
audit does not establish bounded reconciliation.

## Next bounded assignment

Use the existing feature-only SQLite fixture to separate two explicit operations:

1. Normal verified open: validate current authority, required indexed structure
   and bounded recovery-tail work without scanning/reconstructing all history.
2. Full history audit: explicitly traverse retained decisions, records and payloads
   for maintenance/investigation. Validate old state lazily before activation.

Provide a defensible completeness argument based on SQLite's atomic publication
and stated integrity assumptions. Preserve the joint authority of immutable
content/state, transaction identity/intent, decisions/receipt indexes, cursor/head/
Redo, complete lifetime allocation and current publication/recovery metadata.
A high-water counter, row count or unchecked cached marker is not completeness
proof. Missing required structures, corrupt current roots, ambiguous decisions
and incomplete recovery remain Unknown/Blocked. Preserve terminal negative
reconciliation under writer exclusion so a delayed original request cannot commit.

If the fast-open argument cannot satisfy the reviewed contract, report that
limitation rather than weaken negative lookup or build a second backend.

## Required evidence

Cover payload/index/root disagreement, missing receipt/state rows, interrupted
publication, stale horizons, duplicate/conflicting TransactionIds, lost ACKs,
ambiguous reconciliation, terminal negatives followed by delayed edits, incomplete
recovery tails and corrupt old history. Distinguish corruption detected at bounded
open from corruption detected only on lazy load/full audit; comprehensive latent
media detection without historical reads is not promised.

Record exact engine/source/compile options, isolation, VFS assumptions and relevant
PRAGMAs. Keep synchronous FULL as baseline; investigate macOS fullfsync explicitly.
Measure WAL checkpoints, writer backpressure and long readers. No global setup,
system SDK/database replacement or production dependencies are authorized.

After establishing the bounded-open contract, compare 1K with 10K: open versus
verified activation, positive/negative lookup, tail work/page access, decode work,
cache/RSS/domain/scratch memory, DB/WAL/filesystem allocation and commit/ACK/sync
costs. Keep producer/oracle work separate and retain a full-audit comparison.
**Do not begin 100K scale claims before the 10K path demonstrates bounded recovery
and sensible resource behavior.**

## Boundaries and next review

Keep G1 ordinary WB5 Save and G2 unchanged. No canonical-core SQLite dependency,
provider framework, second backend, workbook-format migration, production history
coordinator, identity recycling, capacity increase, UI/protocol or hosted persistence
changes. No six-RID GUI/AOT reruns for this research. Clipboard ISSUE-0001 stays
separate; G4–G7 remain gated.

Return the bounded-open design, explicit failure cases, reproducible non-GUI
evidence and measurements for parent review. This reconciliation records the
authorization only: the next slice has not started and no completed gates reran.
