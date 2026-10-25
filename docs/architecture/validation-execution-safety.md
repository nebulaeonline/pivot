# Validation execution safety — first implementation gate

Latest acceptance (2026-10-25): [Windows Hyper-V x64 owner-attended managed GUI is
accepted/closed](../handoffs/2026-10-25-windows-managed-final-acceptance.md), alongside
both x64 full headless gates. Strict cleanup passed without exclusions/waits/kills.
Next authorization is Windows Native AOT design and non-GUI preparation with a
disabled packet for review, not positive execution. Ubuntu GUI remains separate;
unattended initiation stays unapproved and G3 held. Earlier milestone language
below records the implementation/qualification history rather than current blockers.

Current follow-through (2026-10-25): both Hyper-V x64 headless gates are closed;
owner-attested observed isolation is accepted for bounded activation design.
[Decision](../handoffs/2026-10-25-hyperv-isolation-acceptance.md),
[activation proposal](guest-gui-activation.md). Unattended operational authority
remains disabled. Qualification-pending statements below retain earlier gate context.
The [subsequent parent review](../handoffs/2026-10-25-attended-guest-preflight-direction.md)
narrows current work to Windows attended preflight using the existing broker.
A second guest authorization protocol and supervision remain deferred.

2026-10-24. **First execution-safety milestone accepted and closed.**
[Acceptance and qualification boundary](../handoffs/2026-10-24-execution-safety-acceptance.md).
This milestone implements local safety only; guest approval,
orchestration and expanded-platform acceptance remain deferred.
[Contributor guide](../testing.md).

## Common policy and enforcement

`scripts/execution_policy.py` defines execution classes and the audited entry-point
registry. Headless and fixture-fault are independent capabilities. Machine-wide
destructive and translated execution have distinct classes; neither gets an
unattended authorization path in this milestone.

CI with `--desktop`, direct managed UI smoke and Native AOT wrappers enter the same
desktop-session guard before builds/graphical children. With no inherited permit,
both input and output must be interactive terminals and the user must affirm the
explicit window/focus/clipboard warning. Default, no, EOF and noninteractive
streams refuse. No existing tests or acceptance assertions were removed.

Consent starts an IPv4 loopback broker on an ephemeral port with a random 256-bit
secret, a four-hour maximum validity and bounded request reads/timeouts. Nested
runners validate a fresh nonce/HMAC exchange. The application’s C# guard implements
that same protocol synchronously before `BuildAvaloniaApp`; both smoke and peer
activation require the live broker. Ordinary manual launches require no permit.
Managed and AOT processes use the same source. No JSON reflection is required.

The capability is inherited in the child environment, never command arguments,
reports or disk consent files. Wrappers revoke it by closing the broker at suite
exit. It is not a durable guest-isolation receipt, general RPC service or hostile
same-user security boundary. Code executing as the same user can modify the test
code or emulate the protocol; the goal is preventing accidental unattended entry,
not sandboxing malicious contributors. It performs no remote execution and accepts
no caller-supplied command, path, input action or VM operation.

Headless CI children receive neither broker authority nor `PIVOT_UI_*` activation.
The desktop child receives authority with stale activation cleared, then sets its
own report path. Its C# peers inherit that live authority. The manual launcher
also strips automation activation and authority. The clipboard serialization lock
remains intact, but is not treated as isolation. Direct old self-check commands
without a live broker now refuse rather than silently opening a window.

Policy refusal emits status Blocked/exit 2 and unattended isolation NotConfigured.
The first gate intentionally approves **no unattended guest profiles**. It cannot
prove host redirection settings from within a guest and therefore does not accept
a hostname, Boolean flag or self-authored JSON file as such proof. Configured guest
approval is a later reviewed extension; normal contributor headless CI needs none.

## Transitive audit of current supported entry points

Source audit covers repository Python runners, managed executable sources and
Rust app/crate child-spawn sites. Build/restore tools and dependencies remain
trusted development prerequisites, not an OS sandbox. Arbitrary binaries supplied
as research arguments must be the named audited worker, not unrelated executables.
Archived scripts/logs under artifacts are historical and not supported new runners.

