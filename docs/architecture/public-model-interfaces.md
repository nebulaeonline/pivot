# Public model interfaces

Working models expose explicit input/output declarations. `add_input_port` binds
a public label to an input NodeId with `Required` or `Optional { default }` semantics.
`add_output_port` can bind either an input or computed node. Ports receive stable
process-local PortIds; `rename_port` preserves identity and `remove_port` removes
the declaration. Snapshot capture copies declarations and validates their targets.

Public labels and graph labels have separate directories. Both currently use the
case-sensitive ASCII identifier grammar. Public inputs and outputs share one port
namespace; duplicate names are rejected. One input node may have only one public
input port to avoid conflicting overrides. Multiple output labels may target the
same graph node. Port deletion and recreation allocates a new identity.

## Invocation

`invoke_named(&NamedInvocationRequest)` accepts public input labels and selected
output labels. All public required inputs must be supplied, even if an output does
not depend on them. Optional defaults belong to the interface and are explicit;
they do not inherit the working node's captured value. Unexposed internal inputs
retain captured defaults. Unknown/duplicate input or output names are errors.

Named invocations return public output names mapped to tagged values or public
evaluation errors, the source snapshot revision, and effective public input values.
They do not expose the graph or internal node IDs through the result. Detailed
NodeId diagnostics remain available through the internal invocation API. Public
dependency failures currently report no internal root ID.

Validation order is supplied inputs, missing required inputs/defaults, then outputs.
Input declarations are traversed in label order. Empty output requests still require
valid complete public inputs. All execution remains deterministic and isolated.

`PivotValue` and `ValueType` are non-exhaustive tagged enums. Only Decimal is
implemented today; port type metadata is Decimal, matching current graph types.
Type validation is explicit. IEEE-754 float variants, NaN/infinity handling,
mixed-type promotion, and richer constraints require a separate design. No float
arithmetic or conversion was added as part of interface scaffolding.

## Name directories and stable references

The internal directory maps graph names to NodeIds at formula submission. Stored
formulas retain bound postfix tokens. Renames update the directory, not dependencies.
Deleted targets remain broken references even if the label is reused. Explicitly
editing the formula resolves its submitted names again. Snapshot capture checks
stored bindings and never reparses or rebinds formulas.

Incomplete formulas are still retained with a typed binding/parse error. Forward
references need explicit formula resubmission after the target exists. The displayed
source is the text originally submitted and can contain historical labels after
renames; clients must not treat it as the authoritative bound representation.

Future workbook directories should map scoped spreadsheet coordinates (for example
Sheet1!B22) to the same stable graph IDs, preserving the graph-first design. Neither
A1 parsing nor Avalonia integration is added in this slice.

Initial choices such as one public namespace, all-required-input validation, ASCII
labels, and duplicate-output rejection can be revisited before protocol design.
No serialization, publishing, or portable port identifier format is selected.
