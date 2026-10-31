# ADR-0034: Visible-only operations on filtered data

Date: 2026-10-31.
Status: **Design approved with clarifications by parent review (2026-10-31).
Implementation requires separate owner authorization; no implementation or validation run started.**

The parent accepts the independent commands, exact shape matching, actual-coordinate
formula translation, bounded CB6 plus current CB5, private v12, unchanged WB12/hosted
v1, atomic mapping and existing resource limits. Three mandatory acceptance invariants:

1. **Native precedence:** reject a recognized malformed/unsupported preferred native
   payload without falling through to another native representation or TSV. Genuine
   TSV-only input retains external-text semantics; native-only CB6 is permitted when
   unquoted TSV cannot faithfully carry the content.
2. **Hidden-row integrity:** no direct hidden-cell content or formatting writes.
   Canonical formula binding may resolve hidden targets or demand-create Blank
   reference anchors; it does not authorize hidden value/style assignments.
3. **Exact destination review:** every Paste Visible shows actual ordered row runs,
   columns, addressed-slot count and skipped rows. Never substitute a bounding box
   for the write set. The one-use ticket binds the immutable payload, precise plan,
   workbook state and qualified visibility generation.

The parent accepts the proposed independent correctness, resource, clipboard-race
and six-target managed/AOT validation gates for future implementation. Preserve
original failures, normal attended consent, batch archive/cache maintenance and
diagnostic milestone LoC reporting. This design approval does not start those runs.

Predecessors: [filtering](0033-worksheet-filtering.md),
[rectangular clipboard](0017-rectangular-clipboard-and-formula-copy.md),
[Fill](0022-four-direction-rectangular-fill.md).
Current clipboard contracts are the implemented CB5 rules, including subsequent
typed values, formatting, worksheets and explicit deleted-reference provenance.
Historical ADR-0017 capacities and value restrictions are superseded by those milestones.

## Problem and boundary

ADR-0033 deliberately retains full-rectangle operations with exact hidden-row
confirmation. Users also need to copy visible records or paste into visible rows
without modifying hidden destinations. Simply removing hidden rows from the current
clipboard loop would lose formula origins and leave destination correspondence unclear.

Add explicit **Copy Visible Cells** and **Paste into Visible Rows** commands for
ordinary worksheets. Keep ordinary Copy/Paste, their shortcuts and full-rectangle
confirmations intact. Copy Visible controls source membership; Paste Visible controls
destination membership. Neither establishes a sticky mode. No Cut/Move, visible-only
Clear or Fill, arbitrary multi-area selection, series generation, history, tables,
filter redesign or query language is included. Clear/Fill require later scoped designs:
Clear needs separate content/style choices; Fill needs a reviewed source-edge and
direction contract across gaps. Existing Fill remains unchanged.

## Alternatives considered

| Alternative | Benefit | Problem / disposition |
| --- | --- | --- |
| Keep only confirmed full rectangles | Existing contract remains simple | Does not supply the requested selective workflow; keep as the ordinary alternative |
| Skip hidden slots while treating the image as an ordinary CB5 rectangle | Small apparent change | One CB5 origin cannot describe different source-to-destination row offsets; reject |
| Rewrite formulas into a synthetic dense CB5 source at Copy time | Reuses payload shape | Can create out-of-bounds intermediate references even when the eventual paste is valid; obscures real origins; reject |
| Pack values only, discarding formulas | Simple portable values | Silently changes definitions and fault/typed semantics; defer any explicit Paste Values feature |
| Store disjoint row origins and translate geometrically per formula | Retains existing copy-axis rules and supports both destination modes | Needs a bounded native representation extension; recommended |
| Rewire references through a source-row-to-destination-row lookup | Keeps some cross-record links within copied records | Ambiguous for ranges, uncopied targets and absolute axes; changes established copy semantics; reject for this milestone |

The recommended contract copies selected rows in worksheet order and uses each
formula cell's **actual source and destination coordinates** for translation.
Packing records does not redefine reference geometry or calculation membership.

## Selection and row correspondence

The normal selection remains one inclusive rectangle, with its existing anchor and
active coordinate. A visible operation derives an ephemeral ordered vector of actual
worksheet rows from that rectangle and one qualified visibility map. It creates no
computational SelectionId for that vector and is not a saved multi-range selection.
Ordinary copied formulas may still create their usual destination range identities.

