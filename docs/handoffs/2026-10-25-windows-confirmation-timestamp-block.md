# Windows confirmation — false block on fractional boot timestamp

Subsequent [parent review accepts the refusal and authorizes a bounded read-only identity investigation](2026-10-25-boot-identity-investigation-direction.md).
No additional positive execution or clipboard sentinel is authorized or requested.

2026-10-25. First confirmation attempt returned **Blocked/2 before the GUI runner
started**. No build, application or peer execution is claimed for this attempt.
The previous functional pass/overall evidence Blocked result is unchanged.

The new preflight passed guest/NTFS/source/desktop checks and owner CONFIRM, but
the managed branch compared the boot timestamp to a hardcoded string:

| Field | Reviewed | New capture |
| --- | --- | --- |
| LastBootUpTime | 2026-10-25T00:37:42.0920420-04:00 | 2026-10-25T00:37:42.0932210-04:00 |
| Session | 1 | 1 |
| Window station / desktop | WinSta0 / Default | WinSta0 / Default |
| Guest / firmware UUID / user | Expected qualified guest | Unchanged |

The timestamp differs by **1.179 milliseconds**, within the same displayed boot
second. The exact-string guard refused it as a different boot/session. This is
insufficient evidence of a reboot or session replacement. Treat the refusal as a
brittle boot-identity check, not an established configuration change. The actual
reason for the differing fractional timestamp is not established by the reports.

The owner reports the guest frequently goes to a blank screen. These captures do
not diagnose that behavior or link it to an actual reboot. They establish only
that the sampled desktop was accessible as Default in session 1. No guest power,
screen/lock, integration or permission setting was changed during this review.

Original capture:
`artifacts/attended-guest-preflight/results/Pivot-Win11-x64/preflight-capture-e8e1f124be3e40b8bd0b06e47e03b3a7/`.
[Hash verification and comparison](../../artifacts/windows-peer-backend-correction/confirmation-block-review.json).
Outer and nested native-origin hashes and transfer-manifest digests verify. Keep
the original Blocked result and approved packet unchanged; no retry was performed.

## Proposed next review

Correct the Windows boot-identity measurement/binding before attempting another
positive run. Do not simply update the hardcoded fractional string, ignore all
identity changes or synthesize continuity. A reviewed solution should distinguish
stable boot identity from presentation/measurement precision, retain genuine
boot/session-change refusal and exercise fractional-timestamp drift negatively
without opening windows. Assess a stable Windows boot identifier or explicitly
reviewed comparison semantics with their limitations before choosing implementation.

The approved immutable packet cannot be edited in place. Any guard correction,
new packet and further attempt require scope review; no additional run command is
issued. No repeat clipboard sentinel is justified by this timestamp difference
alone. No functional assertion or peer backend correction is implicated because
the UI runner did not start. Native AOT remains gated; G3 held.
