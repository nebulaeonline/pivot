# Six-RID orchestrator — first practical execution

The [reset direction](2026-10-25-validation-infrastructure-reset.md) is implemented
as [scripts/validate.py](../../scripts/validate.py), with six-target configuration
and [usage documentation](../validation.md). No forensic observer or immutable
execution packet is used. All original historical Blocked results remain unchanged.

First existing headless CI execution through the orchestrator:
`artifacts/validation-matrix/20261025T200239Z-7543fef2/`.
Windows x64 and Linux x64/WSL **both Passed**, each completing all 13 original gates.
Builds ran from native local workspaces, not J:/SMB/DrvFS. Results/logs returned under
`artifacts/validation-runs/20261025T200239Z-7543fef2/`. Git commit recorded as
`a78f112cda34f530cc2c34ae07cd914125db58ec`, dirty working tree; no commit/staging done.
These were local host/WSL executions, not Hyper-V guest executions.

| RID | Current orchestrator evidence | Remaining setup |
| --- | --- | --- |
| win-x64 | Full headless CI Passed on local Windows | Configure Hyper-V guest SSH for that backend |
| linux-x64 | Full headless CI Passed in WSL | Configure Hyper-V guest SSH for that backend |
| win-arm64 | NotConfigured | UTM guest SSH/toolchain endpoint |
| linux-arm64 | NotConfigured | UTM guest SSH/toolchain endpoint |
| osx-arm64 | NotConfigured | Mac SSH alias and mounted source path, or local Mac configuration |
| osx-x64 | NotConfigured | Actual native/emulated x64 environment and matching tools; execution required |

Discovery report: `artifacts/validation-matrix/20261025T200201Z-b0e33273/`.
The two local targets are Available; four remote targets explicitly NotConfigured.
Metadata-only discovery is never a test pass. The headless run selected only two
targets; its Passed status is not six-target acceptance. Machine-readable JSON and
readable matrix are retained for each command.

Features: local/WSL/SSH dispatch, explicit target/suite selection, concurrent headless
dispatch (default two jobs), serial opt-in GUI, native work roots, tool/architecture
checks, ordinary Git revision/dirty state, logs/exits, clear setup/partial/failure
statuses, automatic report return and explicit emulation. Reuses existing CI,
managed UI and AOT runners. GUI uses the existing live interactive consent broker;
no consent is manufactured or silently bypassed. No GUI was launched in this work.

Native AOT host-RID selection now recognizes all six execution RIDs; original
assertions/profile/publish behavior remain unchanged. This is implementation
support, not proof of ARM64/macOS x64 publish or execution. Seven focused tests pass,
including wrong-architecture refusal, missing reports, actual owned-survivor failure,
headless entry selection and six-RID AOT mapping.

Initial discovery exposed duplicate creation of shared temporary aliases and WSL's
non-login PATH omitting installed Rust tools. Both normal implementation defects
were fixed directly and subsequent discovery/CI passed. First failed logs remain.
No new parent-review gate was introduced for those fixes.

Outstanding: remote SSH endpoints/authentication and source paths, Mac x64 execution,
actual remote transport trials, remaining target tool setup and GUI/AOT integration
runs with explicit consent. Hyper-V IP lookup was attempted read-only but this
session lacks host management permission. No sudo, VM setting change, SSH service
enablement or credential installation occurred. Owner input is pending on existing
SSH aliases or permission to prepare key-only SSH setup scripts. SSH is generic to
Hyper-V and UTM; start/stop orchestration is not yet implemented.

Owner follow-up: Ubuntu x64 has sshd installed but cannot currently accept the
intended connection; Ubuntu ARM64 accepts password login. Neither has the validation
key installed. Both Windows guests lack SSH/remoting. Read-only endpoint/service
scripts and [setup notes](../ssh-setup.md) are prepared; no service/firewall/key
changes performed. Mac share path was recovered from existing results and populated
in target defaults, avoiding a request to repeat known information.

Owned-process GUI cleanup is practical rather than forensic: required original
application reports, zero charges, reported application/peer/Rust PID liveness and
launched POSIX process-group checks. Windows Job Objects and detached-child coverage
remain limitations disclosed in usage docs. Actual child/assertion/resource
failures remain failures; unrelated process churn is not a gate. GUI/AOT positive
paths have not been exercised by this new orchestrator and are not declared passed.

No engine/workbook/persistence/IPC/product-limit or production Undo/Redo changes.
G3 remains held while practical multi-platform automation is completed.
