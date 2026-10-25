# Windows boot identity — bounded investigation authorized

2026-10-25 parent review, attachment
`4f8eedd3-9b6f-4548-8bd3-8c553624564b/Pasted text.txt`.
**Timestamp refusal accepted; read-only investigation authorized. No further
positive managed GUI attempt is authorized.**
[Original confirmation refusal](2026-10-25-windows-confirmation-timestamp-block.md).

Preserve Blocked/2: the observed LastBootUpTime string differs by 1.179 ms while
the qualified guest, session 1 and WinSta0/Default are unchanged. This proves
neither reboot nor the cause of the fractional discrepancy. The guard refused as
programmed; its stable-identity definition needs correction. The UI runner never
started, so neither application behavior nor the peer-backend fix is implicated.

## Investigation and proposed contract

Investigate documented Windows interfaces for a stable OS-provided boot identifier.
Evaluate repeated-read stability within one boot and documented across-boot
behavior. No deliberate reboot is needed or authorized for initial investigation.
Do not equate formatted/truncated timestamps with identity. If no suitable stable
identifier is available, propose an explicit independently corroborated continuity
comparison with disclosed uncertainty; rounding to seconds alone is insufficient.

Separate three claims: measured boot identity; current user/session/window-station/
input-desktop identity; owner-attested host Basic Session/resources/VM binding.
The latter cannot be independently proven from guest-local files. Hostname,
firmware UUID, environment or owner receipt alone do not prove current boot.

A supported correction must refuse genuine reboot, changed user/session/VM binding,
unavailable input desktop, malformed or missing measurements. Test repeated reads,
fractional observation variation, meaningful boot changes and those negative cases
using real adapters where available plus synthetic coverage. Do not merely allow
arbitrary timestamps when session numbers match. Existing consent remains unchanged.

Return the chosen contract, source diff, non-GUI evidence and warning-free build
for review. If implemented, derive a new immutable candidate from the previously
approved packet, retaining the peer-backend correction and functional assertions;
change only supported identity logic and needed regressions. Reconcile all manifests
and packet hashes. Do not edit the original candidate or issue another positive
owner command before separate authorization.

## Independent display observation and boundaries

The owner reports intermittent Windows guest blank screens. Record this separately
as unresolved display reliability, not demonstrated reboot or cause of timestamp
drift. Do not change power/display/VMConnect/integration settings, reboot, inspect
clipboard or launch Avalonia as part of this investigation. No repeated sentinel
for fractional timestamp variation alone; reassess only after an actual relevant
change or contrary isolation evidence.

The first managed functional run remains accepted with overall Blocked/2 for peer
diagnostics; the later confirmation is separately Blocked/2 at identity preflight.
Neither result is overwritten or replayed. Headless gates/G1/G2 remain closed;
G3 held; G4–G7/production Undo/Redo deferred. Native AOT/Ubuntu GUI stay separately
gated. No unattended initiation, second protocol, dispatcher, lifecycle automation,
host fallback or product changes.
