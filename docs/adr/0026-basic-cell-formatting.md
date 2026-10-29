# ADR-0026: Basic cell formatting

Status: **Design accepted with parent clarifications (2026-10-28).**
Parent authorizes one bounded implementation milestone. **Implementation is on hold
by the owner's latest instruction until explicitly told to proceed; no code has started.**
Date: 2026-10-28. ADR-0025 is accepted and closed, with its recorded source qualification.

## Decision and scope

Store cell formatting as sparse document-owned presentation metadata, independent
of definitions, calculated values and graph identities. Styling an empty position
must allocate **no CellId, GraphNodeId, graph node, formula or computational selection**.

Deliver numeric display formats, display precision/rounding, bold/italic, horizontal
alignment and foreground/background colors. Apply to rectangular selections and
retain styles through native Copy/Paste, Fill and complete Save/reopen. Continue
using the existing Rust workbook/session and Avalonia desktop; no formatting service,
style plugin framework, migration subsystem or history investigation.

Formatting never changes calculation rounding, value type, dependencies or scenario
inputs. Defer ROUND and other calculation functions, Integer/date/time semantics,
locale configuration, custom format strings, conditional formatting, borders, merged
cells, wrapping, font families/sizes, row/column styles and structural edits.

## Presentation contract

| Setting | Proposed behavior |
| --- | --- |
| General | Existing exact invariant Decimal rendering, retaining submitted scale where present; no grouping or display quantization |
| Number | Fixed 0–28 fractional places, optional thousands grouping |
| Currency | Number rendering plus a selected prefix: dollar, euro or pound symbol; no currency value type or conversion |
| Percentage | Exact decimal-point shift by two for display, then quantization and `%` suffix |
| Display rounding | All seven existing rounding modes; nearest/midpoint-away-from-zero by default |
| Emphasis | Independent bold and italic flags using the existing desktop font |
| Alignment | Auto, Left, Center or Right; Auto uses right for Decimal, left for String/error and center for Boolean |
| Colors | Optional opaque RGB foreground/background; unset uses the desktop's normal theme colors |

Number/Currency/Percentage presets start at two decimal places; all remain editable.
Currency starts with the dollar symbol and grouping enabled. Number starts without
grouping. General ignores numeric precision, rounding and grouping settings while
selected; switching formats must not silently alter values. Negative numeric display
uses a leading minus; a value rounded to displayed zero suppresses its minus sign.
The minus precedes a currency prefix (`-$1.00`). These prefixes label display only;
they do not establish a financial currency code on the value.

Use deterministic separators (`.` decimal and `,` grouping), independent of OS culture.
Examples: 1234.5 as dollar currency/two places is `$1,234.50`; 0.1575 as percentage/one
place is `15.8%`; -2.5 at zero places/default rounding is `-3`. Rounding modes apply
to display only. Padding zeros is presentation, not stored Decimal scale mutation.

Percentage must not multiply through bounded Decimal96 arithmetic: the maximum
Decimal96 value must still display as a percentage without calculation overflow.
Use bounded coefficient/digit operations to shift and round exactly, with at most
128 UTF-8 bytes of formatted numeric output. Do not use binary floating point or
an unbounded arbitrary-precision dependency. Compare display results against exact
integer/rational reference cases, including carries and both signs for all modes.
Parent clarification: display quantization must not accidentally invoke computational
arithmetic or change the underlying Decimal96. Reference expectations must be
independent of the formatter implementation, especially at maximum-value percentages.

Rust owns numeric rendering using the Decimal96 rounding contract. C# receives
formatted numeric presentation separately from the authoritative typed value and
definition; it does not independently round. Boolean/String/error presentation stays
typed: a currency format on text `123` does not parse it, and a percentage format on
TRUE does not turn it into 100%. Visual styles apply to all kinds. Blank positions
can display a background without any calculated value or text.

The formula bar/in-place editor continues to edit exact input or authored/effective
formula source, never the rounded display string. Clipping remains single-line;
styling does not resize rows/columns or introduce wrapping.

## Sparse ownership, bounds and atomicity

Add a persistent style index keyed by worksheet identity and sheet-local axis
position under the existing immutable workbook root. Store only non-default styles.
A default-style reset removes the entry; it must not leave a computational anchor
or accumulate style tombstones. Replacing a position's literal with a formula (or another value kind) retains its
style without following or rebinding the content identity. Test position-style
behavior independently of stable formula bindings and calculated values.
Use compact value records; no style-ID registry or
deduplication subsystem is needed for this slice.

Initially enforce a workbook-wide ceiling of **4,096 non-default styled positions**, and at
most **4,096 visited positions per formatting command**. Retain existing sheet bounds,
16-sheet ceiling and global memory/admission authority. These are limits, not a
promise that every maximum fits simultaneously: account for persistent index nodes,
styles, pinned roots, command scratch, viewport responses and file/clipboard bytes.
Refuse large selections atomically before expansion; no whole-row/column shorthand
or million-cell rectangle materialization. New resource counters must not consume
or increase the existing live/allocated computational-cell budgets.
Existing clipboard and Fill area limits remain; the formatting-command bound is
separate and does not enlarge content-edit or clipboard budgets.

