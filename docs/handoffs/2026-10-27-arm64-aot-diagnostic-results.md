# Windows ARM64 AOT diagnostic results

Current disposition (2026-10-27): six-RID headless, managed GUI and Native AOT
execution validation are accepted and closed. Clipboard reliability remains an
open issue; G3's scheduling hold is released. Pending/open/held statements below
retain their historical context. See [final acceptance](2026-10-27-six-rid-validation-acceptance.md).

Date: 2026-10-27. The latest targeted Windows ARM64 AOT run passed the full
original acceptance suite. Every designated RID now has a passing Native AOT
execution. This does not establish that the intermittent clipboard defect is
fixed; the AOT milestone remains open pending parent disposition.

## Evidence

- Original five-pass/one-failure matrix: `artifacts/validation-matrix/20261027T025246Z-b82c5b95/`.
- First diagnostic reproduction, Failed: `artifacts/validation-matrix/20261027T144402Z-a3ccd622/`.
- Handler-instrumented execution, Passed: `artifacts/validation-matrix/20261027T144914Z-cab4acb5/`.

The failed diagnostic run identified a missing F5 destination after numbered
step `03-paste`, the first native scalar/SUM translation check. This matches
the historical Windows failure's check location, without proving a common cause.

The subsequent run added optional metadata tracing inside the handler. It
recorded native clipboard bytes available at step 03, engine paste completion
and a refreshed grid. It passed the original clipboard, formula, Fill, workbook,
dirty-close and lifecycle checks. The expected shape-rejection scenario at
step 05 records an engine rejection; its atomic-rejection assertion passed.

The passed run published the real win-arm64 native executable without publish
warnings and executed the native main and all three peers. All four reports
identify ARM64 execution and `Avalonia.Win32.WindowImpl`; all four final Rust
charges are zero. Collection reports no surviving owned processes. The original
application assertions and published-image checks were retained.

Detailed passed evidence is under
`win-arm64/aot/20261027T144628315703Z/` within the new collection: `summary.json`,
`publish.log`, `desktop-ui/results.json`, the three peer reports, and
`desktop-ui/results.clipboard.log`.

## Interpretation and review request

Only diagnostics and explicit missing-cell failures were added. No production
clipboard behavior, retry policy or timing delay was changed. Diagnostic source
compiled on the ARM64 guest with zero warnings/errors before owner activation.
Both attended reproductions used the existing live consent and native NTFS work
roots. Neither original failure was overwritten or relabeled.

The passing trace cannot explain the failed read/paste path. Instrumentation
can affect scheduling, so its pass must not be presented as a correction. No
ARM64-specific compiler, trimming or timing cause has been established. We have
stopped after this informative run rather than repeating until green.

Parent review should distinguish the successful native-execution evidence from
the unresolved clipboard reliability issue and decide the remaining closure
condition. No further GUI runs or unrelated target reruns are prepared here.
Headless and managed GUI remain accepted/closed. G1/G2 remain closed; G3 held.
