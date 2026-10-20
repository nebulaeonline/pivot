# ADR-0017: Bounded rectangular clipboard and formula copy

Status: **Proposed — requires parent review before implementation**.
Date: 2026-10-20.

## Context

Rectangular UI selection/navigation is accepted. Clipboard actions currently target
one active cell and native formula copy resubmits historical source without reference
translation. The [parent review](../handoffs/2026-10-20-rectangular-selection-review.md)
requests the smallest useful rectangular workflow with reviewed copy semantics.

Scalar formulas follow stable CellIds; SUM ranges follow fixed coordinate rectangles.
Original source can therefore differ from current effective reference coordinates.
Copy must deliberately create new bindings; move must retain existing identities.
The existing workbook already supplies atomic batches of at most 256 edits. This
proposal reuses that path and the canonical Rust engine, without raising capacity.

## Proposed decision

### 1. Rectangle correspondence and UI behavior

Copy the normalized inclusive UI rectangle from one coherent Rust generation S,
in row-major order. Source anchor orientation and active endpoint do not affect
content order. Reject area greater than 256 before enumerating cells or references,
even if almost every coordinate is blank. Selecting a huge region remains cheap;
copying it is an explicitly admitted operation, not a capacity promise.

For paste outside editing:

- A single selected destination cell supplies the top-left origin; expand to the
  copied width/height, rejecting any extent outside rows/columns 1–1024.
- A multi-cell destination selection must have exactly the source height/width;
  use its normalized top-left. No tiling, broadcasting, truncation or resizing.
- After successful single-cell-origin paste, select the destination rectangle with
  anchor at its top-left and active endpoint at its bottom-right, consistent with
  the accepted UI model; reveal the active endpoint. For an already matching region,
  preserve its anchor/active orientation. Failure leaves selection unchanged.
- Paste inside either text editor remains text insertion with no automatic commit;
  rectangular data is rejected there. F2/typing still targets one active cell.
- Copy and Paste become rectangle operations; Delete/Clear/Move remain active-cell
  operations in this slice and the UI must say so. Cut and drag-fill are excluded.

### 2. Relative and absolute references

Extend only spreadsheet syntax to accept `A1`, `$A1`, `A$1`, `$A$1`. Dollar markers
control **copy-axis translation**, not binding lifetime. All four still bind once to
stable scalar CellIds. Moving or clearing/reusing a target retains the existing
identity rules; `$` does not pin a formula to an occupant at a coordinate.

For each valid bound scalar occurrence, export its target's current coordinate in
S, not a fresh lookup of its historical submitted address. Let the offset be the
copied formula's destination position minus that formula's position in source S.
Add the row/column offset on relative axes; leave absolute axes unchanged. Resolve
those resulting coordinates in the candidate destination workbook through Rust.

| Source B2 copied to D5 (row +3, column +2) | New occurrence |
| --- | --- |
| A2 | C5 |
| $A2 | $A5 |
| A$2 | C$2 |
| $A$2 | $A$2 |

If B1's `=A1+1` follows its target when that cell moves to A5, copying B1 to B2
produces `=A6+1`, **not** `=A2+1`. For an absolute `=$A$1` whose bound target has
moved to A5, the copied source uses `=$A$5`. The original formula/source stays
unchanged. This effective-binding rule is a deliberate policy requiring review.
A formula moved from B1 to B3 still uses B3 as its new copy origin; its existing
bindings are not silently changed by that move.

A referenced live blank anchor has a coordinate and translates normally. A retired
or otherwise unplaced scalar target cannot be represented as an effective address:
reject native copy of the entire block with an actionable source-cell diagnostic.
Do not resurrect that reference by looking up the original source spelling.

### 3. Copying SUM

Translate each endpoint of a bound SUM rectangle using the same relative/absolute
axis rules, with markers allowed independently on each endpoint. Use the descriptor's
actual coordinate geometry in S and retain the original endpoint orientation, not
the positions of endpoint occupants. Normalize the resulting rectangle for binding.
For example, `SUM(A1:B3)` copied one row/column right becomes `SUM(B2:C4)`;
`sum(A3:$B$1)` copied two rows/one column right becomes `sum(B5:$B$1)`.

Create fresh destination SelectionIds in its worksheet axis namespace. No source
SelectionId or CellId is imported as a destination identity. Moving a formula still
preserves its original bound rectangle; copying explicitly creates translated bounds.
Checked row-major Decimal96 reduction and existing selection quotas do not change.

### 4. Source, decimal and error policy

Preserve numeric coefficient, scale and sign, including signed zero, in typed data.
Preserve formula bytes outside reference substrings, including whitespace, operators,
function spelling and decimal literal representation. Replace only reviewed scalar/
range endpoint spans; retain an unchanged lexeme if its effective address is unchanged,
otherwise render its new address with uppercase columns and the original dollar flags.
New source spans must match the transformed UTF-8 text.

