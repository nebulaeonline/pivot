# G3 hierarchical authentication and receipt normalization

2026-10-27. Final native research qualification passed; ready for parent review.
G3 remains open; no production integration or full 100K authorization. A subsequent
owner-authorized [dictionary comparison](2026-10-27-g3-dictionary-results.md) is separate
from this unchanged zlib baseline.

**Result:** bounded hierarchical updates and exact receipt restoration are demonstrated.
The best tested layout occupies 12.52–12.55 MB at 10K. Broader overhead falls to
57.8–58.2% of compressed payload, so the 20% objective remains unmet.

## Scope and comparison

The experiment compares four fresh representations against the same genuine native
1K/10K G2 fixtures: accepted rowid payloads with a growing flat directory; an
unnormalized mutable hierarchical directory; losslessly normalized receipts with
that hierarchy; and the normalized hierarchy with separate branch/leaf tables.
All retain binary G2 records, zlib 6, every receipt/delta/checkpoint, complete lifetime
authority, intent binding, terminal negatives, exact replay and historical auditability.

[Experimental contract and receipt-field inventory](../architecture/history-hierarchy-normalization-experiment.md) ·
[Parent assignment](2026-10-27-g3-layout-growth-acceptance.md).

The hierarchical candidates update existing node addresses in place inside the same
SQLite transaction as receipts and publication authority. SQLite MVCC preserves old
readers; the application retains no superseded paths/generations. Presence/absence
proofs follow a root-authenticated prefix path, without validating or rewriting an
entire flat directory. Full historical audit remains a separate explicit operation.

The first mixed-table candidate exposed an important physical regression: small branch
rows mixed with large leaves consumed many partially filled SQLite pages. The final
separate-table candidate changes physical placement only, keeping the same proof nodes,
canonical receipts, resource bounds and integrity rules. The regressing candidates
remain in the evidence; no VACUUM is used to shrink measured history databases.

## Retained space and metadata

After-ingest database bytes, decimal MB, with the same retained workload per target:

| 10K layout | Windows NTFS | Ubuntu ext4 | Mac APFS |
| --- | ---: | ---: | ---: |
| Fresh flat-directory baseline | 13.640 | 13.611 | 13.619 |
| Hierarchy, original receipts, mixed nodes | 14.578 | 14.574 | 14.590 |
| Hierarchy, normalized receipts, mixed nodes | 13.476 | 13.472 | 13.488 |
| Normalized hierarchy, separate branch/leaf tables | 12.546 | 12.517 | 12.517 |

At 1K the separate-table candidate occupies 1.278/1.290/1.282 MB versus flat
1.384/1.397/1.384 MB (Windows/Ubuntu/Mac). Compressed payload is unchanged at about
7.93 MB at 10K. The final layout saves roughly 1.09–1.10 MB versus the fresh flat
baseline, without deleting history or changing codec.

After the standard negative/replay/cursor probes, every target retains 10,033 receipts.
Exact physical body bytes drop from **1,687,396 to 620,669** (63.2%). Transaction-ID
bytes remain 160,528, intent bytes 321,056 and receipt-hash bytes 321,056. Canonical
restored receipt manifests, payload manifests and independent cold-image hashes match
across all four candidates within each target/fixture. Native fixtures contain different
identities; no cross-target byte-identical model claim is made.

The normalized explicit payload-key column costs another 10,161 logical bytes in the
Mac dbstat sample, without adding payload pages. This is included, not treated as free.
The mixed-node index consumes about 2.01 MB, including 1.34 MB unused space on Mac;
separate tables reduce it to about 1.04 MB (1.012 MB leaves plus 0.025 MB branches).
Authority falls from three pages to one; no superseded directory rows are retained.
Windows lacks dbstat, so its physical object breakdown is unavailable, not zero.

