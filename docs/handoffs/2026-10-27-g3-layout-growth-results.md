# G3 page-layout and index-growth result

2026-10-27. Research packet ready for review. **Identical logical history uses about
13.6 MB at 10K after switching payloads to a conventional rowid layout. Atomic
prefix splitting grows past 256 buckets without losing receipts.** The broader
20% metadata target and a permanent long-term index design remain unestablished.
No production backend/index/codec choice or full 100K G2 run is claimed.

## Controlled physical-layout comparison

Four new native databases were populated from the same read-only 10K snapshot,
with the same insertion order, fields and constraints. Payload, receipt, bucket
and authority SHA-256 manifests match exactly. Local physical rowids are excluded;
external hashes and stable identities are preserved. Every copy passes verified
open and complete audit, and every required lookup remains indexed.

| Bulk-copy database, decimal MB | Windows NTFS | Ubuntu ext4 | Mac APFS |
| --- | ---: | ---: | ---: |
| Existing WITHOUT ROWID payload | 17.641 | 17.621 | 17.629 |
| Conventional rowid + NOT NULL indexed hash | 15.081 | 15.135 | 15.118 |
| Integer-local key + unique external hash | 15.081 | 15.135 | 15.122 |
| Local key + partial non-NULL edit index | 15.081 | 15.135 | 15.122 |

The partial index preserves uniqueness of non-NULL edit locations and omits NULL
control entries; EXPLAIN still uses indexed edit lookup. It freed no pages in this
mostly authoring workload. No necessary receipt identity or query field was removed.
Integer-local keys provided no material size benefit over ordinary rowids here.

Ubuntu payload pages fell from 13,479,936 to 10,522,624 bytes, while a new hash index
uses 471,040 bytes. Payload overflow pages fell **2,159 → 0**; unused payload-page
space fell **5,070,513 → 2,082,825 bytes**. Mac observations are similar. Detailed
dbstat accounts for all allocated pages with zero residual. Windows lacks dbstat;
file/native allocation, logical manifests and indexed query plans remain measured.

