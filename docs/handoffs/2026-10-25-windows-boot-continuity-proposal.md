# Windows boot continuity — guest evidence and offline proposal

Subsequent [parent review accepts research and authorizes bounded implementation](2026-10-25-boot-continuity-implementation-direction.md),
with explicit policy-budget and elapsed-comparison qualifications. No positive
GUI attempt is authorized by that decision.

2026-10-25. [Authorized investigation](2026-10-25-boot-identity-investigation-direction.md)
and [documented interface research](2026-10-25-windows-boot-identity-research.md).
**Investigation complete enough for contract review; production guard unchanged.
No GUI execution, reboot, settings change or new confirmation packet.** G3 held.

## Guest evidence

The owner ran the read-only collector in WinDev2407Eval. Six successful samples
over 10.737 seconds returned the same System / Microsoft-Windows-Kernel-General /
event 12 record **2910**, event time `2026-10-25T04:37:38.8635211Z`, StartTime
`2026-10-25T04:37:38.5000000Z`. Uptime increased throughout. All CIM values were
`2026-10-25T04:37:42.0932210Z`; the earlier fractional variation was not reproduced
inside this short window. Its cause remains unknown.

CIM boot time is 3.593221 seconds later than the event's startup data. Wall time
minus uptime was approximately 0.380–0.397 seconds earlier than CIM. The host's
corresponding offset was approximately +8.54 seconds. These are distinct
measurements; equality of their absolute reconstructed boot times is unsupported.
The event anchor was stable in all six host and all six guest reads, but no restart
was performed to experimentally verify cross-boot behavior. Event/startup semantics
come from the linked Microsoft documentation, not an invented test result.

Native original:
`C:\PivotPreflight\boot-probe-6a2bb78c0a9c4622917eb3d6dfcaf62e`.
[Returned raw observations](../../artifacts/windows-boot-identity/Pivot-Win11-x64/boot-probe-6a2bb78c0a9c4622917eb3d6dfcaf62e/measurements.json)
and [independent review receipt](../../artifacts/windows-boot-identity/guest-review.json).
Native-origin and both transfer-manifest digests verify. No repeated clipboard
test was requested. Blank-screen reports remain separate and undiagnosed.

## Proposed independently corroborated comparison

No documented opaque per-boot GUID contract was established by this research.
Propose this explicit cooperative continuity contract for parent review:

1. Pin a freshly reviewed guest startup-event anchor: provider/channel/event ID,
   record ID, logged timestamp and StartTime. Require the complete anchor to remain
   equal; record number alone is insufficient. Bind this to independently measured
   current user/session/WinSta0/input desktop plus owner-attested host/VM/resources.
2. Retain raw CIM as a corroborating observation, not identity. Proposed variation
   budget relative to that reviewed baseline: **100 ms**, without rounding away
   the raw value. Larger changes block even when session numbers match.
3. Bracket each OS query with actual Win32 uptime and wall time. Require increasing
   values and agreement of **elapsed differences**, not absolute boot estimates.
   Proposed maximum discrepancy: **2 seconds**, both within each query and between
   reviewed/current observations. Uptime reset or clock discontinuity blocks even
   with a stale/reused event record.
4. Missing/malformed/unavailable event or uptime, inconsistent measurements,
   changed user/session/VM binding or inaccessible input desktop block. Log loss,
   rollover or replacement blocks rather than silently creating a new baseline.
   Preserve owner CONFIRM and separate existing GUI consent without modification.

These numeric budgets are explicit **proposal choices**, not Windows guarantees
or empirically proven maxima. The measured windows support feasibility only.
They permit the observed 1.179 ms presentation difference while imposing much
stronger conditions than rounded-time equality. Legitimate time corrections could
still produce conservative refusal; do not silently renew approval or relax budgets.
Session IDs can be reused, timestamps/logs are not hostile-user-proof, and VM
snapshot rollback can replay multiple observations. This is cooperative execution
safety, not cryptographic proof of host or guest integrity. A guest-local saved
baseline cannot independently grant authority.

## Offline regression evidence and limits

`artifacts/windows-boot-identity/continuity_proposal.py` is a standalone offline
research comparator, **not imported by any runner or authorization path**. Seven
tests pass: all six measured guest samples, observed 1.179 ms/submillisecond
variation, changed/missing event, stale event plus uptime reset, clock discontinuity,
arbitrary CIM time, changed/missing user/session/desktop/VM context and malformed
or failed measurements. Context inputs are synthetic for these tests; the collector
does not newly establish a graphical identity or independently authenticate a host.

[Test log](../../artifacts/windows-boot-identity/continuity-proposal-tests.log).
This is a tested contract proposal, not a deployed correction or qualification
after a real reboot. No .NET application code changed; no repeat managed build
is needed to validate the read-only Python investigation. The existing warning-free
peer-correction build remains its original evidence.

## Review request and stop

Review the event/uptime/CIM contract and explicit uncertainty budgets before
integrating a boot adapter into preflight. If accepted, integration must obtain
current measurements with real Windows adapters, preserve missing-data/genuine-
change refusal, retain the peer fix and assertions, and derive a new immutable
candidate with minimal source/input delta. No mutable historical packet edits.
This research does not produce an execution packet or issue a positive-run command.

Both prior Blocked results and original packets remain unchanged. No third GUI
attempt, Native AOT, Linux GUI, new broker, remote dispatch, VM lifecycle or host
fallback. Headless gates/G1/G2 closed; G3 held; product/history work unchanged.
