# ADR-0019 bounded structured-result/projection proof

Date: 2026-10-21. Implementation and Windows/Linux/native macOS ARM64 validation
complete; accepted by the parent on 2026-10-22, experimental milestone closed.
[Acceptance and scope closure](2026-10-22-structured-results-projection-implementation-review.md). The worksheet-text and Native AOT milestones
remain accepted and closed. This implements only the
[approved proof](2026-10-21-structured-results-projection-design-review.md).

## Result

Launch `python scripts/run_desktop.py --projections` to open the in-memory experiment.
One fixed document-owned InvoiceLines producer publishes immutable keyed records;
two read-only flow bands share the result after ordinary row 8. Buttons publish
zero/three/five records, remove L2, reverse one view or move its anchor. Repeated
products have different LineKeys. Projected membership never becomes worksheet cells.
Ordinary A1 names, stable CellIds, scalar bindings and SUM rectangles retain their
existing meanings as bands grow and move.

The numeric adapter checks Decimal96 addition in canonical record order, then feeds
captured TotalAmount into an existing named numeric invocation (`total * 1.1`). It
does not add graph strings, collections or hidden ordinary inputs. Empty means zero;
missing/unavailable/overflow errors remain explicit. Reversing a view does not alter
numeric order. A captured old result remains valid while new generations publish.

The private Rust session is the exclusive mutation/publication/composite-capture
coordinator. Workbook and structured roots are pinned before another command runs;
evaluation uses immutable captures. Worksheet revision, result epoch and presentation
revision remain distinct. The private application protocol is version **3**, replacing
version 2; the hosted protocol, WB4 and PIVOTCB2/cell-block-v2 stay unchanged.

Tagged map rows distinguish ordinary positions from projected keys. Delayed targets
must match producer, generation, epoch and presentation revision. Removing selected
L2 moves selection to its header with an explicit missing-record indication rather
than resolving the new record at visible #2. Ordinary selection, copy and navigation
exclude the bands. The viewport still paints at most 20 by 10 cells.

The experiment has an explicit non-savable banner/footer, rejects Save/Save As before
filesystem work and offers Discard/Cancel (no Save) when closing modified data. Clean
proof startup has no false dirty prompt. Ordinary WB4 workflows remain covered by
the original complete automated suite.

## Bounds and tests

Caps remain one producer, two projections, eight fields (four used), 64 records,
4,096-byte text, 128-byte keys, 256 KiB complete generations, 1 MiB structured
ownership and two retained captures. Generation admission precedes allocation of
owned clones. Payload/index, descriptor, capture and frame reservations use the
existing runtime memory domain and survive with their owners. Exhaustion rejects
without evicting captures, truncating data or replacing the current generation.
These are conservative domain charges, not measured process heap/RSS usage.

Four new Rust regressions cover unavailable/empty/three/five results, repeated
products, absent/duplicate/missing/returning keys, schema mismatch, canonical overflow,
detached G1 evaluation during publication, shared two-view ownership, reflow bounds,
retained captures, admission pressure and zero-charge cleanup. C#↔Rust checks cover
composite old/new workbook/result roots, numeric invocation, limits, save refusal and
transactional discard. Desktop automation adds a ninth `structured_projection`
acceptance field and a projections screenshot in managed and Native AOT reports.
It forces L2 disappearance between reading a target and applying it, tests read-only
entry/paste, geometry, logical copy/navigation and actual Cancel/Discard dialogs.

## Validation evidence

Evidence root: `artifacts/projection-proof/`.
Windows managed full CI passes in `windows-ci/20261021T114921821088Z/summary.json`;
all-feature Rust tests and strict lint pass. Initial desktop test assumed row 9 was
visible in a 16-row viewport; its assertion was corrected to read the logical cell
through the session. The next standalone desktop run passed; final full CI also
includes logical copy/navigation coverage. No product assertions were weakened.
Linux full CI, strict all-feature checks and final Windows/Linux Native AOT gates
also pass. Final managed desktop reruns include the screenshot-driven correction
that confines a selected header outline to the header, rather than also its footer.
The earlier AOT pass is retained; final evidence supersedes its rendering snapshot.

[Fresh macOS/APFS packet](2026-10-21-macos-projection-proof-instructions.md).
Routine manual replay is optional. Physical keyboard, IME and native picker behavior
are not certified by routed automation. Preserve prior WSLg startup allocator abort,
Windows-mounted-path save installation uncertainty and historical Linux viewport p95
exception. No deployment or power-loss guarantee is added.

No connectors, SQL, scheduler, computational collections, per-record transformations,
WB5/persisted projections, permanent `#` syntax, projected editing/materialization,
formatting or printing. The parent has accepted this bounded proof; further integration remains a separate decision.

## Windows and Linux evidence

| Gate | Windows x64 | Linux/WSLg x64 |
| --- | --- | --- |
| Full managed CI | [summary](../../artifacts/projection-proof/windows-ci/20261021T114921821088Z/summary.json) | [summary](../../artifacts/projection-proof/linux-reports/linux-ci/20261021T115023294172Z/summary.json) |
| Default / all-feature Rust | 318 / 334 passed | 319 / 335 passed |
| Final managed desktop | [nine fields and zero charge](../../artifacts/projection-proof/windows-ui-final/results.json) | [nine fields and zero charge](../../artifacts/projection-proof/linux-reports/linux-ui-final/results.json) |
| Final Native AOT | [summary](../../artifacts/projection-proof/windows-aot-final/20261021T115922548156Z/summary.json) | [summary](../../artifacts/projection-proof/linux-reports/linux-aot/20261021T115908037742Z/summary.json) |

