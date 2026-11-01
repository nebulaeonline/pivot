# ADR-0037: Weighted, conservation-preserving allocation

Date: 2026-10-31.
Status: **Accepted and closed by implementation review (2026-11-01).**

## Context and scope

ADR-0036 is accepted and closed. This proposal extends equal allocation to explicitly weighted recipients. Preserve exact settlement amounts, explicit precision,
strict Decimal operands, real SUM reconciliation and understandable formulas. Do not
change ALLOCATE, division, ROUND, SUM, formatting or canonical value types.

The first weighted function needs an explicit ordered weight source. Existing typed
calls consume scalar arguments; ranges currently belong to reducers such as SUM.
A range-valued argument would require ordered membership, Blank-position semantics,
range identity/structure behavior and transfer vocabulary beyond a numeric helper.
This proposal deliberately starts with scalar weight expressions and a small bound.
The scalar-list scope is approved; range support is deferred,
not silently approximated by an aggregate or a list of currently occupied cells.

No feature implementation, benchmark or new platform execution accompanies this design.
ADR-0023 remains parked at G3.

## Approved function and argument contract

`ALLOCATEWEIGHTED(total, recipient_index, decimal_places, weight1, ..., weightN)`

All arguments are required; N is the number of supplied weight arguments. Accept
**1 through 64 weights** and a one-based index in 1..N. The 64-weight cap is a new,
function-specific admission boundary, not a reduction of existing workbook capacity
or a promise that 64 long expressions fit every existing formula limit. Existing
source, token, depth, dependency, clipboard and evaluation bounds apply independently.
No omitted/default precision, inferred current row, array/spill or hidden group.

| Argument | Accepted values |
| --- | --- |
| total | Strict Decimal96, exactly integral in the requested settlement unit |
| recipient_index | Strict Decimal96 integer, 1..N; classify before narrowing |
| decimal_places | Strict Decimal96 integer, 0..28, independent of formatting |
| each weight | Strict Decimal96, nonnegative; at least one weight must be positive |

String, Boolean and Blank are TypeMismatch, including Blank weights and precision.
Do not skip them or coerce them to zero. Explicit numeric zero is a valid weight;
negative zero is zero. Negative weights and an all-zero weight list are InvalidArgument,
even for a zero total. Integer metadata with trailing fractional zeros is valid.
Fractional weights are allowed; they are ratios, not settlement amounts or percentages
requiring a sum of 1 or 100. A positive common rescaling gives the same result when
the rescaled inputs themselves remain exactly representable and valid.

The total follows ADR-0036: 10.005 at precision 2 is InvalidArgument, never rounded.
Return normalized Decimal96 amounts; zero is positive with scale zero. Negative totals
use the positive-magnitude allocation with signs restored, including residual recipients.
The function's settled values are independent of cell number formats.

Example: `ALLOCATEWEIGHTED(10,1,2,1,2,3)` returns 1.67. Indices 2 and 3 return
3.33 and 5. Their actual complete SUM is 10; formatting may display 10.00.
The first three arguments mean how much, which recipient and how many decimal places;
the remaining arguments state each recipient's relative weight in order.

## Exact rule and deterministic ties

Let U be the absolute total in integer settlement units. Express each weight exactly
as a nonnegative integer W_j at a common decimal scale S, and let D=sum(W_j)>0.
For each recipient compute exact integer division:

`U*W_j = D*Q_j + r_j`, with `0 <= r_j < D`.

Set `R=U-sum(Q_j)`. Then 0<=R<N. Rank recipients by descending r_j, breaking exact
ties by ascending explicit list ordinal. The first R recipients receive Q_j+1 units;
all others receive Q_j. Every zero-weight recipient receives zero and is ineligible
for residual units. R=0 means no increments. Rank using exact integer remainders
with their shared denominator, never rounded Decimal division or float comparisons.

