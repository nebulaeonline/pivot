# Boot-corrected Windows managed confirmation — one attended run

Execution complete: [application/backend/continuity checks pass, overall Blocked
for a build-services collector survivor](2026-10-25-boot-corrected-managed-confirmation-results.md).
Do not replay the command below; its one-run authorization is consumed.

2026-10-25. [New parent authorization](2026-10-25-boot-continuity-guard-acceptance.md).
Final read-only verification passed for the exact approved candidate, eleven packet
files, 533 source entries and 262 inputs. Reviewed source delta, retained peer fix,
existing UI assertions and consent bytes verify. No test gate was replayed.
[Final verification receipt](../../artifacts/windows-boot-guard-implementation/final-execution-check.json).

Inside Pivot-Win11-x64, ordinary **Command Prompt**:

```bat
J:\coding\pivot\artifacts\windows-boot-guard-implementation\run-windows-managed-confirmation.cmd
```

This loads the existing x64 developer environment and imports the immutable packet
into a fresh native-local directory
`C:\PivotWork\managed-input\windows-boot-continuity-confirmation-20261025`.
It invokes only the fixed local bootstrap with pinned receipt
`905350c373552132becc0547ef467e368d690033f0bab416465e9909752702e0`.
An existing destination refuses; do not overwrite it or replay a prior command.

The production event/CIM/uptime and guest/session/desktop checks must pass at the
actual invocation. Type **CONFIRM** only when current Basic Session, intended
host/Hyper-V VM binding and reviewed resources/isolation remain unchanged. The
unchanged GUI runner then independently asks for affirmative consent; answer
**y** only when ready for windows/focus/clipboard work. Neither response is injected
or reused. A continuity/context refusal must be preserved, not bypassed.

This new authorization permits **one full managed confirmation**, retaining all
functional assertions, actual main/three peers, independent backend/session
reports and zero-charge checks. It does not grant Native AOT, Linux GUI or
unattended initiation. No manual sentinel absent a concrete relevant change or
contrary observation. Owner stops if connection/resource conditions change.

Source, Cargo/.NET outputs, caches, temp/persistence fixtures and native originals
stay on guest-local NTFS. J: is explicit import/export only. Reports export to a
fresh directory under `artifacts/attended-guest-preflight/results/Pivot-Win11-x64/`.
Python tee capture may omit native build output; report any useful visible error.
Original application/native reports and transfer hashes are retained independently.

Tell the repository-side agent when finished. If failure/refusal/interruption or
transfer error occurs, stop and report it rather than rerunning. Copy-only recovery
can return existing evidence without replay. Keep first outcomes, scoped survivors
and Failed/Blocked distinctions; no unrelated process kills or weakened assertions.
Historical packets/Blocked records are unchanged. Return evidence for final managed
acceptance before Native AOT. No host fallback. G3 remains held.
