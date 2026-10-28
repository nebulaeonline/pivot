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

The desktop is built with **C#/.NET 10 and Avalonia** for Windows, Linux and macOS.
It provides direct grid editing, a formula bar, rectangular selection, copy/paste
and four-direction fill. Formula copies translate relative references and preserve
`$`-locked axes. Cells support decimal values, formulas and literal text.

Workbooks contain up to **16 worksheets**, with Add/Rename controls and tabs that
remember each sheet's selection and scroll position. Cross-sheet formulas such as
`=Revenue!B2-Costs!B2` share one dependency graph. Renaming a sheet updates formula
presentation while keeping its bindings intact.

Functions include `SUM`, `MIN`, `MAX`, `AVERAGE` and `COUNT`, with rectangular ranges,
multiple arguments and nested calls: `=SUM('January Sales'!A1:B20, C1:C20, 5)`.
Workbooks open and save as local `.pivot` files, preserving every sheet, typed
content and stable bindings. The current experimental format is WB7; earlier
experimental workbook versions are rejected.

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
- [Architecture decisions](docs/adr/README.md)
- [Development review records](docs/handoffs/)

## License

Pivot is licensed under the [Apache License 2.0](LICENSE).
