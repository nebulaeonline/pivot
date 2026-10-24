# G1 ordinary native Save promotion

Date: 2026-10-24. **Ordinary WB5 export now uses the accepted validated projection.
Windows/native Linux full gates pass; fresh native Mac ARM64/APFS full
gates and parent review are required for G1 closure. G2–G7 remain deferred.**
[Parent authorization](2026-10-24-historical-save-g1-internal-review.md),
[accepted internal correction](2026-10-24-historical-native-save-g1.md),
[ADR-0023](../adr/0023-document-transaction-history.md).

## Change and authority contract

`Document::export_document` now calls the validated current-content projection in
default builds, for both ordinary current and legitimate historical roots. It is
one explicit path, not retry/fallback from a failed full-ledger transfer. The hidden
cancellable export entry point uses the same implementation. The experimental
`export_historical_document` entry point was removed; the research association now
calls the same ordinary exporter used by the real Rust application session.

The core/workbook projection and correspondence verifier are available in default
builds. Admission for selection metadata uses the active core selection identity
count instead of the feature-only history accounting accessor. It still reserves
work before copying and retains its lease through the unchanged WB5 encoder.
No production limit, wire format, protocol, supported capability or allocation
floor changes. Generic full-ledger graph transfer remains strict. Structured
projection and other unsupported native capabilities remain rejected.

Save preparation and verified installation retain their existing authority and
ownership behavior: no document edit, no Expected/ContentStateId/runtime-ID/UUID
change, no lifetime-ledger reset, no cursor/Redo change, and independent captures
remain usable. Verified completion changes only saved-file/checkpoint authority.
Immutable tickets, cancellation/stale/rejection behavior, file conflict detection
and uncertain-installation fences remain in the existing session/file adapters.
No production history coordinator or session/desktop Undo/Redo was added.

## Evidence and coverage

`artifacts/history-save-g1-promotion/` holds the new evidence, separate from the
accepted internal G1 and original Stage 4 manifests. The new build-input manifest
covers **236 Rust/managed/test/script inputs**, including the client/session/UI
and Native AOT gate sources. Earlier 168-input Rust-only Mac evidence is not reused
as validation of this promoted source.

A default-feature equivalence test compares the new projection with the strict
full-ledger projection for nonhistorical content, including moved scalar bindings,
SUM/AVERAGE, blank identities, retired records, invalid source, exact decimal scale
and signed zero. Both images pass the real WB5 encoder/reader and retain identical
CellViews and provenance. It verifies unchanged Expected and independent captures,
pre-cancellation, stale authority, admission exhaustion and zero-charge cleanup.
The codec rejection/post-capture cancellation tests remain in the feature suite.

The historical sparse-root and actual child-process cold/native tests now exercise
ordinary `export_document`. The 69-case research association also uses ordinary
export, including a new historical B Save after D-only allocations, actual native
installation/reopen, preserved Redo and clean B → dirty D → clean B. Strict generic
transfer remains a negative regression. Existing prepared-B/edit/Undo/Clear,
uncertain install/reconciliation, process-kill and independent-file tests remain.

The production Rust session does not expose history activation. Per the parent
exception, history activation stays in the accepted research boundary and tests
the promoted exporter directly. Managed/session/desktop tests establish ordinary
current-workbook Save/Open/Save As behavior, ticket authority, stale/cancelled/
failed Save and cleanup. They do not certify a historical UI workflow.

## Final local gates

| Gate | Windows NTFS | Native Linux ext4 / WSLg |
| --- | --- | --- |
| Rust default / all-feature tests | 347 / 390 | 348 / 391 |
| Format, default and all-feature strict lint | Passed | Passed |
| Full .NET, protocol, IPC and managed session CI | Passed | Passed |
| Automated managed desktop Save/Open/Save As | Passed | Passed |
| Actual Native AOT publish and execution | Passed on unchanged rerun | Passed |
| Managed/AOT main + three peers, twelve acceptance fields | 8 reports, zero charge | 8 reports, zero charge |
| Historical native matrix / original hashes | 69 cases / 884 | 69 cases / 884 |
| Completed zero-charge logs / deliberate fault kills | 150 / 24 | 150 / 24 |

`verification.json` independently checks all 236 inputs, CI/AOT receipts, all twelve
fields, main/peer charge/execution modes, published binary hashes, native copied
hashes and four managed/AOT WB5 fixture checksums. Native original-vs-copy checks
are in the platform receipts. Linux reports were hashed on native storage before
copy: 1,460 report hashes matched, recorded in `linux-copy-verification.json`.

Final Windows CI: `windows-ci/20261024T131652218847Z/`; AOT:
`windows-aot/20261024T132903294244Z/`; extra Rust: `extra-rust/`; matrix:
`windows-results/`. Final Linux evidence: `linux-native-report/`, with CI
`ci/20261024T132705361964Z/`, AOT `aot/20261024T133140409511Z/`, feature gates
`extra-rust/` and matrix `linux-results/`. Original native Linux source, reports,
test temporaries and journals remain under `/var/tmp/pivot-history-save-g1-promotion*`;
only target cache and returned receipts use the shared volume. Windows uses local J:.

## Validation attempts and qualifications

The first Windows/Linux CI attempts stopped at strict Clippy's test-module ordering
rule after adding the default-feature equivalence test. Test modules were moved to
the end of the file; no lint suppression or exporter behavior change was made.
Those failed logs and initial input/source archives are retained.

The next Windows full CI passed. The next Linux run passed non-UI gates but the
managed desktop failed `Fill preserves reversed selection, active cell and scroll`.
The report's outer message `Worksheet text acceptance missing` was secondary to
that actual Fill error. A fresh full CI with source, temporary files, test data and
report output on Linux storage passed on unchanged source. The earlier report used
shared output; no causal link between storage and the Fill failure is established.
Its files and failure remain visible.

The first Windows Native AOT publish succeeded and the actual native process ran,
but its checklist failed `Unsupported OS Copy leaves clipboard unchanged`. One
unchanged-source full publish/execute rerun passed. This is a newly observed
clipboard assertion; its cause is unresolved and is not assumed to be the older
ClipboardChecklist null-destination issue. No UI or clipboard implementation or
assertion was changed. Preserve all failures alongside final passes.

A report-collection command initially lacked rustc on the direct WSL command PATH;
collection used the normal login-shell toolchain environment and verified all
inputs and original/copy hashes. No tooling or source changes were made.

Native file-install uncertainty, invalid-draft first-core diagnostic ordering,
process termination versus power loss, network-share exclusions and logical
retirement versus physical reclamation remain qualified as in accepted G1.
Passing local gates does not establish arbitrary filesystem or device durability.

## Remaining review gate

[Mac full validation instructions](2026-10-24-macos-g1-promotion-instructions.md)
require a fresh APFS-native copy, matching all 236 inputs, full CI with desktop,
all-feature Rust/lint, the native historical matrix, and actual Native AOT
publish **and execution** of main/peer processes with zero-charge cleanup.
No new manual UI checklist is required. Original APFS evidence must be hashed
before copying to a new shared `artifacts/macos-validation-<timestamp>/` report.

After source-matched Mac reconciliation, return final ordinary-path evidence to
the parent for G1 closure. Do not advance to G2, a storage backend, mutation-local
producer, protocol revision, history coordinator or desktop Undo/Redo.
