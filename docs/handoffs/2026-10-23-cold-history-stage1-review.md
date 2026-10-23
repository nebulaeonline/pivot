# ADR-0023 Stage 1 accepted — representation comparison authorized

Date: 2026-10-23. **Cold-restoration semantic feasibility accepted. Continue to
Stage 2 under the existing bounded experiment authorization. Production Undo/Redo
not authorized.**
[Accepted implementation and evidence](2026-10-23-cold-history-restoration-stage1.md),
[ADR-0023](../adr/0023-document-transaction-history.md),
[experiment plan](../architecture/disk-history-feasibility-plan.md).

## Accepted foundation

The parent accepts Windows/Linux reconstruction of canonical and workbook roots
without authoring replay, reparsing, name resolution or WB5 Open. Same-session cold
activation preserves runtime NodeIds; restart preserves durable identity/binding
equivalence through one fresh runtime mapping; independent copies remain isolated.
Exact typed content, bound references, invalid/cyclic drafts, diagnostics/provenance,
ContentStateId, fresh authority and independent captures are preserved in the tests.
ColdLedger retains discarded-branch reservations and rejects incomplete/stale ledgers.
Portable transfer and core NodeId representation remain unchanged.

Windows 346/377 and Linux 347/378 default/all-feature tests, strict Clippy, formatting
and the tested atomic failure/ownership cases are sufficient for this semantic stage.
The evidence qualifications in the implementation packet remain intact. Native
macOS/APFS is not required for this acceptance; it is required before later
cross-platform filesystem durability claims. This accepts neither a durable journal
nor a finished Undo/Redo system. Prior manifests/evidence remain unchanged.

## Resource-accounting qualification

The 16 MiB research image bound and 8 MiB ledger reservation are prototype settings,
not approved production cache/staging defaults. Approximately 94.1 MB peak domain
charge for 5,483–6,411-byte small-fixture capsules includes loose reservations;
it is neither graph allocation size, RSS nor an efficiency endorsement.

Stage 2 must distinguish actual serialized bytes, transient working buffers,
required versus reserved scratch, reconstructed persistent allocations and peak
simultaneous ownership. Demonstrate admission scaling with bounded actual work.
Measure small, medium, formula-heavy and near-capacity cases. Do not hide unnecessary
allocations behind large reservations or raise workbook/runtime/cleanup limits.

## Authorized Stage 2 comparison

Compare independently loadable complete checkpoints, checkpoints plus canonical
record deltas, and shared immutable objects with root/commit journaling. The complete
checkpoint representation is the semantic baseline; its reservation policy is not
the target resource policy. Any journal mechanics used here do not establish the
later interrupted-commit/recovery gate.

Reuse independent typed-content/CellView, logical identity, bound-target, diagnostic
ordering, provenance, invalidation and ledger comparisons. Check same-session runtime
identity and restart mapping behavior too. Equal numeric results alone are insufficient:
different referenced objects may currently have the same value.

Measure storage amplification, serialization/reconstruction work, bounded chain
depth, index/cache growth and actual resource ownership over 1,000/10,000 transactions.
Avoid a catalog or replay chain whose resident size/work grows linearly with history.
Long fixed-identity histories do not expand lifetime identity allocation capacity.

## Remaining boundary

Durable transaction/cursor atomicity, interrupted-write reconciliation, crash
recovery, verified save-file association, recovery decline and storage exhaustion
remain later gates. No production container/database, protocol/desktop history,
WB5 change, identity recycling, JIT/SIMD or history-compaction implementation is
authorized. Existing reliability issues and experimental limits remain open.

Stop for parent review if later work reveals identity/binding mismatch, unbounded
restoration, broad core redesign or a need to increase hard limits. This review
receipt records authorization only; Stage 2 is not started or proven by it.
