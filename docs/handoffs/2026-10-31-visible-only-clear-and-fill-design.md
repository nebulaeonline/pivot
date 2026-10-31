# ADR-0035 — visible-only Clear and Fill design handoff

2026-10-31. Design approved with clarifications by parent review. No feature work or
new native validation has run. Implementation awaits separate owner authorization.
ADR-0034 is accepted and closed; the owner authorized this design.

The parent approves the 4,096-slot Clear / 256-slot Fill contract and one combined
milestone with Clear-first/Fill-second internal slices, culminating in one final
implementation review, validation acceptance and owner commit/push boundary.

Three mandatory clarifications apply:

1. Canonical no-ops compare final definitions, identity state and explicit per-property
   assignments, not display equality, including Clear Both.
2. All Fill pairs and source values are fixed from the qualified pre-operation state;
   recalculation/filter refresh cannot change the current operation's sources/write set.
3. Clear Both and Fill content/styles stage privately and publish together once;
   no separate committed APPLY/FORMAT or per-cell/per-row sequence is acceptable.

All existing resource budgets, exact reviews including degenerate no-ops, one-use
state/interaction tickets, hidden-row integrity with permitted Blank-anchor demand,
private v13, unchanged WB12/hosted v1/CB5/CB6 and the six-target validation plan are
accepted. Preserve Rosetta/source qualifications, original failures, verified archives
and diagnostic per-project cloc. Do not raise the Clear limit because measurements pass.

Read [ADR-0035](../adr/0035-visible-only-clear-and-fill.md) for the complete contract.

## Accepted decisions

1. Keep ordinary active-cell Clear/Delete, rectangular Fill and shortcuts intact.
   Expose explicit Clear Visible and Fill Visible submenus, without a sticky mode.
2. Clear has three modes: Contents, Formatting (Built-in Defaults), and Both.
   Contents preserves formatting and stable positioned identities; absent cells stay
   absent. Formatting resets every property of every addressed visible cell to explicit
   built-in defaults, including individual-cell exceptions. It does not mean Inherit
   worksheet defaults. Both publishes content and formatting as one transaction.
3. Fill Down/Up uses the first/last visible selected row as its immutable source edge.
   Right/Left uses each visible row's left/right selected cell independently. Hidden
   geometric edges are never sources; no multi-row seed repetition or run restarts.
   All copies use actual-coordinate formula offsets and existing trusted Fill semantics,
   including styles, rejected drafts, binding provenance and candidate-cycle refusal.
4. Every operation reviews its exact actual row runs, source/destination edges, columns,
   counts and skipped rows. A charged one-use ticket and client interaction binding
   prevent stale plans, replay or silent replanning. Filter refresh follows publication,
   never changes the reviewed write vector halfway through it.
5. Approve **4,096 addressed visible slots for every Clear mode** and **256 slots including
   sources for Fill**. Clear's limit is sixteen times clipboard capacity and reflects
   different work: no source image, formula translation or clipboard interchange.
   It is a new command bound, not a styled-position storage cap. Existing full-sheet
   region formatting is unchanged. Reject oversized commands whole; no truncation,
   hidden fallback or multiple independently committed chunks.
6. Use **one design/milestone with two implementation slices: Clear first, Fill second**.
   Clear establishes qualification/review and combined content/reset atomicity; Fill
   then adds trusted duplication, source-edge rules and cycle checks. Recommend one
   final six-target validation/review and owner commit/push boundary. Do not infer
   separate acceptance or implementation permission for either slice.

The parent explicitly approves the initial 4,096-slot Clear ceiling: an unfiltered 65,536-row
column or a 10,000-cell selection would refuse. A sparse selection spanning full height
can qualify if its addressed visible area is at most 4,096. Supporting larger Clear
requires a measured command contract, not copying a clipboard threshold or assuming
region-only formatting costs also describe graph invalidation. All Clear modes use
the same initial bound. No capacity, graph, style or runtime budget is increased.

## Mandatory invariants

- No direct hidden-row content or formatting writes. Canonical formula binding may
  still demand ordinary hidden Blank reference anchors; normal hidden dependents may
  recalculate. Do not redefine reference meaning or calculation membership.
- All selected visible formatting is overwritten for Formatting/Both; no per-cell
  exceptions survive. Explicit built-in defaults remain distinct from inherited defaults.
- One immutable source capture and one canonical transaction. Clear Both and Fill's
  content/styles cannot publish separately. Failed/cancelled candidates preserve roots,
  identity ledgers, revision, dirty/checkpoint and filter definition.
- No new computational multi-area selection, sorting, aggregate behavior, tables,
  Undo/Redo, history, visible Cut/Move or series generation. ADR-0023 stays parked at G3.

## Integration and qualification

Approve private protocol v13 with separate typed Prepare Visible Edit, Clear Visible
and Fill Visible operations. Required typed ClearMode distinguishes Contents, Built-in
Formatting and Both; reuse FillDirection. Rust owns plans and mutation. C# presents
review and preserves interaction state; no formula parsing or edit fan-out. Preserve
APPLY's existing 256-edit cap; a dedicated bounded Clear seam handles the larger plan.

The core transaction seam must combine Clear edits and property-map resets before
installation. Existing independently publishing APPLY/FORMAT helpers cannot simply be
called in sequence. Use property-batched staging, accumulated normalization work and
existing style quotas; no per-row budget reset. Fill reuses trusted duplicate-pair
planning with exact rows rather than serializing through CB6. WB12, hosted v1 and
CB5/CB6 stay unchanged. Temporary plans/tickets are never persisted.

The ADR specifies independent row-mask/pair oracles, complete style reset versus
inheritance tests, exact typed/formula/fault behavior, hidden-row sentinels, identity/
no-op checks, stale-state and one-use ticket failures, late quota/cycle/cancellation
rollback, admission boundaries and current persistence/ordinary-command compatibility.
Proposed native measurements cover 4,096-slot Clear and 256-slot Fill with full-height
gaps, four captures, declared base/anchor populations and fragmented styles. Preserve
1-second engine-operation, 256-MiB steady and 512-MiB peak objectives on stated corpora;
report sample-counted observed maxima and separate planning/commit/refresh timings.

After design approval and owner authorization, qualify all six headless RIDs and
attended managed/AOT graphical suites, macOS x64 under Rosetta. Reuse short consented
launchers, x64 pilot first, host-serial timing, immutable original failures and batch
archive/hash-verification before pruning. Refresh per-project cloc at the implementation
milestone; do not treat line reduction as an objective. No repeat ADR-0034 validation
is requested by this design.

## Parent disposition and next step

Design approved with the three clarifications above. Implementation remains held
until the owner's separate authorization. No additional design packet or ADR split
is needed. Follow the owner-controlled implementation/review/docs/commit-push cadence;
ADR-0023 remains parked at G3. ALLOCATE remains an independent future proposal.
