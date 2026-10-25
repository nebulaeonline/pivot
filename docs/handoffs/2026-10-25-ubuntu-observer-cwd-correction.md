# Ubuntu observer correction — cwd resolution and scoped metadata

Update: [parent accepts correction and authorizes one new attended confirmation](2026-10-25-ubuntu-observer-correction-acceptance.md).
No confirmation has run. Submission below retains its original pre-approval context.

**Ready for parent correction review. No positive execution authorized or performed.**
Implements the [bounded investigation direction](2026-10-25-ubuntu-first-managed-review.md).
Original first GUI packet and overall Blocked/2 result remain unchanged. No GUI
retry, application build, new positive launch command, guest setting change or sudo.

## Findings and correction

The unchanged runner launches the main managed DLL with a relative argument from
the frozen source directory. The original observer selected external runtimes by
an absolute invocation-root substring; it neither read target cwd nor canonicalized
the DLL. This explains a source-level selection gap consistent with the missing
direct main record, without inventing historical process evidence.

The observer now reads raw target argv and actual `/proc/<pid>/cwd`, resolves only
the fixed two-argument dotnet invocation and validates the canonical DLL against
the expected frozen native source target. Both relative main and absolute peer
forms are supported. Native/symlink/path boundaries remain enforced. Records retain
argv, cwd path/device/inode and canonical DLL; runtime validation uses that verified
path rather than requiring its textual presence in the command. Target cwd is
never assumed to be the observer's cwd.

For the compiler environ failure, the historical cause is still unknown. The
original code read display environment for every scoped compiler/build actor even
though those actors do not exercise the graphical backend. The correction limits
direct display-environment reads to actual managed GUI processes. Build/runtime
actors retain direct UID/start/executable/command/cwd/cgroup plus ancestry to the
wrapper and its independently verified live display context. Those fields establish
invocation context without claiming a compiler's environment was observed. No actor
or survivor is excluded; actual Rust-child identities remain required. Parent review
of this narrower environment obligation is requested explicitly.

Scoped capture pins a Linux pidfd and brackets PID/start/UID/parent/executable/
command/cgroup observations. It also rechecks argv and cwd path/device/inode.
Reuse/exec/context changes and structural path errors remain Blocked. On metadata
read failure, only pinned-original exit readiness plus fresh matching dead/zombie
or absent /proc evidence permits a retained exit-during-observation event. Reused
PID or insufficient evidence Blocks. An exited event is not a full process record
and cannot satisfy required main/peer correlation. No blanket PermissionError catch,
process-name exemption, extra cleanup wait or process termination is introduced.

The Linux API supports polling a pidfd for exit notification; observation uses
zero-timeout polling, not waiting for cleanup.
[Linux man-pages: pidfd_open](https://man7.org/linux/man-pages/man2/pidfd_open.2.html).
This is cooperative observation, not lossless tracing or proof against same-user
malicious interference. Existing preexisting-nonactor rules remain unchanged.

## Immutable correction candidate

Packet: `artifacts/ubuntu-process-correction/ubuntu-observer-review-20261025/`.
Receipt SHA-256:
`77239bbc51aef1050a61e8184bfc7ea3bffb4d52d341f5a245d744a026089c95`.
Parent is the accepted executed enabled snapshot receipt
`0c2b9817c0ef685038581f0deecc82d7d8c049d9e4357907e4b47cab3c4095aa`.
545 source entries, 272 inputs, ten payloads.

Exactly three prior source paths change: runtime observer helper, its existing
runtime tests and explanatory architecture document. One focused observer test file
is added. All other prior source bytes and payloads remain identical except the
corresponding archive/manifests/hash packaging. Candidate inherits the accepted True
flag; that is not execution approval. Repository working-tree gate remains False.
No runner, application, assertions, consent/CONFIRM, live boot/session/seat/socket
preflight, actor set, private roots, cleanup timing, recorder or export changes.

Full delta: `artifacts/ubuntu-process-correction/observer-correction.diff`.
Independent reconciliation: `verify_packet.py`, `independent-verification.json`,
`packet-verification.json`. Every archive/input hash and flat helper identity is
checked against the candidate; preserved source bytes are compared to the parent.

## Non-GUI validation

Thirty-seven frozen checks pass on Linux in WSL, none skipped: 21 existing applicable
checks, five enabled-state refusals and eleven focused observer checks. The host
Windows run passes 35 checks with two Linux-only checks explicitly skipped.
Existing disabled-assumption exclusions remain disclosed; historical tests intact.
Logs are `candidate-refusals.linux.log` and `candidate-refusals.win32.log`; reproducible
suite is `check_frozen_refusals.py` in the correction artifacts directory.

Focused checks cover valid relative/absolute target paths, outside-root/wrong/missing
cwd, symlink boundaries, changed cwd inode, PID reuse/exec/context changes, live
environment denial, pinned original exit versus reused/live PID, unavailable pinning,
and compiler context capture without unnecessary environment reads. A Linux-native
read-only test exercises actual self `/proc` identity, cwd metadata and non-exited
pidfd polling. Existing tests retain session/seat/socket/owner/consent, scoped
nonactor policy, child identity, survivor/outcome, source/export and path protections.

One early Windows fixture assertion failed because native Temp used a short-name
alias while the function returned its canonical path. The test expectation was
corrected to compare canonical paths; runtime behavior was not weakened. Subsequent
focused and full frozen checks pass. No GUI/build was involved.

Synthetic races establish proposed control flow, not the cause of the old compiler
denial or successful future process capture. New Linux-native evidence is WSL, not
a fresh Hyper-V GUI run. No new target-guest invocation was requested or performed.

## Review request and boundaries

Review the exact observer changes, narrower non-GUI environment requirement and
pidfd/cwd evidence algorithm before any new positive authorization. A later attempt
must still capture direct main/three-peer identities and matching Rust children,
actual independent backends, four zero charges, unchanged functional/WB5 assertions,
native/source integrity and empty scoped survivors. Prior successful GUI assertions
do not establish this corrected observer's runtime success.

Do not reuse the consumed owner command. No further guest action is needed now.
Any longer future Ubuntu commands remain shared scripts with short instructions
and automatic report return. No repetitive sentinel, fallback, new broker or VM
lifecycle work. Windows gates/Ubuntu headless closed; Ubuntu AOT, ARM64/Rosetta and
unattended activation remain separate. G1/G2 closed; G3 held/unstarted. No product changes.
