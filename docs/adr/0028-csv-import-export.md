# ADR-0028: CSV import and export

Date: 2026-10-29.
Status: **Accepted for implementation, with parent and owner clarifications.**
Implementation review remains pending. The owner authorized proceeding after
settling leading zeros, quotation and the all-quoted import prompt.
[Implementation handoff and scoped validation](../handoffs/2026-10-29-csv-import-export.md).
Native Windows x64/Linux x64/macOS ARM64 headless CI and managed actual-window
suites passed on matching implementation inputs. No new Native AOT pass is claimed.

## Context

The next proposed spreadsheet milestone is CSV interchange. Native WB9 remains
the complete workbook representation. CSV carries a rectangular table of text;
it cannot preserve formulas, stable identities, multiple sheets, styles or types
without an additional contract. Rust on ROBPC reports
`1.101.0-nightly (92ffebece 2026-10-28)`; the owner reports all other hosts updated.

## Decision

Deliver one bounded desktop vertical slice with Rust-owned parsing, conversion,
admission and value serialization. Keep the engine independent of file pickers
and GUI types. CSV is separate from Open/Save and never becomes the document's
native path or clears its dirty state. Retain WB9 and CB4; extend the private
session protocol only as required by the new commands and negotiate its version.

### Import

Import into a new, unsaved workbook with one worksheet, starting at A1. Use the
existing dirty-document Save/Discard/Cancel flow before replacing the current
document. Read and validate the candidate completely before replacement; failed
imports leave the existing workbook, path, revision, selection and dirty state
unchanged (apart from a separately completed user-requested native Save).
No overwrite-at-selection, sheet merging or appended data in this slice.

Provide two explicit modes, defaulting to **Text**:

- Text: every nonempty field becomes a String, including leading `=`, numeric
  spellings and Boolean spellings. Preserve whitespace and Unicode verbatim.
- Numbers and Booleans: eligible fields recognize only exact invariant decimal spellings and
  case-insensitive ASCII `true`/`false`; retain other nonempty fields as String.
  Decimal grammar is `[+-]?[0-9]+(\.[0-9]+)?`; no exponent, separators, currency,
  percentages, dates or whitespace trimming. A numeric spelling must fit Decimal96
  exactly, with no rounding; otherwise report its row/column and reject the import.

Quoted fields remain exact Strings by default. In Numbers and Booleans mode,
offer an independent setting to allow type inference in quoted fields, available
for both mixed-quotation and all-quoted files. Text mode preserves every nonempty
field as a String. In all modes, quoted empty fields are empty Strings and
unquoted empty fields are Blank. Never execute CSV formulas.

Offer **Preserve leading-zero integer identifiers**, enabled by default. Any
eligible unsigned integer spelling with multiple digits beginning with zero stays
an exact String (`001234`, `0000`). Explicit signs and decimal points allow
numeric inference (`+001234`, `-001234`, `00.50`). Disabling preservation permits
the original decimal grammar on these integers. This setting applies equally to
quoted and unquoted fields wherever inference is enabled; it never overrides
String preservation of quoted fields. No identifier, date or locale inference.

When every field is quoted and Numbers and Booleans mode still preserves quoted
Strings, explain that some producers quote everything and offer **Keep as text**
(default), **Recognize numbers and Booleans**, or **Cancel**. The explicit choice
sets the same quoted-field policy for this import only, without changing the
leading-zero setting. Skip this prompt in Text mode or if the user already chose
quoted inference. Retain the parsed input while prompting; do not reread the path.

Accept strict UTF-8, optionally with one leading UTF-8 BOM; reject malformed input
and UTF-16 rather than replacing characters or guessing encodings. Use comma as
the only delimiter. Double quotes enclose fields and doubled quotes escape them.
Inside quoted fields preserve commas and CR/LF verbatim. Accept CRLF and LF record
endings outside quotes; reject lone CR there, unclosed quotes, quotes in unquoted
fields and characters after a closing quote except delimiter/record ending/EOF.
Allow the final record without a terminator. A terminal record ending does not
add another row. Preserve trailing empty fields. A blank record has one empty
field; zero-byte or BOM-only input has zero records and is rejected as empty.

