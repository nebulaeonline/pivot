# Application Slice A: parent review decision

Review received: 2026-10-20. Disposition: **Accept Slice A and ADR-0014**.

The review accepts the real Rust-backed Avalonia worksheet and owned C#–Rust
session, with actual Windows, WSLg and native macOS execution evidence. The
canonical engine architecture remains settled. The initial WSLg native
`free(): invalid pointer` / SIGABRT remains an unresolved startup-reliability
defect. If reproduced, obtain an actionable native backtrace before treating
Linux distribution as ready; later passes do not establish the cause.

## Next assignment: native workbook persistence and application lifecycle — Slice B

Make New, Open, Save, Save As and Close usable through the Rust-owned session.
Success means saving numbers/formulas, closing, reopening in a new process and
continuing with the same identities, stable bindings and calculation semantics.
This is the assigned next milestone; implementation has not begun.

- Define a clean, versioned, bounded workbook envelope and lifecycle ADR. Reuse
  portable graph encoding/validation, while also preserving workbook/worksheet
  identities, sparse positions, CellIds, axis metadata, original source/reference
  provenance, bound GraphNodeIds, retired identity ledgers, exact decimals,
  editable invalid drafts and relevant revision/checkpoint state. Exclude caches
  and internal allocation layouts. No speculative historical migrations.
- Restore graph and projection atomically after framing/version/length/resource
  validation. Restore historical references by identity, never by rebinding A1 text.
- Save one immutable coherent generation. Stage privately, protect existing valid
  files, detect external modification, acknowledge only the installed generation
  and retain dirty state after failure or newer edits. Distinguish definite failure,
  clean success and uncertain installation/recovery. Do not claim unsupported
  power-loss durability. Reuse file-safety groundwork where appropriate.
- Add minimal typed session operations and authoritative document identity,
  saved-generation and dirty-state reporting. Rust owns serialization and files;
  C# owns presentation. Preserve framing/admission bounds, expected-state checks,
  S/T isolation, cancellation/ambiguous-response rules and orderly cleanup.
- Add File actions, document name/unsaved status, actionable errors and confirmation
  before discarding work. Async saving must keep rendering responsive and must not
  mark a newer generation clean. Preserve existing edit/move/clear/scroll behavior.
- Exercise independent-process C#–Rust round trips for exact decimals, formulas,
  move/reuse, retired IDs, drafts/repair and reopened calculation. Cover saving S
  while editing T, save failure, external conflicts, malformed/truncated/oversized
  input, Save As existing/new destinations, cancellation/termination and zero
  remaining charge on orderly close. Validate NTFS, Linux disk storage and APFS.
- Measure coherent capture, export/encoding, write/install, read/decode, validated
  restoration and complete Save/Open separately. Use production-sized fixtures;
  any larger diagnostics remain bounded research evidence. Run full regression
  and executable application checks on Windows, isolated Linux and native macOS.

Keep production limits. No SQLite/hosted persistence, alternate/reference engine,
new computation framework, ranges/SUM, copy/fill, complex formatting, Excel
compatibility, million-cell increase, installer or AOT work is included.

Deliver the ADR, Rust codec/restoration, session/client/UI integration, cross-process
and failure evidence, performance record and Markdown handoff. Stop for review.

The review also suggests trying the application interactively before changing
35 ms viewport coalescing; this is not an instruction to optimize it now.

## Existing evidence

[Accepted implementation handoff](2026-10-19-application-worksheet-slice-a.md),
[ADR-0014](../adr/0014-application-session-and-worksheet.md),
[boundary/UI measurements](../performance/application-worksheet-slice-a.md).
