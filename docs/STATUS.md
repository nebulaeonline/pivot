# Pivot implementation status

Date: 2026-11-03

## ADR-0043 — design accepted; implementation authorization pending

The owner authorized design only for
[Section-Local Headings and Axis Selection](adr/0043-section-local-headings-and-axis-selection.md).
The [design handoff](handoffs/2026-11-03-section-local-headings-design.md) and
[interaction contract](architecture/section-local-headings.md) were accepted by
parent design review on 2026-11-03, without mandatory architectural corrections.
No feature implementation, platform tests or resource measurements accompany this design.

The accepted design adds section-local column headings at horizontal breaks and row headings
at vertical breaks, with local row/column selection and a whole-section corner.
Either boundary works alone; crossing either retains ADR-0042's whole-section
closure and expanded-operation confirmation. Labels retain scope addresses and
outer headings keep their existing active-section behavior.

A master **Show interior headings** toggle, initially enabled, hides both interior
orientations and their input/accessibility targets. It preserves selection, geometry,
zoom, document revisions and workbook dirtiness. Outer headings and review/error
notices remain. The setting is client session state, not authored workbook data.

Heading overlays use captured canonical geometry, bounded ownership and explicit
active-section/density behavior. Unobscured editing, stepped boundaries, scrolling,
filtering, accessibility and toggle/gesture cancellation are implementation gates.
WB16/hosted v1/CB5/CB6 remain unchanged. Private v16 descriptor reuse versus a
coordinated private v17 cut requires a codec audit before implementation qualification.

Implementation must keep heading-divider targets distinct from established geometry
resize controls and publish overlays coherently with the captured cell frame,
preserving ADR-0042's publication-order correction and stale-gesture cancellation.
These are implementation obligations, not further design conditions. Attended GUI
qualification remains required in addition to virtual checks.

No further design review is required. Await separate owner authorization before implementation.
ADR-0042 remains accepted and closed. Its evidence and qualifications are unchanged;
see its [implementation handoff](handoffs/2026-11-03-section-grid-geometry.md) and
[archived closeout status](archive/STATUS-ADR0042-20261103.md).
Historical Windows ARM64 cold structural timing remains unresolved. ADR-0023 stays
parked at G3. No Text, placement, pagination, PDF, provider, licensing or history
implementation is authorized. Commit and push remain with the owner.

Earlier milestone records are in the
[status archive through 2026-11-02](archive/STATUS-history-through-20261102.md).
