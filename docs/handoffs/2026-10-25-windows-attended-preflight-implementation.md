# Windows attended guest preflight — implementation review

Subsequent parent decision: [implementation accepted and closed](2026-10-25-windows-preflight-acceptance.md).
The submission below retains its original review context; the next gate is frozen
packet preparation and actual guest non-GUI preflight, not positive GUI execution.

2026-10-25. Implements the bounded [parent direction](2026-10-25-attended-guest-preflight-direction.md).
**Return for review; positive managed/AOT execution is disabled.** No new broker,
permit type, VM lifecycle action, dispatcher or product change. G3 stays held.

## Behavior

`scripts/qualification/windows_attended_gui.py` accepts only a source bundle,
reviewed receipt digest, existing NTFS work-root and a currently refused
`--run-managed` request. It requires a terminal and native Windows x64, expected
guest hostname, Microsoft Virtual Machine metadata, boot/firmware identity and
current-user interactive nonzero session. The Win32 adapter reads window-station,
thread-desktop and input-desktop names; it requires WinSta0/Default and matching
input desktop. It neither injects input nor accesses the clipboard.

Hyper-V management VM identity is explicitly owner-bound to ROBPC and
`f89ff908-16dc-44c4-af54-008967e48443`. Firmware UUID is recorded, not assumed to
equal management VM ID. Guest metadata alone cannot authenticate current host
configuration. The owner must type CONFIRM for Basic Session, reviewed resources
and unchanged clipboard-isolation conditions. This is not GUI consent.

The pinned packet receipt verifies all packet files and the executing wrapper's
two imported helper files. Source is hash-verified while extracted into a fresh
native-local tree, and build inputs must include the fixed managed UI entry point.
Reparse points in the work path and extracted source are refused. Private Cargo,
NuGet, .NET home, temp and persistence roots are native-local; .NET bin/obj stay
under fresh source. Inherited Pivot variables, including permits and self-check
activation, are discarded. Certificate generation and Rust auto-install are disabled.
Boot/user/session/desktop metadata must still match after owner acknowledgement.

Successful preflight writes `preflight.json` and native `origin-hashes.json`.
It does not compile, launch UI or transfer automatically. Errors return Blocked/2;
once a local report directory exists they also leave `blocked.json`. No arbitrary
command, general shell or remote target argument exists.

`--run-managed` refuses before machine queries. A fixed-entry invocation helper
also refuses while the compile-time approval constant is false. No CLI/profile/
environment switch can unlock it. The future reviewed invocation inherits real
terminal streams without supplying an answer or permit: existing `ui_smoke.py`
and its interactive broker remain responsible for actual GUI consent. That wiring
is not claimed as a successful positive run or nested-peer execution here.

## Verification and limitations

Non-GUI tests exercise a synthetic complete extraction/preflight, owner decline,
identity changes, invalid receipt/input/source/helper data, stale authority
sanitization, redirected/reparse roots, platform/hostname/VM refusal and disabled
positive execution. Children are mocked in the synthetic flow. Windows-only tests
exercise real desktop metadata queries without Avalonia; unavailable desktop is
an expected refusal, not evidence of guest readiness. These host tests do not
qualify the actual guest desktop or independently verify the VM's current settings.

The first sandbox invocation encountered native Temp ACL errors. The outside-
sandbox attempt then exposed a test-fixture mistake: redirecting stdout replaced
the mocked TTY with a non-TTY, correctly causing refusal. The fixture was corrected;
the original failure log is retained. No production checks were weakened.
Logs live under `artifacts/attended-guest-preflight/`.

Final validation: eight preflight tests pass on Windows, plus five existing
packet-helper regression tests (their generated console CI stub, not full Pivot
CI). A separately recorded read-only host adapter query returned session 1,
WinSta0/Default, matching input desktop. This is host adapter evidence only; it
does not claim Windows guest GUI readiness. The future fixed invocation is tested
with a mocked subprocess to verify it inherits terminal streams and does not
inject consent; no GUI child is launched by that test.

No positive GUI tests or previously accepted full headless CI were run. No Linux
activation support is inferred. The cooperative preflight is not a malicious-code
sandbox: same-user mutation after checks and normal share access remain possible.
It does not introduce ongoing identity supervision, revocation or a lease.

## Next gate

Review this implementation and its negative/synthetic evidence. Before positive
execution, freeze and review a source packet including these helpers and approved
input digest; confirm the individual guest's current owner-attested conditions.
Authorize enabling the fixed managed path explicitly, then require the existing
runner's affirmative terminal consent. Use local NTFS, preserve first failures,
existing assertions, actual windows/nested peers and zero-charge cleanup.
Native AOT remains a later separate gate; no unattended initiation or host fallback.
