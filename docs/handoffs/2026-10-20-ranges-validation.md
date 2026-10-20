# Slice C validation and performance

Date: 2026-10-20. Three-platform validation complete; parent implementation review pending.

## Automated results

- Windows: full CI **passed**, 290 default Rust tests; .NET Release, 10,099 decimal vectors (zero errors/scale differences), frozen protocol fixtures, local IPC and real application session. [Summary](../../artifacts/ranges-ci-windows-final/20261020T125229942475Z/summary.json).
- Linux: full CI **passed**, 291 default Rust tests; .NET Release, 10,099 decimal vectors (zero errors/scale differences), frozen protocol fixtures, local IPC and real application session. [Summary](../../artifacts/ranges-ci-linux/20261020T125246039643Z/summary.json).

- macOS ARM64: full CI **passed**, 291 default / 307 all-feature Rust tests, strict all-feature lint, six .NET Release projects without warnings/errors, 10,099 decimal vectors without differences, protocol/IPC and real application checks. [Native APFS report](../../artifacts/macos-validation-20261020T130207Z/REPORT.md).

Windows/Linux/macOS strict all-feature lint and all-feature tests pass
(306/307/307 all-feature tests).
Windows, WSLg and native macOS actual-window SUM/edit/move/Save/Open checks pass with zero-charge
normal close. Native files used NTFS, Linux `/var/tmp` and local APFS, respectively. Rendered
PNGs were inspected. These automated workflows do not certify native dialog choices.

- [Windows window](../../artifacts/ranges-ui-windows-final/results.json)
- [WSLg window](../../artifacts/ranges-ui-linux/results.json)
- [macOS window](../../artifacts/macos-validation-20261020T130207Z/ui/results.json)

## Small sparse range observations

Ordinary production caps, A10:H1024 rectangle with sparse occupants, 11 observations
after one warmup per operation. End-to-end milliseconds; these small samples are
observations, not SLAs. Boundary-move and add/remove each include two requests.
Capture includes create and release. Selected calculation uses an already retained
capture. Diagnostics add private response metadata.

| Platform / operation | Median ms | p95 ms | Index nodes min–max | Candidates | Emitted members | Domain allocation bytes min–max |
| --- | ---: | ---: | ---: | ---: | ---: | ---: |
| Windows / bind | 0.1889 | 0.2230 | 44–45 | 4 | 4 | 7632–8256 |
| Windows / boundary_move | 0.3074 | 0.3288 | 100 | 12 | 7 | 6952 |
| Windows / capture | 0.1613 | 0.3533 | 0 | 0 | 0 | 264 |
| Windows / member_add_remove | 0.3232 | 0.3541 | 97 | 10 | 9 | 11352 |
| Windows / member_edit | 0.1551 | 0.1788 | 18 | 6 | 0 | 4456 |
| Windows / selected_sum | 0.1033 | 0.1350 | 10 | 0 | 2 | 0 |
| Linux / bind | 0.3981 | 0.6804 | 44–45 | 4 | 4 | 7728–8736 |
| Linux / boundary_move | 0.8315 | 1.1258 | 100 | 12 | 7 | 6952 |
| Linux / capture | 0.3403 | 0.4354 | 0 | 0 | 0 | 264 |
| Linux / member_add_remove | 0.7512 | 0.9068 | 97 | 10 | 9 | 11352 |
| Linux / member_edit | 0.3828 | 0.4671 | 18 | 6 | 0 | 4456 |
| Linux / selected_sum | 0.2414 | 0.3286 | 10 | 0 | 2 | 0 |
| macOS / bind | 0.0631 | 0.1605 | 44–45 | 4 | 4 | 7512–8424 |
| macOS / boundary_move | 0.1012 | 0.1497 | 100 | 12 | 7 | 6760 |
| macOS / capture | 0.0396 | 0.0609 | 0 | 0 | 0 | 264 |
| macOS / member_add_remove | 0.0890 | 0.1081 | 97 | 10 | 9 | 11352 |
| macOS / member_edit | 0.0410 | 0.0630 | 18 | 6 | 0 | 4360 |
| macOS / selected_sum | 0.0321 | 0.1057 | 10 | 0 | 2 | 0 |

Index visits cover sparse row/column traversal and coverage AVL traversal, not
every scalar graph node/key lookup. Candidates include repeated invalidation queries
against old/new roots. Emitted members include validation and reduction enumerations.
Allocation traffic counts domain-accounted successful allocations, not scratch
reservations, every process allocation or RSS. These numbers do not prove
output-optimality; topology validation still scans live roots.

