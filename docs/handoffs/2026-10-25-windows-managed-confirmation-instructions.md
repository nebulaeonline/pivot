# Windows managed confirmation — one authorized attended run

The first attempt [blocked before GUI startup on a 1.179 ms boot-timestamp mismatch](2026-10-25-windows-confirmation-timestamp-block.md).
Do not repeat the command below. It retains the original one-attempt instructions;
the guard discrepancy requires review before any further attempt.

2026-10-25. [Parent approval](2026-10-25-peer-backend-correction-acceptance.md).
Final read-only verification passed for the approved immutable packet: eight files,
530 source entries, 259 inputs and the exact two-line diagnostic correction plus
added lifecycle test. No tests or GUI suites were replayed during final checking.
[Final receipt](../../artifacts/windows-peer-backend-correction/final-confirmation-check.json).

Run from ordinary **Command Prompt inside Pivot-Win11-x64**, not the host:

```bat
J:\coding\pivot\artifacts\windows-peer-backend-correction\run-windows-confirmation.cmd
```

The command sets the existing x64 developer environment, imports the unchanged
approved candidate into a fresh native-local directory
`C:\PivotWork\managed-input\windows-managed-confirmation-20261025`, and executes
only its fixed bootstrap with receipt digest
`04aa8cd19f0f92a4064bda4464549b185dead13a54219fd50f16f7c56905bbbd`.
An existing destination refuses; do not remove it and repeat the command.

Type **CONFIRM** only if Basic Session, VM binding and resource/isolation settings
remain as reviewed. The managed branch checks the reviewed boot
2026-10-25T00:37:42.0920420-04:00 and session 1; a changed binding stops before
activation and must return for review. The unchanged UI runner then independently
requests GUI consent; answer **y** only when ready for windows/focus/clipboard use.
Neither response is supplied or reused by the wrapper. Do not repeat clipboard
sentinels without a specific changed condition or contrary observation.

This runs the full unchanged managed UI suite, including main window and three
independent peers, rather than a reduced backend test. Keep source/build/cache/
temp/persistence/original reports on guest-local NTFS. J: is import/export only.
Owner stops if connection/resources change. Native AOT is not included.

Results export automatically to fresh invocation directories under
`artifacts/attended-guest-preflight/results/Pivot-Win11-x64/`. Native originals
remain in C:\PivotWork. The original first-run Blocked capture is not overwritten.
The Python tee may not contain all native build output; preserve any useful visible
error, and use actual runner/application reports as evidence.

Tell the repository-side agent when finished. Do not rerun after failure, refusal,
interruption or transfer error; preserve the first result and report the issue.
Transfer problems can be repaired by copying existing evidence. No unrelated
process kills or assertion changes. Return all evidence for final managed acceptance
before Native AOT. No Linux GUI, unattended initiation or host fallback. G3 held.
