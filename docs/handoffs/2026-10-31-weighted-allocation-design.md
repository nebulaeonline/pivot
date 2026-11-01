# ADR-0037 — weighted allocation design handoff

2026-10-31. Design approved with four clarifications; implementation not started.
No feature implementation, new benchmark or platform validation has run.

Read the [ADR](../adr/0037-weighted-conservation-preserving-allocation.md) for the
complete approved contract and required implementation proofs. ADR-0036 remains accepted and closed;
ADR-0023 remains parked at G3.

## Approved first version

`ALLOCATEWEIGHTED(total, recipient_index, decimal_places, weight1, ..., weightN)`

All operands are strict Decimal96. Accept 1..64 nonnegative scalar weights with at
least one positive weight, explicit precision 0..28 and a one-based ordinal index.
Weights can be literals, references or scalar expressions. No bare range, implicit
intersection, filtered membership, array/spill or hidden group. The 64-weight cap
and scalar-list tradeoff are approved; large ordered-range or
keyed-group support is deferred rather than approximated.

10 at precision 2 with weights (1,2,3) yields 1.67, 3.33 and 5. Weights need not sum
to 1 or 100. Zero weights receive zero; negative weights and all-zero lists refuse.
Blank/String/Boolean operands are TypeMismatch. A nonintegral settlement total refuses
rather than changing the obligation. Negatives are symmetric; zero is normalized.

## Accepted decisions and qualifications

1. **Exact ranking.** Align weight scales with integer arithmetic; divide each exact
   product U*W by the shared denominator D. Rank exact remainders descending, ties by
   list ordinal ascending; award one residual unit to each of the first R recipients.
   No rounded division, float comparison, repeated-period fairness or hidden repair.
   Largest remainder is not monotone across totals: weights (5,3,1) produce (2,1,1)
   for total 4 but (3,2,0) for total 5 at whole-unit precision. This tradeoff is accepted and remains documented.
2. **Explicit association.** Index j selects the jth weight argument, not a worksheet
   row or label. Ordinary reference binding/translation applies. Sort, Copy/Fill and
   structure can change intended record associations; none renumber or repair them.
   Conservation requires the same ordered specification and every index exactly once.
3. **Group-wide admission and actual SUM.** Rank first and validate all actually used
   final shares, including zero-weight recipients in the group decision. Do not test
   unused hypothetical lower/higher shares. For normalized nonzero shares at common
   scale t, require (U−minimum_positive_settled_units)/10^(p−t) to fit 96 bits exactly.
   All-zero groups are a special zero case. Ignoring zero shares in this bound is
   essential: (1,1,1,0) at total 10/precision 28 must retain the accepted three-share
   result plus zero. Full totals may normalize. Production SUM in multiple orders,
   including inserted zeros, is mandatory; no ordinary SUM change is proposed.
4. **Bounded work.** N<=64 gives weight sums below 2^196 and products below 2^380.
   Accept seven-limb denominator/remainders, twelve-limb products and six-limb units,
   fixed 384-bit long division and bounded insertion ranking. Explicit helper scratch
   is at most 16 KiB; approved limb-work bound is 4,096+16,384*N. Unlike ALLOCATE,
   charge 4+16*N existing evaluation visits before numeric work, reserve scratch through
   existing memory admission, and checkpoint cancellation inside division and ranking.
   No global limit increase or unmetered backend is allowed. Layout/work accounting
   and performance require proof; failed bounds return for review.
5. **Integration.** Approve opcode 20 and arity 4..67 in the generic typed-call encoding;
   WB12/private v13/hosted v1/CB5/CB6 appear sufficient. Verify all codecs/entry points;
   return for review if an encoding change is actually needed. No Desktop interaction
   or licensing-enforcement change belongs here.

## Planned evidence and scope

Independent Python Fraction and .NET BigInteger references must check bounded exhaustive
cases, every recipient, extreme/mixed-scale inputs, exact entitlement/remainders,
zero handling, equal-weight ALLOCATE equivalence and deterministic ties. Exercise
actual SUM and the new nonzero-subset guard; preserve strict errors and lazy demand.
Include real session/capture/copy/fill/sort/structure/persistence/filter integration,
changed weight association, malformed tokens, operational refusal/cancellation,
scratch admission and zero final runtime charge.

Declare 16,384-base-cell corpora with a 64-recipient group and SUM plus 256 calls
with 64 weights, five modes, four changed-input samples and four retained captures.
Keep request limits and the original one-second/256 MiB steady/512 MiB peak objectives;
report helper/core timings and charged memory without percentile/RSS/GUI claims.

Require final local CI, strict Clippy, all-feature Rust tests and headless CI/parity on
all six targets; macOS x64 remains Rosetta. Numeric-only headless qualification is approved;
reassess targeted coverage if metering integration changes sensitive paths. No new attended GUI/AOT round is proposed
unless sensitive paths change. No tests, builds or VMs were run for this design.
Continue verified archival/pruning and final per-project diagnostic cloc at implementation.

## Four mandatory implementation clarifications

- Recipient index is the weight-list ordinal, with ordinary references and no implicit
  row/record association or repair after Copy/Fill, sort or structure.
- Rank exact quotas first; only actually awarded final shares and the nonzero-subset
  SUM guard determine representation. All valid indices share admission. Test actual
  SUM with zero terms before, among and after nonzero terms.
- Reserve work and scratch before numeric execution on every entry path, including
  structured expressions and direct typed callers. Prove storage/limb bounds on
  success and failure, check cancellation within expensive loops, release charges
  on refusal, and preserve workbook state. No uncharged fallback or quota expansion.
- Enforce 1..64 weights and arity 4..67 consistently across parser, lowering, typed
  tokens, codecs, restoration and evaluator. Other formula/resource limits still apply.

The design is approved with these clarifications; implementation remains unstarted.
No range-valued weights, arrays, keyed groups, repair, fairness state, Desktop licensing
implementation or history work is included. See the ADR's design disposition for the
accepted validation scope and resource qualifications.
