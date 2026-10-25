# Avalonia collector — scoped non-GUI build experiment

Subsequent update: [actual guest build evidence verified and conditional scoped
correction prepared for review](2026-10-25-scoped-avalonia-optout-correction.md).
Do not replay the experiment command below; it retains the original one-build instructions.

2026-10-25. [Authorized bounded investigation](2026-10-25-managed-cleanup-investigation-direction.md).
**Actual guest build evidence pending. Validation wrapper opt-out is not changed;
no additional GUI execution authorized.** The prior Blocked capture stays original.

## Dependency and licensing branch

The inspected 11.3.2 package declares upstream commit
`777f975b0a0cecf0311273711d56697212c558c0`. The official
[task source](https://raw.githubusercontent.com/AvaloniaUI/Avalonia.BuildServices/777f975b0a0cecf0311273711d56697212c558c0/BuildTask/AvaloniaStatsTask.cs)
checks the documented opt-out in its default branch; Community/Trial paths emit
specific high-importance messages and override it. Importantly,
[tier resolution](https://raw.githubusercontent.com/AvaloniaUI/Avalonia.BuildServices/777f975b0a0cecf0311273711d56697212c558c0/BuildTask/AccelerateTierHelper.cs)
also considers cached sign-in/library tickets. **Zero MSBuild license items alone
does not prove no-license mode.** Do not read/export those tickets or keys.

The experiment records only license-item counts and opt-out value around the actual
AvaloniaStats target, raw normal-verbosity build output, override-message presence
and observed process behavior. It does not infer exact licensing tier solely from
counts. The ordinary build task resolves its own configured licensing as usual;
the diagnostic code never inspects license/telemetry payloads or personal files.

## Controlled build and process observations

One fresh NTFS work directory, unique private NuGet/.NET/cache/temp roots, and the
existing frozen 533-source/262-input packet are used. Only the experiment's child
environment sets `AVALONIA_TELEMETRY_OPTOUT=1`. No machine/project policy or
application defaults change. Supplemental targets outside source record counts
before/after AvaloniaStats without changing its logic. The fixed command builds
Pivot.Desktop Release with existing assertions untouched; no program is launched.

A temporary WMI observer and snapshots select **only** dotnet collector commands
under Pivot's C:\PivotWork attended/collector-build roots. Rows include PID, parent,
creation UTC/ticks and narrow command path. PID plus creation ticks distinguishes
reuse. Initial collectors, new current-build creation observations, an acknowledged
immediate post-build snapshot, and a fixed **15-second** diagnostic interval are
retained separately. Prior-root collectors are reported separately from the fresh
build root, never killed or omitted from historical snapshots.

The kernel process-start trace was denied with this host token; no privilege/group
change was made. WMI intrinsic creation polling works without that privilege.
Its WITHIN-1-second polling plus roughly 0.5-second snapshots is **not lossless
kernel tracing**; a sub-poll transient could be missed. State conclusions as
observed behavior combined with actual task evidence, not proof of every absence.
The fixed diagnostic interval does not retroactively change the GUI cleanup gate.
If observer setup/required snapshots fail, retain Blocked rather than building
without process evidence or extending observation until green.

Build timeout is 900 seconds. Python's subprocess timeout can end its own timed-out
build child; it never targets a collector or unrelated process to obtain success.
A timeout is retained as Blocked, with no assertion of descendant cleanup. Normal
completion stops only the diagnostic observer; no collector/unrelated kill occurs.

## Plumbing validation and packaging

Five non-GUI tests pass: PID reuse/pre-existing classification, missing start
identity refusal, actual read-only WMI startup/shutdown, real MSBuild metadata
hooks on a synthetic SDK project and actual creation/snapshot matching for a
three-second console fixture. That fixture is named like the collector solely to
exercise the filter; it contains no Avalonia or telemetry code. The initial hook
test failed because an item-list expression was concatenated in a scalar field;
it was fixed via a separate scalar property. Original/final test logs are retained.

Experiment packet:
`artifacts/avalonia-collector-investigation/windows-collector-experiment-20261025/`.
Receipt SHA256:
`4cca0aa487e08fdb3dea038d2e1d5047e927dfa8dc6d572c57aecde3e75f4c94`.
Eighteen receipt-covered files include an unchanged copy of the accepted source
packet and experiment/helper files. The inherited enabled wrapper is imported only
for native-root/environment helpers; this script never invokes its GUI entry path.
No new managed confirmation packet or GUI execution permission is created.

## Owner command — ordinary CMD in Pivot-Win11-x64

```bat
J:\coding\pivot\artifacts\avalonia-collector-investigation\run-windows-build-probe.cmd
```

The command loads the installed native x64 developer environment, copies/verifies
the experiment to a fresh local directory and runs one non-GUI build. No CONFIRM,
GUI-consent, clipboard test, administrator shell, installation or settings change.
Cold project restore is ordinary build work. Existing input destination refuses.

Native reports remain under a unique C:\PivotWork\collector-build-*\report path.
Build output, scoped process timeline, target evidence, dependency README/nuspec
and executable hashes, source checks and exact build exit are hashed locally and
returned to `artifacts/avalonia-collector-investigation/results/Pivot-Win11-x64/`.
`EvidenceCollected` means evidence is available for review, **not** a cleanup or
GUI pass. Collector presence remains a result to investigate, not an ignored row.

Tell the repository agent when complete. Stop on failure or transfer error; do
not replay the build to repair copying or produce a better result. Retain native
originals. Only if actual evidence supports the setting may the narrowly scoped
wrapper correction and regressions be prepared for parent review. No new GUI
command before separate authorization. Native AOT/Ubuntu GUI remain gated; no
dispatcher, new broker, lifecycle automation or host fallback. G3 held.
