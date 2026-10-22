# ADR-0020 bounded worksheet binding proof — implementation and validation

Date: 2026-10-22. Experimental implementation is in place. Windows, Linux/WSLg and native macOS ARM64/APFS
managed CI, strict all-feature checks and Native AOT pass. Three-platform validation
is complete and accepted; the bounded milestone is closed. Permanent integration
is not authorized.
[Implementation acceptance and scope closure](2026-10-22-worksheet-structured-binding-implementation-review.md).
[Approved contracts](2026-10-22-structured-output-binding-contract-review.md),
[ADR-0020](../adr/0020-worksheet-structured-output-bindings.md),
[Mac validation packet](2026-10-22-macos-worksheet-bindings-instructions.md).

## Behavior and architecture

Launch `python scripts/run_desktop.py --bindings`. Enter
`=@{InvoiceLines.TotalAmount}*1.1+B1` in an ordinary cell. The fixed source initially
contains three records totaling 60. Publish 5 changes that total to 150 and the
formula result from 66 to 165 when B1 is blank. Cell identity, formula definition,
A1 targets and structured output identity remain unchanged. Ordinary dependents
and SUM evaluate through the same canonical graph.

The proof retains two read-only projections and one Decimal96 output. The document
maps `(DocumentId, ProducerId, OutputId, Decimal96)` to a private NodeId and opaque
CapturedScalarId. The numeric core receives only an immutable context containing
exact Decimal96 or a generic FaultId. It has no producer/record/refresh knowledge.
Contexts pin their exact graph snapshot; a context from another capture rejects,
and a retained context prevents premature runtime disposal. Context-free evaluation
and frozen scalar materialization reject graphs containing source leaves.

Faults preserve generic source identity through ordinary dependency edges. Document
metadata supplies MissingProducer, MissingOutput, UnavailableResult, SchemaMismatch,
TypeMismatch or NumericOverflow. Canonical tests verify source-first versus
arithmetic-first postfix errors and established row-major SUM precedence. The
preflight nonnumeric-marker test established existing behavior before adding the
source leaf. No discrepancy was found or numeric traversal changed. Existing
non-source error formatting and evaluation checks remain in force.

The private application envelope is version **4**; older versions reject. ResolveFormula
returns exact ordered UTF-8 spans and typed portable IDs. ApplyFormula reparses and
validates count, spans, IDs, Decimal96 type and the exact computational-state token.
Source publication rotates that token without a worksheet-definition edit. Captures
retain their own computation token, worksheet and source roots; responses distinguish
captured identity from live authority. The desktop refuses frames for stale authority.
Layout alone does not change computational authority.

Renaming producer/output changes labels without rebinding existing formulas. Unchanged
F2 exit preserves the old authoritative binding. Deletion and same-label recreation
produce fresh IDs; old formulas remain explicitly dangling. A published unavailable
head never consumes cached last-good data; an empty valid result is exact zero.
A rejected publication keeps its previous head/epoch and exposes the attempt error.
The fixed two-view scaffolding has layout/reversal controls; no hide/delete-view
lifecycle API was added, and those future matrix variants are not claimed as separate
UI acceptance tests. Bindings contain no projection identity.
The fixed proof does not provide general schema evolution; mismatch reasons have
explicit document-fault tests, rather than a new connector or type system.

DuplicateBlock is a server-owned same-document operation. It snapshots the source,
preserves structured IDs after rename, translates ordinary A1 operands and stages
the whole destination atomically. OS Copy/Cut cannot serialize these bindings and
refuses before clipboard mutation. Text-editor clipboard behavior and ordinary
PIVOTCB2 remain available. Structured formula strings in rectangle paste reject as
a whole; no native-to-TSV fallback is introduced. WB4 and hosted protocol v1 are
unchanged. The experimental document cannot Save/Save As; close offers Discard/Cancel.

## Ownership and limits

Annotated formula source/tokens have one canonical shared immutable allocation per
distinct version, referenced by persistent records and annotation indexes. Captures
share it. Copy-out views own bounded transient buffers, separately accounted through
the existing runtime. Two snapshots of the same 32 formulas therefore retain 32
versions, not 96. Old and changed current formulas retain 64 versions; a 65th is
rejected until the last old owner is reclaimed. Tests verify this final-owner boundary.

