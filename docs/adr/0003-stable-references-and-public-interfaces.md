# ADR-0003: Stable references and public model interfaces

Status: Accepted
Date: 2026-10-18

## Context

The owner authorized explicit model interfaces and requires formulas to use stable
node identity internally. Pivot must accommodate numeric types beyond decimals.
Spreadsheet addresses such as B22 are appropriate frontend labels, not engine IDs.

## Decision

Maintain a name-to-NodeId directory for command/formula submission. Bind formula
names when adding or editing formulas and retain the resulting ID references.
Snapshot capture and execution do not look up formula names again. Renaming a node
does not change existing dependencies; deletion leaves a missing stable reference.

Public model ports are explicit declarations with stable identities independent of
their names. Bind them to node IDs, keeping public labels separate from internal
names. Named invocation uses only declared ports; the internal NodeId API remains.
Use extensible tagged values and explicit type metadata at the public boundary.
Additional types require deliberate coercion rules; never implicitly route decimal
calculations through IEEE-754 values.

Spreadsheet addressing belongs in a future workbook projection directory mapping
sheet/row/column coordinates to stable IDs. The Avalonia client should default to
traditional spreadsheet addresses. No spreadsheet projection is implemented here.

## Alternatives considered

- Rebind stored formula names at every snapshot: renames break formulas and reused
  names can silently redirect references.
- Expose every graph node automatically: prevents a stable, encapsulated model API.
- Use spreadsheet coordinates as graph IDs: couples reusable models to one frontend.

## Consequences

Source text is editing provenance and can contain an old node label after rename;
the bound ID is authoritative. Automatic formula display rewriting is future work.
Unresolved or invalid edits can be retained but require explicit formula resubmission
after correction; snapshot creation does not perform deferred name resolution.

Initial ports and execution are decimal-only. Tagged value/type enums are
non-exhaustive so clients must allow future variants. Float behavior and mixed-type
evaluation remain undecided. Port IDs are process-local and not wire identifiers.

## Verification / reconsideration trigger

Test renamed dependencies, deleted/recreated names, external labels different from
graph labels, required/default inputs, port validation, and independent invocations.
Review workbook structural editing and numeric promotion before extending them.
