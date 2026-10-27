# G3 first indexed-store feasibility result

Parent accepted this first research baseline on 2026-10-27. G3 remains incomplete;
bounded verified-open/completeness research is authorized next.
[Acceptance and next assignment](2026-10-27-g3-first-result-acceptance.md).

2026-10-27. First bounded research packet ready for architectural review.
**G3 is not complete and no production SQLite choice is requested.** The accepted
infrastructure is used as ordinary headless validation; no GUI suites were rerun.

## Implemented and tested

The feature-only Rust fixture emits real G2 deltas with stable record identities,
content provenance and an independent full-checkpoint oracle. The isolated Python
SQLite experiment commits payload references, receipt/intent identity, state links,
head/cursor/Redo and complete lifetime-ledger references atomically. No production
core dependency or UI/session/protocol integration changed.

Each target ran 1,000 actual transactions: 200 scalar edits, 200 formula changes,
200 external 32-cell rectangle pastes, 200 Fill seed edits and 200 downward Fill
operations over 32-cell rectangles. These are small mixed fixtures, not broad
spreadsheet-scale or formula-heavy feasibility evidence. Logical checkpoints occur
every 64 edits. Producer/oracle execution is measured separately from database work.

All three designated native targets passed:

| Target | Storage | SQLite | G2 regressions | Store tests | Actual transactions |
| --- | --- | --- | --- | --- | --- |
| Hyper-V Windows x64 | NTFS | 3.50.4 | 9/9 | 15/15 | 1,000 |
| Hyper-V Ubuntu x64 | ext4 | 3.46.1 | 9/9 | 15/15 | 1,000 |
| Native Mac ARM64 | APFS | Apple 3.54.0 | 9/9 | 15/15 | 1,000 |

Workspace format check, strict example Clippy and release fixture builds passed on
each target. All three Rust fixture processes finished with zero domain charge.
The reported producer/oracle peak domain charges were approximately 10.53 MB; this
does not include SQLite, Python, RSS or OS cache. Existing G1/G2 code is unchanged.

Store tests cover exact receipt retry and intent mismatch, verified/stale horizons,
terminal negative reconciliation blocking late edits, cold reconstruction, adjacent
cursor/Redo with unchanged lifetime authority, original-state/oracle rejection,
removed receipt, corrupt payload, required-index disagreement, precommit rollback,
lost ACK after commit, actual SQLite page exhaustion, delayed-reader WAL
backpressure, and actual child process termination after payload insertion,
before commit and after commit. Child termination is not power-loss testing.

## Measurements and interpretation

One run per platform, no hardware or cross-engine ranking is justified:

| Measurement | Windows x64 | Ubuntu x64 | Mac ARM64 |
| --- | ---: | ---: | ---: |
| COMMIT p50 / p95, ms | 3.422 / 5.035 | 4.047 / 4.966 | 0.135 / 0.198 |
| Complete publication p50 / p95, ms | 7.037 / 11.818 | 6.506 / 9.676 | 1.911 / 3.515 |
| Verified reopen audit, ms | 5,014.65 | 2,839.71 | 1,903.99 |
| Positive receipt lookup p50, microseconds | 220.6 | 141.2 | 24.0 |
| Terminal negative decision p50, ms | 3.294 | 4.268 | 0.089 |
| Sampled cold retrieval maximum, ms | 7.827 | 7.335 | 3.053 |

Complete publication includes Python decoding, original-state/oracle validation,
bounded cold reconstruction, SQL work and commit. COMMIT wall time includes the
backend's synchronization work but does not measure sync syscalls independently.
The ACK interval records local post-COMMIT receipt return, not transport/UI ACK.
Positive lookups reuse a verified connection; negative decisions include a durable
terminal control write. Cold timings sample only states 0, 500 and 1,000.

Receipt, state and child queries used indexes in all three EXPLAIN QUERY PLAN
reports. This is query-plan evidence, not measured logarithmic page/cache behavior.
The verified reopen audit scanned all 1,001 initial decisions and repeatedly
reconstructed content. **Bounded startup remains unmet in this prototype.** The
full scan is its conservative safety baseline, not a demonstrated SQLite limitation.

Stored payload JSON/hex bytes total 8,758,638 on every target. The SQLite files
were about 12.10 MB; the WAL files before close were about 59 MB, including
physical frames that a passive checkpoint need not truncate. Do not add these
figures and call them logical history size. The size figures were sampled at
different phases: payload/WAL before reopen probes, DB pages after control probes.
Supplementary `storage-metadata.json` records Linux/Mac dbstat page allocations
and filesystem block allocation. Windows dbstat/allocated-block reporting was
unavailable in this build. Index pages and payload pages remain distinguishable
in the available dbstat output, but complete physical/RSS/cache accounting is open.

Configuration: WAL, synchronous FULL (2), cache_size -2048, mmap_size 0,
wal_autocheckpoint 0, page size 4096. Engine source IDs/compile options are retained;
no single engine was pinned across platforms. Supplementary fresh connections
reported fullfsync=0 on all targets and checkpoint_fullfsync=1 only on Mac. The
writer did not override those settings. This is not F_FULLFSYNC instrumentation
or a power-loss durability claim. Actual VFS name remains unverified; platform
default VFS was used. See the [architecture notes](../architecture/history-indexed-store-experiment.md).

## Evidence and reproduction

Evidence: `artifacts/g3-index/g3-index-20261027-c/{win-x64,linux-x64,osx-arm64}/`.
Each includes `command.json`, original `console.log`, returned `evidence.zip`,
extracted step logs, full real `fixture.jsonl`, `result.json`, SQLite `summary.json`
and per-transaction `transactions.jsonl`, plus supplementary storage metadata.
Actual databases and native build/source roots remain on the corresponding target
under `~/PivotValidation/g3-index-20261027-c/<rid>/` (Windows uses User's home).
Results returned to shared storage; builds/databases remained native-local.

From a native-local checkout on a designated target:

```text
python scripts/research_history_index_run.py <new-native-output-directory> --transactions 1000
```

Use `python3` where appropriate. Output directories must be new. The runner copies
source locally, records the Git revision/dirty state, runs the focused gates and
keeps original logs. Larger transaction counts are supported by the tool but have
not been qualified in this packet. ROBPC was controller-only.

Development failures are retained under `artifacts/g3-index/`: the initial `a`
build used an incorrect Rectangle constructor; the `b` Ubuntu run lacked Cargo on
SSH PATH and Windows/Mac fixtures began with a no-op scalar edit. The corrected
constructor, explicit Cargo path and non-no-op seed passed in `c`. These are not
store acceptance failures and were not erased or counted as passes.

## Review boundary

Return this first substantive G3 result as directed. Atomic/indexed mechanics are
promising, but there is no complete feasibility or production-backend recommendation
yet. Prioritize a bounded verified-startup/completeness argument and explicit
engine/VFS/sync policy before interpreting 10K/100K scaling. Remaining reviewed G3
measurements include larger/formula-heavy workloads, precise disk/index and memory
accounting, recovery-tail distributions, reader leases, instance/epoch and branch
control validation, and additional faults. Do not replace these gaps with weaker
negative-lookup rules or silently engineer a custom second backend.

G1/G2 and six-RID execution infrastructure remain accepted/closed. Clipboard
reliability remains ISSUE-0001. G4–G7 and production Undo/Redo remain gated.
