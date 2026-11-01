# ADR-0036 — exact allocation design handoff

2026-10-31. Design approved with three clarifications by parent review. Owner authorized
design only; implementation requires separate authorization.

The parent approves the signature, strict operands, u32 indices, required precision,
fixed arithmetic, group-wide exact-SUM admission and unchanged representation proposal.
Three mandatory clarifications apply:

1. Verify accepted groups through actual production SUM in different accumulation
   orders; independent oracles/algebra do not substitute for evaluator coverage.
   Diagnose arithmetic boundaries without silently weakening reconciliation.
2. All valid indices for one specification must receive the same group-representability
   outcome. Preserve InvalidArgument versus Overflow, dependency faults and argument order.
3. Conservation assumes a complete consistent index set exactly once. Scalar calls
   cannot certify membership; no hidden registration or automatic index repair.

No feature implementation, new benchmark or native validation has run.

Read [ADR-0036](../adr/0036-exact-conservation-preserving-allocation.md) for the complete
contract and verification plan. The starting [proposal](../proposals/allocate.md)
remains in proposals. ADR-0035 is accepted/closed; ADR-0023 remains parked at G3.

## Approved function

`ALLOCATE(total, recipient_count, recipient_index, decimal_places)`

Four required scalar arguments, all actual Decimal values. Count is 1..4,294,967,295;
index is one-based within that count; precision is an integer 0..28 with no default.
Integer-valued decimals with trailing zeros are valid. String, Boolean and Blank are
TypeMismatch, including a Blank precision reference; do not change other functions'
existing Blank conversion rules. The parent approves this strict operand policy. Negative precision and fractional/nonpositive/out-of-range count/index are
InvalidArgument, with full Decimal96 validation before machine narrowing.

The total must be exactly representable at the requested unit. `10.005` at precision
2 errors, without rounding the obligation. Convert its magnitude to exact integer
units U, compute Q=U div N and R=U mod N, and assign Q+1 units to indices 1..R and Q
to the remainder. Restore sign on nonzero shares; normalize all zero results to
positive scale-zero Decimal96. Negative allocations use the same residual recipients.

10.00 split three ways at p=2 gives 3.34, 3.33, 3.33; -10.00 gives -3.34, -3.33,
-3.33. At p=0, 10 splits as 4, 3, 3. Ties depend only on explicit recipient indices,
not coordinates, visibility or evaluation order. The function remains stateless.

For a complete consistently specified index set exactly once,
`R*(Q+1)+(N-R)*Q=U`, and every result is an exact settlement-unit multiple. Independent
scalar calls cannot prove group completeness or consistent parameters. Copy/Fill,
sorting and row changes retain ordinary binding semantics without renumbering indices
or repairing groups. Formatting remains separate: hiding settled digits with a lower-
precision display can still make displayed text fail to reconcile.

## Additional representability decision: protect actual SUM reconciliation

Checking only the requested result is insufficient. All shares that actually occur
must fit Decimal96, with the same group-wide error for every valid index. Check Q
always, and Q+1 only when R>0; an unused higher share must not cause false overflow.

There is a second issue: individually fitting shares can have nonrepresentable
intermediate sums under ordinary Decimal96 arithmetic. The parent approves the exact-reconciliation
admission guard without changing SUM or quietly weakening the allocation promise.

Let t be the maximum scale of the normalized used nonzero shares, and compute
`G=(U-Q)/10^(p-t)`. Q is the smallest share, so G is the largest proper partial sum's
coefficient at the common scale. Require exact division and G<=2^96-1. Every proper
subset then fits exactly; the complete sum is the already-representable input total
and may normalize to a lower scale on the last addition. Existing SUM can reconcile
accepted groups in any order, subject to its unchanged resource/error rules.

With M=2^96-1:

- `(M,1,1,28)` succeeds; wide scaled units do not imply output overflow.
- `(M,3,i,28)` succeeds with exact integer M/3; do not check unused Q+1.
- `(10,3,i,28)` succeeds: proper partials fit and the complete sum normalizes to 10.
- `(M,2,i,1)` overflows because the required shares cannot be represented exactly.
- `(M,10,i,1)` overflows although each share fits: some proper partial sums do not.
- `(M-9,11,i,2)` overflows for every valid index, including the lower-share recipient,
  because one of the other actually used shares is unrepresentable.

