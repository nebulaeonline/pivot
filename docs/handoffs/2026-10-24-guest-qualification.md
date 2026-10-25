# Guest qualification — capture packet and current blockers

2026-10-24. Safety implementation and its documentation are accepted/closed.
**Collector preparation and headless qualification of both Hyper-V x64 guests
are accepted. GUI isolation remains unapproved; ARM64 qualification is pending.**
[Headless acceptance and next CI gate](2026-10-24-x64-headless-qualification-acceptance.md).
[Parent collector review and shared-drive clarification](2026-10-24-guest-collector-preparation-acceptance.md).
This is the already authorized guest-qualification workstream. **G3 remains held and has not started.** No VM dispatch,
provisioning, unattended GUI permit or clipboard operation has been introduced.
[Accepted boundary](2026-10-24-execution-safety-acceptance.md).

## Three independent states

| State | Required evidence | Does not imply |
| --- | --- | --- |
| Installed | Intended guest exists; host identity/configuration bound to guest | Working toolchains or isolation |
| Headless-qualified | Reviewed guest OS/native architecture, compilers/runtimes, local filesystem and actual native executable behavior | Full Pivot CI acceptance or GUI authorization |
| GUI-isolation-approved | Independently reviewed host redirection, exact guest graphical session and controlled clipboard-separation evidence | Perpetual approval or a positive GUI/AOT acceptance run |

Report check outcomes separately: Verified, NotConfigured, Blocked, Unsupported or
Failed. A successful metadata query only verifies that it was obtained, not that
the settings are safe. A native-probe capability candidate is not final approval.
Missing optional infrastructure is not a failed product test or a target pass.

Read the existing result fields precisely:

| Result/field | Meaning |
| --- | --- |
| Host `checks[].status=Verified` | Metadata query completed; inspect the actual setting. Not a safe-configuration verdict. |
| Guest `checks` / `probes` | Individual tool/query/compile/execute outcomes; missing tools and failed executions remain separate. |
| Guest `headless_capability_candidate=Verified` | Required native checks/probes succeeded; bind identity and review before qualification. Not Pivot CI acceptance. |
| `headless_qualified=NotConfigured` | Independent guest qualification has not been recorded, even if collection/probes succeeded. |
| `gui_isolation_approved=NotConfigured` | No GUI authorization; cannot be inferred from successful collection or native capability. |

Collector execution/internally consistent receipts, native capability and isolation
approval are different claims. Do not collapse them into an overall VM pass.

## Shared-drive layout and native work roots

