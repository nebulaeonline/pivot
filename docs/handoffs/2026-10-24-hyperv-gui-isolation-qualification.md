# Hyper-V x64 GUI-isolation qualification — owner capture instructions

2026-10-24. Initial session reports returned; both connections are owner-confirmed
VMConnect Basic Session. [Evidence review and remaining gaps](2026-10-24-hyperv-gui-session-review.md).
Neither guest is GUI-isolation-approved. Full headless CI remains accepted/closed;
do not rerun it. G3 remains held. [Authorization and boundaries](2026-10-24-hyperv-headless-ci-acceptance.md).

Initial observations have since received [partial parent acceptance](2026-10-24-hyperv-gui-session-acceptance.md).
Do not repeat the capture below. To attempt only the missing Ubuntu correlation,
run this from a terminal in the intended Ubuntu desktop:

```sh
bash /mnt/j/coding/pivot/scripts/qualification/correlate_linux_gui_session.sh
```

This queries the caller's logind session and current-user gnome-shell cgroup
metadata only, with targeted session queries where a desktop scope is found.
It preserves unresolved fields as Unknown, writes a native-local original and
hash manifest, then copies and verifies the report on the share. It neither
enumerates unrelated users nor reads process environments or clipboard data.
Resource-exposure confirmation remains an owner record, separate from this query.

This packet covers `Pivot-Win11-x64` and `Pivot-Ubuntu-x64` separately. It introduces
no collector, remote execution, configuration change or GUI-test permit. The first
step is inspection only. **Do not perform the clipboard procedure yet.**

## 1. Record the actual connection, without changing it

For each guest, report how you open its desktop: VMConnect Basic Session,
VMConnect Enhanced Session, Remote Desktop, or another named client. Record the
client's actual current mode, not merely whether Enhanced Session is available.
If the client does not expose a setting, record Unknown rather than assuming Off.
Do not disconnect/reconnect or change settings merely to complete this inspection.

Inspect only this test connection's resource settings. Record clipboard sharing,
drive/folder redirection, printers, smart cards, USB/devices and other available
resource channels as Enabled, Disabled, NotAvailable or Unknown. For Enhanced
Session/RDP, record the selected local resources and expanded drive/device choices
where exposed. Note whether connection settings are saved and which specific
test profile is used; do not copy entire personal RDP profiles or credentials.

Record the deliberate SMB/CIFS exchange separately: guest Windows `J:\` and Linux
`/mnt/j` map the host J: root. This is an approved source/report exchange path, not
proof of GUI separation. Record any additional host filesystem exposure and any
clipboard manager/history synchronization. Unexpected exposure needs review;
this packet does not authorize changing it.

Host Enhanced Session capability and enabled integration services are already
captured. They do not answer these questions; no repeat host preflight is needed.

## 2. Capture identity inside the intended graphical session

Use a terminal opened in the guest desktop that will eventually run the GUI suite,
not SSH or a separate remote shell. These commands read identity/session metadata
only. Do not dump the full environment, process command lines or personal files.
Keep the output locally first; return it with the record below through the explicit
share. No build or application launch is requested.

Windows guest, ordinary PowerShell (no administrator or developer shell needed):

```powershell
Get-Date -Format o
hostname
whoami
[System.Diagnostics.Process]::GetCurrentProcess().SessionId
$env:SESSIONNAME
Get-CimInstance Win32_OperatingSystem | Select-Object LastBootUpTime
```

The expected Windows hostname is `WinDev2407Eval`, not the Hyper-V display name.
Bind it to VM ID `f89ff908-16dc-44c4-af54-008967e48443` using the owner-observed
VM connection. A Console session name alone does not prove clipboard isolation.

Ubuntu guest, a terminal in the intended desktop:

```sh
date --iso-8601=seconds
hostname
id -un
cat /proc/sys/kernel/random/boot_id
printf 'session=%s\ntype=%s\ndesktop=%s\ndisplay=%s\nwayland=%s\n' "$XDG_SESSION_ID" "$XDG_SESSION_TYPE" "$XDG_CURRENT_DESKTOP" "$DISPLAY" "$WAYLAND_DISPLAY"
```

If `XDG_SESSION_ID` is set and `loginctl` is installed, also run:

```sh
loginctl show-session "$XDG_SESSION_ID" -p Id -p Name -p Type -p Class -p Remote -p State -p Seat -p Display
```

If it is unset or the query fails, retain that result; do not substitute another
user's session. Bind the guest to VM ID `9ce1b716-aa20-4b16-a206-25126fab46d5`.
Distinguish desktop session type from Pivot's eventual exercised backend: a
Wayland desktop with DISPLAY set can support XWayland, but does not prove which
backend an application used. That application evidence belongs to a later gate.

## 3. Return one record per guest

Save a plain-text record with the fields below and the command output in a fresh
guest-local directory: Windows under `C:\PivotPreflight`, Ubuntu under
`$HOME/pivot-preflight`. Copy it deliberately to a fresh timestamped directory at
`artifacts/guest-gui-isolation/<VM-name>/<timestamp>/` on the share. Retain the
local original. Screenshots are optional; crop to this connection's settings and
exclude personal information. Describe how each setting was observed.

```text
VM name / VM ID:
Capture time with offset:
Host / owner performing inspection:
Connection client / current Basic, Enhanced, RDP or other mode:
How current mode was verified:
Specific test profile (if applicable; no credentials):
Clipboard redirection / observation source:
Drive/folder redirection / observation source:
Printer, smart card, USB and other device redirection:
Explicit J: or /mnt/j exchange mapping:
Other host filesystem exposure:
Clipboard history/manager synchronization (or Unknown):
Guest user / session ID / boot identity / desktop backend:
Attached session-command output:
Settings changed during capture: None (otherwise describe and stop for review)
Clipboard sentinel: NotExecuted
GUIIsolationApproved: NotConfigured
Unknowns / blockers:
```

A complete metadata packet is ready for inspection, not approval. Missing fields
remain blockers; successful headless compilation cannot fill them in.

## 4. Separate scheduled clipboard step — pending consent

[Concrete owner-run checklist](2026-10-24-hyperv-clipboard-check.md) is prepared;
it remains pending explicit scheduling/consent and does not authorize execution.

After metadata review, explicitly agree a quiet window with the owner before
overwriting either clipboard. The [existing bidirectional procedure](2026-10-24-guest-qualification.md#proposed-clipboard-separation-procedure--not-executed)
uses known harmless host/guest markers and scratch surfaces only. Record the
exact VM, connection configuration, guest session/boot, times and observed result
in both directions. Do not read/export prior clipboard contents, use mail or
production files, or promise restoration of personal clipboard history/formats.

Unexpected propagation is a failed separation check: stop and preserve the first
result. Do not silently change settings and replace it with a pass. Any later
changed-configuration attempt needs a new record and explicit scheduling.

Return each guest's evidence for separate parent approval. No unattended GUI,
managed desktop/AOT execution, automatic permit, host fallback or orchestration
is authorized by completing this checklist. Relevant connection, redirection,
session or boot changes require re-evaluation; approval is not perpetual.
