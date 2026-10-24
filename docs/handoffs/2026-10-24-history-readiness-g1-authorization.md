# ADR-0023 readiness review — G0 approved, G1 authorized

Date: 2026-10-24. **Architecture direction approved with clarifications. Implement
only G1, historical-state native WB5 Save projection. G2–G7 remain deferred.**
[Architecture packet](../architecture/history-production-readiness.md),
[completed feasibility acceptance](2026-10-23-disk-history-final-feasibility-review.md),
[ADR-0023](../adr/0023-document-transaction-history.md).

## Approved G1 scope

Build a narrow, trusted, exhaustive projection of the active historical workbook
root into existing WB5 wire slots. Preserve genuine current-root UUIDs, definitions,
bindings, retired records, exact typed content, diagnostic provenance and supported
range/selection/document metadata. Compact slot numbers are serialization coordinates,
not rebased runtime NodeIds or replacement durable IDs.

Map every authoritative target consistently: scalar bindings, range endpoints,
selection members, aggregates, retired targets and any supported port/selection
metadata. Never omit a required identity, infer a missing definition from newer D,
create numeric placeholders, lower lifetime floors, reparse/rebind formulas or write
the cold-history codec as WB5. A genuine retired record differs from a future-only
reservation. Keep generic full-ledger transfer strict and WB5-only policy unchanged.
If a legitimate state is not representable, stop for explicit format review.

Preparing or completing Save/Save As must preserve B's ContentStateId, current live
Expected authority, original runtime IDs, complete lifetime ledger, cursor, Redo and
independent calculation captures. No document-history transaction is created. Verified
installation updates only the appropriate saved-file/checkpoint authority. Standalone
WB5 Open is independent; verified original history restores the full original ledger.

## Mandatory tests and promotion gate

Prove A → B → D-only allocations → Undo B → **new** Save B through actual WB5
encode/install/Open, with D still Redo. The existing reader must reconstruct equivalent
durable identity relationships, not merely accept encoded bytes.

Cover sparse ordinals, retired versus future-only IDs, moved targets, scalar/aggregate
bindings, Blank versus zero, exact Decimal96 parts, Unicode, rejected/cyclic/invalid
drafts, source/provenance and deterministic mapping. Include cold eviction and a real
fresh-process mapping before Save. Preserve the old full-ledger rejection as a separate
negative regression; original Stage 4 failure artifacts remain historical evidence.

Exercise cancellation, stale Expected, admission/codec failure, missing/duplicate/
foreign/inconsistent mappings and late installation uncertainty. Failures cannot
change live roots, allocations, cursor/Redo, existing file or saved receipts. Test
prepared B surviving edits/Undo/Clear/final install, and Save B → Redo D → Undo B:
cleanliness must use exact content and verified file authority. Retain independent
capture and zero-charge cleanup proofs.

Start with the internal Rust correction and focused semantic/admission/cancellation/
native round-trip tests. **Before ordinary native Save promotion**, run applicable
managed/session, desktop Save/Open and actual Native AOT gates on Windows, native
Linux and native macOS ARM64/APFS, with matched inputs and zero-charge cleanup.
Platform installation uncertainty remains visible; no safeguards may be weakened.
Stop for parent implementation review after G1; do not advance automatically to G2.

## Approved direction, not later implementation

P1–P9 are approved as architectural direction and review contracts. G2's mutation-local
producer needs separate approval after G1. Keep the complete-state differential oracle
and cover semantic dependencies/selections/retirement, not just touched positions.

G3 may evaluate SQLite first only after a future authorization. No SQLite dependency,
container adoption, custom indexed log, multiple backends or provider framework is
selected or authorized now. Atomic indexed receipt/root/cursor/ledger publication,
bounded-memory lookup and efficient verified startup remain requirements. A terminal
negative outcome must fence a delayed original operation; partially verified index
absence cannot establish non-commit.

Witness advancement belongs to acknowledgment eligibility, not proof of delivered ACK.
A store ahead of its witness may contain committed unacknowledged work and cannot be
silently truncated. A witness ahead blocks attachment/mutation. Joint rollback remains
undetectable without another surviving trusted anchor. Placement, trust and precise
filesystem guarantees are deferred to G4. Numeric resource policies remain deferred
with G5; no production defaults were approved.

G6/G7 session/protocol/desktop history integration is not authorized. No WB migration,
quota increase, identity recycling, JIT/SIMD, Optimize Worksheet or unrelated reliability
work. Compilation and history deletion remain independent future capabilities;
logical Clear, physical reclamation and secure erasure remain distinct.

The four-stage feasibility program stays accepted/closed and its original manifests
and limitations remain intact. This receipt records G1 authorization. Implementation status and evidence are
tracked separately in the [G1 implementation packet](2026-10-24-historical-native-save-g1.md);
this authorization is not an implementation acceptance.
