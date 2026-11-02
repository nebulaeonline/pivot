# Pivot

Pivot is a graph-based calculation platform with a spreadsheet desktop and a
headless daemon. Its aim is to make calculations useful as both interactive
documents and callable models: financial models, analytical tools, scenario
calculations and application logic can share the same computation core.

## The calculation engine

The engine is written in **Rust**. A model consists of named inputs, computed
nodes and explicit dependencies. Expressions are parsed and bound to stable node
identities; display names and worksheet addresses provide ways to refer to those
identities at the client boundary.

Models remain editable. Each calculation captures an **immutable snapshot**, so
its inputs and dependencies stay consistent while later edits continue. Persistent
paged data structures share unchanged state between versions, supporting captures
and atomic edits without copying the entire model for every change.

Decimal calculations use a **96-bit significand and scale from 0 to 28**. Operations
check overflow and division by zero, and explicit rounding exposes multiple modes.
The default rounds to nearest with midpoints away from zero: `2.5` becomes `3`,
and `-2.5` becomes `-3`. Decimal values, normalization, errors and rounding are
tested across Rust and .NET. Arithmetic precision limits remain distinct from
explicit rounding and display formatting.

Published model interfaces declare their inputs and outputs. An invocation can
supply scenario-specific inputs without changing the shared model. The core can
also be used directly through Rust crate APIs, independently of a workbook or GUI.

## The spreadsheet desktop

Worksheet-range producers and read-only bound Grid Windows have an approved design in
[ADR-0041](docs/adr/0041-worksheet-range-producers-and-bound-grid-windows.md).
The design includes live worksheet projection and explicitly refreshed fixture
snapshots. Four mandatory review clarifications are incorporated; implementation
is not yet authorized. See the
[design handoff](docs/handoffs/2026-11-02-bound-grid-windows-design.md).

Unbound Grid Windows are accepted and closed under
[ADR-0040](docs/adr/0040-unbound-grid-windows.md). Each has an independent named
coordinate space; `WindowName|A1` references share the canonical calculation graph
with worksheets. Selection stays client-local and capacity remains aggregate.
See the [implementation semantics](docs/architecture/unbound-grid-windows.md).
All six targets passed managed/AOT GUI qualification; Windows ARM64 headless retains
an unresolved cold-run timing qualification. See the
[implementation handoff](docs/handoffs/2026-11-01-unbound-grid-windows.md).

The desktop is built with **C#/.NET 10 and Avalonia** for Windows, Linux and macOS.
It provides direct grid editing, a formula bar, rectangular selection, copy/paste
and four-direction fill. Formula copies translate relative references and preserve
`$`-locked axes. Cells support Decimal96, Boolean and String values, from literals or formulas.

Workbooks contain up to **16 worksheets**, with Add/Rename controls and tabs that
remember each sheet's selection and scroll position. Cross-sheet formulas such as
`=Revenue!B2-Costs!B2` share one dependency graph. Renaming a sheet updates formula
presentation while keeping its bindings intact.

Edit also offers explicit visible-only Clear (Contents, Built-in Formatting, or Both)
and four-direction Fill. They review actual affected rows and preserve hidden cells;
Clear addresses up to 4,096 visible slots, Fill up to 256 including its source edge.
See [visible Clear/Fill semantics](docs/architecture/visible-edit.md).

Functions include `SUM`, `MIN`, `MAX`, `AVERAGE` and `COUNT`, with rectangular ranges,
multiple arguments and nested calls: `=SUM('January Sales'!A1:B20, C1:C20, 5)`.
Comparisons and `IF`, `AND`, `OR`, `NOT`, `LEN`, `MID` and `&` support conditional
and text calculations, such as `=IF(A1>0,"In stock","Sold out")`. IF and logical
functions short-circuit; strings and Booleans are never silently converted to numbers. Calculation
rounding is explicit: `ROUND`, `ROUNDUP` and `ROUNDDOWN` accept positive or negative
integer decimal-place arguments, such as `=ROUND(125,-1)` → `130`. They change the
calculated value; cell number formats change only its presentation.

`ALLOCATE(total,count,index,places)` returns an exact settled equal share with
explicit precision: `=ALLOCATE(10,3,1,2)` returns `3.34`, and indices 2 and 3 return
`3.33`. Conservation requires a complete, consistently specified set of indices.
See [exact allocation](docs/architecture/allocation.md) for admission and error rules.
`ALLOCATEWEIGHTED(total,index,places,weight1,...,weightN)` settles proportional shares
for up to 64 explicit weights. For example, `=ALLOCATEWEIGHTED(10,1,2,1,2,3)` returns
`1.67`. See [weighted allocation](docs/architecture/weighted-allocation.md) for ordinal
association, exact reconciliation and the documented limits.
`ALLOCATEWEIGHTEDRANGE(total,index,places,weight_range)` reads 1..64 physical positions
in one literal row or column range. Missing, Blank, String and Boolean weights are
errors; zero must be explicit. Sorting changes positional weight order, and hidden
rows still count. See [range-weighted allocation](docs/architecture/range-weighted-allocation.md).


A Format pane provides fixed-decimal, grouped, currency and percentage display,
all seven display rounding modes, bold/italic, alignment and cell colors. Formatting
keeps exact calculation values intact; even empty regions can be styled without
creating calculation nodes. Cell, row, column and range formatting uses sparse
property regions: applying a property changes every selected position, including
prior exceptions, while untouched properties remain intact. Reset Formatting and
Use worksheet defaults distinguish explicit built-in defaults from inheritance.
Native copy/paste and Fill carry effective style snapshots with content.