Visibility means row visibility under ADR-0033, not whether a row is currently on
screen. Columns remain contiguous; no hidden-column feature is introduced. A selected
visible header or a visible row outside the filter's data interval participates just
like any other selected visible row. Do not silently remove headers or stop at filter
boundaries. Show addressed row/column counts and the actual destination row runs.

Only no-filter (all rows) and successfully applied, current filter states qualify.
Block both visible commands during Pending, Failed or structural NeedsReview, even
though their fallback presentation shows all rows. Require Review/Reapply/Clear as
appropriate. Do not interpret the fallback as the user's intended visible subset.

Copy Visible takes visible rows within the normalized source rectangle, ascending
by actual worksheet row, and all its columns left-to-right. Selection orientation
does not reverse records. Zero visible rows refuses without changing the clipboard.
Blank positions and empty Strings are included as distinct slots; do not compact
away empty rows or cells, evaluate formulas into values, or create source anchors.

Paste Visible accepts an immutable native image or existing external TSV rectangle:

- **Single-cell destination:** the anchor must be visible. Use the next N visible
  worksheet rows at or below it, including the anchor, and W contiguous columns
  starting there. Insufficient remaining visible rows or a column overflow refuses.
  This can cross a filter boundary; the review must show that actual destination.
- **Multi-cell rectangle:** derive its visible row vector. Its count must be exactly
  N and its column width exactly W. No tiling, broadcasting, truncation, automatic
  selection resizing or silent overflow beyond the selected rectangle.
- Empty or mismatched destinations refuse before any write. A one-row source is
  not broadcast over multiple selected visible rows.

For visible source rows [2,5,9] and visible destination rows [20,22,25], source row
2 maps to 20, 5 to 22 and 9 to 25. Width is unchanged. Source and destination may
have different gap sizes; actual row counts, not bounding-box heights, must match.

| Source command/image | Ordinary Paste | Paste into Visible Rows |
| --- | --- | --- |
| Ordinary Copy / rectangular CB5 | Existing contiguous rectangle | N visible destination rows |
| Copy Visible / proposed CB6 | Packed contiguous rectangle, preserving per-row origins | N visible destination rows, preserving per-row origins |
| External TSV | Existing contiguous rectangle; literal formula addresses | N visible rows; literal formula addresses |

Ordinary Paste of CB6 retains ADR-0033's full destination-rectangle confirmation if
that rectangle includes hidden rows. It does not become visible-only because the
source was copied visibly. A newer receiver must decode a recognized native payload
or refuse; it must never retry the operation as text after native failure.

After single-anchor Paste Visible, keep the active cell at the original anchor;
briefly highlight only addressed cells and report the actual pasted count. Do not
automatically select a bounding rectangle that implies hidden destinations were
written. For a matching multi-cell selection, preserve its orientation. These
highlights are feedback, not an alternative persistent selection or a new command
mode; ordinary commands continue to use their ordinary selection contract.

## Formula and identity contract

For each native formula at source (sr,sc) mapped to destination (dr,dc), apply the
existing Rust translation with offsets (dr-sr,dc-sc). Dollar markers retain their
existing copy-axis meaning, not a new reference-lifetime meaning. Use effective
bound coordinates captured at Copy, not a lookup of historical source spelling.

For B5 pasted to D12, relative =A2 becomes =C9, =$A2 becomes =$A9,
=A$2 becomes =C$2 and =$A$2 remains =$A$2 (assuming those are the effective targets).
It does **not** become =C10 merely because source row 2 was also copied to row 10.
For source B2 =A2 and B5 =A5 mapped to D10 and D12, results are =C10 and =C12.
These examples distinguish geometric translation from record-link rewiring.

Ranges translate their endpoints by that formula cell's offset; they remain ordinary
fixed rectangles and can include hidden or uncopied rows. Never turn SUM(A2:A5)
into a visible-row union or remap its endpoints through copied-row ordinals.
Preserve current worksheet-qualifier rules and bytes outside translated reference
spans. Check bounds only against actual source annotations and final destinations,
not a synthetic packed worksheet. Out-of-bounds translation refuses the whole paste.

Retain current CB5 semantics for opaque rejected drafts, arithmetic/cycle errors,
explicit deleted-reference fault operands, and refusals for genuinely unrepresentable
bindings or structured-output OS clipboard transfer. Fault provenance must not be
resurrected by looking up old source coordinates. Preserve signed zero and exact
Decimal96 parts, exact String bytes/leading zeros, Boolean and Blank/empty String.

