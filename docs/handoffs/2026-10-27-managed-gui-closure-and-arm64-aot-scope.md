# Managed GUI closure and Windows ARM64 AOT investigation

Parent disposition, 2026-10-27: accept and close six-RID managed-GUI validation;
accept five Native AOT results, leaving the AOT milestone open at 5/6.

Accepted managed matrix:
`artifacts/validation-matrix/20261027T025246Z-deb1324f/`.
AOT matrix with five accepted passes and Windows ARM64 Failed:
`artifacts/validation-matrix/20261027T025246Z-b82c5b95/`.
Headless stays accepted and closed. No tests were rerun to record this review.

Parent accepts the existing attended activation/collection architecture, live
consent, real backends and architecture reporting, four zero final charges and
owned-process cleanup. The fixed native-Mac desktop lock remains required across
ARM64/Rosetta and independent invocations. ROBPC remains controller-only.

## Focused next assignment

Investigate `MainWindow.ClipboardChecklistAsync` in the actual Windows ARM64 AOT
failure. Native publication and ARM64 graphical launch succeeded; its failed JSON
report overrides process exit code 0. The available trace does not identify a
source line or clipboard sub-step. Missing `rectangular_fill` success data is a
consequence of the earlier abort, not evidence of a Fill defect.

Inspect nullable operands across clipboard operations and awaited continuations.
Compare the historical Windows null-destination issue without assuming the same
cause. Prefer source-level diagnosis or a narrow non-GUI regression. If needed,
add minimal checkpoints recording sub-step begin/completion and non-sensitive
object/state availability; do not record clipboard contents or workbook data.

Any necessary live reproduction is Windows ARM64 AOT only, with existing explicit
owner consent. Retain the original report, publish log and first stack. Establish
the cause before implementing the smallest correction. Preserve clipboard, Fill,
workbook, lifecycle and zero-charge assertions. No arbitrary sleeps, added
null-forgiving operators, broad exception suppression, success defaults or
retry-to-green loops.

After correction, run the original full Windows ARM64 AOT acceptance suite:
actual native publication/execution, main and all peers, ARM64 architecture,
real backend, four zero charges and no owned survivors. Shared production changes
require appropriate focused managed/AOT checks on affected architectures; do not
rerun the other five accepted targets without such a reason. Bring any successful
final six-RID AOT result back for closure review.

## Boundaries

Keep native-local build roots, existing SSH/attended workflows, ordinary logs and
verified collection. No forensic process inventories, immutable qualification
packets, per-run parent approval rituals, unrelated features, workbook/graph
redesign, persistence/protocol changes or product-limit changes. Historical
blocked/failed reports remain unchanged. Owner suggestions remain a separate
discussion. G1/G2 closed; G3 held until current validation integration completes.
