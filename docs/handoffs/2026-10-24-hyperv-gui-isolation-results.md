# Hyper-V x64 GUI-isolation qualification — owner results

Subsequent decision: [observational evidence accepted for bounded activation design](2026-10-25-hyperv-isolation-acceptance.md).
The pending-review statements below describe the original submission; operational
unattended GUI authority remains unapproved.

2026-10-24. Qualification evidence is ready for separate per-guest parent review.
**Neither guest has GUI-isolation approval yet.** No unattended permit, positive
managed/AOT run or orchestration has been enabled. G3 remains held.

## Owner observations and scope

Both VMs use VMConnect Basic Session, with Enhanced Session unchecked. The owner
reported no other active connections, and “none” when asked about clipboard/history
synchronization, additional shares beyond J: or /mnt/j, or redirected printers,
USB devices and smart cards. These are owner observations, not an independent
configuration audit. Existing enabled host Enhanced Session capability/integration
services are not treated as proof of active redirection.

The owner explicitly authorized replacing host/guest clipboards during the current
manual test window, received the [five-step bidirectional procedure](2026-10-24-hyperv-clipboard-check.md),
then reported: “the clipboards are 100% isolated from each other in line with the
tests.” In context this reports success for both VMs: local guest B copy/paste,
host retention of A after guest copy, local host C copy/paste and guest retention
of B after host copy. No unexpected propagation or failed attempt was reported.
The claim is bounded to the observed tests, not universal/permanent isolation.

| Guest | Connection | Guest-to-host marker separation | Host-to-guest marker separation | GUI approval |
| --- | --- | --- | --- | --- |
| Pivot-Win11-x64 | Owner-confirmed Basic Session | Owner-reported pass | Owner-reported pass | Pending parent review |
| Pivot-Ubuntu-x64 | Owner-confirmed Basic Session | Owner-reported pass | Owner-reported pass | Pending parent review |

No exact sentinel execution timestamp, screenshots, independent observer or machine
trace was supplied. This record preserves the chat attestation; it does not invent
those details. No reboot/session/configuration change was reported in response to
instructions to report such changes first. That is not independent verification
of identity continuity at the exact test time.

## Identity references and evidence limits

- Windows VM ID `f89ff908-16dc-44c4-af54-008967e48443`; captured hostname
  WINDEV2407EVAL, user windev2407eval\user, Console session 3, boot
  2026-10-24 15:00:16 -04:00.
- Ubuntu VM ID `9ce1b716-aa20-4b16-a206-25126fab46d5`; captured hostname
  Pivot-Ubuntu-x64, user robhorvath, boot
  `616f92de-4c4d-4a06-8937-3f64765a039d`, GNOME/Wayland environment.
  Scoped correlation could not resolve a logind session/seat; both remain Unknown.

[Session observations, original reports and correlation hash verification](2026-10-24-hyperv-gui-session-review.md)
retain the initial origin-manifest limitations. The follow-up correlation has a
verified native-generated hash. Neither desktop capture proves Avalonia's eventual
backend. Parent [partial acceptance](2026-10-24-hyperv-gui-session-acceptance.md)
allows explicit Unknown rather than invented session identities.

The deliberate guest J: /mnt/j mapping remains a source/evidence exchange only.
Native builds, persistence fixtures and original evidence must stay on guest
NTFS/ext4, independently of the clipboard observations.

## Review request and stop

Review each guest independently: are these owner-attested Basic Session/resource/
bidirectional sentinel observations sufficient for the next bounded activation
design, with the disclosed Ubuntu session and exact-time identity limitations?
Record any required additional evidence explicitly rather than treating it as an
unreported pass. Do not infer permanent approval from a single quiet test window.

No implementation or execution of unattended GUI permits is included. Positive
managed desktop and Native AOT publish-and-execute remain later gates requiring
accepted isolation, existing assertions, nested authorization, native storage,
host clipboard separation and zero-charge cleanup. Relevant connection/resource/
session/boot changes require re-evaluation. No host fallback or generic dispatcher.

Both full headless CI gates stay accepted/closed without reruns. UTM ARM64 remains
separate. G1/G2 stay closed; G3 held; G4–G7 and production Undo/Redo deferred.
