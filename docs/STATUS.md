# Pivot implementation status

Date: 2026-11-02

## ADR-0042 — section geometry design accepted; implementation not authorized

[Section Breaks and Local Grid Geometry](adr/0042-section-breaks-and-local-grid-geometry.md)
specializes ADR-0039's third scope after accepted ADR-0040/0041. Either horizontal
or vertical breaks independently create sections. Cell selection stays within one
section; crossing either boundary expands to whole touched sections. Independent
width/height profiles preserve values, identities, formatting and range semantics.

The design defines one bounded geometry map for painting/hit testing/editor/selection,
complete overlap refusal, stable authored-axis transforms, explicitly positional
bound-view geometry, filter presentation suspension and aggregate charged admission.
A concrete header/line-items/totals invoice and separate one-boundary selection cases
support design review. Ordinary invoice workflows must succeed without merge/spacer
or manual compensation workarounds. The positional bound-view and filter policies
are explicit review decisions, not implementation assumptions.

The final design review accepts all three required clarifications: per-operation
confirmation when closure enlarges a destructive/bulk-changing selection; explicit
positional-layout identity and generation-qualified Review/Keep/Reset after meaningful
source schema/order changes; and complete invoice construction/revision sequences
through ordinary controls, not merely a valid final fixture. Final design acceptance
was granted on 2026-11-02; no further design review is required. Usability and
performance remain implementation qualification gates. No implementation
authorization has been given.

See the [design handoff](handoffs/2026-11-02-section-grid-geometry-design.md) and
[architecture contract](architecture/section-grid-geometry.md). WB16/private v16
are proposed audit cuts, not current implemented formats. WB15/private v15 remain
the shipped experimental baseline. No feature, GUI/AOT, VM or performance validation
was run for this design. Prior cold Windows ARM64 failures remain unresolved.
ADR-0023 remains parked at G3. Feature work awaits separate owner authorization.

Historical milestone records are preserved in the
[status archive through 2026-11-02](archive/STATUS-history-through-20261102.md).
