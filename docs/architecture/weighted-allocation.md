# Weighted exact allocation

Accepted and closed under ADR-0037 by implementation review, 2026-11-01.

`ALLOCATEWEIGHTED(total, recipient_index, decimal_places, weight1, ..., weightN)`
returns one settled proportional share. There must be 1..64 explicitly ordered scalar
weights, all nonnegative, with at least one positive weight. Every operand must be
Decimal96; Blank, String and Boolean are type errors. Index is a one-based integer
within the list, and precision is a required integer 0..28. Weights are ratios and
need not sum to 1 or 100. Numeric trailing zeros do not change these meanings.

For weights 1,2,3, `=ALLOCATEWEIGHTED(10,1,2,1,2,3)` returns 1.67; indices 2 and 3
return 3.33 and 5. The complete consistent set sums exactly to 10. Negative totals
use the same magnitudes and residual winners with negative signs. Zero-weight shares
are zero; all computational zero is positive at scale zero. Cell formatting remains
separate. A total of 10.005 at precision 2 is invalid, never silently rounded.

## Recipient association

Index j selects weight argument j, not a worksheet row, label or durable business
record. Weights can be scalar expressions and ordinary/cross-sheet references, but
not bare ranges. An example for three weights is
`=ALLOCATEWEIGHTED($F$1,C2,2,$B$2,$B$3,$B$4)`. Normal binding and Copy/Fill translation
apply. Sorting or editing a range does not infer membership or repair indices. A sort
of weight cells alone can move their identities while leaving an allocation's ordered
references and explicit ordinal distinct from its current worksheet row.

Conservation requires the same total, precision and ordered weights, with every index
exactly once. Scalar formulas do not certify completeness or intended association.
Equal positive weights match ALLOCATE over the supported count domain. Largest-remainder
settlement does not promise monotonic shares across changed totals: weights (5,3,1)
give (2,1,1) for total 4 and (3,2,0) for total 5 at whole-unit precision. There is no
hidden fairness rotation, history, array/spill or keyed allocation group.

## Arithmetic and representation

Convert the absolute total to integer settlement units U. Align weight coefficients
exactly to a common input scale, giving integers W and denominator D=sum(W). For each
recipient divide U*W by D, retaining the floor and exact remainder. Award residual
units by descending remainder, breaking ties by ascending argument ordinal. This uses
integer arithmetic, not rounded Decimal division or floating-point comparisons.
Stored weight scales, including insignificant zeros, need not be normalized first;
exact scale alignment preserves ratios within the approved width/work bounds.

Ranking precedes Decimal96 narrowing. Only the actually awarded shares are validated;
an unused floor or ceiling cannot cause rejection. All final shares and the largest
proper subset of strictly nonzero shares must fit at their common normalized scale.
The full total may normalize further. Zero shares are ignored in the subset bound,
so adding zero recipients cannot falsely reject an otherwise admitted specification.
All valid indices receive the same group representation decision.

Wrong domains produce InvalidArgument, wrong scalar types TypeMismatch, and final-share
or exact-SUM representation failure arithmetic Overflow. Local validation follows
argument order before cross-argument admission. Dependency errors retain provenance.
IF/AND/OR laziness is unchanged. Cancellation, allocation refusal and work exhaustion
remain operational failures; they do not become numeric errors or mutate the workbook.

## Execution bounds and metering

The fixed helper supports six-limb units/weights, seven-limb sums/remainders and
twelve-limb products. With N<=64, aligned weights are below 2^190, sums below 2^196,
and products below 2^380. Multiplication and at most 384 division-bit steps per weight
are exact. Insertion ranking has at most N*(N−1)/2 comparisons, at most 2,016.

Explicit helper storage is 9,472 bytes including bounded input copies and 2 KiB of
fixed local/control allowance, within the 16-KiB admitted reservation. Compiler frames
and the evaluator's existing argument stack are outside that accounting. The helper
uses no dynamic allocation internally; the typed caller reserves memory before
constructing its fixed scratch and bounded input copy, then releases both on every exit.

The arithmetic counter measures limb-loop work, not CPU instructions. A worst-case
per-recipient bound includes 168 alignment iterations, seven-word sum/copy operations,
42 multiplication iterations, at most 384*(8+7+7+1)=8,832 division iterations,
normalization/narrowing and final-share checks. Ranking adds at most 7*N*(N−1)/2
iterations; final proper-partial normalization adds at most 168. Including fixed
control/copy allowance, these remain below 4,096+16,384*N. Fixed coefficient/control copies are covered by the conservative bound. The arithmetic
loop counter additionally refuses an increment beyond the ceiling; it is not a CPU
instruction profiler. Independent oracles, cancellation
checks and failure tests supplement this bound; observed counts are not the proof.

Canonical and direct typed execution use one context-bearing API with explicit
remaining visits, allocation domain and cancellation callback. Weighted numeric work
reserves 4+16*N existing visits (1,028 at N=64) in addition to normal argument charges.
There is no public context-free typed execution fallback. Pure low-level arithmetic
accepts caller-owned scratch and independently enforces limb/cancellation bounds;
it is not a separate evaluator. Structured/scenario/captured adapters route through
the same canonical execution. Global limits have not increased.

Cancellation is checked at recipient/ranking boundaries and whenever limb work crosses
a 128-iteration block, which is tighter than the approved division-bit interval.
Malformed arities refuse before scratch construction. Parsing, lowering, token validation,
native decoding and execution share arity 4..67; source/token/depth budgets also apply.
Opcode 20 uses existing WB12/private v13/hosted v1/CB5/CB6 encodings. Older engines
reject unsupported operations. No numeric value type or Desktop interaction changed.

## Qualification

Independent Python Fraction/subset and .NET BigInteger/ranking references, real SUM in
varied orders with inserted zeros, and real session/structural/native-persistence tests
qualify the arithmetic. All six selected headless targets passed 13/13 and 12,841
weighted vectors each, with zero mismatches; macOS x64 is Rosetta. Final local CI
passed 13/13, all-feature Rust regression passed 515 and strict Clippy passed. See the
[implementation review](../handoffs/2026-11-01-weighted-allocation.md) for retained
failures and source/resource qualifications. Resource counters
reported by successful helper returns are distinguished from errors: zero reported
`max_limb_visits` in the all-Overflow corpus means no successful return supplied that
metric, not that failing calls performed no work. Failure paths still enforce the
same checked ceiling and reservation and release their charges.

See [ADR-0037](../adr/0037-weighted-conservation-preserving-allocation.md) for the
approved contract, bounds and four mandatory implementation clarifications.

Implementation acceptance preserves the documented exact-SUM, association, metering,
arity and validation guarantees, including the test-only Linux ARM64 correction and
selected-source qualifications. Range-based weights are a separate future design;
this scalar contract and its bounds remain unchanged.
