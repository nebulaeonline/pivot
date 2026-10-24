# Pivot implementation status

Date: 2026-10-23

Latest accepted milestone: **Four-Direction Rectangular Fill — accepted; ADR-0022 closed (2026-10-23)**.
[Final implementation acceptance](handoffs/2026-10-23-four-direction-fill-final-review.md),
[implementation and three-platform evidence](handoffs/2026-10-23-four-direction-fill-implementation.md).
Four-direction source-edge Fill, typed identity/no-op semantics, atomicity and the
focused staged cycle validator are accepted. Windows 346/362 and Linux/Mac 347/363
Rust tests pass, with managed/Native AOT execution, twelve fields and zero-charge
main/peers. Mac 419 source hashes, 211 inputs and 465 APFS-original evidence hashes
were independently verified, including manifest/receipt checksums. ADR-0021 remains
accepted/closed; its earlier evidence limitations remain historical.

**Open reliability issue:** intermittent Windows ClipboardChecklistAsync null
destination after paste. The unchanged rerun passed; the cause remains unresolved.
[Failure evidence and recurrence diagnostics](handoffs/2026-10-22-worksheet-structured-binding-implementation-review.md#open-reliability-issue--windows-clipboard-checklist).
Do not weaken tests or redesign the clipboard speculatively. Existing WSLg startup,
mounted-filesystem Save uncertainty and historical Linux timing issues remain separate.

Current workstream: **ADR-0023 four-stage feasibility accepted and closed; production-readiness architecture packet prepared, parent design approval pending; production implementation gated**.
[Production-readiness design packet and proposed gates](handoffs/2026-10-23-history-production-readiness-design.md).
Recommend validated current-root WB5 projection with full live/history ledger and
Redo preserved, atomic durable indexing, mutation-local records and explicit
witness/resource/reclamation contracts. SQLite is an evaluation candidate only.
No correction or runtime integration was implemented.
[Final feasibility acceptance and next design gate](handoffs/2026-10-23-disk-history-final-feasibility-review.md).
Historical-state native Save is a production integration blocker: the prepared deliverable
is one architecture packet covering that correction, durable indexing/startup,
mutation-local encoding, resource/filesystem/reclamation contracts and staged gates.
Save must preserve Redo. No correction, production coordinator, UI/protocol or container
implementation is authorized during this design phase.
[Stage 4 association, save/retention evidence and remaining limits](handoffs/2026-10-23-history-association-stage4.md).
Windows/Linux/Mac: 69 cases each (207 total), 2,652 original hashes; Rust suites
346/382, 347/383 and 347/383, with strict lint passing on all three.
[Native APFS evidence and independent verification](handoffs/2026-10-23-macos-history-association-validation.md)
confirm 1,092 evidence hashes, 166 build inputs and 462 copied source hashes
(461 frozen files plus the ignored GPT baseline). New exports of older historical
roots with later identity floors still reject; prepared exports survive later
Undo/Clear. No codec relaxation or production integration was made.
[Stage 3 acceptance and Stage 4 safeguards](handoffs/2026-10-23-history-durability-stage3-review.md).
[Stage 3 decision protocol, fault/restart evidence and limitations](handoffs/2026-10-23-history-durability-stage3.md).
366 scenarios passed across Windows/Linux, including parent-killed workers, ambiguous
outcomes, atomic branch/cursor decisions, corrupt-payload rejection and acknowledged
restart controls. Rust default/all-feature suites: Windows 346/380, Linux 347/381;
strict lint passes. Full export, research reconstruction and linear disk-ID lookup
remain costs; physical power-loss and general filesystem durability remain unproven. Stage 3 alone did not
establish file association.
[Stage 2 acceptance and Stage 3 safeguards](handoffs/2026-10-23-history-representation-stage2-review.md).
[Stage 2 comparison, evidence and bounded adaptive checkpoint experiment](handoffs/2026-10-23-history-representation-stage2.md).
100 baseline/final-policy runs passed; cache counts/capacities remain fixed from
1,000 to 10,000 edits. Windows 346/378 and Linux 347/379 Rust suites and strict lint
pass; 792 original sample hashes verified.
Checkpoint plus canonical-record deltas are accepted as the preferred Stage 3 candidate; fixed
intervals and the additional adaptive byte/work thresholds are experimental.
Stage 2 alone establishes neither durable transaction recovery nor production
container or Undo/Redo integration.
[Acceptance and Stage 2 resource-accounting qualifications](handoffs/2026-10-23-cold-history-stage1-review.md).
[Cold reconstruction, identity/capture proofs and validation](handoffs/2026-10-23-cold-history-restoration-stage1.md).
[Parent inventory acceptance and next gate](handoffs/2026-10-23-cold-history-inventory-review.md).
[Architecture approval and experiment authorization](handoffs/2026-10-23-tiered-history-design-review.md).
[Parent cold-identity clarification](handoffs/2026-10-23-cold-history-identity-review.md).
[Cold-state inventory and executable boundary probes](architecture/cold-history-state-inventory.md).
Windows/Linux probes distinguish portable UUIDs from process-local runtime NodeIds;
separate processes legitimately reuse the same numeric model/node pair. Portable
transfer is not a history capsule and rejects an older root with a later allocation
floor. A separate feature-only checkpoint adapter now reconstructs both roots from
bytes, leaving portable transfer unchanged. Windows/Linux same-session, fresh-process
and independent-copy proofs pass with exact content/provenance and zero-charge finish.
Default/all-feature Rust suites pass: Windows 346/377, Linux 347/378. The parent
approves durable logical IDs/reservations across restart, exact runtime NodeIds for
hot/cold activation in a live session, and coherent remapping in a fresh reopened
runtime namespace. Separate gates cover same-session eviction, restart logical
equivalence and simultaneous-copy isolation. No NodeId representation change or
persistent model-number ownership is authorized; the identity-contract hold is resolved.

The separate identity measurement completed 10,000 existing-cell replacements using
one identity, while create/restore/branch reached the unchanged floor of 8,192 after
8,191 new commits with one active cell. Both platforms finished at zero charge.
This reports the requested product scaling constraint, not retained disk history.
The parent accepts the inventory/probes and measurements. Identity-space scalability
is a separate prerequisite for removing experimental workbook limits; no quota increase,
recycling or rebasing is authorized. The three cold-restoration proofs above are
accepted as the semantic foundation for representation comparison. Portable transfer
remains unchanged. Durable fault/recovery and association remain later gates.
[Parent acceptance and superseding direction](handoffs/2026-10-23-undo-redo-feasibility-review.md),
[accepted executable foundation and evidence](handoffs/2026-10-23-undo-redo-feasibility.md),
[revised ADR-0023](adr/0023-document-transaction-history.md),
[review packet](handoffs/2026-10-23-undo-redo-design.md),
[bounded disk-history feasibility plan](architecture/disk-history-feasibility-plan.md).
The accepted feature-only foundation restores historical roots under fresh authority,
retains lifetime identity floors and isolates existing calculation captures, including
invalid/cyclic drafts. Its original receipt records Windows 346/367 and Linux 347/368
default/all-feature tests and ten zero-charge measurement runtimes; those historical
manifests remain intact beside the new Stage 1 evidence.

The parent declines 32 transitions/33 states and 16 MiB as product history limits.
These remain experimental measurements, not reasons to discard recoverable history
or reject otherwise valid edits. The revised target is effectively unlimited logical
history, bounded resident roots and extensible disk history, constrained by real
storage and explicit user retention/failure policy. Across-session history is an
intended capability. Keep transaction/content identities, fresh authority, monotonic
allocation floors and existing save/dirty/uncertain-installation contracts separate.

Next work: execute the authorized Stage 2 comparison of
checkpoints, canonical record deltas and shared persistent-state objects with commit
journaling, without choosing a container/database. The proposal covers cold restoration,
durable-before-publish transaction/cursor/ledger decisions, uncertain-write reconciliation,
saved-file association, copies/moves/replacement, explicit retention/failure policies
and resident eviction without history loss. No extensible store, durable journal,
recovery/association implementation or measurements are claimed. Additional safeguards
require separate create/Undo/branch identity-exhaustion measurements (completed as an
inventory finding), preserved journals when recovery is declined, and independent history
deletion/compilation decisions. Return experiment evidence for parent feasibility
review before production container selection or full history/session/desktop integration.
The Stage 1 16 MiB image/8 MiB ledger reservations are prototype bounds, not production
defaults. Stage 2 must separate serialized bytes, actual working buffers, required/
reserved scratch, reconstructed allocations and peak simultaneous ownership across
small/medium/formula-heavy/near-capacity cases. Preserve independent exact identity,
binding, diagnostic/provenance and invalidation oracles; output equality alone is insufficient.
Optimize Worksheet is separate; no JIT,
SIMD, format changes or irreversible history compaction begin here. ADR-0022 remains
accepted and closed.

Accepted preceding milestone: **Bounded Structured-Result and Presentation-Projection Proof — accepted; workstream closed**.
[Acceptance and scope closure](handoffs/2026-10-22-structured-results-projection-implementation-review.md),
[implementation and three-platform evidence](handoffs/2026-10-21-structured-results-projection-implementation.md).

Accepted preceding milestone: **Bounded Document-Owned Worksheet Text — accepted; workstream closed**.
[Acceptance and scope closure](handoffs/2026-10-21-worksheet-text-implementation-review.md),
[implementation/evidence](handoffs/2026-10-21-worksheet-text-implementation.md).

Accepted preceding milestone: **Native AOT Publish-and-Execute Validation — accepted; workstream closed**.
Windows x64, Linux x64 and native macOS ARM64 publish and execute without warnings.
Managed CI remains available; shared Windows/WSLg clipboard serialization is complete.
[Acceptance and next design scope](handoffs/2026-10-20-native-aot-review.md),
[implementation and evidence](handoffs/2026-10-20-native-aot-baseline.md).

Accepted milestone evidence: **bounded document-owned worksheet text**. ADR-0018 is accepted
with clarifications; implementation is in place. Final Windows/Linux full CI
(314/315 default Rust tests), strict all-feature lint/tests (330/331), and actual
Native AOT publish/execute pass without publish warnings. All eight desktop fields
and native peers pass with zero charge. Fresh native Mac/APFS managed CI, strict
all-feature lint, 315/331 default/all-feature Rust tests and Native AOT also pass
without warnings, product fixes or reruns. All 359 copied source hashes and 183
build/test inputs match; 438 returned files are present. Parent implementation
review is complete: accepted, including the escape-overhead rule;
the milestone is closed. Rust-owned classification,
reversible apostrophe/TSV escapes, strict numeric-consumer/SUM type errors, bounded
single-line text and WB4/private version cuts are approved. Rejected Open must
preserve the live workbook/revision/dirty state. The narrow nonnumeric marker
satisfied the design stop condition without a general value-system redesign.
Projections/connectors remain outside this closed scope.
[Implementation and evidence](handoffs/2026-10-21-worksheet-text-implementation.md),
[Mac APFS validation packet](handoffs/2026-10-21-macos-worksheet-text-instructions.md),
[accepted ADR-0018](adr/0018-document-owned-worksheet-text.md),
[review decision and stop conditions](handoffs/2026-10-20-worksheet-text-design-review.md).

Accepted preceding milestone: **Bounded Rectangular Clipboard, Formula Copy, WB3 and Desktop Acceptance Automation — accepted; workstream closed**.
[ADR-0017](adr/0017-rectangular-clipboard-and-formula-copy.md),
[implementation/evidence/optional inspection](handoffs/2026-10-20-rectangular-clipboard.md),
[fresh Mac/APFS packet](handoffs/2026-10-20-macos-rectangular-clipboard-instructions.md).
Rust now supplies bounded typed native copy, effective-binding scalar/SUM translation,
dollar-axis syntax, exact external TSV and one atomic paste within 256 slots.
Desktop clipboard transport highlights successful destinations; editor paste stays
text insertion. PIVOTWB3 replaces WB2 with explicit WB1/WB2 rejection, no legacy
reader/parser mode. Preserved rejected drafts remain non-executable until explicit
resubmission. Frozen hosted v1 and capacity limits remain unchanged.
Windows/Linux full CI (309/310 default Rust tests), strict all-feature lint and
325/326 all-feature tests pass. Final actual-window and C#–Rust boundary workflows
pass with zero-charge close for the clipboard foundation. The owner subsequently requested
[automated desktop acceptance](handoffs/2026-10-20-desktop-acceptance-automation.md):
`python scripts/ci.py --desktop` now covers the five groups, cross-process native
clipboard and real close choices. Plain inline-editor arrows commit/move; modified
arrows and formula-bar arrows retain text editing. Final Windows/WSLg acceptance
and all helper-process reports pass with zero charge; all 173 build/test input hashes
match. Fresh native Mac/APFS integrated desktop CI, strict lint and 310/326 default/
all-feature tests now pass, including cross-process UTI clipboard, all close peers,
arrow commits and zero-charge cleanup. All 339 copied source hashes matched before
this doc update; all 173 build/test inputs match, with 391 returned reports verified
by the transfer record. No Mac fixes or reruns. Routine manual replay is optional;
physical input/native pickers are not inferred. The parent review received 2026-10-20
accepts the implementation separately from the earlier design acceptance and closes
the milestone. [Implementation acceptance and scope closure](handoffs/2026-10-20-rectangular-clipboard-implementation-review.md).
[Parent review packet](handoffs/2026-10-20-rectangular-clipboard-review-packet.md),
[native Mac report](../artifacts/macos-validation-20261020T235001Z/REPORT.md).

Accepted preceding milestone: **Rectangular Worksheet Selection and Navigation — accepted; workstream closed**.
Anchor/active/extent live only in C# UI state. Drag and Shift extension, bounded
keyboard navigation, auto-scroll and active/region/inactive visuals are implemented.
Selection-only checks leave Rust revisions/dirty state unchanged and create no
computational selections or worksheet content. Editing/clipboard still target the
active cell only. [Interaction specification and checks](handoffs/2026-10-20-rectangular-selection.md),
[fresh APFS packet](handoffs/2026-10-20-macos-rectangular-selection-instructions.md).
Windows/Linux/native Mac full CI (290/291/291 default Rust tests) and actual-window
checks pass with zero-charge cleanup. Mac strict all-feature lint and 307 all-feature
tests pass. All 327 Mac copied source hashes and 168 reference build inputs match;
368 returned reports are transfer-verified without fixes/reruns. User-confirmed
manual tests pass on all three platforms. Rectangular cut/copy/paste is deliberately
not implemented; selection/navigation and active-cell operations are the current
scope. The parent review received 2026-10-20 accepts the implementation.
[Acceptance and next design scope](handoffs/2026-10-20-rectangular-selection-review.md).

Accepted milestone evidence: **Native AOT Publish-and-Execute Validation** on Windows
x64, Linux x64 and macOS ARM64. The parent direction review prioritizes a native
publish/run baseline, then worksheet text, then structured-result/projection design.
[Direction and bounded next scope](handoffs/2026-10-20-native-aot-next-scope.md).
Implementation is in place. Native Windows x64 and Linux x64 publish/run gates pass
without warnings, including all desktop/peer workflows and zero-charge shutdown.
Ordinary Windows/Linux full CI also passes. Fresh native Mac/APFS managed CI,
strict all-feature lint, 310/326 default/all-feature Rust tests and Native AOT
publish/run now pass without warnings or product fixes/gate reruns. Main and all
three native peers pass with zero charge. All 349 copied source hashes and 178
build/test inputs match; 426 returned files are present. Cross-platform evidence
was accepted by the parent review received 2026-10-20; the milestone is closed.
[Implementation and evidence](handoffs/2026-10-20-native-aot-baseline.md),
[Mac APFS validation instructions](handoffs/2026-10-20-macos-native-aot-instructions.md).
Retain ordinary CI and the separate Rust child; no installer, graph redesign or data feature work.
Dynamic producers/results/projections remain distinct, screen addressing belongs to
presentation, refresh stays separate from recalculation, and concrete `#` syntax
remains provisional. This direction does not freeze a grammar or start implementation.

The runner-enforced Windows/WSLg clipboard serialization follow-up is complete.
Managed and Native AOT suites share an exclusive lock; real cross-OS exclusion
checks pass in both directions. Details and evidence are in the Native AOT handoff.

Accepted preceding milestone: **Direct single-cell paste commit — accepted; workstream closed**.
Grid paste submits immediately through Rust and retains selection; editor/formula-bar
paste remains uncommitted text insertion. Invalid numbers/conflicts retain a
correctable draft; late/cancelled clipboard reads do not overwrite newer state.
[Implementation and checks](handoffs/2026-10-20-direct-single-cell-paste.md),
[fresh APFS packet](handoffs/2026-10-20-macos-direct-paste-instructions.md).
Windows/Linux/native Mac full CI (290/291/291 default Rust tests) and actual-window
checks pass with zero-charge cleanup. Mac strict all-feature lint and 307 all-feature
tests pass. All 324 copied Mac source hashes and 168 build inputs match; 367 returned
reports are transfer-verified, with no fixes/reruns. The user confirms manual
functionality across all three platforms works as intended. Broader paste features
remain future scope. The parent review received 2026-10-20 accepts this correction.
[Acceptance and next scope](handoffs/2026-10-20-direct-paste-review.md). No engine,
protocol, WB2 or capacity changes are included.

The accepted review selected **Rectangular Worksheet Selection and Navigation** next.
The user authorized implementation; see current work above. Multi-cell clipboard,
copy/fill and reference translation remain outside this scope.

Accepted preceding milestone: **In-place worksheet editing — accepted; workstream closed**.
[Review packet](handoffs/2026-10-20-in-place-editing-review-packet.md),
[implementation and checks](handoffs/2026-10-20-in-place-editing.md).
The editor, synchronized formula bar, clipboard commands and focus/caret feedback
are implemented. Windows/WSLg window checks and the fresh native Mac automated
run pass; Mac has 291 default / 307 all-feature tests, 320 matching source hashes,
168 matching build inputs and 366 verified returned reports, without fixes/reruns.
The user subsequently confirmed manual acceptance of clipboard key presses:
copy/paste shortcuts work. The earlier failed outcome and the report’s original
pending status remain historical evidence; clipboard usability is not yet complete.

The parent review received 2026-10-20 accepts the editor/clipboard/focus milestone.
[Acceptance and next scope](handoffs/2026-10-20-in-place-editing-review.md).
The recommended next assignment is direct single-cell paste commit: selected-grid
paste submits one checked Rust edit immediately; active-editor paste remains text
insertion. No engine, protocol, WB2 or capacity changes are requested. The user subsequently authorized implementation; see the current-work entry above.

Accepted preceding milestone: **First-Class Ranges and SUM — Slice C accepted; workstream closed**.
[Implementation and exact quotas](handoffs/2026-10-20-ranges-and-sum-slice-c.md),
[validation evidence](handoffs/2026-10-20-ranges-validation.md),
[native APFS packet](handoffs/2026-10-20-macos-ranges-instructions.md).
Canonical sparse selections, row-major checked SUM and PIVOTWB2 are implemented.
WB1 is explicitly unsupported; frozen hosted v1 rejects selection capability.
Windows manual native-dialog checks passed per the user. Native macOS APFS full
validation and all six user-confirmed manual checklist groups pass. Full
Windows/Linux/macOS CI (290/291/291 default, 306/307/307 all-feature), strict lint
and actual-window checks pass with zero-charge cleanup. All 318 Mac copied source
hashes and 168 build inputs match; no product fixes or validation reruns were needed.
[Native report](../artifacts/macos-validation-20261020T130207Z/REPORT.md).
The user confirmed WSLg Save-on-close passes on local WSL2 storage and an untouched
workbook closes without prompting. WSLg Windows-volume Save-on-close reported
installation uncertainty; that mounted-path limitation and the historical native
abort remain unresolved.
The parent review received 2026-10-20 accepts the implementation separately from
the ADR-0016 design. [Acceptance record](handoffs/2026-10-20-ranges-and-sum-review.md).
The user authorized focused in-place worksheet editing after discussing scope.

Accepted preceding milestone: **Native Workbook Persistence and Application Lifecycle — Slice B**.
[ADR-0015](adr/0015-native-workbook-documents.md) and the
[implementation handoff](handoffs/2026-10-20-workbook-persistence-slice-b.md) describe
native workbook files, validated stable-identity restoration, safe staged Save,
external conflicts, exact saved-generation acknowledgement and Avalonia file actions.
The document codec is separate from the storage-independent workbook; the canonical
engine, product limits and frozen hosted format remain intact. Windows/Linux/native macOS full CI passes (280/281/281 default Rust tests,
296/297/297 all-feature), with .NET/parity/real IPC and actual Save/Open window
checks, zero-charge normal close and matching source hashes. The fresh native
APFS run needed no fixes or reruns; all 305 copied-file hashes match. Native file
pickers and dirty-close dialog choices remain untested manually. Slice B and
ADR-0015 are accepted by the parent review received 2026-10-20. The next assignment
is First-Class Ranges and SUM — Spreadsheet Semantics Slice C, beginning with an
explicit range-semantics decision before permanent representation.
[Review decision and next scope](handoffs/2026-10-20-workbook-persistence-review.md).
[Native report](../artifacts/macos-validation-20261020T111827Z/REPORT.md).

Accepted preceding milestone: **Runnable Application — C# Session and Worksheet Slice A**.
A real Avalonia worksheet edits an owned Rust Workbook through asynchronous framed
Protobuf pipes. Numeric/formula entry, clear/move, bounded scrolling, readable draft
errors, exact revision/generation tracking and orderly session disposal are implemented.
The backend retains production limits; no C# evaluator, file container or new engine
is introduced. [ADR-0014](adr/0014-application-session-and-worksheet.md),
[run/verification handoff](handoffs/2026-10-19-application-worksheet-slice-a.md),
[native APFS packet](handoffs/2026-10-19-macos-application-slice-a-instructions.md).
Windows/isolated Linux full CI passes (275/276 default Rust tests, 291/292 all-feature),
.NET Release and real boundary tests. Final application checks include queue-byte
admission, exact input, stale-display conflicts, viewport bounds, cancellation and
zero-charge disposal. Both final Windows/WSLg window checks and image inspection
pass. The initial WSLg native startup abort remains unresolved; no unconditional
Linux startup-reliability claim is made. Boundary p95 edit/viewport/selected-chain
is 0.130/0.308/0.281 ms Windows and 0.387/0.907/0.326 ms Linux; UI refresh includes
35 ms coalescing. See the handoff for actual timings and limits.

Run `python scripts/run_desktop.py` from the repository root. This is a volatile,
unsaved worksheet; closing the application discards it. Native macOS full CI and
actual Avalonia execution now pass from a source-matched internal APFS copy, with
276 default / 292 all-feature Rust tests, six .NET projects and zero final Rust charge.
No fixes or reruns were needed. Boundary p95 edit/viewport/calculation is
0.117/0.220/0.119 ms; eight UI observations span 39.45–108.56 ms.
[Native report](../artifacts/macos-validation-20261020T101213Z/REPORT.md).
Slice A and ADR-0014 are accepted by the parent review received 2026-10-20.
Next assignment: Native Workbook Persistence and Application Lifecycle — Slice B.
See the [review decision and scope](handoffs/2026-10-20-application-slice-a-review.md).

Accepted preceding milestone: **Canonical Engine Consolidation** implements the owner direction
in [ADR-0013](adr/0013-canonical-engine-consolidation.md). WorkingModel and Workbook
now always use persistent storage. Convenience constructors share the lazy runtime;
new_in retains application ownership. Reference backends/constructors, activation,
selector enums/features and obsolete comparison prototypes are removed. Host,
document/storage and daemon paths use the same canonical graph; codecs and product
caps are unchanged. See [architecture](architecture/canonical-engine-consolidation.md),
[evidence](performance/canonical-engine-consolidation.md) and
[review handoff](handoffs/2026-10-19-canonical-engine-consolidation.md).

Windows/isolated WSL2 Linux/native macOS CI passes 275/276/276 default tests and
291/292/292 all-feature tests, strict lint, .NET/parity and actual IPC. All 99 matrix children pass correctness,
resource guards and zero final charges. Existing construction/bulk targets pass;
one Linux 1K-start fan-out viewport p95 misses 5 ms at 5.343 ms. Its single matched
repeat passes at 3.381 ms; preserve the original miss as a review limitation.
The [fresh native APFS report](../artifacts/macos-validation-20261020T015824Z/REPORT.md)
passes 33 cases and all native targets, with all 280 copied source hashes matching
before documentation edits and no fixes/reruns. All eight pressure samples were
normal; completed-child high-water is not live RSS guarding. Including the Linux
diagnostic repeat, all 100 children finish with zero domain charge.
The parent accepts consolidation and ADR-0013. The graph migration workstream is
closed; the recorded viewport exception does not block the application slice.

Historical readiness milestone (constructor policy superseded by ADR-0013):
Previous milestone: **Persistent Workbook Production Readiness and Construction Profiling**
is implemented with Windows/Linux/native macOS validation complete. A bounded private-index insertion optimization reuses uniquely
owned candidate nodes while preserving shared roots, admission and atomic rollback.
Production Workbook remains reference; explicit new_in remains the application-owned
runtime path. ADR-0012 is Accepted. See the
[readiness architecture/compatibility matrix](architecture/persistent-workbook-readiness.md),
[matched profile/evidence](performance/persistent-workbook-readiness.md),
[handoff](handoffs/2026-10-19-persistent-workbook-readiness.md) and
[fresh native APFS packet](handoffs/2026-10-19-macos-persistent-workbook-readiness-instructions.md).
Full default CI passes (251/252/252 tests), all-feature regression passes 300/301/301,
fallback 21 each and workbook-only 49 each. All 234 matched ordinary/profile/core
children pass with identical 68 source hashes and zero final charges. At 100K,
workbook construction improves 1.7–2.1x Windows / 1.5–1.9x Linux / 1.44–1.68x macOS; numeric requested
traffic falls from 1.492 GB to 651 MB. Retained/peak memory is substantially unchanged.
Core name/page work explains much of the remaining reference gap. Existing interactive
and bulk investigation budgets pass. Recommend explicit new_in for the next bounded
application/session slice after review; keep new reference.
The [fresh native report](../artifacts/macos-validation-20261020T010244Z/REPORT.md)
validates all 296 copied source hashes on internal APFS. All 78 native cases pass;
52 pressure samples were normal and completed-child high-water peaked at 1,418.8 MiB.
This does not establish a capacity increase; native live RSS guarding remains unavailable.
The preceding foundation run remains historical. Stop for review before constructor changes.

Preceding accepted milestone: **Persistent Workbook Integration and Bulk Mutation Foundation**
is implemented behind `pivot-workbook/persistent-core`; production Workbook remains
reference. Canonical core batches reserve identities/resources, bind final names,
publish atomically and dispatch once. Persistent workbook edits retain sparse
stable identity and pinned S/T metadata with bounded rectangle reads. See
[ADR-0012](adr/0012-bulk-mutation-and-persistent-workbook.md),
[architecture](architecture/persistent-workbook-bulk.md),
[evidence](performance/persistent-workbook-bulk.md) and
[handoff](handoffs/2026-10-19-persistent-workbook-bulk.md).
Full default CI passes on Windows/Linux/native macOS (248/249/249 Rust tests),
with all-feature regression 296/297/297, fallback 21 each and workbook-only 48 each.
All 335 Release/diagnostic children pass with zero final domain charges; native
validation used a fresh internal APFS copy and all 290 source hashes matched before
this documentation update. Core batching improves roughly 5–9x Windows / 21–25x
Linux / 4.5–5.3x macOS over individual persistent edits. Native 100K workbook
population is 1.14–1.40 s, still slower than optimized reference. The
[native report](../artifacts/macos-validation-20261019T220845Z/REPORT.md) records
warning-level memory pressure and sampled RSS only, not native child high-water.
Foundation accepted by the parent; production workbook selection stays reference.

The preceding **Standalone Default Transition** is accepted.
Default-feature standalone WorkingModel::new now starts directly on persistent
state with a shared convenience runtime; application/session owners can use new_in
and an explicit runtime with aggregate admission and deterministic shutdown.
Reference fallback is explicit. Host, storage, workbook, legacy import and daemon
constructors deliberately retain their reference paths; frozen formats and limits
are unchanged. That transition is accepted. See
[ADR-0011](adr/0011-standalone-persistent-default.md),
[constructor/lifecycle policy](architecture/standalone-default-transition.md) and
[handoff](handoffs/2026-10-19-standalone-default-transition.md).
Native macOS transition validation also passed from a fresh APFS copy. At transition review,
Windows/Linux/macOS default counts were 241/242/242 and combined counts 264/265/265;
all ninety transition Release processes passed. Parent acceptance is recorded in ADR-0011. That earlier native run predates bulk; current bulk validation is recorded above.

The Rust workspace and .NET 10/Avalonia solution are scaffolded. Architecture
records live in `docs/adr/` using `ADR-NNNN` identifiers. ADR-0001 records the
previously agreed graph-first design.

`pivot-types` provides `PivotDecimal`, a private `rust_decimal` wrapper with
decimal text parsing, integer construction, formatting, comparisons, and checked
addition, subtraction, multiplication, division, and negation. Arithmetic errors
distinguish overflow and division by zero. Unrepresentable literals are rejected.
No float conversion API is exposed. Arithmetic outputs are canonicalized; explicit
rounding supports seven modes with nearest/ties-away-from-zero as default. .NET
helpers expose the same behavior. A shared 10,099-vector suite passes for values,
errors, and scales, including seeded arithmetic cases and independent rounding
expectations. See `tests/decimal-parity/verify.py` and the numeric architecture note.

`pivot-core::expression` parses decimal literals, ASCII identifiers, unary signs,
binary arithmetic, and parentheses into an AST. Multiplication/division precede
addition/subtraction; binary operators associate left. Names are case-preserving
and unresolved. The AST has no evaluation or I/O behavior.

Editable models now support stable node IDs, named decimal inputs, computed nodes,
and immutable snapshots with captured inputs and model-scoped revisions. Snapshot
validation checks bound IDs, extracts forward/reverse dependencies, orders nodes, and
rejects cycles. See ADR-0002 and the model/snapshot architecture note.

Snapshot evaluation now supports selected outputs or all nodes, bound decimal
expressions, calculation-local state, typed arithmetic/dependency errors, and
immutable results retaining their snapshot revision. Isolated invocation accepts
temporary input overrides keyed by NodeId, with captured defaults and typed binding
validation. Independent invocations share only immutable snapshot definitions.
Independent arithmetic failures do not prevent other outputs from being computed.
Forty-four Rust tests pass on Windows and Ubuntu WSL2 Linux for the command-REPL
slice. macOS ARM64 last passed 41 tests before this slice; the REPL changes require
a fresh Mac run. Public model ports now support named invocation,
required inputs, explicit optional defaults, and stable port identities. Formula
names bind only at edit submission; snapshots reuse stable-ID references.
Public values have extensible tagged value/type boundaries (Decimal-only today).

Invalidation, IEEE-754 and other non-decimal
values remain deferred. A demo daemon and minimal CLI now use named pipes on
Windows and Unix-domain sockets on Unix (Linux WSL2 and macOS ARM64 verified). ADR-0004 is accepted after the review
refinements. The portable contract has generated Rust/C# codecs, numeric provenance
validation, explicit input sources, negotiated limits, and an in-process pivot-host
registry/dispatcher with mandatory handshake sessions. ADR-0005 is Accepted;
local IPC includes current-user peer checks, bounded framing, connection limits,
and a conservative calculation work budget. Idle/partial-read/write deadlines,
connection caps, and work budgets are configurable operational policy. A persistent
command REPL shares the actual IPC client and supports session-local history,
line editing, explicit reconnect, and no automatic replay after disconnect.
Minimal named-model authoring and selected-node evaluation are available through
typed daemon requests; general scripting remains deferred.
Local CI is available through `python scripts/ci.py`, covering Rust formatting,
lint/tests, .NET Release builds, and cross-language decimal compatibility. See
`docs/local-ci.md` for prerequisites and current platform/reproducibility limits.
The complete CI pipeline now passes in Ubuntu WSL2 with .NET SDK 10.0.112,
including both cross-language suites and the .NET Release solution build.
The complete CI pipeline also passes on macOS 27.0.1 ARM64 with .NET SDK
10.0.402 using an isolated local APFS source/build copy. See
`docs/platform-validation.md` for reports and the SMB build limitation.
Native AOT, GUI execution, and actual cross-user endpoint denial remain unverified.

The editable working-model service is now designed in accepted ADR-0006 and
`docs/architecture/working-model-service.md`: connection-independent resources,
atomic expected-revision edit batches, draft validation, unpublished evaluation,
publication boundaries, and operation-receipt reconciliation. Slice A is implemented
in `pivot_host::management`: memory-only resources, portable identities, authorized
discovery/inspection and attachments, explicit quotas, and replay-safe creation.
Slice B adds private transactional staging, atomic edit batches, one public revision
per batch, portable node/port IDs, revision-pinned raw draft reads and diagnostics,
final-directory binding for submitted formulas, and retained edit receipts.
Slice C adds exact-revision structural validation, detached immutable evaluation
captures with temporary overrides, and replay-safe publication into a coordinated
immutable invocation registry. Published revisions and working revisions remain
distinct. Serialized authoring is available in process and through opt-in local
daemon IPC and the minimal authoring REPL. See
`docs/architecture/working-model-management.md` for the in-process API and limits.
Transactional editing details are in `docs/architecture/working-model-editing.md`.
Calculation/publication details are in
`docs/architecture/working-model-calculation-publication.md`. Persistence, identity
compaction and richer edit diagnostics remain follow-ups; memory-only commits do
not survive service shutdown. The latest native macOS CI now covers the REPL; Native AOT remains
on the verification backlog.

ADR-0007 is Accepted. Production authoring schemas and shared types, Rust/C# codecs,
bounded recursive mutation preflight, typed intent and opt-in v1.1 negotiation are
implemented and compatibility-tested. The daemon defaults to invocation-only;
`--authoring` explicitly enables v1.1 authoring. Stage A implements all twelve
serialized authoring operations in process, signed bounded paging, session-owned
attachment cleanup, staged partial-port patches and exact wire-intent replay.
Evaluation runs outside the service lock and delivery rechecks authorization.
Detailed receipt diagnostic retention remains deferred. See
`docs/architecture/serialized-authoring-adapter.md` and
`docs/architecture/authoring-protocol-foundation.md`. Stage B integrates actual
local IPC, OS-derived principals, one demo/authored publication registry and
blocking dispatch with no service lock across transport writes. Five new IPC
tests cover daemon authoring, legacy compatibility, lost responses, concurrent
edits, denial/revocation, cancellation cleanup, quotas and malicious traffic.
See `docs/architecture/daemon-authoring.md` and the Stage B handoff for current
Windows/WSL validation; the latest macOS CI also passes these retained tests.
The Stage B Rust suite had 130 tests, including 23 adapter/router tests and five
daemon IPC integration tests. Full Windows and WSL CI reports are linked from
`docs/handoffs/2026-10-18-daemon-authoring.md`.
The subsequent minimal authoring REPL adds create/list/select/inspect, decimal
inputs, opaque formula submissions/repairs, public ports, validation, evaluation
and exact-revision publication. It preserves pending operation IDs/intent after
ambiguous responses and requires explicit recovery/retry; conflicts never trigger
automatic refresh. Creating/opening a model warns that restart loses its state.
Five executable REPL tests bring the Rust suite to 135 tests. The real-process
IPC harness also checks authoring, reopening and immutable published invocation.
See `docs/architecture/command-repl.md` and
`docs/handoffs/2026-10-18-minimal-repl-authoring.md` for evidence and limits.
Optional hosted persistence direction is a common transactional contract with configurable
providers: SQLite first/default locally with a filesystem path, and planned
PostgreSQL/MySQL adapters. Storage choice is independent of local/network service
mode. Initially one active service owns a store; multi-instance coordination remains
separate work. See `docs/architecture/persistence-providers.md`. Provider support,
schemas and recovery are not implemented; runtime durability remains MemoryOnly.

ADR-0008 is Accepted for optional durable hosted storage with the required review refinements: database-authoritative domain transactions, portable bound
expressions/exact values, durable identities and receipt scopes, SQLite ownership
and durability profile, recovery/failure matrix and shared provider conformance.
See `docs/handoffs/2026-10-18-transactional-persistence-design.md` for the review
packet. Durable wire semantics require a separately reviewed capability extension.
The first implementation slice now adds owned portable graph/publication records,
bounded deterministic-CBOR v1 encoding/decoding, a validated no-rebinding core import
boundary, retired identities, exact decimals and coherent collection restoration.
There are 11 frozen codec fixtures and 18 new tests (153 Rust workspace tests total).
The running daemon/REPL remain MemoryOnly; no database/provider, migration, durable
coordinator, wire fields or cross-restart replay is implemented. See
`docs/architecture/portable-storage-foundation.md` and
`docs/handoffs/2026-10-18-portable-storage-foundation.md`.

## Accepted document foundation

The parent review accepts the ADR-0008 serialization/restoration foundation and
preserves ADR-0006/0007/0008 history. Further hosted persistence is deferred:
SQLite ownership feasibility, schema/migrations, durable commit coordination,
cross-restart receipts and PostgreSQL/MySQL adapters are not the next work.

[Accepted ADR-0009](adr/0009-portable-documents-and-local-lifecycle.md) defines
ordinary model documents and local New/Open/Edit/Calculate/Save/Save As/Close
behavior. Document Foundation Slice A now implements hosting-free graph payloads
and thin one-graph in-memory sessions in pivot-storage. Sessions reuse core staging,
binding, validated import and evaluation, with no hosted registration, publication,
store/epoch identities or receipts. Immutable export/restore, independent copies,
exact values, invalid drafts, stale-token checks and saved checkpoints are tested.
See [implementation details](architecture/document-foundation.md) and the
[Slice A review handoff](handoffs/2026-10-18-document-foundation-slice-a.md).

The Slice A suite had 167 Rust tests, including 13 new document integration tests and
one new hosted resource-budget compatibility test. Full Windows CI and WSL Linux
Rust/lint/IPC checks passed at submission; the latest macOS CI also covers these tests.
Document Foundation Slice B now adds an explicitly experimental versioned envelope
and a headless Rust file broker for New/Open/Save/Save As/Close. It stages complete
private candidates, checks observed target fingerprints, installs without truncation,
verifies installed content and only then acknowledges the exact export. Fault tests
cover conflicts, failed/uncertain saves, reconciliation and S/T dirty-state behavior.
Independent-process tests create, reopen, calculate/invoke, edit/save, copy and reopen
models without hosting. The core and frozen hosted storage-v1 fixtures are unchanged.
See the [experimental envelope](architecture/experimental-document-format.md),
[broker and limits](architecture/experimental-file-broker.md) and
[Slice B handoff](handoffs/2026-10-19-document-foundation-slice-b.md).

Slice B passes full Windows CI (185 Rust tests) and available WSL Linux checks
(186 tests, including Unix-only permissions/alias coverage) plus actual IPC/REPL.
File tests now cover NTFS, native Linux disk-backed storage and local macOS APFS.

No production .pivot format, SQLite/ZIP container, document RPC, C# broker, worksheets
or UI is implemented. Process termination checks do not prove power-loss durability.
The latest native macOS suite passes the retained file tests. The hosted storage-v1 record is
not the native document format.

## Workbook projection and current performance gate

The parent review accepts ADR-0010's structural architecture while reserving
spreadsheet compatibility semantics. [ADR-0010](adr/0010-workbook-projection-and-spreadsheet-semantics.md)
now distinguishes accepted boundaries from future cut/copy/delete, range affinity,
blank/text/aggregate and shared-writable-node policies. No Excel compatibility claim
or production workbook format is made.

Sparse Worksheet and Graph Projection Slice A is implemented in the dedicated
pivot-workbook crate. It has one sparse worksheet/graph, independent cell/axis/node
identities, exact decimal input metadata, typed stable references, clear/move/retire,
atomic bounded edit batches, no-op detection, diagnostics, immutable calculation
captures and basic bounded rectangular reads. The trusted resolved-token seam in
the core transfer module shares existing token/dependency validation and evaluator;
it does not parse fabricated identifiers or introduce a second expression engine.

See [implementation and small costs](architecture/workbook-projection-foundation.md)
and [Slice A handoff](handoffs/2026-10-19-workbook-projection-slice-a.md).
Full Windows CI passes 200 Rust tests, .NET/parity and actual IPC/REPL regressions.
Available WSL Linux checks passed 201 tests and real IPC at Slice A submission.
The later Slice B macOS run covers these regressions too. Tiny debug-profile
measurements at 17/129/513 live
cells are not spreadsheet-scale claims; full-copy graph staging/snapshot rebuilds
and logical small-model budgets remain explicit.

Spreadsheet Scalar Formula Parsing and Binding Slice B is implemented. The existing
Rust parser has an explicit spreadsheet mode for optional `=`, same-sheet A1
references, decimal arithmetic, unary signs and parentheses. Text formula edits bind
once to stable cell/node targets and preserve original source, reference spans and
transaction/candidate provenance. Malformed/out-of-bounds drafts are diagnosable and
non-executable; hard failures roll back the batch. Moves and coordinate reuse do
not rebind formulas; explicit resubmission does.

See [grammar and implementation](architecture/spreadsheet-scalar-formulas.md) and
[Slice B handoff](handoffs/2026-10-19-spreadsheet-formulas-slice-b.md) for current
verification and costs. Full Windows CI passes 209 Rust tests plus .NET/parity/IPC;
available WSL2 checks pass 210 Rust tests plus IPC. Native macOS ARM64 now passes
full CI with 210 Rust tests, .NET Release, both parity suites and actual IPC/REPL.
At Slice B validation, the working tree matched all 215 Mac source hashes;
builds and .NET outputs stayed on local APFS, with reports copied back to SMB.
See the [macOS review](handoffs/2026-10-19-macos-validation.md).
The parent review accepted Slice B and replaced the following C# priority with a
mandatory graph/workbook performance-feasibility gate. The opt-in research harness
now compares optimized baseline and shared-topology/input-overlay behavior at
1K/10K/100K cells; see the [feasibility assessment](architecture/graph-workbook-scaling-feasibility.md)
and [research handoff](handoffs/2026-10-19-scaling-feasibility.md). Its recommendation
is B: viable with architectural restructuring, not production adoption of the
prototype. Default behavior and small-model production limits remain unchanged.
The parent accepted assessment B and authorized the bounded Structural Mutation
Feasibility slice. A separate opt-in paged core and fixed-projection workbook
adapter now test localized formula/dependency edits and coherent immutable captures.
See the [representation comparison](architecture/structural-mutation-feasibility.md)
and [structural handoff](handoffs/2026-10-19-structural-mutation-feasibility.md).
Matched end-to-end timings include the baseline's deferred validation. Baseline
and prototype memory workloads run in separate guarded processes. No production
replacement is adopted. The parent accepted the structural research and authorized
Production Graph Representation - Compatibility Foundation. That opt-in slice now
uses persistent name/reverse-edge indexes, supports ports/invocation and invalid
restoration, shares accepted binding/validation rules, charges unique core storage
and candidate work, and provides bounded off-thread reclamation.
See the [compatibility design](architecture/persistent-core-compatibility-foundation.md)
and [foundation handoff](handoffs/2026-10-19-compatibility-foundation.md) for matched
Release results, 32-generation retention, 1M repeats and remaining blockers.
That foundation recommended focused integration work on revisions/diagnostics,
allocation ownership, ports and workbook coverage. The parent accepted it and
requested Controlled Production Core Integration Stage 1, now implemented through
WorkingModel as described at the top of this status. Review its explicit API and
scheduling gaps before another migration slice. C# work stays held pending Stage 1
review. Relative/absolute
copy/fill, cross-sheet references, row/column structural edits, ranges/SUM, typed
blank/text engine values, incremental calculation, workbook files, C# IPC and
Avalonia rendering remain unimplemented.

Keep production file ACL/alternate-stream policy, final-check writer races and
power-loss guarantees separate from this worksheet proof. GUI execution, Native AOT
and actual cross-user denial remain unverified.
Do not expand the experimental file broker or resume optional hosted persistence
automatically. Production container selection remains a later workbook-driven
decision. Native AOT, native macOS corroboration of this new experiment and
production-scale validation remain backlog; the earlier Slice B Mac pass is retained
as historical evidence. Stop after the controlled-integration Stage 1 review, without new features.


Native macOS Stage 2 validation (20261019T182116Z) passed on Apple M1 ARM64 from
an isolated internal APFS copy: 210 default, 237 integration-only, 234 research-only
and 260 combined tests, three strict feature lint configurations, complete .NET/
parity/real IPC CI and all thirty Release benchmark processes. All copied source
hashes matched before this documentation update; no product changes were needed.
All review targets passed and final domain charges were zero. The runner does not
support macOS RSS; separate sparse native samples do not establish child high-water
marks. See the [native report](../artifacts/macos-validation-20261019T182116Z/REPORT.md)
and [Stage 2 performance addendum](performance/controlled-core-integration-stage2.md).

Standalone transition final Windows/Linux results: full default CI passed (241/242 tests), combined regression passed 264/265, reference-only core passed 21 on both, all strict lint configurations passed and all sixty verified Release children passed the interactive targets. Native macOS transition subsequently passed (242 default, 265 combined and 21 fallback tests, full CI and thirty more Release children); see the transition handoff for current three-platform results and the retained bulk-construction tradeoff.


Native standalone transition follow-up (20261019T191136Z): all requested gates
passed on internal APFS, with no product changes or reruns. Full CI included .NET,
10,099 decimal vectors, protocol compatibility and actual Unix-socket IPC; all
thirty native benchmark children passed with zero convenience/session final domain
charges. Native 100K p95 input/capture was 0.013 ms, input/chain 0.331 ms, tail
formula 0.491 ms and bounded inspection 0.007 ms. Bulk fixture construction still
cost 1.300 seconds versus 0.0467 reference; full snapshot 48.748 ms versus 30.313
reference. Native RSS guard/high-water support remains unavailable; separate
monitoring is sampled evidence only. All three-platform validation gates are now
satisfied for parent review; no downstream backend migration is included.

Parent implementation review: [computational implementation acceptable; final
acceptance deferred pending WB5-only correction and fresh three-platform gates](handoffs/2026-10-22-spreadsheet-aggregate-implementation-review.md).
