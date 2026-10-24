# ADR-0023 — G2 accepted/closed; G3 held for infrastructure

2026-10-24. The parent accepts and closes the feature-only Rust G2 implementation
and its three-platform evidence. **The owner directs that the infrastructure
detour remain active. G3 has not started and must not start during this detour.**

Source: owner-supplied parent review attachment
`f12cb741-ab24-4ebf-8c96-291ed44982ed/Pasted text.txt`.
[G2 implementation](2026-10-24-mutation-local-records-g2.md),
[Mac reconciliation](2026-10-24-macos-mutation-local-records-g2-validation.md),
[active infrastructure safety milestone](2026-10-24-execution-safety.md).

## Accepted G2 scope

The parent accepts mutation-local resolved stable-key canonical/workbook records,
original-state coalescing, coupled selection/retirement/validation/provenance,
ContentStateId and lifetime reservations, guarded in-memory paired publication,
failed/no-op/stale/cancellation isolation and independent calculation captures.
The independent complete-checkpoint oracle and adversarial omission checks are
accepted for currently supported operations.

The ephemeral validated-content marker prevents same-floor private branches from
being treated as equivalent sources. It is accepted only as a feature-only
source-integrity mechanism, not persisted identity, a witness or durable authority.
The paired publication helper is not a production transaction coordinator.

Accepted evidence: Windows 347/401 and Linux/native Mac 348/402 default/all-feature
Rust tests, strict format/lint and 135 fresh-process zero-charge samples. Mac used
native ARM64/APFS with 173 inputs, 483 frozen source entries and 85 verified returned
hashes. No managed/desktop/AOT gate was required for this feature-only Rust slice.

Retain range-formula validation scans, roughly 16.8 MB invalidation reservation,
old/new lifetime-ledger peak-charge costs, three-sample timing limitations and the
distinction between domain charge and RSS. Constant sampled record emission is not
constant-time complete workbook editing. The original Linux Rust clipboard shutdown
failure and passing unchanged-source rerun remain unresolved and separate from GUI
Fill/clipboard observations. G2 acceptance proves no host-desktop safety property.

## Recorded future authorization — not active work

The parent authorizes a bounded feature-only SQLite-first indexed-store feasibility
evaluation, **not production database adoption**. The owner's later scheduling
instruction holds that work until after the infrastructure detour; this receipt
records its boundaries without beginning design, dependencies, code or experiments.

When resumed, its atomic indexed publication must bind immutable TransactionId/
intent/receipt identity, content and checkpoint/delta payload references, parent/
head/cursor/Redo relationships, complete lifetime-ledger authority and required
indexes under a verified committed horizon. Missing receipts are evidence of
noncommit only after horizon/tail verification; otherwise outcomes remain unknown.
Preserve exact-transaction reconciliation and no authoring replay.

The requested future evidence uses real native storage and 1K/10K/100K fixed-identity
transactions where admitted/practical, including scalar/formula-heavy/broad Fill/
Paste work. Separate logical/index bytes, database/WAL/physical allocation, commit/
sync/ACK/recovery distributions, positive/negative lookup, cursor/Redo/cold retrieval,
verified startup/tail work, backend/cache/decoded-root/domain memory and short/delayed
reader checkpoint behavior. Controlled exhaustion, interruption, stale readers,
corruption and independent reopen must retain their real failures and raw evidence.

Use bounded readers/backpressure, adequate test disk capacity and explicit engine/
dependency/VFS/filesystem/sync configuration. A SQLite cache setting is not Pivot's
production RAM limit. Do not impose artificial logical retention caps or conflate
logical history checkpoints with WAL checkpoints, process death with power loss,
or native filesystems with network shares.

Keep the accepted G2 producer independent of backend assumptions. No canonical-core
SQLite dependency, production dependency/container/provider selection, session/UI/
protocol integration, witness, retention/GC or custom second backend is authorized.
Return comparative findings rather than silently replacing an unsuitable candidate.
Future Windows NTFS/native Linux research requires native Mac/APFS evidence before
cross-platform feasibility acceptance. Feature-only Rust gates suffice; no unattended
host GUI testing. Stop for parent review before G4 or a production backend choice.

G1 and G2 remain closed. G3 is authorized in principle but **held, not started**.
G4–G7 and production Undo/Redo remain independently gated. Continue only the
separately approved infrastructure safety milestone for now.
