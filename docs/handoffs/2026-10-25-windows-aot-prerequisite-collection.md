# Windows guest Native AOT prerequisite collection

Read-only collection ready; actual guest results pending. This implements the
[accepted preparation direction](2026-10-25-windows-aot-preparation-acceptance.md).
No native publish, GUI launch, managed rerun or installation is authorized.

Update: collection completed and independently reconciled; see the
[guest results and candidate submission](2026-10-25-windows-aot-guest-prerequisites-and-candidate.md).
The preparation instructions and packet below retain their original stage context.

## Owner command

In **Pivot-Win11-x64**, open ordinary Command Prompt and run:

```cmd
J:\coding\pivot\artifacts\windows-aot-prerequisite-qualification\collect-windows-aot-prerequisites.cmd
```

No administrator session, PowerShell policy change or manual output transcription
is needed. The command initializes the existing VS 2022 Community x64 tools,
verifies the frozen packet, copies to a fresh C:\PivotWork directory and runs
metadata queries only. Keep any failure; do not replay automatically.

Native originals remain under C:\PivotWork\aot-prerequisites-<id>\report.
Verified copies return to
`artifacts/windows-aot-prerequisite-qualification/results/Pivot-Win11-x64/`.
If export fails, retain the printed native location for copy-only recovery.

## Scope and evidence

Collector receipt SHA-256:
`124264381f467095b7ff7eb855e08d7ddce91dcfd75d7dddbc5c06a42dfe0c13`.
The nested accepted disabled packet remains byte-identical, receipt
`e74a9fee3c44874cf2823becf181b61b37d163b79ca83371f6b2ae946133ad64`.
Packaging verification and three passing mocked non-GUI tests are retained under
`artifacts/windows-aot-prerequisite-qualification/`. These establish collector
behavior, not guest capability. No guest run has occurred during preparation.

The report distinguishes Verified, Missing and Blocked, retaining raw prerequisite
status/reasons, actual guest identity/context, installed executable paths/hashes/PE
architecture/file versions, SDK and Rust metadata, VS component query, SDK/UCRT/MSVC
library paths/hashes and installed native Rust standard-library hashes. It verifies
source integrity and before/after context and exports origin hashes with a transfer
receipt. Compiler/linker tools are inspected, not invoked to compile or link.

After actual results, prepare the enabled candidate and exact delta for parent
review. Do not provide a positive AOT execution command before that review.
Windows managed/headless gates remain closed; no clipboard retest. G3 remains held.