Existing destination cells retain their identities; newly materialized destinations
receive local identities. No source CellId, SelectionId or executable handle crosses
the clipboard boundary. Prepare source slots before candidate writes, including
overlap, and resolve all copied definitions against one final candidate placement.
Source changes or close after a successful Copy do not invalidate the portable image.

Visible-only restricts **direct content/style writes** to the planned rows. Ordinary
formula binding may resolve hidden targets or demand-create live Blank anchors there;
that does not write hidden values/styles or create visible-only calculation semantics.
Existing hidden cells retain their identities, content, bindings and formatting.
Previously absent hidden reference targets remain Blank even if an anchor is needed.

## Native representation and external text

Recommend a separate **PIVOTCB6 visible-row image**: row count N, width W, first
source column, strictly increasing actual source row coordinates, then N*W slots
and their resolved styles in dense row-major order. Each slot uses the current CB5
typed content, effective-reference annotations, rejected-draft and fault conventions.
Derive a formula's original position from its row entry and column ordinal. Require
N>=1, unique/in-range rows, valid source column extent, exact counts and no trailing
bytes. All headers, lengths, references and typed fields are untrusted input.

Use distinct v6 private clipboard identifiers on Windows/Linux and macOS, following
the existing platform naming scheme. Ordinary Copy continues to publish CB5; new
receivers support both current representations. Prefer CB6 if both are supplied;
a malformed/unsupported preferred native representation refuses without falling
through to CB5 or text. Do not publish a fabricated CB5 rectangle alongside CB6.
This is not a general obsolete-version reader or workbook migration facility.

The extension is justified by per-row formula origins that CB5 cannot represent
reliably. WB12 and hosted v1 stay unchanged. Propose private session **v12** for
explicit visible Copy/Paste planning, row mappings and exact destination tickets.
No native workbook version change or computational value/range type is needed.

Publish packed TSV alongside CB6 only when every rendered slot can use existing
unquoted TSV without changing cell boundaries/content. Strings retain the apostrophe
type prefix, including leading-zero identifiers and empty Strings. A stored String
or rendered formula containing tab/CR/LF makes the image native-only; do not replace
characters, introduce CSV quoting, omit cells or reject otherwise valid native data.
Say that external text is unavailable for that image. Do not omit TSV merely to
evade combined byte admission. Plain TSV retains existing literal-address formula
semantics; it cannot preserve native origins or all provenance.

Older Pivot builds and other applications do not understand CB6. If they consume
the published TSV, that is external text, not a compatible native relative copy.
Do not claim control over unrecognized-format handling in older software. The new
receiver enforces no fallback for its recognized native representations. Copy while
editing remains ordinary text copying; both new worksheet commands are unavailable
inside editors, structured proofs and projections.

Native paste transfers the same resolved per-slot styles as CB5, including styles
on Blank slots; it does not transplant worksheet defaults or row/column style layers.
External TSV retains destination formatting. Sparse destination style fragmentation
uses existing admission and refuses atomically if it cannot fit; no fallback to
values-only, broader defaults or raised style limits.

## Exact planning, confirmation and atomicity

Rust derives membership and complete addresses; C# neither infers row mappings nor
parses/translates formulas. Copy captures one qualified source state/map and validates
that interaction before OS publication using existing stale-copy/ownership guards.
Do not retain source captures in an OS data provider. Once published, the owned
image is a self-contained snapshot; later source-map changes are irrelevant to paste.

Paste Visible always presents an explicit review before installation, even if there
are no skipped rows: source dimensions, destination sheet, ordered row runs/columns,
N*W addressed slots, skipped destination rows and native/external formula policy.
Warn that cross-record references use actual copy offsets rather than packed-record
rewiring. Never present the bounding box as the set of written cells. Refuse shape,
state, capacity or visibility errors before offering confirmation.

A one-use session ticket binds the exact payload bytes/kind, operation, destination
selection/anchor, ordered actual destinations, target worksheet, complete expected
state, visibility generation and counts. Retain at most one charged bounded plan;
derive and verify it again at dispatch. Destination edits, formatting changes, sheet
switch, structural change, Reapply, map change, clipboard replacement or selection/
interaction changes invalidate it, even if row bits happen to remain equal. The
desktop revalidates clipboard ownership/content and interaction state after the
dialog. Cancel/stale/error never dispatches an edit or causes implicit re-preview/
confirmation/replay. A later deliberate command can create a new plan.

