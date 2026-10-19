# Spreadsheet scalar formulas — Slice B

`pivot-core::expression::parse_spreadsheet` uses the existing Pratt parser, decimal
parser, precedence rules and resource bounds. `parse` retains the standalone model
grammar: `A1` remains a name there and a leading `=` remains invalid.

## Current grammar

```text
formula    = whitespace, ["="], expression, whitespace
expression = decimal | reference | unary | binary | "(", expression, ")"
reference  = ASCII-letter+, nonzero-digit, digit*
unary      = ("+" | "-"), expression
binary     = expression, ("+" | "-" | "*" | "/"), expression
```

The grammar above abbreviates precedence: unary signs bind most tightly, then
multiplication/division, then addition/subtraction; binary operators associate
left. ASCII whitespace is allowed between tokens. Column letters are case
insensitive; rows and columns must fit nonzero u32 coordinates and the worksheet's
configured dimensions. Leading row zeros are rejected. Literals use existing exact
decimal parsing (96-bit coefficient, scale 0–28); no exponent or locale syntax.
Bounds remain 4,096 source bytes and 64 recursive expression levels.

`Expr::CellReference { row, column, span }` is a typed AST element. Spans are
half-open UTF-8 byte ranges into the original source, including any leading spaces
and `=`. References are never rewritten into generated variable names.

## Submission and binding

Submit `Edit::SetFormulaText { position, source }` through `Workbook::apply` with
its expected session/state/graph tag. Edits execute sequentially on one private
candidate. A formula therefore sees preceding edits in the same batch. A later
move follows the stable identity already selected; a later retirement breaks it.
Any hard failure discards the entire candidate, including demanded blank anchors.

Parsing and coordinate validation complete before target allocation. In-bounds
never-materialized references demand explicit Blank anchors, retaining Slice A's
provisional arithmetic-zero backing. Parsed expressions lower to postfix tokens;
Rust resolution maps coordinates to stable CellRef and GraphNodeId values, then
installs validated runtime references through the internal trusted transfer seam.
Evaluation uses the existing core snapshot/evaluator. No client text can supply
runtime IDs through this text-edit path. Typed Slice A descriptors remain a trusted
in-process proof facility, with no new protocol exposure.

`Content::TextFormula` retains original_source, each reference's source span,
submitted_position, stable target/node, diagnostic, transaction_base, edit_ordinal
and resolved_graph. The base Expected tag identifies the checked transaction;
the zero-based ordinal identifies its sequential worksheet candidate and
resolved_graph identifies the graph revision after output/reference anchor
materialization and immediately before installing the resolved formula. Preceding
graph edits and new anchors can advance it, while positional moves need not. Internal
normalized tokens are private. No public prepared-binding object can be installed
later against another state.

Moving a target or reusing its old coordinate never reparses the formula. Explicit
resubmission always resolves the current candidate, even when text is identical.
It creates new binding provenance and advances state. Ordinary unchanged inputs
and the existing typed no-op rules retain their earlier behavior.

## Drafts and presentation

Malformed syntax, unrepresentable literals and out-of-bounds references are editable,
non-executable formula drafts. Workbook diagnostics retain the precise parser or
coordinate error offset. Invalid expressions install a core computed parse-failure
marker, never an input zero. That narrow internal setter validates a live owned
output, source length and UTF-8 offset before mutation. It reuses the existing
core transfer error category; richer workbook diagnostics are not serialized and
no hosted codec/error-code table changes were made.

A retired target retains its old stable identity and source span. Validation
reports both MissingReference and a located Formula diagnostic. A newly created
cell at that address does not repair an old binding. Self/dependency cycles remain
editable but prevent capture. Structural errors still block the entire snapshot;
independent structural error isolation is deferred. Stale states, invalid edit
positions, ownership violations, malformed trusted stacks and quota failures reject
whole batches. All retained reference metadata participates in the logical byte
estimate; estimates exclude core/snapshot allocator overhead.

The API deliberately labels text original_source. It is not a rendering of current
effective coordinates. Clients must use stable targets and current cell views to
explain effective references; a complete formula renderer is deferred. Submitted
positions remain provenance and must not override the targets after a move.

## Verification and limits

See the [Slice B handoff](../handoffs/2026-10-19-spreadsheet-formulas-slice-b.md)
for platform reports, independent semantic tests and small cost observations.
Windows full CI (209 Rust tests), WSL2 Rust/IPC (210 tests) and native macOS ARM64
full CI (210 tests, including .NET/parity/IPC) pass. The Mac built the current SMB
working tree from a verified local APFS copy; see the
[macOS review](../handoffs/2026-10-19-macos-validation.md).
At 513 cells, its debug sample observed 1,572 ns parsing, 2,572 ns resolution,
386 us formula edit, 431 us input edit and 1,770 us capture. These observations
support the existing full-copy cost concern, without a platform speed ranking.
The cost harness times the actual parser separately from AST lowering and stable
reference resolution on existing anchors. Full formula/input edits include staging
and quotas; capture/evaluation are timed separately. Resolution microbenchmarks
exclude first-use blank allocation, installation and retained presentation metadata.

No absolute/mixed references, copy/fill, functions, names, ranges, sheet qualifiers,
multiple sheets, text/error value execution, incremental calculation, workbook
serialization, document behavior changes, RPC, C# client or UI is added.
