# Pivot implementation status

Date: 2026-10-27

**Parent accepted the hierarchy/normalization and separate dictionary research.
G3 remains open; physical-storage accounting and scalable admission are next.**
[Acceptance, revised objective and exact next scope](handoffs/2026-10-27-g3-hierarchy-dictionary-acceptance.md).
The 20% metadata figure is now an unproven research aspiration, not a mandatory
engineering gate. Assess indispensable bytes versus duplicate/index/page costs and
predictable growth. Document larger admission bounds and worst-case costs before
expanded execution; return with the assessment/proposal and readiness comparison.
New feasibility work has not started. No further codec optimization round is planned.

The accepted hierarchical authentication and lossless receipt normalization reduce
genuine 10K retained
history to **12.52–12.55 MB**. Publication updates existing path nodes; the application
retains no superseded paths. Logical index writes fall from about 71.5 MB to 32.8 MB.
All receipts, canonical hashes, lifetime authority and reconstruction oracles survive.
Separate branch/leaf tables avoid the mixed-table page-density regression. Growth
probes retain 16,385 receipts through 421–427 leaves; the 1,024-leaf research bound
remains and full 100K G2 work is not authorized.

Native Windows NTFS, Ubuntu ext4 and Mac APFS qualification: **249 passes, three
explicit skips**. Broader overhead remains **57.8–58.2%** of compressed payload;
the 20% aspiration remains unmet and complete production feasibility remains open.
[Hierarchy/normalization results](handoffs/2026-10-27-g3-hierarchy-normalization-results.md) ·
[Experimental contract](architecture/history-hierarchy-normalization-experiment.md).

The owner subsequently authorized a separate trained-dictionary zstd investigation.
One frozen 1K training corpus and identical dictionaries were tested on all native
10K histories: **18 additional codec test passes**, and 21 candidate databases passed
complete audit/reconstruction. Dictionaries reduce payload bytes but yield only about
1% total-size savings in the best dictionary case; plain zstd-9 is slightly smaller.
Normalized receipts do not benefit, and the 20% metadata goal remains unmet. The
unchanged zlib baseline retains its own evidence; no production codec is selected.
[Dictionary results and limits](handoffs/2026-10-27-g3-dictionary-results.md) ·
[Dictionary/corpus contract](architecture/history-dictionary-experiment.md).
G1/G2 and six-RID execution acceptance remain closed; G4–G7 remain gated.

Parent review **accepted and closed six-RID headless CI, managed GUI and
Native AOT execution validation**. Every designated RID has passed each mode.
Windows ARM64's instrumented pass is accepted as execution evidence. Its original
and diagnostic failures remain Failed; clipboard reliability is not established.

Evidence under `artifacts/validation-matrix/`:

- Headless: `20261026T002242Z-f76230c2/`.
- Managed GUI: `20261027T025246Z-deb1324f/`.
- Five AOT passes plus original ARM64 failure: `20261027T025246Z-b82c5b95/`.
- Accepted Windows ARM64 AOT pass: `20261027T144914Z-cab4acb5/`.

[Final acceptance](handoffs/2026-10-27-six-rid-validation-acceptance.md) ·
[Open Windows clipboard reliability issue](issues/0001-windows-clipboard-paste-reliability.md).
No further confirmation runs are required. The validator must continue failing
actual failed assertions. Retain metadata-only diagnostics and explicit missing-
cell failures without suppression or added retries.

**Parent accepted G3 compact-storage research and the under-25-MB objective.
G3 remains active and incomplete.**
The accepted correctness experiments are preserved. Byte accounting identifies
JSON/hex payloads, superseded trie paths and page slack as the main old costs;
only four distinct complete ledgers were stored, so ledger duplication was not
the explanation.

The same native 1K/10K G2 fixtures now occupy about 17.3–17.4 MB in a compact SQL
baseline or 18.3–18.4 MB with authenticated hash buckets at 10K, versus about 196 MB
before. No history was pruned. The under-25-MB target is met. Authentication data
is about 8% of compressed payload, but broader receipt/index metadata still exceeds
20%; both binary and compressed denominators are reported without cherry-picking.
Plain SQL cannot claim bounded authenticated absence: it returns Unknown or requires
full audit. Its timed ingestion uses an explicitly known-fresh fixture stream.

Final workloads and tests passed on Windows NTFS, Ubuntu ext4 and native Mac APFS:
117 test passes and three explicit unsupported-guarantee skips. An independent
ordinary WB5 export/reopen is 7,365 bytes at 1K and 7,367 bytes at 10K for the same
42-cell fixture, demonstrating separation from retained history. The new Rust
baseline example finishes with zero charge on every platform; production code is
unchanged.

[Byte accounting, comparison and evidence](handoffs/2026-10-27-g3-storage-efficiency-results.md) ·
[Compact representation and integrity boundary](architecture/history-compact-storage-experiment.md) ·
[Storage-efficiency assignment](handoffs/2026-10-27-g3-storage-efficiency-reset.md).
The owner identified `J:\coding\dotZstd` as an existing .NET zstd wrapper; Rust
zstd is another future option. This slice uses zlib 6 only as a research baseline;
no production codec or dependency is selected.

**Parent accepted the page-layout and authenticated-growth research.**
Ordinary rowid payload storage is the next experimental baseline and reduces the same
10K history to about 13.6 MB, preserving all logical rows and restored cold images.
Integer-local keys and a partial receipt index offer no material space benefit in
this workload. Atomic prefix splits retain every receipt, preserve old-reader
snapshots and refuse stale publication. A separate 16,384-control index-only probe
reaches 376–377 current buckets; it is not a full 100K G2 qualification.

Windows NTFS, Ubuntu ext4 and native Mac APFS completed the final research gates:
147 test passes and three explicit skips. Broader metadata still exceeds the 20%
target, and the bounded flat directory still has linear decode/rewrite cost.
The authorized hierarchical directory and receipt-normalization investigation is now
accepted, as reported at the top of this document. Its zlib-6
baseline is unchanged by the separate owner-authorized dictionary comparison.
[Acceptance, next assignment and stop conditions](handoffs/2026-10-27-g3-layout-growth-acceptance.md) ·
[Results, limits and native evidence](handoffs/2026-10-27-g3-layout-growth-results.md) ·
[Layout and split contract](architecture/history-layout-growth-experiment.md) ·
[Accepted assignment](handoffs/2026-10-27-g3-compact-storage-acceptance.md).
No full 100K run, production backend/index choice,
retention/GC, witness, coordinator, format migration or Undo/Redo integration.
G1/G2 and six-RID execution validation remain accepted/closed; clipboard ISSUE-0001
stays separate and G4–G7 remain gated.
The owner shelved Dev Drive changes; VM storage, build paths and Defender remain
unchanged. Earlier infrastructure entries below are historical where they describe
open gates or a G3 scheduling hold.

**Six-RID headless validation accepted and closed by parent review: 78/78 gates.**
Unified result: `artifacts/validation-matrix/20261026T002242Z-f76230c2/`.
All Windows/Ubuntu targets ran in their VMs, with native Mac ARM64 and Rosetta x64;
ROBPC remained controller-only. [Implementation and review handoff](handoffs/2026-10-25-runtime-shutdown-and-six-rid-validation.md).

Fixed task-ticket release ordering before lane completion; finish and original
assertions remain unchanged. Rust cancellation setup now uses a batch, and client
cancellation tests synchronize after dispatch instead of racing fast responses.
Original failures are preserved. The historical finish branch was not recorded.

An independent system-SDK CoreCLR abort on Linux x64 remains unexplained. Owner
approved private SDK10.0.403/runtime10.0.14 selection; it passed the final matrix.
The earlier failed matrix remains Failed. No system SDK replacement.

Mac headless capacity is two across both macOS targets and both UTM guests;
Hyper-V jobs are independent. Fourteen focused runner tests and six-target
discovery validate this policy (`20261026T004147Z-5b250c62`). The completed CI
matrix used its original global-two invocation; scheduler verification is separate.

Parent accepted the shutdown ordering correction, both cancellation test fixes,
private Linux x64 SDK selection and scheduling policy. No further headless
acceptance run is required. See [closure and GUI/AOT scope](handoffs/2026-10-25-six-rid-headless-closure.md).

Next authorized work is practical managed-GUI integration, then Native AOT actual
publish-and-execute through the existing profiles. Preserve original functional
assertions, real backends, four zero charges, owned-process cleanup and live GUI
consent. Native Mac ARM64/Rosetta GUI requires machine-wide mutual exclusion
across independent invocations. No new six-RID GUI/AOT claim yet.
G1/G2 closed; G3 held; G4–G7 and production Undo/Redo deferred.

Earlier acceptance/setup records follow. Their pending states are historical;
the current results above supersede them without changing original evidence.

Parent review accepted and closed both Windows VM SSH integration and native
headless validation. Five designated RIDs are accepted at 13/13 gates each; no
additional Windows connectivity qualification is required. See the
[five-RID acceptance and next scope](handoffs/2026-10-25-five-rid-headless-acceptance.md).

Next work: diagnose the existing Linux ARM64 runtime-finish failure with focused
native tests, preserving the original exit 101 and assertion. After a justified
correction and targeted verification, run Linux ARM64 full CI, then one unified
six-target headless matrix. GUI/Native AOT integration follows headless completion;
no new six-RID GUI/AOT coverage is claimed. G1/G2 closed; G3 held.

[Both Windows guests passed native headless CI, 13/13 each](handoffs/2026-10-25-windows-ssh-automation.md).
Five designated RIDs now have passing headless evidence; Linux ARM64 retains its
original Rust failure. No ROBPC/WSL diagnostic result is counted toward this matrix.

Both Windows guests now report OpenSSH Ready after private host-key ACL correction.
The installing-user ownership/access caused the SYSTEM-only protection failure;
keys and fingerprints were retained. Setup reports: `WINDEV2407EVAL-ssh-15e05693`
and `WIN-EO8VVET7M4Q-ssh-36528a4c`. Historical failed attempts remain unchanged.

Windows x64 SSH/key authentication works at User@192.168.5.118. Discovery verified
native x64 Rust, .NET SDK10.0.402 and NTFS. Source deployment quoting was corrected
for Windows PowerShell. Guest headless CI passed all 13 existing gates:
`artifacts/validation-matrix/20261025T233611Z-f3b72f89/`.

Windows ARM64 key login works at robhorvath@192.168.5.125 after bridging. The
Private-only firewall rule blocked its Public network; setup now covers all
profiles while retaining LocalSubnet scope. Discovery verified native ARM64
Python/Rust, SDK10.0.403, and local NTFS:
`artifacts/validation-matrix/20261025T234006Z-b7df87c2/`.
ARM64 full headless CI passed all 13 gates (`20261025T234038Z-222a8ed9`). Both Windows
profiles use SSH/native source staging; ROBPC is controller-only. No GUI run.
G3 remains held; the original Linux ARM64 Rust failure remains unresolved.

The following records describe earlier setup states; the accepted results above
supersede their pending/failure status. Original evidence remains unchanged.

Parent six-RID review accepted practical automation and corrected the completion
matrix: ROBPC is controller-only; its local Windows/WSL passes are diagnostic
baselines, not designated guest results. Both Windows RIDs require their VMs.
Default/private profiles now use SSH for Windows x64, with no local fallback.

Hyper-V Windows x64 SSH setup failed at service startup after installation and
configuration validation; original report:
`artifacts/validation-endpoints/WINDEV2407EVAL-ssh-0a5a54ac/`.
Configuration was restored; key/firewall/service installation changes remain.
x64 diagnostics report service exit 1067, System event 7034, and an OpenSSH
warning that BUILTIN Users can write the SSH directory. Incorrect permissions are
a documented possible cause, not yet confirmed here. ARM64 also failed startup
(`WIN-EO8VVET7M4Q-ssh-23f82551`). Installer now backs up directory ACLs and limits
SSH/log directory writes to SYSTEM/Administrators, retaining directory-only Users
read/execute. It collects service/event diagnostics before configuration rollback.
Syntax and in-memory ACL construction checked; corrected guest execution pending. Linux ARM64 runtime-finish diagnosis
remains authorized, preserving the original failed CI and its assertion. G3 held.

