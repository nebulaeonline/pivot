# Execution-safety milestone — native Mac reconciliation

2026-10-24. **Three-platform headless validation accepted; first safety milestone closed.**
[Parent acceptance](2026-10-24-execution-safety-acceptance.md). No guest isolation is approved. G2 remains accepted/closed;
G3 remains held and has not started during this infrastructure detour.
[Implementation](2026-10-24-execution-safety.md),
[policy and audit](../architecture/validation-execution-safety.md).

The [native Mac report](../../artifacts/macos-validation-20261024T181811Z/summary.md)
records native ARM64/no-Rosetta execution, local APFS source/target/temp/.NET outputs
and evidence acquisition under
`/private/tmp/pivot-execution-safety-20261024T181811Z._r4g3n3e/`.
Original source, binaries and fixtures remain there; SMB supplied source and
received reports. No upgrades, source/test/limit changes or reruns were reported.

| Headless gate | Windows | Native WSL Linux | Native Mac |
| --- | --- | --- | --- |
| Full CI steps | 13 passed | 13 passed | 13 passed |
| Default Rust tests | 347 | 348 | 348 |
| Execution-safety tests | 9 passed | 9 passed | 9 passed |
| Seven-project .NET build, parity, real IPC/session | Passed | Passed | Passed |

Mac raw logs confirm zero failed/ignored Rust tests and all nine policy tests.
Cleanup reports no owned probe/helper survivors; ten application-session charge
fields are zero. Synthetic consent only exercises stubbed CI routing and the
standalone console probe linking the actual guard; it does not launch Avalonia.
No real GUI, clipboard sentinel, VM or Native AOT gate ran on Mac. The Windows
warning-free AOT publication/refusal evidence remains a separate, limited check,
not positive graphical acceptance.

## Independent verification and source qualification

Windows independently verified all **366 APFS-origin evidence hashes**, the
manifest hash and transfer-receipt checksum. All **247 frozen code/build inputs**
match the original safety manifest and current code. The Mac used the original
**494-entry source manifest**, copied 495 files including the ignored GPT baseline,
and reports an unchanged complete local tree before/after the run. Native binaries
were not returned and were not independently rehashed on Windows.

The shared tree changed during the Mac run: **seven documentation edits and one
added handoff**. Their exact old/new hashes match the previously recorded
`artifacts/validation-safety/g2-acceptance-doc-update.json`: accepting/closing G2
and explicitly holding G3 for infrastructure. This is a reconciled documentation
qualification, not an unchanged complete shared-tree claim. No implementation input
changed, no source was recopied, no tests were rerun and no frozen expected manifest
was regenerated. These new review-documentation edits are likewise outside the
tested local snapshot.

[Independent verification receipt](../../artifacts/validation-safety/macos-review-verification.json)
and [review script](../../artifacts/validation-safety/review_macos.py) preserve the
checks. The original Mac summary explicitly reports the shared drift; retain it,
the old safety source/evidence manifests and the earlier Windows/Linux attempts.
Do not rewrite them to remove their historical pending statuses or failures.

## Review boundary

The safety milestone now has source-matched headless evidence on the existing
Windows/Linux/native Mac platform families. It preserves VM-independent contributor
CI, deliberate local interactive desktop approval and refusal of unattended GUI
without validated isolation. It does **not** certify the four provisioned guests,
host clipboard separation, unattended GUI acceptance or expanded architectures.

The parent accepts this reconciliation with the implementation handoff and
policy/audit and closes the safety gate. Guest qualification is the next bounded
deliverable; it has not begun. Host-side redirection settings, guest/session/runtime verification
and a deliberately scheduled harmless clipboard sentinel check remain prerequisites
for future guest approval. No dispatch, provisioning, VM lifecycle automation,
package installation, G3 experiment or production history work begins here.
