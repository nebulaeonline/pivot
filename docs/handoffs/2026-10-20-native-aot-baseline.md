# Native AOT baseline — implementation and validation

Date: 2026-10-20. **Windows x64, Linux x64 and native macOS ARM64 publish/run
validation pass; implementation accepted, milestone closed.** No text/projection/connector or installer work is included.
[Bounded assignment](2026-10-20-native-aot-next-scope.md),
[native Mac packet](2026-10-20-macos-native-aot-instructions.md).

## Changes

`NativeAot.pubxml` explicitly requests Native AOT/self-contained publication and
detailed trim warnings. Ordinary development and `python scripts/ci.py --desktop`
remain managed. The desktop disables reflection-based System.Text.Json even in
managed runs; named DesktopReport/PeerReport/RenderTiming DTOs and source-generated
metadata replace anonymous report serialization. Existing keys remain, with additive
execution mode, image path and startup diagnostics. JsonDocument parsing remains
explicit, without reflection-based typed deserialization. XAML remains compiled,
with compiled bindings the project default. No dependency upgrades, warning suppressions,
root-all-assembly descriptors or UI framework changes were needed.

Helper windows start the current executable. Framework-dependent `dotnet app.dll`
execution supplies the adjacent DLL argument; managed apphosts and Native AOT images
launch directly. Helpers inherit the correct image/test environment. Rust remains a
separate native process over unchanged private Protobuf. The shared Rust-path resolver
supports a developer override or an adjacent child. No FFI or codec/capacity change.

## Separate native verification

```sh
python scripts/verify_native_aot.py
```

The script selects the native baseline RID (win-x64, linux-x64 or osx-arm64), records
SDK/compiler prerequisites, builds Rust, publishes through NativeAot with warnings
treated as errors, retains complete publish output, and rejects emitted warnings.
It copies the native Rust child beside the published desktop solely for validation;
ordinary `dotnet publish` does not bundle Rust automatically. This is not an installer.

Execution unsets PIVOT_SESSION_PATH, launches the published image directly and runs
the actual window suite. Every main/peer report must identify native-aot execution,
the exact published image path, successful functionality and zero Rust charge.
No Pivot.Desktop.dll may appear as the published application. Self-contained managed
or ReadyToRun execution cannot pass those checks. Scripts/test harnesses remain
Python/.NET; they are not all required to be AOT executables.

The native suite exercises rendered startup, real Rust children, number/formula/SUM
editing, rectangular selection/native clipboard across processes, arrow editing,
WB3 save/reopen and rejected drafts, actual dirty-close Cancel/Save/Discard, clean
close, independently reopened generations and orderly zero-charge shutdown. Failure,
missing evidence, timeout or wrong image/mode produces a failed summary. No recurring
manual gate was added. Physical keys, IMEs/accessibility and native pickers retain
the existing targeted spot-check boundary.

## Runner serialization follow-up completed

Both ordinary and Native AOT window runners hold one atomic exclusive-file lock for
the whole clipboard suite, including peers. Windows uses its user's LocalAppData;
WSL resolves that same Windows location through interop and wslpath. WSL fails closed
if it cannot resolve the shared location. Mac uses the user's native cache; native
Linux uses its runtime directory. An explicit override must designate the same file
for suites sharing a clipboard. Build/publish can overlap; clipboard suites queue.

Each acquisition has PID/platform/UUID metadata and a bounded 300-second wait. Release
removes only its matching ownership record. Unknown/stale owners are never stolen;
after a crash, verify the suite and children have stopped before manual lock removal.
Three retained Python regression tests cover independent-process exclusion, failure
cleanup and unknown owners; they run in desktop CI and the Native AOT gate.
Real Windows→WSL and WSL→Windows exclusion checks also pass:
[Windows-held proof](../../artifacts/native-aot-cross-os-lock.json),
[WSL-held proof](../../artifacts/native-aot-cross-os-lock-reverse.json).
This completes the narrow operational follow-up from the clipboard acceptance review.

## Results

| Host | Native publish/run | Desktop image bytes | Runtime payload bytes | Main-to-ready observation |
| --- | --- | ---: | ---: | ---: |
| Windows x64 | Passed, no warnings | 22,190,080 | 42,747,944 | 518.50 ms |
| Ubuntu/WSLg x64 | Passed, no warnings | 24,955,616 | 41,377,272 | 360.30 ms |
| Native Mac ARM64 | Passed, no warnings | 18,994,240 | 40,813,936 | 910.22 ms |

Payload includes the Rust child and published native dependencies, excluding debug
symbols. The complete output including symbols is 233,328,700 bytes on Windows and
107,636,364 on Linux and 97,513,918 on Mac. These are recorded build outputs, not installer/download sizes.
Startup is one observation from Main entry through the first awaited rendered frame
and Rust bootstrap; OS loader time is excluded. These observations are not comparative
benchmarks, distributions or performance guarantees.

