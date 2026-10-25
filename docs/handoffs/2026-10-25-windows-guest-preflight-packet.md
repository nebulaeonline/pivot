# Windows guest non-GUI preflight — execution packet

## First execution — report recovery pending

Subsequent update: [copy-only recovery is complete and native preflight verified](2026-10-25-windows-guest-preflight-results.md).
The pending statements below preserve the original recovery request. No rerun occurred.

The first guest invocation returned capture
`preflight-capture-693ef95913dc463aa6a77ae30a7e6cba` as Blocked/2.
All five returned origin hashes and transfer-manifest digests verify. The console
records completion of preflight and native report path
`C:\PivotWork\attended-be006f4110234dc3b69314576948e96e\report`.
The bootstrap incorrectly required that path message to begin a log line; input's
prompt and the subsequent path print share one line in the tee log. It therefore
failed to collect the successful wrapper report and conservatively returned Blocked.

This is an exporter defect, not an observed guest desktop/preflight refusal.
Final preflight acceptance awaits the native report and its hash verification.
Preserve the original Blocked capture and frozen packet unchanged. The preparation
bootstrap's parser is corrected for future use, but the frozen copy remains original.
Do not rerun preflight. The owner can run the copy-only
`artifacts/attended-guest-preflight/recover-native-report.cmd` in the Windows guest
to return the already-created native report; it refuses an existing destination.
No tests, builds, clipboard access or confirmation replay occurs during recovery.

## Original execution instructions

2026-10-25. [Implementation accepted; actual guest preflight authorized](2026-10-25-windows-preflight-acceptance.md).
**No positive GUI execution, build, clipboard operation or full CI rerun.**

Frozen packet: `artifacts/attended-guest-preflight/windows-preflight-20261025/`.
Its `bundle-receipt.json` covers source.zip, full source/build-input manifests,
accepted wrapper/imported helpers and the report-return bootstrap. The preparation
receipt alongside the packet supplies its pinned digest and exact inventory counts.
Do not edit or regenerate this packet. It retains the accepted compile-time refusal.

## Owner command — inside Pivot-Win11-x64

Open ordinary **Command Prompt inside the Windows VM desktop**, with the existing
J: mapping. No administrator shell, developer shell or execution-policy change is
needed: this gate only reads metadata and verifies/extracts source. Use the already
installed Python that ran qualification; do not install tools.

```bat
J:\coding\pivot\artifacts\attended-guest-preflight\run-windows-preflight.cmd
```

The command copies the complete frozen packet to the fresh local directory
`C:\PivotWork\preflight-input\windows-preflight-20261025` before executing its
local bootstrap. It refuses an existing copy to prevent accidental replay or
replacement. Receipt/file hashes are checked before importing the wrapper. Source
is extracted into a new native-local work directory by the accepted wrapper.

When asked, type **CONFIRM** only if you are in the intended Basic Session with
the previously reviewed resource/clipboard conditions unchanged. This acknowledges
configuration only; it grants no GUI consent and opens no application windows.
If the conditions changed, decline and report the change.

The bootstrap tees output to a native-local capture without replacing the real
input stream or misrepresenting terminal status. It supplies no answers, permits
or activation variables. An early policy refusal remains Blocked, with console
evidence retained where local report acquisition was possible. Preflight success
is recorded as PreflightPassed, never a GUI pass.

## Results and transfer

Successful preflight and blocked attempts are retained under fresh
`C:\PivotWork\preflight-capture-<unique-id>` directories; the wrapper also retains
its own native original work/report directory. The bootstrap copies that exact
successful report (not the latest directory by timestamp), adds packet manifests,
console output and a capture summary, hashes locally, then deliberately exports
only evidence to:

`J:\coding\pivot\artifacts\attended-guest-preflight\results\Pivot-Win11-x64\`.

Returned hashes are checked against the native-generated manifest and a transfer
receipt records both manifest digests. Preserve both native originals and shared
reports. If transfer fails, report the printed local path/error; **do not rerun
preflight to repair copying**. An existing input-copy refusal likewise requires
inspection, not deletion and replay. No VM connection or integration settings change.

Tell the repository-side agent when results are available; no transcription is
needed. Return packet and actual guest evidence for parent review before changing
the positive-path refusal. Native AOT, Linux GUI, unattended initiation and new
authorization mechanisms stay deferred. G3 remains held.
