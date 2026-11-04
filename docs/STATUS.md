# Pivot implementation status

Date: 2026-11-03

## ADR-0043 — accepted and closed

[Section-Local Headings and Axis Selection](adr/0043-section-local-headings-and-axis-selection.md)
is implemented and qualified under separate owner authorization following accepted
design review. Parent implementation review accepted and closed the milestone on
2026-11-03, retaining the recorded timing qualifications. See the [implementation handoff](handoffs/2026-11-03-section-local-headings.md)
and [interaction contract](architecture/section-local-headings.md).

Interior row/column headings and section corners select their exact local bounds.
Crossing either boundary retains canonical whole-section closure and expanded-
operation confirmation. Labels are centered and retain scope addresses. One master
Show interior headings toggle removes interior controls and input/accessibility
nodes without changing document geometry, revisions or selection. Outer rulers and
existing geometry tools remain available.

The implementation uses bounded, capture-qualified Rust heading descriptors and
coherently published viewport overlays. The codec audit required private session
v17; WB16, hosted v1 and CB5/CB6 remain unchanged. No new calculation or document
authority is introduced.

The local baseline passed 13 CI gates, all-feature Rust tests and strict Clippy.
All six full native headless targets passed, with a separate retained-cache retry
on Windows x64 after its cold sparse-sort sample exceeded the existing objective.
That failure remains preserved. Subsequent focused complete native UI checks passed
on all six targets after the UI-only production contrast correction; the handoff
records the distinction from the earlier full headless baseline.

All twelve final attended GUI suites passed on the corrected runtime: managed and
Native AOT on Windows/Linux x64 and ARM64, macOS ARM64, and macOS x64 under Rosetta.
All 952 executed source files match per suite. Forty-eight graphical application
reports have zero final charges and no owned process survivors. Light/dark rendered
pixel checks pass in every suite.

Owner inspection exposed invisible headings in Linux ARM64's dark theme despite
passing control/event assertions. The corrected worksheet-compatible palette is
covered by a regression that fails the original styling. Original passing receipts
and failed attempts retain their outcomes and source provenance.

Verified archives precede cache pruning. The final cleanup batch removed 72.8 GiB
of logical disposable caches across six targets, in addition to an earlier 35.3 GiB
x64 batch. Source, reports, workbooks and published products are preserved. cloc 2.06
reports 107,836 source-code lines across 443 files, with per-project breakdowns in
the handoff. Runtime and test code still match the qualified snapshot; only closeout
documentation changed afterward.

Parent implementation review accepts the source-qualified evidence, including the
full baseline followed by focused and attended qualification of the UI-only palette
correction. No further implementation, benchmarking or platform runs are required.
Commit/push remains with the owner; no next milestone is authorized. ADR-0042 remains accepted and closed; ADR-0023 remains
parked at G3. Historical Windows ARM64 cold structural timing remains unresolved.
The Windows x64 cold sparse-sort observation of 1,044 ms against the 1,000 ms
objective also remains unresolved; subsequent passing results do not erase it.
No Text, placement, pagination, PDF, provider, licensing or history work is included.

Product direction remains spreadsheet-first. Human usability is not established by
automated interaction qualification. Visually distinct section surfaces with an
independent placement layer are a possible future direction, preserving parent-scope
addresses and cell ownership rather than creating new Grid Windows. Evaluating
that direction requires separate owner authorization; it is outside ADR-0043.

Historical records: [ADR-0042 closeout](archive/STATUS-ADR0042-20261103.md) and
[status archive through 2026-11-02](archive/STATUS-history-through-20261102.md).
