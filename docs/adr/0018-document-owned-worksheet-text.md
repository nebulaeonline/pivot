# ADR-0018: Document-owned worksheet text

Status: **Proposed — semantic review required before implementation**.
Date: 2026-10-20.
[Accepted preceding milestone and direction](../handoffs/2026-10-20-native-aot-review.md).

## Context

Pivot can display exact decimals, numeric formulas and rejected formula drafts,
with stable CellIds, atomic edits, typed rectangular clipboard and WB3 documents.
It cannot yet store ordinary invoice labels or report headings. This milestone
adds bounded document-owned text, without defining computational strings or a
structured-record value system.

Existing `Content::TextFormula` means source-bearing numeric formula, not literal
text. Existing text editors do not imply that arbitrary text is accepted as content.
The desktop currently trims input for classification and rejects nondecimal text;
external TSV likewise supports only numbers, formulas and blanks.

## Proposed semantics

### Content, ownership and identity

Add a distinct literal-text content variant and explicit typed SetText edit in the
canonical Rust workbook. Preserve the submitted UTF-8 payload exactly: no trimming,
Unicode normalization, case conversion, numeric parsing or reference resolution
once classified as text. Text belongs to immutable workbook/document generations,
not to a frontend cache, graph-produced string or external data source.

Text occupies a normal stable cell identity. Decimal/formula/text replacement keeps
the existing occupant identity under ordinary Set semantics. Clear, Move, Retire,
blank-anchor reuse and scalar references retain their established identity policies.
Moving text moves its identity; copying creates destination content using existing
paste rules. Text edits participate in expected-revision checks, dirty checkpoints,
atomic batches, memory accounting and snapshot isolation. Old captures retain the
old content and numeric behavior while the working workbook changes.

### Entry and editing

Use one Rust-owned entry classifier for ordinary desktop entry and external TSV
fields, with explicit typed operations available at the application boundary.
C# must not independently decide whether a decimal can be represented exactly.
Keep native typed paste separate from entry classification.

Classify an unescaped submitted field as follows:

1. Exactly empty source clears the cell.
2. A leading apostrophe forces literal text: remove exactly that first apostrophe
   and store the rest unchanged. A lone apostrophe clears rather than creating a
   second kind of empty cell. Two leading apostrophes store one literal apostrophe.
3. Otherwise, if the existing whitespace-trimmed classification view begins with
   `=`, submit the original source as a numeric formula. Invalid formula syntax is
   still a preserved editable formula draft, never silently converted to text.
4. If the trimmed field matches the existing fixed-point decimal input grammar,
   require exact Decimal96 representability, preserving scale and signed zero.
   Overflow/excess scale remains a correctable numeric rejection; no rounding or
   fallback to text. Decimal-shaped means optional sign, ASCII digits and optional
   decimal point, with at least one digit, as accepted by the existing numeric path.
5. All other nonempty source becomes literal text, preserving the original bytes.
   Whitespace-only source is text. Scientific notation, dates, currencies and boolean
   words are text in this slice; no new number/date/locale coercions are introduced.

| Submitted entry | Stored content |
| --- | --- |
| `Invoice total` | Text `Invoice total` |
| `  Invoice  ` | Text including both outer spaces |
| `001.20` | Exact decimal under existing rules |
| `'001.20` | Text `001.20` |
| `'=A1` | Text `=A1` |
| `=A1` | Existing numeric formula |
| Decimal-shaped overflow | Rejected numeric draft |
| `1e3` | Text `1e3` |

For existing text, F2/formula-bar editing reconstructs a leading apostrophe escape
before the exact payload. The escape is editor syntax, not stored/displayed text.
It prevents an unchanged text label such as `123` or `=A1` being reclassified on
resubmission. No-op editing preserves identity/content; typing to replace uses the
new entry classifier. Grid text is rendered literally without the escape, using
single-cell clipping; no adjacent-cell spill, wrapping, rich text or layout changes.
Plain arrows/Enter/Tab commit and move as today; Escape cancels. Save and dirty-close
continue to commit or preserve drafts under existing acceptance rules.

### Numeric references and SUM

Literal text is valid worksheet content and is not itself a formula error. Display
reads return its content without a numeric value; ordinary text cells are excluded
from viewport numeric-output requests and formula-error counts.

A numeric formula consuming a text cell returns an explicit nonnumeric-input/type
error. Never coerce text to zero, parse numeric-looking text or rebind a reference.
Downstream formulas receive the existing dependency-failure behavior. Direct scalar
reference to text is also an error, not a string-valued formula result.

SUM encountering a text occupant likewise returns a nonnumeric-input error; it does
not skip text. This is a deliberate proposed policy requiring review, not a claim
of Excel compatibility. Existing blank-as-zero behavior, fixed-rectangle membership,
row-major checked decimal addition, error ordering and quotas remain unchanged for
numeric/blank workbooks. Replacing text with a decimal or clearing it invalidates
consumers normally and removes the type error when appropriate.

