# Hyper-V x64 observed isolation — accepted for activation design

Recorded 2026-10-25 from the owner-supplied parent review (decision text dated
2026-10-24), attachment `216d92ea-dbde-49d5-8afb-c0a2d59e4761/Pasted text.txt`.

**Observed isolation evidence is accepted for both x64 guests. Operational
unattended GUI execution remains unapproved and disabled.**
[Evidence packet](2026-10-24-hyperv-gui-isolation-results.md).

| Guest | Observational isolation evidence | Unattended GUI authority |
| --- | --- | --- |
| Pivot-Win11-x64 | Accepted for bounded activation design | Not approved |
| Pivot-Ubuntu-x64 | Accepted for bounded activation design | Not approved |

Accept owner-confirmed Basic Session, absence of other reported graphical clients
and extra resource/clipboard synchronization, and bidirectional harmless-marker
passes. Preserve the owner-attested scope and missing exact timestamps, independent
settings audit and exact test-time boot/session continuity. Do not repeat successful
manual checks merely to improve presentation. These are observations during a
controlled window, not universal or perpetual isolation guarantees.

Ubuntu correlation is complete. Logind session/seat remain Unknown without blocking
design. Future activation must identify the actual guest-owned GUI process/session
and runtime/backend; a historical logind ID or Wayland environment hint is insufficient.

## Next authorized deliverable

Prepare a bounded activation proposal and controlled implementation plan, starting
with Windows x64, then applying the reviewed mechanism to Ubuntu x64. Define an
individual profile bound to exact host/Hyper-V VM, Basic Session configuration,
current guest boot/graphical session, reviewed integration/resource settings,
owner clipboard evidence, explicit test category and executable entry point,
limited validity and configuration-change invalidation.

Current authority must fail closed when required evidence cannot be established.
No hostname, environment variable, static JSON receipt or historical sentinel pass
may independently unlock execution. Preserve the live local consent broker;
unattended guest authorization must be a separately reviewed capability. This
record adds no permit or implementation bypass.

Clipboard separation is distinct from filesystem isolation. Keep source execution,
build outputs, caches, temporary files, persistence fixtures and original evidence
on guest-local NTFS/ext4. J: and /mnt/j remain deliberate SMB/CIFS source/evidence
exchange only, never GUI scratch space or unrestricted automation targets. Do not
disconnect, change or expand the shares without separate approval.

After separate activation-design and per-guest profile review, a bounded positive
managed desktop run and later Native AOT publish-and-execute require explicit
authorization. Retain real application/nested-peer execution, source-matched native
builds, host clipboard separation, existing assertions, original failures and
zero-charge cleanup. No host-desktop fallback.

Headless qualification/full CI remain closed without reruns. No generic dispatcher,
VM lifecycle automation, provisioning or final --vm CLI. UTM ARM64 remains separate.
G1/G2 remain closed; G3 held; G4–G7 and production Undo/Redo deferred. No product,
graph, workbook, persistence, protocol or limit changes are authorized.
