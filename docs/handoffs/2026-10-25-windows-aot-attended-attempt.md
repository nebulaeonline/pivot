# One Windows x64 attended Native AOT attempt

Update: [single attempt completed Passed/0](2026-10-25-windows-native-aot-result.md).
Authorization consumed; do not reuse the command. Final parent acceptance pending.
Instructions below retain their original pre-execution context.

Prepared under the [parent's one-attempt authorization](2026-10-25-windows-aot-candidate-acceptance.md).
Execution pending. No publish or GUI was launched during command preparation.

## Owner instructions

Inside **Pivot-Win11-x64**, using its VMConnect **Basic Session**, open ordinary
Command Prompt. Confirm that the intended VM and previously reviewed resource and
isolation settings are unchanged. No new clipboard sentinel is requested.

Run once:

```cmd
J:\coding\pivot\artifacts\windows-native-aot-candidate\run-one-windows-aot.cmd
```

The command initializes the existing VS 2022 Community Hostx64/x64 environment,
checks every pinned import hash and copies the accepted packet to a fresh directory
under C:\PivotWork. The unchanged wrapper verifies guest/context and tools.
Type `CONFIRM` only when its stated current conditions are true. The original GUI
runner then asks separately for consent; respond interactively if you agree.
No responses are supplied by the command. Allow the full publish and application/
peer workflow to finish; this is not a managed GUI rerun.

**Run once only.** Any Blocked, Failed, consent refusal, interruption or export
failure ends this authorization. Do not rerun or delete the local attempt receipt.
Report completion even if the command fails; no manual transcription is needed
when evidence returns successfully. Copy-only export recovery is allowed.

The bootstrap exclusively creates
`C:\PivotWork\windows-aot-attended-attempt-20261025.json` immediately before its
single launcher invocation, preventing accidental replay through this command.
This local record is not a GUI permit, consent or security boundary. It does not
change the accepted packet or authorize any second attempt.

## Final verification and evidence

`artifacts/windows-native-aot-candidate/final-read-only-verification.json` records
final independent reconciliation: receipt
`6989a33a4438fc049a67c6f9ae09f65f3f67c28d1b5ebb1d7bf515669163b7ec`,
fifteen packet files, 537 archived source entries, 265 inputs and exactly the
accepted AOT false-to-true line. No frozen file was changed or regenerated.
Receipt scope/approval metadata retains disabled-stage provenance, as in the
explicitly accepted packet; the verified executable flag is True. This historical
metadata is not an execution permission. Parent authorization governs this attempt.

Bootstrap/command packaging remains outside the immutable packet. It invokes only
the accepted fixed launcher with its pinned receipt and `--run-aot`, inheriting the
real terminal. No command/RID selector, consent injection or managed fallback.

All source/build/cache/publish/temp/fixture/original reports stay on native NTFS.
The unchanged launcher returns verified reports automatically under
`artifacts/windows-native-aot/results/Pivot-Win11-x64/aot-capture-<id>/`.
Native capture originals remain under `C:\PivotWork\aot-capture-<id>`; the console
also identifies the underlying native preflight report. Preserve both on failure.

The next review must inspect warning-free publish, actual published native image
and three native peers, independent backend/session/mode/path/charge reports,
adjacent x64 Rust child, WB5 integrity, source integrity, strict scoped survivors,
all original logs/exit codes and origin/transfer hashes. Publish success alone is
insufficient. No retries, assertion changes, extra cleanup waits or unrelated kills.

Windows managed/headless gates remain closed. Ubuntu GUI/AOT, ARM64/Rosetta and
unattended execution remain separate. G3 held; no product changes.
