# Windows VM SSH automation

Parent review accepted and closed both Windows SSH integrations and native
headless results. No further Windows connectivity qualification is required.
See [five-RID acceptance and next scope](2026-10-25-five-rid-headless-acceptance.md).

ROBPC is controller-only. Windows validation runs inside Hyper-V
Pivot-Win11-x64 and UTM Pivot-Win11-arm64, with native-local NTFS source,
build, cache, temporary and report roots. Source and results move through SSH/SCP;
neither shared J: builds nor the earlier ROBPC pass satisfy these guest targets.

Both Windows guests now have working key authentication. Their Ed25519 server
fingerprints were checked against the returned setup reports before recording
them in separate project known-hosts files. The dedicated private validation key
remains on ROBPC; it was not copied to the guests or shared storage.

| Target | Endpoint | Verified tools | Headless result |
| --- | --- | --- | --- |
| Hyper-V Windows x64 | User@192.168.5.118 | Native x64 Rust nightly, .NET SDK 10.0.402 | Passed 13/13 |
| UTM Windows ARM64 | robhorvath@192.168.5.125 | Native ARM64 Python/Rust nightly, .NET SDK 10.0.403 | Passed 13/13 |

Windows x64 evidence:
`artifacts/validation-matrix/20261025T233611Z-f3b72f89/win-x64/`.
Original CI summary: `ci/20261025T233619516104Z/summary.json`.
All 13 existing steps passed, including Rust tests, .NET build, safety, decimal
and protocol parity, IPC and application-session checks. Discovery:
`artifacts/validation-matrix/20261025T233550Z-755d9237/`.

ARM64 discovery:
`artifacts/validation-matrix/20261025T234006Z-b7df87c2/`.
ARM64 full headless run:
`artifacts/validation-matrix/20261025T234038Z-222a8ed9/`.
All 13 existing ARM64 gates passed; original summary is
`ci/20261025T234051221131Z/summary.json` within its returned report directory.
Both Windows result.json files confirm native target architecture and local NTFS.

Setup failures and diagnostic reports remain unchanged. Initial service starts
failed with exit 1067. Correcting SSH directory permissions alone did not resolve
them. Administrator foreground startup succeeded, but SYSTEM diagnostics emitted
the private-key protection warning banner. Key ACL metadata showed installer-user
ownership and explicit account access. Normalizing private host-key ownership to
SYSTEM and access to SYSTEM/Administrators resolved startup on both guests.
The key contents and fingerprints were retained. Original ACL metadata was backed
up without copying private-key contents into reports.

ARM64 then moved from UTM NAT to bridged networking. Its Public network was blocked
by the existing Private-only SSH firewall rule. Setup now enables that SSH rule
across profiles while retaining LocalSubnet remote scope. Connectivity and key
login succeeded afterward. No global firewall, network-category or UAC changes.

PowerShell's native argument handling stripped double quotes in the source ZIP
extraction expression. Using a single-quoted Python archive name fixed deployment;
the original failed discovery is retained at
`artifacts/validation-matrix/20261025T233508Z-c7c076d3/`.

No GUI/AOT acceptance is claimed by these headless runs. Existing functional
assertions remain unchanged. Linux ARM64's original Rust failure remains a separate
unresolved investigation. G1/G2 remain closed; G3 remains held.
