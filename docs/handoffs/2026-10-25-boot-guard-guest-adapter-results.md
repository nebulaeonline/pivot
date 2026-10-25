# Boot continuity production adapters — guest validation complete

Subsequent [parent acceptance closes this non-GUI gate and authorizes one new attended confirmation after verification](2026-10-25-boot-continuity-guard-acceptance.md).
The pending-review language below retains the original submission context.

2026-10-25. **All six guest probe samples verify.** Implementation/candidate evidence
is ready for parent review. No new positive GUI run is authorized or executed.
[Implementation and candidate](2026-10-25-boot-continuity-guard-implementation.md).

## Actual production results

The owner ran only the frozen read-only adapter probe inside Pivot-Win11-x64.
Each of six samples records actual event/CIM/Win32 uptime, interactive desktop
and qualified guest identity measurements. All four comparisons per sample verify:
within-collection boot continuity, reviewed boot continuity, reviewed guest context
and within-collection guest identity. These are **24 evaluations**, not 24
independent guest runs. No adapter errors are reported.

| Observation | Verified result |
| --- | --- |
| Guest/user | WinDev2407Eval / User |
| Probe process/session | PID 8964 / session 1 |
| Window station/thread/input desktop | WinSta0 / Default / Default |
| Startup event | Reviewed System / Kernel-General event 12 / record 2910 |
| CIM variation from reviewed baseline | 0 seconds in this capture |
| Largest query-local elapsed discrepancy | 0.021591 seconds (rounded up) |
| Largest cross-observation elapsed discrepancy | 0.016881 seconds (rounded up) |
| Longest baseline-to-current elapsed interval | 1054.118 seconds (rounded up) |
| Policy budgets | 0.100 seconds CIM; 2 seconds elapsed discrepancy |
| GUI/clipboard/settings/reboot work | None |

The source contains negative regression coverage for the original 1.179 ms
difference; that variation was not newly reproduced by these adapter reads.
The current capture proves real-adapter operation in the sampled guest context,
not recognition of every possible reboot or cryptographic integrity. No reboot
was deliberately performed. Blank-screen behavior remains separate/undiagnosed.

## Integrity and independent reconciliation

Native original:
`C:\PivotPreflight\continuity-adapters-5ac621dc6ef04a198c54e0d93e8c9e59`.

[Returned raw adapter measurements](../../artifacts/windows-boot-guard-implementation/guest-adapters/continuity-adapters-5ac621dc6ef04a198c54e0d93e8c9e59/adapters.json)
match the native-origin manifest. Both transfer-manifest digests verify. All five
executing helper hashes match the immutable candidate's helper bytes, including
the inherited enabled wrapper code state; that state grants no positive authority.

The candidate's eleven receipt-covered files verify against SHA256
`905350c373552132becc0547ef467e368d690033f0bab416465e9909752702e0`.
All 24 comparison results were independently recomputed from the returned raw
measurements using the production contract; every recomputed result matches.
[Reconciliation receipt](../../artifacts/windows-boot-guard-implementation/guest-adapter-review.json).

The reviewed candidate has 533 source entries/262 inputs; its earlier exact-delta
and frozen test verification remain original evidence. No source, packet, budget
or result was edited to produce a pass. This check did not replay product tests,
builds or the already completed non-GUI regression gates.

## Parent review packet and stop

Return these actual-adapter results together with the implementation handoff,
source diff, 21 applicable frozen non-GUI tests, eight comparator/integration tests,
thirteen preflight tests, retained lifecycle regression and warning-free managed
build. The previous peer-corrected confirmation packet and both historical Blocked
outcomes stay unchanged; actual functional execution from the first run remains
accepted within its documented scope.

Separate approval is still needed before another attended managed confirmation.
No new GUI command is issued. No clipboard sentinel is requested absent a concrete
changed condition or contrary observation. Native AOT/Ubuntu GUI remain separate;
no unattended authority, second protocol, dispatcher, lifecycle automation or host
fallback. Headless gates/G1/G2 closed; G3 held; no product/history changes.
