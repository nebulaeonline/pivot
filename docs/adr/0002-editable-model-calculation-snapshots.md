# ADR-0002: Editable models with immutable calculation snapshots

Status: Accepted
Date: 2026-10-18

## Context

The owner requires models to remain editable while calculations run. A result
must describe one consistent model state, rather than a changing mixture of edits.

## Decision

Keep a mutable working model. Capture an immutable snapshot for each calculation,
including input values and an opaque model-scoped revision identity. Edits after
capture do not alter the snapshot. Evaluation state and results belong to the
calculation, not to globally mutable caches on model nodes.

Use stable opaque node IDs independent of names and worksheet coordinates.
Resolve expression names against the captured model and derive explicit direct
dependencies from the bound expression. Validate snapshots before calculation.
The working model may contain incomplete formulas or references while being edited.

## Alternatives considered

- Freeze the working model during calculation: prevents continued editing.
- Require immutable models as the editing API: does not match the editing workflow.
- Read the live working model during execution: allows inconsistent results.

## Consequences

Calculation capture needs a consistent view. The current Rust API uses borrowing:
mutations require `&mut self`, while capture holds `&self`. Returned snapshots own
their data and can outlive the working model. Shared-service locking and transaction
boundaries remain future work.

The initial implementation copies definitions and input values per capture and
privately shares the completed snapshot through `Arc`. This is an implementation
choice, not a commitment to deep copies, persistent data structures, or a concurrency
strategy. Published model revisions remain a separate future concern.

## Verification / reconsideration trigger

Tests must show that edits and deletion do not change an existing snapshot, that
failed capture does not prevent further edits, and that snapshot revision identities
change after successful edits. Revisit storage sharing when profiling demonstrates
capture cost; preserve the snapshot behavior while changing its implementation.
