# Range-based exact weighted allocation

Accepted and closed under ADR-0038 by implementation review, 2026-11-01.
The Windows ARM64 cold-build timing issue remains unresolved; the accepted
retained-cache evidence and other qualifications are recorded in the
[implementation handoff](../handoffs/2026-11-01-range-based-weighted-allocation.md).

`ALLOCATEWEIGHTEDRANGE(total, recipient_index, decimal_places, weight_range)`
uses the unchanged exact weighted allocator with a literal one-row or one-column
A1 rectangle containing 1..64 physical positions. A singleton must use `B2:B2`.
Reversed endpoints retain ascending coordinate order. An optional worksheet qualifier
belongs on the first endpoint. Scalar expressions, unions and computed range arguments
are not supported in the fourth position.

Every position contributes exactly one weight. Missing, cleared and calculated Blank
positions, Strings (including empty or numeric-looking Strings), and Booleans produce
TypeMismatch. Zero is a valid explicit weight; negative weights and all-zero groups
produce InvalidArgument. Metadata is checked left to right before range resolution,
then geometry and each positional weight, then the existing group-wide admission.
IF/AND/OR retain lazy evaluation and dependency errors retain their source.

Identical live rectangles may share a descriptor; retired identities are never reused.
This shares geometry, never an allocation result or numeric invocation.

The rectangle has spatial SelectionId affinity. Insertion at or before its first axis
shifts it; insertion inside expands it and includes the newly inserted positions.
Deletion contracts it or retires the whole reference. Absolute markers control Copy
and Fill translation, not structural membership. Sorting weight cells changes their
positional order; filtering never removes hidden positions. Explicit recipient indices
are not repaired. Complete consistent index membership remains the author's responsibility.

The accessor returns dense optional slots, preserving both holes and repeated node
identities. Dependency sets may deduplicate identities. Invalid geometry retains its
descriptor subscription but creates no member data edges and traverses no members.
Geometry restoration rebuilds edges and cycle-validates the candidate before publication;
a restored cycle rejects the candidate atomically. Immutable captures retain their own
geometry, membership and values.

An independent 8-KiB adapter reservation covers its bounded slot vector, numeric input
vector, frame and control allowance. The existing 16-KiB numeric reservation, fixed
limb algorithm, `4 + 16N` numeric visit charge and `4096 + 16384N` limb bound are
unchanged. Actual descriptor, row and column AVL node visits consume the invocation's
budget. Collection and calculation share the evaluator task stack, cancellation,
allocation domain and immutable capture. Calculated Blank is checked before the
ordinary formula-result Blank-to-zero boundary; ordinary formula behavior is unchanged.

Operation 21 requires exactly three scalar roots and an authorized range operand or
valid deleted-range fault. Four scalars cannot substitute for a range, and a range
operand cannot escape into another scalar operation. Graph tag 8 and workbook tag 9
carry the explicit operand. WB13 is the only current native format; WB12 is refused,
with no migration reader. Private session v13, hosted v1 and CB5/CB6 retain their
existing representations: source, effective endpoints and fault provenance suffice.
Experimental history support is not added by this feature.

See [ADR-0038](../adr/0038-range-based-weighted-allocation.md) and the unchanged
[weighted arithmetic](weighted-allocation.md).
