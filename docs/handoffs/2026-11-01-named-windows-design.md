# ADR-0039 design handoff

Date: 2026-11-01.
Disposition: **Architecture accepted with three clarifications (2026-11-01); implementation requires separate authorization.**

[ADR-0039](../adr/0039-named-windows-and-document-composition.md) defines workbook-owned
Windows, producers and layouts for spreadsheet-driven business documents. Supporting
contracts cover [references/binding](../architecture/named-windows-and-binding.md)
and [geometry/Text/pagination](../architecture/document-composition.md).

The accepted architecture separates Grid/Text from bound/unbound ownership. Windows have
independent A1 scopes and stable IDs; placements and page fragments have no calculation
authority. Worksheet producers publish coherently with committed workbook state;
external producers refresh explicitly and retain successful immutable snapshots on
failure. Formula consumption is resolved: worksheet sources retain their CellIds;
external Decimal outputs share captured scalar leaves. Other external types remain
presentation-only until a separately reviewed typed-source extension.

The review preserves these consequential choices:

- Existing bound scalar references follow source item identity, while copy/fill is
  an explicit coordinate translation/rebind. Generation-only sources cannot pretend
  their ordinals are durable keys. Window deletion has a lifetime guard.
- Independent geometry bands avoid editable 2D regions, but combined mappings can
  create gaps/overlaps. Gaps remain background; overlapping candidates refuse. This
  requires invoice-oriented usability qualification before implementation promotion.
- Initial Grid-to-Text conversion is lossless but limited to blank, unreferenced
  rectangles. All anchors/formatting remain dormant; nonempty conversion refuses.
  Reverse requires empty Text and the intact reservation. No implicit destructive
  merge, identity retirement/resurrection or history support is promised.
- One ordered body flow plus fixed page bands bounds sibling competition and final
  page layout. Ordinary content defaults to every page without becoming a Window.
- Typed invocation forks an isolated template instance; capture pins calculation,
  selected producer generations, layout, assets and fonts. Rendering never queries
  or mutates a workbook. Leading-zero identifiers remain exact Strings.
- New native/private contracts are required for production objects; no historical
  migrations or CB5/CB6 extension is assumed. Rendering has separate subordinate
  admission limits; proposed ceilings are not performance measurements.

The ADR includes a Mermaid architecture diagram, alternatives, explicit source
findings and six proposed implementation scopes without speculative ADR numbers.
PDF is a headless architectural boundary; no backend has been chosen or implemented.
No feature code, tests, GUI/AOT runs or platform activity accompanies this design.
ADR-0038 remains accepted with its original Windows ARM64 cold-build qualification;
ADR-0023 remains parked at G3.

## Review clarifications and next boundary

Geometry promotion requires concrete invoice header, line-item and totals/payment
examples, interacting horizontal/vertical sections and gap/seam selection tests.
Frequent refusal of normal layouts requires a model revision, not user workarounds.
Text conversion must explain refusal reasons and restore the original grid/formatting
only with empty Text and its intact reservation. All invocation routes, including
direct-cell submission, remain confined to declared InputIds; exact String identifiers,
decimal lexemes, Blank distinctions and atomic rejection are mandatory.

Resource ceilings are accepted provisionally, subject to qualification without a global
budget increase. Preserve the six-scope sequence: unbound Grid identity/references,
producer bindings, section geometry, Text, capture/layout/preview, then invocation/PDF.
Architecture acceptance does not authorize implementation or establish that the
geometry usability gate passes. Await separate implementation direction.