Registry producer/output descriptors reserve two entries at 1,024 bytes each;
retained directory versions count against the 64-entry ceiling. The adapter pins
its original registry entries, including after deletion. The sole 2,048-byte slot
continues to count while old roots/captures retain it; replacement cannot bypass
that limit. After all references and pins are reclaimed, a new distinct slot can
be allocated. Registry saturation and reclamation are tested without raising caps.

Each annotated formula version reserves 8,192 bytes, with 32 current/64 retained,
and at most 16 occurrences. Each immutable context reserves 4,096 bytes, with at
most two retained contexts. These conservative admission charges share the existing
1 MiB structured budget with result generations, indexes and capture pins. A joint
budget test retains a large generation and 64 formula versions, rejects another
publication atomically, verifies old results, then succeeds after final-owner release.
Canonical graph/persistent-index allocations also retain their existing runtime-domain
charges; the structured ceiling is not a process-RSS claim.

## Evidence

| Platform | Managed CI and desktop | Final Rust default / all-feature | Native AOT |
| --- | --- | --- | --- |
| Windows x64 | Passed | 328 / 344 | Passed, win-x64 |
| Linux x64 / WSLg | Passed | 329 / 345 | Passed, linux-x64 |
| Native macOS ARM64 / APFS | Passed | 329 / 345 | Passed, osx-arm64 |

Windows evidence under `artifacts/worksheet-bindings/`:

- `windows-ci-final/20261022T144528842375Z/summary.json`: full managed CI with desktop.
- final CI `rust-tests.log`, `windows-all-features.log`,
  `windows-all-features-lint.log`: final test-only additions and strict lint.
- `windows-native-aot-final/20261022T144648802593Z/summary.json`: actual native desktop
  and adjacent Rust child, three peers, all ten desktop fields and zero charge.
- `windows-boundary/results.json`: C#–Rust binding checks include malformed span,
  ordinal, document/producer/output ID, type, missing/extra annotations, stale token,
  old capture, dependent SUM, rename/duplicate and missing/unavailable/empty heads.
- `windows-ui-4/results.bindings.png`: inspected bound-cell/source-ID display,
  private formula entry, experimental banner and separate read-only source band.

Managed and native reports require all ten fields: `worksheet_binding`,
`structured_projection`, `worksheet_text`, `arrow_edit`, `clipboard_checklist`,
`rectangular_clipboard`, `rectangular_selection`, `direct_paste`, `clipboard_focus`,
`in_place_editing`. Main and peer reports require zero remaining charge. The native
gate publishes and executes the native image, rather than substituting managed or
ReadyToRun execution. Shared clipboard locking remains enabled.

The final Rust suite adds two core context tests, one canonical preflight test and
seven workbook binding tests. Earlier CI Rust logs predate the final quota/diagnostic
test-only extension. Final CI/all-feature reruns also include checked coordinate
addition in DuplicateBlock: hostile 0..u32::MAX rectangles reject without overflow
or root mutation. Development
checks caught a forbidden core UUID dependency (removed; UUIDs originate in the
document), nullable clipboard access and a C# local-name collision; final builds
pass with warnings as errors. Failed development checks are not acceptance results.

One final Windows managed run (`windows-ci-final/20261022T144326138920Z`)
passed the binding checks but failed the existing ClipboardChecklistAsync at line 52
with a null destination cell after paste. The failed report is retained. The unchanged
full rerun passed; this intermittent clipboard-automation failure is recorded rather
than treated as a source-binding semantic failure or silently erased. No assertion,
limit, clipboard lock or test was weakened to obtain the rerun pass.

Linux source, .NET outputs, native publish and document fixtures reside in
`/var/tmp/pivot-worksheet-bindings-20261022`, on native Linux storage. The existing
Rust target cache remains `/mnt/j/coding/pivot/target/ranges-linux`. Direct protocol probes on Windows and Linux also reject versions 1/2/3, accept
version 4 and close with zero charge (`windows-version-cut.json` / `version-cut.json`).
Final managed evidence is `linux-validation/managed-ci-final/20261022T144338592806Z/summary.json`;
strict lint/all-feature logs and `native-aot-final/20261022T144619831248Z/summary.json`
are alongside it. Returned reports, source/build-input verification and transfer hashes
are under `linux-validation/`.
All 199 returned Linux build-input hashes match the final share; all 768 returned
files match their local originals. The final macOS packet freezes 390 source files
and 199 build/test inputs in `artifacts/worksheet-bindings/`.
`validation-summary.json` indexes the final Windows/Linux evidence and retained failure.
The macOS packet requires all source, targets, .NET/native outputs and fixtures on
native APFS, with returned reports on SMB under `artifacts/macos-validation-<timestamp>`.