Range-file timings below are **one observation each**, distinct from the repeated
production scalar persistence fixtures also present in full CI.

| Platform | Prepare ms | Commit ms | Open ms | File bytes | Peak domain charge bytes | Final charge |
| --- | ---: | ---: | ---: | ---: | ---: | ---: |
| Windows | 0.9151 | 12.3260 | 1.3756 | 4117 | 18677545 | 0 |
| Linux | 3.6836 | 7.3956 | 1.2845 | 4117 | 18677329 | 0 |
| macOS | 0.4502 | 16.8220 | 0.6397 | 4117 | 18677329 | 0 |

Raw range observations: [Windows](../../artifacts/ranges-ci-windows-final/20261020T125229942475Z/application-session/ranges.json),
[Linux](../../artifacts/ranges-ci-linux/20261020T125246039643Z/application-session/ranges.json),
[macOS](../../artifacts/macos-validation-20261020T130207Z/ci/20261020T130229206467Z/application-session/ranges.json).
S/T correctness and independent-reader restoration assertions run before timing;
peak charge includes reserved invalidation scratch and is not resident memory.

## Manual evidence and retained failures

The user reports Windows Open/Save As and dirty-close Save/Discard/Cancel passed.
Untouched New prompting was subsequently corrected with real-session regression
checks; this does not claim a second manual pass on the corrected build.

On WSLg, the user reports Save As and standalone Save succeeded to Windows storage,
but Save-on-close left the window open. The reported footer was **“Unsaved Changes -
Installation uncertain: reopen to verify”**. Thus the session did not acknowledge a
clean Save; retaining the window protects work. New bytes may be present without a
confirmed install. Do not report successful Windows-mount Save-on-close or assume
standalone-save success proves installation certainty. The user subsequently
confirmed the manual WSLg Save-on-close repeat **passed** when the `.pivot` file was
on local WSL2 storage: Save correctly closed the application window. The user also
confirmed the corrected WSLg build closes an untouched workbook without a
Save/Discard/Cancel prompt.
The earlier Windows-mount replacement failure and WSLg SIGABRT remain unresolved.
If the native abort recurs, obtain a native backtrace. This run's WSLg automated
native-path check passed without an abort.

Initial sandbox runs failed native transport access/desktop build; authorized reruns
passed. Broader tests also found stale assertions for invalid `SUM(A1)` error offset
and graph revisions after moves. Both now reflect the explicit range/placement
contract; numeric/reference oracles remain intact. Earlier failed logs/CI summaries
remain under artifacts; none are erased. Linux initially tested the earlier source
copy and reproduced the revision assertion; the refreshed source passed full CI.

## Native macOS provenance and manual acceptance

The fresh run used internal APFS for source, Cargo/.NET outputs, temporary files,
file tests and initial reports, then copied reports to SMB. All 318 copied source
hashes matched before/after validation; all 168 reference build inputs matched.
This review independently compared those recorded hashes to the current tree,
with no differences before these documentation updates. No product fixes or
validation reruns were needed. The transfer record verifies 363 returned files,
including manual follow-up, with no missing report links. The rendered image was
also inspected here: controls/grid/footer are readable and SUM shows 3685.

[Source verification](../../artifacts/macos-validation-20261020T130207Z/source-final-verification.json),
[build-input verification](../../artifacts/macos-validation-20261020T130207Z/reference-input-final-verification.json),
[report transfer](../../artifacts/macos-validation-20261020T130207Z/transfer-verification.json).

The Mac report records user confirmation that all six manual checklist groups
passed: untouched fresh/New close, SUM/edit/move, native Open/Save As, dirty-close
Cancel, Save and Discard. These are user-reported interactions, separate from the
automated window check and timing measurements.
[Manual evidence](../../artifacts/macos-validation-20261020T130207Z/manual-gui-results.json).

Native APFS success does not resolve uncertain installation on WSL mounted Windows
paths, explain the historical WSLg allocator abort or establish power-loss durability.
Small cross-platform timings are observations, not controlled hardware comparisons.
The [implementation note](2026-10-20-ranges-and-sum-slice-c.md) is ready for parent
review; design acceptance does not imply implementation acceptance. No next
milestone or broader capacity claim is made.

All 168 final build/test/resource input hashes match the isolated Linux copy:
[manifest](../../artifacts/ranges-build-inputs.json). Documentation was updated after
the copy; compiled inputs did not change. No files were staged or committed.
