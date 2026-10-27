# G3 storage-efficiency comparison

Parent accepted this compact-storage research and the size objective on 2026-10-27.
G3 remains incomplete; page-layout, metadata and authenticated-growth research
are authorized next. [Acceptance and next scope](2026-10-27-g3-compact-storage-acceptance.md).

2026-10-27. **The existing 10K workload now occupies 17.3–18.4 MB, rather than
approximately 196 MB, without pruning history.** The authenticated candidate meets
the under-25-MB research target. Its authentication data is about 8% of compressed
payload, but broader receipt/index metadata still exceeds 20%. G3 remains research;
no production backend, index, compression codec or retention policy is selected.

## Byte accounting of the old representation

Representative preserved Ubuntu 10K database: **195,944,448 bytes**. All dbstat
pages, including payload, unused space, headers and ordinary indexes, reconcile
with zero unexplained page-accounting residual. Mac has the same detailed report;
Windows lacks dbstat and retains exact SQL value totals plus the measured physical
residual instead of guessed per-table pages.

| Old storage obligation | Measured bytes / interpretation |
| --- | --- |
| Payload table pages | 109,547,520; includes 21,071,446 unused bytes |
| Delta JSON/hex values | 81,895,726 |
| Checkpoint JSON/hex values | 5,666,945 |
| Distinct complete-ledger JSON/hex values | 18,140: only **four** payloads, referenced by 10,001 states |
| Proof-node pages | 75,284,480; includes 10,767,780 unused bytes |
| Proof key/value bytes | 61,730,064: 8,648,534 reachable, **53,081,530 superseded** |
| Proof-node counts | 285,120 total; 40,209 reachable; 244,911 superseded |
| Receipt decision pages | 6,881,280 |
| State pages | 2,576,384 |
| Ordinary secondary indexes | 1,642,496 |
| Authority, publication and schema pages | 12,288 |
| Freelist | 0 pages |

The JSON/hex value rows are inside the payload table; they must not be added to
its page total. The same applies to proof values versus proof pages. Both large
representations and page packing matter; ledger duplication is not the culprit.
This is why merely reclaiming trie paths would not solve the storage problem.

The original stream contains 178,092 coupled record operations and **84,000 actual
cell-record replacements**, not 10,000 spreadsheet rows. All authoritative
before/after data remains required. Binary delta accounting is approximately
42.03 MB before format headers and 7.36 MB compressed individually. The implemented
strict codec adds version/count headers; its exact retained delta total is about
42.09 MB binary / 7.38 MB compressed.

Across the 156 periodic checkpoints, 1,071,980 of 2,859,292 binary record bytes
repeat an earlier identical stable-key/value pair. Compressed checkpoints are only
about 0.55 MB, so they are not the dominant problem. Complete checkpoint and tag-
level accounting is retained; checkpoints were not removed to improve the result.

## Controlled compact comparison

Both candidates consume the **same previously produced G2 fixtures** as the trie
experiment, preserving its independent checkpoint oracle. All 10,000 deltas,
157 checkpoints including seed, four distinct complete ledgers and all receipt
identities remain retained. No GC, history pruning, VACUUM or retention cap was used.

| History database after ingest, decimal MB | Windows NTFS | Ubuntu ext4 | Native Mac APFS |
| --- | ---: | ---: | ---: |
| 1K compact SQL | 1.782 | 1.790 | 1.774 |
| 1K authenticated buckets | 1.876 | 1.888 | 1.868 |
| 10K compact SQL | 17.310 | 17.437 | 17.318 |
| 10K authenticated buckets | 18.338 | 18.436 | 18.321 |

These exclude the separately exported workbook. Subsequent query/control probes
add a few pages: final authenticated files remain below 18.45 MB. Original trie
figures include its own control probes; its pre-query footprint is about 195 MB,
so that phase difference does not explain the roughly order-of-magnitude reduction.

