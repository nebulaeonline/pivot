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

Basic Session is now owner-confirmed for both guests. Actual resource exposure
still needs an owner record: clipboard/history synchronization, any other active
remote clients, redirected drives/devices and host filesystem access beyond the
explicit J: or /mnt/j exchange. Unknown settings remain Unknown; enabled Hyper-V
integration services alone cannot answer these questions.

Ubuntu still needs its intended graphical session correlated to a logind
session/seat if available, without inspecting unrelated users or dumping a full
environment. No requirement to transcribe output manually is imposed.

After metadata review, the harmless bidirectional sentinel check requires a
separately agreed quiet window and explicit consent to overwrite clipboards.
That consent has not been supplied. Return each guest's complete packet for
individual parent review before any unattended activation. No permit or positive
managed/AOT run follows automatically from this metadata.

[Capture workflow and boundaries](2026-10-24-hyperv-gui-isolation-qualification.md).
Both full headless CI gates remain accepted/closed; no repeats were run. G3 stays held.
