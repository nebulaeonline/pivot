# Editable models and snapshots

`pivot_core::model::WorkingModel` supports named decimal inputs and computed
expressions. Add, edit, rename, and remove nodes through exclusive mutable access.
Node names use the parser's case-sensitive ASCII identifier grammar. IDs are opaque,
stable during a node's lifetime, scoped to one model, and never reused after removal.
They are process-local handles, not persistent IDs or a wire representation.

`snapshot()` captures input values and pre-bound definitions, validates stable
references, and derives deduplicated dependencies and reverse dependencies, and constructs
a deterministic dependency order. Unknown names, invalid expressions, and cycles
return typed errors. Cycle diagnostics include blocked downstream nodes, not only
the exact cycle. Validation currently returns the first failure.

Names in source formulas are resolved at submission, not snapshot capture. Bound
references survive renaming and cannot silently retarget when a name is reused.
See [public model interfaces](public-model-interfaces.md) and ADR-0003. This
supersedes the earlier provisional snapshot-time name rebinding behavior.

Computed expressions have a bound postfix representation for inspection. It is an
initial in-process API, not a serialized execution format. Read-only snapshot access
prevents changes to captured definitions. Snapshot clones share immutable data.

The working model stores input values for convenient editing. These are captured
values, not cached results. Public ports are implemented; calculation policy capture and dirty propagation
remain deferred. Per-invocation overrides and evaluation
contexts now execute one snapshot and retain its revision identity with results;
see [snapshot evaluation](snapshot-evaluation.md). Atomic multi-edit transactions and cross-process
revision identifiers remain unspecified.
