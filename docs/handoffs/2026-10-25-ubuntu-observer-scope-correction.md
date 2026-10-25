# Ubuntu non-GUI observer refusal and bounded correction

Update: [corrected guest non-GUI evidence verifies](2026-10-25-ubuntu-implementation-review-submission.md).
No repeat requested. Original first result below remains Blocked and preserved.

Original guest outcome: **Blocked**, preserved without reclassification.
Report:
`artifacts/ubuntu-managed-implementation/non-gui-results/Pivot-Ubuntu-x64/ubuntu-wrapper-nongui-b43536af8d594dbe820ca39ee97b2bd5/`.
Twenty-two frozen tests pass in the guest. Live session/seat/boot/socket continuity,
installed tool/library queries and util-linux script 2.41.3 discovery succeeded.
The observer then Blocked; context-after/source-after checks were not reached.
No GUI, build, clipboard access, setting change or elevation occurred.

Original hashes and transfer verify. Origin manifest SHA-256:
`42ca4fbc07158236e7c292a2417028300d5339034704dc003c08ae8f7dbf0aff`.
Independent reconciliation: `artifacts/ubuntu-managed-implementation/first-blocked-verification.json`.
Preserve the exact original report and disabled packet receipt
`322def344aa89f1032d9662fcb08c59dfd17a5d0d454e899acac0a3a63849900`.

## Cause and correction

The observer resolved `/proc/<pid>/exe` before deciding scope. Ubuntu denied reads
for preexisting systemd/PAM/ssh-agent processes (3401, 3403, 3683). The checker itself
(30073) was selected by its native work-path command but ancestry walked beyond the
checker into the unrelated user manager, causing another permission denial.
This is an observer scope defect, not failed Pivot GUI functionality or evidence
that VM isolation changed. Original result remains correctly Blocked.

Ancestry now stops at the wrapper root even when the checker itself is selected.
An executable PermissionError is retained outside invocation scope only if readable
command metadata has no invocation-root reference, process name is not in the
build/runtime actor set and valid process start ticks strictly predate the wrapper.
No named-service exception is used. Records retain PID, name, start ticks, wrapper
start and original error. Any actor/root-reference/new/unknown-start process still
Blocks on unreadability. Missing observer identity itself Blocks.

This defines a cooperative invocation boundary; it is not a same-user adversarial
security guarantee against preexisting processes changing their executable state.
No applicable scoped survivor is ignored, no build collector is excluded, and no
cleanup delay/kill/privilege workaround is introduced. Runtime PID/start/UID,
ancestry/cgroup/display and child requirements remain unchanged.

## New disabled packet and checks

`artifacts/ubuntu-managed-implementation/ubuntu-disabled-observer-correction-20261025/`.
Receipt SHA-256:
`7a8450f4ccd5d1f1b39b6ecd10fb8e2fa87d319824ae9a439f8fb70a1843a4c3`.
Same 544 source paths, 271 input paths and ten payloads. Compared with the previous
implementation, only runtime helper, runtime tests and architecture documentation
change, plus their archive/hash packaging. Accepted prototype and first implementation
remain immutable. Previous frozen logs/check summaries are retained with
`pre-correction-` names.

Twenty-four frozen tests pass (three contract, 21 runtime); 21 runtime checks pass
in WSL. Added tests reject broad exemptions for actors, root references, newer or
unknown-start processes, and verify checker ancestry stops at itself. No actual
Hyper-V corrected-adapter result is claimed yet. Positive gate remains false.

Run the same **non-GUI** guest launch script to collect evidence against this new
explicitly pinned revision, in the intended desktop terminal without sudo:

```sh
bash /mnt/j/coding/pivot/scripts/qualification/ubuntu-wrapper-checks.sh
```

This is a corrected implementation check, not replay of a GUI attempt or replacement
of the original Blocked evidence. Keep any new failure and return it for review.
Native originals and verified exports follow the prior handoff. No positive GUI
command, Ubuntu AOT, repeated sentinel or accepted-suite rerun. G3 held.