The calculation graph must not contain the text payload. Preserve the stable numeric
node binding with a narrowly scoped nonnumeric-input definition/error marker, owned
by the workbook adapter. It is an evaluation failure marker, not a string runtime
value. It must propagate through both scalar and range evaluation; a zero placeholder,
synthetic parse error or UI-only error overlay is unacceptable. Core export/import
must preserve this marker wherever canonical numeric nodes are persisted. Hosted
v1 must not start accepting strings or silently serialize the marker as a decimal;
keep its existing schema and reject unsupported exports explicitly.

This marker is the only proposed core semantic extension. If it cannot fit current
snapshot/evaluation/storage boundaries without a broader value-system change, stop
and report the smallest revised design before implementation proceeds.

### Clipboard and bounded text

Native clipboard adds a tagged text slot containing the exact payload, with no
formula translation or entry parsing. Mixed text/number/formula/blank rectangles
remain one atomic paste, with the existing 256-slot bound, exact-shape rules,
overlap snapshots, selection guards and all-or-nothing admission. Existing formula
copy, dollar-axis translation, SUM geometry and rejected-draft rules are unchanged.

For plain TSV, retain the existing unquoted tab and LF/CRLF table grammar. Empty
fields mean blank; nonempty fields use the same Rust entry classifier. Export text
with a leading apostrophe escape, so numeric-looking and formula-looking labels
round-trip as text through Pivot TSV. This may display an apostrophe in other
applications: exact native copy is preferred, and no universal spreadsheet import
compatibility is claimed. Numbers/formulas keep existing TSV rendering behavior.

For this bounded slice, accept single-line text only: reject TAB, CR, LF, NUL and
other C0/C1 control characters in stored text, atomically. Do not strip or substitute
them. Ordinary Unicode letters, combining marks, emoji and spaces are allowed.
This avoids introducing quoted TSV and multiline editor semantics together with
text. Multiline/quoted clipboard interoperability is deferred explicitly.

Limit each text payload and submitted editor source to the existing 4,096-byte
source ceiling, measured in UTF-8, not UTF-16 code units. Thus an escape also counts
toward submitted-source admission. Existing batch/workbook/domain memory limits,
128-KiB combined clipboard source-work bound and 256-KiB combined native/TSV output
bound remain; text bytes and escape bytes count before allocation/publication.
A text payload valid individually may still fail a larger copy/paste admission.
Failed entry retains a correctable draft; failed paste leaves all cells unchanged.

### Persistence and private application protocol

Propose an explicit PIVOTWB4 format cut for text content and its nonnumeric marker.
Reject WB1/WB2/WB3 with an unsupported-version diagnostic before replacing the live
document. No migration reader or heuristic legacy decode in this milestone. WB4
reopen must preserve content tags, exact text bytes, stable bindings, checkpoints,
rejected drafts and snapshots; verify document/graph marker consistency during import.
This compatibility cost needs explicit review before implementation.

Use a new private application-session version and new native clipboard envelope/
OS format version for the new tagged content (proposed PIVOTCB2 and cell-block-v2).
Both sides reject unsupported versions; do not reuse a v1 native tag with changed
semantics. When a supported native format is present but malformed, fail without
TSV fallback. An old-only Pivot native format is unsupported, not an invitation to
reinterpret its text representation. Keep the accepted frozen hosted v1 unchanged.

Application edits and reads distinguish text from number, formula and blank using
explicit oneof arms. User-entry classification is a dedicated Rust request/edit,
not an overloaded text-content arm. Numeric-result errors remain separate from
content, so a label is never rendered as an error solely because it lacks a number.
No anonymous/reflection-dependent frontend serializers are introduced.

## Implementation and validation plan after semantic approval

1. Add canonical text content, bounded admission and the nonnumeric evaluation marker;
   test snapshot replacement, scalar/range propagation and stable identity transitions.
2. Extend WB4 codec/import and the private application protocol; verify exact content
   and graph consistency, malformed input rollback and explicit old-format rejection.
3. Add native text clipboard and TSV entry/escape handling with atomic mixed rectangles.
4. Connect in-place/formula-bar editing and literal grid rendering to Rust classification.
5. Run ordinary CI plus actual desktop workflows and the separate Native AOT gate on
   Windows, Linux and native Mac/APFS, preserving shared clipboard serialization.

Require independent cases for spaces/Unicode/emoji, apostrophes, numeric/formula-looking
text, overprecision/overflow, empty/whitespace, control rejection and UTF-8 boundaries.
Test number-to-text-to-number/blank transitions, text in SUM and downstream errors,
old immutable captures, move versus copy, mixed rectangle overlap/blanks, stale revisions,
quota rollback, save/reopen and active-draft dirty-close. Retain current exact-decimal,
formula-copy and rejected-draft regression coverage. Automate routine acceptance;
no new recurring manual gate. Record AOT warnings and zero-charge shutdown on all hosts.

## Scope and review stop

No computational strings, string formulas/functions, implicit coercions, multiline
cells, formatting, Undo/Redo, row-height changes, database connectors, structured
results/projections or permanent `#` grammar. No capacity increase or installer work.

Review must approve or revise numeric-consumer/SUM type errors, apostrophe/entry
classification, single-line bounds, escaped TSV interoperability and the WB4/private
version cuts. This ADR proposes these policies; no implementation is authorized by
its proposed status. Stop here for semantic review.