Insert or delete selected rows and columns as one atomic operation. Surviving cell
identities and references remain stable; deleting a referenced target produces
`#REF!`, which can be repaired by editing the broken operand. Insertion protects
outgoing values and Blank anchors. Formatting follows the approved band inheritance
rules, with notice when explicit formatting is clipped at the worksheet edge.

Sort selected rectangular records by ordered column keys, with explicit header
exclusion and a choice to move formatting or keep it at worksheet positions.
Scalar references follow moved cell identities; range geometry stays fixed. The
dialog displays the final range and offers explicit inclusion of adjacent columns.
[Sorting implementation and validation status](docs/architecture/worksheet-sorting.md).

Workbooks open and save as local `.pivot` files, preserving every sheet, typed
content, presentation and stable bindings. The current experimental format is WB14; earlier
experimental workbook versions are rejected.

Worksheet filters preserve typed criteria and hide whole rows without changing cell
coordinates, identities or calculation membership. Hidden-row bulk operations use
explicit full-range confirmation. [Filtering implementation and validation status](docs/architecture/worksheet-filtering.md).

Copy Visible Cells and Paste into Visible Rows are separate explicit commands.
Paste reviews the actual worksheet rows before an atomic mapped write; formulas
translate from each cell's original coordinates. Exact Strings, including leading
zeros, remain intact. [Visible clipboard implementation and validation status](docs/architecture/visible-clipboard.md).

CSV import creates a new unsaved workbook; CSV export writes calculated values
from the selection or active sheet. Text is the default import mode. Optional
Numbers and Booleans mode preserves quoted Strings and leading-zero integer
identifiers by default, with independent controls for each conversion policy.
Quoted empty fields remain empty Strings; unquoted empty fields are Blank. Import
never executes formulas. Production admission allows 65,536 positioned graph cells
and 262,144 allocated cell identities across the workbook, with 65,536 rows and
1,024 columns per sheet. Cleared cells and referenced blanks retain positions;
formatting-only blanks remain graph-free. CSV admits 1,048,576 fields and 16 MiB;
excessive imports fail without replacing the current workbook. Formatting has a
separate limit of 65,536 stored property regions and an 8 MiB style-data/index budget;
coverage is not limited to 65,536 styled positions.
CSV uses UTF-8 and comma separators, with quoted multiline text supported.
Strings are always quoted on export; other applications may interpret them differently.

The desktop owns a **Rust session process (`pivot-session`)**, communicating through
framed Protobuf messages over standard input/output. Rust owns workbook state,
validation and calculation; Avalonia handles presentation and interaction. The .NET
client also supports **Native AOT** publishing.

## The daemon and command-line client

**`pivotd`** hosts named calculation models without the desktop. Its Protobuf
protocol supports model discovery, interface inspection and invocation. Public
model references identify a specific revision, keeping the model being calculated
explicit.

Local transport uses **Windows named pipes** or **Unix-domain sockets**, with
operating-system peer-identity checks. The transport and hosting layers are separate
from the calculation core. The CLI and persistent REPL use the same daemon boundary
rather than maintaining their own evaluator.

```text
Avalonia desktop → pivot-session → workbook layer → Rust calculation core
CLI / REPL       → pivotd                        → Rust calculation core
Rust application → direct crate APIs             → Rust calculation core
```

For example, a model can expose `Revenue` and `Costs` as inputs and calculate
`Net = Revenue - Costs`. The worksheet is one way to work with that calculation;
an application can invoke the model with another set of inputs through the daemon.

## Goals

- Combine familiar spreadsheet interaction with reusable, explicitly defined models.
- Keep calculation semantics consistent across desktop, daemon and embedded use.
- Make dependencies, inputs, outputs and errors inspectable.
- Support independent scenarios and calculations over consistent snapshots.
- Extend the value model toward additional numeric types and structured outputs.
- Keep clients, hosting and storage concerns separate from the computation core.

## Try Pivot

Install Rust, the .NET 10 SDK and Python, then run from the repository root:

```sh
python scripts/run_desktop.py
```

To try the daemon, start the sample model in one terminal:

```sh
cargo run -p pivotd -- --demo
```

Use a second terminal to discover and invoke it:

```sh
cargo run -p pivot-cli -- models
cargo run -p pivot-cli -- describe RestaurantProfit
cargo run -p pivot-cli -- invoke RestaurantProfit Revenue=250000 Costs=85000
```

See [daemon and CLI usage](docs/architecture/local-ipc.md) and
[REPL usage](docs/architecture/command-repl.md) for endpoints, authoring and commands.

## Development and technical documentation

Run `python scripts/ci.py` for the contributor checks, including Rust and .NET,
cross-language decimal/protocol checks and real process communication. See
[testing](docs/testing.md) for prerequisites, desktop validation and Native AOT.

- [Decimal and expression semantics](docs/architecture/expressions-and-decimals.md)
- [Build output and artifact retention](docs/artifact-retention.md)
- [Milestone code-size reports](docs/code-size.md)
- [Architecture decisions](docs/adr/README.md)
- [Development review records](docs/handoffs/)

## License

Pivot is licensed under the [Apache License 2.0](LICENSE).
