# ADR-0036: Exact, conservation-preserving allocation

Date: 2026-10-31.
Status: **Design approved with three clarifications by parent review (2026-10-31).
Feature implementation requires separate owner authorization.**

## Context

ADR-0035 is accepted and closed. The parent assigned allocation next, and the owner
subsequently authorized this design. Start from the [captured ALLOCATE proposal](../proposals/allocate.md)
and its required-precision clarification. This record contains no feature implementation,
new benchmark or platform execution. ADR-0023 remains parked at G3.

Independent division expressions do not describe a complete allocation group. Rounding
each result independently can leave an undistributed settlement unit; retaining hidden
fractional amounts can instead make displayed items disagree with their displayed sum.
The function returns actual settled Decimal96 amounts, independent of display
formatting, without changing division, ROUND or SUM.

Two distinct representability problems matter. Multiplying a large Decimal96 total by
10^28 can require about 190 bits of integer storage even if the final answer is an
ordinary representable integer. Separately, representable individual shares do not
by themselves prove that existing Decimal96 SUM can accumulate them without precision
reduction. The design must address both rather than promise mathematical conservation
while allowing an accepted complete group to lose units in the normal sum.

## Decision

The parent approves one pure, deterministic equal-weight scalar function:

`ALLOCATE(total, recipient_count, recipient_index, decimal_places)`

All four scalar arguments are required. Names follow existing case-insensitive function
recognition. No omitted argument, default precision, implicit recipient index, current
row lookup, spill, hidden group registry or automatic correction of other formulas.
The parent approves the strict operand and group-admission decisions below, subject
to the three mandatory clarifications in the disposition section. Await separate
owner implementation authorization.

### Argument and result contract

| Argument | Approved accepted values |
| --- | --- |
| total | Any Decimal96 value exactly representable in the requested settlement unit, subject to complete-group admission below |
| recipient_count N | Exact integer-valued Decimal96, 1 through 4,294,967,295 (u32 maximum) |
| recipient_index i | Exact integer-valued Decimal96, 1 through N; one-based |
| decimal_places p | Exact integer-valued Decimal96, 0 through 28; required |

All four operands must actually be Decimal values. Boolean, String and Blank return
TypeMismatch; do not parse numeric Strings or convert Blank to zero. This is an
ALLOCATE-specific validator, using the existing strict typed-argument pattern, not a
change to the shared numeric conversion helper or other functions. In particular,
a supplied reference whose precision cell is Blank must not silently choose whole
units. Numeric zero remains a valid total. Integer-valued 3.0 and 2.00 are accepted;
3.5 is not truncated. Numeric negative zero is accepted as precision 0, but is not a
valid count/index. Classify full Decimal96 values before narrowing machine integers.

p=0 means whole units, p=2 hundredths, p=3 thousandths. Negative precision is invalid
here, even though ROUND separately supports negative digits. A total with unnecessary
trailing fractional zeros is acceptable: 10.000 at p=2 is exactly 1,000 units.
10.005 at p=2 is InvalidArgument, never silently rounded to a different obligation.
Callers may explicitly round a total in a separate expression if they intend to
change it; ALLOCATE itself never performs that change.

Return a normalized Decimal96: remove insignificant fractional zeros and canonicalize
all computational zero to positive zero at scale zero. Requested precision defines
the settlement unit, not mandatory stored scale or cell formatting. A negative nonzero
total produces negative shares with exactly the positive-total magnitudes. Every
zero share, including from a negative allocation, is canonical positive zero.

### Allocation rule and conservation

Let unit = 10^-p. Convert the magnitude of the exact total into a nonnegative integer
unit count U. Compute Q=floor(U/N) and R=U mod N, with 0<=R<N.

Recipient i gets Q+1 units if i<=R, otherwise Q units. Restore the total's sign for
nonzero results. Lowest indices receive the residual units first; ties never depend
on evaluation order, worksheet coordinates, visibility, threads or platform. For
negative totals, the same indices receive the additional negative unit. No floor-
division asymmetry for negative inputs is permitted.

