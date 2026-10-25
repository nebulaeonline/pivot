# Windows attended Native AOT — disabled preparation design

2026-10-25. [Preparation authorization](../handoffs/2026-10-25-windows-managed-final-acceptance.md).
**Design/implementation for review only. Positive AOT execution is hard-disabled.**
Windows managed GUI is accepted/closed and is not rerun by this milestone.

## Fixed audited flow

Reuse `scripts/verify_native_aot.py` unchanged. Its existing GUI consent guard runs
before any work, then it runs clipboard-lock console checks, tool metadata, release
Rust session build and local-RID .NET Native AOT publish. The NativeAot profile sets
PublishAot/SelfContained and leaves trimming/AOT analysis warnings unsuppressed;
the runner requires warning-free publish and no adjacent Pivot.Desktop.dll.

It copies the actual Rust child adjacent to the published executable, hashes the
publish tree, removes developer child overrides and executes that native image.
Main/three-peer reports must identify native-aot mode and the actual published path,
pass functional and zero-charge assertions, and prove native cross-process clipboard/
Cancel-Save. WB5 fixture envelope/hash is checked. Original per-step logs/summary,
exits and first errors are retained. This is not just a compile or startup probe.

## Windows guest wrapper

`windows_attended_aot.py` exposes only metadata preparation and one future fixed
AOT path, controlled by `AOT_EXECUTION_APPROVED=False`. Positive requests refuse
before preflight/tools. Both invocation helpers also refuse while disabled. No
command/RID selector, managed fallback, environment override or new authorization
protocol. Existing direct contributor AOT tooling retains its interactive policy.

Reuse the accepted common Windows preflight **without --run-managed**. It verifies
the packet, native roots, live event/CIM/uptime continuity, user/session/desktop
and owner CONFIRM. The AOT wrapper uses a TTY-preserving tee to recover exactly the
reported native path, validates its parent and records its own AOT scope metadata.
The shared preflight report retains its original managed-default entry label;
`aot-preparation.json` identifies the fixed AOT entry. Neither is GUI consent.

The future AOT invocation runs only the unchanged script through runpy with the
frozen source/scripts import root, real terminal streams and private environment.
The existing runner independently requests affirmative GUI consent. No response
or permit is supplied/reused. Scoped opt-out reaches restore/publish children;
caller environment/cwd/argv/path are restored afterward. No machine/project policy
changes. The native application's existing pre-Avalonia live broker check remains.

## Prerequisites and artifact identity

Microsoft documents Visual Studio 2022+ with Desktop development with C++ and its
default components for Windows Native AOT:
[official prerequisites](https://learn.microsoft.com/en-us/dotnet/core/deploying/native-aot/).
This wrapper additionally refuses missing/non-x64 tools, wrong Hostx64/x64 MSVC
pair, missing component discovery, nonstable SDK 10, wrong dotnet/Rust host or
missing selected x64 SDK/UCRT/MSVC libraries. It records tool/library paths/hashes
and query output, installing nothing. Windows environment names are matched
case-insensitively. Package restore during a later publish remains ordinary project
dependency work; successful metadata checks do not prove Native AOT compilation.

Additional result checks require exactly one original successful win-x64 summary,
warning-free publish, native image and adjacent child x64 PE headers, no managed
desktop DLL, exact published-file inventory/hashes, actual four distinct native
application/peer paths, nonempty independent backend values, matching Windows
session and four zero charges. Existing functional checks stay in the unchanged
runner; these completeness checks do not replace them.

Source/build-input manifests and live boot/context are checked before/after. The
Windows AOT observer retains the managed policy's strict root/command scope and
adds cl/link/rc/ilc to cover globally installed compiler actors. Selected processes
record PID, creation ticks, command/path and snapshot UTC; missing start identity
blocks. Private-root published apps/peers/ILCompiler/collectors remain in scope.
There is no extra cleanup wait, collector exclusion or kill to obtain success.
The accepted managed observer is not modified. This is a cooperative snapshot,
not complete hostile-process or system-wide isolation proof.

## Evidence, failures and gates

`launch_windows_aot.py` verifies a pinned packet, captures Python-level output and
the exact native report, hashes originals and verifies deliberate report export.
The original AOT runner also retains native build/publish/desktop logs. No latest-
directory guessing, automatic retries or host fallback. J: is import/export only;
source/cache/publish/temp/persistence/original reports stay guest NTFS.

Setup/consent/incomplete-evidence refusals remain Blocked. Existing runner failures
remain Failed even with later cleanup errors. Preserve its native published-app
180-second timeout semantics and owned-child termination on timeout; the wrapper
does not claim successful descendant cleanup then. Publish duration is not a new
lease or active-GUI timeout. No new revocation/supervision system is added.

Return design, source additions, non-GUI tests, host-only metadata and immutable
disabled candidate for parent review. No positive publish/execution or owner GUI
command is issued. Actual guest prerequisites, warning-free native publish and
real native peer execution are future gates, not inferred from managed success.
Ubuntu GUI/AOT, ARM64/Rosetta, unattended initiation and orchestration remain
separate. Blank screens/flakiness remain unresolved. No repeated sentinel absent
concrete changed conditions/contrary observation. G1/G2 and managed/headless gates
closed; G3 held; no product/history changes.
