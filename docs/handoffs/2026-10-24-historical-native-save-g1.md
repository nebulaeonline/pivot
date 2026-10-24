# G1 — historical-state native WB5 Save

Date: 2026-10-24. **Internal correction and its three-platform Rust validation
accepted. The separate ordinary Save promotion is now implemented; G1 closure
pending final three-platform managed/session/desktop/AOT gates. G2–G7 deferred.**
[Promotion implementation and current gates](2026-10-24-historical-save-g1-promotion.md).
This packet records the accepted internal source; the promotion packet supersedes
its experimental-only export call boundary below. Original manifests remain intact.
[Parent acceptance and bounded promotion assignment](2026-10-24-historical-save-g1-internal-review.md).
[Authorization](2026-10-24-history-readiness-g1-authorization.md),
[architecture](../architecture/history-production-readiness.md),
[ADR-0023](../adr/0023-document-transaction-history.md).

## Behavior and boundary

The feature-only history association can initiate a new Save of historical B after
D-only allocations, install the existing WB5 format, and reopen B independently.
The live workbook retains its Expected, ContentStateId, runtime IDs, lifetime
allocation ledger, history cursor, Redo and independent calculation captures.
Save creates no history transaction. Save B → Redo D → Undo B becomes clean again
only against verified saved-file authority and the exact saved ContentStateId.

`Document::export_historical_document` is an explicit, hidden Rust seam under
`history-experiment`; only the experimental association calls it. In that accepted internal snapshot, ordinary
`Document::export_document` and application/session Save used the original
full-ledger export contract. This is the initial internal correction authorized
by G1, **not ordinary Save promotion or desktop Undo/Redo integration**.

Core projection verifies exhaustive stored-record coverage, namespace/allocation
bounds and unique wire mappings. It materializes the actual historical definitions
and emits compact wire slots directly. It never lowers the materialized graph's
allocation floor, changes runtime IDs, synthesizes future-only nodes, looks up newer
D definitions, reparses/rebinds formulas, or writes a cold-history capsule as WB5.

The workbook directory supplies genuine live and retired durable identities,
sorted by durable node UUID. A separate verifier checks its bijection, liveness,
positions, exact runtime decimal parts, blank bitmap, scalar bound tokens,
aggregate overlays, selection identities/rectangles, placements and source/reference
provenance against the emitted core image. Unsupported structured/port/captured
capabilities remain rejected. Imported unreferenced retired slots may have no core
record; their authoritative workbook tombstones remain included. Future-only ledger
reservations have no current-root workbook record and are excluded from WB5.

An admitted lease covers owned projection/verification/encoding work, including
selection metadata, and stays alive through the common WB5 encoder. Cancellation
is checked before admission, during core materialization, per definition/cell and
before returning bytes. All work is read-only. Native installation, uncertain
installation fencing, immutable prepared tickets and receipt verification continue
through the existing file adapter and association; their safeguards are unchanged.
Standalone Open receives fresh runtime authority. Only verified original history
attachment restores the full original ledger.

## Tests

- New historical Save after Undo uses real native encode/install/Open, preserves
  Redo, verifies original B content and return-to-B cleanliness. Generic historical
  full-ledger export remains an explicit negative regression.
- Sparse discarded-branch ordinals, genuine retired reference versus D-only
  reservation, moved target, SUM/AVERAGE, Blank/zero, signed scaled zero and decimal
  scale, Unicode, cyclic/invalid/copied-rejected drafts, exact CellViews (including
  durable UUIDs, source and formula diagnostic/transaction provenance) round-trip
  through the unchanged WB5 reader.
- Cold historical projection runs both in-process and in an actual child process;
  the child restores cold authority, initiates native Save, installs and reopens
  WB5, checks authority invariants and exits after zero-charge cleanup. The parent
  opens that actual file and compares it with the original historical workbook.
- Core tests reject missing, duplicate and foreign target maps, reverse wire order
  without rewriting runtime IDs, preserve independent numeric captures and retain
  the old strict transfer failure. Workbook tests reject corrupt durable/runtime
  directories, missing/duplicate identities, missing placements/aggregate/range
  metadata, altered definitions and out-of-range slots.
- Stale Expected, exhausted admission, pre-cancellation, deterministic cancellation
  during materialization, cancellation after capture, and real codec rejection of
  an injected out-of-range slot preserve source authority and cleanly release work.
- Existing 69-case association matrix retains late B tickets across edits/Undo/Clear,
  actual native late-install uncertainty/reconciliation, receipt/cursor/Redo fencing,
  Save As, file replacement/copy, process termination and restart. Its original
  historical-export rejection case is now a new positive native Save regression;
  the generic-transfer rejection remains within that case and focused tests.