[Mac x64 Rust/tool orchestration](handoffs/2026-10-25-mac-x64-toolchain.md):
x64 Rust installed; ARM64 default retained. Native ARM64 Python controls verified
x64 Rust/.NET tools under Rosetta. All 13 existing headless CI gates passed on
APFS; the built Rust executable is confirmed Mach-O x86_64. The initial linker
setup failure is preserved. Eleven focused runner checks pass.

[Native Mac ARM64 CI](handoffs/2026-10-25-mac-ssh-automation.md) also passed all 13
gates. SSH deployment and automatic report retrieval work with Remote Login Full
Disk Access off. No x64 Python installation is needed.

[Ubuntu VM SSH automation](handoffs/2026-10-25-ubuntu-ssh-automation.md): Hyper-V
x64 passed all 13 gates; UTM ARM64 reached an actual Rust cancellation-test failure
at runtime.finish. Its Failed result remains preserved without a retry or product
change. Local Windows x64 and WSL x64 full headless CI passed as development diagnostics.
Windows guest SSH setup is underway as recorded above. No GUI/AOT acceptance is claimed for these new runs. G3 remains held.

**Current direction: [validation infrastructure reset](handoffs/2026-10-25-validation-infrastructure-reset.md).**
Owner/parent supersede the forensic qualification and per-run approval workflow.
Build practical six-RID automation using unchanged existing test assertions,
ordinary revisions/logs, trusted development VMs and explicit GUI consent. Unrelated
/proc churn is diagnostic; actual assertion/resource/owned-process failures still
fail. Preserve all historical Blocked reports below without reclassification.
[Unified runner](validation.md) is implemented and its first practical execution
completed. Earlier one-attempt/packet/forensic requirements below are historical,
not current development policy. G3 remains held; no product changes.

[First practical six-RID execution report](handoffs/2026-10-25-six-rid-first-practical-run.md):
Windows x64 and Linux x64/WSL each pass all 13 existing headless CI gates through
the new orchestrator; seven focused tests pass. Four remote ARM64/macOS targets
are explicitly NotConfigured. No new GUI/AOT run. Remote endpoints/authentication,
Mac x64 execution and actual remote trials remain setup work; no SSH service was
enabled.

[Third Ubuntu attempt overall Blocked/2](handoffs/2026-10-25-ubuntu-third-managed-result.md).
Runner exit0, direct main/peer/child identities, four managed X11/zero-charge reports,
WB5/source/context and final empty scope verify. Diagnostics retain 121 unresolved
observations: 108 before binding, 13 after; 120 ENOENT and one main environ denial.
Eighty complete records/three exit events do not erase errors. Thirty capture/28
overlapping nested hashes and transfer digests verify. Third authority consumed;
no retry/code repair. Parent contract/lifecycle direction next; no guest action.
Earlier third-run-pending notes superseded. Qualification open; G3 held/unstarted.

[Third, lifecycle-corrected Ubuntu confirmation prepared](handoffs/2026-10-25-ubuntu-lifecycle-attended-confirmation.md).
Final exact-packet/delta reconciliation passes. New short shared-script command
uses fresh ext4 input and third distinct exclusive receipt, preserving both previous
attempts. Fresh live-context/owner CONFIRM/separate GUI consent remain mandatory.
Third run pending; no GUI/build ran in preparation. Stop after first outcome; no
retry. Overall GUI open; G3 held/unstarted.

[Parent accepts lifecycle correction](handoffs/2026-10-25-ubuntu-lifecycle-correction-acceptance.md).
Non-GUI gate closed; one new owner-attended Ubuntu managed-GUI confirmation is
authorized after final exact-packet/live-context verification. Prepare a third,
distinct attempt receipt and short shared script; fresh CONFIRM/separate GUI consent
required. No third run has occurred. Preserve both prior Blocked outcomes; no replay.
Overall GUI open; Ubuntu AOT/unattended unapproved; G3 held/unstarted. Earlier
pending-correction-review notes below are superseded.

[Ubuntu lifecycle correction ready for review](handoffs/2026-10-25-ubuntu-lifecycle-correction.md).
Early-pinned outer lifecycle covers selection/cwd/initial/full recheck windows;
precise stage/operation/identity/errno diagnostics preserve unresolved failures.
New immutable packet: 546 sources/273 inputs/ten payloads, observer/doc changes
and one added test file only. All 48 frozen Linux checks pass; Windows 45 passes/
three Linux-only skips. Real non-GUI Python-child pidfd fixture exercised on WSL.
Both old Blocked runs preserved; no GUI/build/new positive command. Parent review
next; no guest action requested. G3 held/unstarted.

[Parent accepts second-run direct identities/application evidence; overall Blocked retained](handoffs/2026-10-25-ubuntu-confirmation-lifecycle-review.md).
Next authorized scope: bounded non-GUI observer lifecycle/read-order investigation,
precise failure-stage diagnostics and potential correction with focused tests/new
immutable packet. Both Ubuntu GUI authorizations consumed; no new positive command
or execution. Preserve four original disappearance errors and both Blocked outcomes.
Earlier confirmation-review-pending notes superseded. G3 held/unstarted.

[Ubuntu corrected confirmation: overall Blocked/2](handoffs/2026-10-25-ubuntu-observer-confirmation-result.md).
Original runner exit0 and full runtime identity correlation now verify, including
direct main/three peers/canonical DLL/cwd and matching Rust children. Four X11/
zero-charge reports, WB5/source/context integrity and empty final scope pass.
Four unresolved observation disappearances remain; 90 records/three corroborated
exit events do not erase them. Thirty capture/28 overlapping nested hashes verify.
New authority consumed; no retry or code repair. Parent direction next; earlier
confirmation-pending notes superseded. Qualification open; G3 held/unstarted.

[New Ubuntu observer-corrected confirmation prepared](handoffs/2026-10-25-ubuntu-observer-attended-confirmation.md).
Final read-only candidate integrity/delta checks pass. New short shared script uses
fresh native ext4 import and a distinct exclusive attempt receipt; first attempt
and evidence remain untouched. Fresh current-context, owner CONFIRM and separate
GUI consent remain required. Confirmation pending; no GUI/build ran in preparation.
Stop after first outcome; no replay. G3 held/unstarted.

[Parent accepts observer correction](handoffs/2026-10-25-ubuntu-observer-correction-acceptance.md).
Non-GUI correction gate is closed. One newly authorized attended Ubuntu managed-GUI
confirmation may be prepared after final read-only packet reconciliation, with a
distinct one-attempt receipt and fresh current-context/owner/GUI consent checks.
No confirmation has run. First Blocked result and its consumed authority remain
separate. Overall GUI qualification open; Ubuntu AOT/unattended unapproved;
G3 held/unstarted. Earlier correction-review-pending notes below are superseded.

[Ubuntu observer correction ready for parent review](handoffs/2026-10-25-ubuntu-observer-cwd-correction.md).
Actual target-cwd/canonical DLL resolution and bracketed pinned process observation
address both observer failure modes; build actors retain direct identity/ancestry/
cgroup and wrapper context without unnecessary display-environment reads. New
immutable candidate: 545 sources/272 inputs/ten payloads; 37 frozen Linux checks
pass, Windows 35 pass/two Linux-only skips. Exact observer/test/doc delta verifies.
No GUI/build/new positive command or guest retry. Original Blocked/2 preserved;
qualification open, further execution requires parent approval. G3 held/unstarted.

[Parent accepts Ubuntu actual application evidence, retains Blocked/2](handoffs/2026-10-25-ubuntu-first-managed-review.md).
Functional/backend/zero-charge/WB5/context/final-snapshot evidence is accepted within
scope; full qualification remains open. Next authorized work is narrowly scoped
non-GUI observer investigation/correction for relative main DLL/cwd identification
and compiler environ-read failure, with focused tests and a new immutable candidate
for review. No GUI replay/new positive command; original authority consumed.
Earlier pending first-run-review notes are superseded. G3 held/unstarted.

[First Ubuntu attended GUI attempt: overall Blocked/2](handoffs/2026-10-25-ubuntu-first-managed-result.md).
Original UI runner exited 0; four actual managed X11 backend reports/zero charges,
WB5/source integrity, continuity and final empty scoped snapshot verify. Observer
retains one Rust environ permission failure and missing main direct identity;
these prevent qualification. Thirty capture/28 overlapping nested hashes and
separate identical report export verify. Authorization consumed; no retry or code
change. Parent direction next; earlier execution-pending notes superseded.
Ubuntu GUI remains open; AOT/unattended unapproved; G3 held/unstarted.

[One Ubuntu attended managed-GUI attempt prepared](handoffs/2026-10-25-ubuntu-attended-managed-attempt.md).
Final read-only accepted packet/delta verification passes. Short shared-script
command performs pinned native ext4 import and exclusive one-attempt reservation,
then the fixed launcher with fresh CONFIRM and separate original GUI consent.
Execution pending; no GUI/build occurred during command preparation. Stop after
first outcome and return evidence; no retry. G3 held/unstarted.

[Parent accepts Ubuntu enabled candidate](handoffs/2026-10-25-ubuntu-enabled-candidate-acceptance.md).
Preparation gate is closed. One owner-attended managed-GUI attempt is authorized
after final read-only packet/current-context checks, fresh owner CONFIRM and separate
original live GUI consent. Command preparation is authorized; no positive attempt
has run. Stop after first outcome, with copy-only transfer recovery and no replay.
Earlier pending-candidate-review notes below are superseded. Actual GUI acceptance
remains open; Ubuntu AOT/unattended not approved; G3 held/unstarted.

[Ubuntu enabled candidate ready for parent review](handoffs/2026-10-25-ubuntu-enabled-candidate.md).
Exact one-line positive-flag delta independently verifies against accepted disabled
snapshot: same 544 sources/271 inputs/ten payloads. Twenty-six frozen non-GUI checks
pass on Windows and Linux/WSL. Working-tree gate stays false; no positive command,
GUI/build or new guest run. Fixed one-run command design is proposed, not issued.
Separate parent execution approval remains required; G3 held/unstarted.

[Parent accepts Ubuntu disabled operational implementation](handoffs/2026-10-25-ubuntu-disabled-implementation-acceptance.md).
Implementation/observer correction and actual guest non-GUI validation are closed;
no repeat needed. Next authorized scope is one immutable enabled candidate with
only reviewed flag/necessary launch-packaging delta and non-GUI refusal checks.
Working-tree positive path stays disabled. No positive Ubuntu command or GUI/AOT
execution before separate review. Earlier pending implementation-review notes below
are superseded. Windows/Ubuntu headless closed; G3 held and unstarted.

[Ubuntu disabled implementation/native results ready for parent review](handoffs/2026-10-25-ubuntu-implementation-review-submission.md).
Corrected actual guest non-GUI adapters verify; 24 frozen tests pass, seven origin
hashes and transfer digests match, source unchanged, context verified before/after.
Snapshot includes checker, not post-GUI survivors; three preexisting nonactor
permission denials remain explicit. Original Blocked result is preserved. Final
disabled correction packet: 544 sources/271 inputs/ten payloads. No Ubuntu GUI,
AOT/build or positive command. Earlier guest-results-pending notes are superseded.
No further guest run requested; parent implementation review next. G3 held.

[Ubuntu observer scope correction](handoffs/2026-10-25-ubuntu-observer-scope-correction.md):
first actual adapter run remains Blocked on unrelated preexisting service executable
permissions and checker ancestry beyond its own root. Guest tests/context/tools
passed; original hashes verify. A new immutable disabled correction packet and
24 frozen tests/21 WSL runtime tests pass; corrected Hyper-V adapter evidence is
pending. No GUI/build/elevation or isolation change. Earlier pending first-run
notes below retain historical context. Positive gate false; G3 held.

[Ubuntu disabled operational implementation](handoffs/2026-10-25-ubuntu-disabled-implementation.md)
is frozen: live Linux preflight, scoped process identity/cleanup, fixed original
runner, terminal recording and native export. Positive flag stays false. Final
packet has 544 sources/271 inputs/ten payloads; 22 frozen non-GUI host tests and
19 runtime tests in WSL pass. Actual Hyper-V non-GUI adapter results are pending;
short shared-script command is provided. No positive GUI/AOT command or execution.
Earlier prototype-only implementation notes below are superseded; G3 held.

