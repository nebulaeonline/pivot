# Windows boot continuity guard — implementation and adapter validation

Subsequent update: [actual guest adapter validation is complete](2026-10-25-boot-guard-guest-adapter-results.md).
All six samples/24 comparisons verify with native-origin and source hashes; no
positive GUI execution is authorized. Pending statements below retain preparation history.

2026-10-25. Implements the [reviewed contract](2026-10-25-boot-continuity-implementation-direction.md).
**No positive GUI execution authorized or performed. Actual guest adapter result
is pending; host and non-GUI validation complete.** No clipboard/settings/reboot work.

## Contract integration

`windows_boot_continuity.py` uses documented, narrowly filtered event/CIM queries
and native GetTickCount64 observations. Its baseline is the accepted guest sample:
Kernel-General event 12, System record 2910, full event/startup timestamps and raw
CIM/uptime/wall observations. Static baseline is evidence only; the runner still
requires live qualified guest/session/desktop checks, owner CONFIRM and independent
existing GUI consent. No environment/profile file can mint execution authority.

Exact event-anchor equality is required. CIM variation is computed by subtracting
timezone-aware datetimes, with a 100 ms maximum, avoiding epoch-float error at the
boundary. Within each OS query and baseline-to-current observation, in seconds:
`D = abs(delta_wall - delta_uptime) <= 2`, with nonnegative elapsed values.
Diagnostics separately report baseline-query, current-query and baseline-to-current
interval lengths and discrepancies. Absolute reconstructed boot times are not equated.
Budgets are Pivot policy, not Windows guarantees. Excess/invalid data reports
continuity uncertainty, not an assertion that reboot occurred. Changed startup
anchor and changed execution context have distinct mismatch classifications.

Preflight records raw before/after measurements, review-baseline and invocation
continuity results. The fixed managed path repeats those checks immediately before
and after the runner. Non-boot identity fields, including current process/user/
session/desktop, remain exact within an invocation. The reviewed guest firmware
UUID, hostname, User, session 1 and WinSta0/Default are checked separately. Firmware
identity is not mistaken for Hyper-V management VM identity; host binding/resources
remain owner-attested. Full fractional CIM string equality is removed everywhere.

The report path is emitted once when its directory is created, so early continuity
refusals can return raw diagnostic reports through the unchanged exact-path exporter.
Missing/malformed events, failed queries, uptime reset, clock discontinuity, budget
excess, changed VM/context and failed owner acknowledgement remain Blocked. Real
desktop adapter failures remain refusal; no desktop is inferred from environment.
Snapshot rollback/same-user malicious mutation remain outside the cooperative claim.

## Non-GUI evidence

- Eight new production comparator/integration tests pass, covering the prior
  fractional difference, submillisecond and exact budget boundaries, all event
  anchor fields, missing/malformed data, stale event/reset, clock discontinuity,
  query versus long-interval diagnostics and changed user/session/desktop/VM.
- Thirteen existing preflight tests pass with proper reviewed-measurement fixtures;
  the accepted non-GUI peer lifecycle regression also passes.
- Managed desktop build succeeds with zero warnings/errors. No application source
  or functional assertions changed in this boot integration.
- Frozen candidate passes 21 applicable non-GUI tests (twelve preflight, eight new,
  one lifecycle). The disabled-only test is inapplicable because the candidate
  inherits the old enabled code state; it does not grant new execution permission.
  A real noninteractive invocation returns Blocked/2 before any GUI/build work.

Final host production-adapter capture:
`host-adapters-final/continuity-adapters-31fff0c017d44d349add519c609e8651` under
`artifacts/windows-boot-guard-implementation/`. Six within-host comparisons verify;
all six reviewed-guest comparisons refuse with anchor-mismatch, as expected on a
different machine. WinSta0/Default/session 1 metadata was readable. Host evidence
does not qualify the guest. The earlier host adapter capture remains preserved;
the final probe adds timeout-error reporting before the final capture. No passing
product gates were replayed.

## New immutable candidate and exact delta

`artifacts/windows-boot-guard-implementation/windows-boot-continuity-review-20261025/`

Receipt SHA256:
`905350c373552132becc0547ef467e368d690033f0bab416465e9909752702e0`.

Independently verified **533 source entries / 262 inputs / eleven packet files**.
Derived from the approved peer-corrected confirmation snapshot. Modified paths are
the Windows wrapper and its preflight regression fixtures. Added paths are the
production continuity module, read-only adapter probe and comparator/integration
tests. All other bytes, including peer lifecycle fix/test, unchanged ui_smoke.py,
consent broker, exporter and functional assertions, match the approved parent
snapshot. Original line endings and historical packets/results remain untouched.
The working wrapper stays disabled; candidate's inherited enabled constant is not
positive-run authority. No new GUI launch command is issued.

[Candidate verification](../../artifacts/windows-boot-guard-implementation/candidate-verification.json),
[implementation diff](../../artifacts/windows-boot-guard-implementation/boot-continuity.diff).

## Actual guest adapter collection only

Run from ordinary Command Prompt inside Pivot-Win11-x64:

```bat
J:\coding\pivot\artifacts\windows-boot-guard-implementation\run-windows-adapter-check.cmd
```

This imports the new candidate to a fresh native-local directory, verifies its
pinned receipt/files, then calls **only windows_continuity_probe.py**. That probe
collects actual production event/CIM/uptime/desktop and qualified guest adapter
results six times. It never calls the GUI runner, requests GUI consent, touches
clipboard, reboots or changes settings. Existing destination refuses, not overwrite.

Original reports/hashes stay under C:\PivotPreflight; verified exports return to
`artifacts/windows-boot-guard-implementation/guest-adapters/`. Any Blocked comparison
remains its true result, not an automatic threshold adjustment. Tell the repository
agent when complete; do not transcribe data or repeat to fix transfer failures.

Return actual guest observations, source delta, tests/build and candidate for
parent review before any new positive confirmation authorization. Native AOT/
Ubuntu GUI stay separately gated. Blank screens remain separate/undiagnosed.
No new protocol, unattended authority, dispatcher or host fallback. G1/G2 and
headless gates closed; G3 held; product/history work unchanged.
