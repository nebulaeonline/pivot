# ADR-0022 — Four-direction Fill implementation

Date: 2026-10-23. **Implemented; final Windows/Linux/native Mac/APFS validation passed;
parent implementation acceptance received; ADR-0022 closed.**
[Approved design](../adr/0022-four-direction-rectangular-fill.md),
[governing clarifications](2026-10-23-four-direction-fill-design-review.md),
[architecture](../architecture/rectangular-fill.md).
ADR-0021 remains accepted/closed. No Undo/Redo implementation is included.

## Result and boundaries

Fill Up/Down/Left/Right uses the bottom/top/right/left edge, independent of selection
orientation, capturing every source from one original generation. Existing trusted
duplication now shares typed planning; there is one cancel-aware transaction, never
repeated paste/duplicate commits. Blank/Decimal96/text/rejected drafts retain their
classification; scalar and all five aggregate references use authoritative effective
coordinates and dollar flags. Source output bindings retain identity under rename.
Projected records stay outside logical worksheet coordinates. OS clipboard unchanged.

Destination comparison precedes range/node allocation. Repeating identical Fill
preserves definitions, CellIds, SelectionIds, graph revision, checkpoint and dirty
state. Blank Fill into a populated cell retains its CellId; absent blank remains
unallocated. COUNT/AVERAGE observe explicit zero becoming Blank while scalar reads
still return zero. Old captures retain their original values.

Real selected area is bounded at 256 including source edge; zero-destination shapes
are validated no-ops, even for a large axis. Source/text 128 KiB, tokens 16,384 and
references 8,192 retain conservative duplication budgets. Shared scratch is admitted
through the allocation domain before planning; no quotas/capacities increase.
Late invalid translation, cycles, live/structured quotas, planning-work admission,
stale expected state and cancellation reject without partial live installation.

A cycle regression exposed that existing general authoring deliberately accepts
cycles as drafts. Fill requires rejection: a new narrow canonical cycle-only
validator walks the staged dependency graph, including computational selection
members, before installation. It evaluates no values or captured outputs and does
not change ordinary entry/paste draft rules or accepted aggregate arithmetic.
All transient owners retain existing deferred reclamation behavior. Tests drain
noninteractive cleanup before asserting reclaimed structured-version quotas; immediate
authoritative roots and ledgers remain unchanged on failure.

Private v4 now carries typed FILL31/direction field28, selection and exact expected
revision/computational state. Unknown direction rejects. The client preserves its
existing lifecycle: pre-dispatch cancellation changes nothing; post-dispatch caller
cancellation drains the response and reconciles an installed Fill, without replay.
WB5 and CB2 are unchanged; hosted v1 remains frozen. Actual pre-Fill v4 Windows
binary probe: `artifacts/fill/old-v4-opcode-probe.json`, error `unknown operation`,
revision/value preserved and zero-charge close. Preserved binary SHA256:
`e76e7c02f41c33c38b94eda8ac9d216f54f11722f067160e9b4e2f81f04fad35`.
This is supplementary Windows old-peer evidence, not a Mac executable dependency.

Accessible Fill submenu and grid-scoped primary+D/Shift+D/R/Shift+R work in all
four directions. Menu items are disabled during editing/pending work/drag/projected
selection; shortcuts do not intercept editors/unrelated controls. Fill preserves
selection geometry/orientation, active cell and viewport, restoring grid focus.
A pending draft is never implicitly committed or discarded. Rejection is visible
in the footer; no partial change or selection retarget occurs.

## Tests and evidence

New focused Rust coverage: six integration tests plus one internal deterministic
planning-cancellation/order test. Cover four directions/all aggregate kinds, no-op
identity/export/usage stability, negative translation, source independence, multi-cell
edges, signed/scaled zero, text and rejected drafts, moved references, blank/zero
classification, captures, scalar/range cycles, work/live/structured quota rollback,
stale state, area/degenerate bounds and pre-/mid-planning cancellation. Existing
clipboard/structured suites remain enabled. Real C#–Rust Fill checks include typed
request/repeat/identity, unknown direction, area rejection, cancellation before and
after dispatch, WB5 Save/Open and zero charge.