[Parent accepts Ubuntu session evidence and disabled contract](handoffs/2026-10-25-ubuntu-disabled-contract-acceptance.md).
That preparation gate is closed. Next authorized scope: fixed Linux managed-GUI
wrapper, live preflight, process observation, native export and non-GUI validation,
then a new immutable disabled implementation packet for review. Positive Ubuntu
GUI/AOT remains unauthorized; no positive command. Preserve terminal/logind and
owner-attested VM-binding distinctions. Earlier pending review notes below are
superseded. Windows gates/Ubuntu headless closed; G3 held.

[Ubuntu session evidence and disabled proposal](handoffs/2026-10-25-ubuntu-session-and-disabled-proposal.md)
are ready for parent review. Actual user display session 2/seat0 correlate; terminal
itself remains outside a known logind session. Native ext4 reports and transfer
hashes verify. DMI UUID/serial permission denial and unproven Avalonia backend remain
explicit. Disabled contract/proposal packet has 540 sources/267 inputs/six files;
three frozen contract tests and two real refusal invocations pass on the host.
Live Linux runner supervision/export remain proposed, not execution-ready.
No Ubuntu GUI/AOT/build ran; no positive command. Earlier collection-pending notes
below are superseded. Windows/Ubuntu headless accepted gates unchanged; G3 held.

[Ubuntu x64 read-only session preparation](handoffs/2026-10-25-ubuntu-gui-session-preparation.md)
has a frozen collector and short shared-script launch command. Three pure non-GUI
checks pass. Actual guest session/seat results are pending; fixed disabled GUI
runner and review packet will follow their review. No Ubuntu GUI/AOT/build ran.
Accepted Windows and Ubuntu headless gates remain closed; G3 held.

[Parent accepts and closes Windows x64 Native AOT](handoffs/2026-10-25-windows-native-aot-acceptance.md).
Windows x64 headless, attended managed GUI and attended Native AOT gates are all
accepted/closed. Preserve unresolved nonblocking publish diagnostics; no rerun.
Next authorized scope is Ubuntu x64 non-GUI session/seat inspection, fixed managed
runner design, refusal tests and an immutable disabled packet for review. No Ubuntu
GUI/AOT activation is authorized. Long Ubuntu commands must be shared-storage
scripts with short launch instructions. Earlier pending acceptance notes below
are superseded. G3 remains held.

[Windows x64 Native AOT attempt completed Passed/0](handoffs/2026-10-25-windows-native-aot-result.md),
independently reconciled and ready for final parent acceptance. Real warning-free
publish, native main/three peers, four backend/session/zero-charge identities,
WB5/source integrity, boot continuity and empty strict survivor snapshot verify.
Fifty outer/44 overlapping nested hashes and transfer digests match. Publish log
retains unexplained command-line-length/syntax diagnostics despite exit 0 and
successful native generation/execution; handoff flags these explicitly.
One-attempt authorization consumed; no retry. Earlier execution-pending notes below
are superseded. Windows managed/headless closed; G3 held.

[One attended Windows Native AOT attempt](handoffs/2026-10-25-windows-aot-attended-attempt.md)
is prepared. Final read-only reconciliation passes against the exact accepted
packet; the fixed owner command retains current-condition CONFIRM, original
separate GUI consent and a local accidental-replay guard. Execution is pending;
no native publish or GUI ran during preparation. Stop after the first outcome.

[Parent accepts Windows AOT prerequisites and enabled packet](handoffs/2026-10-25-windows-aot-candidate-acceptance.md).
Both preparation gates are closed. One owner-attended Pivot-Win11-x64 Native AOT
publish-and-execute attempt is authorized after final read-only packet/current
guest verification, owner CONFIRM and separate original GUI consent. No positive
attempt has run. Stop after the first outcome and return evidence for final review;
no automatic retry. Earlier pending-review statements below are superseded.
Windows managed/headless gates remain closed; G3 held.

[Windows AOT guest prerequisites and enabled candidate](handoffs/2026-10-25-windows-aot-guest-prerequisites-and-candidate.md)
are ready for parent review. Actual Windows guest prerequisites, source/context
and returned hashes verify. Candidate preserves 537 sources/265 inputs/15 files
with only the reviewed AOT false-to-true source change; eleven non-GUI checks pass.
No publish or GUI execution occurred. Positive AOT command remains withheld pending
separate parent review. Earlier collector-pending notes below are superseded.
G3 remains held.

[Windows guest AOT prerequisite collector](handoffs/2026-10-25-windows-aot-prerequisite-collection.md)
is frozen and ready for one owner-run read-only collection. Three mocked non-GUI
tests pass; actual guest qualification is pending. No publish or GUI ran. After
guest evidence, prepare the enabled candidate for separate parent review; no
positive AOT command is authorized. G3 remains held.

Latest infrastructure result: **scoped-opt-out Windows managed confirmation Passed/0.
Existing UI runner exited 0; main/three peers passed unchanged assertions with
zero charges, session 1 and independently captured Avalonia.Win32.WindowImpl.
Strict scoped survivor snapshot is empty.** Boot/context continuity and source
integrity verify, as do 31 outer hashes/25 overlapping nested entries and transfer
digests. No exclusion, extra cleanup wait, kill or automatic retry. **Parent accepts
and closes Windows Hyper-V x64 owner-attended managed-GUI qualification.**
[Final acceptance and bounded Native AOT preparation scope](handoffs/2026-10-25-windows-managed-final-acceptance.md).
Next authorized work is Windows Native AOT design/non-GUI preparation and a frozen
disabled packet for review; no positive AOT run or further managed rerun is approved.
Unattended authority remains disabled; G3 held. Historical pending/open statements
below retain their original stage context and are superseded by this closure.
[Windows Native AOT design and disabled preparation packet](handoffs/2026-10-25-windows-native-aot-preparation.md)
are ready for review. Nine frozen non-GUI checks and a real disabled-positive
refusal pass; host tool metadata verifies without qualifying the guest. Candidate
has 537 sources/265 inputs/15 files, all prior source unchanged and AOT flag false.
No native publish, AOT GUI, managed rerun or full CI repeat occurred.
[Parent accepts/closes disabled AOT preparation](handoffs/2026-10-25-windows-aot-preparation-acceptance.md).
Next authorized work is one actual guest prerequisite collector and, after its
results, an enabled immutable candidate with exact delta/negative checks. Positive
publish/execution still requires separate review; no positive command authorized.
[Successful confirmation and final review packet](handoffs/2026-10-25-windows-managed-confirmation-pass.md).
The prior collector-blocked run is preserved separately:
Parent accepts functional/backend/continuity evidence; overall Blocked/2 is retained.
[Collector investigation and conditional wrapper-only correction authorized](handoffs/2026-10-25-managed-cleanup-investigation-direction.md).
Next is a non-GUI guest build with scoped PID/start/lifetime observations and
opt-out evaluation; no new managed confirmation is authorized.
[Scoped non-GUI collector build experiment](handoffs/2026-10-25-avalonia-collector-build-experiment.md)
is prepared with five passing plumbing checks and immutable input verification.
Actual guest build/process evidence is pending; wrapper/environment policy remains
unchanged outside the experiment. No GUI run or clipboard check is requested.
[Actual guest build and private wrapper opt-out correction](handoffs/2026-10-25-scoped-avalonia-optout-correction.md)
are ready for parent review. Guest build is warning-free; 62 process records show
no collector through a 15.814-second post-build interval, with native/transfer
hashes verified. Scoped correction and caller-environment restoration tests pass;
533-source/262-input candidate passes 22 applicable frozen checks. Strict cleanup
and prior Blocked outcomes remain unchanged; no new GUI run is authorized.
[Parent closes scoped correction gate and authorizes one new attended confirmation](handoffs/2026-10-25-scoped-optout-acceptance.md).
Final candidate reconciliation precedes that run; not yet executed. Overall managed
acceptance remains open for unchanged strict cleanup evidence; Native AOT still gated.
[Final scoped-opt-out verification and one-run instructions](handoffs/2026-10-25-scoped-optout-confirmation-instructions.md)
are ready. Twelve files/533 sources/262 inputs and the exact reviewed delta verify
without test replay. Current guest conditions and both separate owner responses
remain runtime requirements; the newly authorized run has not executed yet.
That new authorization is now consumed by the Passed confirmation recorded at the
top of this status. No further execution is authorized pending parent acceptance.
[Latest confirmation and survivor evidence](handoffs/2026-10-25-boot-corrected-managed-confirmation-results.md).
The earlier diagnostic-blocked run is retained separately:
[First managed result and evidence](handoffs/2026-10-25-windows-managed-first-run.md).
One-run authorization is consumed. Native AOT stays gated, unattended authority
disabled and G3 held. The following links retain the qualification/approval history.
[Parent accepted functional execution; narrow diagnostic correction authorized](handoffs/2026-10-25-windows-managed-functional-acceptance.md).
Overall managed acceptance remains Blocked/2. Next authorized work is live peer
backend capture, non-GUI regression/build validation and a new immutable packet;
another positive GUI run requires separate review and approval.
[Peer backend correction and immutable confirmation candidate](handoffs/2026-10-25-peer-backend-correction.md)
are ready for review: one live capture plus existing-handler serialization, actual
non-GUI C# lifecycle regression, thirteen preflight tests and warning-free build.
Candidate hashes verify 530 source files/259 inputs with only the production
diagnostic change and added test. No positive confirmation execution authorized.
[Parent correction acceptance and one new attended confirmation authorization](handoffs/2026-10-25-peer-backend-correction-acceptance.md).
Correction gate is closed. One managed confirmation is now authorized after final
candidate verification under unchanged guest conditions; not yet executed. First
run remains Blocked/2 and Native AOT remains separately gated.
[Final packet verification and one-run confirmation command](handoffs/2026-10-25-windows-managed-confirmation-instructions.md)
are ready. Exact candidate hashes/delta rechecked without replaying tests; actual
guest binding is still checked at invocation. Confirmation not yet executed.
The first confirmation attempt is now Blocked/2 before GUI startup: the exact
boot timestamp string differs by 1.179 ms while session 1 and WinSta0/Default are
unchanged. This does not establish a reboot. No retry; the overly precise boot
binding needs review. Owner-reported blank screens remain undiagnosed.
[Confirmation timestamp refusal and evidence](handoffs/2026-10-25-windows-confirmation-timestamp-block.md).
[Parent accepts refusal; bounded read-only boot-identity investigation authorized](handoffs/2026-10-25-boot-identity-investigation-direction.md).
Investigate documented stable boot measurements and supported comparison semantics;
no deliberate reboot/settings change or new GUI attempt. Blank-screen reliability
is separate and unresolved. Return contract/correction evidence for review.
[Boot-identity research and read-only guest collection](handoffs/2026-10-25-windows-boot-identity-research.md)
record documented event/uptime candidates and six host samples. Guest measurements
are pending; no comparison tolerance or guard correction is selected/implemented.
No new GUI command, reboot, integration change or clipboard recheck.
[Guest boot evidence and offline continuity proposal](handoffs/2026-10-25-windows-boot-continuity-proposal.md)
now verify six guest samples with a stable provider-bound startup-event anchor and
increasing uptime. Seven offline measured/synthetic tests pass for a corroborated
comparison proposal; budgets require review. Production guard is unchanged and
no new packet or positive execution is authorized.
[Parent accepts research and authorizes bounded guard integration](handoffs/2026-10-25-boot-continuity-implementation-direction.md).
The 100 ms CIM and two-second elapsed-difference budgets are approved as initial
Pivot policy, with distinct query-local and cross-observation checks. Next is
real-adapter non-GUI implementation/validation and a new candidate for review;
no further positive execution is authorized.
[Boot guard implementation, immutable candidate and read-only adapter check](handoffs/2026-10-25-boot-continuity-guard-implementation.md)
are prepared. Eight new comparator tests, thirteen preflight tests, lifecycle
regression and warning-free build pass; 21 applicable frozen tests verify. Host
adapters behave as expected; actual guest production-adapter results are pending.
No new GUI command or positive execution permission.
[Actual production-adapter guest results](handoffs/2026-10-25-boot-guard-guest-adapter-results.md)
complete this implementation packet for parent review: all six samples/24
comparisons verify, independently recomputed; five helper hashes and native/
transfer digests match. Session 1/WinSta0/Default and reviewed event anchor hold.
Max query/cross discrepancies are about 21.6/16.9 ms, within policy. No GUI retry.
[Parent closes boot-continuity gate and authorizes one new attended managed confirmation](handoffs/2026-10-25-boot-continuity-guard-acceptance.md).
Final read-only candidate/delta verification precedes the run; not yet executed.
Native AOT remains gated and prior Blocked outcomes remain original.
[Final integrity verification and boot-corrected one-run instructions](handoffs/2026-10-25-boot-guard-managed-confirmation-instructions.md)
are ready. Eleven packet files/533 sources/262 inputs and exact reviewed delta
verify without rerunning tests. The new attended confirmation has not yet executed;
current guest continuity and separate owner responses remain runtime requirements.
That authorization has now been used; the result is the collector-blocked
confirmation recorded at the top of this status. No additional run is authorized.

