# Initial workbook projection considerations

Status: Accepted ADR-0009 constraints for later work; not a workbook implementation.

A standalone computational model remains useful without a sheet. A workbook adds
stable WorksheetIds, sheet names, sparse cells, formats, resources and projection
bindings over one or more embedded graphs. A sheet need not equal a graph. Multiple
sheets may project one graph; model nodes may have no visible cell.

Represent a displayed address such as B22 as worksheet/address metadata mapping
to a stable graph/node binding. Bound computational formulas retain node identities.
Worksheet renames, row insertion, movement, copy/paste and relative references need
explicit projection edit semantics; they cannot be implemented by casually rebinding
formula source. Cell-to-node mapping must allow presentation-only/empty cells without
allocating a graph node for every grid position. Numeric cells participating in
calculation can project graph inputs; formula cells project computed nodes.

Spreadsheet syntax resolves addresses/ranges through the projection at explicit
edit submission, then delegates actual expression semantics and binding to the Rust
engine. It must not become an independent C# parser/evaluator. Extending current
identifier-only grammar for address syntax and ranges requires its own reviewed
design. Range references may need structural identity beyond per-cell nodes; avoid
promising that a million-cell range becomes a million scalar formula tokens.

Store sparse populated regions and presentation separately from graph definitions.
Leave room for chunked cell indexes, compressed repeated styles and independent
resource sections. Document-only changes, such as widths or formatting, advance
document state without invalidating calculations unless they explicitly affect
numeric/semantic settings. Sheet-name changes may require projection metadata
updates while retaining bound graph identity.

Range edits must stage atomically with affected graph/projection updates. Define
dependency invalidation, range membership changes and cycle behavior before adding
range operations. Current full snapshot/staging behavior is the correctness
baseline, not a proven large-workbook strategy. Incremental recalculation and
structural sharing remain separate measured work; calculations still observe an
immutable revision even while the document is edited.

Viewport requests should eventually use bulk bounded ranges, selected fields and
generation-tagged caches rather than one RPC per cell. Cache keys need SessionId,
document/projection state and graph/calculation provenance; stale results must not
overwrite a newer viewport. Save does not wait for every cell to calculate, and
cached results cannot substitute for saved definitions.

Embedded models are the initial offline composition direction. Future external
references must pin a specific version/digest and expose missing or stale content.
Automatic model fetching, graph-to-graph invocation, volatile functions, collaboration
and XLSX semantics are not established here. Keep separate backlog items for graph
APIs, sparse/range projection, bulk transport, incremental calculation and the
Avalonia virtualized control.
