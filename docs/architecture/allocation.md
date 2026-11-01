# Exact equal allocation

Accepted and closed under ADR-0036 by parent implementation review, 2026-10-31.

`ALLOCATE(total, recipient_count, recipient_index, decimal_places)` returns one
settled share of an equal allocation. All four arguments are required Decimal
values. Blank, String and Boolean operands are type errors. Count and index must
be integers from 1 through 4,294,967,295, with index no greater than count.
Precision must be an integer from 0 through 28; formatting never supplies it.

For example, `ALLOCATE(10,3,1,2)` returns 3.34; indices 2 and 3 return 3.33.
Negative totals use the same distribution with reversed signs. Calculated zero
is positive and normalized. Results do not retain a mandatory display scale.

The total must be an exact multiple of the settlement unit. For example,
`ALLOCATE(1.001,3,1,2)` produces InvalidArgument rather than rounding the total.
Nonintegral or out-of-range metadata also produces InvalidArgument. Representability
failure produces Overflow (the session exposes the existing NumericError category).

The implementation divides the absolute integer settlement units by the count.
The first remainder-many indices receive one additional unit. Both actually used
shares must fit Decimal96, and the largest proper partial sum must fit at their
common normalized scale. Every valid index receives the same admission result.
The complete sum may normalize further: the admission rule does not incorrectly
require the full unnormalized total to fit that common-scale coefficient.

Conservation requires every index exactly once, with consistent total, count and
precision. Independent calls do not validate a group's completeness or uniqueness.
Copy, Fill, sorting and row/column operations retain explicit formulas and normal
reference behavior; they do not assign, renumber or repair recipient indices.

Arithmetic uses six fixed u32 limbs, without runtime BigInteger, binary floating
point, recipient enumeration or group state. Explicit scratch/control accounting
is bounded by 256 bytes and limb visits by 1,024 per call; compiler stack frames
and the existing evaluator argument stack are outside that accounting. Production
SUM, division, rounding, formatting and resource limits are unchanged.

Typed opcode 19 uses the existing WB12, private session v13, hosted v1 and CB5/CB6
encodings. No protocol version or canonical value type changes. Readers that do
not recognize the opcode must refuse execution under existing validation rules.

See [ADR-0036](../adr/0036-exact-conservation-preserving-allocation.md) for the
admission proof, error ordering and approved validation scope.

## Understandable settlement intent

The four arguments answer practical questions:

| Argument | Question |
| --- | --- |
| total | How much am I dividing? |
| recipient_count | How many ways? |
| recipient_index | Which share do I want? |
| decimal_places | How many decimal places? |

A user can split 10 among three recipients at cent precision without constructing
residual arithmetic. The resulting 3.34, 3.33 and 3.33 are actual values for later
calculations, copy and export, rather than a display-rounding effect. The total must
already be exactly expressible in the requested settlement units, and the formulas
must specify a complete consistent group to conserve it.

The accompanying owner/parent discussion establishes the design principle: make
correct settlement operations easy to understand for ordinary spreadsheet users,
with numerical complexity handled by the engine. See the
[accepted implementation handoff](../handoffs/2026-10-31-exact-allocation.md) for
validation and the accepted structural Blank-anchor correction. No further validation
or new graphical/AOT round is required by this acceptance.
