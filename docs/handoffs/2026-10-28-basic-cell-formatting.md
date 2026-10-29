# Basic cell formatting — ADR-0026 implementation

Date: 2026-10-28. **Implementation complete; ready for parent review.**
ADR-0023 remains parked at G3. This is the single feature handoff.

## Delivered

Formatting is sparse document-owned metadata keyed by worksheet/axis position,
independent of content identity and the canonical calculation graph. Styled blanks
allocate no cells, nodes or computational selections. Style edits change workbook
revision/dirtiness without changing graph revisions, exact values or dependencies.
Persistent roots, candidate work and presentation responses use the existing allocation
domain. The initial 4,096-position/command policy is explicit in `Limits.styles`,
independent of computational limits and file/identifier widths.

The Format pane provides General, fixed decimals, grouping, currency ($/€/£),
percentage, 0–28 display places, all seven rounding modes, bold/italic, alignment and
basic foreground/background palettes. Mixed selections patch only touched fields.
Apply/Reset first commit an active draft and stop on failure. Reset retains content;
editors retain exact input/effective formula source.

Rust renders numeric presentation separately from authoritative typed values using
bounded coefficient/digit operations, without floats or Decimal96 multiplication
for percentages. Independent integer references cover every rounding mode; maximum
percentages, carries, trailing zeros and negative displayed zero are tested. Numeric
formats never coerce String/Boolean values or change downstream arithmetic.

Clear retains position styles. Move transfers/resets complete styles, including a
default source overwriting destination styles. Native rectangular Paste and Fill
publish content/styles atomically, including graph-free styled blanks. External TSV
retains destination styles and exports reversible raw content. Viewport frames reuse
the existing document/sheet/revision/view guards.

WB9 persists sparse styles; private v7 adds masked style commands, independent style
frames and numeric presentation; CB4 adds complete per-slot styles and distinct OS
clipboard identifiers. Older experimental versions are rejected, without migration
or malformed-native fallback. Open validates ownership, uniqueness, canonical
records and resource bounds before installation. Unsupported styled operations are
explicitly refused by parked history/proof adapters; no history work was resumed.

## Validation

| Gate | Result |
| --- | --- |
| Workbook/document all-feature tests | 161 passed, including optional-adapter regressions |
| Rust formatting and strict workspace all-target/all-feature Clippy | Passed |
| C# ↔ Rust session, captures, clipboard and Save/reopen | Passed |
| Native Windows x64/Linux x64/macOS ARM64 headless CI | 13/13 each |
| Managed actual-window automation, all three targets | Four successful application reports each; zero charges/no owned survivors |
| Native macOS ARM64 publish/execution | Zero publish warnings; four successful native application reports; qualified post-check below |
| Current-format verifier regression tests | 3 passed: WB9, obsolete versions, corrupt/truncated envelopes |

Headless evidence: `artifacts/formatting-local/final-headless/`; local regression
logs: `artifacts/formatting-rust-final.log`, `artifacts/formatting-clippy-final.log`
and `artifacts/formatting-session-final3/`.
Managed reports: `artifacts/validation-matrix/20261029T013803Z-2bcb6387/` (Windows)
and `20261029T014206Z-b7b8cfeb/` (Linux/Mac).

**AOT qualification:** `artifacts/validation-matrix/20261029T014206Z-9bd82870/`
retains the original failed gate. Publishing and all four actual-window native runs
succeeded; the final Python post-check incorrectly expected WB8. Corrected it to
validate WB9, payload length and checksum, and ran the three focused verifier tests.
A read-only audit of the existing Mac artifacts passed: valid WB9, all 10 published
file hashes unchanged, four successful native reports, zero charges, no reported
owned PID or published-path survivors. Evidence:
`artifacts/formatting-local/aot-offline-review.json`. No GUI was rerun and no original
summary was relabeled. This is post-run evidence verification, not a fresh full gate.

Source qualification: headless originally matched 363 implementation inputs.
Subsequent changes are confined to two desktop automation files and the AOT verifier,
plus its new regression-test file. Successful managed/AOT execution used the corrected
desktop tests; only the verifier/test changed afterward. All product implementation
inputs are unchanged. Exact differences and current 364-input hashes are recorded in
`artifacts/formatting-local/final-source-reconciliation.json`.

Earlier failures remain preserved: stale WB8 desktop assertion (`20261029T003207Z-319e761e`),
then clipboard checks racing the Format pane transition (`20261029T003509Z-1f002b56`,
`20261029T003920Z-34195768`). The latter was initially misdiagnosed as occurring after
reopen. Automation now waits for Expanded/Collapsed completion and a current frame,
requires copy publication/paste completion, and retains diagnostics/failure images.
Assertions were retained; product clipboard guards were not weakened.

## Limits

No custom/locale formats, financial currency value type, Date/Time/Integer, ROUND,
conditional formatting, borders, wrapping, structural edits or history integration.
Style-only Move is deferred; Copy/Paste handles those regions. Before real users
rely on saved files, the experimental current-format-only policy needs a deliberate
compatibility commitment. Stop here for parent implementation review.
