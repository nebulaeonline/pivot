# Windows boot identity — documented candidates and first measurements

2026-10-25. [Bounded investigation authorization](2026-10-25-boot-identity-investigation-direction.md).
**Research only. No identity guard change, new confirmation packet or GUI authority.**
No VM reboot, clipboard access, power/display/integration setting change or blank-
screen diagnosis is part of this work.

## Documented interfaces and limits

Microsoft documents Kernel-General event 12 as OS startup and cautions that event
IDs must be bound to their provider:
[startup-event guidance](https://learn.microsoft.com/en-us/troubleshoot/windows-server/performance/troubleshoot-unexpected-reboots-system-event-logs).
[Get-WinEvent](https://learn.microsoft.com/en-us/powershell/module/microsoft.powershell.diagnostics/get-winevent)
supports narrow provider/ID queries. The
[EventRecordID schema](https://learn.microsoft.com/en-us/windows/win32/wes/eventschema-eventrecordid-systempropertiestype-element)
defines a log-assigned record number, not a globally unique boot identifier.

[GetTickCount64](https://learn.microsoft.com/en-us/windows/win32/api/sysinfoapi/nf-sysinfoapi-gettickcount64)
reports elapsed milliseconds since startup at timer resolution (typically 10–16 ms).
[Windows Time](https://learn.microsoft.com/en-us/windows/win32/sysinfo/windows-time)
states this includes sleep/hibernation. It is an uptime observation, not an opaque
boot ID. Wall-clock adjustments and differing measurement origins must not be
silently converted into a false exact boot token.

The research did not establish a documented public contract for a per-boot GUID
through SystemBootEnvironmentInformation. The documented
[NtQuerySystemInformation](https://learn.microsoft.com/en-us/windows/win32/api/winternl/nf-winternl-ntquerysysteminformation)
page warns the API may change/unavailable and does not specify that structure's
boot-identifier contract. Do not use an undocumented information-class number or
reverse-engineered layout as if it were a supported interface. This is a bounded
research finding, not proof that no other documented boot identifier exists.

## Read-only collector and host evidence

`windows_boot_probe.py` takes six samples using a tightly filtered newest System /
Microsoft-Windows-Kernel-General / event 12 query and CIM LastBootUpTime. It records
only provider/channel/ID/record/time and named StartTime data, not unrelated events
or full messages. Native GetTickCount64 and UTC readings bracket each query. Raw
observations/errors are retained; collection never grants boot continuity or GUI
approval. Helper hashes and native report hashes are recorded before return.

An initial sandbox CIM query was Access denied; an outside-sandbox collection
succeeded on **ROBPC**, not the guest. All six samples returned the same event
record 194076, startup event data 2026-10-15T12:17:19.5000000Z and stable CIM
LastBootUpTime. Uptime increased. Wall-clock-minus-uptime differed from event
StartTime by approximately 8.533–8.540 seconds. The reason for that offset is not
established; it argues against choosing an unsupported exact reconstructed-time
comparison or a casually selected tolerance.

Host native original:
`C:\Users\Public\PivotPreflight\boot-probe-676b70f26fa14d928381bf78b8323a43`.
Returned evidence:
`artifacts/windows-boot-identity/host/boot-probe-676b70f26fa14d928381bf78b8323a43/`.
Host measurements do not establish guest stability, documented across-boot
guarantees or successful enforcement after a real reboot. No reboot was performed.

## Guest collection — no manual transcription

Inside Pivot-Win11-x64, ordinary Command Prompt:

```bat
J:\coding\pivot\artifacts\windows-boot-identity\run-windows-boot-probe.cmd
```

This copies only the collector and filesystem helper into a new native-local
directory, collects six read-only samples under C:\PivotPreflight and returns
hashed evidence to `artifacts/windows-boot-identity/Pivot-Win11-x64/`. No developer
shell, administrator access, installations, GUI launch or consent replay is needed.
Existing source destination refuses instead of overwriting. If copying fails,
retain native originals and report the error; do not replay to fix a transfer.

## Proposed contract direction — not selected or implemented

Evaluate a provider-bound exact startup-event anchor (channel, provider, record,
event timestamp and startup data), corroborated with independent uptime observations
and bounded CIM presentation variation. Separate boot continuity from exact
current user/session/window-station/input-desktop checks and owner-attested host
resources. Missing/malformed/inconsistent data, replaced/cleared/rolled-over logs
and ambiguous correlation must block; record number alone cannot prove continuity.

Guest samples are pending. Do not pick a tolerance, hardcode a new boot string,
truncate to seconds or integrate this candidate into the existing guard yet.
Changes after restart are currently based on documented startup-event semantics,
not an experimentally forced guest reboot. Fast startup/resume, clock correction,
log replacement and VM snapshot rollback need explicit limits in any proposed
contract; same-user malicious mutation is outside the cooperative guard's claims.

After guest evidence, return the chosen contract/limits and any narrowly supported
non-GUI correction for review. Preserve existing packets, peer backend fix and both
Blocked outcomes. No new positive-run command. Native AOT/Ubuntu GUI stay gated;
G3 held. Blank screens remain an independent unresolved observation.
