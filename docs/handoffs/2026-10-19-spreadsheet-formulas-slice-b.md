# Spreadsheet Scalar Formula Parsing and Binding — Slice B

Date: 2026-10-19
Status: Implemented; ready for parent review.

## Scope and result

Accepted Slice A remains the sparse, single-sheet graph projection foundation.
This slice extends the existing Rust expression parser with same-sheet scalar A1
syntax and adds atomic text formula submission in pivot-workbook. ADR-0010 remains
Accepted with its compatibility reservations. No document, hosted storage, protocol,
C# or UI behavior changed. Stop here for review.

The representative workflow now uses raw text:

```text
A1 = 100
B1 = 25
C1 formula =A1+B1 -> 125
A1 = 200 -> C1 = 225
Move A1 to D1 -> C1 remains 225
New A1 = 999 -> C1 remains 225
Clear D1 -> C1 = 25 under the provisional blank policy
Retire D1, create new D1 = 500 -> original C1 remains broken
Resubmit C1 as =D1+B1 -> C1 = 525
Resubmit C1 as =A1+B1 -> C1 = 1024
```

Independent expected results are asserted directly in the tests. An earlier
calculation capture still returns 125 after the live workbook has changed.

## Implementation for review

- [Shared parser](../../crates/pivot-core/src/expression.rs): explicit
  parse_spreadsheet entry point, optional leading `=`, typed CellReference AST
  elements with original UTF-8 byte spans, case-insensitive column decoding,
  checked u32 address arithmetic. Same Pratt parser, precedence, decimal parsing,
  4,096-byte and 64-depth limits. Standalone parse/name behavior unchanged.
- [Workbook](../../crates/pivot-workbook/src/lib.rs): SetFormulaText atomic edit,
  iterative AST lowering, shared validated coordinate/identity resolver and
  existing trusted expression installation. No synthetic identifiers or evaluator.
- [Core transfer seam](../../crates/pivot-core/src/model/transfer.rs): narrow
  non-executable draft setter, validating live ownership, source length and UTF-8
  offset before mutation. Existing bound setter and stack/dependency checks reused.
- [Grammar and semantics](../architecture/spreadsheet-scalar-formulas.md): current
  API, submission rules, provenance, diagnostic and presentation limits.

Content::TextFormula stores original source independently of effective bindings.
Each bound reference stores its source span, submitted coordinate, stable cell and
portable graph node. Transaction base session/state/graph, zero-based edit ordinal
and the staged graph revision after demanded anchor allocation and before formula
installation identify resolution context. Edits see preceding operations in their
private candidate. There is no publicly installable prepared-binding object.

Moving a target never reparses its formulas. Explicit resubmission resolves again,
even for identical text, and records fresh provenance. Original text is explicitly
named original_source; it must not be presented as a current effective coordinate
rendering after a move. Full rendering is deferred; spans and stable targets provide
its required inputs.

Malformed syntax, invalid/unrepresentable literals and out-of-bounds reference
coordinates are editable non-executable drafts. Coordinate validation completes
before reference anchors are created. Invalid formulas never become zero inputs.
Workbook diagnostics preserve precise parser/reference offsets. Retired targets
produce stable missing-reference diagnostics plus the original reference location.
Cycles and structural errors still block the whole graph snapshot; isolation of
unrelated structural failures is deferred.

Core invalid-draft markers reuse an existing parse-failure category. Rich workbook
messages remain workbook metadata. No new transfer error codes or frozen storage
fixtures were introduced, and no workbook serialization exists. The trusted typed
Slice A edit remains an in-process facility; no bound-token injection RPC was added.

## Test evidence

Nine additional tests: seven [formula integration tests](../../crates/pivot-workbook/tests/formulas.rs),
one core transfer boundary test and one internal cost harness.

Coverage includes raw-text arithmetic, precedence and parentheses, unary signs,
lowercase/multi-letter addresses, exact literal scales 0/4/28, division errors,
malformed/overflow/zero/leading-zero/out-of-bounds addresses, Unicode error offsets,
unsupported syntax, never-materialized blanks, retirement and coordinate reuse,
explicit repair, cycles, snapshots, sequential batch binding, rollback, stale
sessions/states and source/token/cell quotas. Existing dependency-boundary,
model-only parsing, typed projection, file and daemon/REPL tests still run.

Full Windows CI passed: **209 Rust tests**, formatting, workspace Clippy with
warnings denied, .NET 10.0.402 Release with warnings as errors, all 10,099 decimal
vectors, Rust/C# protocol parity including 58 decimal layouts and frozen legacy
compatibility, plus actual named-pipe daemon/REPL checks.

