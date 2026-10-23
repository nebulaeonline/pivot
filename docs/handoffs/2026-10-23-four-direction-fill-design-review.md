# ADR-0022 design review — approved with implementation clarifications

Date: 2026-10-23. **Parent disposition: design approved; bounded implementation
authorized.** No further design cycle is required. This is not implementation
acceptance or passing Fill evidence.
[Accepted decision](../adr/0022-four-direction-rectangular-fill.md),
[acceptance matrix](2026-10-23-four-direction-fill-design.md).
ADR-0021 remains accepted and closed.

## Approved scope

Command-driven Up/Down/Left/Right duplicates original source-edge definitions and
bindings into aligned destinations in one atomic document mutation. No intermediate
destination becomes a source. Preserve all typed content, rejected non-executable
drafts, effective scalar coordinates, dollar flags, range kinds and structured
producer/output identities. No name re-resolution or OS clipboard intermediary.
Projected records/chrome never participate as worksheet positions.

For operations with destinations, admit at most 256 selected positions including
the source edge; retain source/text, token/reference, memory and experiment limits.
Do not chunk, truncate or partially install. Degenerate directions return validated,
non-enumerating no-ops; malformed, out-of-bounds and stale requests still reject.

## Clarifications governing implementation

1. **No-op before identity allocation.** Compare intended authoritative typed
   definitions, source and effective bindings before creating SelectionIds/graph
   identities. Repeated identical Fill must preserve node/range identities, ledgers,
   revision, checkpoint and dirty state. Equal results alone are insufficient;
   Blank and explicit zero remain distinct.
2. **Blank destination identity.** Clearing populated destinations preserves their
   CellIds, including when referenced elsewhere. Blank into unallocated blank must
   not materialize cells. Include COUNT/AVERAGE regressions for explicit zero becoming
   Blank while scalar zero behavior remains unchanged.
3. **Cancellation and acknowledgement.** Poll capture, planning and atomic apply.
   Before installation, cancellation changes nothing. After installation, reconcile
   the committed result even if the caller cancels. Do not claim caller cancellation
   implies rollback and do not automatically replay ambiguous requests.

A failure at any destination, including the last, must preserve roots, CellIds,
identity ledgers, revision, dirty state, path and saved checkpoint. Shared trusted
planning and one cancel-aware apply are required, not repeated mutating duplicates.

## Session, desktop and validation

Typed request carries direction, selection and exact expected state. Retain v4 only
after actual older-peer unsupported-opcode rejection is tested without side effects;
a genuine incompatibility requires separate version-cut review. WB5/CB2/hosted v1
remain unchanged.

Approve visible accessible Fill submenu and grid-scoped Ctrl/Cmd+D, Shift+D, R and
Shift+R. Disable during editing, pending mutation, drag or projected selection.
Preserve active cell, selection, scroll and focus; errors must be actionable.
Do not intercept editor/unrelated-control input.

Targeted Rust, real session/C#–Rust boundary and routed grid/menu/shortcut tests
must cover the design matrix and clarifications. Retain WB5 Save/Open, clipboard and
all existing regressions. Final managed CI, strict all-feature checks and actual
Native AOT are required on Windows/Linux/native Mac/APFS, with eleven previous
fields plus distinct Fill acceptance, matching source/build provenance, warning-free
native publication and zero-charge main/peers. Retain individual APFS-original
transfer hashes where practical for independent return comparison.

## Stop point

After implementation and three-platform validation, stop for parent implementation
review. No series, fill handles, history, formatting, extra functions, structural
edits, persistent structured data, new clipboard formats or capacity increases.
Only after Fill acceptance begin a separate Undo/Redo architecture design; immutable
calculation captures are not document mutation history. One Fill is one future Undo.

This record captures the parent's approval before implementation. The bounded
implementation is now in place; [current evidence](2026-10-23-four-direction-fill-implementation.md).
