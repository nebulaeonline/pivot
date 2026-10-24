# ADR-0023 G1 — internal correction accepted; ordinary Save promotion authorized

Date: 2026-10-24. **Parent accepts the internal G1 historical WB5 projection and
its three-platform evidence. Bounded ordinary Save promotion is authorized as the
remaining G1 work. G1 closure is pending final validation; G2–G7 remain deferred.**
[Internal implementation](2026-10-24-historical-native-save-g1.md),
[Mac evidence review](2026-10-24-macos-historical-save-g1-validation.md),
[architecture](../architecture/history-production-readiness.md),
[ADR-0023](../adr/0023-document-transaction-history.md).

## Accepted internal correction

The deterministic, exhaustive current-content projection is accepted: genuine
current-root live and retired identities are serialized into temporary WB5 slots;
future-only reservations stay with the live/history ledger. No allocation floors,
runtime NodeIds, durable UUIDs, bindings or historical content are rewritten.
Source/provenance, Blank/zero, decimal representation, scalar/aggregate selections,
independent captures, native installation safeguards, cursor and Redo remain intact.
Generic full-ledger transfer stays strict; WB5-only policy and limits are unchanged.

The parent accepts the source-matched Windows/Linux/Mac Rust gates, release
matrices, native hashes and zero-charge evidence: 207 cases, 2,652 original-file
hashes; Rust default/all-feature counts 346/389, 347/390 and 347/390. Preserve the
original Stage 4 negatives, G1 manifests and all failures. The documented
mixed-invalid first-core-error traversal-order qualification is accepted; exact
historical content, binding and diagnostic provenance comparisons remain required.

## Authorized remaining G1 implementation

Make the validated projection the ordinary native WB5 export contract for supported
ordinary current roots and restored historical roots. Use one explicit path, not a
catch-and-retry fallback from failed full-ledger transfer. No new user-facing mode,
protocol operation, capability expansion, compatibility reader or WB5 format cut.
Preserve unsupported-capability rejection and existing workbook/codec limits.

Preparing and completing Save must preserve content/ContentStateId, live Expected,
session authority, original runtime/logical identities, the complete lifetime
ledger, cursor/Redo, independent captures, ticket ownership and uncertainty fences.
Save creates no document-history transaction. Verified completion changes only
saved-file/checkpoint authority, including when live content has moved away from
the prepared ticket. Save B → Redo D → Undo B cleanliness still uses exact content
identity plus verified file authority.

## Required final evidence

Run against the actual promoted exporter with frozen source/build inputs on local
Windows NTFS, native Linux and native macOS ARM64/APFS:

1. Full default/all-feature Rust regression, strict lint and formatting.
2. Managed/.NET, Rust session and IPC compatibility gates.
3. Actual Native AOT publish **and execute** validation.
4. Automated desktop native Save/Open and Save As workflows.
5. Nonhistorical Save equivalence and exact WB5 round-trip.
6. Historical Save, Redo preservation and exact saved-content cleanliness at the
   Rust boundary, without adding production history integration.
7. Cancelled, stale, rejected, failed and uncertain Save preservation.
8. Zero outstanding charge on completed normal and failure paths.
9. Source-matched manifests and verified native-origin Mac evidence.

History is not exposed by the ordinary application session yet. Use the accepted
internal research activation boundary to exercise historical content against the
promoted exporter directly; ordinary desktop tests establish nonhistorical Save
regressions. Do not add production Undo/Redo to manufacture a historical UI test,
or claim that the existing desktop suite validates such a workflow.

Return for review if promotion requires a broader session redesign, protocol or
WB5 change, weakened generic transfer, changed allocation authority, unsupported
capability expansion or a new history coordinator. Do not proceed to G2.

G1 closes only after ordinary supported native Save uses the validated projection
and all applicable three-platform managed/native/desktop/AOT gates pass, followed
by parent review. This receipt records acceptance and the next bounded assignment;
ordinary Save promotion has not been implemented by this documentation update.

Implementation follow-through: [ordinary Save promotion](2026-10-24-historical-save-g1-promotion.md).
This remains the original acceptance/authorization receipt, not final G1 closure.