| Call or complete group | Result |
| --- | --- |
| ALLOCATE(10,3,1,2) | 3.34 |
| ALLOCATE(10,3,2,2), ALLOCATE(10,3,3,2) | 3.33 each |
| ALLOCATE(-10,3,1,2) | -3.34; remaining two indices -3.33 each |
| ALLOCATE(0.01,3,i,2), i=1,2,3 | 0.01, 0, 0 |
| ALLOCATE(-0.01,3,i,2), i=1,2,3 | -0.01, 0, 0 |
| ALLOCATE(10,3,i,0), i=1,2,3 | 4, 3, 3 |
| ALLOCATE(0.001,3,i,3), i=1,2,3 | 0.001, 0, 0 |
| ALLOCATE(0,3,i,28), i=1,2,3 | 0, 0, 0 |

For every accepted specification and all indices 1..N exactly once:

`R*(Q+1) + (N-R)*Q = N*Q+R = U`.

Thus every returned amount is an exact settlement-unit multiple and the complete
consistent set conserves the exact total. No rounded Decimal96 division is used to
derive Q or R. Shares differ in magnitude by at most one settlement unit.

Independent scalar calls cannot prove a user supplied every index once, or used the
same total/count/precision. Duplicate indices, omitted recipients, inconsistent inputs
and user changes to returned values invalidate that group guarantee without necessarily
causing a per-call error. No workbook-wide reconciliation validator is claimed.
Fixed residual ordering is repeatable, not long-term fairness; rotation is deferred.

### Complete-group representability and exact SUM admission

Use group-wide, index-independent admission. First verify that every distinct share
which actually occurs is exactly representable as Decimal96: Q always occurs; Q+1
occurs only when R>0. Never reject because an unused Q+1 would overflow when R=0.
Validate the whole specification, not just the requested recipient's share.

Next let t be the maximum scale of those normalized nonzero share values (zero adds
no scale requirement; an all-zero group has t=0). The smallest share is Q units,
which always occurs. Compute the largest proper partial sum magnitude at common
scale t:

`G = (U-Q) / 10^(p-t)`.

This division must be exact and G must fit an unsigned 96-bit coefficient. Otherwise
every call for that specification returns arithmetic Overflow, even if its requested
share fits. For N=1, G=0: do not reject a valid single recipient merely because the
scaled full total is wider than 96 bits.

Every proper subset of these same-sign shares has magnitude at most total minus the
smallest share, and has scale at most t. The bound on G makes every such partial sum
exactly representable. The final complete sum is the original already-representable
total; it may normalize to a lower scale than t. Thus existing Decimal96 addition/SUM
can reconcile the accepted complete group in any accumulation order without new sum
semantics. Do not demand that the full total's coefficient at scale t fit 96 bits:
that would unnecessarily reject cases whose last exact addition normalizes, such as
10 divided three ways at p=28.

Existing evaluation/resource limits, cancellation and errors still apply. This is
an arithmetic guarantee, not a promise to materialize billions of recipients in one
workbook or complete an over-budget SUM. Additional unrelated operands or subsequent
rounding/multiplication are outside this guarantee. Group validity is computed from
the specification, without reading or certifying other cells.

Important boundaries for parent review and implementation tests:

- ALLOCATE(M,1,1,28) succeeds with M. U is wider than 128 bits, but normalization
  returns the exact existing integer; the unused Q+1 must not be inspected.
- M is divisible by 3. ALLOCATE(M,3,i,28) succeeds with M/3 for every valid i; the
  reduced common scale is zero and exact SUM reaches M.
- ALLOCATE(10,3,i,28) succeeds: all proper partial sums fit at scale 28 and the
  complete total normalizes exactly to 10. Do not reject this using the wider full U.
- ALLOCATE(M,2,i,1) returns Overflow for every i: the required half-integer shares
  cannot be represented exactly by Decimal96 at that magnitude.