Infrastructure workstream: **optional isolated-validation architecture approved;
first execution-safety gate accepted/closed; bounded guest qualification in progress.
Both Hyper-V x64 guests are HeadlessQualified following parent acceptance of the
73-hash/native-probe packet. Full Pivot headless CI passed in both qualified x64
guests: 13 steps each, Rust 347/348 and nine safety tests each. Review verified
692 returned hashes, 508 source files/253 inputs and twenty zero-charge fields.
Full Hyper-V x64 guest headless CI is accepted/closed for both targets. Individual
GUI-isolation qualification has partial session evidence for both guests and
owner-confirmed VMConnect Basic Session and no other owner-reported connections.
Ubuntu correlation was attempted with verified transfer hashes; logind session/seat
remain Unknown. Owner reports no additional resource exposure and successful
bidirectional clipboard separation for both guests in the consented manual window.
Parent accepted observed isolation for both guests as sufficient for bounded
activation design, with identity/timing limitations retained. Operational unattended
GUI authority remains unapproved and disabled**.
Contributor headless CI remains VM-independent. Local desktop/AOT acceptance needs
interactive consent; direct self-checks need a live consent permit before Avalonia
startup. Unattended isolation remains NotConfigured pending host/guest/sentinel
verification. No VM dispatch, provisioning or full Pivot acceptance on the new guests.
[Guest qualification workflow, actual blockers and three separate states](handoffs/2026-10-24-guest-qualification.md).
[Collector preparation acceptance and shared-drive layout](handoffs/2026-10-24-guest-collector-preparation-acceptance.md).
Guest J: (Windows SMB) and /mnt/j (Linux CIFS) map the host J: root for deliberate
source/evidence exchange only. Builds/probes/original evidence require guest-local
NTFS/Linux storage; share presence is not GUI isolation proof.
Installed, headless-qualified and GUI-isolation-approved are independent; no guest
is GUI-approved. Both Hyper-V x64 guests are now headless-qualified; UTM ARM64
qualification remains pending.
[Headless qualification acceptance and next CI boundary](handoffs/2026-10-24-x64-headless-qualification-acceptance.md).
[Frozen guest-local headless CI instructions](handoffs/2026-10-24-hyperv-headless-ci-instructions.md).
[Actual Hyper-V guest CI evidence](handoffs/2026-10-24-hyperv-headless-ci-validation.md).
[Guest CI acceptance and separate GUI-isolation boundary](handoffs/2026-10-24-hyperv-headless-ci-acceptance.md).
[Per-guest GUI-isolation inspection instructions](handoffs/2026-10-24-hyperv-gui-isolation-qualification.md).
[First returned GUI-session metadata review](handoffs/2026-10-24-hyperv-gui-session-review.md).
[Parent acceptance of preliminary session evidence](handoffs/2026-10-24-hyperv-gui-session-acceptance.md).
[Owner clipboard results and per-guest qualification review packet](handoffs/2026-10-24-hyperv-gui-isolation-results.md).
[Observed isolation acceptance and gated activation-design scope](handoffs/2026-10-25-hyperv-isolation-acceptance.md).
[Parent scope reduction: Windows attended preflight](handoffs/2026-10-25-attended-guest-preflight-direction.md)
authorizes bounded implementation and non-GUI tests using existing interactive
consent, then parent review before positive GUI execution. The second capability
protocol/revocation design is deferred; no unattended authority is enabled.
[Windows attended preflight implementation](handoffs/2026-10-25-windows-attended-preflight-implementation.md)
is accepted/closed with non-GUI tests; the original preflight packet stays disabled.
[Acceptance and next packet/actual-guest preflight gate](handoffs/2026-10-25-windows-preflight-acceptance.md).
Frozen packet preparation and non-GUI preflight inside the actual Windows VM are
authorized; current guest evidence and packet review precede any positive-path enabling.
[Frozen Windows non-GUI preflight packet and owner command](handoffs/2026-10-25-windows-guest-preflight-packet.md).
Actual Windows guest preflight passed; its native report was recovered and
hash-verified without a rerun after a bootstrap parser defect left the first
export Blocked. Original outcome/packet are preserved. Current boot/session differ
from the historical clipboard capture; owner CONFIRM and that limitation are
disclosed for review. This original preflight gate performed no GUI execution.
[Guest preflight results and reconciliation](handoffs/2026-10-25-windows-guest-preflight-results.md).
[Actual guest preflight accepted/closed; managed packet preparation authorized](handoffs/2026-10-25-windows-guest-preflight-acceptance.md).
The subsequent corrected-exporter packet and negative tests are complete; the fresh
owner-authorized Windows clipboard observation was accepted before the first run.
[Managed packet preparation and proposed enabling diff](handoffs/2026-10-25-windows-managed-packet-preparation.md)
now record the fresh Windows owner-reported clipboard pass and unchanged settings,
disabled review path, corrected exporter and non-GUI validation. Preparation was
accepted; that review packet remains unchanged and grants no unattended authority.
[Parent acceptance closes preparation and conditionally authorizes one attended managed run](handoffs/2026-10-25-windows-managed-preparation-acceptance.md).
The separate enabled source snapshot passed exact hash/delta and negative checks,
then the owner executed the single managed run. Disabled packet remains unchanged;
Native AOT remains separately gated.
[Verified enabled packet and one-run owner instructions](handoffs/2026-10-25-windows-managed-execution-packet.md).
The new snapshot differs from the accepted disabled packet only by the reviewed
approval toggle. Its 529 source/258 input hashes, twelve applicable non-GUI tests
and two real noninteractive refusals verify. The authorized first run is now
recorded above with successful UI assertions and an overall diagnostic Blocked result.
[First x64 guest evidence review and follow-up](handoffs/2026-10-24-x64-guest-evidence-review.md)
retains the Windows Blocked candidate and .NET first-run certificate side effect;
full guest headless CI now passes as linked above; no GUI acceptance is claimed.
[Contributor testing](testing.md), [policy and transitive audit](architecture/validation-execution-safety.md).
[Safety acceptance and next qualification boundary](handoffs/2026-10-24-execution-safety-acceptance.md).
[Implementation and safety evidence](handoffs/2026-10-24-execution-safety.md).
[Mac safety reconciliation](handoffs/2026-10-24-macos-execution-safety-validation.md)
verifies 366 returned hashes and 247 inputs; shared documentation drift matches the
recorded G2 acceptance update, with no tested code changes. All three platforms pass
13 headless steps and nine safety tests each; no guest/GUI acceptance is claimed.
Owner-provisioned guests: Hyper-V `Pivot-Win11-x64` / `Pivot-Ubuntu-x64`;
UTM `Pivot-Win11-arm64` / `Pivot-Ubuntu-arm64`. Names are inventory, not validated
isolation or native architecture evidence. History G2 is accepted/closed; G3 is held, not started, during this infrastructure detour.

Latest accepted milestone: **ADR-0023 G2 mutation-local authoritative records â€” accepted and closed (2026-10-24)**.
[G2 acceptance and G3 scheduling hold](handoffs/2026-10-24-g2-acceptance-g3-held.md).
G1 ordinary native Save promotion remains accepted/closed.
[G1 closure and bounded G2 authorization](handoffs/2026-10-24-g1-closure-g2-authorization.md).

Previous accepted application milestone: **Four-Direction Rectangular Fill; ADR-0022 closed (2026-10-23)**.
[Final implementation acceptance](handoffs/2026-10-23-four-direction-fill-final-review.md),
[implementation and three-platform evidence](handoffs/2026-10-23-four-direction-fill-implementation.md).
Four-direction source-edge Fill, typed identity/no-op semantics, atomicity and the
focused staged cycle validator are accepted. Windows 346/362 and Linux/Mac 347/363
Rust tests pass, with managed/Native AOT execution, twelve fields and zero-charge
main/peers. Mac 419 source hashes, 211 inputs and 465 APFS-original evidence hashes
were independently verified, including manifest/receipt checksums. ADR-0021 remains
accepted/closed; its earlier evidence limitations remain historical.