## macOS/APFS review — three-platform validation complete

Reviewed `artifacts/macos-validation-20261022T150456Z/` on 2026-10-22.
[Mac report](../../artifacts/macos-validation-20261022T150456Z/REPORT.md).
The dirty/untracked tree at HEAD `5557e4bda8df871e8a7f3aeec603e672cb1fbec6`
was copied from SMB to `/private/tmp/pivot-bindings-jugdpijr`. Source, Cargo target,
.NET build/publish, temporary files and fixtures were on APFS `/dev/disk3s1`.
The desktop and child are Mach-O ARM64; Rosetta translation was off.

All four gates exited 0 without product/dependency/test/assertion/limit changes:

- Managed CI including desktop: `managed-ci/20261022T145620532055Z/summary.json`.
- Strict all-feature clippy: `all-feature-clippy.log`.
- All-feature Rust suite: `all-feature-tests.log`.
- Actual osx-arm64 publish/execute: `native-aot/20261022T145828566024Z/summary.json`.

Default/all-feature Rust totals are **329/345**, independently recounted from logs.
Both desktop modes pass all ten fields; the main and all three peers in each mode
pass with zero remaining charge. Execution reports distinguish managed and native-aot.
The native run discovers its adjacent Rust child with PIVOT_SESSION_PATH unset;
publish warnings are empty. Clipboard suites ran sequentially under the shared lock.

Before/after verification records zero mismatches for **199 build/test inputs and
390 source files** against both APFS and SMB. This review also compared those hashes
to the current workspace before documentation edits. All **475 returned evidence
files** are present and independently match their recorded SHA-256 hashes; the
transfer receipt matches the transfer-manifest hash. Expected validation manifests
remain unchanged as provenance; the following documentation reconciliation does not
change tested product/build inputs.

Inspected the native binding screenshot: A1 displays 165, the original private
formula remains visible, inspection shows renamed producer labels and portable IDs,
and the read-only source band stays separate from ordinary rows. Managed/native
screenshots, peers, clipboard locks and generated fixtures are retained. This
confirms automated routed behavior, not physical keyboard/native-picker certification.

The native desktop is 19,193,312 bytes; its Rust child is 2,434,352 bytes. Total
publish size is 98,820,231 bytes, runtime payload 41,317,488 bytes. Startup to ready
was 800.494 ms native and 1,451.290 ms managed. These are observations, not thresholds
or physical-display timings. Native binaries/symbols are retained on APFS, excluded
from SMB transfer and inventoried by hash/size in the native summary.

The initial sandbox attempt failed before tests because rustup could not remove
rustup-init and diskutil could not access its framework. Its logs remain in
`sandbox-attempt/`. The approved unrestricted run passed without source changes.
A directory argument to diskutil was invalid; querying disk3s1 established APFS.
These were environment/provenance issues, not product/test failures. The prior
Windows clipboard-check failure and successful unchanged rerun remain recorded;
a successful Mac run does not erase that intermittent observation or the separate
historic WSLg/save/timing limitations.

## Boundary and next gate

The prototype remains opt-in and non-savable. There is no permanent `#` grammar,
WB5/clipboard revision, persistence, SQL/connectors, automatic refresh, collections,
per-record graph, structural row editing, formatting or capacity increase. Existing
WSLg allocator startup abort, Windows-mounted-path save uncertainty and historical
Linux viewport p95 observations remain separate known platform limits. This milestone
adds no recurring manual replay requirement.

The parent accepted this bounded implementation and its three-platform evidence.
The milestone is closed, with the Windows clipboard observation explicitly open.
Stop before another implementation milestone; the recommended next step is a
product-direction review, not automatic permanent structured-data integration.