The final total database space beyond compressed payload is **4.59–4.61 MB**.
The Mac physical accounting illustrates the remaining cost: payload pages 8,994,816
versus 7,932,104 compressed bytes; payload-hash index 454,656; receipt pages 1,646,592;
receipt indexes 385,024; branch/leaf pages 1,036,288; authority/schema 8,192. This
counts necessary identity/integrity fields as well as avoidable page overhead. It
does not substitute authentication-only accounting for the broader target.

Native final file allocation is recorded after probes and can differ from the
after-ingest sizes above. Mac separate-table EOF is 12,525,568 and allocation
12,648,448 bytes; its flat EOF is 13,627,392 and allocation 13,697,024. Every target
reports its own native allocation; file length is not assumed to equal APFS allocation.

## Incremental work, startup and growth

Across the 10K genuine publications, hierarchical path depth peaks at eight. Logical
node writes are p50 eight, p95 nine, maximum ten per publication. Index bytes rewritten
fall from p95 about **11.6–11.7 KB to 4,284 bytes**, maximum 4,480. Total logical index
writes fall from about 71.5–71.6 MB to 32.8 MB. The complete flat directory is no
longer rewritten; these totals do not establish physical device-write savings.

Each local index-path proof peaks at 4,296 bytes. A publication reads its path twice
(admission and update), so per-publication p95 index bytes read are 8,334, versus flat
7,936. The benefit is bounded routing and less directory work, not fewer read bytes
in every sample. Separate-table lookup uses indexed integer addresses in two tables.

| 10K sample, flat → separate-table | Windows | Ubuntu | Mac |
| --- | ---: | ---: | ---: |
| Verified activation median, ms | 6.554 → 6.642 | 3.744 → 4.518 | 4.172 → 3.652 |
| Positive receipt median, µs | 1,290 → 733 | 817 → 415 | 875 → 233 |
| Terminal-negative median, ms | 4.691 → 4.028 | 4.986 → 3.773 | 4.438 → 0.877 |
| Publication p95, ms | 5.444 → 5.248 | 5.617 → 6.180 | 1.700 → 1.232 |
| COMMIT p95, ms | 3.990 → 4.343 | 4.781 → 5.716 | 0.987 → 0.891 |
| Complete audit, seconds | 6.031 → 6.617 | 4.122 → 4.603 | 3.593 → 3.877 |

Normalization adds reference/predecessor lookups. Verified startup and publication
do not uniformly improve; in particular the Linux publication/COMMIT samples regress.
The 1K head decodes 40 deltas, the 10K head 16. These warm samples do not imply that
larger histories activate faster or establish host/OS rankings or latency budgets.
Separate-table ingest RSS is about 25–33 MB and final process peaks about 28–35 MB;
activation Python peaks are about 148–156 KiB. Native cache ownership/high-water is
not measured. Pre-checkpoint WAL footprints are about 37–38 MB versus 11–17 MB for
the flat runs, illustrating that lower logical index bytes do not establish lower
physical writes. WAL reset thresholds/checkpoints make these footprints non-cumulative.
Existing Windows/Linux SQLite qualification caveats and lack of power-loss testing
remain; this research does not close those independent backend/runtime questions.

Both hierarchical candidates completed separate 16,384-control probes on each target.
They retain all **16,385 receipts**, reaching **427/421/422 leaves** and
853/841/843 current nodes (Windows/Ubuntu/Mac), exactly `2*leaves-1`. Path depth peaks
at nine, proof footprint at 4,361 bytes and publication node writes at eleven. There
is one current authority row and no retained superseded application-level paths.
Reopen performs no historical scan: separate-table samples are 1.219/0.879/0.722 ms.
Its control-only databases occupy 3.961/3.944/3.932 MB. These smaller seed-only
activation/database results are not comparable to a head requiring G2 reconstruction.

## Final qualification and recommendation

All three targets passed the original compact suite (39 passes, one explicit
unsupported-guarantee skip), existing layout/growth suite (10 passes), and final new
suite (34 passes): **249 passes and three explicit skips** total. The new suite covers
all three hierarchical representations, split exceptions, actual process death at
split/COMMIT boundaries, old snapshots, stale writers, missing/corrupt pages, invalid
coverage, admission refusal, canonical baseline equivalence, duplicate physical
addresses, changed payload references and local-reference stability through VACUUM.
The separate VACUUM unit test is not used to improve benchmark density.

