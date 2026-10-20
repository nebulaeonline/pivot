# Native workbook persistence and application lifecycle — Slice B

Status: implemented for review; Windows, isolated Linux and native macOS/APFS
CI/application/GUI checks pass. The parent review received 2026-10-20 accepts
ADR-0015 and Slice B; known validation limitations remain explicit.
ADR-0014/Slice A is accepted. [ADR-0015](../adr/0015-native-workbook-documents.md)
records the native file format, restoration and lifecycle decisions for review.

## User workflow

Run `python scripts/run_desktop.py`. Use New, Open, Save, Save As and Close workbook
above the worksheet. File selection uses native Avalonia storage dialogs. The title
and footer show the document name and unsaved status. Replacing/closing dirty work
prompts Save/Discard/Cancel. Saved files reopen in a new child process with stable
cell/reference identities, original formula source, exact numeric input and drafts.
Editing continues through Rust, including existing move/reuse/clear semantics.

## Implementation

The storage-independent workbook exposes owned logical transfer; the new
`pivot-workbook-document` crate handles a bounded versioned workbook envelope and
cross-validation, reusing portable graph CBOR and canonical restoration. The shared
file broker stages privately, fingerprints external changes and conservatively
reports uncertainty after an installation attempt. No derived cache/layout is saved.

The additive private application protocol exposes lifecycle and prepare/commit save,
document identity, saved checkpoint and dirty state. One retained save ticket holds
one exact exported generation. A later edit does not mark itself clean when that
older save completes. The C# client drains cancelled dispatched replies and updates
authoritative state; prepared exports are explicitly released on abandonment.

A disk operation occupies the child’s single request slot; the UI continues rendering.
This slice does not introduce concurrent file workers, automatic replay/autosave,
crash recovery of unsaved work or unsupported power-loss guarantees. Uncertain
installation can be checked by reopening; Save As to a new file can preserve the
current workbook first. Dirty replacement requires explicit discard.

## Verification scope

Rust round trips cover stable CellIds/axes, graph and retired identity ledgers,
original text and historical binding provenance, moved cells/coordinate reuse,
invalid syntax and out-of-bounds drafts, repair, corruption/truncation and cleanup.
Native broker fault injection covers pre-write, partial-write, pre-install and
post-install uncertainty, external conflicts and competing destination creation.
A real terminated writer leaves the prior document readable; private orphan staging
files may remain and are not silently installed.

The executable C#–Rust suite independently reopens a saved generation in a second
process, checks exact decimal sign/scale/coefficient, formula results and moved
bindings, repairs drafts and exercises Save S/edit T, dirty guards, conflicts,
malformed/oversized Open, failed Save As, cancellation and orderly zero-charge close.
The real-window self-check also saves, creates a new workbook, reopens the saved
file, edits and saves again through the same application methods used by controls.
File picker and confirmation dialogs still require manual interaction coverage;
the self-check supplies a test destination rather than automating OS dialogs.

Full default CI retains decimal/protocol/portable/document/daemon/CLI/REPL checks.
Strict all-feature lint/regression and actual rendered-window checks are additional.
The frozen hosted-budget and storage-independent-workbook architectural regressions
remain intact; early validation failures exposed and corrected both layering issues.
Earlier attempt logs are retained, not represented as clean first-attempt runs.

## Measurements and limits

The application suite writes `persistence.json` for a small semantic fixture and
`persistence-production.json` for the existing approximately 282-cell workload with
its 256-formula chain. The latter records 11 Save/Open observations after one warmup,
plus a bounded larger diagnostic adding 512 numeric cells under unchanged production
limits (`persistence-larger-diagnostic.json`), separating owned graph/projection
capture, graph/envelope encoding, write/install,
read, decode/cross-validation and canonical restoration from complete client times.
These are local observations, not UI latency or power-loss evidence. No larger
research constructor is used in application sessions and no product caps increase.

The fresh Slice B native macOS run completed the
[APFS validation packet](2026-10-20-macos-workbook-persistence-instructions.md);
its results are recorded below, independently of historical Slice A evidence.
The first WSLg `free(): invalid pointer` / SIGABRT remains unexplained despite later
passes. If reproduced, obtain an actionable native backtrace; do not add an unproven
workaround. The historical Linux backend viewport p95 5.343 ms miss and 3.381 ms
repeat remain evidence. No engine redesign or excluded spreadsheet features are added.

[Save/Open component measurements](../performance/workbook-persistence-slice-b.md).

## Final Windows/Linux evidence

| Check | Windows | Isolated Linux |
|---|---:|---:|
| Full CI default Rust tests | 280 | 281 |
| All-feature tests / strict lint | 296 / Passed | 297 / Passed |
| .NET 10 Release solution (six projects) | Passed | Passed |
| Decimal vectors, zero value/error/scale differences | 10,099 | 10,099 |
| Frozen protocol, portable codecs, real daemon/CLI/REPL/document gates | Passed | Passed |
| Final cross-process Save/Open, conflicts, cancellation and cleanup | Passed | Passed |
| Actual window Save/New/Open/edit/Save and image inspection | Passed | Passed (WSLg) |
| Final normal boundary/UI close charge | 0 | 0 |