| Entry / call chain | Classification and observed boundary |
| --- | --- |
| `ci.py` default | Headless + fixture-fault. Rust tools/workspace tests → solution build → safety probe → aggregate/decimal/protocol → IPC → application-session. Does not select UI. |
| `ci.py --desktop` | Above + GUI. Shared consent before any suite work → lock regression → guarded ui_smoke. |
| `ui_smoke.py` | GUI + headless builds. Shared guard → Rust session/managed desktop builds → clipboard lock → managed Desktop Program guard → Avalonia self-check → three independent GUI peers, each with same startup guard. |
| `verify_native_aot.py` | GUI + headless publish/build. Shared guard → toolchain diagnostics → native publish/adjacent child → same Program guard → self-check/three peers. No publish-only acceptance implied. Current RIDs remain win-x64/linux-x64/osx-arm64. |
| Desktop activation | Program checks both smoke/peer report variables before Avalonia. MainWindow Opened chooses peer/smoke; `ClipboardAutomation` launches only current executable/dotnet DLL peers. `RaiseEvent` tests synthesize Avalonia events, not OS key injection. Windows/focus and OS clipboard are still real. |
| Manual `run_desktop.py` | Deliberate GUI launcher, stripped automation environment → Rust child build → normal `dotnet run`. Projection/binding proof flags select manual proof content, not unattended acceptance. |
| Decimal verifier | Headless Rust decimal probe + managed DecimalParity executable; Python decimal/reference vectors. No GUI/native clipboard. |
| Aggregate reference | Headless Fraction-based corpus generation/check, no process/UI calls. |
| Protocol verifier | Headless protocol/authoring example writers/readers, frozen LegacyProtocol codec, canonical host probe, current managed ProtocolParity. File fixtures only. |
| IPC verifier | Headless pivotd/pivot-cli/ipc_probe/repl_peer processes with named pipes/Unix sockets. Cleanup kills only owned child handles; endpoints/fixtures belong to the suite. |
| Application-session verifier | Headless Rust pivot-session + managed ApplicationSession. WorkbookSession starts specified Rust child with redirected streams. Its byte-level ClipboardChecks do not reference Avalonia or the OS clipboard. Broken-child test kills its own known PID; session disposal kills only its owned process tree. Persistence uses fixture workbook paths. |
| Desktop lock module/tests | Atomic lock file; WSL may call PowerShell for LocalAppData and wslpath for shared lock location. No clipboard contents/input access. Tests spawn only a lock-owner Python child and clean its private temporary directory. |
| Rust default/all-feature workspace tests | Headless + fixture-fault. App authoring tests spawn their own daemon/CLI binaries; cold inventory/restore/history projection and storage file tests spawn the current test executable with exact worker names. Storage crash worker exits 17 at private staging; no unrelated PID kills or disk filling. |
| Rust file/history fault fixtures | Private UUID/temp or explicit PIVOT_FILE_TEST_ROOT subdirectories; staged corruption/truncation/error hooks, permission tests, own writer exits. History cleanup verifies canonical temp prefix/UUID naming. No GUI dependency or clipboard API in Rust app/crate test paths. Existing platform-specific fixture-root policies unchanged. |
| `research_history_matrix.py` | Headless history_representation worker, supplied native sample/receipt roots, successful checkpoint/delta workloads and hashes. |
| `research_history_durability.py` | Headless + fixture-fault history_durability worker; private journals and sync/error budgets; SIGKILL/TerminateProcess only to the child just spawned at its controlled pause. No actual shared-disk exhaustion. |
| `research_history_acknowledgments.py` | Headless successful history_durability ACK and independent reopen, private journals; write allowance is simulated, not disk fill. |
| `research_history_association.py` | Headless + fixture-fault history_association worker; fresh root per invocation, private corruption/renames/control files and own paused-child termination. Native-origin copies retained. |
| `research_mutation_records.py` | Headless + fixture-fault G2 worker, 45 isolated processes, in-memory capacity/semantic checks and separate oracle/publication accounting. |
| `bulk_workbook.py`, benchmark_resources | Headless bulk/readiness workers; reads own child RSS via Windows query/proc status, stops owned child/process group on timeout/RSS bound. Not a shared-resource exhaustion campaign. |
| Safety verifier/probe | Headless policy tests; real C# guard linked without Avalonia. Synthetic consent/routing, no input/clipboard/window APIs. |

Manual native file pickers remain normal application actions. Automated workbook
acceptance supplies paths; it does not certify native dialog interactions. Current
source contains no whole-machine disruption campaign needing a new VM-only
restriction. Future disk-fill, unrelated-process kills or machine-setting tests
must use a separately reviewed disposable profile. This gate does not broaden
permissions or change existing native file fault assertions/cleanup policies.

## Owner target inventory — not runner configuration

