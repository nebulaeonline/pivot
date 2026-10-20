# Pivot

Pivot is a reusable computation platform with a spreadsheet frontend. Its core
is an expression and dependency graph that can run independently of a workbook
or graphical interface.

The engine and daemon are written in Rust. The desktop client uses
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

Pivot is in early development. A canonical persistent Rust graph/workbook supports
stable references, atomic edits, immutable calculations and exact decimal arithmetic.
The daemon and REPL support local invocation and optional memory-only authoring.
A headless document proof supports experimental model files.

The first Avalonia worksheet now edits and calculates through a real Rust-owned
local session. It supports decimal/formula entry, clear/move and bounded scrolling.
Native workbook files now support New, Open, Save, Save As and Close, preserving
stable bindings and editable drafts. Same-sheet `SUM(A1:B20)` uses current occupants
of a fixed rectangle and checked decimal arithmetic. Native files now use WB3; WB1/WB2 files are unsupported. Advanced spreadsheet features and deployment
are still ahead.

## Run the desktop

With Rust, .NET 10 and Python installed, run from the repository root:

```sh
python scripts/run_desktop.py
```

Drag or use Shift+click/arrows to select a rectangle. Copy/Paste operates on up to
256 selected cells; editing, Clear and Move still target the active cell. Select a cell and type a number or `=A1+B1`. F2 or double-click edits in place;
Enter/Tab commits and moves, and Escape cancels. The formula bar is optional.
Copy/Paste buttons and Ctrl+C/V (Cmd+C/V on Mac) operate on the rectangle or selected
editor text. Grid paste commits atomically; paste while editing inserts text. Native
formula copies translate current bound references; `$` locks copy axes. External TSV
formulas use literal addresses. Paste onto one cell to expand, or select an exact
matching destination shape.
The active outline dims when unfocused; caret blinking is configurable.
See the [clipboard handoff](docs/handoffs/2026-10-20-rectangular-clipboard.md)
for workflow, limits and validation status.

The transport-independent Protobuf protocol has Rust/C# serialization tests and an
in-process host adapter for discovery and invocation. See [local IPC usage](docs/architecture/local-ipc.md)
to run the demo daemon and CLI. Engine/daemon coverage includes Windows, WSL2 Linux
and macOS ARM64. Earlier ranges/SUM and WB2 checks passed on all three platforms; the new WB3/clipboard
slice requires fresh native Mac and manual validation. See [platform validation](docs/platform-validation.md) for
coverage and the recorded WSLg startup limitation.

See [implementation status](docs/STATUS.md) and
[architecture decisions](docs/adr/README.md) for more detail.

## License

Pivot is licensed under the [Apache License 2.0](LICENSE).

## Local CI

Run `python scripts/ci.py` to check Rust formatting, linting, tests, the .NET
Release build, cross-language decimal/protocol compatibility, real IPC, and the
Rust/C# application session. Logs and a JSON summary
are saved under `artifacts/ci/`. See [local CI setup](docs/local-ci.md).
