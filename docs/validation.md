# Multi-platform development validation

Use the existing CI and desktop runners through one entry point.
See [attended GUI/AOT activation and collection](validation-gui-aot.md) for graphical-session workflows:

```text
python scripts/validate.py
python scripts/validate.py --suite headless
python scripts/validate.py --targets linux-x64 --suite managed --gui
python scripts/validate.py --targets win-x64 --suite aot --gui
```

A target profile may specify `dotnet_root`, the absolute path to a private SDK
installation on that target. The worker requires a dotnet executable there,
prepends it to its private PATH and sets matching DOTNET_ROOT variables before
tool verification and tests. No system installation or global environment is
changed. ROBPC's Linux x64 profile selects the owner-approved Microsoft SDK
10.0.403 installation after a crash in Ubuntu's packaged runtime; its failed
matrix evidence remains preserved.

Default is discovery, not a build/test pass. Targets: win-x64, linux-x64, win-arm64,
linux-arm64, osx-arm64, osx-x64. Headless/discovery targets run concurrently with a
global ceiling of six by default (`--jobs` can lower it). Profiles set
`concurrency_limits: {"mac-host": 2}` and assign `concurrency_group: "mac-host"`
to both UTM guests and both macOS targets. Per invocation, those four share two
slots; the Hyper-V targets run independently of that group. GUI targets run
serially and require explicit `--gui`
plus the existing live interactive consent. Native macOS ARM64 and Rosetta x64
GUI runs must never overlap: they share the host desktop and interfere with each
other. The Mac's two-worker capacity applies to headless/discovery work, not to
concurrent GUI execution. Future GUI integration must preserve this desktop exclusion across separately
launched runs as well. Never preanswer the prompt. The same
existing consent broker is used by nested original runners; no new permit system.

Configure transports and source paths in [validation-targets.json](../scripts/validation-targets.json),
or pass `--config` with a private copy. A host-specific profile at
`artifacts/validation-config/<hostname>.json` takes precedence automatically; ROBPC's
profile uses designated guests and the Mac over SSH. Supported transports are local, WSL and SSH.
SSH uses existing key authentication/known-host trust in BatchMode, with a PTY for
GUI consent. Hyper-V and UTM guests use configured SSH endpoints; no hypervisor
permissions are needed for ordinary test dispatch. Automatic VM start/stop is not
implemented. A target must already be running/reachable with its tools installed.

The six-RID completion matrix requires Hyper-V Windows/Ubuntu x64, UTM Windows/Ubuntu
ARM64, and Mac mini native ARM64/Rosetta x64. ROBPC is controller-only. Default
Windows entries use SSH with unset endpoints until guest setup succeeds; there is
no local ROBPC/WSL fallback. Local/WSL profiles are development diagnostics only
and do not satisfy this designated matrix.
For SSH targets set `host` to an SSH alias or user@address, `source_root` to the
checkout/shared-source path on that machine, and `python` to its Python command.
Windows SSH requires `shell: powershell`. With `deploy: true`, `source_root` is a
native remote staging base: the host uploads an ordinary source ZIP over SCP and
extracts a fresh per-target checkout there. `bootstrap_python` can use native Python
even when the intended target Python is emulated. This avoids SSH access to SMB
mounts; the Mac profile uses `/Users/robhorvath/PivotValidation/remote` and keeps
Remote Login Full Disk Access off. No source hash inventory/immutable packet is made.
For native execution on a Mac, use transport `local` for its RID;
other targets can remain SSH. Local execution always verifies actual architecture.

For Hyper-V x64 guest runs, configure SSH and the guest's source path (Windows `J:/coding/pivot`, Ubuntu `/mnt/j/coding/pivot`). The
same worker handles both virtualization products. No new per-execution review or
packet is required. Keep credentials/private keys out of this repository; use SSH
configuration/agent. No server is enabled by this script.

macOS x64 uses explicit Rosetta execution of x64 .NET and Rust tools. Python can
remain native ARM64: its job is orchestration, not establishing the test RID.
Selected .NET/Rust execution architectures must match the requested target;
Native AOT RID selection also uses actual .NET host architecture, not Python.
Actual test execution is required. Cross-compilation is never counted as execution. Emulation is
recorded and requires explicit target configuration. Native AOT host selection
recognizes all six RIDs; actual publish/run success is still required separately.

Each worker creates fresh local work under `~/PivotValidation/<run>/<rid>`.
Windows requires local NTFS; Linux a block-backed local filesystem; macOS APFS.
Source is copied from the share before builds, excluding generated output, artifacts,
repository metadata and local credential/tool settings. Cargo/NuGet/.NET/temp/
fixture/bundle caches remain local. Do not build .NET directly on SMB.

Reports return to `artifacts/validation-runs/<run>/<rid>` on the source checkout.
The host matrix is under `artifacts/validation-matrix/<run>/matrix.json` and
`matrix.md`. SSH also retrieves remote reports with SCP. Native originals remain
available if export fails. Reports retain Git commit/dirty state, actual execution
architecture, commands, tool queries, exit codes, elapsed times and console logs.
No per-execution full-source hash inventory or immutable packet is generated.

Statuses distinguish Passed, Available (discovery only), NotConfigured, NotSelected,
NeedsConsent, Blocked, SetupFailed, Failed and ExportFailed. Missing platforms are
never Passed. Overall Partial means selected targets remain unavailable or blocked;
it is not full six-target acceptance. Exit codes: 0 complete, 1 failure/setup failure,
2 partial/blocked configuration. Consult JSON for the precise result.

Existing full application assertions and four zero-charge checks remain mandatory.
GUI completion checks actual report mode/backend and application/peer/Rust PIDs,
plus the launched POSIX process group where available. Actual surviving owned
processes fail. This is practical owned-process cleanup on trusted machines, not
hostile-process identity proof; no unrelated /proc-tree inspection is a development
gate. Windows currently uses reported owned PID liveness rather than a Job Object;
POSIX process groups do not cover descendants that deliberately detach. These
limits are explicit, and no unrelated process is killed. Original runner timeouts
and failure handling remain unchanged.

The previous forensic packets/reports retain their original classifications as
historical evidence. They are not routine development entry points. See the
[infrastructure reset](handoffs/2026-10-25-validation-infrastructure-reset.md).
Six-RID execution validation is accepted and closed; G3 scheduling is released.
See [final acceptance](handoffs/2026-10-27-six-rid-validation-acceptance.md).
[Windows clipboard reliability](issues/0001-windows-clipboard-paste-reliability.md)
remains open; actual assertion failures must still fail validation.