Final tested implementation hashes match all three native snapshots and current source:
`tested-input-equivalence.json`. Later documentation and standalone dictionary-research
files do not alter those tested implementation inputs. Existing integrity assumptions
and unsupported guarantees remain explicit; no fault/assertion suppression was added.

**Recommendation:** the directory now demonstrates bounded incremental updates without
persistent-path accumulation, and receipt normalization is lossless within this
contract. Prefer separate branch/leaf tables for further experiments. Return for the
next G3 feasibility decision with the metadata target still open; production readiness
is not established. Any full 100K proposal must first address the admitted 1,024-leaf
research ceiling and measurement/resource budgets. Do not silently raise limits or
weaken authentication to obtain a larger pass.

## Interpretation boundaries

The primary denominator remains compressed authoritative payload, not raw binary
records. Receipt normalization reconstructs and hashes the exact original canonical
body; no authoritative field is silently omitted. Explicit integer primary keys are
local payload references only; external IDs and canonical content hashes remain intact.
Each normalized insertion rejects a body that cannot be restored exactly.

Logical node writes and authentication bytes are not physical SQLite pages or device
writes. The directory root envelope is bounded; publication work depends on path depth
and affected split nodes. SQLite B-tree page layout, WAL traffic, checkpoints and file
allocation are reported separately. Flat baseline node-count counters are unavailable
(the generic raw JSONL default is zero), not evidence of zero reads/writes.

The local proof footprint measures full branch/leaf bytes on the index path, excluding
the separately read authority and receipt. No wire protocol is proposed. Path-node
demand, configured 2 MiB cache, activation Python memory and process RSS are observable;
actual native SQLite cache-miss/ownership high-water counters are unavailable through
this Python binding. Own-process kernel writes include harness output and are not
device writes. Single-run warm samples establish no production latency budget.

Admission remains bounded at 60 entries/leaf, depth 24, 1,024 leaves and eight splits
per publication. Overflow refuses the entire transaction without pruning. The separate
16,384-terminal-negative-control workloads are index-only probes, not G2 edits or 100K
qualification. Untouched historical corruption is detected on access or full audit;
startup is not a claim of a fresh complete-history integrity scan. Existing trusted-root,
SQLite atomicity and hash-collision assumptions remain; no coherent rollback witness.

## Evidence and unchanged boundaries

Final-source evidence: `artifacts/g3-hierarchy/g3-hierarchy-20261027-b/<rid>/`.
Each native worker records filesystem, commands, revision/dirty state and tested Python
input hashes, plus logs, raw publication JSONL, reports and reconstructed cold images.
`comparison.json` is generated by `artifacts/g3-hierarchy/summarize.py`.
Native databases and frozen source remain under each target's
`~/PivotValidation/g3-hierarchy-20261027-b/<rid>/`; results return to shared storage.

The initial mixed-table comparison is preserved at `g3-hierarchy-20261027-a/` for its
own tested inputs. `artifacts/g3-hierarchy/launcher-notes.md` records the first Windows
command-length refusal and collection failure, redeployment refusal, and the successful
file-based launch. Those refusals are not successful workload evidence. Final-source
qualification uses short transferred worker files on all targets, with no overlapping
experiments on the same target. Only Mac ARM64 is used on the Mac host.

No codec change: `J:\coding\dotZstd` and Rust zstd remain future options. No production
SQLite integration, format migration, witness, retention/GC, coordinator, Undo/Redo,
protocol changes, identity recycling or product-limit increase. G1/G2 and six-RID
headless/GUI/AOT acceptance remain closed; G4–G7 stay gated. Dev Drive remains shelved
and clipboard ISSUE-0001 remains separate. No manual test is requested.
