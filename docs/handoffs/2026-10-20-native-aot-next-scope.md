# Direction review and proposed Native AOT milestone

Date received: 2026-10-20. Parent response supplied by the user.
**Original direction and bounded assignment.** The user subsequently authorized
implementation. [Current implementation/evidence](2026-10-20-native-aot-baseline.md):
Windows/Linux/native Mac Native AOT and managed CI pass; the milestone is accepted
and closed. [Acceptance and next design scope](2026-10-20-native-aot-review.md).
This does not freeze a dynamic-data grammar. The preceding clipboard/WB3 milestone
remains accepted and closed.

## Product direction

Pivot is a compositional computational document with spreadsheet presentation and
structured external inputs. It should consume business data from external systems,
not become an RDBMS. Full spreadsheet functionality remains a product goal.

Keep producers, immutable result generations, record identity and displayed projection
identity distinct. Dynamic records do not implicitly become ordinary cells/CellIds.
Presentation may interleave dynamic blocks with worksheet rows without changing A1
positions or bindings. A presentation-owned address translation layer must resolve
screen positions to tagged worksheet or projection/record/field locations for hit
testing, selection, scrolling and eventually printing; it does not belong in evaluation.

The `#` namespace is a concept to reserve, not an implemented or approved concrete
grammar. References should bind producing-node identity and fields/records, independently
of visible `#1` labels or projection placement. Keyed versus ordinal access, missing/
duplicate keys, schema changes and result-generation semantics need explicit design.

Refresh is separate from recalculation. Acquisition publishes an immutable result;
normal evaluation consumes captured inputs without implicit database/network calls.
Independent sources do not automatically represent one external transactional snapshot.
Layout rules for zero records, overflow, page breaks and overlapping visual regions
must not be solved by mutating canonical worksheet identities. Initial projection
experiments should use fixed typed records, not SQL/connectors or write-through edits.

## Proposed sequence

1. Native AOT publish-and-execute baseline.
2. Ordinary document-owned worksheet text, with numeric graph behavior explicit.
3. Structured-result/projection design using fixed typed records; concrete grammar
   and implementation remain separate review decisions.

Formatting, Undo/Redo and broader spreadsheet features remain product priorities,
not automatically authorized parts of these milestones. No dynamic grammar is frozen.

## Proposed assignment: Native AOT Publish-and-Execute Validation

Validate the existing .NET 10 Avalonia desktop as an actual Native AOT executable
on native Windows x64, Linux x64 and macOS ARM64. Keep the separate native Rust
child and accepted workbook, clipboard, numeric, capacity and WB3 semantics.
This is compatibility readiness, not installer development, packaging redesign,
optimization or feature expansion.

Audit anonymous JSON report serialization, helper launch assumptions, Avalonia
XAML/bindings, runtime-created controls and dependencies. Where required, replace
reflection-dependent report objects with named DTOs and explicit System.Text.Json
source-generated contexts. Test scripts/harnesses need not all become AOT binaries.
Preserve framework-dependent development execution; when testing AOT, helper GUI
processes must launch the published native application, not `dotnet app.dll`.

Publish on corresponding native platforms/toolchains. A self-contained or ReadyToRun
publish is not Native AOT evidence. Ensure the published desktop locates and launches
the native Rust executable; .NET publishing does not establish child availability.

The published binary must render, start its real Rust child, edit/calculate numbers
and formulas including SUM, exercise rectangular selection/native clipboard, save
and reopen WB3, handle dirty close, and shut down with zero remaining Rust charge.
Retain trim/AOT warnings, native prerequisites, executable sizes, startup observations
and platform exceptions. Do not indiscriminately suppress warnings to pass a gate.

Keep `python scripts/ci.py --desktop`; introduce a separate AOT verification path
until proven reliable. If a dependency or Avalonia feature blocks AOT, report the
exact warning/error, affected path and smallest viable remediation. Do not rewrite
the UI framework to claim success. No text values, projections, connectors, graph
redesign or installer work in this assignment. Deliver a concise cross-platform
report and stop for parent review.

## Existing operational follow-up

Historical assignment: runner-enforced Windows/WSLg clipboard-suite serialization was outstanding from
the accepted desktop review. Preserve it as an explicit follow-up; do not let either
ordinary or future AOT desktop tests rely solely on a sequencing convention where
they share a clipboard. No enforcement or AOT results are claimed by this record.

This follow-up is now implemented, tested in both cross-OS directions and accepted
as complete; see the implementation evidence and acceptance linked above.