- ALLOCATE(M,10,i,1) returns Overflow even though each individual M/10 share fits.
  The largest proper partial coefficient is 9*M, and some intermediate sums cannot
  be represented exactly. Do not quietly rely on ordinary addition rounding.
- For T=M-9, N=11, p=2, the lower share Q has a representable normalized encoding,
  while Q+1 does not. All indices, including the lower-share recipient, must error.
- For the smallest positive Decimal96 total, p=28 and N=u32 maximum, index 1 gets
  that total and all other indices get zero. There is no count-sized iteration.

The stricter exact-SUM admission is an approved ALLOCATE contract, not a pre-existing
Decimal96 guarantee. Production SUM coverage is mandatory in addition to the algebraic
bound. If implementation uncovers an arithmetic boundary, diagnose it and return for
review; do not silently omit the gate, weaken conservation or alter ordinary SUM.

### Error and evaluation order

| Condition | Outcome |
| --- | --- |
| Missing/extra arguments, empty argument syntax, malformed expression | Existing parse/lowering diagnostic and non-executable draft behavior; defensive typed-token validation rejects bad arity |
| Any operand is Boolean, String or Blank | TypeMismatch; no new coercion |
| Fractional/out-of-range count, index or precision; zero/negative count; index outside 1..N | InvalidArgument, not DivisionByZero |
| Total is not an integral multiple of 10^-p | InvalidArgument |
| Used share or common-scale exact-sum admission fails | Existing arithmetic Overflow |
| Argument dependency/reference fault or arithmetic fault | Preserve its existing category and origin |
| Oversized formula, exhausted evaluation/admission budget or cancellation | Existing refusal/cancellation behavior; no partial workbook mutation |

Demand arguments left to right with existing per-argument short-circuit failure.
Type-check each Decimal argument and check local bounds/integrality for count/index/
precision before demanding later arguments. After all four succeed, check i<=N,
then total settlement representability, then group representability/admission.
Cross-argument validation is not speculative: ALLOCATE(10,2,3,1/0) reports the fourth
argument's dependency arithmetic error before the later i<=N check. In contrast,
ALLOCATE(10,0,1/0,2) stops at invalid count without demanding the third argument.
Zero total never bypasses argument validation.

Honor existing lazy IF/AND/OR demand. An inactive, syntactically valid ALLOCATE call
must not evaluate its arguments, overflow, allocate group state or consume count-sized
work. Laziness does not make a syntactically invalid call valid. Runtime failures
remain formula outcomes with provenance; they do not rewrite the formula as a draft.

### Bounded exact algorithm

Use a transport-independent helper in pivot-types, called by the canonical Rust typed
evaluator. Its boundary accepts PivotDecimal plus u32 count/index/precision and returns
a normalized PivotDecimal or a bounded allocation error mapped to existing
InvalidArgument / arithmetic Overflow categories. A small domain-specific helper
error is permitted; no new wire error or value kind is needed. Production C# does not
implement allocation; independent Python/.NET references belong only in tests.

For total magnitude coefficient C and stored scale s:

1. If s>p, divide C by 10^(s-p) using exact checked integer arithmetic; a nonzero
   remainder rejects the total. C is at most 96 bits and this divisor fits u128.
2. Otherwise construct U=C*10^(p-s) in a fixed six-u32-limb unsigned buffer. Because
   C<2^96 and p-s<=28, U<2^190; 192 bits suffice. This is local arithmetic scratch,
   not a new workbook numeric type or arbitrary-precision dependency.
3. Divide the six limbs by N from high to low. Each step uses a u64 accumulator
   `(remainder<<32)|limb`; remainder<N<=u32 maximum guarantees it fits. Obtain exact
   quotient Q and one-word R. Form Q+1 only if used, with checked carry.
4. Reduce each used share by exact factors of ten, at most p times, then require
   the remaining coefficient to fit 96 bits. Normalize zero and restore sign.
