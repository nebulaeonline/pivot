# Mac SSH automation and native ARM64 CI

Mac endpoint: robhorvath@192.168.5.103, hostname Robs-Mac-mini.local, ARM64.
Public-key authentication works; reported host fingerprint matches real handshake.
No Mac security settings were changed by the agent. Owner enabled Remote Login
without Full Disk Access, and installed only the validation public key.

SSH initially could not read `/Volumes/RobMac/coding/pivot` (Operation not permitted).
The orchestrator now supports ordinary SCP source deployment to fresh native remote
checkouts; no full-source hashes or review packets. Mac source staging is
`/Users/robhorvath/PivotValidation/remote/sources/<run>/<rid>`.
Worker builds/reports/caches remain local APFS; source revision/dirty state travels
with the ordinary ZIP. The original share need not be visible to the SSH session.

Native ARM64 discovery passes:
`artifacts/validation-matrix/20261025T203002Z-49788c26/`.
Actual native ARM64 headless CI **Passed all 13 existing gates**:
`artifacts/validation-matrix/20261025T203033Z-b7334587/`.
Returned original logs/reports:
`artifacts/validation-matrix/20261025T203033Z-b7334587/osx-arm64/` (retrieved by SCP).
No GUI/AOT run. Eight focused orchestrator checks pass, including local APFS versus
SMB rejection; original headless test assertions remain unchanged.

Setup corrections were direct implementation fixes: distinguish failed worker
launch from export failure; deploy without SMB access; add installed Mac .NET paths
to the child PATH; identify local APFS via df/mount rather than diskutil directory
lookup (which returned Could not find disk). Early failed setup/discovery reports
are preserved. No extra acceptance cycle or privacy permission was requested.

Follow-up: [Mac x64 toolchain and full headless CI](2026-10-25-mac-x64-toolchain.md)
is now complete. Owner installed x64 SDK10.0.403; the agent installed the authorized
x64 Rust toolchain while preserving the ARM64 default. ARM64 Python controls the
x64 tools under Rosetta. All 13 existing gates passed; the Rust release executable
is verified x86_64. Earlier setup failures remain preserved in their original logs.

Current RID coverage: Windows x64 local full CI passed; Hyper-V Linux x64 full CI
passed; macOS ARM64 and Rosetta x64 full CI passed; UTM Linux ARM64 reached an
actual Rust test failure; Windows ARM64 SSH/toolchain setup pending.
Windows guest SSH setup scripts remain prepared, not claimed executed.

Remote Login Full Disk Access grants broader disk access per
[Apple's documentation](https://support.apple.com/guide/mac-help/mchlp1066/mac).
It may help with protected-location access, but was unnecessary for this working
native-deployment workflow. Keep it off. This is ordinary development automation,
not a forensic qualification assertion. G3 remains held; no product behavior changes.
