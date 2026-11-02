# ADR-0042 — section geometry design handoff

Date: 2026-11-02.
Disposition: **Final design acceptance granted, 2026-11-02. No feature implementation or platform qualification.**

[ADR-0042](../adr/0042-section-breaks-and-local-grid-geometry.md) specializes the
third accepted ADR-0039 scope, after accepted ADR-0040/0041. The detailed
[map/selection contract](../architecture/section-grid-geometry.md) accompanies it.

## Required review clarifications incorporated

- Expanded destructive/bulk-changing selections require explicit confirmation per
  invocation, with requested/effective bounds and operation context. Cancellation
  changes nothing; changed state/context invalidates consent. No blanket approval.
- Bound controls explicitly identify positional presentation. Meaningful source
  schema/order changes show a persistent generation-qualified review notice with
  Review/Keep/Reset actions, without blocking publication or moving sizes with fields.
- Usability qualification runs complete construction and revision sequences from
  default geometry through ordinary controls, recording intermediate outcomes.
  A valid final fixture or specially discovered editing order cannot pass the gate.

The final review accepts all three clarifications and the architectural contract.
No further design packet or review cycle is required. Usability and performance
remain implementation acceptance gates, not demonstrated results. WB16/private v16
remain proposed cuts pending codec audit. Implementation needs separate owner
authorization; no additional scope is authorized.

## Decisions for review

1. Either break type independently creates sections. Horizontal-only sections span
   the scope width; vertical-only sections span its height. With both, intersections
   identify derived sections without storing editable two-dimensional regions.
2. Cell selection stays within one section. Crossing **either** orientation expands
   to complete touched sections. The ADR includes separate horizontal-only,
   vertical-only and combined examples. Typed formula/data descriptors are not
   silently expanded. Interactive bulk operations use actual selection closure.
3. Independent width/height profiles keep logical identity/calculation unchanged.
   Positive-area overlap, including Blank cells, refuses before publication. A
   constant-size tile sweep proves the complete map without enumerating all cells.
4. Rulers identify the active section. Local divider editing preserves its span
   with visible adjacent-size changes; shared seam movement is an explicit batch.
   Ordinary invoice workflows must succeed without merge/spacer/manual compensation.
5. Worksheet/unbound anchors use stable authored axes. Bound geometry deliberately
   uses positional presentation ordinals, not producer item/field identity. This
   means source reordering or column insertion does not carry widths with a field.
   Review that tradeoff explicitly; formula authority remains unchanged.
6. Filter compaction uses the same captured map. Ordinary aligned profiles support
   any mask. A view conflict rejects an attempted geometry/filter edit; a later
   data-driven conflict suspends filter presentation visibly rather than blocking
   the data commit or showing mixed generations. Visible-only actions remain blocked
   until explicit revalidation.
7. One Rust map supplies explicit rectangles for paint/hit/edit/selection and future
   layout consumers. Bounded viewport pages share one canonical capture; no second
   C# geometry arithmetic, independent calculation revision or additional capture pool.
8. Propose WB16/private v16 with complete geometry authority validation. Hosted v1
   and CB5/CB6 remain unchanged. No historical reader or history integration.

## Concrete feasibility and required usability evidence

The ADR specifies a 36×8 invoice, horizontal breaks before rows 5/29, optional
vertical break before G, three width profiles, and differing right-side row heights.
It has common x=480-point and y=72/504-point outer seams, broad description fields,
narrow metadata, separate totals/payment geometry and exact identifier Strings.
The design arithmetic independently checks all 288 cell rectangles and the six
selection-closure cases. This is mathematical feasibility only, not product testing,
font/text fitting, screenshots, GUI usability or measured performance.

The decisive implementation gate is zero unexpected refusals in the reviewed
ordinary invoice workflows, including normal sizing tools and one-boundary selection.
If ordinary layouts fail, revisit the map/editor model before promotion. Intentionally
invalid overlap and budget cases must continue refusing. The safe-gap/collision pair,
independent dense/selection oracles and stale-frame cases prevent paint-order or
bounding-box shortcuts from appearing to satisfy this gate.

## Scope and validation boundary

The design inspects current fixed-size `WorksheetView`, visibility mapping, compact
axis identities and workbook structural transactions. It proposes presentation-only
metadata, atomic transforms, source-qualified captures and aggregate charged limits.
No app/core/protocol code changed. No Rust/.NET build, GUI/AOT, VM command, platform
benchmark or implementation validation was performed for this design.

Implementation, if separately approved and authorized, needs six-target headless
and managed/AOT qualification, resource measurements, per-project cloc and verified
retention. macOS x64 remains Rosetta coverage. All prior original Windows ARM64
cold timing failures remain unresolved; this design neither fixes nor reclassifies
those receipts. ADR-0023 remains parked at G3.

Text, placements, page rules, PDF, SQL/providers, invocation, licensing and production
history remain outside this milestone. No subsequent design scope has started.
