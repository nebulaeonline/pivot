# ADR-0032: Worksheet sorting — design review

Date: 2026-10-30.
Disposition: **Design approved with clarifications by parent review. No implementation.**

The parent accepts the design and future validation gates, with three mandatory
clarifications: completed cell calculation errors are sortable while operational
evaluation/capture failures abort; no-ops compare final identity placements and
explicit assignments; the final rectangle, partial-record warning and confirmed
expansion choice must be visible in the ordinary dialog. Retain WB11/CB5; private
v10 and the existing 33,554,432-unit work ceiling are approved. Separate owner
implementation authorization is required. No feature work or tests follow from
this documentation update.

The owner committed ADR-0031 (`43c0b8c`) and authorized sorting design. The full
[ADR-0032 proposal](../adr/0032-worksheet-sorting.md) compares identity movement,
fixed-identity content exchange and replayed Move/Copy before recommending one
atomic sparse row permutation within an explicit selected rectangle.

The accepted design decisions:

- Records move together within selected columns. Existing CellId/GraphNodeId values,
  formula bindings and source travel intact; scalar references follow their targets.
  No Copy-style translation, reparsing, allocation or retirement of identities.
- Rectangular ranges retain geometry and SelectionIds and read current occupants.
  Partial overlap can change membership/results, even when scalar references follow
  a moved record. The design includes concrete examples and range-selection guidance.
- Formula keys are evaluated once from the pre-sort immutable capture. Later
  recalculation may change them; no automatic repeated sorting. Equal key tuples
  retain original order. Optional first-row exclusion is explicit.
- Sort-only successful-kind order is Decimal, String, Boolean; descending reverses
  those comparisons. Errors then Blanks remain last in both directions. Errors tie;
  later key levels can distinguish tied Blank/error rows. Text is ordinal and
  case-sensitive. Empty String differs from Blank. Leading-zero Strings are never
  converted, normalized or truncated.
- Move formatting with records is an explicit default-on option. It permutes each
  property's assignment intent, including explicit defaults/inheritance and styled
  blanks. Unchecked keeps positions' formatting. Whole-column assignments naturally
  remain invariant; no historical row/column precedence is invented.
- One charged candidate publishes placements, graph membership and styles together.
  Existing memory, identity, selection and fragmentation limits remain. Sparse
  full-extent selections avoid area-sized storage. Style fragmentation can refuse
  atomically; it cannot discard formatting or silently switch policy.
- Retain unchanged WB11/CB5 because final state uses existing representations, and
  private v10 for the new command contract. No compatibility readers/migrations.

The ADR defines resource accounting, proposed performance gates, independent
permutation/formatting oracles, fault/capture/stale/cancel/persistence checks and a
future six-target managed/AOT GUI plan with short attended launchers. These are
future implementation gates, not measured sorting results. No tests or platform
commands were run for this design-only change.

The parent accepts the identity/reference, fixed-range, calculated-key, mixed-type
and formatting decisions with the clarifications above. ADR-0031 remains closed. ADR-0023 stays
parked at G3; filtering is only a later candidate, provisionally ADR-0033. Stop for
explicit owner implementation authorization under the standing
milestone cadence.
