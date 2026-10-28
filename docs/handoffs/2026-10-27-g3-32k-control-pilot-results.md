# G3 32K control pilot: failed/stopped packet, no replay

2026-10-27. **The one-attempt authorization was consumed on all three targets.
This is not a passing three-platform milestone. Return for parent review.**
Windows and Ubuntu published all 32,768 controls, passed full native audit and
seed reconstruction, then failed the post-run read-only accounting guard. The
controller stopped Mac at 18,595 controls after the first peer failure. No attempt
was resumed, retried, reclassified or run with adjusted resource settings.

[Authorization](2026-10-27-g3-engine-acceptance-32k-authorization.md) ·
[Pilot contract and observer definitions](../architecture/history-32k-control-pilot.md).

## Original outcomes

| Target/native filesystem | Controls retained | Decisions including seed | Leaves | Original outcome |
| --- | ---: | ---: | ---: | --- |
| Windows x64 NTFS | 32,768 | 32,769 | 853 | Failed in final read-only accounting guard |
| Ubuntu x64 ext4 | 32,768 | 32,769 | 851 | Failed in final read-only accounting guard |
| Mac ARM64 APFS | 18,595 | 18,596 | 493 | Stopped on peer failure; no 32K result |

Preflight verified accepted engine/adapter/model/test source hashes, exact native
library hashes, pinned SQLite 3.51.3 identity, win32/unix VFS delegation, native
storage and ample free disk. Fresh file-backed connections confirmed file-control
VFS and the exact PRAGMAs. All **nine observer checks passed** (three per target)
before any pilot began. Those checks used tiny separate databases, not a pilot
retry. Native pilot source was snapshotted; SMB was used for returned evidence,
not live pilot databases. Only one Mac-hosted job ran; no GUI or accepted CI/AOT
gate ran.

The unchanged profile was **1,024 leaves / depth 24 / 60 entries / eight splits**,
zlib 6, 1 MiB staging, 2 MiB estimated native headroom, 64 MiB private SQLite heap,
64 MiB DB page quota, -2 MiB configured cache, mmap/automatic checkpoint off,
synchronous FULL, and both Mac fullfsync flags on. No legitimate admission refusal
occurred in the observed prefixes. No product code, codec, format or engine setting
was changed.

## What passed before failure

Every acknowledged control's exact ordinal, ID, intent and canonical receipt was
checked against an independent oracle. Seed identity, lifetime ledger, head and
cursor stayed unchanged. All three 16,384-control milestones exactly match their
accepted canonical receipt manifests. The deliberate stale-reader test passed
on every native target: snapshot absence remained visible and a stale writer
was refused before DML.

Windows and Ubuntu completed full native audit over all 32,769 decisions, verified
the unchanged cold seed, and produced complete canonical manifests and exact
dbstat page accounting. Native audit took 14.58 / 12.02 seconds respectively;
this is explicit full audit, not ordinary startup time.

Supplemental **offline analysis of checkpointed evidence copies**, using an
immutable read-only connection, independently restored all canonical receipts
and matched the oracle for each retained prefix, including Mac. An independent
tree traversal recomputed node hashes, checked key prefixes/order/capacity,
compared every scoped key/receipt pair and rejected unreachable/extra nodes.
Coverage exactly matches all receipts on each copy; integrity_check is OK.
This strengthens the preserved-prefix evidence; it does not replace an incomplete
native run or change any original outcome.

**Not reached:** the final native verified-reopen samples, SQL no-full-history-scan
check and authenticated present/absent lookup distributions. Mac also did not reach
its native final full audit, cold-reconstruction gate or explicit final checkpoint.
Normal connection close checkpointed its retained prefix; offline copy checks are
reported separately. No final three-platform startup/lookup scalability claim is made.

## Accounting guard failure and observer limitation

Both completed native runs stopped with:

`AssertionError: source changed during read-only accounting`