Windows guests map `J:\` over SMB to the host's J: root; Pivot is
`J:\coding\pivot`. Linux guests mount that root over CIFS at `/mnt/j`; Pivot is
`/mnt/j/coding/pivot`. This applies to Hyper-V and UTM guests. These mappings are
the explicitly allowed source/evidence exchange exception, not native test storage
or clipboard-isolation evidence. Do not confuse guest-mapped J: with host-local J:.

Copy to local NTFS such as `C:\PivotWork` or local Linux disk such as
`$HOME/pivot-work` before builds/probes/persistence tests. Collect and hash original
evidence locally, then deliberately copy it back to fresh per-guest report directories
under the shared `artifacts/guest-qualification/`. Record this share's scope in the
isolation review; it does not authorize other host drive/folder/device redirection.

## Actual inspected state

| VM | Installed | Headless qualification | GUI isolation |
| --- | --- | --- | --- |
| Hyper-V `Pivot-Win11-x64` | Verified by corrected owner host receipt | HeadlessQualified; full headless CI Passed and accepted/closed | Not approved |
| Hyper-V `Pivot-Ubuntu-x64` | Verified by corrected owner host receipt | HeadlessQualified; full headless CI Passed and accepted/closed | Not approved |
| UTM `Pivot-Ubuntu-arm64` | Owner-reported | Separate guest/host evidence pending | Not approved |
| UTM `Pivot-Win11-arm64` | Owner-reported, optional | Deferred | Not approved |

The Hyper-V module is present on ROBPC. Initial read-only queries failed with
Access denied in the sandbox; outside it Windows reported insufficient Hyper-V
authorization. The collector captured five Blocked queries: host Enhanced Session
capability plus VM/integration-service queries for each x64 guest. No permission,
group, VM state, integration setting or guest was changed. No UAC elevation or
guest command was attempted. Raw report and native-origin hashes:
`artifacts/guest-qualification/host/hyperv-preflight-20261024T1840202873187Z/`.
The tool wrapper reported exit 1; the per-check permission blockers are authoritative,
not a fabricated guest pass. Original collector source and attempt history are retained.

Subsequent owner reports resolve host identity capture and establish native Rust/.NET
compile/execute capability in both x64 guests. [Detailed guest review](2026-10-24-x64-guest-evidence-review.md)
verifies 73 returned file hashes and preserves the original Windows Blocked candidate,
its resolved follow-up and the disclosed .NET first-run certificate side effect.
The initial access failures above are historical, not the latest host result.

## Capture host evidence — owner action

**Corrected capture complete:** owner report `hyperv-preflight-20261024T1856028006233Z`
passes all five metadata queries. Its origin hash and collector-source hash were
verified before and after copying from `C:\Users\Public\PivotPreflight` to
`artifacts/guest-qualification/host/`. Both intended guests exist:

| Guest | Host VM identity | Generation / configuration | vCPUs |
| --- | --- | --- | --- |
| `Pivot-Win11-x64` | `f89ff908-16dc-44c4-af54-008967e48443` | 2 / 12.0 | 16 |
| `Pivot-Ubuntu-x64` | `9ce1b716-aa20-4b16-a206-25126fab46d5` | 2 / 12.0 | 16 |

Host Enhanced Session capability is enabled; all six listed integration services
are enabled in each guest's configuration. This establishes neither actual current
session/redirection settings nor guest OS/architecture or executable capability.
No VM setting was changed. [Host reconciliation receipt](../../artifacts/guest-qualification/host-reconciliation.json).
Next is guest-local collection; no further host rerun is needed for the corrected
name queries. The failed earlier attempts below remain historical evidence.

Owner capture `hyperv-preflight-20261024T1847110624262Z` was returned and its native
hash verified. Host Enhanced Session capability is enabled; that alone proves no
current clipboard/drive redirection state. Its four per-guest queries were invalid:
the collector's case-insensitive PowerShell `$Name` parameter shadowed the outer
`$name`, querying labels such as `vm:Pivot-Win11-x64` rather than the actual VM.
These results are a **collector defect**, not evidence that any VM is absent or
misconfigured. Original C: and returned artifact reports remain unchanged.

The collector now uses distinct `$CheckId` and `$guestName` variables. A new
synthetic-provider regression runs the actual script and verifies exact arguments
for two guests, returned identities, hash integrity and no qualification/isolation
approval. It passes without accessing Hyper-V. A preliminary synthetic run exposed
a test-only scope error in its call recorder; that fixture was corrected, with
its report preserved. The corrected owner capture above resolves this request;
do not replay VM operations or alter VM settings to address this defect.

In an **administrator PowerShell on the Windows Hyper-V host**, from the repository:

```powershell
& .\scripts\qualification\hyperv_host_preflight.ps1 `
  -VMName 'Pivot-Win11-x64','Pivot-Ubuntu-x64' `
  -OutputRoot 'C:\Users\Public\PivotPreflight'
```

Choose a local NTFS report root. The script only reads host/VM metadata and writes
a fresh report/hash manifest; it never executes guest commands, opens a console,
starts/stops/configures VMs or reads the clipboard. Recheck actual settings and
permissions manually if blocked. It installs nothing and changes no access policy.
Copy the complete report to a new directory under `artifacts/guest-qualification/`;
verify all origin hashes after transfer and retain the native originals.

Enhanced Session availability and Integration Services alone do not establish
current clipboard/drive separation. Separately record the actual connection mode,
VMConnect/RDP resource-redirection selections and relevant host settings. Inspect
only the intended test connections; do not scan personal RDP profiles or credentials.
No automatic redirection-setting changes are authorized by these collectors.

## Capture guest evidence — run inside each guest

Start with the two x64 Hyper-V guests. Copy the standalone
[guest collector](../../scripts/qualification/guest_preflight.py) into each guest
by an owner-controlled transfer. No SSH, WinRM, PowerShell Direct or persistent
shared-folder dependency is required. Do not use the legacy Ubuntu 18.04 build
environment. A minimal copy needs only this Python file, not the entire repo.

Use an actually installed native Python 3.8+ interpreter. On Windows, avoid the
Microsoft Store alias that would trigger installation, or an install-manager command
that could install Python. An already installed packaged Python is supported; a
protected alias file does not prove its running process is nonnative. If tools are
absent, report NotConfigured; do not install them under
this gate. Open a native developer shell if C++ linker discovery requires it.

Windows guest (replace the Python/script paths with their actual local locations):

```powershell
New-Item -ItemType Directory -Path 'C:\PivotPreflight' -Force | Out-Null
New-Item -ItemType Directory -Path 'C:\PivotWork\qualification' -Force | Out-Null
Copy-Item -LiteralPath 'J:\coding\pivot\scripts\qualification\guest_preflight.py' `
  -Destination 'C:\PivotWork\qualification\guest_preflight.py'
& 'C:\Path\To\Installed\Python\python.exe' `
  'C:\PivotWork\qualification\guest_preflight.py' --output-root 'C:\PivotPreflight'