The source formula remains untouched. The clipboard can retain its original source
and effective reference annotations for diagnostics; the copied formula's submitted
source is the transformed text with normal new transaction/binding provenance. No
unbounded copy-history chain is saved. Do not promise byte-identical copied source
when translating its reference substrings is the requested change.

- Syntax/bounds-invalid source drafts with no authoritative bindings are copied
  as unchanged opaque invalid formula text/state, without guessing a transformation
  or automatically promoting an old rejected draft under the expanded grammar.
- Valid bound formulas with arithmetic errors or cycles are translated normally;
  Rust may accept the result as an editable erroneous graph, as for ordinary entry.
- Valid syntax with retired/unplaced bindings rejects native copy, as above.
- If translating a valid reference exceeds worksheet bounds, reject the whole paste
  before installation. Do not clamp it, keep the old address, or invent `#REF!` tokens.
- Arbitrary typed `Content::Formula` whose source is not the validated spreadsheet
  syntax is unsupported by native block copy in this first slice; reject the block
  explicitly. Existing Rust typed-formula APIs remain unchanged.
- External nonformula text must be a representable exact decimal or blank. Reject
  unsupported text values/numeric overflow as a whole paste, retaining a correctable
  error/cell location without partially changing the workbook.

### 5. Clipboard representations and external text

Rust encodes a versioned portable typed block in private OS clipboard format
`application/x-pivot-cell-block-v1`, alongside plain UTF-8 TSV. C# transports bytes
and handles selection/focus, but does not parse, translate or bind formulas.
The typed block contains shape/source coordinates, exact numbers, original formula
text, effective coordinate/axis annotations and opaque-invalid draft kinds. It
contains no executable handles, trusted graph IDs, cached calculation results or
retained live lookup callbacks. Decode it as untrusted bounded data in Rust.

A copied block is independent of later source edits/moves/close. It can be pasted
into another local Pivot workbook/process using destination-local bindings. After
encoding, release temporary source roots and charges; there is no durable clipboard
capture/ticket retained in the Rust child. Clipboard/caller byte buffers are separate
from continuing engine-domain charges.

If typed data is present, use it or reject it explicitly if malformed/unsupported;
never silently drop into text and lose semantics. If only plain TSV is present,
parse its rectangle in Rust. Its formulas bind the literal supplied coordinates:
external text has no trustworthy copy origin and receives no implicit translation.
Thus copying Pivot-native B1 `=A1+1` to B2 uses relative native translation, whereas
pasting external text `=A1+1` binds A1 as written. This intentionally supersedes the
previous native single-cell formula-copy behavior after approval.

Render TSV formulas from effective coordinates at zero copy offset, so historical
lexemes do not silently point to reused occupants when exporting text. Preserve
numeric scale/sign in Rust formatting; no floating-point conversion. TSV cannot
carry all native provenance; losing the private format through another application
means the next paste is external text, not a native relative copy.

External TSV supports tabs and LF/CRLF row separators, no quoting/CSV/HTML parsing.
Remove at most one terminal row separator, then preserve empty columns/rows. Empty
input is one blank cell; `1\t\n` is one row with a trailing blank cell, and `1\n\n`
is two rows, the second blank. Reject ragged rows, bare CR, oversized or unsupported
contents. A copied blank clears destination content rather than preserving it.

### 6. Overlap, identity and atomic installation

Prepare every source value/reference from captured S before reading/writing any
destination. Pasting A1:A2 values [1,2] into A2:A3 yields [1,1,2] in A1:A3; it must
not cascade reads through the newly overwritten A2.

Map all slots, including blanks, to one row-major existing Workbook.apply batch,
with one exact expected destination revision. Fresh destinations get fresh local
identities; existing destination cells keep their identities and are overwritten
in place so existing dependents continue following them. Never reuse a source
CellId as a copied destination identity or implement copy by Move/Retire.

Ordinary canonical binding applies to the staged destination: references between
pasted cells bind destination identities, including anchors demand-created before
later batch entries. Bind SUM descriptors against the same final candidate placement
state. Syntax/cycle/arithmetic diagnostics remain editable; hard bounds, format,
resource, cancellation/admission or transaction failure installs nothing. Source
snapshots and candidate roots must be released on success and failure. One paste
must not be split into multiple visible commits just to fit a quota.

Selection/clipboard interaction generations and expected-revision checks retain
the accepted late-response protection. Escape before dispatch cancels; after dispatch
drain the authoritative response without claiming undo or automatic replay. A stale
revision is an explicit conflict. UI changes its selection only after success.