The compact SQL candidate stores binary records, compressed logical checkpoints
and indexed immutable receipts. It **does not** authenticate arbitrary SQL absence.
Such a request returns Unknown; explicit full audit under the writer lock is
required for definitive negatives and general new unknown-ID admission. Timed
ingestion uses an explicitly known-fresh generated stream, not a claim of cheap
safe general admission. At 10K, explicit audited negatives took 5.34/3.46/2.70 s
on Windows/Linux/Mac; those costs must not be hidden behind its fast Unknown result.

The second candidate authenticates receipts through 256 sorted hash buckets and
an 8-KiB current commitment directory, updated atomically with content references,
receipts, head/cursor/Redo and latest lifetime authority. Its global directory
contains every retained receipt; historical state locations resolve through those
immutable receipts. It needs no per-transaction retained trie path. Active SQLite
snapshots retain consistent older directory/bucket versions; tests force same-bucket
updates and verify old-reader proofs plus stale-negative refusal. No G5 lifecycle
policy is invented. Bucket overflow blocks rather than deleting receipts; the
64-KiB per-bucket research budget and fixed partitioning are not a product index
growth policy or a 100K scalability result.

## Targets and remaining overhead

At 10K, retained binary payload is **45,075,504 bytes**, compressed to about
**7.93 MB**: deltas ~7.38 MB, checkpoints ~0.549 MB, lifetime payloads ~4.7 KB.
Authentication entries plus directory total **638,255 bytes**, about 8.05% of
compressed payload. This meets an authentication-data-only interpretation of 20%.

The broader target is **not met**: separate receipt/index/bucket/authority/schema
objects occupy about 52% of compressed payload on Linux/Mac, before counting
embedded payload primary-key metadata. Including all non-compressed-payload DB
space is larger still, around 10.4 MB. Using the 45.08-MB raw binary denominator
would give a much friendlier ratio; it is reported separately, not substituted for
the stricter compressed denominator. Windows' separate metadata-page split is
unavailable; its complete file/allocation size is measured directly.

There is also substantial remaining payload-page slack: Ubuntu's compact payload
table uses 14,290,944 bytes for 7,931,326 compressed bytes plus hash/row structure.
The WITHOUT ROWID blob layout and ordinary receipt indexes deserve a controlled
page-packing comparison before adding more elaborate index machinery.

On Ubuntu, old DB storage was about 19,594 bytes/transaction or 2,333 bytes per
changed cell record. Compact authentication uses about **1,844 / 219 bytes**
respectively. Compressed deltas alone are approximately 88 bytes per actual cell-
record replacement. These denominators describe this small, repeatedly edited
42-cell fixture, not a large/formula-heavy workbook.

## Performance, resources and independent workbook baseline

Single-run observations; different engines/hosts/sync policies are not rankings:

| 10K authenticated candidate | Windows | Ubuntu | Mac |
| --- | ---: | ---: | ---: |
| Backend publication p95, ms | 4.801 | 6.138 | 1.099 |
| Binary preparation/compression p95, ms | 0.509 | 0.343 | 0.294 |
| Proof computation p95, ms | 0.080 | 0.064 | 0.046 |
| Non-COMMIT SQL API p95, ms | 0.263 | 0.166 | 0.074 |
| COMMIT p95, ms | 4.056 | 5.835 | 0.927 |
| Ordinary activation median, ms | 4.555 | 2.473 | 2.104 |
| Positive receipt p50, microseconds | 428.6 | 177.8 | 83.6 |
| Definitive terminal negative p50, ms | 3.553 | 3.881 | 0.804 |
| Streaming full audit, seconds | 5.614 | 3.735 | 2.891 |

Phase percentiles are not additive. SQL timers cover API execution; fetch overhead
and remaining Python work are in the residual. COMMIT includes synchronization but
does not isolate sync syscalls. Oracle work is outside backend timings. The ordinary
1K head decodes 40 deltas, the 10K head 16; faster 10K activation is not a favorable
history-size trend. Full audit now streams deltas rather than repeatedly rebuilding
every state from its checkpoint.

