# Hyper-V x64 — full headless CI execution packet

2026-10-24. Both x64 guests are HeadlessQualified. This executes the separately
authorized existing **headless** `python scripts/ci.py`, once per guest.
[Acceptance and boundaries](2026-10-24-x64-headless-qualification-acceptance.md).
No GUI/AOT acceptance, VM lifecycle action, remote dispatch or G3 work is authorized.

## Frozen input and local-only work

Input packet on the share:
`artifacts/guest-headless-ci/x64-20261024/`.
It contains `source.zip`, complete source/build manifests, the two local helper
files and `bundle-receipt.json`. Source is frozen from this repository; it is not
the older host/WSL test snapshot. Do not regenerate the packet or edit its files.

The manually invoked helper validates packet hashes, refuses an unexpected hostname
or nonnative/non-x64 root/process, extracts into a **new** native work directory,
verifies source and runs only the existing CI entry point without `--desktop`.
It adds no test implementation, product assertion or contributor CI requirement.
Actual CI commands/exits remain in CI's own summary. The helper does not connect
to another machine, select or start VMs, install tools, change redirection or grant
desktop permits. It is not a remote orchestration interface.

Source, target, .NET obj/bin, caches, temporary fixtures and original report acquisition
use native NTFS/ext4. Cargo/NuGet project dependency restore is the normal existing
build step, not an OS package/SDK installer. Tools must already be installed.
The existing Linux IPC suite also creates its transient guest-owned socket endpoints
under `/tmp`; persistence fixtures use the explicitly native work/temp roots.
No native-file guarantee is inferred from SMB/CIFS.

## Windows guest — Pivot-Win11-x64

Run inside the Windows guest as its ordinary test user. Open CMD, not an elevated
PowerShell. This avoids script-policy changes and the separate elevated J: mapping.
The exact installed VS path and guest hostname were verified in qualification.

```bat
call "C:\Program Files\Microsoft Visual Studio\2022\Community\Common7\Tools\VsDevCmd.bat" -arch=amd64 -host_arch=amd64
python -c "__import__('shutil').copytree(r'J:\coding\pivot\artifacts\guest-headless-ci\x64-20261024',r'C:\PivotWork\ci-input\x64-20261024')"
python C:\PivotWork\ci-input\x64-20261024\run_headless_ci.py --work-root C:\PivotWork --expected-hostname WinDev2407Eval --return-root J:\coding\pivot\artifacts\guest-headless-ci\results\Pivot-Win11-x64
```

Copytree deliberately refuses an existing input destination. If it already exists,
stop and inspect whether the run has already started/completed; don't overwrite it
or replay tests to make transfer work. Use the same installed Python that ran the
guest collector; do not install tools or invoke Store installation aliases.

## Ubuntu guest — Pivot-Ubuntu-x64

Run inside the qualified Hyper-V Ubuntu guest, **not WSL or the legacy Ubuntu 18.04
native-library build environment**:

```sh
mkdir -p "$HOME/pivot-work/ci-input"
python3 -c 'import shutil,sys; shutil.copytree(sys.argv[1],sys.argv[2])' /mnt/j/coding/pivot/artifacts/guest-headless-ci/x64-20261024 "$HOME/pivot-work/ci-input/x64-20261024"
python3 "$HOME/pivot-work/ci-input/x64-20261024/run_headless_ci.py" --work-root "$HOME/pivot-work" --expected-hostname Pivot-Ubuntu-x64 --return-root /mnt/j/coding/pivot/artifacts/guest-headless-ci/results/Pivot-Ubuntu-x64
```

No sudo, guest package installation, new service, GUI session manipulation or host
clipboard action is needed. If prerequisites or network restore are unavailable,
preserve the first failure and report it; do not install or retry until green.

## Outcomes and evidence handling

The helper prints the native `ci-<timestamp>-<nonce>` work directory before execution.
Watch its `report/console.log` from another guest terminal if desired. A run may take
time on cold dependency caches. An explicit one-hour overall timeout bounds the
invocation. On cancellation/timeout, preserve artifacts and inspect owned processes;
do not automatically restart or kill unrelated processes.

On completion, source files and frozen inputs are verified again. Build outputs
are excluded from source-addition checks. A scoped, read-only process snapshot checks
for survivors referring to the unique work root; it is not a machine security sandbox
or proof about unrelated processes. Existing tests retain their own explicit runtime
finish/charge assertions. No orphan is automatically killed by this wrapper.

The complete report subtree, raw logs, native environment, source manifests,
before/after integrity, process snapshot, CI exit and wrapper result are hashed
**on the guest before transfer**. The helper copies to a new invocation directory
under the explicit shared return root and verifies those hashes there, adding a
transfer receipt. No automatic fallback to host tests or host desktop exists.

Exit 0 indicates completed headless CI and evidence checks. An executed CI failure
remains Failed, with its actual exit code and raw summary preserved. Missing tool,
interrupted execution, unreadable cleanup evidence or transfer problems are Blocked,
never a platform pass. If transfer fails, the native report is retained: copy that
existing directory manually later, **do not rerun CI to retry copying**. Original
source/targets/caches/fixtures remain native and are not included in report transfer.

Expected current pipeline: 13 steps, seven-project .NET build, nine policy tests,
347 default Rust tests on Windows / 348 on Linux, plus decimal/aggregate/protocol,
real IPC and application-session tests. Verify actual counts from logs; investigate
differences without changing expectations or skipping tests. This slice does not
add all-feature research/AOT/desktop gates to the accepted headless entry point.

Tell the host-side Codex when the shared `results` directories appear. It must
independently verify returned hashes, actual outcomes, zero-charge fields, source
identity and setup/failure distinctions before preparing the parent review packet.
Do not relabel old host/WSL results as guest runs. Formal guest CI acceptance still
requires parent review. Headless success does not authorize GUI activation.

## Separate GUI work remains open

Host connection/redirection settings, actual guest session ownership and the proposed
owner-scheduled bidirectional clipboard sentinel remain separate. Do not perform that
procedure implicitly, run a desktop executable, use `--desktop`, invoke native AOT
publish-and-execute, approve a guest permit or begin remote orchestration.
ARM64 qualification remains separate. G1/G2 and execution safety stay closed;
G3 remains held and G4–G7 deferred. Return this slice's guest evidence for review.
