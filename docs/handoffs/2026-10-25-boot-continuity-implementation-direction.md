# Windows boot continuity — implementation authorized

2026-10-25 parent decision, attachment
`7b6a44d7-5a76-4ced-98e4-44dfddaf10cc/Pasted text.txt`.
**Research accepted; bounded production-guard implementation and non-GUI validation
authorized. No additional positive GUI execution authorized.**
[Research and evidence](2026-10-25-windows-boot-continuity-proposal.md).

Accept six guest samples, exact provider-bound startup-event stability, uptime/CIM
observations, hashes and seven offline tests within their stated scope. No documented
opaque boot GUID or cause of the earlier 1.179 ms discrepancy was established.
Blank screens remain separate and undiagnosed.

## Approved contract and policy semantics

Require exact equality of the reviewed startup-event channel/provider/event ID/
record ID/logged timestamp/StartTime. Separately require current qualified guest,
user/session/WinSta0/input desktop and fresh owner CONFIRM of host VM binding,
Basic Session and resources. Preserve the existing separate GUI-consent broker.

Retain raw CIM boot measurements; permit at most **100 ms** difference from the
reviewed baseline. This and the **2-second** elapsed discrepancy are initial Pivot
policy budgets, not Windows guarantees. Budget excess is continuity uncertainty,
not proof of reboot. Missing/malformed/ambiguous data or changed anchor/context
must Block; no undocumented system-information layout or time-rounding shortcut.

For each compared pair of points `(wall, uptime)`, in seconds, define
`D = abs((wall_last - wall_first) - (uptime_last - uptime_first))` and require
nonnegative elapsed values and `D <= 2`. Distinguish:

- Query-local: before/after each OS query (baseline and current separately).
- Cross-observation: reviewed baseline-before to current-before, potentially a
  much longer interval. Preserve both actual interval lengths and discrepancies.

Do not compare reconstructed absolute boot instants for equality. Legitimate clock
correction may conservatively Block; report uncertainty accurately. Changed event
data, log replacement/clear/ambiguous continuity, uptime reset, failed owner
acknowledgement or inaccessible/replaced interactive context must not grant authority.
Same-user adversaries/snapshot rollback remain outside this cooperative claim.

## Implementation, validation and packet

Integrate this contract into Windows attended preflight and fixed managed path
using actual event/CIM/Win32 uptime adapters. Cover stable repeats, observed drift,
inside/outside 100 ms, changed event fields, stale event plus reset, malformed/
unavailable event, clock discontinuity, user/session/desktop changes and inconsistent
bracketing. Validate real adapters without Avalonia or clipboard access. Retain
first failures and appropriate existing non-GUI checks; no unrelated source changes.

Derive a new immutable candidate from the last approved peer-corrected confirmation
packet. Retain its peer fix/regression, functional assertions and consent bytes.
Permit only reviewed boot logic and necessary regressions. Independently reconcile
modified paths, all source/input hashes, unchanged runner/assertions and historical
packets. Return diff, real adapter measurements, tests and candidate for parent review.

No positive owner command, deliberate reboot, display/power/integration change,
routine clipboard sentinel, blank-screen intervention or GUI retry. Native AOT and
Ubuntu GUI stay separate. No unattended authority, new protocol/dispatcher,
lifecycle automation or host fallback. Headless gates/G1/G2 closed; G3 held;
G4–G7/production Undo/Redo deferred. No product behavior/limits changes.
