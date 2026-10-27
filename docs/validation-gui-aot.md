# Managed GUI and Native AOT validation

Headless validation is accepted and closed. These are separate GUI execution
gates using the existing application and peer assertions, four zero final charges,
actual backends/process architectures and owned-process cleanup.

## Prepare from ROBPC

```text
python scripts/validate.py --suite managed
python scripts/validate.py --suite aot
```

Without `--gui`, these commands inspect prerequisites and prepare target-local
activation commands. They do not open windows, acquire consent or publish AOT.
`NeedsGraphicalSession` and `NeedsConsent` are pending states; missing compiler/SDK
prerequisites are `SetupFailed`. Neither discovery nor a successful probe is a
GUI/AOT execution pass.

Windows SSH runs in session 0. Linux SSH is not the owner's local graphical
session, and Mac SSH uses a background bootstrap context. Use a terminal inside
the designated desktop for the activation step. No VM lifecycle automation,
security-setting changes or host-desktop fallback is performed.

## Activate on the target desktop

One short command runs both suites sequentially. It uses the existing live
consent broker once for the selected work; no stored permit or new broker is used.
Already collected passing suites are skipped. Run one Mac-hosted target at a time
for the attended acceptance round.

Windows guest, ordinary desktop Command Prompt:

```cmd
J:\coding\pivot\scripts\attend.cmd both
```

Ubuntu guest, terminal inside the unlocked graphical desktop:

```sh
bash /mnt/j/coding/pivot/scripts/attend.sh both
```

Mac, Terminal inside the logged-in desktop, ARM64 and Rosetta sequentially:

```sh
bash /Volumes/RobMac/coding/pivot/scripts/attend.sh both mac
```

Use `managed` or `aot` instead of `both` to select one suite, or supply a specific
RID after the suite, such as `both osx-x64`. Generated `activate-<rid>.cmd/.sh`
files in each prepared matrix directory are also available.

The launcher resolves the latest relevant prepared/collected matrix. It refuses
to reuse a started run or silently fall back from a completed result to an older
plan. Collect a completed attempt before preparing a fresh one. Failures and
declined consent are retained; there is no retry-to-green loop.

GNOME terminals can belong to a user-manager scope rather than directly to the
Wayland/X11 session. For local terminals, readiness resolves logind's primary
display for the same user and requires it to be active and unlocked. SSH shells
cannot use that fallback. A display variable alone is insufficient. Actual GUI
reports remain the execution proof.

Blocking launcher diagnostics are saved under native
`~/PivotValidation/activation-reports/` and copied to
`artifacts/validation-activation/`. No manual command-output transcription is
needed. Native build/cache/temp/report roots remain NTFS, ext4 or APFS; the share
is used for instructions/source exchange and returned reports.

## Collect from ROBPC

```text
python scripts/validate.py --collect <prepared-matrix-run-id>
```

Collection launches no tests. It writes a new matrix and preserves the original
prepared and previous collected matrices. A pass requires the expected attended
run/RID/suite, the worker's execution/cleanup result, and all four original GUI
reports with correct mode, architecture, backend and zero charges. Missing or
incomplete reports cannot become Passed. Worker PID cleanup is checked on the
target at completion, not by querying unrelated controller PIDs later.

Current prepared matrices:

- Managed: `20261026T034147Z-35def040`
- AOT: `20261026T034443Z-eb878098`

## Mac desktop exclusion

Native ARM64 and Rosetta GUI must not overlap, including separate invocations or
checkouts. Both original GUI runners acquire a kernel file lock at the fixed
`/private/var/tmp/pivot-native-desktop.lock` during the real window/clipboard suite,
before the existing clipboard lock. Clipboard-path environment overrides cannot
bypass it. The inode is retained; closing the descriptor releases ownership,
including process exit. Acquisition failure blocks GUI execution.

This is desktop resource exclusion, not a permission broker or an isolation
claim. Live consent remains necessary. Headless/discovery scheduling separately
uses global ceiling six and shared mac-host capacity two across both macOS RIDs
and both UTM guests; Hyper-V jobs are independent. GUI dispatch remains serial.

Native AOT still requires a real native publish, the actual native application
and all peers, matching process architecture, unchanged functional assertions and
zero-charge shutdown. No publish-only or backend-discovery pass is accepted.
