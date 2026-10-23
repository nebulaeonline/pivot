# Four-direction Fill — design review packet

Historical design packet. The bounded implementation is now in place; see
[implementation and evidence](2026-10-23-four-direction-fill-implementation.md).

Date: 2026-10-23. **Design approved with implementation clarifications; no implementation or passing Fill evidence claimed.**
[Parent approval and implementation boundary](2026-10-23-four-direction-fill-design-review.md).
[ADR-0022](../adr/0022-four-direction-rectangular-fill.md).
[Accepted predecessor and follow-up discussion](2026-10-23-spreadsheet-aggregate-final-review.md).

## Approved design decisions

Approve command-driven Up/Down/Left/Right, original source-edge capture, direct
per-destination translation and one cancel-aware atomic transaction. Retain all
current capacities: for real fills, selected area (source included) at most 256.
Approve 128 KiB aggregate source/text work, 16,384 tokens and 8,192 references using
existing copy budgets, charging repeated source/translation work conservatively.
One-row vertical and one-column horizontal requests are no-ops after authority,
shape, bounds and cancellation checks, without the work-area quota.

Approve typed blank/decimal/text/rejected-draft duplication and existing bound
reference policies. Preserve same-document structured output identity/quotas;
exclude all projected records. Preserve unchanged-definition no-ops without identity
churn. Out-of-bounds translation, cycles, limits and cancellation reject atomically.

Approve an additive typed private Fill request, preserving v4 only if unsupported
opcodes reject without mutation; WB5/CB2/hosted v1 unchanged. Approve accessible
Fill menu plus scoped primary+D/Shift+D/R/Shift+R shortcuts, disabled during cell
editing/pending mutation/drag, with selection orientation and active cell preserved.
This avoids a separate draft commit disguised as part of Fill.

No series/drag handles, history implementation, formatting, extra functions or
capacity changes. A separate Undo/Redo design follows this bounded feature.

## Inspection findings and implementation approach

`crates/pivot-workbook/src/clipboard.rs` already exports effective scalar/range
coordinates and translates relative/absolute references, preserves rejected drafts,
accounts bounded planning work and commits paste through `apply_cancelled`.
`crates/pivot-workbook/src/structured.rs::duplicate_structured_block` already handles
mixed blank/numeric/text/ordinary/structured content and preserves output bindings,
but currently plans one equal-sized destination rectangle and uses non-cancel-aware
apply. It cannot be called repeatedly to produce an atomic Fill. Factor its trusted
planning into internal shared helpers, supporting source-to-destination pairs,
coherent capture, conservative budgets and one final cancel-aware apply.

The private session already validates revision/computational context. Propose
FILL=31 and field28 for a direction enum; validate the old-reader unsupported-opcode
behavior rather than assuming additive protobuf means semantic compatibility.
C# transports direction/selection/state and renders results; Rust owns all planning,
reference translation, quota decisions and mutation. No OS clipboard intermediary.

## Acceptance matrix for approved implementation

| Area | Required evidence |
| --- | --- |
| Four directions | Multi-row/multi-column source edges, reversed anchor/active selection, positive/negative offsets; source edge unchanged |
| Origin independence | Each destination derives from original edge, not previous destination; identical results under alternate planner enumeration |
| Content | Explicit blank/unallocated blank clears; unallocated blank no materialization; signed/scaled decimal preserved; exact literal text/apostrophes; rejected drafts remain non-executable |
| Scalar bindings | Relative/mixed/absolute flags; moved source origin and moved target effective position; retired/unplaced target rejects entire operation |
| Ranges | All five functions; operation identity, mixed dollar flags and reversed endpoints; translated geometry and invalidation; no stale SUM cache reuse |
| No-ops | Single-cell and degenerate direction; repeated identical Fill; no revision/dirty/checkpoint or identity/selection-ledger churn; blank and explicit zero remain distinct |
| Failures | First and final destination invalid; underflow/overflow; cycles; memory/work/live/allocated/selection/structured quotas; stale revision/context; no partial installation or leaked IDs |
| Limits | Selected area256 accepted when other quotas permit,257 rejected even mostly blank; zero-destination large axis no-op; source/token/reference bounds and repeated-source charges |
| Structured proof | Trusted mixed ordinary/structured duplication, ProducerId/OutputId preserved under rename/retirement; A1 translates, no source-name re-resolution or quota expansion; non-savable remains |
| Projections | Bands inserted inside visual selection do not become logical sources/destinations; read-only projected selection rejected; producer/layout unchanged |
| Cancellation | Pre-cancel and during planning/apply; roots/checkpoints/dirty/ledgers intact; dispatched caller cancellation drains response with no replay |
| Captures/resources | Old captures immutable; unchanged source publication state; scratch released on all paths; drop/finish zero-charge receipts |
| Session | Real Rust child; typed direction and exact state; malformed/unknown direction and unsupported opcode rejected; no legacy adapter or silent fallback |
| Desktop | Routed selection/menu/shortcuts for four directions; suppress extra letters; active-editor/projection/drag/pending safeguards; preserve selection/focus/scroll; OS clipboard unchanged |
| Persistence | Ordinary Fill WB5 Save/Open retains definitions, CellIds/bindings, blanks and checkpoints; failed Fill leaves path/saved generation/dirty state intact |
| Final gates | Windows/Linux/Mac managed desktop, strict all-feature and actual Native AOT; matching source/build inputs; twelve fields including new Fill plus previous eleven; zero-charge main/peers |

Use targeted invariants rather than a duplicate evaluator or tests mirroring loops.
No numerical oracle changes are needed: accepted aggregate/decimal suites remain.
Automation is the routine gate; optional human inspection does not become a repeated
three-platform manual checklist. Retain failed attempts and unchanged reruns.
Mac evidence transfer should retain per-file SHA-256 values from APFS originals so
Windows can independently repeat comparison, improving the accepted ADR-0021 receipt
limitation without rerunning that closed milestone.

## Proposed implementation sequence and stop points

1. Parent design review complete: approval with three clarifications recorded in
   the linked review. No additional design cycle required; implementation not started.
2. Shared Rust duplication planning plus focused Fill/atomicity/accounting tests.
3. Typed private-session/client request and real child-process boundary tests.
4. Accessible grid commands and a distinct automated desktop Fill check, retaining
   all existing coverage. No new recurring manual gate.
5. Final Windows/Linux gates and a fresh Mac/APFS packet, then parent implementation
   acceptance. Keep manifests and returned hash receipts concrete and source matched.
6. Only afterward, separate Undo/Redo architecture design: transaction history
   identity, root ownership/captures, quotas/eviction, dirty checkpoints, branches,
   failed transactions, save-in-flight and separation of document history from source
   publications/presentation state. This list is future design scope, not settled
   history semantics or permission to implement it now.

ADR-0021 remains accepted/closed. Known Windows clipboard, WSLg startup,
Windows-mounted Save uncertainty and historical Linux viewport p95 issues remain
visible; this design does not diagnose, close or broaden those issues.
