# Managed GUI closure and Windows ARM64 AOT investigation

Current disposition (2026-10-27): six-RID headless, managed GUI and Native AOT
execution validation are accepted and closed. Clipboard reliability remains an
open issue; G3's scheduling hold is released. Pending/open/held statements below
retain their historical context. See [final acceptance](2026-10-27-six-rid-validation-acceptance.md).

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

## Investigation in progress

Source review found multiple destination-cell dereferences in the checklist.
`ClipboardAsync` can decline a stale or unavailable read and catches errors for
display in the UI; completing its task therefore does not prove a paste occurred.
The older Windows failure explicitly identified the first translated destination
check (then line 52). The ARM64 AOT offset-only trace cannot establish the same
location or cause.

Diagnostic-only changes add numbered copy/paste begin/completion checkpoints in
`results.clipboard.log`, recording state availability booleans without clipboard
contents, formulas or values. Required destination cells now fail explicitly with
their fixture coordinates and the last operation, preserving the existing value,
formula and lifecycle assertions. No production clipboard behavior was changed.

A fresh Windows ARM64 AOT plan is prepared at
`artifacts/validation-matrix/20261027T143855Z-9d71c7ac/`.
Its SSH preflight correctly reports NeedsGraphicalSession in session 0; no GUI
has been launched. The original failed run remains unchanged. This is a diagnostic
reproduction, not a correction or acceptance result. Use the existing attended
launcher in the Windows ARM64 desktop: `J:\coding\pivot\scripts\attend.cmd aot win-arm64`.

The deployed diagnostic source compiled on the Windows ARM64 guest with SDK
10.0.403: zero warnings and zero errors. `git diff --check` passed. A controller
build attempt returned failure without compiler diagnostics and is not target
evidence; no controller GUI was launched. The native AOT reproduction remains
pending owner activation.

### First diagnostic reproduction

Collected at `artifacts/validation-matrix/20261027T144402Z-a3ccd622/`:
native ARM64 publication and launch succeeded, but the checklist failed explicitly
because F5 was missing after `03-paste`. The numbered copy/paste steps all returned
with session/frame available and no editing, draft, file action or shutdown.
This matches the older failure's first translated destination check, without
establishing a common underlying cause. The original and diagnostic failures are
both retained; neither is an acceptance result.

Additional optional handler diagnostics distinguish unavailable clipboard
snapshots/bytes/text, copy/read invalidation, stale paste state, engine completion
or rejection and exception type. They emit metadata only through the checklist's
callback and preserve application behavior. The next targeted plan is
`artifacts/validation-matrix/20261027T144521Z-a9f2697a/`; owner activation uses the
same short command above. No fix has been claimed.
