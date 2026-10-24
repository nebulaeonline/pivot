# Hyper-V x64 guests — first native evidence review

2026-10-24. Seven owner-returned reports reviewed: metadata and explicit native
probes for each x64 guest, plus three Windows metadata follow-ups. **Native Rust/.NET
probes passed on both; Windows x64 toolchain metadata is now complete. The combined
headless-capability packet is accepted: both x64 Hyper-V guests are HeadlessQualified.
Full Pivot guest CI has not run, and GUI isolation is not approved.
[Parent acceptance and next assignment](2026-10-24-x64-headless-qualification-acceptance.md).** G3 remains held.
[Qualification workflow](2026-10-24-guest-qualification.md).

## Verified receipts

All **70 file hashes** in the four origin manifests match the returned files.
All four collector fingerprints match the archived capture source at
`artifacts/guest-qualification/guest-collector-at-x64-capture.py`. Native Rust probe
and .NET apphost headers were independently inspected from the copied binaries;
all four are x64. Recorded Rust/.NET compilation and execution exit codes are zero.
No guest binary was executed on the host to manufacture guest execution evidence.
[Independent receipt](../../artifacts/guest-qualification/x64-guest-review.json).

| Guest / owner-mapped reports | Native evidence | Remaining qualification |
| --- | --- | --- |
| `Pivot-Ubuntu-x64`: `191425Z-e9a9d77a` metadata, `191457Z-fb5e7a67` probes | Ubuntu 26.04.1 LTS; x64 Python/Rust/.NET; Hyper-V detection `microsoft`; ext4 `/dev/sda2`; native Rust/.NET compile/execute passed | Capability candidate Verified; independent qualification review pending; GUI isolation unapproved |
| `Pivot-Win11-x64`: `191023Z-d27ee9d1` metadata, `191129Z-bdc23459` probes, latest `193440Z-6e0649ab` metadata | Windows 11 Enterprise Evaluation 10.0.22621; native OS/Python process x64; Rust/.NET images x64; local C: NTFS; native Rust/.NET compile/execute passed; selected MSVC Hostx64/x64 | Combined capability evidence ready for review; original Blocked/NotConfigured candidates preserved; GUI isolation unapproved |

Full directory names are `guest-preflight-20261024T<suffix>` under each VM's folder
in `artifacts/guest-qualification/`. Native originals remain under
`C:\PivotPreFlight` and `/home/robhorvath/pivot-preflight`; reports identify exact
paths. Metadata-only Blocked outcomes are expected because no probes were requested;
they are not failed native tests. Original results remain unchanged.

Linux records Rust 1.101.0-nightly (`48f4b0553`), x86_64-unknown-linux-gnu,
.NET SDK 10.0.113/runtime 10.0.13, GCC 15.2, Clang 21.1.8 and binutils 2.46.
Windows records the same Rust revision with x86_64-pc-windows-msvc and .NET SDK
10.0.402/runtime 10.0.13. Rust outputs respectively `linux x86_64 64` and
`windows x86_64 64`; both .NET probes output OS/process `X64 X64`.

Linux hostname matches its VM name. Windows guest hostname is `WinDev2407Eval`,
not the Hyper-V VM display name; the owner-returned folder associates it with
`Pivot-Win11-x64`. Retain that distinction when binding the host VM identity and
actual guest session. No hostname-based isolation approval is inferred.

## Windows follow-up and collector correction

**Follow-up complete:** `193440Z-6e0649ab` selects `Hostx64\x64` first for both
compiler and linker. The compiler banner reports 19.44.35229 **for x64**; linker
banner is 14.44.35229.0. Older x86 entries also remain later on PATH, but are not
the first selected tools. Actual Python process and native OS are x64; collector
fingerprint matches current source. Rust revision and boot/session metadata match
the earlier successful native probe run. No successful probes were replayed.

All **73 guest file hashes across seven reports** verify. The new metadata-only
candidate is Blocked because it intentionally contains no probes; combine it with
the existing successful probe receipt rather than rewriting it. `link.exe /?`
again returned 1100 after help output. Preserve that raw metadata result as Failed,
separately from the prior successful native compilation/execution. No additional
collector, tool installation or collection rerun is required for this question.
[Combined capability reconciliation](../../artifacts/guest-qualification/x64-capability-reconciliation.json).

The earlier follow-up sequence and remedy below remain historical context; its
requested x64 selection is now demonstrated, not a remaining task.

