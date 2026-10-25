# Avalonia collector opt-out — guest evidence and scoped correction

Subsequent [parent acceptance closes the correction gate and authorizes one new attended run after final verification](2026-10-25-scoped-optout-acceptance.md).
The pending-review statements below retain the original submission context.

2026-10-25. [Authorized conditional correction](2026-10-25-managed-cleanup-investigation-direction.md).
**Actual guest build supports the private setting. Correction/candidate ready for
parent review; no new managed GUI run is authorized or executed.** G3 held.

## Guest experiment results

The owner ran the [single non-GUI build experiment](2026-10-25-avalonia-collector-build-experiment.md)
from the accepted frozen source/toolchain. Release build exited 0 with zero warnings
and errors. Source remained unchanged. Opt-out was absent in the parent environment
and present as `1` only in the experiment's private child environment; NCrunch was
also absent. No machine/user/project policy was changed.

| Observation | Result |
| --- | --- |
| Build time | 2026-10-25 15:38:01–15:38:17 UTC |
| Actual target evidence | Before/after AvaloniaStats in Pivot.Avalonia and Pivot.Desktop |
| Opt-out values | All four records: 1 |
| MSBuild license-item counts | All four records: 0; cached tickets not inspected |
| Community/Trial override messages | None in retained normal-verbosity build output |
| Scoped process timeline | 62 records; no collector in any record |
| Pre-existing collector baseline | None observed |
| Current-build creation / immediate presence | None observed / none observed |
| Final bounded observation | None observed, 15.814 seconds after build completion |
| Observer | Completed normally; no collector or unrelated process killed |
| Native-origin report hashes | 18 verified; transfer-manifest digests match |

This is consistent with the configured build honoring opt-out. It does not prove
an exact license tier: item counts omit cached sign-in/library tickets. The
inspected task's Community/Trial branch would emit an override message, which was
not observed, and no collector was observed. WMI creation polling/snapshots cannot
prove absence of every possible sub-poll transient. No payload, cached license,
personal content or unrelated process was inspected. The earlier PID 2888 was not
observed in this later baseline; its original presence remains historical fact,
not a claim of permanent survival or a diagnosed hang.

Native original:
`C:\PivotWork\collector-build-358dad6175764e80b01015231352a8cc\report`.
Returned evidence lives under
`artifacts/avalonia-collector-investigation/results/Pivot-Win11-x64/collector-build-358dad6175764e80b01015231352a8cc/`.
The experiment receipt, helper hashes, unchanged original source receipt, raw
process summary, phase coverage, task evidence and final duration were independently
verified/recomputed. The strict GUI survivor gate was not run or waived here.

[Guest reconciliation](../../artifacts/avalonia-collector-investigation/guest-build-review.json),
[raw build/result](../../artifacts/avalonia-collector-investigation/results/Pivot-Win11-x64/collector-build-358dad6175764e80b01015231352a8cc/result.json).

## Minimal correction

`windows_attended_gui.clean_environment()` now assigns
`AVALONIA_TELEMETRY_OPTOUT='1'` in its returned private validation environment.
The fixed runner's existing environment scope makes it available to restore/build
children; the setting is inherited only within that validation process tree and
is build-time-only in the dependency. Caller environment is restored afterward.
No ordinary app-launch default, machine variable, project property, source dependency,
registry, licensing configuration or user setting changes.

Strict scoped survivor checking, its timing, boot continuity, live peer capture,
native roots, owner CONFIRM and independent GUI consent are byte-for-byte unchanged
in the candidate. No collector exclusion, additional cleanup wait or termination
was added. A future override/failure would still be caught rather than converted
to a pass.

The added non-GUI regression exercises absent and pre-existing caller opt-out
values, demonstrates no mutation by environment preparation, verifies the fixed
runner sees `1` with stale authority/self-check variables removed, and verifies
exact caller restoration after a simulated refusal. Real terminal streams remain
unchanged; no consent response or capability is supplied. Fourteen preflight tests
pass. Earlier experiment plumbing checks remain five passes with original failed
MSBuild-format fixture and corrected logs preserved. Actual guest warning-free
build supplies compilation evidence; no full CI or GUI acceptance was replayed.

## Immutable correction candidate

`artifacts/avalonia-collector-investigation/windows-scoped-optout-review-20261025/`

Receipt SHA256:
`ffb8ab507dc709ec0ae9cc8b8502e8fae79e9d10c819e7454f7985cae0b60571`.

Derived directly from the previously approved boot-corrected packet. Independent
verification covers **533 source entries / 262 inputs / twelve packet files**.
Only the wrapper environment assignment and necessary regression method differ;
no source files were added. All other source bytes, functional UI assertions,
strict survivor check, consent, boot adapter and peer correction remain unchanged.
Original line endings, packets and all Blocked results remain preserved. The
candidate inherits enabled code state, not new execution authority; the working
wrapper stays disabled.

Twenty-two applicable frozen non-GUI checks pass: thirteen preflight tests (the
disabled-only case is inapplicable to this inherited-enabled snapshot), eight boot
contract tests and the peer lifecycle regression. The worktree includes that
disabled-path check among its fourteen passes. No approval test is reclassified.

[Minimal source diff](../../artifacts/avalonia-collector-investigation/scoped-optout.diff),
[candidate verification](../../artifacts/avalonia-collector-investigation/scoped-candidate-verification.json),
[frozen regression log](../../artifacts/avalonia-collector-investigation/scoped-candidate-frozen-tests.log).

## Review boundary

Return actual guest build/process/licensing evidence, scope regression, minimal
diff and candidate for parent review. The previous application/backend/continuity
evidence remains accepted; overall managed cleanup acceptance remains Blocked/2
until a separately approved complete confirmation satisfies it. No new GUI command
is issued here. Do not reuse the consumed authorization or rerun until green.

No manual clipboard recheck absent a specific changed condition/contrary observation.
Native AOT/Ubuntu GUI stay separate; no unattended authority, second protocol,
dispatcher, lifecycle automation or host fallback. Headless gates/G1/G2 closed;
G3 held; no workbook/graph/persistence/protocol/quota/product changes.