Since sum(r_j)=R*D, if R>0 there are more than R nonzero remainders; zero weights
cannot displace a positive-remainder recipient. Restore the total's sign for nonzero
results. For each consistent full index set exactly once, sum(Q_j)+R=U, so settlement
units and the original total are conserved. Each magnitude is a floor or ceiling of
its exact proportional entitlement; recipients need not differ by only one unit.

| Total / precision / weights | Complete settled results |
| --- | --- |
| 10 / 2 / (1,2,3) | 1.67, 3.33, 5 |
| -10 / 2 / (1,2,3) | -1.67, -3.33, -5 |
| 0.01 / 2 / (1,1,1) | 0.01, 0, 0; first ordinal wins the tie |
| 0.05 / 2 / (1,2,3) | 0.01, 0.02, 0.02 |
| 10 / 2 / (0,1,0) | 0, 10, 0 |
| 10 / 2 / (0.1,0.2,0.3) | 1.67, 3.33, 5 |
| 0 / 28 / (0,1) | 0, 0, after complete argument validation |

Equal positive weights must match ALLOCATE at the same count/index/precision, including
admission, for every supported N. There is no rotation or long-term fairness state.

Largest-remainder settlement is not monotone across changed specifications. For example,
weights (5,3,1) at precision 0 give (2,1,1) for total 4, but (3,2,0) for total 5.
A larger total can reduce one recipient's settled share. This is an accepted consequence,
not an implementation defect to repair through hidden state. No
cross-total, population-change or repeated-period fairness guarantee is proposed.

## Recipient association and reference semantics

Recipient j means **the jth weight expression in the argument list**. List order is
also the tie order. It is not worksheet row order, label text, sort order, visibility
or a persisted recipient identity. Scalar weights may be literals, ordinary references,
cross-sheet references or scalar-producing expressions such as SUM. A bare rectangle
is not a weight, and filtered/visible membership does not implicitly select recipients.

For example, with weights in B2:B4 and explicit indices in C2:C4, a formula in D2 is:

`=ALLOCATEWEIGHTED($F$1,C2,2,$B$2,$B$3,$B$4)`

Fill uses the existing reference rules: C2 translates, while the explicitly anchored
weight expressions retain normal bindings. The function does not infer that C2 is
associated with a particular name or row. Other calls must use the same ordered
weight specification, total and precision for the complete-group guarantee.

Sorting can move referenced weight identities and explicit index cells separately.
Insertion/deletion can change references or yield ReferenceDeleted under normal
structural rules; inserting a new weight cell does not add an argument. Relative
Copy/Fill can create a different weight list. Absolute markers control ordinary
translation, not business-record association. None of these actions renumber indices,
reorder weight arguments, add recipients or repair groups. Explicitly test a sort
whose moved record no longer matches its ordinal; do not claim association survived.

Changing list order is a changed specification and can change tie winners. Changing
one weight legitimately changes the other results when recalculated. Immutable captures
must retain their earlier weights and allocations. Independent scalar calls cannot
certify consistent weights, uniqueness, completeness or a user's intended association.
A durable keyed/group allocation object would require a separate design.

## Group-wide representation and production SUM

Compute the entire final settled unit vector A_j=Q_j+award_j before narrowing results.
Validate **every final share that actually occurs**, independent of requested index.
Do not reject for a hypothetical unawarded Q_j+1 or an unused lower Q_j when that
recipient actually gets the increment; normalization may make only the final value fit.
Every valid index for the same specification shares the same admission outcome.
For example, total M at precision 1 with weights (M,10^-28) must return (M,0):
the dominant recipient's unawarded floor is M−0.1, which cannot fit Decimal96,
but its actual awarded share M fits. Here 10^-28 describes the literal
0.0000000000000000000000000001; exponent input syntax is not introduced.

Normalize all nonzero settled shares and let t be their maximum scale. If all shares
are zero, use t=0 and admit the already-valid zero group. Otherwise let m be the
smallest strictly positive settled-unit magnitude and require:

`G=(U-m)/10^(p-t)` to be exact and at most `M=2^96-1`.

