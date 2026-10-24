# Execution safety accepted; guest qualification is next

2026-10-24. **The first execution-safety milestone is accepted and closed.**
Active workstream: optional isolated CI infrastructure. **History G3 remains held;
do not begin its design, dependencies, experiments or implementation.**

Source: owner-supplied parent review attachment
`29402cb6-3d95-4a5a-9f40-910d3a7e3877/Pasted text.txt`.
[Implementation](2026-10-24-execution-safety.md),
[Mac reconciliation](2026-10-24-macos-execution-safety-validation.md),
[policy and transitive audit](../architecture/validation-execution-safety.md),
[G2 closure/G3 hold](2026-10-24-g2-acceptance-g3-held.md).

## Accepted gate

The parent accepts the common policy, interactive consent, live broker, pre-Avalonia
managed/native self-check guard and nested-peer authorization. Headless CI remains
VM-independent; declined/invalid/noninteractive local GUI requests return Blocked /
exit 2. Existing assertions, contributor workflows, clipboard lock and fixture-
contained fault tests remain intact. No automatic host fallback was introduced.

Accepted evidence: 13 headless CI steps on Windows, native WSL Linux and native Mac;
347/348/348 default Rust tests and nine policy tests each. Windows additionally
compiled warning-free Native AOT and passed 11 refusal checks. Mac matched 247
inputs with 366 independently verified evidence hashes. Its concurrent documentation
drift is separately reconciled, not relabeled as unchanged shared source.
Preserve original failures, reruns, frozen manifests and native-origin evidence.

Limitations remain explicit: the broker prevents accidental execution, not hostile
same-user code, and proves no VM isolation. Interactive consent makes disruption
intentional, not harmless. No unattended guest profile is accepted. Negative/refusal
tests are not positive managed GUI or AOT acceptance after the guard change.
Do not close that gap by running unattended tests on the owner's active desktop.

## Next authorized deliverable — qualification only

Prepare a narrow, primarily read-only guest-capability/isolation preflight packet.
Start with Hyper-V `Pivot-Win11-x64` and `Pivot-Ubuntu-x64`; qualify UTM
`Pivot-Ubuntu-arm64` separately. UTM `Pivot-Win11-arm64` remains optional.
Names and installation alone establish no capability or accepted coverage.

Collect actual guest OS/release/kernel/hypervisor, process and executable
architecture, compiler/linker, Rust host/target, .NET SDK/RID, filesystem and native
work-root evidence. Identify actual graphical backend and session ownership.
Require host-side clipboard/drive/folder/device integration inspection; guest
information alone is insufficient.

Propose a deliberately scheduled harmless clipboard sentinel procedure demonstrating
that guest clipboard changes do not modify the host clipboard. Do not execute it
implicitly, read personal clipboard data, use email/production files or rely on
restoration as a safety guarantee. Any host state preservation must be explicitly
controlled and safe. Approval binds an individual VM configuration and graphical
session; relevant host integration or session changes invalidate it.

Report Verified, NotConfigured, Blocked, Unsupported and genuine failed checks
separately. Missing optional guests neither break contributor CI nor create passes.
Return actual inspected settings, missing prerequisites, native roots and proposed
authorization boundaries for review. No guest has been inspected by this receipt.

## Stop before activation

No SSH/PowerShell remote dispatcher, VM lifecycle automation, final `--vm` flags,
automatic provisioning/package installation or unattended GUI authorization yet.
Obtain parent approval for each guest activation profile separately. Once approved,
its later first positive managed-desktop and Native AOT publish-and-execute run must
reuse existing assertions and prove nested authorization, real peers, host clipboard
separation, native storage, source integrity and zero-charge cleanup. Preflight
alone must never be called graphical acceptance.

G1/G2 remain closed; G3 is held and G4–G7/production history/UndoRedo remain deferred.
No WB5, CB2, private v4/hosted v1 protocol, graph, limit or persistence changes.
Leave the Ubuntu 18.04 library compatibility environment unchanged. Do not diagnose
earlier intermittent failures as host interference without evidence.
