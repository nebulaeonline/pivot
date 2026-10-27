# G3 accounting accepted; engine/VFS and admission qualification authorized

Parent disposition, 2026-10-27: accept the physical storage accounting, scalable
admission proposal and readiness assessment within their stated limits. **G3 remains
active.** Retire 20% as a mandatory acceptance threshold; retain it only as an
optimization aspiration. Stop storage/codec optimization for now.

[Accepted assessment and evidence](2026-10-27-g3-storage-admission-assessment.md).

## Accepted findings and unchanged limits

Mac and Ubuntu reconcile exactly against all nine dbstat objects. Windows record
lengths are modeled; dbstat is unavailable. The current representation's serialized
records alone leave about 41.21% beyond compressed payload, so page packing cannot
meet 20%. The fixed-field budget is conditional, not a universal entropy lower bound.
Preserve indexed reconciliation, exact receipts and independent integrity checks.

The 4,096-leaf proposal has validated logical arithmetic and 30 native model passes.
It is not a qualified physical transaction/resource bound and remains disabled.
Keep the accepted **1,024-leaf** runtime profile unchanged. Preserve all earlier
evidence under its actual engine identity; passing tests do not establish vendor
WAL-reset patch status.

## Next assignment: engine/VFS and resource-admission qualification

**Fixed engine.** Identify and qualify a fixed research SQLite build on Windows
NTFS, Ubuntu ext4 and Mac APFS using a consistent, reviewable strategy. Record actual
version, source ID, compilation options, VFS identity and WAL-reset correction status.
Do not infer backports from version strings or substitute an unrelated library for
the connection actually tested. No production backend/package choice is authorized.

**Native instrumentation.** Instrument that actual connection. Measure native
allocation/cache usage, dirty-page and WAL-frame behavior, checkpoint progress,
synchronization calls and transaction costs where supported. Separate logical writes,
pager work, OS write counters and physical-device durability. Configured cache size,
logical node counts and process RSS are not sufficient resource-limit evidence.

**Pre-write admission.** Wire the reviewed bounded insertion planner into the
research store while retaining the 1,024-leaf profile. Admit resources before SQL
mutation. Preserve atomic rollback, stale-writer refusal, authenticated absence,
exact intent/receipts and all history. Exercise scratch exhaustion, SQLite allocation
failures, SQLITE_FULL, invalid configuration, WAL pressure, pinned readers, partial
transaction failures and crash/reopen. Report any resource that cannot reliably be
bounded/admitted; do not invent guarantees.

**Native qualification.** Use small/adversarial workloads no larger than the
already-qualified receipt count. Preserve original assertions, independent reconstruction
oracles, canonical receipts and native filesystem evidence. This is not permission to
enable 4,096 leaves or execute 32K/65K scaling points.

## Deliverable and stop

Return actual engine/VFS results, measured memory/pager/WAL/sync behavior, pre-write
admission and failure-isolation evidence, proposed experimental resources/stop conditions,
and a decision on readiness for **separate** 32,768/65,536-control authorization.
Stop for parent review after engine/VFS and small-bound admission qualification.
Neither expanded control workload is authorized by this assignment.

This document reconciles the review only; new qualification implementation has not
started. Accepted gates need not be rerun for this documentation change.

No full 100K G2 run, production backend selection/integration, codec work, format
migration, witness, retention/GC, Undo/Redo, protocol changes, identity recycling or
product-limit changes. G1/G2 and six-RID acceptance remain closed; G4–G7 stay gated.
Clipboard ISSUE-0001 remains independent; Dev Drive work remains shelved.