Mixed invalid graphs retain exact stored content and provenance. The existing
validation API reports a first core failure using runtime traversal order; after
independent Open, a cycle may be reported before another invalid draft. Mixed-draft
tests compare document diagnostics separately from that first-core-cycle report;
isolated cycle tests assert the complete diagnostic list. G1 does not redesign
validation ordering or claim that mixed-invalid first-error enumeration is stable.

## Validation and evidence

Evidence: `artifacts/history-save-g1/`. The input manifest freezes 168 Rust source/
manifest inputs; native Linux runs use a hash-verified source copy on `/var/tmp`,
with native test/data paths there. Shared Linux target cache and copied receipts
may reside on J:. Windows data is local NTFS. Final gate totals and matrix receipts
are recorded in `verification.json`; initial results remain separately preserved.

| Final gate | Windows | Native Linux | Native Mac ARM64/APFS |
| --- | --- | --- | --- |
| Default Rust tests | 346 | 347 | 347 |
| All-feature Rust tests | 389 | 390 | 390 |
| Format / strict all-target all-feature lint | Passed | Passed | Passed |
| Release-worker native matrix | 69 cases | 69 cases | 69 cases |
| Native original hashes (all copied matches) | 884 | 884 | 884 |
| Parent-killed fault workers / separate holder | 24 / 1 | 24 / 1 | 24 / 1 |

Final gates are under `final-gates/`; accepted-evidence candidate matrices are
`windows-results-reviewed/` and `linux-results-reviewed/`. Both workers were built
with `--release --features history-experiment`. `tested-binaries.json` fingerprints
them, `*-native-verification.json` verifies native originals against copied files,
and `verification.json` records the 168 final input checks and unchanged prior
Stage 2–4 manifest/receipt checksums. Prior payloads were not all rehashed anew.
The normally idle child-worker helper is included in the Rust test count; the
rich parent test invokes it separately and requires successful zero-charge exit.

The original Stage 4 failure and all original Stage 2–4 manifests remain intact.
They describe their original source, not this corrected implementation. The first
Windows debug-worker 69-case pass precedes final lease/codec-test refinements;
final release-worker matrices and source hashes are the review evidence. The first
Linux release matrix failed the mixed-invalid diagnostic-order assertion; its log
and original native files are retained. The final oracle also clears only the
unrelated parse draft in private probes and requires the exact cyclic-cell
diagnostic in both original and independently opened workbooks.

Development findings retained in the work record: an initial raw-decimal comparison
incorrectly compared submitted negative-zero parts with canonical runtime parts;
it was corrected to compare runtime parts while WB5 retains exact submitted parts.
A mixed-invalid diagnostic comparison exposed the traversal-order distinction
above. A child-process test initially shadowed a historical root with a filesystem
path and failed runtime shutdown; the test now releases the original root and
requires zero charge. A proposed oversized codec fixture was rejected by existing
workbook limits and removed without changing limits; codec failure is instead
injected into a private captured image and passed through the real encoder.
An initial node-slot-count fault was ineffective because the storage conversion
uses supplied durable-ID slots; the effective fault is an out-of-range live slot.
Initial failed logs in this artifact scope are retained; they are not final passes.

## Three-platform review and deferred promotion

[macOS/APFS evidence review](2026-10-24-macos-historical-save-g1-validation.md)
independently verifies 1,097 evidence hashes, 168 matching inputs, 470 frozen source
files plus the ignored GPT baseline, and 884 retained native hashes. The actual
historical Save case and zero-charge worker checks passed. Across all three
platforms: **207 cases and 2,652 original-file hashes**. Recommend acceptance of
the internal correction; the parent has accepted it under the linked review.

Mac report: `artifacts/macos-validation-20261024T100245Z/`; independent receipt:
`artifacts/history-save-g1-macos-review/verification.json`. Original G1 and Mac
manifests/receipts remain unchanged, including the G1 preparation receipt's
historical pending-Mac status. This review extends that evidence without treating
previous Mac runs as validation of the correction.

No ordinary Save promotion is included. Before that promotion, the corrected path
must pass the full applicable managed/session, desktop Save/Open and actual Native
AOT gates on Windows, native Linux and native macOS ARM64/APFS, with matched inputs
and zero-charge cleanup. Existing ordinary-path evidence alone cannot prove a
promotion of this new seam. No new manual checklist is required for this internal
Rust correction. Ordinary Save promotion is now authorized under the linked bounded assignment.
Return with its final evidence for G1 closure; G2 is not authorized.
