# ADR-0013: One canonical persistent graph and workbook

Status: Accepted owner direction (2026-10-19); implementation validated on Windows/Linux/macOS; implementation review pending.
Supersedes the continuing reference-backend and opt-in workbook selection policies
in ADR-0011/ADR-0012, not their semantic or resource guarantees.

## Context and decision

Pivot is pre-release, with no deployed consumers requiring migration. Three-platform
readiness evidence supports choosing the persistent representation. Maintaining two
editable engines now adds complexity without a corresponding product requirement.

WorkingModel and Workbook always own the canonical persistent graph and worksheet
indexes. Their `new` constructors share the existing lazy convenience runtime;
`new_in` accepts the application-owned IntegrationRuntime. No worker pool is created
per convenience model/workbook. Resource caps and admission remain unchanged.

Remove reference constructors, backend enums, activation/rehome bridges, full-copy
worksheet staging, reference capture evaluation and backend-selection features.
`scaling-experiment` remains optional instrumentation/low-level measurement access,
not a backend selector. Keep the canonical private-index optimization; remove its
immutable-insertion comparison switch. Preserve ordinary functional index insertion
for shared/heap-owning data where required by accounting and failure semantics.

Explicit complete snapshots and owned portable definition images remain supported.
An image has no node authoring, transaction branching, runtime or selectable
backend. Complete validation and port authoring use one set of rules; immutable
snapshot execution reuses the decimal expression executor. Portable import builds
canonical pages directly from one checked image decoder without rebinding source.

Preserve stable identities, invalid drafts/repair, local versus complete validation,
atomic expected-state edits, immutable S/T, exact decimals, public ports, codecs,
Save/Open, receipts/replay, publication, IPC authentication and deterministic cleanup.
Changing a representation diagnostic does not authorize silent semantic changes.
Research graph clones now retain their construction history instead of pretending
construction occurred at commit zero; public per-operation revision rules remain.

## Consequences and validation

Host, storage, document and daemon paths construct canonical models. There is no
fallback backend or conversion step. Large authoring still requires worker scheduling
and bounded caller-owned buffers. Shared runtime admission is not an RSS ceiling.
No persistence provider, format, formula feature or UI/IPC expansion is included.

The final reference source is recoverable at
`3a6fd9719f491a9236dbf0197ed9d4fb7ade3e24`; retain historical benchmarks and fixture
specifications. Do not keep an archived duplicate source tree. Independent expected
results, transaction/lifecycle tests and a small workbook scalar oracle replace
full-engine comparison as the correctness basis. Canonical branch and immutable
snapshot comparisons additionally exercise ownership and capture consistency.

Require fresh Windows/Linux/native APFS macOS CI and Release workloads before
milestone acceptance. Earlier readiness results are comparison evidence only.
See [architecture](../architecture/canonical-engine-consolidation.md) and
[handoff](../handoffs/2026-10-19-canonical-engine-consolidation.md).

Fresh native APFS consolidation validation now passes on the same source (280
copied-file hashes matched before documentation edits), with 276 default / 292
all-feature / 62 canonical no-default-feature tests and 33 Release children.
Three-platform correctness and resource evidence is complete. Preserve the original
Linux viewport p95 miss and passing diagnostic repeat for owner review; native
success does not change that evidence or authorize a threshold increase.
