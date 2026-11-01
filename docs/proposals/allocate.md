# ALLOCATE: exact, conservation-preserving allocation

Date: 2026-10-31.
Status: **Historical starting proposal; ADR-0036 is accepted and closed.
ADR-0037 weighted allocation is accepted and closed.**

Recorded at the owner's request from a sidebar with the parent GPT while ADR-0035
was under review. The later ADR-0035 acceptance assigns this proposal to ADR-0036
design next; that assignment does not authorize starting design or implementation
before the owner-controlled cadence, or change ordinary arithmetic. This document
records the captured proposal; examples below remain illustrative.

## Motivation

When a total is divided among recipients, individually settled amounts should add
up exactly to that total at the specified settlement precision. Displaying three
amounts as 3.33 while displaying their sum as 10.00 leaves the user to reconcile
the visible/settled discrepancy. Rounding each share separately can instead leave
a real undistributed unit.

For a 10.00 total split three ways at cent precision, a valid allocation is 3.34,
3.33 and 3.33, summing exactly to 10.00. Allocation should be explicit rather than
inferred from several independent division formulas.

## Proposed invariant

For one complete allocation specification and every recipient index exactly once:

- Each result is an exact multiple of the requested settlement unit.
- The sum of the allocated results equals the original representable total exactly.
- Results and tie-breaking are deterministic across platforms.

The function cannot guarantee reconciliation if users omit recipients, duplicate
indices, or use inconsistent totals/counts/precision across formulas. Do not imply
that Pivot detects or silently repairs such groups.

## Initial direction: equal scalar allocation

Start with a deterministic scalar function for equal shares, without requiring
spill ranges or dynamic arrays. Illustrative syntax, **not a final API**:

`ALLOCATE(total, recipient_count, recipient_index, decimal_places)`

| Formula | Illustrative result |
| --- | ---: |
| `=ALLOCATE(10,3,1,2)` | 3.34 |
| `=ALLOCATE(10,3,2,2)` | 3.33 |
| `=ALLOCATE(10,3,3,2)` | 3.33 |
| Sum of those three results | 10.00 |

Convert a representable total to integral settlement units, compute whole-unit
shares, and distribute remaining units by a documented recipient order. This is
the equal-weight case of largest-remainder allocation. No binary floating-point
arithmetic is needed. Using exact Decimal96 values does not remove the need to
specify allocation and indivisible settlement units explicitly.

Ordinary division remains mathematical division; ROUND remains individual rounding.
Neither automatically assigns residual units. Do not alter identical independent
formulas, infer an allocation group from their placement, or hide additional
precision behind formatting as a substitute for settled results.

## Questions for a future ADR

- Final function signature, argument order and terminology, evaluated against
  alternatives. Settlement precision must be explicit and required; no implicit
  two-decimal default is permitted.
- Valid recipient count/index bounds, integer validation, one-based ordering and
  behavior for zero recipients, invalid indices, nonnumeric inputs and errors.
- Negative totals and signed zero: define exact, symmetric or otherwise explicitly
  documented residual distribution and sign behavior without ambiguous truncation.
- Representability: reject a total such as 10.005 when allocating whole cents unless
  a separately explicit policy first changes the total. No silent rounding of the
  obligation to fit the requested precision.
- Decimal96 scale/range and intermediate overflow: scaling must be checked even where
  the final shares fit. Resource and execution bounds need explicit admission.
- Deterministic tie-breaking and its relationship to stable recipient indices,
  formula copying, sorting, row insertion/deletion and range identities.
- Independent conservation, unit-multiple, determinism, sign, boundary and error
  oracles; persistence and native-platform qualification under the normal ADR cadence.

## Deferred extensions

Weighted allocation can use largest remainders: compute each recipient's whole-unit
share, rank fractional remainders and distribute residual units using deterministic
ties. Weight validation, zero/negative weights, exact comparisons, range membership
and recipient identity require their own design. The sidebar's weighted-range syntax
is exploratory and leaves recipient-index/precision arguments unsettled.

A multi-output allocation computation could eventually represent the group directly,
but requires graph/output and authoring design. It is not a prerequisite for the
initial scalar proposal. Rotation or cumulative balancing across repeated allocations
would require explicit period/ordering inputs or state; defer it rather than adding
hidden history. A fixed tie order does not promise fairness over repeated allocations.

Potential uses include invoice discounts, shared expenses, tax amounts, payroll and
cash reconciliation. This proposal records intent, not accounting policy or a change
to Pivot's canonical numeric, reference, filtering or history semantics.

