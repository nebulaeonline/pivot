# Mac x64 tools with native Python orchestration

Owner authorized x64 Rust installation. Installed with rustup:
`nightly-x86_64-apple-darwin`, minimal profile plus rustfmt/clippy. Includes x64 Rust
compiler and standard-library target; default remains nightly-aarch64-apple-darwin.
Rustup required its explicit non-host option because physical hardware is ARM64;
actual x64 compiler execution under Rosetta was then verified.

Discovery now passes using ARM64 Python, x64 Rust 1.101.0-nightly (12433c307,
2026-10-24), and x64 .NET SDK10.0.403/runtime10.0.14:
`artifacts/validation-matrix/20261025T203914Z-df968f01/`.
Python architecture is not the test RID. The runner checks selected Rust/.NET host
architectures, records controller RID separately from tool execution RID, and
requires explicit emulation configuration. Native AOT derives its RID from actual
.NET host architecture rather than Python/platform.machine. No Python installation
or replacement was needed. Original functional assertions remain unchanged.

First real x64 CI failed in Rust linking because `/usr/bin/cc` invoked an x64
xcrun launcher while installed Command Line Tools/libxcrun are ARM-only. Original
failed log is retained:
`artifacts/validation-matrix/20261025T204021Z-2cd18dc0/osx-x64/`.

A small native-local Rust probe with the installed direct CLT Clang produced a
Mach-O x86_64 executable and executed successfully, printing x86_64. Probe:
`artifacts/validation-ssh/mac-x64-link-probe.log`; native original under
`~/PivotValidation/tool-probes/x64-link-x7_rvkze/`. ARM64 Clang can link x64 output
when given the target architecture; no developer-tool installation was changed.

The x64 private environment now selects direct CLT Clang/linker, x64 .NET root and
x64 Rust toolchain. Other target environments and global shell/Rust defaults stay
unchanged. Eleven focused orchestrator checks pass, including ARM Python controlling
x64 tools, Native AOT host selection and private environment restoration.

Headless x64 CI with the linker correction **Passed all 13 existing gates**:
`artifacts/validation-matrix/20261025T204253Z-dad8b2b5/osx-x64/`.
Original CI summary: `ci/20261025T204301347888Z/summary.json` within that directory.
Result records ARM64 controller, osx-x64 tool execution under Rosetta, and the
built release pivot-session as Mach-O x86_64. Rust tests, .NET build, parity, IPC
and application-session checks all passed with unchanged functional assertions.
No GUI/AOT acceptance run. Native APFS source/build/cache/report roots and ordinary
SSH source/report transfer remain in use; Remote Login Full Disk Access stays off.
No Pivot engine/workbook/IPC/persistence behavior changes; G3 remains held.
