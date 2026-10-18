# ADR-0001: Graph-first computation core

Status: Accepted
Date: 2026-10-18

## Context

Pivot must support spreadsheet editing, named computational models, headless
Rust use, a REPL, and clients in other languages. The project baseline already
settled the graph-first direction. This record captures that decision.

## Decision

Use a general expression/dependency graph as the primary computation model.
Workbook coordinates are projections and bindings into that graph. Graph nodes
may exist without cell addresses; A1 strings are not canonical engine IDs.

Keep the Rust core independent of transport, daemon lifecycle, .NET, and UI
types. Rust programs can use the core in process. The daemon hosts it for
external clients; the GUI and REPL use the public protocol.

## Alternatives considered

- Cell-addressed engine: convenient for spreadsheets, but couples model reuse
  and external clients to worksheet coordinates.
- Calculation in GUI view models: convenient initially, but prevents headless
  use and creates competing sources of calculation truth.
- Daemon-only computation API: supports external clients, but unnecessarily
  prevents direct Rust library use.

## Consequences

Workbook references require a projection layer. Core expressions and values
must remain UI-free and transport-independent. Protocol and storage schemas
are separate boundaries, not core graph representations. Crate scaffolds do
not imply that these layers have been implemented.

## Verification / reconsideration trigger

Build and test the core without loading a GUI or starting a daemon. Once graph
execution exists, demonstrate a named model with no cell addresses through a
pure Rust test. Revisit this record only if product requirements change enough
to remove reusable, headless computation as a goal.