This is the largest proper subset of **nonzero** settled shares, expressed at their
common scale. Any proper subset containing fewer than all nonzero shares fits exactly;
any subset containing all nonzero shares equals the already-representable input total.
Zero terms cannot affect the value. The complete total may normalize below t; do not
require its full coefficient at scale t to fit 96 bits.

Ignoring zero shares in this bound is essential: weights (1,1,1,0), total 10, precision
28 must admit the same three nonzero amounts as ADR-0036 plus zero. A naive minimum
including zero would incorrectly require the full scaled total to fit. Likewise
(M, precision 28, weights (0,1,0)) must return (0,M,0). This does not weaken the
proper-partial proof or change ALLOCATE's existing admission rule.

Tests must exercise the actual production SUM with zero terms first, last and mixed,
plus forward, reverse and permuted nonzero accumulation. Independent mathematical
oracles are additional evidence, not a substitute. A representational failure is
Overflow for every valid index, including zero-weight recipients. Invalid parameters
remain InvalidArgument. Existing operational resource failures remain operation failures.

## Fixed-width arithmetic and bounded execution

With normalized Decimal96 weight coefficients C_j and scales s_j, choose S=max(s_j).
Each `W_j=C_j*10^(S-s_j)` is less than 2^190. With N<=64, D<2^196, so seven u32
limbs suffice for weights/sum/remainders. U<2^190 as in ADR-0036. Products U*W_j
are below 2^380 and fit twelve u32 limbs. Quotients/settled unit amounts are at most
U and fit six limbs. Do not limit the denominator or product to Decimal96 or u128.

Propose schoolbook six-by-six-limb multiplication and bounded 384-bit long division
by a seven-limb denominator, using checked carries, shifts and narrowing. Each
remainder shift fits seven limbs because it is below 2D<2^197. Preserve exact quotient
and remainder. Sort a fixed ordinal array by exact remainder using bounded insertion
sort (at most 2,016 comparisons for 64 entries), with ordinal tie order explicit.
No runtime BigInteger, float, unbounded sort, recipient-sized global cache or mutable
worksheet state. Fixed arrays hold normalized weights, quotients, remainders, awards
and final shares; the requested share is returned only after complete admission.

Approve **16 KiB maximum explicit helper scratch/control storage** and an audited upper
bound of **4,096+16,384*N limb visits** per call (1,052,672 at N=64). These are new
weighted-helper bounds, not ADR-0036's 256-byte/1,024-visit primitive. Implementation
must prove layout and worst-path accounting, including failed calls, before acceptance.
If they do not suffice, return to design review; do not silently expand them.

Unlike a cheap scalar primitive, weighted work must be visible to the existing engine
budget. Reserve `4+16*N` existing evaluation visits before numeric helper work,
corresponding to the conservative bound in blocks of 1,024 limb visits. Keep normal
argument/task charges too; no refund, limit increase or implicit group memoization.
Reserve the scratch through the existing runtime memory admission before allocation,
release it on every exit, and do not hide it in uncharged retained state. Cancellation
checks occur at recipient boundaries, every at most 128 division-bit steps, and each
ranking outer iteration. Preserve cancellation as an operational result, not Overflow.

Audit every typed execution entry point so none bypasses metering, cancellation or
memory admission. A transport-independent helper may accept caller-provided bounded
scratch and a cancellation callback; the canonical evaluator owns resource accounting.
Do not add workbook/graph state to pure `typed::apply` or create divergent arithmetic
implementations. The metering unit and cap are approved initial accounting policy, subject to proof
across every supported execution and failure path.
A formula can still refuse under normal global limits; a 64-weight group requires
repeated scalar computation, not a hidden shared allocation computation.

## Errors, laziness and representations

| Condition | Outcome |
| --- | --- |
| No weights, more than 64, missing arguments, bare range or malformed syntax | Existing parse/lowering diagnostic and non-executable draft; defensive token validation rejects unsupported arity/shape |
| Non-Decimal scalar argument | TypeMismatch |
| Index nonintegral/outside 1..64, precision nonintegral/outside 0..28, negative weight | InvalidArgument at that argument |
| Index greater than actual N, or all weights zero | InvalidArgument after local argument validation |
| Total not integral in the requested unit | InvalidArgument |
| Any final share or proper-nonzero-partial admission fails | Arithmetic Overflow for every valid index |
| Dependency/reference fault | Preserve established category, origin and propagation |
| Budget/memory refusal or cancellation | Existing operational outcome, no partial workbook mutation |