Install all mapped slots and styles through one cancel-aware canonical transaction.
Check cancellation and state again at publication. Never loop committed per-row
pastes or split work to fit admission. On failure, leave roots, identity ledger,
styles, revision, dirty state, saved checkpoint and selection unchanged. Paste never
writes or restores the OS clipboard; another application's replacement remains theirs.
No-op detection follows existing typed definition/style equality, not displayed
values alone. After success, ordinary filter refresh may hide newly edited rows;
the write plan remains the pre-operation row vector and must not chase changing
membership while applying. Mutation and its normal derived refresh remain distinct.

## Bounds and accounting

Keep **256 addressed slots**, **256 KiB combined native/text bytes**, **128 KiB
source/text work**, **16,384 token work**, **8,192 reference work** and existing
**8 MiB transient admission**. Include row metadata in byte/scratch accounting and
repeated formula translation in work. No increase to worksheet capacity, transaction,
frame, style, graph, selection, filter or allocation limits is proposed.

The source/destination bounding rectangle can exceed 256 positions, but N*W including
Blank slots cannot. Count visible rows with the already qualified bounded map before
enumerating slots; check multiplication and bounds before allocation. At most 256
row origins/destinations are needed. Bound row-map traversal by worksheet height and
the existing 33,554,432-unit work ceiling, with cancellation during scan, decoding,
translation and staging. Do not allocate a bounding-box matrix or demand-create
unselected virtual cells. Release captures, plans and charges on every path.

The initial implementation must demonstrate these inherited limits are sufficient.
If a bounded representation cannot fit, return for a specific design refinement;
do not silently raise budgets or fall back to another transfer contract.

## Implementation acceptance plan (not executed)

Independent small-coordinate oracles must enumerate explicit visibility masks and
compute source/destination row lists separately from production rank/select code.
Cover unequal gap spacing, opposite selection orientations, header/outside-filter
rows, single anchors, exact multi-selection, no filter, zero rows, shape mismatch,
end-of-axis exhaustion and Pending/Failed/NeedsReview refusal.

Verify every source/destination command combination, overlap from immutable snapshots,
cross-sheet/workbook/process transfer, source edit/close after Copy, and no hidden
direct writes. Compare hidden nonempty identities/styles/bindings before/after and
separately test ordinary hidden Blank-anchor demand from new formulas. Exercise
mixed absolute axes, live moved targets, cross-record references, fixed SUM geometry,
qualified references, bounds refusal, rejected drafts, explicit reference faults,
signed zero, exact leading-zero Strings, empty Strings, Blank and resolved formatting.

Test malformed/truncated/duplicate/unordered/out-of-bounds CB6 metadata, native
precedence/no fallback, native-only separator-containing Strings, CB5 interoperability
and external TSV's literal formulas. Test clipboard and map replacement during
confirmation, generation-only invalidation, one-use tickets, cancellation, late quota
and style-fragment failure, no partial installation and zero final runtime charge.

Use bounded 256-slot transfers with maximal sparse gaps and full-height masks,
four retained roots, dense versus sparse styles, long source/ref limits and near-
quota payloads. Report observed planning/commit maxima separately from OS clipboard
and dialog time; target <=1 second per bounded engine operation and retain the
256 MiB steady/512 MiB peak objectives on stated corpora. Source enumeration and
map reads must not materialize the 65,536 by 1,024 virtual worksheet. These are
future acceptance objectives, not measurements or promises of raised capacity.

For the first mapped-write/native-format implementation, use the existing six-target
headless and managed/AOT GUI matrix, Windows x64 pilot first, then remaining targets
with normal local graphical consent and short launchers. Include Windows/Linux ARM64
and macOS x64 Rosetta explicitly. Use actual OS clipboard round trips and independent
peer processes; preserve original failures, avoid shared-host timing contention,
archive/hash-verify before pruning each completed batch, and refresh per-project LoC
as diagnostic evidence. Do not repeat accepted ADR-0033 qualification merely for this
design review. Future implementations require separate parent and owner approval.

## Parent disposition and next step

The parent approves the independent command split, geometric cross-record formula
rule, bounded CB6/private-v12 representation, native-only text policy, exact destination
review and initial Copy/Paste-only scope, subject to the three invariants above.
The current resource limits and six-target future validation plan are accepted;
no budget increase, implicit mode fallback or migration is approved. Clear, Fill,
Cut/Move, arbitrary multi-area selection, tables, history and filter redesign remain
outside scope. Wait for separate owner implementation authorization. No feature
code, workbook/protocol change, build, GUI launch or native validation was performed
in this phase. ADR-0023 remains parked at G3.