The accepted analyzer fingerprints the main file and any WAL before and after a
`mode=ro` SQLite connection. On a closed, checkpointed WAL-mode database, that
connection can create a new **zero-byte WAL** (and SHM), changing the file set.
On preserved copies this exactly reproduces the guard refusal while the main
database SHA-256 stays unchanged. Immutable read-only analysis of copies avoids
creating sidecars and passes the canonical/accounting checks. Immutable mode is
appropriate here only because the copies are fully checkpointed and have no writer;
it is not a proposed mode for live databases.

The original native guard did not persist its before/after fingerprints before
raising, so the copy reproduction is a diagnosis of that failure pattern, not a
claim that missing original fingerprints were captured. Original summaries and
archives remain untouched. The native attempts remain failed, with no bypass.

The new harness reused the accepted RSS helper once per publication on Windows.
That helper defines a ctypes structure each call; Python 3.13's pointer-type cache
retains it. A separate **database-free** native Windows diagnostic grew from
24,444,928 to 56,315,904 RSS bytes in 4,001 calls, while pointer-cache entries grew
from four to 4,004; garbage collection did not release it. This explains a major
observer contribution to the pilot's increasing Windows process RSS. It does not
prove that every byte of pilot growth has that cause, and measured RSS must not be
subtracted or relabeled as engine-only memory.

These are harness defects/coverage gaps: the observer self-check exercised only
three controls and did not exercise the closed-database final accounting lifecycle
or repeated Windows sampling. Neither defect has been silently corrected in the
frozen tested engine, accepted analyzer or completed attempts.

## Measured publication resources

The table covers only the acknowledged publications, not hypothetical Mac 32K.
MB here means decimal bytes / 1,000,000. Native heap is the private SQLite library;
RSS is the whole observer process. VFS bytes are writes observed at SQLite's VFS,
not device writes or proof of physical durability.

| Observation | Windows 32K | Ubuntu 32K | Mac retained 18,595 |
| --- | ---: | ---: | ---: |
| Checkpointed retained DB bytes | 7,860,224 | 7,868,416 | 4,616,192 |
| Native allocated DB bytes after close | 7,860,224 | 7,868,416 | 5,308,416 |
| SQLite heap high-water, publication phase | 2,266,312 | 2,266,232 | 2,634,944 |
| Sampled connection-cache maximum | 2,154,912 | 2,155,000 | 2,155,264 |
| SQL-step dirty-list peak | 26 | 27 | 27 |
| Logical index-node writes/publication, maximum | 12 | 12 | 11 |
| Staged index-node bytes, maximum | 4,556 | 4,556 | 4,491 |
| Depth reached | 10 | 10 | 9 |
| Publication WAL xWrite bytes | 1,260,765,896 | 1,253,605,336 | 661,408,608 |
| Publication WAL frame-header events | 306,011 | 304,273 | 160,536 |
| Publication WAL xSync calls | 32,804 | 32,804 | 18,613 |
| Maximum sampled WAL EOF bytes | 67,151,912 | 67,151,912 | 67,143,672 |
| Maximum publication-sampled process RSS | 294,404,096 | 37,052,416 | 25,608,192 |

The largest observed WAL EOF exceeds the 64 MiB trigger by 43,048 bytes on
Windows/Ubuntu and 34,808 on Mac, remaining below the 128 MiB recovery bound.
The trigger is still not a hard cap. Full native before/after telemetry, connection
cache hits/misses/writes/spills, checkpoint progress, DB/WAL/SHM EOF and allocation,
COMMIT times, per-publication IDs/intents/receipts and diagnostic staging samples
are preserved in compressed JSONL. Four Python-traced publications (three on Mac)
show roughly 1.06–1.07 MB traced peaks; they do not measure complete Python/native
transaction reservations. Traced publications are excluded from latency tables.

Final retained point files are about 16.8 / 16.7 / 9.4 MB respectively, below the
256 MiB point provision; periodic free-space checks retained the 512 MiB floor.
Sampled ceilings are not within-operation reservations. Actual device durability
and complete resource admission remain unestablished.

## Growth and comparison