Demand total, index, precision and weights left to right, applying local type/domain
checks before demanding later operands. After all succeed, check index<=N, positive
weight sum, total settlement representability, then complete arithmetic admission.
For `ALLOCATEWEIGHTED(10,3,2,1,1/0)`, the final dependency error precedes index>N.
A negative first weight stops before a later division error. Zero total bypasses no
validation. Existing IF/AND/OR laziness applies; inactive calls do not execute numeric
work or reserve weighted scratch. Static grammar/arity validation still applies.

Approve additive typed opcode **20**, arity 4..67, subject to codec verification. Code 19 remains ALLOCATE.
The existing typed-call opcode/arity vocabulary appears sufficient: propose keeping
WB12, private session v13, hosted v1 and CB5/CB6 unchanged, with no array/range-valued
argument or saved group. Verify all codecs, structured expressions and authoring
paths before asserting this. Older engines must refuse unsupported operations.
If an actual encoding change is discovered, return for review instead of smuggling
it under old versions. No Desktop DTO, license enforcement or UI interaction change
belongs to this arithmetic ADR.

## Alternatives and review decisions

| Alternative | Proposed disposition |
| --- | --- |
| Ordered scalar weight list, at most 64 | Recommend for a bounded first version; explicit but verbose for large groups |
| Single ordered rectangular weight range | Defer; requires an ordered range-operand and membership/association contract, not just SUM-style reduction |
| Keyed weight/recipient table | Defer; could represent association explicitly, but needs new group/identity semantics |
| New array/spill value or variadic unbounded list | Reject for this milestone |
| Rounded Decimal division and approximate remainder comparisons | Reject; can choose the wrong residual recipient or lose units |
| Negative weights, implicit Blank zero or all-zero denominator | Reject as InvalidArgument/TypeMismatch, not signed-offset accounting |
| Divisor methods or stateful period balancing | Different contract; reconsider explicitly if cross-total monotonicity/fairness is required |
| Treat every weighted invocation as one cheap unmetered operation | Reject; repeated group scans must consume bounded admitted work |

## Required implementation verification

Use independent Python Fraction and .NET BigInteger references with algorithms distinct
from production limbs/ranking. Exhaust small totals, signs, precisions and bounded
weight lists, checking every index, exact unit totals, floor/ceiling entitlement,
residual ranking, ties, zero-weight exclusion, sign symmetry and normalized zero.
Cross-check equal weights against accepted ALLOCATE and common positive scaling where
inputs remain representable. Include changed-order tie outcomes and the nonmonotonic
(5,3,1) example; neither should be silently repaired.

Include Decimal96 extremes, mixed scales 0/28, weight sums beyond 96 bits, products
beyond 128/256 bits, all-zero/negative/Blank/String/Boolean weights, trailing-zero
metadata, index bounds, rejected arities and lazy/dependency-order provenance. Explicitly
cover zero settled shares, one positive settled share, hypothetical unused shares,
actually fitting requested shares with a failing peer, and proper-partial refusal.
Use actual SUM in multiple orders, including zero insertion and final normalization.

Exercise scalar/cross-sheet/structured expressions, captures, Copy/Paste CB5/CB6,
ordinary/visible Fill, sorting and both structural axes/actions, WB12 Save/reopen and
error filtering through the real C#–Rust session. Test changed weight association,
ReferenceDeleted, inactive laziness, budget exhaustion before helper work, cancellation
inside division/ranking, scratch reservation failure and zero charge after teardown.
Audit all execution entry points and malformed-token refusal.

