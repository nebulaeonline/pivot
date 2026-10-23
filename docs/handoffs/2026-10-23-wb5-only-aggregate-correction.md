# ADR-0021 — WB5-only final correction

Date: 2026-10-23. WB4 decoder removed; revised-source Windows/Linux validation
passed. Fresh native Mac/APFS validation also passed; final parent acceptance remains pending.
[Required final direction](2026-10-22-spreadsheet-aggregate-implementation-review.md).
[Earlier three-platform evidence](2026-10-22-spreadsheet-aggregate-implementation.md)
is historical evidence for the WB4-reading source, not this corrected candidate.

## Result

The native document reader and writer support PIVOTWB5 only. WB1–WB4 reject
explicitly before document decoding or live installation. Removed the legacy magic,
reader mode, conditional token paths and optional blank reconstruction pathway.
WB5 requires its explicit blank-slot overlay to agree with authoritative content.
Unknown tags/kinds and malformed metadata continue to reject. No aggregate reducer,
rounding, allocation limits, private application v4, CB2 or hosted v1 change occurred.

The genuine WB4 migration fixture is removed from the source test suite; its bytes
and provenance remain in historical validation artifacts. A minimal WB4 header
serves as the unsupported-format fixture, alongside WB1–WB3 rejection. The former
migration test now creates WB5 with numeric values, literal text, blank references,
strict SUM and a rejected draft; it checks blank CellId identity, explicit-zero
transitions, COUNT/AVERAGE metadata, stable formula cells and saved checkpoints
through WB5 round trips. Existing moved-bound-reference and malformed-document
checks remain. Application-session tests reject all four obsolete versions while
preserving root values, document identity, revision, path, dirty flag and saved
checkpoint, then Save successfully through the preserved path.

Compatibility is an explicit product commitment. Experimental workbook, native
clipboard and private application formats support only their current version unless
an identified consumer and explicit commitment justify more. Incompatible changes
require version cuts; adding features alone does not. Frozen hosted v1 retains its
established commitment. No new adapter is introduced.

## Revised-source evidence

Focused document suite: 10 passed. Windows default/all-feature: **339/355**;
Linux and native Mac: **340/356** (one added obsolete-version test).
All eleven desktop fields and main/three peers pass in managed/native modes,
with zero remaining charge and no Native AOT publication warnings. The 544-case
independent reference gate remains passing. No tests or limits were weakened.

Windows managed CI: `artifacts/aggregate-wb5-only/windows-ci/20261023T085344149591Z/summary.json`.
Windows strict lint: `windows-all-feature-lint.log`; unrestricted all-feature tests:
`windows-all-feature-tests-final.log` under the same artifact root.
Windows Native AOT: `windows-native-aot/20261023T085630536281Z/summary.json`.
Linux managed: `linux-validation/managed-ci/20261023T085600644001Z/summary.json`;
Linux native: `linux-validation/native-aot/20261023T085845314023Z/summary.json`.
Strict all-feature logs and native-storage provenance accompany returned Linux reports.
The earlier failed Linux missing-tool summary `20261023T085500726337Z` is retained.

The first Windows all-feature run was sandboxed and failed five CLI named-pipe
connection tests with Access denied; the required unrestricted rerun uses unchanged
source. The initial Linux runner lacked Rust in PATH and stopped before tests;
its failed summary is retained. The login-environment rerun supplies the installed
toolchain. Neither failure led to test weakening or product changes.

Linux source, .NET outputs, publish, fixtures and original reports reside under
`/var/tmp/pivot-wb5-only-20261023` on native ext4; the reused Rust cache is
`/mnt/j/coding/pivot/target/ranges-linux`. Native publish trees stay on ext4. All 206 build-input hashes matched the native
source and share; Windows independently verified all 392 returned evidence files.
Final Mac manifests freeze 206 build/test inputs and 407 source files. Documentation
was finalized after the platform runs; compiled inputs did not change.

## Completed macOS validation

[Mac report](../../artifacts/macos-validation-20261023T090419Z/REPORT.md):
`macos-validation-20261023T090419Z`. Managed summary:
`managed-ci/20261023T090518282002Z/summary.json`; native summary:
`native-aot/20261023T090733444817Z/summary.json` within that run.
All four sequential gates passed without product changes, setup failures or gate
reruns. Default/all-feature Rust counts independently recount to 340/356, no failures
or ignored tests. The independent 544-case rational oracle gate passed. All eleven
desktop fields passed in managed/native modes; main and all three helper processes
closed with zero remaining charge, correct execution modes and no owned survivors.
Native publication emitted no warnings. WB5 fixture header/length/digest checks and
current-format invariants passed, including WB1–WB4 rejection and failed Open
preservation. Application version probes confirm v4 acceptance/v1–v3 rejection;
native clipboard remains CB2/v2. Hosted v1 compatibility remains unchanged.

Source, Cargo target, .NET outputs, publish, temporary files and fixtures stayed on
native APFS under `/private/tmp/pivot-wb5-only-20261023T090419Z.6zwlbazu`.
Mach-O/file/lipo evidence confirms ARM64 desktop and Rust child. The actual native
image ran with PIVOT_SESSION_PATH unset and adjacent-child discovery; native peers
used the same published image. Executable size: 19,226,480 bytes; runtime payload:
41,398,448 bytes. Native startup-to-ready observation: 932.3585 ms, a single
instrumented observation, not a performance acceptance claim or physical display timing.
Publish trees/binaries/symbols remain on APFS and are excluded from SMB transfer.

Before these documentation edits, Windows independently matched all 407 frozen
source hashes and 206 build inputs against the returned expected manifests.
The Mac before/after receipts also report matching local/share hashes. Original
manifests remain unchanged. Exactly 458 evidence files were returned; Mac's transfer
receipt reports SHA-256 comparison of every file to its APFS original. It does not
retain individual original hashes, so Windows could inventory/hash the returned
458 files but could not independently repeat that particular transfer comparison.
The separate review receipt is `artifacts/aggregate-wb5-only/macos-review-receipt.json`.

Windows inspected the native aggregate screenshot: selected AVERAGE source,
bounded mean display, translated mean 15 and expected no-values/strict-SUM error
cells appear correctly. Exact numeric/error assertions are automated. Physical
keys, IME/accessibility, clipboard managers and native pickers are not certified by
routed automation. No additional manual checklist is required.

Some returned report material is historical boilerplate: the old optional manual
failure receipt is not a new failure, and the report's older v1 clipboard label is
superseded by its explicit CB2/v2 version evidence. No current gate failed and no
new clipboard regression is reported. The separately unresolved Windows clipboard
issue is not closed by this run.

| Corrected candidate | Default / all-feature Rust | Managed desktop | Actual Native AOT |
| --- | --- | --- | --- |
| Windows x64 | 339 / 355 | Passed | Passed |
| Linux/WSLg x64 | 340 / 356 | Passed | Passed |
| macOS ARM64 / APFS | 340 / 356 | Passed | Passed |

## Remaining gate

Three-platform validation of the corrected candidate is complete. Submit this
handoff for final parent acceptance. The [Mac packet](2026-10-23-macos-wb5-only-instructions.md)
is completed run provenance; no repeat run is requested. No new manual checklist, ADR or architecture review is
required. The unresolved Windows ClipboardChecklistAsync observation remains open;
successful reruns do not establish its cause. Preserve existing WSLg startup,
mounted-filesystem Save uncertainty and historical Linux viewport p95 limits.
Do not start MEDIAN/MODE, formatting, Undo/Redo, structured-data expansion or further
compatibility work. ADR-0021 is not yet finally accepted.