5. Derive common scale t. Subtract Q from U in the fixed-width buffer, then exactly
   divide by 10^(p-t) to obtain G. Reject if that proper-partial coefficient exceeds
   96 bits; return the already validated selected share. Do not narrow full U first.

Do not multiply in Decimal96 or convert the scaled U to u128. No binary floating
point, rounded division, saturation, underflow-to-zero or discarded remainder is
allowed. Do not enumerate recipients, sort equal remainders, allocate an N-entry
vector, memoize groups, or add hidden persistent state.

The parent approves at most 1,024 limb visits per helper call and 256 bytes of explicit fixed
limb/control scratch, independent of N and input magnitude. These are internal
algorithm bounds, not an estimate of compiler frames or the existing expression stack.
A multiplication has at most 28 six-limb passes; quotient division has six steps;
normalizing two used shares and the proper-partial bound has at most 28 passes each.
Keep checked carries and coefficient narrowing explicit. Bound conversion of all
three integer-valued arguments before machine casts as well.

Retain existing source/token/depth, dependency, selection, evaluation-visit, workbook,
frame and runtime budgets. Admit the function as one bounded existing typed instruction,
as with other bounded numeric helpers; the 1,024 internal primitive bound is not a new
global work budget. Existing evaluation cancellation checks bracket calls; no helper
loop may depend on count. Measure worst-case helper latency and expression density
before implementation acceptance. A discovered need for new quota, bigger scratch,
different bounds or weaker guarantees returns to parent review rather than silently
raising limits.

### Integration, representation and user expectations

Add one named typed operation with arity four, approved code 19 (0–18 are already
assigned, including ReferenceDeleted at 18). Reuse the generic expression-call parser,
left-to-right typed validation, canonical evaluator, graph bindings and captures.
ALLOCATE works in scalar/structured expressions through existing shared semantics;
scalar-producing nested expressions are allowed. No range-valued argument, implicit
intersection, array/spill result or formatting side effect is introduced.

Typed operations already encode opcode plus arity. Retain WB12, private
session v13, hosted v1 and CB5/CB6. No new message shape, Decimal96 bit layout, cell
kind, token framing, saved group, migration or obsolete-format reader is required.
Extend opcode/name validation consistently wherever the shared typed vocabulary is
used. Existing files remain readable; older engines cannot execute the new function.
An older WB12 reader rejects unknown opcode 19 rather than silently interpreting it.
Source-only unknown-function handling retains its existing refusal/non-executable
behavior; no fallback to division or a different native clipboard format is allowed.
Feature availability is not promised merely because two engines share a file/protocol
version. Reconsider versioning only if implementation finds an actual encoding change.

Copy/Paste, ordinary/visible Fill, cross-sheet references, row structure and sorting
use their existing binding/translation rules for all four argument expressions.
Explicit index expressions move or bind normally; there is no automatic renumbering,
recipient identity inferred from cell location or group repair. Preserve signed
literal and rejected-draft provenance independently of normalized calculated results.

Formatting stays separate. At p=2, amounts 3.34/3.33/3.33 can display and sum as cents.
At p=3, forcing a two-decimal display can hide settled thousandths again. ALLOCATE
guarantees accepted underlying amounts and their exact sum, not reconciliation of
arbitrarily rounded or clipped display text. Function documentation should explain
that settled amounts must be displayed with enough precision; do not autoformat cells,
change the worksheet default or add a warning/approval dialog.

## Alternatives considered

