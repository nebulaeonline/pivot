# Worksheet text — design review decision

Date received: 2026-10-20. Parent review supplied by the user.
**Disposition: ADR-0018 approved with clarifications; implementation pending.**
[Accepted design](../adr/0018-document-owned-worksheet-text.md).

The parent authorizes the bounded document-owned worksheet-text milestone: entry,
editing, rendering, mixed rectangular clipboard and WB4 Save/Open through Rust.
Text remains normal stable cell content, distinct from computational strings.
Existing identity, revisions, atomic edits, immutable captures and dirty tracking remain.

## Required clarifications

- Rust owns entry classification. Decimal-shaped overflow/excess scale is rejected,
  never silently reclassified as text. Formula-shaped invalid input remains a draft.
- Apostrophes are reversible input syntax, not stored text. Strip exactly one on
  entry; always prepend one on text TSV export, including for apostrophe-leading
  payloads. A lone apostrophe clears; empty literal text is unsupported.
- Text is valid content. Numeric scalar references and SUM consuming text produce
  explicit type errors, with no zero coercion, skipping or synthetic parse error.
- The nonnumeric marker must remain a small compatible graph extension. Verify
  scalar/range invalidation, snapshots, restoration and unsupported hosted export
  rejection. Stop for a smaller revised design if broad evaluator/value redesign
  would be required.
- Retain single-line/no-controls text and the 4,096-byte UTF-8 admission ceiling,
  existing rectangular grammar and all existing admission/capacity ceilings.
- WB4 and new private application/clipboard versions are approved. WB1/WB2/WB3
  rejection must leave the open document, revision and dirty state untouched.
  Frozen hosted v1 remains unchanged; no migration readers or lossy exports.

## Verification and stop point

Test Unicode/emoji/combining marks, spaces, escapes, numeric/formula-looking text,
invalid numeric input, controls, UTF-8 boundaries and zero-length behavior. Cover
number/text/number transitions, scalar/SUM errors and downstream invalidation,
snapshot isolation, mixed and overlapping rectangles, rollback and stable identity.
Verify WB4 reopen, old-version rejection, draft/dirty-close flows and zero charge.

Run managed desktop acceptance and actual Native AOT publish/execute on Windows
x64, Linux x64 and native Mac ARM64/APFS. No recurring five-group manual replay;
retain targeted manual checks only for gaps in automated coverage.

Deliver the implementation/validation handoff and stop for parent review. No
computational strings, string formulas, dates, booleans, structured records,
projections, connectors, permanent `#` grammar, formatting, Undo/Redo, capacity
increase or installer work. Implementation acceptance has not been granted.
