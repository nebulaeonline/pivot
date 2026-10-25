# First Windows guest managed UI run — assertions pass, evidence blocked

Subsequent [parent review accepts functional execution and authorizes only diagnostic correction/preparation](2026-10-25-windows-managed-functional-acceptance.md).
Overall Blocked/2 is retained; no additional positive execution is authorized.

2026-10-25. **Preserve the first overall result as Blocked/2.** The unchanged
managed UI runner exited 0, and main/three-peer assertion reports pass, but peer
graphical-backend diagnostics are missing. This is not full managed-gate acceptance.
No rerun has been performed or requested. Native AOT stays gated; G3 held.

## Execution and independent verification

The owner executed the [single approved attended command](2026-10-25-windows-managed-execution-packet.md)
inside Pivot-Win11-x64. Console records show preflight CONFIRM and the separate
existing GUI consent prompt; application evidence establishes actual execution.
The packet receipt is
`2055df34ffc0cfdff9a3951ce3e8798976d49710fb50b0f696619e519c345b7f`.
Returned source/build manifests match the immutable 529-source/258-input packet.
All eight packet-file hashes, 27 outer native-origin report hashes, transfer
manifest digest and 21 nested native-report hash entries verify. Nested entries
overlap the outer inventory; these are not 48 distinct reports.

| Observation | Result |
| --- | --- |
| Runner start / elapsed | 2026-10-25 14:21:22 UTC (10:21:22 EDT) / 146.067 seconds |
| Native work | C:\PivotWork\attended-4251ab2aad6a429ab0c23b9e5077278a |
| Boot/session | Reviewed October 25 00:37:42 -04:00 / session 1 |
| Preflight/post identity | Equal; WinSta0/Default, user User, wrapper PID 10216 |
| Managed UI runner | Exit 0; existing assertions passed |
| Main application | PID 3596, session 1, Avalonia.Win32.WindowImpl |
| Nested peers | PIDs 7908 / 6256 / 3208, all session 1 |
| Peer Rust children | PIDs 9524 / 8952 / 2220 |
| Remaining charge | Main and all three peers: 0 |
| Source after / scoped survivors | No changes / none |
| Wrapper/capture | Blocked/2: missing backend diagnostics in peer reports |

Main report records the unchanged clipboard, Fill, Save/reopen, editing, selection,
aggregate, text and structured-result/binding assertions as passed. Clipboard-save
peer reports native clipboard receipt and close-Cancel checks true; discard/clean
peers also pass. Peer/application process IDs are distinct. These are real managed
application/peer reports, not mocked preparation tests. They do not certify physical
keyboard input, native pickers, Native AOT or Linux GUI. Timings are diagnostic,
not a benchmark claim. The Python console tee is not a complete build transcript.

## Diagnostic defect and retained boundary

Main `graphical_backend` is present. All three peers omit that field while retaining
session/process and passing cleanup evidence. `ClipboardPeerAsync` constructs the
peer report inside its `Closed` handler and evaluates `PlatformImpl` there. The
platform implementation is no longer available at that point; the resulting null
is omitted by the report's JSON null policy. This is consistent across all peers.

The wrapper correctly refuses to infer peer backend from the main window. Its
error text says “Missing actual graphical backend/process evidence”; actual peer
process IDs are present, so the missing component here is specifically backend.
No UI assertion failed, but the broader evidence gate remains Blocked. Do not
rewrite that status or waive the check simply because the functional suite passed.

Proposed narrow correction for review: capture the peer's actual platform type
while the window is alive, before closing, and serialize that captured value from
the existing Closed handler. Keep the cleanup assertions/report timing unchanged.
Any correction, new packet and diagnostic-confirmation run need separate review;
the one-run authorization has been used. No patch, new packet or additional run is
implied by this results handoff. Preserve all original packets/reports unchanged.

No post-run machine audit of host clipboard/configuration was collected; the
accepted owner observations and unchanged guest boot/session remain the stated
isolation evidence. No repetitive sentinel test is requested without a concrete
reason. Do not extend this result into unattended or perpetual authority.

## Review evidence

- [Independent reconciliation](../../artifacts/windows-managed-preparation/first-managed-result-review.json)
- [Original capture](../../artifacts/attended-guest-preflight/results/Pivot-Win11-x64/preflight-capture-70800ea234f1408dba2d495297b70ddd/capture.json)
- [Managed wrapper result](../../artifacts/attended-guest-preflight/results/Pivot-Win11-x64/preflight-capture-70800ea234f1408dba2d495297b70ddd/native-report/managed-run.json)
- [Main application report](../../artifacts/attended-guest-preflight/results/Pivot-Win11-x64/preflight-capture-70800ea234f1408dba2d495297b70ddd/native-report/managed/results.json)

Return for parent review of the result and proposed diagnostic correction. Existing
headless gates/G1/G2 stay closed; no full CI repeats, product changes or G3 work.
