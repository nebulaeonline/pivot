# Windows x64 Native AOT attended qualification result

Update: [parent accepts and closes this gate](2026-10-25-windows-native-aot-acceptance.md),
retaining the publish diagnostics as nonblocking and unresolved. No repeat needed.
The submission below retains its original review-stage context.

**Passed/0, independently reconciled; final parent acceptance pending.**
The single authorized attempt completed. No retry or subsequent GUI execution was
performed. [Authorization](2026-10-25-windows-aot-candidate-acceptance.md) and
[owner instructions](2026-10-25-windows-aot-attended-attempt.md).

## Evidence and provenance

Returned capture:
`artifacts/windows-native-aot/results/Pivot-Win11-x64/aot-capture-723d02279fec432284c2d65bdb9af704/`.
Exact accepted packet receipt:
`6989a33a4438fc049a67c6f9ae09f65f3f67c28d1b5ebb1d7bf515669163b7ec`.
Returned receipt/source/input manifests match the accepted packet byte-for-byte.
All fifteen accepted packet payloads still verify. Actual guest source-after is
unchanged; source/input inventory remains 537/265.

Fifty outer hash entries and 44 nested entries verify; **all 44 nested entries
overlap the outer inventory**, not 94 distinct files. Native and returned outer
origin-manifest hashes agree:
`9a200b7d1394ccf85adbe9fa2e45c12a6b9d5435ddaa89a315a569f9a9bd0188`.
Capture reports Passed/0 with no export errors; original runner and AOT wrapper
also report Passed/0. Independent verifier and result:
`artifacts/windows-native-aot/verify_returned_attempt.py` and
`artifacts/windows-native-aot/returned-attempt-verification.json`.

Native capture original:
`C:\PivotWork\aot-capture-723d02279fec432284c2d65bdb9af704`.
Underlying original report:
`C:\PivotWork\attended-b462ac31eb734f60adfd4929b4882912\report`.
All source/build/cache/publish/temp/fixtures remained guest-local; J: was exchange.
Console retains both current-condition CONFIRM and separate original GUI consent
prompts. Current tools, intended VM binding, user/session/input desktop and
event-anchored before/after boot continuity verify. No new clipboard sentinel.

## Real publish and application evidence

All seven runner steps exit 0, with their original logs retained: desktop lock,
.NET metadata, Rust metadata, native component discovery, locked release Rust
build, Native AOT publish and published desktop execution. Publish uses win-x64,
the unchanged NativeAot profile and `-warnaserror`; no warning matches are present.
Rust build took 43.929 s, publish 52.747 s and desktop workflow 15.117 s; wrapper
elapsed 119.189 s. These are one-run observations, not benchmark conclusions.

**Publish-log diagnostic caveat:** before “Generating native code”, `publish.log`
contains “The input line is too long.” and “The syntax of the command is incorrect.”
The origin of these messages is not established by the returned evidence. They
did not produce a nonzero publish exit or a warning match; actual native generation,
image verification and execution succeeded. Retain them for parent assessment.
Do not describe the log as diagnostic-free, silently suppress them or rerun to
remove them. No tooling fix or extra execution was attempted.

All ten published files are present and independently hash-verified, with exact
inventory and lengths recorded. Desktop and adjacent `pivot-session.exe` both
have x64 PE headers; no adjacent `Pivot.Desktop.dll` exists. Desktop hash:
`236bc678c5bf8c0ee5baccfd959e77fa79c622cadce19b3c7e2d82cbc551a121`;
adjacent Rust child hash:
`44561c5464fc3eacf2cc0331c7c83ced43b7ec490148ea8a898ba038ef7cc3a3`.

| Process | PID | Mode | Backend | Session | Final charge |
| --- | ---: | --- | --- | ---: | ---: |
| Main | 2576 | native-aot | Avalonia.Win32.WindowImpl | 1 | 0 |
| Clipboard/save peer | 3132 | native-aot | Avalonia.Win32.WindowImpl | 1 | 0 |
| Discard peer | 6240 | native-aot | Avalonia.Win32.WindowImpl | 1 | 0 |
| Clean peer | 1752 | native-aot | Avalonia.Win32.WindowImpl | 1 | 0 |

All four report the exact published executable path. Peer Rust process IDs are
4724, 6032 and 1300. Full unchanged workflow reports passed for editing, focus,
direct/rectangular clipboard, selection/navigation, four-direction fill,
aggregates, text, structured projections and bindings. Clipboard/save peer confirms
native receipt and Cancel handling; saved WB5 signature and payload SHA-256 verify
independently. Routed-event tests exclude physical input and native picker
certification; this is not expanded manual acceptance.

Strict scoped survivor snapshot is **Verified with no matches**, taken without
extra cleanup waits, exclusions or unrelated kills. No evidence errors, source
changes or continuity failures. No timeout/interruption occurred. Outer capture's
`GUI_executed:null` does not deny execution: actual native-image reports and original
runner establish it. Shared metadata preflight retains its historical managed
entry/disabled label and `GUI_executed:false`; the separate AOT preparation/run
records and pinned AOT entry establish the executed path.

## Size and timing observations

Desktop executable: 22,658,560 bytes. Runtime payload: 43,685,928 bytes. Full publish
including symbols: 229,101,404 bytes. Startup-to-ready: 1083.119 ms. Render timings
are retained in the application report and do not measure physical presentation.
No comparative performance or architecture-matrix claim follows from one VM run.

## Review boundary

Submit original evidence and this reconciliation for final Windows x64 Native AOT
acceptance. The one-attempt authorization is consumed; no repeat is authorized.
Windows managed/headless gates remain accepted/closed. Ubuntu GUI/AOT, ARM64/Rosetta,
unattended execution and VM orchestration remain separate. G1/G2 closed; **G3 held**;
G4–G7 and production Undo/Redo deferred. No product or accepted packet changes.
