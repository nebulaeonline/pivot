# Ubuntu x64 first attended managed-GUI result

Update: [parent accepts successful application evidence and retains overall Blocked](2026-10-25-ubuntu-first-managed-review.md).
Only bounded non-GUI observer investigation/correction is authorized next. No replay.
Original result and analysis below remain unchanged within their stated scope.

**Overall Blocked/2; original functional UI runner exited 0.** Preserve both facts.
The one authorized attempt is consumed. No retry, packet modification or changed-
source execution was performed. Return for parent direction; qualification is open.
[Authorization](2026-10-25-ubuntu-enabled-candidate-acceptance.md),
[owner command](2026-10-25-ubuntu-attended-managed-attempt.md).

## Verified actual execution

The unchanged Rust/.NET build and UI workflow completed; original UI runner exited
0. All four actual application reports are passed, managed mode, executable
`/usr/lib/dotnet/dotnet`, with independently captured `Avalonia.X11.X11Window` and
zero final charges. This establishes actual X11 implementation execution in the
reviewed GNOME/Wayland environment; sockets alone were not used to infer backend.
Linux process-session fields are absent/null, not translated into logind IDs.

| Report | PID | Final charge | Rust child report PID |
| --- | ---: | ---: | ---: |
| Main | 32038 | 0 | Not supplied by main report |
| Clipboard/save | 32121 | 0 | 32176 |
| Discard | 32195 | 0 | 32250 |
| Clean | 32269 | 0 | 32324 |

Original assertions cover editing, selection, rectangular clipboard, Fill,
aggregates, text, structured results/bindings, Save/reopen and close lifecycle.
Saved WB5 signature and payload hash verify independently. Actual source-after is
unchanged. Final scoped snapshot is Verified/empty with no snapshot errors;
before/after Linux context remains verified. These successful findings do not
override earlier observer errors or incomplete main-process identity.

## Why overall qualification is Blocked

`run.json` retains two evidence errors:

- Permission denied reading `/proc/30968/environ` during observation.
- `Missing/reused application PID` during runtime report correlation.

There are 97 captured process identities. The main application's direct process
record (32038) is absent. It appears as an ancestor of observed peers and Rust
child 32098, but that is not a substitute for its required executable/cgroup/display
observation. All three peer direct records exist, with distinct start ticks and
their matching observed Rust children. No PID reuse is demonstrated for the main;
the generic error covers missing or reused identity, and this evidence shows missing.

Read-only source analysis identifies a likely deterministic scope gap: original
`ui_smoke.py` starts main with `dotnet clients/.../Pivot.Desktop.dll` relative to
the frozen source cwd. Observer selection recognizes external runtime executables
only when their command contains the absolute private invocation root. It does
not inspect cwd; absolute peer commands are captured. Runtime command validation
also expects the absolute DLL path. This is a source-based explanation, not a
reconstructed direct observation of the missing main command.

PID30968 is recorded at another sample as a scoped Rust compiler, UID1000/start
8583785, compiling fixedbitset under private roots. The retained permission denial
occurred on an environment read; successful metadata from another sample does not
erase it. The evidence does not establish whether the denial was an exit/exec race,
a security restriction or another cause. Do not label it transient/harmless or
broaden the accepted preexisting-nonactor rule to ignore it.

No tool/app defect is established by these observer gaps. No observer assertion is
weakened and no overall pass is claimed. The final empty snapshot establishes the
reported final scope, not complete historical observation or lossless tracing.

## Provenance and native originals

Accepted packet receipt:
`0c2b9817c0ef685038581f0deecc82d7d8c049d9e4357907e4b47cab3c4095aa`.
All ten payloads verify; returned 544-source/271-input manifests match packet bytes.
Working-tree positive gate remains false; original accepted packet is unchanged.

Capture:
`artifacts/ubuntu-managed-implementation/captures/Pivot-Ubuntu-x64/ubuntu-capture-173432473e3f419fa263c5828051bac2/`.
Native-report export:
`artifacts/ubuntu-managed-implementation/results/Pivot-Ubuntu-x64/ubuntu-attended-1b0259e0cca046bfa6057b5f0d44b87e/`.
Original directories are corresponding `~/pivot-preflight/ubuntu-capture-.../report`
and `~/pivot-preflight/ubuntu-attended-.../report` on guest ext4.

Thirty capture hash entries and 28 nested report entries verify; all 28 nested
entries overlap the capture inventory. The separate direct report export is an
identical copy, not independent additional execution evidence. Transfer receipts
match native-original digests; no export errors are reported.

Capture origin SHA-256:
`21a49345814702548c646502da832e3a99304ffecce68fc00034eaa2a76fa642`.
Native-report origin SHA-256:
`e0ed2814f74863b42d29c65ec5041bcdf57b0818bde7a5b2141a550c65d476bc`.
Original console retains builds, functional reports, prompts and util-linux recorder
exit2. Capture is Blocked/2; underlying UI runner is exit0. PreflightPassed describes
setup, not overall acceptance. `GUI_executed:null` is not a denial of execution;
actual four reports prove the GUI ran.

Independent verifier:
`artifacts/ubuntu-enabled-candidate/verify_first_gui_result.py`.
Reconciliation:
`artifacts/ubuntu-enabled-candidate/first-gui-result-verification.json`.
It reads existing evidence only. No additional test or process was launched.

## Parent direction requested

Review successful functional/backend/charge/continuity/final-snapshot evidence while
retaining overall Blocked. Proposed next bounded work is non-GUI investigation of
cwd-aware fixed managed-process selection/canonical command validation, and the
scoped compiler environment-read failure. Preserve strict actual identity and
first-failure evidence; no blanket permission-error suppression, extra waits,
unrelated kills, skipped assertions or alternate runner.

Any correction must be independently reviewed in a new immutable packet before
another positive attempt is authorized. No new guest work or repeated clipboard
sentinel is requested now. Ubuntu GUI acceptance remains open; Ubuntu AOT and
unattended execution remain unauthorized. Windows/Ubuntu headless gates stay closed;
ARM64/Rosetta/orchestration separate. G1/G2 closed; G3 held/unstarted. No product changes.
