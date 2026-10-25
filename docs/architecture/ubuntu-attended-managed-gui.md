# Ubuntu x64 owner-attended managed GUI preparation

Current decision: [session evidence and disabled contract accepted](../handoffs/2026-10-25-ubuntu-disabled-contract-acceptance.md).
Bounded live-wrapper implementation and non-GUI validation are authorized next;
positive Ubuntu GUI/AOT remains unapproved. Preserve the accepted prototype snapshot
and submit a new disabled implementation packet rather than enabling this prototype.

Status: disabled contract prototype and implementation proposal for parent review.
No positive Ubuntu GUI or Native AOT execution is authorized. The prototype CLI
always exits Blocked/2 and its fixed invocation function independently refuses
while `POSITIVE_RUN_APPROVED=False`. Enabling that constant alone is insufficient:
live preflight, runner supervision and evidence export remain to be implemented
under a subsequent reviewed direction. No executable positive owner command exists.

## Session and display identity

Actual evidence identifies UID 1000/robhorvath, an active local Wayland user session
2, seat0 with ActiveSession=2, and user Display=2/Sessions=3 2. Session 3 is the
systemd user manager, not a second graphical login. Terminal and GNOME processes
run under user@1000.service scopes: failed `show-session self` is retained. The
correlation establishes the user's display session, not terminal membership in it.

The prototype requires matching user display-session and seat observations, own
active/nonremote graphical user session, matching display endpoint paths and owned
socket metadata. Environment variables alone cannot pass. Actual observed boot UUID
is a Linux boot identifier; no Windows event/CIM policy is reused. A later live
preflight must compare boot UUID before/after and against reviewed evidence, retain
process start ticks and socket device/inode/owner, and stop on change/unavailability.
Do not infer which Avalonia backend will execute from Wayland environment or sockets.
DISPLAY=:0 and Xwayland are present; actual managed main/peers must independently
report their exercised backend. Native Wayland support is not presumed.

Hyper-V vendor/model and hostname corroborate intended guest but do not uniquely
bind its management VM ID. DMI UUID/serial reads were permission-denied; preserve
that fact. Propose owner confirmation of ROBPC/Pivot-Ubuntu-x64 and previously
reviewed VM ID 9ce1b716-aa20-4b16-a206-25126fab46d5, Basic Session and unchanged
resource/isolation settings. Parent must review this cooperative binding; no sudo,
new permit protocol or repeated clipboard sentinel is proposed.

## Fixed managed path proposed

Pinned immutable import to native ext4 work directory; verify receipt, exact source
archive, build inputs and helper bytes before imports. Future live preflight must
check current guest/user/session/seat, endpoints, tool architecture and installed
dependencies. Existing Ubuntu headless evidence is retained, not rerun.

Keep unchanged `tests/application-session/ui_smoke.py` and its original affirmative
interactive consent mechanism. Fixed invocation takes no alternate command, RID,
fallback or consent argument. Current prototype includes only its disabled runpy
entry, not a production activation wrapper. Preserve real stdin/stdout and original
runner timeouts and assertions in the future implementation. Owner-condition
CONFIRM is distinct from original GUI consent; neither can be preanswered/reused.

Source, target, Cargo/NuGet homes, .NET home, bundle extraction, temp and file fixtures
must remain native guest-local with private child environment, certificate/workload
updates disabled and Avalonia telemetry opt-out set. Preserve session environment
needed by original runner. J:/mnt/j is frozen import and hash-verified export only.

## Evidence and cleanup proposal

Retain original runner logs and four real managed-mode main/peer paths, independent
graphical backends, PIDs and zero resource charges. Linux .NET process-session values
are not logind IDs; never compare them as though they were Windows session numbers.
Correlate runtime processes using Linux UID, /proc PID start ticks, executable,
ancestry/cgroup and reviewed display/seat observations. Race/unreadable evidence
must remain visible; missing actual runtime identity blocks acceptance.

Propose a read-only scoped /proc snapshot for private work-root executables and
build/runtime actors whose command lines reference that root, retaining boot UUID,
PID/start ticks/executable and observation time. No system-wide process cleanup,
unrelated kills, exclusions, added waits or retry-until-green. Exact implementation
and any .NET Linux session-field limitations require review before activation.

Retain original WB5 and functional checks, screenshots, source-after integrity,
scoped survivors, exact exit codes and native-origin hashes. Export a verified copy
and transfer digest, with native originals available for copy-only recovery. Failed
runner assertions remain Failed; missing prerequisites/consent/incomplete evidence
remain Blocked. No success inferred after interruption/timeout.

Return actual metadata, this proposal, refusal checks and disabled packet now.
Positive managed GUI and Ubuntu AOT are future independent gates. Windows gates
closed; ARM64/Rosetta/unattended/orchestration separate. G3 held; no product changes.