```

Linux guest:

```sh
mkdir -p "$HOME/pivot-preflight" "$HOME/pivot-work/qualification"
cp /mnt/j/coding/pivot/scripts/qualification/guest_preflight.py "$HOME/pivot-work/qualification/"
python3 "$HOME/pivot-work/qualification/guest_preflight.py" --output-root "$HOME/pivot-preflight"
```

Default mode collects metadata only, and intentionally leaves native executable
proof Blocked. With tools present, repeat into a **new** report with `--probe`:

```sh
python3 "$HOME/pivot-work/qualification/guest_preflight.py" --output-root "$HOME/pivot-preflight" --probe
```

On Windows append `--probe` to its command instead. This explicit option compiles
and runs tiny private Rust and .NET console programs, not Pivot or Avalonia. It
preserves its unique report/probe tree and never installs tools. The first captures
also reported .NET development-certificate generation; see the disclosed side effect
in the guest review. Future runs explicitly disable certificate generation.
.NET selects stable SDK 10 with the repo's feature-band roll-forward policy;
NuGet package sources are cleared for this dependency-free probe. Rust automatic
toolchain installation is disabled. A missing reference pack/SDK remains a blocker.

The collector checks native kernel/process/image architecture, Rust host/ABI, Cargo,
.NET info, compiler/linker availability, guest-local NTFS or Linux disk, virtualization
metadata and session identity hints. Probe results include real native binary headers
and runtime architecture output; Windows x64 emulation cannot establish ARM64
qualification. If it reports failure, preserve it rather than replaying until green.
No approved isolated guest is inferred from a name, env flag or successful probe.

Output is a unique directory with `guest.json`, artifacts and `origin-hashes.json`.
Hashing occurs before transfer. Preserve it on guest-local storage, copy the entire
directory to a fresh per-guest directory under `artifacts/guest-qualification/`, and
verify returned hashes. Do not run probes on SMB/DrvFS/redirected folders; unsupported
or unproven work-root types fail closed before creating a report there. A default
metadata-only Blocked outcome is expected until native probes and review are complete.

Record the intended host VM identity next to each guest report. Hostname alone is
not binding proof. Correlate guest identity, boot, connection/session ownership and
host inventory/configuration in the reviewed packet. For Linux record X11/XWayland/
Wayland explicitly; environment hints alone do not certify the exercised backend.
For Windows record actual guest logon/session and Basic/Enhanced/RDP connection mode.

## Proposed clipboard-separation procedure — not executed

Only after the owner deliberately schedules a quiet test window, reviews the host
integration settings and agrees it is safe to overwrite clipboard contents:

1. Use only scratch test surfaces/files in permitted roots, with unrelated apps
   closed/unused. Do not read/export any prior personal clipboard data or open mail.
2. Deliberately put a newly generated harmless host sentinel in the host clipboard.
   Record only that known sentinel and the exact VM/configuration/session identity.
3. Inside the intended guest session, deliberately copy a different guest sentinel.
   Verify guest-local paste yields it; host-local paste into its scratch surface
   must still yield the host sentinel.
4. Replace the host sentinel with another known marker and verify the guest retains
   its guest marker. This additionally checks host-to-guest clipboard integration.
5. Record both directions, timestamps, connection mode, boot/session identity and
   configuration evidence. Unexpected propagation is a failed isolation check:
   stop and preserve it; no GUI acceptance follows. Clear only test sentinels when
   safe; do not promise restoration of personal clipboard formats or history.

This is an owner-controlled procedure proposal, not automated host input injection
or permission to launch GUI suites. A quiet scheduled window is not proof of VM
isolation. Review the full host/guest/sentinel evidence before approving each
activation profile. Reconnection, graphical-session replacement, redirection,
Enhanced Session/RDP/UTM integration or other relevant changes invalidate approval.
No environment variable can renew it, and no one-time check grants perpetual authority.

UTM Linux ARM64 needs its own Mac-host integration/settings evidence and native
guest reports. Do not infer virtualization versus emulation from the VM name.
Windows ARM64 remains optional; Rosetta coverage is not established here.

## Collector verification and stop

Only new collector tests were run: Windows four passing checks plus one explicit
Linux-only skip; WSL five passing checks. These synthesize PE/ELF files and tool/
filesystem outcomes; they do not inspect guests or rerun completed product gates.
PowerShell syntax is checked. Real x64 host/guest receipts are now reviewed, but
the Windows follow-up and x64 headless qualification are accepted; GUI isolation
remains unapproved. Full Pivot headless CI has since passed and been accepted in both x64 guests;
individual GUI-isolation qualification is the next priority.
[Acceptance](2026-10-24-hyperv-headless-ci-acceptance.md). Later correction tests are recorded separately
from the original synthetic checks and do not rerun accepted product gates.

Return the actual evidence and missing prerequisites for parent review. Stop before
unattended permits, positive managed/AOT GUI acceptance, orchestration, VM lifecycle
automation or provisioning. Positive GUI/AOT runs belong to a later individually
approved activation gate with existing assertions and zero-charge checks intact.
G1/G2 and execution safety remain closed; G3 remains held throughout.
