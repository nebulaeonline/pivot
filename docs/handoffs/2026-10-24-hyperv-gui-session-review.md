# Hyper-V GUI session metadata — first review

2026-10-24. Both owner-returned reports are readable and consistent with the
previously qualified guest identities. This is partial qualification evidence,
not GUI-isolation approval. No clipboard checks or application tests were run.
[Parent partial acceptance and next metadata boundary](2026-10-24-hyperv-gui-session-acceptance.md).

| Field | Windows x64 | Ubuntu x64 |
| --- | --- | --- |
| Intended VM | Pivot-Win11-x64 | Pivot-Ubuntu-x64 |
| Connection | VMConnect Basic, owner confirmed unchecked Enhanced Session | VMConnect Basic, owner confirmed unchecked Enhanced Session |
| Capture time | 2026-10-24 21:48:15 -04:00 | 2026-10-24 21:51:06 -04:00 |
| Hostname | WINDEV2407EVAL | Pivot-Ubuntu-x64 |
| User | windev2407eval\user | robhorvath |
| Session | ID 3, Console | XDG_SESSION_ID unset; loginctl query not performed |
| Boot | 2026-10-24 15:00:16 -04:00 | 616f92de-4c4d-4a06-8937-3f64765a039d |
| Desktop hints | Console session | ubuntu:GNOME, type wayland, DISPLAY=:0, WAYLAND_DISPLAY=wayland-0 |
| Clipboard sentinel | NotExecuted | NotExecuted |
| GUI isolation | Not approved | Not approved |

The Windows hostname agrees with earlier qualification despite differing from its
Hyper-V display name. VM display-name/ID binding and Basic Session mode rely on
owner observations, not independent detection by these commands. Ubuntu's
environment describes a Wayland desktop with an X display available; it does not
establish Pivot's exercised backend or a specific logind session/seat. The missing
session variable is a capture limitation, not a failed product test.

## Evidence and integrity limits

Returned reports:

- [Windows session](../../artifacts/guest-gui-isolation/Pivot-Win11-x64/gui-session-20261024T214802136/session.txt)
- [Ubuntu session](../../artifacts/guest-gui-isolation/Pivot-Ubuntu-x64/gui-session-20261025T015106587232775/session.txt)
- [Returned-file hash receipt](../../artifacts/guest-gui-isolation/session-review.json)

The reports have been hashed after return. Neither has a native-origin hash
manifest; this is not an independently verified origin-hash claim. The Linux
capture helper compares the copied file against its local original. Windows's
copy command does not perform that comparison. Preserve the local originals and
returned reports; no recapture of these metadata fields is requested.

## Remaining qualification work

### Ubuntu correlation follow-up — reviewed

Owner-returned [correlation report](../../artifacts/guest-gui-isolation/Pivot-Ubuntu-x64/gui-correlation-20261025T015904541494478/correlation.txt),
captured 2026-10-24 21:59:04 -04:00, matches the initial hostname, user and boot
UUID. Its returned bytes match the native-generated SHA256SUMS entry
`7581c4ca33ec43f819b8dc37699ce1b57370b93817b3d6a305e2da352c2a57eb`.
[Verification receipt](../../artifacts/guest-gui-isolation/correlation-review.json).
This strengthens the follow-up's transfer evidence without altering the initial
reports or claiming retroactive origin manifests for them.

The terminal lives under user 1000's systemd user service in a ptyxis-spawn scope.
`loginctl show-session self` reports that the caller belongs to no known session.
Current-user gnome-shell PID 3718 is under
`user@1000.service/session.slice/org.gnome.Shell@ubuntu.service`, without a
`session-*.scope` from which to derive a logind session. This explains why this
narrow correlation attempt did not resolve session/seat. Both remain **Unknown**;
no failed desktop/product test or absent graphical session is inferred.

The parent explicitly permits Unknown when correlation cannot be established
confidently. This bounded attempt is complete; no additional capture or broader
session enumeration is requested. Return the limitation for review alongside the
remaining resource-exposure record. Clipboard testing remains NotExecuted.

Owner follow-up (2026-10-24): “No. No other connections other than through the
session manager.” In the context of the earlier VMConnect Basic Session
confirmation, this establishes owner-reported absence of other active graphical
connections for both guests. It is not an independently inspected settings claim.
The reply does not explicitly establish clipboard-history synchronization,
additional filesystem exposure or redirected-device settings; those remain
Unknown rather than being inferred disabled.

Subsequent owner response resolves that owner-record question: asked about
clipboard/history synchronization, shares beyond J: or /mnt/j and redirected
printers/USB/smart cards for both VMs, the owner answered “none.” Record these as
none owner-reported, not independently audited settings. The owner also explicitly
consented to replacing host/guest clipboards now for the manual sentinel procedure.
This consent is scoped to that quiet manual test window, not unattended GUI
execution or future clipboard access. The owner subsequently reported successful
separation for both guests. [Results and review packet](2026-10-24-hyperv-gui-isolation-results.md)
supersede the pending-test statements below while preserving the capture history.

Basic Session is now owner-confirmed for both guests. Actual resource exposure
still needs an owner record: clipboard/history synchronization, any other active
remote clients, redirected drives/devices and host filesystem access beyond the
explicit J: or /mnt/j exchange. Unknown settings remain Unknown; enabled Hyper-V
integration services alone cannot answer these questions.

Ubuntu's attempted logind correlation is recorded above with session/seat Unknown.
No requirement to transcribe output manually is imposed.

After metadata review, the harmless bidirectional sentinel check requires a
separately agreed quiet window and explicit consent to overwrite clipboards.
That consent has not been supplied. Return each guest's complete packet for
individual parent review before any unattended activation. No permit or positive
managed/AOT run follows automatically from this metadata.

[Capture workflow and boundaries](2026-10-24-hyperv-gui-isolation-qualification.md).
Both full headless CI gates remain accepted/closed; no repeats were run. G3 stays held.