## Parent follow-up: required precision and next assignment

The parent assigns ADR-0036: Exact, Conservation-Preserving Allocation for design
and parent review after owner commit/push and explicit authorization. The starting
proposal lives here in `docs/proposals/allocate.md`; the review's reference to
`docs/architecture/allocate.md` identifies this existing document, not a separate file.
No move, duplicate proposal, ADR-0036 draft or implementation is created at closeout.

The owner's follow-up and parent response require `decimal_places` (the fourth
argument in the illustrative interface). Zero means whole units, 2 hundredths,
3 thousandths and 4 ten-thousandths. Do not silently assume 2. The total must already
be exactly representable at that precision: `ALLOCATE(10.005,3,1,2)` must error rather
than silently discard the fractional cent. The overall signature remains subject
to design review; the requirement for explicit precision does not.

The first milestone considers equal-weight scalar allocation and deterministic
recipient ordering, with exact unit multiples and conservation across a complete
consistent recipient set. Specify negatives/signed zero, all invalid inputs,
overflow/admission, deterministic parity and compatibility; compare scalar and
multi-output alternatives. Independent calls cannot prove users supplied every
recipient exactly once. Weighted allocations, spills, rotation, persistent allocation
groups, financial reporting and automatic formula repair are deferred.

## Owner-authorized design follow-up

The owner subsequently authorized ADR-0036 design. See the
[proposed ADR](../adr/0036-exact-conservation-preserving-allocation.md) and
[design handoff](../handoffs/2026-10-31-exact-allocation-design.md). They propose concrete
strict-operand, u32 index, sign/zero, fixed-integer and exact-reconciliation admission
rules for parent review. The required precision clarification remains in force.
This proposal is retained as provenance; the linked design is not implementation
approval. No feature work or new platform execution has started.

## Parent design acceptance

The parent approved ADR-0036 with mandatory production-SUM coverage in different
orders, index-independent group validity/refusal and an explicit complete/consistent
recipient-set boundary. Strict Decimal arguments, required precision, deterministic
residuals, fixed 192-bit arithmetic and the proper-partial-sum guard are accepted.
No numeric-only attended GUI/AOT round is required; reassess if sensitive paths change.
No implementation is authorized by this design acceptance. Weighted allocation remains
an unassigned nice-to-have, outside current scope. Wait for parent direction after
ADR-0036 before assigning any next milestone or ADR number.

## Implementation acceptance — 2026-10-31

ADR-0036 is accepted and closed by parent implementation review. This proposal and
its earlier approval/authorization statements are historical provenance; see the
[accepted implementation handoff](../handoffs/2026-10-31-exact-allocation.md).
The parent recommends ADR-0037 weighted allocation as a next design candidate only.
Neither that design nor implementation is authorized yet. Stop for owner commit/push
and explicit next-work authorization.

## Weighted design follow-up

The owner subsequently authorized ADR-0037 design. See the
[proposed weighted ADR](../adr/0037-weighted-conservation-preserving-allocation.md)
and [parent handoff](../handoffs/2026-10-31-weighted-allocation-design.md).
Its explicit scalar-list association, cap and metered exact arithmetic are proposals;
no weighted implementation is authorized yet. Earlier candidate/authorization statements
above retain the chronology of the original equal-allocation proposal.

ADR-0037 design is now approved with four clarifications: positional association,
final-award representation/SUM, fully admitted cancellable execution, and consistent
arity enforcement. The linked ADR records the accepted technical contract and proofs
required before implementation acceptance. This update contains no implementation.

ADR-0037 implementation and validation completed on 2026-11-01; see the
[implementation review](../handoffs/2026-11-01-weighted-allocation.md).
ADR-0037 is accepted and closed. This proposal is retained as technical provenance.

The next design direction, ADR-0038, concerns ordered range-based weights with explicit
Blank-position and recipient-association semantics, reusing the accepted arithmetic.
No range design or implementation is included in this proposal update.

ADR-0038 now records a proposed [range-based design](../adr/0038-range-based-weighted-allocation.md),
with a separate range function, vector ordering, strict Blank positions and explicit
representation/resource decisions. See the
[design review](../handoffs/2026-11-01-range-based-weighted-allocation-design.md).
No range implementation accompanies this proposal.

ADR-0038 design is approved with four clarifications: positional multiplicity, bounded
invalid-geometry dependencies and reliable restoration, range-operand authority, and
an explicit WB13-only compatibility cut. The linked ADR records the approved contract
and six-target headless/managed/AOT qualification scope. No implementation has started.
