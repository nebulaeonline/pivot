# Pivot Handoff — Public model interfaces and stable references

Date: 2026-10-18
Branch / commit: uncommitted
Status: Complete

## Objective

Implement public model ports and named invocation, respecting the owner's
requirements for stable formula identity, name directories, and future numeric
types beyond Decimal. Preserve spreadsheet addressing as a frontend projection.

## Implemented

- `pivot-types`: non-exhaustive `PivotValue` and `ValueType` tagged boundaries.
- `pivot-core::interface`: stable PortIds, required/optional input declarations,
  output declarations, named requests/results, public errors, and metadata.
- Working-model port addition, rename, removal, and snapshot validation.
- Public invocation uses declared labels; internal NodeId invocation is preserved.
- Formula parsing/binding moved to edit submission. Snapshot capture validates
  stored references without name lookup or recompilation.
- ADR-0003 and public-interface documentation; corrected current architecture notes.

## Interface / semantics

Input ports bind only input nodes; outputs may bind any node. Labels differ from
internal names. Optional defaults are explicit interface values; required ports
must be supplied even if unused by requested outputs. One public namespace forbids
duplicate names across inputs/outputs. Duplicate input target aliases are rejected
to avoid conflicting overrides. Multiple output labels may share a node.

Port renames preserve identity. Named results expose only requested public labels,
public errors, effective public bindings, and the source revision. They do not
return internal graph nodes or IDs. Internal invocation retains detailed diagnostics.

Graph name directories resolve submitted formula text to stable IDs. Renaming never
changes bound dependencies. Deletion produces a missing-reference error even if
the name is recreated. Invalid/unresolved edits remain stored; forward references
require explicit resubmission after targets exist. Formula source text may retain
historical labels, while bound tokens remain authoritative.

## Tests actually run

- `cargo test --workspace --quiet` — PASS: twenty-two tests.
- `cargo clippy --workspace --all-targets -- -D warnings` — PASS.
- `cargo fmt --all -- --check` — PASS.

Three new tests cover named invocation with different public/internal names,
required and default inputs, public output selection, concurrent isolated calls,
working-model edits and deletion, snapshot revisions, stable port rename, duplicate
declarations and requests, invalid targets, missing/unknown names, stable formula
rename behavior, and deleted/recreated graph labels. Existing model tests were
updated for the explicitly changed edit-time binding semantics.

No numeric arithmetic changed; cross-language decimal checks and .NET/AOT builds
were not rerun. No new dependencies were added.

## Decisions made or proposed

- **Accepted:** owner-directed stable-ID formula references, separate name lookup
  boundary, extensible types, public model interfaces, and eventual spreadsheet
  coordinate projection recorded in ADR-0003.
- **Implementation choices:** explicit optional defaults, all-required-input
  enforcement, case-sensitive ASCII labels, shared input/output port namespace,
  and no conflicting input aliases. These are documented, not permanent wire rules.

## Open issues / risks

Only Decimal values execute today. IEEE-754 variants, coercion, NaN/infinity, and
mixed-type arithmetic need defined semantics. Type mismatch paths are structured
for extension but cannot be exercised with a second value type yet. Snapshot
validation is still global. Port IDs and NodeIds are process-local. Source display
rewriting and automatic forward-reference repair are not implemented. Public
dependency errors hide internal root IDs; richer public diagnostics are future work.

No workbook projection, Avalonia integration, transport, persistence, serialization,
publishing, or incremental recalculation was added.

## Questions for ChatGPT / owner

No blocking questions. Confirm interface defaults and alias restrictions before
making them part of a versioned external contract. Design float/coercion behavior
separately when introducing the next executable numeric type.

## Next recommended slice

Review the public invocation contract, then define portable values and versioned
protocol requests. Do not expose process-local IDs as portable identities.