Actual desktop automation adds `rectangular_fill` to the existing eleven required
fields in managed and Native AOT gates. It routes menu clicks and all four platform
shortcuts, tests repeated no-ops, editor exclusion, original rectangle/scroll,
unchanged OS clipboard and WB5 Save/Open. Existing projection automation additionally
tests logical Fill across bands and rejection on a selected projected record.
These are routed events, not physical-key/native-picker/IME certification.

Development tests first exposed the deferred structured resource cleanup timing and
the preexisting cycle-draft behavior; neither was hidden by removing assertions.
The former now waits for explicit noninteractive reclamation; the latter gained
staged canonical cycle validation. Initial development desktop/session runs passed
before that final validator; final gates below must cover the resulting source.
No test/limit was weakened and no compatibility adapter was introduced.

Final Windows evidence: full managed/desktop CI
`artifacts/fill/windows-ci/20261023T120615326958Z/summary.json`;
strict lint/all-feature logs `artifacts/fill/windows-all-feature-lint.log` and
`windows-all-feature-tests.log`; actual Native AOT
`artifacts/fill/windows-native-aot/20261023T120722918341Z/summary.json`.
Windows Rust counts independently recount to 346 default/362 all-feature. Both
managed and native reports contain all twelve fields, correct modes, main/three
peers with zero charge, and no native publication warnings. Native `results.fill.png`
was inspected: visible Fill menu, selected B2:D4, row 2 translated values 2 and preserved
ordinary geometry; the blocked-action footer is intentional editor-guard test state.

The first full Windows CI `20261023T120308673576Z` and Linux CI
`20261023T120357065396Z` failed the newly added projection automation: it assumed
an offscreen ordinary row was present in the viewport cache. Corrected only that
assertion to read the logical cell through the real session; the check still requires
value 8, unchanged projection presentation and logical layout. Failed summaries and
logs remain. The runner's generic worksheet-text-missing message reflects that early
abort, not a diagnosed text or clipboard regression. Windows unchanged-runtime rerun
passes. Final Linux gates against the corrected test also pass.

Linux and native Mac final Rust counts are 347/363.
The 544-case rational oracle and 10,099 decimal-parity vectors remain unchanged.

Final Linux evidence under `artifacts/fill/linux-validation/`:
`managed-ci/20261023T120802940119Z/summary.json`,
`native-aot/20261023T121109674104Z/summary.json`, strict all-feature logs,
`provenance.json`, `build-input-verification.json` and `transfer-verification.json`.
Source/.NET outputs/publish/fixtures/original reports are on native ext4 under
`/var/tmp/pivot-fill-20261023`; the reused Rust target cache remains
`/mnt/j/coding/pivot/target/ranges-linux`. This retained Rust-cache exception is
not a claim that all Linux output is native. Mac requires all build trees on APFS.
All 211 compiled/test input hashes match native source/share; Windows independently
verified all 739 returned evidence SHA-256s. Native publish binaries/symbols stay on
ext4 and are inventoried in the native summary, excluded from report transfer.

| Platform | Default / all-feature Rust | Managed desktop | Native AOT |
| --- | --- | --- | --- |
| Windows x64 | 346 / 362 | Passed, twelve fields | Passed, twelve fields |
| Linux/WSLg x64 | 347 / 363 | Passed, twelve fields | Passed, twelve fields |
| Native macOS ARM64/APFS | 347 / 363 | Passed, twelve fields | Passed, twelve fields |

Final manifests: `artifacts/fill/build-inputs.json` freezes 211 build/test inputs;
`source-manifest.json` freezes 419 source files, including final documentation.
Compiled inputs did not change after final platform runs; documentation was finalized
after those runs. Original prior-milestone manifests remain untouched. Mac must
verify both current manifests before/after and retain original per-file evidence hashes.

## Completed native macOS validation