Strict all-feature lint and all-feature tests pass on both hosts. Native publishes
have no warnings, execute their actual images with adjacent Rust children, and pass
all nine fields and all three peers with zero charge. Windows/Linux protocol probes
reject application v1/v2 before document creation, accept v3 and close at zero charge
(`windows-version-probe.json` / `linux-version-probe.json` under the evidence root).
Decimal parity remains 10,099 vectors with zero value/error or scale discrepancies.

Linux source, .NET outputs, files and publish image reside under native ext4
`/var/tmp/pivot-projection-proof-20261021`; its Cargo target uses the existing Linux
cache. All **190** build/test inputs match the reference manifest, native Linux copy
and current Windows source. All **392** returned report/fixture files match their
native originals. Published Linux binaries/symbols stay on ext4; their inventories
are in the returned AOT summary. [Input verification](../../artifacts/projection-proof/linux-reports/build-input-verification.json),
[transfer verification](../../artifacts/projection-proof/linux-reports/transfer-verification.json).
The initial source.tar is staging history; the final 190-entry manifest is authoritative.

Screenshot inspection confirms distinct read-only flow chrome and # numbering,
ordinary row labels and a visible non-savable state. Native executable startup/size
observations remain diagnostics, not performance conclusions. The historical limits
listed above are not closed by these successful runs. No new manual acceptance gate
is introduced. Three-platform validation and parent implementation acceptance are complete. The
experimental milestone is closed; no next feature is started.

## macOS/APFS review — three-platform validation complete

Reviewed `artifacts/macos-validation-20261021T230643Z` on 2026-10-21.
[Mac report](../../artifacts/macos-validation-20261021T230643Z/REPORT.md),
[independent Windows-side review checks](../../artifacts/projection-proof/macos-review-verification.json).
The dirty/untracked source was copied from SMB to internal native APFS at
`/private/tmp/pivot-projection-proof-20261021T230643Z.2j08hdcd`; Cargo targets, .NET
builds, document fixtures, temporary files and native publish all stayed on APFS.
Both published desktop and adjacent Rust child are Mach-O ARM64; native execution
is established by file/lipo and actual main/peer reports, without Rosetta or JIT substitution.

| Gate | Result | Evidence |
| --- | --- | --- |
| Managed CI | Passed; 319 default Rust tests | [summary](../../artifacts/macos-validation-20261021T230643Z/managed-ci/20261021T230658469298Z/summary.json) |
| Strict all-feature lint / tests | Passed; 335 tests | [lint](../../artifacts/macos-validation-20261021T230643Z/all-features-lint.log), [tests](../../artifacts/macos-validation-20261021T230643Z/all-features-tests.log) |
| Managed desktop | All nine fields, all three peers, zero charge | [result](../../artifacts/macos-validation-20261021T230643Z/managed-ci/20261021T230658469298Z/desktop-ui/results.json) |
| Native AOT publish / execute | Passed; no publish warnings | [summary](../../artifacts/macos-validation-20261021T230643Z/native-aot/20261021T230908663998Z/summary.json) |
| Native desktop | All nine fields, all three native peers, zero charge | [result](../../artifacts/macos-validation-20261021T230643Z/native-aot/20261021T230908663998Z/desktop-ui/results.json) |

No product changes, dependency changes, assertion changes or gate reruns were needed.
The session suite executes ProjectionChecks before emitting its normal results.json;
there is no separate projections.json. Its assertions cover coherent captures,
key disappearance, numeric invocation and non-savable lifecycle. Managed/native
WB4 fixtures independently pass magic, length and SHA-256 checks. The private
application probe confirms v1/v2 rejection, v3 acceptance and zero-charge close.

Before these documentation edits, all **373** source-manifest hashes matched the
current tree, and all **190** build/test input hashes matched both the returned
reference and current source. The Mac reports unchanged local/share inputs before
and after testing. All **441** returned files are present. The Mac transfer receipt
reports SHA-256 comparisons against local APFS originals and no missing report links;
those APFS originals are not accessible from Windows for a second transfer comparison.
[Source verification](../../artifacts/macos-validation-20261021T230643Z/source-final-verification.json),
[input verification](../../artifacts/macos-validation-20261021T230643Z/build-input-local-share-verification.json),
[transfer receipt](../../artifacts/macos-validation-20261021T230643Z/transfer-verification.json).
Native binaries/symbols remain on APFS and are inventoried in the native summary.

The native projection screenshot confirms read-only band chrome, separate # numbering,
ordinary rows 4/5 surrounding the reflow, TotalAmount 150, the explicit non-savable
banner/footer and the corrected header-only selection outline.
[Screenshot](../../artifacts/macos-validation-20261021T230643Z/native-aot/20261021T230908663998Z/desktop-ui/results.projections.png).
Native desktop size is 19,126,944 bytes; runtime payload excluding symbols is
41,125,680 bytes. Main-to-ready was 848.4324 ms in one diagnostic observation, not
a performance benchmark or cross-platform comparison.

Report clarification: an earlier generic paragraph in REPORT.md says clipboard
`org.pivot.cell-block-v1`. The report's later version evidence correctly identifies
**PIVOTCB2 / org.pivot.cell-block-v2** in the tested source. No clipboard downgrade
occurred. `prior-manual-failure.json` is a carried-forward report dated 2026-10-20,
not a failure observed in this run; it does not override the subsequent user manual
acceptance recorded in project history. This run used routed automated input and
adds no physical-key, IME, accessibility or native-picker certification.

The historical WSLg allocator abort, Windows-mounted-path save uncertainty and Linux
viewport p95 exception remain documented. Validation of the bounded proof is complete
on all three platforms; the parent accepted the implementation on 2026-10-22.
Permanent product integration and further scope remain unapproved.