**Open reliability issue:** intermittent Windows ClipboardChecklistAsync null
destination after paste. The unchanged rerun passed; the cause remains unresolved.
[Failure evidence and recurrence diagnostics](handoffs/2026-10-22-worksheet-structured-binding-implementation-review.md#open-reliability-issue--windows-clipboard-checklist).
[G1 promotion attempt history](handoffs/2026-10-24-historical-save-g1-promotion.md)
retains a Linux managed Fill selection/scroll failure and a Windows Native AOT
unsupported-copy clipboard assertion failure. Final unchanged-source runs passed;
causes remain unresolved and distinct from the older null-destination issue.
G2 also retains a separate Linux Rust clipboard-test shutdown assertion failure:
`runtime.finish().is_ok()`. An unchanged-source targeted rerun and final hardened-
source full gates passed; the cause is unresolved. [G2 evidence/qualification](handoffs/2026-10-24-mutation-local-records-g2.md).
Do not weaken tests or redesign the clipboard speculatively. Existing WSLg startup,
mounted-filesystem Save uncertainty and historical Linux timing issues remain separate.

History workstream: **ADR-0023 feasibility accepted/closed; G0 approved; G1 and G2 accepted/closed. G3 bounded indexed-store feasibility is active; its first baseline is accepted and its bounded verified-open 1K/10K result and failure-admission correction are accepted; compact-storage research is accepted; page-layout/metadata and authenticated-growth research is accepted; hierarchical authentication/receipt normalization and owner-authorized dictionary research are accepted; physical accounting and scalable admission proposal are next, with the 20% aspiration unproven and admitted resource ceilings unresolved; no complete feasibility or production adoption approval. G4â€“G7 and production Undo/Redo remain gated**.
[G2 implementation, ownership, coverage and measurements](handoffs/2026-10-24-mutation-local-records-g2.md).
[Native Mac G2 validation instructions](handoffs/2026-10-24-macos-mutation-local-records-g2-instructions.md).
G2 final Rust default/all-feature totals: **Windows 347/401, native Linux and Mac 348/402 each**.
Format and strict default/all-feature lint pass. **135 fresh-process zero-charge
samples** measure staging, guarded in-memory publication and the independent full
oracle separately. The **173-input** G2 source is validated on all three platforms;
Mac reconciliation verifies 483 frozen source entries and 85 returned evidence hashes.
[Fresh native Mac G2 reconciliation](handoffs/2026-10-24-macos-mutation-local-records-g2-validation.md). No default mutation or persisted journal integration.
Existing-input/scalar changes emit three records independent of sampled root size;
range-formula validation and the existing large range/ledger reservations remain
explicit costs. G2 includes deterministic original-state coalescing, source/ledger
validation, local reservation publication, full-checkpoint reconstruction oracles,
failed/no-op/stale/cancellation/admission isolation and concurrent capture proofs.
[G1 acceptance, unresolved reliability observations and exact G2 scope](handoffs/2026-10-24-g1-closure-g2-authorization.md).
[Ordinary Save promotion and final validation packet](handoffs/2026-10-24-historical-save-g1-promotion.md).
[Final Mac reconciliation and G1 closure recommendation](handoffs/2026-10-24-macos-g1-promotion-validation.md).
[Internal acceptance and ordinary Save promotion authorization](handoffs/2026-10-24-historical-save-g1-internal-review.md).
[Accepted internal G1 implementation](handoffs/2026-10-24-historical-native-save-g1.md).
[macOS/APFS validation and independent evidence review](handoffs/2026-10-24-macos-historical-save-g1-validation.md).
[G0 acceptance, G1 authorization and promotion requirements](handoffs/2026-10-24-history-readiness-g1-authorization.md).
Accepted internal-correction evidence: Windows default/all-feature Rust tests **346/389**; native Linux and Mac: **347/390**
each. Strict lint/format pass on all three. Release matrices: **207 cases and 2,652
original hashes** in total. Mac verification checks 1,097 returned evidence hashes,
168 inputs and 471 copied source files (470 frozen plus the ignored GPT baseline).
The parent accepts the internal correction and its three-platform evidence.
Ordinary Save now uses the validated projection directly in default builds.
Promotion full CI, managed/session/IPC, actual managed desktop and Native AOT
execution pass on all three platforms. Rust totals: Windows **347/390**, Linux/Mac
**348/391** each (default/all-feature). Strict lint and **207 historical native
cases / 2,652 original hashes** pass. Fresh Mac evidence verifies **236 build inputs**,
474 frozen source files plus the ignored baseline, 1,522 transferred evidence hashes
and 884 retained native-file hashes. All twelve desktop fields and 24 managed/AOT
main/peer zero-charge reports pass across the three platforms. The parent accepts
ordinary Save promotion and closes G1. That review authorized G2 alone; the later
G2 closure and held G3 authorization are recorded above.
The ordinary path uses one validated export contract without fallback, format/protocol
changes or capability expansion. Generic full-ledger transfer remains strict. Preserve live Expected,
ContentStateId, lifetime ledger, runtime IDs, Redo/cursor and calculation captures.
Ordinary Save promotion has passed the required managed/session/desktop Save/Open
and actual AOT gates on Windows, native Linux and Mac ARM64/APFS; G1 is closed.
[Production-readiness design packet and proposed gates](handoffs/2026-10-23-history-production-readiness-design.md).
Recommend validated current-root WB5 projection with full live/history ledger and
Redo preserved, atomic durable indexing, mutation-local records and explicit
witness/resource/reclamation contracts. SQLite is an evaluation candidate only.
The accepted feature-only correction is now promoted into ordinary native export;
the research association and real Rust session call the same ordinary exporter.
Generic full-ledger transfer stays strict. No production history integration,
protocol change, capability expansion, format cut or quota increase was added.
[Final feasibility acceptance and next design gate](handoffs/2026-10-23-disk-history-final-feasibility-review.md).
Historical-state Save promotion is accepted and closed. G2 now produces typed
stable-key authoritative changes during private mutation, coalesced against the
original transaction state and verified against an independent full checkpoint.
Failed/no-op candidates cannot publish a transition or ledger advance. Preserve
bound definitions, workbook/selection metadata, provenance, invalidation, content
identity, captures, Redo and lifetime reservations. G2 measurements must separate
mutation-local and full-oracle work, scratch admission, peak ownership and cleanup.
No default mutation promotion, persisted journal replacement, backend, witness,
coordinator, protocol or desktop history integration is authorized.
[Stage 4 association, save/retention evidence and remaining limits](handoffs/2026-10-23-history-association-stage4.md).
Windows/Linux/Mac: 69 cases each (207 total), 2,652 original hashes; Rust suites
346/382, 347/383 and 347/383, with strict lint passing on all three.
[Native APFS evidence and independent verification](handoffs/2026-10-23-macos-history-association-validation.md)
confirm 1,092 evidence hashes, 166 build inputs and 462 copied source hashes
(461 frozen files plus the ignored GPT baseline). On that original Stage 4 source, new exports of older historical
roots with later identity floors rejected; prepared exports survived later Undo/Clear.
G1 supersedes that native-export limitation only in the explicit experimental seam;
original evidence and strict generic-transfer rejection remain intact.
[Stage 3 acceptance and Stage 4 safeguards](handoffs/2026-10-23-history-durability-stage3-review.md).
[Stage 3 decision protocol, fault/restart evidence and limitations](handoffs/2026-10-23-history-durability-stage3.md).
366 scenarios passed across Windows/Linux, including parent-killed workers, ambiguous
outcomes, atomic branch/cursor decisions, corrupt-payload rejection and acknowledged
restart controls. Rust default/all-feature suites: Windows 346/380, Linux 347/381;
strict lint passes. Full export, research reconstruction and linear disk-ID lookup
remain costs; physical power-loss and general filesystem durability remain unproven. Stage 3 alone did not
establish file association.
[Stage 2 acceptance and Stage 3 safeguards](handoffs/2026-10-23-history-representation-stage2-review.md).
[Stage 2 comparison, evidence and bounded adaptive checkpoint experiment](handoffs/2026-10-23-history-representation-stage2.md).
100 baseline/final-policy runs passed; cache counts/capacities remain fixed from
1,000 to 10,000 edits. Windows 346/378 and Linux 347/379 Rust suites and strict lint
pass; 792 original sample hashes verified.
Checkpoint plus canonical-record deltas are accepted as the preferred Stage 3 candidate; fixed
intervals and the additional adaptive byte/work thresholds are experimental.
Stage 2 alone establishes neither durable transaction recovery nor production
container or Undo/Redo integration.
[Acceptance and Stage 2 resource-accounting qualifications](handoffs/2026-10-23-cold-history-stage1-review.md).
[Cold reconstruction, identity/capture proofs and validation](handoffs/2026-10-23-cold-history-restoration-stage1.md).
[Parent inventory acceptance and next gate](handoffs/2026-10-23-cold-history-inventory-review.md).
[Architecture approval and experiment authorization](handoffs/2026-10-23-tiered-history-design-review.md).
[Parent cold-identity clarification](handoffs/2026-10-23-cold-history-identity-review.md).
[Cold-state inventory and executable boundary probes](architecture/cold-history-state-inventory.md).
Windows/Linux probes distinguish portable UUIDs from process-local runtime NodeIds;
separate processes legitimately reuse the same numeric model/node pair. Portable
transfer is not a history capsule and rejects an older root with a later allocation
floor. A separate feature-only checkpoint adapter now reconstructs both roots from
bytes, leaving portable transfer unchanged. Windows/Linux same-session, fresh-process
and independent-copy proofs pass with exact content/provenance and zero-charge finish.
Default/all-feature Rust suites pass: Windows 346/377, Linux 347/378. The parent
approves durable logical IDs/reservations across restart, exact runtime NodeIds for
hot/cold activation in a live session, and coherent remapping in a fresh reopened
runtime namespace. Separate gates cover same-session eviction, restart logical
equivalence and simultaneous-copy isolation. No NodeId representation change or
persistent model-number ownership is authorized; the identity-contract hold is resolved.

The separate identity measurement completed 10,000 existing-cell replacements using
one identity, while create/restore/branch reached the unchanged floor of 8,192 after
8,191 new commits with one active cell. Both platforms finished at zero charge.
This reports the requested product scaling constraint, not retained disk history.
The parent accepts the inventory/probes and measurements. Identity-space scalability
is a separate prerequisite for removing experimental workbook limits; no quota increase,
recycling or rebasing is authorized. The three cold-restoration proofs above are
accepted as the semantic foundation for representation comparison. Portable transfer
remains unchanged. Durable fault/recovery and association remain later gates.
[Parent acceptance and superseding direction](handoffs/2026-10-23-undo-redo-feasibility-review.md),
[accepted executable foundation and evidence](handoffs/2026-10-23-undo-redo-feasibility.md),
[revised ADR-0023](adr/0023-document-transaction-history.md),
[review packet](handoffs/2026-10-23-undo-redo-design.md),
[bounded disk-history feasibility plan](architecture/disk-history-feasibility-plan.md).
The accepted feature-only foundation restores historical roots under fresh authority,
retains lifetime identity floors and isolates existing calculation captures, including
invalid/cyclic drafts. Its original receipt records Windows 346/367 and Linux 347/368
default/all-feature tests and ten zero-charge measurement runtimes; those historical
manifests remain intact beside the new Stage 1 evidence.

The parent declines 32 transitions/33 states and 16 MiB as product history limits.
These remain experimental measurements, not reasons to discard recoverable history
or reject otherwise valid edits. The revised target is effectively unlimited logical
history, bounded resident roots and extensible disk history, constrained by real
storage and explicit user retention/failure policy. Across-session history is an
intended capability. Keep transaction/content identities, fresh authority, monotonic
allocation floors and existing save/dirty/uncertain-installation contracts separate.

Next work: execute the authorized Stage 2 comparison of
checkpoints, canonical record deltas and shared persistent-state objects with commit
journaling, without choosing a container/database. The proposal covers cold restoration,
durable-before-publish transaction/cursor/ledger decisions, uncertain-write reconciliation,
saved-file association, copies/moves/replacement, explicit retention/failure policies
and resident eviction without history loss. No extensible store, durable journal,
recovery/association implementation or measurements are claimed. Additional safeguards
require separate create/Undo/branch identity-exhaustion measurements (completed as an
inventory finding), preserved journals when recovery is declined, and independent history
deletion/compilation decisions. Return experiment evidence for parent feasibility
review before production container selection or full history/session/desktop integration.
The Stage 1 16 MiB image/8 MiB ledger reservations are prototype bounds, not production
defaults. Stage 2 must separate serialized bytes, actual working buffers, required/
reserved scratch, reconstructed allocations and peak simultaneous ownership across
small/medium/formula-heavy/near-capacity cases. Preserve independent exact identity,
binding, diagnostic/provenance and invalidation oracles; output equality alone is insufficient.
Optimize Worksheet is separate; no JIT,
SIMD, format changes or irreversible history compaction begin here. ADR-0022 remains
accepted and closed.

Accepted preceding milestone: **Bounded Structured-Result and Presentation-Projection Proof â€” accepted; workstream closed**.
[Acceptance and scope closure](handoffs/2026-10-22-structured-results-projection-implementation-review.md),
[implementation and three-platform evidence](handoffs/2026-10-21-structured-results-projection-implementation.md).

Accepted preceding milestone: **Bounded Document-Owned Worksheet Text â€” accepted; workstream closed**.
[Acceptance and scope closure](handoffs/2026-10-21-worksheet-text-implementation-review.md),
[implementation/evidence](handoffs/2026-10-21-worksheet-text-implementation.md).

Accepted preceding milestone: **Native AOT Publish-and-Execute Validation â€” accepted; workstream closed**.
Windows x64, Linux x64 and native macOS ARM64 publish and execute without warnings.
Managed CI remains available; shared Windows/WSLg clipboard serialization is complete.
[Acceptance and next design scope](handoffs/2026-10-20-native-aot-review.md),
[implementation and evidence](handoffs/2026-10-20-native-aot-baseline.md).

Accepted milestone evidence: **bounded document-owned worksheet text**. ADR-0018 is accepted
with clarifications; implementation is in place. Final Windows/Linux full CI
(314/315 default Rust tests), strict all-feature lint/tests (330/331), and actual
Native AOT publish/execute pass without publish warnings. All eight desktop fields
and native peers pass with zero charge. Fresh native Mac/APFS managed CI, strict
all-feature lint, 315/331 default/all-feature Rust tests and Native AOT also pass
without warnings, product fixes or reruns. All 359 copied source hashes and 183
build/test inputs match; 438 returned files are present. Parent implementation
review is complete: accepted, including the escape-overhead rule;
the milestone is closed. Rust-owned classification,
reversible apostrophe/TSV escapes, strict numeric-consumer/SUM type errors, bounded
single-line text and WB4/private version cuts are approved. Rejected Open must
preserve the live workbook/revision/dirty state. The narrow nonnumeric marker
satisfied the design stop condition without a general value-system redesign.
Projections/connectors remain outside this closed scope.
[Implementation and evidence](handoffs/2026-10-21-worksheet-text-implementation.md),
[Mac APFS validation packet](handoffs/2026-10-21-macos-worksheet-text-instructions.md),
[accepted ADR-0018](adr/0018-document-owned-worksheet-text.md),
[review decision and stop conditions](handoffs/2026-10-20-worksheet-text-design-review.md).

Accepted preceding milestone: **Bounded Rectangular Clipboard, Formula Copy, WB3 and Desktop Acceptance Automation â€” accepted; workstream closed**.
[ADR-0017](adr/0017-rectangular-clipboard-and-formula-copy.md),
[implementation/evidence/optional inspection](handoffs/2026-10-20-rectangular-clipboard.md),
[fresh Mac/APFS packet](handoffs/2026-10-20-macos-rectangular-clipboard-instructions.md).
Rust now supplies bounded typed native copy, effective-binding scalar/SUM translation,
dollar-axis syntax, exact external TSV and one atomic paste within 256 slots.
Desktop clipboard transport highlights successful destinations; editor paste stays
text insertion. PIVOTWB3 replaces WB2 with explicit WB1/WB2 rejection, no legacy
reader/parser mode. Preserved rejected drafts remain non-executable until explicit
resubmission. Frozen hosted v1 and capacity limits remain unchanged.
Windows/Linux full CI (309/310 default Rust tests), strict all-feature lint and
325/326 all-feature tests pass. Final actual-window and C#â€“Rust boundary workflows
pass with zero-charge close for the clipboard foundation. The owner subsequently requested
[automated desktop acceptance](handoffs/2026-10-20-desktop-acceptance-automation.md):
`python scripts/ci.py --desktop` now covers the five groups, cross-process native
clipboard and real close choices. Plain inline-editor arrows commit/move; modified
arrows and formula-bar arrows retain text editing. Final Windows/WSLg acceptance
and all helper-process reports pass with zero charge; all 173 build/test input hashes
match. Fresh native Mac/APFS integrated desktop CI, strict lint and 310/326 default/
all-feature tests now pass, including cross-process UTI clipboard, all close peers,
arrow commits and zero-charge cleanup. All 339 copied source hashes matched before
this doc update; all 173 build/test inputs match, with 391 returned reports verified
by the transfer record. No Mac fixes or reruns. Routine manual replay is optional;
physical input/native pickers are not inferred. The parent review received 2026-10-20
accepts the implementation separately from the earlier design acceptance and closes
the milestone. [Implementation acceptance and scope closure](handoffs/2026-10-20-rectangular-clipboard-implementation-review.md).
[Parent review packet](handoffs/2026-10-20-rectangular-clipboard-review-packet.md),
[native Mac report](../artifacts/macos-validation-20261020T235001Z/REPORT.md).

Accepted preceding milestone: **Rectangular Worksheet Selection and Navigation â€” accepted; workstream closed**.
Anchor/active/extent live only in C# UI state. Drag and Shift extension, bounded
keyboard navigation, auto-scroll and active/region/inactive visuals are implemented.
Selection-only checks leave Rust revisions/dirty state unchanged and create no
computational selections or worksheet content. Editing/clipboard still target the
active cell only. [Interaction specification and checks](handoffs/2026-10-20-rectangular-selection.md),
[fresh APFS packet](handoffs/2026-10-20-macos-rectangular-selection-instructions.md).
Windows/Linux/native Mac full CI (290/291/291 default Rust tests) and actual-window
checks pass with zero-charge cleanup. Mac strict all-feature lint and 307 all-feature
tests pass. All 327 Mac copied source hashes and 168 reference build inputs match;
368 returned reports are transfer-verified without fixes/reruns. User-confirmed
manual tests pass on all three platforms. Rectangular cut/copy/paste is deliberately
not implemented; selection/navigation and active-cell operations are the current
scope. The parent review received 2026-10-20 accepts the implementation.
[Acceptance and next design scope](handoffs/2026-10-20-rectangular-selection-review.md).

Accepted milestone evidence: **Native AOT Publish-and-Execute Validation** on Windows
x64, Linux x64 and macOS ARM64. The parent direction review prioritizes a native
publish/run baseline, then worksheet text, then structured-result/projection design.
[Direction and bounded next scope](handoffs/2026-10-20-native-aot-next-scope.md).
Implementation is in place. Native Windows x64 and Linux x64 publish/run gates pass
without warnings, including all desktop/peer workflows and zero-charge shutdown.
Ordinary Windows/Linux full CI also passes. Fresh native Mac/APFS managed CI,
strict all-feature lint, 310/326 default/all-feature Rust tests and Native AOT
publish/run now pass without warnings or product fixes/gate reruns. Main and all
three native peers pass with zero charge. All 349 copied source hashes and 178
build/test inputs match; 426 returned files are present. Cross-platform evidence
was accepted by the parent review received 2026-10-20; the milestone is closed.
[Implementation and evidence](handoffs/2026-10-20-native-aot-baseline.md),
[Mac APFS validation instructions](handoffs/2026-10-20-macos-native-aot-instructions.md).
Retain ordinary CI and the separate Rust child; no installer, graph redesign or data feature work.
Dynamic producers/results/projections remain distinct, screen addressing belongs to
presentation, refresh stays separate from recalculation, and concrete `#` syntax
remains provisional. This direction does not freeze a grammar or start implementation.

The runner-enforced Windows/WSLg clipboard serialization follow-up is complete.
Managed and Native AOT suites share an exclusive lock; real cross-OS exclusion
checks pass in both directions. Details and evidence are in the Native AOT handoff.

Accepted preceding milestone: **Direct single-cell paste commit â€” accepted; workstream closed**.
Grid paste submits immediately through Rust and retains selection; editor/formula-bar
paste remains uncommitted text insertion. Invalid numbers/conflicts retain a
correctable draft; late/cancelled clipboard reads do not overwrite newer state.
[Implementation and checks](handoffs/2026-10-20-direct-single-cell-paste.md),
[fresh APFS packet](handoffs/2026-10-20-macos-direct-paste-instructions.md).
Windows/Linux/native Mac full CI (290/291/291 default Rust tests) and actual-window
checks pass with zero-charge cleanup. Mac strict all-feature lint and 307 all-feature
tests pass. All 324 copied Mac source hashes and 168 build inputs match; 367 returned
reports are transfer-verified, with no fixes/reruns. The user confirms manual
functionality across all three platforms works as intended. Broader paste features
remain future scope. The parent review received 2026-10-20 accepts this correction.
[Acceptance and next scope](handoffs/2026-10-20-direct-paste-review.md). No engine,
protocol, WB2 or capacity changes are included.

The accepted review selected **Rectangular Worksheet Selection and Navigation** next.
The user authorized implementation; see current work above. Multi-cell clipboard,
copy/fill and reference translation remain outside this scope.

Accepted preceding milestone: **In-place worksheet editing â€” accepted; workstream closed**.
[Review packet](handoffs/2026-10-20-in-place-editing-review-packet.md),
[implementation and checks](handoffs/2026-10-20-in-place-editing.md).
The editor, synchronized formula bar, clipboard commands and focus/caret feedback
are implemented. Windows/WSLg window checks and the fresh native Mac automated
run pass; Mac has 291 default / 307 all-feature tests, 320 matching source hashes,
168 matching build inputs and 366 verified returned reports, without fixes/reruns.
The user subsequently confirmed manual acceptance of clipboard key presses:
copy/paste shortcuts work. The earlier failed outcome and the reportâ€™s original
pending status remain historical evidence; clipboard usability is not yet complete.

The parent review received 2026-10-20 accepts the editor/clipboard/focus milestone.
[Acceptance and next scope](handoffs/2026-10-20-in-place-editing-review.md).
The recommended next assignment is direct single-cell paste commit: selected-grid
paste submits one checked Rust edit immediately; active-editor paste remains text
insertion. No engine, protocol, WB2 or capacity changes are requested. The user subsequently authorized implementation; see the current-work entry above.

Accepted preceding milestone: **First-Class Ranges and SUM â€” Slice C accepted; workstream closed**.
[Implementation and exact quotas](handoffs/2026-10-20-ranges-and-sum-slice-c.md),
[validation evidence](handoffs/2026-10-20-ranges-validation.md),
[native APFS packet](handoffs/2026-10-20-macos-ranges-instructions.md).
Canonical sparse selections, row-major checked SUM and PIVOTWB2 are implemented.
WB1 is explicitly unsupported; frozen hosted v1 rejects selection capability.
Windows manual native-dialog checks passed per the user. Native macOS APFS full
validation and all six user-confirmed manual checklist groups pass. Full
Windows/Linux/macOS CI (290/291/291 default, 306/307/307 all-feature), strict lint
and actual-window checks pass with zero-charge cleanup. All 318 Mac copied source
hashes and 168 build inputs match; no product fixes or validation reruns were needed.
[Native report](../artifacts/macos-validation-20261020T130207Z/REPORT.md).
The user confirmed WSLg Save-on-close passes on local WSL2 storage and an untouched
workbook closes without prompting. WSLg Windows-volume Save-on-close reported
installation uncertainty; that mounted-path limitation and the historical native
abort remain unresolved.
The parent review received 2026-10-20 accepts the implementation separately from
the ADR-0016 design. [Acceptance record](handoffs/2026-10-20-ranges-and-sum-review.md).
The user authorized focused in-place worksheet editing after discussing scope.

Accepted preceding milestone: **Native Workbook Persistence and Application Lifecycle â€” Slice B**.
[ADR-0015](adr/0015-native-workbook-documents.md) and the
[implementation handoff](handoffs/2026-10-20-workbook-persistence-slice-b.md) describe
native workbook files, validated stable-identity restoration, safe staged Save,
external conflicts, exact saved-generation acknowledgement and Avalonia file actions.
The document codec is separate from the storage-independent workbook; the canonical
engine, product limits and frozen hosted format remain intact. Windows/Linux/native macOS full CI passes (280/281/281 default Rust tests,
296/297/297 all-feature), with .NET/parity/real IPC and actual Save/Open window
checks, zero-charge normal close and matching source hashes. The fresh native
APFS run needed no fixes or reruns; all 305 copied-file hashes match. Native file
pickers and dirty-close dialog choices remain untested manually. Slice B and
ADR-0015 are accepted by the parent review received 2026-10-20. The next assignment
is First-Class Ranges and SUM â€” Spreadsheet Semantics Slice C, beginning with an
explicit range-semantics decision before permanent representation.
[Review decision and next scope](handoffs/2026-10-20-workbook-persistence-review.md).
[Native report](../artifacts/macos-validation-20261020T111827Z/REPORT.md).

Accepted preceding milestone: **Runnable Application â€” C# Session and Worksheet Slice A**.
A real Avalonia worksheet edits an owned Rust Workbook through asynchronous framed
Protobuf pipes. Numeric/formula entry, clear/move, bounded scrolling, readable draft
errors, exact revision/generation tracking and orderly session disposal are implemented.
The backend retains production limits; no C# evaluator, file container or new engine
is introduced. [ADR-0014](adr/0014-application-session-and-worksheet.md),
[run/verification handoff](handoffs/2026-10-19-application-worksheet-slice-a.md),
[native APFS packet](handoffs/2026-10-19-macos-application-slice-a-instructions.md).
Windows/isolated Linux full CI passes (275/276 default Rust tests, 291/292 all-feature),
.NET Release and real boundary tests. Final application checks include queue-byte
admission, exact input, stale-display conflicts, viewport bounds, cancellation and
zero-charge disposal. Both final Windows/WSLg window checks and image inspection
pass. The initial WSLg native startup abort remains unresolved; no unconditional
Linux startup-reliability claim is made. Boundary p95 edit/viewport/selected-chain
is 0.130/0.308/0.281 ms Windows and 0.387/0.907/0.326 ms Linux; UI refresh includes
35 ms coalescing. See the handoff for actual timings and limits.

Run `python scripts/run_desktop.py` from the repository root. This is a volatile,
unsaved worksheet; closing the application discards it. Native macOS full CI and
actual Avalonia execution now pass from a source-matched internal APFS copy, with
276 default / 292 all-feature Rust tests, six .NET projects and zero final Rust charge.
No fixes or reruns were needed. Boundary p95 edit/viewport/calculation is
0.117/0.220/0.119 ms; eight UI observations span 39.45â€“108.56 ms.
[Native report](../artifacts/macos-validation-20261020T101213Z/REPORT.md).
Slice A and ADR-0014 are accepted by the parent review received 2026-10-20.
Next assignment: Native Workbook Persistence and Application Lifecycle â€” Slice B.
See the [review decision and scope](handoffs/2026-10-20-application-slice-a-review.md).

Accepted preceding milestone: **Canonical Engine Consolidation** implements the owner direction
in [ADR-0013](adr/0013-canonical-engine-consolidation.md). WorkingModel and Workbook
now always use persistent storage. Convenience constructors share the lazy runtime;
new_in retains application ownership. Reference backends/constructors, activation,
selector enums/features and obsolete comparison prototypes are removed. Host,
document/storage and daemon paths use the same canonical graph; codecs and product
caps are unchanged. See [architecture](architecture/canonical-engine-consolidation.md),
[evidence](performance/canonical-engine-consolidation.md) and
[review handoff](handoffs/2026-10-19-canonical-engine-consolidation.md).

Windows/isolated WSL2 Linux/native macOS CI passes 275/276/276 default tests and
291/292/292 all-feature tests, strict lint, .NET/parity and actual IPC. All 99 matrix children pass correctness,
resource guards and zero final charges. Existing construction/bulk targets pass;
one Linux 1K-start fan-out viewport p95 misses 5 ms at 5.343 ms. Its single matched
repeat passes at 3.381 ms; preserve the original miss as a review limitation.
The [fresh native APFS report](../artifacts/macos-validation-20261020T015824Z/REPORT.md)
passes 33 cases and all native targets, with all 280 copied source hashes matching
before documentation edits and no fixes/reruns. All eight pressure samples were
normal; completed-child high-water is not live RSS guarding. Including the Linux
diagnostic repeat, all 100 children finish with zero domain charge.
The parent accepts consolidation and ADR-0013. The graph migration workstream is
closed; the recorded viewport exception does not block the application slice.

Historical readiness milestone (constructor policy superseded by ADR-0013):
Previous milestone: **Persistent Workbook Production Readiness and Construction Profiling**
is implemented with Windows/Linux/native macOS validation complete. A bounded private-index insertion optimization reuses uniquely
owned candidate nodes while preserving shared roots, admission and atomic rollback.
Production Workbook remains reference; explicit new_in remains the application-owned
runtime path. ADR-0012 is Accepted. See the
[readiness architecture/compatibility matrix](architecture/persistent-workbook-readiness.md),
[matched profile/evidence](performance/persistent-workbook-readiness.md),
[handoff](handoffs/2026-10-19-persistent-workbook-readiness.md) and
[fresh native APFS packet](handoffs/2026-10-19-macos-persistent-workbook-readiness-instructions.md).
Full default CI passes (251/252/252 tests), all-feature regression passes 300/301/301,
fallback 21 each and workbook-only 49 each. All 234 matched ordinary/profile/core
children pass with identical 68 source hashes and zero final charges. At 100K,
workbook construction improves 1.7â€“2.1x Windows / 1.5â€“1.9x Linux / 1.44â€“1.68x macOS; numeric requested
traffic falls from 1.492 GB to 651 MB. Retained/peak memory is substantially unchanged.
Core name/page work explains much of the remaining reference gap. Existing interactive
and bulk investigation budgets pass. Recommend explicit new_in for the next bounded
application/session slice after review; keep new reference.
The [fresh native report](../artifacts/macos-validation-20261020T010244Z/REPORT.md)
validates all 296 copied source hashes on internal APFS. All 78 native cases pass;
52 pressure samples were normal and completed-child high-water peaked at 1,418.8 MiB.
This does not establish a capacity increase; native live RSS guarding remains unavailable.
The preceding foundation run remains historical. Stop for review before constructor changes.

Preceding accepted milestone: **Persistent Workbook Integration and Bulk Mutation Foundation**
is implemented behind `pivot-workbook/persistent-core`; production Workbook remains
reference. Canonical core batches reserve identities/resources, bind final names,
publish atomically and dispatch once. Persistent workbook edits retain sparse
stable identity and pinned S/T metadata with bounded rectangle reads. See
[ADR-0012](adr/0012-bulk-mutation-and-persistent-workbook.md),
[architecture](architecture/persistent-workbook-bulk.md),
[evidence](performance/persistent-workbook-bulk.md) and
[handoff](handoffs/2026-10-19-persistent-workbook-bulk.md).
Full default CI passes on Windows/Linux/native macOS (248/249/249 Rust tests),
with all-feature regression 296/297/297, fallback 21 each and workbook-only 48 each.
All 335 Release/diagnostic children pass with zero final domain charges; native
validation used a fresh internal APFS copy and all 290 source hashes matched before
this documentation update. Core batching improves roughly 5â€“9x Windows / 21â€“25x
Linux / 4.5â€“5.3x macOS over individual persistent edits. Native 100K workbook
population is 1.14â€“1.40 s, still slower than optimized reference. The
[native report](../artifacts/macos-validation-20261019T220845Z/REPORT.md) records
warning-level memory pressure and sampled RSS only, not native child high-water.
Foundation accepted by the parent; production workbook selection stays reference.

The preceding **Standalone Default Transition** is accepted.
Default-feature standalone WorkingModel::new now starts directly on persistent
state with a shared convenience runtime; application/session owners can use new_in
and an explicit runtime with aggregate admission and deterministic shutdown.
Reference fallback is explicit. Host, storage, workbook, legacy import and daemon
constructors deliberately retain their reference paths; frozen formats and limits
are unchanged. That transition is accepted. See
[ADR-0011](adr/0011-standalone-persistent-default.md),
[constructor/lifecycle policy](architecture/standalone-default-transition.md) and
[handoff](handoffs/2026-10-19-standalone-default-transition.md).
Native macOS transition validation also passed from a fresh APFS copy. At transition review,
Windows/Linux/macOS default counts were 241/242/242 and combined counts 264/265/265;
all ninety transition Release processes passed. Parent acceptance is recorded in ADR-0011. That earlier native run predates bulk; current bulk validation is recorded above.

The Rust workspace and .NET 10/Avalonia solution are scaffolded. Architecture
records live in `docs/adr/` using `ADR-NNNN` identifiers. ADR-0001 records the
previously agreed graph-first design.

`pivot-types` provides `PivotDecimal`, a private `rust_decimal` wrapper with
decimal text parsing, integer construction, formatting, comparisons, and checked
addition, subtraction, multiplication, division, and negation. Arithmetic errors
distinguish overflow and division by zero. Unrepresentable literals are rejected.
No float conversion API is exposed. Arithmetic outputs are canonicalized; explicit
rounding supports seven modes with nearest/ties-away-from-zero as default. .NET
helpers expose the same behavior. A shared 10,099-vector suite passes for values,
errors, and scales, including seeded arithmetic cases and independent rounding
expectations. See `tests/decimal-parity/verify.py` and the numeric architecture note.

`pivot-core::expression` parses decimal literals, ASCII identifiers, unary signs,
binary arithmetic, and parentheses into an AST. Multiplication/division precede
addition/subtraction; binary operators associate left. Names are case-preserving
and unresolved. The AST has no evaluation or I/O behavior.

Editable models now support stable node IDs, named decimal inputs, computed nodes,
and immutable snapshots with captured inputs and model-scoped revisions. Snapshot
validation checks bound IDs, extracts forward/reverse dependencies, orders nodes, and
rejects cycles. See ADR-0002 and the model/snapshot architecture note.

Snapshot evaluation now supports selected outputs or all nodes, bound decimal
expressions, calculation-local state, typed arithmetic/dependency errors, and
immutable results retaining their snapshot revision. Isolated invocation accepts
temporary input overrides keyed by NodeId, with captured defaults and typed binding
validation. Independent invocations share only immutable snapshot definitions.
Independent arithmetic failures do not prevent other outputs from being computed.
Forty-four Rust tests pass on Windows and Ubuntu WSL2 Linux for the command-REPL
slice. macOS ARM64 last passed 41 tests before this slice; the REPL changes require
a fresh Mac run. Public model ports now support named invocation,
required inputs, explicit optional defaults, and stable port identities. Formula
names bind only at edit submission; snapshots reuse stable-ID references.
Public values have extensible tagged value/type boundaries (Decimal-only today).

Invalidation, IEEE-754 and other non-decimal
values remain deferred. A demo daemon and minimal CLI now use named pipes on
Windows and Unix-domain sockets on Unix (Linux WSL2 and macOS ARM64 verified). ADR-0004 is accepted after the review
refinements. The portable contract has generated Rust/C# codecs, numeric provenance
validation, explicit input sources, negotiated limits, and an in-process pivot-host
registry/dispatcher with mandatory handshake sessions. ADR-0005 is Accepted;
local IPC includes current-user peer checks, bounded framing, connection limits,
and a conservative calculation work budget. Idle/partial-read/write deadlines,
connection caps, and work budgets are configurable operational policy. A persistent
command REPL shares the actual IPC client and supports session-local history,
line editing, explicit reconnect, and no automatic replay after disconnect.
Minimal named-model authoring and selected-node evaluation are available through
typed daemon requests; general scripting remains deferred.
Local CI is available through `python scripts/ci.py`, covering Rust formatting,
lint/tests, .NET Release builds, and cross-language decimal compatibility. See
`docs/local-ci.md` for prerequisites and current platform/reproducibility limits.
The complete CI pipeline now passes in Ubuntu WSL2 with .NET SDK 10.0.112,
including both cross-language suites and the .NET Release solution build.
The complete CI pipeline also passes on macOS 27.0.1 ARM64 with .NET SDK
10.0.402 using an isolated local APFS source/build copy. See
`docs/platform-validation.md` for reports and the SMB build limitation.
Native AOT, GUI execution, and actual cross-user endpoint denial remain unverified.

The editable working-model service is now designed in accepted ADR-0006 and
`docs/architecture/working-model-service.md`: connection-independent resources,
atomic expected-revision edit batches, draft validation, unpublished evaluation,
publication boundaries, and operation-receipt reconciliation. Slice A is implemented
in `pivot_host::management`: memory-only resources, portable identities, authorized
discovery/inspection and attachments, explicit quotas, and replay-safe creation.
Slice B adds private transactional staging, atomic edit batches, one public revision
per batch, portable node/port IDs, revision-pinned raw draft reads and diagnostics,
final-directory binding for submitted formulas, and retained edit receipts.
Slice C adds exact-revision structural validation, detached immutable evaluation
captures with temporary overrides, and replay-safe publication into a coordinated
immutable invocation registry. Published revisions and working revisions remain
distinct. Serialized authoring is available in process and through opt-in local
daemon IPC and the minimal authoring REPL. See
`docs/architecture/working-model-management.md` for the in-process API and limits.
Transactional editing details are in `docs/architecture/working-model-editing.md`.
Calculation/publication details are in
`docs/architecture/working-model-calculation-publication.md`. Persistence, identity
compaction and richer edit diagnostics remain follow-ups; memory-only commits do
not survive service shutdown. The latest native macOS CI now covers the REPL; Native AOT remains
on the verification backlog.

ADR-0007 is Accepted. Production authoring schemas and shared types, Rust/C# codecs,
bounded recursive mutation preflight, typed intent and opt-in v1.1 negotiation are
implemented and compatibility-tested. The daemon defaults to invocation-only;
`--authoring` explicitly enables v1.1 authoring. Stage A implements all twelve
serialized authoring operations in process, signed bounded paging, session-owned
attachment cleanup, staged partial-port patches and exact wire-intent replay.
Evaluation runs outside the service lock and delivery rechecks authorization.
Detailed receipt diagnostic retention remains deferred. See
`docs/architecture/serialized-authoring-adapter.md` and
`docs/architecture/authoring-protocol-foundation.md`. Stage B integrates actual
local IPC, OS-derived principals, one demo/authored publication registry and
blocking dispatch with no service lock across transport writes. Five new IPC
tests cover daemon authoring, legacy compatibility, lost responses, concurrent
edits, denial/revocation, cancellation cleanup, quotas and malicious traffic.
See `docs/architecture/daemon-authoring.md` and the Stage B handoff for current
Windows/WSL validation; the latest macOS CI also passes these retained tests.
The Stage B Rust suite had 130 tests, including 23 adapter/router tests and five
daemon IPC integration tests. Full Windows and WSL CI reports are linked from
`docs/handoffs/2026-10-18-daemon-authoring.md`.
The subsequent minimal authoring REPL adds create/list/select/inspect, decimal
inputs, opaque formula submissions/repairs, public ports, validation, evaluation
and exact-revision publication. It preserves pending operation IDs/intent after
ambiguous responses and requires explicit recovery/retry; conflicts never trigger
automatic refresh. Creating/opening a model warns that restart loses its state.
Five executable REPL tests bring the Rust suite to 135 tests. The real-process
IPC harness also checks authoring, reopening and immutable published invocation.
See `docs/architecture/command-repl.md` and
`docs/handoffs/2026-10-18-minimal-repl-authoring.md` for evidence and limits.
Optional hosted persistence direction is a common transactional contract with configurable
providers: SQLite first/default locally with a filesystem path, and planned
PostgreSQL/MySQL adapters. Storage choice is independent of local/network service
mode. Initially one active service owns a store; multi-instance coordination remains
separate work. See `docs/architecture/persistence-providers.md`. Provider support,
schemas and recovery are not implemented; runtime durability remains MemoryOnly.

ADR-0008 is Accepted for optional durable hosted storage with the required review refinements: database-authoritative domain transactions, portable bound
expressions/exact values, durable identities and receipt scopes, SQLite ownership
and durability profile, recovery/failure matrix and shared provider conformance.
See `docs/handoffs/2026-10-18-transactional-persistence-design.md` for the review
packet. Durable wire semantics require a separately reviewed capability extension.
The first implementation slice now adds owned portable graph/publication records,
bounded deterministic-CBOR v1 encoding/decoding, a validated no-rebinding core import
boundary, retired identities, exact decimals and coherent collection restoration.
There are 11 frozen codec fixtures and 18 new tests (153 Rust workspace tests total).
The running daemon/REPL remain MemoryOnly; no database/provider, migration, durable
coordinator, wire fields or cross-restart replay is implemented. See
`docs/architecture/portable-storage-foundation.md` and
`docs/handoffs/2026-10-18-portable-storage-foundation.md`.

## Accepted document foundation

The parent review accepts the ADR-0008 serialization/restoration foundation and
preserves ADR-0006/0007/0008 history. Further hosted persistence is deferred:
SQLite ownership feasibility, schema/migrations, durable commit coordination,
cross-restart receipts and PostgreSQL/MySQL adapters are not the next work.

[Accepted ADR-0009](adr/0009-portable-documents-and-local-lifecycle.md) defines
ordinary model documents and local New/Open/Edit/Calculate/Save/Save As/Close
behavior. Document Foundation Slice A now implements hosting-free graph payloads
and thin one-graph in-memory sessions in pivot-storage. Sessions reuse core staging,
binding, validated import and evaluation, with no hosted registration, publication,
store/epoch identities or receipts. Immutable export/restore, independent copies,
exact values, invalid drafts, stale-token checks and saved checkpoints are tested.
See [implementation details](architecture/document-foundation.md) and the
[Slice A review handoff](handoffs/2026-10-18-document-foundation-slice-a.md).

The Slice A suite had 167 Rust tests, including 13 new document integration tests and
one new hosted resource-budget compatibility test. Full Windows CI and WSL Linux
Rust/lint/IPC checks passed at submission; the latest macOS CI also covers these tests.
Document Foundation Slice B now adds an explicitly experimental versioned envelope
and a headless Rust file broker for New/Open/Save/Save As/Close. It stages complete
private candidates, checks observed target fingerprints, installs without truncation,
verifies installed content and only then acknowledges the exact export. Fault tests
cover conflicts, failed/uncertain saves, reconciliation and S/T dirty-state behavior.
Independent-process tests create, reopen, calculate/invoke, edit/save, copy and reopen
models without hosting. The core and frozen hosted storage-v1 fixtures are unchanged.
See the [experimental envelope](architecture/experimental-document-format.md),
[broker and limits](architecture/experimental-file-broker.md) and
[Slice B handoff](handoffs/2026-10-19-document-foundation-slice-b.md).

Slice B passes full Windows CI (185 Rust tests) and available WSL Linux checks
(186 tests, including Unix-only permissions/alias coverage) plus actual IPC/REPL.
File tests now cover NTFS, native Linux disk-backed storage and local macOS APFS.

No production .pivot format, SQLite/ZIP container, document RPC, C# broker, worksheets
or UI is implemented. Process termination checks do not prove power-loss durability.
The latest native macOS suite passes the retained file tests. The hosted storage-v1 record is
not the native document format.

## Workbook projection and current performance gate

The parent review accepts ADR-0010's structural architecture while reserving
spreadsheet compatibility semantics. [ADR-0010](adr/0010-workbook-projection-and-spreadsheet-semantics.md)
now distinguishes accepted boundaries from future cut/copy/delete, range affinity,
blank/text/aggregate and shared-writable-node policies. No Excel compatibility claim
or production workbook format is made.

Sparse Worksheet and Graph Projection Slice A is implemented in the dedicated
pivot-workbook crate. It has one sparse worksheet/graph, independent cell/axis/node
identities, exact decimal input metadata, typed stable references, clear/move/retire,
atomic bounded edit batches, no-op detection, diagnostics, immutable calculation
captures and basic bounded rectangular reads. The trusted resolved-token seam in
the core transfer module shares existing token/dependency validation and evaluator;
it does not parse fabricated identifiers or introduce a second expression engine.

See [implementation and small costs](architecture/workbook-projection-foundation.md)
and [Slice A handoff](handoffs/2026-10-19-workbook-projection-slice-a.md).
Full Windows CI passes 200 Rust tests, .NET/parity and actual IPC/REPL regressions.
Available WSL Linux checks passed 201 tests and real IPC at Slice A submission.
The later Slice B macOS run covers these regressions too. Tiny debug-profile
measurements at 17/129/513 live
cells are not spreadsheet-scale claims; full-copy graph staging/snapshot rebuilds
and logical small-model budgets remain explicit.

Spreadsheet Scalar Formula Parsing and Binding Slice B is implemented. The existing
Rust parser has an explicit spreadsheet mode for optional `=`, same-sheet A1
references, decimal arithmetic, unary signs and parentheses. Text formula edits bind
once to stable cell/node targets and preserve original source, reference spans and
transaction/candidate provenance. Malformed/out-of-bounds drafts are diagnosable and
non-executable; hard failures roll back the batch. Moves and coordinate reuse do
not rebind formulas; explicit resubmission does.

See [grammar and implementation](architecture/spreadsheet-scalar-formulas.md) and
[Slice B handoff](handoffs/2026-10-19-spreadsheet-formulas-slice-b.md) for current
verification and costs. Full Windows CI passes 209 Rust tests plus .NET/parity/IPC;
available WSL2 checks pass 210 Rust tests plus IPC. Native macOS ARM64 now passes
full CI with 210 Rust tests, .NET Release, both parity suites and actual IPC/REPL.
At Slice B validation, the working tree matched all 215 Mac source hashes;
builds and .NET outputs stayed on local APFS, with reports copied back to SMB.
See the [macOS review](handoffs/2026-10-19-macos-validation.md).
The parent review accepted Slice B and replaced the following C# priority with a
mandatory graph/workbook performance-feasibility gate. The opt-in research harness
now compares optimized baseline and shared-topology/input-overlay behavior at
1K/10K/100K cells; see the [feasibility assessment](architecture/graph-workbook-scaling-feasibility.md)
and [research handoff](handoffs/2026-10-19-scaling-feasibility.md). Its recommendation
is B: viable with architectural restructuring, not production adoption of the
prototype. Default behavior and small-model production limits remain unchanged.
The parent accepted assessment B and authorized the bounded Structural Mutation
Feasibility slice. A separate opt-in paged core and fixed-projection workbook
adapter now test localized formula/dependency edits and coherent immutable captures.
See the [representation comparison](architecture/structural-mutation-feasibility.md)
and [structural handoff](handoffs/2026-10-19-structural-mutation-feasibility.md).
Matched end-to-end timings include the baseline's deferred validation. Baseline
and prototype memory workloads run in separate guarded processes. No production
replacement is adopted. The parent accepted the structural research and authorized
Production Graph Representation - Compatibility Foundation. That opt-in slice now
uses persistent name/reverse-edge indexes, supports ports/invocation and invalid
restoration, shares accepted binding/validation rules, charges unique core storage
and candidate work, and provides bounded off-thread reclamation.
See the [compatibility design](architecture/persistent-core-compatibility-foundation.md)
and [foundation handoff](handoffs/2026-10-19-compatibility-foundation.md) for matched
Release results, 32-generation retention, 1M repeats and remaining blockers.
That foundation recommended focused integration work on revisions/diagnostics,
allocation ownership, ports and workbook coverage. The parent accepted it and
requested Controlled Production Core Integration Stage 1, now implemented through
WorkingModel as described at the top of this status. Review its explicit API and
scheduling gaps before another migration slice. C# work stays held pending Stage 1
review. Relative/absolute
copy/fill, cross-sheet references, row/column structural edits, ranges/SUM, typed
blank/text engine values, incremental calculation, workbook files, C# IPC and
Avalonia rendering remain unimplemented.

Keep production file ACL/alternate-stream policy, final-check writer races and
power-loss guarantees separate from this worksheet proof. GUI execution, Native AOT
and actual cross-user denial remain unverified.
Do not expand the experimental file broker or resume optional hosted persistence
automatically. Production container selection remains a later workbook-driven
decision. Native AOT, native macOS corroboration of this new experiment and
production-scale validation remain backlog; the earlier Slice B Mac pass is retained
as historical evidence. Stop after the controlled-integration Stage 1 review, without new features.


Native macOS Stage 2 validation (20261019T182116Z) passed on Apple M1 ARM64 from
an isolated internal APFS copy: 210 default, 237 integration-only, 234 research-only
and 260 combined tests, three strict feature lint configurations, complete .NET/
parity/real IPC CI and all thirty Release benchmark processes. All copied source
hashes matched before this documentation update; no product changes were needed.
All review targets passed and final domain charges were zero. The runner does not
support macOS RSS; separate sparse native samples do not establish child high-water
marks. See the [native report](../artifacts/macos-validation-20261019T182116Z/REPORT.md)
and [Stage 2 performance addendum](performance/controlled-core-integration-stage2.md).

Standalone transition final Windows/Linux results: full default CI passed (241/242 tests), combined regression passed 264/265, reference-only core passed 21 on both, all strict lint configurations passed and all sixty verified Release children passed the interactive targets. Native macOS transition subsequently passed (242 default, 265 combined and 21 fallback tests, full CI and thirty more Release children); see the transition handoff for current three-platform results and the retained bulk-construction tradeoff.


Native standalone transition follow-up (20261019T191136Z): all requested gates
passed on internal APFS, with no product changes or reruns. Full CI included .NET,
10,099 decimal vectors, protocol compatibility and actual Unix-socket IPC; all
thirty native benchmark children passed with zero convenience/session final domain
charges. Native 100K p95 input/capture was 0.013 ms, input/chain 0.331 ms, tail
formula 0.491 ms and bounded inspection 0.007 ms. Bulk fixture construction still
cost 1.300 seconds versus 0.0467 reference; full snapshot 48.748 ms versus 30.313
reference. Native RSS guard/high-water support remains unavailable; separate
monitoring is sampled evidence only. All three-platform validation gates are now
satisfied for parent review; no downstream backend migration is included.

Parent implementation review: [computational implementation acceptable; final
acceptance deferred pending WB5-only correction and fresh three-platform gates](handoffs/2026-10-22-spreadsheet-aggregate-implementation-review.md).