Parent clarification: both 4,096 bounds are revisable admission policy, independent
of computational-cell limits. Keep them in explicit style-limit configuration rather
than fixed-capacity representations, narrow identifiers or a file-format maximum.
Readers validate counts against the configured policy and existing byte/memory
bounds; raising style capacity later must not itself require a representation or
format cut. No capacity increase or configuration UI is assigned now.

Commands carry explicit WorksheetId and exact workbook expected state. A patch has
Keep, Set and Reset semantics per field; mixed selections must not silently acquire
defaults for untouched fields. Validate the complete patch/rectangle and admission
before publishing one candidate root. Invalid/stale/quota/cancelled operations leave
styles, definitions, dirty state and graph unchanged. A semantic no-op creates no
revision. A real formatting edit changes workbook revision/dirtiness, with no change
to graph definition/input revision, invalidation or calculated values.

Read styles for the bounded visible rectangle independently of populated cells,
including style-only blank positions. Stamp the presentation frame with document
generation, worksheet and workbook revision. Reuse existing stale-response guards;
never paint old styles onto another tab. Workbook captures/Save pin coherent style
and content roots; calculation captures retain their values regardless of later
formatting. Active-tab/view preferences remain client state.

## Editing, clipboard and Fill

Styles belong to positions, not stable cell identities or formula targets. Specify
each editing operation explicitly:

- Clear content retains the position's formatting; Reset Formatting retains content.
- Existing same-sheet Move transfers the source's complete style with its content
  and resets the source style, atomically under existing destination-admission rules.
  A default source style overwrites any destination style. Moving a style-only blank
  with the content Move command is deferred; native Copy/Paste handles such regions.
- Native rectangular Copy/Paste includes the complete source style for every slot,
  including blank/default slots. It replaces destination styles as well as content.
  A styled blank copied onto an absent position stays style-only; clearing an absent
  destination during paste must not allocate a blank graph anchor.
- External TSV imports retain destination styles. TSV exports retain existing
  reversible raw definitions/values, without display rounding or styles. Do not
  replace currency input with a dollar-prefixed String or copy a formula's result
  instead of its expression.
- Four-direction Fill copies the same source edge's complete style alongside the
  existing definition translation. Blank edge entries can propagate styles without
  creating graph cells. Content and styles publish together or neither publishes.

Cross-sheet and cross-window native paste use destination worksheet positions and
the existing qualified-reference rules. Styles create no cross-workbook references.
Preserve malformed-native/unsupported-version refusal without silent TSV fallback.
Cut, Paste Special and separate paint-format tools are outside this slice.

## Desktop and representation plan

Use a compact Format pane for selection settings: numeric preset, decimal places,
rounding mode, grouping/currency symbol, bold/italic, alignment and a basic color
palette. Show Mixed for differing fields; Apply changes only explicitly edited fields.
Provide Reset Formatting. Use the existing editor commit path before Apply/Reset;
failed commits preserve the draft and prevent formatting. Pane navigation/cancel
alone does not dirty the workbook. No new shortcut system or native color dialog.

Extend the existing private protocol with bounded formatting commands and a style
viewport independent of Cell records. Preserve typed result data; rendered text is
presentation only. Use generated protobuf/JSON metadata compatible with Native AOT.
The daemon/frozen hosted v1 contract remains unchanged.

Propose **WB9** for the sparse style section, **private v7** for style commands/frames,
and **CB4** with distinct clipboard platform identifiers for per-slot styles.
Confirm the actual incompatible fields during implementation, then cut once with
explicit rejection. Save/Open validates sheet/axis ownership, unique style positions,
known enum/color/precision values, non-default canonical records and all resource
bounds before installation. No default entries, graph anchors or cached display
strings are persisted merely to represent formatting. Failed Open preserves the
complete prior document. Keep the pre-user compatibility obligation explicit;
do not create a migration or version-negotiation project.

Parked history adapters and separate structured-result proof modes may reject
unsupported formatting operations explicitly. Preserve their existing supported
behavior/builds; no sheet history or ADR-0023 work is implied.

## Implementation and acceptance

Parent approved direct implementation as one vertical slice, subject to the owner
hold above: sparse metadata/atomic
commands and exact rendering; session/viewport/UI integration; clipboard/Fill/Move
and native persistence; focused automated regression evidence. Deliver one concise
implementation handoff and stop for parent implementation review.

Acceptance must prove styling a blank rectangle leaves graph identities, allocated
cell count, dependencies and calculation revisions unchanged. Reset/reapply must
not accumulate entries; saved style-only blanks must reopen without graph nodes.
Test all seven rounding modes, maximum-value percentages, trailing zeros, negative
displayed zero, nonnumeric kinds, untouched exact values/downstream totals, mixed
patches/no-ops, stale/quota/cancelled rollback, old captures, sheet rename/switch,
Move/Clear, styled blank native paste/Fill, malformed/versioned clipboard refusal,
complete Save/reopen and atomic failed Open. Automate actual pane/grid behavior on
relevant native targets and verify AOT-compatible presentation handling using the
existing infrastructure. Do not repeat unrelated platform qualification.
