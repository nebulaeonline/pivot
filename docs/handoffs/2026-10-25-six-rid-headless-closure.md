# Six-RID headless closure and GUI/AOT scope

Date: 2026-10-25. Parent disposition: accept and close six-RID headless validation.
All six designated execution environments passed all 13 original gates, 78/78:
`artifacts/validation-matrix/20261026T002242Z-f76230c2/`.
ROBPC orchestrated only. No gates were rerun to record this acceptance.

Parent also accepts and closes the task-ticket release-ordering correction and
accepts both cancellation fixture corrections. The original shutdown branch was
not positively identified; the controlled before/after evidence proves the
ordering defect, not a reconstruction of the historical failure. Original failed
runs, counterexample logs and diagnostic history remain unchanged.

Private Linux x64 SDK10.0.403/runtime10.0.14 selection is accepted. The system
CoreCLR abort remains an unexplained reliability issue. Do not replace or modify
Ubuntu's system SDK as part of the next work.

The accepted scheduling policy has a global headless/discovery ceiling of six,
with a shared mac-host capacity of two across both UTM guests and both macOS RIDs.
Hyper-V jobs are independent. The final functional matrix used the earlier
global-two invocation. The new scheduler passed 14 focused tests and six-target
discovery; another complete CI run under that scheduler has not been performed.
No immediate headless rerun is required.

## Next authorized implementation

Integrate the original managed GUI runner with the unified profiles. Require a
real user graphical session, actual application/peer/backend execution, existing
functional assertions, four zero final charges and owned-child cleanup. SSH,
display variables or backend discovery alone do not prove a usable graphical
session. Provide short target-specific activation instructions where ordinary
SSH cannot reach the graphical session. Retain the existing live explicit consent;
do not introduce a replacement broker or preanswer prompts.

Native macOS ARM64 and Rosetta x64 share a desktop. Enforce a machine-wide
exclusive lock across their GUI runs, including independently started orchestrator
invocations. Per-run serial scheduling alone does not implement that requirement.
The accepted two-job Mac capacity concerns headless/discovery execution.

Integrate real Native AOT publish-and-execute next, preserving original application
and peer assertions, actual process architecture, explicit Rosetta labeling,
zero charges and owned-child cleanup. Discovery or publishing alone is not a pass.
Missing prerequisites must produce setup failures.

Keep headless, managed and AOT results separate in per-RID JSON/matrices. Use
ordinary Git revision, logs and exit codes. Individual GUI/AOT checks do not
require repeating all headless gates. Report what actually executed and any
missing session/toolchain prerequisites.

## Boundaries

No VM lifecycle automation, global security changes, Mac Remote Login Full Disk
Access changes, host-desktop fallback, unrelated-process cleanup, forensic process
inventories, immutable source packets or per-run parent approvals. No workbook,
graph, formula, persistence, protocol or product-limit changes.

G1/G2 remain accepted and closed. G3 stays held until validation infrastructure
integration is complete. G4–G7 and production Undo/Redo remain deferred.
