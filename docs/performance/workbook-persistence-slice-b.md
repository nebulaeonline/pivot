# Native workbook Save/Open measurements — Slice B

Release executable C#–Rust operations, unchanged production limits. Windows uses
NTFS; Linux uses the disk-backed `/var/tmp` test root; native macOS uses verified
internal APFS for source, build outputs, temporary and document files. Full Save includes prepare
and commit; Open includes file read, validation and reconstruction. Milliseconds.

| Platform / fixture | Component | Samples | Median ms | Maximum ms |
|---|---|---:|---:|---:|
| Windows, approximately 282 cells | save | 11 | 11.902 | 13.386 |
| Windows, approximately 282 cells | open | 11 | 4.149 | 4.455 |
| Windows, approximately 282 cells | capture | 11 | 0.789 | 0.847 |
| Windows, approximately 282 cells | encode | 11 | 0.230 | 0.287 |
| Windows, approximately 282 cells | write_install | 11 | 10.529 | 11.984 |
| Windows, approximately 282 cells | read | 11 | 0.519 | 0.554 |
| Windows, approximately 282 cells | decode_validate | 11 | 1.089 | 1.202 |
| Windows, approximately 282 cells | restore | 11 | 2.318 | 2.516 |
| Windows, baseline + 512 numeric cells | save | 11 | 14.333 | 22.974 |
| Windows, baseline + 512 numeric cells | open | 11 | 9.534 | 10.027 |
| Windows, baseline + 512 numeric cells | capture | 11 | 1.604 | 1.813 |
| Windows, baseline + 512 numeric cells | encode | 11 | 0.289 | 0.505 |
| Windows, baseline + 512 numeric cells | write_install | 11 | 12.175 | 20.573 |
| Windows, baseline + 512 numeric cells | read | 11 | 0.596 | 0.750 |
| Windows, baseline + 512 numeric cells | decode_validate | 11 | 2.446 | 2.598 |
| Windows, baseline + 512 numeric cells | restore | 11 | 6.149 | 6.554 |
| Linux disk / WSL2, approximately 282 cells | save | 11 | 7.516 | 9.191 |
| Linux disk / WSL2, approximately 282 cells | open | 11 | 3.078 | 3.288 |
| Linux disk / WSL2, approximately 282 cells | capture | 11 | 0.630 | 0.718 |
| Linux disk / WSL2, approximately 282 cells | encode | 11 | 0.137 | 0.220 |
| Linux disk / WSL2, approximately 282 cells | write_install | 11 | 6.358 | 7.802 |
| Linux disk / WSL2, approximately 282 cells | read | 11 | 0.140 | 0.176 |
| Linux disk / WSL2, approximately 282 cells | decode_validate | 11 | 0.760 | 0.821 |
| Linux disk / WSL2, approximately 282 cells | restore | 11 | 1.884 | 2.158 |
| Linux disk / WSL2, baseline + 512 numeric cells | save | 11 | 9.312 | 11.387 |
| Linux disk / WSL2, baseline + 512 numeric cells | open | 11 | 7.049 | 8.589 |
| Linux disk / WSL2, baseline + 512 numeric cells | capture | 11 | 1.271 | 1.508 |
| Linux disk / WSL2, baseline + 512 numeric cells | encode | 11 | 0.300 | 0.346 |
| Linux disk / WSL2, baseline + 512 numeric cells | write_install | 11 | 7.135 | 9.558 |
| Linux disk / WSL2, baseline + 512 numeric cells | read | 11 | 0.230 | 0.374 |
| Linux disk / WSL2, baseline + 512 numeric cells | decode_validate | 11 | 1.617 | 1.890 |
| Linux disk / WSL2, baseline + 512 numeric cells | restore | 11 | 4.839 | 5.860 |
| Native macOS/APFS, approximately 282 cells | save | 11 | 13.178 | 14.312 |
| Native macOS/APFS, approximately 282 cells | open | 11 | 3.020 | 3.050 |
| Native macOS/APFS, approximately 282 cells | capture | 11 | 0.322 | 0.355 |
| Native macOS/APFS, approximately 282 cells | encode | 11 | 0.492 | 0.495 |
| Native macOS/APFS, approximately 282 cells | write_install | 11 | 12.265 | 13.396 |
| Native macOS/APFS, approximately 282 cells | read | 11 | 0.546 | 0.557 |
| Native macOS/APFS, approximately 282 cells | decode_validate | 11 | 0.963 | 0.981 |
| Native macOS/APFS, approximately 282 cells | restore | 11 | 1.449 | 1.461 |
| Native macOS/APFS, baseline + 512 numeric cells | save | 11 | 14.871 | 14.956 |
| Native macOS/APFS, baseline + 512 numeric cells | open | 11 | 6.307 | 6.374 |
| Native macOS/APFS, baseline + 512 numeric cells | capture | 11 | 0.712 | 0.730 |
| Native macOS/APFS, baseline + 512 numeric cells | encode | 11 | 0.770 | 0.787 |
| Native macOS/APFS, baseline + 512 numeric cells | write_install | 11 | 13.311 | 13.451 |
| Native macOS/APFS, baseline + 512 numeric cells | read | 11 | 0.808 | 0.829 |
| Native macOS/APFS, baseline + 512 numeric cells | decode_validate | 11 | 1.726 | 1.736 |
| Native macOS/APFS, baseline + 512 numeric cells | restore | 11 | 3.696 | 3.718 |

