# Document-owned worksheet text

ADR-0018 is accepted; the bounded implementation passes Windows/Linux/native Mac
managed and Native AOT validation. Implementation and escape-overhead clarification are accepted; the milestone is closed. [Semantics](../adr/0018-document-owned-worksheet-text.md),
[implementation and verification](../handoffs/2026-10-21-worksheet-text-implementation.md).

LiteralText is a canonical worksheet content variant, distinct from TextFormula
(which retains numeric formula source). Text uses normal CellId/revision/capture
and checkpoint semantics. Its UTF-8 payload lives in document/workbook generations;
the numeric graph has only a payload-free NonNumeric definition at the stable node.

The marker is valid, snapshot-safe content. Evaluating it numerically produces
NonNumericInput; scalar and SUM consumers propagate that origin, and later numeric
replacement/clear invalidates dependents. Display reads do not calculate text cells.
There is no string runtime value or synthetic invalid formula. Frozen hosted scalar
exports/codecs reject the marker; the hosted v1 schema is unchanged.

Rust owns entry classification, including exact numeric rejection and apostrophe
escapes. Text editing reconstructs an escape; native typed paste preserves payloads
without reclassification. TSV escapes text reversibly. The 4,096-byte payload bound
admits one extra escape byte as syntax, counted in aggregate budgets; this explicit
implementation clarification is accepted. Existing capacity limits otherwise remain.

WB4 validates document content against graph definitions. The private session
request/response text extension was introduced in version 2; the current envelope
is version 3 for the [projection proof](structured-results-and-projections.md),
retaining typed text and entry arms. Clipboard data and
platform identifiers are version 2. Old WB1–WB3 Open leaves live state untouched.
No strings in formulas, connectors, projections or permanent dynamic namespace are
introduced. Native AOT remains a separate regression gate.


ADR-0021 adds a distinct Blank definition and text-ignoring MIN/MAX/AVERAGE/COUNT
without changing strict scalar/SUM text errors. Current saves use WB5 with validated
legacy WB4 reading; [aggregate architecture](spreadsheet-aggregates.md).