[Windows CI](../../artifacts/workbook-ci-review-windows/20261020T110818564969Z/summary.json),
[Linux CI](../../artifacts/workbook-ci-review-linux/20261020T111048937848Z/summary.json),
[Windows feature tests](../../artifacts/workbook-review-tests-windows.log),
[Linux sequential gates](../../artifacts/workbook-linux-review.log),
[final Windows boundary](../../artifacts/workbook-boundary-review-windows/results.json),
[source manifest](../../artifacts/workbook-source-manifest.json),
[evidence audit](../../artifacts/workbook-evidence-audit.json).
Native file tests ran on verified C: NTFS and Linux `/dev/sdd` ext4 (`/var/tmp`).
All 162 source/build/resource hashes match the final isolated Linux copy. Windows
full CI is followed by final targeted boundary/window checks covering actionable
file messages and the UI failure/test-filesystem fixes. Linux full CI and final
window checks use the final source. Existing caps/fixtures/assertions are unchanged.

For the approximately 282-cell fixture, complete Save/Open medians are
11.90/4.15 ms Windows and 7.52/3.08 ms Linux disk. With 512 additional numeric cells,
medians are 14.33/9.53 ms and 9.31/7.05 ms respectively. Each uses 11 observations
after one warmup; these are not SLAs. Full component/raw results are linked above.

Earlier failures remain evidence: shared graph-budget validation and workbook
layering failures were fixed without weakening their regressions. A restricted
sandbox denied a private-file test; ordinary user execution passes. Linux reused
outputs retained a failed budget regression during refactoring; a fresh source and
target run passed it. The first isolated copy also omitted `app.manifest`; the
final 162-file copy includes project resources. Do not treat those initial attempts
as passes. The first WSLg file workflow failed on a Windows-mounted destination
and then timed out awaiting a discard dialog. The final self-check uses the Linux
disk test root and disposes a failed smoke session without a dialog. The original
failure report/log remain linked in the performance record; mounted-Windows-path
replacement is not established by native Linux disk validation.

No native allocator abort recurred in the final WSLg check; the historical startup
issue remains unresolved. OS file dialogs, dirty-close confirmation, installers,
AOT, mounted-share file replacement and power-loss durability are not certified.
Native macOS has now completed the APFS packet; three-platform Slice B executable
coverage is recorded below, with the existing limitations retained.

## Native macOS follow-up — 2026-10-20

[Native report](../../artifacts/macos-validation-20261020T111827Z/REPORT.md),
[CI summary](../../artifacts/macos-validation-20261020T111827Z/ci/20261020T111841740354Z/summary.json).
Apple M1 ARM64, macOS 27.0.1, .NET 10.0.402 and Rust 1.101.0-nightly.
The working tree was copied from SMB to verified internal APFS; source, Cargo
outputs, .NET bin/obj, temporary files, file-test root and original reports were local.
All 305 copied-file SHA-256 hashes match the workspace before this documentation
update. Final local/share comparisons show no source changes. HEAD was
`117fd4521b0d7a45411c7f9cefa17864aa4938e4`; the manifest covers working-tree content.
Transfer verification checked 356 returned report files with no missing report links.

| Native check | Result |
|---|---|
| Full CI default Rust tests | 281 passed |
| All-feature tests / strict lint | 297 passed / Passed |
| .NET 10 Release solution | Six projects, zero warnings/errors |
| Decimal parity | 10,099 vectors, zero value/error/scale differences |
| Frozen descriptors, portable codecs, real IPC and document regressions | Passed |
| Independent-process Save/Open and native APFS failure/terminated-writer tests | Passed |
| Actual Avalonia Save As → New → Open → edit → Save | Passed |
| Boundary, persistence and GUI normal close charge | 0 |

No product/dependency/fixture edits, assertion weakening or validation reruns were
needed. The native run confirms saved identities, moved historical references,
exact decimal parts/source, invalid drafts/repair, Save S/edit T dirty state,
external conflicts, failed Open/Save As, cancellation and orderly cleanup.
Abrupt writer termination is tested for prior-file preservation; it does not
provide a graceful cleanup receipt or power-loss guarantee.

The rendered 17x10 image was independently inspected here: document controls,
formula entry, grid and saved-status footer fit without clipping or overlap.
The returned `.pivot` fixture passes header, exact payload-length and SHA-256 checks;
no owned desktop/session processes remained. OS file pickers and Save/Discard/Cancel
choices were not manually exercised. Automated workflow execution does not certify
those dialogs, physical scanout or accessibility.

APFS complete Save/Open medians are 13.18/3.02 ms for the approximately 282-cell
fixture and 14.87/6.31 ms with 512 additional numeric cells. Each is 11 observations
after one warmup. Nine edit-to-render observations span 37.92–66.29 ms, including
35 ms coalescing; they are not a UI p95/SLA. See the component performance record.
The historical WSLg startup defect, mounted-Windows-path replacement limitation
and Linux backend viewport variability remain unresolved. The parent review
received 2026-10-20 accepts Slice B and ADR-0015. See the
[review decision and next assignment](2026-10-20-workbook-persistence-review.md).
