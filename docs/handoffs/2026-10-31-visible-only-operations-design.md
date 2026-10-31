# ADR-0034 — visible-only operations design review

2026-10-31. Design approved with clarifications by parent review.
No implementation or test runs; separate owner implementation authorization required.

The parent accepts the design and makes three requirements explicit acceptance
invariants: preferred recognized native failure never falls back; hidden cells
receive no direct content/style writes despite permitted ordinary Blank-anchor
demand; every Paste Visible reviews the actual ordered destination rows/columns,
counts and skipped rows, bound to the precise one-use immutable operation ticket.
Native-only CB6 is accepted where TSV is unfaithful. Existing budgets, atomicity,
WB12/hosted-v1 preservation, CB6/private-v12 scope and future six-target validation
are accepted. Preserve original failures, attended consent, batch cleanup and LoC
reporting. Implementation remains on hold for separate owner authorization.

The owner committed ADR-0033 (`196dfd2`) and authorized ADR-0034 design. Filtering
remains accepted and closed; its source-qualified evidence does not require reruns.
The [accepted design ADR](../adr/0034-visible-only-operations.md) compares transfer contracts
before recommending explicit Copy Visible Cells and Paste into Visible Rows.

## Accepted contract

- Keep ordinary Copy/Paste, shortcuts and exact full-rectangle confirmation intact.
  The new commands independently choose source and destination membership; there is
  no sticky visible-only mode. Defer visible-only Clear/Fill, Cut/Move, arbitrary
  multi-area selection, history and tables.
- Derive an ordered actual-row vector from the normal rectangular selection and a
  qualified visibility map, including visible headers/outside-filter rows if selected.
  Do not create computational range identities for that vector. Pending, Failed and
  NeedsReview block visible operations; no-filter explicitly means all rows.
- Copy Visible packs source rows in worksheet order, retaining every column and
  Blank slot. Paste Visible maps N image rows to N visible destination rows of width
  W. A single visible anchor finds the next N visible rows; a multi-cell selection
  must have exactly N visible rows and width W. Mismatch/overflow refuses, without
  tiling, truncation, broadcasting or silent extension beyond a multi-cell selection.
- Preserve each formula's actual source origin and use existing geometric copy
  offsets and dollar-axis rules. Source B5 =A2 pasted to D12 becomes =C9, even if
  source row 2 was separately copied to row 10. It does not become =C10 by record
  lookup. SUM remains a fixed rectangle, potentially including hidden/uncopied rows.
  This deliberately avoids changing established binding or calculation semantics.
- Existing destinations retain identities; copied definitions receive destination-
  local bindings. Hidden rows receive no direct content/style writes. Ordinary
  formula references may still resolve hidden targets or create live Blank anchors
  there under existing semantics; no visible-only computational membership appears.
- Always review the exact Paste Visible destination row runs, columns, counts and
  formula policy. Bind a one-use ticket to immutable payload, operation, ordered
  destinations, expected state, visibility generation and selection/interaction.
  Changed clipboard/map/state refuses without implicit confirmation, replay or a
  different transfer mode. One atomic transaction applies the pre-operation plan;
  filter refresh cannot change membership midway through the writes.

## Accepted representation decision

CB5 has one rectangular source origin. Removing hidden rows makes a formula's
packed ordinal different from its real origin; pretending it is a dense rectangle
translates references incorrectly. Pre-adjusting formulas into a synthetic CB5
rectangle can cross axis bounds before an otherwise valid final paste. Values-only
packing loses definitions, and record-lookup remapping changes reference semantics.

Approve **CB6 for visible-row images**, preserving a bounded strictly increasing
source-row list plus first source column, dense typed slots, resolved styles and
the current CB5 effective-reference/rejected-draft/fault conventions. Ordinary Copy
still produces CB5. New receivers understand both current formats; recognized native
failure never falls through to another native representation or plain text.

Ordinary Paste of CB6 packs into a contiguous destination rectangle, with ADR-0033's
hidden-row confirmation if necessary. Paste Visible accepts CB5, CB6 or external TSV
and controls destination membership independently. External text formulas bind their
literal addresses, without invented source origins. Older applications consuming
TSV do not gain compatible native-relative-copy semantics.

Approve v6 platform clipboard identifiers and private session v12; **WB12 and hosted
v1 remain unchanged**. No general migration or obsolete-version reader is assigned.
The representation extension is specifically required by per-row formula origins.

Native data preserves exact decimal parts/signed zero, leading-zero Strings, empty
Strings, Blank, Boolean, formulas, current fault provenance and resolved per-slot
formatting. Publish packed TSV only if existing unquoted TSV can represent every
slot faithfully. Separator-containing Strings/formulas produce a clearly reported
native-only image; never flatten, quote as CSV, drop cells or coerce values.
Native style transfer includes Blank slots; external TSV retains destination styles.

## Bounds and future evidence

Retain 256 addressed slots, 256 KiB combined bytes, 128 KiB source/text work,
16,384 token and 8,192 reference work, 8 MiB transient admission and existing global
limits. Bounding boxes may be tall, but count visible rows before allocating slots;
do not enumerate the virtual rectangle. Row origins/mappings are at most 256 entries.
Budget refusal remains atomic; no per-row commits or quota-driven mode switch.

Future implementation must use independent small-coordinate membership/translation
oracles, verify complete hidden-cell preservation and normal Blank-anchor behavior,
exercise all source/destination mode combinations, overlap, different gap sizes,
malformed native data, clipboard/map races, cancellation and late quota/style failure.
Use bounded maximal-gap workloads and unchanged memory/performance objectives.
The proposed initial mapped-write/clipboard-format qualification uses all six
headless and managed/AOT targets, Windows x64 pilot first, with actual graphical
consent, short launchers, Rosetta qualification, retained failures and batch cleanup.
Refresh per-project LoC at implementation close as diagnostic evidence, not a target.
None of these checks was executed during design.

Parent design review is complete. The independent command split, geometric formula
rule, CB6/private-v12 representation, native-only text policy, exact destination
review and Copy/Paste-only scope are approved with the three mandatory invariants.
Wait for separate owner implementation authorization. ADR-0023 remains parked at
G3; no feature work is started by this design acceptance.
