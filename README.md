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
evaluation with isolated input overrides and named public model interfaces.
A demo daemon and CLI now support discovery and invocation over local IPC.
A persistent command REPL is available. Opt-in daemon authoring supports transactional
editing, evaluation and publication through the protocol; state is memory-only.
Minimal REPL authoring is available; the spreadsheet UI is still ahead.

The transport-independent Protobuf protocol has Rust/C# serialization tests and an
in-process host adapter for discovery and invocation. See [local IPC usage](docs/architecture/local-ipc.md)
to run the demo daemon and CLI. Windows, Linux under WSL2, and macOS ARM64 are verified; see
[platform validation](docs/platform-validation.md) for coverage and build constraints.

See [implementation status](docs/STATUS.md) and
[architecture decisions](docs/adr/README.md) for more detail.

## License

Pivot is licensed under the [Apache License 2.0](LICENSE).

## Local CI

Run `python scripts/ci.py` to check Rust formatting, linting, tests, the .NET
Release build, and cross-language decimal compatibility. Logs and a JSON summary
are saved under `artifacts/ci/`. See [local CI setup](docs/local-ci.md).