Two metadata-only follow-ups are now returned. `192353Z-c394a30f` confirms actual
Python process x64 but still has no compiler/linker on PATH. `192649Z-f11a54fc`
finds Visual Studio 2022 Community MSVC 14.44.35207, compiler banner 19.44.35229 and
linker banner 14.44.35229.0. However the selected binaries are **Hostx86/x86** and
the compiler explicitly reports **for x86**. A 64-bit PowerShell process does not
establish the developer toolchain's host/target selection.

All **72 file hashes across six reports** now verify. The latest Windows report's
collector fingerprint matches current code; Rust version and boot/session fields
match the earlier passed native probes. The linker help request returned 1100
after printing help; retain that nonzero metadata-command result. It is not a
failed Rust/.NET compilation or execution. Both later candidates remain Blocked by
design because no probes were requested; do not replay passed probes to change that.
[Latest shell reconciliation](../../artifacts/guest-qualification/windows-developer-shell-review.json).

Use the installed Visual Studio initialization explicitly, from the Windows guest:

```powershell
& 'C:\Program Files\Microsoft Visual Studio\2022\Community\Common7\Tools\Launch-VsDevShell.ps1' `
  -Arch amd64 -HostArch amd64 -SkipAutomaticLocation
where.exe cl.exe
where.exe link.exe
```

Both paths should select `Hostx64\x64`. These options configure host and target
architecture as documented by [Microsoft](https://learn.microsoft.com/en-us/visualstudio/ide/reference/command-prompt-powershell?view=vs-2022).
Once the path check is correct, collect a new metadata-only report with the existing
local script; no new probe, Linux run or installation is needed. No additional
Visual Studio discovery mechanism was implemented after the owner supplied this
capture: installed tools are now located, and only shell selection needs resolving.

Windows Rust linked and ran successfully despite `where link.exe` / `where cl.exe`
finding no PATH entries. That does not prove MSVC is uninstalled: the compiler may
discover its linker independently. Do not install tools based solely on those
PATH results. Capture metadata from a native x64 developer shell to establish
compiler/linker paths and versions.

The original collector could not read the packaged Python alias's PE header and
therefore left its candidate Blocked. It now additionally records the **actual
running process architecture** using IsWow64Process2, keeping the original header
result. Native and emulated process cases remain distinct; no pointer-width guess
or unconditional Unknown-to-native conversion is used. Synthetic tests cover
native x64/ARM64, x64-on-ARM64, x86-on-x64 and unknown machine types. The API was
also exercised on the Windows host as a local implementation check, not guest proof.

The original packaged interpreter demonstrably ran the collector. It need not be
reinstalled merely because its alias file could not be inspected. Avoid only
uninstalled aliases that would launch the Store or trigger installation.

For one targeted Windows follow-up, copy the updated collector to the same local
work path and run **metadata-only**, without `--probe`, from an **x64 native
developer shell**, preserving a new report. Use the same working Python invocation
as before. Combine that report with the existing successful native probes during
review; the new metadata-only candidate will remain Blocked by design. No successful
probe replay, elevation, package installation or Linux rerun is requested. If
compiler/linker tools still cannot be found, return the report as missing
prerequisites rather than changing the VM.

## Disclosed .NET first-run side effect

Both probe build logs report generating an ASP.NET Core HTTPS development
certificate. This was unintended: the prior claim that probe writes were confined
to its report tree was too strong. There is no recorded certificate trust command;
actual certificate-store contents have not been inspected remotely. No cleanup or
blanket certificate removal was attempted.

Future collection sets `DOTNET_GENERATE_ASPNET_CERTIFICATE=false` before every
dotnet invocation, as documented by [Microsoft](https://learn.microsoft.com/en-us/dotnet/core/tools/dotnet-environment-variables#dotnet_generate_aspnet_certificate).
The original source, stdout/stderr and results remain preserved; this correction
does not retroactively erase that side effect. Guest-state cleanup, if desired,
must be deliberate and targeted; do not run broad dev-certificate clean/trust commands.
No successful Linux probe replay is required merely to reconcile this finding.

## Scope and next gate

Linux reports a Wayland environment with DISPLAY `:0` and WAYLAND_DISPLAY
`wayland-0`, but no XDG_SESSION_ID. Windows reports Console/session 3. These are
session hints, not verified GUI backend/ownership or connection-redirection evidence.
No host clipboard settings, sentinel separation or positive managed/AOT GUI run is
established. No full Pivot headless CI ran in these VMs; tiny probes prove native
toolchain/executable capability only. ARM64/Rosetta receipts are absent.

Preserve the host permission failures and corrected name-shadowing defect alongside
these successful native probes. The Windows follow-up is complete. The combined
headless-capability packet is accepted and both guests are HeadlessQualified.
Do not grant unattended permits or start orchestration/G3. Full Pivot guest CI and
GUI isolation remain separate, unpassed gates.