This is an approved domain restriction. Production SUM coverage is mandatory; do not
silently remove the guard or narrow the reconciliation guarantee. The fixed-smallest-
unit, u32-maximum-count case remains efficient: one recipient gets a unit, others zero.

## Algorithm, errors and representation

A fixed six-u32-limb helper suffices: a 96-bit coefficient multiplied by at most 10^28
fits below 2^190. Six high-to-low divisions by a one-word count use bounded u64
accumulators. Strip exact trailing zeros to pack used shares into Decimal96; compute
U-Q and the common-scale guard in the fixed buffer. Never use rounded Decimal96
intermediates, binary floats, recipient enumeration or a runtime BigInteger dependency.
Approved bounds are 1,024 limb visits and 256 bytes explicit fixed scratch per helper
call, independent of N. Keep existing evaluation/admission/cancellation limits.

Demand and locally validate arguments left to right. After all four succeed, check
index<=count, total unit representability, then group admission. Preserve dependency
fault origins. Zero total does not bypass validation; inactive syntactically valid
IF/AND/OR branches do not evaluate ALLOCATE. Use existing InvalidArgument, TypeMismatch
and arithmetic Overflow outcomes; no new wire error category or value type.

Recommend equal scalar evaluation first. A multi-output computation could express
membership, but adds output/group authoring, identity and lifecycle contracts and is
not equivalent to an existing scalar cell. Weighted allocation, spills, rotation,
persistent group state, financial reporting, automatic formula repair and history
remain outside this initial milestone.

Approve additive typed opcode 19 with arity four, using existing generic call/token
framing. Keep WB12, private v13, hosted v1 and CB5/CB6. Old documents remain readable;
older engines reject unsupported opcode/name behavior without executing a different
function. Sharing a format version does not promise support for every function.
No migration, obsolete reader, production C# evaluator or automatic cell format is
proposed. Revisit versions only for a demonstrated encoding change.

## Accepted validation and next step

The parent approves strict operands, u32 count/index bounds, normalized sign/zero,
residual ordering, the proper-partial-sum guard, fixed arithmetic bounds, unchanged
representations (subject to encoding confirmation) and this validation scope.

Require independent Python integer/Fraction and .NET BigInteger references, exhaustive
small-group conservation, algebraic large-count checks, all representability boundaries,
actual Pivot SUM in shuffled orders, invalid operands/arity, lazy and left-to-right
errors, carry/division tests and stable graph/copy/fill/sort/persistence behavior.

Measure helper batches separately from canonical evaluation and IPC. Proposed corpus:
16,384 base positions, 4,096 allocation formulas, four captures and declared extra
anchors; a complete 4,096-recipient group can use one bounded SUM output. Keep private
256-output requests and existing one-second engine, 256-MiB steady/512-MiB peak targets
on stated workloads. Label observed maxima and charged memory precisely.

Require local CI, all-feature compatibility/strict Clippy and all-six-RID native
headless parity, with macOS x64 explicitly Rosetta. No new attended desktop/
AOT GUI round is required solely for these numeric changes: ADR-0035 just qualified all six, while this design changes no interaction,
DTO/source-generated metadata or production C# computation. Reassess targeted attended
qualification if implementation changes those paths; normal local consent remains
mandatory. Retain original failures/source qualifications, batch archive verification
and pruning, and per-project cloc at implementation close.

Design is approved. Implementation still requires separate owner authorization under
the normal milestone cadence. Weighted allocation remains an unassigned nice-to-have;
wait for parent direction after ADR-0036 before assigning subsequent scope or an ADR
number. Any future weighted range membership/stable-index association and hidden state
would require separate design. ADR-0023 remains parked at G3.

## Owner implementation authorization

The owner subsequently authorized ADR-0036 implementation and validation. Numeric
implementation, independent oracle checks and resource qualification are in progress.
The design-only statements above describe the original handoff, rather than the
current work state. See [allocation semantics](../architecture/allocation.md).

Implementation and six-target headless validation subsequently completed. See the
[implementation handoff](2026-10-31-exact-allocation.md); parent implementation review
has accepted and closed ADR-0036. Earlier design-only/in-progress statements preserve handoff chronology.