Ubuntu publication write_bytes were about **0.378 GB plain / 0.635 GB authenticated**
at 10K, versus about 4.4 GB in the earlier trie run. These are own-process kernel
accounting, not measured device writes. WAL sizes/checkpoints, freelist, dbstat and
allocated blocks are separate in raw reports. Supplemental native Windows allocation
queries match final EOF sizes. Auto-vacuum is OFF; bucket cases retain two free pages.
No physical truncation or secure-erasure claim is made.

Ingest RSS was roughly 20–33 MB and traced Python activation scratch 134–158 KB.
Configured SQLite cache remains approximately 2 MiB; actual native cache ownership,
OS cache and production domain admission remain unqualified. WAL/FULL is unchanged;
Mac fullfsync is explicitly ON. Old SQLite engine qualification/WAL-reset and VFS
sync caveats remain. No system software was replaced.

The new feature-only Rust baseline example cold-restores the actual reconstructed
state, exports through unchanged ordinary WB5 and reopens it, verifying all 42 cell
identities, contents and numeric evaluations. On **all three platforms**, the normal
workbook is **7,365 bytes at 1K and 7,367 bytes at 10K**. The two-byte difference is
current-content representation, not retained edit history. Both Rust processes per
platform finish with zero domain charge. No .pivot format or packaging change was made.

## Validation, evidence and recommendation

Final source runs 40 tests/platform: **117 passes, three explicit skips** overall.
Only the plain-mode bucket-corruption guarantee is skipped, because it does not
exist. Coverage includes codec null/empty/truncation, receipt replay/conflicts,
late edits after negatives, deleted historical terminal negatives, stale readers,
same-bucket MVCC proofs, lost ACK, corruption/quarantine, SQLite exhaustion and real
process death. No silent Unknown-to-Uncommitted conversion is accepted. Initial
native format, strict Clippy and release WB5-example build gates also passed.

Final evidence: `artifacts/g3-storage/g3-storage-20261027-b/<rid>/`, including returned
logs, exact prior accounting, mode/size summaries, per-transaction phase timings,
supplementary allocation facts and exported sample .pivot files. Earlier `a` results
remain development history; final workloads and tests use `b`, including the explicit
plain-mode fresh-stream admission boundary. Native databases/source remain under
each target's `~/PivotValidation/<run>/<rid>/`; results alone return to shared storage.
Offline checkpoint-duplication counts use original native-produced fixture bytes
without making controller performance claims.

[Design and reproduction worker](../architecture/history-compact-storage-experiment.md).
`research_history_storage_run.py` accepts a new native output directory and `--prior`
pointing to that target's preserved `g3-verified-20261027-a/<rid>` fixture/database
root. It preserves original data, runs gates, accounts bytes and compares both sizes.

**Recommendation:** retain binary authoritative deltas and compressed periodic
checkpoints as the promising storage direction. Do not adopt plain SQL as the bounded
reconciliation solution or the old persistent trie as the preferred index. Review
the compact bucket completeness argument, broader metadata denominator and payload/
receipt page packing next; preserve the integrity contract. No 100K run yet.

The owner supplied an existing .NET zstd wrapper at **`J:\coding\dotZstd`**; Rust
zstd is another option for a later controlled codec comparison. This slice used
standard-library zlib level 6 to isolate representation/index costs. **No production
compression choice is made**, and better compression alone would not solve receipt
metadata overhead relative to a compressed denominator.

G1/G2 and six-RID execution validation remain closed. Clipboard ISSUE-0001 stays
separate; Dev Drive work is shelved. No production backend/coordinator, Undo/Redo,
retention/compaction, witness, migration, UI/protocol or capacity changes. G4–G7 stay gated.