Windows, approximately 282 cells: 120,214 encoded bytes, one warmup; normal close charge zero.
Windows, baseline + 512 numeric cells: 203,670 encoded bytes, one warmup; normal close charge zero.
Linux disk / WSL2, approximately 282 cells: 120,214 encoded bytes, one warmup; normal close charge zero.
Linux disk / WSL2, baseline + 512 numeric cells: 203,670 encoded bytes, one warmup; normal close charge zero.

Native macOS/APFS, approximately 282 cells: 120,214 encoded bytes, one warmup; normal close charge zero.
Native macOS/APFS, baseline + 512 numeric cells: 203,670 encoded bytes, one warmup; normal close charge zero.

[Windows raw observations](../../artifacts/workbook-boundary-review-windows/persistence-production.json),
[Windows larger diagnostic](../../artifacts/workbook-boundary-review-windows/persistence-larger-diagnostic.json),
[Linux raw observations](../../artifacts/workbook-ci-review-linux/20261020T111048937848Z/application-session/persistence-production.json),
[Linux larger diagnostic](../../artifacts/workbook-ci-review-linux/20261020T111048937848Z/application-session/persistence-larger-diagnostic.json).

Capture includes owned graph/projection export and portable graph validation; it is
not a measurement of only an O(1) root pin. Encode includes graph CBOR, projection
encoding and integrity digest. Decode includes semantic/provenance cross-validation;
restore includes canonical import and reconstruction of retained worksheet indexes.
Write/install includes staging, sync, baseline checks, installation and verification.
Independent component medians do not sum exactly to complete request medians.

These 11 observations after one warmup are local file measurements, not p95/SLA,
cold-start distributions, GUI latency or power-loss tests. The larger diagnostic
adds 512 numeric cells using ordinary production sessions and the same limits;
it does not establish 100K or million-cell desktop capacity. Caller-owned encoded
and decoded buffers remain outside continuing Rust domain charges, with a 4 MiB
document limit and one retained save export.

## Actual window observations

All three final window checks pass, including Save As → New → Open → edit → Save.
Their rendered 17x10 images were inspected; file controls, grid, selected cell and
saved-status footer fit without overlap. OS picker and confirmation dialogs were
not automated. The self-check supplies a native-filesystem destination and copies
the final saved bytes back with its report.

| Platform | Actions | Edit-to-render range ms | Final Rust charge |
|---|---:|---:|---:|
| Windows | 9 | 45.51–63.45 | 0 |
| WSLg Linux | 9 | 39.71–115.95 | 0 |
| Native macOS | 9 | 37.92–66.29 | 0 |

[Windows UI report/image](../../artifacts/workbook-ui-final-windows/results.json),
[Windows image](../../artifacts/workbook-ui-final-windows/results.png),
[Linux UI report](../../artifacts/workbook-ui-review-linux/results.json),
[Linux image](../../artifacts/workbook-ui-review-linux/results.png).

UI timings include 35 ms coalescing and dispatcher/render scheduling; these few
actions are not a UI p95, physical scanout or OS input automation measurement.

The first Slice B WSLg persistence check saved onto the Windows-mounted artifacts
path and failed on replacing the reopened document; its failure then waited on an
unattended discard prompt until timeout. [Original report](../../artifacts/workbook-ui-linux/results.json),
[log](../../artifacts/workbook-linux-verified.log). The final harness uses native Linux
disk storage and closes failures without an interactive prompt. Replacement semantics
on the mounted Windows path are not established by the passing native disk run.

Native macOS/APFS full CI, file-failure and actual-window execution passed without
fixes or reruns. OS file-picker and discard-dialog interactions were not manually
tested. [Native report](../../artifacts/macos-validation-20261020T111827Z/REPORT.md),
[Save/Open observations](../../artifacts/macos-validation-20261020T111827Z/ci/20261020T111841740354Z/application-session/persistence-production.json),
[larger diagnostic](../../artifacts/macos-validation-20261020T111827Z/ci/20261020T111841740354Z/application-session/persistence-larger-diagnostic.json),
[UI report](../../artifacts/macos-validation-20261020T111827Z/ui/results.json),
[inspected image](../../artifacts/macos-validation-20261020T111827Z/ui/results.png).
The small semantic fixture's 35.31 ms commit is a separate single observation,
not part of the production-fixture distribution or a latency guarantee. The accepted
[Slice A boundary/UI record](application-worksheet-slice-a.md) remains historical;
its unresolved WSLg native startup abort and the earlier Linux backend viewport
5.343 ms miss / 3.381 ms repeat are not resolved by these file measurements.
