# Calculation rounding — ADR-0027 implementation

Date: 2026-10-29. **Accepted and closed by parent implementation review.**
Parent accepts the supplied evidence and separate Ubuntu frozen-source recovery;
the original failed attempt is not reclassified. No additional GUI/AOT, migration,
platform qualification or reconciliation round is required. Retain this single
handoff and the adopted artifact-retention policy. ADR-0023 remains parked at G3.

`ROUND(number,digits)` rounds nearest with ties away from zero, `ROUNDUP` away from
zero and `ROUNDDOWN` toward zero. Negative digits round tens, hundreds and larger
places: `ROUND(125,-1)=130`, `ROUNDUP(-2.1,0)=-3`, `ROUNDDOWN(-2.9,0)=-2`.
Both arguments are required scalar expressions. Digits must be integer-valued
Decimal96; fractional digits are InvalidArgument, Boolean/String operands are
TypeMismatch, and Blank retains the existing numeric zero semantics. Results are
normalized Decimal96 values, independent of display formatting.

The canonical Rust graph uses new typed instruction codes 15–17. One bounded
coefficient division/remainder decision avoids double rounding, binary floats and
bounded Decimal96 intermediate scaling. Huge digits are classified before machine
conversion; positive digits >=28 preserve the numeric value, digits <=-30 yield
zero for ROUND/ROUNDDOWN and Overflow for nonzero ROUNDUP. At -29, ROUND can overflow
for sufficiently large inputs; zero stays zero. Final quantum multiplication checks
96-bit representability. Runtime arithmetic failures retain the existing arithmetic
fault category/origin; arguments validate left to right and inactive IF/AND/OR branches
remain undemanded. The public Rust signed-place helper supports all seven modes.

Cross-sheet references, invalidation, captures, Copy/Paste/Fill and Save/reopen use
the existing implementations. No C# evaluator, control, message shape, value kind or
clipboard slot change was needed. The additive instruction vocabulary fits existing
encodings: WB9/private v7/CB4 remain current; existing WB9 documents stay readable.
New functions require a supporting engine; unknown names/opcodes are rejected.
No migration, new hosted-v1 commitment or history work was introduced.

| Validation | Result |
| --- | --- |
| Types/core/workbook/document all-feature tests | 268 passed; final extra boundary/nested-aggregate assertions passed in focused reruns |
| Rust formatting and strict workspace all-target/all-feature Clippy | Passed |
| Real .NET ↔ Rust session, errors, captures, clipboard, formatting and WB9 reopen | Passed |
| Windows x64/Linux x64/macOS ARM64 native headless CI | 13/13 each; all 368 implementation inputs match |

Independent integer references cover 63,021 combinations across signs, precision
positions and all seven modes. Tests include exact ties, huge digits, smallest
fractions, maximum-coefficient overflow, arity/types, lazy overflow suppression,
left-to-right failure order, nested aggregates, cross-sheet changes, immutable old
captures, ordinary Decimal capture evaluation, copy/fill translation and display
independence. Local evidence: `artifacts/rounding-rust.log`,
`artifacts/rounding-clippy-final.log`, `artifacts/rounding-session/`.
Native Windows/Mac evidence: `artifacts/validation-matrix/20261029T123045Z-af5642e1/`.
Ubuntu initially exhausted its native disk while linking test binaries and could not
finish writing/exporting its original receipt. Reclaimed 12.17 GB from three obsolete
October 25–26 Cargo caches, preserving source, reports and published images. The
recovery reused the exact frozen source and passed all 13 CI steps. Separate evidence:
`artifacts/rounding-local/linux-x64-recovery/`; the original failed setup outcome is
not reclassified. Disk/cleanup receipts are `artifacts/rounding-linux-disk-before.json`
and `artifacts/rounding-linux-cache-cleanup.json`. Byte-hash reconciliation for all
three targets/current source: `artifacts/rounding-local/verified.json` (368 matches).


The initial local test fixture incorrectly captured a model after submitting an
invalid-arity draft; corrected it to assert parser rejection directly while keeping
runtime type/precision/error assertions. Product assertions were retained.

Interaction, clipboard transport and .NET AOT metadata are unchanged. No attended
GUI/AOT rerun is assigned for this Rust scalar-function slice. Earlier accepted
platform GUI/AOT evidence remains historical, not an execution claim for these new
Rust functions. No ROUND mode parameter, conversion function, Integer/date support,
formatting change or other spreadsheet feature was added. The milestone is closed;
further feature work awaits explicit owner direction after the planned restart.