Run: [macOS report](../../artifacts/macos-validation-20261023T121442Z/REPORT.md),
`macos-validation-20261023T121442Z`. Managed summary:
`managed-ci/20261023T121457566510Z/summary.json`; native summary:
`native-aot/20261023T121712756293Z/summary.json` inside that run.
All four sequential gates passed on the first attempt: full managed/desktop CI,
strict all-feature lint, all-feature Rust tests and actual osx-arm64 Native AOT.
No setup failures, product changes or gate reruns were reported. Windows independently
recounted 347 default / 363 all-feature tests, with no failed or ignored tests.
All twelve desktop fields passed in managed and native reports. Main and three peers
in each mode report zero remaining charge, correct execution modes and orderly close;
clipboard-save peers confirm native transfer and Cancel coverage. No owned survivors
or native publication warnings remain. Fill session checks, 544 independent rational
cases, WB5 header/length/digest checks and accepted regressions pass.

Source, Cargo target, .NET bin/obj, native publish, temporary files and document
fixtures stayed on native internal APFS disk3s1 under
`/private/tmp/pivot-fill-20261023T121442Z._m7tuz32`. Environment and Mach-O/file/lipo
receipts confirm native ARM64 desktop and Rust child. The actual published desktop
ran with PIVOT_SESSION_PATH unset and its adjacent Rust child; native peers used
the same published image. Desktop size: 19,276,128 bytes; runtime payload: 41,485,248
bytes. Native startup-to-ready was 992.8456 ms, one instrumented observation rather
than a performance acceptance claim or physical display timing. Published binaries
and symbols remain on APFS, excluded from SMB transfer and inventoried in the summary.

Before these documentation edits, Windows independently matched all 419 frozen source
hashes and 211 build/test input hashes to the current tree. The Mac also verified
local/share source and inputs before/after without regenerating expectations.
Original manifests remain unchanged as tested-source provenance.
The APFS-original `evidence-sha256.json` retains individual hashes/sizes for 465
returned evidence files. Windows independently repeated all 465 comparisons, matched
the manifest to the receipt's APFS/SMB hash, and verified the receipt checksum.
The manifest and separate receipt/checksum metadata are excluded from the manifest's
own entries to avoid circular hashes. All report links resolve; no failed transfers
were reported. Review receipt: `artifacts/fill/macos-review-receipt.json`.

Windows inspected native `results.fill.png`: visible Fill menu, B2:D4 selection,
D4 active outline and row 2 translated values 2 match the existing geometry. The
blocked-action footer is expected editor-exclusion automation state. Exact values,
source translation, no-ops and editor/presentation safeguards are asserted by the
routed tests; no physical-key, IME, accessibility, clipboard-manager or native-picker
certification is inferred. No recurring manual checklist is requested.

Some returned boilerplate mentions the historical v1 clipboard UTI and includes a
prior manual-failure receipt. Those are not new test results: the explicit current
version evidence confirms PIVOTCB2/`org.pivot.cell-block-v2`, and all current gates
pass. Private v4, WB5-only handling and hosted v1 remain unchanged. The unresolved
Windows clipboard issue below remains open; this Mac pass does not diagnose it.

## Stop point and existing limits

Three-platform gates and source/transfer reconciliation are complete. The parent
accepted the implementation on 2026-10-23; ADR-0022 is closed.
[Final acceptance and scope closure](2026-10-23-four-direction-fill-final-review.md).
The [Mac packet](2026-10-23-macos-fill-instructions.md) is completed run provenance,
not a request to repeat validation. No recurring manual checklist is added.
No series/drag handles/history/formatting/new functions/structural edits/persistent
structured integration/capacity expansion. Undo/Redo follows as a separate design
only after Fill acceptance. The Windows ClipboardChecklistAsync null destination
remains unresolved. Earlier WSLg allocator startup failure, mounted-Windows Save
uncertainty and historical Linux viewport p95 limitation remain separate and open.

Completed Mac packet: [native APFS Fill validation](2026-10-23-macos-fill-instructions.md).
