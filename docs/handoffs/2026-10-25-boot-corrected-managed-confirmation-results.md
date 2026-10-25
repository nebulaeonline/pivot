# Boot-corrected managed confirmation — application passes, collector remains

Subsequent [parent review accepts application/backend/continuity and authorizes only non-GUI collector investigation](2026-10-25-managed-cleanup-investigation-direction.md).
Overall Blocked/2 remains unchanged; no additional GUI run is authorized.

2026-10-25. **Overall first outcome remains Blocked/2**, now solely because the
scoped survivor snapshot contains one Avalonia build-services collector. The full
unchanged UI runner exited 0. All four graphical backend/session reports and zero
charges are present. Boot/context continuity verifies. No retry or process kill.

## Verified execution

The owner executed the [one newly approved attended command](2026-10-25-boot-guard-managed-confirmation-instructions.md).
The immutable 533-source/262-input packet matches receipt
`905350c373552132becc0547ef467e368d690033f0bab416465e9909752702e0`.
All eleven packet files verify. Returned source/input manifests match the packet.
The 31 outer native-origin hashes, 25 overlapping nested entries and both transfer
manifest digests verify; overlapping entries are not 56 distinct reports.

| Observation | Result |
| --- | --- |
| Existing managed runner | Exit 0; unchanged functional assertions passed |
| Main/peer PIDs | 6928 / 6720 / 4480 / 5024 |
| Main/three-peer graphical backend | Each independently reports Avalonia.Win32.WindowImpl |
| All GUI sessions | 1 |
| Final application/peer charges | Four zeros |
| Pre/post boot and context | Verified, independently recomputed |
| Source after execution | No changes |
| Runner duration | 78.992 seconds (diagnostic, not benchmark) |
| Wrapper/capture | Blocked/2; one scoped collector process present |

Clipboard/Fill/Save/Open, editing, selection, aggregates, structured results/bindings
and close-state checks remain intact. Clipboard-save peer reports native-format
receipt and Cancel evidence. Actual independent peer processes are verified.
The peer live-capture correction produced the required backend fields; no missing
backend is inferred from the main report. The new boot contract also passed live
before/after checks without weakening the policy or repeating a sentinel.

## Scoped survivor and diagnosis

The snapshot records PID **2888**, `dotnet.exe`, with command line referencing:

`C:\PivotWork\attended-2a37b91c258046ad83795e478816a623\nuget\avalonia.buildservices\11.3.2\tools\netstandard2.0\Avalonia.BuildServices.Collector.dll`

This is a build dependency process under the unique private run root, not one of
the four reported GUI PIDs. The snapshot establishes presence at collection time;
it does not establish whether the collector later exited, hung or transmitted
anything. No later process state or root cause of its lifetime was collected.
It must not be silently excluded to convert the run to Passed.

The installed same-version package README identifies build-time telemetry and
documents `AVALONIA_TELEMETRY_OPTOUT=1`. Its nuspec records upstream commit
`777f975b0a0cecf0311273711d56697212c558c0`. The corresponding official
[README](https://github.com/AvaloniaUI/Avalonia.BuildServices/tree/777f975b0a0cecf0311273711d56697212c558c0)
and [build-task source](https://raw.githubusercontent.com/AvaloniaUI/Avalonia.BuildServices/777f975b0a0cecf0311273711d56697212c558c0/BuildTask/AvaloniaStatsTask.cs)
show opt-out handling and background collector launch. The task respects opt-out
in its default/no-license branch; Community/Trial cases override it. Do not assume
all licensing modes honor the variable or infer uncollected guest licensing state.
This source investigation identifies a supported candidate remedy, not a proven
corrected build or a byte-level audit of the guest's dependency DLL.

## Proposed next bounded correction for review

Review setting the documented opt-out explicitly in this validation wrapper's
private build environment, then verify with a non-GUI build that the applicable
configured dependency does not start a collector. Retain the strict survivor check
and all assertions. Do not broaden this into machine-wide settings or project-wide
telemetry policy without explicit scope. Preserve first build/process evidence.

No opt-out change is applied by this review. No new packet or positive command is
issued. The current one-run authorization has been consumed. Any source/environment
correction, new packet and another attended confirmation require review. Do not
rerun until green, kill unrelated processes, waive cleanup or relabel Blocked.

## Review evidence and boundaries

- [Independent result reconciliation](../../artifacts/windows-boot-guard-implementation/managed-confirmation-review.json)
- [Original capture](../../artifacts/attended-guest-preflight/results/Pivot-Win11-x64/preflight-capture-1440268eef1241a5a8f6b2c3c4d02ee9/capture.json)
- [Managed wrapper result and survivor](../../artifacts/attended-guest-preflight/results/Pivot-Win11-x64/preflight-capture-1440268eef1241a5a8f6b2c3c4d02ee9/native-report/managed-run.json)

Return for parent review: genuine application/backend/continuity evidence is
complete, while overall cleanup acceptance remains Blocked. Original earlier
missing-backend and timestamp-refusal captures remain unchanged. Python tee is not
a full native build transcript; rely on actual reports and their stated limits.
No clipboard recheck requested. Native AOT/Ubuntu GUI stay separately gated;
blank screens remain independent/undiagnosed. No unattended authority, dispatcher,
lifecycle automation or host fallback. Headless gates/G1/G2 closed; G3 held.
