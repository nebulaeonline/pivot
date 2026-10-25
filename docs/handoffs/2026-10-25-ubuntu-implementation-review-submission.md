# Ubuntu x64 disabled implementation and native non-GUI results

Update: [parent accepts and closes this gate](2026-10-25-ubuntu-disabled-implementation-acceptance.md).
Enabled-candidate preparation/non-GUI validation is authorized next; positive GUI
execution remains unapproved. Submission below retains original review context.

**Ready for parent implementation review.** Fixed Linux wrapper, process observer,
terminal recorder and exporter are implemented with positive execution hard-disabled.
Actual corrected guest non-GUI adapters and 24 frozen tests verify. **No Ubuntu GUI,
AOT or build ran; no positive command is issued.**

Direction: [accepted session/contract gate](2026-10-25-ubuntu-disabled-contract-acceptance.md).
Implementation: [initial packet](2026-10-25-ubuntu-disabled-implementation.md),
[first Blocked result and observer correction](2026-10-25-ubuntu-observer-scope-correction.md),
[architecture](../architecture/ubuntu-attended-managed-gui.md).

## Exact packet and implementation

Review packet:
`artifacts/ubuntu-managed-implementation/ubuntu-disabled-observer-correction-20261025/`.
Receipt SHA-256:
`7a8450f4ccd5d1f1b39b6ecd10fb8e2fa87d319824ae9a439f8fb70a1843a4c3`.
544 source files, 271 inputs, ten receipt-covered payloads. Relative to the accepted
prototype, only the contract helper and architecture document change; additions
are runtime helper, original collector source, runtime tests and fixed recorder.
All other prior source bytes, application, original UI runner, assertions and
consent remain unchanged. Positive flag stays false. All original packets remain.

Live preflight checks reviewed boot UUID, exactly one suitable graphical login and
matching active seat, expected user/guest corroboration, socket ownership/device/
inode, user cgroup, native ext4 roots, selected stable SDK10, native x64 tools,
Rust GNU host and selected native libraries. It preserves actual display variables,
failed terminal-self logind lookup and owner-attested management binding without
sudo or invented identities. Owner CONFIRM stays separate from original GUI consent.

Fixed supervision invokes only unchanged `ui_smoke.py`, preserving real streams,
original timeout/assertions and private native roots. The installed util-linux
recorder supplies a genuine PTY and retains native build/application console output;
no input or consent responses are supplied. Exact report path, native hashes and
copied/exported evidence are verified. Export failures preserve local originals
for copy-only recovery.

Cooperative /proc polling records actual UID, PID/start ticks, executable/command,
ancestry to wrapper, cgroup and selected display fields. Main/three peers require
distinct managed reports/backends, actual Rust-child identity and four zero charges.
Missing/reused process identity cannot pass. Linux process-session fields are not
logind IDs. Strict final snapshot adds no cleanup wait, exclusion or unrelated kill.
Polling can miss a transient process; required missing identity Blocks.

## First Blocked result and precise correction

First guest result `ubuntu-wrapper-nongui-b43536af8d594dbe820ca39ee97b2bd5` remains
**Blocked**, with verified original hashes. Its tests/context/tools passed before
overbroad executable inspection and checker ancestry caused permission failures.
It was not a GUI attempt and is not reclassified as passing.

Correction stops ancestry at the wrapper root. Unreadable executable metadata is
outside invocation scope only when process start strictly predates wrapper start,
name is not a build/runtime actor and readable command has no work-root reference.
Raw permission denials are retained. Relevant actors/root-referenced/new/unknown
identities still Block. No named-service exemption or applicable survivor was
removed. This is a cooperative scope boundary, not a defense against a preexisting
same-user adversary changing execution state. Parent review of this boundary is
requested explicitly.

Compared with first implementation receipt
`322def344aa89f1032d9662fcb08c59dfd17a5d0d454e899acac0a3a63849900`,
only runtime helper, its tests and architecture documentation change, plus matching
archive/hash packaging. Independent archive comparison confirms identical paths
and all other bytes. Original packet/results/logs remain immutable.

## Actual corrected guest evidence

Returned report:
`artifacts/ubuntu-managed-implementation/non-gui-results/Pivot-Ubuntu-x64/ubuntu-wrapper-nongui-5d5258456dbf4918a8be5ffd52703c90/`.
Native originals:
`/home/robhorvath/pivot-preflight/ubuntu-wrapper-nongui-5d5258456dbf4918a8be5ffd52703c90/report`.

Seven origin entries verify; native/returned manifest SHA-256 agrees, recorded in
`artifacts/ubuntu-managed-implementation/corrected-guest-verification.json`.
Returned receipt/source/input manifests match the exact corrected packet, and
returned checker bytes match the issued checker. Source-after is unchanged.

All 24 frozen tests pass on the actual guest (three contract, 21 runtime). Same
frozen host checks pass; 21 runtime tests also pass in WSL. Actual read-only adapter
checks verify expected user, session 2/seat0, reviewed unchanged boot and display
socket identities before/after, ext4 roots, tools/libraries and util-linux script
2.41.3. Selected SDK is 10.0.113; Rust host remains x86_64-unknown-linux-gnu.

Process snapshot is Verified with no observer errors and one match: the checker
PID 30184, UID1000, start ticks 8497912, own executable/cgroup/display and ancestry
ending at itself. **This is not an empty post-GUI survivor check.** No application
was launched. Actual main/peer identity correlation and cleanup remain future
execution requirements, tested only through non-GUI fixtures so far.

Three original service executable denials remain visible outside scope: systemd
3401/start2490, sd-pam3403/start2492 and ssh-agent3683/start2529, all preceding
wrapper start8497912. None is silently discarded from the evidence. No privileges,
VM integration settings or clipboard state were changed.

## Requested review boundary

Review disabled operational implementation, exact correction, native adapter
evidence and non-GUI tests before any enabling or positive managed GUI attempt.
Actual Avalonia backend, four-process behavior, full runtime cleanup and native
subprocess recording are not proven by this non-GUI run. No speculative backend
choice or runtime-session claim is made. DMI UUID/serial limitation remains.

No more guest work is requested now. Longer future Ubuntu commands remain shared
scripts with short launch commands and automatic report return. Windows gates and
Ubuntu headless stay accepted/closed; Ubuntu AOT, ARM64/Rosetta, unattended activation
and orchestration remain separate. G1/G2 closed; G3 held; no product changes.