Declare resource corpora before running them: 16,384 base cells, a complete 64-recipient
group with production SUM, and 256 weighted scalar calls each with 64 weights. Use
five modes (ordinary mixed weights, mixed-scale extremes, zero/tied weights, wide
products/final normalization, group Overflow), four real changed-input samples and
four retained captures. Report exact extra input/output positions and helper versus
engine timings separately. Evaluate at most the existing 256 outputs per request;
never raise a request/global budget to finish a corpus. Keep the original one-second
bounded-engine and 256 MiB steady/512 MiB peak objectives. These are observed core
samples and charged memory, not GUI latency, percentiles or RSS. No benchmark has run.

Require final local CI, strict all-target/all-feature Clippy, all-feature Rust tests
and native headless CI/parity on all six targets, retaining macOS x64 as Rosetta.
The approved numeric-only scope does not automatically require a new attended GUI/AOT
round. Reassess targeted qualification if implementation changes interaction, generated
DTO metadata, serialization or AOT-sensitive paths. Preserve existing GUI
receipts as historical. Archive/verify and prune completed caches after every batch,
measure diagnostic per-project cloc once on the final canonical source, and retain
original failures and selected-source qualifications. No native run is authorized now.

## Design disposition — 2026-10-31

**Approved with four mandatory clarifications.** The scalar-list signature, 1..64
weight bound, strict Decimal operands, explicit precision, exact largest-remainder
ranking and ordinal tie order are accepted. Equal positive weights must reproduce
ALLOCATE results and admission. The lack of monotonic shares across changed totals
is accepted and must remain documented. Range-based weights remain a separate design.

1. **Recipient association is positional.** Index j selects argument j in the weight
   list. It does not identify a row, label, visible record or persistent business
   entity. Copy/Fill, sorting and structure retain ordinary reference rules without
   repairing changed associations. Complete consistent membership remains explicit.
2. **Representability follows final awards.** Exact quotas and remainder ranking
   precede narrowing. Validate every final share, including zero, plus the approved
   proper-nonzero-subset SUM bound. Hypothetical unused floors/ceilings cannot cause
   rejection. Every valid index shares the same arithmetic admission result. Actual
   production SUM must reconcile complete accepted groups in varied orders, including
   zeros first, interspersed and last.
3. **All arithmetic is admitted and cancellable.** The twelve-limb product,
   seven-limb denominator/remainders, bounded ranking, 16-KiB explicit scratch and
   4,096+16,384*N limb ceiling are accepted design bounds. Reserve 4+16*N existing
   evaluation visits before numeric work, retaining normal argument charges. Every
   evaluator, structured-expression and direct typed-operation entry must honor
   accounting. Reserve/release scratch through existing memory admission and check
   cancellation inside division/ranking. Prove storage and work bounds on successful
   and failing paths; refusals release charges and leave workbook state unchanged.
   Higher material cost or exceeded bounds require design reconsideration, not
   automatic quota expansion, uncharged fallback or hidden memoization.
4. **Arity is enforced throughout.** Parsing, lowering, typed instructions, decoding,
   restoration and canonical evaluation must agree on 4..67 total arguments and
   1..64 weights. Unsupported shapes cannot enter the helper or be reinterpreted.
   Existing source/token/depth/graph/resource limits remain independently enforced.

Opcode 20 is accepted subject to confirming generic encodings support the full arity.
Retain WB12/private v13/hosted v1/CB5/CB6 and current value types unless an actual
incompatibility requires further design review. ALLOCATE, division, ROUND, SUM,
formatting, bindings, captures and worksheet operations retain their semantics.

The verification plan is accepted: independent Python/.NET oracles, exhaustive small
and extreme/mixed-scale cases, actual SUM, association/nonmonotonicity, strict errors,
laziness, malformed tokens, work/scratch refusal, cancellation and real-session
worksheet/clipboard/native-persistence integration. Keep the declared 16,384-base-cell,
64-recipient and 256-call corpora and original one-second/256-MiB steady/512-MiB peak
objectives. Material workload failures require diagnosis and design reconsideration.

