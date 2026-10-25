# Windows guest non-GUI preflight — verified results

Subsequent [parent acceptance closes this gate](2026-10-25-windows-guest-preflight-acceptance.md).
The pending-review language below retains the original submission; positive GUI
execution is still unapproved and current-session clipboard revalidation is required.

2026-10-25. **Actual Windows guest preflight passed.** Native report recovered and
hash-verified without rerunning preflight. Positive managed/AOT execution remains
hard-disabled and requires parent review/explicit authorization. G3 remains held.

## Guest findings

| Field | Captured result |
| --- | --- |
| Hostname/user | WinDev2407Eval / User |
| Guest metadata | Microsoft Corporation / Virtual Machine |
| Firmware UUID | B439CDC2-EA73-42DB-A4C3-825CE132C0C0 |
| Current boot | 2026-10-25 00:37:42.0920420 -04:00 |
| Process/session | PID 9008 / session 1 |
| Desktop | WinSta0 / thread Default / input Default |
| Owner binding | ROBPC / Pivot-Win11-x64 / f89ff908-16dc-44c4-af54-008967e48443 |
| Source / inputs | 525 / 257 hash-verified files |
| Fixed entry point | tests/application-session/ui_smoke.py; not executed |
| Native work | C:\PivotWork\attended-be006f4110234dc3b69314576948e96e |
| Status / GUI | PreflightPassed / GUI_executed=false |

The wrapper passed fixed native-x64 identity, NTFS/reparse/root checks, frozen
receipt/source/input verification and owner CONFIRM, with unchanged measured
identity across that acknowledgement. Private Cargo/.NET/NuGet/temp/fixture roots
are recorded under the native work directory; .NET bin/obj remain under source.
Firmware UUID is evidence, not assumed to equal the owner-attested Hyper-V VM ID.

**Boot/session differ from the October 24 clipboard observation references.**
The earlier reference was boot 2026-10-24 15:00:16 -04:00 / session 3; this capture
has October 25 boot / session 1. Owner CONFIRM attests current Basic Session and
reviewed conditions, but does not independently prove continuity or a repeat
sentinel check on this boot. Return that distinction for parent review; do not
silently treat historical isolation evidence as perpetual or invent a new test.

## First export failure and copy-only recovery

The first bootstrap capture remains **Blocked/2**. The wrapper completed and
printed its native report path after the input prompt on the same log line. The
bootstrap incorrectly required a line-start match, so it omitted the native report
and conservatively returned Blocked. Five original returned hashes and the original
transfer manifest digest verify. This was an exporter defect, not a guest preflight
or GUI assertion failure.

The owner then ran only the supplied copy operation on the exact native report
directory. Its native-generated manifest verifies recovered `preflight.json` SHA256
`67e39e212d06903fd9dda509281cdf127694189f5f4b5172c0e41277475112d0`.
Packet receipt digest matches both the original capture and frozen packet:
`7480bb2e90d6ceba019e334ec84bb7a1d69117b59e0decff5e5f1bfaa952ca39`.
The native work path agrees with the original console. No recapture, preflight
replay, build, full CI or GUI execution occurred during recovery.

The preparation bootstrap now handles the prompt/path same-line case; its targeted
regression check passed. The original frozen packet/bootstrap and first returned
capture remain unchanged. A future execution packet must separately freeze/review
any corrected bootstrap or positive-path enabling change.

## Review evidence

- [Recovered native preflight](../../artifacts/attended-guest-preflight/results/Pivot-Win11-x64/recovered-native-be006f4110234dc3b69314576948e96e/preflight.json)
- [Original blocked capture](../../artifacts/attended-guest-preflight/results/Pivot-Win11-x64/preflight-capture-693ef95913dc463aa6a77ae30a7e6cba/capture.json)
- [Independent reconciliation receipt](../../artifacts/attended-guest-preflight/guest-preflight-reconciliation.json)
- [Packet instructions and first-export history](2026-10-25-windows-guest-preflight-packet.md)

Return these findings for parent review before enabling the exact managed path.
Preflight CONFIRM remains distinct from the existing GUI runner's affirmative
terminal consent. No GUI/backend/nested-peer/zero-charge acceptance is claimed by
this metadata gate. Native AOT, Linux GUI and unattended initiation stay separate.
Accepted headless gates/G1/G2 remain closed; no product/history changes.
