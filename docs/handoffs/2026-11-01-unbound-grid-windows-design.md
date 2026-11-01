# Unbound Grid Windows — detailed design handoff

Date: 2026-11-01.
Disposition: **Design approved with three mandatory clarifications incorporated.**
Requested next step: **Confirmation of the revised exact contract; implementation remains unauthorized.**

## Revised contract for confirmation

1. Selection is client-only. There is no server-side select-scope command or retained
   active scope; every scope-dependent request identifies its GridScopeId and expected
   state. Delayed replies remain qualified by their original scope/generation.
2. `capture_scope` returns ScopeCapture, a bounded shared view of axes, sparse index,
   formatting and visibility over the same canonical immutable workbook calculation
   generation. Cross-scope dependencies use that generation; no isolated graph,
   separate calculation cache or independently published Window revision is created.
3. Declared dimensions do not allocate per-position identities. Empty Windows use
   compact axis descriptors. Actual namespace/run allocations, positioned/demanded
   cells, graph nodes and retained retired authority are admitted transactionally
   across the workbook under existing limits, including simultaneous retained roots.

The accepted WB14/private v14 cuts remain subject to the normal implementation codec
audit. The six-target qualification plan remains unchanged. Subsequent numbered
milestones mentioned in review are proposals, not assigned or started work.

[ADR-0040](../adr/0040-unbound-grid-windows.md) specifies the first implementation
scope of accepted ADR-0039. It creates real workbook-owned unbound Grid Windows,
not hidden worksheets or document page regions.

The concrete change is a tagged `GridScopeId` throughout cell references, directories,
position/format/filter indexes, ranges, structural commands, transfer and private
requests. Worksheet convenience APIs remain wrappers. Windows own sparse axes/cells
in the same canonical graph and allocation domain; quotas are aggregate, not per
Window. A Window is independent of its editor and future placements.

Review these decisions:

- `!` and `|` resolve separate namespaces; quoted names, copy flags, original source
  and bound authority survive rename/reopen. One graph enforces cross-scope cycles.
- Deletion atomically retires owned anchors, faults inbound references and preserves
  old captures. Referenced scope tombstones persist; name/address reuse cannot repair.
- Proposed WB14-only and private v14 cuts encode scope tags explicitly, with WB13
  refusal and no migration reader. Hosted v1 is unchanged.
- CB5/CB6 do not carry Window authority. Unsupported native Copy/Cut/paste refuses
  before mutation; explicit values-only transfer and server-owned copy are distinct
  paths. Cross-scope cut/move is deferred.
- The normal Grid command set requires owner-tagged formatting, structural, sorting,
  filtering and visibility paths. Partial owner audits are not acceptable promotion.

The ADR gives API shapes, native validation requirements, UI lifecycle and targeted
qualification cases. Implementation validation is planned across all six targets,
including managed/AOT GUI with short owner-run launchers, per-project LoC and verified
archival/cleanup. No implementation or validation was run for this design packet.

Bindings/producers, Text, section geometry, placements, document capture, invocation
and PDF remain outside this slice. The accepted ADR-0039 geometry usability gate and
other clarifications remain applicable to their later scopes; they are not claimed
complete here. ADR-0023 remains parked, and ADR-0038 cold-build qualifications remain.
