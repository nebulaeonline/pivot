# Visible-only Clear and Fill

ADR-0035 is accepted and closed by parent implementation review (2026-10-31),
including the submitted six-target validation and resource/source qualifications.

Edit offers three Clear Visible commands: Contents, Formatting (Built-in Defaults),
and Both. Contents sets existing content to Blank, retaining stable positioned identities
and Blank anchors; absent positions remain absent. Formatting explicitly overwrites all
ten built-in style properties at every addressed visible position, including individual
cell exceptions. It does not inherit worksheet defaults. Formatting-only Clear is
graph-free and leaves the graph revision unchanged; explicit-default assignments are
canonical changes even when they look identical to inherited values.

Visible Fill Down/Up uses the first/last visible selected row; Right/Left uses each
visible row's left/right edge. All source/destination pairs are fixed before staging,
using one immutable pre-operation capture. Rust's trusted duplication preserves exact
types, leading-zero Strings, styles, rejected drafts, reference/fault annotations and
structured bindings, translates formulas by actual worksheet offsets, and validates
candidate cycles. Hidden geometric edges never supply content. Degenerate operations
still validate admission and require exact review before returning unchanged state.

Both commands derive temporary rows from the qualified no-filter/applied view and
ordinary selected rectangle. Pending/failed/NeedsReview maps, drafts/editors, dragging
and projected records block commands. No permanent multi-area selection is created.
Clear is bounded at 4,096 logical slots including Blank/absent cells; Fill at 256 including
source edges. Oversized operations refuse with the bound and no auto-chunking. Ordinary
active-cell Clear/Delete, Fill, shortcuts and full-rectangle confirmations remain intact.

Prepare stores one exact, charged, one-use confirmation bound to command bytes,
worksheet, selection, revision/computational state, visibility generation and actual
rows. The SDK additionally checks interaction state. Review shows all actual row runs,
source and destination edges/columns/counts, skipped hidden rows, mode/direction and
limit, including zero skipped rows and degenerate no-ops. Cancel is the default;
explicit matching-ticket cancellation releases abandoned review state. No clipboard
access is needed. Changed state refuses; actions never replay automatically.

A dedicated bounded Clear transaction seam stages Clear edits and batched sparse
property-map resets privately before one publication. Reset coalesces consecutive rows,
subtracts only addressed spans and normalizes property maps once, with one accumulated
existing work meter. Fill uses the same atomic content/style duplication seam as
ordinary Fill with exact pairs. Cancellation or admission failure installs no partial
content, style, identity or document transition. No direct hidden-row content/style
writes occur; ordinary binding may demand hidden Blank anchors and hidden dependents
may recalculate. Filter refresh follows the complete canonical write set.

Private session v13 adds typed Prepare/Clear/Fill and matching-ticket Cancel operations;
The current WB13 container retains the metadata; hosted v1, CB5/CB6 and existing
resource limits are unchanged. Core mask/identity/
style/rollback/formula tests, real IPC and in-memory routed-menu checks supplement all
six successful headless and attended managed/AOT suites. See the
[implementation handoff](../handoffs/2026-10-31-visible-only-clear-and-fill.md) for
selected receipts, source qualifications, resource observations, cleanup and per-project
cloc. No additional validation or Native AOT rerun is required by the acceptance.