### 7. Initial admission profile (proposed ceilings)

| Resource | Ceiling |
| --- | ---: |
| Source/destination rectangle area | 256 slots, checked before enumeration |
| Output transaction | 256 edits, one per destination coordinate |
| Combined native and TSV copy output | 256 KiB |
| Accepted paste payload | 256 KiB |
| Each formula, before and after translation | 4096 UTF-8 bytes / 4096 tokens |
| Aggregate transformed formula source per block | 128 KiB |
| Aggregate parsed/postfix token work per block | 16,384 tokens |
| Aggregate annotated reference/endpoint occurrences | 8192 |
| Additional clipboard planning scratch admission | 8 MiB |

Existing 1024×1024 dimensions, 1024 live cells, 8192 allocated cell identities,
1 MiB workbook owned-byte estimate, 128 active selections, identity/consumer/member
work quotas and 1 MiB private frame limit remain unchanged. Clipboard planning scratch
is separate from existing batch mutation/admission reservations; every reservation
still fits the shared allocation domain. A final state that fits but requires an
unadmittable intermediate candidate may be conservatively rejected. These are
initial safety ceilings, not a promise that every rectangle of area 256 is admitted.

Reject oversized OS data immediately after receipt, before application parsing or
retention. OS APIs may materialize a provider's oversized payload before its length
can be checked; this proposal does not claim to bound arbitrary clipboard-provider
memory or process RSS. Enumeration and engine allocations remain admitted/bounded.

## Implementation and compatibility consequences

Add Rust workbook-owned copy-image/translation helpers and private child-session
COPY_BLOCK/PASTE_BLOCK operations with a versioned payload, same exact-revision and
framing rules. Old/mismatched child operations fail explicitly. Frozen hosted v1
schemas/capabilities remain unchanged. C# only supplies UI rectangles and transports
clipboard representations; it must not build a separate translator/evaluator.

Extend spreadsheet parsing with dollar-axis markers and endpoint spans. Validate
copy annotations against parsed source and authoritative captured bindings. Preserve
the general model grammar. Use existing WB2 logical records and formula source/spans;
copy flags can be recovered from validated source syntax without new persisted graph
identities or fields. Readers predating dollar syntax must reject unsupported syntax,
not reinterpret bindings. Verify this against the actual codec before implementation;
if an unavoidable format/capability change emerges, return for review instead of
silently changing WB2 or frozen hosted behavior.

A concrete compatibility case already exists: WB2's invalid marker currently
recomputes diagnostics using parse_spreadsheet, and `=$A$1` can be an existing
invalid saved draft. The new grammar must not auto-promote it on reopen or native
copy. Preserve the stored non-executable state. Use the same parser with a legacy
no-dollar diagnostic mode when necessary to validate that old rejected source;
this is diagnostic/provenance validation, not a WB1 reader or another evaluator.
Valid records still undergo full current syntax/token/binding consistency checks.
Native opaque-invalid paste needs an explicit Rust-internal rejected-draft batch
path, rather than blindly resubmitting its text through the new parser. The user
can explicitly resubmit it to adopt new syntax. Do not just accept arbitrary markers
or weaken the combined graph/selection-image checks. Add the old-invalid-dollar
WB2/native-copy case before claiming compatibility; return for review if this cannot
be implemented safely without a format revision.

## Alternatives considered

Historical-lexeme translation is easy but can silently revive a reused coordinate
after a stable target moves. Frozen source CellIds would make copy behave like move
and cannot serve cross-document copy. Translating in C# duplicates authoritative
semantics. Native-only clipboard data prevents basic external numeric tables;
TSV-only loses exact/provenance and source-origin distinctions. Unbounded enumeration,
tiling, incremental multi-commit paste and implicit text coercion exceed this slice.

## Verification / reconsideration trigger

Require independent transformation tables for all four axis modes; moved formulas/
targets, retired/blank targets, reversed/mixed-absolute SUM endpoints, overlap and
cross-document native copy. Check exact signed-zero/scale, UTF-8 spans, opaque invalid
drafts, translated bounds failures, shape/TSV rules, quota exhaustion, late admission/
cancellation rollback, S/T source captures, same-revision failures, source/destination
identity behavior, WB2 reopen and explicit unsupported-format handling.

Run actual Rust/C# application workflows on Windows, WSLg and native APFS/macOS,
including private clipboard plus external TSV, manual shortcuts, selection/focus,
late clipboard reads, normal zero-charge close and unchanged prior milestones.
Retain historical platform/filesystem failures. Revisit policy if missing bindings,
external-text expectations, identity/provenance needs or codecs require a semantic
exception. Stop for review before implementation and again after the bounded slice.
