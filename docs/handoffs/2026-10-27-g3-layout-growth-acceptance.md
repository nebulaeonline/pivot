# G3 layout and growth accepted; hierarchical indexing research authorized

Parent disposition, 2026-10-27: accept the page-layout and authenticated-growth
research within its tested scope. G3 remains active; production readiness and
permanent index scalability are not established.

## Accepted findings

Use ordinary SQLite rowid payload storage as the next experimental baseline,
retaining indexed, NOT NULL authoritative content hashes. The 10K retained history
occupies about 13.6 MB without pruning or loss of logical completeness. Explicit
integer-local keys and a partial edit-location index offered no material benefit.

Atomic prefix splitting is an accepted bounded correctness proof: receipts survive,
older SQLite reader snapshots remain consistent, and incomplete coverage fails
closed. Its 1,024-leaf ceiling and linear directory validation/rewrite remain
limitations. The 16,384-control probe is not a genuine G2 edit qualification.

The broader metadata objective remains unmet: about 5.7 MB beyond 7.93 MB compressed
authoritative payload, or 71–72%. Receipt bodies alone occupy about 1.69 MB, so
index tuning alone cannot meet the 20% target. Warm-cache Python measurements do
not establish a new production latency budget.

[Accepted result and native evidence](2026-10-27-g3-layout-growth-results.md) ·
[Experimental contract](../architecture/history-layout-growth-experiment.md).
Preserve all evidence, failed attempts and experimental bounds unchanged.

## Next bounded assignment: hierarchical authentication and receipt normalization

Design and experimentally implement a page-bounded authenticated directory whose
publication work depends on affected pages and tree depth, rather than total bucket
count. Preserve authenticated presence/absence, complete coverage, atomic publication,
stale-reader isolation and fail-closed reconciliation. Exploit SQLite transactional
pages and MVCC; do not retain a new immutable tree path per transaction and recreate
the original storage amplification.

Measure updated pages, logical bytes rewritten, proof size, page-cache demand,
verified startup, lookups, splits and publication latency. Distinguish logical
amplification from SQLite/WAL physical writes and retained physical space.

Inventory canonical receipt fields and evaluate lossless body/reference normalization.
Preserve exact transaction identity, intent binding, terminal-negative outcomes,
replay receipts, integrity commitments and historical auditability. Measure logical
metadata and allocated SQLite pages against compressed authoritative payload as the
primary denominator. Do not omit authoritative information to meet a percentage.

Retain binary G2 records, zlib level 6, checkpoints, complete lifetime authority
and receipt semantics. No codec selection: `J:\coding\dotZstd` and Rust zstd remain
future options.

## Qualification and stopping boundary

Use native Windows x64 NTFS, Ubuntu x64 ext4 and Mac ARM64 APFS. Repeat genuine
1K/10K G2 workloads and independent reconstruction oracles. Cover split faults,
crash/reopen, stale readers, missing/corrupt index pages, invalid coverage, admission
refusal and complete-audit agreement. Report synthetic index growth separately.

Return for parent review if bounded incremental updates cannot be achieved,
authentication requires routine full-history scans, normalization loses authority,
or savings require weaker integrity. No full 100K G2 workload until directory growth
and its resource costs are demonstrated and understood.

Deliver a comparative evidence packet, remaining limitations and a recommendation
on readiness for the next G3 feasibility gate. This assignment is authorized;
implementation has not started as part of this documentation reconciliation.

No production SQLite integration, codec selection, format migration, witness,
retention/GC, coordinator, Undo/Redo, protocol changes, identity recycling or
product-limit increases. G1/G2 and six-RID GUI/AOT acceptance remain closed;
G4–G7 remain gated. Dev Drive remains shelved; clipboard ISSUE-0001 remains separate.