These are density experiments, not production migration or incremental-write timings.
Their canonical bulk insertion order differs from normal replay, so they are not
substituted for replay results. The finding is consistent with SQLite's own
[row-size guidance](https://sqlite.org/withoutrowid.html), not a claim that rowid
tables are universally faster or smaller.

## Actual 1K/10K replay

The exact original G2 streams and independent oracles were replayed through the
existing layout, the rowid layout and the growing index. All logical manifests
match between fixed-index layouts. Growth payload/receipt manifests match them;
only bucket/publication representation differs. Reconstructed cold images match
byte-for-byte across all three cases at each size on every target.

| After-ingest database, MB; 1K / 10K | Windows | Ubuntu | Mac |
| --- | ---: | ---: | ---: |
| Existing payload layout / fixed index | 1.876 / 18.338 | 1.888 / 18.436 | 1.868 / 18.321 |
| Rowid payload / fixed index | 1.389 / 13.627 | 1.393 / 13.599 | 1.389 / 13.595 |
| Rowid payload / prefix growth | 1.397 / 13.640 | 1.397 / 13.611 | 1.384 / 13.619 |

All 10K authoritative deltas, 157 checkpoints including seed, four lifetime payloads
and every receipt remain retained. Zlib level 6 and complete semantic fields are
unchanged. No retention, GC, VACUUM or format migration produced these savings.
Ordinary WB5 and the previously validated 42-cell workbook baseline are unchanged.

## Growth argument and measured cost

[Design, work bounds and failure contract](../architecture/history-layout-growth-experiment.md).
The new directory starts with sixteen prefix regions. Overflow splits a region by
the next receipt-hash bit, partitions every old entry plus the new one, and commits
children, parent retirement, directory generation and publication authority in
one SQLite transaction. Complete prefix coverage is validated; missing children
are Unknown, not absence. Exact-intent receipts, terminal negatives, current state,
head/cursor/Redo and complete lifetime authority retain their original meanings.

There is one current application directory. Retained historical states resolve
through current authenticated receipts. Older SQLite reader transactions see both
their old directory and old parent rows through MVCC; actual same-bucket split
tests verify this and refuse stale reconciliation afterward. No application-level
directory-generation archive or speculative G5 retention policy is introduced.

At 10K the experiment made 230–236 splits and reached 246–252 current leaves. A
separate **16,384-control index-only probe** reached **376–377 leaves**, retaining
16,385 receipts including seed, and reopened with no historical scan. That probe
does not contain 16K G2 edits and is not a 100K production scalability qualification.
It verifies growth beyond the former fixed 256-bucket structure.

Research limits are explicit: 64 entries/bucket, 1,024 leaves, 24 prefix bits, eight
splits/publication and 64-KiB directory decode. An insert stages one admitted bucket
plus one entry; routing takes at most 25 prefix checks. Resource refusal rolls back,
never prunes receipts. These bounds are not a user-facing history cap or an unlimited
growth solution. The flat directory is still decoded and rewritten on publication.
At the index-only probe it reached about **13.9 KB**, with at most 4-KiB bucket data.
Before the leaf budget is reached, further research must address directory paging;
silently raising limits is not a production design.

| 10K observations, rowid fixed / growing | Windows | Ubuntu | Mac |
| --- | ---: | ---: | ---: |
| Backend publication p95, ms | 5.083 / 6.078 | 6.371 / 5.505 | 1.077 / 1.660 |
| COMMIT p95, ms | 4.540 / 4.544 | 6.090 / 4.644 | 0.941 / 0.939 |
| Verified activation median, ms | 3.784 / 6.159 | 2.362 / 3.644 | 2.025 / 3.805 |
| Positive receipt p50, microseconds | 422 / 1,367 | 229 / 832 | 85 / 748 |
| Definitive negative p50, ms | 3.497 / 4.928 | 3.525 / 4.588 | 0.670 / 1.746 |
| Full audit, seconds | 5.346 / 5.901 | 3.576 / 4.191 | 2.846 / 3.566 |

Growing-index split-publication p95 was 6.252/5.672/1.809 ms on Windows/Linux/Mac.
The index-only probe's p95 was 6.278/5.465/2.104 ms; verified reopening took
2.045/1.294/1.504 ms for its small seed content. These are single-run warm-cache
observations, not host rankings, isolated split causality or constant physical I/O.
The ordinary 1K head decodes 40 deltas and the 10K head 16, as in previous packets.

Growth has a measurable CPU cost: full flat-directory validation and bucket routing
make receipt lookup slower. Sync variation also matters; the slower Ubuntu rowid
COMMIT sample prevents a claim that better packing universally improves latency.
Raw split/non-split and SQL/proof/COMMIT/residual phases remain separate; producer/
full-oracle work is outside backend timings.

## Remaining metadata and resources

Compressed retained payload remains approximately **7.93 MB**, the primary denominator.
Total database space beyond compressed payload falls from **10.39–10.50 MB** to
**5.66–5.71 MB**. That improves overall overhead, but still amounts to roughly
71–72% of compressed payload, not 20%.

Separate receipt/index/bucket/authority/schema pages are about 58% on Linux/Mac,
up from the earlier 52% figure because the new payload-hash index is counted here.
That classification shift must not hide the reduction in total unused space.
Embedded payload hash/row overhead is additional. Authentication-only ratios do
not establish the overall target; raw binary payload is not substituted for the
compressed denominator. Windows per-object page attribution remains unavailable.

Supplemental final-phase queries, including control probes, find identical receipt
value bytes across candidates: IDs **160,528**, intents **321,056**, bodies
**1,687,396**, receipt hashes **321,056**, totaling **2,490,036 bytes** for 10,033
rows. Receipt bodies alone exceed 20% of the compressed payload before indexes.
The measured field set is preserved, so simply dropping an index cannot solve that
budget. Further representation-level normalization must retain canonical values,
intent/receipt identity and all integrity assertions; it needs explicit review.

Ubuntu 10K kernel write_bytes were about 635 MB old layout, 603 MB rowid and 481 MB
growing. These are own-process observations, not physical-device writes or a reliable
cross-run durability ranking. WAL sizes, checkpoint times, freelist, table/overflow
pages and filesystem allocation are retained separately. Native NTFS allocation
matches EOF; Linux/Mac block allocation is reported. Auto-vacuum is OFF; current
replay files retain two free pages, without truncation claims.

Ingest RSS was roughly 25–33 MB; observed final peak RSS roughly 28–35 MB. Traced
Python activation scratch was about 144–186 KB at 10K. Configured cache is not actual
native cache accounting, and no production domain-memory limit is established.
WAL/FULL and Mac fullfsync=ON remain. The older Windows/Linux SQLite qualification/
WAL-reset caveat and lack of VFS sync/physical-page instrumentation remain explicit.

## Verification, reproduction and recommendation

All three native targets passed the existing compact regression suite and ten
layout/growth tests: **147 passes, three explicit plain-mode guarantee skips**.
New tests cover exact logical equivalence, NULL/uniqueness/query preservation,
authenticated splits, old readers, partial split faults, leaf-budget refusal,
incomplete coverage, missing children, orphan-generation checks and actual process
termination before split, during reorganization, before COMMIT and after COMMIT.
Reopen observes a complete old/new generation; known integrity failures block mutation.

Evidence: `artifacts/g3-layout/g3-layout-20261027-a/<rid>/`, with original logs,
returned archives, physical-layout summaries/manifests/query plans, replay timing
JSONL, split-only distributions, page accounting and index-only stress results.
`comparison.json`, `cold-equivalence.json` and native allocation/metadata supplements
retain the review calculations. Native databases/source remain under each target's
`~/PivotValidation/g3-layout-20261027-a/<rid>/`; only evidence is returned to shared
storage. Source revision/dirty state and commands are recorded. No GUI/AOT gate reran.

For native-local reproduction use `research_history_layout_measure.py`: `physical`
with `--source` for canonical copies, `replay --layout without|rowid` for fixed-index
work, `replay --layout rowid --growth` for splitting, and `stress` for the bounded
index-only case. Every invocation requires a new output directory and an original
G2 fixture. The configured SSH controller is retained alongside the artifacts.

**Recommendation:** use ordinary rowid payload storage as the next research baseline;
there is no measured reason here to prefer explicit local IDs or the partial receipt
index. Keep both required receipt lookups. Review a page-bounded hierarchical directory
inside SQLite, updating affected authenticated pages plus a small root, before full
100K work. Current atomic splitting supports that direction, but the flat directory's
linear validation/rewrite cost is not the final structure. Do not restore permanent
per-publication trie copies or change integrity rules to improve measurements.

Review canonical receipt-body/reference normalization separately to address the
remaining metadata budget; do not omit correctness fields or mask it with a codec
change. Zlib 6 stays fixed; `J:\coding\dotZstd` and Rust zstd stay future candidates.
No production backend/index/codec, coordinator, witness, GC/retention, UI/protocol,
WB5 format or capacity change. G1/G2 and six-RID validation stay closed; G4–G7 remain
gated, clipboard ISSUE-0001 stays open, and Dev Drive work stays shelved.