- [Final Windows native summary](../../artifacts/native-aot-windows/20261021T024635162290Z/summary.json)
- [Windows native window](../../artifacts/native-aot-windows/20261021T024635162290Z/desktop-ui/results.json)
- [Final Linux native summary](../../artifacts/native-aot-linux-local/20261021T024455105110Z/summary.json)
- [Linux native window](../../artifacts/native-aot-linux-local/20261021T024455105110Z/desktop-ui/results.json)
- [Windows ordinary CI](../../artifacts/native-aot-managed-ci-windows/20261021T024255064489Z/summary.json)
- [Linux ordinary CI](../../artifacts/native-aot-managed-ci-linux/20261021T024749357268Z/summary.json)
- [Linux input verification](../../artifacts/native-aot-linux-input-verification.json)
- [Windows input verification](../../artifacts/native-aot-windows-input-verification.json)
- [178 build/test input hashes](../../artifacts/native-aot-build-inputs.json)

Windows used .NET SDK 10.0.402 and installed Visual Studio C++ tools; Linux used
.NET SDK 10.0.113, clang 21.1.8 and Ubuntu 26.04/glibc 2.43. Exact inventories and
compiler versions are retained with each summary. Linux source and .NET bin/obj are
under `/var/tmp/pivot-native-aot-20261020`; the final publish and execution are also
on ext4. Reports were returned without the publish directory; native binaries/symbols
remain in that preserved source tree. Windows published binaries remain with reports.
Linux native file tests use `/var/tmp/pivot-experimental-file-tests`.

Ordinary Windows/Linux full CI and managed desktop checks pass with 309/310 default
Rust tests. All 178 current build/test input hashes match on both hosts. Rust sources are unchanged
from the accepted 309/310 default and 325/326 all-feature baseline. Native AOT and
managed main/peer reports retain zero-charge close and the accepted semantics.

## Native Mac integration

The [fresh Mac report](../../artifacts/macos-validation-20261021T025530Z/REPORT.md)
passes managed CI with desktop automation, strict all-feature lint, 310 default /
326 all-feature Rust tests and the separate Native AOT gate. Native source, builds,
TMPDIR, file fixtures, publish output and original reports all used internal APFS.
The desktop and adjacent Rust child are Mach-O ARM64, with Rosetta translation off.
Apple clang 21.0.0/Command Line Tools were recorded.

The [native summary](../../artifacts/macos-validation-20261021T025530Z/native-aot/20261021T025804949956Z/summary.json)
has no publish warnings. Main and all three peers executed the exact published
native image, with adjacent Rust discovery and zero charge. All seven acceptance
fields pass, including cross-process private UTI clipboard, WB3 save/reopen and
Cancel/Save/Discard/clean close. Ordinary main/peers retain managed execution.
Both lock ownership records remain; the shared cache lock was released without
stale-lock removal. No owned application or child processes survived.

The [independent review audit](../../artifacts/native-aot-macos-review-verification.json)
confirms all 349 copied source hashes and 178 build/test inputs against this workspace
before these documentation edits, independently counts Rust results and checks main/peer
reports and report links. All 426 returned files are present and locally hashed;
the Mac transfer record reports SHA-256 equality for every returned file. Native
binaries/symbols remain on preserved APFS with size/hash inventories in the summary.
A returned native clipboard screenshot was inspected: the rectangle, active cell,
SUM formula and computed result are visible. This is not physical-input certification.

No product edits, gate reruns or manual gate were needed. An initial sandbox-restricted
sysctl prerequisite probe used approved native execution; an optional guessed Python
path was absent. Both setup observations remain recorded, separate from passing gates.

## Findings, limits and next step

The first Windows native publish/run succeeded without compatibility warnings.
Its startup stopwatch initialized too late and reported an invalid near-zero metric;
Main now explicitly restarts it, and later native runs provide the observations above.
The original report remains intact and must not be used as startup evidence.
Initial sandboxed Python lock tests lacked temporary-file access; unsandboxed tests
pass. WSL lacked clang/zlib development prerequisites; sudo required authentication,
and the user installed them. No privileged workaround was used. An initial Linux run
published to the Windows artifact mount; the final run repeated execution from ext4.
No trim/AOT warnings were suppressed to obtain a pass.

This validates current executable paths on these hosts, not clean-machine deployment,
minimum OS/glibc compatibility, arbitrary future plugins, physical-input handling or
power-loss durability. Prior WSLg native startup abort, mounted-path installation
uncertainty and historical Linux viewport p95 exception remain unresolved. Native
Mac APFS success does not resolve those separate findings or establish compatibility
with untested OS/dependency combinations.

Cross-platform validation and Mac evidence integration are complete. The parent
review received 2026-10-20 accepts the implementation and closes this milestone.
[Acceptance and next design scope](2026-10-20-native-aot-review.md).
Worksheet text and structured-result/projection design remain subsequent work.

## Reference guidance

[Avalonia Native AOT](https://docs.avaloniaui.net/docs/deployment/native-aot),
[Microsoft Native AOT prerequisites/deployment](https://learn.microsoft.com/en-us/dotnet/core/deploying/native-aot/),
[System.Text.Json source generation](https://learn.microsoft.com/en-us/dotnet/standard/serialization/system-text-json/source-generation).
These guide the compatibility work; executable results above are the acceptance evidence.
