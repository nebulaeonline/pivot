# G3 storage-efficiency reset

The authorized comparison is now implemented and ready for review; see
[byte accounting and compact results](2026-10-27-g3-storage-efficiency-results.md).
The assignment and original not-started receipt below retain historical context.

Parent direction, 2026-10-27: the approximately 196 MB database for **10,000
transactions**, not 10,000 spreadsheet rows, is rejected as a production direction.
The authenticated-index experiment remains accepted as a correctness demonstration.
G3 stays research-only and incomplete; storage efficiency is a primary criterion.

This assignment supersedes reclamation-first sequencing in the preceding
[verified-open acceptance](2026-10-27-g3-verified-open-acceptance.md). First measure
representation costs and a compact alternative; do not optimize/garbage-collect
the trie as though its production adoption were already justified.

## Research targets

| Measure | Initial target |
| --- | --- |
| Existing 10K mixed-transaction retained history | Under 25 MB, excluding separately measured current-workbook baseline |
| Authentication/index metadata | Under 20% of retained logical history bytes |
| Ordinary verified opening | No complete history scan |
| Small edit writes | Mutation-local records; avoid repeated full-state serialization |

These are demanding feasibility targets, not permanent product quotas. Report
failure against them honestly; realistic data may justify a reviewed revision.
Do not begin 100K work before addressing the 10K size problem.

## Workbook versus retained history

Treat current workbook state, retained Undo/Redo history and transaction indexes as
separate storage obligations. A normal .pivot workbook's size must not grow merely
with edit count or be dictated by the optional retained history. Measure its current
content/stable-identity baseline separately from historical checkpoints/deltas and
compact receipt/state-location indexes. This direction does not authorize a WB5
format change, packaging implementation or production retention policy.

## Required accounting

Account for actual compressed authoritative deltas, checkpoint bytes/duplication,
lifetime-ledger serialization, receipt/decision rows, primary/secondary index pages,
reachable/superseded trie nodes, fragmentation/freelist, DB/WAL and filesystem
allocation. Report bytes per transaction and per changed cell with explicit
denominators, distinguishing logical information from physical storage.

Existing Linux/Mac 10K dbstat observations attribute roughly 75 MB to proof nodes
and 109 MB to the payload table. Neither figure alone establishes what compressed
authoritative records require. JSON/hex interchange, before/after records,
checkpoints and lifetime payloads need independent byte accounting. The prototype
already hash-deduplicates identical payload blobs; do not assume the remaining
payload cost is repeated identical ledgers without measuring it.

## Controlled comparison

Compare the present design with a compact SQLite-only research baseline using
indexed immutable receipts, atomic authority/decision publication, binary resolved
records, compressed periodic logical checkpoints and indexed payload locations.
Use the same real G2 1K/10K streams and independent checkpoint oracle. JSON/hex is
acceptable fixture transport, not a production-size estimate.

The compact baseline is only a measurement candidate: ordinary transactional SQL
indexes do not prove that a missing historical receipt never existed. Preserve
that distinction. Investigate compact authenticated page/segment commitments as
a possible lower-cost proof option, without building another database engine or
custom append-only backend. No authentication structure is a preferred production
representation yet.

Preserve exact committed-intent replay, terminal negatives where justified,
head/cursor/Redo, monotonic lifetime authority and fail-closed ambiguity. A design
unable to distinguish a deleted committed receipt from never-committed must report
that limitation/Unknown rather than silently claim definitive Uncommitted.
No assertion weakening or silent integrity-contract change is allowed.

## Evidence and stop boundary

Use native Windows x64/NTFS, Ubuntu x64/ext4 and Mac ARM64/APFS. Record retained
logical bytes, DB/WAL/allocation, write amplification, open/lookup/reconstruction,
memory and sync-policy qualifications. Do not rank unequal machines/engines/sync
settings or call process write_bytes device writes. Preserve prior evidence and
the accepted failure-admission correction.

Return byte-level accounting, a controlled compact comparison and a recommendation
for substantially smaller history/index storage. No production backend, Undo/Redo,
witness, coordinator, retention controls, migration or workbook-format changes.
No six-RID GUI/AOT reruns. G1/G2 and execution validation remain closed; clipboard
ISSUE-0001 stays separate; G4–G7 stay gated. Dev Drive changes remain shelved.

This reconciliation records the revised direction only; no new experiment or
reclamation implementation has started.
