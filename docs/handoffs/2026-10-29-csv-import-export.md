# CSV import/export — ADR-0028 implementation

Date: 2026-10-29. **Accepted and closed by parent implementation review.**
Native headless and managed desktop validation passed on Windows x64, Linux x64
and macOS ARM64. Parent accepts this evidence, the export installation contract
clarification below and the absence of a fresh Native AOT run. No additional
GUI/AOT, platform qualification or acceptance packet is required. Retain this
single handoff and the original evidence under the accepted retention policy.

The desktop CSV menu imports into a new unsaved Sheet1 workbook and exports either
the selection or active sheet content. Native WB9 Open/Save remains separate.
Rust owns strict UTF-8/comma parsing, quotation provenance, conversion, candidate
admission, captured-value serialization and local file installation. No dependencies
were added. WB9 and CB4 remain current; private v7 gains additive CSV operations
37–41 and fields, with no hosted-v1 changes or message-size increase. CSV bytes
stay in Rust; the client exchanges paths and single-use tickets. Cancelled prepared
imports release their retained table, including client cancellation after dispatch.

Text mode is the default. Numbers and Booleans mode has two independent settings:
preserve quoted fields as Strings by default, and preserve leading-zero integer
identifiers by default. Unquoted `001234` stays an exact String; `+001234`,
`-001234` and `00.50` are eligible for exact Decimal96 conversion. Allowing quoted
inference applies the same rules inside quotes, in mixed or all-quoted files.
Disabling leading-zero preservation permits numeric conversion wherever inference
is enabled. No field is submitted as a formula.

When every field is quoted and typed mode still preserves quoted Strings, a real
dialog offers Keep as text (default), Recognize numbers and Booleans, or Cancel.
The choice applies only to that import and does not change the leading-zero
setting. Text mode and an explicit quoted-inference choice skip the prompt. The
parsed table is retained across the dialog; later source-file changes cannot
change the data the user reviewed. Quoted empty fields always become empty Strings;
unquoted empty fields remain graph-free Blank.

Exports use one immutable capture and batches of at most 128 positions. Decimal96
uses exact plain notation; Boolean uses TRUE/FALSE; all Strings are quoted and
embedded quotes doubled; Blank is empty. Output is UTF-8 without BOM, with CRLF
records and a final terminator. Evaluation errors report the first address in
row-major order. Formatting-only positions and reference-only anchors do not extend
whole-sheet bounds; that choice is disabled for an empty sheet. Export cannot
overwrite the open native workbook path and does not acknowledge a native save.

Input/output admission is 4 MiB and 65,536 logical fields; the existing 1,024 live
cells and 1,024 × 1,024 worksheet bounds remain. Diagnostics separate interchange
quotas, coordinates, String limits, live cells and runtime/allocation admission.
Candidate construction can span bounded edit batches but never replaces the live
document until the whole candidate succeeds. Stored Strings now admit CR, LF and
tabs for multiline CSV; single-line editor admission and other control-character
rejections remain. WB9, native clipboard and Fill carry stored String values.

| Validation | Result |
| --- | --- |
| Changed-crate Rust all-feature tests | 171 passed, including six CSV tests |
| Strict workspace all-target/all-feature Clippy | Passed |
| .NET desktop and session builds with warnings as errors | Passed |
| Real .NET ↔ Rust session regression | Passed, including CSV lifecycle, 1,024 cells and zero remaining charge |
| Native Windows x64 headless CI | Passed 13/13 |
| Native Linux x64 headless CI | Passed 13/13 after disk cleanup; original failure preserved |
| Native macOS ARM64 headless CI | Passed 13/13 |
| Implementation input reconciliation | All 375 non-Markdown inputs match on all three native targets, for both headless and attended desktop source |
| Actual-window CSV interaction checks | Passed on Windows x64/Linux x64/macOS ARM64; four application reports per target, zero remaining charges and no owned survivors |
| New Native AOT publish/execution | Not run; existing pickers and static protobuf serialization are reused |

Regression coverage includes leading-zero and quotation combinations, signed and
decimal-point inference, Text mode, empty String versus Blank, multiline Unicode,
escaped quotes, terminal records, ragged rows, malformed UTF-8/syntax, Decimal96
boundaries, quota errors, multi-batch atomic rollback, retained input, old captures,
calculated exports, destination preservation on evaluation/quota failure, native
save/reopen and multiline native clipboard. Existing editor/formula/formatting and
clipboard suites also passed. Desktop automation exercises the actual option
controls, all three all-quoted dialog choices, mixed quotation, prompt suppression,
grid types, typed export bytes and native lifecycle. Native file pickers remain
manual interaction boundaries, as in the existing desktop suites.

Evidence is under `artifacts/csv-*`. Native Windows/Mac receipts are in
`artifacts/validation-matrix/20261029T152339Z-714787a0/`; Linux recovery receipts are
in `artifacts/validation-matrix/20261029T152453Z-373a6f37/`. The original
`20261029T151528Z-a2369238` attempt retains Ubuntu's disk-full failure and the
controller's Windows/Mac sandbox temporary-directory setup failures. Four verified
obsolete Ubuntu Cargo caches were removed under the accepted retention policy,
reclaiming 15.95 GB without removing source or reports. Inventory/receipt:
`artifacts/csv-linux-cache-cleanup.json`. Local sandbox file-access failures remain
in `artifacts/csv-rust.log`; the unrestricted all-feature pass is
`artifacts/csv-rust-native.log`. Source reconciliation:
`artifacts/csv-source-audit.json`, `artifacts/csv-desktop-source-audit.json` and the
per-target input manifests.

## Completed desktop validation

The owner completed all three attended managed runs from their graphical sessions,
following the existing [desktop test policy](../testing.md). Collected results:
`artifacts/validation-matrix/20261029T153120Z-78e20b73/`. Each target's main
`desktop-ui/results.json` explicitly reports CSV success; the clipboard-save,
discard and clean peer reports also passed. Native backends were Win32, X11 and
Avalonia.Native respectively. All twelve application reports show zero remaining
runtime charge, and each target's cleanup receipt reports no surviving owned PIDs.

The retained preparation record
`artifacts/validation-matrix/20261029T152505Z-dc85cae9/` still correctly reports
NeedsGraphicalSession; it is not relabeled as execution evidence. Actual attended
execution used `20261029T152505Z-dc85cae9-attended`. Its 375 implementation inputs
match the current tree and the headless-qualified inputs on all three targets.
No application change or additional run was required after collection.

CSV does not preserve formulas, styles or multiple sheets. Other applications may
interpret quoted values and formula-like text differently. File export reuses the
established native safe-install broker: failures before replacement preserve the
destination; errors after an attempted atomic replacement retain the broker's
explicit uncertain-installation outcome. Absolute rollback after a post-install
filesystem failure is not claimed. Parent accepts this as a clarification of the
ADR's earlier blanket preservation wording, consistent with native Save. An
uncertain outcome must not be presented as success, rollback or an unchanged
destination. No file-broker rewrite is required.

No capacity increase, XLSX, structural editing or history work was added.
ADR-0023 remains parked at G3. Parent recommends production workbook capacity and
identity-allocation scalability as the next product priority, beginning with a
bounded design for live cells, identities, import performance and memory. This
recommendation does not reopen history work or authorize a capacity increase.