Require equal field counts on every record, with no header inference or skipping.
Headers, when present, are ordinary first-row data. Before reading, show the
1,024 live-cell limit and the 1,024 by 1,024 coordinate bounds. Limit input to
4 MiB, including BOM, and logical fields to 65,536, including blanks. Enforce
these bounds during parsing before large allocations. Apply existing string and
workbook admission limits as well. At most 1,024 nonblank imported cells may be
created; blank fields consume rectangle/coordinate budgets but no graph cells.
Reject all excessive or malformed imports atomically, with an actionable diagnostic.
These are configurable interchange admission policies, not permanent CSV-format
constraints or capacity increases. Distinguish CSV bytes/fields, worksheet bounds,
workbook live-cell limits and runtime memory/allocation admission in diagnostics.

### Export

Export a user-selected rectangle on the active sheet, or its content bounding
rectangle when the user chooses the whole-sheet option. Ignore formatting-only
positions when finding that bound. Disable whole-sheet export for an empty sheet.
Use one immutable calculation capture for the entire export, retaining sheet,
revision and bounds even if later edits occur. Fetch values in bounded batches;
do not assume a viewport-sized or single-response export.

Serialize calculated values, never formula source or formatted display text:
Decimal96 uses exact invariant plain decimal notation, Boolean uses `TRUE`/`FALSE`,
String retains its text, and Blank is empty. Reject an export containing a cell
error, identifying the first address in row-major order; do not invent error text
or silently drop failures. No implicit recalculation from a second snapshot.

Write UTF-8 without BOM, comma separators and CRLF record endings, including a
final terminator. Always quote String values, doubling embedded quotes. Emit
Decimal96 and Boolean without quotes, empty String as `""`, Blank as unquoted empty.
Preserve all fields in every selected row, including trailing blanks. Enforce
65,536 logical fields and a 4 MiB encoded-output limit before replacing a file.
Stage output in a sibling temporary file and use the existing platform-appropriate
safe replacement pattern; failure must preserve an existing destination. Clean
owned temporary files on failure. Native workbook state remains unchanged.

Show a concise export notice: CSV omits formulas, styles and other sheets. Pivot
uses quotation as a type hint; other consumers need not respect it.
Strings beginning with formula-like characters remain literal CSV text;
other spreadsheet programs may interpret them as formulas. Do not silently prefix
apostrophes or promise safe interpretation by another application. Text-mode
reimport preserves these strings; automatic type mode can change string types.
CSV is not a lossless native-workbook round trip.

## Alternatives considered

Importing into the selection is useful but adds overwrite, style-retention and
capacity interactions. A new workbook keeps this milestone bounded. Automatic
locale/delimiter/type inference makes results dependent on environment and input
samples; explicit modes and one dialect are easier to explain and reproduce.
Using display text on export loses exact values; exporting formulas would require
a different interchange contract. XLSX, delimiter options, preview grids, column
schemas, dates, capacity increases and history integration are deferred.

## Consequences

Desktop work includes native import/export pickers, import mode, limit/error
messages and explicit selection/whole-sheet export choice. Rust owns the
transaction boundary and authoritative conversions; C# does not grow another
parser or evaluator. Reuse existing admission, snapshot and document lifecycle
facilities. Inspect the available CSV dependencies before choosing a parser;
avoid adding a dependency solely for a small encoder. File I/O and payload sizes
must fit existing session framing limits, with chunking or tighter admission if
necessary; do not raise shared protocol limits implicitly.

## Verification / reconsideration trigger

Implement one complete slice with focused Rust
parser/conversion/admission tests, independent CSV fixtures, real .NET session
checks and desktop lifecycle coverage. Exercise quoted multiline Unicode,
escaped quotes, trailing blanks, malformed UTF-8/syntax, exact decimal boundaries,
formula-like strings, ragged rows, each budget boundary and atomic rollback.
Verify snapshot-consistent calculated exports, dependent formulas, cell errors,
destination preservation on write failure and native Save/reopen after import.
Run native Windows/Linux/macOS headless checks and focused actual-window tests
for the new controls. AOT validation is warranted if new serialization or picker
code changes its compatibility. Record what actually ran in one milestone handoff.

Stored String admission allows CR, LF and tabs for CSV payloads; direct editor
entry keeps its existing single-line checks. Native persistence, native clipboard
and Fill preserve these values using the existing representations. No new value
type or encoding is introduced. Other control characters remain rejected.

Implementation and validation evidence belong in the single feature handoff;
design acceptance does not imply implementation acceptance. Reconsider limits only
from a concrete user requirement and measured admission costs. ADR-0023 remains
parked at G3.
