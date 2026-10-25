# Ubuntu process lifecycle correction — review submission

Update: [parent accepts this correction and authorizes one new attended confirmation](2026-10-25-ubuntu-lifecycle-correction-acceptance.md).
No third confirmation has run. Submission below retains its original review context.

**Non-GUI correction complete and ready for parent review. No further GUI attempt
is authorized or performed.** Implements the
[second confirmation review direction](2026-10-25-ubuntu-confirmation-lifecycle-review.md).
Both original attended outcomes remain Blocked/2 with their accepted successful
application evidence. Original packets, attempt receipts, consoles and reports remain
unchanged; no new positive command exists.

## Audit and lifecycle correction

The accepted observer performed these reads outside its inner exit handler:

| Previous read window | Correction |
| --- | --- |
| Selection comm/command/executable | Inside outer early-pinned lifecycle |
| Selected argv/cwd/canonical DLL | Inside same lifecycle |
| Initial full identity before inner pin | Covered by the outer pin and handler |
| Inner pidfd acquisition/first full recheck | Covered by the outer pin and handler |
| Existing post-pin capture | Retained, with exact per-field operation labels |
| Early metadata before a proven pin | Explicit unresolved failure; no inferred exit |

`observe_candidate` first obtains minimal PID/UID/start and proc-directory device/
inode metadata. That preliminary token is untrusted. It opens a pidfd immediately
and rechecks the token before binding the observed original identity. Failure or
change during this bracket remains unresolved/Blocked; generic ENOENT/ESRCH or a
later absent PID does not establish original exit. A descriptor attached to a reused
numeric PID cannot pass a changed original-token check.

Selection, actual-target cwd, relative/absolute canonical DLL resolution and full
capture then execute under that outer lifecycle. Existing inner pinning remains
as an additional bracket. Full capture must match the early original start and
UID and the selected executable/command. `/proc` directory identity supplements
start checks and is checked during exit corroboration when present. PID/start/
UID/executable/command/cgroup or cwd/argv changes still Block. Required missing main,
peer or Rust-child records still Block; no ancestry or exit-event substitution.

Only a bound original's exit readiness plus fresh dead/zombie or absent-process
evidence can produce an exit-during-observation record. Live unreadability,
structural/path violations, reused PID, unbound identity and ambiguous corroboration
remain unresolved. No blanket permission/disappearance suppression was introduced.
An early pre-pin disappearance may still Block: the correction does not fabricate
evidence where Linux did not provide a reliable binding.

Existing actor scope and preexisting-nonactor permission rule are unchanged.
Metadata of other UIDs is outside scope; unreadable relevant identities are not
exempted. Direct managed graphical-environment reads and non-GUI actor ancestry/
cgroup/wrapper-context obligations remain as previously reviewed. No full process
environment is recorded.

Diagnostics distinguish full observation, outside scope, corroborated exit and
unresolved failure. They retain PID, known start, scope reason, exact stage/operation,
pin-acquired/bound/corroborated flags, original error type/errno and corroboration
failure details. Initial/post identity operations distinguish stat, directory UID/
identity, executable, command and cgroup. No historical error's exact failing read
or clean-exit cause is assigned retrospectively.

## Immutable candidate and exact delta

Packet: `artifacts/ubuntu-lifecycle-correction/ubuntu-lifecycle-final-review-20261025/`.
Receipt SHA-256:
`0e79adba5a46c1d2687c9070a7a126adaf7e5a039d3d659b11089abc8fc9b3c6`.
Parent receipt:
`77239bbc51aef1050a61e8184bfc7ea3bffb4d52d341f5a245d744a026089c95`.
546 source entries, 273 build inputs, ten payloads.

Only two existing source paths change: runtime observer and explanatory architecture
document. One lifecycle regression file is added. All other prior source bytes,
original runner/app/assertions/CONFIRM/consent, session/seat/socket preflight, actor
set, private ext4 roots, polling frequency, cleanup timing, recorder and exporter
remain unchanged. Matching archive/manifests/helper hashes and receipt provenance
are updated. Candidate inherits True from the accepted enabled parent; this is not
execution authorization. Working-tree positive gate remains False.

Independent verifier/diff/results:
`artifacts/ubuntu-lifecycle-correction/verify_packet.py`, `lifecycle-correction.diff`,
`independent-verification.json`, `packet-verification.json`. Every archive/input hash,
flat helper and payload verifies; exact parent comparison confirms preserved bytes.
The intermediate lifecycle packet is retained but is not the final review candidate.

## Non-GUI evidence

**48 frozen checks pass on Linux/WSL, no skips.** Windows passes 45 with three
explicit Linux-only skips. This retains the full previous 37 applicable/refusal
checks and adds eleven lifecycle regressions. Disabled-assumption tests remain
historical exclusions, not enabled passes. Original frozen test sources are retained.

New tests cover disappearance before initial identity, ESRCH at pin acquisition,
failure during original binding, PID reuse, selection executable/cwd loss, initial/
post capture failures within outer corroboration, live command unreadability,
structural path errors, reuse during corroboration, full-record diagnostics and
unavailable pidfd support. Prior tests retain cwd/path/symlink/exec/cgroup changes,
main/child identity, scope/survivors, consent/owner/export/integrity and outcome rules.

The Linux-native fixture starts only a small Python child, checks real `/proc`
token binding and pidfd readiness while alive, then exercises real exit corroboration
after the fixture exits. It does not build or launch Pivot, connect to a display,
kill unrelated processes or alter production cleanup timing. Synthetic race cases
test branches; they do not reconstruct the four old PID failures or prove corrected
GUI qualification. Actual native evidence is WSL, not another Hyper-V GUI run.

Host/Linux frozen logs and hashes are retained as `candidate-refusals.win32.log`,
`candidate-refusals.linux.log` and `independent-verification.json`. Reproduce only
non-GUI tests with `check_frozen_lifecycle.py`. An early Windows fixture failed
because its fabricated executable used POSIX separators while the simulated Windows
selection returned a Windows path. Only that fixture literal was normalized;
subsequent focused and full suites pass. No production rule was relaxed for it.

## Review boundary

Review lifecycle binding, diagnostic granularity, exact candidate and non-GUI
results before any positive authorization. Both old attended authorizations remain
consumed. No guest action is requested now. A future confirmation must still prove
complete direct main/three-peer/Rust-child identities, independent backends, four
zero charges, unchanged functional/WB5 suite, native/source/context integrity, no
unresolved observer failures and empty strict scoped survivors.

No cleanup waits, polling-frequency change, actor exemption, broad error suppression,
unrelated termination, alternative runner, sudo, fallback, new broker or lifecycle
automation. Long Ubuntu commands stay shared scripts with short instructions and
automatic reports; no routine sentinel. Windows gates/Ubuntu headless closed;
Ubuntu GUI open, AOT/ARM64/Rosetta/unattended separate. G1/G2 closed; G3 held/unstarted.
No product changes.
