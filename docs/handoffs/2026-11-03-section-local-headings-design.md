# ADR-0043 — section-local headings design handoff

Date: 2026-11-03.
Disposition: **Design accepted by parent review, 2026-11-03. Owner subsequently authorized implementation; implementation and qualification accepted and milestone closed by parent review, 2026-11-03, with timing qualifications retained.**

The [implementation handoff](2026-11-03-section-local-headings.md) records the final
disposition and qualification. This file preserves the accepted design decisions.

The review accepts the design without mandatory architectural corrections or
another review cycle. The owner supplied implementation authorization separately
after design acceptance.

[ADR-0043](../adr/0043-section-local-headings-and-axis-selection.md) addresses
discoverability and local row/column selection after accepted ADR-0042.
The [interaction contract](../architecture/section-local-headings.md) accompanies it.
No feature code, platform tests, new benchmark or format change is included.

## Accepted design decisions

1. Show column headings at horizontal Section Breaks and row headings at vertical
   breaks, scoped to derived section intersections. Either orientation works
   alone. Scope-edge outer rulers serve their edge without duplicate labels.
2. Heading clicks select entire local rows/columns; corner clicks select the
   section. Labels preserve scope addresses. Crossing either boundary retains
   whole-section closure and expanded-operation confirmation. Outer headings keep
   their existing active-section behavior; whole-scope actions are explicit.
3. Add one master **Show interior headings** toggle, initially enabled. It hides
   both orientations and all associated input/accessibility targets without
   changing selection, document geometry, revisions or workbook dirtiness.
   Keep outer rulers and error/review notices. The setting is client session state.
4. Use bounded editor overlays from captured canonical geometry. Sticky/segmented
   headings remain section-qualified; density prioritizes the active section with
   explicit access to others. No permanent cell/editor occlusion is permitted.
   Useful ordinary invoice layouts must not depend on routinely hiding headings.
5. Reuse canonical capture/selection authority and global admission. No new
   section objects, axis identities, calculation model or independent client map.
   Audit private v16 sufficiency; if incompatible descriptors are needed, use
   private v17 alone. WB16/hosted v1/CB5/CB6 stay unchanged.

## Concrete acceptance scenario

In the 36×8 invoice, horizontal breaks before 5/29 expose headings for header,
line-item and totals bands. C in the line-items strip selects C5:C28. Row 10
selects A10:H10; with a vertical break before G, the left section's row 10 selects
A10:F10 and the right section's selects G10:H10. Crossing C to G selects the
whole touched sections, with confirmation before an expanded bulk change.

Toggle all interior headings off to inspect the document, then on again. Content,
selection, scope-relative addresses, canonical geometry, zoom and workbook revisions
remain unchanged. Hidden headings cannot intercept a click. Existing bound-layout
review notices remain visible. Repeat scrolled, filtered, at zoom extremes and
after changing source generation; no stale heading may act on the wrong section.

## Implementation risks and qualification gates

The review records two implementation obligations: keep heading-divider targets
distinct from established geometry-resize controls so hiding headings preserves
available geometry editing; and publish overlays coherently with cells and related
controls, preserving ADR-0042's publication-order correction. Obsolete responses
must not restore hidden chrome or cancelled gestures. These require implementation
checks, not another architectural revision.

The main risk is overlay usability around stepped boundaries, dense sections and
small viewports. Actual invoice editing, accessible keyboard navigation, unobscured
inline editing and toggle/gesture cancellation must be demonstrated, not inferred
from valid geometry. Bounded density fallback must be explicit and usable.

Source-qualified six-target headless and managed/AOT attended qualification,
verified archive/cache cleanup and per-project cloc remain planned gates after
separate implementation authorization. macOS x64 is Rosetta. Existing Windows
ARM64 cold timing failures remain unresolved; ADR-0023 remains parked at G3.

ADR-0042 remains accepted and closed. No next feature beyond this design is
authorized. No further design review is required; implementation is now separately
authorized. Attended GUI evidence remains required and cannot be replaced by virtual tests.
