# ALLOCATE: exact, conservation-preserving allocation

Date: 2026-10-31.
Status: **Proposal captured for future design; not approved or scheduled for implementation.**

Recorded at the owner's request from a sidebar with the parent GPT while ADR-0035
is under review. This does not interrupt ADR-0035, assign an ADR number, change
ordinary arithmetic, or authorize feature work. The [original sidebar](allocate-sidebar.txt)
is preserved verbatim alongside this summary; examples below remain illustrative.

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

- Final function signature, argument order, defaults and terminology; whether precision
  is expressed as decimal places or an explicit settlement unit.
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
