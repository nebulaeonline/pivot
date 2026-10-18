# Pivot

Pivot is a reusable computation platform with a spreadsheet frontend. Its core
is an expression and dependency graph that can run independently of a workbook
or graphical interface.

The engine and daemon are written in Rust. The planned desktop client uses
C#/.NET 10 and Avalonia for a cross-platform interface. Decimal arithmetic is
the default for financial and analytical calculations.

## Goals

- Build named computation models with explicit inputs and outputs.
- Use spreadsheets to view, edit, and explore those models.
- Run models headlessly, directly from Rust or through a stable public protocol.
- Support a REPL and clients written in other languages.
- Evaluate independent scenarios without changing shared model state.
- Keep calculations predictable, errors explicit, and clients replaceable.

## Current state

Pivot is in early development. The repository contains the Rust and .NET project
skeletons, a decimal numeric foundation with cross-language tests, and a basic
expression parser, editable models with immutable snapshots, and headless decimal
evaluation. The daemon protocol, REPL, and spreadsheet
UI are still ahead.

See [implementation status](docs/STATUS.md) and
[architecture decisions](docs/adr/README.md) for more detail.

## License

Pivot is licensed under the [Apache License 2.0](LICENSE).