| Alternative | Disposition |
| --- | --- |
| Infer groups from identical division formulas or cell locations | Reject: cannot know intended membership and would alter ordinary arithmetic |
| Make decimal_places optional or inherit cell format | Reject: obligation precision must be explicit and independent of presentation |
| Uniform Blank-as-zero like ordinary numeric conversion | Reject for this new function: Blank total/precision must not silently create an obligation or select whole-unit settlement; other functions remain unchanged |
| Use checked Decimal96 division followed by ROUND | Reject: double rounding and residual loss can violate conservation |
| Use u128 scaled units only | Reject: false overflow for valid large-total/high-precision groups such as (M,1,1,28) |
| Fixed 192-bit scratch | Recommend: proven bounded size and one-word divisor, no runtime big-integer dependency |
| Accept representable shares but ignore SUM intermediate precision | Reject for the promised reconciliation contract; expose as a parent decision if a broader mathematical-only domain is preferred |
| Require the full scaled total to fit 96 bits | Reject as unnecessarily strict: proper partial sums plus a representable normalized final total suffice |
| Return only the requested share even if other members overflow | Reject: group validity and errors should not depend on requested index |
| Multi-output allocation computation first | Defer: it could make membership explicit, but requires output/group authoring, identity/lifecycle and resource semantics beyond a scalar formula; existing model outputs are not worksheet spills |
| Stateful residual rotation, weighted shares or arbitrary unit argument | Defer: useful separate contracts; no hidden fairness/history state in the initial function |

## Consequences

Allocation becomes an explicit exact settlement calculation. Inputs/counts/precision
are visible, deterministic and stateless. Strict typing and common-scale admission
reject some otherwise numerically meaningful requests, but keep the guarantee useful
through existing SUM rather than only on paper. Fixed arithmetic adds a small audited
numeric primitive; it does not increase Decimal96 capacity or alter other operations.
A maximum count larger than worksheet capacity is meaningful mathematically; it does
not promise a complete group can fit one document. Scalar calls cannot certify user
membership, and formatting remains the user's separate presentation choice.

## Verification / reconsideration trigger

This is source-based design, with algebraic reasoning, not new execution evidence.
After parent review and separate owner implementation authorization:

- Use an independent Python integer/Fraction oracle and .NET BigInteger oracle in
  tests. They may allocate arbitrary precision; production must not. Do not reuse
  the six-limb algorithm or reconstruct the oracle through rounded decimal division.
- Exhaust bounded small totals, both signs, N and p, every valid index, exact unit
  sums, one-unit spread, deterministic residual order, sign symmetry and canonical
  zero. Repeat shuffled recipient/evaluation order and verify exact Pivot SUM.
- For very large N, verify quotient/multiplicity conservation algebraically and sample
  indices 1, R, R+1 and N where valid; never enumerate u32-maximum recipients. Cover
  tiniest totals, zero, count/index bounds, 28-place precision and decimal-valued
  integer arguments with trailing zeros. Reject count/index above u32 before narrowing.
- Test input representability, wrong arity/type/Blank, fractional/negative precision,
  left-to-right dependency errors, inactive lazy calls, all-zero invalid parameters,
  native decimal overflow and exact error provenance.
- Include every M boundary above: accepted wide units, unused Q+1, member-specific
  representability, and individually fitting shares rejected by exact-SUM admission.
  Verify all valid indices agree on group-wide error. Differentially check the
  common-scale condition against arbitrary-precision sums; ordinary SUM stays unchanged.
- Test all carries, six-limb high values, one-word quotient division at divisor 1
  and u32 maximum, powers of ten, quotient increment and exact trailing-zero removal.
  Prove/test the fixed scratch and work bounds without raising existing admission.
- Exercise cross-sheet updates, immutable old captures, nested SUM/IF/ROUND,
  error-filter predicates, exact formatting versus calculation, ordinary and visible
  copy/fill, sorting, insertion/deletion, native CB5/CB6 and WB12 save/reopen. Reject
  malformed opcode/arity and retain existing unknown-function non-execution behavior.
