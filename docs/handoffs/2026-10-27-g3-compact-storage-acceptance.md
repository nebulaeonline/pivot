# G3 compact storage accepted; metadata and growth research authorized

Parent disposition, 2026-10-27: accept the compact-storage research result and
the under-25-MB objective as met. G3 remains active and incomplete. No production
backend, index layout or compression codec is selected.

## Accepted baseline

The same 10K mixed G2 stream occupies 18.3–18.4 MB with authentication, rather than
approximately 196 MB. All 10,000 deltas, 157 checkpoints including seed, four distinct
lifetime payloads and receipt identities remain retained. Strict binary records and
independently compressed periodic checkpoints are the preferred logical representation
for further research. The bucket directory is promising experimental evidence, not
a permanent extensible index.

Accept independent ordinary WB5 export/reopen sizes of 7,365 bytes at 1K and 7,367 at
10K for the same 42-cell fixture. This establishes separation from retained history,
not large-workbook sizing or history embedded in the ordinary workbook.

The broader metadata target remains unmet. Authentication-only data near 8% does not
satisfy the overall 20% target when receipt/index/authority metadata is counted.
Use **compressed retained payload bytes** as the primary denominator.

## Priority 1: page layout and required indexes

Compare the current WITHOUT ROWID blob/hash layout with conventional rowid tables
and indexed hash identity, suitable compact composite/integer-local keys preserving
stable external identities, and minimal receipt indexes supporting the same queries.
Do not omit semantic fields or weaken guarantees to obtain a smaller schema.

Use the same native 10K G2 workload/oracle. Separate payload bytes, table/index pages,
unused space, overflow pages, freelist, WAL and filesystem allocation. Distinguish
correctness metadata from avoidable representation/page overhead.

## Priority 2: authenticated growth

Propose and measure a bounded extensible directory: growth beyond 256 buckets,
oversized-bucket splitting/reorganization, maximum proof/decode/admission work,
atomic directory-generation transitions, old-reader consistency, stale-horizon
refusal and recovery without rebuilding from all history. Measure space/write
amplification as receipt count grows.

Splits must retain provability and classification of every receipt. Do not retain
all superseded application-level directory generations merely for SQLite reader
snapshots. The 64-KiB budget remains a research resource bound, not a user history
cap. If safe growth depends on unresolved G5 lifetime/retention policy, report that
dependency instead of inventing it.

## Priority 3: complete metadata costs and performance

Report commitments/buckets, receipt bodies, transaction/state-location indexes,
publication/lifetime metadata, row/page overhead and total non-payload space against
compressed retained payload. Preserve alternative denominators only as separate
observations, not replacements for the primary target.

Measure verified open, positive/definitive-negative lookup, publication/COMMIT p95,
splitting/bucket updates, WAL/checkpoints and full audit. Producer/full-oracle work
stays outside database timings. Report native allocation and process memory with
their limits; no cross-host ranking or device-write interpretation of write_bytes.

## Integrity and scope

Keep plain SQL as a measurement baseline, without bounded authenticated absence or
cheap safe unknown-ID admission claims. Retain exact-intent replay, terminal
negatives, authenticated receipt presence/absence, stale-reader refusal, integrity
quarantine and atomic head/cursor/Redo/lifetime publication. The existing SQLite
atomicity/collision/current-publication trust boundary and lack of independent
witness/coherent-rollback protection remain explicit.

Use Windows x64/NTFS, Ubuntu x64/ext4 and native Mac ARM64/APFS. Keep 1K/10K as the
controlled comparison. A narrowly bounded index-only growth probe beyond 10K is
permitted to exercise splitting, but is not 100K production qualification. No full
100K G2 workload until growth/admission design is sufficiently established.

Keep zlib level 6 for this slice. `J:\coding\dotZstd` and Rust zstd remain future
codec candidates; codec selection is not the next priority. Dev Drive remains shelved.

No second backend/custom storage engine, workbook-format change, production history
coordinator, witness, retention/GC, session/UI Undo/Redo, protocol change, identity
recycling or capacity increase. G1/G2 and six-RID execution validation remain closed;
clipboard ISSUE-0001 stays separate; G4–G7 remain gated. No GUI/AOT requalification.

Return a controlled compact page-layout comparison and an extensible authenticated-
index proposal with measured costs, integrity invariants and a credible growth path.
This reconciliation records acceptance and next authorization only; implementation
of the new slice has not started.