| Same-build cumulative milestone | Windows leaves / outer p50 | Ubuntu leaves / outer p50 | Mac leaves / outer p50 |
| --- | --- | --- | --- |
| 10,000 controls | 255 / 3.111 ms | 254 / 4.985 ms | 253 / 9.023 ms |
| 16,384 controls | 427 / 3.158 ms | 421 / 4.980 ms | 422 / 9.011 ms |
| Final retained prefix | 853 / 3.410 ms | 851 / 5.019 ms | 493 / 9.039 ms |

Windows/Ubuntu final outer p95 is 5.702 / 6.548 ms, p99 6.983 / 7.767 ms;
COMMIT p50 is 1.663 / 3.742 ms. Mac's partial-prefix COMMIT p50 is 5.190 ms.
Outer timing includes native telemetry snapshots but excludes subsequent file/RSS
observation and artifact serialization. It is not end-to-end harness throughput.
All quantiles and maxima remain in comparison.json and the original raw records.

The older 16K control databases were 3,960,832 / 3,944,448 / 3,932,160 bytes with
427 / 421 / 422 leaves. Windows/Ubuntu 32K checkpoints are about twice that size,
with about twice as many leaves, while observed path depth grows to ten. Their
index/storage growth is encouraging, within the measured prefixes. The older
control engine/runtime and measurement harness differ; especially Mac's old
0.813 ms median versus the new roughly 9 ms must not be attributed to history
size alone. This packet does not isolate that engine/VFS/synchronization difference.

Accepted genuine 10K G2 databases were 12,546,048 / 12,517,376 / 12,517,376 bytes;
their publication WAL writes were roughly 440 MB and private SQLite heap peaks
2.27–2.63 MB. The new 10K control milestones write about 325 MB through WAL, and
completed 32K controls about 1.26 GB: roughly 38 KB/control over the full attempt.
The control workload retains only seed payload and many decisions, not 32K edits.
Its small database is not evidence that 100K genuine G2 payload fits. WAL write
amplification and sync cost remain operational concerns despite modest final size.

## Evidence and next review decision

Packet: [g3-pilot32k-20261027-a](../../artifacts/g3-pilot32k/g3-pilot32k-20261027-a/).
Each RID has an original evidence.zip, native paths, preflight/build/source hashes,
observer logs and untouched attempt summary, database and publications.jsonl.gz.
comparison.json is derived analysis. diagnostics-b contains explicitly supplemental
copy accounting/authentication diagnosis; rss-diagnosis.json is the separate native
Windows sampler reproduction. The first offline diagnosis utility had an argument
name error; its partial diagnostics directory is retained separately, not a pilot.

**Findings:** 32K publications fit the unchanged logical bounds on Windows/Ubuntu;
three-platform 32K feasibility is incomplete. Native startup/lookup behavior remains
unmeasured in this pilot. No admission ceiling was reached; no larger profile is
automatically justified. At 853/851 leaves, remaining capacity is finite and uneven
prefix growth still matters. A 65K run cannot fit the 61,440-entry aggregate ceiling.

**Recommended next bounded scope, requiring parent direction:** correct and test
observer lifecycle (stable Windows ctypes definitions; explicit closed-WAL/copy
accounting contract with persisted before/after evidence), then decide whether to
authorize a fresh pilot or only missing read-only checks on preserved databases.
Do not replay under the consumed authorization. Before any 4,096-leaf enablement,
qualify that runtime profile and physical resource behavior separately. Larger
genuine G2 runs need reviewed payload, DB/WAL/disk/heap budgets; the current 64 MiB
DB quota is not a 100K-G2 capacity promise. No codec research is proposed here.

G3 remains active. G1/G2 and six-RID gates remain accepted/closed; G4–G7, production
SQLite selection/integration, 65K/full100K, 4,096-leaf enablement, retention/GC,
product Undo/Redo and protocol/workbook changes remain gated. Dev Drive remains
shelved and clipboard ISSUE-0001 separate. Stop here for parent review.
