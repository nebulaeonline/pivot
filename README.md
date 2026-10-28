# Pivot

Pivot combines a spreadsheet desktop application with a reusable calculation engine
and daemon. Build models interactively, work with local workbook files, or run
calculations headlessly from another application.

The Rust engine represents calculations as expressions with explicit dependencies.
The desktop client uses C#/.NET 10 and Avalonia on Windows, Linux and macOS.

## Desktop

The spreadsheet interface provides direct cell editing, formulas, text labels and
rectangular selection. Copy, paste and four-direction fill translate formula
references, including absolute references. Workbooks open and save as local `.pivot`
files.

Supported aggregate functions include `SUM`, `MIN`, `MAX`, `AVERAGE` and `COUNT`.
Decimal arithmetic has explicit rounding behavior and reports calculation errors.

## Daemon and application integration

`pivotd` exposes named computation models through a Protobuf protocol over local
IPC. Clients can discover models, inspect their inputs and outputs, and invoke
calculations. The command-line client and REPL provide another way to use the engine.

Models remain editable while each calculation runs against an immutable snapshot.
Invocation-specific inputs support independent scenarios without changing the shared
model. Stable identities keep references independent of display names.

## Goals

- Make calculation models useful through both spreadsheets and application APIs.
- Keep calculation behavior consistent across desktop and headless use.
- Support clear inputs, outputs, dependencies and errors.
- Keep the engine reusable independently of the user interface or hosting model.

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
cargo run -p pivot-cli -- invoke RestaurantProfit Revenue=250000 Costs=85000
```

See [daemon and CLI usage](docs/architecture/local-ipc.md) and
[REPL usage](docs/architecture/command-repl.md) for more options.

## Development

Run the contributor checks with `python scripts/ci.py`. See
[testing](docs/testing.md) for prerequisites, desktop validation and artifacts.
Design decisions are recorded in [ADRs](docs/adr/README.md); development review
records live in [docs/handoffs/](docs/handoffs/).

## License

Pivot is licensed under the [Apache License 2.0](LICENSE).
