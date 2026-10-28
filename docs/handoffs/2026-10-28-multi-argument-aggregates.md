# Multi-argument spreadsheet aggregates

2026-10-28. **Accepted and closed by parent review.** Three-platform headless
regressions passed; no additional qualification round is required.

The review accepts the intentional WB6-only cut while recommending that workbook
compatibility commitments be revisited before real users depend on saved documents.
It recommends bounded multiple-worksheet design next; no design/implementation or
migration assignment follows automatically. ADR-0023 remains parked at G3.

SUM, MIN, MAX, AVERAGE and COUNT now accept scalar expressions, cell references,
multiple rectangles and nested calls. Examples:

```text
=SUM(A1:A10,C1:C10,5)
=MIN(A1,2*3,MAX(B1:B4,7))
=AVERAGE(AVERAGE(A1:A2),9)
```

The existing desktop editor/formula bar submits these formulas through the same
Rust session. Copy/fill translates every reference, including independent dollar
flags. Save/reopen preserves stable bindings; a moved scalar target stays bound
while a range follows its current occupants. No synthetic graph nodes or frontend
evaluator is introduced.

Arguments visit left to right; ranges visit row-major, with repeated/overlapping
members counted per occurrence. Existing blank/text/error policies are retained,
including direct blank/text exclusions for non-SUM reducers and strict SUM text
errors. Successful scalar expressions count as numbers. AVERAGE accumulates exactly
across the call and rounds once; a nested function result is one scalar argument.
Missing arguments reject; existing source/depth/work/memory limits remain in place.
See [aggregate semantics](../architecture/spreadsheet-aggregates.md).

Native documents are **WB6-only**: persisted postfix call controls require a format
cut. WB1-WB5 explicitly reject; no migration reader is added. Failed Open still
preserves the live document, saved checkpoint/path and dirty state. Clipboard and
application-protocol representations are unchanged. Their tests and active saved-file
verifiers now expect the current workbook header.

## Validation

All 13 existing headless CI steps passed on each native target:

| Target | Final evidence |
| --- | --- |
| Windows x64 / NTFS | [Successful recovery CI](../../artifacts/aggregate-arguments-final/windows-recovery/recovery-ci/20261028T122917930389Z/summary.json) |
| Ubuntu x64 / ext4 | [Headless CI](../../artifacts/validation-matrix/20261028T122432Z-aa285fcb/linux-x64/ci/20261028T122436061665Z/summary.json) |
| Mac ARM64 / APFS | [Headless CI](../../artifacts/validation-matrix/20261028T122432Z-aa285fcb/osx-arm64/ci/20261028T122440026995Z/summary.json) |

This includes workspace Rust tests/lint/format, .NET build, execution-safety checks,
independent aggregate/decimal/protocol parity, real local IPC and application-session
tests. An additional **240 all-feature tests** passed for the three changed Rust
crates; workspace all-target/all-feature Clippy passed with warnings denied.
[Final all-feature log](../../artifacts/aggregate-arguments-all-feature-final.log).
[SHA-256 verification](../../artifacts/aggregate-arguments-final/tested-inputs.json)
confirms all 28 changed implementation inputs match the three tested snapshots;
later documentation reconciliation does not imply an unchanged complete shared tree.

Coverage includes parser/typed-token rejection, mixed/nested reductions, one-rounding mean,
overflow/error order, overlapping arguments, capture immutability/invalidation,
copy/fill, stable-target Save/reopen and obsolete-format rejection. Real .NET session
tests exercise all five functions and repeat their assertions after Save/reopen.

Initial regressions exposed old tests treating `SUM(A1)` as invalid; those now use
genuinely malformed `SUM(A1,)` lists. Review also found and fixed confusion between
a bare rectangle and a nested single-range function result; dedicated Rust and
session regressions cover `MAX(SUM(...),...)` and `COUNT(COUNT(...),...)`.

The first final-source Windows attempt exhausted its disk while building; its
[failed report](../../artifacts/validation-matrix/20261028T122432Z-aa285fcb/win-x64/result.json)
is preserved. Only this feature's earlier disposable target/cargo caches were
removed, preserving source and reports, freeing 8,783,249,408 bytes before recovery.
The successful rerun used the same final frozen source, not a settings/code bypass.
Linux/Mac did not rerun for that disk-only recovery. No user VM setup task remains.

No GUI or AOT suite is assigned for unchanged interaction/transport code. This is
feature regression coverage, not another platform-qualification milestone. Existing
optional-feature adapters carry the new typed tokens so they continue to compile
and round-trip; no history research, instrumentation or scaling run resumed.
ADR-0023 remains parked at G3, with its failed/stopped pilot evidence preserved.
