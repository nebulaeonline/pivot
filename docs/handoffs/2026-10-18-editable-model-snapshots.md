# Pivot Handoff — Editable models and immutable snapshots

Date: 2026-10-18
Branch / commit: uncommitted
Status: Complete

## Objective

Implement M1 named inputs, computed nodes, and explicit dependencies using the
owner-approved editable-model / immutable-calculation-snapshot design.

## Implemented

- `pivot-core::model`: editable definitions, process-local stable opaque node IDs,
  revision identities, and immutable owned snapshots.
- Snapshot name binding, deduplicated dependencies, reverse dependencies,
  deterministic dependency ordering, and cycle rejection.
- ADR-0002, architecture notes, and status updates.

## Interface / semantics

Snapshots capture current input values and definitions and remain unchanged after
edits or deletion. Working models may be incomplete; snapshot validation returns
typed errors. Mutations require exclusive access; capture requires shared access.
No external locking, execution cache, or scheduling framework is introduced.

Renames preserve node IDs but do not rewrite formulas. Formula names bind at each
capture; deleting and recreating a name binds future snapshots to the new ID.
This initial editing behavior is documented rather than a promise of spreadsheet
reference semantics. Snapshot storage is private and replaceable.

## Tests actually run

- `cargo test --workspace` — PASS: eleven tests, including four new integration
  tests covering financial model bindings, old snapshots after edits, forward and
  missing references, invalid syntax, cycles, identity isolation, deletion, rejected
  mutations, and snapshot use from another thread.
- `cargo clippy --workspace --all-targets -- -D warnings` — PASS.
- `cargo fmt --all -- --check` — PASS.

Initial tests passed but lint rejected an atomic API deprecated by the installed
nightly. Identity allocation now uses a checked compare/exchange loop.

## Decisions made or proposed

- **Accepted:** editable models and immutable per-calculation snapshots (ADR-0002).
- **Implementation choices:** copy during capture, private shared snapshot storage,
  case-sensitive names, and first-error validation. These can evolve.

## Open issues / risks

No evaluator, calculation results, dirty propagation, input overrides, explicit
output ports, or policy capture exists yet. IDs and revisions are not persistence
contracts. Capture is synchronous and not optimized for large models. Cycle reports
include downstream blocked nodes. Model size limits and multi-edit transactions
remain future work.

## Questions for ChatGPT / owner

No blocking questions. Review name-edit semantics before introducing workbook
structural edits or reference-preserving rename.

## Next recommended slice

Add an evaluation context over one snapshot, with typed arithmetic errors and
results carrying the snapshot revision. Keep caches local to the calculation.