Local CI, strict Clippy, all-feature regression and all-six-target headless/parity are
approved validation scope; macOS x64 remains Rosetta. Reassess targeted GUI/AOT coverage
only if sensitive paths change. Preserve original failures, selected-source qualifications,
verified archives/cache cleanup and final diagnostic per-project code-size reporting.

This is design approval, not implementation completion. No feature code, benchmark
or platform execution accompanies this disposition. Range/array/keyed-group extensions,
implicit repair, fairness state, Desktop licensing implementation and history are
excluded. ADR-0023 remains parked at G3.

## Implementation progress — 2026-11-01

The fixed-width helper, context-bearing typed execution, resource/cancellation handling
and opcode/arity integration are implemented. Local independent oracles, worksheet
SUM, structural persistence and session/resource regressions have passed. A controlled
console failure boundary was added to the headless session runner; application behavior
and Desktop licensing are unchanged. Final qualification is pending; no implementation
acceptance is recorded. Earlier design-only statements describe the design disposition.

## Implementation disposition — 2026-11-01

Implementation and final qualification are complete and accepted.
See the [implementation review](../handoffs/2026-11-01-weighted-allocation.md) and
[semantics](../architecture/weighted-allocation.md). All four clarifications are
implemented. Direct typed execution requires explicit visits, memory and cancellation
context. Explicit helper storage is 9,472 bytes within the 16-KiB reservation; approved
work bounds and global limits are unchanged. Opcode 20 fits existing representations.

Final local CI passed 13/13, all-feature Rust tests passed 515, strict Clippy passed,
and all six selected headless targets passed 13/13 with 12,841 weighted vectors each.
Production SUM passed 1,116 checks in three orders. The review records the test-only
cleanup-fixture correction and Linux ARM64 retry, original failures, selected-file
source qualification, Rosetta designation, observed core/charged-memory scope, final
code-size report and verified archival/pruning. No new graphical/AOT round or Desktop
licensing implementation is included. Earlier design-only and in-progress statements
retain their historical scope rather than describing current qualification.

## Implementation acceptance — 2026-11-01

**Accepted and closed.** Review accepts all four design requirements: ordinal
recipient association without repair, final-award-only group representation and actual
SUM, fully metered/cancellable execution, and consistent arity across every boundary.
The (M,10^-28) weight case, zero-share handling, exact ranking, equal-weight equivalence,
negative symmetry and documented nonmonotonicity satisfy the allocation contract.

The context-bearing typed API, 9,472-byte explicit helper layout within the 16-KiB
reservation, unchanged limb/visit ceilings and failure-path charge release are accepted.
Opcode 20 and unchanged WB12/private v13/hosted v1/CB5/CB6 remain appropriate.

Accepted evidence includes 1,116 production-SUM checks over 372 groups, 12,841 independent
weighted vectors per target, 515 all-feature Rust tests, strict Clippy, final local CI
13/13 and all six selected headless targets 13/13 with zero final session charges.
Preserve four-sample observed-core maxima, charged-memory versus RSS distinctions,
original objectives and macOS x64 Rosetta designation. The numeric-only graphical/AOT
scope decision is accepted; historical graphical receipts remain historical.

The Linux ARM64 setup correction is accepted as a test-fixture repair: the original
failure remains Failed, with its fresh successful retry separate. Queue capacity and
the scratch-admission assertion were not weakened. Five earlier passing snapshots
precede that final test-only edit; the retry and final local CI include it. Eleven
selected files match the final checkout on all six targets, without implying complete
historical byte identity. Verified archives/cleanup and the final diagnostic count of
90,859 lines across 400 files are accepted; 90,855 remains an earlier measurement.

The next design direction is ADR-0038, Range-Based Weighted Allocation. It should
investigate an ordered worksheet range as the weight source while reusing this exact
arithmetic. Range ordering, Blank positions, reference/structural behavior, bounds and
representation need an explicit design. Skipping Blank positions must never silently
renumber recipients. No range implementation or platform execution is included in
this acceptance. ADR-0037 remains unchanged; ADR-0023 remains parked at G3.