- [Final Windows summary](../../artifacts/ci/20261019T092359828775Z/summary.json)
- [Rust test log](../../artifacts/ci/20261019T092359828775Z/rust-tests.log)
- [IPC results](../../artifacts/ci/20261019T092359828775Z/ipc/results.txt)

Ubuntu WSL2 checks passed: workspace formatting, Clippy, **210 Rust tests** and
actual Unix-socket daemon/REPL integration. The extra test is Unix-specific file
permission/alias coverage. CARGO_TARGET_DIR remains target/linux-wsl; file tests
use PIVOT_FILE_TEST_ROOT=/var/tmp/pivot-experimental-file-tests on native disk.

- [Linux Rust tests](../../artifacts/workbook-slice-b-linux-tests.log)
- [Linux IPC results](../../artifacts/workbook-slice-b-linux-ipc/results.txt)

Native macOS ARM64 full CI also passed: **210 Rust tests**, formatting, Clippy,
.NET 10.0.402 Release with zero warnings/errors, all 10,099 decimal vectors,
protocol/legacy compatibility and actual Unix-socket daemon/REPL checks. The separate
formula cost run passed; its test is already included in the workspace total.

- [Tracked macOS review](2026-10-19-macos-validation.md)
- [Original Mac report](../../artifacts/macos-validation-20261019T093243Z/macos-validation.md)
- [Mac CI summary](../../artifacts/macos-validation-20261019T093243Z/ci/20261019T093324130572Z/summary.json)
- [Mac IPC results](../../artifacts/macos-validation-20261019T093243Z/ci/20261019T093324130572Z/ipc/results.txt)

All 215 source files in the Mac SHA-256 manifest matched our working tree before
these documentation-only updates. The current SMB working tree was copied to local
APFS, including uncommitted/untracked files. Source, Cargo and .NET build outputs
remained local; reports were copied back to SMB. No product fixes or CI reruns were
required. The Mac used a newer nightly compiler than Windows/Linux, so the timing
samples are not controlled platform comparisons.

No new Linux .NET run was needed for this Rust-only change. GUI execution, Native
AOT and actual cross-user denial remain unverified. Git comparison
confirms Cargo manifests/lockfile, pivot-storage (including all 11 frozen storage
fixtures), pivot-protocol, pivot-host, apps and C# sources are unchanged.

## Small cost observations

[Windows observations](../../artifacts/workbook-slice-b-windows-costs.log),
[Linux observations](../../artifacts/workbook-slice-b-linux-costs.log),
[macOS observations](../../artifacts/macos-validation-20261019T093243Z/formula-costs.log).
Run `cargo test --locked -p pivot-workbook --lib report_text_formula_costs -- --nocapture`.

Windows debug-profile observations for `=A1+B1*2`:

| Live cells | Parse mean ns | Resolve mean ns | Formula edit us | Input edit us | Capture us | Selected evaluation us |
| --- | --- | --- | --- | --- | --- | --- |
| 17 | 1,631 | 2,715 | 149 | 42 | 83 | 21 |
| 129 | 1,610 | 2,721 | 262 | 171 | 371 | 25 |
| 513 | 1,597 | 2,700 | 668 | 526 | 1,651 | 58 |

Native macOS ARM64 debug-profile observations, using the same harness:

| Live cells | Parse mean ns | Resolve mean ns | Formula edit us | Input edit us | Capture us | Selected evaluation us |
| --- | --- | --- | --- | --- | --- | --- |
| 17 | 2,354 | 3,275 | 63 | 45 | 83 | 22 |
| 129 | 2,270 | 3,334 | 215 | 152 | 522 | 32 |
| 513 | 1,572 | 2,572 | 386 | 431 | 1,770 | 52 |

Parser and resolution numbers are means across 500 repetitions, not distributions.
Resolution measures actual AST lowering/coordinate validation and stable-ID token
resolution against existing anchors, excluding parser/staging, first-demand blank
allocation, installation and retained presentation metadata. The other columns
are one sample each. Formula/input edits include full staging and quota scans;
capture includes dependency/order rebuilding and position-map copying. Values are
checked against an independent expected output of 4 after changing A1 to 2.

Full-copy editing/capture remains the visible growth cost at these tiny sizes.
No optimized release benchmark, allocator profiling, latency target or spreadsheet
capacity claim is made. Logical quota estimates count reference metadata but remain
separate from allocator/RSS/core/snapshot memory.

## Deferred and next boundary

No `$` flags, copy/fill, sheet qualifiers, multiple sheets, names, ranges, SUM or
functions, comparisons, locale syntax, general text/error values, incremental
calculation, workbook files, RPC, C# IPC, Avalonia or hosted persistence.

The following priority is a runnable executable C# application/session IPC client,
subject to parent review. No client or further workbook infrastructure is started.
