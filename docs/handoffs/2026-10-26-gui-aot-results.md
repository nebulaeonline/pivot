# Six-RID GUI and Native AOT results

Current disposition (2026-10-27): six-RID headless, managed GUI and Native AOT
execution validation are accepted and closed. Clipboard reliability remains an
open issue; G3's scheduling hold is released. Pending/open/held statements below
retain their historical context. See [final acceptance](2026-10-27-six-rid-validation-acceptance.md).

Subsequent diagnostic evidence: Windows ARM64 passed the full AOT workflow in
`20261027T144914Z-cab4acb5/`. The original matrix and failures below remain
unchanged. No behavioral fix was made; the intermittent clipboard issue remains
unresolved and AOT closure awaits parent disposition. See
[diagnostic results](2026-10-27-arm64-aot-diagnostic-results.md).

Parent review on 2026-10-27 accepted and closed managed GUI on all six targets
and accepted the five passing AOT targets. Windows ARM64 AOT remains Failed and
the AOT milestone stays open. See [closure and focused next scope](2026-10-27-managed-gui-closure-and-arm64-aot-scope.md).

All attended runs have been collected. Managed GUI passed on all six designated
targets. Native AOT passed on five; Windows ARM64 failed its UI self-check after
successful native publication and launch. The GUI/AOT workstream is not fully
closed. Owner suggestions are reserved for the parent discussion; no additional
GUI run or functional change was made while reconciling these results.

| RID | Managed GUI | Native AOT |
| --- | --- | --- |
| win-x64 | Passed | Passed |
| linux-x64 | Passed | Passed |
| win-arm64 | Passed | **Failed — clipboard UI self-check** |
| linux-arm64 | Passed | Passed |
| osx-arm64 | Passed | Passed |
| osx-x64 | Passed under Rosetta | Passed under Rosetta |

Original collected evidence under `artifacts/validation-matrix/`:

- Managed, all six Passed: `20261027T025246Z-deb1324f/`
- AOT, five Passed / one Failed: `20261027T025246Z-b82c5b95/`

Every passing result has four original application/peer reports, real Win32,
X11 or native Mac window backends, matching process architecture, zero final
charges and no reported owned-process survivors. AOT passing results also retain
the original native publish/image/peer assertions and publish logs. Publication
alone is not counted as execution success.

All four native-Mac mode/RID combinations recorded the fixed machine desktop
lock `/private/var/tmp/pivot-native-desktop.lock`. Kernel-lock unit tests previously
verified exclusion across independent processes, including release on process
exit. The same guard protects native ARM64 and Rosetta suites across invocations;
no GUI overlap experiment or unrelated-process inspection was performed here.

## Windows ARM64 AOT failure

The compiler, Rust release build and native publish succeeded without publish
warnings. The actual published ARM64 executable launched in Windows graphical
session 1 and reported execution mode `native-aot` and process architecture
`arm64`. It then emitted a failed report containing:

```text
System.NullReferenceException: Object reference not set to an instance of an object.
at Pivot.Desktop.MainWindow.<ClipboardChecklistAsync>d__6.MoveNext() + 0x29a0
```

The subsequent frames are `ClipboardChecklistAsync`, `SmokeAsync` and `StartAsync`.
There is no source line or identified clipboard sub-step in this AOT stack, so
the underlying cause is unresolved. The native executable returned exit code 0,
but its failed JSON report correctly caused the AOT runner and matrix to fail.

The runner's first missing acceptance-field message names `rectangular_fill`;
that does not establish a fill defect. The UI smoke sequence aborted earlier in
the clipboard checklist before producing the later success fields. No failed
assertion was weakened, no retry-to-green was performed, and neither four-peer
completion nor zero-charge acceptance is claimed for this failed AOT run.

Specific original failure evidence:

```text
20261027T025246Z-b82c5b95/win-arm64/console.log
20261027T025246Z-b82c5b95/win-arm64/aot/20261027T023453679066Z/summary.json
20261027T025246Z-b82c5b95/win-arm64/aot/20261027T023453679066Z/desktop-ui/results.json
```

Windows ARM64 managed GUI passed, as did all other Native AOT targets. Those
results narrow the observed configuration, but do not prove an ARM64 compiler,
trimming, clipboard timing or application cause.

## Review boundary

The accepted six-RID headless milestone remains closed and was not rerun. The
existing Linux x64 system-runtime abort remains separately recorded; selected
private SDK qualification is unchanged. G1/G2 stay closed; G3 remains held.
No workbook, graph, formula, persistence, protocol, product-limit or owner-suggested
feature changes were included in this results reconciliation.