| Target | Owner-reported environment/name | Acceptance status |
| --- | --- | --- |
| win-x64 | Hyper-V `Pivot-Win11-x64` | Installed; guest preflight/isolation/execution unverified |
| linux-x64 | Hyper-V `Pivot-Ubuntu-x64` | Installed; guest preflight/isolation/execution unverified |
| mac-arm64 | Native Apple Silicon/APFS | Existing platform history; unattended GUI isolation unresolved |
| linux-arm64 | UTM `Pivot-Ubuntu-arm64` | Installed; native toolchains/AOT/GUI unverified |
| mac-x64-rosetta | Version-conditional translated macOS | Not configured/validated; no native Intel claim |
| win-arm64 | UTM `Pivot-Win11-arm64` | Installed optional sixth target; acceptance deferred |

Names are owner inventory only, not hardcoded dispatch identifiers or evidence of
architecture. Do not alter the separate Ubuntu 18.04 legacy library-build system.
The installed OS release, native dependencies and SDK support require verification
inside each guest; no package installation or bootstrap script is introduced here.

## Required isolation approval before unattended activation

1. Host-side identification of intended guest and configuration evidence disabling
   host clipboard/drive/folder redirection and unnecessary device integration.
2. Guest identity/OS/architecture, native filesystem work roots, native executable
   machine types, graphical session/backend and toolchain receipts.
3. Deliberately scheduled harmless sentinel test showing guest clipboard changes
   leave host clipboard unchanged. No personal content or unrelated applications.
4. Review/approve a narrow profile binding those host and guest receipts, invalidated
   by relevant host/session/configuration changes. Runtime flags alone are insufficient.

Do not launch a VM console and inject host input as a substitute. Never fall back
to host GUI. Preserve first failures and hash native-origin evidence before copy.
Missing infrastructure must remain distinguishable from failed executed tests and
must not create a target pass. Per-target dispatcher statuses will be reviewed
with orchestration; no `--vm` CLI or discovery service is added now.

Return for parent review after the safety implementation evidence. No remote
SSH/WinRM service, lifecycle automation, system installation, expanded acceptance
claim, product protocol or G3 history work is authorized by this milestone.

The [implementation handoff](../handoffs/2026-10-24-execution-safety.md) and
[native Mac reconciliation](../handoffs/2026-10-24-macos-execution-safety-validation.md)
now record 13-step headless CI and nine policy tests passing on Windows, native
WSL Linux and native Mac. All 247 inputs match; the Mac's shared documentation drift
is reconciled separately from its unchanged local tested snapshot. The parent accepts this
evidence and closes the safety gate. These results do not approve unattended guest
activation. The next authorized deliverable is primarily read-only guest qualification;
no inspection or activation is claimed here.

Current qualification progress: [capture workflow and blockers](../handoffs/2026-10-24-guest-qualification.md).
The initial host collector encountered Windows Hyper-V authorization failures;
later owner-run host and guest receipts are reviewed in the linked workflow.
No activation approval exists yet. Track Installed, Headless-qualified and
GUI-isolation-approved independently. Preflight collectors issue no GUI permits.

[Collector preparation is accepted; qualification remains open](../handoffs/2026-10-24-guest-collector-preparation-acceptance.md).
Existing fields distinguish successful metadata capture from native capability
candidates and independent approval; reading unsafe settings does not verify safety.
Owner-confirmed SMB/CIFS exchange shares are Windows guest `J:\coding\pivot` and
Linux guest `/mnt/j/coding/pivot`, both mapping the host J: root. This is an explicit
host-folder integration exception, not native storage or clipboard separation.
Build/run/acquire evidence on guest-local NTFS/Linux disk, hash there, then transfer.
No automatic share disconnection or extra host redirection is authorized here.

[First x64 guest evidence](../handoffs/2026-10-24-x64-guest-evidence-review.md)
verifies native Rust/.NET execution on NTFS/ext4 in both guests and 73 returned file
hashes. Windows metadata follow-up is complete; headless qualification is accepted for both x64 Hyper-V guests. No GUI
authority or full guest CI acceptance is inferred. The capture's .NET first-run
certificate side effect is disclosed, with future certificate generation disabled.

The [full Hyper-V guest headless CI packet](../handoffs/2026-10-24-hyperv-headless-ci-validation.md)
now records genuine source-matched execution in both qualified x64 guests, with
13 steps and nine safety tests each, 692 independently verified report hashes and
twenty zero-charge fields. Parent acceptance closes both full x64 headless CI gates. Actual
GUI connection/redirection and clipboard-separation evidence is still absent;
headless success grants no unattended GUI authority.
