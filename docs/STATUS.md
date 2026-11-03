# Pivot implementation status

Date: 2026-11-03

## ADR-0042 — accepted and closed

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
are accepted contracts, not optional implementation choices.

The final design review accepts all three required clarifications: per-operation
confirmation when closure enlarges a destructive/bulk-changing selection; explicit
positional-layout identity and generation-qualified Review/Keep/Reset after meaningful
source schema/order changes; and complete invoice construction/revision sequences
through ordinary controls, not merely a valid final fixture. Final design acceptance
was granted on 2026-11-02; no further design review is required. Usability and
performance remain implementation qualification gates. The owner authorized
implementation on 2026-11-02. Implementation and qualification are complete;
parent implementation review accepted and closed the milestone on 2026-11-03.
No further validation is required. See the
[implementation handoff](handoffs/2026-11-03-section-grid-geometry.md).

See the [design handoff](handoffs/2026-11-02-section-grid-geometry-design.md) and
[architecture contract](architecture/section-grid-geometry.md). The Rust map,
immutable scope roots, structural transforms, WB16 persistence and private v16
geometry requests and editor integration are implemented and qualified with the
full/focused headless source lineage recorded below and in the handoff.
The current publication fix passed [13-gate local CI](../artifacts/adr0042-local/final-10/ci/20261103T121130769473Z/summary.json),
strict all-feature Clippy and all-feature Rust tests. The earlier implementation
baseline passed full headless CI on all six configured RIDs. After the latest UI
change, [focused native UI qualification](../artifacts/adr0042-local/native-fixture-final-10/summary.json)
passed on all six targets, including the complete invoice, editing and clipboard
sequences and a deterministic frame-publication regression. These virtual tests
use no native windows or OS clipboard.

The Desktop now uses captured canonical rectangles for rendering, hit testing,
headers, scrolling and section closure. Local sizing, balanced divider edits,
explicit shared-seam batches and reset controls have a non-publishing preview.
Bound positional-layout Review/Keep/Reset notices and filter suspension are
implemented. Virtual-window tests construct and revise the invoice through the
actual editor controls in two orders, preserve leading-zero Strings, and exercise
cancelled, accepted and stale expanded Clear confirmations. Those checks supplement,
and do not replace, attended GUI qualification.

All four VMs passed attended managed and Native AOT GUI suites before the latest
publication fix. macOS ARM64 managed also passed; its AOT suite exposed a product
publication gap: current cells could appear before their filter warning and recovery
controls. Filter and layout controls now publish synchronously with the captured
cell frame, before deferred cleanup and selection resolution. A regression fails
on the old ordering and passes on the fix. Earlier successful and failed receipts
retain their original source hashes and outcomes.

The corrected runtime snapshot is `a983aa351bd9646ab7f45d9c082a2d4b92f3ad5a45c1fa2a121207d53921e40b`.
Fresh macOS ARM64 managed/AOT and x64 managed/AOT under Rosetta have all passed;
their executed source snapshots match the corrected runtime snapshot. Fresh Windows
x64 managed/AOT reports also pass on that runtime. Windows ARM64 managed/AOT pass
on snapshot `ae3591ebe1f850e45eec7a8ada6ae1b9476225b32931465a120812eeec920469`,
with all 944 executed files verified in each suite. That snapshot changes only the
stale-view clipboard test and this status document relative to the corrected runtime:
the fixture explicitly establishes an older displayed frame and verifies clipboard
contents separately with bounded readback. Production code is unchanged. The complete
virtual UI sequence passes locally and on both Linux targets, including three fresh
iterations on each ARM64 VM. Linux x64 and Linux ARM64 managed/AOT also pass on
that snapshot, with all 944 executed files verified in every suite. All twelve
attended GUI suites pass. Native 1,024/10,000/65,536-node resource observations are
complete on all six targets; macOS RSS is a sampled lower bound on peak. The cloc
refresh reports 107,251 source code lines across 441 files with per-project detail.
Completed final GUI evidence and original failures are archived with verified
hashes. Disposable per-run caches are pruned separately from retained source,
reports and published products. The handoff records evidence and qualifications.
Parent implementation review accepts this evidence, including the two final source
snapshots and the full-headless/focused-UI qualification distinction. Automated
invoice sequences qualify the specified workflows without claiming convenience
for every possible human-authored layout.

Prior cold Windows ARM64 failures remain unresolved.
ADR-0023 remains parked at G3. No next milestone is started or authorized by this
acceptance. Commit and push remain with the owner. Documentation closeout is complete.

Historical milestone records are preserved in the
[status archive through 2026-11-02](archive/STATUS-history-through-20261102.md).