- Benchmark helper batches separately from canonical capture/evaluation and SDK/IPC
  timing. Use four declared samples, maximum count/precision and boundary/error mixes,
  four old captures and a 16,384-position base plus 4,096 allocation formulas (declare
  reference anchors). A full 4,096-recipient group may be evaluated via one existing
  bounded SUM output; private requests keep their 256-output limit. Cover 100 at
  p=2, tiny totals at p=28, and representability refusal; force real recalculation.
  Preserve the one-second bounded engine-operation, 256-MiB steady/512-MiB peak
  objectives on the stated corpus; label maxima, sample counts, charged memory versus
  RSS and round-trip scope. Material failures return for review rather than lowering
  the target, increasing quotas or removing the exact-SUM gate.
- Require local CI, strict all-target/all-feature Clippy, compatibility tests and
  headless CI/parity on all six RIDs, macOS x64 explicitly Rosetta. Use native target
  workspaces, host-serial timing, retained original failures, source-qualified results,
  verified archival/pruning per completed batch and final canonical per-project cloc.
- The parent approves no new attended desktop/AOT GUI round for this numeric-only milestone:
  ADR-0035 just qualified all six managed/AOT desktops, and this design changes no
  UI interaction, message schema or production C# evaluator/metadata. Headless tests
  exercise actual Rust-session interoperability and six-platform arithmetic. If
  implementation changes desktop interaction, DTO/source-generated metadata or other
  AOT-sensitive paths, reassess targeted attended qualification with normal local
  consent and short launchers. This scope is approved; it does not permit bypassing
  docs/testing.md or relabeling earlier receipts as new results.

## Parent disposition and next step

The parent approves the design with three mandatory implementation clarifications:

1. **Actual production SUM:** independently establish the mathematical bounds and
   exercise accepted complete groups through Pivot's real SUM evaluator in different
   recipient/accumulation orders. Oracle-only algebra is insufficient. Diagnose any
   representable-result rounding boundary; do not silently weaken reconciliation.
2. **Index-independent group refusal:** every valid index for the same total/count/
   precision receives the same group-representability outcome. A fitting recipient
   cannot succeed when another actually used share cannot fit. Preserve InvalidArgument
   for invalid specifications versus Overflow for representability failure, dependency
   fault provenance and left-to-right argument demand.
3. **Explicit completeness boundary:** conservation requires a complete, consistently
   specified index set exactly once. Independent calls do not certify completeness,
   uniqueness or consistency. Copy/Fill, sorting and structure do not renumber or
   repair recipient indices; no hidden group or cell-position state is introduced.

The parent approves the four required strict Decimal96 arguments, precision 0..28,
one-based u32 counts/indices, exact settlement-total validation, low-index residuals,
symmetric negatives and normalized positive zero. Used-share validation includes Q+1
only when R>0; the normalized-scale largest-proper-partial-sum guard is mandatory.
Fixed six-u32-limb arithmetic and the local scratch/work bounds are accepted without
runtime BigInteger, binary floats or recipient enumeration.

Opcode 19 and unchanged WB12/private v13/hosted v1/CB5/CB6 are approved provided existing
encodings prove sufficient. Ordinary division, ROUND, SUM, formatting, lazy evaluation,
bindings, captures, copy/fill/sort/structure and persistence retain their semantics.
Six-target native headless CI/parity, independent Python/.NET oracles, actual SUM and
integration/boundary tests, unchanged resource objectives, source qualifications,
verified archival/pruning and diagnostic cloc are approved. macOS x64 is Rosetta.
No new attended GUI/AOT round is required solely for numeric changes; reassess targeted
qualification if implementation alters interaction, serialization metadata or other
AOT-sensitive paths. Prior receipts remain historical evidence.

See the [design handoff](../handoffs/2026-10-31-exact-allocation-design.md). No feature
implementation, new benchmark or native run has occurred. Await separate owner
implementation authorization and retain the normal review/docs/commit-push cadence.
Weighted allocation, dynamic arrays, rotation, persistent groups, financial reporting,
automatic repair and history remain outside ADR-0036. ADR-0023 remains parked at G3.
Weighted allocation remains an unassigned nice-to-have. Wait for parent direction
after ADR-0036; no subsequent ADR number or scope is assigned.
