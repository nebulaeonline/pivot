# ADR-0023 feasibility accepted — production-readiness design gate

Date: 2026-10-23. **Stage 4 accepted for its tested scope; the four-stage disk-history
feasibility investigation is complete and closed. ADR-0023 production implementation
remains open and gated. Only focused architecture design is authorized next.**
[Stage 4 evidence](2026-10-23-history-association-stage4.md),
[native APFS evidence review](2026-10-23-macos-history-association-validation.md),
[ADR-0023](../adr/0023-document-transaction-history.md),
[completed experiment plan](../architecture/disk-history-feasibility-plan.md).

[Subsequent production-readiness design packet](2026-10-23-history-production-readiness-design.md)
is now prepared for review; the acceptance receipt below records the earlier
authorization, not approval of the new proposals.

## Accepted outcome

The parent accepts hot-root activation and Stages 1–4 as demonstrating a viable
tiered, disk-backed history direction within the reported conditions. Stage 4 adds
207 association cases across Windows NTFS, native Linux and macOS APFS, 2,652
original journal/oracle hashes, strict Rust validation and source/evidence reconciliation.
Earlier accepted findings and all original evidence qualifications remain unchanged.

History/cursor authority, actual WB5 installation, immutable prepared-save tickets,
recovered unsaved work and fresh session identity remain distinct. Accept the tested
non-destructive recovery decline, verified attachment, copy/replacement isolation,
Save As reassociation, exclusive writers, Clear epoch switching, retained/session-only
policies and interrupted operations. These are research results, not settled product
copy/retention/packaging policies.

Checkpoint-plus-canonical-record-deltas remains the leading logical representation.
No production container, default checkpoint interval, scratch policy or cache limit
is approved. Stop broad feasibility experimentation here; the remaining work concerns
specific production engineering contracts.

## First mandatory design gate: historical native Save

A new WB5 export of historical B can reject after later D-only allocations advance
the lifetime ledger. An earlier prepared B export completing while live A is correct,
but does not solve initiating a new Save from any legitimate historical state.
This is a production integration blocker, not merely a negative-test detail.

Compare separation of current content from lifetime reservations with a narrow,
validated export projection or another explicitly justified approach. Preserve exact
definitions, bindings and identity authority. Do not lower allocation floors, reuse
abandoned IDs, fabricate active nodes, weaken portable-transfer validation or silently
use the cold-history codec for native Save. Preserve WB5-only policy; a demonstrated
representation incompatibility requires a separately approved format decision.

Saving an undone state changes only its verified saved-file baseline; it must not
implicitly destroy Redo. Saved ContentStateId identifies exactly the exported content,
independent of fresh mutation revision and lifetime allocation authority. Return a
concrete recommendation and adversarial test plan before implementing this correction.

## Next deliverable: one bounded production-readiness architecture packet

1. Historical-state native Save/export design, identity invariants, alternatives and
   adversarial test plan. Prioritize this before session/UI integration.
2. Durable, bounded-memory transaction-ID indexing and efficient verified startup.
   Linear history scans cannot become normal edit admission or reconciliation for
   effectively unlimited history.
3. Mutation-local authoritative-record construction. Full export and repeated research
   reconstruction must not become assumed per-edit production costs; retain separate
   export, validation, construction, write, sync, reconciliation and acknowledgment
   measurements.
4. Resource and storage contracts requiring explicit approval: bounded cache and
   scratch admission, measured checkpoint/cold-retrieval scheduling, filesystem sync
   and uncertainty rules, valid-suffix loss/acknowledgment anchors, external file
   operations, reclamation/compaction, retention and user-visible deletion semantics.
   Logical Clear is neither demonstrated physical reclamation nor secure erasure.
5. A staged implementation plan with independent stop gates and final three-platform
   acceptance requirements. No implementation authorization is inferred from design
   authorization.

Effectively unlimited transactions do not remove finite document-lifetime identity
ceilings. Keep that separate scaling finding; no recycling, rebasing or capacity
increase is authorized. The former 32-transition/16 MiB research settings are not
product retention limits.

## Remaining boundaries

The evidence does not certify physical power-loss durability, network filesystem
safety, general cross-filesystem/copy relocation, arbitrary hostile rewriting, full
external filesystem-race coverage or production interactive latency. Physical identity
tokens are local observations, not cryptographic ownership, and may be reused.
Archived epochs and tombstoned bytes remain on disk in the research implementation.
Preserve these facts and the trusted-anchor/suffix-loss qualification without
reinterpreting the accepted evidence.

Do not implement the production coordinator, desktop Undo/Redo, protocol operations,
production container, WB5 migration or the historical-Save correction during this
design phase. No Optimize Worksheet, JIT/SIMD, identity recycling or unrelated
reliability remediation. Preserve all research manifests, failed attempts and evidence.
This receipt closes feasibility and records the next design authorization; the
production-readiness packet has not yet been prepared.
