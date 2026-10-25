# Windows attended Native AOT — disabled packet ready for review

Subsequent [parent acceptance closes this disabled-preparation gate](2026-10-25-windows-aot-preparation-acceptance.md).
Actual guest prerequisite collection and enabled-candidate preparation are next;
positive AOT publish/execution remains unapproved.

2026-10-25. [Authorized preparation](2026-10-25-windows-managed-final-acceptance.md).
**Windows managed qualification stays accepted/closed. No AOT publish, AOT GUI,
managed GUI rerun or full headless CI was executed in this preparation.**

[Design and fixed-flow audit](../architecture/windows-attended-native-aot.md).

## Audited existing flow and necessary additions

The fixed existing `scripts/verify_native_aot.py` and NativeAot publish profile are
unchanged. They retain GUI consent before work, Rust build, native local-RID
publish, warning rejection, actual published executable/three-peer native execution,
adjacent Rust-child discovery, original functional assertions, zero charges,
published file hashes and WB5 verification. Native logs and first exits remain
original evidence. No reduced diagnostic-only suite or alternate command path.

New `windows_attended_aot.py` reuses the exact accepted common preflight, passing
no managed execution flag. It recovers the exact report path through a TTY-preserving
tee, verifies its native parent, records AOT-specific scope/prerequisites and uses
the existing private environment/opt-out. Shared preflight's managed-default entry
label is retained in its original report; `aot-preparation.json` records actual
AOT scope. CONFIRM is owner configuration acknowledgement, never GUI consent.

The fixed future AOT invocation uses runpy with the frozen scripts import root and
real streams. The unchanged AOT runner independently requests GUI consent. Stale
activation/authority is removed by the existing environment helper, caller state
is restored, and the published application/peers retain the existing live broker.
No response is supplied or reused. No second consent/capability protocol.

`AOT_EXECUTION_APPROVED=False` blocks positive CLI and both invocation helpers.
No managed fallback, arbitrary command or RID selector exists. The new bootstrap
verifies packet/report hashes and exports exact native originals. Additional
evidence checks require win-x64 PE images, exact publish inventory/hashes, no managed
desktop DLL, native-aot execution/path identity, four distinct process/session/
backend reports and zero charges. The Windows AOT snapshot includes native cl/link/
rc/ilc actors plus the existing scoped names, with PID/start ticks and observation
time. Managed snapshot code, filter and timing are unchanged. No new wait or kill.

## Prerequisites and actual evidence limits

The prerequisite adapter reads installed tool PE headers/hashes, vswhere C++
component metadata, stable SDK 10 and native dotnet/Rust host outputs, selected
Hostx64/x64 compiler/linker and x64 Windows SDK/UCRT/MSVC libraries. No tool or OS
installation, dependency restore, compilation or publish is triggered by it.
Windows environment lookup is case-insensitive. Missing/incorrect tools block.

Final host metadata was Verified using the installed Visual Studio 2026 x64
developer environment, with real library/tool hashes. Read-only AOT observer
smoke query also verifies. This is **host-only adapter evidence**, not qualification
of the guest's native toolchain or successful Native AOT compilation. No guest AOT
prerequisite result or warning-free native publish is claimed. The future guest
invocation must check its own installed VS2022+ C++ workload/SDK and publish outcome.
[Host metadata](../../artifacts/windows-native-aot-preparation/host-prerequisites.json).

## Non-GUI verification

Nine preparation tests pass in both the worktree and independently extracted frozen
candidate. Coverage includes disabled positive path before preflight/tool queries,
arbitrary command/RID/managed fallback refusal, original AOT runner noninteractive
refusal before output/publish, fixed invocation with real-stream/caller restoration,
case-insensitive Windows SDK metadata and x86/missing-library/prerelease refusal,
shared metadata-only preflight with prompt/path on one line, native artifact/hash/
runtime/backend/session/charge/PID rejection, native compiler survivor identities
and Failed/Blocked/Passed preservation. Invocation and artifact fixtures are
synthetic; no native GUI image is executed by tests.

An additional real invocation of the frozen disabled guest wrapper with --run-aot
returned Blocked/2 before preflight, publish or GUI. There were no failed preparation
test outcomes. No C# or project changes required another managed build; earlier
accepted warning-free build/managed execution remain their original evidence.

## Frozen disabled candidate

`artifacts/windows-native-aot-preparation/windows-aot-disabled-review-20261025/`

Receipt SHA256:
`e74a9fee3c44874cf2823becf181b61b37d163b79ca83371f6b2ae946133ad64`.

Independent verification covers **537 source entries / 265 inputs / fifteen
packet files**. Derived from the accepted scoped-opt-out managed packet. Every
previous source byte is unchanged, including AOT runner/profile, managed assertions,
boot contract, peer correction, opt-out and consent. Exactly four paths were added:
the two AOT helpers, their regression file and design document. All originals and
historical failures remain immutable.

The new AOT flag is false in both archived source and flat helper. The shared
managed helper retains its accepted snapshot state, but is invoked only for
metadata preflight; no new managed execution authority is created. Disabled means
the new guest AOT path refuses, not that the normal contributor tool is removed.
No positive owner command or enabling patch is applied/issued in this milestone.

[Source-addition diff](../../artifacts/windows-native-aot-preparation/aot-preparation.diff),
[disabled packet verification](../../artifacts/windows-native-aot-preparation/disabled-packet-verification.json),
[frozen non-GUI checks](../../artifacts/windows-native-aot-preparation/frozen-aot-negative-tests.log),
[hard-refusal output](../../artifacts/windows-native-aot-preparation/disabled-positive-refusal.log).

## Review and stop

Return design, fixed-flow audit, source additions, actual host-only prerequisite
metadata, non-GUI tests and disabled packet for parent review before enabling or
executing a positive AOT run. A later enabling change creates new source/packet
hashes; never mutate this disabled packet. Source/publish/cache/temp/persistence/
original reports must stay guest NTFS; J: is deliberate import/export only.

Future acceptance must establish actual guest prerequisites, warning-free native
publish, real main/three native peers/backends, adjacent child, zero charges,
unchanged source/input/publish hashes, live boot/context and empty scoped survivors.
Original failures and native runner timeouts remain meaningful; no fallback, retry
until green or proof of cleanup after interrupted execution is inferred.

No routine clipboard recheck, settings/reboot action or blank-screen intervention.
No unattended initiation, new broker, dispatcher, lifecycle operation or host
fallback. Ubuntu GUI/AOT, UTM ARM64 and Rosetta remain separate. Managed/headless
gates/G1/G2 stay closed; G3 held; G4–G7/production Undo/Redo deferred. No product changes.
